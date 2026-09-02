---
name: feedback-a-gradient-gate-says-may-punch-only-the-image-says-punches
description: "Gate de gradiente diz «PODE furar»; só a imagem diz «fura» — quando discordam, manda a imagem"
metadata:
  node_type: memory
  type: feedback
---

Um gate de `‖∇f‖` (ou de qualquer **cota de segurança**) diz *«este campo PODE fazer a marcha
atravessar»*. Ele **não** diz que atravessa. Curar o número sem medir o efeito troca uma feature
visível por um número que ninguém vê.

Caso medido (PH2D, `line/3DModeling`, 2026-08-30): a dobra sozinha media `‖∇f‖ = 1,72` dentro da
caixa de recorte. Apertei a parede da curvatura para o curar, e num **bloco** as voltas `0,3`, `0,6`
e `1,0` passaram a dar **a mesma peça** — ela deixou de dobrar. Report do Enio: *«VC danificou o
Bend que funcionava antes das últimas mudanças»*.

⭐ E a régua que faltava deu-lhe razão: com a lei que ele tinha, a imagem concorda com uma **marcha
honesta** (f64, passo minúsculo, sem JIT) em **`0` de `1 678`–`4 274` pixels**, pior desvio
`0,0°`–`5,1°`. ⇒ *o `1,72` é real e **não tem consumidor**: onde os raios de facto passam, o campo
continua a ser um minorante.*

**How to apply:**
- Antes de curar um número de segurança, **meça o efeito dele na imagem**. Se a imagem estiver
  limpa, o número é uma dívida a **declarar**, não a pagar
  ([[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]]).
- ⚠️ Quando o gate de cota e a imagem discordam, **manda a imagem** — e a dívida fica escrita com os
  dois números lado a lado.
- ⭐ O oráculo tem de ser **lento e burro** e não partilhar a lei do produto
  ([[feedback_an_oracle_that_shares_the_law_of_what_it_judges_is_a_mirror]]).
- ⚠️ E uma cura que o dono recusa por report é uma **recusa medida**: escreva-a onde a próxima
  pessoa vai procurar, com o mecanismo, para ela não a reconstruir
  ([[feedback_do_not_ask_the_owner_to_judge_a_trade_already_measured_as_destroyed]]).
