---
name: a-gate-that-compares-two-constructions-is-blind-to-a-shared-mutation
description: "Um gate que afirma «A e B concordam» morre a nenhuma mutação que afecte A e B igual — a lei em si pede um oráculo, não um irmão."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: a3aecff0-9443-4279-add5-9df37168d691
  modified: 2026-09-13T21:41:15.232Z
---

Um gate escrito como *«esta construção concorda com aquela»* é cego a **toda** mutação que atinja as
duas da mesma maneira. O controlo partilha o defeito do sujeito, e então ele não é um controlo.

**Caso medido (`line/3DModeling`, 2026-08-28, W97).** A lei nova era `fold_verb(parent, child) =
child.unwrap_or(parent)` — *«sem verbo próprio, herda o do pai»*. Sete gates sobre o campo avaliado,
todos verdes. A mutação `child.unwrap_or(Op::Union(Blend::Sharp))` — que **apaga a herança inteira** —
**sobreviveu a todos**.

Porquê: cada gate comparava dois documentos (o N-ário contra o aninhado, o antes contra o depois), e
a maioria das fixturas tinha `Union(Sharp)` como operação do pai. A mutação reescrevia os **dois
lados** para a mesma coisa, e a igualdade continuava a valer.

**Segundo caso medido (`line/UIUX`, 2026-09-14) — a forma MAIS pura, e eu escrevi-a de propósito.**
Ao partir a porta da linha de propriedade em duas (`property_row_columns` passou a **chamar**
`property_label_col_w`), escrevi um gate a afirmar que as duas nunca divergem, em 108 células. A
mutação que apaga o tecto da coluna (`.min(usable − gap − control_min)`) **sobreviveu**: as duas
respostas mudam juntas porque **uma é a outra**. *Quando A delega em B, «A concorda com B» não é uma
asserção — é a definição.* O substituto mede a lei contra um oráculo de fora (*o controlo nunca fica
abaixo do piso nomeado quando cabe*) e duas mutações morrem nele.
⚠️ O sinal que devia ter-me parado: **o gate não tinha um único número absoluto lá dentro**.

**Why:** uma igualdade entre duas coisas produzidas pelo mesmo código defeituoso é uma tautologia com
cara de medição. Ela prova *consistência*, que é uma propriedade mais fraca do que *correcção* — e a
diferença só aparece quando alguém tenta matá-la.

**How to apply:** para a **lei** em si, meça contra um **oráculo** — um facto que é verdade ou falso
sozinho, sem segundo documento. Aqui: *«com o pai a subtrair e ninguém pronunciado, o coração da 2.ª
forma está FORA da peça»*. Comparações entre construções continuam a valer para o que elas de facto
provam (equivalência, byte-identidade, ausência de regressão) — mas nunca ponha a lei nova a depender
só delas. O sinal de alerta é o gate não ter nenhum `assert` sobre um valor **absoluto**.

**Mordeu outra vez (`line/motion-value`, 2026-09-13, `ph2d-contact`).** O gate *«a grelha dá os MESMOS
BITS que todos-os-pares»* era verdadeiro e forte para o que prova (a ordem das somas e o conjunto de
vizinhos), e a mutação do eixo dos centros coincidentes — as duas peças empurradas para o MESMO lado,
nunca se separando — **sobreviveu** a ele, porque os dois caminhos chamam a mesma função por par. A
cura foi a deste ficheiro: um gate de valor absoluto (*«duas peças no mesmo ponto acabam à soma dos
raios, com o meio no sítio»*), e o doc-comment do gate de igualdade passou a dizer o que ele NÃO prova.

Vizinhos: [[feedback-a-mutation-proof-needs-a-control-on-its-own-filter]] ·
[[feedback-a-cure-measured-on-a-fixture-that-lacks-the-phenomenon-reads-as-useless]] ·
[[feedback-an-inequality-accepts-a-whole-interval-only-an-oracle-accepts-an-answer]] ·
[[feedback-a-claim-no-mutation-can-kill-is-a-claim-about-nothing]]

## Segunda ocorrência — 2026-09-15, `line/UIUX`

⚠️ **Ela repetiu-se no gate seguinte da MESMA porta, e a primeira já estava escrita ao lado.** O
`the_lone_field_of_a_row_spans_what_the_pair_spans` provava a ordem do dono de 2026-05-24 (*«a caixa
única de Rotation deve se alinhar à caixa de X à esquerda e à direita»*) comparando `sozinha_dir`
com `par_dir` — os **dois** derivados de `property_fields_layout`. A mutação que desconta um vão a
mais na largura da célula encolhe os dois lados ao mesmo tempo: a igualdade continua a valer e a
prova **SOBREVIVEU**.

⇒ o oráculo tem de ser **outra porta**: a borda direita da coluna do controlo
(`property_row_columns`). Reescrito assim — *«a linha ENCHE a coluna do controlo, nos dois
regimes»* —, a mesma mutação morre (`183,00` contra `186,00` à largura mínima do dock).

⭐ **A regra prática:** quando um gate afirma que **A == B** e os dois saem da função que ele mede,
ele mede a função contra si própria. Procure a grandeza que a função **não** produz.
