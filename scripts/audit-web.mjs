// Review the full build graph and fail on unknown moderate-or-higher findings.
// This exception is limited to trusted source builds, expires automatically,
// and must never apply to a package shipped in the static application runtime.
import { spawnSync } from 'node:child_process';

const allowed = 'GHSA-fc5q-39xc-vjpf';
const expiry = '2026-12-17';

function audit(args) {
  const result = spawnSync('npm', ['audit', '--json', ...args], {
    encoding: 'utf8',
    maxBuffer: 8 * 1024 * 1024
  });
  if (result.error || ![0, 1].includes(result.status)) {
    throw new Error('npm audit could not complete');
  }
  const report = JSON.parse(result.stdout);
  if (report.error || !report.vulnerabilities) {
    throw new Error('npm audit returned an invalid report');
  }
  return report.vulnerabilities;
}

const runtime = audit(['--omit=dev']);
if (Object.keys(runtime).length) {
  throw new Error('Runtime dependency findings require review');
}
const dependencies = audit([]);

function reviewed(name, ancestors = new Set()) {
  if (ancestors.has(name)) return false;
  const finding = dependencies[name];
  if (!finding || !finding.via.length) return false;
  const path = new Set(ancestors).add(name);
  return finding.via.every(cause =>
    typeof cause === 'string'
      ? reviewed(cause, path)
      : cause.url === 'https://github.com/advisories/' + allowed
  );
}

let blocked = false;
for (const finding of Object.values(dependencies)) {
  if (!['moderate', 'high', 'critical'].includes(finding.severity)) continue;
  if (new Date().toISOString().slice(0, 10) <= expiry && reviewed(finding.name)) {
    console.log('BUILD-ONLY reviewed advisory:', finding.name, allowed);
  } else {
    console.error('UNREVIEWED dependency finding:', finding.name, finding.severity);
    blocked = true;
  }
}
if (blocked) process.exitCode = 1;
else console.log('PASS runtime dependencies and bounded build-tool advisory review');
