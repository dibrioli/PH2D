# F50 — Boolean e Convert saem de «Advanced» para a fila do topo, à Figma (dono 02/11)

> Pedido do dono (02/11, depois do smoke OK da F49): *«Vamos continuar melhorando a UI e a funcionalidade de nossas
> tools e seções dentro do vector como fizemos com stroke, fill, filter e Effects. Busque saber como figma, inkscape
> e ilustrator arrumam as coisas. Tenho preferência pelo jeito figma. Dentro de advanced temos várias seções que
> precisam ser revisadas e melhoradas. Escolha algumas para começar»*. Entre três blocos propostos (Boolean +
> Convert · Path Effects em lista · Blend + Morph) o dono escolheu **Boolean + Convert** (recomendado).

## 1 — O que existe (inventário de 02/11, HEAD `ad78595d4a`)

O grupo `VECTOR_SECTION_ADVANCED` (`ph2d-panel-vector/src/ids/vector_aba.rs:25`, nasce dobrado) pinta, em
`paint_body_plan.rs:~381-420`: Convert · Blend · Morph · Envelope · Pattern on Path · Contour · Boolean. Nenhum
controlo morto encontrado. Ordem do corpo: Layout, Appearance, Fill, Pattern, Stroke Pattern, Brush, Connector, Text,
Font, Paragraph, Text on Path, Axes, Vertex, Effects, Filters, **Advanced**, Widget Skin, States, Morph States.

| Secção | Controlos hoje | Despacho | Auto Key |
|---|---|---|---|
| Convert (`paint_expand.rs:47`) | botão «Outline Stroke» (`VECTOR_EXPAND_OUTLINE_STROKE` → `ph2d-app-vec/src/expand.rs:49`); Ctrl+Alt+O (`tool_keys.rs:82`) | — | não |
| Boolean (`paint_boolean.rs:21`) | «Live» Off/On (`set_bool_live_on`); «This Shape» Union/Subtract/Intersect/Exclude (só num grupo, `vec_bool_shape`); oito botões (Union…Merge) + «Apply Boolean» | `fase_bus_clicks.rs:208,324`; a mesma família na barra ④ (`bars_actions.rs:110-183`) | não |

## 2 — Como os três arrumam (pesquisa 02/11)

- **Figma (UI3):** fila de ACÇÕES DA SELECÇÃO no topo do painel Design — Create component, **Use as mask**, menu
  **Boolean**, Edit object; o que não cabe vai para um «⋯». Booleana = **grupo booleano VIVO** na Layers (as formas
  ficam dentro, cada uma ainda move/escala/roda; a operação troca-se depois pelo mesmo menu). Atalhos Union ⌥⇧U,
  Subtract ⌥⇧S, Intersect ⌥⇧I, Exclude ⌥⇧E. **Flatten** (⌥⇧F) e **Outline stroke** (⌘⌥O) são comandos de UMA vez
  (menu de contexto). Ungroup desfaz o grupo. Fonte: help.figma.com/hc/en-us/articles/360039957534 ·
  …/33052305733015 (outline stroke) · figma-signup.helpjuice.com (UI3, flatten).
- **Illustrator:** Pathfinder (Shape Modes Unite/Minus Front/Intersect/Exclude + Expand; Alt+clique = compound
  VIVO; Divide/Trim/Merge/Crop/Outline/Minus Back) e, no Properties, «Quick Actions» contextuais; Effect = vivo ·
  Object = destrutivo; Expand fixa.
- **Inkscape:** Path ▸ Union/Difference/Intersection/Exclusion/Division/Cut Path, Combine/Break Apart, Flatten,
  Stroke to Path — todos de uma vez; o vivo é a LPE «Boolean Operation».
- Síntese adoptada: (1) booleana e máscara na fila do topo, nunca numa área «Advanced»; (2) booleana VIVA como grupo
  na Hierarchy, operação trocável; (3) o permanente é UMA acção explícita (Flatten / Outline Stroke), marcada como
  tal.

## 3 — Decisões técnicas a tomar na implementação (delegadas, padrão-ouro)

- Medir antes: o que o «Live» de hoje já exprime (o grupo booleano vivo existe? `bool_live.rs`, `vec_bool_shape`) e o
  que os oito botões da ④ fazem — a composição pode já cobrir o grupo vivo da Figma; não reconstruir.
- A fila do topo é chrome partilhado: ver se o Inspector já tem uma fila de acções da selecção (Figma) antes de criar.
- Divide/Trim/Merge/Crop (o resto dos oito) — lugar à Figma = o «⋯» ou o menu Boolean abaixo de um separador.
- Atalhos ⌥⇧U/S/I/E/F (conferir colisões em `tool_keys.rs`/`file_chords`); Ctrl+Alt+O fica.
- Gates vermelhos primeiro; só tokens e i18n; app só em inglês; tooltip em Plain English; Auto Key onde houver campo.
- Próxima cena de smoke = **53**, fotografada antes do smoke.
- Depois deste bloco (ordem proposta): Path Effects em lista (Contour, Pattern on Path, Envelope) → Blend + Morph.
