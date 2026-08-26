---
name: i-write-the-right-guard-and-do-not-gate-it
description: Três vezes na mesma linha escrevi a guarda certa e nenhum gate a cobria — a mutação é o único instrumento que acha isso
metadata:
  type: feedback
---

Vector / máquina de estados do Morph, 2026-08-25/26. **Três vezes na mesma linha**, em waves
diferentes, eu escrevi uma guarda correta e **nenhum gate a cobria** — apagá-la deixava a
suíte inteira verde:

1. **W9** — `vec_pen.select_many(&[p.path])` (o conjunto novo fica selecionado). Sem ele, a
   seção volta a oferecer *"Make Morph States"* sobre as formas que acabaram de virar filhas.
2. **W11c** — `self.morph_steps = f.tr.morph_steps(t)` no `advance`. Sem ele, a
   compatibilidade **inteira** com o sistema States ficava morta e nada dizia.
3. **W11c** — a guarda do `VecMorphMachine` no `install`. Sem ela, uma pose com forma
   prendia um morph autorado à mão num par degenerado, matando a curva da timeline.

**Why:** os três são a mesma forma. Eu escrevo a guarda *porque penso no dano*, e depois
escrevo o gate *da feature* — que passa com ou sem a guarda. O código fica certo e a
afirmação fica **sem sustentação**: a próxima pessoa a refatorar apaga a linha e nada
protesta. *Uma afirmação que mutação nenhuma mata é uma afirmação sobre nada.*

⚠️ E há um padrão no que escapa: são sempre linhas cujo dano é **noutro subsistema** (a
seleção, o motor a jusante, a timeline) — precisamente as que o gate da feature não olha.

**How to apply:**
- Depois de escrever uma guarda (`if ... { return }`, um `&& is_some()`, uma atribuição
  «de arrumação»), pergunte **imediatamente**: *que gate fica vermelho se eu a apagar?* Se
  a resposta não for um nome, o gate ainda não existe.
- ⛔ Não confie no gate da feature: ele mede o caminho feliz, e a guarda existe para o
  caminho que ele não percorre.
- Rode a prova de mutação **sobre as guardas**, não só sobre a lei principal — e trate
  «não alcançável por este gate» como **buraco**, nunca como isenção.
- O sinal de que é esta classe: a guarda protege algo que vive noutro ficheiro/subsistema.

Ver [[feedback-a-claim-no-mutation-can-kill-is-a-claim-about-nothing]] ·
[[feedback-counting-the-work-done-is-not-counting-the-work-delivered]] ·
[[reference-topic-mutation-proofs]]
