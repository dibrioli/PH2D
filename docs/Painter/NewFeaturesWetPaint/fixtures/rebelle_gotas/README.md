# Fixtures — a bancada das GOTAS do Rebelle 8.3.4 (2026-10-08)

Saída do oráculo sobre gestos NOSSOS ([doc 06](../../06_gotas_rebelle_x_ph2d.md)); nenhum asset do
Rebelle. Gerador: [`ferramentas/gotas.py`](../../ferramentas/gotas.py), enviado pela porta WebSocket
(`ws_eventos.mjs`); foto da janela (`foto.sh`), recortada à tela e reduzida a 40 %.

| ficheiro | tela | inclinação (`SET_CANVAS_TILT`) | gotas (`SET_ENGINE_PARAMS`) | colunas |
|---|---|---|---|---|
| `plano.png` | 1000×640 branca, `Default/EX00 White Simple`, semente 7 | desligada | on, 5/5 | Watercolor/Round, água 100 · 70 · 40 · 20 |
| `inclinado_2s.png` · `inclinado.png` | idem | (0, 1) ligada | on, 5/5 | idem |
| `meio.png` | idem | (0, 0,5) | on, 5/5 | idem |
| `sem_gotas.png` | idem | (0, 1) | **off** | idem |
| `gotas_max.png` | idem | (0, 1) | on, **10/10** | idem |
| `pinceis.png` | idem | (0, 1) | on, 5/5 | Round · Wash Round · Mop (WATERCOLOR) · Dip Pen (INK_PEN), cada um com a água DO PRESET |
| `s_inclinado.png` | idem, **1:1** (zoom 100 %, sem redução) | (0, 1) ligada | on, 5/5 | **UM S** (`gotas.py --forma s`): Watercolor/Round, `size_px` 100, água 100, pressão 1 — 2.ª corrida da sessão |
| `s_inclinado_1.png` | idem, 1:1 | (0, 1) ligada | on, 5/5 | o mesmo S, 1.ª corrida — ⚠️ o painel do pincel mostrou Water **80** / Size 25 depois dela (as outras três: 100 / 11) |
| `s_sem_gotas.png` | idem, 1:1 | (0, 1) | **off** | o mesmo S |
| `s_plano.png` | idem, 1:1 | desligada | on, 5/5 | o mesmo S |
| `s_zoom100.png` | idem, 1:1 (09/10, auditoria da W1c) | (0, 1) ligada | on, 5/5 | o mesmo S, 1.ª corrida de uma sessão nova (o painel: Size 47, Opacity 60) — `gotas_tela.sh az_s --forma s --size_px 100 --agua 100 --tilt 0 1`, foto `_c` |
| `s_zoom400.png` | a MESMA tela a **400 %** (campo de zoom do Navigator), a vista inteira da janela (1470×1024) | idem | idem | a descida e as raízes, como o dono olha: o ecrã do Rebelle amplia SUAVE (sem blocos), a orla é pálida e com meio-tom, a raiz é um funil |

Traço: 150 px horizontal em y = 150, `size_px` 40, pressão 1, azul `#1765b3`, `POINTER_MOVE` a 3 px.
Tempos: `_2s` ≈ 2 s depois do último traço; os outros depois de `SIMULATION` +400 e +1 200 passos
(≈ 1 min de água). Gota mais longa abaixo do traço (px da tela; régua `255 − min(rgb) > 25`):

| tela | água 100 | 70 | 40 | 20 |
|---|---|---|---|---|
| plano | 0 | 0 | 0 | 0 |
| inclinado | 283 | 249 | 0 | 0 |
| meio | 0 | 0 | 0 | 0 |
| sem_gotas | 0 | 0 | 0 | 0 |
| gotas_max | 463 | 465 | 0 | 0 |
| pinceis (água do preset) | Round 302 | Wash Round 0 | Mop 0 | Dip Pen 0 |

**O S (08/10, pedido do dono: a orla de baixo de uma CURVA com a tela inclinada).** `y = 180 − 85·sin θ`,
x 100 → 900, um ponto a cada 3 px de arco; foto `_z` depois de `SIMULATION` +400 +1 200, com o
zoom pedido a 100 % pelo campo do Navigator (`inj.sh "click 1852 266; …type 100…"`), recorte
`1000x640+480+220` sem redução. Um `DRY` depois da 2.ª corrida mudou 1 px (máx. 10). As posições das
gotas NÃO se repetem com a semente 7 (as duas corridas inclinadas diferem em 53 k px > 10). Na descida
(x 440–632, declive 0,72–0,75), régua `255 − min(rgb)`:

| | plano | inclinado (2.ª · 1.ª) | sem gotas |
|---|---|---|---|
| orla de baixo, salto máx. entre colunas | 5 px | 4 · 7 px | 9 px |
| orla de baixo, resíduo rms (cúbica) | 0,93 | 1,10 · 1,91 | 1,88 |
| corpo do traço (escuridão) | ~101 | 17–38 | 15–24 |
| pico na orla (mediana na descida) | ≈ corpo (+7 a +20, linha de 1–3 px) | 102 · 60 | 46 |
| vale (x 682): corpo → pico, rampa 90→10 % | 100 → 102 | 33 → 127 · 20 → 146, 35–37 px | 20 → 166, 37 px |
| tinta total Σ(escuridão), tarde ÷ cedo (~1,5 s) | 93 % | 51 · 44 % | 27 % |
| orla de baixo, tarde − cedo (mesma tela, mediana) | 0 | −2 · −3 px | −3 px |
| gotas | 0 | 11 · 10 | 0 |

Gotas: em TODA a orla de baixo (as duas subidas, a descida, o arco, o vale, a ponta baixa), não só nos
pontos baixos; 3 px de largura a meia altura (3–7), 228–430 px de comprimento, escuridão mediana 27–187
contra o corpo 17–29.
