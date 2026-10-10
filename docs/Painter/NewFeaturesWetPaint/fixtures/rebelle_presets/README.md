# Fixtures — quatro PINCÉIS de aguarela do Rebelle 8.3.4, plano × inclinado (2026-10-10)

Saída do oráculo sobre gestos NOSSOS ([doc 06](../../06_gotas_rebelle_x_ph2d.md)); nenhum asset do
Rebelle. Gerador [`ferramentas/gotas.py`](../../ferramentas/gotas.py) pela porta WebSocket
(`ws_eventos.mjs`), sessão aberta por `gotas_inicia.sh` + Registration (Try → Pro) + New Artwork cancelado.
Foto da janela (`foto.sh`) a **zoom 100 %** (Navigator: `inj.sh "click 1853 273; … type 100; … key enter"`),
recortada à tela `-crop 1000x640+480+220`, **1:1**. A posição da tela foi conferida na foto
(x 480..1479, y 220..859 = 1000×640).

```
gotas.py pres_plano --sem_tilt  --sem_motor --colunas C1 C2 C3 C4
gotas.py pres_incl  --tilt 0 1  --sem_motor --colunas C1 C2 C3 C4
C1 = "WATERCOLOR|Watercolor/Round|"       C2 = "WATERCOLOR|Watercolor/Wash Round|"
C3 = "WATERCOLOR|Watercolor/Flat Dry|"    C4 = "WATERCOLOR|Watercolor/Round|100"
```

Cada tela é um `NEW_ARTWORK` próprio: 1000×640, `Default/EX00 White Simple`, semente 7, sem
`SET_ENGINE_PARAMS` (gotas na omissão do Rebelle). Runner = `gotas_tela.sh` com uma diferença: o
`NEW_ARTWORK` + `SET_CANVAS_TILT` vão primeiro, depois o clique do zoom, depois os traços — assim a foto
`_2s` sai ≈ 2,1 s depois do início dos traços (0,4 s de envio + 1,5 s de espera).

Traço `i` (0..3): horizontal de `x0 = 60 + 240·i` a `x0 + 150`, `y = 150`, `size_px` 40, pressão 1,
azul `#1765b3`, `POINTER_MOVE` a 3 px. Água: a do PRESET salvo nas colunas 1–3; **100** na 4.ª.

| ficheiro | tela | tempo |
|---|---|---|
| `plano_2s.png` · `plano.png` | inclinação desligada | ≈ 2 s · depois de `SIMULATION` +400 e +1 200 |
| `inclinado_2s.png` · `inclinado.png` | `SET_CANVAS_TILT` (0, 1) ligada | idem |

Extensão vertical da tinta (px da tela; régua `255 − min(rgb) > 25`; gota = abaixo de y 170):

| tela | Round | Wash Round | Flat Dry | Round, água 100 |
|---|---|---|---|---|
| plano | 135..167 | 129..169 | 134..167 | 134..167 |
| inclinado | 136..510 (gota 340) | **144..155** | 134..167 | 136..514 (gota 344) |

⚠️ O Wash Round na tela inclinada sai com ~12 px de altura (41 na plana) já aos 2 s — o mesmo
`SET_BRUSH`; causa não investigada.
