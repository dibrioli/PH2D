---
name: a-quantity-cannot-both-position-and-limit
description: "Uma estimativa que POSICIONA um elemento não pode ser reusada como LIMITE dele — é circular, e a troca de uma lei tolerante por uma estrita transforma-a de inofensiva em destrutiva"
metadata:
  type: feedback
---

**Erro MEU, medido por auditoria na `line/motion-value`, 2026-08-30.**

Ao varrer 26 rótulos de `paint_text` (cujo `max_width` é orçamento de **QUEBRA**) para
`paint_text_elided` (cujo `max_width` é limite de **CORTE**), troquei também um sítio onde a
largura passada era `approx_width_px()` — uma **estimativa por avanço médio** que é a MESMA
quantidade que **posiciona** o rótulo (`x = … − text_w`, alinhamento à direita).

- Como orçamento de quebra, uma subestimativa é inofensiva: o parley não parte dentro de uma
  palavra, logo o texto transborda ~2 px para o padding.
- Como limite de corte, ela **apaga texto**: medido, a porta `Out` (estimativa `16,80`, real
  `19,07`) saía como **`…`** — o rótulo inteiro desaparecido.

**Why:** a estimativa e o limite são o mesmo número, então o texto nunca pode caber nele —
é circular por construção. E o modo de falha só aparece quando a lei a jusante deixa de ser
tolerante. *Uma varredura mecânica é segura enquanto a semântica do argumento for a mesma em
todos os sítios; onde ela muda, a varredura é uma mudança de comportamento disfarçada.*

**How to apply:** antes de trocar em massa uma porta por outra mais estrita, para cada sítio
pergunte **de onde vem o argumento**, não só o que ele significa. Se ele for derivado do
próprio conteúdo (uma estimativa da largura do texto, a caixa que o texto gerou), ele não é um
limite — é geometria, e o sítio é uma excepção. Relacionado:
[[feedback_a_parameter_that_changes_nothing_is_discarded_downstream]] ·
[[feedback_the_design_being_asked_for_may_already_be_law_in_another_half_of_the_app]].
