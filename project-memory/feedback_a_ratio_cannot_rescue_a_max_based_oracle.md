---
name: feedback-a-ratio-cannot-rescue-a-max-based-oracle
description: "Uma razão cancela deriva de máquina só entre reduções comparáveis; sobre um MAX o ruído é aditivo e só no numerador — se a propriedade é estrutural, o oráculo é o fonte"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 39ec3808-26ec-4cf4-b80e-b2291882bc64
  modified: 2026-08-02T16:32:29.867Z
---

Trocar wall-clock por **razão** é o reflexo certo contra deriva de máquina — **e ele falha sobre um
`max`**. Uma razão cancela a máquina quando os dois termos são **reduções comparáveis do mesmo
trabalho** (média÷média, mínimo÷mínimo). Um **MAX é ímã de outlier**: sob carga ele mede *a pior
preempção do SO na janela*, ruído **aditivo e só no numerador**, e o denominador não tem outlier
nenhum para cancelá-lo.

**Caso medido (PH2D, `the_tick_never_waits_for_a_whole_stage`, 2026-08-02):** barra `worst < 30 ms`
flakando **isolada** sob `load 38` (FAILED/ok/FAILED). Virei razão `pior tick ÷ custo de um passo`,
medida na mesma corrida pela porta do produto — e ela flakou igual: **1,82 · 1,41 · ok · 0,77**, com
`worst` em 47,63 ms contra 26,12 de passo.

**Why:** a doc do gate já trazia a flake com o número e concluía *"a barra fica onde está"*. Um gate
que alterna sob carga não é acreditado — é **silenciado**, que é pior que não o ter.

**How to apply:**
1. Antes de escolher a barra, pergunte **qual redução** cada lado é. `max ÷ min` não é razão, é ruído
   sobre sinal.
2. **Se a propriedade é ESTRUTURAL, o oráculo é o fonte, não o relógio.** Ali era *a porta do tick pede
   o motor com espera limitada* (`recv_timeout` e não `recv`) — scanner que roda em 0,00 s e é imune à
   máquina, com **controle positivo nas duas pontas** (a porta que DEVE bloquear ainda bloqueia).
3. O número não se joga fora: vira sonda `#[ignore]` dizendo que sob carga ele não fala sobre o código
   (ver [[reference_topic_gate_discipline]] e [[feedback_probes_that_measure_parallelism_must_run_alone]]).
