---
name: feedback_remeasure_a_documented_residual_before_curing_it
description: "Antes de curar um resíduo/custo anotado num handoff, RE-MEÇA: a causa que a nota declara pode ser refutada pelos próprios números dela, e o número pode vir de onde/como foi medido"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 1ad2e828-0576-4788-947a-5980948e93be
---

Uma nota de trabalho aberto registra um **número** e uma **causa**. O número costuma sobreviver; a causa é
uma hipótese, e o lugar/metrica da medição some junto com quem a fez. **Aconteceu duas vezes no mesmo dia**
(2026-07-18, linha Painter), e nas duas eu estava a caminho de otimizar a coisa errada.

**Caso 1 — a causa era refutada pelos números da própria nota.** *"A montagem do SMOOTH custa 10,5 ms
porque aloca o memo do blur"*. Media **10,02 @2048 e 9,70 @4096**: **plana na tela**. Uma alocação
canvas-inteira quadruplicaria. (Mecanismo: `vec![0.0; n]` de `f32` cai no `alloc_zeroed` — alocar é quase
grátis, **o que custa é o TOQUE**.) A causa real era serialização de trabalho independente.

**Caso 2 — o número vinha de ONDE e COM QUE MÉTRICA foi medido.** *"Cada dab normaliza o próprio aro =
produto sobre a lista de dabs; residual 0,0286"*. Medido: a razão de ondulação entre 2 px e 1 px é
**~1,0×** (uma corrugação por-dab quase dobraria) ⇒ **não era espaçamento**. E a janela `80..110`
atravessava o **fim do traço**, onde a crista rampa até zero, com uma métrica **pico-a-pico** — sobre uma
rampa ela reporta a rampa. No meio do traço: **0,0894 → 0,0180**. O item não existia; ficou planejado por
um ano.

**Why:** curar um defeito inexistente custa uma wave e pode piorar um comportamento correto (no caso 2 a
"cura" mexeria no banco, que já estava certo). E notas envelhecem sem avisar — quem as escreveu tinha o
harness na cabeça, não no papel.

**How to apply:** antes de tocar no código, faça a sonda mais barata que **distingue as hipóteses**:
varie a dimensão que a nota culpa (custo plano nela refuta a culpa) · **reordene** a sequência de medição
(o 1º item de um harness paga o aquecimento do processo — se o custo é do item, ele acompanha a
reordenação) · re-meça **em outra janela** e pergunte **o que a métrica de fato computa** (pico-a-pico
sobre uma rampa ≠ ondulação). Se a re-medição derruba o item, o entregável é o **gate que pina a
propriedade verdadeira** — para o próximo leitor não re-derivar o item falso a partir da nota.

Irmão de [[feedback_measure_perf_symptom_scale]] (meça a ESCALA antes da causa) e de
[[feedback_a_deferral_notes_bar_may_exceed_the_projects_policy]] (a nota diferida não é spec — aqui, nem a
CAUSA nem o NÚMERO dela são). Parentes de oráculo: [[reference_topic_oracle_discipline]] e
[[feedback_a_green_gate_may_be_green_by_accident]].
