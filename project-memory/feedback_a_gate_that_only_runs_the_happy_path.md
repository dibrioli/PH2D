---
name: feedback-a-gate-that-only-runs-the-happy-path
description: Um gate que só percorre o caminho que FUNCIONA não afirma nada sobre a cara de uma falha — e a cara de uma falha é o que o dono fotografa
metadata:
  type: feedback
---

**Medido em 2026-09-21.** Uma foto do dono mostrou toda recusa de um gesto a sair na tela com o
**✓ verde de sucesso**. A causa era de tipo: a porta devolvia uma `String`, logo o chamador não
tinha como saber se o gesto correra, e fazia `Toast::success(…)` em **todos** os casos.

A cura foi um enum de veredito. ⛔⛔ **E ela ficou sem régua durante uma hora:** a mutação que
troca `Err(…) => Recusado` por `Assado` passava os **dois** gates de ponta-a-ponta daquele gesto
**e a suíte inteira** — porque os dois só percorrem o caminho que **funciona**, e o braço do erro
nunca era exercitado por ninguém.

⇒ **duas leis:**

1. *Ao escrever um gate de gesto, pergunte qual é a POPULAÇÃO dele.* Um gate «ponta a ponta» que
   monta o caso bom mede metade da porta; a outra metade — **a recusa** — precisa do seu.
2. *A tradução de um `Result` para o que o artista vê é uma função PURA e mora fora do laço.*
   Escrita dentro do gesto ela só era alcançável com um `GpuContext`, e o gate nasceria
   `#[ignore]` — logo **o CI nunca o correria**. A lei da casa já o diz: *quando um gate precisa
   de um device para medir uma decisão que não tem pixel nenhum, a lei está no sítio errado*.

⭐ E o sinal de que falta este gate é barato de ver: **nenhum teste chama a porta com uma entrada
que ela tem de recusar.**

Relacionado: [[an-exemption-that-calls-screen-text-terminal-silences-the-census]]
