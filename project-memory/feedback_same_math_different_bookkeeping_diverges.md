---
name: feedback-same-math-different-bookkeeping-diverges
description: Dois caminhos que calculam "a mesma coisa" mas ESCRITURAM diferente acumulam arredondamentos diferentes — 1 ulp. Gate byte-idêntico pega; "soa igual" não
metadata:
  type: feedback
---

No [ADR-0118] a voz por **streaming** tinha de renderizar o mesmo buffer que a voz **residente**.
Cinco dos seis gates passaram de primeira — inclusive os difíceis (advance fracionário 44,1→48 kHz,
up-mix mono, o fim exato). O sexto, o **loop**, falhou por **1 ulp**: `-0.18182747` contra
`-0.18182749`.

Não era off-by-one, nem janela errada, nem costura torta. A aritmética era idêntica. O que diferia
era a **escrituração do cursor**: a voz residente **subtrai o comprimento do clipe** a cada volta
(`cursor -= frame_count`), e a stream deixava o cursor crescer indefinidamente. Mesma música,
sequência **diferente** de arredondamentos `f64` → `frac` diferente → saída diferente no último bit.

**Why:** "as duas fazem a mesma conta" não basta. Se os dois caminhos **guardam o estado de forma
diferente**, o ponto flutuante os separa — devagar, silenciosamente, e de um jeito que nenhum ouvido
e nenhum teste de tolerância pega. E aqui mora a armadilha: 1 ulp é −140 dB, é *inaudível*, e
"inaudível" é exatamente o raciocínio que deixa bug de verdade passar. Foi por isso que eu escrevi o
gate em **igualdade de bits** e não em tolerância.

O conserto foi na raiz, não no gate: a stream passou a fazer o **mesmo wrap** (um `base` guarda o
que foi subtraído, e a janela ainda sabe qual frame absoluto procurar). E o cursor limitado é
**correto por mérito próprio** — uma stream ambiente de horas com cursor ilimitado perde resolução
fracionária de verdade. Quando o conserto certo é bom por outro motivo além de passar o gate, é
sinal de que o gate estava certo.

**How to apply:** ao escrever um 2º caminho para algo que já existe (streaming vs residente, GPU vs
CPU, incremental vs batch, cache vs recompute):
1. O gate é **byte-idêntico**, nunca "dentro de ε". ε esconde exatamente a classe de bug que só
   aparece depois de muito tempo rodando.
2. Compare não só a **fórmula** mas o **estado**: o que cada caminho acumula, e como e quando ele o
   reseta/normaliza. Divergência de escrituração é invisível na leitura do código e óbvia no diff de
   bits.
3. Se um lado tem menos informação que o outro (a stream não sabia o comprimento), **dê a ele a
   informação** — não relaxe o gate. Aqui o produtor já sabia; só precisava contar e publicar.

Parente de [[feedback_derived_coordinate_seed_must_match_sample]] (autoria e leitura usam a MESMA
transform) e de [[feedback_frozen_bar_check_the_arithmetic_before_gaming_it]].
