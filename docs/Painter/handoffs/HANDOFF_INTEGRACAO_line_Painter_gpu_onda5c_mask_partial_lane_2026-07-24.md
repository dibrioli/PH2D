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

## 9b. Continuação (2026-07-25) — smoke do Enio: dois defeitos de QUALIDADE da máscara

O smoke aprovou o FPS mas achou dois defeitos VISUAIS (a máscara é translúcida ⇒ revela o
que a tinta opaca esconde). Ambos mask-route, integram com o resto.

**(i) Retângulos/blocos translúcidos (commit `2da916c99`).** O upload PARCIAL da GPU do
overlay translúcido deixava costuras no device (byte-idêntico no cache da CPU e num sim
headless — artefato só-wgpu). Bissectado com `PH2D_PAINT_FULL_UPLOAD=1`. Fix: a máscara
mantém o **composite PARCIAL** (o ganho) mas força **upload CHEIO** (`preview_upload_bbox =
None` com scratch vivo) — ~6 ms @ 2048², sob os 16,7 ms ⇒ **60 fps**, byte-idêntico à
referência. Gate estendido (`a_mask_stroke_takes_the_partial_lane…` afirma `bbox=None` p/
máscara). Doc 25 §13.5. **Aberto:** isolar a costura do upload parcial no device (importa a
4096², onde o upload cheio estoura o orçamento).

**(ii) A borda ENDURECE sob muitas passadas (commit `600a79606`).** A cobertura da máscara
acumulava como PRODUTO entre traços (`255·m^N`, prova aritmética por 3 agentes) ⇒ o feather
colapsa numa borda dura/serrilhada. É o build-up per-dab COMPARTILHADO (pintura normal
endurece igual; a máscara só revela). Fix (escolha do Enio: **Envelope**): traços se
combinam por `min` (Paint) / `max` (Erase) num buffer por-traço, idempotente ⇒ N passadas =
1, borda nunca endurece; UMA passada byte-idêntica (fingerprint intacto). **Escopo: só a
rota da máscara** (`stamp_dabs_mask` + `begin_mask_stroke`/`fold_mask_stroke` + o campo
`mask_stroke_rgba`); caminho per-dab e build-up da pintura normal INTOCADOS. Gate red-first
`the_mask_feather_does_not_harden_across_passes` (mutação-provado). Doc 25 §13.6.
**Trade documentado:** passadas rápidas idênticas convergem (não empilham); aprofunde com
pincel mais forte/lento ou traços sobrepostos.

**Arquivos (ii):** `crates/ph2d-tool-painter/src/tool/paint.rs` (campo `mask_stroke_rgba`) ·
`…/paint/state_default.rs` (init) · `…/paint/mask.rs` (`begin_mask_stroke`/`fold_mask_stroke`) ·
`…/paint/stroke_lifecycle.rs` (chama `begin_mask_stroke`) · `…/paint/stamp_route.rs`
(`stamp_dabs_mask` ramo envelope) · `…/paint/tests.rs` (o gate). **Sem schema, sem contrato
congelado**; `paint.rs` no teto de 700 LOC.

**Rodei:** `cargo test -p ph2d-tool-painter` **822 pass / 44 ign** (inclui os 2 gates novos +
os 35 de máscara) · clippy `--all-targets` limpo · fmt limpo · `architecture_workspace_file_loc_cap`
verde · render-and-look confirma 15 passadas == 1 (liso).

## 9c. Smoke pendente (Enio)

Build padrão (sem env), 2048², os 3 cenários de máscara, com MUITAS passadas no mesmo lugar:
a borda tem de continuar **lisa** (não serrilha/endurece) e a 60 fps. Se ainda serrilhar,
`PH2D_PAINT_PERF=1` mostra o branch; mas o gate + o render headless já provam 15==1 passada.

## 10. Aberto (nomeado)

- **`impasto=true` num stack de UMA camada** ainda seria `FULL-composite` na CPU (o
  `try_drive` bowa em stack trivial; o log do Enio deu `impasto=false`, então NÃO é o caso
  dele). Se um smoke futuro mostrar `branch=FULL-composite impasto=true` num traço simples,
  é a mesma classe (o passe de luz da GPU não alcança o stack trivial) — wave própria.
- O agregador `PH2D_PAINT_PERF` (§7) — manter/remover é decisão de ship.
