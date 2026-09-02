---
name: feedback-a-permanent-band-must-return-more-screen-than-it-eats
description: "O alvo do PH2D é tablet/iPad — toda faixa permanente de chrome tem de dizer o que custa em % de área nos três alvos, e uma NOTA sobre o preço não trava nada; só um gate"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: af27d1c2-3a56-4abe-9acd-e2c91caf58f0
  modified: 2026-08-31T22:22:13.799Z
---

O PH2D é para **tablet e iPad** (Enio, 2026-08-31). ⇒ **nenhuma faixa permanente de chrome nasce
sem dizer o que custa**, em percentagem de área de desenho, nos três alvos — e não só no maior.

**Medido (`medicoes/06`):** área de desenho com as duas colunas abertas —

| alvo | normal | **a pintar** | colunas fechadas |
|---|---:|---:|---:|
| iPad 12.9 (1366×1024) | 50,8 % | 50,8 % | 92,0 % |
| iPad 11 (1194×834) | 44,0 % | **40,8 %** | 90,2 % |
| iPad mini (1133×744) | 40,9 % | **37,6 %** | 89,0 % |

⛔⛔ **Medir só no alvo declarado esconde o problema:** o `tokens.json` declara `1366 × 1024`, que é
o **mais generoso** dos três. As duas colunas são `612 px` **absolutos** ⇒ `44,8 %` da largura no
12,9" e **`54,0 %`** no mini. *A mesma decisão de desenho custa 20 % mais no aparelho pequeno.*

⛔⛔ **E uma NOTA sobre o preço não trava nada.** A decisão D2 tinha escrito *«o preço que o Enio
aceitou: a barra global come uma faixa de altura permanente»* — e **uma segunda faixa foi
construída na semana seguinte** (o cabeçalho da área), custando `28 px` = `−1,5` ponto no alvo, e
foi revertida no mesmo dia.

⇒ a restrição virou gate: `the_chrome_never_eats_more_of_a_tablet_than_this` — piso por célula,
**tecto** de obsolescência, e as duas passagens do `frame_layout` (a altura da fila depende da
largura da área). Provado por mutação.

⭐ **E a maior alavanca não é cortar chrome — é RECOLHER:** fechar as duas colunas devolve `89–92 %`,
mais do que todas as faixas somadas valem.

**How to apply:** antes de propor uma banda, uma coluna ou um painel permanente, corra o gate e diga
**o que ela devolve**. Se a resposta é *«dá casa a N controlos»*, ela tem de caber onde já se paga
altura.

Relacionadas: [[feedback_the_ceiling_is_the_hardwares_never_the_fallbacks]] ·
[[feedback_a_rule_only_exists_if_it_is_on_the_path_of_who_executes_it]] ·
[[feedback_a_ratchet_without_a_staleness_census_only_ratchets_up]]
