---
name: feedback_a_key_census_that_sees_only_literals_prescribes_deleting_live_labels
description: A metade das ÓRFÃS de um censo de chaves é CEGA a uma chave montada em runtime — e a cura que ela prescreve é apagar rótulos VIVOS.
metadata:
  type: feedback
---

Medido em 2026-09-17 (`line/UIUX`, 8.ª fatia do HR-15). O censo de chaves tem duas metades, e a das
**órfãs** diz: *«esta chave está na tabela e ninguém a usa — apague-a»*. Sobre o painel Wet Paint ela
acusou **19** chaves `panel.wet_tuning.knob.*`. Nenhuma era órfã: o painel monta-as em runtime —

```rust
label: format!("panel.wet_tuning.knob.{}", d.key)
```

⇒ seguir a cura prescrita **apagava os rótulos de dezanove knobs do painel**, e o gate ficava verde.

**Why:** o censo varre **LITERAIS** (`"prefixo…"` no fonte) e uma chave assemblada não é um literal.
⚠️ A metade *usada-e-não-declarada* erra para o lado seguro (acusa a mais); a metade das **órfãs**
erra para o lado **destrutivo** — e as duas leem-se com a mesma confiança no mesmo relatório.
⛔ E o inverso também existe: um **teste** que inventa uma chave para provar um negativo
(`assert!(slot_of("…add.nao_existe").is_none())`) é lido como um USO real, porque a régua vê a forma
e não o contexto.

**How to apply:** antes de apagar uma órfã, **procure quem monta o prefixo**:
`grep -rn 'format!("<prefixo>' --include='*.rs'`. Se existir, a isenção é nomeada **e o gate lê o
fonte** que a monta — sem isso, apagar o `format!` deixaria as chaves genuinamente mortas isentas
para sempre ([[feedback_a_ratchet_without_an_obsolescence_census_becomes_a_licence]]). ⭐ **E os dois
pisos do censo passam a ser DIFERENTES** (`declaradas ≥ 40`, `usadas ≥ 8`): um piso simétrico reprova
sobre produto correcto, e *a diferença entre os dois lados É a família montada*. ⚠️ Irmã de forma: um
prefixo pode acabar em `_` e não só em `.` — a cura de 2026-09-13 curou **uma** das duas formas.
Irmãs: [[reference_topic_gate_discipline]] · [[reference_topic_measurement_discipline]]
