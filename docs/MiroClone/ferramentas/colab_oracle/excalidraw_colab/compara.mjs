// Junta DUAS corridas completas em saidas/excalidraw/<caso>.json + meta.json.
// Campo `deterministico` tem de ser igual nas duas (senão: divergência registada); `medidas` e
// `variavel` levam a faixa das duas. O detalhe (rodadas, estado, mensagens) é o da 1.ª corrida.
// Uso: node compara.mjs <corrida1> <corrida2> <saida> <meta_extra.json>
import fs from "fs";
import path from "path";
import crypto from "crypto";

const [d1, d2, out, metaExtra] = process.argv.slice(2);
const HERE = path.dirname(new URL(import.meta.url).pathname);
fs.mkdirSync(out, { recursive: true });
const ids = fs.readdirSync(d1).filter((f) => /^c\d\d_.*\.json$/.test(f)).sort();

// folhas numéricas -> [min, max] das duas corridas; o resto: igual ou {c1, c2}
function faixa(a, b) {
  if (typeof a === "number" && typeof b === "number") return a === b ? a : [Math.min(a, b), Math.max(a, b)];
  if (a && b && typeof a === "object" && typeof b === "object" && Array.isArray(a) === Array.isArray(b)) {
    const ks = [...new Set([...Object.keys(a), ...Object.keys(b)])];
    const o = Array.isArray(a) ? [] : {}; for (const k of ks) o[k] = faixa(a[k], b[k]); return o;
  }
  return JSON.stringify(a) === JSON.stringify(b) ? a : { corrida1: a, corrida2: b };
}
function difs(a, b, p = "") {
  if (JSON.stringify(a) === JSON.stringify(b)) return [];
  if (a && b && typeof a === "object" && typeof b === "object") return [...new Set([...Object.keys(a), ...Object.keys(b)])].flatMap((k) => difs(a[k], b[k], `${p}.${k}`));
  return [{ campo: p || "(raiz)", corrida1: a, corrida2: b }];
}

const resumo = [];
for (const f of ids) {
  const a = JSON.parse(fs.readFileSync(path.join(d1, f), "utf8"));
  const b = fs.existsSync(path.join(d2, f)) ? JSON.parse(fs.readFileSync(path.join(d2, f), "utf8")) : null;
  const div = b ? difs({ existe: a.existe, erro: !!a.erro, deterministico: a.deterministico }, { existe: b.existe, erro: !!b.erro, deterministico: b.deterministico }) : [{ campo: "(caso)", corrida1: "corrido", corrida2: "AUSENTE" }];
  const msgs = Array.isArray(a.mensagens) ? a.mensagens : a.mensagens?.primeiras ?? [];
  const o = {
    id: a.id, o_que: a.o_que, pergunta: a.pergunta, existe: a.existe,
    resposta: a.resposta, resposta_corrida2: b?.resposta ?? null,
    deterministico_igual_nas_duas: div.length === 0, divergencias_entre_corridas: div,
    deterministico: a.deterministico, variavel: b ? faixa(a.variavel, b.variavel) : a.variavel,
    medidas_faixa_das_duas: b ? faixa(a.medidas, b.medidas) : a.medidas,
    erro: a.erro, erro_corrida2: b?.erro ?? null,
    rodadas: a.rodadas, prova: a.prova, estado_final: a.estado_final,
    pedidos_bloqueados: [...new Set([...(a.pedidos_bloqueados ?? []), ...(b?.pedidos_bloqueados ?? [])])].sort(),
    polling_http: a.polling_http, erros_pagina: a.erros_pagina, eventos_ws: a.eventos_ws, link_forma: a.link_forma,
    mensagens: { campos: ["t_ms", "par", "dir", "evento", "bytes"], n: msgs.length, linhas: msgs.map((m) => [m.t, m.par, m.dir, m.ev, m.bytes]) },
  };
  const txt = JSON.stringify(o, null, 1).replace(/\[\n\s+(\d+),\n\s+"([^"]*)",\n\s+"([^"]*)",\n\s+"([^"]*)",\n\s+(\d+)\n\s+\]/g, '[$1,"$2","$3","$4",$5]');
  fs.writeFileSync(path.join(out, f), txt + "\n");
  resumo.push({ id: a.id, igual: div.length === 0, divergencias: div.length, erro: !!a.erro || !!b?.erro });
}
const casos = fs.readFileSync(path.join(HERE, "../casos.json"));
const c1 = JSON.parse(fs.readFileSync(path.join(d1, "_corrida.json"), "utf8")), c2 = JSON.parse(fs.readFileSync(path.join(d2, "_corrida.json"), "utf8"));
const meta = {
  oraculo: "excalidraw_colab — o app do excalidraw.com (excalidraw-app/) + excalidraw-room, tudo em 127.0.0.1",
  sha256_casos_json: crypto.createHash("sha256").update(casos).digest("hex"),
  corridas: [{ quando: c1.quando, chromium: c1.versoes.chromium }, { quando: c2.quando, chromium: c2.versoes.chromium }],
  ...JSON.parse(fs.readFileSync(metaExtra, "utf8")),
  resumo_das_duas_corridas: resumo,
};
fs.writeFileSync(path.join(out, "meta.json"), JSON.stringify(meta, null, 1) + "\n");
console.log(resumo.map((r) => `${r.id}: ${r.igual ? "igual" : `DIVERGE (${r.divergencias})`}${r.erro ? " · ERRO" : ""}`).join("\n"));
