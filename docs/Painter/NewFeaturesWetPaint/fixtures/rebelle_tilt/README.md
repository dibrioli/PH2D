# O tilt do lápis no Rebelle 8.3.4 (sessão de 2026-10-18)

Saída do Rebelle 8.3.4 (teste, sob Wine, porta WebSocket 8265) sobre eventos NOSSOS — nenhuma textura, preset ou
ficheiro dele copiado. Recorte 1000×640 da janela a zoom 100 % (`SAVE` não existe por WebSocket), ~2 s depois do
último Up, sem `SIMULATION` (o lápis é seco). Gerado por `ferramentas/tilt_lapis_rebelle.sh` +
`ferramentas/tilt_lapis_rebelle.py` (eventos e régua); a régua é `ferramentas/mede_traco.py`, medida SÓ sobre estas
fixtures (`medidas.json` tem tudo, inclusive o perfil da largura de C, E, F, G).

Comum a todas: PENCIL · `Graphite Pencil/8B` · size_px 40 (slider 63,0 de Max 80) · `pressure` 100 no SET_BRUSH ·
cor (23, 101, 179) · semente 7 · papel `Default/EX00 White Simple` · tela 1000×640 · traço recto horizontal x 60 → 560,
um evento a cada 3 px (salvo `_6px`), pressão 1,0 (salvo `rampa_p`: p = 0,15 + 0,85·sin πf) · SET_BRUSH completo
antes de CADA traço · eventos enviados a 2 ms (salvo F: 20 ms). Os mesmos pincel/papel/semente/cor de
`../rebelle/penciltilt.png`. `pen_tilt` = `{"x": graus, "y": graus}` em todos os eventos de ponteiro; o traço corre
em +x, logo **(t, 0) = ao longo** do eixo e **(0, t) = através**.

| ficheiro | linhas (y → pen_tilt) | eventos (sha256-16 de `<bloco>_0_1.json`) | sha256 (16) |
|---|---|---|---|
| `tilt_lapis_A.png` | y 45+75·i → (0,0) (20,0) (35,0) (45,0) (50,0) (55,0) (60,0) (63,0) | `640a2aafe4e46996` | `45f97dd9a6a38661` |
| `tilt_lapis_B.png` | y 80+160·i → (0,0) (0,35) (0,40) (0,45) | `2e12647ff03ffbf2` | `2234939e8b80ecef` |
| `tilt_lapis_C.png` | 80 rampa_p a (55,0) · 240 rampa_p a (0,0) · 400 tilt (63f, 0) · 560 tilt (0, 63f) | `a6e9eb5648497335` | `f408a1cd23f1373d` |
| `tilt_lapis_D.png` | y 80+160·i → (0,50) (0,55) (0,63) (39,39) | `2c13b0bc1deafe14` | `b788960ef176a4c8` |
| `tilt_lapis_E.png` | 80 tilt (63(1−f), 0) · 240 (0, 63(1−f)) · 400 rampa_p a (0,55) · 560 PRESS a (0,0), o resto a (0,55) | `0a47cfd49e5e978f` | `bc87985cd5c105f5` |
| `tilt_lapis_F.png` | as rampas de C/E a **20 ms/evento**: 80 (0, 63f) · 240 (0, 63(1−f)) · 400 degraus de 9° (0, 9·⌊8f⌋) · 560 (63f, 0) | `7e527dea701b40ba` | `f2bbb7ab2e54bc5e` |
| `tilt_lapis_G.png` | 80 degrau (0,0)→(0,55) em x 200 · 240 degrau (0,55)→(0,0) em x 200 · 400 o degrau 0→55 a **6 px/evento** · 560 (0, 63(1−f)) a 6 px/evento | `3c6e283cb60e12ad` | `6f886e4893ed857e` |

⚠️ 1.ª corrida: B tinha 6 traços a 100 px de passo e o através a 55° e 63° (~95 px de largura) **sobrepôs-se**; B foi
repintado a 160 px de passo e o resto do através passou para D. A, C ficaram da 1.ª corrida (não mudaram).

## Medidas (`mede_traco.py`, recorte `0,y−m,660,2m`, m = 37 em A e 75 nos outros)

