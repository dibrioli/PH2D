---
name: feedback-a-click-toggles-a-marquee-adds-and-the-asymmetry-is-the-law
description: Um clique tem um alvo visível e pode alternar; um retângulo cobre vários e tem de SOMAR — alternar em lote depende de estado que o artista não vê
metadata:
  type: feedback
---

Fiz o laço de seleção 3D **alternar**, pelo mesmo raciocínio que o impedia de limpar (*a tecla que o
abriu diz «selecção»*). Enio: *"se uma peça estiver selecionada e outra não, o retângulo não
seleciona todas, mas inverte a seleção"*.

**Why:** o raciocínio estava certo até meio caminho — ele justifica **não limpar**, e eu dei um passo
a mais para **alternar**. A assimetria com o clique é a lei: um clique tem um alvo **único e
visível**, e alternar é preciso e reversível; um rectângulo cobre **vários**, alguns já escolhidos, e
alternar mistura estados que o artista **não vê**. *Um gesto cujo resultado depende de estado
invisível não é usável.*

**How to apply:** gesto de **um** alvo visível → pode alternar. Gesto de **lote** → soma. E ao gatear:
⚠️ **uma fixtura que começa com a seleção VAZIA não distingue os dois verbos** — com ela vazia,
alternar e acrescentar são a mesma coisa. Ponha estado prévio.
[[feedback-where-new-objects-are-born-is-the-fixture-your-gates-are-missing]] é a irmã espacial desta.
