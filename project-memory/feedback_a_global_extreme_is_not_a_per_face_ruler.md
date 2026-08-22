---
name: feedback_a_global_extreme_is_not_a_per_face_ruler
description: Um extremo global (máximo/mediana da malha) não vê um defeito por-face; meça a grandeza local e tire a barra de um oráculo
metadata:
  type: feedback
---

**Uma grandeza GLOBAL não mede um defeito LOCAL — e melhorá-la pode deixar o defeito
intacto.**

⛔ **Caso medido (quad-remesh, 2026-08-22).** No mesmo dia em que a `edge_max` da
peça caiu de **57 % da diagonal para 5,5 %** — e o relatório ficou verde em toda a
coluna — a foto seguinte do Enio veio com a palavra **«péssimo»**. As duas réguas
geométricas existentes eram a **aresta mais longa da malha inteira** e a **mediana
de todas as arestas**; o defeito era **por-face**: quads esmagados e enviesados em
faixas. *Um quad de `0,02 × 0,30` não move nenhuma das duas* — a longa dele está
muito abaixo da máxima, a curta afunda-se na mediana de dezenas de milhares.

**Why:** um extremo responde *"alguma coisa se partiu?"*; uma mediana responde *"o
tamanho médio está certo?"*. Nenhuma das duas responde *"esta face tem forma?"*, que
é a pergunta que o olho faz. E as três grandezas por-face são precisas **juntas**:
aspecto é cego ao losango (que tem aspecto `1`), enviesamento é cego ao rectângulo
`1 × 10` (que tem cantos rectos).

**How to apply:**
1. Quando um artista recusar uma saída que passa em tudo, pergunte **de que ordem é
   a grandeza que ele viu** — global, ou por-elemento? Se as suas réguas são todas
   agregados, a régua que falta é a local.
2. ⭐ **A barra da régua nova sai de um ORÁCULO, medido com o MESMO código nos dois
   lados** — nunca de uma opinião. «aspecto ≤ 4» soa razoável e não é medição; saber
   que a referência entrega `1,08` e **zero** faces acima de `4×` é que decide se
   estamos piores que ela ou se o defeito é outro. Ver
   [[feedback_a_binarys_output_is_a_legal_and_stronger_oracle_than_its_code]].
3. ⚠️ **A régua tem de ficar no caminho do PRODUTO, não numa sonda `#[ignore]`** —
   ver [[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]. Aqui
   ela passou uma hora a viver só numa sonda, que é onde uma régua não existe.
4. ⚠️ **O corpus de bancada tem de conter a peça de que alguém se queixou.** O nosso
   media nove peças de que ninguém se queixou e não a única que aparecia nas fotos —
   ver [[reference_topic_fixture_discipline]].

⭐ **E a régua nova também paga o diagnóstico:** foi por a mediana do enviesamento
não se mexer sob dezasseis rondas de relaxação que ficou provado que o defeito está
na **conectividade** e não nas posições — ver
[[feedback_if_relaxation_cannot_move_the_median_the_defect_is_in_the_connectivity]].
