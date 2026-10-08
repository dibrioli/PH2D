// Triagem de licença por ARTEFACTO instalado: para cada pacote de node_modules (cwd = ORACLE_HOME),
// o campo `license` do package.json INSTALADO e o ficheiro LICENSE* que vem no pacote (primeira linha
// não vazia). Saída TSV: pacote, versão, license, ficheiro, 1.ª linha. Uso: node licencas.mjs [--md]
import fs from "node:fs";
import path from "node:path";

const root = path.join(process.cwd(), "node_modules");
const md = process.argv.includes("--md");
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
    rows.push([path.relative(root, p), j.version, lic, lf ? path.relative(root, path.join(p, lf)) : "(nenhum)", first.slice(0, 90)]);
    const nested = path.join(p, "node_modules");
    if (fs.existsSync(nested)) visit(nested);
  }
};
visit(root);
rows.sort((a, b) => a[0].localeCompare(b[0]));
if (md) {
  console.log("| pacote | versão | license (package.json) | ficheiro lido | 1.ª linha do ficheiro |");
  console.log("|---|---|---|---|---|");
  for (const r of rows) console.log(`| \`${r[0]}\` | ${r[1]} | ${r[2]} | ${r[3]} | ${r[4].replace(/\|/g, "\\|")} |`);
} else for (const r of rows) console.log(r.join("\t"));
