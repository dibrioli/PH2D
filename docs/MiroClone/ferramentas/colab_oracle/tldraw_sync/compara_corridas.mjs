// Compara duas corridas completas do corre_colab.mjs: os blocos `deterministico` (estado final,
// convergência, ordem de z, contagens) têm de ser IGUAIS; dos `variavel` regista a faixa dos números
// que o relatório cita (c13, c18, c19, ms até convergir). Grava `duas_corridas` no meta.json da 1.ª.
// Uso: node compara_corridas.mjs <corrida1> <corrida2>
import fs from "node:fs";
import path from "node:path";

const [d1, d2] = process.argv.slice(2);
if (!d1 || !d2) { console.error("uso: node compara_corridas.mjs <corrida1> <corrida2>"); process.exit(2); }
const casos = fs.readdirSync(d1).filter((f) => /^c\d\d_.*\.json$/.test(f)).sort();

// Primeira diferença entre dois valores JSON, com o caminho.
const difere = (a, b, p = "") => {
  if (JSON.stringify(a) === JSON.stringify(b)) return null;
  if (a && b && typeof a === "object" && typeof b === "object") {
    for (const k of new Set([...Object.keys(a), ...Object.keys(b)])) { const d = difere(a[k], b[k], `${p}/${k}`); if (d) return d; }
  }
  return { caminho: p, corrida1: JSON.stringify(a)?.slice(0, 200), corrida2: JSON.stringify(b)?.slice(0, 200) };
};
const faixa = (xs) => { const v = xs.filter((x) => typeof x === "number"); return v.length ? [Math.min(...v), Math.max(...v)] : null; };
const pega = (o, caminho) => caminho.split(".").reduce((x, k) => (x == null ? x : x[k]), o);

const NUMEROS = {
  c13_entrar_a_meio: ["entrada.ms_ate_synced_remote", "entrada.ms_ate_B_igual_a_A", "entrada.ate_igual.mensagens", "entrada.ate_igual.bytes_recebidos", "entrada.ate_igual.bytes_enviados", "entrada.primeira_resposta_connect_bytes"],
  c18_cursor: ["cursor.A_envia.mensagens", "cursor.A_envia.por_s", "cursor.A_envia.bytes_por_mensagem", "cursor.A_envia.bytes_por_s", "cursor.posicoes_distintas_por_s", "cursor.hz_de_amostragem", "cursor.salto_px.mediana", "cursor.intervalo_entre_mudancas_ms.mediana"],
  c19_arrastar_custo: ["arrasto.A_envia.por_s", "arrasto.A_envia.bytes_por_s", "arrasto.A_envia.bytes_por_mensagem", "arrasto.B_ve.posicoes_x_distintas_de_S0"],
  c21_seguir: ["camara.atraso_ms_B_alcanca_a_camara_final_de_A"],
};

const out = {};
let divergencias = 0;
for (const f of casos) {
  const a = JSON.parse(fs.readFileSync(path.join(d1, f), "utf8"));
  const b = JSON.parse(fs.readFileSync(path.join(d2, f), "utf8"));
  const subs = a.subcorridas.map((s, i) => {
    const t = b.subcorridas[i];
    const d = t ? difere(s.deterministico, t.deterministico) : { caminho: "(subcorrida ausente na 2.ª)" };
    if (d) divergencias++;
    const r = { subcorrida: s.subcorrida, deterministico_igual: !d, ...(d ? { primeira_diferenca: d } : {}) };
    r.ms_ate_convergir = faixa([s.variavel?.ms_ate_convergir_depois_do_ultimo_gesto, t?.variavel?.ms_ate_convergir_depois_do_ultimo_gesto]);
    for (const c of NUMEROS[a.caso] || []) r[c] = faixa([pega(s.variavel, c), pega(t?.variavel, c)]);
    return r;
  });
  out[a.caso] = { existe_igual: a.existe === b.existe, resposta_igual: a.resposta === b.resposta, ...(a.resposta !== b.resposta ? { resposta1: a.resposta, resposta2: b.resposta } : {}), subcorridas: subs };
  if (a.existe !== b.existe) divergencias++;
}
const resumo = { comparado_em: new Date().toISOString(), corrida2: path.resolve(d2), casos: casos.length, subcorridas_com_deterministico_diferente: divergencias, por_caso: out };
const metaP = path.join(d1, "meta.json");
if (fs.existsSync(metaP)) { const m = JSON.parse(fs.readFileSync(metaP, "utf8")); m.duas_corridas = resumo; fs.writeFileSync(metaP, JSON.stringify(m, null, 2) + "\n"); }
for (const [k, v] of Object.entries(out)) console.log(`${k}: ${v.subcorridas.every((s) => s.deterministico_igual) && v.existe_igual ? "igual" : "DIVERGE " + JSON.stringify(v.subcorridas.filter((s) => !s.deterministico_igual).map((s) => [s.subcorrida, s.primeira_diferenca]))}`);
console.log(`subcorridas com o determinístico diferente: ${divergencias}`);
