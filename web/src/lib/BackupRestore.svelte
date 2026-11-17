<script lang="ts">
  import { api, ApiError } from '#lib/api.ts';
  import { t } from '#lib/i18n.svelte.ts';

  let {
    setup = false,
    onrestored
  }: { setup?: boolean; onrestored: () => void } = $props();
  let busy = $state(false);
  let error = $state('');
  const maxArchive = 16 * 1024 * 1024;

  async function restore(event: SubmitEvent) {
    event.preventDefault();
    if (busy) return;
    const form = new FormData(event.currentTarget as HTMLFormElement);
    const file = form.get('archive');
    error = '';
    if (!(file instanceof File) || file.size === 0 || file.size > maxArchive) {
      error = t('backupFileSize');
      return;
    }
    busy = true;
    try {
      const bytes = new Uint8Array(await file.arrayBuffer());
      const digits = Array.from({ length: 256 }, (_, i) =>
        i.toString(16).padStart(2, '0')
      );
      const chunks = [];
      for (let offset = 0; offset < bytes.length; offset += 8192) {
        chunks.push(
          Array.from(
            bytes.subarray(offset, offset + 8192),
            (n) => digits[n]
          ).join('')
        );
      }
      await api(
        setup ? '/setup/restore' : '/backup/restore',
        'POST',
        {
          archive: chunks.join(''),
          passphrase: form.get('passphrase'),
          confirm: form.has('confirm')
        },
        setup ? { 'X-Setup-Token': String(form.get('setup_token')) } : undefined
      );
      onrestored();
    } catch (cause) {
      if (cause instanceof ApiError) {
        switch (cause.code) {
          case 'backup_invalid':
            error = t('backup_invalid');
            break;
          case 'backup_incompatible':
            error = t('backup_incompatible');
            break;
          case 'backup_busy':
            error = t('backup_busy');
            break;
          case 'setup_complete':
            error = t('setup_complete');
            break;
          case 'forbidden':
            error = t('forbidden');
            break;
          default:
            error = t(
              cause.status === 413 ? 'backupFileSize' : 'requestFailed'
            );
        }
      } else error = t('requestFailed');
    } finally {
      busy = false;
    }
  }
</script>

<form onsubmit={restore} aria-busy={busy}>
  <p class="hint">{t(setup ? 'backupSetupHint' : 'backupReplaceHint')}</p>
  {#if error}<div class="error" role="alert">{error}</div>{/if}
  {#if setup}
    <label class="field">
      {t('setupToken')}
      <input
        class="input"
        name="setup_token"
        type="password"
        required
        autocomplete="off"
        disabled={busy}
      />
    </label>
  {/if}
  <label class="field">
    {t('backupFile')}
    <input
      class="input"
      type="file"
      name="archive"
      accept=".ddnsbackup,application/octet-stream"
      required
      disabled={busy}
    />
  </label>
  <label class="field">
    {t('backupPassphrase')}
    <input
      class="input"
      name="passphrase"
      type="password"
      minlength="12"
      maxlength="256"
      required
      autocomplete="off"
      disabled={busy}
    />
  </label>
  <label class="checkbox">
    <input type="checkbox" name="confirm" required disabled={busy} />
    {t(setup ? 'backupConfirmSetup' : 'backupConfirmReplace')}
  </label>
  <div class="form-actions">
    <button class="btn btn-primary" disabled={busy}
      >{t(busy ? 'loading' : 'backupRestore')}</button
    >
  </div>
</form>
