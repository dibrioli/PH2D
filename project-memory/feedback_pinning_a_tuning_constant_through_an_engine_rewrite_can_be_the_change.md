---
name: feedback_pinning_a_tuning_constant_through_an_engine_rewrite_can_be_the_change
description: Fixar uma constante de afinação numa subida de versão parece conservador e não é — o número afinava o motor ANTIGO, e quando o motor é reescrito o valor preservado passa a ser ele o defeito
metadata:
  type: feedback
---

⭐⭐⭐ **Preservar uma constante de AFINAÇÃO através de uma reescrita de MOTOR não é conservador —
pode ser ELA a mudança.** O reflexo *«fixo o número para não mudar o tato em silêncio»* está certo
para um valor que descreve uma **decisão de produto**, e errado para um que **afina um mecanismo**:
o número afinava o mecanismo *antigo*, e o novo pede outro.

**Medido (2026-08-29, `rapier2d` 0.28 → 0.35).** O `sleep_linear_threshold` foi fixado no `0,4` da
0.28. A `rapier` baixou o dela para `0,05` — 8× — e nós tínhamos fixado exactamente o valor que ela
abandonou. Com o solver reescrito, um corpo assenta por outro caminho e cai abaixo de `0,4 m/s`
**a meio do assentamento**: adormece torto e não acorda mais.

| limiar | pior ângulo em repouso | corpos congelados |
|---|---|---|
| ⛔ `0,4` (fixado da versão antiga) | **`2,320°`** | 12/12 |
| ⭐ `0,05` (medido) | `0,04455°` | 12/12 |
| **controle: proibido dormir** | `0,04455°` | **0/12** |

⭐⭐ **O CONTROLE é o que separa cura de troca.** Sem ele, *«igual a não dormir»* podia ser a
tautologia de **nada ter adormecido**. Com ele: os corpos adormecem mesmo (deriva exactamente zero
entre o segundo 10 e o 20) **e** na pose de quem nunca dorme.

⚠️ **O botão vizinho não era a alavanca, e mexer nele sozinho PIORA 3×** (`7,62°`). *Muda-se o que a
medição exige, e nada mais.*

⛔⛔ **E a cura destapou um gate verde pela razão errada:** um teste afirmava *«um motor fraco não
levanta o braço»* e media `Σ|d|` — **caminho percorrido** — enquanto a afirmação era sobre
**posição**. Um braço parado a tremer acumula caminho sem sair do sítio; com o limiar antigo ele
adormecia e o tremor parava de somar. A rotação líquida sempre foi `0,0016 rad`. *Um gate que passa
porque o corpo adormeceu está a medir o sono, não a lei que diz medir.*

**Why:** o argumento a favor de fixar continua válido para valores **persistidos** (têm de ser
escritos, não lidos de uma fonte que se move). O errado não era escrever o número — era escolher,
para o escrever, **o número de um motor que já não existe**. *Um valor fixado herda a autoridade da
versão de onde veio, e essa autoridade caduca com ela.*

**How to apply:** ao fixar um valor numa subida, pergunte **o que ele afina**. Se afina um
mecanismo que a subida reescreveu, ele não se fixa — **mede-se**, contra um oráculo que desliga o
mecanismo. Ver
[[feedback_a_changelog_describes_what_the_authors_announced_not_what_our_code_touches]] e
[[feedback_a_gate_on_the_mark_i_chose_is_green_when_the_marks_premise_is_false]].
