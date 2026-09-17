---
name: feedback-a-consumer-that-picks-among-clocks-by-hand-is-the-answer-that-ages
description: "Quando o quadro já escolhe entre N relógios (ou N fontes) em dois sítios, um consumidor novo que escolhe sozinho escreve a resposta que envelhece — e ela envelhece na configuração de FÁBRICA"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 288048cc-7a6f-40b2-8ea6-37cc673393d2
  modified: 2026-09-14T18:27:35.764Z
---

Medido em 2026-09-14 (W8 do plano `docs/Skeleton/03`). Report do dono: *«só aparece a silhueta do
futuro»*.

O onion da timeline desenha fantasmas nos instantes vizinhos, e a pose de um fantasma fala o tempo do
**clip activo**. Mas o relógio que a vista dirige não é sempre o mesmo objecto: a aba **Keys** (a de
omissão) move o `clip_playhead`, um contêiner aberto move o `container_playhead`, e só o Arrange move
o `playhead` da cena. O quadro escolhia entre os três em **dois** sítios, com o mesmo `if` escrito
duas vezes — e a chamada do onion passava um **quarto** palpite à mão, o da cena.

⇒ arrastar o cursor na aba Keys **não movia** esse número: ele ficava em `0`, não existia keyframe
antes dele, e só o futuro tinha vizinhos. *Um recurso meio mudo lê-se como um recurso partido.*

⚠️ A cura não é escolher melhor: é ler o que a vista **já publica** uma vez por quadro (aqui o
`clip_time` do snapshot do painel, que sai do relógio activo e é `None` quando o clip não toca ali —
e aí a resposta honesta é *nenhum fantasma*).

**Why:** um `if` de três braços copiado é barato de escrever e invisível de auditar: cada cópia está
certa sozinha, e o defeito é a **quarta** cópia não existir como cópia — ela é um literal. Nenhum
gate do recurso o vê, porque todos recebem o instante como ARGUMENTO: *um gate que recebe o instante
mede tudo menos de onde ele veio.*

**How to apply:** ao escrever um consumidor que precisa de «qual relógio / qual fonte / qual vista»,
procure primeiro quem já responde (um snapshot publicado, uma porta) em vez de repetir a escolha. Se
a escolha viver num sítio que nenhum teste alcança (uma fase do quadro), a lei é um **arch-gate
textual** que exige a porta e proíbe as fontes cruas — ⚠️ apagando os comentários antes de a aplicar,
senão ele acusa a prosa que explica a cura. Ver
[[feedback-two-guards-that-exclude-each-other-disable-a-feature-silently]].
