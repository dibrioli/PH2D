---
name: feedback_the_first_crossing_of_a_resonant_response_is_not_the_boundary
description: "Numa resposta não-monótona o primeiro cruzamento da barra é uma RESSONÂNCIA, não a fronteira — a pergunta é «todos os valores até este sobrevivem?», e só o prefixo-máximo a faz"
metadata:
  type: feedback
---

Medindo o `MAX_DT` dos integradores (bloco Z, [[reference_topic_gate_discipline]]), a
sonda varria `dt` e parava no **primeiro** valor que saía da barra. Devolveu `0,0300`.

⚠️ **E `0,0333` — um quadro perdido a 30 fps, o caso mais comum de todos — media
`0,89` contra uma barra de `1,66` e passava folgadamente.** A varredura grossa e a
fina **discordavam**, e foi essa discordância que denunciou o instrumento.

A causa: um laço fechado com força central tem **ressonâncias**. A excursão não é
monótona em `dt` (`0,0325` → 3,57 · `0,0333` → 0,89 · `0,05` → 4,43 · `0,0525` →
4,39 · `0,085` → 3,62 · `0,0825` → **96,22**). O mesmo em `strength` (120 → 1,63 ·
160 → 0,89).

**Why:** *"este valor sobrevive?"* e *"todos os valores até este sobrevivem?"* são
perguntas diferentes, e **só a segunda define um teto** — um grampo admite tudo o que
está abaixo dele, não só o valor que ele nomeia. Sobre uma resposta monótona as duas
coincidem, e é por isso que o erro passa despercebido até ao dia em que a resposta
deixa de o ser.

**How to apply:**
1. Antes de procurar uma fronteira, pergunte se a resposta **pode** ser não-monótona:
   realimentação, ressonância, caos, hash/seed. Se pode, o primeiro cruzamento não
   serve.
2. Use **prefixo-máximo** — `prefix = prefix.max(e)` a cada passo, e o teto é o
   último em que `prefix` cabe na barra. Ele é monótono por construção.
3. **IMPRIMA a varredura inteira**, não só o veredito: foi o platô no fim (a excursão
   a congelar em `127,03` acima de `0,1`) que provou que a sonda media o grampo real,
   e foi a coluna toda que mostrou as ressonâncias.
4. Duas réguas que discordam é **informação**; uma régua só é uma opinião.

*Irmã de [[feedback_an_unlabelled_probe_column_gets_read_backwards]] e de
[[feedback_a_cure_measured_on_a_fixture_that_lacks_the_phenomenon_reads_as_useless]]:
as três são o instrumento a mentir com o produto correcto.*
