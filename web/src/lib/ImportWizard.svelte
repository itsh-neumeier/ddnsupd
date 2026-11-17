<script lang="ts">
  import { api, ApiError } from '#lib/api.ts';
  import { t, language, type Key } from '#lib/i18n.svelte.ts';
  type Item = {
    source_index: number;
    fqdn: string;
    record_type: string;
    managed_ip: string;
    interval_seconds: number;
    ttl: number;
    proxied: boolean;
    errors: string[];
    warnings: string[];
  };
  type Preview = {
    id: string;
    digest: string;
    expires_at: string;
    plan: { items: Item[] };
  };
  let {
    providerId,
    zoneId,
    zoneName,
    onSaved
  }: {
    providerId: string;
    zoneId: string;
    zoneName: string;
    onSaved: () => Promise<void>;
  } = $props();
  let config = $state<unknown>(null);
  let fileName = $state('');
  let busy = $state(false);
  let error = $state('');
  let notice = $state('');
  let family = $state('');
  let interval = $state(300);
  let preview = $state<Preview | null>(null);
  let selected = $state<number[]>([]);
  let acknowledged = $state(false);
  let fileInput: HTMLInputElement;
  let fileVersion = 0;
  function invalidate() {
    preview = null;
    selected = [];
    acknowledged = false;
    notice = '';
  }
  const diagnostics: Record<string, Key> = {
    credentials_ignored: 'importCredentialsIgnored',
    legacy_name_converted: 'importLegacyName',
    family_selected_explicitly: 'importFamilySelected',
    provider_settings_preserved: 'importSettingsPreserved',
    import_paused: 'importAlwaysPaused',
    invalid_entry: 'importInvalidEntry',
    unsupported_provider: 'importUnsupportedProvider',
    provider_mismatch: 'importProviderMismatch',
    legacy_provider_removed: 'importLegacyRemoved',
    unsupported_option: 'importUnsupportedOption',
    unsupported_ipv6_suffix: 'importSuffixUnsupported',
    invalid_interval: 'importInvalidInterval',
    invalid_ip_version: 'importInvalidFamily',
    family_choice_required: 'importChooseFamily',
    outside_zone: 'importOutsideZone',
    invalid_domain: 'importInvalidDomain',
    zone_identifier_mismatch: 'importZoneMismatch',
    remote_record_missing: 'importMissingRecord',
    invalid_existing_address: 'importInvalidAddress',
    ambiguous_remote_record: 'importAmbiguousRecord',
    duplicate_entry: 'importDuplicate',
    already_managed: 'importAlreadyManaged',
    conflicting_legacy_name: 'importConflictingName'
  };
  const label = (code: string) => t(diagnostics[code] ?? 'importInvalidEntry');
  function explain(err: unknown) {
    return err instanceof ApiError &&
      [
        'provider_auth',
        'provider_permission',
        'provider_rate_limit',
        'provider_unavailable',
        'provider_conflict',
        'discovery_limit',
        'preview_changed',
        'preview_consumed',
        'preview_expired',
        'conflict',
        'invalid_input'
      ].includes(err.code)
      ? t(err.code as Key)
      : t('requestFailed');
  }
  async function chooseFile(event: Event) {
    const version = ++fileVersion;
    error = '';
    notice = '';
    preview = null;
    config = null;
    acknowledged = false;
    selected = [];
    const file = (event.currentTarget as HTMLInputElement).files?.[0];
    if (!file) return;
    if (file.size > 60 * 1024) {
      error = t('importFileLimit');
      fileInput.value = '';
      return;
    }
    try {
      const parsed = JSON.parse(await file.text());
      if (version !== fileVersion) return;
      config = parsed;
      fileName = file.name;
    } catch {
      if (version !== fileVersion) return;
      error = t('importInvalidFile');
      fileName = '';
      fileInput.value = '';
    }
  }
  async function analyze() {
    if (config === null) return;
    busy = true;
    error = '';
    notice = '';
    preview = null;
    acknowledged = false;
    try {
      preview = await api<Preview>('/imports/preview', 'POST', {
        provider_id: providerId,
        zone_id: zoneId,
        zone_name: zoneName,
        config,
        ambiguous_family: family || null,
        interval_seconds: interval
      });
      selected = preview.plan.items.flatMap((item, index) =>
        item.errors.length ? [] : [index]
      );
      config = null;
      fileInput.value = '';
    } catch (err) {
      error = explain(err);
    } finally {
      busy = false;
    }
  }
  function toggle(index: number, checked: boolean) {
    selected = checked
      ? [...selected, index]
      : selected.filter((value) => value !== index);
  }
  async function confirmImport() {
    if (!preview || !acknowledged || selected.length === 0) return;
    busy = true;
    error = '';
    try {
      const result = await api<{ created: unknown[]; active: false }>(
        `/imports/${preview.id}/confirm`,
        'POST',
        {
          digest: preview.digest,
          selected_indices: selected,
          accept_warnings: acknowledged
        }
      );
      notice = `${result.created.length} · ${t('importCompletePaused')}`;
      preview = null;
      selected = [];
      acknowledged = false;
      await onSaved();
    } catch (err) {
      error = explain(err);
    } finally {
      busy = false;
    }
  }
