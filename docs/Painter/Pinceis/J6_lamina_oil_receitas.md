# J6 — a lâmina dos Oil/* (receitas prontas para o agente do Real Oil)

Lei medida (L3, [`J6_lamina.md`](J6_lamina.md)): **nenhum preset chato do Rebelle gira com inércia nem guarda o ângulo
entre traços**; os PEN_ROTATION/PEN_TILT são FIXOS na tela, cada um no seu eixo; os FOLLOW_TRAJECTORY seguem sem atraso
visível (λ ≤ 0,3·D, o Follow de hoje cabe). Motor: nada a construir — `dab_flatten` × `dab_angle_deg` ×
`dabRotation` (0 fixa · 1 segue) já exprimem os dois. Barril: fora (dono 08/10; sem barril o Rebelle é o fixo).

Ruler: `python3 -I docs/Painter/Pinceis/ferramentas/leis_lacunas.py lamina <foto gestos_A>` — a mesma conta no nosso
(`exame_pinta`, cena `gestos`). Ajuste fixo `w(θ) = L|sin(θ−φ)| + T|cos(θ−φ)|`; FIXA = rms do ajuste ≤ ⅓ do da constante.

| Oil (Rebelle, modo) | w0 · w90 · S+40 · S−40 (px a «40») | φ · L × T · T/L | rms fixa/const | o nosso hoje | receita |
|---|---|---|---|---|---|
| Flat Oily (PEN_ROTATION, force) | 40 · 11 · 35 · 35 | 86° · 39,3 × 7,8 · 0,20 | 0,8 / 9,7 | sem receita (P53 saiu) | nasce do Flat Bristle: `achatado (0,80, 86)`, `dabRotation 0`, `dab_quina` 1 |
| Flat (Oil, PEN_ROTATION) → G-6 Flat Bristle | 40 · 13 · 35 · 36 | 84° · 39,0 × 8,5 · 0,22 | 0,8 / 9,0 | `achatado (0,73, 90)` | `achatado (0,78, 84)`, `dabRotation 0` |
| Knife Oily (G-7) | 14 · 34 · 34 · 23 | 172° · 33,0 × 10,3 · 0,31 | 1,8 / 7,9 | `(0,69, 172)`, fixa ✓ | fica (a assimetria S+40 34 × S−40 23 é a do eixo 172°: confira-a) |
| Fan (Dabber/Fan, G-8) | 14 · 33 · 23 · 23 | 6° · 29,1 × 9,8 · 0,34 | 3,0 / 6,9 (fraca) | `(0,78, 0)` | eixo 6° (`(0,78, 6)`); o ajuste é fraco — as riscas do leque, não a pegada, decidem |
| Filbert (G-5, aprovado) | 40 · 40 · 41 · 41 | constante | 2,1 / 0,4 | `(0,0, 90)` | fica (redondo: o Rebelle não o achata) |
| Flat Dry (FOLLOW) | 40 · 41 · 41 · 41 | constante | 2,3 / 0,7 | — | seguidora: `dabRotation 1` |

`dab_flatten` → T/L medido (Wet, cerdas, à escala 1): 0,65 → 0,27 · 0,75 → 0,21 · 0,80 → 0,16 · 0,85 → 0,17 (Bristle
Watercolor); a borda macia e o pó engordam o lado fino (o Compressed Charcoal: +~10 px de halo em qualquer direcção).
Calibre cada um com uma linha nova em `familias/aguarela/lamina.rs` (`ORACULO`: w90/w0 lido e, se o ajuste fixo for
justo — rms ≤ 1,3 —, os ±40° da lâmina ajustada); o controlo por dentro (`dabRotation` 1) tem de reprovar.
