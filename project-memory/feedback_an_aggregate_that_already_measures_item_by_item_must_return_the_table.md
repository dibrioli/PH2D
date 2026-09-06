---
name: feedback_an_aggregate_that_already_measures_item_by_item_must_return_the_table
description: Uma régua que percorre N itens e devolve só o pior/a mediana deita fora o ÍNDICE — e «1 de 5 está mau» não começa cura nenhuma
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1246816c-63cf-414b-842d-663a8baa86ca
  modified: 2026-09-04T22:18:49.020Z
---

Quando o dono reporta *«muitas pontas boas no mesmo mesh e apenas uma ruim»*, uma régua
agregada (pior `p50`, mediana, quantas passam da barra) **não consegue responder qual**. As
três réguas de ponta do quad remesh já corriam ápice a ápice e **deitavam fora o índice antes
de devolver** — a cura começou por publicar a TABELA (`ph2d_quadfill::tip_rows`) e tornar as
agregadas **dobras** dela.

**Why:** um extremo ou uma média sobre a peça inteira nunca vê UM item, e — quando só um está
mau — também não diz **qual**, que é a pergunta com que uma cura começa. Esta linha pagou a
mesma forma quatro vezes: `edge_max` global cego ao quad de `0,02 × 0,30`, `χ` cego à almofada,
`ENTREGA` cega à ponta que engrossou, e agora `tip_deviation`/`tip_density` cegas a *qual*
ponta. Ver [[feedback_a_ruler_that_counts_leftover_defect_rewards_overshooting]].

**How to apply:** se a função já tem um laço sobre itens, devolva `Vec<Row>` e faça o agregado
ser uma dobra — com um gate que dobra a tabela à mão e exige o mesmo número (*uma régua nova
que muda o veredito da anterior não é a mesma régua*). E a fixtura do gate precisa de **dois**
itens com vereditos diferentes: com um só, «a tabela diz qual» é vazio, porque a única linha é
sempre a acusada.
