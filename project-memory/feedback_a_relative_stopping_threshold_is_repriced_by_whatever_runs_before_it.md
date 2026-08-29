---
name: feedback_a_relative_stopping_threshold_is_repriced_by_whatever_runs_before_it
description: Limiar de paragem escolhido numa varredura SEM o passo anterior deu 23 rondas em vez de 93 — um limiar só se calibra na configuração em que corre
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-28T22:33:46.065Z
---

Medido 2026-08-28 (acabamento da cadeia de quads): a fracção `settle = 1e-2` foi escolhida de
uma tabela varrida com a relaxação **sozinha** (93 rondas, enviesamento `7,8° → 4,5°`). Na
porta do produto ela corre **depois** de 6 rondas de Laplaciano, e a mesma fracção deu **23
rondas** e `7,8° → 6,8°` — menos de metade do ganho.

**Why:** o limiar é sobre o **movimento por ronda**, e o passo anterior pré-condiciona a
malha: ela chega à relaxação já mais perto do ponto fixo, o movimento começa menor, e o mesmo
limiar relativo é atingido muito mais cedo. *O número não mudou; o que ele mede mudou.*

**How to apply:** um limiar relativo (movimento, resíduo, delta) só se calibra na
**composição inteira** em que vai correr — e a varredura tem de passar pela **porta do
produto**, não por uma composição escrita no instrumento. Se a porta ainda não existe, faça-a
primeiro e varra através dela; se a varredura precisa de um parâmetro que a porta fixa, abra
uma forma aberta (`fn_with(...)`) em vez de uma variável de ambiente — a env some da
assinatura e volta a permitir varrer o programa errado. Relacionado:
[[feedback_a_measured_refusal_answers_one_question_recheck_it_when_yours_is_another]] ·
[[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]] ·
[[feedback_i_write_the_right_guard_and_do_not_gate_it]]
