---
name: feedback-a-law-measured-by-a-handbuilt-fixture-has-no-gesture
description: Um gate que CONSTRÓI à mão a entrada da lei nunca pergunta se o gesto consegue produzi-la — a F30 shipou uma lei que nenhum clique alcançava
metadata:
  type: feedback
---

Um gate que constrói **à mão** a entrada de uma lei (`Correccao { centro: [20.0, 0.0], … }`)
mede a lei e **nada diz sobre o GESTO** que deveria produzi-la. Em 2026-09-19 a F30 fez a arte
seguir o peso entre dois nós, o gate leu `0,000000 → 0,836850`, e o pincel **recusava** aquele
sítio: ele ancorava a mancha no NÓ mais perto (`3,041` de distância) contra um raio de `0,40`.

**Why:** é o terceiro elo do `CLAUDE.md` §5.0 numa forma nova — o censo prova que a PORTA faz
efeito, a costura prova que o clique chega ao BARRAMENTO, e a fixtura escrita à mão **salta
exactamente o troço que falta**. E os gates vizinhos não o apanham porque eles apontam sempre
para um vértice, que é o caso em que as duas leis concordam: *uma fixtura que aponta sempre
para um nó não testa o que acontece entre eles.*

**How to apply:** ao fechar uma wave cuja lei tem entrada estruturada, escreva **um** gate que
produz essa entrada pela **porta do gesto** e mede o efeito no fim. Se a porta do gesto não a
consegue produzir, a wave não está feita — ela tem lei e não tem gesto. Relacionado:
[[feedback-every-ruler-measured-the-goal-and-none-measured-the-cost]] ·
[[feedback-the-inner-channel-fixture-is-below-the-break]] ·
[[reference-topic-gate-discipline]]
