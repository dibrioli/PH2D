---
name: feedback-a-rule-that-never-observes-cannot-fire
description: HR-13 somava budgets DECLARADOS no boot e nunca olhava uma alocação — o editor chegou a 4351 MB sem a regra piscar. Toda regra precisa OBSERVAR, não só declarar
metadata:
  type: feedback
---

O HR-13 manda cada subsistema declarar um `MemoryBudget`, e `ph2d_core::budget::check_budget`
**soma os structs declarados no boot** e compara com o teto da plataforma. Ele nunca observa **um
único byte de fato alocado** — e não havia sequer sítio de declaração para o Audio Editor.

Resultado medido: o editor chegou a **4351 MB de pico** (64 edições num clipe de 3 min) contra um
teto de **3500 MB para o app INTEIRO**, e a regra não piscou. O número declarado estava certo.
Ninguém o estava conferindo.

> **Um budget que só é declarado não pode ser violado — só excedido em silêncio.**

**Why:** uma regra sem observação é uma opinião com sintaxe de teste. Ela dá a *sensação* de
cobertura (existe um `check_`, existe um unit test, o CI está verde) enquanto o defeito cresce
duas ordens de grandeza embaixo dela. Pior que não ter regra: você **para de procurar**.

**How to apply:** antes de confiar num gate, pergunte **o que ele efetivamente lê**. "Compila",
"o número declarado bate com a tabela" e "o enum tem N variantes" são propriedades ESTRUTURAIS —
não dizem nada sobre o comportamento. Se a regra fala de um recurso em runtime (memória, tempo,
alocação, banda), ela precisa de um gate que **meça o real** (dhat, `Instant`, contador), e a
barra tem de estar na unidade do recurso. Emenda ao HR-13 ([ADR-0117] D4): *quem declara budget
possui também um gate executável que MEDE.*

Generaliza para além de memória — é a mesma classe de [[feedback_painter_inefficiency_4_causes]]
("audit" = compilar) e de [[feedback_no_industrial_claims_without_verification]]. E o corolário
operacional: [[feedback_mutate_the_code_not_just_the_test]] — um gate que fica verde quando você
quebra o código não estava olhando.
