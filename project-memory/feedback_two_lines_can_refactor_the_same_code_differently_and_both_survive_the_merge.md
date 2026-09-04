---
name: feedback_two_lines_can_refactor_the_same_code_differently_and_both_survive_the_merge
description: "Duas linhas que extraem a MESMA função de maneiras diferentes fundem limpo e deixam a cópia morta — só um aviso de dead_code a denuncia, gate nenhum"
metadata:
  type: feedback
---

Integração de 2026-09-04 (6 linhas). Aconteceu **quatro vezes na mesma jornada**, sempre com a
mesma forma: duas linhas olharam para o mesmo bloco duplicado, tiveram a mesma boa ideia, e
cortaram-no em sítios **diferentes**.

| o bloco | linha A | linha B |
|---|---|---|
| a miniatura de um assado | cortou-a para o irmão `motion_object_thumb.rs` (tecto de LOC) | desceu a LEI para `crate::thumbnail::reduce` (3.º consumidor) |
| as 7 linhas do Transform no Inspector | extraiu `mirror_live_values_without_stomping_the_edit` | extraiu `write_transform_rows` (+ a unidade de ângulo) |
| a catraca de LOC do `paint_inspector` | baixou-a a `273` (censo de obsolescência) | baixou-a a `268` (coluna ancorada) |
| a lista de nós que estouram o dock | ACRESCENTOU um (`source.lsystem`) | RETIROU dois (o dock cresceu) |

**Why:** cada corte toca linhas diferentes do ficheiro ⇒ o git funde os dois **sem conflito**. O
resultado tem as DUAS versões: uma viva e uma **morta**. No caso da miniatura só um
`warning: function is never used` a denunciou — ⛔ **gate nenhum deste repo pergunta se uma função
tem chamador**, e o `clippy -D warnings` do ship teria reprovado a jornada inteira num sítio que
não explica nada.

E os dois últimos são a espécie **numérica** da mesma coisa: os dois lados baixam uma catraca por
motivos diferentes e **nenhum sabe o número final**. O `263` do `paint_inspector` não estava em
lado nenhum — quem o disse foi o censo de obsolescência do próprio gate.

**How to apply:**
1. ⭐⭐ **Ao resolver um conflito, pergunte se os dois lados estão a fazer a MESMA coisa** — se sim,
   fica o **nome** de um com o **corpo** do outro, e ⚠️ **confira a aritmética linha a linha antes
   de os fundir** (as duas médias de caixa eram idênticas; se não fossem, colapsar mudava pixels).
2. ⭐⭐ **Um número que os dois lados baixaram CONTA-SE, e o instrumento já existe:** ponha um valor
   provisório e deixe o **censo de obsolescência** do gate dizer o verdadeiro. Escolher um dos dois
   lados é uma licença com cara de catraca ([[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]]).
3. ⚠️ **Depois de toda a integração, um `cargo check --workspace` a olhar para os WARNINGS** — não
   só os erros. `dead_code` é o único sintoma desta família.

Relacionado: [[feedback_when_two_lines_pick_the_same_literal_the_collision_probe_goes_blind]],
[[feedback_collision_surface_reads_the_fork_point_not_the_tip_of_main]].
