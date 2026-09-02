---
name: feedback-loose-bounds-that-each-look-fine-multiply-into-a-catastrophe
description: "Três bounds frouxos por 1,3×–2,5× compõem-se em 24,7× — meça a FOLGA (cobrado ÷ preciso), não cada factor"
metadata: 
  node_type: memory
  type: feedback
  originSessionId: eed39e8c-c3cb-4514-a6c1-5e9da25f6c30
  modified: 2026-09-01T00:57:16.128Z
---

Quando bounds conservadores se **compõem por multiplicação**, cada um pode parecer razoável e o
produto ser catastrófico. A régua não é *«este factor está certo?»* — é a **FOLGA**: `cobrado ÷
realmente preciso`, medida na composição.

Caso medido (PH2D, `line/3DModeling`, 2026-08-31, report do Enio *«algumas combinações muito
lentas»*):

| pilha | cobrado | preciso | folga | passos/raio |
|---|---:|---:|---:|---:|
| `[]` | `1,00` | `1,0` | `1,0×` | `7,1` |
| `[Bend]` | `10,00` | `4,0` | `2,5×` | `72,2` |
| `[Bend, Twist]` | `33,82` | `7,3` | `4,6×` | `233,1` |
| `[Bend, Twist, Taper]` | `240,29` | `9,9` | **`24,7×`** | **`1 543,6`** |

Nenhum dos três está errado sozinho. **217× o custo do caso base**, e `24,7×` é desperdício provado.

**How to apply:**
- ⭐ **A régua da folga é `bound × medida_real`**: se o consumidor exige `‖∇f‖ ≤ 1` e a peça mede
  `0,041`, o bound é `24×` maior do que precisa. *Um bound sem essa razão ao lado é um número que
  ninguém sabe se é apertado.*
- ⛔⛔ **A cura «por região» tem de ser MEDIDA antes de construída.** Aqui a hipótese natural era que
  o pior caso vivia num canto e que especializar por ladrilho o mataria — e o pior **oitavo** media
  `0,0416` contra `0,0405` da caixa inteira: *o desperdício era uniforme, e cortar o domínio não
  compra nada.* A sonda custou minutos e poupou uma wave inteira
  ([[feedback_measure_the_defects_structure_before_designing_its_cure]]).
- ⚠️ A mesma sonda achou um **bloqueador escondido** no caminho que ia ser construído (a
  especialização por região estava desligada para documentos sem perfil). *Meça a cura no ponto onde
  ela entraria, não em abstracto.*
- ⭐ **Use uma CONTAGEM e não um relógio** para custo, sempre que houver uma: `passos ÷ raios` é
  determinístico e imune à carga da máquina, ao contrário de `ms`
  ([[reference_topic_gate_discipline]]). Aqui a carga estava em `8,39` e nenhum relógio valia nada.
- ⇒ quando a folga é **uniforme**, a cura é apertar o bound (matemática), não restringir o domínio
  (engenharia).
