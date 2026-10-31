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
**Motor construído (ordem do coordenador, `c4564d3294`):** `DabJitter::VEU_OPACIDADE_MAX = 4` (o tecto do recurso:
a cobertura do dab satura em 1; `1 ÷ fluxo` mínimo 0,25); `stroke/spray.rs` multiplica e satura em 1 só quando
`o > 1` (a ≤ 1 a mesma conta, ao bit); o leitor do preset (`wetpaint_preset/json.rs`) abre o mesmo tecto (só esse
hunk comitado — o resto do ficheiro é do irmão). Portão red-first `marca::tests::o_veu_acima_de_um_e_a_populacao_escura`
(vermelho antes, verde depois; `o_veu_e_uma_segunda_populacao_maior_e_palida` continua verde). Sem painel/slider do véu.

| hoje + véu escuro (fluxo 0,4) | cob 200 | buracos | gestos escuro | pálido |
|---|---|---|---|---|
| G 0,2 × 0,3 · H 0,3 × 0,35 · I 0,15 × 0,25 (18 gotas) | 0,837 · 0,811 · 0,848 | ≤ 0,012 | 0,042 · 0,035 · 0,046 | 0,21 |
| J 0,3 × 0,55 | 0,834 | 0,009 | 0,043 | 0,204 |

**Recusa MEDIDA: a lei não escurece ESTE pincel.** A cobertura do dab é `strength × pressão` (`dab_build.rs:142`) e
o Spray Can tem opacidade 1: à pressão cheia o dab já está em 1, o véu × 2,5 satura, e o fluxo 0,4 aplica-se DEPOIS
(no composto) — os salpicos saem do tom das gotas. Prancha `r2_spray_can.png`: J ≈ antes. **Não promovido.**
O que falta (fila, 1 variante): pôr a tinta na `strength` e não no fluxo (opacidade 0,4, fluxo 1) para o véu × 2,5
chegar a 1 — muda a acumulação do traço (o tecto da Strength), medir antes; ou um `veu_fluxo` próprio no spray.
A lei fica (neutra, com portão); o coordenador decide se a mantém sem consumidor de fábrica.

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