| bloco | traço | larg. mediana | larg. 5/50/95 % | textura % | borda esc. | cob. miolo | orla 10–90 | comprimento |
|---|---|---|---|---|---|---|---|---|
| A | (0,0) | 29 | 31/28/28 | 8.5 | 0.31 | 0.796 | 7 | 533 |
| A | (20,0) | 29 | 25/29/32 | 9.3 | 0.30 | 0.794 | 7 | 528 |
| A | (35,0) | 31 | 32/27/25 | 8.8 | 0.30 | 0.800 | 7 | 537 |
| A | (45,0) | 36 | 22/39/23 | 5.8 | 0.31 | 0.838 | 6 | 616 |
| A | (50,0) | 36 | 24/37/20 | 5.4 | 0.32 | 0.849 | 6 | 619 |
| A | (55,0) | 36 | 28/37/19 | 5.5 | 0.31 | 0.844 | 6 | 612 |
| A | (60,0) | 36 | 33/36/20 | 5.7 | 0.31 | 0.844 | 6 | 606 |
| A | (63,0) | 35 | 28/34/26 | 5.5 | 0.32 | 0.848 | 6 | 609 |
| B | (0,0) | 30 | 28/30/31 | 8.8 | 0.31 | 0.799 | 7 | 529 |
| B | (0,35) | 33 | 31/37/33 | 9.2 | 0.30 | 0.789 | 8 | 529 |
| B | (0,40) | 42 | 40/44/38 | 9.0 | 0.26 | 0.785 | 9 | 528 |
| B | (0,45) | 95 | 99/109/89 | 8.6 | 0.21 | 0.764 | 19 | 533 |
| D | (0,50) | 98 | 94/101/92 | 8.5 | 0.22 | 0.764 | 20 | 530 |
| D | (0,55) | 100 | 93/110/95 | 8.6 | 0.21 | 0.764 | 20 | 529 |
| D | (0,63) | 97 | 92/99/92 | 8.5 | 0.22 | 0.764 | 20 | 528 |
| D | (39,39) | 75 | 38/76/50 | 7.3 | 0.22 | 0.782 | 13 | 588 |
| C | rampa_p a (55,0) | 35 | 18/36/15 | 5.6 | 0.32 | 0.801 | 6 | 492 |
| C | rampa_p a (0,0) | 29 | 17/28/14 | 9.9 | 0.29 | 0.727 | 8 | 433 |
| C | tilt (63f, 0) | 30 | 32/25/32 | 9.1 | 0.31 | 0.799 | 7 | 538 |
| C | tilt (0, 63f) | 30 | 29/37/41 | 9.1 | 0.30 | 0.795 | 7 | 529 |
| E | tilt (63(1−f), 0) | 35 | 22/36/28 | 6.3 | 0.32 | 0.838 | 6 | 572 |
| E | tilt (0, 63(1−f)) | 92 | 93/110/30 | 8.6 | 0.24 | 0.770 | 17 | 529 |
| E | rampa_p a (0,55) | 91 | 59/87/60 | 9.4 | 0.21 | 0.694 | 22 | 435 |
| E | PRESS (0,0), resto (0,55) | 97 | 22/94/107 | 8.7 | 0.21 | 0.763 | 19 | 567 |
| F | (0, 63f) a 20 ms | 30 | 28/30/38 | 8.8 | 0.31 | 0.797 | 7 | 529 |
| F | (0, 63(1−f)) a 20 ms | 92 | 93/110/30 | 8.6 | 0.24 | 0.770 | 17 | 529 |
| F | degraus 9° através | 31 | 29/34/39 | 9.2 | 0.30 | 0.793 | 7 | 528 |
| F | (63f, 0) a 20 ms | 30 | 31/37/31 | 9.2 | 0.30 | 0.797 | 7 | 543 |
| G | degrau 0→55 em x 200 | 32 | 28/31/92 | 8.8 | 0.29 | 0.783 | 8 | 530 |
| G | degrau 55→0 em x 200 | 34 | 93/35/30 | 8.8 | 0.28 | 0.778 | 8 | 529 |
| G | degrau 0→55, 6 px/evento | 32 | 29/35/92 | 9.2 | 0.28 | 0.779 | 8 | 528 |
| G | (0, 63(1−f)), 6 px/evento | 90 | 100/112/33 | 8.6 | 0.24 | 0.773 | 16 | 533 |

Perfil da largura (px a meia altura, mediana em ±6 colunas, a cada 25 px de x = 60, 85, …, 560):

