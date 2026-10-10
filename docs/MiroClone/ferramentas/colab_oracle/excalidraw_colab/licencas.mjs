// Triagem de licença por ARTEFACTO instalado: lê o campo `license` do package.json INSTALADO
// (nunca do registry) dos pacotes que correm, e varre todo o node_modules à procura de não-permissivas.
// Uso: node licencas.mjs <COLAB_HOME> [--json]
import fs from "fs"; import path from "path";
const H = process.argv[2]; const asJson = process.argv.includes("--json");
const X = path.join(H, "excalidraw"), R = path.join(H, "excalidraw-room");
const lic = (j) => { let l = j.license; if (l && typeof l === "object") l = l.type;
  if (!l && Array.isArray(j.licenses)) l = j.licenses.map((x) => x.type).join(" OR "); return l ?? null; };
const read = (p) => { try { return JSON.parse(fs.readFileSync(p, "utf8")); } catch { return null; } };
const firstLine = (dir) => { for (const f of ["LICENSE", "LICENSE.md", "license", "LICENSE.txt"]) {
  const p = path.join(dir, f); if (fs.existsSync(p)) return fs.readFileSync(p, "utf8").split("\n").find((l) => l.trim()) ?? ""; } return ""; };
const rows = [];
const add = (onde, dir, papel) => { const j = read(path.join(dir, "package.json"));
  rows.push({ pacote: j?.name ?? path.basename(dir), versao: j?.version ?? null, license: j ? lic(j) : "AUSENTE",
    license_ficheiro: firstLine(dir), papel, onde: path.relative(H, dir) }); };
rows.push({ pacote: "excalidraw (monorepo, clone)", versao: null, license: null, license_ficheiro: firstLine(X), papel: "o app do excalidraw.com (excalidraw-app/)", onde: "excalidraw" });
rows.push({ pacote: "excalidraw-room (clone)", versao: null, license: read(path.join(R, "package.json"))?.license, license_ficheiro: firstLine(R), papel: "servidor de salas", onde: "excalidraw-room" });
for (const [d, papel] of [["packages/excalidraw", "editor (bundled no app)"], ["packages/common", "bundled"], ["packages/element", "bundled"], ["packages/math", "bundled"], ["packages/utils", "bundled"]])
  if (fs.existsSync(path.join(X, d))) add("x", path.join(X, d), papel);
add("x", path.join(X, "excalidraw-app/node_modules/socket.io-client"), "cliente da sala (o que o app embarca)");
for (const [p, papel] of [["engine.io-client", "transporte do cliente"], ["socket.io-parser", "protocolo"],
  ["firebase", "persistência (bundled; bloqueado)"], ["react", "UI"], ["react-dom", "UI"], ["jotai", "estado"], ["idb-keyval", "IndexedDB"],
  ["@excalidraw/random-username", "nome do colaborador"], ["roughjs", "desenho"], ["perfect-freehand", "traço"], ["uqr", "QR do link"],
  ["i18next-browser-languagedetector", "idioma"], ["@sentry/browser", "bundled, desligado"], ["pako", "compressão"], ["dompurify", "sanitização"], ["khroma", "mermaid (bundled)"], ["vite", "build/preview"]])
  add("x", path.join(X, "node_modules", p), papel);
for (const [p, papel] of [["socket.io", "servidor"], ["engine.io", "transporte do servidor"], ["socket.io-parser", "protocolo"], ["ws", "WebSocket"], ["express", "HTTP"], ["debug", "log"], ["dotenv", "env"]])
  add("r", path.join(R, "node_modules", p), papel);
for (const base of [path.join(H, "harness/node_modules"), path.join(H, "tools/node_modules")])
  for (const p of ["playwright", "playwright-core", "yarn"]) if (fs.existsSync(path.join(base, p))) add("h", path.join(base, p), "harness/ferramenta");
// varredura
const PERM = /^\(?(MIT|ISC|BSD-2-Clause|BSD-3-Clause|Apache-2\.0|0BSD|CC0-1\.0|Unlicense|BlueOak-1\.0\.0|Python-2\.0|CC-BY-4\.0|Zlib)((?: (?:OR|AND) [A-Za-z0-9.-]+)*)\)?$/;
const odd = []; let total = 0;
const walk = (d) => { let es; try { es = fs.readdirSync(d, { withFileTypes: true }); } catch { return; }
  for (const e of es) { if (!e.isDirectory() && !e.isSymbolicLink()) continue; const p = path.join(d, e.name);
    if (e.name.startsWith("@")) { walk(p); continue; } const j = read(path.join(p, "package.json"));
    if (j) { total++; const l = lic(j); if (!l || !PERM.test(String(l)) || /MPL|GPL|AFL/.test(String(l))) odd.push(`${j.name}@${j.version}: ${l} | ficheiro: ${firstLine(p).slice(0, 60)}`); }
    if (fs.existsSync(path.join(p, "node_modules"))) walk(path.join(p, "node_modules")); } };
walk(path.join(X, "node_modules")); walk(path.join(R, "node_modules"));
if (asJson) { console.log(JSON.stringify({ rows, varridos: total, a_triar: odd }, null, 2)); }
else { console.log("--- licenças por artefacto instalado ---");
  for (const r of rows) console.log(`${r.pacote}\t${r.versao ?? "-"}\t${r.license ?? "-"}\t[${r.license_ficheiro}]\t${r.papel}`);
  console.log(`--- varredura: ${total} package.json; fora do padrão permissivo (triar à mão):`); for (const o of odd) console.log("  " + o); }
