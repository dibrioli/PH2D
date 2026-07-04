---
name: feedback-communication-simplicity
description: "Seguir os protocolos do projeto de forma simples — sem expansão preventiva de tópicos não pedidos, sem AskUserQuestion com múltiplas perguntas sobrepostas, sem antecipar decisões arquiteturais antes que o protocolo peça"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 8dd4bacc-01e4-4931-9976-82e9878a91e5
---

Quando há protocolo escrito (CLAUDE.md, docs/IntegracaoMultiAgente/02-Coordenador.md, etc.), siga-o **literal e simples**. Não invente passos extras. Não use `AskUserQuestion` com 4 perguntas complexas e múltiplas opções cada quando o que o protocolo pede é uma resposta direta ou um briefing curto.

**Why:** Enio relatou explicitamente "achei confusa essa forma de comunicação" depois de eu responder um pedido de "atribuir slot a feature" com: 3 opções de pasta + decisões em 4 categorias (ADR / budget / modelo / escopo) + AskUserQuestion com 4 perguntas. Era 5× o volume necessário e antecipava decisões que ainda não tinham sido pedidas.

**How to apply:**
- Quando o pedido é §3.1(a) "atribuir slot a nova feature": gere o briefing minimal (ESCOPO + SLOT + UI block se tem painel + cola integral 03-Agente-Periferico.md), atualize STATE.md, comite, devolva ao Enio. Sem AskUserQuestion. Sem antecipar discussão de ADR/budget/escopo a menos que o agente esteja bloqueado.
- Quando o pedido é §3.1(b) "agente propôs pasta — aprovar?": valide só conflito + arquitetura, dê aprovação direta OU 2-3 opções concisas se ambíguo. Não expanda para discussão de blockers tangenciais que o agente nem mencionou nesse turno.
- Quando o pedido é §3.1(c) "agente precisa de coisa fora da pasta": atenda só o que ele pediu. Pedidos arquiteturais maiores (ADR novo, mudança em HR, novo budget bucket) escalam separadamente.
- `AskUserQuestion`: usar com MUITA parcimônia. Uma pergunta por vez, no máximo. Se a decisão exige escolha do Enio entre 2-3 opções concretas — texto direto, sem AskUserQuestion estruturada.

Distinto do modo "design discussion" (vide [[feedback-communication-style]] sobre pt-BR direto + opções concretas + recomendação primeiro): aquele é para conversas abertas onde Enio pediu input. Em protocolo Coordenador o gatilho é diferente — Enio relayou pedido, eu executo o passo definido.
