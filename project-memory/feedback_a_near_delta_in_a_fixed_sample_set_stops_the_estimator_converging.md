---
name: feedback-a-near-delta-in-a-fixed-sample-set-stops-the-estimator-converging
description: Um integrando quase-delta amostrado por um conjunto FIXO de direcções não faz ruído — faz o estimador deixar de convergir, e mais amostras leem PIOR
metadata:
  type: feedback
---

⛔⛔⛔ **Um integrando com um pico estreito (um lóbulo especular, um polo `1/r²`) amostrado por um
conjunto FIXO de direcções não produz ruído: produz um estimador que NÃO CONVERGE.** Medido na peça
do dono (`ph2d-field-render`, o ricochete a `256×256`), a quebra de segunda diferença no `p99`:

| direcções | com o pico | sem o pico |
|---:|---:|---:|
| `16` | `3,912` | `0,969` |
| `48` | `0,872` | `0,827` → `0,588` |
| `96` | **`6,292`** | `0,450` |

`96` lê **7×** pior que `48`. Tirado o pico, a coluna fica **monótona**.

⚠️ **E o sintoma que o dono vê não é granulado — são ARCOS.** Com um conjunto fixo, o pixel vizinho
usa as MESMAS direcções, logo o pico cai ou não cai em regiões **contíguas**: os fireflies ficam
**correlacionados** e desenham curvas na peça. *«Como se fosse muitas sombras duras» é a leitura
exacta de fireflies coerentes*, e quem tratar isso como ruído vai à cura errada — que aqui estava
**proibida por medição** (sortear por pixel faz a peça ferver ao rodar a câmera).

⇒ **A cura é tirar o pico do integrando, não amostrar mais.** Aqui: o recolhedor de hemisfério passa
a ler a parte **DIFUSA** do ponto acertado, com a divergência declarada (a luz que sai dali tem
mesmo especular; o que não se faz é **transportá-lo** por um recolhedor difuso — o modelo de toda
GI em tempo real).

⚠️ **O discriminador barato existe e é uma linha:** varra o número de amostras e veja se a coluna é
**monótona**. Um estimador consistente melhora; um com quase-delta salta. *Uma única medição a `48`
não distingue os dois casos.*

Ver [[feedback-the-owners-fixture-is-the-one-that-must-be-measured]] — nesta mesma jornada a caixa
de Cornell tinha um mecanismo VERDADEIRO e DIFERENTE (o polo `1/r²` de uma lâmpada a 6 cm do tecto)
que **não existe no produto**, e curá-lo não teria tocado num pixel da foto do dono.
