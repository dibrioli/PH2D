# Handoff de integração — `line/Painter` · Onda 5a (a pintura para de copiar o canvas por movimento)

**Para:** o agente integrador (DIRETRIZ §1.5.9). **Data:** 2026-07-24.

> ⚠️ **Esta branch já carrega as Ondas 1 e 2** (compositor GPU — handoff
> [`HANDOFF_INTEGRACAO_line_Painter_gpu_ondas_1_2_2026-07-23.md`](HANDOFF_INTEGRACAO_line_Painter_gpu_ondas_1_2_2026-07-23.md))
> **e a transferência sRGB do Wet Paint**
> ([`HANDOFF_INTEGRACAO_line_Painter_wet_transfer_2026-07-23.md`](HANDOFF_INTEGRACAO_line_Painter_wet_transfer_2026-07-23.md)).
> As três integram juntas por estarem na mesma branch; esta é a de cima.

## 1. Identidade

| | |
|---|---|
| branch | `line/Painter` |
| HEAD | `a9057588c` (+ 1 commit de docs em cima) |
| commit desta onda | **1** (`a9057588c`) + 1 docs |

## 2. O que muda, em uma frase

Pintar um traço simples num canvas grande caía de FPS, CPU-bound. A causa era um
**`Arc::make_mut` do canvas INTEIRO por movimento** (16 MiB @ 2048², 64 MiB @ 4096²,
independente do pincel de 0,5 px), forçado porque a shell segurava um clone do
`canvas_rgba` vivo do tool para detectar mudança por identidade de ponteiro. A shell
passa a **possuir o próprio buffer de preview** e a detectar mudança por uma **versão**
(`canvas_version()`) — o tool fica dono único do canvas e sua escrita é **in place**.

Medido, caminho real: 4096² **9,834 → 0,097 ms/move (~100×)**, e agora **plano no
tamanho da tela**. O depósito não muda ⇒ **aparência byte-idêntica**.

## 3. Foundational / contrato

- **`ph2d-render`: NÃO tocado nesta onda.** (As Ondas 1–2 o tocaram; esta não.)
- **`ph2d-tool-painter` (superfície pública): +1 método** — `PainterTool::canvas_version(&self) -> u64`.
  Aditivo; nenhum chamador existente quebra.
- **Nenhum contrato congelado** (`Tool`/`RasterEditTool`/`CanvasPaintTool`/`PanelEvent`,
  `NodeOp`/`OpResolver`/`NodeManifest`) tocado — conferido por grep.
- **Nenhum schema** — `PROJECT_SCHEMA` fica **29**; o preview não viaja em arquivo.
- `PainterPreviewGpu` (alias de `BgremovalPreviewGpu`, compartilhado) **não muda de
  forma** — o campo `arc_token` (token opaco de mudança) só passa a carregar a *versão*
  do lado do painter; o bgremoval segue com `Arc::as_ptr`. Doc do campo atualizado.

## 4. Arquivos

`ph2d-tool-painter`: `tool/mod.rs` (campo `preview_version`) · `tool/runtime.rs`
(`canvas_version()` + o bump) · `tool/paint/measure_gpu_frontier.rs` (a diagnose que
achou o bug) · `tool/paint/tests.rs` (o gate da versão).
`shells/desktop`: `render_loop/painter_bridge.rs` (o `own_preview_buffer` + a chave de
versão) · `render_loop/painter_gpu_preview.rs` (doc do sentinela) · `app_state.rs` (doc
do token) · `render_loop/mod.rs` (novo módulo de teste) ·
`render_loop/painter_preview_pipeline_tests.rs` (religado à versão + oráculo de verdade
independente + split de LOC) · `render_loop/painter_preview_handoff_tests.rs` (versão +
**lever corrigido**) · `render_loop/painter_preview_ownership_tests.rs` (NOVO, split) ·
`tests/the_paint_drain_owns_its_preview_buffer.rs` (NOVO arch-gate).

