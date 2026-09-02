---
name: a-dependency-fence-is-not-a-capability-fence
description: "Uma cerca que lê o Cargo.toml defende «não depende de X», nunca «não CONSEGUE fazer X» — a std não aparece em manifesto nenhum"
metadata:
  type: feedback
---

**Medido por auditoria adversarial, 2026-08-30 (fonte de dados do Motion).**

A lei é boa e é do repo: *o trabalho pesado — descodificar som, ler um disco — nunca entra no
cook*. E a defesa é a melhor que há: **estrutural**. Sem a crate no `Cargo.toml`, o nó não
consegue ter opinião. Escrevi o gate que lê os manifestos e prova-o.

⛔⛔ **Três mutações passaram por ele:**

1. `[dependencies.ph2d-table]` — a forma **sub-tabela**. O parser só entrava na secção quando
   a linha era exactamente `[dependencies]`.
2. `[target.'cfg(unix)'.dependencies]` — idem.
3. ⚠️ **`std::fs::read_to_string` dentro do `eval`, sem tocar num `Cargo.toml`.** A `std` não
   aparece em manifesto nenhum, então a lei *«nenhum nó abre um ficheiro»* **não tinha
   instrumento**: o gate só sabia dizer *«não depende do leitor»*, que é mais fraco do que o
   doc afirmava.

**Why:** um manifesto lista o que se IMPORTA, e a capacidade perigosa pode já estar na
linguagem. E um parser de conveniência defende a forma que o autor calhou de escrever, não a
lei — o Cargo aceita quatro grafias da mesma dependência.

**How to apply:** uma cerca de capacidade precisa de DUAS metades — as dependências **e** uma
varredura dos `src/**.rs` pelos símbolos proibidos (`std::fs`, `std::net`, `std::process`,
`include_str!`, `include_bytes!`). E ao escrever o lado do manifesto, enumere as grafias:
`[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, `[target.'…'.dependencies]`, e
todas as suas formas `[….NOME]`. Relacionado:
[[feedback_a_textual_gate_must_strip_comments_or_documenting_the_cure_fails_it]] ·
[[reference_topic_gate_discipline]].
