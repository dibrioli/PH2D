// Envia ao Rebelle (WebSocket, UM evento por mensagem — o formato que o servidor aceita, medido 2026-10-07) um
// documento Motion IO e espera que o servidor responda ao ÚLTIMO BOOKMARK (o fim da parte), em vez de um prazo
// fixo; sem resposta até `prazo_ms`, sai com 3. Imprime as respostas que não são o eco do servidor.
// uso: node ws_bateria.mjs <ficheiro.json> [prazo_ms=120000] [intervalo_ms=2] [porta=8265] [depois_ms=400]
import { readFileSync } from "node:fs";
const [, , fonte, prazoArg, intArg, portaArg, depoisArg] = process.argv;
const prazo = Number(prazoArg ?? 120000), intervalo = Number(intArg ?? 2), porta = Number(portaArg ?? 8265);
const depois = Number(depoisArg ?? 400);
const doc = JSON.parse(readFileSync(fonte, "utf8"));
const eventos = doc.frames ? doc.frames.flatMap(f => f.events) : (Array.isArray(doc) ? doc : [doc]);
const marca = [...eventos].reverse().find(e => e.event_type === "BOOKMARK")?.id;
const ws = new WebSocket(`ws://127.0.0.1:${porta}`);
const t0 = Date.now();
let fim = false;
const sai = (codigo, msg) => { if (fim) return; fim = true; console.log(msg); setTimeout(() => { try { ws.close(); } catch {} process.exit(codigo); }, depois); };
setTimeout(() => sai(3, `PRAZO ${prazo} ms sem o bookmark ${marca}`), prazo);
ws.onopen = async () => {
  for (const e of eventos) { ws.send(JSON.stringify(e)); if (intervalo > 0) await new Promise(r => setTimeout(r, intervalo)); }
  if (!marca) sai(0, `enviados ${eventos.length} (sem bookmark)`);
};
ws.onmessage = (m) => {
  const s = String(m.data);
  if (marca && s.includes(marca)) return sai(0, `OK ${marca} ${Date.now() - t0} ms`);
  if (!s.startsWith("Rebelle 8 Pro here")) console.log("<<", s.slice(0, 300));
};
ws.onerror = (e) => { console.log("erro", e.message ?? e); process.exit(2); };
ws.onclose = () => sai(fim ? 0 : 4, "fechado pelo servidor");
