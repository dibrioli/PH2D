# Handoff de integração — linha `line/FLIP` (2026-07-12)

> **Para o agente integrador** (DIRETRIZ §1.5.9). A linha está FECHADA: gates batched verdes,
> smokes do Enio aprovados (traço/mordida · W3 · W4/âncora). A linha **não integra nem pusha** —
> este documento é o insumo; a integração roda `scripts/foundational-integrate.sh` + merge
> `--ff-only`, por ordem explícita do Enio.

## 1. Identidade

- **Branch:** `line/FLIP` (worktree `Worktrees/line-FLIP`).
- **Base do fork:** `3805f650` (= HEAD do `main` na abertura da jornada; merge-base confirmado).
- **Conteúdo:** ~22 commits lineares (sem merges), cobrindo **WT** (traço: a mordida morta,
  união global da polilinha), **W3** (frames/exposição/ciclos/ghosts/tween + a tira docada),
  **W4** (o balde `ph2d-flip-fill` + auditoria de 12 bugs + âncora no EIXO, BUGS #14) e o
  **alvo vivo** (`flip_live`). Último commit de código: `b8e281fa` (Precision default 1,6);
  HEAD = o commit deste handoff.
- **Trackers:** [`HANDOFF_flip_impl.md`](HANDOFF_flip_impl.md) (exaustivo) ·
  [`docs/Flip/BUGS_flip.md`](Flip/BUGS_flip.md) #1–#14 (saga completa) ·
  [`HANDOFF_flip_NEXT.md`](HANDOFF_flip_NEXT.md) (próximo implementador; W5 Reshape).

## 2. Foundational/compartilhado tocado (tudo ADITIVO; nenhuma remoção fora do módulo)

| Arquivo | O quê / por quê |
|---|---|
| `crates/ph2d-editor-core/src/ids/chrome/flip.rs` | ids novos W3/W4 (fill + strip) — **módulo próprio do Flip**, append-only por design |
| `crates/ph2d-editor-core/src/screens/layout.rs` | slot novo `flip_strip: Rect` + `FLIP_STRIP_H = 132.0` (faixa inferior, coluna do timeline) |
| `crates/ph2d-editor-core/src/screens/hero/paint.rs` | entrada `FLIP_STRIP_PANEL` no walk de z-order (sem ela o painel docado nunca é pintado) |
| `crates/ph2d-editor-core/tests/node_id_collisions.rs` | entradas novas dos ids Flip (append na lista do gate) |
| `crates/ph2d-editor-core/tests/architecture_panel_wiring_parity.rs` | crate nova `ph2d-panel-flip-frames` incluída no scan |
| `crates/ph2d-ui-testkit/{src/lib.rs,Cargo.toml}` | **`MockPanelHost::paint`** — o gate "pintado ≠ populado" (BUGS #8): roda `Panel::paint` real headless; +2 path-deps (`ph2d-vector`, `ph2d-text`). Extensão aditiva; qualquer linha pode adotar |
| `crates/ph2d-panel-registry-init/{src/lib.rs,Cargo.toml}` | registro do painel novo `ph2d-panel-flip-frames` (feature `panel-flip-frames`) |
| `shells/desktop/*` | arquivos NOVOS `flip_autokey/flip_fill(+tests)/flip_live/flip_strip` + rewrites nos `flip_*` existentes; wiring pontual (poucas linhas cada) em `app_state`, `forwarding`, `input_dispatch`, `input_handlers`, `main`, `project`, `undo`, `render_loop/{mod,present,flip_*}` |
| `.typos.toml` | allowlist pt-BR dos comentários do módulo (append de chaves) |
| `Cargo.toml` (raiz) + `Cargo.lock` | 2 membros novos (abaixo); **zero dependência externa nova** |

## 3. Símbolos que podem colidir com outra linha (grep-áveis)

- **Crates novas:** `ph2d-flip-fill` (solver do balde; dep única `ph2d-core`) e
  `ph2d-panel-flip-frames` (a tira; deps `editor-core`/`a11y`/`tokens`).
- **NodeIds:** todos por `hash_node_id("flip.*")` em `ids/chrome/flip.rs` (namespace próprio;
  o gate `node_id_collisions` detecta colisão de hash). Ids novos desta jornada:
  `flip.mode.fill`, `flip.fill.{swatch,paint,behind,unpaint,gap,gap_num,grow,grow_num,precision,precision_num}`,
  `flip.strip.{panel,close,play,prev,next,fps_num,ghost,ghost_before_num,ghost_after_num,autokey,additive,key_add,key_dup,key_del,hold_num,key_left,key_right,tween_num,tween_add,cycle_dd}`.
- **Const de layout:** `FLIP_STRIP_H: f32 = 132.0` (`screens/layout.rs`).
- **Consts de tool:** `DEFAULT_PRECISION: f64 = 1.6` + `GROW_MIN/MAX`, `PRECISION_MIN/MAX`,
  `GAP_MAX_PX` (`ph2d-tool-flip`, crate do módulo).
- **`FLIP_SCHEMA_VERSION: u32 = 3`** (era 1 no main; 1→2 camadas com `cycle`/`use_onion`,
  2→3 `holes`+`hide_stroke` no traço). Postcard posicional: save Flip de versão antiga é
  REJEITADO — deliberado. O par é checado em `shells/desktop/src/project.rs`
  (`(PROJECT_SCHEMA, ph2d_flip::FLIP_SCHEMA_VERSION)`).
- **Sem** IconId novo (reusa `Play/Pause/Plus/Trash/Copy/Chevron*/Skip*`), **sem** token novo,
  **sem** chave i18n nova no editor-core, **sem** variant novo em enum compartilhado.

## 4. Contratos congelados (CLAUDE.md §6)

**Nenhum encostado.** `Tool=12`/`CanvasPaintTool`/`PanelEvent=4` intactos (a tool Flip já
existia no main; os gates `architecture_*_contract_surface` não foram tocados). Nodes e
vector data-model idem.

## 5. O que só o `ship.sh` pega (o gate de integração NÃO roda)

- **fmt:** `rustup run 1.95 cargo fmt` rodado nas crates Flip tocadas, mas NÃO workspace-wide —
  skew pré-fork de outras crates aparece no ship, não é desta linha.
- **typos:** rodado nas pastas do módulo; o `.typos.toml` ganhou a allowlist pt-BR — se o ship
  acusar typo em `docs/Flip`/comentário do módulo, a resposta certa é estender a allowlist
  (palavra portuguesa), não "corrigir" o português.
- **machete:** 2 crates novas com deps mínimas; `ph2d-ui-testkit` ganhou `ph2d-vector` +
  `ph2d-text` (usadas pelo harness de paint — não são sobra).
- **deny/audit (RUSTSEC):** zero dependência EXTERNA nova → improvável; o `Cargo.lock` só
  ganhou as crates internas.
- **nextest-impacted:** as crates novas entram no grafo; se o impacted-set der vazio para elas,
  rode a suíte cheia 1× (memória: false-green pós-cutover).
- Commits da linha foram todos `--no-verify` (fast mode) — os hooks nunca rodaram nesta jornada;
  o ship é quem prova o conjunto.

## 6. Ordem, riscos de merge e o que smoke-testar depois de integrar

- **Ordem:** nenhuma — a branch é linear; integração é a branch inteira (ff-only). NÃO
  cherry-pique: o fix da âncora (`7477641b`) pressupõe o rewrite do W4 (`380c6d8c`+) e
  supersede `111637cd`/`42cf4d96` (histórico mantido de propósito).
- **⚠️ Untracked na árvore primária:** `docs/Flip/`, `docs/architecture/decisions/0114-*.md` e
  as memórias `project-memory/*flip*` existem **untracked/modificados no clone primário** — a
  linha os commitou (docs) ou NÃO os tocou (ADR/memórias, que o Enio commita por fora). Se o
  `merge --ff-only` reclamar de "untracked working tree files would be overwritten"
  (`docs/Flip/*`, `docs/HANDOFF_flip_*`), a resolução é o Enio commitar/mover os untracked do
  primário ANTES da integração — não sobrescrever às cegas.
- **Pontos de atenção de conflito** (se outra linha tocou os mesmos arquivos): o walk em
  `hero/paint.rs`, o slot em `layout.rs`, o registro em `panel-registry-init`, `.typos.toml` e
  os wirings pontuais do shell (`input_dispatch`/`forwarding`/`main`) — todos appends curtos;
  Mergiraf resolve, mas confira mesmo-símbolo (§1.5.5) se alguma linha criou OUTRO painel
  docado hoje.
- **Smoke pós-integração (5 min):** os três roteiros já aprovados na linha — (a) traço macio
  em zigzag/laço, hardness baixo (mordida); (b) tira: add/dup/delete de chave, ghosts, tween,
  Loop/PingPong; (c) balde: fill em linha fina, **zoom depois do fill**, Grow −3..+3 contínuo
  (roteiro detalhado em [`HANDOFF_flip_NEXT.md` §C.7](HANDOFF_flip_NEXT.md)).
  **Não smokado na linha** (registrado em "Aberto" do tracker): round-trip Ctrl+S/Ctrl+O com
  objeto Flip TRANSFORMADO (pose via gizmo) — funciona por construção (ECS snapshot), mas
  ninguém olhou.

---

*Linha `FLIP` pronta. Aguardo ordem de integração.*
