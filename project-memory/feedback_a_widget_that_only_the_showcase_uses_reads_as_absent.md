---
name: feedback-a-widget-that-only-the-showcase-uses-reads-as-absent
description: O chip com `×` existia há meses em `widget/tag.rs` e eu escrevi no plano que não existia — o único consumidor era o showcase, e um `grep` pelos painéis lê zero.
metadata:
  type: feedback
---

Medido em 2026-09-13 (`line/components`, Tags W3a). O plano da secção *Tags* dizia, por escrito:
*«não existe widget de chip — os chips serão botões pequenos mais um ícone `×`»*. Existia:
`ph2d_editor_core::widget::tag` — `Tag` (a.k.a. Chip), `WidgetKind::Tag`, `TagTone` de cinco tons,
`InteractiveState::Tag`, hover despachado, e um **`close_rect()`** que dá ao `×` uma zona de acerto
própria, que é exactamente o que separa *tirar esta tag* de *carregar no chip*.

**Why:** o **único** consumidor era `widget/showcase/actions.rs`. Um levantamento feito a partir dos
painéis (*«que painel desenha um chip?»*) lê **zero** e conclui ausência — e é essa a forma natural
de procurar, porque é onde o trabalho vai acontecer. ⚠️ A mesma lei que este repo já pagou noutro
sítio (*«uma ausência afirmada pelo endereço do código é um palpite com cara de medição»*, o
`paint_hover_tooltip`), num tema diferente: aqui a ausência foi afirmada pela POPULAÇÃO DE
CONSUMIDORES em vez de pelo catálogo.

**How to apply:** antes de escrever *«não existe widget para isto»* num plano, leia o **catálogo**:
`WidgetKind` em [`widget/skin/kind.rs`](crates/ph2d-editor-core/src/widget/skin/kind.rs) (20
variantes) e os ficheiros de `crates/ph2d-editor-core/src/widget/`. ⚠️ E quando o único consumidor
for o **showcase**, conte com o preço que este caso trouxe: o widget foi escrito com as portas certas
(`Tag::visual((estado, hover))`) e **sem os atalhos que os irmãos com uso real ganharam** — não há
`store.tag_visual()` como há `button_visual()`/`dropdown_visual()`, porque nunca houve quem o pedisse.
*Um widget sem consumidor tem a API por estrear, não errada.*

Irmãs: [[feedback_a_surface_that_only_counts_is_usually_missing_a_datum_not_a_widget]] ·
[[feedback_widget_is_done_when_a_test_clicks_it]] ·
[[reference_topic_control_design_hazards]]
