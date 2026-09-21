---
name: a-parity-gate-that-picks-its-own-mode-measures-a-program-nobody-runs
description: Um gate de paridade que CRAVA o modo que quer medir afirma sobre um programa que o artista não está a correr — a paridade lia 0,000215 e o app abria noutra lei, a 0,347
metadata:
  type: feedback
---

⛔⛔⛔ **Um gate que ESCOLHE o modo em que entra prova a LEI e não o PRODUTO.** Se a feature tem
modos e o gate crava o que lhe interessa, ele fica verde para sempre enquanto o app abre noutro.

Medido 2026-09-21 (`line/3DModeling`, report do dono **repetido**: *«o bake não é idêntico ao que se
vê em 3d»*). O visor da escultura tinha acabado de ganhar o modo `Pbr` — a lei que assa o sprite — e
a paridade contra a referência em CPU lia **`0,000215`** de desvio por canal. Verdade, e inútil: o
gate entrava por uma vista de sonda que escrevia `lighting: Lighting::Pbr`, e o
**`DEFAULT_LIGHTING` do app era um MATCAP**. Varridos os modos contra a mesma lei, na mesma forma e
sob o mesmo rig:

```text
  Pbr        0,000215 medio     <- o que o gate media
  Rig        0,088611
  Flat       0,301561
  Matcap(0)  0,347275           <- o que o app MOSTRAVA
```

`1 615×` o resíduo medido, e `177×` a barra de meio código de oito bits.

⚠️ **E a causa era ESTRUTURAL, não um epsilon:** a estrutura do bake não tinha campo de modo de luz
nem de material — ele corria **sempre** uma lei —, enquanto o visor corria o que o default dissesse.
*Duas leis sobre o mesmo objecto por construção*, e nenhuma paridade medida **dentro** de um dos
ramos o pode ver.

⚠️ **O report veio DUAS vezes**, e a segunda é a lição: na primeira eu construí o modo certo e
tornei-o alcançável por um chip. *Uma cura que o artista tem de encontrar e escolher não é uma cura —
é uma opção*, e o valor de fábrica continuou a ser o defeito.

**Why:** a paridade responde *«as duas implementações desta lei concordam?»*. A pergunta do dono é
*«o que eu vejo é o que eu asso?»*, e entre as duas está uma escolha — o default — que nenhum gate
de lei atravessa.

**How to apply:** quando um subsistema tem **modos** e só um deles concorda com a outra metade do
sistema, escreva o gate que entra pelo **valor de FÁBRICA** (`DEFAULT_*`), nunca por um modo que o
teste monta — e ponha o **controlo** dentro: um dos outros modos tem de reprovar por uma ordem de
grandeza, senão a sonda deixou de distinguir duas leis e o gate fica verde a afirmar nada. E antes
de trocar o default, **meça o preço** dele (aqui: a diferença entre as leis ficou abaixo do ruído,
`±0,14 ms` a `load 25` e `40`, com a leitura de volta a dominar).

Ver [[reference_topic_gate_discipline]] · [[reference_topic_measurement_discipline]] ·
[[feedback_a_ruler_that_stops_at_world_space_approves_a_broken_click]].
