---
name: feedback-a-clip-box-that-touches-the-piece-stalls-a-sphere-tracer
description: "Apertar a caixa de recorte até ENCOSTAR na peça faz o traçador de esferas parar em cima dela — a margem vive na marcha, nunca no bordo"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-01T19:26:32.612Z
---

Um traçador de esferas anda o **valor** do campo. Se a caixa de recorte é justa — e uma caixa justa
**toca** a superfície, é essa a definição dela —, um raio que entra em cima da peça lê `f ≈ 0`, dá
passos de tamanho zero e fica parado. A marcha honesta (passo fixo minúsculo) continua a andar, e as
duas divergem **sem que o campo tenha deixado de ser um minorante**.

Caso medido (PH2D, `line/3DModeling`, 2026-09-01): a `Ball::aabb` passou do cubo do raio para as
meias-extensões. Os `1 000` trios e os `100` pares de `‖∇f‖ ≤ 1` ficaram **verdes**; a imagem contra
a marcha honesta é que acusou — a roseta foi de `6` para `16` pixels divergentes em `6 844`, e
bisectando lei a lei nenhum bordo estava errado: *o que mudou foi só onde o raio começa.*

**How to apply:**
- ⭐ **A margem vive no RECORTE DA MARCHA, não no bordo.** O exportador quer a caixa justa (é dela
  que sai a resolução da grade) e o enquadramento também. Uma margem no bordo é paga por todos.
  ⇒ uma porta só (`march_clip`), e **os gates de gradiente têm de ler essa porta**, senão medem uma
  região mais pequena do que a que o raio visita.
- ⭐⭐ **Varra o número e escolha o DOBRO da primeira célula que cura**, nunca a primeira: aqui `0`
  reprovava, `0,005` curava, e ficou `0,01` — o preço de duplicar é `≤ +1,0` passo por raio
  ([[feedback_a_sweep_whose_cells_all_agree_has_not_chosen_anything]]).
- ⚠️ **O custo da margem é concentrado nas peças BARATAS** (uma caixa nua foi de `1,0` a `6,4`
  passos/raio; a pilha cara foi de `89,0` a `89,5`): um raio que não tinha nada a fazer passa a
  atravessar a casca. *Meça as duas pontas do espectro antes de dizer que a margem é grátis.*
- ⛔ **A margem é uniforme, tirada da MAIOR extensão.** Proporcional ao eixo, uma peça achatada
  recebe `~0` no eixo fino — que é exactamente onde o raio entra rente.
- ⭐ O gate certo mede a **propriedade** (*o campo é `> 0` em toda a fronteira do recorte*) e traz a
  metade que prova que ela não é vazia (*sem a margem, `N` fixturas encostam*)
  ([[feedback_a_bucket_nobody_fills_reads_as_perfect]]).
