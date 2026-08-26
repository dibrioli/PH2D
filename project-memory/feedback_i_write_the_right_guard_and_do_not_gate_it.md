---
name: i-write-the-right-guard-and-do-not-gate-it
description: CINCO vezes na mesma linha escrevi a guarda certa e nenhum gate a cobria — a mutação é o único instrumento que acha isso, e o padrão é o dano viver UM PASSO à frente
metadata:
  type: feedback
---

Vector / máquina de estados do Morph, 2026-08-25/26. **Cinco vezes na mesma linha**, em waves
diferentes, eu escrevi uma guarda correta e **nenhum gate a cobria** — apagá-la deixava a
suíte inteira verde:

1. **W9** — `vec_pen.select_many(&[p.path])` (o conjunto novo fica selecionado). Sem ele, a
   seção volta a oferecer *"Make Morph States"* sobre as formas que acabaram de virar filhas.
2. **W11c** — `self.morph_steps = f.tr.morph_steps(t)` no `advance`. Sem ele, a
   compatibilidade **inteira** com o sistema States ficava morta e nada dizia.
3. **W11c** — a guarda do `VecMorphMachine` no `install`. Sem ela, uma pose com forma
   prendia um morph autorado à mão num par degenerado, matando a curva da timeline.
4. **W11g** — o `machines.remove(&bits)` na reconciliação. Sem ele o resquício **voltava no
   quadro seguinte** — e só dentro do modo de pré-visualização, que é onde o artista acabou
   de estar. ⚠️ Nenhum gate corria o `tick` **depois** da varredura.
5. **W11g, o inverso** — uma afirmação que **nenhuma** mutação podia matar: eu documentei que
   o destino tem precedência sobre a origem, e a guarda anterior já garante que no máximo um
   dos dois passa. *Trocar a ordem não mudava uma única resposta.* A cura foi **apagar a
   afirmação**, não inventar um gate para ela.

**Why:** os três são a mesma forma. Eu escrevo a guarda *porque penso no dano*, e depois
escrevo o gate *da feature* — que passa com ou sem a guarda. O código fica certo e a
afirmação fica **sem sustentação**: a próxima pessoa a refatorar apaga a linha e nada
protesta. *Uma afirmação que mutação nenhuma mata é uma afirmação sobre nada.*

⚠️ E há um padrão no que escapa: o dano vive **um passo à frente** do que o gate da feature
olha — noutro subsistema (a seleção, o motor a jusante, a timeline) ou **no quadro seguinte**
(a máquina viva que reescreve o que a varredura acabou de arrumar). O gate mede o estado
**logo depois** da chamada; a guarda existe para o que vem **a seguir**.

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
