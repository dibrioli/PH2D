# J6 — L3 / C38: a ponta que gira (28/10)

## §0 — Veredito

**Nada de motor a construir.** A premissa do C38 («a lâmina gira com o arrasto, com inércia e memória») está refutada
pela bateria ([06 §1, §10.1–10.2](06_LEIS_DAS_LACUNAS.md)) e volta a sê-lo nos cinco pincéis desta onda: os cinco
oráculos são FIXOS na tela, cada um no seu eixo. A rotação de barril (PEN_ROTATION do Rebelle) fica fora — decisão do
dono 08/10 (`PB/pen.rs`), e sem barril o PEN_ROTATION do Rebelle É a lâmina fixa. A lei que o Rebelle faz, preset a
preset, já se exprime: `dab_flatten` × `dab_angle_deg` × `dabRotation` (0 fixa · 1 segue · 2 inclinação). O que faltava
era a lâmina CERTA em cada preset e um portão que a prenda.

## §1 — A medição (régua `leis_lacunas.py lamina`, cena `gestos`, foto A)

| pincel | oráculo (modo) | w0 · w90 · S+40 · S−40 | φ · L × T | rms fixa / const | nós antes (escala 1) |
|---|---|---|---|---|---|
| A-4 Flat Block-in | Rb Pencil/Digital/Flat (PEN_ROTATION) | 39 · 17 · 36 · 36 | 84° · 37,4 × 12,4 | 1,0 / 7,4 | 38 · 16 · 36 · 36, 84° — igual |
| B-9 Craft Flat | Rb Graphite/Craft Flat (PEN_TILT) | 21 · 30 · 25 · 25 | 10° · 24,3 × 17,1 | 2,9 / 3,5 (fraca) | 29 · 41 · 35 · 35 (w90/w0 1,41 × 1,43) |
| D-3 Broad Nib | MyPaint classic/calligraphy | 36 · 34 · 12 · 49 | 48° · 47,9 × 2,3 | 1,3 / 10,1 | 40 · 36 · 12 · 53, 48° — igual |
| D-3 (Rb Fountain Square) | PEN_ROTATION | 25 · 19 · 32 · 32 | 90° · 25,1 × 19,3 | 0,7 / 4,2 | não é bico chato (quase quadrado) |
| F-2 Compressed Charcoal | Rb Charcoal/Charcoal Flat (PEN_TILT) | 16 · 37 · 28 · 28 | 4° · 33,4 × 12,7 | 2,4 / 7,4 | à escala do oráculo: w90/w0 **1,88** (0,55) |
| I-8 Bristle Watercolor | Rb Watercolor/Bristle Flat (PEN_ROTATION) | 46 · 16 · 41 · 37 | 100° · 45,2 × 7,6 | 1,2 / 10,2 | w90/w0 **0,44**, T/L 0,27 (0,65) |

Arranque da vertical (memória): 0,875–1,00 nos oráculos, 1,00 nos nossos. Oil/*: [`J6_lamina_oil_receitas.md`](J6_lamina_oil_receitas.md).

## §2 — As curas (só o eixo da lâmina; ronda 1)

- **I-8 Bristle Watercolor** `achatado` 0,65 → **0,75** (variantes 0,65/0,75/0,80/0,85): w90/w0 0,44 → **0,36**
  (Rebelle 0,35), S/w0 0,94 → 0,91 (0,89), T/L 0,27 → 0,21 (0,17; 0,80 dá 0,16 mas S cai a 0,81).
- **F-2 Compressed Charcoal** `achatado` 0,55 → **0,62** (à escala do oráculo, 0,38): w90/w0 1,88 → **2,31** (Rebelle
  2,31); S/w0 1,25 → 1,38 (1,75 — o S do nosso fica abaixo do da lâmina ajustada: o halo de pó). ⚠️ Medido no portão
  sem a tabela da largura: o halo soma ~10 px em QUALQUER direcção (a SEGUIR o traço lê 1,6–1,8 em vez de 1), então a
  razão depende da escala (2,50 a 16 px, 1,50 a 24) — fica fora do portão, preso pela régua do exame.
- A-4, B-9, D-3: a lâmina já era a do oráculo — sem mudança.

## §3 — Portão

`familias::aguarela::lamina::a_lamina_de_cada_preset_e_a_do_oraculo` — quatro rectos (0°, 90°, ±40°) a 40 px por
preset; w90/w0 = o lido no oráculo ± 15 %, e ±40° = os da lâmina ajustada onde o ajuste é justo (Flat Block-in, Broad
Nib). Controlo por dentro: a mesma ponta com `dabRotation` 1 tem de REPROVAR (lê 0,97–1,07). Verde: Flat Block-in
0,421 (0,436) · Craft Flat 1,517 (1,429) · Broad Nib 0,929 · 0,232 · 1,321 (0,944 · 0,241 · 1,291) · Bristle Watercolor
0,323 (0,348). Mutação vista VERMELHA: Bristle Watercolor 0,75 → 0,65 (lê 0,419) e Broad Nib a 132° (os ±40 trocam).
Sem knob novo ⇒ os censos dos knobs não mudam; nenhum ficheiro de fábrica mudou (as curas são de `EM_ESTUDO`).

## §4 — Fila

- **L16 (lâminas finas, C40)** — já não depende de nada (a L3 fecha sem motor): Cv da borda 0,004 → 0,056 a 4,8–6,0 px
  (grelha do Wet; `03_MOTORES` L16). Pincéis G-7, D-3, G-14. Não medido de novo nesta onda.
- O halo de pó do carvão engorda o lado fino da barra em px fixos (F-2): a lâmina do Rebelle não o tem.
