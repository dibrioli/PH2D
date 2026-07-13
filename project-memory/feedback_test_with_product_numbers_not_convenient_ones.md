---
name: feedback-test-with-product-numbers-not-convenient-ones
description: Um teste que escolhe a constante mais conveniente é uma tautologia — use os números do PRODUTO (câmera real, janela real)
metadata:
  type: feedback
---

O balde do Flip (W4) foi entregue **verde em 1251 testes e incapaz de preencher um círculo**.

A espessura do traço é guardada em **px de TELA**; os pontos, em **unidades de documento**. A
conversão (`× px_to_world`) nunca foi aplicada. No zoom padrão (`height_world = 10`, janela 1080p →
`px_to_world ≈ 0,0093`), um traço de 6px virava uma linha de **3 unidades de mundo (~324px!)**
atravessando um desenho de 2,8 unidades: o clique caía sempre *dentro* do traço e o balde respondia
**"clicked on a line", sempre**.

Os cinco unit tests passavam **`px_to_world = 1.0`** — o **único** valor em que um px de tela vale
uma unidade de documento, e portanto o único em que o bug é invisível. Eles não testavam o produto;
testavam um mundo onde o bug não existe.

**Why:** ao escrever um teste, a tentação é escolher constantes redondas (1.0, identidade, escala
neutra). Mas é exatamente nas constantes neutras que os **erros de unidade e de espaço** somem — e
erro de unidade é a classe que mais mata feature no PH2D (px de tela × mundo × local × buffer).

**How to apply:** todo teste de um caminho que **cruza espaços** usa os números do PRODUTO — a
câmera real, a janela real, a escala real do objeto. Se a matemática só funciona quando um fator é
1.0, o teste não prova nada. Gate que virou vermelho no instante em que existiu:
`the_bucket_fills_at_the_real_camera_scale` (`shells/desktop/src/flip_fill.rs`).

**Corolário:** quando um valor de teste é escolhido "porque simplifica", escreva ao lado *por que*
ele não esconde nada — ou troque-o pelo número real.

Relacionadas: [[feedback_derived_coordinate_seed_must_match_sample]] ·
[[feedback_pixel_center_vs_edge_coord]] · [[feedback_harness_reproduces_mechanism_not_context]] ·
[[feedback_tool_unit_green_integration_dead]]
