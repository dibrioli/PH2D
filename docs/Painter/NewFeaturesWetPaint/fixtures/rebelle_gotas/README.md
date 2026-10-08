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
