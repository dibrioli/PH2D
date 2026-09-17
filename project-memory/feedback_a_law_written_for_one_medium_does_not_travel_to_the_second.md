---
name: feedback-a-law-written-for-one-medium-does-not-travel-to-the-second
description: "Uma lei de ORDEM escrita num comentário ao lado do primeiro consumidor não viaja para o segundo — e o segundo pode estar noutra metade do quadro, onde ela é falsa"
metadata:
  node_type: memory
  type: feedback
---

Medido em 2026-09-14 (W14c, `line/Vector`). O dono reportou, com fotos: *«ao acrescentar o IK o osso
perde influência sobre a ponta da malha»* — o **gizmo** do osso na pose resolvida, a **arte** parada.

⭐ **A lei violada estava escrita, à letra, num comentário do próprio ficheiro** — posto ali quando a
primeira mídia (a pele **vectorial**) foi ligada: *«ela escreve a pose dos ossos, e o recook é quem
transforma a pose em geometria. Ao contrário, a pele mostraria a pose do quadro anterior.»* Os
motores corriam imediatamente antes daquele consumidor, e estava certo.

⛔⛔ **A SEGUNDA mídia (a pele de IMAGEM) chegou meses depois e foi ligada noutra METADE do quadro** —
o `attach_skin_meshes` vive no extract das sprites, que corre antes da fase de canvas inteira. A lei
não viajou porque ela vivia **ao lado do primeiro consumidor**, e não ao lado dos motores.

⚠️ **E o sintoma não foi um atraso de um quadro** (que é o que a nota previa) — foi **permanente**:
outro passe da mesma metade (o apply da timeline) repõe todo objecto keyado pela curva mesmo a tempo
de o consumidor a ler. *Um atraso de um quadro é invisível parado; um atraso que outro passe
re-alimenta é um defeito para sempre.*

**Why:** uma ordem correcta escrita como PROXIMIDADE (*«corre imediatamente antes de X»*) é uma
afirmação sobre dois endereços, e ela morre em silêncio quando aparece um segundo X noutro sítio.
Nada no repo pergunta *«quem escreve este estado depois de ele ser lido?»*.

**How to apply:**
- ao ligar um consumidor NOVO a um estado que um motor escreve, a pergunta é *«em que metade do
  quadro o motor corre?»* — não *«que função faz o que eu quero?»*;
- a ordem gateia-se **dentro de UM ficheiro** (a posição de duas chamadas no mesmo índice de fases);
  ⛔ concatenar dois ficheiros para medir uma ordem é fraude — tudo o que está no segundo vem depois
  de tudo o que está no primeiro, e a asserção passa por construção;
- e gateie também a **outra porta**: o motor não pode continuar a ser chamado no sítio antigo, senão
  ele corre duas vezes. Ver [[feedback-a-new-gesture-inherits-the-enemies-of-the-old-one]] e
  [[feedback-two-guards-that-exclude-each-other-disable-a-feature-silently]].
