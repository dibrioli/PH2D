---
name: a-per-piece-fit-breaks-the-property-that-lives-on-the-seam
description: Um ajuste resolvido peça a peça em isolamento quebra exactamente a propriedade que vive na FRONTEIRA entre as peças — e cada peça, lida sozinha, está certa.
metadata:
  type: feedback
---

**Um ajuste que resolve cada peça SOZINHA não pode preservar uma propriedade que vive na FRONTEIRA
entre duas peças.** Lida em isolamento, cada peça está certa; o defeito só existe no encontro.

Medido (PH2D, `line/Vector`, 2026-09-19 — report do dono *«muitas irregularidades … mau tratamento
das alças dos handles»*): a correcção das alças de uma Bézier resolvia **um segmento de cada vez**
por mínimos quadrados. As duas alças que se encontram num nó saíam de dois sistemas que não se
conhecem ⇒ deixavam de ser colineares, e o nó que o artista desenhou LISO virava uma QUINA — `28,6°`
a `120°` de dobra, `9,05°` no nó mediano.

⭐ **O CONTROLO é o que nomeia a causa, e ele costuma ser a lei ANTIGA:** a lei ingénua media
`0,000°` em todas as dobras, porque ela aplica **UM** afim às três metades de cada vértice e *um
afim preserva colinearidade*. ⇒ *a quebra não vinha do motor: vinha do ajuste que se lhe pôs por
cima.*

⚠️ **A cura NÃO é prender cada peça** (isso dá a propriedade e mata a capacidade — ver
[[a-constrained-fit-buys-continuity-by-killing-the-feature]]): é **acoplar as peças no nó**, deixando
o que é partilhado rodar JUNTO. Aqui: as duas alças de um nó rodam pelo mesmo ângulo a partir do
eixo que o afim daquele nó lhes dá. A propriedade fica exacta **e** a fidelidade MELHORA
(`0,03371 → 0,01900`), porque conciliar não tira graus de liberdade — redistribui-os.

⛔ **E a coisa a procurar antes de construir:** se o produto tem uma lei «ingénua» que já preserva a
propriedade, ela é o controlo e a referência — o ajuste tem de ser medido **contra ela**, nunca
contra zero.
