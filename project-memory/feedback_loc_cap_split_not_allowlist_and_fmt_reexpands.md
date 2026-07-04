---
name: feedback-loc-cap-split-not-allowlist-and-fmt-reexpands
description: "LOC-cap gate — split by responsibility (not allowlist); and cargo fmt re-expands condensed multi-arg calls so don't trim to exactly 600"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 93ee2f69-a04d-4352-82ac-69624f6a510d
---

Quando uma feature empurra arquivos sobre o cap de LOC (`architecture_panel_loc_cap` /
`architecture_workspace_file_loc_cap`, 600 default; alguns com baseline em `FILE_OVERAGE_OK`):

**Why:** o gate conta crescimento cumulativo; o objetivo declarado é "drive entries DOWN, never up".
A correção certa é **split por responsabilidade em módulos-irmão**, NÃO adicionar entrada no allowlist.

**How to apply:**
- Extraia o código NOVO (teu) para um sibling: ex. `texture/shape.rs`, `paint/shape_settings.rs`,
  `paint/stamp_route.rs` (tool), `paint_shape.rs` (painel). Métodos `impl PainterTool` num arquivo
  novo continuam acessíveis se forem `pub(super)` (privacidade é por-módulo — um `fn` privado movido
  para outro módulo deixa de ser chamável do arquivo original; use `pub(super)`).
- Campos de struct (`PaintState`/`BrushSettings`) **não dá pra mover** (ficam na def); então mover só
  setters pode não bastar — às vezes precisa extrair uma região maior (ex. todos os setters de uma seção).
- **⚠️ `cargo fmt` RE-EXPANDE chamadas multi-arg condensadas.** Se você apertar um arquivo para
  EXATAMENTE 600 e depois rodar `rustfmt`/`cargo fmt`, ele volta a quebrar as chamadas em várias linhas
  e estoura o cap de novo. **Deixe margem (~590) OU rode fmt ANTES de medir/cortar.** Rode o gate de LOC
  **depois** do fmt, não antes.
- Allowlist (`FILE_OVERAGE_OK`) só para crescimento legítimo inevitável de um arquivo já-grande-frozen
  (ex.: +1 variante numa enum central tipo `action_bus.rs` — um action-enum cresce com features). Bump
  **mínimo** (+1) com comentário de sign-off do Coordenador; nunca para evitar um split que dá pra fazer.

Relacionado: [[feedback-painter-inefficiency-4-causes]] (gates executáveis), [[feedback-ci-direct-lint-gates-and-fmt-skew]] (use o toolchain pinado p/ fmt: `rustup run 1.95 cargo fmt`).
