<script lang="ts">
  import '../app.css';
  import { onMount } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import Icon from '#lib/Icon.svelte';
  import ProviderDiscovery from '#lib/ProviderDiscovery.svelte';
  import BackupRestore from '#lib/BackupRestore.svelte';
  import {
    api,
    apiResponse,
    ApiError,
    fqdn,
    setCsrf,
    type User,
    type Provider,
    type RecordItem,
    type History
  } from '#lib/api.ts';
  import { t, language, toggleLanguage, type Key } from '#lib/i18n.svelte.ts';

  let user = $state<User | null>(null);
  let loading = $state(true);
  let needsSetup = $state(false);
  let setupRestore = $state(false);
  let busy = $state(false);
  let error = $state('');
  let notice = $state('');
  let stale = $state(false);
  let providerItems = $state<Provider[]>([]);
  let recordItems = $state<RecordItem[]>([]);
  let historyItems = $state<History[]>([]);
  let total = $state(0);
  let metrics = $state({
    total_records: 0,
    verified: 0,
    attention: 0,
    zones: 0
  });
  let search = $state('');
  let filter = $state('all');
  let offset = $state(0);
  let historyOffset = $state(0);
  let roleItems = $state<{ id: string; name: string; permissions: string[] }[]>(
    []
  );
  let userItems = $state<
    { id: string; username: string; role_id: string; active: boolean }[]
  >([]);
  let permissionKeys = $state<string[]>([]);
  let auditItems = $state<
    { id: string; action: string; object_id: string; created_at: string }[]
  >([]);
  let modalKind = $state('');
  let editedProvider = $state<Provider | null>(null);
  let aaaaSource = $state<RecordItem | null>(null);
  let createMissingTarget = $state<RecordItem | null>(null);
  let deletion = $state<{
    kind: 'records' | 'providers';
    id: string;
    name: string;
  } | null>(null);
  let dialog: HTMLDialogElement;
  const nav = [
    'dashboard',
    'records',
    'providers',
    'history',
    'administration'
  ] as const;
  type View = (typeof nav)[number];
  let view = $derived(
    nav.includes(page.url.searchParams.get('view') as View)
      ? (page.url.searchParams.get('view') as View)
      : 'dashboard'
  );
  let selected = $derived(
    recordItems.find((r) => r.id === page.url.searchParams.get('panel'))
  );
  let visibleRecords = $derived(
    recordItems.filter(
      (r) =>
        (!search ||
          fqdn(r).toLowerCase().includes(search.toLowerCase()) ||
          r.provider_name.toLowerCase().includes(search.toLowerCase())) &&
        (filter === 'all' ||
          (filter === 'active' ? r.active : r.status === 'failed'))
    )
  );
  let recordProvider = $state('');
  const can = (permission: string) =>
    user?.permissions.includes(permission) ?? false;
  const dt = (value: string | null) =>
    value
      ? new Date(value).toLocaleString(
          language.value === 'de' ? 'de-DE' : 'en-GB',
          { dateStyle: 'short', timeStyle: 'short' }
        )
      : t('never');
  const statusLabel = (status: string) =>
    t(
      (['pending', 'updated', 'no_change', 'failed', 'skipped'].includes(status)
        ? status
        : 'unknown') as Key
    );
  const badgeClass = (status: string) =>
    status === 'failed'
      ? 'badge-bad'
      : ['updated', 'no_change'].includes(status)
        ? 'badge-ok'
        : 'badge-muted';

  function explain(err: unknown) {
    if (err instanceof ApiError)
      return t(
        ([
          'invalid_login',
          'rate_limited',
          'invalid_input',
          'backup_too_large',
          'backup_busy',
          'conflict',
          'forbidden'
        ].includes(err.code)
          ? err.code
          : 'requestFailed') as Key
      );
    return t('requestFailed');
  }
  async function refresh() {
    if (!user) return;
    try {
      const [records, providers, history, dashboard] = await Promise.all([
        can('records.read')
          ? api<{ items: RecordItem[]; total: number }>(
              `/records?offset=${offset}`
            )
          : Promise.resolve({ items: [], total: 0 }),
        can('providers.read')
          ? api<{ items: Provider[] }>('/providers')
          : Promise.resolve({ items: [] }),
        can('history.read')
          ? api<{ items: History[] }>(
              `/history?offset=${historyOffset}${page.url.searchParams.get('record') ? '&record_id=' + encodeURIComponent(page.url.searchParams.get('record')!) : ''}`
            )
          : Promise.resolve({ items: [] }),
        can('dashboard.read')
          ? api<typeof metrics>('/dashboard')
          : Promise.resolve({
              total_records: 0,
              verified: 0,
              attention: 0,
              zones: 0
            })
      ]);
      recordItems = records.items;
      total = records.total;
      providerItems = providers.items;
      historyItems = history.items;
      metrics = dashboard;
      stale = false;
    } catch (err) {
      if (err instanceof ApiError && err.status === 401) {
        user = null;
        setCsrf('');
      } else stale = true;
    }
  }
  $effect(() => {
    const recordId = page.url.searchParams.get('record');
    if (user && can('history.read')) {
      void api<{ items: History[] }>(
        `/history?offset=0${recordId ? '&record_id=' + encodeURIComponent(recordId) : ''}`
      )
        .then((data) => {
          historyItems = data.items;
        })
        .catch(() => {
          stale = true;
        });
    }
  });
  async function refreshAdmin() {
    if (!user?.admin) return;
    const [roles, users, audit] = await Promise.all([
      api<{ items: typeof roleItems; permissions: string[] }>('/roles'),
      api<{ items: typeof userItems }>('/users'),
      api<{ items: typeof auditItems }>('/audit')
    ]);
    roleItems = roles.items;
    permissionKeys = roles.permissions;
    userItems = users.items;
    auditItems = audit.items;
  }
  async function action(fn: () => Promise<unknown>, success: Key = 'saved') {
    busy = true;
    error = '';
    notice = '';
    try {
      await fn();
      notice = t(success);
      dialog?.close();
      modalKind = '';
      await refresh();
      if (user?.admin) await refreshAdmin();
    } catch (err) {
      error = explain(err);
    } finally {
      busy = false;
    }
  }
  function openModal(
    kind: string,
    provider: Provider | null = null,
    source: RecordItem | null = null
  ) {
    error = '';
    modalKind = kind;
    editedProvider = provider;
    aaaaSource = source;
    recordProvider = source?.provider_id ?? providerItems[0]?.id ?? '';
    dialog.showModal();
  }
  async function authenticate(event: SubmitEvent) {
    event.preventDefault();
    const form = new FormData(event.currentTarget as HTMLFormElement);
    busy = true;
    error = '';
    try {
      const input = {
        username: String(form.get('username')),
        password: String(form.get('password'))
      };
      if (needsSetup) {
        await api('/setup', 'POST', input, {
          'X-Setup-Token': String(form.get('setup_token'))
        });
        needsSetup = false;
      }
      await api('/auth/login', 'POST', input);
      user = await api<User>('/auth/me');
      setCsrf(user.csrf_token);
      await refresh();
      await refreshAdmin();
    } catch (err) {
      error = explain(err);
    } finally {
      busy = false;
    }
  }
  function backupRestored() {
    user = null;
    setCsrf('');
    needsSetup = false;
    setupRestore = false;
    recordItems = [];
    providerItems = [];
    historyItems = [];
    error = '';
    notice = t('backupRestored');
  }
  async function downloadBackup(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    const form = event.currentTarget as HTMLFormElement;
    const passphrase = new FormData(form).get('passphrase');
    await action(async () => {
      const response = await apiResponse('/backup', 'POST', { passphrase });
      const url = URL.createObjectURL(await response.blob());
      const link = document.createElement('a');
      link.href = url;
      link.download = `ddnsupd-${new Date().toISOString().slice(0, 10)}.ddnsbackup`;
      link.click();
      setTimeout(() => URL.revokeObjectURL(url), 1000);
      form.reset();
    }, 'backupDownloaded');
  }
  async function signOut() {
    await action(async () => {
      await api('/auth/logout', 'POST');
      user = null;
      setCsrf('');
      recordItems = [];
      providerItems = [];
      historyItems = [];
    }, 'logout');
  }
  async function submitModal(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    if (modalKind === 'createMissing' && createMissingTarget) {
      const { id } = createMissingTarget;
      await action(
        () => api(`/records/${id}/create-missing`, 'POST'),
        'createMissingQueued'
      );
      return;
    }
    if (modalKind === 'delete' && deletion) {
      const { kind, id } = deletion;
      await action(async () => {
        await api(`/${kind}/${id}`, 'DELETE');
        if (kind === 'records') await goto('/?view=records');
      });
      return;
    }
    const f = new FormData(event.currentTarget as HTMLFormElement);
    const value = (key: string) => String(f.get(key) ?? '');
    if (modalKind === 'provider')
      await action(() =>
        api('/providers', 'POST', {
          kind: value('kind'),
          name: value('name'),
          token: value('token')
        })
      );
    else if (modalKind === 'providerEdit' && editedProvider)
      await action(() =>
        api(`/providers/${editedProvider!.id}`, 'PATCH', {
          name: value('name'),
          enabled: f.has('enabled'),
          ...(value('token') ? { token: value('token') } : {})
        })
      );
    else if (modalKind === 'record')
      await action(() =>
        api('/records', 'POST', {
          provider_id: recordProvider,
          zone_remote_id: value('zone_remote_id'),
          zone_name: value('zone_name'),
          name: value('name'),
          record_type: value('record_type'),
          remote_record_id: value('remote_record_id'),
          manual_ip: value('manual_ip'),
          managed_ip: value('managed_ip'),
          interval_seconds: Number(value('interval_seconds')),
          site: value('site'),
          active: f.has('active')
        })
      );
    else if (modalKind === 'user')
      await action(() =>
        api('/users', 'POST', {
          username: value('username'),
          password: value('password'),
          role_id: value('role_id')
        })
      );
    else if (modalKind === 'role')
      await action(() =>
        api('/roles', 'POST', {
          name: value('name'),
          permissions: f.getAll('permissions')
        })
      );
  }
  async function editRecord(event: SubmitEvent) {
    event.preventDefault();
    if (!selected) return;
    const f = new FormData(event.currentTarget as HTMLFormElement);
    await action(() =>
      api(`/records/${selected!.id}`, 'PATCH', {
        active: f.has('active'),
        manual_ip: String(f.get('manual_ip')),
        interval_seconds: Number(f.get('interval_seconds')),
        site: String(f.get('site'))
      })
    );
  }
  function removeRecord() {
    if (!selected) return;
    deletion = { kind: 'records', id: selected.id, name: fqdn(selected) };
    openModal('delete');
  }
  function removeProvider(item: Provider) {
    deletion = { kind: 'providers', id: item.id, name: item.name };
    openModal('delete');
  }
  async function setPage(delta: number) {
    offset = Math.max(0, offset + delta * 100);
    await refresh();
  }
  async function setHistoryPage(delta: number) {
    historyOffset = Math.max(0, historyOffset + delta * 100);
    await refresh();
  }
  async function grantAccess(event: SubmitEvent, enabled: boolean) {
    event.preventDefault();
    const f = new FormData(event.currentTarget as HTMLFormElement);
    await action(() =>
      api('/grants', 'POST', {
        user_id: String(f.get('user_id')),
        [String(f.get('resource_type'))]: String(f.get('resource_id')),
        enabled
      })
    );
  }
  async function passwordChange(event: SubmitEvent) {
    event.preventDefault();
    const f = new FormData(event.currentTarget as HTMLFormElement);
    await action(async () => {
      await api('/auth/password', 'POST', {
        current: String(f.get('current')),
        password: String(f.get('password'))
      });
      user = null;
      setCsrf('');
    }, 'passwordChanged');
  }
  onMount(() => {
    try {
      language.value =
        localStorage.getItem('ddnsupd-language') === 'en' ? 'en' : 'de';
      document.documentElement.lang = language.value;
    } catch {
      /* Use default locale. */
    }
    void (async () => {
      try {
        const setup = await api<{ needs_setup: boolean }>('/setup');
        needsSetup = setup.needs_setup;
        if (!needsSetup) {
          try {
            user = await api<User>('/auth/me');
            setCsrf(user.csrf_token);
          } catch (err) {
            if (!(err instanceof ApiError && err.status === 401)) throw err;
          }
        }
        await refresh();
        await refreshAdmin();
      } catch (err) {
        error = explain(err);
      } finally {
        loading = false;
      }
    })();
    const timer = setInterval(() => {
      if (!document.hidden && !busy && !dialog?.open) void refresh();
    }, 10000);
    return () => clearInterval(timer);
  });
