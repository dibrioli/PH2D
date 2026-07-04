---
name: feedback-hier-companion-dispatch-allowlist
description: "Companion NodeIds de hierarchy row (eye/expand/lock/group/icon) precisam estar na allowlist em interaction/dispatch/pointer.rs OU clicks são silenciosamente dropados — o painel não registra esses ids no WidgetStore (sem &mut store no paint), is_focusable rejeita, dispatch nunca emite Click."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8ec2f80f-2868-448b-a785-053233ede789
---

Ao adicionar UM NOVO companion hit em hierarchy row (qualquer bit em [[user-role]] ECS toggle / focus / etc), DOIS sites precisam ser estendidos em [`crates/ph2d-editor-core/src/interaction/dispatch/pointer.rs`](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-editor-core/src/interaction/dispatch/pointer.rs):

1. **Down handler** (~linha 642): allowlist que captura companion como "ephemeral button" (set_active + active_rect). Sem isso o click vai pra `is_focusable`, é rejeitado (id não está no WidgetStore — painter só tem `&mut HitIndex`), e Click(id) nunca é emitido pelo Up branch.
2. **Right-click strip** (~linha 370): `hier_row_id = hit_id.and_then(strip_companion)` precisa cobrir o novo bit pra que right-click no companion ainda abra o context-menu da row pai.

**Why:** descobri esse padrão depois de implementar Lock+Group+Icon companions completamente (UI, bus, drain, ECS) e ver Enio reportar "não funciona". Stack: 4 arquivos perfeitos, 1 allowlist hardcoded de 2 entries (eye+expand) bloqueando tudo silenciosamente. Commit `4739e2c` documenta o fix.

**How to apply:** quando adicionar bit companion (LOCK_TOGGLE_BIT, GROUP_TOGGLE_BIT, ICON_COMPANION_BIT, EYE_TOGGLE_BIT, EXPAND_COMPANION_BIT são os atuais), grep `hier_eye_companion_to_row` em [pointer.rs](file:///Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva/crates/ph2d-editor-core/src/interaction/dispatch/pointer.rs) e adicione o novo `hier_<x>_companion_to_row(id).is_some()` em AMBOS os sites encontrados.

Pode valer um arch-gate: teste que enumera todos os `hier_*_companion_to_row` em ids.rs e checa que cada um aparece na allowlist do pointer.rs.
