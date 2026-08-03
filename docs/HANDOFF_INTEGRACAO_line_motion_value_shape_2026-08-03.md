# Handoff de integração — `line/motion-value` · `source.shape` (vetor vivo na GPU)

**Data:** 2026-08-03 · **Branch:** `line/motion-value` · **Commit:** `97b318f98`
**ADR:** [0154](architecture/decisions/0154-motion-shapes-are-live-gpu-vector-not-baked-tiles.md) (⚠️ **número PROVISÓRIO** — renumera na integração se o `main` do dia já tiver 0154).

## O que landou

Um nó **`source.shape`** que gera uma forma paramétrica (Circle · Square · Ellipse ·
Rectangle · Polygon · Star · Heart · Gear) que flui pelo grafo como **geometria
VIVA** e renderiza **nítida em qualquer zoom** via Vello — não uma tile assada. É a
resposta ao pedido do Enio ("um nó de formas como o MiniCavalry V2, mas melhor"): o
MiniCavalry usa `ctx.fill` de canvas 2D **raster**; nós temos Vello (o estado da arte,
como Cavalry/AE/Blender/Rive).

**A espinha (ADR-0154):** `geometry_id` é uma **CONVENÇÃO DE STREAM**, o gêmeo exato do
`texture_id` do doc 86 — um handle para um `VecPath` num **`VecPathStore`** (cache por
conteúdo, o gêmeo do `IndividualTextureStore`). **Ausente ⇒ byte-idêntico** (o caminho
pré-forma intacto). O nó é handed só params (não alcança a GPU nem a lib vetorial),
então a **SHELL** constrói o `VecPath` a partir dos params e o publica sob a **chave de
conteúdo** que o nó lê; o `motion.duplicator` carimba a MESMA geometria em cada ponto (o
`geometry_id` atravessa o duplicator como o `texture_id`). Desenhado no `VectorScene`
que o `VelloPass` **já** renderiza — **SEM passe de GPU novo**.

## Contrato / schema — INTACTOS (conferir por grep na árvore combinada)

- **Contrato congelado §6 intacto:** `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` — `geometry_id`
  é convenção de stream, **não** campo do `NodeManifest`. Gate `architecture_contract_surface`
  verde (3/3). O nó `source.shape` é um TIPO novo (manifest + register), não um campo/método novo.
- **Schema intocado:** `PROJECT_SCHEMA` / `VEC_SCENE_SCHEMA` / `DOC_VERSION` **não mudam** — o
  grafo viaja como TEXTO e carrega a própria versão; nó + convenção são aditivos (precedente doc 86).
- **Nenhum ADR de contrato**, nenhuma dep externa nova.

## Arquivos tocados

**Foundational (aditivo — projetado para isolamento):**
- `crates/ph2d-eval-motion/src/lower.rs` — a convenção `geometry_id` no `lower_to_instances_onto`
  (o `match` no fim: sem a coluna ⇒ VERBATIM; com ⇒ filtra os sprites `id≤0.5`), o tipo novo
  `VectorInstance`, e `lower_to_vector_instances_onto`. `lib.rs` re-exporta + o pump ganhou
  `vector_instances: Vec<VectorInstance>` (preenchido no ramo Sinks do cook).
- `crates/ph2d-vec-render/src/lib.rs` — `pub fn draw_shape_instance(path, transform, tint, scene)`
  (append-only: desenha um `VecPath` avulso com tint por-instância; `draw_path_isolated` baka o
  `path.fill`, então esta porta é necessária para o tint por-cópia).

**Crate nova (leaf, deps `ph2d-nodegraph` + `ph2d-node-registry`):**
- `crates/ph2d-node-motion-shape/` — o nó `source.shape`. `ShapeKind` (8 fillable, append-only),
  `ShapeParams` (7 params reusados: `sides` = lados/pontas/dentes, `inner` = profundidade), a porta
  única `ShapeParams::read`, `shape_key` (content-addressed, `to_bits` exato), o `MANIFEST`
  (`kind` = `ParamWidget::Enum`, o resto sliders).

**Shell:**
- `shells/desktop/src/render_loop/motion_shape_gen.rs` — `VecPathStore` + `read_params` (a mesma
  porta que o nó, ver abaixo) + `build_shape_path` (dispatch p/ `ph2d-vec-scene`) + `publish`
  (varre nós `source.shape`, interna, publica) + `encode` (desenha no present).
