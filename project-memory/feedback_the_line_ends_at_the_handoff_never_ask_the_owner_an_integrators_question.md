---
name: feedback-the-line-ends-at-the-handoff-never-ask-the-owner-an-integrators-question
description: "O trabalho de uma linha acaba no HANDOFF — como destravar a integração e em que ordem as linhas entram são perguntas do INTEGRADOR, nunca do dono"
metadata:
  type: feedback
---

Em 2026-09-16 o dono mandou *«Antes de seguir vamos integrar ao main. Escreva handoff»*. Eu escrevi o
handoff **e depois perguntei-lhe** duas coisas: como destravar os 74 ficheiros por comitar na árvore
primária, e se integrava só esta linha ou esperava a rodada.

A resposta dele:

> *«não sei. desde do início do projeto nenhum agente me fez essa pergunta. Simplesmente escrevia o
> handoff e deixava o integrador integrar todas as linhas ativas na ordem mais adequada»*
> · *«Vc não integra. vc apenas prepara o handoff para que um outro agente integre todas as linhas.»*

**Why:** o dono decide **produto e envio** ([[user-role]]); *mecânica de integração* — ordem das
linhas, árvore suja, re-contagem de degraus — é o ofício do **agente integrador**, que existe
exactamente para isso (DIRETRIZ §1.5.3–1.5.4). Perguntar-lhe isso devolve-lhe um trabalho que ele
delegou, e com vocabulário que ele não usa. ⚠️ E o sinal de que a pergunta está errada estava na
própria resposta: *«nenhum agente me fez essa pergunta»* — **uma pergunta inédita depois de dezenas
de rodadas é quase sempre uma pergunta do papel errado**, não uma lacuna que ninguém tinha visto.

**How to apply:** o que você descobre sobre a integração **escreve-se no handoff**, não se pergunta.
Um bloqueador medido (ex.: *«um `--ff-only` é recusado hoje porque há 74 ficheiros por comitar na
primária, 6 deles meus»*) entra como secção própria, **com as saídas e o preço de cada uma e a que a
linha recomenda** — e a linha **não lhe toca**, porque aquelas alterações são de outras sessões.
Depois: **PARE**. Ver [[feedback-architecture-decisions-are-delegated-to-the-gold-standard]] para o
lado simétrico (decisão TÉCNICA não se pergunta: decide-se pelo padrão-ouro e executa-se).
