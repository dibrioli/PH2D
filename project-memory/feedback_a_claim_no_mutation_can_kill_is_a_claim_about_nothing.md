---
name: feedback_a_claim_no_mutation_can_kill_is_a_claim_about_nothing
description: "Quando uma mutação que remove o mecanismo deixa a suíte VERDE, a cura primeira é encolher a AFIRMAÇÃO até ao que a máquina faz — não inventar um gate para a versão grande que você escreveu no doc-comment"
metadata:
  type: feedback
---

Escrevi no doc-comment do `TimeFans` (ADR-0163) que empurrar um `ScopeKey` por
fatia fazia *"duas fatias que pedem o mesmo instante partilharem a faixa e o
custo"*. Mutação: trocar `push_scope(in_key, node, map)` por `in_key`. **Seis
gates ficaram verdes.**

Tentei então construir o gate que a matava, e o primeiro que escrevi (a fotografia
do `pre` a ser pisada) **também não a matou** — o `advance_tick` recoze as fontes
na faixa raiz depois de tudo.

**Why:** a afirmação era maior do que a máquina. Dentro do laço do leque cada
leitura segue a própria cozedura, então os VALORES saem certos com faixa
partilhada ou não; e duas fatias **adjacentes** no mesmo instante batem no memo de
qualquer forma. O que a faixa própria compra é bem menor e bem preciso: o
instante repetido **fora de ordem** (`t−1`, depois `t−2`, depois `t−1` outra vez).

⚠️ **Perseguir o gate antes de reler a afirmação custa duas voltas.** A segunda
tentativa mediu um cenário real e verdadeiro — e continuava a não ser sobre o
mecanismo.

**How to apply:**
1. Mutação sobrevivente ⇒ **releia a afirmação primeiro**. As duas saídas são
   *"falta um gate"* e *"a afirmação está errada"*, e a segunda é mais comum
   quando quem escreveu o doc e quem escreveu o código são o mesmo turno.
2. Pergunte **o que observaria** se o mecanismo sumisse. Se a resposta for *"nada,
   só custo — e nem sempre"*, a afirmação tem de encolher até esse caso, e o gate
   nasce dele. [[reference_topic_mutation_proofs]]
3. Um gate escrito a perseguir uma mutação e que não a mata **pode ficar** se o
   cenário for real e ninguém mais o cobrir — mas **renomeie-o e diga no
   doc-comment que ele não é o gate daquele mecanismo**, senão a próxima janela
   lê-o como prova do que ele não prova.
   [[feedback_before_declaring_the_design_rejects_an_invariant_grep_for_its_gate]]
