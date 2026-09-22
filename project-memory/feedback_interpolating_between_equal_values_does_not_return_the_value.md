---
name: feedback_interpolating_between_equal_values_does_not_return_the_value
description: Interpolar entre três valores iguais não devolve o valor em `f32` — uma região «chapada» tem bits todos diferentes, e um desenho que contasse com o contrário mede o oposto no produto
metadata:
  type: feedback
---

Uma média ponderada `Σ wᵢ·cᵢ` com todos os `cᵢ` **iguais** não devolve `c` em `f32`: os pesos somam
`1` com erro de último bit, e o resultado difere por alguns ULP de amostra para amostra. ⇒ **uma
região visualmente chapada tem bits todos diferentes**, e toda a lei que dependa de igualdade
BIT-A-BIT lê o contrário do que o olho diz.

**Medido** (2026-09-21, tinta fina): o plano de amostras vai para o `.ph2dproj` em CORRIDAS de
amostras iguais. A sonda que justificou o desenho corria sobre `Tinta::nova` (uma constante
repetida) e deu **`1` corrida, `0,000×`**. O produto usa `Tinta::semeada` — a interpolação da cor
por vértice —, e ali o `8x` da peça de fábrica lê `3 145 729` corridas (`0,542×`) com cor chapada e
**`6 291 458` (`1,083×`)** com cor variada: *pior do que guardar cru*.

⇒ a cura foi **duas formas com o escritor a escolher a menor**, por uma conta exacta sobre as
corridas — e a decisão de NÃO as ter também estava escrita, com o número errado ao lado.

**Why:** a intuição «isto é tudo a mesma cor» é sobre o VALOR e a lei é sobre os BITS. Uma sonda
construída com um construtor que não interpola confirma a intuição e some com o caso real —
e o erro é para o lado caro, porque a tabela parece medida.

**How to apply:** ao medir uma propriedade de IGUALDADE (corridas, deduplicação, memos por valor,
caches por chave de bits), a fixtura tem de vir do construtor que o PRODUTO usa, não do mais
simples que produz «a mesma coisa». E o gate que a defende mede a fixtura interpolada, não a
constante: aqui foi um gate VERMELHO — a barra do desenho antigo sobre a fixtura semeada — que
apanhou o erro, e não uma releitura.

Vizinhas: [[feedback_a_fence_that_never_bites_hides_the_one_that_does]] ·
[[feedback_python_replace_silent_noop_after_fmt]] ·
[[feedback_a_promise_written_in_a_debug_assert_is_not_a_promise_the_product_makes]]
