---
name: feedback_a_cure_that_moves_the_defect_names_it
description: Uma cura que não reduz o número mas o move de uma fase para outra NÃO é um fracasso — ela prova qual das duas fases o carregava, e é frequentemente a medição mais informativa da investigação
metadata:
  type: feedback
---

Quando uma cura deixa o total **igual** mas transfere o defeito de uma fase medida para
outra, ela não falhou: ela **provou de quem era**. Guarde-a desligada e escreva a
transferência, não o «não moveu».

**Why:** medido no quad remesh (2026-08-23). O achatamento de um patch prendia a
fronteira; trocá-lo pelo mapa **conforme** (fronteira a deslizar) deu, na esfera lisa:

| rectângulos | domínio | superfície | folga |
|---|---|---|---|
| fronteira **presa** | `1,0°` | `16°` | ⛔ `15°` sem nome |
| fronteira a **deslizar** | `12,4°` | `14°` | ⭐ `1,6°` |

Lido como «`16° → 14°`, ganho de ruído» seria um descarte. Lido nas **duas colunas** é
outra coisa: com a fronteira presa há `15°` que aparecem entre o domínio e a superfície
e não têm explicação; com ela a deslizar sobram `1,6°`. ⭐ *A quase-igualdade é a prova
de que o mapa é de facto conforme* — um mapa que preserva ângulos entrega o ângulo que o
domínio encomendou. ⇒ a conformalidade **não reduz** o enviesamento, **muda-o de sítio**,
e o que fica a carregá-lo — a colocação dos pontos de bordo por comprimento de arco —
passou de suspeito a **causa nomeada**. A obra seguinte deixou de ser «tentar outro mapa».

**How to apply:**
1. Meça a mesma grandeza **nas duas pontas** da fase que está a mexer (encomendado vs
   entregue). Uma cura só se julga com a folga entre elas, nunca com o valor final.
2. Se a folga fechar e o total não mexer, **a cura é um instrumento**: ela isolou a
   fase. Registe a transferência com as duas colunas e mantenha o interruptor no código
   como testemunha de controlo.
3. ⚠️ Uma cura pode ganhar na fixtura limpa e **perder** na real por outro eixo — o mapa
   conforme é fiel ao ângulo e não à **área**, e pagou-o em aspecto, dobras e aresta
   máxima na escultura. *Meça as duas, e rejeite pelo produto.*

Irmãs: [[feedback_a_bucket_nobody_fills_reads_as_perfect]] ·
[[feedback_two_good_hypotheses_failing_refutes_the_family_not_the_two]] ·
[[feedback_a_correct_mechanism_can_prescribe_the_wrong_cure]] ·
[[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]]
