---
name: feedback-wide-mechanical-refactor-use-a-fingerprint
description: Refactor mecânico em muitos sítios de DSP/numérico = impressão digital antes/depois na MESMA máquina; golden pinado é mina no CI multi-SO (ulp de transcendental varia)
metadata:
  type: feedback
---

O [ADR-0117] D2 trocou o **container** de saída de 67 sítios em 39 efeitos da rack de áudio (de
`Vec` + cópia para `Arc` de escrita-única). A aritmética não muda **por construção** — mesmo laço,
mesma ordem, outro container. O risco real não era de design: era **erro de transcrição**.

A rede certa para isso:

1. **ANTES de tocar em nada**, rode uma varredura que imprime um digest por unidade (FNV sobre os
   **BITS** das amostras, não `==`: um zero com sinal trocado é diferença real, e NaN precisa
   comparar igual a si mesmo). Guarde a saída.
2. Refatore.
3. Rode de novo e **`diff`**. Idêntico = zero samples mudaram.

Deu certo: **39/39 byte-a-byte**. E pegou o que tinha de pegar — o `haas` lê um frame **anterior**
do canal direito; em cima do buffer de saída ele realimentaria a própria linha de atraso.
Corrupção plausível, que compila e passa em smoke.

**Por que NÃO virou golden pinado no repo:** o DSP do editor usa transcendentais (`tanh`/`sin`/
`exp` — o HR-5 não vale fora da thread RT), e o **último ulp desses varia entre libms de
plataforma**. Um hash pinado seria mina terrestre na matriz linux/macOS/windows do CI: vermelho
por um motivo que não tem nada a ver com o código. O digest fica como *printout* (um `#[test]` que
imprime), não como assert.

**Why:** varredura antes/depois na mesma máquina cobre 100% do risco que existe (transcrição) e 0%
do risco que não existe (design), sem importar um risco novo (fragilidade de plataforma). Um
golden pinado inverte isso.

**How to apply:** hospede a varredura **onde a enumeração já vive** (no caso, a tabela `KINDS` do
painel) — assim ela cobre o 40º item automaticamente, sem drift. Não invente uma lista à mão. E se
delegar os sítios a subagentes, o contrato deles é *"zero aritmética alterada"* + a impressão
digital como juiz — não "os testes passam".

Parente de [[feedback_mutate_the_code_not_just_the_test]] e do padrão oráculo-lento-julga-o-rápido
(`lpc.rs::solve` vs Levinson, `convolve.rs::direct` vs FFT, `ops_oracle.rs` vs o splice novo).
