---
name: feedback_a_sampled_maximum_that_becomes_a_safety_bound_errs_only_downwards
description: Máximo por amostragem erra sempre PARA BAIXO — e se ele vira um limite de segurança, o erro é o lado perigoso
metadata:
  type: feedback
---

Um máximo obtido por **amostragem** nunca está acima do verdadeiro: o erro é **sempre para baixo**.
Se esse máximo vira um **limite de segurança** (um divisor de Lipschitz, um teto de passo, um
orçamento), o erro está sempre no lado que **quebra a garantia** — ao contrário de uma média, onde
ele só faz ruído.

Caso medido (`line/3DModeling`, W128, superfórmula de Gielis, 2026-09-06). O divisor do campo é
`max‖∇g‖`, achado por varredura. Sobre `1 728` combinações de parâmetros, o **défice** de cada
tentativa:

| a varredura | pior défice |
|---|---:|
| grelha uniforme na variável do produto (`θ`), `512` amostras | **`16,3 %`** |
| + bracket e secção áurea em cada pico | `4,5 %` |
| grelha com contagem a seguir à simetria | `23,9 %` |
| na variável da ESTRUTURA (`α`), um período, + secção áurea | `1,7 %` |
| + os **ângulos críticos** como candidatos explícitos | **`0,0000 %`** |

As quatro coisas que isso ensina, por ordem de força:

1. ⭐⭐⭐ **A variável de amostragem é parte da correcção.** A feição vivia em `α = m(θ+π)/4`: de
   largura `Δα`, ela mede `4Δα/m` em `θ`, e a `m = 24` encolhe **24×**. ⛔ *Adensar não é a cura* —
   o erro de uma grelha sobre um pico cai como `1/n²`, e isto corre **por quadro**.
2. ⭐⭐ **A fórmula diz onde estão as suas próprias esquinas.** O supremo estava numa
   **descontinuidade** (`α = kπ/2`), que é um limite lateral: nenhuma grelha o atinge. Avaliar em
   `kπ/2 ± ε` levou o défice a zero.
3. ⭐ **Uma janela de varredura são DOIS números.** Escrevi só a largura e varri `[−w, w]` onde a
   janela real era `[mπ/8, 3mπ/8]`: o divisor saiu **`69 %` curto**.
4. ⛔ **`>=` dos dois lados faz de cada amostra de um patamar um «pico».** O refinamento passou a
   correr nas `512` e o custo saltou para **`1,45 ms` por quadro**.

**Why:** um bound de segurança só é um bound se **majorar**. Quando ele sai de uma amostragem, a
única leitura honesta é *quanto ele fica ABAIXO de uma referência independente*, e essa referência
tem de correr na **variável do produto** — senão ela partilha o erro que se está a medir.

**How to apply:** ao escrever um máximo amostrado que vira garantia — (1) meça o **défice** contra
uma referência densa e independente, sobre a **caixa inteira de parâmetros** e não sobre um corpus
simpático (o meu lia `1,0000` em 20 células e `16,3 %` de erro nas 1 728); (2) amostre na variável
em que a feição tem largura **constante**; (3) junte os pontos críticos **analíticos** como
candidatos; (4) refine só onde a grelha **bracketou** um pico, com `>` de um lado.

Relacionado: [[feedback_interval_error_is_first_order_so_a_uniform_grid_pays_n_cubed]] ·
[[feedback_a_ruler_placed_after_the_tidying_step_measures_the_tidying]] ·
[[reference_topic_implicit_field_laws]] · [[reference_topic_measurement_discipline]]
