import assert from 'node:assert/strict';
import { readFile, mkdir } from 'node:fs/promises';
import { createServer, connect } from 'node:net';
import { chromium } from 'playwright';

const fixture = JSON.parse(await readFile(process.env.DDNSUPD_BROWSER_FIXTURE || '../../.local/qa-credentials.json', 'utf8'));
const suffix = Date.now().toString(36);
const providerName = `Browser test provider ${suffix}`;
const recordName = `browser-${suffix}`;
const host = `${recordName}.example.org`;
let proxy;
if (process.env.DDNSUPD_UPSTREAM) {
  // Preserve the configured localhost Origin inside the browser container.
  // This forwards only within the disposable QA network; no DNS/provider access.
  proxy = createServer(socket => {
    const upstream = connect(8080, process.env.DDNSUPD_UPSTREAM);
    socket.pipe(upstream); upstream.pipe(socket);
    socket.on('error', () => upstream.destroy());
    upstream.on('error', () => socket.destroy());
    socket.on('close', () => upstream.destroy());
  });
  await new Promise(resolve => proxy.listen(Number(new URL(fixture.url).port), '127.0.0.1', resolve));
}
const browser = await chromium.launch({ headless: true });
const errors = [];
const context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, locale: 'de-DE' });
const page = await context.newPage();
const browserPopups = [];
page.on('dialog', async popup => {
  browserPopups.push(popup.type());
  await popup.dismiss();
});
page.on('pageerror', error => errors.push(error.message));
page.on('console', message => { if (message.type() === 'error' && /content security|refused to execute|refused to load/i.test(message.text())) errors.push(message.text()); });
await mkdir('../../.local/screenshots', { recursive: true });
try {
  await page.goto(fixture.url);
  await page.locator('input[name=username]').waitFor();
  await page.screenshot({ path: '../../.local/screenshots/login.png', fullPage: true });
  await page.locator('input[name=username]').fill(fixture.username);
  await page.locator('input[name=password]').fill(fixture.password);
  await page.getByRole('button', { name: 'Anmelden', exact: true }).click();
  await page.locator('.sidebar').waitFor();
  await page.getByRole('heading', { name: 'Übersicht', exact: true }).waitFor();
  await page.waitForTimeout(350);
  await page.screenshot({ path: '../../.local/screenshots/dashboard.png', fullPage: true });

  await page.goto(fixture.url + '/?view=administration');
  const backupSection = page.locator('section').filter({
    has: page.getByRole('heading', { name: 'Datensicherung', exact: true })
  });
  await backupSection.locator('input[name=passphrase]').fill('fixture-backup-passphrase');
  const downloaded = page.waitForEvent('download');
  await backupSection.getByRole('button', { name: 'Verschlüsseltes Backup herunterladen' }).click();
  const backupDownload = await downloaded;
  assert.match(backupDownload.suggestedFilename(), /^ddnsupd-\d{4}-\d{2}-\d{2}\.ddnsbackup$/);
  const backupBytes = await readFile(await backupDownload.path());
  assert.equal(backupBytes.subarray(0, 14).toString(), 'DDNSUPD-BACKUP');
  await backupDownload.delete();

  await page.goto(fixture.url + '/?view=providers');
  await page.getByRole('button', { name: 'Provider verbinden', exact: true }).first().click();
  const dialog = page.locator('dialog[open]');
  await dialog.locator('[name=name]').fill(providerName);
  await dialog.locator('[name=token]').fill('fixture-browser-only-token');
  await dialog.getByRole('button', { name: 'Speichern', exact: true }).click();
  await page.locator('main > section.card').first().getByText(providerName, { exact: true }).waitFor();
  const providerRow = page.getByRole('row').filter({ hasText: providerName });
  await providerRow.getByRole('button', { name: 'Provider löschen', exact: true }).click();
  await dialog.getByText(providerName, { exact: true }).waitFor();
  await dialog.getByRole('button', { name: 'Abbrechen', exact: true }).click();

  const discovery = page.getByTestId('provider-discovery');
  await discovery.locator('select').first().selectOption({label:providerName});
  const discoveryProviderId=await discovery.locator('select').first().inputValue();
  await discovery.getByRole('button',{name:'Zugriff prüfen / Zonen laden',exact:true}).click();
  await discovery.getByRole('status').filter({hasText:'Zonen-Lesezugriff bestätigt'}).waitFor();
  await discovery.getByRole('button',{name:'DNS-Records laden',exact:true}).click();
  await discovery.getByRole('button',{name:'Record verwalten home 8.8.8.8',exact:true}).click();
  await discovery.getByRole('status').filter({hasText:'Recordzuordnung angelegt und pausiert'}).waitFor();
  const wizard=page.getByTestId('config-import');
  await wizard.locator('input[type=file]').setInputFiles({name:'config.json',mimeType:'application/json',buffer:Buffer.from(JSON.stringify({settings:[{provider:'hetznercloud',domain:'browser-import.example.org',ip_version:'ipv4',token:'source-credential-not-imported'}]}))});
  await wizard.getByTestId('import-preview').click();
  await wizard.getByRole('heading',{name:'Importvorschau',exact:true}).waitFor();
  assert.equal(await wizard.getByTestId('import-confirm').isEnabled(),false);
  await page.screenshot({path:'../../.local/screenshots/discovery-import.png',fullPage:true});
  await wizard.getByRole('checkbox',{name:/Ich habe die ausgewählten Records/}).check();
  await wizard.getByTestId('import-confirm').click();
  await wizard.getByRole('status').filter({hasText:'Zuordnungen importiert und pausiert'}).waitFor();
  const paused=await page.evaluate(async()=>{const response=await fetch('/api/v1/records');return (await response.json()).items;});
  for(const name of ['home','browser-import'])assert(paused.some(record=>record.provider_id===discoveryProviderId&&record.name===name&&record.active===false));

  await page.goto(fixture.url + '/?view=records');
  await page.getByRole('button', { name: 'Record hinzufügen', exact: true }).first().click();
  await dialog.locator('[name=provider_id]').selectOption({ label: providerName });
  await dialog.locator('[name=zone_name]').fill('example.org');
  await dialog.locator('[name=zone_remote_id]').fill('browserzone');
  await dialog.locator('[name=name]').fill(recordName);
  await dialog.locator('[name=manual_ip]').fill('8.8.4.4');
  await dialog.locator('[name=active]').check();
  await dialog.getByRole('button', { name: 'Speichern', exact: true }).click();
  await page.getByRole('link', { name: host, exact: true }).waitFor();
  await page.getByRole('link', { name: host, exact: true }).click();
  await page.locator('.panel').waitFor();
  await page.locator('.panel').getByRole('button', { name: 'Fehlenden DNS-Record anlegen', exact: true }).click();
  await dialog.getByText(host + ' · A', { exact: true }).waitFor();
  await dialog.getByRole('button', { name: 'Bei Hetzner anlegen', exact: true }).waitFor();
  await dialog.getByRole('button', { name: 'Abbrechen', exact: true }).click();
  await page.locator('.panel input[name=site]').fill('Browser QA');
  await page.locator('.panel').getByRole('button', { name: 'Speichern', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'Gespeichert' }).waitFor();
  await page.locator('.panel').getByRole('button', { name: 'Jetzt prüfen', exact: true }).click();
  await page.getByRole('status').filter({ hasText: 'Prüfung eingereiht' }).waitFor();
  assert(new URL(page.url()).searchParams.get('panel'));
  await page.locator('.panel').getByRole('button', { name: 'AAAA-Record hinzufügen', exact: true }).click();
  const companion = await dialog.locator('form').evaluate(form => Object.fromEntries(new FormData(form)));
  assert.equal(companion.provider_id, discoveryProviderId);
  assert.equal(companion.zone_name, 'example.org');
  assert.equal(companion.zone_remote_id, 'browserzone');
  assert.equal(companion.name, recordName);
  assert.equal(companion.record_type, 'AAAA');
  assert.equal(companion.interval_seconds, '300');
  assert.equal(companion.site, 'Browser QA');
  assert.equal(companion.manual_ip, '');
  assert.equal(companion.managed_ip, '');
  assert.equal(companion.active, undefined);
  for (const viewport of [{ width: 1440, height: 1000 }, { width: 360, height: 560 }]) {
    await page.setViewportSize(viewport);
    const box = await dialog.boundingBox();
    assert(box, 'AAAA dialog is visible');
    assert(Math.abs(box.x + box.width / 2 - viewport.width / 2) <= 1, 'dialog is horizontally centered');
    assert(Math.abs(box.y + box.height / 2 - viewport.height / 2) <= 1, 'dialog is vertically centered');
    assert(box.width < viewport.width && box.height < viewport.height, 'dialog fits the viewport');
  }
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.keyboard.press('Escape');

  for (const width of [360, 768, 1280]) {
    await page.setViewportSize({ width, height: 900 });
    for (const dark of [false, true]) {
      await page.evaluate(dark => { document.documentElement.classList.toggle('dark', dark); localStorage.setItem('ddnsupd-theme', dark ? 'dark' : 'light'); }, dark);
      for (const view of ['dashboard', 'records', 'providers', 'history', 'administration']) {
        await page.goto(fixture.url + '/?view=' + view);
        await page.locator('.sidebar').waitFor();
        await page.waitForTimeout(280);
        assert(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `horizontal page overflow: ${width} ${dark} ${view}`);
        assert.equal(await page.evaluate(() => document.documentElement.classList.contains('dark')), dark);
      }
    }
  }
  await page.setViewportSize({ width: 360, height: 900 });
  await page.goto(fixture.url + '/?view=records');
  await page.getByRole('link', { name: host, exact: true }).click();
  await page.locator('.panel').waitFor();
  await page.waitForTimeout(350);
  await page.screenshot({ path: '../../.local/screenshots/mobile-record.png', fullPage: true });
  await page.getByRole('button', { name: 'Sprache', exact: true }).click();
  await page.goto(fixture.url + '/?view=history');
  await page.getByRole('heading', { name: 'Activity', exact: true }).waitFor();
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto(fixture.url + '/?view=records');
  await page.getByRole('button', { name: 'Add record', exact: true }).first().click();
  assert.equal(await dialog.locator('[name=record_type]').inputValue(), 'A');
  assert.equal(await dialog.locator('[name=zone_remote_id]').inputValue(), '');
  await page.keyboard.press('Escape');
  assert.equal(await page.locator('dialog[open]').count(), 0, 'native dialog closes with Escape');
  await page.getByRole('link', { name: host, exact: true }).click();
  const deleteBinding = page.locator('.panel').getByRole('button', { name: 'Delete binding', exact: true });
  await deleteBinding.click();
  await dialog.getByText(host, { exact: true }).waitFor();
  await dialog.getByRole('button', { name: 'Cancel', exact: true }).click();
  await deleteBinding.click();
  await dialog.getByRole('button', { name: 'Delete binding', exact: true }).click();
  await page.getByRole('link', { name: host, exact: true }).waitFor({ state: 'detached' });
  assert.deepEqual(browserPopups, [], 'no browser popups');
  await page.getByRole('button', { name: 'Sign out', exact: true }).click();
  await page.locator('input[name=username]').waitFor();
  assert.deepEqual(errors, [], 'no browser runtime or CSP errors');
  console.log('PASS browser discovery / paused binding / paginated config preview / explicit confirmation; create / edit / enqueue / locale / logout; 30 responsive theme views; reduced motion / dialog keyboard; no runtime/CSP errors');
} finally {
  await browser.close();
  proxy?.close();
}
