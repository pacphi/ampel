#!/usr/bin/env node
// i18n key-coverage audit: every component t('ns:key') reference must resolve to a real EN key.
// Reports UNRESOLVED references (missing keys or mis-pathed refs). Exit 1 if any unresolved.
//
// Scope/heuristics:
//  - Resolves explicit `t('ns:key.path')` against frontend/public/locales/en/<ns>.json.
//  - For `t('key.path')` without a namespace, uses the file's first useTranslation([...]) namespace.
//  - Dynamic keys (template literals) are expanded for the known provider suffix set; anything else
//    still dynamic is listed under "dynamic (manual)" rather than failed.
import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { join, extname } from 'node:path';

const SRC = 'frontend/src';
const EN = 'frontend/public/locales/en';
const PROVIDERS = ['github', 'gitlab', 'bitbucket']; // for tokenPlaceholder_${provider} etc.

const walk = (d, acc = []) => {
  for (const e of readdirSync(d, { withFileTypes: true })) {
    const p = join(d, e.name);
    if (e.isDirectory()) walk(p, acc);
    else if (['.ts', '.tsx'].includes(extname(e.name)) && !p.endsWith('.d.ts')) acc.push(p);
  }
  return acc;
};

const nsCache = {};
const loadNs = (ns) => {
  if (ns in nsCache) return nsCache[ns];
  const f = join(EN, `${ns}.json`);
  nsCache[ns] = existsSync(f) ? JSON.parse(readFileSync(f, 'utf8')) : null;
  return nsCache[ns];
};
const resolve = (obj, path) => path.split('.').reduce((a, k) => (a && typeof a === 'object' ? a[k] : undefined), obj);

const unresolved = [];
const dynamic = [];

for (const file of walk(SRC)) {
  const txt = readFileSync(file, 'utf8');
  // default namespaces from the first useTranslation([...]) / useTranslation('x')
  const m = txt.match(/useTranslation\(\s*(\[[^\]]*\]|['"][^'"]+['"])/);
  let defaultNs = null;
  if (m) {
    const first = m[1].replace(/[[\]'"]/g, '').split(',')[0].trim();
    if (first) defaultNs = first;
  }
  // all t('...') / t("...") / t(`...`) first-arg literals
  const re = /\bt\(\s*([`'"])((?:\\.|(?!\1).)*)\1/g;
  let c;
  while ((c = re.exec(txt))) {
    const raw = c[2];
    // Dynamic keys (template interpolation): expand ONLY the known provider-suffix family
    // (e.g. tokenPlaceholder_${provider}); other dynamic keys resolve to runtime enum values
    // we can't enumerate here — list them as informational, never as failures.
    if (raw.includes('${')) {
      const isProviderSuffix = /_\$\{[^}]+\}$/.test(raw); // ..._${provider}
      if (isProviderSuffix) {
        for (const p of PROVIDERS) {
          const key = raw.replace(/\$\{[^}]+\}/g, p);
          const [ns, rest] = key.includes(':') ? key.split(/:(.+)/) : [defaultNs, key];
          if (!ns) continue;
          if (typeof resolve(loadNs(ns), rest) !== 'string') unresolved.push(`${ns}:${rest}  (${file})`);
        }
      } else {
        dynamic.push(`${raw}  (${file})`);
      }
      continue;
    }
    const key = raw;
    const [ns, rest] = key.includes(':') ? key.split(/:(.+)/) : [defaultNs, key];
    if (!ns) continue; // can't resolve namespace; skip
    const data = loadNs(ns);
    if (typeof resolve(data, rest) !== 'string') unresolved.push(`${ns}:${rest}  (${file})`);
  }
}

const uniq = (a) => [...new Set(a)].sort();
const U = uniq(unresolved);
const D = uniq(dynamic);
if (D.length) {
  console.log(`\nDynamic keys (verify manually) — ${D.length}:`);
  D.forEach((e) => console.log('  ? ' + e));
}
console.log(`\nUNRESOLVED t() references — ${U.length}:`);
U.forEach((e) => console.log('  ✗ ' + e));
if (U.length === 0) console.log('  (none — every static t() reference resolves to an EN key ✓)');
process.exit(U.length ? 1 : 0);
