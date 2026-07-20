---
name: feedback_making_a_node_eligible_can_regress_a_partial_claim_retreat_dont_refuse_whole
description: Making a previously-uncovered node claimable can REGRESS documents that use it partially, when a whole-or-nothing rule fires; re-measure the real doc, and RETREAT (un-claim the offender) instead of refusing the whole plan.
metadata:
  type: feedback
---

Quando você torna elegível para um caminho rápido um nó que antes era a fronteira
(dar um kernel/handling a um nó de contêiner ou escopo), **RE-MEÇA o plano dos
documentos REAIS antes e depois** — não só a capacidade nova.

**Por quê:** uma regra "tudo-ou-nada" pode transformar a nova elegibilidade numa
REGRESSÃO para o caso comum. Caso real (GPU/M5, ADR-0135, 2026-07-20): dar kernel
ao `sim.zone` o tornou reivindicável, mas a neve de boot tem nós que MUDAM CONTAGEM
sem kernel no interior, e como a zona alimenta uma aresta `pre`, a regra
`sim_state_on_gpu` (uma fronteira dentro de um laço com estado ⇒ refuta o plano
INTEIRO) passou a disparar. Antes: `HYBRID`, 2 stages de render na GPU. Com a zona
elegível e a refutação-inteira antiga: `CPU`, 0 stages — a cadeia de render que
SEMPRE esteve válida na GPU, jogada fora à toa.

**A cura não é gatear a elegibilidade — é RECUAR.** Quando um nó reivindicado
alimenta o laço mas o laço não pode ser coberto inteiro, **proíba o `pre`-source e
RE-PLANEJE** (o nó vira fronteira, como era antes de ter kernel): o laço recua ao
caminho lento e o trabalho a JUSANTE dele fica no rápido. Um laço totalmente
coberto segue sendo reivindicado inteiro. Refutar tudo (`boundaries=[(sink,0)]`)
descarta um sufixo válido por nada.

**Como pegar:** o gate do recuo precisa de um controle POSITIVO (um laço
totalmente coberto É reivindicado) ou ele passa com "nunca reivindique o
contêiner" — o oposto da feature ([[feedback_a_negative_search_needs_a_positive_control]]).
E o CENSO do documento real é o oráculo de não-regressão (o plano dele fica
idêntico; aqui só o rótulo mudou de `[no-kernel]` para `[refused-despite-kernel]`).

Relacionado: [[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]] (MEÇA
antes) · [[feedback_a_frontier_is_not_a_census]] (conte o que o documento REAL
faz, não a capacidade). Detalhe: ADR-0135.
