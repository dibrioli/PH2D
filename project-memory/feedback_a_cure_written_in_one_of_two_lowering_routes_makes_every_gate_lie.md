---
name: feedback-a-cure-written-in-one-of-two-lowering-routes-makes-every-gate-lie
description: "Duas rotas baixam o mesmo documento: a cura numa delas deixa TODOS os gates verdes sobre um programa que ninguém corre"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-08-31T00:50:02.892Z
---

Quando um módulo tem **duas rotas** que baixam a mesma estrutura (documento → árvore,
IR → código, modelo → buffer), uma cura escrita numa delas é **invisível** aos gates que
avaliam pela outra — e os gates ficam **verdes sobre um programa que ninguém corre**.

Caso medido (PH2D, `line/3DModeling`, 2026-08-30 — report do Enio *«piorou os artefatos
ao rotacionar»*): `ph2d-field-eval` tem `compile_with` (as sondas, os gates, o `Field`) e
`hybrid::Builder` (**o produto**, único que sabe o que é uma escultura). As duas duplicavam
a linha que baixa uma FOLHA. O divisor da aresta entrou só na primeira ⇒ **o campo do
traçado vinha `8×` o campo dos gates**, a marcha andava o passo cheio sobre o campo cru e
atravessava a superfície, enquanto **catorze** gates diziam `passo × ‖∇f‖ ≤ 0,80`.

**Why:** um gate mede o *caminho que ele mesmo percorre*, não o produto. Duas rotas para a
mesma pergunta não divergem no dia em que nascem — divergem no dia em que alguém cura uma.

**How to apply:**
- Antes de curar, **grepe quantas rotas baixam aquilo**. Se forem duas, a cura desce para a
  função que as duas chamam — *uma lei escrita em dois sítios ainda não é uma lei, só uma
  PORTA é* ([[feedback_paint_and_dispatch_must_read_the_same_source]]).
- Escreva o gate **estrutural** que pergunta se as duas portas concordam (não «o divisor
  está lá», que a próxima rota também não terá): ele não sabe o que é a cura, ele compara.
- ⚠️ **Se os gates ficaram verdes durante o defeito, o veredito não é «faltava um gate»: é
  que os gates avaliam por outra porta.** Confira por qual porta o produto avalia
  ([[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]]).
- ⭐ **Corolário medido:** uma mutação que **sobrevive** pode estar a dizer que o código
  mutado é **inerte**, não redundante. O orçamento da marcha tinha uma sobrevivente
  declarada; ela sobrevivia porque a cura que ela compensava nunca chegava ao produto — com
  a cura na porta única, ela **morre** ([[reference_topic_mutation_proofs]]).
