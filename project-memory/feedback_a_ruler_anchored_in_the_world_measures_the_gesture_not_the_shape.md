---
name: feedback_a_ruler_anchored_in_the_world_measures_the_gesture_not_the_shape
description: "Medir a FORMA de uma coisa que se move a partir do ponto de mundo onde ela nasceu soma o percurso à deformação — ancore a régua no próprio objeto (o centroide), e desconfie de toda grandeza DERIVADA que troca intervalos por pontos"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7c66683a-d39b-477a-ad5a-a6529d503e36
  modified: 2026-08-23T22:21:36.480Z
---

Gate da cena `=87` (`motion.soft_body`, corpo pendurado): *"a envergadura foi de
1,260 para 3,700 (2,94×) — o corpo desfez-se"*. O corpo estava **perfeitamente
inteiro**. A régua media a distância de cada partícula ao ponto de mundo onde a
cena pousa o corpo — e um corpo pendurado **balança e cai**, então aquela
distância cresce com o GESTO, não com a deformação.

**Why:** *forma* é uma propriedade **interna** — invariante a translação. Toda
régua de forma tem de ser ancorada no próprio objeto (centroide, caixa própria,
eixo principal). Uma âncora de mundo mistura duas grandezas que a suíte não sabe
separar, e o número resultante **passa em todo type check**: ele é um `f32`
plausível, cresce quando a coisa se estraga, e cresce também quando ela
simplesmente anda.

⚠️ **O modo de falha é o caro: ele acusa código CORRETO.** Um gate assim manda a
próxima janela procurar um defeito no solver.

**How to apply:**
1. Antes de escrever a barra, pergunte **de que ponto** a régua mede — e se esse
   ponto se mexe com o fenómeno. Se mexer, ancore no objeto.
2. A mesma armadilha na segunda metade da mesma wave, com outro rosto:
   **a EXTENSÃO conta intervalos e a CONTAGEM conta pontos.** Uma malha `16 × 8`
   mede `15s × 7s` — razão **2,143**, não 2. Derivar «a grelha equivalente» da
   razão crua devolvia `17 × 7`, que a lei a jusante cortava a **metade** das
   regiões, em silêncio. *Toda grandeza derivada que atravessa a fronteira
   discreto↔contínuo carrega um ±1; escreva-o na conta, não na cabeça.*
3. Quando um gate novo acusa código que você acredita correcto, **suspeite da
   régua antes do algoritmo** — nesta linha ela se corrigiu **três vezes** antes
   do código ([[reference_topic_quad_remesh_rulers]]), e nas três a lição foi
   esta. Irmã de [[feedback_an_unlabelled_probe_column_gets_read_backwards]].
