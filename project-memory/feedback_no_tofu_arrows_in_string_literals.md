---
name: feedback-no-tofu-arrows-in-string-literals
description: "gate no_tofu_glyphs: nunca use → ← ⌘ dentro de STRING LITERAL em ph2d-editor-core ou shells/desktop — assert!/expect() contam; comentários são livres"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 14afaada-70a5-49d0-a3c1-e84cd2bb2756
---

A fonte bundled (Inter) **não cobre** os blocos Unicode `U+2190..U+21FF` (setas `→ ← ↵ ⇒`) nem `U+2300..U+23FF` (técnicos `⌘ ⎕ ⌥`). Qualquer um deles numa string que o editor renderiza vira **caixa de tofu**. O gate `crates/ph2d-editor-core/tests/no_tofu_glyphs.rs` varre **`ph2d-editor-core` + `shells/desktop`** e falha se achar esses glifos **DENTRO de string literal** (normal ou raw). **Comentários e doc-comments são pulados de propósito** — `//! grid → tint → output` é legítimo e comum nos nós.

A pegadinha que me pegou (Motion M1, integração 2026-07-09): **mensagens de teste são string literal**. `.expect("grid → stagger → oscillator is well-typed")` e `assert!(…, "X → Y")` em `shells/desktop/src/render_loop/motion_bridge_tests.rs` derrubaram o gate. O `cargo test -p` da crate do nó nunca roda esse gate (ele vive em `ph2d-editor-core`), então passou 22 commits e só apareceu no gate da árvore combinada, no integrador.

**Why:** o bug já recorreu **≥3×** (tooltips Cmd/Return da topbar, samples de ListItem/ContextMenu, toasts `"Tool → X"` / `"Theme → X"`) — por isso viraram gate. O `→` é o glifo mais natural do mundo pra descrever pipeline, e ninguém lembra que `expect(...)`/`assert!(...)` são strings, não comentários.

**How to apply:** em `ph2d-editor-core` e `shells/desktop`, dentro de **aspas** use ASCII `->` (ou `Cmd+S`, não `⌘S`). O único não-ASCII seguro como separador é `·` (U+00B7, Latin-1, in-font — a topbar já usa). Fora de aspas (comentário/doc) o `→` é livre. Nas crates de nó (`ph2d-node-*`) o gate não alcança, mas se a mensagem puder migrar pro shell, prefira ASCII. Rode `cargo test -p ph2d-editor-core --test no_tofu_glyphs` (0.05s) antes de fechar trabalho que toque o shell. Vide [[feedback-full-gate-periodically]] (é mais um gate que o loop por-crate esconde) e `docs/UI_Bugs/README.md` §9.19 + DIRETRIZ §4.1 regra 1.
