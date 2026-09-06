---
name: feedback_a_constant_folded_into_a_tree_is_recomputed_wherever_the_tree_is
description: Uma constante calculada dentro do construtor de uma árvore corre onde a ÁRVORE é construída, não onde a forma é criada
metadata:
  type: feedback
---

Uma constante cara calculada **dentro do construtor** de uma expressão (uma árvore, um shader, um
plano de query) corre tantas vezes quantas a **expressão** for construída — e isso pode não ter
nada a ver com quantas vezes a **coisa** é criada.

Caso medido (`line/3DModeling`, auditoria da W128, 2026-09-06). O divisor de Lipschitz da
superfórmula sai de quatro varreduras de uma dimensão, escritas dentro do `sd_superformula`. Elas
parecem correr «uma vez por forma». Mas o traçador **especializa a árvore por ladrilho × fatia de
profundidade**, e o passo de especialização percorre **todos os nós** e reconstrói cada folha:

| a cena (um quadro a `640×360`) | varreduras |
|---|---:|
| a forma **sozinha** (sem especialização) | `6` |
| a forma **ao lado de um desenho** (que LIGA a especialização) | **`3 852`** |

⚠️ **`642×`, e a imagem estava perfeita** — *um defeito só de CUSTO é invisível a todo gate de
saída*. O que o apanha é um **contador**, e ele tem de correr no caminho em que a especialização
liga (aqui: com um perfil na cena).

**Why:** o construtor de uma árvore é um sítio que parece «uma vez por objecto» e é «uma vez por
avaliação do pipeline». A regra é a mesma de um `format!` dentro de um laço de render: a pergunta
não é *quantas coisas há*, é *quantas vezes este código corre*.

**How to apply:**
1. Ao pôr uma conta cara num construtor de expressão, **conte** quantas vezes ele corre no caminho
   do produto — com um contador atómico, não com um relógio (um contador é imune à carga da
   máquina, e nesta workstation nenhuma leitura de tempo vale acima de `load ~5`).
2. Se a conta é uma **função pura** dos parâmetros, memoize-a. ⚠️ *Isto não é o cache de estado
   derivado que envenena o undo*: aquele guarda um resumo do **documento**; este guarda o valor de
   uma função, e não há nada para invalidar. `thread_local` evita trava e corrida.
3. Escreva o **gate de custo** no mesmo commit, e faça-o correr no caminho que liga a
   especialização — senão ele mede a cena fácil e fica verde para sempre.
4. Antes de aceitar «esta forma é inerentemente mais cara», procure os **atalhos exactos**: aqui,
   `n = 2` é o quadrado e `n = 1` é a raiz, e isso tirou `42 %` do quadro sem mudar um bit.

Relacionado: [[feedback_a_per_glyph_outline_effect_is_paid_every_frame]] ·
[[feedback_a_sampled_maximum_that_becomes_a_safety_bound_errs_only_downwards]] ·
[[reference_topic_measurement_discipline]] · [[reference_topic_implicit_field_laws]]
