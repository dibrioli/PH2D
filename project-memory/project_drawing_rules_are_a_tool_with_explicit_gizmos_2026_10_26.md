---
name: project_drawing_rules_are_a_tool_with_explicit_gizmos_2026_10_26
description: "Dono 26/10 (smoke da cena 6, line/BlenderConstraints): regras de desenho (CAD) = TOOL NOVA na barra lateral esquerda + gizmos explícitos como os das Shapes; nunca lista no Inspector"
metadata:
  node_type: memory
  type: project
  originSessionId: c048c25b-b983-4a8a-991c-8f0278eb08c6
  modified: 2026-10-26T19:51:02.431Z
---

Smoke da cena 6 «Régua de desenho» (26/10/2026): a medida animada funcionou, mas o dono REPROVOU a forma.
Palavras dele: «Milhões de regras no painel, nomes repetidos, contra intuitivo» · «não existe nada que possa ser
entendido não. Apenas alguns pontos de cores diferentes» · «Se for para fazer isso que seja feito de forma muito bem
desenhada e explícita como os gizmos das Shapes e por meio de uma nova tool na barra lateral esquerda, não com um
painel no Inspector completamente incompreensível».

**Why:** a secção «Drawing Rules» do Inspector listava 33 linhas («Vertical» ×10…) sem dizer QUAL lado; os traços
coloridos e losangos do `cad_marks` no canvas não se leem; não havia forma de CRIAR uma regra à mão (só a guia da
caneta cria H/V/coincidente) — logo a medida animada estava fora do alcance do artista.

**How to apply:**
- A regra cria-se, vê-se, edita-se e apaga-se NO CANVAS, por uma tool própria na barra esquerda do Vetor; o selo de
  cada regra desenhado junto ao lado/ponto (idioma Fusion/SolidWorks/Shapr3D: `|` `—` `=`n 🔒, cota com setas e número
  clicável), hover acende o que a regra liga, a cor da forma diz livre/definida/conflito.
- O Inspector não é a superfície das regras (no máximo um resumo); não invista na secção `INSP_LIVE_CAD_SECTION`.
- Cena de smoke com POUCAS regras, cada uma visível (proposta: caixa com tampa que abre por ângulo animado).
- Relacionado: [[feedback_a_smoke_for_the_owner_explains_what_each_thing_on_screen_is]].
