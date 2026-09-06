---
name: feedback-a-gate-that-measures-the-representative-leaves-the-control-travel-unmeasured
description: Todo censo de primitivas media a forma NO PONTO EM QUE ELA NASCE — uma peça podia estar perfeita ali e rasgar em quase todo o curso dos próprios controlos; o gate derivado das faixas declaradas achou 7 defeitos em 5 formas na 1.ª corrida, 4 sem relação com o report.
metadata:
  type: feedback
---

⭐⭐⭐ Medido em 2026-09-05 (doc 06 §122.2). O módulo tinha **24 gates** sobre as primitivas — caixa,
arestas vivas, alcance do filete, tecto de contagem — e **todos** construíam a forma pelo
construtor do catálogo, isto é, no **representante**. A nuvem passava em todos e rasgava a marcha
assim que alguém mexia num slider:

| linha | ao nascer | arrastada |
|---|---|---|
| `Width` a 0,5 | 0,94 | **1,29** |
| `Span` a 2,0 | 0,94 | **1,54** |

⚠️ **O gate de contagem da wave anterior não cobria isto**: ele varria a **contagem** (lobos, lados,
dentes) e os defeitos viviam nas linhas **contínuas**. *Ter um gate que varre uma linha não é ter um
gate que varre as linhas.*

O gate novo — `every_row_of_every_primitive_marches_safely_across_its_range` — deriva os pontos de
teste da **`Span` que cada linha declara** (três por linha) e escreve pela **porta do produto**
(`ph2d_field::dims` + `set_dim`). Na primeira corrida: **sete casos em cinco formas**, quatro deles
sem qualquer relação com o report que motivou a wave.

**Why:** um representante é, por construção, um ponto onde o autor viu a forma funcionar. Um
controlo é uma **faixa**, e o número de pontos que nunca foram medidos é toda ela menos um. E há
uma armadilha em cima: a coerção de uma cerca **estaciona no pior caso**
([[feedback_a_coercion_parks_at_the_fence_which_is_where_the_shape_degenerates]]), então o extremo
da faixa não é um caso exótico — é onde o artista chega ao arrastar até ao fim.

**How to apply:** quando um objecto declara as próprias faixas (uma tabela de dims, um manifesto de
params, um hint de slider), **derive o censo delas** em vez de escolher fixturas. A régua tem de
atravessar a porta do produto, senão mede outro programa
([[feedback_a_probe_with_the_knob_at_another_point_measures_another_piece]]). Irmãs:
[[feedback_a_gate_that_measures_the_rare_case_leaves_the_normal_one_without_a_ruler]] ·
[[feedback_a_literal_corpus_count_in_a_gate_makes_every_new_feature_edit_someone_elses_test]]
