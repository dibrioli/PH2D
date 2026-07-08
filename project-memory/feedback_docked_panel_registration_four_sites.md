---
name: feedback_docked_panel_registration_four_sites
description: Registrar painel docado novo exige 4 sites além da crate; 2 gates verdes mascaram o gap
metadata:
  type: feedback
---

Um `ph2d-panel-*` novo **compila, linka e passa nos testes** enquanto está **morto na tela** — a UI só aparece quando TODOS estes sites estão feitos (diagnosticado 2026-07-08 no `ph2d-panel-timeline`, 3 fixes até aparecer):

1. **crate + panel-sync + `EXPECTED_TYPED`** em `ph2d-panel-registry-init` (push block gerado; contador manual).
2. **Feature no SHELL** (`shells/desktop/Cargo.toml`): o shell puxa `panel-registry-init` com `default-features=false` e re-liga cada painel via **proxy-feature própria** — `panel-<slug> = ["ph2d-panel-registry-init/panel-<slug>"]` + entrada no `[features] default` do shell. Sem isso o `reg.push(Painel)` é **compilado fora** → nunca entra no `PANEL_REGISTRY`. Uma dep DIRETA do painel no shell (pra publicar snapshot) NÃO liga essa feature (resolver-v2). **Este é o site mais fácil de esquecer.**
3. **z-order walk** em `crates/ph2d-editor-core/src/screens/hero/paint.rs` (~l.270-300): a iteração que chama `panel.paint()` caminha `panel_z_order()` + uma **lista de fallback HARDCODED**. Painel registrado+visível **ausente da lista nunca é pintado** (o comentário dos `MOTION_*_PANEL` avisa exatamente isso). Adicione `ids::<PANEL>` lá.
4. **Visibilidade** em `hero.rs`: `default_panel_visibility()` + `canonical_panel_id()` (o `set_panel_visible` filtra por canonical → sem a entrada, o toggle é no-op).

**Por que mascara:** o teste `EXPECTED_TYPED` do registry-init compila com os defaults DELE (todos os painéis on) → verde mesmo com a feature do shell off. `cargo check -p <shell>` também passa (a crate é dep direta). Só o **run visual** revela. Verifique os 4 sites ao adicionar painel; ao debugar "painel não aparece", cheque na ordem 2→3→4→geometria.

**Widgets:** pinte pela **fonte da verdade = Widget Gallery** (`ph2d-editor-core/src/widget/showcase/*.rs`), não improvise dimensões. Toggle canônico = switch `TypeToken::Xl3` largura × `Density::Compact` altura (pílula 2:1) + label à esquerda; `Toggle::new(id,label).on().state()`. Rect quadrado → vira círculo sem trilho. Ver [[feedback_ui_source_of_truth_gallery_inspector]].
