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

---

## ⛔⛔⛔⛔ A SEGUNDA METADE, no mesmo dia: não era só o MODO — era o ORÁCULO

O dono reportou **outra vez** (*«a malha 3d parece ter mais luz indireta que a imagem do Bake. Mas
precisa ser idêntica.»*), e ele tinha razão outra vez. Trocar o modo que o visor mostra não chegou,
porque **a outra metade também escolhia**: a lei que acende um objecto assado saía de uma porta cuja
escada era `Some("1") => Forma, _ => Tinta` — ou seja, **sem a variável de ambiente o produto assava
por OUTRA lei** (um modelo de tinta, sem GGX e sem conservação de energia).

⚠️⚠️ **Todas as sondas da linha tinham como oráculo a lei que só corre com a bandeira LIGADA.** Elas
liam `0,000215` e estavam certas — sobre um programa que ninguém corria. O visor contra a sprite que
o produto **de facto** assava media **`0,055042`** por canal (`28×` a barra).

⛔⛔⛔ **E o pior: esse número já tinha sido medido uma vez, por mim, e EXPLICADO.** Numa sonda
anterior ele apareceu como `0,055` e eu anotei *«são duas leis diferentes»* — o que era verdade e era,
à letra, o defeito do dono. *Uma explicação correcta que dissolve a evidência é mais cara que nenhuma:
ela fecha a investigação.*

**How to apply (acrescenta-se ao de cima):**

1. Antes de escrever uma paridade, pergunte **por que porta o PRODUTO passa** — e escreva no gate a
   asserção de que o oráculo É essa porta. Um oráculo escolhido pelo harness que estava à mão mede o
   que aquele harness mede.
2. Quando um sistema tem **duas** metades com valor de fábrica próprio (aqui: o que o visor mostra e
   a lei que assa), a régua tem de as **amarrar uma à outra**, e essa régua tem de ser **pura** —
   os gates de GPU são `#[ignore]` e o CI nunca os corre.
3. ⭐ **Uma prova de PONTA A PONTA vale mais que N paridades de lei:** comparar o que o artista VÊ
   contra os **bytes** do que ele assa fecha de uma vez todas as escolhas intermédias. Se ela não
   existe, as paridades podem estar todas verdes com o produto partido.
4. ⛔ Quando aparecer um desvio que você consegue **explicar**, a pergunta seguinte não é *«a
   explicação está certa?»* — é *«esta é a grandeza de que o dono se queixa?»*.
