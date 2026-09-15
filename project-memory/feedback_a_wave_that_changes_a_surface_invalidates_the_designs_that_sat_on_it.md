---
name: a-wave-that-changes-a-surface-invalidates-the-designs-that-sat-on-it
description: O chip da unidade estava certo sobre o campo transparente de ontem e errado sobre o campo AFUNDADO de hoje — e quem o partiu fui eu, dois commits antes, sem reconferir nada.
metadata:
  type: feedback
---

Mudar uma **superfície** do design system não muda só a superfície: invalida, em silêncio, todo
desenho que assentava nela. O código desses desenhos não se mexe, os gates deles continuam verdes,
e o defeito aparece como um report de aparência dias depois — atribuído à feature errada.

**Caso medido (`line/UIUX`, 2026-09-14), duas vezes no MESMO dia.**

1. Em 2026-09-05 o **painel desceu** um degrau para o cartão se ler (cura de um report do dono). Isso
   pôs o token do cartão exactamente onde seis pintores enchiam o corpo dos controlos deles ⇒
   *«caixas de input numérico sem cor de fundo»* e *«Checkbox invisível»*, nove dias depois.
2. Nesse mesmo dia eu tornei o campo uma **superfície afundada**. Dois commits à frente entreguei a
   unidade num CHIP com fundo próprio dentro dele — um desenho que era coerente com o campo
   transparente-com-borda de ontem. Veredito do dono: *«não ficou legal. Melhor junto ao número
   dentro da caixa»*. **O chip não mudou; mudou o que estava por baixo dele.**

**Why:** um desenho é uma relação entre duas superfícies, e um commit que mexe numa delas só toca o
ficheiro de uma. Nenhum gate deste repo pergunta *«que desenhos assentavam nisto?»* — os censos são
por token, por pintor, por ficheiro, e a relação não tem endereço nenhum.

**How to apply:** ao mudar uma superfície (um tom de fundo, um degrau da escada, a presença de uma
moldura), o commit tem de trazer o **censo dos vizinhos**: *quem pinta EM CIMA disto, e quem pinta
um corpo DENTRO dele?* — varrido pela propriedade, nunca pelo widget que o report nomeia
([[feedback-a-doc-that-states-the-law-the-code-does-not-implement-reads-as-audited]]). E quando um
report de aparência chegar sobre uma feature nova, pergunte primeiro **o que mudou por baixo dela**,
antes de discutir a feature.

⭐ O lado bom, e vale registá-lo: a cura da aparência **dissolveu a classe inteira** de um defeito —
os dois gates do chip (partição e contenção num host estreito) deixaram de ter sujeito, porque já
não há um segundo rectângulo para conter. *Um gate sem sujeito substitui-se pela propriedade que
sobrou; não se apaga em silêncio.*

Vizinhos: [[feedback-a-ported-law-carries-the-source-apps-premise-about-its-own-layout]] ·
[[feedback-a-door-the-neighbour-does-not-call-is-not-a-door-yet]] ·
[[reference-topic-control-design-hazards]] · [[reference-topic-ui-seam-discipline]]
