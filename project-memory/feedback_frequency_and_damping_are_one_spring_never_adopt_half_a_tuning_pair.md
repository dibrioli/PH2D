---
name: feedback_frequency_and_damping_are_one_spring_never_adopt_half_a_tuning_pair
description: Quando o upstream muda UM número de um par de afinação, adoptá-lo sozinho mistura duas afinações — meça o par inteiro e o nosso, porque a recomendação deles é sobre a mola DELES
metadata:
  type: feedback
---

⭐⭐ **Dois coeficientes que descrevem o mesmo objecto são UM valor.** Quando o upstream sobe um
deles e publica o motivo, adoptar **só esse** não é seguir a recomendação — é misturar duas
afinações. *A recomendação deles é sobre a mola deles.*

**Medido (2026-08-29, `rapier2d` 0.35).** Ela dobrou o `contact_softness.damping_ratio` de `5` para
`10`, com um mecanismo escrito que descrevia **exactamente** o sintoma que tínhamos: *«softer
contacts settle deeper under load … wedging and creeping instead of resting»*. A nossa frequência é
`120 Hz`; a deles, `30`.

| afinação | pior ângulo em repouso | afundamento máx |
|---|---|---|
| ⭐ a nossa: `120 Hz` / `ζ 5` | **`0,04455°`** | **`0,000264`** |
| ⛔ só o `ζ` deles: `120 Hz` / `ζ 10` | **`24,28°`** | `0,000201` |
| o par INTEIRO deles: `30 Hz` / `ζ 10` | `0,27936°` | ⛔ `0,003780` |

⇒ **545× pior** com metade do par, e o par inteiro afunda **14×** mais que o nosso. A nossa
afinação ganha nos **dois** eixos.

⚠️ **A terceira célula é o que torna isto uma conclusão e não uma preferência.** Sem ela, «o deles é
pior» leria-se como *«a nossa é melhor por acaso»*; com ela vê-se que a diferença é o **acoplamento**
— o `ζ` deles é bom com o `f` deles.

**Why:** um SUSPEITO de auditoria com mecanismo publicado e sintoma coincidente é a hipótese mais
convincente que existe, e mesmo assim tem de ser medida. Aqui ela estava **refutada por 545×**.

**How to apply:** ao adoptar um número de um upstream, pergunte **de que par ele faz parte** e meça
**três** células: o nosso, o deles inteiro, e o híbrido. Escreva a tabela ao lado da constante e o
gatilho de reconferência (*«se e só se o outro lado do par mudar»*). Ver
[[feedback_pinning_a_tuning_constant_through_an_engine_rewrite_can_be_the_change]].
