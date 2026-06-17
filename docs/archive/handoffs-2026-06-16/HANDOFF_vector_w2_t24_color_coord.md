═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Vector · W2 T2.4 Color — Coord half DONE + apply-fill ask
Autor: Coordenador · 2026-06-02
═══════════════════════════════════════════════════════════════════

## §0 — Correção de rota (importante)
Minha 1ª chamada ("swatch flutuante no chrome") estava ERRADA em 2 frentes:
o swatch flutuante top-right foi REMOVIDO há tempos ("wrong home", hero.rs), e
o paint de `FloatingPanel` de tool foi APOSENTADO em 2026-05-17. O home correto
é um **panel crate dedicado** (padrão atual, igual ao Painter sidebar). Então a
tua opção (a) original ("scaffolda Coord-B antes?") era a certa.

## §1 — Coord half ENTREGUE (commits `a84af91` + `f6ba1d3`, gate-clean)
- **Dispatch genérico de picker-swatch** (`a84af91`): `WidgetStore::is_picker_swatch`
  + `register_picker_swatch`. Qualquer panel que pinta um `ColorSwatch` e registra
  o id → Down abre o Blender picker. Aposentou o special-case `PAINTER_COLOR_THUMB`.
- **Crate novo `ph2d-panel-vector-inspector`** (`f6ba1d3`): panel docado mínimo
  (chrome + 1 row "Fill" com o picker swatch). Right-dock (slot Inspector); visível
  quando vector_select/vector_direct ativo (o shell esconde o Inspector real).
  Registry-init wirado (panel-sync + count test), z-order, feature default.
- **Shell `vector_inspector_bridge`**: visibilidade + **read-back** (cor escolhida no
  picker → swatch + `App.vector_fill_color`) + publish pro panel.

**Smoke (teu/Enio):** tool vetorial ativo → Inspector aparece → clica no swatch "Fill"
→ picker abre → swatch reflete a cor. (A cor ainda NÃO preenche regions — ver §2.)

## §2 — Tua parte: apply-fill (a divisão que tu reservaste, "meu")
Falta só **aplicar a cor às regions selecionadas**. Contrato:
- **Entrega** em `ph2d-vector-doc` (junto de `VectorSelection`/undo, replay-safe):
  ```rust
  pub fn apply_fill_to_selection(
      committed: &mut [Ph2dVectorAsset],
      selection: &VectorSelection,
      rgba: [u8; 4],
  );
  ```
  Para cada `selection.networks[i]`: `insert_fill` (ref fresco) + `SetRegionFill`
  push no `edit_log` daquele asset (logado → `revert_last_op` desfaz, como
  combinaste). sRGB8 → o teu FillSolid (converte na fronteira).
- **Eu wiro** a chamada no hook marcado em `vector_inspector_bridge.rs` (passo
  `&mut committed` + `&selection` pro bridge e chamo no bloco do read-back). Me
  avisa quando o helper landar — é 1 linha minha.

## §3 — Gradient (decisão anterior reafirmada)
Linear 2-stop = task foundational MINHA (ADR FillSolid→enum + schema bump +
bounded_decode + cook-hash). Depois do solid fechar/smokar. Não te bloqueia.
═══════════════════════════════════════════════════════════════════
