<script lang="ts">
  import { api, ApiError, type Provider } from '#lib/api.ts';
  import { t, type Key } from '#lib/i18n.svelte.ts';
  import ImportWizard from '#lib/ImportWizard.svelte';
  type Zone = { id: string; name: string; mode: string };
  type Remote = {
    name: string;
    record_type: string;
    record_id: string;
    value: string;
    ttl: number;
    proxied: boolean;
    bindable: boolean;
  };
  type Discovery = {
    zones: Zone[];
    records: Remote[];
    page: number;
    next_page: number | null;
    permission_verification: string;
  };
  let {
    providers,
    canBind,
    canImport,
    onSaved
  }: {
    providers: Provider[];
    canBind: boolean;
    canImport: boolean;
    onSaved: () => Promise<void>;
  } = $props();
  let providerId = $state('');
  let zoneId = $state('');
  let zones = $state<Zone[]>([]);
  let records = $state<Remote[]>([]);
  let zonePage = $state(1);
  let recordPage = $state(1);
  let nextZones = $state<number | null>(null);
  let nextRecords = $state<number | null>(null);
  let busy = $state(false);
  let error = $state('');
  let notice = $state('');
  let zonesLoaded = $state(false);
  let recordsLoaded = $state(false);
  let zone = $derived(zones.find((z) => z.id === zoneId));
  $effect(() => {
    if (!providers.some((p) => p.id === providerId)) {
      providerId = providers[0]?.id ?? '';
      reset();
    }
  });
  function reset() {
    zones = [];
    records = [];
    zoneId = '';
    zonesLoaded = false;
    recordsLoaded = false;
    notice = '';
    error = '';
  }
  function changedZone() {
    records = [];
    recordsLoaded = false;
    notice = '';
    error = '';
  }
  function explain(err: unknown) {
    return err instanceof ApiError &&
      [
        'provider_auth',
        'provider_permission',
        'provider_rate_limit',
        'provider_unavailable',
        'provider_conflict',
        'rate_limited',
        'conflict'
      ].includes(err.code)
      ? t(err.code as Key)
      : t('requestFailed');
  }
  async function load(operation: 'zones' | 'records', page = 1) {
    if (!providerId || (operation === 'records' && !zone)) return;
    busy = true;
    error = '';
    notice = '';
    try {
      const result = await api<Discovery>(
        `/providers/${providerId}/discover`,
        'POST',
        {
          operation,
          page,
          zone_id: zone?.id ?? '',
          zone_name: zone?.name ?? ''
        }
      );
      if (operation === 'zones') {
        zones = result.zones;
        zonePage = result.page;
        nextZones = result.next_page;
        zoneId = zones[0]?.id ?? '';
        zonesLoaded = true;
        changedZone();
        notice = t('readConnectionOk');
      } else {
        records = result.records;
        recordPage = result.page;
        nextRecords = result.next_page;
        recordsLoaded = true;
      }
    } catch (err) {
      error = explain(err);
    } finally {
      busy = false;
    }
  }
  async function bind(record: Remote) {
    if (!zone || !canBind) return;
    busy = true;
    error = '';
    notice = '';
    try {
      await api('/records', 'POST', {
        provider_id: providerId,
        zone_remote_id: zone.id,
        zone_name: zone.name,
        name: record.name,
        record_type: record.record_type,
        remote_record_id: record.record_id,
        managed_ip: record.value,
        manual_ip: '',
        active: false,
        interval_seconds: 300,
        site: ''
      });
      notice = t('boundPaused');
      await onSaved();
    } catch (err) {
      error = explain(err);
    } finally {
      busy = false;
    }
  }
</script>

