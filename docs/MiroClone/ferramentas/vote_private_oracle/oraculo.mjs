// Oráculo Scrumlr — votação anónima e modo privado. Corre os casos de casos.json contra um
// scrumlr-server em 127.0.0.1 (start.sh) e grava, por caso, a linha do tempo de pedidos HTTP e de
// TODAS as mensagens que cada participante RECEBE no WebSocket.
//
//   node oraculo.mjs --corrida 1 --out saidas/scrumlr [--so v01,p01]
//
// Cada participante é uma sessão anónima própria (POST /login/anonymous → cookie jwt).
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const AQUI = dirname(fileURLToPath(import.meta.url));
const BASE = process.env.SCRUMLR_URL ?? "http://127.0.0.1:58080";
const args = Object.fromEntries(process.argv.slice(2).reduce((a, x, i, v) => (x.startsWith("--") && a.push([x.slice(2), v[i + 1]]), a), []));
const CORRIDA = Number(args.corrida ?? 1);
const OUT = args.out ?? join(AQUI, "saidas/scrumlr");
const SO = args.so ? new Set(args.so.split(",")) : null;
const CASOS = JSON.parse(readFileSync(join(AQUI, "casos.json"), "utf8"));
const META_FIXA = JSON.parse(readFileSync(join(AQUI, "meta_scrumlr.json"), "utf8"));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

class Caso {
  constructor(id) { this.id = id; this.t0 = Date.now(); this.linha = []; this.passo = 0; this.parts = {}; this.board = null; this.cols = {}; this.notas = {}; }
  t() { return Date.now() - this.t0; }
  marca(texto) { this.passo++; this.linha.push({ t: this.t(), passo: this.passo, tipo: "passo", texto }); }
  nota(texto) { this.linha.push({ t: this.t(), passo: this.passo, tipo: "observacao", texto }); }

