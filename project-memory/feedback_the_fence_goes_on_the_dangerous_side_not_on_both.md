---
name: feedback_the_fence_goes_on_the_dangerous_side_not_on_both
description: Um bicondicional defende as duas direcções com o mesmo rigor, mas elas raramente custam o mesmo — quando o modelo do teste é um PROXY da lei e as duas se separam numa célula, aperte o lado caro e liberte o barato
metadata:
  type: feedback
---

Um teste que afirma uma **equivalência** (*«sobrevive exactamente onde X»*) defende as
duas direcções com o mesmo rigor. **Elas raramente custam o mesmo** — e quando o modelo
do teste é um **proxy** da lei real, elas separam-se na fronteira.

**Medido (2026-08-29).** A lei tinha **duas** cláusulas — *«já passei»* (caixas
envolventes) **e** *«a prancha parou de me pegar»* (um cone) — e o teste modelava só a
primeira. Nove de dez células concordavam; numa, a caixa já passou e o cone ainda pega.
⚠️ **A margem sozinha não explicava qual falhava**: outra célula com a *mesma* margem
concordava.

⇒ **A cerca mudou-se para o SINAL.** Aposentar **cedo** torna a plataforma sólida com o
personagem a cair através dela (o defeito caro). Sobreviver **de mais** apenas o deixa
continuar a cair (o barato). Logo: bicondicional **intacto no lado perigoso**, faixa de
cruzamento no lado seguro, **com a tabela das dez células escrita no teste**.

⛔ **Isto NÃO é baixar a barra.** Baixar a barra é alargar as duas pontas até o vermelho
sumir. Aqui a ponta cara **apertou** (virou um `assert!` incondicional) e só a barata
cedeu, com o mecanismo nomeado.

**Why:** um teste que pesa as duas direcções por igual está a defender o defeito barato
com o mesmo rigor com que defende o caro — e é a ponta barata que reprova primeiro
quando o substrato muda.

**How to apply:** quando um bicondicional falhar numa célula, pergunte **(1)** o modelo é
a lei ou um proxy dela? **(2)** as duas direcções custam o mesmo? Se a resposta for
*proxy* e *não*, a cerca é assimétrica. Ver
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]].
