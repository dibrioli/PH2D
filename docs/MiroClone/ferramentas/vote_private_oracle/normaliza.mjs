// Normalização para ler e comparar corridas: ids aleatórios → rótulos estáveis; carimbos de tempo → "<ts>".
//   · id de utilizador → M/A/B/C · quadro → BOARD · votação → VOT1, VOT2 (ordem de criação)
//   · nota → a chave de casos.json (n1…n5, h1, h2) · coluna → c1/c2/nome
//   · qualquer outro UUID → UUID#k pela ordem da 1.ª aparição no ficheiro
//   · ISO-8601 → "<ts>" (createdAt, lastModifiedAt, timerStart…)
const UUID = /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/g;
const ISO = /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})$/;

// `semIndice`: UUID desconhecido vira só "UUID" (a ordem da 1.ª aparição depende do entrelaçar das
// mensagens entre participantes, que varia de corrida para corrida).
export function rotulador(ids, linha = [], semIndice = false) {
  const m = new Map();
  if (ids.BOARD) m.set(ids.BOARD, "BOARD");
  for (const [k, v] of Object.entries(ids.users ?? {})) if (v) m.set(v, k);
  for (const [k, v] of Object.entries(ids.notas ?? {})) if (v) m.set(v, k);
  for (const [k, v] of Object.entries(ids.colunas ?? {})) if (v) m.set(v, "col:" + k);
  // votações pela ordem em que o POST /votings as devolveu
  let nv = 0;
  for (const e of linha) if (e.tipo === "http" && e.metodo === "POST" && /\/votings$/.test(e.caminho) && e.resposta?.id && !m.has(e.resposta.id)) m.set(e.resposta.id, "VOT" + ++nv);
  if (ids.voting && !m.has(ids.voting)) m.set(ids.voting, "VOT?");
  let k = 0;
  const lab = (u) => { if (!m.has(u)) { if (semIndice) return "UUID"; m.set(u, "UUID#" + ++k); } return m.get(u); };
  const rot = (x) => {
    if (typeof x === "string") return ISO.test(x) ? "<ts>" : x.replace(UUID, lab);
    if (Array.isArray(x)) return x.map(rot);
    if (x && typeof x === "object") return Object.fromEntries(Object.entries(x).map(([a, b]) => [a.replace(UUID, lab), rot(b)]));
    return x;
  };
  return rot;
}

// Ordem sem significado, medida entre as corridas 1 e 2: (a) as chaves de um objecto JSON saem em
// duas ordens (campos da struct ou alfabética, conforme o caminho que o servidor tomou para aquele
// participante, e isso varia entre corridas); (b) listas de objectos (notas, votos, participantes,
// colunas, votesPerNote) saem em ordem não estável — a ordem com sentido vive em position.rank/index,
// que se compara. ⇒ chaves ordenadas; arrays de OBJECTOS ordenados pelo seu JSON canónico.
export function ordena(x) {
  if (Array.isArray(x)) {
    const y = x.map(ordena);
    return y.every((e) => e && typeof e === "object") ? y.sort((p, q) => (JSON.stringify(p) < JSON.stringify(q) ? -1 : 1)) : y;
  }
  if (x && typeof x === "object") return Object.fromEntries(Object.keys(x).sort().map((k) => [k, ordena(x[k])]));
  return x;
}

// Forma canónica de uma corrida para comparar: por passo, cada pedido (sem tempo) e, por
// participante, o multiconjunto das mensagens WS recebidas (ordenadas dentro do passo).
export function canonico(d) {
  const r0 = rotulador(d.ids, d.linha_do_tempo, true);
  const rot = (x) => ordena(r0(x));
  const passos = [];
  let cur = null;
  for (const e of d.linha_do_tempo) {
    if (e.tipo === "passo" || !cur) { cur = { passo: e.passo, texto: e.texto ?? "", http: [], ws: {} }; passos.push(cur); if (e.tipo === "passo") continue; }
    if (e.tipo === "http") cur.http.push(rot({ quem: e.quem, metodo: e.metodo, caminho: e.caminho, corpo: e.corpo, status: e.status, resposta: e.resposta }));
    else if (e.tipo === "ws") (cur.ws[e.quem] ??= []).push(JSON.stringify(rot(e.msg)));
    else (cur.ws[e.quem ?? "_"] ??= []).push(JSON.stringify(rot({ tipo: e.tipo, code: e.code, texto: e.texto })));
  }
  for (const p of passos) for (const q of Object.keys(p.ws)) p.ws[q].sort();
  return passos;
}