- `shells/desktop/src/motion_state.rs` — campo `shape_store: VecPathStore` no `MotionState`.
- `shells/desktop/src/render_loop/mod.rs` — `pub(crate) mod motion_shape_gen;` · `motion_shape_gen::publish(motion)`
  **no call-site do `publish_shapes` (~linha 5086)**, não dentro de `motion_bridge.rs` — ⚠️ **de propósito**:
  `motion_bridge.rs` estava a 599/600 LOC, e a chamada lá o estourava (603); no call-site a ordem é a
  mesma (roda logo após o `shapes::publish` que limpa os externals). · a chamada `encode` logo após o
  `ph2d_vec_render::dispatch` (~linha 6340), gated em `motion_tool_active` (como os sprites do Motion).
- `shells/desktop/src/motion_shape_smoke.rs` + `main.rs` (`mod`) + `mod.rs` (o hook no prólogo).
- `shells/desktop/Cargo.toml` — **a ÚNICA mudança de `Cargo.toml`**: dep na crate-nó (a shell chama
  `ShapeParams::read`/`shape_key`).

**Node-sync (regenerado, idempotente):** `crates/ph2d-node-registry-init/{Cargo.toml,src/lib.rs}` — o
`register_all_nodes` ganhou `ph2d_node_motion_shape::register`. `Cargo.lock` acompanha. Gate de staleness
verde (`register_all_nodes_is_in_sync_with_folder`). ⚠️ **Se o rebase mudar a lista de nós, rode
`cargo run -p ph2d-node-sync` de novo** antes de conferir o gate.

## Pontos sensíveis de merge

- ⚠️ **`ADR-0154`** — número provisório. Se o `main` do dia já tiver um 0154, renumere (o rewrite do
  token escopa aos arquivos DA LINHA — hoje só o `.md` do ADR e as citações em `motion_shape_gen.rs`/
  handoff; **não** varra a árvore).
- ⚠️ **`ph2d-eval-motion` é foundational** — as adições (`geometry_id`, `VectorInstance`,
  `lower_to_vector_instances_onto`, `pump.vector_instances`) são **append-only** e o caminho pré-forma
  é **byte-idêntico** (gate `a_stream_without_geometry_id_is_all_sprites_and_no_vectors`). Se outra
  linha tocou `lower.rs`, o merge é textual; confira o gate na árvore combinada.
- ⚠️ **`render_loop/mod.rs`** ganhou uma `mod` decl + 2 chamadas — arquivo grande, mas as edições são
  pontuais (o `dispatch`/`publish_shapes` são âncoras estáveis).

## Gates (12, todos mutation-proven, DEBUG + RELEASE)

- **`ph2d-eval-motion` (3):** `geometry_id_splits_sprites_from_vectors` (mutação: filtro invertido ⇒ 2
  RED) · `a_stream_without_geometry_id_is_all_sprites_and_no_vectors` (aditivo) · `a_shape_row_is_not_also_a_sprite`.
- **`ph2d-node-motion-shape` (4):** key determinística + separa cada param · índice→kind round-trip · read
  sobre defaults · hints nomeiam params declarados.
- **Shell `motion_shape_gen` (5):** **`publish_then_cook_the_node_reads_its_own_shape`** (o SINGLE DOOR —
  mutação: `read_params` ignora overrides ⇒ chaves divergem ⇒ count 0, RED) · `a_mismatched_key_decouples`
  (controle negativo) · **`a_shape_stamped_on_a_grid_lowers_to_sixteen_vectors`** (e2e: Star × grade 4×4 →
  16 instâncias vetoriais, 1 handle interned — prova que o duplicator preserva `geometry_id`) ·
  `every_kind_builds_distinct_geometry` · `an_unpublished_handle_is_none_and_encodes_without_panic`.

Close-out: clippy limpo (crates + shell `--bins`), `file_loc_caps` (shell 600) + `workspace_file_loc_cap`
(700) + `architecture_contract_surface` + `no_tofu_glyphs` + node-sync staleness — **todos verdes**.

## Smoke (número MEDIDO)

**`env PH2D_SHAPE_SMOKE=1 cargo run -p ph2d-host-desktop --release`** — no frame 3 monta
`source.shape(Star) → motion.duplicator ← motion.grid(4×4) → output`, abre a tool Motion, e imprime o que
montou (**16 cópias nítidas**, o número que o gate e2e mede). No frame 90 troca `kind → Gear` ao vivo. Julgue:
as 16 estrelas devem estar **NÍTIDAS em qualquer zoom** (não pixeladas — é vetor, não tile), e ao chegar no
frame 90 as 16 devem virar engrenagens sem re-abrir nada. ⚠️ **Pendente de smoke visual.**

## Aberto (Fase 2, nomeado — não escondido)

- **Z-interleave por-instância** entre forma-vetor e sprite-texturizado: Fase 1 é **vetor SOBRE sprite** (a
  ordem do compositor; o vetor desenha no `vello_pass` que escreve por cima do `sprite_pass`). O interleave
  fino é segmentação de passe (o problema que o compositor do Painter já resolve).