{#if providers.length}
  <section
    class="card"
    style="margin-top:24px"
    aria-busy={busy}
    data-testid="provider-discovery"
  >
    <div class="card-heading"><h2>{t('discoveryTitle')}</h2></div>
    <div class="card-pad">
      <p class="hint">{t('discoveryHint')}</p>
      {#if error}<div class="error" role="alert">{error}</div>{/if}
      {#if notice}<div class="notice" role="status">{notice}</div>{/if}
      <div class="toolbar" style="flex-wrap:wrap">
        <label class="field" style="margin:0;flex:1"
          >{t('provider')}<select
            class="input"
            bind:value={providerId}
            onchange={reset}
            disabled={busy}
            >{#each providers as provider}<option value={provider.id}
                >{provider.name}</option
              >{/each}</select
          ></label
        >
        <button
          class="btn"
          onclick={() => load('zones')}
          disabled={busy || !providerId}>{t('testAndLoadZones')}</button
        >
      </div>
      {#if zonesLoaded}
        {#if zones.length}
          <div class="toolbar" style="flex-wrap:wrap">
            <label class="field" style="margin:0;flex:1"
              >{t('zones')}<select
                class="input"
                bind:value={zoneId}
                onchange={changedZone}
                disabled={busy}
                >{#each zones as item}<option value={item.id}
                    >{item.name}{item.mode === 'secondary'
                      ? ' · ' + t('readOnlyZone')
                      : ''}</option
                  >{/each}</select
              ></label
            >
            <button
              class="btn"
              onclick={() => load('records')}
              disabled={busy || !zone}>{t('loadRemoteRecords')}</button
            >
          </div>
        {:else}<p class="hint">{t('noZones')}</p>{/if}
        <div class="pagination">
          <button
            class="btn"
            onclick={() => load('zones', zonePage - 1)}
            disabled={busy || zonePage <= 1}>{t('back')}</button
          ><button
            class="btn"
            onclick={() => load('zones', nextZones!)}
            disabled={busy || nextZones === null}>{t('next')}</button
          >
        </div>
      {/if}
    </div>
    {#if recordsLoaded}
      {#if records.length}<div class="scroll-x">
          <table class="table">
            <thead
              ><tr
                ><th>{t('record')}</th><th>{t('type')}</th><th
                  >{t('providerValue')}</th
                ><th class="col-opt">TTL</th><th></th></tr
              ></thead
            ><tbody
              >{#each records as record, index}<tr
                  ><td
                    >{record.name === '@'
                      ? zone?.name
                      : `${record.name}.${zone?.name}`}{#if record.proxied}<div
                        class="hint"
                      >
                        {t('proxiedOrigin')}
                      </div>{/if}</td
                  ><td>{record.record_type}</td><td class="mono"
                    >{record.value}</td
                  ><td class="col-opt">{record.ttl}</td><td
                    >{#if canBind}<button
                        class="btn"
                        disabled={busy ||
                          !record.bindable ||
                          zone?.mode === 'secondary'}
                        onclick={() => bind(record)}
                        aria-label={`${t('bindExisting')} ${record.name} ${record.value}`}
                        >{t('bindExisting')}</button
                      >{/if}{#if !record.bindable}<span class="hint"
                        >{t('nonPublicRecord')}</span
                      >{/if}</td
                  ></tr
                >{/each}</tbody
            >
          </table>
        </div>{:else}<div class="empty">
          <p>{t('noAddressRecords')}</p>
        </div>{/if}
      <div class="card-pad pagination">
        <button
          class="btn"
          onclick={() => load('records', recordPage - 1)}
          disabled={busy || recordPage <= 1}>{t('back')}</button
        ><button
          class="btn"
          onclick={() => load('records', nextRecords!)}
          disabled={busy || nextRecords === null}>{t('next')}</button
        >
      </div>
    {/if}
  </section>
  {#if canImport && zone && zone.mode !== 'secondary'}{#key providerId + ':' + zone.id}<ImportWizard
        {providerId}
        zoneId={zone.id}
        zoneName={zone.name}
        {onSaved}
      />{/key}{/if}
{/if}
