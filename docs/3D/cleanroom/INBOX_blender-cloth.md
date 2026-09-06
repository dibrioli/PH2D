# INBOX — canal cego do Implementador para o ledger `blender-cloth`

> O Implementador **só acrescenta** (`cat >>`), nunca lê. Um subagente E/R transcreve para o
> `LEDGER_blender-cloth.md`. Formato livre: data · session-id · o que aconteceu.

## Declaração da janela I (2026-09-05, sessão ph2d-d8 / 1246816c)

Nenhum conteúdo do fonte do alvo entrou no CONTEXTO desta janela: o fonte foi lido
apenas pelos subagentes E e R-pré; esta janela leu a espec só depois do atestado do
R-pré no cabeçalho; dos ficheiros do scratchpad quarentenados (INC-1) leu apenas a
listagem de nomes, nunca o conteúdo; o `.claude/settings.local.json` da worktree nega
`Read` aos dois checkouts GPL desde 2026-09-05.

## Medições do I contra as 46 fixtures (2026-09-06) — para o E emendar e o R atestar

Arnês: `crates/ph2d-cloth/tests/oraculo_do_pincel.rs` (a lei nossa em `ph2d-cloth/src/verlet*.rs`).

- ✅ Os SEIS traços de um passo de força dão erro **0,0000** por vértice — §4.1/§4.2/§5.4 ao bit.
- ⚠️ **Anel-1:** com o anel sobre os QUADS, `plano_arrastar_radial_global` bate a 1 % e o `_local` sai
  **2×**; com o anel sobre a grelha TRIANGULADA (diagonal 1.º→3.º canto), o `_local` bate
  (`0,35` vs `0,33` no centro) e o `_global` cai para `0,38` (oráculo `0,59`). ⇒ ou o anel é o da
  triangulação e há OUTRA diferença Local/Dinâmica por explicar, ou vice-versa. Pergunta 1 ao E.
- ⚠️ **Local vs Dinâmica:** no oráculo o Local é `0,35–0,57×` o Dinâmica **uniformemente** ao longo
  do traço; na espec tal como está os dois deviam ser quase iguais para este traço, e na nossa lei
  são (`0,35`/`0,34`). Pergunta 3 ao E.
- ⚠️ **A esfera move 6 050/6 050 no Local**, e a bola de `3,5R = 1,225` cobre ~37 % de uma esfera
  unitária ⇒ no alvo, vértices além da área movem-se. No plano o Local move exactamente o disco de
  `3,5R` (2 144). Perguntas 2 e 4 ao E (φ sem banda? parede = célula inactiva? tamanho da folha?).
- Medido e descartado: a ORDEM de resolução (cinco ordens: `0,55–0,64` no centro ⇒ é a barra do
  gate 15); `20`/`50` varreduras (matam o movimento — `5` é o certo); só arestas sem pares (mole
  demais); escalar o alcance da banda de φ e da retenção (nenhuma escala dá Local baixo E Dinâmica
  alto); φ sem banda (não muda o padrão).
- ⚠️ **Âncoras:** `gancho_1passo` dá `0,378` contra `0,489` e `agarrar_1passo` `0,098` contra `0,134`
  — a correcção da âncora parece maior do que `Δ/2`. Pergunta 5 ao E.
- Bug meu já curado: a massa entrava duas vezes (`massa2_1passo` lia metade); hoje ao bit.
