---
name: feedback-a-surviving-mutation-can-mean-the-code-is-wrong-not-the-gate-missing
description: "Mutação que SOBREVIVE tem duas leituras — falta gate, ou o código é inerte/errado; escreva o gate que o código PROMETE e veja-o nascer vermelho antes de assumir a primeira"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T02:44:39.518Z
---

O reflexo ao ver uma mutação sobreviver é *«falta-me um gate»*. **Metade das vezes é o contrário: o
código não faz o que o comentário dele diz.**

**Medido na `line/UIUX`, 2026-08-30 (entrega 23), duas vezes no mesmo dia:**

1. **Código inerte com comentário confiante.** Escrevi uma supressão de clique com um cenário
   explicado ao lado; a mutação que a apagava sobreviveu. O cenário **já estava coberto** por outra
   guarda (`still_hot`). ⚠️ E o que a supressão de facto fazia era um **dano**: matava o empurrão de
   5 px, pondo a troca de aba a depender da firmeza da mão. ⇒ o gate honesto
   (*«um empurrão ainda troca de aba»*) nasceu **VERMELHO**, e a cura foi **apagar o código**.
2. **Consequência no sítio errado.** O limiar do arrasto sobreviveu porque nenhuma asserção de
   *estado final* o via — largar sobre o próprio encaixe é um no-op. O que ele decide é **o que se
   vê**: sem ele, pousar o dedo acende as zonas de largada e apaga-as ao levantar. ⇒ aí sim faltava
   gate, e ele mede a **superfície**, não o resultado.

⚠️ **E há um irmão que a mutação NÃO apanha, medido no mesmo dia:** uma decisão escrita **inline
num hook** (`if c.get() != Some(h)`) com um comentário a prometer o contrário. Ela não é gateável,
logo nunca é confrontada — a cura não é corrigir a condição, é **dar-lhe nome** (`should_save`) para
um gate a poder ler. *Uma decisão dentro de um hook é uma afirmação que ninguém pode contradizer.*

⭐ **O procedimento que separa as duas:** *escreva o gate que o código PROMETE e corra-o antes de o
declarar correcto.* Se nasce vermelho, o código é que está errado. Se nasce verde, a promessa é
verdadeira e a mutação estava a apontar para uma consequência noutra dimensão (o que se vê, o que se
publica, quanto custa).

**Why:** um gate escrito para explicar uma mutação sobrevivente tende a ser escrito *à medida do
código*, e passa por construção — legitimando código que não faz nada.

**How to apply:** ao ver «SOBREVIVEU», pergunte primeiro **que facto observável o código promete**,
e escreva-o como asserção. Só depois de o ver verde procure a dimensão que a mutação tocava.

Relacionadas: [[feedback_a_cure_written_in_one_of_two_lowering_routes_makes_every_gate_lie]] ·
[[feedback_mutate_the_code_not_just_the_test]] ·
[[feedback_i_write_the_right_guard_and_do_not_gate_it]] ·
[[feedback_a_stale_comment_and_dead_code_lie]]
