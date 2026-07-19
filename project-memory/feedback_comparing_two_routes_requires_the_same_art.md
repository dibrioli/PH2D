---
name: feedback-comparing-two-routes-requires-the-same-art
description: "Ao comparar dois caminhos de código, use a MESMA entrada — senão você mede a diferença entre as fixtures e a chama de diferença entre as rotas; e um gate que mora dentro do que ele defende não defende nada"
metadata:
  type: feedback
---

Flip, plano `10_regiao_por_curvas.md` §12 (2026-07-18). Uma fatia de plano mandava aposentar um
ramo do balde de tinta (`filled_shape_target`) porque ele *"virou caso particular"* da lei geral.
Antes de apagar, medi — e a primeira medição **estava errada de um jeito que quase virou a
história oficial**.

Ela comparou a **rota A numa forma fechada** com a **rota B numa célula de grade** — arte onde a
rota A nunca dispara. O número saía do *fixture*, não da rota. A conclusão que ela produziu
(*"o pincel Smooth é o que quebra"*) era falsa: medido na mesma arte, **sem seleção todos os
pincéis empatam em zero**, e o discriminador real era a **seleção** (o auto-masking exclui uma
região que seja traço à parte). A correção mudou os gates de lugar: uma fixture sem seleção
ficaria verde nas duas rotas e não provaria nada.

**Why:** comparar caminhos é uma subtração, e subtrair exige que tudo, exceto o caminho, seja
igual. Quando a rota A só existe em certas entradas, é tentador medi-la onde ela roda e medir a
B onde *ela* roda — e isso é exatamente a comparação que não se pode fazer. O resultado tem a
forma certa (uma tabela, dois números, um contraste) e é sobre outra coisa.

**How to apply:**

1. **Force as duas rotas na MESMA entrada**, mesmo que uma delas precise ser desligada por
   mutação para ceder o caso à outra. Se uma rota não consegue rodar naquela arte, essa é a
   descoberta — não é licença para trocar a arte.
2. **Um gate que mora dentro do que ele defende não defende nada.** O único gate próximo do ramo
   pinava que *a rota dispara*; ele seria apagado junto com a rota, e o produto ficaria verde.
   Ao proteger uma propriedade, pergunte **onde o gate mora** em relação ao código que a entrega.
3. **Uma nota de plano é uma HIPÓTESE, não uma autorização.** Ela foi escrita quando o ramo
   parecia redundante e envelheceu para uma instrução que teria removido uma propriedade
   aprovada em smoke. Irmão de [[feedback_a_deferral_notes_bar_may_exceed_the_projects_policy]] e
   de [[feedback_documented_decision_chesterton_fence]].
4. Corolário sobre *"derivado de"*: uma rota que **deriva** geometria de outra e a **copia** não
   herda as propriedades da fonte. Copiar é fotografar; o que muda depois não aparece na foto.
   Só uma costura fonte≠cozido (o padrão do ADR-0121) mantém a relação viva.

Irmão de [[feedback_the_approved_reference_may_already_be_in_the_product]] — as duas nasceram no
mesmo dia, do mesmo hábito: **medir a coisa certa é mais difícil que medir**.