  async login(nome) {
    const r = await fetch(BASE + "/login/anonymous", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ name: nome }) });
    const cookie = r.headers.getSetCookie().map((x) => x.split(";")[0]).join("; ");
    const user = await r.json();
    const p = { nome, cookie, id: user.id, ws: null, recebido: [] };
    this.parts[nome] = p;
    this.linha.push({ t: this.t(), passo: this.passo, tipo: "http", quem: nome, metodo: "POST", caminho: "/login/anonymous", corpo: { name: nome }, status: r.status, resposta: user });
    return p;
  }

  async api(quem, metodo, caminho, corpo) {
    const p = this.parts[quem];
    const r = await fetch(BASE + caminho, { method: metodo, headers: { "content-type": "application/json", cookie: p.cookie }, body: corpo === undefined ? undefined : JSON.stringify(corpo) });
    const txt = await r.text();
    let j; try { j = txt === "" ? null : JSON.parse(txt); } catch { j = txt; }
    this.linha.push({ t: this.t(), passo: this.passo, tipo: "http", quem, metodo, caminho, corpo: corpo ?? null, status: r.status, resposta: j });
    return { status: r.status, body: j };
  }

  async abreWs(quem) {
    const p = this.parts[quem];
    const ws = new WebSocket(BASE.replace(/^http/, "ws") + `/boards/${this.board}`, { headers: { cookie: p.cookie } });
    p.ws = ws;
    let init;
    const got = new Promise((res) => (init = res));
    ws.onmessage = (e) => {
      let m; try { m = JSON.parse(String(e.data)); } catch { m = String(e.data); }
      const ent = { t: this.t(), passo: this.passo, tipo: "ws", quem, msg: m };
      this.linha.push(ent); p.recebido.push(ent);
      if (m?.type === "INIT") init();
    };
    ws.onclose = (e) => this.linha.push({ t: this.t(), passo: this.passo, tipo: "ws-fechado", quem, code: e.code, reason: e.reason });
    ws.onerror = () => this.linha.push({ t: this.t(), passo: this.passo, tipo: "ws-erro", quem });
    await Promise.race([got, sleep(5000)]);
  }

  async fechaWs(quem) {
    const p = this.parts[quem];
    if (!p.ws) return;
    const ws = p.ws; p.ws = null;
    const fechou = new Promise((r) => ws.addEventListener("close", r));
    ws.close(1000, "sai");
    await Promise.race([fechou, sleep(3000)]);
  }

  // Espera até 250 ms sem mensagens novas (máx. 4 s): as mensagens de um passo ficam no passo dele.
  async assenta(silencio = 250, max = 4000) {
    const ini = Date.now(); let n = this.linha.length, ult = Date.now();
    while (Date.now() - ini < max) {
      await sleep(50);
      if (this.linha.length !== n) { n = this.linha.length; ult = Date.now(); }
      else if (Date.now() - ult >= silencio) break;
    }
  }

  // Quadro novo: M cria (2 colunas), A e B entram e abrem o WebSocket.
  async monta({ quem = ["A", "B"], colunasExtra = [] } = {}) {
    this.marca("montagem: M cria o quadro; " + quem.join(", ") + " entram");
    await this.login("M");
    const colunas = CASOS.colunas.map((c, i) => ({ name: c.nome, color: c.cor, visible: true, index: i })).concat(colunasExtra);
    const b = await this.api("M", "POST", "/boards", { name: "oraculo " + this.id, accessPolicy: "PUBLIC", columns: colunas });
    this.board = b.body.id;
    const cols = (await this.api("M", "GET", `/boards/${this.board}/columns`)).body;
    for (const c of CASOS.colunas) this.cols[c.chave] = cols.find((x) => x.name === c.nome).id;
    for (const c of cols) if (!Object.values(this.cols).includes(c.id)) this.cols[c.name] = c.id;
    await this.abreWs("M");
    for (const q of quem) await this.entra(q);
    await this.assenta();
  }

  async entra(q) {
    if (!this.parts[q]) await this.login(q);
    await this.api(q, "POST", `/boards/${this.board}/participants`, {});
    await this.abreWs(q);
  }

  async escreve(chave) {
    const n = CASOS.notas.find((x) => x.chave === chave);
    const r = await this.api(n.autor, "POST", `/boards/${this.board}/notes`, { column: this.cols[n.coluna], text: n.texto });
    if (r.body?.id) this.notas[chave] = r.body.id;
    await this.assenta();
    return r;
  }

  async board_(corpo) { const r = await this.api("M", "PUT", `/boards/${this.board}`, corpo); await this.assenta(); return r; }
  async abreVotacao(cfg) { const r = await this.api("M", "POST", `/boards/${this.board}/votings`, cfg); this.voting = r.body?.id; await this.assenta(); return r; }
  async vota(quem, chave) { const r = await this.api(quem, "POST", `/boards/${this.board}/votes`, { note: this.notas[chave] }); await this.assenta(); return r; }
  async desvota(quem, chave) { const r = await this.api(quem, "DELETE", `/boards/${this.board}/votes`, { note: this.notas[chave] }); await this.assenta(); return r; }
  async fechaVotacao(status = "CLOSED") { const r = await this.api("M", "PUT", `/boards/${this.board}/votings/${this.voting}`, { status }); await this.assenta(); return r; }
  async olha(quem, oque) { return this.api(quem, "GET", `/boards/${this.board}/${oque}`); }

  async termina() {
    for (const q of Object.keys(this.parts)) await this.fechaWs(q);
    const ids = { BOARD: this.board, voting: this.voting ?? null, colunas: this.cols, notas: this.notas, users: Object.fromEntries(Object.entries(this.parts).map(([k, p]) => [k, p.id])) };
    const recebido = Object.fromEntries(Object.entries(this.parts).map(([k, p]) => [k, p.recebido.map(({ t, passo, msg }) => ({ t, passo, msg }))]));
    const meta = { ...META_FIXA, caso: this.id, descricao: CASOS.casos[this.id], data: new Date().toISOString(), corrida: CORRIDA, servidor: BASE };
    const out = { meta, ids, linha_do_tempo: this.linha, recebido };
    mkdirSync(OUT, { recursive: true });
    writeFileSync(join(OUT, `${this.id}.json`), JSON.stringify(out, null, 1) + "\n");
    console.log(`${this.id}: ${this.linha.length} entradas, ws M/A/B/C = ${["M", "A", "B", "C"].map((q) => this.parts[q]?.recebido.length ?? "-").join("/")}`);
  }
}