- **Composição em nível de geometria** (deform/boolean/trim/morph sobre a `VecPath`) — cai de graça no
  idioma do módulo Vector, porque a geometria é DADO.
- **Arc/Spiral** (curvas ABERTAS) ficaram fora do v1 — o `draw_shape_instance` preenche, e um preenchimento
  ignora contorno aberto; entram quando o desenho decidir wedge-close/stroke.
- **Cor da forma:** v1 é branca + tint rio-abaixo (`motion.tint`/`color.*`), como os sprites. Um param de cor
  no nó (via `ParamWidget::Color`) é nice-to-have.
- ⚠️ **Crescimento do `VecPathStore`:** mantido across-frames (forma estática constrói UMA vez). Um param
  animado (slider arrastado, ou dirigido) re-interna cada valor visitado — **nomeado**, limitado às formas
  distintas da sessão (o precedente do cook do voronoi). Um LRU/cache-por-nó bounda isso se medir.
- **Visibilidade:** as formas renderizam gated em `motion_tool_active` (como os sprites do Motion). Se o Enio
  quiser a cena Motion sempre visível, é decisão de produto (muda o gate).

## 2ª rodada (mesmo dia) — flicker, per-shape params, gear

Cinco pedidos do Enio: (1) editar a forma faz ela PISCAR; (2) `sides` ≤ 32;
(3) cada forma mostra só os ajustes que USA; (4) mais params por forma + a
engrenagem estava ruim; (5) melhore tudo.

- **Flicker FECHADO.** A causa era ordem de frame: `apply_graph_intents`
  (motion_bridge.rs:264) drena o edit de param DENTRO da bridge, mas o shape
  publish rodava ANTES da bridge (mod.rs:5091) — então publicava a chave PRÉ-edit
  enquanto o cook (pós-drain) lia a chave PÓS-edit ⇒ o nó clonava um external
  vazio por 1 frame ⇒ a forma sumia. Movido para **pós-drain, pré-cook**
  (motion_bridge.rs:356, `super::motion_shape_gen::publish(motion)`); o call-site
  em mod.rs:5091 só ganhou um comentário. O doc-comment de `publish` registra a lei.
- **`sides` 128 → 32** (min 3).
- ⚠️ **Per-shape params = uma ADIÇÃO FOUNDATIONAL a `ph2d-node-registry`
  (append-only):** o canal novo **`ParamGate { param, when, values }`** +
  `register_param_gates`/`param_gates` — o gêmeo exato de `register_param_ui`
  (mesmo `BTreeMap<NodeTypeId, &'static [T]>`, `#[derive(Default)]` ⇒ zero init).
  **Nenhum campo do `NodeManifest`** (contrato congelado intacto), **zero churn
  nos 78 nós** (um nó sem gate = tudo visível, o pré-comportamento). O tipo mora
  em `ui.rs` (ao lado de `ParamHardMax`), re-exportado em `lib.rs`; round-trip
  gated (`param_gates_round_trip`).
- **O filtro é UMA porta** em `build_params_snapshot` (motion_bridge_params.rs):
  `shown(param)` = sem gate OU o `when` corrente está nos `values`; aplicado por
  `.filter()` nos DOIS laços de row. ⚠️ `build_params_snapshot` foi de `pub(super)`
  para **`pub(crate)`** para o gate de shell alcançá-lo.
- **Params redesenhados (9, cada um gated ao seu kind):** `size` (todos) ·
  `aspect` (Ellipse/Rectangle) · `sides` (Polygon/Star/Gear) · `corner`
  (Square/Rectangle/Polygon/Star, agora **fração do size** ⇒ escala) · `star_depth`
  (Star) · `cleft` (Heart) · `tooth_depth` + `hole` (Gear). O `frame` MORTO foi
  removido; o `inner` reusado virou params dedicados (labels honestos). O key
  hasheia os 9 (params escondidos ficam no default ⇒ formas iguais ainda dedupam).
- **Gear consertado:** passava `hole = 0.0` (disco dentado sem furo — o "muito
  ruim"); agora `hole` (default 0.45) + `tooth_depth` (0.35) dedicados.

**Gates novos (todos mutation-proven, DEBUG+RELEASE):** registry
`param_gates_round_trip` · node `every_gate_names_declared_params_and_valid_kinds`
· shell **`the_params_panel_shows_only_the_shapes_kind_params`** (Circle esconde 7,
Gear mostra sides/tooth_depth/hole; mutação: neutralizar o filtro ⇒ Circle mostra
os 9, RED). Range/behavior tests da params-panel (5+14) seguem verdes. Contrato
§6 intacto, LOC caps ok (motion_bridge.rs 600, motion_bridge_params.rs 596).

**A linha está FECHADA. Não integrei nem pushei (§0.7). Aguardando ordem explícita do Enio.**
