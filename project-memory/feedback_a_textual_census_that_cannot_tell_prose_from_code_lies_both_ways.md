---
name: feedback_a_textual_census_that_cannot_tell_prose_from_code_lies_both_ways
description: Um censo de fonte que varre o ficheiro inteiro acusa o COMENTÁRIO e absolve o CÓDIGO — os dois erros de uma vez, e só a mutação os separa.
metadata:
  type: feedback
---

Escrevi dois censos irmãos sobre `shells/desktop/src/render_loop/mod.rs` (2026-09-02, cura do fundo
do cartão de asset): *"ninguém escreve o fundo do canvas à mão"* (proíbe o literal
`0.047, 0.047, 0.055`) e *"o `clear` sai da porta"* (exige `canvas_clear::canvas_clear_rgb`). Ao
lado da cura ficou uma nota histórica que **cita as duas coisas** — o valor legado, porque é ela que
carrega o mecanismo da regressão da M14.5, e o nome da porta, porque é para lá que ela aponta.

Resultado, na primeira corrida:

- o censo do literal ficou **VERMELHO sobre a prosa** — sobre a única linha do ficheiro que tinha de
  a conter;
- o censo da porta ficou **VERDE com o literal reposto** — a prova de mutação apanhou-o: a menção
  no comentário satisfazia a busca.

**Why:** os dois erros têm a mesma raiz e leem-se como problemas diferentes. Um falso positivo
convida a *apagar a nota* para calar o gate — trocar a coisa certa pela coisa medível, e a nota era
a única coisa que documentava uma cerca medida. Um falso negativo faz o censo **testemunhar a favor**
de um defeito, e ele nunca aparece numa corrida verde: sem mutação, um censo que casa contra prosa
parece um censo a funcionar.

**How to apply:** todo censo que varre fonte lê **só linhas de código** — filtre o que começa por
`//` antes de procurar (uma função `code_of(path)` partilhada, não a mesma condição escrita duas
vezes). E prove-o pelos DOIS lados: reponha o defeito e confirme que o censo sangra, além de confirmar
que ele passa sobre a árvore sã. ⚠️ A cura NÃO é reescrever o comentário para não casar: uma nota
que não pode nomear o valor que defende deixou de defender coisa nenhuma.

Irmão de [[feedback_stale_comment_and_dead_code_lie]] e de
[[reference_topic_gate_discipline]]; a mutação que o revelou é do género de
[[reference_topic_mutation_proofs]].
