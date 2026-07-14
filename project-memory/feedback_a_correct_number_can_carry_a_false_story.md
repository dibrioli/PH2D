---
name: feedback-a-correct-number-can-carry-a-false-story
description: Um número medido corretamente pode sustentar uma explicação errada — o gate fica verde e a afirmação do produto é falsa
metadata:
  type: feedback
---

**Motion, 2026-07-14.** Pus o `motion.delay` no documento de boot afirmando que ele *"tirava o tremor
da neve"*. Medi a 3ª diferença da trajetória e ela **caiu 47%** — número correto, gate verde,
commit escrito, ADR escrito.

O Enio: *"não sei se a neve treme como vc diz"*. **Ele estava certo.**

Medindo o que eu deveria ter medido **antes**: o desvio da aceleração de um floco é **0,00024 — 0,1%
da largura do próprio floco** — e a deriva lateral é **exatamente zero**. **A neve não treme.** O
`gust` do `force.wind` modula a *magnitude* de uma força que aponta reto pra baixo: o floco cai em
linha reta, só que mais rápido ou mais devagar.

Então o que a minha medição de 47% pegou? A ease amaciando **o SPLASH** — a batida no leito. Um
evento agudo, real, com 3ª diferença enorme. **O número estava certo. A HISTÓRIA que eu contei em
cima dele estava errada.**

**Why:** um gate prova que **a grandeza que você mediu mudou**. Ele **não** prova que ela é a
grandeza que você **disse** que era, nem que a causa é a que você **nomeou**. Verde não é verdade —
verde é *"o que eu afirmei sobre o número que eu escolhi se confirmou"*. A afirmação do produto ("a
neve para de tremer") vive numa camada acima, e **nenhum gate a alcança se a métrica não a
representar**.

**How to apply:**
- **Meça a CAUSA, não só o efeito.** Antes de dizer *"X causa Y"*, rode com **X desligado**. Eu tinha
  o botão (`gust = 0`) e não apertei. Um A/B de uma linha teria matado a afirmação em 30 segundos.
- **Ponha a grandeza numa unidade HUMANA.** "0,0283 de 3ª diferença" não quer dizer nada. "0,1% da
  largura do objeto" quer — e teria gritado. Se você não consegue exprimir o número no tamanho de uma
  coisa que o usuário vê, você ainda não entendeu o que mediu.
- **O gate tem que afirmar que o FIXTURE tem a propriedade.** O meu gate novo começa com
  `assert!(raw_twitch > 0.07)` — *"a coisa que eu digo que treme, treme"*. Sem isso eu estaria
  provando a suavização **de nada** ([[feedback_zero_valued_fixture_is_a_gate_that_cannot_fail]] é o
  irmão).
- **Quando o Enio duvida de uma afirmação sua, MEÇA — não defenda.** As duas vezes que ele duvidou
  nesta linha, ele estava certo, e nas duas o custo de checar foi de minutos.
- Irmão direto: [[feedback_oracle_must_model_appearance_not_implementation]] (o oráculo tem que
  modelar o que o usuário VÊ) e [[feedback_no_industrial_claims_without_verification]].