</script>

<section
  class="card"
  style="margin-top:24px"
  aria-busy={busy}
  data-testid="config-import"
>
  <div class="card-heading">
    <h2>{t('importTitle')}</h2>
    <span class="hint">{zoneName}</span>
  </div>
  <div class="card-pad">
    <p class="hint">{t('importIntro')}</p>
    {#if error}<div class="error" role="alert">
        {error}
      </div>{/if}{#if notice}<div class="notice" role="status">
        {notice}
      </div>{/if}
    <label class="field"
      >{t('importFile')}<input
        class="input"
        type="file"
        accept="application/json,.json"
        bind:this={fileInput}
        onchange={chooseFile}
        disabled={busy}
      /></label
    >
    <div class="field-row">
      <label class="field"
        >{t('importAmbiguousFamily')}<select
          class="input"
          bind:value={family}
          onchange={invalidate}
          disabled={busy}
          ><option value="">{t('importNoFamilyChoice')}</option><option
            value="A">IPv4 (A)</option
          ><option value="AAAA">IPv6 (AAAA)</option></select
        ></label
      ><label class="field"
        >{t('interval')}<input
          class="input"
          type="number"
          min="60"
          max="86400"
          bind:value={interval}
          onchange={invalidate}
          disabled={busy}
        /></label
      >
    </div>
    <button
      class="btn btn-primary"
      onclick={analyze}
      disabled={busy ||
        config === null ||
        !Number.isInteger(interval) ||
        interval < 60 ||
        interval > 86400}
      data-testid="import-preview">{t('importAnalyze')}</button
    >
    {#if fileName}<p class="hint" style="margin-top:10px">{fileName}</p>{/if}
  </div>
  {#if preview}
    <div class="card-pad">
      <h2>{t('importPreview')}</h2>
      <p class="hint">
        {t('importExpires')}
        {new Date(preview.expires_at).toLocaleString(
          language.value === 'de' ? 'de-DE' : 'en-GB'
        )}
      </p>
    </div>
    <div class="scroll-x">
      <table class="table">
        <thead
          ><tr
            ><th>{t('importSelect')}</th><th>{t('record')}</th><th
              >{t('type')}</th
            ><th class="col-opt">{t('providerValue')}</th><th
              >{t('importDiagnostics')}</th
            ></tr
          ></thead
        ><tbody
          >{#each preview.plan.items as item, index}<tr
              ><td
                ><input
                  type="checkbox"
                  checked={selected.includes(index)}
                  disabled={busy || item.errors.length > 0}
                  onchange={(event) =>
                    toggle(index, event.currentTarget.checked)}
                  aria-label={`${t('importSelect')} ${item.fqdn || index + 1}`}
                /></td
              ><td
                >{item.fqdn || '—'}
                <div class="hint">
                  {t('interval')}: {item.interval_seconds}
                </div></td
              ><td>{item.record_type || '—'}</td><td class="mono col-opt"
                >{item.managed_ip || '—'}</td
              ><td
                >{#each item.errors as code}<div
                    class="error"
                    style="padding:5px 8px;margin-bottom:5px"
                  >
                    {label(code)}
                  </div>{/each}{#each item.warnings as code}<div class="hint">
                    {label(code)}
                  </div>{/each}</td
              ></tr
            >{/each}</tbody
        >
      </table>
    </div>
    <div class="card-pad">
      <p class="hint">
        {selected.length}
        {t('of')}
        {preview.plan.items.length} · {t('importAlwaysPaused')}
      </p>
      <label class="checkbox"
        ><input
          type="checkbox"
          bind:checked={acknowledged}
          disabled={busy}
        />{t('importAcknowledge')}</label
      ><button
        class="btn btn-primary"
        style="margin-top:18px"
        onclick={confirmImport}
        disabled={busy || !acknowledged || selected.length === 0}
        data-testid="import-confirm">{t('importConfirm')}</button
      >
    </div>
  {/if}
</section>
