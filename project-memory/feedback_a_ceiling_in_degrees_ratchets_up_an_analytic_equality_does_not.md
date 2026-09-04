---
name: feedback-a-ceiling-in-degrees-ratchets-up-an-analytic-equality-does-not
description: "Quando uma cura CORRECTA piora um número tolerado, não suba o tecto — procure a conta fechada que prevê o número novo; se ela bate, quem estava errado era o tecto"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-03T16:57:10.440Z
---

Uma lista de dívida tolerada («esta forma pode chegar a `48°`») parece uma catraca que só desce.
⚠️ **Ela sobe na primeira vez que uma cura correcta piora o número** — e sobe com uma justificação
plausível, porque a cura *é* correcta. Foi o que ia acontecer aqui: `48` → `85`.

Caso medido (PH2D, `line/3DModeling`, 2026-09-03 — o chanfro honesto). O corte deixou de descer
`c/sin 2α` e passou a descer `c`. O pior giro da estrela **subiu** de `45,8°` para `82,4°`.

⭐⭐⭐ **A saída não é o tecto: é perguntar se existe uma conta fechada que preveja o número NOVO.**
Numa ponta de estrela o chanfro do aro deixa duas facetas com normais `(nᵢ + ẑ)/√2`, logo o ângulo
entre elas é `arccos((κ+1)/2) = 83,81°` — e a sonda lê `82,4°`–`84,9°` em **toda** a varredura do
chanfro. ⇒ os `82,4°` **são** a geometria; os `45,8°` eram comprados a cortar a ponta `1,61×` mais
fundo do que o slider dizia, o que apagava o vértice antes de o aro lá chegar.

**How to apply:**
- ⭐⭐ Substitua o tecto por uma **igualdade**: o gate passou a exigir `|medido − previsto| ≤ folga`.
  Ela reprova nos **dois** sentidos — e é o sentido de baixo que interessa, porque *encolher* aquele
  número só se consegue voltando a cortar a mais. Um tecto só por cima premiaria o defeito removido
  ([[feedback_a_ruler_that_counts_leftover_defect_rewards_overshooting]]).
- ⭐ O sinal de que a conta fechada existe é a **planura**: varra o knob que a hipótese de tamanho
  diz que cura, e se a curva não se mexer, o número é estrutural — um vértice, não uma quantidade
  ([[feedback_a_second_error_can_be_load_bearing_for_the_first]]).
- ⚠️ Ache o **controlo** que separa as duas explicações. Aqui foi medir a mesma peça com o mesmo
  filete pequeno e **sem** chanfro: `14,3°`. Isso matou *«o par usa metade do filete, por isso é
  mais afiado»* e deixou só *«o chanfro deixa duas facetas»*.
- ⛔ E confirme que a entrada não virou licença: a metade do censo que diz *«esta entrada ainda
  descreve alguma coisa?»* fica, mesmo depois de o tecto virar lei
  ([[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]]).
