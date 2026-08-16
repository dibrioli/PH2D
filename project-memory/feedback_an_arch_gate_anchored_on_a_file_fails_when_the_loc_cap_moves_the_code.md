---
name: feedback-an-arch-gate-anchored-on-a-file-fails-when-the-loc-cap-moves-the-code
description: Arch-gate ancorado num ARQUIVO reprova sobre produto CORRETO no dia do corte de LOC; leia a FAMÍLIA por uma porta única
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 19d5e73b-a74f-4f8e-b11e-3bdf0947db8e
  modified: 2026-08-13T00:33:16.728Z
---

Um arch-gate que varre `src/foo.rs` **reprova sobre produto correto** no dia em
que o teto de LOC (HR-18) move a função vigiada para o irmão. Aconteceu quatro
vezes neste repo — `line/Vector` (23/07), `line/Painter` (08/08), `line/Vector`
de novo (i18n, 10/08) e `line/physics` (12/08, dois gates de uma vez quando o
`kinematic_settle` e o `push_from_hits` saíram do `player.rs`).

**Why:** o teto de LOC é uma força que age sobre TODO arquivo que cresce, então
todo arch-gate ancorado num nome de arquivo tem prazo de validade. E o modo de
falha caro não é o vermelho — é o **VERDE POR VÁCUO**: um scanner que não acha
mais a âncora satisfaz *"no máximo uma ocorrência"* com zero, e passa a proteger
nada.

**How to apply:** afirme a PROPRIEDADE, nunca o endereço.

- Quem **CONTA** (quantos leitores existem, onde uma chamada mora) varre a
  **FAMÍLIA** (`foo.rs` + `foo_*.rs`), por uma **porta única** partilhada por
  `#[path]` — duas cópias de *"que texto é este assunto?"* divergem no primeiro
  corte que uma delas não vir.
- Quem afirma **ORDEM DENTRO de uma função** continua a ler só aquele arquivo:
  ordem entre pontos de arquivos diferentes é a ordem alfabética dos irmãos, não
  uma propriedade do produto. As duas perguntas são diferentes.
- **Todo scanner precisa do controle positivo** (`expect` que a âncora existe) —
  é ele que transforma o corte num vermelho alto em vez de numa varredura vazia,
  e foi ele que disparou nas quatro vezes.

Irmãos: [[feedback_a_condition_that_enumerates_its_readers_rots]] ·
[[reference_topic_gate_discipline]] · [[feedback_a_negative_search_needs_a_positive_control]]
