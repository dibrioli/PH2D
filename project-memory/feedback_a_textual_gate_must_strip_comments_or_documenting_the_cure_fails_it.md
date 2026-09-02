---
name: a-textual-gate-must-strip-comments-or-documenting-the-cure-fails-it
description: "Um gate que varre TEXTO tem de apagar comentários e literais antes de procurar — senão documentar a própria cura reprova o portão; e um controlo que pende da excepção declarada fica vermelho quando a lei vence"
metadata:
  type: feedback
---

**Medido por auditoria adversarial na `line/motion-value`, 2026-08-30 — nove derrotas num
gate acabado de escrever.**

### A armadilha principal: o falso positivo é a documentação

O gate procurava `paint_text(` no fonte cru. Escrever no comentário da própria cura *"antes
isto era `paint_text(ts, sc, label, x, y, f, rect.w, cor)` e quebrava"* — **a frase natural do
próximo autor** — reprovava o portão, acusando uma linha de comentário de ser um rótulo. Cinco
falsos positivos confirmados (comentário de linha, doc-comment, comentário de bloco, literal,
`concat!`).

E o mesmo buraco dá o **falso negativo**: um literal com parêntese desequilibrado (`"Tropism :("`)
desalinha a contagem de argumentos e a chamada seguinte escapa.

⇒ **descasque comentários e literais primeiro**, trocando cada byte por um espaço (as linhas e
os offsets não se movem, então a mensagem de erro continua a apontar certo).

### As outras duas, que valem por si

- ⛔ **`contains("INFINITY")` aceita `const INFINITY_BUDGET_PX: f32 = 120.0;`** e
  `if false { f32::INFINITY } else { rect.w }`. Use igualdade EXACTA sobre o argumento
  normalizado.
- ⛔⛔ **Um controlo de «a ferramenta ainda existe» que conta no ALVO da lei fica vermelho
  quando a lei vence por completo** — e com a mensagem errada (*"foram renomeadas"*). O meu
  contava as portas que quebram nos ficheiros varridos, e pendia de UMA chamada: a excepção
  declarada. Conte-o na **árvore inteira**, ou num canário fora do alcance da lei.

### E o que um gate textual NÃO alcança, declare-o

`use ph2d_editor_core::paint::paint_text as wrap_label;` passa sempre. *Um gate que finge
alcançar o que não alcança é pior que um que diz onde acaba.*

**How to apply:** ao escrever um gate que varre fonte, escreva ANTES as cinco entradas que o
enganariam (comentário · doc-comment · literal · nome parcial · alias) e prove cada uma por
mutação. Relacionado: [[reference_topic_gate_discipline]] ·
[[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]].
