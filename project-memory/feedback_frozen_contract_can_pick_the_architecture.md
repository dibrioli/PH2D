---
name: feedback-frozen-contract-can-pick-the-architecture
description: Um contrato congelado que proíbe o desenho óbvio não é só um obstáculo — o desenho que sobra costuma vir com um invariante (e um gate) que o óbvio não teria.
metadata:
  type: feedback
---

Quando um contrato congelado (§6) barra o caminho óbvio, **não pergunte primeiro como
quebrá-lo**. Pergunte que desenho sobra — ele frequentemente é **mais forte**, e a força vem
com um **guard executável** que o desenho óbvio não conseguiria ter.

**O caso (doc 57, subgrafos do Motion, 2026-07-13):** o óbvio era um nó `motion.subgraph` cujo
`eval` cozinha um sub-grafo. Impossível: `NodeManifest.inputs` é `&'static [PortSpec]` (portas
dinâmicas não cabem) e `NodeOp::eval` não recebe `Cook`/`OpResolver`. O que sobrou foi *nesting
é uma dobra da VISTA; o grafo continua PLANO* — e isso deu o gate
`grouping_never_changes_the_cook`: **agrupar produz um buffer de instâncias byte-idêntico**.
Um nó-subgrafo de verdade jamais poderia prometer isso.

**Why:** a restrição carrega informação. `&'static` não é burocracia — é o substrato dizendo
"a topologia é estática e o memo é `(NodeId, ScopeKey)`". O desenho que respeita isso herda a
garantia; o que a contorna passa a ter que *provar* o que antes era de graça.

**How to apply:** ao bater num contrato congelado, escreva as 3 razões pelas quais ele proíbe o
óbvio (elas são o *diagnóstico*), depois derive o desenho que sobra e **procure o invariante que
ele te dá de graça**. Se achar um, transforme-o em gate falsificável — e só então considere
pedir ADR. Se o desenho alternativo não tem invariante nenhum, aí sim o contrato é o problema.

Corolário: cheque a indústria antes de achar que a restrição é sua. A Unreal chegou ao MESMO
lugar sem contrato nenhum a respeitar (*collapsed graph* é organizacional, o compilador achata
os tunnels) — quando o caminho forçado coincide com o que o padrão-ouro escolheu de livre
vontade, você não está fazendo concessão, está fazendo engenharia. Ver
[[feedback_no_industrial_claims_without_verification]].
