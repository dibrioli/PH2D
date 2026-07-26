# Handoff de integração — `line/Vector`: FX RASTER (Blur / Glow / Drop Shadow)

**Plano:** [`docs/Vector Module/24_plano_fx_raster.md`](Vector%20Module/24_plano_fx_raster.md) · **Data:** 2026-07-25 · **1 commit** na `line/Vector`.

O FX raster de alta qualidade para formas vetoriais — a resposta ao pedido "efeitos FX de alta
qualidade, estado da arte, compatível com o que temos". Melhor que o Rive (cujo FX deriva do
feather e é acoplado ao tesselador) e no rumo do próprio Vello (o grafo de filtros SVG). Wave 1:
**Blur · Glow · Drop Shadow**. As waves W2 (grafo componível: color-matrix/displacement/turbulence)
e W3 (feather analítico) ficam para depois.

## A costura (o inegociável)

Um FX raster produz **PIXELS**, não `VecPath` ⇒ **não é PathEffect** (`effect::run_stack` é
`VecPath->VecPath`, puro, sem GPU, dentro da `ph2d-vec-scene`) **nem `LiveGeometry`**. É uma
`FxImages` que o **shell produz** e o `dispatch` só **encoda** no z da forma. É por isso que a
seção do painel se chama **Filters**, distinta de **Effects** (deformadores vetoriais, ADR-0132).

## ⚠️ ARQUITETURA: 100% RESIDENTE NA GPU (Enio: "tudo é para o game em runtime, total performance")

O 1º corte foi CPU-first (render→**readback GPU→CPU**→Gaussiana na CPU→**re-upload**) — padrão de
PREVIEW de editor. Em runtime a forma anima, e esse roundtrip roda por frame por forma: o readback
bloqueia o pipeline e o re-upload **vaza o atlas do Vello** (Blob nova por frame = id novo = upload
que cresce sem fim — medido: recook 37→793 ms num smoke parado). **Reescrito GPU-resident:**
- **`ph2d_render::FxBlurPass`** — passe de compute textura→textura, Gaussiana SEPARÁVEL (H/V
  ping-pong) + finalize fundido no V (des-premul p/ Blur; cor do efeito × alfa da silhueta p/
  Glow/Drop Shadow; opacity). Molde do `ImpastoLightPass`. Gate `#[ignore]` `fx_blur_gpu.rs`
  (verde na RTX; **roda na lane de GPU** com `--ignored`).
- **`VelloPass::register_texture`/`override_image`/`unregister_texture`** — uma textura da GPU vira
  imagem desenhável por **id ESTÁVEL**, sem upload de CPU; re-borrar escreve NA MESMA textura ⇒
  zero churn de id no atlas.
- **`fx_live`** — por forma: um scratch `VelloPass` (render isolado, sem readback) → `FxBlurPass` →
  textura de saída persistente registrada no **renderer PRINCIPAL** (o que desenha `vector_scene`;
  registrar noutro faz o Vello entrar em pânico). O memo vira OTIMIZAÇÃO (pula o re-blur), não
  correção. Desregistro enfileirado (`forget` não tem `vello_pass`; o `recook` drena).
- **`ph2d_vector::StableImage::from_image_data`** + re-export `ImageData`.
- **Removidos:** `process_fx`/`blur_premul`/`gaussian` (blur da CPU) + o readback + os gates de CPU
  (substituídos pelo gate de GPU).

Custo por frame: 1 render isolado + 2 passes de blur + 1 cópia GPU→GPU no atlas — tudo na placa,
escala para formas animadas. ⚠️ **`dispatch` ganhou +1 arg** (`fx: &FxImages`); e o `recook` do
shell recebe `&mut VelloPass` (o principal).

## Deltas que a integração precisa CONFERIR (o número se conta, não se escolhe)

| Item | Antes | Depois | Como |
|---|---|---|---|
| `ph2d-ecs` registry | 37 | **38** | `VecFilter` registrado (blob-key) |
| espelhos `ph2d-render`/`ph2d-script` | 38 | **39** | ecs+Sprite / ecs+LuauScript |
| `PROJECT_SCHEMA` | 31 | **31 (INTOCADO)** | componente por blob-key = sem bump posicional |
| `VEC_SCENE_SCHEMA` | 13 | **13 (intocado)** | — |
| **§6 contrato vetorial** | — | **INTACTO** | `architecture_vector_contract_surface` verde |
| `VECTOR_SECTIONS` (painel) | 26 | **27** (append) | Filters (gate de contagem atualizado 26→27) |
| seam headers count | 26 | **27** | mesmo motivo |

