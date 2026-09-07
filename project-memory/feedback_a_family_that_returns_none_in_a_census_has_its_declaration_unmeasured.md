---
name: feedback_a_family_that_returns_none_in_a_census_has_its_declaration_unmeasured
description: Uma família que faz `return None` num censo tem as declarações dela SEM régua — e o primeiro membro que o censo consegue construir expõe o buraco
metadata:
  type: feedback
---

Um censo derivado da enumeração (`for k in Kind::ALL`) parece completo, e não é: os membros que
devolvem `None` no construtor de representantes (porque precisam de um insumo que o censo não tem —
um contorno desenhado, um ficheiro, uma malha) **saltam TODOS os gates dele**. As afirmações que
esses membros declaram noutros sítios ficam sem régua nenhuma, indefinidamente.

**Medido** (2026-09-06): `fillet_inflates(Extrude) == false` viveu waves sem uma medição, porque o
`Extrude` faz `return None` no censo (*«precisa de um contorno desenhado»* — uma razão **correcta**).
O `Polygon` da W132 é da mesma família e **carrega o próprio contorno** ⇒ é o primeiro membro que o
censo alcança, e reprovou à primeira: `passo 1,0000 × ‖∇f‖ 1,0216`.

⚠️ E a causa **não era a forma nova**. A mesma régua, célula a célula:

| contorno | só chanfro | só filete | os dois |
|---|---:|---:|---:|
| quadrado (`90°`) | `1,0000` | `1,0000` | `0,7071` |
| pentágono convexo (`108°`) | **`1,0267`** | `1,0000` | `0,7071` |

O `Extrude` lê o mesmo em todas — defeito **pré-existente**, exposto por um vizinho construível.

**Why:** um `return None` é uma resposta declarada e honesta sobre *aquele* gate; o que ele não diz é
que **o resto do módulo continua a acreditar nas declarações daquela família**. A isenção não tem
data de validade e nada a acorda.

**How to apply:** ao escrever um `return None` num censo, pergunte *que AFIRMAÇÕES desta família
ficam agora sem régua?* — e, quando um membro construível dela aparecer, **inclua-o** em vez de
herdar a isenção. Cure pela **célula** medida, não pela família: aqui a cura foi `c != 0 && r == 0`,
porque a coluna «os dois» já lia `0,7071` e o `true` da família levaria o divisor a `4` e o campo a
`0,177` — 4× mais conservador do que o medido, num módulo já acima do orçamento.

Vizinhas: [[reference_topic_gate_discipline]] ·
[[feedback_a_gate_that_measures_the_representative_leaves_the_control_travel_unmeasured]] ·
[[feedback_a_new_feature_can_empty_an_existing_gates_population]]