| traço | larguras |
|---|---|
| C rampa_p a (55,0) | 0 18 22 28 32 35 37 38 35 37 37 37 39 38 35 36 34 28 20 18 0 |
| C rampa_p a (0,0) | 0 0 17 20 24 30 34 33 35 30 29 31 31 29 30 31 26 21 17 0 0 |
| C tilt (63f, 0) | 23 28 28 30 35 29 38 30 27 33 27 35 31 29 30 31 26 27 31 33 31 |
| C tilt (0, 63f) | 30 30 28 30 31 31 30 30 34 26 35 26 35 30 30 30 30 31 34 42 43 |
| E tilt (63(1−f), 0) | 32 35 36 39 39 36 37 38 38 36 36 36 35 37 30 33 31 28 29 28 25 |
| E tilt (0, 63(1−f)) | 91 96 90 106 99 96 100 101 95 102 109 101 74 52 43 36 32 32 30 35 27 |
| E rampa_p a (0,55) | 0 0 60 68 79 90 102 93 109 103 91 97 101 103 93 95 84 74 56 0 0 |
| E PRESS (0,0), resto (0,55) | 66 105 96 89 100 103 99 99 100 106 95 109 86 96 108 99 96 99 97 95 80 |
| F (0, 63f) a 20 ms | 24 32 28 32 31 30 30 32 31 30 31 28 32 32 27 31 31 28 32 36 42 |
| F (0, 63(1−f)) a 20 ms | 91 96 90 106 99 96 100 101 95 102 109 101 74 52 43 36 32 32 30 35 27 |
| F degraus 9° através | 29 31 29 33 29 29 27 31 34 34 34 27 29 31 32 30 29 30 35 36 43 |
| F (63f, 0) a 20 ms | 29 32 30 27 28 33 30 31 31 31 33 29 31 27 29 32 29 27 29 32 30 |
| G degrau 0→55 em x 200 | 24 32 28 32 31 30 30 32 31 30 31 28 32 33 32 40 43 51 76 96 101 |
| G degrau 55→0 em x 200 | 91 96 90 106 99 96 45 37 38 33 33 29 30 32 31 31 30 32 30 35 27 |
| G degrau 0→55, 6 px/evento | 29 31 29 33 29 29 27 30 34 34 35 27 29 31 36 40 44 53 73 92 92 |
| G (0, 63(1−f)), 6 px/evento | 98 99 99 92 96 98 101 107 95 105 108 93 74 42 37 38 29 27 28 31 26 |

## O que as medidas dizem

- **Zona morta: até ~35–40°, nas duas direcções.** Ao longo: 0/20/35° = 29/29/31 px (controlo 29–30); a 45° salta para
  36 e satura (45–63° = 35–36). Através: 35° = 33, 40° = 42, 45° = 95 e satura (50–63° = 97–100, ~3,3× o controlo).
- **Anisotropia.** AO LONGO o tilt ALONGA o carimbo (comprimento 533 → 606–619: ~40 px a mais em cada ponta), alarga só
  +20 % e ALISA (textura 8,5–9,3 → 5,4–5,8 %, cobertura 0,80 → 0,84–0,85, orla 7 → 6). ATRAVÉS ele alarga 3,3×, a
  textura fica (8,5–9,2 %), a cobertura cai um pouco (0,76), a orla fica mole e rasgada (7 → 20) e a borda escura cai
  (0,30 → 0,21). A diagonal (39,39) (módulo 55°) = 75 px, pontas em paralelogramo.
- **O tilt é FILTRADO ao longo do traço** — por isso as rampas não desenham a curva: a subida 0→63° fica estreita até à
  última ponta (~42 px em x 535–560), a descida 63→0° fica larga até a entrada valer ~25° (x ≈ 360). O filtro NÃO é no
  tempo (F a 20 ms/evento sai BYTE A BYTE igual a E a 2 ms) e NÃO é por evento (G a 6 px/evento demora os mesmos px que
  a 3 px): é por DISTÂNCIA. Uma média exponencial em px com λ ≈ 170 px, que arranca no 1.º MOVE (o PRESS a (0,0) seguido
  de (0,55) alarga logo), explica a subida (prevê 42,6° no fim), a descida (prevê 43° em x 360) e o degrau 0→55 (prevê
  o limiar ~260 px depois: medido ~235–260). ⚠️ NÃO explica o degrau 55→0, que estreita em ≤ 15 px (a média prevê ~40):
  a descida rápida a 0 corta logo, a lenta não — por medir (degrau 55 → 30, 55 → 20).
- **Pressão** só afina as pontas (rampa a (0,55): 60 → 109 → 56 px), a largura de tilt escala com ela.
- Bate com `../rebelle/penciltilt.png` (55,0): 37 × 30 px, textura 5,2 × 9,1 %; aqui 36 × 29–30, 5,5 × 8,5–8,8 %.

## Reproduzir

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor/docs/Painter/NewFeaturesWetPaint/ferramentas
bash gotas_inicia.sh &                       # tela virtual 1920×1080 + Rebelle com a porta 8265 (prazo 1 h)
# passar Registration (Try → Pro) e New Artwork — as duas primeiras etapas de ~/Apps/rebelle/oraculo/arranque.sh:
#   inj.sh "click 1567 1049; wait 4000; click 1355 1049; wait 1000"  ·  inj.sh "click 1407 790; wait 6000"
bash tilt_lapis_rebelle.sh                    # as 7 telas, as fotos recortadas aqui e a régua
TILT_BLOCOS="B D E" bash tilt_lapis_rebelle.sh   # só esses blocos (a régua mede sempre todos)
python3 -I tilt_lapis_rebelle.py regua ../fixtures/rebelle_tilt   # só a régua
bash gotas_fecha.sh                           # fecha SÓ esta tela virtual (PID conferido) + wineserver -k
```