</script>

<svelte:head
  ><title>DDNSupd · {t(view)}</title><meta
    name="description"
    content="Self-hosted dynamic DNS management"
  /></svelte:head
>

{#snippet brand()}
  <a class="brand" href="/" aria-label="DDNSupd"
    ><span class="brand-mark"><Icon name="globe" size={23} /></span>
    <div>DDNS<span>upd</span></div></a
  >
{/snippet}
{#snippet emptyState(
  icon: string,
  title: Key,
  hint: Key,
  button?: Key,
  callback?: () => void
)}
  <div class="empty">
    <div class="empty-symbol"><Icon name={icon} size={28} /></div>
    <h2>{t(title)}</h2>
    <p>{t(hint)}</p>
    {#if button && callback}<button class="btn btn-primary" onclick={callback}
        ><Icon name="plus" size={15} />{t(button)}</button
      >{/if}
  </div>
{/snippet}
{#snippet historyTable(items: History[])}
  <div class="scroll-x">
    <table class="table">
      <thead
        ><tr
          ><th>{t('record')}</th><th>{t('result')}</th><th class="col-opt"
            >{t('newIp')}</th
          ><th>{t('time')}</th></tr
        ></thead
      ><tbody
        >{#each items as entry}<tr
            ><td
              ><span class="record-link mono">{entry.fqdn}</span><small
                >{entry.provider_name} · {t(
                  entry.trigger === 'create_missing'
                    ? 'createMissing'
                    : entry.trigger === 'manual'
                      ? 'manual'
                      : 'automatic'
                )}</small
              ></td
            ><td
              ><span class="badge {badgeClass(entry.outcome)}"
                >{statusLabel(entry.outcome)}</span
              >{#if entry.error_code}<div class="hint">
                  {entry.error_code === 'record_exists'
                    ? t('remoteRecordExists')
                    : entry.error_code}
                </div>{/if}</td
            ><td class="col-opt mono">{entry.new_ip || '—'}</td><td
              ><small>{dt(entry.created_at)}</small>
              <div class="hint">{entry.duration_ms} ms</div></td
            ></tr
          >{/each}</tbody
      >
    </table>
  </div>
{/snippet}

<header class="topbar">
  {@render brand()}
  <div class="top-actions">
    <button class="icon-btn" onclick={toggleLanguage} aria-label={t('language')}
      >{language.value.toUpperCase()}</button
    ><button
      class="icon-btn"
      aria-label={t('theme')}
      onclick={() => {
        const dark = document.documentElement.classList.toggle('dark');
        localStorage.setItem('ddnsupd-theme', dark ? 'dark' : 'light');
      }}><Icon name="sun" size={18} /></button
    >{#if user}<span class="hint user-name">{user.username}</span><button
        class="icon-btn"
        onclick={signOut}
        aria-label={t('logout')}
        disabled={busy}><Icon name="logout" size={18} /></button
      >{/if}
  </div>
</header>

{#if loading}
  <div class="auth-wrap">
    <div class="card auth-card" aria-busy="true">
      <p class="muted">{t('loading')}</p>
      <div class="skeleton-block">
        {#each [1, 2, 3] as i}<div
            class="skeleton"
            style={`width:${100 - i * 10}%`}
          ></div>{/each}
      </div>
    </div>
  </div>
{:else if !user}
  <div class="auth-wrap">
    <section class="card auth-card">
      {@render brand()}
      <h1>{t(needsSetup ? 'setup' : 'welcome')}</h1>
      <p class="auth-intro">{t(needsSetup ? 'setupIntro' : 'loginIntro')}</p>
      {#if error}<div class="error" role="alert">
          {error}
        </div>{/if}{#if notice}<div class="notice" role="status">
          {notice}
        </div>{/if}
      {#if needsSetup && setupRestore}
        <BackupRestore setup onrestored={backupRestored} />
      {:else}
        <form onsubmit={authenticate}>
          {#if needsSetup}<label class="field"
              >{t('setupToken')}<input
                class="input"
                name="setup_token"
                type="password"
                required
                autocomplete="off"
              /></label
            >{/if}<label class="field"
            >{t('username')}<input
              class="input"
              name="username"
              required
              minlength="3"
              maxlength="80"
              autocomplete="username"
            /></label
          ><label class="field"
            >{t('password')}<input
              class="input"
              name="password"
              type="password"
              required
              minlength={needsSetup ? 12 : 1}
              maxlength="256"
              autocomplete={needsSetup ? 'new-password' : 'current-password'}
            />{#if needsSetup}<span class="hint">{t('passwordHint')}</span
              >{/if}</label
          ><button class="btn btn-primary" disabled={busy}
            >{t(needsSetup ? 'createAdmin' : 'login')}<Icon
              name="arrow"
              size={16}
            /></button
          >
        </form>
      {/if}
      {#if needsSetup}
        <button
          class="btn"
          type="button"
          style="margin-top: 12px"
          onclick={() => {
            setupRestore = !setupRestore;
            error = '';
          }}
        >
          {t(setupRestore ? 'createAdmin' : 'backupRestore')}
        </button>
      {/if}
      <div class="auth-bottom">
        <Icon name="lock" size={13} />
        {t('selfHosted')}
      </div>
    </section>
  </div>
{:else}
  <aside class="sidebar">
    <div class="nav-caption">{t('management')}</div>
    {#each nav as item}{#if item !== 'providers' || can('providers.read')}<a
          href={`/?view=${item}`}
          class="nav-item"
          aria-current={view === item ? 'page' : undefined}
          ><Icon name={item} />{t(item)}</a
        >{/if}{/each}
    <div class="sidebar-note">
      <div><span class="status-dot"></span>DDNSupd</div>
      {t('subtitle')}
    </div>
  </aside>
  <main class="page-in">
    <div class="page-header">
      <div>
        <h1>{t(view)}</h1>
        <p>
          {t(
            view === 'dashboard'
              ? 'overviewIntro'
              : view === 'records'
                ? 'recordIntro'
                : view === 'providers'
                  ? 'providerIntro'
                  : view === 'history'
                    ? 'activityIntro'
                    : 'selfHosted'
          )}
        </p>
      </div>
      {#if view === 'records' && can('records.create')}<button
          class="btn btn-primary"
          onclick={() => openModal('record')}
          disabled={!providerItems.length}
          ><Icon name="plus" size={15} />{t('addRecord')}</button
        >{:else if view === 'providers' && user.admin}<button
          class="btn btn-primary"
          onclick={() => openModal('provider')}
          ><Icon name="plus" size={15} />{t('addProvider')}</button
        >{/if}
    </div>
    {#if error && !modalKind}<div class="error" role="alert">
        {error}
      </div>{/if}{#if notice}<div class="notice" role="status">
        {notice}
      </div>{/if}{#if stale}<div class="error" role="alert">
        {t('loadFailed')}
        <button class="btn" onclick={refresh}>{t('retry')}</button>
      </div>{/if}
    {#if view === 'dashboard'}
      <div class="stats">
        {#each [{ key: 'totalRecords', value: metrics.total_records, icon: 'records' }, { key: 'healthy', value: metrics.verified, icon: 'check' }, { key: 'attention', value: metrics.attention, icon: 'warning' }, { key: 'zones', value: metrics.zones, icon: 'globe' }] as stat}<div
            class="card stat"
          >
            <div class="stat-top">
              {t(stat.key as Key)}<span class="stat-icon"
                ><Icon name={stat.icon} size={17} /></span
              >
            </div>
            <div class="stat-value">{stat.value}</div>
            <div class="stat-foot">
              {total > 100
                ? `${Math.min(100, recordItems.length)} ${t('of')} ${total}`
                : 'DDNSupd'}
            </div>
          </div>{/each}
      </div>
      <div class="content-grid">
        <section class="card">
          <div class="card-heading">
            <h2>{t('recentActivity')}</h2>
            <a class="hint" href="/?view=history">{t('viewAll')} →</a>
          </div>
          {#if historyItems.length}{@render historyTable(
              historyItems.slice(0, 6)
            )}{:else}{@render emptyState(
              'history',
              'emptyHistory',
              'emptyHistoryHint'
            )}{/if}
        </section>
        <section class="card card-pad">
          <h2>{t('infrastructure')}</h2>
          <p class="hint">{t('connected')}</p>
          {#each providerItems as p}<div class="provider-row">
              <span class="provider-symbol"
                ><Icon name="providers" size={19} /></span
              >
              <div>
                <p>{p.name}</p>
                <span class="hint">{p.kind}</span>
              </div>
              <span
                class="badge {p.suspended ? 'badge-bad' : 'badge-muted'}"
                style="margin-left:auto"
                >{t(
                  p.suspended ? 'suspended' : p.enabled ? 'active' : 'paused'
                )}</span
              >
            </div>{:else}<p class="hint">{t('emptyProvidersHint')}</p>
            {#if user.admin}<button
                class="btn"
                onclick={() => openModal('provider')}
                ><Icon name="plus" size={15} />{t('addProvider')}</button
              >{/if}{/each}
        </section>
      </div>
      <p class="fineprint">{t('verification')} {t('propagation')}</p>
    {:else if view === 'records'}
      {#if can('records.read')}<div class="toolbar">
          <input
            class="input"
            bind:value={search}
            aria-label={t('search')}
            placeholder={t('search')}
          /><select
            class="input"
            bind:value={filter}
            aria-label={t('status')}
            style="max-width:160px"
            ><option value="all">{t('all')}</option><option value="active"
              >{t('active')}</option
            ><option value="failed">{t('errors')}</option></select
          >
        </div>
        <section class="card">
          {#if visibleRecords.length}<div class="scroll-x">
              <table class="table">
                <thead
                  ><tr
                    ><th>{t('record')}</th><th class="col-opt"
                      >{t('provider')}</th
                    ><th>{t('type')}</th><th class="col-mobile-hide"
                      >{t('address')}</th
                    ><th>{t('status')}</th><th class="col-opt"
                      >{t('lastCheck')}</th
                    ></tr
                  ></thead
                ><tbody
                  >{#each visibleRecords as record}<tr
                      ><td
                        ><a
                          class="record-link"
                          href={`/?view=records&panel=${record.id}`}
                          >{fqdn(record)}</a
                        ><small>{record.site || record.zone_name}</small></td
                      ><td class="col-opt">{record.provider_name}</td><td
                        ><span class="badge badge-muted"
                          >{record.record_type}</span
                        ></td
                      ><td class="mono col-mobile-hide"
                        >{record.managed_ip || '—'}</td
                      ><td
                        ><span
                          class="badge {record.active
                            ? badgeClass(record.status)
                            : 'badge-muted'}"
                          >{record.active
                            ? statusLabel(record.status)
                            : t('paused')}</span
                        ></td
                      ><td class="col-opt"
                        ><small>{dt(record.last_check)}</small></td
                      ></tr
                    >{/each}</tbody
                >
              </table>
            </div>{:else}{@render emptyState(
              'records',
              'emptyRecords',
              'emptyRecordsHint',
              can('records.create') && providerItems.length
                ? 'addRecord'
                : undefined,
              () => openModal('record')
            )}{/if}
        </section>
        <div class="pagination">
          <span>{visibleRecords.length} {t('of')} {total}</span>
          <div>
            <button
              class="btn"
              disabled={offset === 0}
              onclick={() => setPage(-1)}>{t('back')}</button
            >
            <button
              class="btn"
              disabled={offset + 100 >= total}
              onclick={() => setPage(1)}>{t('next')}</button
            >
          </div>
        </div>{:else}<p>{t('noPermission')}</p>{/if}
    {:else if view === 'providers'}
      <section class="card">
        {#if providerItems.length}<div class="scroll-x">
            <table class="table">
              <thead
                ><tr
                  ><th>{t('provider')}</th><th>{t('type')}</th><th
                    >{t('status')}</th
                  ><th></th></tr
                ></thead
              ><tbody
                >{#each providerItems as p}<tr
                    ><td
                      ><strong>{p.name}</strong>
                      <div class="hint mono">{p.id}</div></td
                    ><td>{p.kind}</td><td
                      ><span
                        class="badge {p.suspended
                          ? 'badge-bad'
                          : 'badge-muted'}"
                        >{t(
                          p.suspended
                            ? 'suspended'
                            : p.enabled
                              ? 'active'
                              : 'paused'
                        )}</span
                      ></td
                    ><td
                      >{#if user.admin}<button
                          class="btn"
                          onclick={() => openModal('providerEdit', p)}
                          >{t('edit')}</button
                        >
                        <button
                          class="btn btn-danger"
                          onclick={() => removeProvider(p)}
                          disabled={busy}>{t('deleteProvider')}</button
                        >{/if}</td
                    ></tr
                  >{/each}</tbody
              >
            </table>
          </div>{:else}{@render emptyState(
            'providers',
            'emptyProviders',
            'emptyProvidersHint',
            user.admin ? 'addProvider' : undefined,
            () => openModal('provider')
          )}{/if}
      </section>
      <p class="fineprint">{t('verification')}</p>
      <ProviderDiscovery
        providers={providerItems}
        canBind={can('records.create')}
        canImport={user.admin}
        onSaved={refresh}
      />
    {:else if view === 'history'}
      {#if can('history.read')}<section class="card">
          {#if historyItems.length}{@render historyTable(
              historyItems
            )}{:else}{@render emptyState(
              'history',
              'emptyHistory',
              'emptyHistoryHint'
            )}{/if}
        </section>
        <div class="pagination">
          <button
            class="btn"
            disabled={historyOffset === 0}
            onclick={() => setHistoryPage(-1)}>{t('back')}</button
          ><button
            class="btn"
            disabled={historyItems.length < 100}
            onclick={() => setHistoryPage(1)}>{t('next')}</button
          >
        </div>
        <p class="fineprint">{t('propagation')}</p>{:else}<p>
          {t('noPermission')}
        </p>{/if}
    {:else if view === 'administration'}
      <div class="admin-grid">
        {#if user.admin && can('backup.create')}
          <section class="card card-pad">
            <h2>{t('backup')}</h2>
            <p class="hint">{t('backupExportHint')}</p>
            <form onsubmit={downloadBackup}>
              <label class="field">
                {t('backupPassphrase')}
                <input
                  class="input"
                  name="passphrase"
                  type="password"
                  minlength="12"
                  maxlength="256"
                  required
                  autocomplete="new-password"
                />
              </label>
              <button class="btn btn-primary" disabled={busy}
                >{t('backupDownload')}</button
              >
            </form>
          </section>
        {/if}
        {#if user.admin && can('backup.restore')}
          <section class="card card-pad">
            <h2>{t('backupRestore')}</h2>
            <BackupRestore onrestored={backupRestored} />
          </section>
        {/if}
        {#if user.admin}<section class="card card-pad">
            <div class="dialog-head">
              <h2>{t('administration')}</h2>
              <button class="btn" onclick={() => openModal('user')}
                >{t('addUser')}</button
              >
            </div>
            {#each userItems as u}<div class="provider-row">
                <div>
                  <p>{u.username}</p>
                  <span class="hint mono">{u.id}</span>
                </div>
                <span class="badge badge-muted" style="margin-left:auto"
                  >{roleItems.find((r) => r.id === u.role_id)?.name}</span
                >
              </div>{/each}
          </section>
          <section class="card card-pad">
            <div class="dialog-head">
              <h2>{t('userRole')}</h2>
              <button class="btn" onclick={() => openModal('role')}
                >{t('addRole')}</button
              >
            </div>
            {#each roleItems as role}<div class="provider-row">
                <div>
                  <p>{role.name}</p>
                  <div class="hint">{role.permissions.join(', ')}</div>
                </div>
              </div>{/each}
          </section>
          <section class="card card-pad">
            <h2>{t('grant')}</h2>
            <p class="hint">{t('grantHint')}</p>
            <form onsubmit={(e) => grantAccess(e, true)}>
              <label class="field"
                >{t('userId')}<select class="input" name="user_id" required
                  >{#each userItems as u}<option value={u.id}
                      >{u.username}</option
                    >{/each}</select
                ></label
              ><label class="field"
                >{t('resourceType')}<select class="input" name="resource_type"
                  ><option value="provider_id">{t('provider')}</option><option
                    value="record_id">{t('record')}</option
                  ></select
                ></label
              ><label class="field"
                >{t('resourceId')}<input
                  class="input"
                  name="resource_id"
                  required
                /></label
              ><button class="btn btn-primary" disabled={busy}
                >{t('grantSave')}</button
              >
              <button
                class="btn"
                type="button"
                disabled={busy}
                onclick={(e) => {
                  const form = e.currentTarget.closest('form');
                  if (form && form.reportValidity())
                    void grantAccess(
                      {
                        preventDefault() {},
                        currentTarget: form
                      } as unknown as SubmitEvent,
                      false
                    );
                }}>{t('grantRevoke')}</button
              >
            </form>
          </section>{/if}
        <section class="card card-pad">
          <h2>{t('passwordChange')}</h2>
          <form onsubmit={passwordChange}>
            <label class="field"
              >{t('currentPassword')}<input
                class="input"
                name="current"
                type="password"
                required
                maxlength="256"
                autocomplete="current-password"
              /></label
            ><label class="field"
              >{t('newPassword')}<input
                class="input"
                name="password"
                type="password"
                required
                minlength="12"
                maxlength="256"
                autocomplete="new-password"
              /></label
            ><button class="btn btn-primary" disabled={busy}>{t('save')}</button
            >
          </form>
        </section>
      </div>
      {#if user.admin}<section class="card" style="margin-top:22px">
          <div class="card-heading"><h2>{t('audit')}</h2></div>
          <div class="scroll-x">
            <table class="table">
              <thead
                ><tr
                  ><th>{t('time')}</th><th>{t('operation')}</th><th
                    class="col-opt">{t('object')}</th
                  ></tr
                ></thead
              ><tbody
                >{#each auditItems as entry}<tr
                    ><td>{dt(entry.created_at)}</td><td>{entry.action}</td><td
                      class="mono col-opt">{entry.object_id || '—'}</td
                    ></tr
                  >{/each}</tbody
              >
            </table>
          </div>
        </section>{/if}
    {/if}
  </main>
  {#if selected}
    <aside class="panel" aria-label={t('details')}>
      <div class="panel-title">
        <div>
          <div class="hint" style="margin-bottom:6px">{t('details')}</div>
          <h2>{fqdn(selected)}</h2>
          <span class="badge {badgeClass(selected.status)}"
            >{statusLabel(selected.status)}</span
          >
        </div>
        <a class="icon-btn" href="/?view=records" aria-label={t('close')}
          ><Icon name="close" /></a
        >
      </div>
      <dl>
        <dt>{t('provider')}</dt>
        <dd>{selected.provider_name}</dd>
        <dt>{t('type')}</dt>
        <dd>{selected.record_type}</dd>
        <dt>{t('address')}</dt>
        <dd class="mono">{selected.managed_ip || '—'}</dd>
        <dt>{t('lastCheck')}</dt>
        <dd>{dt(selected.last_check)}</dd>
        <dt>{t('nextCheck')}</dt>
        <dd>{dt(selected.next_check)}</dd>
        <dt>{t('source')}</dt>
        <dd>{selected.manual_ip || t('autoSource')}</dd>
        <dt>{t('object')}</dt>
        <dd class="mono">{selected.id}</dd>
      </dl>
      {#if selected.provider_suspended}<div class="error">
          {t('credentialsSuspended')}
        </div>{/if}
      {#if can('records.edit')}{#key selected.id}<form onsubmit={editRecord}>
            <label class="checkbox"
              ><input
                type="checkbox"
                name="active"
                checked={selected.active}
              />{t('enabled')}</label
            ><label class="field" style="margin-top:17px"
              >{t('manualIp')}<input
                class="input"
                name="manual_ip"
                value={selected.manual_ip}
              /></label
            ><label class="field"
              >{t('interval')}<input
                class="input"
                type="number"
                min="60"
                max="86400"
                name="interval_seconds"
                value={selected.interval_seconds}
                required
              /></label
            ><label class="field"
              >{t('site')}<input
                class="input"
                name="site"
                value={selected.site}
                maxlength="120"
              /></label
            ><button class="btn btn-primary" disabled={busy}>{t('save')}</button
            >
          </form>{/key}{/if}
      {#if selected.record_type === 'A' && can('records.create') && providerItems.some((provider) => provider.id === selected?.provider_id)}
        <button
          class="btn"
          disabled={busy}
          onclick={() => openModal('record', null, selected)}
          ><Icon name="plus" size={15} />{t('addAaaaRecord')}</button
        >
      {/if}
      {#if selected.provider_kind === 'hetznercloud' && selected.active && can('records.create') && can('updates.trigger')}
        <button
          class="btn"
          disabled={busy || selected.provider_suspended}
          onclick={() => {
            createMissingTarget = selected ?? null;
            openModal('createMissing');
          }}><Icon name="plus" size={15} />{t('createMissing')}</button
        >
      {/if}
      {#if can('updates.trigger')}<button
          class="btn"
          disabled={busy || !selected.active}
          onclick={() =>
            action(
              () => api(`/records/${selected!.id}/update`, 'POST'),
              'queued'
            )}><Icon name="refresh" size={15} />{t('update')}</button
        >{/if}{#if can('records.delete')}<button
          class="btn btn-danger"
          onclick={removeRecord}
          disabled={busy}>{t('remove')}</button
        >{/if}
      <a class="btn" href={`/?view=history&record=${selected.id}`}
        >{t('historyFor')}</a
      >
      <p class="fineprint">{t('propagation')}</p>
    </aside>
  {/if}
{/if}

<dialog
  bind:this={dialog}
  aria-labelledby="dialog-title"
  oncancel={(event) => {
    if (busy) event.preventDefault();
  }}
  onclose={() => {
    modalKind = '';
    aaaaSource = null;
    createMissingTarget = null;
    deletion = null;
    error = '';
  }}
>
  <div class="dialog-head">
    <h2 id="dialog-title">
      {t(
        modalKind === 'record'
          ? aaaaSource
            ? 'addAaaaRecord'
            : 'addRecord'
          : modalKind === 'user'
            ? 'addUser'
            : modalKind === 'role'
              ? 'addRole'
              : modalKind === 'providerEdit'
                ? 'edit'
                : modalKind === 'delete'
                  ? deletion?.kind === 'records'
                    ? 'remove'
                    : 'deleteProvider'
                  : modalKind === 'createMissing'
                    ? 'createMissing'
                    : 'addProvider'
      )}
    </h2>
    <button
      class="icon-btn"
      type="button"
      disabled={busy}
      aria-label={t('close')}
      onclick={() => dialog.close()}><Icon name="close" /></button
    >
  </div>
  {#if error}<div class="error" role="alert">{error}</div>{/if}
  <form onsubmit={submitModal}>
    {#if modalKind === 'delete' && deletion}
      <p><strong>{deletion.name}</strong></p>
      <p class="hint">
        {t(deletion.kind === 'records' ? 'confirmDelete' : 'confirmProvider')}
      </p>
    {:else if modalKind === 'createMissing' && createMissingTarget}
      <p>
        <strong
          >{fqdn(createMissingTarget)} · {createMissingTarget.record_type}</strong
        >
      </p>
      <p class="hint">{t('createMissingHint')}</p>
      <p class="hint">
        {t('source')} · {createMissingTarget.manual_ip || t('autoSource')}
      </p>
    {:else if modalKind === 'provider' || modalKind === 'providerEdit'}
      {#if modalKind === 'provider'}<label class="field"
          >{t('provider')}<select class="input" name="kind"
            ><option value="hetznercloud">Hetzner Cloud DNS</option><option
              value="cloudflare">Cloudflare</option
            ></select
          ></label
        >{/if}<label class="field"
        >{t('name')}<input
          class="input"
          name="name"
          value={editedProvider?.name ?? ''}
          required
          maxlength="120"
        /></label
      ><label class="field"
        >{t(
          modalKind === 'providerEdit' ? 'credentialsReplace' : 'token'
        )}<input
          class="input"
          name="token"
          type="password"
          required={modalKind === 'provider'}
          maxlength="8192"
          autocomplete="off"
        /><span class="hint">{t('tokenHint')}</span></label
      >{#if modalKind === 'providerEdit'}<label class="checkbox"
          ><input
            name="enabled"
            type="checkbox"
            checked={editedProvider?.enabled}
          />{t('providerEnabled')}</label
        >{/if}
      <p class="fineprint">{t('verification')}</p>
    {:else if modalKind === 'record'}
      <p class="hint">{t('bindHint')}</p>
      <label class="field"
        >{t('provider')}<select
          class="input"
          name="provider_id"
          bind:value={recordProvider}
          required
          >{#each providerItems as p}<option value={p.id}>{p.name}</option
            >{/each}</select
        ></label
      >
      <div class="field-row">
        <label class="field"
          >{t('zoneName')}<input
            class="input"
            name="zone_name"
            value={aaaaSource?.zone_name ?? ''}
            placeholder="example.org"
            required
          /></label
        ><label class="field"
          >{t('zoneId')}<input
            class="input"
            name="zone_remote_id"
            value={aaaaSource?.zone_remote_id ?? ''}
            required
          /></label
        >
      </div>
      <div class="field-row">
        <label class="field"
          >{t('recordName')}<input
            class="input"
            name="name"
            placeholder="home"
            value={aaaaSource?.name ?? ''}
            required
          /></label
        ><label class="field"
          >{t('type')}<select
            class="input"
            name="record_type"
            value={aaaaSource ? 'AAAA' : 'A'}
            ><option>A</option><option>AAAA</option></select
          ></label
        >
      </div>
      {#if providerItems.find((p) => p.id === recordProvider)?.kind === 'cloudflare'}<label
          class="field"
          >{t('remoteRecordId')}<input
            class="input"
            name="remote_record_id"
            required
          /></label
        >{/if}<label class="field"
        >{t('manualIp')}<input class="input" name="manual_ip" /></label
      ><label class="field"
        >{t('managedIp')}<input class="input" name="managed_ip" /></label
      >
      <div class="field-row">
        <label class="field"
          >{t('interval')}<input
            class="input"
            type="number"
            name="interval_seconds"
            value={aaaaSource?.interval_seconds ?? 300}
            min="60"
            max="86400"
            required
          /></label
        ><label class="field"
          >{t('site')}<input
            class="input"
            name="site"
            value={aaaaSource?.site ?? ''}
            maxlength="120"
          /></label
        >
      </div>
      <label class="checkbox"
        ><input type="checkbox" name="active" />{t('enabled')}</label
      >
    {:else if modalKind === 'user'}
      <label class="field"
        >{t('username')}<input
          class="input"
          name="username"
          required
          minlength="3"
          maxlength="80"
          autocomplete="off"
        /></label
      ><label class="field"
        >{t('password')}<input
          class="input"
          name="password"
          type="password"
          required
          minlength="12"
          maxlength="256"
          autocomplete="new-password"
        /></label
      ><label class="field"
        >{t('userRole')}<select class="input" name="role_id" required
          >{#each roleItems as role}<option value={role.id}>{role.name}</option
            >{/each}</select
        ></label
      >
    {:else if modalKind === 'role'}
      <label class="field"
        >{t('name')}<input
          class="input"
          name="name"
          required
          maxlength="80"
        /></label
      >
      <p class="hint">{t('permissions')}</p>
      <div class="checklist">
        {#each permissionKeys as permission}<label class="checkbox"
            ><input
              type="checkbox"
              name="permissions"
              value={permission}
            />{permission}</label
          >{/each}
      </div>
    {/if}
    <div class="form-actions">
      <button
        class="btn"
        type="button"
        disabled={busy}
        onclick={() => dialog.close()}>{t('cancel')}</button
      >
      {#if modalKind === 'delete'}
        <button class="btn btn-danger" disabled={busy}>
          {t(deletion?.kind === 'records' ? 'remove' : 'deleteProvider')}
        </button>
      {:else}
        <button class="btn btn-primary" disabled={busy}>
          {t(modalKind === 'createMissing' ? 'createMissingConfirm' : 'save')}
        </button>
      {/if}
    </div>
  </form>
</dialog>
<footer class="footer">
  <span><strong>DDNSupd</strong> · {t('selfHosted')}</span><span
    >{t('version')} ·
    <a
      href="https://www.gnu.org/licenses/agpl-3.0.html"
      target="_blank"
      rel="noreferrer">AGPL-3.0</a
    ></span
  >
</footer>
