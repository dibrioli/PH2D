---
name: feedback-a-threshold-smaller-than-the-sampling-step-cannot-see-zero
description: "Uma régua cujo limiar é menor que o próprio passo de amostragem não distingue «em zero» de «perto de zero» — e o erro que ela fabrica lê-se como uma propriedade do sujeito"
metadata:
  type: feedback
---

⛔⛔ **Uma régua cujo LIMIAR é menor que o próprio PASSO de amostragem não distingue «em zero» de
«perto de zero» — e o erro que ela fabrica lê-se como uma propriedade do sujeito.**

Medido em 2026-09-23 (`ph2d-field-eval`, a sonda do vaso por fórmula). Ela varre `u` de `0` a
`u_max` em `4 096` passos (`8,4e-5` cada) e pergunta se o ponto está dentro, para achar onde a
parede interna do vaso começa. O teste de «começa no eixo» era `u > 1e-6`. ⚠️ **O ponto `u = 0` está
EM CIMA da costura do eixo e lê `sd = 0`, que não é `< 0`** ⇒ o primeiro «dentro» era o passo
seguinte, `8,4e-5` — que passa o limiar de `1e-6` e se lê como *«a parede começa aqui»*.

⇒ a função `dentro(v)` ganhava um **DEGRAU** de `~0` para `0,153` à altura da base, e o ajuste
polinomial lia **`0,09` de erro em TODOS os graus** (`4` a `24`), *sem melhorar* — que é a
assinatura exacta de uma descontinuidade. Com o limiar posto em `4 × passo`, os mesmos graus leem
`0,048` → **`0,0011`**, uma convergência limpa.

⛔⛔ **E eu quase escrevi o número errado como um facto sobre o produto:** *«o desenho tem uma feição
que polinómio nenhum consegue dizer»*. Isso teria fechado uma rota que **funciona** (`4×` mais
rápida), e a 1.ª hipótese sobre a CAUSA — o fundo horizontal da cavidade — foi construída, medida e
**REFUTADA** antes da certa.

**Why:** um limiar responde *«isto é zero?»* e uma varredura só sabe responder à resolução do passo
dela. Quando os dois discordam por duas ordens de grandeza, o limiar deixa de medir o sujeito e
passa a medir a grelha — e o sintoma (um erro que não converge) tem exactamente a forma de um
achado sobre o sujeito.

**How to apply:**
1. **Todo limiar de «é zero» sai do passo da varredura**, derivado, nunca escrito à mão. Se o passo
   é `h`, o limiar é um múltiplo pequeno de `h`.
2. ⭐ **Quando um erro não converge com o refinamento, peça à régua ONDE ele está** antes de o
   explicar. A coluna `onde` resolveu isto numa corrida, depois de duas hipóteses e duas curas
   erradas — *um extremo sem endereço convida a uma explicação, e a explicação plausível chega antes
   da certa*.
3. ⚠️ **Cuidado com a fronteira que lê `= 0`**: um `<` contra uma função com sinal trata a
   superfície como FORA, e num ponto exactamente sobre ela isso desloca a resposta um passo.

Ver [[reference_topic_measurement_discipline]] ·
[[feedback_a_grid_never_lands_on_a_measure_zero_set_so_it_reports_it_clean]].
