# HANDOFF DE INTEGRAÇÃO — `line/FLIP` · 2.5D MULTIPLANO (2026-07-25)

> **Para o AGENTE INTEGRADOR.** A linha `line/FLIP` foi reaberta após a integração
> da wave "A tira ganhou mãos" (2026-07-25) e recebeu **UM commit**: o **multiplano
> 2.5D** (paralaxe por camada, ADR-0114 §Decisão 3). Aprovado pelo Enio ("sim,
> siga"). **Não integrar nem pushar sem ordem EXPLÍCITA do Enio.**

## 0. GO — o essencial em 6 linhas

- **1 commit** (`9ac0f2fe7`), **ff-only** sobre o `main` de hoje (a linha está
  exatamente 1 à frente; `git rev-list --count main..line/FLIP` = 1).
- **Bump de schema:** `FLIP_SCHEMA_VERSION` 9→**10**, `PROJECT_SCHEMA` 31→**32**,
  tripla do pin `(32, 10, 13)`. ⚠️ **O 32 se CONTA, não se escolhe** — se outra
  linha bumpar o `PROJECT_SCHEMA` na mesma janela, **renumere** (o certo é a soma,
  não o que a linha escreveu) e atualize a tripla em `project_tests.rs:432`
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
- **Contratos congelados (§6): INTOCADOS.** Nenhum ADR novo (era um item já
  aceito, §Decisão 3 do ADR-0114). `FlipLayerWidget::ALL` 7→8 é dado de UI, não
  contrato.
- **Smoke:** `env PH2D_FLIP_MULTIPLANE_SMOKE=1 cargo run -p ph2d-host-desktop --release`.
- **Gotchas do `ship.sh`** (herdados, não novos): `ph2d-flip-colorize` **panica em
  build DEBUG** e passa em `--release`/`ci-test`; os gates GPU do Flip são
  `#[ignore]` (esta wave **não adiciona** gate GPU — ver §4). Nada aqui precisa de
  adapter.
- **Sem MERGE textual esperado** (a linha só toca arquivos do Flip + o wiring de
  schema do shell; se houver conflito no `project.rs`/`project_tests.rs` por outra
  linha ter bumpado, resolva pela SOMA dos schemas, não por lado).

## 1. O que a wave entrega

Cada `FlipLayer` ganha `depth: f32` (fração de paralaxe; `1.0` = flat = o comum) e
um **slider Depth** no bloco de camada do painel Flip, logo abaixo da Opacity.
Panhar a câmera-viewport desliza cada camada `depth × pan` na tela: fundos (depth
baixo) ficam para trás, a frente (`1.0`) corre com o mundo. **Ortográfico puro** —
sem perspectiva, sem Z-buffer, sem câmera de cena animável.

**Medido** (sonda headless, 1280×720, `height_world` 10, pan de 3 unidades de
mundo): near `1.0` → 216 px · mid `0.5` → 108 px · far `0.15` → 32,4 px. O
deslocamento é **`depth × pan`**, exato.

## 2. A espinha (para revisar com confiança)

- **Porta ÚNICA `parallax_model(model, cam_center, depth)`** em
  `shells/desktop/src/render_loop/flip_pass.rs`: desloca a translação do `model` do
  objeto por `(cam_center − origem)·(1 − depth)` **antes** do `fold_model`. É a
  MESMA porta que a arte assada E o preview vivo da camada usam (o esboço não
  descola). **`depth == 1.0` devolve o `model` intacto** ⇒ todo o caminho flat
  comum é **byte-idêntico** ao pré-multiplano (`is_identity()` ainda pega o fast
  path). Âncora na origem do objeto: enquadrado de frente os planos coincidem;
  panhar os separa.
- **Wiring:** `composite_layers` recebe `cam_center: [f32;2]` (o callsite passa
  `camera.center`) e chama `parallax_model(&l.model, cam_center, l.depth)` por
  camada. `LayerRef.depth` é populado nos DOIS sítios de construção (arte + ghost).
- **UI:** `FlipLayerWidget::Depth` (`ids/chrome/flip.rs`, `ALL` 7→8) · `FlipLayerRow.depth`
  no snapshot (`state.rs` + `flip_bridge.rs`) · Line 4 do bloco por **porta única
  `paint_bare_slider_row`** (Opacity e Depth são a MESMA linha `0..1` + `NN%` —
  colapsei as duas cópias, que era o que estourava o cap de fn de 200 LOC) ·
  `event.rs` roteia `ValueChanged → SetValue` (arm `Opacity | Depth`) · a shell
  escreve `l.depth` clampado (`flip_layers.rs`).

## 3. Gates (todos red-first, mutação-provados nesta sessão)

- `render_loop::flip_pass::multiplane_tests` (`flip_multiplane_tests.rs`):
  `depth_of_one_returns_the_model_untouched` · `a_layer_at_depth_zero_pins_to_the_camera`
  · **`the_far_layer_lags_the_near_one_under_pan`** (projeta a origem pela matriz
  REAL do passe — oráculo mais forte que um readback de GPU, que só rasteriza a
  matriz; ver §4) · `all_planes_coincide_when_the_camera_is_over_the_origin` ·
  **`composite_layers_threads_the_camera_center_and_layer_depth`** (arch-gate sobre
  o fonte — o wiring exige `GameRt`/wgpu, nenhum unit test o alcança; afirma a
  PROPRIEDADE, não distância em bytes [[feedback_a_gate_anchored_on_a_byte_distance_is_a_proxy_that_expires]]).
- `flip_layers::tests::depth_setvalue_sets_the_layer_depth` (shell) — o drag chega
  à camada + clamp.
- `ph2d-panel-flip/tests/seam.rs::the_multiplane_depth_slider_paints_and_forwards_setvalue`
  — pintado + registrado + `ValueChanged→SetValue`.
- `ph2d-flip/src/layer.rs::depth_defaults_to_flat_and_survives_the_round_trip` —
  default 1.0 + round-trip postcard.
- Pin: `project_tests.rs::a_schema_bump_anywhere_must_bump_the_project_schema`
  atualizado para `(32, 10, 13)`.

**Verificação de mutação feita e restaurada:** `parallax_model` ignorando `depth`
(→ `*model`) sangra `far_layer_lags` + `depth_zero_pins`; o wiring `l.depth → 1.0`
sangra o arch-gate.

## 4. Por que NÃO há gate de GPU (decisão de oráculo, não omissão)

A paralaxe vive 100% no transform de vértice: `parallax_model` desloca a translação,
`fold_model` a dobra no `world_to_clip`, e o rasterizador só desenha triângulos nas
posições de clip que a MATRIZ produziu. Um readback de GPU testaria o **rasterizador**
(inalterado por esta feature), não a paralaxe. O oráculo fiel é projetar o vértice
pela MESMA matriz que a GPU consome — determinístico, sem wgpu. É o que
`the_far_layer_lags_the_near_one_under_pan` faz. (Nota no topo do
`flip_multiplane_tests.rs`.)

## 5. Passos de integração (quando o Enio mandar)

1. `cd` na worktree do integrador, `git fetch`, confira que `line/FLIP` está 1 à
   frente do `main` e é **ff-only** (`git merge-base --is-ancestor main line/FLIP`).
2. Se `PROJECT_SCHEMA` colidir com outra linha da jornada: **renumere pela SOMA** e
   atualize a tripla `(N, 10, 13)` em `project_tests.rs`.
3. `git merge --ff-only line/FLIP` (ou o fluxo do `foundational-integrate.sh` se a
   jornada tiver outras linhas na árvore combinada).
4. `./scripts/ship.sh` — corrija todo `✗`. ⚠️ rode com o perfil `ci-test`/`--release`
   (o `ph2d-flip-colorize` panica em debug); os gates GPU `#[ignore]` do Flip
   precisam de adapter na RTX (esta wave não adiciona nenhum, mas os herdados valem).
5. Smoke do Enio: `env PH2D_FLIP_MULTIPLANE_SMOKE=1 cargo run -p ph2d-host-desktop --release`
   — a cena imprime `[multiplane-smoke] cena montada: 3 planos [...]`. **Se essa
   linha não aparecer, PARE** (árvore/env errada). Depois: panhar separa os 3
   planos; o slider Depth do 'Ceu' a 100% funde-o com a cerca.

## 6. Aberto (não-bloqueante, decisões de produto)

- **Paralaxe de ZOOM** (planos fundos escalam menos que a frente ao dar zoom) —
  §Decisão 7 do ADR-0114, deferido: hoje só o PAN dá paralaxe; o zoom afeta todos
  igual. Precisa de desenho próprio (a âncora de escala ≠ a de pan).
- **Câmera de cena ANIMÁVEL** (uma câmera keyável na timeline que dirige o pan) —
  hoje o multiplano lê a câmera-VIEWPORT de edição (`AppGfx.camera`), não uma
  câmera de documento. Outro sistema; §Decisão 7.
- O `unfolded` (preview fallback quando a camada-alvo está oculta) rende **flat**
  (sem paralaxe) — transitório e aceitável (o usuário nunca desenha às cegas).
