---
name: feedback-a-probe-that-sums-two-signals-cannot-say-which-failed
description: "Sonda agregada (tinta da janela inteira, soma, hash) sobrevive à mutação quando o estado que ela mede tem MAIS DE UM emissor — mude a lei para uma função pura com gate próprio"
metadata:
  type: feedback
---

Medido 2026-08-24 (`line/Vector`, janela do Input Map). Gate novo:
*"armar a escuta pinta um aviso"* — a sonda era `scene.encoding().draw_data` da **janela inteira**,
comparada armada vs. calma. Verde. E a mutação **sobreviveu**: pôr a faixa do título a dizer sempre
`Input Map` deixava o gate verde, porque armar muda a tinta por **duas** razões independentes — a
faixa passa a nomear a acção, **e** o botão `+` daquela linha troca de estilo. A sonda somava as
duas e não sabia distingui-las.

**Why:** uma sonda agregada responde *"alguma coisa mudou?"*. Se o facto que ela quer provar tem
mais de um emissor, ela fica verde enquanto **qualquer** um deles funcionar — inclusive o que não
interessa. É o mesmo defeito de forma de [[feedback_a_green_gate_may_be_green_by_accident]], mas a
causa aqui não é o fixture: é a **especificidade** do instrumento. É a quarta face das três de
[[feedback_ask_what_number_the_opposite_answer_would_print]] (alcance · extensão · invariância):
**ESPECIFICIDADE — a régua tem de responder por UM emissor.**

**How to apply:** antes de escrever um gate sobre um agregado (tinta de uma cena, soma, hash,
contagem total), enumere **quantas coisas** mexem naquele agregado quando o estado sob teste muda.
Se forem duas ou mais, a lei não cabe ali: extraia-a para uma **função pura** — a frase, o número, a
lista — e gateie a função. O pintor fica com **um** sítio a chamá-la, e é o `dead_code`/clippy que
guarda esse elo. Precedente na mesma crate: `binding_label` já existia por este motivo, e a nota
dela dizia-o (*"o pintor e o gate precisam da MESMA frase"*) — eu não a apliquei à frase seguinte.
Ver também [[feedback_an_identity_gate_cannot_see_a_defect_in_the_shared_body]] e
[[reference_topic_mutation_proofs]] (foi a prova de mutação que o apanhou, não a leitura do gate).
