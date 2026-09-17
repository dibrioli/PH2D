---
name: feedback-an-inert-mode-can-still-pay-its-full-price
description: Um modo caro com orçamento GLOBAL repartido fica inerte quando a cena enche — e pode continuar a pagar a avaliação inteira; e a sonda do orçamento tem de exercer o trabalho que ele limita
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 288048cc-7a6f-40b2-8ea6-37cc673393d2
  modified: 2026-09-17T00:00:07.891Z
---

Medido em 2026-09-16 (fila do esqueleto, F6-t): o `Smooth` da pele de imagem tinha um orçamento de
peças do QUADRO, repartido pelas imagens. Com 2+ imagens do tamanho do smoke a soma das malhas
guardadas passava o orçamento ⇒ nenhuma podia partir ⇒ a saída era a do `Fast` ao bit — e o quadro
pagava a lei inteira para o descobrir (8 imagens: 5,9 ms contra 0,21, 35 % de um quadro, no modo de
fábrica). E o próprio orçamento vinha de uma sonda que já não refinava (arte pequena demais no ecrã):
o custo real tinha duas partes (avaliar 0,36 µs, peça nova ~1,0 µs) e o orçamento cheio custava 19 %
em vez dos 10 % prometidos. A causa do custo era um `BTreeMap` só PROCURADO, nunca percorrido.

**Why:** um controlo «morto» costuma ser procurado pelo efeito que falta; este também custava, e só
uma sonda com a cena CHEIA (várias instâncias, o tamanho do produto, zoom onde o trabalho acontece)
o mostra. Um relógio de sonda cujo sujeito não faz o trabalho mede outra coisa com o mesmo nome.

**How to apply:** ao escrever um orçamento repartido, (1) curto-circuite o caminho caro quando a
parte não deixa trabalho (e gate-o por CONTAGEM de execuções, não por relógio); (2) meça o custo com
a sonda a exercer o trabalho de verdade e mostre uma coluna que acuse quando não exerceu; (3) um mapa
ordenado que nunca é iterado é custo sem função — um índice direto dá as mesmas respostas (prove com
impressão digital da saída antes/depois). Ver [[project-pixel-tools-flatten-bone-bound-art]] e a
família [[reference-topic-control-design-hazards]].
