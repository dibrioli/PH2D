---
name: feedback-a-correlation-with-zero-counterexamples-may-describe-another-question
description: Uma tabela com zero contra-excepcoes pode estar a descrever DISPONIBILIDADE em vez de CORRECCAO — pergunte que pergunta ela responde antes de a promover a lei.
metadata:
  type: feedback
---

Antes de promover uma correlação perfeita a **regra**, pergunte *«que pergunta é que esta
tabela responde?»*. Uma separação sem contra-exemplos pode estar a descrever **qual das
alternativas está disponível** e não **qual está certa** — e as duas leem-se igual numa
tabela.

**Why:** medido no `ph2d-quadextract` (2026-08-27). Cruzando «qual convenção de direcção
acertou» com «que faces estão dobradas», saíram **18 casos, zero contra-exemplos, zero
ambíguas**, e a regra até se explicava pela composição das orientações. ⛔ Implementada, ela
deu **exactamente** o mesmo resultado que «tentar as duas» — regressão incluída — porque as
duas alternativas **nunca colidem**: onde só existe uma candidata, escolher e tentar-as-todas
escolhem a mesma. *A tabela descrevia qual chave estava no mapa.*

**How to apply:** o teste é barato — **compare a regra derivada com a alternativa ingénua
que ela deveria bater** (tentar tudo, escolher ao acaso, a primeira). Se derem o mesmo,
a regra não está a decidir nada. E o sinal a montante é o `ambíguas = 0`: *se as
alternativas nunca coexistem, nenhuma regra que escolha entre elas pode ter conteúdo.*
Parente de [[feedback-a-probe-in-the-failure-branch-cannot-see-the-other-sides-successes]]
e [[feedback-a-claim-no-mutation-can-kill-is-a-claim-about-nothing]].
