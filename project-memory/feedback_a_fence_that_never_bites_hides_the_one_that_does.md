---
name: feedback_a_fence_that_never_bites_hides_the_one_that_does
description: Uma cerca conservadora a mais dentro de um `min` torna as outras INOBSERVÁVEIS — a mutação que apaga a que importa sobrevive, e a régua fica verde sobre a lei errada
metadata:
  type: feedback
---

Quando um tecto é o `min` de várias candidatas, uma candidata **mais apertada do que todas as
outras** não é «margem de segurança»: ela é o que **decide sempre**, e as outras deixam de ser
observáveis. ⇒ apagar qualquer uma delas é uma mutação que **SOBREVIVE**, e o gate que as devia
medir fica verde sobre um `min` de uma só coisa.

**Medido** (2026-09-07, `line/3DModeling` W134, o tecto da corda do nó de toro): eu tinha
acrescentado uma quarta candidata — *«a corda não passa de metade do furo do anel»* — para dar chão
ao divisor num caso degenerado. Ela era a mais apertada em quase toda a grelha `(p, q)`, e com ela
lá dentro **apagar o termo da CURVATURA passava despercebido**. ⛔ E ela nem era necessária: o censo
inteiro passa sem ela (quem cura o caso degenerado é o majorante do gradiente, que cresce quando o
raio útil encolhe), e o `0,5` dela não nomeava recurso nenhum (`CLAUDE.md` §0).

Retirada, a régua do alcance passou imediatamente a acusar uma sobra **real** de `1,242×` numa
célula que ela dava por boa havia meia jornada.

**Why:** o modo de falha é silencioso nos dois sentidos. A cerca a mais **não** dá erro — dá um
produto mais conservador do que precisa —, e ao mesmo tempo **branqueia** a prova de mutação de
todas as outras. Ler *«a mutação sobreviveu»* leva a escrever um gate novo para uma lei que já está
gateada; o defeito está no `min`, não no gate.

**How to apply:** ao acrescentar uma candidata a um `min` de cercas, pergunte **quantas células ela
decide** — se for quase todas, ou ela substitui as outras (e as outras saem) ou ela está errada. E
antes de aceitar uma mutação sobrevivente sobre um `min`, **mute as vizinhas primeiro**: a que
sobrevive pode ser a que está a ser mascarada.

Vizinhas: [[feedback_a_surviving_mutation_can_mean_the_code_is_redundant]] ·
[[feedback_a_fence_can_guard_two_things_and_name_only_one]] ·
[[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]]
