---
name: feedback_disabled_button_still_dispatches
description: "botão \"dimmed\" que registra hit_index continua clicável — o dim é cosmético e a ação destrutiva roda"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6d3039ad-668d-4133-a295-f69680a93752
---

Pintar um botão **dimmed** NÃO o desabilita. Se o `paint` faz `hit_index.register(id, rect)` incondicionalmente, o clique dispara: dispatch resolve o id → `apply_event` arma o comando → o shell executa. O `InteractiveState::Button { state: Normal }` do `populate` nunca vira `Disabled`, então nada no caminho barra.

Achado pela auditoria multiagêntica do módulo de áudio (2026-07-09): os botões de range-op (Trim/Cut/Silence/Fade) eram pintados dimmed sem seleção, mas clicar no "Silence desabilitado" caía em `EditClip::target()`, que faz fallback silencioso pro clipe INTEIRO — zerava o áudio todo. Trim/Cut escapavam só porque `apply_trim`/`apply_delete` têm `if let Some(sel)`; `apply_silence`/`apply_fade` não tinham.

**Why:** o comentário "disabled is a visual hint only" vira dívida latente assim que uma das ações é destrutiva e a API subjacente tem um fallback (`selection.unwrap_or(whole_clip)`). Dois defaults inocentes se compõem num apagão.

**How to apply:** (1) botão desabilitado **não registra hit rect**; (2) segunda camada no `event.rs` — recuse ARMAR o comando quando o pré-requisito falta (testável no seam, ao contrário do hit_index); (3) desconfie de qualquer `unwrap_or(escopo_maior)` atrás de um op destrutivo. Vide [[feedback_tool_unit_green_integration_dead]] (o inverso: pintado+clicável mas morto) e [[feedback_panel_populate_register]].
