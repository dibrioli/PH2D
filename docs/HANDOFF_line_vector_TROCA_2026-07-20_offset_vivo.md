# HANDOFF — troca de agente na `line/Vector` (2026-07-20): Offset AO VIVO, o bug aberto e a fila

> **Você assumiu esta linha pelo bloco do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).**
> FASE 0 primeiro: `cd Worktrees/line-Vector && pwd && git branch --show-current` —
> a janela abre na raiz (= `main`) e os MESMOS paths existem nas duas árvores.
>
> Worktree: `Worktrees/line-Vector` · branch `line/Vector` · HEAD `c4b371fe` · árvore limpa.
> Modo L: **você NÃO integra nem pusha** — fecha, escreve handoff de integração e PARA
> (CLAUDE.md §0.7). Commits: `git commit --no-verify -F <arquivo>` terminando em
> `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

---

## §1 — A TAREFA Nº 1: o bug ABERTO do Offset ao vivo

Report do Enio (2026-07-20, o último, **depois** de `c4b371fe`):

> *"mesmos problemas: queda de fps, muda em tempo real para round mas não muda para
> Miter e Bevel em tempo real"*

⚠️ **O meu harness NÃO contém o fenômeno dele** — essa é a informação mais importante
deste handoff. O nível 18 (`PH2D_BUILD_SMOKE=18`, ver §3) dirige o app REAL pelo input
real e mostra tudo VERDE: arrasto com Round a 11–20 ms/frame (debug), retunes
Round→Bevel→Miter derrubando verts 302→54→26 na hora, janela de retune viva, um passo
de undo por retune. E o Enio vê o oposto. **Não re-rode o harness que passa e declare
consertado** ([[feedback_harness_reproduces_mechanism_not_context]],
[[feedback_nonreproduction_is_not_proof_of_fix]]) — **estenda o harness até ele
reproduzir o report**, e só então conserte.

Diferenças candidatas entre o harness e o uso real (por onde começar):

1. **A assimetria "para Round funciona, para Miter/Bevel não" cheira a 1º-clique-funciona,
   seguintes-não** ⇒ a **janela de retune morrendo depois do 1º retune**. O oráculo de
   morte é a profundidade do undo (`OffsetRetune::step`): se QUALQUER passo landa entre os
   cliques dele, a janela morre em silêncio e os chips voltam a só armar o próximo arrasto.
   O harness clica em cadência limpa de 20 frames e nada intervém; o fluxo real dele pode
   estar registrando um passo que o harness não gera (um passo espúrio pós-retune? um
   clique dele em outro widget? o `capture_project` vendo algo não-convergido na CENA
   DELE, que tem mais objetos?). **Instrumento pronto: `PH2D_UNDO_LOG=1`** imprime cada
   passo com o diff (world/vec/flip + ids). Peça ao Enio a sequência exata OU imite-a no
   nível 18: **arrastar com Miter → clicar Round → clicar Bevel** (o harness atual clica
   Round ANTES do arrasto; a ordem dele é outra e a ordem importa).
2. **A queda de FPS persiste para ele** apesar do `flat_lines` (medido: motor 19–43 ms →
   4,6–9,8 ms; arrasto 24–82 → 11–20 ms/frame, debug, janela ~800×900 do smoke). Candidatos:
   `d` grande (o slider vai a ±4; o harness só chegou a ~1 — meça o custo a d=4), janela
   maximizada (custo de render), a cena real dele (mais formas na seleção?), ou algo fora
   do motor (o `post_frame_undo` captura o projeto TODO frame com input — no drag o
   `held_button` suprime, mas confira NA CENA DELE). Meça antes de mexer
   ([[feedback_measure_perf_symptom_scale]]).
3. **Round↔Bevel é visualmente sutil** (os dois cortam o bico no MESMO recuo `d`; arco vs
   chanfro). Já expliquei isso ao Enio e ele re-reportou "não muda" — trate como bug real
   até prova em contrário, mas confirme com VERTS (o oráculo que não mente), nunca com área.

## §2 — O que JÁ foi consertado nesta janela (não re-derive, não re-litigue)

Commits, do mais novo ao mais velho — cada um com gates mutation-tested:

- **`c4b371fe`** — `flat_lines` no motor (`ph2d-vec-boolean/src/expand.rs::loop_region`):
  tudo que entra num sweep do offset é achatado em RETAS na tolerância RELATIVA da forma.
  Causa medida: a quina Round produz banda de ARCOS e o sweep sobre cúbicas custava
  19–43 ms/offset (~82 ms/frame no arrasto, ~12 fps). Gate de RAZÃO
  `a_round_live_offset_costs_like_the_other_joins` (Round/Bevel < 15×; a mutação que
  reverte o flatten sangra com 30,3×).
- **`8c92bf46`** — a **janela de RETUNE** (`OffsetRetune` em `shells/desktop/src/vec_expand.rs`):
  o release do slider vira a sessão numa janela (cena do grab + poses congeladas + `d`
  comitado); trocar Join/Side re-offseta ao MESMO `d`. Morre quando o undo ANDA (qualquer
  direção) ou no próximo grab. ⚠️ O `apply` **reseta as entidades do resultado à IDENTIDADE**
  antes do preview — sem isso a pose DOBRA (ver §4.1). Arch-gate do sítio em
  `tests/the_live_offset_preview_is_a_gesture_to_the_settle.rs` (os unit gates espelham o
  frame e NÃO veem a render_loop — provado por mutação).
- **`9c0446df`** — o preview vivo é **GESTO** (o `settle_origins` o pula via lista `drawing`)
  + os 3 destinos da fonte não-consumida (zona morta pré-churn · cópia-mundo em d≈0
  pós-churn · aniquilada some). Consertou o "pula pro canto direito" (transform dobrado).
- **`e8339102`** — EvenOdd swap (contorno cruzando TROCA de papel em vez de sumir — pedido
  explícito do Enio) + poses congeladas na sessão (não pula pra origem).
- **`aedc0f3a`/`3f24e175`/`594cb1d0`** — o Offset ao vivo em si, o seletor **Side**
  (Outer/Inner/Both, modelo B: cada contorno pra fora) e o Power Stroke de fita (liso,
  **aprovado no smoke**).

**Panic**: os reports antigos de panic no offset ao vivo **cessaram** depois de `9c0446df`
(a causa era o estado dobrado). O motor foi varrido fino (27k sweeps, zero panic/NaN —
sonda `crates/ph2d-vec-boolean/tests/probe_offset_fine_sweep.rs`, `--ignored`). Se voltar,
peça o backtrace (`RUST_BACKTRACE=1`).

## §3 — As ferramentas (USE-AS, elas acharam tudo até aqui)

- **`PH2D_BUILD_SMOKE=17`** — a cena manual do Expand (zig-zag/estrela/rosquinha/arco).
  Rodar: `cd Worktrees/line-Vector && PH2D_BUILD_SMOKE=17 cargo run -p ph2d-host-desktop`.
- **`PH2D_BUILD_SMOKE=18`** — **o harness AUTO-DIRIGIDO** (`build_smoke_expand.rs::
  smoke_expand_retune_drive` + primitivos em `build_smoke_drive.rs`): rola o painel, clica
  Round, agarra o slider com o PONTEIRO, arrasta por frames, solta, clica Bevel e Miter —
  logando por frame `dt` (o FPS), profundidade do undo, janela viva, join do painel e
  VERTS da cena. **Modifique o roteiro para a sequência do Enio** — é ali que o bug dele
  tem de aparecer primeiro. Roda sozinho: `PH2D_BUILD_SMOKE=18 timeout 30 cargo run -p
  ph2d-host-desktop 2>&1 | grep retune-smoke`.
- **`PH2D_UNDO_LOG=1`** — cada passo de undo com o diff que o causou (é como se pega a
  morte da janela de retune e passo espúrio).

## §4 — Armadilhas PAGAS desta linha (custaram smoke/bug cada uma)

1. **O `clone_from(&pre)` restaura o CONTADOR de ids** ⇒ o resultado do preview renasce com
   o MESMO id todo frame ⇒ o `sync` mantém a MESMA entidade ⇒ estado por-entidade
   (assentamento!) vaza entre frames. Mundo × centro = pose DOBRADA. Toda re-inserção de
   geometria de mundo sob id reusado exige entidade na IDENTIDADE
   ([[feedback (memória) a-restored-snapshot-resurrects-its-id-counter]]).
2. **A ÁREA é oráculo CEGO a Side=Both**: o arredondamento perde `(4−π)d²` na borda externa
   e ganha o MESMO no furo — cancela EXATO. Use VERTS ou amostra de canto.
3. **Gate espelho não vê a render_loop**: os unit gates de vec_expand reencenam o frame
   (preview→sync→settle) — mutilar o wiring real deixa todos verdes. Todo fato de COSTURA
   tem arch-gate sobre o fonte (2 no arquivo `the_live_offset_preview_is_a_gesture...`).
4. **O estado dos chips do painel é thread-local** (`ph2d_panel_vector::expand_join/side`,
   Cells). No app é tudo main-thread — mas em TESTE cada thread tem o seu, e um teste que
   os altera deve restaurar (os setters são `pub` para os gates).
5. **`|d| < MIN_OFFSET` é identidade, não "sumiu"** — `MIN_OFFSET` é público no motor de
   propósito (porta única); o preview tem TRÊS destinos para fonte não-consumida (doc do
   `OffsetSession::preview`). O slider RECENTRA em 0 no release: todo grab começa na zona
   morta.
6. **O `settle_origins` só assenta entidade na IDENTIDADE** — e o gate
   `settle_skips_every_derived_geometry` tem exceção NOMEADA para `vec_expand.rs` (o
   retune força identidade PARA ser re-assentado — o oposto dos hosts `*_live`).

## §5 — A FILA de implementações (depois do bug)

Pendências de smoke/decisão do Offset (perguntas ao Enio, já formuladas):
- Faixa do slider (−4..+4) é ergonômica? · **Both = modelo B** (cada contorno pra fora,
  join visível no furo) — confirmar que é a semântica desejada · o botão **Offset Path**
  (caminho numérico) ficou redundante com o slider ao vivo?

A fila grande da linha (CLAUDE.md §5 "Vector Module", handoffs
`HANDOFF_line_vector_continuacao_2026-07-16.md` / `_2026-07-13c.md`):
- **Live Path Effects como NÓS** (o multiplicador; a costura fonte≠cozido do ADR-0121 é o
  pré-requisito e JÁ existe) · tipos de quina (chamfer é quase de graça) · texto em
  caminho · trim path · repeater · largura variável · mais primitivas · **morph vivo**
  (t animável — o desenho é o do conector; `steps()`/`morph(t)` do motor já servem) ·
  blend em CADEIA (>2 formas) · o lerp de coordenadas em rotação grande (Sederberg 1992 /
  Alexa 2000). Rig+skinning = deferido pro FIM de tudo.

## §6 — Para o INTEGRADOR (foundational tocado nesta janela)

- `ph2d-editor-core`: `WidgetStore::set_slider_value` (append) · ids
  `VECTOR_EXPAND_SIDE_OUTER/INNER/BOTH`.
- `ph2d-vec-scene`: enum `OffsetSide` (novo, exportado).
- `ph2d-vec-boolean`: `MIN_OFFSET` público; `flat_lines` (interno); `offset_path` ganhou
  o parâmetro `side`.
- `ph2d-panel-vector`: `set_expand_join/side` viraram `pub` (para gates).
- Shell: `App.vec_offset_session` + `App.vec_offset_retune`; drive block na
  `render_loop/mod.rs` (~linha 2737); o chain do `drawing` no `settle_origins` (~3670).

Handoffs de integração anteriores da linha: `docs/HANDOFF_line_vector_integracao_2026-07-18b.md`
e irmãos (12/13/14-07).
