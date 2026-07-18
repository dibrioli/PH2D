---
name: feedback_a_magnitude_bound_misses_a_systematic_off_by_one
description: "Gate de paridade com tolerância: conte TAMBÉM quantos valores diferem — 'quão longe' e 'quantos' são perguntas diferentes, e erro sistemático é pequeno E onipresente"
metadata:
  node_type: memory
  type: feedback
---

Um gate de paridade que só limita a **magnitude** (`max |a-b| <= N`) é cego para a classe de bug mais
provável num port: o **erro sistemático de convenção**, que move quase tudo por **um** passo.

Medido (luz do impasto na GPU, 2026-07-18): tirar o `+ 0.5` do `quantise` do shader — virando
round-half-away em truncamento, exatamente a divergência CPU/GPU que a função existe pra abolir — moveu
**2375 de 16384 bytes, todos por 1 nível**. Passou tranquilo sob um limite de 2 bytes que eu tinha
orçado a partir do precedente do Bloom (≤5) e do Shadows/Highlights (≤4). O gate estava verde sobre um
shader errado.

**Why:** as duas perguntas medem fenômenos diferentes. *Quão longe* alguém foi capta ruído de ponto
flutuante (contração FMA, um ULP, uma fórmula genuinamente diferente) — é grande e raro. *Quantos*
foram capta convenção (arredondamento, off-by-one de índice, um clamp trocado) — é pequeno e
**onipresente**. Um limite de magnitude frouxo o bastante pra tolerar o primeiro é frouxo o bastante
pra esconder o segundo por completo.

**How to apply:** todo gate de paridade com tolerância asserta DUAS coisas — `max <= N` **e**
`differing <= M`, com `M` uns três ordens de grandeza abaixo do total. Imprima os dois (um gate que só
diz "sob a barra" esconde uma deriva caminhando até ela). E o teste de que os números prestam é a
mutação: troque a convenção de arredondamento e veja qual dos dois limites sangra — se nenhum sangra,
os dois estão errados. Vide [[feedback_loose_oracle_hides_systematic_bias]] e
[[feedback_cpu_gpu_rounding_conventions_diverge]].
