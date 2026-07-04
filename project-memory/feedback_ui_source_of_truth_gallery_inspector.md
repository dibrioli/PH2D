---
name: feedback-ui-source-of-truth-gallery-inspector
description: UI nova deve espelhar widget-gallery (canon dos widgets) + inspector (padrão painel-com-seções); não improvisar chrome
metadata: 
  node_type: memory
  type: feedback
  originSessionId: c543cb41-2db3-4418-990a-4f3ef501efbc
---

Ao construir QUALQUER UI nova (painel, seções de params, controles), a **fonte da
verdade é o `ph2d-panel-widget-gallery`** (canon de todos os widgets: Checkbox/Slider/
NumberInput/Dropdown showcased) **+ o `ph2d-panel-inspector`** (padrão de painel-com-
seções: `sections/*.rs`, row-builders `check_row`/`number_row`/slider rows,
`paint_section_separator`, scroll via `store.panel_scroll`, macro `live_section!`).

**Why:** o Enio disse explicitamente (2026-06-06) "a fonte da verdade da UI encontra-se
no painel widget gallery e no inspect". Improvisar chrome/controles diverge do design
system e do canon — retrabalho garantido.

**How to apply:** antes de pintar UI nova, abra esses dois painéis e ESPELHE a estrutura
(não invente layout/widget). O Inspector é o molde de painel-de-edição-de-params; a
gallery é onde confirmar qual widget existe e como se usa. Aplica-se ao Brush Studio
(W5) e a todo painel futuro. Ver [[feedback-app-ui-english-only]] e o HR-15 (tokens/i18n).
