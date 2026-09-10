---
name: feedback-an-exact-analytic-law-can-still-be-an-unsafe-ceiling
description: "Uma lei geométrica EXACTA pode não ser um TECTO: a construção degenera antes da geometria, e só a varredura da família inteira o mostra"
metadata:
  type: feedback
---

⭐⭐⭐ **Uma lei derivada analiticamente e verificada num ponto pode estar exacta e ainda assim ser um
tecto INSEGURO** — porque quem degenera primeiro não é a geometria, é a **construção**.

**Why:** medido em 2026-09-09 (W144). O tecto do chanfro das quinas de uma estrela sai de
`outer − t·cos α_ponta > inner + t·cos α_vale`, e cada metade bate a medição **a cinco casas**
(recuo `0,029903` contra `0,0299034`; vale `0,18 + 0,01808` exacto). Varrido em **112 estrelas**, o
ponto em que a peça deixa de ser uma estrela é `1,000` da conta para `n` par e **`0,7483`** para
`n = 11`. ⇒ *a conta estava certa sobre a FORMA e errada sobre o PROGRAMA.*

⭐⭐ **A causa foi encontrada e tem nome geral: um operador aplicado por DOBRAGEM aos pares não é
local.** Uma união dobrada (`a ∪ b ∪ c ∪ …`) constrói o chanfro do passo `k` a partir da **forma já
acumulada**, não do vizinho — logo ele acrescenta material longe da junta que devia tratar.

**How to apply:**
1. ⛔ Nunca promova uma lei analítica a cerca sem **varrer a família inteira** dos parâmetros que ela
   governa — o caso de omissão pode ser o único que se comporta.
2. A folga que sobra escreve-se **com a tabela ao lado** e diz de que recurso é (aqui: o preço do
   dobrar aos pares), não «por segurança».
3. Ao ver o desvio, pergunte se o operador é aplicado por **dobragem** — se for, teste a versão
   n-ária: ela costuma repor a lei, e o preço dela é a outra metade da decisão.
4. ⚠️ E a régua que decide «ainda é a forma?» é a primeira suspeita: três foram deitadas fora nesta
   wave, todas por suporem onde estava o extremo.

Irmãs: [[feedback_a_blend_never_rounds_an_edge_that_lives_inside_one_piece]] ·
[[feedback_a_sampled_maximum_that_becomes_a_safety_bound_errs_only_downwards]] ·
[[feedback_a_fence_can_guard_two_things_and_name_only_one]]