⚠️ Se `line/physics` ou `line/FLIP` mexerem no registry/`PROJECT_SCHEMA` na MESMA janela, o número
final **se recalcula** ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). O `VecFilter`
não move `PROJECT_SCHEMA` (blob-key), então esse eixo não conflita; o registry (38/39) pode.

## O que landou

- **`ph2d-ecs::VecFilter`** ([vec_filter.rs](../crates/ph2d-ecs/src/vec_filter.rs)) — kind + radius
  (MUNDO) + offset (MUNDO) + color (straight RGBA [0,1]) + opacity. `tints()`/`displaces()`.
- **`ph2d-render::VelloPass::render_and_readback`** — readback de campo-cheio (irmão do `read_pixel`).
- **`ph2d-vec-render`**: `FxImages`/`FxMode`/`FxImage` + `dispatch(...,fx,...)` (⚠️ **+1 arg**, único
  chamador é o render_loop) + pure `path_screen_bounds` + `draw_path_isolated`. `lib.rs` foi
  **splitado** (`lib_tests.rs`) pelo teto de LOC.
- **`fx_live`** (shell) — o produtor: bounds → scratch VelloPass (lazy) → readback → **Gaussiana
  separável em PREMULTIPLICADO** (o intermediate do Vello é premul) → tint/offset → memo por
  (spec,w,h,sigma). `set_filter`/`edit`/`default_for`.
- **Painel "Filters"** (`ph2d-panel-vector`, `paint_filters`/`state_filters`/`populate_filters`/
  `event_filters`) + bridge nos 3 sítios do `render_loop` (publish · SetValue · command) + picker
  OKLCH. Foundational: ids `VECTOR_FILTER_*`/`VECTOR_SECTION_FILTERS` (append-only).

## Gates (o de dispatch achou um BUG REAL)

- ⚠️ **O Blur (Replace) desenhava NADA** — o `dispatch` só desenhava a imagem no ramo `Below`; para
  `Replace` pulava a forma e nunca desenhava a imagem. **A estrela de Blur do smoke aprovado estava
  BRANCA** (só Glow/Shadow, que são `Below`, funcionaram). O gate `a_replace_filter_...` (n_paths)
  pegou; corrigido. **PEDE RE-SMOKE do Blur.**
- CPU (fx_live): rampa alarga com sigma · tint/mode. dispatch (vec-render): Replace no-lugar ·
  Below forma+imagem · bounds cobre+escala 2×. Painel (seam_filters): 4 chips ao bus · seção ausente
  sem forma. Fechamento: clippy limpo · LOC caps · §6 · registry counts · node_id_collisions ·
  workspace test-compile exit 0.

## Smoke

`cd <worktree> && env PH2D_BUILD_SMOKE=33 cargo run -p ph2d-host-desktop --release` — quatro
estrelas: controle nítido · **Blur** · **Glow** ciano · **Drop Shadow** preta 60% deslocada. Arma
via `set_filter` programaticamente (não exercita o painel).

**PENDENTE (smoke):** re-smoke de **PERFORMANCE** — a versão CPU caía o FPS (readback + vazamento
do atlas); a GPU-resident deve rodar liso. `PH2D_FX_PERF=1` imprime `re-borrado(s)` + ms do recook.
E o smoke do **PAINEL** (abra o Vector, desenhe uma forma, selecione → seção *Filters* → Blur/Glow/
Shadow → afine Radius/Offset/Color/Opacity). ⚠️ **Os gates GPU do FX são `#[ignore]`** — o
integrador roda `cargo test -p ph2d-render --test fx_blur_gpu -- --ignored` na RTX (sem adapter =
skip gracioso, que não é verde).

## Aberto / follow-ups (nomeados, não contrabandeados)

- **1 filtro re-borrava todo frame na versão CPU** (memo miss) — a causa exata do miss não foi
  isolada, mas na GPU-resident ela virou barata (um blur de GPU por frame) e o id estável mata o
  vazamento. Se o `PH2D_FX_PERF` ainda mostrar `re-borrado` constante em cena parada, vale caçar o
  miss (é uma otimização, não correção).
- **Radius é slider em unidades de MUNDO** (`FILTER_RADIUS_MAX=2.0`) — fração-do-tamanho seria mais
  robusto para formas de tamanhos diferentes (a mesma nota que o Contour faz do Offset).
- **`FxBlurPass` cap `MAX_HALF=96`** (sigma ≈ 32 px de tela): acima o borrão satura — limite de
  CUSTO honesto do passe, não de produto.
- **W2/W3** do plano 24 (grafo componível além do Rive; feather analítico).
- Sombra/glow do FX **não** compõem com a pilha de Effects na mesma forma numa ordem escolhida
  (cada forma tem UM VecFilter); é decisão de produto se um dia precisar.
