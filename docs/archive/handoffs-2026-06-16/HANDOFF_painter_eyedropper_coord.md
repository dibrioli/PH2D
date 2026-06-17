═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · Painter eyedropper — 2 itens foundational
Autor: Implementador Painter (sessão 2026-05-31) · reporte de fronteira
Origem: Enio smoke W2.T2.4 — eyedropper não funciona; o do picker fecha o painel
═══════════════════════════════════════════════════════════════════

> ✅ **RESOLVIDO pelo Implementador (2026-05-31, commit `cb976b3`).** Por ordem do
> Enio ("vá em frente a passos largos"), implementei o eyedropper funcional
> end-to-end em vez de só reportar. Toquei 2 arquivos foundational (editor-core
> `dispatch/pointer.rs` + shell `input_dispatch.rs`) — FORA da tua área ativa
> (ph2d-render/ph2d-asset KTX2), sem colisão. Detalhe no commit. Pendente: smoke
> do Enio. **⚠️ 2 REDS PRÉ-EXISTENTES no main (NÃO meus) que vão travar teu ship:**
> (a) shell HR-18 LOC cap: `app_state.rs`=617, `render_loop/inspector_commits.rs`=616
> (assim no HEAD, não-dirty); (b) clippy `ph2d-asset` doc-lint "doc list item without
> indentation" + unused import `TierIndex` (db.rs) — tua WIP KTX2/imageio em
> `ph2d-asset/{db,logical_texture,tier}.rs`. Ambos fora do meu escopo.

╔═══════════════════════════════════════════════════════════════════╗
║ TL;DR (histórico) — Havia DOIS eyedroppers. (1) O da SIDEBAR (meu)   ║
║ só armava um flag órfão sem consumidor → não sampleava. (2) O do     ║
║ PICKER tinha readback funcional mas o popover fechava no clique-fora ║
║ ANTES de samplear. UNIFIQUEI no mecanismo do picker + guard de       ║
║ dismiss (editor-core) + guard de paint (shell). Ambos funcionam.     ║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
O QUE O IMPLEMENTADOR JÁ FEZ (in-scope, commitado)
───────────────────────────────────────────────────────────────────
- Eyedropper da sidebar agora é um ícone compacto (`IconId::EyePencil`)
  ao lado do swatch de cor, na Color row (commit `602f32d`). Pedido do Enio.
  Mesmo id/rota de antes (`PAINTER_SIDEBAR_MODIFIER_SQUARE` → `Click` →
  `PanelEvent::Click` → `PainterUiEdit::ToggleEyedropper` → `eyedropper_armed`).
- Armar NÃO quebra a pintura (`eyedropper_armed` é flag de display; não
  gateia `begin_stroke`).

───────────────────────────────────────────────────────────────────
BUG #1 (FUNCIONAL) — picker eyedropper fecha o popover antes de samplear
───────────────────────────────────────────────────────────────────
Mecanismo funcional EXISTENTE (o do BlenderColorPicker):
  picker eyedropper btn → `store.set_eyedropper_pending(Some(parent))`
  → clique no canvas → dispatch emite `WidgetEvent::EyedropperPick{parent,px,py}`
  (crates/ph2d-editor-core/src/interaction/dispatch/pointer.rs:591-608)
  → shell lê o pixel renderizado e aplica
  (shells/desktop/src/forwarding.rs:37-43 → `vello_pass.read_pixel` +
   `store.set_blender_value(parent, color)`).

ROOT CAUSE do bug: no mesmo Down handler, o **dismiss on click-outside**
roda ANTES da interceptação do eyedropper:
  - pointer.rs:570-584 — se `picker_target().is_some()` e o clique está
    FORA do outer rect do picker → `set_picker_target(None)` (fecha).
  - pointer.rs:591-608 — só DEPOIS checa `eyedropper_pending()` p/ emitir o pick.
Clicar no canvas pra samplear cai no dismiss → o picker fecha.

FIX recomendado (1 linha, editar editor-core dispatch — foundational):
  guardar o dismiss da pointer.rs:581 com `&& store.eyedropper_pending().is_none()`
  (ou reordenar: interceptar o eyedropper ANTES do bloco de dismiss). Assim,
  enquanto um pick está armado, clique-fora NÃO fecha — cai na interceptação
  e samplea. Verificar se o dismiss também deveria limpar `eyedropper_pending`
  no caminho normal (hoje só limpa `picker_target`).

───────────────────────────────────────────────────────────────────
BUG #2 (FUNCIONAL) — eyedropper da sidebar não samplea (flag órfão)
───────────────────────────────────────────────────────────────────
`eyedropper_armed` (no `PainterParams`/snapshot) NÃO tem consumidor: nada
emite `EyedropperPick` nem lê pixel com base nele. Logo o ícone da sidebar
arma o highlight mas não samplea.

RECOMENDAÇÃO (unificar — padrão-ouro): em vez de manter o flag órfão,
roteie o eyedropper da sidebar pelo MESMO mecanismo do picker (já funcional):
  - Opção A (mínima, dá pra fazer no painel via `host.store_mut()` — API
    pública, sem editar editor-core): no `Click` do ícone, abrir o picker
    como o swatch faz (`set_picker_target(PAINTER_COLOR_THUMB)` +
    `set_blender_value(INSP_BLENDER_PICKER, cor)`) E armar
    `set_eyedropper_pending(Some(INSP_BLENDER_PICKER))`. Com o BUG #1
    corrigido, clicar no canvas samplea → `set_blender_value(picker)` → o
    `painter_bridge` (picker_target==PAINTER_COLOR_THUMB) aplica `SetColorSrgb`
    no Painter. Custo: o popover do picker aparece (não é "quick").
  - Opção B (UX melhor, FOUNDATIONAL — shell): readback direto pro Painter
    sem abrir o picker — novo caminho no shell que, quando o eyedropper da
    sidebar está armado, lê o pixel e aplica `PainterUiEdit::SetColorSrgb`
    direto no tool (+ desarma). Não precisa do picker como intermediário.

  Decisão tua (dono do shell readback + dispatch). Se preferir B, posso
  deletar o `eyedropper_armed`/rota da sidebar e deixar só o teu caminho —
  é só me sinalizar. Se A, o painel já tem `host.store_mut()` disponível.

───────────────────────────────────────────────────────────────────
NICE-TO-HAVE (foundational, editor-core)
───────────────────────────────────────────────────────────────────
- Ícone: usei `IconId::EyePencil` (melhor disponível); um SVG dropper/pipeta
  dedicado + `IconId` novo (ordem alfabética — cuidado com o índice de
  `ICON_CMDS_BY_ID`, gate `enum_order_matches_svgs`) leria melhor como
  "eyedropper". Editar `docs/design/icons/` + `icons.rs` é editor-core = teu.

───────────────────────────────────────────────────────────────────
ESCOPO
───────────────────────────────────────────────────────────────────
Tudo aqui toca editor-core (`dispatch/pointer.rs`, `icons.rs`) e/ou shell
(`forwarding.rs`, readback GPU) = foundational. Implementador PAROU e
reportou (§0 #2). Itens da sidebar (ícone/arm/rota) já estão prontos e
verdes; só falta o readback funcional que é teu.
═══════════════════════════════════════════════════════════════════
