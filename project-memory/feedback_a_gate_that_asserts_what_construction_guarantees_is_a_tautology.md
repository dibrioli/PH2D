---
name: feedback_a_gate_that_asserts_what_construction_guarantees_is_a_tautology
description: Um gate que afirma o que a construção já garante nunca fica vermelho pela razão que diz medir — e rodar a fixtura não o salva
metadata:
  type: feedback
---

**Um gate que afirma o que a CONSTRUÇÃO já garante não pode ficar vermelho pela razão
que ele diz medir.**

⛔ **Caso medido (2026-08-22).** A lei nova devolvia `wₖ = h·iᵏ` — o quadrado mais
próximo de quatro pontos. O gate dizia *"o losango vira quadrado"* e verificava que
os quatro cantos saíam a 90° e equidistantes do centro. ⚠️ **Mas `h·iᵏ` é um quadrado
para QUALQUER `h`, incluindo um `h` errado.** Com um sinal trocado na fórmula, **3
dos 4 gates do ficheiro continuavam verdes**.

⛔ **E rodar a fixtura NÃO os salvou** — porque o defeito não estava na fixtura,
estava na **afirmação**. Foi a segunda tentativa e falhou pela mesma razão.

**Why:** a prova de mutação diz *que* um gate é fraco; ela não diz *porquê*. Quando
uma mutação sobrevive, há duas causas possíveis e elas pedem curas opostas:
- a **fixtura** não contém o fenómeno ⇒ mude a fixtura
  ([[reference_topic_fixture_discipline]]);
- a **asserção** é implicada pela construção ⇒ mude a asserção.

*Trocar a fixtura quando a doença é a asserção gasta uma iteração e devolve o mesmo
verde.*

**How to apply:**
1. ⭐ Pergunte: **existe alguma implementação errada desta função que ainda assim
   passe neste assert?** Se a resposta for «sim, qualquer uma que devolva a forma
   certa no sítio errado», a asserção é tautológica.
2. ⭐⭐ **A cura é um ORÁCULO INDEPENDENTE, não outra fixtura.** Aqui:
   `the_closed_form_finds_the_nearest_square` compara o resíduo da forma fechada com
   uma **busca por força bruta** sobre 36 000 ângulos (com o raio óptimo por
   projecção). ⚠️ *A varredura não sabe nada da álgebra que está a julgar* — é isso
   que a torna testemunha. Ela mata a mutação; a outra não.
3. ⚠️ **O gate tautológico pode FICAR**, rebaixado a *necessário e não suficiente*,
   se descrever o produto em palavras do utilizador — mas com o link para quem
   prova a coisa a sério, senão ele volta a ler-se como prova.
4. Vale para grandezas, não só para leis: uma sonda que media **uma** família de
   linhas de grade contra um campo de cruzes dava verde a um quad esmagado, porque
   basta uma família seguir o campo. Ver
   [[feedback_if_relaxation_cannot_move_the_median_the_defect_is_in_the_connectivity]].
