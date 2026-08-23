---
name: feedback_a_better_instrument_can_make_the_product_worse_and_that_is_the_finding
description: Quando uma propriedade melhora muito (medida na régua da própria promessa) e o produto PIORA, a premissa que ligava as duas está refutada — e o mecanismo lê-se na fase intermédia que a melhoria deixou de mascarar
metadata:
  type: feedback
---

Melhorar uma propriedade **por um fator grande, medido na régua que mede exactamente a
promessa dessa propriedade**, e ver o produto **piorar**, não é um fracasso da
implementação: é a **refutação da premissa** que ligava as duas.

⭐ **E o mecanismo costuma estar visível na fase INTERMÉDIA** — a que a versão pior
estava a mascarar.

**Why:** medido no quad remesh (2026-08-23). Meia semana assentou em *«um mapa mais
conforme dá quads mais quadrados»*. O LSCM (o único achatamento sem condição de
fronteira) levou o erro conforme de **`4,32` a `1,01`** — quase perfeito — e o
enviesamento **piorou de `18°` para `28°`**, com as dobras a ir de `0` para `68`.

⭐⭐ O mecanismo estava na coluna do **domínio**, que ninguém olhava porque com o mapa
antigo ela era boa: `1,0° → 21,4°` (e `18,7° → 50,8°`). *Num domínio conforme, os pontos
de bordo — postos por comprimento de arco — caem em posições muito desiguais, e a grade
entre eles nasce torta.* ⇒ o mapa antigo, mau, **redistribuía** os pontos e escondia a
discordância; o mapa bom deixou de a esconder. **A conformalidade não removia o defeito:
mascarava-o.**

⇒ A conclusão inverteu-se: o constrangimento não era o mapa (quatro achatamentos
medidos, família fechada) e era **maior** do que a medição anterior dizia.

**How to apply:**
1. ⭐ **Meça a propriedade que a mudança promete, na régua dela**, ao lado do resultado
   do produto. Sem isso, «melhorou X e o produto não mexeu» não distingue *X não é o
   constrangimento* de *o meu X tem um bug*. Dê a essa régua **controlo positivo e
   negativo** (identidade → `1,0`; esticão de `3×` → `3,0`).
2. Quando as duas discordam, **olhe a fase intermédia** — a que estava boa antes. Uma
   melhoria que piora o produto quase sempre destapa um defeito a jusante.
3. ⚠️ **Confirme a CONVERGÊNCIA antes de concluir.** O mesmo LSCM a `4 000` rondas dava
   `1,82` e a `100 000` dava `1,01` — e as duas contam histórias diferentes. *Dois
   solvers diferentes não partilham um teto de espera*: o número de rondas do vizinho não
   é o seu.
4. A fixtura tem de **distinguir** as duas versões: uma calota suave deu Tutte `1,046` e
   LSCM `1,049` — empate, e o gate reprovava sobre código correcto. A que discrimina foi
   uma faixa alongada e **plana** (alvo exacto `1,0`, e o alongamento é o que castiga o
   concorrente).

Irmãs: [[feedback_a_cure_that_moves_the_defect_names_it]] ·
[[feedback_a_bucket_nobody_fills_reads_as_perfect]] ·
[[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]] ·
[[feedback_two_good_hypotheses_failing_refutes_the_family_not_the_two]]
