---
name: feedback_a_rename_probe_is_void_if_the_check_stops_at_the_first_error
description: Renomear um símbolo para ver quem o chama só afirma alguma coisa se a corrida chegar ao FIM — o `cargo check` pára de agendar unidades no 1.º erro e as crates a jusante ficam por verificar, com o relatório a ler-se «sem chamadores»
metadata:
  type: feedback
---

A sonda de órfão mais barata que existe é **renomear o símbolo e ver quem grita**: se nada partir,
ninguém o chama e ele pode ser apagado.

⛔⛔ **Ela é INVÁLIDA se a corrida não chegar ao fim.** O `cargo check` **pára de agendar unidades
de compilação no primeiro erro** — as crates a jusante daquela que partiu nunca chegam a ser
verificadas, e o relatório que se lê é *«só estes N sítios»* quando a verdade é *«estes N, mais
tudo o que vem depois e não foi medido»*.

⚠️ **O modo de falha é o caro, porque a leitura errada é a CONFORTÁVEL:** «poucos chamadores»
convida a apagar.

**Caso medido (`line/UIUX`, 2026-09-19, a fronteira dos motores do HR-15).** A sonda custou **seis
apagões errados**, dois deles sobre símbolos com **22 rótulos vivos em três painéis**.

**Why:** o `check` é um *scheduler*, não um verificador exaustivo; o `--keep-going` existe
precisamente porque o comportamento de omissão é desistir cedo. Uma sonda que conta erros herda a
política de agendamento da ferramenta, e essa política não é sobre a sua pergunta.

**How to apply:** toda sonda por renomeação corre com `--keep-going` (ou
`cargo check --workspace --all-targets --keep-going`) **e** com um **controlo positivo**: um
símbolo que sabidamente tem N chamadores tem de devolver os N. Se a corrida abortar por qualquer
razão, o resultado é **VOID**, nunca «zero chamadores» — a mesma lei que
[[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]] cobra do filtro de uma mutação e
que [[feedback_a_tail_is_a_window_not_a_verdict]] cobra de uma leitura truncada.

⭐ **Os símbolos, nomeados** (a mesma corrida): `BlendMode::name()` tinha **22 rótulos vivos em
três painéis** e o `SymmetryKind::label()` era pintado pela fileira de simetria do painel de vector
— os dois declarados órfãos e apagados, e o `cargo check` seguinte devolveu `no method named 'name'`
em **cinco painéis de produto**. ⚠️ E a sonda continua cega a um consumidor atrás de
`#[cfg(feature = …)]` desligada: `--all-targets` **não é** `--all-features`.

Vizinhos: [[reference_topic_measurement_discipline]] · [[reference_topic_gate_discipline]]
