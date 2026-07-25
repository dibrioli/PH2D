# Handoff de integração — `line/Painter` · Onda 5c (o traço de máscara toma a via parcial)

**Para:** o agente integrador (DIRETRIZ §1.5.9). **Data:** 2026-07-24.

> ⚠️ Esta branch empilha, em ordem: Ondas 1-2 (compositor GPU), a transferência sRGB do
> Wet Paint, a Onda 5a (a pintura trivial para de copiar o canvas), a 5b (upload parcial da
> camada) e esta (5c). Todas integram juntas; handoffs próprios de cada uma ao lado deste.

## 1. Identidade

| | |
|---|---|
| branch | `line/Painter` |
| commits desta onda | **2** — `9835b3100` (diagnóstico: nomeia o braço do drain) + `d5cfc8aa7` (o fix) + 1 de docs |

## 2. O que muda, em uma frase

Pintar uma máscara (qualquer dos 3 cenários: pintar a máscara · pintar com máscara ·
pintar após limpar) fazia um **recompose de tela inteira + upload cheio de 16 MiB por
quadro pintado** (medido: 17 ms preview + 6,9 ms upload @ 2048², CPU — a queda de FPS que
o Enio reportou). Agora um dab de máscara toma a **via parcial**, re-tingindo só a região
do dab, byte-idêntico a um recompose cheio.

## 3. A causa e a cura (resumo; detalhe no doc 25 §13)

`take_preview_arc` (runtime.rs) fazia `force_full = mask_scratch_active()`. Mas o overlay
de proteção (`apply_mask_overlay`) é **per-pixel** e um dab muda a cobertura só no próprio
`dirty_rect`; as mudanças globais de tint (cor / canvas-op / 1º scratch) **já** invalidam o
composite. `force_full` era super-cautela. Removido; o braço parcial ganhou
`apply_mask_overlay_region` (re-tint só da região), que **compartilha o kernel per-pixel
`tint_pixel`** com o overlay cheio.

## 4. Arquivos (só a `ph2d-tool-painter` + o diagnóstico da shell)

- `crates/ph2d-tool-painter/src/tool/paint/mask.rs` — `apply_mask_overlay_region` novo +
  `tint_pixel` extraído (kernel único, compartilhado com `apply_mask_overlay`).
- `crates/ph2d-tool-painter/src/tool/runtime.rs` — `force_full` removido; o braço parcial
  chama `apply_mask_overlay_region`. (Também carrega o tag de braço do diagnóstico.)
- `crates/ph2d-tool-painter/src/tool/mod.rs` — `enum DrainBranch` + campo/accessor (diag).
- `crates/ph2d-tool-painter/src/lib.rs` — re-export `DrainBranch` (diag).
- `crates/ph2d-tool-painter/src/tool/paint/tests.rs` — o gate novo.
- `shells/desktop/src/render_loop/{painter_bridge,paint_perf}.rs` — o agregador
  `PH2D_PAINT_PERF` (só-diagnóstico; ver §7).

## 5. Contrato / schema / foundational

- **Nenhum contrato congelado, nenhum schema** (`PROJECT_SCHEMA` fica **29**).
- **Nenhum foundational** (a 5b já mexeu `LayerPixels`; esta onda não toca `ph2d-render`).
- Superfície pública nova, toda ADITIVA: `ph2d_tool_painter::DrainBranch` (+ o accessor
  `PainterTool::preview_drain_diag`) — só o `painter_bridge` diagnóstico os usa.

## 6. Símbolos que podem COLIDIR

Nada numerado. `DrainBranch` é nome novo no re-export de `tool::{...}` (só ADIÇÃO à lista).

## 7. O `PH2D_PAINT_PERF` — manter ou remover?

O agregador (`paint_perf.rs` + os marks no `painter_bridge`) é **só-diagnóstico, custo
zero quando a env não está setada** (o `perf_t0` nasce `None`). Deixei-o **no build** porque
o Enio o usa para confirmar o fix (deve ver `branch=partial-composite` e `frame ~16.7 ms`
nos cenários de máscara). **Decisão do integrador/Enio:** manter como infra de perf do
Painter (útil pra próxima regressão) ou remover no ship. Se remover: os 2 arquivos da shell
+ o `DrainBranch`/`preview_drain_diag`/`last_drain_branch` do tool + o re-export. O **fix**
(5c) **não depende** de nada disso — mask.rs + o `force_full` do runtime.rs bastam.

## 8. O que rodei

- `cargo fmt --check -p ph2d-tool-painter -p ph2d-host-desktop` limpo · `clippy
  --all-targets` limpo na `ph2d-tool-painter`.
- `cargo test -p ph2d-tool-painter --release`: **821 passed, 44 ignored** (inclui o gate
  novo `a_mask_stroke_takes_the_partial_lane_byte_identical_to_a_full_recompose`).
- Gates de preview da shell (binary unittests): screen-truth 5/5 + upload-plan + ownership
  verdes; **o handoff GPU real `the_screen_survives_the_gpu_to_cpu_producer_handoff`
  (`--ignored`) passou no device (RTX)**.
- 2 mutações provadas RED→GREEN (§4 do commit `d5cfc8aa7`).
- **NÃO rodei** o gate batched completo do fechamento da linha (nextest --workspace +
  clippy --all-targets + machete/deny/typos) — isso é do fechamento da JORNADA, não desta
  onda. Ver §9.

## 9. O que smoke-testar (pendente de Enio)

Sprite **2048²**, Painter aberto, com `env PH2D_PAINT_PERF=1`:

1. **Pintar a máscara** (Mask mode) — deve estar fluido; a linha WORST deve mostrar
   `branch=partial-composite` (não `FULL-composite`) e `frame p50 ~16.7 ms`.
2. **Pintar com a máscara presente** (imagem, proteção viva) — idem.
3. **Limpar/remover a máscara e pintar** — idem.
4. ⚠️ **A aparência NÃO pode mudar** — o overlay de proteção é byte-idêntico ao recompose
   cheio (gate); o tint só é re-derivado na região do dab. Se alguma cor/borda de proteção
   piscar ou ficar velha fora do dab, é regressão (e o gate a pegaria).

## 10. Aberto (nomeado)

- **`impasto=true` num stack de UMA camada** ainda seria `FULL-composite` na CPU (o
  `try_drive` bowa em stack trivial; o log do Enio deu `impasto=false`, então NÃO é o caso
  dele). Se um smoke futuro mostrar `branch=FULL-composite impasto=true` num traço simples,
  é a mesma classe (o passe de luz da GPU não alcança o stack trivial) — wave própria.
- O agregador `PH2D_PAINT_PERF` (§7) — manter/remover é decisão de ship.
