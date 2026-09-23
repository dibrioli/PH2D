---
name: feedback-linear-in-the-count-means-dynamic-work-not-text-size
description: "Custo linear na contagem de instruções é trabalho dinâmico por amostra; se fosse tamanho de texto a curva seria um degrau — e isso decide se trocar texto por laço compra algo"
metadata:
  type: feedback
---

⭐⭐⭐ **Antes de trocar texto desenrolado por um LAÇO, pergunte de que o custo é feito — e a curva
responde: LINEAR na contagem é trabalho DINÂMICO por amostra; TAMANHO DE TEXTO daria um DEGRAU.**

Medido em 2026-09-23 (`ph2d-app-field3d`, o torno da cena `5` num quadro de `1920×1080`,
`--release`, `98 %` de CPU ociosa, mínimo de `3`). A mesma silhueta com o contorno reamostrado:

| linhas de WGSL | `18` | `87` | `198` | `684` | `1 342` | `2 639` |
|---|---:|---:|---:|---:|---:|---:|
| quadro | `4,32 ms` | `5,51` | `8,56` | `23,01` | `43,34` | `116,31` |

⭐ `0,028`–`0,031 ms` por linha ao longo de **duas ordens de grandeza**, e a extrapolação prevê a
cena real ao décimo (`31,8` contra `31,96`). Acima de `1 342` linhas vira super-linear (`0,056`),
que é onde a **ocupação** começa a morder.

**Why:** um shader grande é caro por duas razões diferentes — ele **executa** mais operações por
amostra, e ele **ocupa** mais registos, o que reduz quantas threads cabem num multiprocessador. As
curas são opostas: contra o trabalho dinâmico vale **PODAR** (fazer cada amostra tocar menos
dados); contra a ocupação vale **ENCOLHER O TEXTO** (um laço, uma consulta, um interpretador). ⛔ E
trocar texto por laço quando o custo é dinâmico não compra nada: o laço executa o mesmo trabalho
com uma leitura de buffer por cima.

**How to apply:** varra a grandeza (a contagem) por uma ordem de grandeza e olhe a FORMA da curva
antes de desenhar a cura. Linear ⇒ a obra é a poda, e o tecto dela mede-se contando quantos dados
uma amostra de facto precisa. Degrau ⇒ a obra é o tamanho. ⚠️ E meça o **PISO** (a mesma cena com a
folha trocada por uma primitiva analítica): sem ele o tecto do ganho é um palpite, e um modelo de
custo ajustado sobre um corpus prevê a média dele, nunca esta peça — aqui o modelo do corpus dizia
`0,039` e a cena diz `0,030`.

Ver [[reference_topic_measurement_discipline]] · [[feedback_an_operation_count_is_not_a_profile_and_the_build_profile_decides_the_number]].
