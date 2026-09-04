---
name: feedback-a-ruler-that-counts-leftover-defect-rewards-overshooting
description: "Uma régua que conta «quanto defeito sobrou» premeia exagerar a cura — ela não distingue «está certo» de «está grande demais», e o exagero lê-se como melhoria"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-02T23:55:59.918Z
---

Uma régua da forma **«que fracção da peça ainda tem o defeito?»** é monótona no tamanho da cura:
aplicar mais cura apaga mais defeito, sempre. ⇒ ela **não consegue** distinguir *«a cura tem o
tamanho pedido»* de *«a cura é grande demais»*, e um operador que exagera pontua **melhor** que o
correcto.

Caso medido (PH2D, `line/3DModeling`, 2026-09-02 — o filete que só era um arco a 90°). A sonda de
arestas conta a fracção de superfície sobre um vinco depois do filete. Ao pôr o operador a entregar
o **arco verdadeiro** em qualquer quina:

| forma | quina | o que a lei antiga entregava | leitura da sonda |
|---|---|---|---|
| estrela | ponta, `19,2°` | `2,29×` a **menos** | melhora muito (`3,71 → 0,66`) |
| prisma hexagonal | parede, `60°` | `2,19×` a **mais** | **piora** (`0,15 → 0,18`) |
| octaedro | face, `54,7°` | `1,60×` a **mais** | **piora** (`0,06 → 0,13`) |

⇒ *as duas «pioras» são o preço de a peça passar a ter o tamanho que o número pede.*

**How to apply:**
- ⭐⭐ **Antes de aceitar um número como veredito, pergunte se a régua é monótona na grandeza que se
  está a afinar.** Se for, ela mede *quantidade de cura*, não *correcção* — e a barra dela é uma
  licença para exagerar.
- ⭐ A régua que decide tem de ser **o valor analítico** (aqui: o recuo de um arco de raio `r` é
  `r·(1/sin α − 1)`), com a de fracção a servir de rede contra regressões grosseiras.
- ⚠️ **Meça os DOIS sentidos do erro antes de escrever a nota.** A nota original deste módulo dizia
  só *«uma ponta aguda arredonda de menos»*; a metade obtusa — *arredonda de mais* — não estava
  escrita em lado nenhum, e é ela que explica por que a régua premiava a lei errada.
- ⛔ O mesmo padrão já mordeu este repo noutra roupa: [[feedback_a_ratio_bar_tightens_itself_when_the_denominator_is_a_knob]]
  (aqui, curar o filete melhorou o **denominador** de um gate de razão e ele reprovou sobre um
  numerador que não mexeu um bit).
