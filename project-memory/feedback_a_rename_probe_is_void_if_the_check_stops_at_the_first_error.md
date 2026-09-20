---
name: feedback_a_rename_probe_is_void_if_the_check_stops_at_the_first_error
description: "A sonda de órfão por renomeação só afirma algo se a corrida chegar ao FIM — um cargo check pára de agendar unidades no primeiro erro, e o relatório lê-se igual a «sem chamadores»"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: e990a2f5-7d16-405a-8ddf-54393edf203d
  modified: 2026-09-19T18:08:26.263Z
---

⛔⛔⛔ **A prova canónica de que um acessório é ÓRFÃO — renomeá-lo e ver a workspace compilar — é
INVÁLIDA se a corrida não chegar ao fim.** Um `cargo check` **pára de agendar unidades novas no
primeiro erro** (acaba só as já lançadas). Logo, se a renomeação partir uma crate a MONTANTE, todas
as crates a jusante ficam **por verificar** — e o relatório lê-se **exactamente igual a «sem
chamadores»**.

**Medido 2026-09-19** (`line/UIUX`, a fronteira dos motores): renomeei **sete** acessórios de uma
vez. A `ph2d-vec-scene` não compilou, e com ela ficaram por verificar os painéis que dela dependem —
que eram precisamente os que pintavam dois deles. Declarei os seis órfãos e **apaguei-os**. O
`cargo check` seguinte devolveu `no method named 'name'` em **cinco painéis** de produto:
`BlendMode::name()` tinha **22 rótulos vivos em três painéis**, e o `SymmetryKind::label()` era
pintado pela fileira de simetria do painel de vector.

**Why:** o modo de falha não é o teste a mentir — é o teste a **não correr**, e a ausência de
vermelho a ser lida como verde. É a mesma família de *«um filtro que casou zero testes imprime `ok`
e lê-se como sobreviveu»*, uma camada acima: ali não corre o TESTE, aqui não compila a CRATE.

**How to apply:**
- **uma renomeação de cada vez**, ou `cargo check --workspace --all-targets --keep-going`;
- **leia os erros TODOS**, não só os que espera — o que prova a orfandade é a corrida chegar ao fim
  sem nenhum, e não a ausência do erro que você procurava;
- ⭐ o inverso é a boa notícia: **quando a corrida completa fica verde com a função apagada, isso É a
  prova** — mais forte que qualquer `grep`, porque apanha o `Tipo::metodo` passado como valor de
  função, que nenhuma régua textual vê;
- ⚠️ ela continua cega a um consumidor atrás de `#[cfg(feature = …)]` desligada — `--all-targets`
  não é `--all-features`.

Irmãs: [[feedback_a_mutation_proof_needs_a_control_on_its_own_filter]] ·
[[feedback_a_bins_run_never_reaches_the_gates_that_live_in_tests]] ·
[[reference_topic_measurement_discipline]]
