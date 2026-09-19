---
name: an-inverse-law-in-f32-does-not-close-by-algebra
description: Um par ida/volta `w = t + c` / `orçamento = w − c` NÃO fecha em f32 — a inversa tem de se conferir contra a LEI, e a prova tem de VARRER, não amostrar.
metadata:
  type: feedback
---

⛔⛔⛔ **Toda lei desta casa do tipo *«que caixa serve este texto?»* / *«quanto desta caixa é do
texto?»* é `w = t + c` e `orçamento = w − c`. Em aritmética real isso fecha; em `f32` NÃO.**

O arredondamento de `t + c` faz `(t + c) − c` cair **abaixo** de `t`, e toda elisão desta casa
compara `<=` ⇒ ***um défice de um ULP corta a palavra inteira***.

Medido em 2026-09-19 (`line/UIUX`), varrendo o domínio `0,5..600 px` em passos de `~1e-3`:

| par ida/volta | falhas | pior défice |
|---|---:|---:|
| `rect_for_label` / `label_budget` (**já shipava**) | `2,65 %` | `3,05e-5 px` |
| `Tag::width_for` / `Tag::label_budget` | **`~96 %`** | `3,05e-5 px` |
| `dropdown_chip_width_for` / `dropdown_label_budget` | apanhado pelo **PRODUTO** | — |

⭐ **A cura é a inversa perguntar à LEI, nunca confiar na subtracção:**

```rust
let w = text_w + chrome;
if orcamento(w) < text_w { w.next_up() } else { w }
```

Um `next_up` basta (verificado em `7 361 552` pontos, `0` falhas).

⛔⛔ **E a prova que guardava o primeiro par amostrava SEIS valores e passava nos seis.** *Uma prova
por amostras sobre uma lei que falha em `2,65 %` do domínio lê-se como prova.* A prova a sério
**varre** — `~460 000` pontos por altura — e tem um **controlo** (o invólucro EXISTE), senão uma
inversa que devolvesse o próprio texto passaria.

⚠️ **O terceiro par foi encontrado pelo ECRÃ, não pela varredura:** uma coluna passou a pedir
**exactamente** o que a palavra mais larga mede (`63,62`), recebeu esse número de volta, e o rótulo
saiu `Dictiona…`. *Quando uma coluna TIGHT corta o membro que a define, o suspeito é a aritmética,
não a lei.*

Ver [[a-column-of-a-family-measures-the-family]] e [[reference-topic-measurement-discipline]].
