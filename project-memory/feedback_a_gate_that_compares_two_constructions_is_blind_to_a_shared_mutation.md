---
name: a-gate-that-compares-two-constructions-is-blind-to-a-shared-mutation
description: Um gate que afirma «A e B concordam» morre a nenhuma mutação que afecte A e B igual — a lei em si pede um oráculo, não um irmão.
metadata:
  type: feedback
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

**Why:** uma igualdade entre duas coisas produzidas pelo mesmo código defeituoso é uma tautologia com
cara de medição. Ela prova *consistência*, que é uma propriedade mais fraca do que *correcção* — e a
diferença só aparece quando alguém tenta matá-la.

**How to apply:** para a **lei** em si, meça contra um **oráculo** — um facto que é verdade ou falso
sozinho, sem segundo documento. Aqui: *«com o pai a subtrair e ninguém pronunciado, o coração da 2.ª
forma está FORA da peça»*. Comparações entre construções continuam a valer para o que elas de facto
provam (equivalência, byte-identidade, ausência de regressão) — mas nunca ponha a lei nova a depender
só delas. O sinal de alerta é o gate não ter nenhum `assert` sobre um valor **absoluto**.

Vizinhos: [[feedback-a-mutation-proof-needs-a-control-on-its-own-filter]] ·
[[feedback-a-cure-measured-on-a-fixture-that-lacks-the-phenomenon-reads-as-useless]] ·
[[feedback-an-inequality-accepts-a-whole-interval-only-an-oracle-accepts-an-answer]] ·
[[feedback-a-claim-no-mutation-can-kill-is-a-claim-about-nothing]]
