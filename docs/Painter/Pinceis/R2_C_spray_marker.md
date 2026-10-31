# R2-C (31/10) — Spray Can (J-2), Splatter (J-3), Dry Marker (E-4)

Sonda ignorada `familias::aerografo::sonda_r2c_spray_e_marcador` (`PH2D_EXAME_DIR`, `PH2D_R2C_SO=<prefixo>`): pinta
`gestos` e `tamanhos` (foto A) de cada variante à escala do exame (spray 0,975, marcador 0,744); variantes em
`SPRAY_R2C` (aerografo.rs) e `caligrafia_marcadores::variantes_r2c`. Medição: `cov = max(255−canal)/232`. ⚠️ Pintado
com a árvore partilhada a meio (edições dos irmãos R2-A/R2-B por comitar) — os números são indicativos. **Nada promovido,
nenhuma receita mudada**: 2 rondas por pincel, nenhuma variante claramente melhor nos DOIS eixos.

## J-2 Spray Can — o diagnóstico confere na foto
A 200 px: discos pálidos grandes soltos acima da faixa; nos gestos: faltam os salpicos ESCUROS (o Krita tem pontinhos
azul-escuros por cima da névoa). Prancha `pranchas/r2_spray_can.png` (antes | F | Krita).

| variante | cob 200 | buracos 200 | gestos escuro >0,9 | gestos pálido 0,1–0,45 |
|---|---|---|---|---|
| hoje (EM_ESTUDO) | 0,837 | 0,004 | 0,040 | 0,226 |
| A/B/C `particle_max_px` 9/6 (+`po`) | ralo (a 200 px some) | — | 0,032 · 0,033 · 0,021 | 0,24 |
| D/E véu ×3/×2,5 meio-pálido, fluxo 0,85/0,7 | 0,77 (E) | 0,004 | 0,023 (E) | 0,14 |
| **F** fluxo 0,75, 10 gotas, véu 0,35 ×0,3 opaco | 0,815 | 0,064 | **0,085** | 0,071 |
| Krita `y) Texture Spray` | 0,876 | 0,000 | 0,063 | 0,260 |

Recusas MEDIDAS: o tecto da gota em px (A–C) tira a densidade a 200 px; o jitter de opacidade 0,9 NÃO escurece (0,032);
o `po` não faz salpicos (só à borda); véu MAIOR (D/E) = exactamente os discos pálidos do veredito. F dá os salpicos
escuros mas perde a névoa (0,071 × 0,26) e abre buracos a 200 px.
**Necessidade de motor** (`stroke/spray.rs`, não proibido mas seria a 3.ª ronda): `veu_opacidade` está em
`clamp(0,1)` (linha 213) — a 2.ª população não pode ser MAIS escura que a 1.ª. Com `veu_opacidade` até ~2,5
(`coverage.min(1)`; neutro ao bit para ≤ 1), «hoje» + véu 0,2 ×0,3 opacidade 2,5 = névoa de hoje + salpicos escuros.
Simulação a fazer: essa variante única.

## J-3 Splatter
Não tocado (prazo); o diagnóstico J4 continua: orla de aguarela (dono `aguarela`) e lei de potência do tamanho.

## E-4 Dry Marker
Prancha `pranchas/r2_dry_marker.png` (Wet D | Digital EM_ESTUDO | Rebelle). Riscas do Rebelle a 200 px MEDIDAS:
período **8 px**, ±0,018 (as nossas: 0). Cobertura 4 · 26 · 71 px: Rebelle 0,49 · 0,62 · 0,63.

| variante | cob 3–4 · 26–29 · 71–103 px | riscas |
|---|---|---|
| Wet D (hoje) | 0,51 · 0,51 · 0,84 (101 px) | ruído 24 px, não riscas |
| Wet + grão `Stripes` Tiled 8/16 px, `grain_depth` 0,15/0,3 | = hoje ao bit (o Wet não lê o grão aqui) | nenhuma |
| Wet pigmento 115 + `spacing` 6/8 (dureza 0,9/1) | 0,66 · 0,60 · 0,89 | nenhuma (o `spacing` do Wet é inerte na foto) |
| **Digital EM_ESTUDO «Dry Marker»** | 0,30 · 0,57 · 0,76 (103 px) | nenhuma |

Achado: o **Digital** (Multiply, bisel 90°, fluxo 0,2) está visivelmente mais perto do Rebelle nos gestos que o Wet D
(liso, rampa cheia em cima, pontas rectas; o Wet sai granulado). Falta-lhe: riscas, o fio de 5 px fraco (0,30 × 0,49),
200 px largo (103 × 71). Candidato à cega/ao dono antes da fábrica.
**Necessidades de motor:** (1) CAP-11 glaze por traço com o carimbo visível — os dabs do bisel com a borda DURA ao longo do
traço a passo fixo (~0,11·L) somando dentro do tecto: as riscas de 8 px; (2) no Wet o grão (`texture`) não modula a
tinta deste caminho — conferir `wetpaint/pousa_o_dab.rs` (do irmão nesta onda).

Commits: ver `git log -- docs/Painter/Pinceis/R2_C_spray_marker.md`.
