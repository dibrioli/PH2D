---
name: feedback-a-ratio-gate-measured-at-the-degenerate-point-invents-a-debt-list
description: Gate medido no extremo do parâmetro acusa 11 formas; no ponto de trabalho só 2 eram defeito — varra o parâmetro antes de escrever a catraca
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-08-31T02:05:02.722Z
---

Uma barra medida no **extremo** de um parâmetro mede a **geometria do pedido**, não um defeito — e
a lista de dívida tolerada que sai daí é, na maior parte, **inventada**.

Caso medido (PH2D, `line/3DModeling`, 2026-08-30): o gate do chanfro media `filete = chanfro`, que
é onde o filete já consumiu a faceta inteira do chanfro e **não há aresta distinta para
arredondar**. Onze formas apareciam «pioradas». Varrendo a razão:

| filete | `0,25c` | `0,5c` | `0,75c` | `c` |
|---|---:|---:|---:|---:|
| bando (18) | `0,92`–`3,37` | **`0,90`–`2,40`** | `0,92`–`2,36` | `0,85`–`2,41` |
| cruz | `16,16` | `15,95` | `11,18` | `9,31` |
| engrenagem | `3,83` | `3,78` | `2,72` | `2,27` |

⇒ no **ponto de trabalho** (`0,5c`) só **duas** destoavam, e as duas eram estruturais. Curadas, a
lista de toleradas ficou **VAZIA** — de onze para zero, e nove delas nunca tinham sido defeito.

**Why:** uma catraca com entradas que não são defeitos é pior que nenhuma — ela **licencia** o que
não precisava de licença, e esconde as que importam no meio do ruído.

**How to apply:**
- **Varra o parâmetro ANTES de escrever a catraca.** A coluna onde o bando é mais apertado é o
  ponto de trabalho; um destoante ali é estrutural, um destoante só no extremo é geometria.
- ⭐ Procure a **forma da curva**, não o valor: uma que melhora monotonamente com o parâmetro é o
  bando; uma plana ou explosiva num extremo tem outra causa (o ápice do cone, a saturação).
- ⚠️ Barra de **corpus** sobre uma lista **FECHADA** (um enum `ALL`) é a coisa certa — é justamente
  onde «as peças que testei» **são** «as peças que existem», e uma entrada nova que a estoure é o
  que se quer acusado. Sobre um corpus aberto seria o erro de sempre.
- ⚠️ Meça em **dois pontos** com barras diferentes se o extremo ainda interessa: o de trabalho
  aperta, o extremo apanha catástrofe. Ver [[reference_topic_gate_discipline]] e
  [[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]].
