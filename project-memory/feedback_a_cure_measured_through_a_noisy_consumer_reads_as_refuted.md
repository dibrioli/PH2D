---
name: feedback-a-cure-measured-through-a-noisy-consumer-reads-as-refuted
description: "Cura medida ATRAVÉS de um consumidor barulhento lê-se como inútil — meça-a onde ela vive, com o controlo ao lado"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7499b0f4-218e-489b-879b-1e5a1c8b851f
  modified: 2026-08-31T23:00:44.134Z
---

Uma cura de uma FASE tem de ser medida **naquela fase**, com a fixtura mais simples que
contenha o fenómeno. Medi-la pela saída final de uma cadeia longa mede a soma de todos os
ruídos a jusante — e se a cadeia amplifica, o sinal da cura desaparece.

Medido 2026-08-31 (`line/quadextract`): ancorar a grelha de densidade da fase zero na caixa
da peça foi declarado **REFUTADO** porque, correndo o botão inteiro, a contagem de vértices
continuava a variar e uma célula piorou. Isolada na crate (`uv_sphere`, cinco translações,
**12 segundos**), a mesma mudança leva a dispersão de **`4,9 %` para `0,0 %`** — inequívoca.
*Um dia inteiro a medir pela saída final, e a resposta estava a 12 segundos de distância.*

⭐ **O que a tornou legível foi o CONTROLO na mesma corrida:** o caminho **sem** o campo tem
de ser exactamente invariante. Ele é ⇒ o laço já estava certo e o defeito era do campo. Sem
essa coluna, «varia» não tem dono.

**Why:** «a cura não funciona» e «o meu instrumento não a vê» produzem a mesma tabela — e a
primeira leitura deita fora trabalho correcto.

**How to apply:** antes de declarar uma cura refutada, pergunte *estou a medi-la na fase em
que ela vive?* Escreva a sonda da fase (barata, sem GPU, uma fixtura sintética) e corra o
controlo — a mesma fase com o mecanismo **desligado** — na mesma corrida. Só depois volte ao
consumidor. Ver [[feedback-a-phase-measured-alone-can-improve-and-make-the-pipeline-worse]],
que é a lei no sentido CONTRÁRIO: as duas juntas dizem *meça nos dois sítios e diga qual é
qual*.

---

⭐⭐ **Terceira forma, e o consumidor barulhento é o PARALELISMO** (2026-09-01, auditoria de
performance do Motion): içar dois `BTreeMap<String,_>` para fora de um laço por elemento
poupa **48,0 ms em 4,19 M linhas** (`11,7 → 0,2 ns/linha`, **47,8×**) — medido sozinho. Medida
**pelo caminho que a consome**, a mesma cura **não se mexeu**: aquele lowering já corria em
`par_extend`, logo 48 ms repartidos por 32 fios são ~1,5 ms, dentro do ruído de uma medição de
43 ms. Eu tinha escrito *«~73% do custo são os lookups»* por aritmética sobre as colunas, e a
medição refutou-o.

**A forma que se lê:** o consumidor **amortiza** (paralelismo, cache, um passo que já
domina) ⇒ toda poupança real aparece dividida por um factor que a régua não mostra. ⇒ **meça
o trabalho REMOVIDO sozinho, num laço serial**, e só depois olhe para o consumidor. E note
que a cura continua certa nos dois casos: no caminho serial irmão ela pagou `2,47×`.
