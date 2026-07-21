# HANDOFF — troca de agente na `line/Vector` (2026-07-20): Offset AO VIVO — o bug FECHADO e a fila

> **Você assumiu esta linha pelo bloco do
> [`MODELO_TROCA_DE_AGENTE_NA_LINHA.md`](IntegracaoMultiAgente/MODELO_TROCA_DE_AGENTE_NA_LINHA.md).**
> FASE 0 primeiro: `cd Worktrees/line-Vector && pwd && git branch --show-current` —
> a janela abre na raiz (= `main`) e os MESMOS paths existem nas duas árvores.
>
> Worktree: `Worktrees/line-Vector` · branch `line/Vector` · HEAD `6831b43d` · árvore limpa.
> Modo L: **você NÃO integra nem pusha** — fecha, escreve handoff de integração e PARA
> (CLAUDE.md §0.7). Commits: `git commit --no-verify -F <arquivo>`.

---

## §1 — O bug do Offset ao vivo: **FECHADO** (`6831b43d`, 2026-07-20) — leia antes de re-litigar

Report do Enio: *"mesmos problemas: queda de fps, muda em tempo real para round mas não
muda para Miter e Bevel em tempo real"*. O protocolo do handoff anterior foi seguido ao
pé da letra — **o harness foi estendido até conter o fluxo real** (ordem do Enio, cliques
com Down/Up em frames SEPARADOS, `d` até saturar, bbox na telemetria — verts é CEGO ao
Miter/Bevel) e a mecânica do retune saiu inocentada fim-a-fim **incluindo a TELA**
(screenshots das 3 fases via `spectacle`: arcos → chanfros → quinas retas). O que sobrou
era o BUG REAL, no **MOTOR**, no `d` extremo:

- **O FANTASMA (consertado):** com caneta `2|d|` maior que o próprio laço, o contorno
  interno da banda degenerava (winding-ruído) e o refugo atravessava o sweep. Medido no
  donut do smoke: encolher além da morte (`d=−3/−4`) devolvia NADA no Miter (correto) e
  uma **ILHA de 12 verts/área 2,52 no Round/Bevel** — não-monotônico e **diferente por
  join**, que é o report ao pé da letra (uns cliques "mudavam", outros não); crescer a
  `d=+4` inflava a área de 19,8 (exata) para 30,7 (furo-fantasma). Fix: **`drop_phantoms`
  na porta única `loop_region`** (discriminador = distância ao laço fonte, teste pelo
  MÁXIMO do contorno; caminho comum zero-custo). Gate
  `an_offset_past_the_shapes_death_leaves_no_phantom` (ausência + identidade do
  cancelamento + PRESENÇA do legítimo), 3 mutações, 3/3 sangram. Sonda `--ignored`:
  `probe_offset_extreme_d`.
- **A queda de FPS é o BUILD DEBUG — MEDIDO em `--release` (`6831b43d`):** o arrasto vivo
  custa **0.8–1.5 ms/frame em release** (60 fps+; os frames de 16.7 ms são vsync ocioso),
  e o motor no pior caso (Round, `d=4`, `--release`) é **1.5 ms** (Miter/Bevel ~0.1 ms).
  Em **debug** o mesmo arrasto é 8–26 ms — e o Enio smoke-testa em debug (`cargo run` sem
  `--release`, visto no terminal dele), então o "trava" é o build, não o produto. **Sonda:
  `probe_offset_cost_on_the_d_ladder` (`--release --ignored`).** A MESMA lição do áudio W7
  (*"`--release` não é preferência"*). **O smoke agora exige `--release`** (ver §3).
  - ⛔ **O MEMO DO PREVIEW FOI CONSTRUÍDO (`726c7723`) e REVERTIDO (`43a6f4d0`).** Ele
    memoizava `(d, knobs)` p/ pular o re-clone+re-offset em frames de mesmo `d` (held-still
    numa cena grande, onde o `clone_from` é O(cena toda)). O Enio reportou regressão:
    *"melhorou FPS mas regrediu: Round para Bevel ou Miter não muda mais"*. **Não consegui
    inocentar o memo** — o retune FUNCIONA com ele no teste de cadeia determinístico E no
    smoke (vários runs), e as falhas que vi foram a interferência do AMBIENTE (§1, WM ×
    cursor físico → passo de undo espúrio → janela de retune morre), pré-existente. Mas
    correção > otimização, e o ganho de FPS que o Enio viu **não era o memo** (o memo só
    ajuda held-still; o nível 18 é arrasto ATIVO, onde ele nunca dispara — veio do
    `flat_lines`/fix do fantasma, menos verts) ⇒ reverter não custou FPS. **Não
    reconstrua sem uma reprodução do que ele quebra.**
  - **FICOU o gate `a_chain_of_retunes_changes_the_shape_at_every_step`** (guarda
    PERMANENTE do retune — o antigo só provava UMA troca; este prova Round→Bevel→Miter,
    cada um mudando, e espelha o frame "aprende-depth" que o app real tem entre o `apply` e
    o clique seguinte). E o **log de `RetuneStep::Dead`** (a janela que fecha em silêncio
    agora avisa — é o instrumento pra diagnosticar se o "não muda" persistir).
  - **A janela de retune é FRÁGIL a passos de undo espúrios** (design de `8c92bf46`): ela
    morre no oráculo da profundidade do undo, e QUALQUER passo entre a aprendizagem e o
    clique a mata em silêncio. O `settle_origins` roda todo frame; se ele produzir um
    `Transform` levemente diferente (jitter de f64) a `WorldSnapshot` muda → passo espúrio
    → morte. **NÃO investiguei a fundo** — se o Enio confirmar "não muda" no build
    revertido, este é o próximo suspeito (o log de Dead dirá se é isso). Tornar a janela
    robusta (só morrer em edição REAL, não em re-assentamento) é wave própria.
  - **Decisão de produto pendente (faixa do slider):** ±4 unidades de MUNDO sobre ~115 px
    de track (~60 px além do centro satura; clique no track TELEPORTA para ±4). Racional
    em `params.rs` ("a vista mede ~10 unidades") produz gesto grosso e formas-balão; faixa
    RELATIVA à seleção é a alternativa.
  - ⚠️ **No `d` extremo os joins CONVERGEM por correção** (`drop_phantoms`, §1): a `d=+4` os
    três dão a MESMA área (19.8) — Round mantém arcos (264 verts), mas Bevel (16) e Miter
    (8) ficam quase idênticos (o chanfro é minúsculo perto do offset gigante). Se o Enio
    testar a `d` grande, "Bevel ou Miter não muda" ENTRE SI é o comportamento CORRETO
    (eles convergem); só Round→Bevel/Miter deve mudar (arcos somem).
