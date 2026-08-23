---
name: feedback_n_sources_need_the_cross_check_not_n_self_checks
description: Um portão que confere cada fonte CONSIGO PRÓPRIA e nunca as fontes entre si passa sobre dados de objectos diferentes — com N fontes, o que falta são as N−1 comparações CRUZADAS
metadata:
  type: feedback
---

Quando um cálculo cruza **N ficheiros/fontes**, o portão natural é conferir cada uma
consigo própria (*«o cabeçalho diz 9 534 e eu li 9 534 direcções ✓»*). ⛔ **Isso não
prova nada sobre elas falarem do mesmo objecto.** O que falta são as **N−1 comparações
cruzadas**, e é exactamente a que ninguém escreve.

**Why:** medido no quad remesh (2026-08-23). Uma sonda cruzava três ficheiros do
oráculo: a malha (`_rem_p0.obj`), o **campo** dele (`_rem.rosy`) e a **decomposição**
dele (`_rem_p0.patch`). A conferência era:

```rust
if dirs.len() != count || owner.len() != pn || pn != om.mesh.faces().len() { … }
```

— campo contra o próprio cabeçalho ✓, decomposição contra o próprio cabeçalho ✓,
decomposição contra a malha ✓. **`count` e `pn` nunca se olharam.** ⚠️ E são de malhas
diferentes: `_rem.obj` tem 9 534 faces e `_rem_p0.obj` tem 9 638 — o segundo é o
primeiro **já cortado nas *feature lines***. ⇒ a sonda media *o campo de uma face nos
patches de outra*, e o número saía plausível: `18,6°`, que foi citado como controlo para
**ilibar** uma acusação. ⛔ O comentário ao lado do `if` **já nomeava o risco** («o campo
vem do `_rem.obj` e a decomposição do `_rem_p0.obj`») — *nomear o risco e não o testar
lê-se, meses depois, como se ele estivesse testado*.

**How to apply:**
1. Conte as fontes. Com `N` fontes o portão precisa de **`N−1` igualdades entre elas**,
   não de `N` auto-conferências. Escreva-as como uma cadeia: `a == b && b == c`.
2. ⚠️ **Um comentário que descreve o risco não é o portão.** Se o comentário diz «estes
   dois podem ser de malhas diferentes», a linha seguinte tem de os comparar.
3. Quando duas fontes **não são cruzáveis**, a resposta é **recusar alto** e dizer as
   duas contagens — não medir e não silenciar. Depois procure um instrumento que não
   precise do controlo emprestado (aqui: contar as singularidades **sem canto** na nossa
   própria decomposição respondeu melhor à mesma pergunta, sem tocar no oráculo).
4. Todo número já citado a partir da fonte cruzada **volta ao estaleiro**, mesmo que a
   conclusão sobreviva por outra via.

Irmãs: [[feedback_a_bucket_nobody_fills_reads_as_perfect]] ·
[[feedback_a_negative_search_needs_a_positive_control]] ·
[[reference_topic_oracle_discipline]] · [[reference_topic_fixture_discipline]]
