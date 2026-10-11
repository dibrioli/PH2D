// Mostra um <caso>.json em texto compacto: passo a passo, o pedido HTTP (status) e o que cada
// participante recebeu no WebSocket, com os ids trocados por rótulos (n1, A, VOT…).
//   node resume.mjs saidas/scrumlr/v01.json [--cheio]
import { readFileSync } from "node:fs";
import { rotulador } from "./normaliza.mjs";

const f = process.argv[2];
const cheio = process.argv.includes("--cheio");
const d = JSON.parse(readFileSync(f, "utf8"));
const rot = rotulador(d.ids, d.linha_do_tempo);
const curto = (x, n = cheio ? 100000 : 400) => { const s = JSON.stringify(rot(x)) ?? "∅"; return s.length > n ? s.slice(0, n) + "…" : s; };
console.log(`# ${d.meta.caso} — ${d.meta.descricao}`);
for (const e of d.linha_do_tempo) {
  if (e.tipo === "passo") console.log(`\n[${e.passo}] ${e.texto}`);
  else if (e.tipo === "http") console.log(`   ${e.quem} ${e.metodo} ${rot(e.caminho)} ${e.corpo ? curto(e.corpo, 200) : ""} → ${e.status} ${e.resposta == null ? "" : curto(e.resposta)}`);
  else if (e.tipo === "ws") console.log(`   ⇐ ${e.quem} ${e.msg?.type} ${curto(e.msg?.data)}`);
  else if (e.tipo === "observacao") console.log(`   (obs) ${e.texto}`);
  else console.log(`   ${e.tipo} ${e.quem} ${e.code ?? ""}`);
}
