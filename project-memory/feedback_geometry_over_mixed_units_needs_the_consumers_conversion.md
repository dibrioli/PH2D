---
name: feedback-geometry-over-mixed-units-needs-the-consumers-conversion
description: Ângulo/normal/inclinação sobre um campo cujos eixos têm unidades diferentes é lixo até os dois eixos virarem a MESMA unidade — e o conversor certo é o que o consumidor já usa
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 362c6c4f-9b8e-4ef4-b261-2d5564753f1a
---

Campo de altura do impasto: `x` é **texel**, `h` é **carga de tinta**. Duas grandezas geométricas foram
construídas sobre ele no mesmo dia:

- **Inflate** (`h += Depth · n_z`) — acertei, porque fui procurar **qual normal a LUZ usa**
  (`impasto_shade::shade` → `normalize([-dhx·DEPTH_UNIT_PX, -dhy·DEPTH_UNIT_PX, 1])`).
- **Chisel** (o V de `tan(ângulo)`) — **errei**, porque não fui. `tan(36°)` cru inclinava o plano em
  **0,73 load por texel**: 8,7 loads ao longo do footprint, **4× o teto do campo**. O "ângulo" era um
  número num espaço sem geometria dentro.

**Why:** um ângulo é razão de **comprimentos**; uma normal também. Se os dois eixos não estão na mesma
unidade, a razão não é um ângulo — é um número. E o conversor não é uma escolha: é o que o **consumidor**
(quem RENDERIZA) já aplica. Uma normal que a luz não usa é uma normal que o artista não vê.

**How to apply:** ao derivar qualquer coisa geométrica (normal · ângulo · inclinação · curvatura · "45°")
sobre um campo com eixos heterogêneos, **ache primeiro a constante que o renderizador usa** e passe por
ela. Grep no shader/no shade/no paint antes de escrever a fórmula.

E o sintoma no gate é característico: **o número sai numa escala absurda numa borda degenerada.** O meu
acusou "0,36 load poupado *no próprio eixo*" — meio texel de lado. Quando um valor de borda explode, é a
escala que está errada, não a tolerância.

Gateie **as duas pontas**: mutação que tira o conversor em CADA sítio que o usa. Uma lição gateada só onde
você já sabia olhar não está gateada ([[feedback_a_mutation_that_survives_may_mean_a_missing_gate]]).

Relacionadas: [[feedback_oracle_must_model_appearance_not_implementation]] (o oráculo modela a aparência) ·
[[feedback_derived_coordinate_seed_must_match_sample]] (coordenada derivada: quem escreve usa a transform de
quem lê) · [[feedback_test_with_product_numbers_not_convenient_ones]] (`px_to_world = 1.0` é o único valor
que esconde erro de unidade).
