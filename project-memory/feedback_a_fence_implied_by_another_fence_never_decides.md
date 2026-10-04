---
name: feedback_a_fence_implied_by_another_fence_never_decides
description: "Uma mutação que «sobrevive à escala de 0,1 px» pode ser uma cerca que NUNCA decide (outra cerca + o nível de detalhe já a implicam) — meça a premissa numa varredura de CPU, não procure a fixtura de pixel"
metadata:
  type: feedback
---

A `S6` (doc 121 §9.4, sobrevivente desde 01/10) tirava a cerca `FAIXA_FOLGA = 0,1` px da esquadria
num vértice liso. A conta: a cerca do recuo (`r·tan(θ/2) ≤` meia corda) dá `r ≤ R·cos(θ/2)`, logo o
excesso da esquadria `≤ R·(1 − cos(θ/2))` = a flecha do aplanamento no ecrã, que o nível de detalhe
do shader mantém `≤ 0,125` px. Medido (gate de CPU `a_esquadria_de_um_vertice_liso_nunca_passa_da_flecha_do_nivel`):
`126 870` lisos numa varredura de formas × larguras × afins, ZERO acima da cerca, pior `0,086` px.
Nenhuma fixtura de pixel a mataria — e procurá-la era a acção que ficou «sem acção» três dias.

**Why:** uma mutação sobrevivente é uma pergunta sobre a PREMISSA da cerca, não um pedido de fixtura;
quando a premissa sai de outras leis, ela mede-se em CPU em milissegundos e vira o gate que avisa se
alguém mexer nos níveis.

**How to apply:** antes de escrever a fixtura que mate um sobrevivente, derive o domínio em que a
cerca pode decidir e meça-o; se for vazio, feche como equivalente COM o número e gateie a premissa.
Ver [[reference_topic_mutation_proofs]].
