---
name: feedback_a_test_that_got_slow_is_a_cost_measurement_nobody_asked_for
description: "Um teste que passa de segundos a minutos depois da sua mudança está a medir um custo de produto — leia-o como medição, não como incómodo"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-28T23:25:19.215Z
---

Medido 2026-08-28: ao pôr um passe de acabamento dentro de `ph2d_quadchain::quads_from_mesh`,
o gate `when_the_chain_loses_the_mesh_the_artist_asked_for_comes_back` passou de segundos a
**mais de vinte minutos**. Lido como medição, ele dizia uma coisa sobre o **produto**: numa
peça dura o veto deita a saída fora, e o acabamento tinha corrido até ao tecto **duas vezes**
sobre uma malha que ninguém ia usar.

⭐ **A cura não foi tornar o passe mais rápido — foi a ORDEM.** O veto tem duas metades e só
a segunda precisa do acabamento: *«a peça continua fechada?»* é topologia, e uma relaxação
move vértices e mais nada. ⇒ a função parte-se em `..._raw` + acabamento, o veto de topologia
decide com a malha crua, e a suite inteira volta a `10 s`. A propriedade que torna a
reordenação legítima (o acabamento não muda o censo de arestas) ganhou gate próprio — sem ela
seria uma aposta.

**Why:** um teste corre o **caminho do produto**; quando ele fica lento, mediu um custo que
nenhuma sonda de perf estava a pedir — e frequentemente o custo está numa **ordem errada**
(trabalho caro feito antes da decisão que o descarta), não no passo caro em si.

**How to apply:** quando um teste fica lento depois da sua mudança, **não o marque como
lento**: pergunte *que trabalho é que ele acabou de pagar e quem o deita fora*. E ao mover
trabalho caro para depois de uma decisão, gateie a **propriedade** que torna a troca neutra
(aqui: «este passe não muda a topologia»). Relacionado:
[[feedback_a_cost_only_defect_is_invisible_to_every_output_gate]] ·
[[feedback_specialisation_pays_by_amortisation_count_the_rays_per_region]] ·
[[feedback_i_write_the_right_guard_and_do_not_gate_it]]