- **A morte da janela de retune era MUDA** (`RetuneStep::Dead => {}`) — agora LOGA.
- **A dança de layout do painel (ABERTO, decisão de produto):** quando o resultado do
  offset morre (aniquilação), a seleção esvazia, a seção TRANSFORM some e **os chips de
  Join/Side sobem ~230 px debaixo do cursor**; cada retune que ressuscita/mata a forma
  faz o painel OSCILAR — um clique mirado no layout anterior cai em zona morta ("não
  muda"). Com o motor consertado o extremo é consistente (aniquilado fica aniquilado em
  todo join), mas a dança segue possível em fluxos que alternam resultado vazio/não-vazio.
- ⚠️ **Interferência do AMBIENTE no harness (lição paga 2×):** o desktop é vivo — o KWin
  reposiciona a janela recém-aberta sob o cursor FÍSICO parado e emite `CursorMoved`
  REAIS, que o slider ativo obedece (um hold sem re-assert foi teleportado a `d=−4` pelo
  ambiente; a investigação perseguiu esse fantasma achando que era do app). O nível 18
  agora re-afirma a posição sintética TODO frame do hold.

**Falta o smoke do Enio** (veredito condicional, DIRETIVA §5): `PH2D_BUILD_SMOKE=18`
roda sozinho a fases de ~2 s (dá pra VER cada retune); o 17 é a cena manual. Se o report
persistir no fluxo dele, os instrumentos são `PH2D_UNDO_LOG=1` + o log novo da janela de
retune — e a pergunta focada é: *qual `d` o release comitou, e a forma saiu da tela?*

## §2 — O que JÁ foi consertado nesta janela (não re-derive, não re-litigue)

Commits, do mais novo ao mais velho — cada um com gates mutation-tested:

- **`6831b43d`** — o **fantasma do offset extremo** (ver §1): `drop_phantoms` em
  `loop_region` + gate `an_offset_past_the_shapes_death_leaves_no_phantom` + sonda
  `probe_offset_extreme_d` (`--ignored`) + log da morte da janela de retune + nível 18
  na ordem do report, com cliques de timing real e hold blindado contra o cursor físico
  (`smoke_click_screen` removido — clique de 1 frame não contém as corridas de um clique
  humano).
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

⚠️ **SEMPRE `--release` para julgar FPS** (lição do áudio W7): em debug o motor é ~16×
mais lento e "trava" é o build, não o produto. Em release o arrasto é 0.8–1.5 ms/frame.

- **`PH2D_BUILD_SMOKE=17`** — a cena manual do Expand (zig-zag/estrela/rosquinha/arco).
  Rodar: `cd Worktrees/line-Vector && PH2D_BUILD_SMOKE=17 cargo run --release -p ph2d-host-desktop`.
- **`PH2D_BUILD_SMOKE=18`** — **o harness AUTO-DIRIGIDO** (`build_smoke_expand.rs::
  smoke_expand_retune_drive` + primitivos em `build_smoke_drive.rs`): rola o painel, agarra
  o slider com o PONTEIRO, arrasta por frames na ordem do report (Miter default → Round →
  Bevel → Miter, cliques com Down/Up em frames separados), logando por frame `dt` (o FPS),
  undo, janela viva, join, VERTS e a LARGURA do bbox (o oráculo do arrasto — verts é cego
  ao Miter/Bevel). Roda sozinho: `cd Worktrees/line-Vector && PH2D_BUILD_SMOKE=18 timeout
  30 cargo run --release -p ph2d-host-desktop 2>&1 | grep retune-smoke`. ⚠️ Ele re-afirma
  a posição do hold por frame porque o WM injeta `CursorMoved` reais sob a janela recém-
  aberta (ver §1) — não remova.
- **`probe_offset_cost_on_the_d_ladder`** (`crates/ph2d-vec-boolean/tests/
  probe_offset_extreme_d.rs`, `--release --ignored --nocapture`) — o custo do motor por
  join/`d` (Round `d=4` = 1.5 ms; o resto <0.2 ms).
- **`probe_every_join_on_the_d_ladder`** (mesmo arquivo, `--ignored`) — paths/verts/área
  por join/`d`, o que decodificou o fantasma.
- **`PH2D_UNDO_LOG=1`** — cada passo de undo com o diff. O log da janela de retune que
  fecha (`RetuneStep::Dead`) sai sem env (é `eprintln` incondicional).

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
