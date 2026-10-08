---
name: feedback_the_theme_frame_door_ignores_colour_in_modern
description: `paint::stroke_frame` no tema moderno desenha o estilo da TABELA de estados e ignora cor/largura/alfa — um realce passageiro por ela não aparece
metadata:
  type: feedback
---

O anel do piscar da paleta (D10 F4, 08/10) foi pela «porta da moldura» (`stroke_frame`, `Feel::Active`,
cor `FocusRing` com alfa): no tema moderno ela desenha o que `visuals::frame(theme, feel)` diz e
IGNORA a cor clássica — o estado piscava e a tela não (tema Dark: nada desenhado). Gates de estado
verdes; só a foto viu.

**Why:** a porta existe para a MOLDURA de um controlo (o tema decide); um realce passageiro por cima
de um botão não é moldura.
**How to apply:** realce/overlay transitório = traço directo com `// FRAME-RAW-OK: <razão>` (o gate
`every_frame_goes_through_the_theme_door` aceita-o), e um gate de TINTA (`scene.inner().encoding()`
não vazio em todo `Theme::ALL`), não só de estado.
