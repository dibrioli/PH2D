---
name: feedback_a_streak_from_spaced_dabs_moves_the_whole_tip_and_composes_like_the_medium
description: Fazer dabs espaçados desenharem o traço denso (o Continuous): desloque a PONTA INTEIRA por sub-dab e componha como o MEIO compõe — nem o máximo nem a média do padrão sob a queda do centro
metadata:
  type: feedback
---

Para um dab com passo `D` desenhar o que `n` dabs a passo fino desenhariam, as leis «do padrão» falham as duas:
o MÁXIMO do padrão ao longo do passo traceja (a sobreposição pinta duas vezes) e a MÉDIA, sob a queda do CENTRO
do dab, faz RETÂNGULOS (o peso salta a meio do passo), dilui (tinta 0,26×) e esvaziou o Impasto (os pelos caem
sob o limiar do filme). Medido no Painter, prompts 32 → 34 (13/10), régua normalizada pela tinta contra o traço
a 0,025: hoje 0,08–0,21 → **≤ 0,017** com a lei certa, nos cinco meios, a 0,18–0,57× o custo do traço denso.

**A lei:** avalie a ponta INTEIRA (queda + pegada + padrão) em cada sub-dab de volta ao centro ANTERIOR (para trás:
o 1.º dab é o 1.º do denso) e componha pela lei DO MEIO: passagens da mesma cor por cima = `1 − Π(1 − c·x_k)`;
um meio de máximo = o máximo por sub-dab; um depósito com LIMIAR por dab (o portão de papel do Wet Paint) =
entregue os sub-dabs ao motor — compor antes do limiar dilui sob ele ou alarga o traço.

**Why:** a régua que aprovou a média (o CV ao longo, 0,055) não via a FORMA; o dono viu os retângulos. E «uma
lei do padrão» trata como textura o que é uma lei da PONTA.

**How to apply:** meça contra o traço denso NORMALIZADO pela tinta (mais a costura por fase do passo), nunca só o
CV ao longo; o sub-passo é uma fracção do diâmetro (como o Spacing), não 1 px. Doc 31 §5 do Painter
(`ph2d-painter-brush/src/capsula.rs`). Ver [[feedback_forcing_a_property_after_the_optimum_is_what_ripples]].