## 5. Símbolos que podem COLIDIR com outra linha

Nada numerado (sem id de widget, token, chave i18n, ADR, variante serde). Ponto de
atenção único: `own_preview_buffer` é **novo** em `painter_bridge.rs` e o drain foi
reescrito; um merge que toque o mesmo bloco do drain resolve-se **semanticamente** (o
drain estashha `own_preview_buffer(...)`, nunca o `Arc` drenado).

## 6. Latente da Onda 2 consertado de carona

O gate GPU-adapter `the_screen_survives_the_gpu_to_cpu_producer_handoff` usava uma
**máscara** como lever para flipar elegibilidade para a CPU — mas a **Onda 2 tornou
máscara representável**, então o lever parou de flipar e o gate ficou **vermelho-latente
por uma onda** (o `ship.sh` não roda gate GPU-adapter). Lever movido para um **ajuste
não-portado** (`ColorBalance`: sem código escalar nem espacial). ⚠️ Toda vez que uma
onda alarga o que a GPU representa, o lever DESTE gate tem de ser algo ainda de fora.

## 7. O que só o `ship.sh` pega / o que rodei

- `cargo fmt --all --check` limpo · `clippy --all-targets` limpo nas 2 crates tocadas ·
  `typos` limpo · nenhuma dep nova, `Cargo.toml`/`Cargo.lock` intocados.
- `cargo nextest --workspace --cargo-profile ci-test`: **8899/8899** (excluída a flake
  conhecida `the_cost_of_depth_is_linear_not_explosive`, que **passa isolada** — 1/1).
- LOC: `painter_preview_pipeline_tests.rs` estourou 600 com os gates novos → **split**
  (`painter_preview_ownership_tests.rs`), não isentado. `file_loc_caps` da shell ✓ ·
  `architecture_workspace_file_loc_cap` ✓.
- ⚠️ **Suítes GPU-adapter (`#[ignore]`, o `ship.sh` NÃO roda) — rodei aqui:**
  - `ph2d-render --test layer_compositor_gpu -- --ignored` → **36/36**.
  - `ph2d-host-desktop ... painter_preview_handoff -- --ignored` → **2/2** (o dance
    CPU→GPU→CPU byte-exato com o lever novo).
  - `a_plain_stroke_is_footprint_bound... -- --ignored` → **1,0× (fix) vs 22,3×
    (controle)**.
  **Rode as três na integração.**

## 8. O que smoke-testar

`env PH2D_IMPASTO_SMOKE=1 cargo run -p ph2d-host-desktop --release` (ou qualquer cena que
abra o Painter), num sprite **2048×2048** (ou 4096²):

1. **Pinte um traço simples** (pincel pequeno, uma camada). Deve ficar **fluido** —
   antes caía de FPS. É o cerne desta onda.
2. ⚠️ **A aparência NÃO pode mudar** — o depósito é byte-idêntico; esta onda é custo. Se
   alguma cor/borda mudar, é regressão (e a paridade está gateada).
3. Confira o resto do Painter (máscara, ajuste, impasto, wet) — nada além do preview
   mudou; deve estar como nas Ondas 1–2.

**Não smokado por mim:** só gates headless (com device real); nenhuma janela aberta.

## 9. Aberto (nomeado)

- **Onda 5 (residência de canvas na GPU)** segue por fazer e **não é necessária para
  este problema** (o depósito CPU já é barato uma vez removida a cópia). Vira otimização
  de escala extrema / liberar a CPU. Doc 25 §11.3.
- O caminho **não-trivial** (composite multi-camada na CPU) tinha a mesma cópia por-frame
  e a mesma cura o alcança (a shell possui o buffer; o tool fica dono do `composited`) —
  coberto pela mesma mudança, mas o ganho ali é menor (composite multi-camada é mais raro
  que traço simples) e não foi medido em separado.
