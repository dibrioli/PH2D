// Triagem de licença por ARTEFACTO instalado (cwd = ORACLE_HOME): para cada pacote de node_modules,
// o campo `license` do package.json INSTALADO e o ficheiro LICENSE*/COPYING* que vem no pacote (1.ª
// linha não vazia), mais a RAIZ que o puxa (pelo package-lock: fecho de dependências de cada pacote
// directo). Marca o que não é permissivo. Uso: node licencas.mjs [--md | --resumo]
import fs from "node:fs";
import path from "node:path";

const root = path.join(process.cwd(), "node_modules");
const md = process.argv.includes("--md");
const resumo = process.argv.includes("--resumo");
const PERMISSIVAS = new Set(["MIT", "ISC", "BSD-2-Clause", "BSD-3-Clause", "Apache-2.0", "0BSD", "Unlicense", "BlueOak-1.0.0", "Python-2.0", "Zlib", "(MIT AND Zlib)"]);
// Dupla licença: vale a escolha permissiva (declarada na tabela).
const permissiva = (lic) => PERMISSIVAS.has(lic) || /^\(?MIT OR /.test(lic);

const lock = JSON.parse(fs.readFileSync("package-lock.json", "utf8"));
const pk = lock.packages;
const top = pk[""];
const directos = Object.keys({ ...top.dependencies, ...top.devDependencies });
const resolve = (from, name) => {
  let dir = from;
  for (;;) {
    const k = (dir ? dir + "/" : "") + "node_modules/" + name;
    if (pk[k]) return k;
    if (!dir) return null;
    const i = dir.lastIndexOf("/node_modules/");
    dir = i < 0 ? "" : dir.slice(0, i);
  }
};
const raizes = {};
for (const d of directos) {
  const seen = new Set();
  const st = ["node_modules/" + d];
  while (st.length) {
    const k = st.pop();
    if (!k || seen.has(k)) continue;
    seen.add(k);
    const e = pk[k];
    for (const n of Object.keys({ ...e.dependencies, ...e.optionalDependencies, ...e.peerDependencies })) st.push(resolve(k, n));
  }
  for (const k of seen) (raizes[k.replace(/^node_modules\//, "")] ??= new Set()).add(d);
}

const rows = [];
const visit = (dir) => {
  for (const name of fs.readdirSync(dir)) {
    if (name.startsWith(".")) continue;
    const p = path.join(dir, name);
    if (name.startsWith("@")) { visit(p); continue; }
    const pj = path.join(p, "package.json");
    if (!fs.existsSync(pj)) continue;
    const j = JSON.parse(fs.readFileSync(pj, "utf8"));
    const lic = typeof j.license === "string" ? j.license : j.license?.type ?? (j.licenses ? j.licenses.map((l) => l.type).join(" OR ") : "—");
    const lf = fs.readdirSync(p).find((f) => /^(licen[cs]e|copying)/i.test(f));
    let first = "";
    if (lf) first = fs.readFileSync(path.join(p, lf), "utf8").split("\n").map((s) => s.trim()).find((s) => s) ?? "";
    const rel = path.relative(root, p).replace(/\\/g, "/");
    rows.push({ pkg: rel, ver: j.version, lic, file: lf ? path.relative(root, path.join(p, lf)) : "(nenhum)", first: first.slice(0, 90), raiz: [...(raizes[rel] ?? [])].sort().join(" ") || "?", ok: permissiva(lic) });
    const nested = path.join(p, "node_modules");
    if (fs.existsSync(nested)) visit(nested);
  }
};
visit(root);
rows.sort((a, b) => a.pkg.localeCompare(b.pkg));
if (resumo) {
  const c = {};
  for (const r of rows) c[r.lic] = (c[r.lic] ?? 0) + 1;
  console.log(`${rows.length} pacotes instalados:`, Object.entries(c).sort((a, b) => b[1] - a[1]).map(([k, v]) => `${v} ${k}`).join(" · "));
  for (const d of directos.sort()) {
    const sub = rows.filter((r) => r.raiz.split(" ").includes(d));
    const bad = sub.filter((r) => !r.ok);
    console.log(`  ${d}: ${sub.length} pacotes no fecho; não-permissivos: ${bad.length ? bad.map((r) => `${r.pkg} (${r.lic})`).join(", ") : "nenhum"}`);
  }
  const semFicheiro = rows.filter((r) => r.file === "(nenhum)");
  console.log(`  sem ficheiro de licença no artefacto (vale o campo license): ${semFicheiro.map((r) => `${r.pkg} (${r.lic})`).join(", ")}`);
} else if (md) {
  console.log("| pacote | versão | license (package.json) | ficheiro lido | 1.ª linha do ficheiro | puxado por |");
  console.log("|---|---|---|---|---|---|");
  for (const r of rows) console.log(`| \`${r.pkg}\` | ${r.ver} | ${r.lic}${r.ok ? "" : " ⛔"} | ${r.file} | ${r.first.replace(/\|/g, "\\|")} | ${r.raiz} |`);
} else for (const r of rows) console.log([r.pkg, r.ver, r.lic, r.file, r.first, r.raiz, r.ok ? "ok" : "NAO-PERMISSIVA"].join("\t"));
