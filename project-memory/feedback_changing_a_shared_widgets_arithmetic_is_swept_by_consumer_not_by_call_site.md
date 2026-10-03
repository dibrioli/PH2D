---
name: feedback_changing_a_shared_widgets_arithmetic_is_swept_by_consumer_not_by_call_site
description: "Mudar a ARITMÉTICA de um widget partilhado tem de ser varrido por CONSUMIDOR (que largura cada um lhe dá), nunca pelos sítios que a wave editou — e um «deixei de fora de propósito» sem número ao lado é indistinguível de «não medi»"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e990a2f5-7d16-405a-8ddf-54393edf203d
  modified: 2026-09-16T02:31:54.483Z
---

Quando uma wave muda **quanto espaço um widget exige** — não o que ele desenha, mas a aritmética
dele —, a varredura certa é *«quem o instancia, e com que LARGURA?»*. Varrer os sítios que a wave
editou é varrer o conjunto errado.

Caso medido (2026-09-15, `line/UIUX`): a caixa de marcar ganhou uma **caixa de campo** na coluna do
controlo, por ordem do dono. O piso dessa caixa é o do campo numérico (`72 px`). A wave converteu os
sítios de **linha inteira** e declarou os outros fora de âmbito. Um dia depois, a medição:

| consumidor | largura que ele dá | coluna que sobra ao NOME |
|---|---|---|
| 5 fileiras EMPARELHADAS do Inspector (10 caixas) | meia linha (`83`–`183`) | `0,0` · `15,6` · `31,0` ⛔ |
| `BitmaskGrid32` (32 células, Cull Mask e Layer) | um QUARTO de linha (`43`–`64`) | **`0,00` em todo o curso** ⛔ |

⇒ **42 controlos** ficaram sem nome pintado, e o app compilava, passava 14 mil testes e tinha o
smoke do dono aprovado.

**Why:** o piso do controlo é absoluto e a linha é relativa, logo *o mesmo widget cabe numa largura e
não cabe noutra* — e só o consumidor sabe que largura lhe dá. Nenhum gate deste repo pergunta isso:
os censos de registo provam que o controlo é alcançável, não que ele tem onde caber.

⚠️ **E a segunda metade: o que eu escrevi para justificar deixá-los de fora era um palpite com cara
de decisão.** O handoff dizia *«continuam no default, de propósito — ali a secção é o PAR»*. A frase
é plausível, não tem número ao lado, e estava errada. ⛔ *Um «de propósito» sem medição é
indistinguível de um «não medi», e o leitor seguinte não consegue separá-los.*

⚠️ **E um defeito atrás de uma dobra FECHADA sobrevive a um smoke aprovado** — a `Cull Mask` nasce
recolhida, e o dono aprovou a wave anterior sem nunca a abrir.

## 2.º caso — a LEI de um compositor partilhado (2026-10-03, `line/sculpt3d`, ADR-0177)

A P1 mudou o `LayerCompositor` (`ph2d-render`) de luz para tons de ecrã, por decisão do dono PARA O
PAINTER. O Flip usa o mesmo compositor para as camadas dele — e junta os traços de UMA camada em
linear 16F. Herdar a lei nova abriu no Flip a costura do report, ao contrário: branco a 50 % numa
camada por cima de preto `0,216` linear, na mesma camada `0,5`. **Três ondas passaram sem o ver:** o
gate que o media (`composite_blend::top_layer_opacity_fades_toward_backdrop`) é de PLACA e de OUTRA
crate, e a lista de gates da linha só corria a placa do `ph2d-render`. O levantamento automático dos
consumidores classificou o Flip como «herda — sem mistura própria»: verdade sobre o código, falso sobre
a lei (a mistura própria dele está DENTRO da camada, no rasterizador).

⇒ a pergunta por consumidor não é «ele chama a porta?», é **«a lei da porta tem de concordar com
alguma lei DELE?»**. Cura: o espaço passou a ser do produto que chama (`CompositeSpace`), com uma
porta só no Flip (`compositor_do_flip`).

**How to apply:** ao mudar a aritmética de um widget partilhado, (1) liste os **consumidores** e a
largura que cada um lhe passa, (2) meça a resposta da porta em cada uma, (3) só então escreva o
âmbito — e se deixar algum de fora, ponha o **número** ao lado da razão. Ver
[[feedback_a_dead_control_and_an_absent_one_read_the_same_and_building_is_the_wrong_cure]] e
[[reference_topic_measurement_discipline]].
