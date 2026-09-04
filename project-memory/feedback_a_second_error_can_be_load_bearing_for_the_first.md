---
name: feedback-a-second-error-can-be-load-bearing-for-the-first
description: "Uma lei pode parecer boa porque erra em DUAS coisas na direcção que se compensa — curar uma expõe a outra, e o sintoma é a cura não melhorar com o knob que devia curá-la"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-03T16:02:01.206Z
---

Quando uma cura correcta **piora** o produto, o reflexo é *«a régua premeia o defeito»* — e às vezes
é. ⚠️ **A outra explicação é mais dura: a lei antiga erra em DUAS grandezas, e o segundo erro está a
compensar o primeiro.** Curar um expõe o outro.

Caso medido (PH2D, `line/3DModeling`, 2026-09-02 — o plano do chanfro). O corte de hoje desce
`c/sin 2α`, que numa ponta de estrela é `1,61×` o número pedido. Honrar o recuo piorou a peça, e a
hipótese de régua era boa: *«ela corta menos, logo sobra mais ponta»*.

⭐ **A varredura matou-a:** de `1/8` a `7/8` do limite do chanfro, o pior giro fica em `~85°` **do
princípio ao fim** — a lei honesta **não melhora com mais corte**. ⇒ não é quantidade.

A causa é a **normalização**: o plano é `(a+b)·escala` e `‖∇(a+b)‖ = √(2+2κ)`, logo a lei que ship
tem `‖∇plano‖ = 0,4644` (subestima `2,15×`) e a honesta `1,0000`. A região em que o filete mistura
sobre o plano é `{|plano| < r}` — um campo `2,15×` menor torna-a **`2,15×` mais larga**. *A lei
antiga escondia o vinco porque errava numa segunda coisa, na direcção certa.*

**How to apply:**
- ⭐⭐ **Varra o knob que a hipótese de régua prevê que cura.** Se a curva for **plana**, a
  explicação de quantidade caiu e o defeito é estrutural. *Uma leitura num ponto só não distingue
  «corta menos» de «erra a escala».*
- ⭐ Procure o segundo erro **na mesma expressão**, e nas grandezas que ninguém mede porque são
  «seguras»: aqui era um `‖∇f‖` **subestimado**, que a doutrina da casa declara seguro para a marcha
  — e que estava a fazer trabalho de PRODUTO sem ninguém saber.
- ⛔ Quando as duas metades se movem juntas, a cura deixa de ser um remendo e passa a ser wave com
  espec. Escreva isso, com os dois números, em vez de tentar meia cura
  ([[feedback_curing_half_a_family_can_leave_the_other_half_worse]]).
- ⚠️ E confira o **prognóstico** da nota anterior contra a medição: a minha dizia que o preço da
  saída alternativa seria o *vértice*, e o que a medição cobrou foi a **sobreposição de dois arcos**
  numa faceta estreita — outro mecanismo, outra cura.
