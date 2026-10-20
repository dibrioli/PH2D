# Fixtures — a BATERIA do EXAME pintada pelo motor do MyPaint (libmypaint)

```
# cabecalho
#   motor        = libmypaint 1.6.1-2.1 (ISC)  — superficie MyPaintTiledSurface2 propria (pigmento espectral do MyPaint 2.x)
#   pinceis      = mypaint-brushes 2.0.2-2 (/usr/share/mypaint-data/2.0/brushes, 7 pacotes, 196 .myb)
#   bateria      = docs/Painter/Pinceis/ferramentas/bateria.json  sha256 cff23b71a54593d6118b2c1f5f7fe6cd5ed270f3094d53c5a5f3bfa741d914fa
#   tela         = 1000x640, camada transparente composta sobre papel branco (255,255,255), PNG RGB 8 bits sem perdas
#   tempo        = dt 0,010 s por evento: passo 3 px = 300 px/s (lento), passo 25 px = 2500 px/s (rapido)
#   caneta       = paira 50 eventos (0,5 s) de pressao 0 no 1.o ponto, pousa, segue os pontos, levanta com 1 evento de pressao 0
#   tamanho      = radius_logarithmic = ln(size_px/2); cor = color_h/s/v do RGB da bateria; inclinacao 0; zoom 1; rotacao 0
#   molhado/seco = o MyPaint nao tem: simula/seca sao no-ops e a foto A = B (grava-se uma so)
#   base (sobre) = as faixas vermelhas sao o MESMO pincel
#   compilador   = cc (GCC) 16.2.1, -O2; maquina = workstation do dono, 2026-10-19
```

**Comando** (regenera tudo, ~6 min 40 s num núcleo):

```
python3 -I docs/Painter/Pinceis/ferramentas/mypaint_bateria.py
python3 -I docs/Painter/Pinceis/ferramentas/mypaint_bateria.py --pacotes deevad          # um pacote
python3 -I docs/Painter/Pinceis/ferramentas/mypaint_bateria.py --pincel /usr/share/mypaint-data/2.0/brushes/classic/pen.myb
```

O motor corre em `ferramentas/mypaint_bateria.c` (compilado à 1.ª corrida para `~/.cache/ph2d-oraculos/`).
Determinístico: um fio só (`threadsafe_tile_requests = FALSE`), superfície própria zerada (a `fixed_tiled_surface`
nasce com lixo — `docs/_ComoInvestigarApps/01_o_arsenal.md` §2-bis). Escala medida: 1,0 = `1<<15` (o `assert` do
`.py` reprova outra).

**Saída:** `<pacote>/<nome>_<cena>.png`, cenas `gestos` · `tamanhos` · `sobre`; nomes com `#`, `%`, `(`… trocados por
`_`. `manifesto.json`: por pincel e cena, `px_com_tinta` (alfa > 1/255), `tinta_sum_alfa` (Σ alfa) e o sha256 (16).

| pacote | pincéis | licença (por artefacto) | portar? |
|---|---:|---|---|
| classic | 35 | CC0 (`/usr/share/licenses/mypaint-brushes/COPYING`) | ⭐ sim |
| deevad | 36 | CC0 + `readme.txt`: «released in public domain» | ⭐ sim |
| Dieterle | 19 | CC0 (pacote) | ⭐ sim |
| experimental | 23 | CC0 (pacote) | ⭐ sim |
| kaerhon_v1 | 20 | CC0 + `ReadMe.txt`: «licensed under the CC0» | ⭐ sim |
| ramon | 28 | CC0 (pacote) | ⭐ sim |
| tanda | 35 | CC0 (pacote) | ⭐ sim |
| **total** | **196** | o motor `libmypaint` é ISC; o app `mypaint` (GPL-2.0+) NÃO é tocado | |

0 falhas (os 196 carregam em `mypaint_brush_from_string`). Total 77 MB, sem quantizar (abaixo do tecto de 150 MB;
quantizar mudaria as métricas).

⚠️ **O que esta fixture NÃO mede:** inclinação/rotação da caneta (0), o «pousar em movimento» (a caneta pousa parada —
pincéis com raio por velocidade, como `classic/pen`, fazem uma bolinha no início que a mão real em movimento não faz),
o tempo de secagem (não existe no MyPaint) e a mistura com camadas de baixo (uma camada só).