const VB = CASOS.votacao_base;
const todasNotas = async (c) => { for (const n of CASOS.notas) await c.escreve(n.chave); };

const casos = {
  async v01(c, cfg = VB) {
    await c.monta(); await todasNotas(c);
    c.marca("M abre a votação " + JSON.stringify(cfg)); await c.abreVotacao(cfg);
    c.marca("A vota em n3"); await c.vota("A", "n3");
    c.marca("B vota em n1"); await c.vota("B", "n1");
    c.marca("cada um pede GET /votes e GET /votings"); for (const q of ["M", "A", "B"]) { await c.olha(q, "votes"); await c.olha(q, "votings"); }
    c.marca("B pede GET /votes filtrado (?voting= e ?note=n3)"); await c.olha("B", `votes?voting=${c.voting}`); await c.olha("B", `votes?note=${c.notas.n3}`);
    c.marca("M fecha"); await c.fechaVotacao();
  },
  async v01b(c) { await casos.v01(c, { ...VB, showVotesOfOthers: true }); },
  async v02(c) {
    await c.monta(); await todasNotas(c);
    c.marca("M abre a votação"); await c.abreVotacao(VB);
    c.marca("A vota em n3"); await c.vota("A", "n3");
    c.marca("C entra a meio (login + join + WebSocket)"); await c.entra("C"); await c.assenta();
    c.marca("C vota em n1"); await c.vota("C", "n1");
    c.marca("C pede GET /votes"); await c.olha("C", "votes");
    c.marca("M fecha"); await c.fechaVotacao();
  },
  async v03(c, saida = "ws") {
    await c.monta(); await todasNotas(c);
    c.marca("M abre a votação"); await c.abreVotacao(VB);
    c.marca("A vota em n3 duas vezes e em n1"); await c.vota("A", "n3"); await c.vota("A", "n3"); await c.vota("A", "n1");
    c.marca("B vota em n1"); await c.vota("B", "n1");
    if (saida === "ws") { c.marca("A fecha o WebSocket"); await c.fechaWs("A"); await c.assenta(); }
    if (saida === "conta") { c.marca("A apaga a própria conta (DELETE /users/{id})"); await c.api("A", "DELETE", `/users/${c.parts.A.id}`); await c.fechaWs("A"); await c.assenta(); }
    if (saida === "banido") { c.marca("M bane A (PUT participants/{A} {banned:true})"); await c.api("M", "PUT", `/boards/${c.board}/participants/${c.parts.A.id}`, { banned: true }); await c.assenta(); }
    if (saida === "expulso") { c.marca("M expulsa A (DELETE participants/{A})"); await c.api("M", "DELETE", `/boards/${c.board}/participants/${c.parts.A.id}`); await c.assenta(); }
    c.marca("M fecha a votação"); await c.fechaVotacao();
    c.marca("M e B pedem GET /votes e GET /votings/{id}"); for (const q of ["M", "B"]) { await c.olha(q, "votes"); await c.olha(q, `votings/${c.voting}`); }
  },
  async v03b(c) { await casos.v03(c, "conta"); },
  async v03c(c) { await casos.v03(c, "expulso"); },
  async v03d(c) { await casos.v03(c, "banido"); },
  async v04(c, status = "CLOSED") {
    await c.monta(); await todasNotas(c);
    c.marca("M abre a votação"); await c.abreVotacao(VB);
    c.marca("A vota n3, n3; B vota n1, n3; M vota n4"); await c.vota("A", "n3"); await c.vota("A", "n3"); await c.vota("B", "n1"); await c.vota("B", "n3"); await c.vota("M", "n4");
    c.marca(`M termina a votação com status ${status}`); await c.fechaVotacao(status);
    c.marca("B pede GET /votes, GET /votings/{id}, GET /notes"); await c.olha("B", "votes"); await c.olha("B", `votings/${c.voting}`); await c.olha("B", "notes");
    c.marca("B pede GET /boards/{id} (showVoting depois de fechar)"); await c.api("B", "GET", `/boards/${c.board}`);
    if (status === "CLOSED") { c.marca("M põe board.showVoting = votação (mostrar resultado)"); await c.board_({ showVoting: c.voting }); c.marca("B pede GET /boards/{id} depois do PUT"); await c.api("B", "GET", `/boards/${c.board}`); }
    c.marca("M abre uma 2.ª votação (a 1.ª ainda conta?)"); await c.abreVotacao(VB);
    c.marca("B pede GET /votings"); await c.olha("B", "votings");
  },
  async v04b(c) { await casos.v04(c, "ABORTED"); },
  async v05(c) {
    await c.monta(); await todasNotas(c);
    c.marca("M abre a votação"); await c.abreVotacao(VB);
    c.marca("A vota em n3"); await c.vota("A", "n3");
    const ini = new Date(), fim = new Date(Date.now() + 3000);
    c.marca("M põe o relógio (PUT board timerStart/timerEnd = +3 s)"); await c.board_({ timerStart: ini.toISOString(), timerEnd: fim.toISOString() });
    c.marca("espera 6 s (o relógio acaba aos 3 s)"); await sleep(6000); await c.assenta();
    c.marca("A tenta votar depois do relógio acabar"); await c.vota("A", "n1");
    c.marca("M pede GET /votings/{id}"); await c.olha("M", `votings/${c.voting}`);
  },
  async v05b(c) {
    await c.monta(); await todasNotas(c);
    c.marca("M abre a votação"); await c.abreVotacao(VB);
    c.marca("A vota em n3"); await c.vota("A", "n3");
    c.marca("M liga o cronómetro (POST /timer minutes=1)"); await c.api("M", "POST", `/boards/${c.board}/timer`, { minutes: 1 }); await c.assenta();
    c.marca("espera 65 s"); await sleep(65000); await c.assenta();
    c.marca("A tenta votar depois do cronómetro acabar"); await c.vota("A", "n1");
    c.marca("M pede GET /votings/{id}"); await c.olha("M", `votings/${c.voting}`);
  },
  async v06(c) {
    await c.monta(); await todasNotas(c);
    c.marca("votação 1: voteLimit=2, allowMultipleVotes=false"); await c.abreVotacao({ ...VB, voteLimit: 2, allowMultipleVotes: false });
    c.marca("A vota n1"); await c.vota("A", "n1");
    c.marca("A vota n1 de novo (repetir)"); await c.vota("A", "n1");
    c.marca("A vota n2"); await c.vota("A", "n2");
    c.marca("A vota n3 (3.º voto, acima do limite)"); await c.vota("A", "n3");
    c.marca("A retira o voto de n1"); await c.desvota("A", "n1");
    c.marca("A vota n3 (agora há folga)"); await c.vota("A", "n3");
    c.marca("A retira um voto de n4 (onde não votou)"); await c.desvota("A", "n4");
    c.marca("M fecha"); await c.fechaVotacao();
    c.marca("votação 2: voteLimit=2, allowMultipleVotes=true"); await c.abreVotacao({ ...VB, voteLimit: 2, allowMultipleVotes: true });
    c.marca("A vota n5 três vezes"); await c.vota("A", "n5"); await c.vota("A", "n5"); await c.vota("A", "n5");
    c.marca("A vota quando NÃO há votação aberta (depois de fechar)"); await c.fechaVotacao(); await c.vota("A", "n1");
    c.marca("M pede GET /votings"); await c.olha("M", "votings");
  },
  async v07(c, isAnonymous = true) {
    await c.monta(); await todasNotas(c);
    c.marca(`M abre a votação isAnonymous=${isAnonymous} showVotesOfOthers=false`); await c.abreVotacao({ ...VB, isAnonymous });
    c.marca("A vota n3; B vota n1, n3"); await c.vota("A", "n3"); await c.vota("B", "n1"); await c.vota("B", "n3");
    c.marca("M fecha"); await c.fechaVotacao();
    c.marca("M, A e B pedem GET /votes e GET /votings/{id}"); for (const q of ["M", "A", "B"]) { await c.olha(q, "votes"); await c.olha(q, `votings/${c.voting}`); }
    c.marca("M exporta o quadro (GET /export)"); await c.api("M", "GET", `/boards/${c.board}/export`);
  },
  async v07b(c) { await casos.v07(c, false); },
  async v08(c) {
    await c.monta();
    c.marca("M esconde as notas dos outros"); await c.board_({ showNotesOfOtherUsers: false });
    c.marca("A escreve n1; B escreve n3"); await c.escreve("n1"); await c.escreve("n3");
    c.marca("M abre a votação"); await c.abreVotacao(VB);
    c.marca("B vota na nota escondida de A (n1, id conhecido por fora)"); await c.vota("B", "n1");
    c.marca("B pede GET /votes"); await c.olha("B", "votes");
    c.marca("M revela"); await c.board_({ showNotesOfOtherUsers: true });
    c.marca("M fecha"); await c.fechaVotacao();
  },
  async p01(c) {
    await c.monta();
    c.marca("M esconde as notas dos outros (showNotesOfOtherUsers=false)"); await c.board_({ showNotesOfOtherUsers: false });
    c.marca("A escreve n1"); await c.escreve("n1");
    c.marca("A escreve n2"); await c.escreve("n2");
    c.marca("B escreve n3"); await c.escreve("n3");
    c.marca("B pede GET /notes e GET /notes/{n1}"); await c.olha("B", "notes"); await c.olha("B", `notes/${c.notas.n1}`);
    c.marca("A pede GET /notes"); await c.olha("A", "notes");
    c.marca("C entra com as notas escondidas: que INIT recebe?"); await c.entra("C"); await c.assenta();
  },
  async p02(c) {
    await c.monta();
    c.marca("M esconde"); await c.board_({ showNotesOfOtherUsers: false });
    c.marca("A escreve n1"); await c.escreve("n1");
    c.marca("M escreve n4 (nota do moderador em modo privado)"); await c.escreve("n4");
    c.marca("M pede GET /notes; B pede GET /notes"); await c.olha("M", "notes"); await c.olha("B", "notes");
    c.marca("M promove B a MODERATOR"); await c.api("M", "PUT", `/boards/${c.board}/participants/${c.parts.B.id}`, { role: "MODERATOR" }); await c.assenta();
    c.marca("B (agora moderador) pede GET /notes"); await c.olha("B", "notes");
    c.marca("A escreve n2 (B já é moderador)"); await c.escreve("n2");
  },
  async p03(c, reesconde = false) {
    await c.monta();
    c.marca("M esconde"); await c.board_({ showNotesOfOtherUsers: false });
    c.marca("A escreve n1, n2; B escreve n3"); await c.escreve("n1"); await c.escreve("n2"); await c.escreve("n3");
    c.marca("M revela (showNotesOfOtherUsers=true)"); await c.board_({ showNotesOfOtherUsers: true });
    c.marca("B pede GET /notes"); await c.olha("B", "notes");
    if (reesconde) {
      c.marca("M esconde de novo"); await c.board_({ showNotesOfOtherUsers: false });
      c.marca("B pede GET /notes"); await c.olha("B", "notes");
      c.marca("A edita n1 com as notas escondidas de novo"); await c.api("A", "PUT", `/boards/${c.board}/notes/${c.notas.n1}`, { text: "café na sala (editado)" }); await c.assenta();
    }
  },
  async p04(c) { await casos.p03(c, true); },
  async p05(c) {
    await c.monta();
    c.marca("M esconde"); await c.board_({ showNotesOfOtherUsers: false });
    c.marca("A escreve n1, n2"); await c.escreve("n1"); await c.escreve("n2");
    c.marca("A edita n1"); await c.api("A", "PUT", `/boards/${c.board}/notes/${c.notas.n1}`, { text: "café na sala, mas melhor" }); await c.assenta();
    c.marca("A move n1 para a coluna c2"); await c.api("A", "PUT", `/boards/${c.board}/notes/${c.notas.n1}`, { position: { column: c.cols.c2, rank: 0, stack: null } }); await c.assenta();
    c.marca("A apaga n2 (corpo {deleteStack:false})"); await c.api("A", "DELETE", `/boards/${c.board}/notes/${c.notas.n2}`, { deleteStack: false }); await c.assenta();
    c.marca("A tenta reler n2 depois de apagada (há restaurar?)"); await c.olha("A", `notes/${c.notas.n2}`);
    c.marca("B tenta editar a nota escondida de A (n1)"); await c.api("B", "PUT", `/boards/${c.board}/notes/${c.notas.n1}`, { text: "B mexeu" }); await c.assenta();
    c.marca("B tenta apagar a nota escondida de A (n1)"); await c.api("B", "DELETE", `/boards/${c.board}/notes/${c.notas.n1}`, { deleteStack: false }); await c.assenta();
    c.marca("M revela"); await c.board_({ showNotesOfOtherUsers: true });
    c.nota("desfazer: a API (swagger) não tem rota de undo/redo nem de restaurar nota apagada.");
  },
  async p06(c) {
    await c.monta();
    c.marca("M esconde o autor (showAuthors=false), notas visíveis"); await c.board_({ showAuthors: false });
    c.marca("A escreve n1"); await c.escreve("n1");
    c.marca("B escreve n3"); await c.escreve("n3");
    c.marca("B e M pedem GET /notes"); await c.olha("B", "notes"); await c.olha("M", "notes");
    c.marca("B pede GET /participants e GET /users/board/{id}"); await c.olha("B", "participants"); await c.api("B", "GET", `/users/board/${c.board}`);
    c.marca("M mostra o autor (showAuthors=true)"); await c.board_({ showAuthors: true });
  },
  async p07(c) {
    await c.monta();
    c.marca("M esconde"); await c.board_({ showNotesOfOtherUsers: false });
    c.marca("A escreve n1"); await c.escreve("n1");
    c.marca("M partilha (sharedNote) a nota escondida n1"); await c.board_({ sharedNote: c.notas.n1 });
    c.marca("B pede GET /notes/{n1}"); await c.olha("B", `notes/${c.notas.n1}`);
    c.marca("M deixa de partilhar (PUT {sharedNote:null})"); await c.board_({ sharedNote: null });
    c.marca("B pede GET /boards/{id}"); await c.api("B", "GET", `/boards/${c.board}`);
    c.marca("controlo: novo quadro escondido, M partilha e desfaz com PUT {sharedNote:null, showNotesOfOtherUsers:false}"); await c.board_({ showNotesOfOtherUsers: false }); await c.board_({ sharedNote: c.notas.n1 }); await c.board_({ sharedNote: null, showNotesOfOtherUsers: false });
  },
  async p08(c) {
    await c.monta({ colunasExtra: [{ name: "Escondida", color: "planning-pink", visible: false, index: 2 }] });
    const h = c.cols["Escondida"];
    c.marca("M escreve na coluna escondida"); { const r = await c.api("M", "POST", `/boards/${c.board}/notes`, { column: h, text: "plano secreto do moderador" }); c.notas.h1 = r.body?.id; await c.assenta(); }
    c.marca("A tenta escrever na coluna escondida"); { const r = await c.api("A", "POST", `/boards/${c.board}/notes`, { column: h, text: "A na escondida" }); c.notas.h2 = r.body?.id; await c.assenta(); }
    c.marca("B pede GET /notes e GET /columns"); await c.olha("B", "notes"); await c.olha("B", "columns");
    c.marca("M revela a coluna (visible=true)"); await c.api("M", "PUT", `/boards/${c.board}/columns/${h}`, { name: "Escondida", color: "planning-pink", visible: true, index: 2, description: "" }); await c.assenta();
    c.marca("B pede GET /notes"); await c.olha("B", "notes");
  },
};

const ordem = Object.keys(casos).filter((k) => !SO || SO.has(k));
for (const id of ordem) {
  const c = new Caso(id);
  try { await casos[id](c); } catch (e) { c.nota("ERRO do script: " + (e?.stack ?? e)); console.error(id, e); }
  await c.termina();
}
process.exit(0);
