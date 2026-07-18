---
name: feedback_a_frontier_is_not_a_census
description: "Contar o que FALTA não dá o número de COSTURAS — a fronteira é da região que cresce, e o custo do que falta depende da POSIÇÃO, não da existência"
metadata:
  node_type: memory
  type: feedback
---

Quando um motor **reivindica** uma região que cresce a partir de uma raiz (um cook GPU
que sobe do sink, um type-checker que se espalha de um root, um bundler que anda pelo
import graph), a contagem de **elementos não-cobertos** não diz **nada** sobre o número
de **fronteiras**. A fronteira é uma propriedade da **ramificação da região**, não um
censo do que falta. E o **custo** de um elemento não-coberto é função da **POSIÇÃO** dele
em relação à raiz — não da existência dele.

**Why:** O handoff da `line/gpu-nodes` (2026-07-18) recomendava construir um seam de **N
fronteiras** no shell como *"o multiplicador"*, sobre a frase *"com 52 nós descobertos,
qualquer grafo real tem várias [fronteiras]"*. A frase confunde as duas coisas. A região
reivindicada cresce **para cima** a partir do sink, então as `boundaries` são a
**fronteira** dela — e uma **CADEIA** de nós descobertos apresenta exatamente **um** nó
de fronteira: o walk para no primeiro e **nunca vê o resto**. Para a fronteira ramificar,
um nó **estagiado** precisa de 2+ entradas com fontes ambas descobertas — e medindo os 3
únicos nós com kernel e 2 portas, **todos** declinam essa forma. **Medido em 5 grafos:
todos dão exatamente 1 fronteira.** Ou seja `boundaries.len() > 1` era **inalcançável**, o
arco de recusa do shell era **código morto**, e a fatia recomendada era maquinaria para um
estado que não pode ocorrer.

O que de fato mordia era o eixo que ninguém tinha olhado: a costura só entrega o **SUFIXO**
ao dispositivo, então **UM** nó descoberto no caminho derrubava a sim inteira de 4,19 M
partículas na CPU (`dispatching 3 → 0`). Mesmo número de costuras (uma), custo
catastroficamente diferente — **porque a posição mudou**, não a contagem.

**How to apply:**
- Antes de construir sobre um "N" herdado de um handoff/comentário, **construa o grafo
  mínimo que produziria N ≥ 2 e MEÇA**. Se você não consegue produzir o caso, a feature não
  tem caso. (Aqui: 6 formas sondadas, 30 min, e economizou a fatia inteira.)
- Pergunte **"o que faz a região RAMIFICAR?"**, não *"quantos elementos faltam?"*. A resposta
  é sempre uma propriedade de aridade/topologia, e costuma ser enumerável — e curta.
- Quando o custo depende de posição, **gateie as duas pontas**: um fixture com o elemento
  ausente perto da raiz e outro longe. Os dois têm o mesmo "número de faltantes" e resultados
  opostos; um fixture só provaria a metade conveniente ([[reference_topic_fixture_discipline]]).
- Um achado desses vira **TRIPWIRE**, não comentário: o gate
  `no_plan_can_leave_more_than_one_seam_today` fica vermelho no dia em que a premissa mudar
  (um kernel multi-input pousar) e diz *"agora a fatia é real — construa"*. Comentário
  apodrece; gate não ([[feedback_a_condition_that_enumerates_its_readers_rots]]).
- **Corolário medido no mesmo dia:** ao desenhar o híbrido inverso (rodar a parte cara no
  acelerador e **ler de volta** para a CPU fazer a cauda), **meça o readback antes de
  desenhar**. Aqui ele era **negativo**: 268 ms para ler 736 MB contra 3,8 ms de cook — e
  **pior que os 227 ms que a CPU levava sozinha**. Mover o dado pode custar mais que
  recomputá-lo, e o piso é banda, não engenharia ([[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]]).
