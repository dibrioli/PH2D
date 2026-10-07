// Envia ao Rebelle (WebSocket, um EVENTO por mensagem — é o formato que o servidor aceita; medido 2026-10-07)
// todos os eventos de um documento Motion IO {"frames":[{"events":[…]}]}, pela ordem. Imprime as respostas.
// uso: node ws_eventos.mjs <ficheiro.json> [espera_final_ms] [intervalo_ms] [porta]
import { readFileSync } from "node:fs";
const [, , fonte, esperaArg, intArg, portaArg] = process.argv;
const espera = Number(esperaArg ?? 5000), intervalo = Number(intArg ?? 2), porta = Number(portaArg ?? 8265);
const doc = JSON.parse(readFileSync(fonte, "utf8"));
const eventos = doc.frames ? doc.frames.flatMap(f => f.events) : (Array.isArray(doc) ? doc : [doc]);
const ws = new WebSocket(`ws://127.0.0.1:${porta}`);
let respostas = 0;
ws.onopen = async () => {
  console.log("ligado; a enviar", eventos.length, "eventos");
  const t0 = Date.now();
  for (const e of eventos) { ws.send(JSON.stringify(e)); if (intervalo > 0) await new Promise(r => setTimeout(r, intervalo)); }
  console.log("enviados em", Date.now() - t0, "ms; à espera", espera, "ms");
  setTimeout(() => ws.close(), espera);
};
ws.onmessage = (m) => { respostas++; const s = String(m.data); if (!s.startsWith("Rebelle 8 Pro here") && respostas < 40) console.log("<<", s.slice(0, 300)); };
ws.onerror = (e) => { console.log("erro", e.message ?? e); process.exit(2); };
ws.onclose = () => { console.log("fechado;", respostas, "respostas"); process.exit(0); };
