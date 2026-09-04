---
name: a-per-glyph-outline-effect-is-paid-every-frame
description: "Um efeito que reescreve a GEOMETRIA de cada glifo custa por quadro e nenhum cache o salva — o font_embolden do Vello ficou caro e invisível"
metadata:
  type: feedback
---

Antes de ligar qualquer efeito de texto novo, pergunte **o que ele faz à geometria**, não o que ele
parece fazer à tinta.

Medido 2026-09-03. O `Scene::font_embolden` do Vello 0.10 parecia uma bandeira de rasterizador — a
resposta ao pedido do dono de *«colocar o que o vello consegue fazer como mais uma opção de
aparência das fonts»*. Não é: com `amount != 0` o Vello **desvia o glifo para outro caminho** —
desenha a outline para um buffer, corre `kurbo::expand_path` (offset de contorno, com junção e
limite de miter) e **re-codifica** o resultado. O contorno deslocado tem muito mais segmentos, e
esses segmentos são **re-percorridos a cada quadro, para cada glifo no ecrã**.

Veredito do dono, à primeira: ***«o cursor ficou lento e não se pode abrir nada mais. não vi
diferença na font»***.

**Why:** ⚠️ **o cache de glifos NÃO salva disto** — ele guarda a *codificação* do glifo, não o
trabalho de a percorrer por quadro. Num editor tudo se repinta, e o custo escala com
*glifos × segmentos*: milhares de glifos por quadro.

⭐ E *caro E invisível* é o par que **fecha** a questão: só invisível, afinava-se o número; só caro,
pesava-se o ganho. Sem ganho e com custo, não há o que pesar — recusa, não iteração.

**How to apply:**
- Ao avaliar API nova de desenho, classifique-a primeiro: **muda a TINTA** (barato, por pixel) ou
  **reescreve a GEOMETRIA** (caro, por quadro, multiplicado pela contagem de elementos)?
- ⛔ Um deslocamento de contorno **anisotrópico com uma componente a ZERO** (`Diagonal2::new(a, 0)`)
  é mal condicionado: a normal do eixo nulo recebe deslocamento `0` e a curva degenera ⇒ **muitos
  segmentos e forma quase igual**, que é exactamente o par observado.
- ⚠️ **Verifique a PERSISTÊNCIA antes de responder ao report:** um modo que congela o app e que fica
  gravado nas preferências prende o dono no estado partido. Aqui não era persistido, e reiniciar
  limpava — foi a primeira coisa que se confirmou.
- ⛔ Uma entrada de menu que congela o app é **pior que uma ausente**: retire-a com o wiring todo,
  não a deixe desligada.

Relacionado: [[feedback_checking_a_fence_can_reveal_that_the_feature_it_guards_does_not_exist]] ·
[[feedback_a_literal_corpus_count_in_a_gate_makes_every_new_feature_edit_someone_elses_test]] ·
[[project_m5_perf_validated]]
