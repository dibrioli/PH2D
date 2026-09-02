---
name: feedback-a-sliding-window-over-repeated-copies-needs-a-swept-number-not-a-derived-one
description: "Janela deslizante sobre cópias repetidas: a exigência não é monótona nem em n nem em count — varra, não derive"
metadata:
  node_type: memory
  type: feedback
---

Uma repetição por **dobra do domínio** (`opRepLim` e parentes) avalia a cópia do ponto e algumas
vizinhas. Essa janela **desliza com o ponto** — e se uma cópia de fora dela ainda puder ser a mais
próxima, o `min` troca de membros quando o índice salta e o campo **descontinua**. O que o artista
vê é a peça **estilhaçada**, com lascas soltas.

Caso medido (PH2D, `line/3DModeling`, 2026-08-30): `[Taper, Radial]` lia `‖∇f‖ = 730,5`, dívida
desde a W18.

⛔ **A derivação geométrica está errada, e a medição é que o diz.** A conta óbvia — meia-largura
angular `asin(R/d)`, e `π` quando a pegada contém o eixo — dá `count/2` para toda forma nascida na
origem, que é **toda** forma (a pilha corre em coordenadas locais, antes da pose). Custo: `79,4 ms`
por quadro num `radial 64`, contra `2 ms`.

E é conservador de mais:

| janela | `c=5` | `c=6` | `c=7` | `c=10` | `c=12` | `c≥16` |
|---|---:|---:|---:|---:|---:|---:|
| `n=1` | `561,6` | `730,5` | `1 327,5` | `1 198,7` | `3 684,7` | `0,47` |
| `n=2` | `0,68` | `0,69` | `736,3` | `1 562,0` | `10 698,9` | `0,64` |
| **`n=3`** | **`0,68`** | **`0,69`** | **`0,60`** | **`0,68`** | **`0,67`** | **`0,64`** |

**Why:** a exigência **não é monótona em `n`** (a `c=12` o `n=2` é pior que o `n=1`) nem em `count`
(acima de `16` as cópias ficam tão densas que a união é quase um sólido de revolução e qualquer
fatia responde o mesmo). Nenhuma fórmula fechada descreve isso; a varredura descreve.

**How to apply:**
- **Varra a faixa INTEIRA do parâmetro** (aqui `3..=64`, o `MAX` todo) e escolha o menor valor que
  a limpa toda — e depois **gateie a varredura**, não um ponto
  ([[feedback_a_ratio_gate_measured_at_the_degenerate_point_invents_a_debt_list]]).
- ⚠️ Um gate que mede **uma** contagem daria verde a metade da faixa: com `n=1` só `5,6,7,10,12`
  reprovam e `≥16` passa.
- ⚠️ **A grelha da sonda decide o que ela vê**: a mesma combinação lê `0,17` a `20³` e `245,77` a
  `40³` e `80³` ([[feedback_a_bucket_nobody_fills_reads_as_perfect]]).
- ⭐ E leve a **imagem** ao oráculo: numa forma côncava «fundo rodeado de peça» é o aspecto normal,
  então régua de silhueta não vê o estilhaço
  ([[feedback_an_oracle_that_shares_the_law_of_what_it_judges_is_a_mirror]]).
