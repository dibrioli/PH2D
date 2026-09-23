---
name: feedback-on-a-gpu-a-prune-charges-divergence
description: "Numa CPU uma poda é grátis; numa GPU ela cobra DIVERGÊNCIA, e o imposto tem o tamanho da poda — medido 11,8× contra 12,1×, wave cancelada"
metadata:
  type: feedback
---

⛔⛔⛔ **Uma poda por ramo-e-limite é um algoritmo de CPU. Numa GPU ela cobra DIVERGÊNCIA, e o
imposto pode ter exactamente o tamanho do que ela compra.**

Medido em 2026-09-23 (`ph2d-app-field3d`, `diag_o_custo_de_uma_aresta_lida_contra_dobrada`). A
`W9` tinha escrito que a cura do torno *«já existe — para a CPU»* e que a obra era *«levá-la ao
dispositivo»*. A MESMA aritmética em seis formas, `1 048 576` pontos, compilação fora do relógio,
inclinação em `N` (o que cancela despacho, leitura e compilação):

| forma | × a forma DOBRADA (literais no texto) |
|---|---:|
| `storage`, acesso uniforme | `3,2×`–`3,3×` |
| **`uniform`, acesso UNIFORME no grupo** | **`1,6×`** |
| `storage`, acesso DIVERGENTE (4 listas por grupo) | **`12,1×`** |
| `uniform`, acesso DIVERGENTE (4 listas por grupo) | **`17,3×`** |

⭐ **E a curva é LINEAR no número de listas distintas que um grupo lê** (1 · 2 · 4 · 8 →
`0,110` · `0,167` · `0,290` · `0,533 ms` em `storage`), com a pegada a caber na cache de 1.º nível
⇒ o que move a coluna é a serialização das leituras, não a memória.

⇒ A poda do perfil comprava `11,8×` menos arestas e a divergência cobrava `12,1×`: **ganho
`0,97×`**, e `0,28×` no pior caso — *três vezes e meia mais lenta que a fita desenrolada*.

**Why:** numa CPU uma poda é um `if` que salta trabalho, e o salto é grátis. Num SIMD as threads
de um grupo executam **a união** dos caminhos e o hardware **serializa** acessos a endereços
diferentes: a poda deixa de subtrair trabalho e passa a somar acessos. *O que decide não é quantos
dados uma amostra precisa, é quantas LISTAS DIFERENTES um grupo lê.*

**How to apply:**
1. Antes de levar uma estrutura de aceleração (BVH, grelha, índice) de CPU para GPU, meça o
   **imposto de acesso** na forma em que ela vai ser lida — e meça-o com a **divergência** que ela
   de facto terá, não com um acesso uniforme. ⭐ A sonda é pequena: a mesma lei em duas formas,
   inclinação em `N`, a compilação fora do relógio.
2. ⚠️ **O tipo de buffer escolhe-se pela COERÊNCIA do acesso, não pelo tamanho dos dados**: um
   `uniform` é o mais barato com acesso uniforme e o **mais caro** sem ele (`43×` do próprio caso
   uniforme a oito listas, contra `8×` de um `storage`).
3. ⭐ O desenho que sobrevive é o **coerente por grupo**: uma lista por grupo de threads, indexada
   pelo `workgroup_id`, e a unidade coerente por construção é o **ladrilho do ecrã** — que é por
   onde o caminho de CPU já cortava.
4. ⚠️ **Cronometrar com uma porta que compila lá dentro mede o compilador** — a
   `ph2d_field_gpu::probe::evaluate` tinha esse defeito, e a cura foi uma porta que compila fora e
   devolve o mínimo de `N` despachos.

⭐⭐⭐⭐ **E a segunda metade da lição, medida no mesmo dia: a granularidade que é COERENTE não
poda, e a que PODA não é coerente — e o produto das duas é ~constante.**

A saída óbvia da tabela de cima é *«uma lista por GRUPO de threads»*, e o grupo coerente por
construção é o ladrilho de `8×8` px que um `workgroup_size(64)` cobre. Medido pela aritmética de
região do próprio produto, no vaso a `1920×1080`:

| granularidade | poda p50 | poda pior | imposto | ganho p50 | ganho pior |
|---|---:|---:|---:|---:|---:|
| célula em `(u, v)` (fina) | `11,8×` | `3,4×` | `12,1×` | `0,97×` | `0,28×` |
| **ladrilho `8 px`, sem fatias** (coerente) | `2,7×` | `1,1×` | `1,58×` | **`1,7×`** | **`0,7×`** |
| ladrilho `8 px` × `4` fatias | `5,2×` | `1,8×` | (rompe a coerência) | — | — |

⭐ **E encolher o ladrilho não ajuda** (`64 → 8 px` move a poda de `2,4×` para `2,7×`): o que limita
é a **PROFUNDIDADE** do tronco do ladrilho, que atravessa a peça e por isso vê quase tudo. Repartir
em fatias cura a poda e é exactamente o que faz as threads lerem listas diferentes. ⇒ **a wave foi
recusada**: `1,7×` na mediana e uma PERDA no pior caso.

**How to apply (a segunda metade):** antes de desenhar uma consulta para GPU, ponha as duas colunas
lado a lado — *quanto a poda compra nesta granularidade* e *quantas listas um grupo lê nela* — e
multiplique. Se o produto não for confortavelmente maior que `1`, a estrutura de aceleração não
transfere, **por muito bem que ela funcione na CPU**.

Ver [[feedback_linear_in_the_count_means_dynamic_work_not_text_size]] ·
[[reference_topic_measurement_discipline]] ·
[[feedback_timing_one_call_of_a_new_structure_measures_the_compiler]].
