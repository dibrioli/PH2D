---
name: feedback-an-optimization-needs-a-gate-that-proves-it-fires
description: Otimização com fallback precisa de um gate que prove que ela DISPARA — senão vira código morto, todos os outros gates ficam verdes, e o único sintoma é que nada acelerou
metadata:
  type: feedback
---

Toda otimização com **caminho de fallback** ("tenta o rápido; se não der, faz o completo") tem um
modo de falha que **nenhum teste de correção pega**: o caminho rápido **nunca dispara**. O programa
continua **perfeitamente correto** — ele só nunca acelera. Byte-identidade: verde. Correção: verde.
Perf: exatamente igual a antes, e você não sabe por quê.

**O caso (ADR-0120, preview de knob, 2026-07-13).** O preview passou a reescrever só a região da
seleção num buffer que já possuímos, em vez de copiar o clipe inteiro (16,86 ms → 0,27 ms). O
acesso mutável é `Arc::get_mut`, que **recusa** enquanto existir um clone. Inicializei o scratch
com `head.data().clone()` — e um `clone()` de `SampleData` **bumpa o `Arc`, não copia os dados**.
O head continuava segurando o buffer ⇒ refcount 2 ⇒ **`get_mut` recusaria PARA SEMPRE** ⇒ o
caminho rápido cairia no fallback **todo frame**, para sempre. Era `SampleData::map_in_place` (uma
cópia de verdade).

Todos os outros gates ficariam **verdes**. O único sintoma seria que os knobs continuavam
exatamente tão lentos quanto antes — e "otimizei e não melhorou" é um beco onde se perde um dia.

**Why:** um fallback é uma **rede de segurança que também é uma mordaça**. Ele garante que o bug
nunca apareça como erro — só como ausência de ganho. E ausência de ganho é o sintoma mais fácil de
racionalizar do mundo ("deve ser outra coisa", "a máquina está carregada").

Parente próximo: [[reference_arc_from_vec_always_copies]] (`Arc::from(Vec)` SEMPRE copia) — este é
o **espelho**: `Arc::clone` **NUNCA** copia. As duas surpresas do mesmo tipo, em direções opostas.

**How to apply:** ao escrever um caminho rápido com fallback, escreva **dois** gates:

1. **Corretude** — o rápido e o lento concordam (byte a byte, se possível).
2. **Que ele DISPARA** — conte as vezes que o caminho rápido foi tomado, num cenário que reproduz
   as condições REAIS (no ADR-0120: o mixer segurando um buffer e devolvendo pela return ring; 8 de
   8 frames). Se a condição real envolve outra thread/sistema, **simule-a no teste** — não confie
   em que "deve dar certo".

Sem (2), você não construiu uma otimização; construiu um fallback com um comentário otimista.
