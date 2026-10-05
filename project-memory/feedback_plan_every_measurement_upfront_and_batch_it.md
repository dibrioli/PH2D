---
name: feedback_plan_every_measurement_upfront_and_batch_it
description: "O dono reclamou do passo a passo sem fim (04/10) — planeie TODAS as variantes e ablações à partida e meça-as numa só rodada intercalada, nunca rodada a rodada"
metadata:
  node_type: memory
  type: feedback
  originSessionId: 05b46104-5739-47da-9ad9-45eca9b9e2a4
  modified: 2026-10-04T12:23:16.204Z
---

Itens 3 e 4 do doc 121 (03–04/10, line/motion-value): ~4 h de relógio, das quais ~2 h à espera de máquina
calma/placa livre, em QUATRO rodadas de medição em série (base → 3a/3b → ablações zera/varre → custos
fixos da esparsa → app). Cada rodada só foi desenhada depois de ler a anterior. O Enio perguntou: *«por que
vc está há milênios neste único assunto num passo a passo sem fim? Não seria possível fazer uma análise
mais global e resolver os problemas de uma vez?»*

⭐ **Virou REGRA do dono (04/10): `CLAUDE.md` §0.10 — problemas equivalentes resolvem-se num ÚNICO bloco.**
*«Minha intenção é acabar esse app antes que a morte chegue. Então vamos trabalhar mais rapidamente.»*
Vale para tudo, não só para medição: defeitos da mesma família, variantes, gates, docs.

**Why:** cada rodada calma custa 30–60 min nesta máquina partilhada; o custo é o NÚMERO de rodadas, não o de
binários. Quase tudo o que medi em série era previsível à partida (as ablações de um teto e dos custos fixos
de uma variante são sempre as mesmas perguntas). E eu próprio causei esperas: uma corrida trocada a meio,
e mutações lançadas em paralelo com o verificador na MESMA árvore (tive de as matar).

**How to apply:**
- Antes de medir, escreva a LISTA de todos os binários que a decisão vai precisar (base, cada pedaço da
  mudança, as ablações do teto e dos custos de cada alternativa) e compile-os todos de seguida.
- Uma só corrida intercalada com todos eles na mesma janela calma; a decisão sai da tabela inteira.
- Trabalho que muta a árvore (mutação) e trabalho que a compila (gates) correm EM SÉRIE, planeados.
- Ao dono: diga logo no início quanto tempo é espera de máquina e quanto é trabalho.

⛔⛔ **05/10 — e mesmo UMA rodada «de binários» levou 4–5 h** (o §9.15: `360` células, `15` compilações, `20` s de
calma antes de CADA célula = `2 h` de espera com a máquina ociosa, mais `2 h 35` presa pela carga de outras linhas).
O dono: *«não podemos esperar horas»*, e *«20 a 30 minutos é muito lento»*. ⇒ **variantes são PASSES do mesmo
processo, não binários**: constantes `override` escolhidas ao criar o pipeline, blocos de quadros intercalados
por ordem rodada, o MÍNIMO das rodadas (interferência só soma), nenhuma espera de calma — a mesma rodada em
**`37 s`** com `load 20`–`29`, mínimo = mediana ao `0,01` (doc 121 §9.16, `mede_intercalado.sh`). Peça nova
que se vai medir nasce já como `override`/porta de execução, nunca como `const` a trocar por script.

Relacionado: [[feedback_an_old_attribution_is_worth_more_after_ablating_todays_suspect]] ·
[[reference_topic_measurement_discipline]]
