---
name: project_checkbox_in_a_frame_like_godot_2026_11_13
description: "Dono 13/11 — a caixa de marcar do APP TODO passa a morar numa MOLDURA (como a Godot), e o losango de animação segue o padrão dos campos (acima do canto superior direito)"
metadata:
  node_type: memory
  type: project
  originSessionId: e9530d79-55b6-4bb2-aabe-5e8c592f9826
  modified: 2026-11-13T21:54:00.368Z
---

Dono, 13/11/2026, no smoke do cartão Projectile Motion (GEF 06), ao ver o losango colado ao rótulo «Faces path»:
*«a solução é mudar o checkbox para todo o app. Como na Godot o checkbox deve ficar dentro de uma moldura e o
losango entra no padrão (acima do canto superior direito)»* — e, se houver um ficheiro central, corrigir o app todo.

**Why:** a caixa de marcar dos cartões pintava o losango logo a seguir ao texto (colado quando o rótulo enche a
célula); os outros campos já têm a caixa-moldura com o losango acima do canto superior direito. Um idioma só.

**How to apply:** a caixa central é `ph2d_editor_core::widget::paint_card_check` / `card_check_rect` (T64) e a
linha `property_row::paint_check_row`: mudar ALI, varrer por CONSUMIDOR ([[feedback_changing_a_shared_widgets_arithmetic_is_swept_by_consumer_not_by_call_site]]),
foto antes/depois de vários cartões. Continua checkbox, nunca toggle ([[feedback_the_app_standard_is_a_checkbox_not_a_toggle]]).
