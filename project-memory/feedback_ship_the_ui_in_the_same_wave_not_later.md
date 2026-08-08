---
name: feedback-ship-the-ui-in-the-same-wave-not-later
description: "A UI de uma feature entra na MESMA wave que o motor — sem ela o Enio não consegue smokar, e eu não consigo diagnosticar"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 6c930288-8614-4136-a817-2a85c1b12c8d
  modified: 2026-08-05T21:22:03.341Z
---

Toda wave entrega a **UI junto com a implementação**. Atalho de teclado com
valores fixos num array não é UI: é um harness meu vazando para o produto.

**Why:** Enio, 2026-08-05, sobre o módulo 3D — *"O efeito existe embora de baixa
qualidade. Sem UI, fazer testes assim é Ruim! Por que não trabalhar a UI junto
com as implementações?"*. Duas consequências, e a segunda é a que eu não tinha
visto: (1) ele não consegue **julgar** um canal contínuo por quatro degraus que
eu escolhi — a pergunta *"quanto disto?"* é dele, e um ciclo de tecla a
responde por ele; (2) sem controle e sem indicador na tela **eu perco o
instrumento de diagnóstico** — no mesmo report ele disse que o pincel não cai
onde o mouse aponta, e sem um cursor desenhado na malha nem ele nem eu
conseguimos separar *a fiação está errada* de *a forma é vista de esguelha ali*.

O módulo tinha ~40 gates de aparência e nenhum jeito de o artista mexer num
número. Gate verde não é UI: ele prova que o motor faz o que eu disse, nunca
que o artista alcança.

**How to apply:** ao planejar uma wave, a UI é uma FATIA dela, não a wave
seguinte — as quatro condições que a `line/physics` já exige (o componente
existe · é pintado e registrado · o clique chega ao barramento · a sequência
leva a algum lugar) valem no MESMO commit range. E quando o gesto é espacial
(pincel, alça, campo), o **indicador na tela conta como UI** e costuma ser o
instrumento que responde ao primeiro report de "está no lugar errado".
Ver [[feedback_ergonomics_verdict_is_a_design_bug]] e
[[reference_topic_ui_seam_discipline]].
