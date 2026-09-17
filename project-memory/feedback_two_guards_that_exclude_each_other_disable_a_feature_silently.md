---
name: feedback-two-guards-that-exclude-each-other-disable-a-feature-silently
description: Duas guardas que se excluem uma à outra desligam um recurso sem nunca o dizer — e curar só metade da pergunta deixa-o mudo na configuração de fábrica
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 288048cc-7a6f-40b2-8ea6-37cc673393d2
  modified: 2026-09-14T01:23:55.769Z
---

Medido em 2026-09-13 (W7 do plano `docs/Skeleton/03`). O onion da timeline exigia que o objecto
seleccionado **(a)** estivesse animado **e (b)** tivesse instância de desenho. Numa personagem
riggada quem leva keyframes são os **ossos** (que não desenham) e quem desenha é a **imagem** (que
não leva keyframes) ⇒ **zero fantasmas, sempre**, sem um erro, sem um log, sem um gate vermelho.

A fila do módulo descrevia o defeito como *«os fantasmas desenham o quad de repouso»* — verdade **se
houver fantasma**. Só medir mostrou que não havia nenhum.

⚠️ **E a mesma pergunta aparecia num SEGUNDO eixo:** curado o escopo (quem é ghostado), os
*instantes* do modo `Keys` — **o de omissão** — continuavam a sair das keyframes do alvo
**desenhado**. Metade curada = recurso mudo na configuração de fábrica.

**Why:** cada guarda está certa sozinha e foi escrita por uma razão real. O que ninguém escreve é a
**conjunção**, e ela pode ser vazia para uma população inteira de cenas. Nenhum gate acusa: um
recurso que devolve *nada* passa em toda asserção de *«não desenhou lixo»*.

**How to apply:** quando um recurso tem mais de uma condição de entrada, pergunte **de que população
a conjunção é verdadeira** — e escreva a fixtura dessa população (aqui: um rig, e não um objecto
keyado). E ao curar, faça o censo dos EIXOS da mesma pergunta: *quem move isto?* valia para o escopo
**e** para os instantes. Ver [[feedback-making-a-thing-exist-gives-answers-where-there-were-none]] e
[[feedback-a-probe-that-arms-a-module-by-env-var-measures-another-program-than-the-pill]].
