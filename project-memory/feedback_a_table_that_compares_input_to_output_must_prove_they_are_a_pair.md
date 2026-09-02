---
name: feedback-a-table-that-compares-input-to-output-must-prove-they-are-a-pair
description: Uma tabela entrada→saída tem de provar que as duas linhas são o MESMO sujeito; chamei ENTRADA a uma saída e a conclusão inverteu-se
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-31T01:20:49.211Z
---

Em 2026-08-30 (`line/quadextract`) pus numa tabela só cinco ficheiros `.obj` do Enio —
entrada, duas retopologias de referência e a nossa saída — e concluí que *«a peça que ele
mete no botão já tem a densidade certa nas pontas; o trabalho é parar de a deitar fora»*.

⛔ **A linha que dizia «a ENTRADA dele» era uma SAÍDA.** `sculpt_t003.obj` tem **zero
triângulos e valência máxima 6** — a assinatura de uma retopologia, não de uma escultura
(as esculturas dele têm tris misturados e valência até **144**). Com as entradas a sério a
conclusão **inverte-se**: elas medem `2,026` e `3,650` contra o alvo `0,59`, ou seja a
graduação **não chega** — tem de ser criada. A rota barata que eu tinha anunciado não
existia, e as duas recusas medidas que eu dava por ultrapassadas continuavam a ser as
únicas rotas.

**Why:** é a irmã exacta do achado de 28/08 em que a barra do oráculo foi lida a 1/9 da
densidade dele. Ali a régua omitia a **contagem de faces** dos dois lados; aqui omitia o
**sujeito**. Uma tabela é persuasiva pela forma, e a forma não distingue «medi A e B» de
«medi A e um primo de B». Nenhuma régua do repo dizia se dois ficheiros eram comparáveis.

**How to apply:**
- Antes de escrever uma tabela entrada→saída, imprima uma **assinatura invariante a
  movimento rígido** de cada peça — **área e volume** (o bounding box NÃO serve: roda com
  a peça, e separou como «peças diferentes» ficheiros que eram a mesma escultura).
- Pergunte a cada linha *«isto é um insumo ou um produto?»* e responda com uma coluna
  medida, não com o nome do ficheiro. Aqui a coluna é `tri/quad + valência máxima`.
- ⚠️ Uma conclusão que **reenquadra o trabalho** («afinal não é preciso inventar X») é
  precisamente a que merece a re-medição: ela dissolve recusas já pagas.
- A régua durável ficou em `piece_signature` (`shells/desktop/src/sculpt3d_photo_rulers.rs`).

Relacionadas: [[feedback_comparing_two_measurements_with_different_denominators_invents_an_effect]] ·
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]] ·
[[feedback_a_perfect_input_producing_a_worse_output_localises_the_damage]] ·
[[reference_topic_quad_remesh_rulers]]
