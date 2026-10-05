---
name: feedback-a-defect-attributed-to-a-component-is-swept-before-the-cure
description: "O defeito atribuído a um componente varre-se ANTES de desenhar a cura — o doc dizia «o kurbo morde» e a cura ia mudar as marcas da placa; varrido, o kurbo não morde em ângulo nenhum: quem morde é o traçador do Vello"
metadata:
  type: feedback
---

A «mordida» do traço rente depois de uma quina (doc 121 §9.18 C, 05/10) vinha escrita como do `kurbo::stroke`. A
cura desenhada a partir disso trocava também as MARCAS conformes da placa (que saem do `expand_stroke` do kurbo)
pela lei da placa. Uma varredura na CPU antes de construir (quina `30°`–`170°` × `0,05`–`0,8` larguras × pontas
redonda/rente/quadrada) deu ao kurbo os MESMOS pontos pintados que a lei: quem morde é o traçador do Vello, na
placa gráfica, e só na rota Vello. A troca das marcas teria mexido nas `7` famílias da placa por nada.

**Why:** a atribuição de um defeito num doc é uma hipótese de quem o viu numa imagem; o controlo positivo do teste
(«o componente suspeito morde ESTA fixtura») foi o que a derrubou — ele falhou na primeira corrida.

**How to apply:** antes de curar, um teste com CONTROLO que prova que o componente nomeado produz o defeito na
fixtura; se o controlo falha, varra a vizinhança e procure o componente que de facto o produz. Família:
[[reference_topic_repro_discipline]].
