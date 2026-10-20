# Fotos do Rebelle 8 sobre a BATERIA do EXAME — os 281 presets de fábrica (2026-10-19/20)

> Oráculo: **Rebelle 8.3.4 Pro, versão de TESTE** («Trial (18 days left)» a 19/10 ⇒ expira ~06/11), sob Wine 11,
> prefixo `~/Apps/rebelle/wineprefix`, tela virtual `kwin_wayland --virtual` 1920×1080, porta WebSocket (Motion IO).
> Corrida: 2026-10-19 22:43 → 2026-10-20 00:10, cinco sessões de ≤ 30 min com a vez da placa (`PH2D_GPU=1`), fecho
> só por `gotas_fecha.sh`. **766 cenas · 1 532 fotos · 0 falhas · 0 diálogos de erro.**
> Bateria: [`../../ferramentas/bateria.json`](../../ferramentas/bateria.json) **versão 1**, sha256
> `cff23b71a54593d6118b2c1f5f7fe6cd5ed270f3094d53c5a5f3bfa741d914fa` (gerada por `bateria.py`, commit `4eaee9d72c`).

## O comando (da raiz da árvore)

```bash
python3 -I docs/Painter/Pinceis/ferramentas/lista_rebelle.py > docs/Painter/Pinceis/ferramentas/lista_281.txt
timeout 21600 bash docs/Painter/Pinceis/ferramentas/lote_bateria.sh \
  docs/Painter/Pinceis/ferramentas/lista_281.txt docs/Painter/Pinceis/fixtures/rebelle_bateria
python3 -I docs/Painter/Pinceis/ferramentas/indice_bateria.py docs/Painter/Pinceis/ferramentas/lista_281.txt \
  docs/Painter/Pinceis/fixtures/rebelle_bateria > docs/Painter/Pinceis/fixtures/rebelle_bateria/indice.csv
```

`lote_bateria.sh` abre cada sessão (`ph2d-run` + `gotas_inicia.sh`), passa o arranque (`rebelle_pronto.sh`), corre
`fotos_bateria.sh` até ao prazo e fecha; é RETOMÁVEL (salta a cena que já tem as duas fotos). 1.ª passagem = a
primeira cena de cada preset (uma foto de CADA um antes das repetições); 2.ª = o resto. ~7 s por cena.

## O que cada foto é

`<TOOL>/<nome>_<cena>_<A|B>.png` — recorte 1000×640 da tela a zoom 100 % (o `SAVE` não existe por WebSocket),
RGB, papel `Default/EX00 White Simple` branco, semente 7. O recorte é conferido em cada foto (fundo cinzento fora,
papel dentro) e o título da janela tem de dizer `1000 x 640 pixels` (o NEW_ARTWORK entrou).

- **A** = depois dos traços + `SIMULATION 240` (molhada) · **B** = depois do `DRY` (seca).
- **cenas** (o docstring de `bateria.py`): `gestos` (tamanho 40: recto lento, recto rápido, curva em S, espiral,
  rampas de pressão 0→1 e 0,15→1→0,15, vertical lenta e rápida) · `tamanhos` (recto a 5, 40 e 200 px) · `sobre`
  (duas faixas VERMELHAS de 60 px — a de baixo SECA, a de cima molhada — e o pincel AZUL a cruzá-las em três
  verticais e num recto sobre o papel).
- **Eventos** (`bateria_rebelle.py`): SET_BRUSH COMPLETO antes de CADA traço (a lei REPETE_PARAMS de
  `gera_eventos.py`), `size_px` e `size` = 100·(px/max)^(2/3) % do Max Size do preset; os pontos e a pressão
  EXACTOS da bateria; `seca` = DRY, `simula` = SIMULATION.
- ⚠️ **Tamanho acima do Max Size vai ao max** (31 presets na cena `tamanhos`, o recto de 200 px: Pencil a 70/80/100,
  Ink 100 ×7, Marker 100 ×2, Pastel 100 ×1). Medido 19/10: `size_px` 200 num preset de max 80 (Graphite 8B)
  carimba QUADRADOS soltos — não é o pincel; ao max ele pinta o traço normal.
- **Ferramentas sem tinta** (Blend, Smudge, Eraser, Clone, Water, Dry) e os 7 **Marker/Effect** não marcam papel
  branco: fazem só a cena `sobre`, com as faixas e o recto de uma BASE — `OIL_AND_ACRYLIC:Round/Round Hard` e,
  com o sufixo `_aq`, `WATERCOLOR:Watercolor/Round` (Blend/Smudge/Eraser/Clone nas duas; Water e Dry só na aguarela;
  os Effect sobre `MARKER:Marker/Bullet`). Nas fotos A do **Water** o ciano é o ecrã de «área molhada» do próprio
  Rebelle, não tinta.

## Contagens (presets por TOOL; cenas)

| TOOL | linhas da lista | cenas | | TOOL | linhas | cenas |
|---|---|---|---|---|---|---|
| PENCIL (grafite, carvão, cor, dry media, digital, riscos) | 44 | 132 | | MARKER (4 + 7 Effect) | 11 | 19 |
| PASTEL (seco, óleo, textura, scrape) | 24 | 72 | | AIRBRUSH | 5 | 15 |
| OIL_AND_ACRYLIC (bristle, flat, round, knife, grunge, metallic, dabber, rake) | 62 | 186 | | BLEND (8 × 2 bases) | 16 | 16 |
| EXPRESS_OIL (flat, glaze, grunge, round) | 32 | 96 | | SMUDGE (6 × 2) | 12 | 12 |
| WATERCOLOR (aguarela, guache, bristle, spatter, sumi-e, granulation) | 47 | 141 | | ERASER (10 × 2) | 20 | 20 |
| INK_PEN (pen, brush) | 10 | 30 | | WATER · DRY · CLONE (10 · 7 · 5 × 2) | 10 · 7 · 10 | 27 |

281 presets = os 281 de `NewFeaturesWetPaint/fixtures/rebelle_presets_281.json` (o `Grupo/Nome` é o do FICHEIRO,
que é o que o SET_BRUSH resolve em `Brushes/<pasta>/`; o campo `Name` difere em 9). O `Acrylic/Default preset` da
pasta do utilizador ficou de fora: não há TOOL `ACRYLIC` no Motion IO.

## O índice e o que olhar com cuidado

[`indice.csv`](indice.csv): preset · TOOL · nome · cena · ficheiro_A · ficheiro_B · estado · motivo · A_igual_B ·
tinta_A_pct. [`log.csv`](log.csv): uma linha por cena corrida (hora, segundos).

- **A = B byte a byte em 651 de 766 cenas**: só aguarela/guache/spatter/sumi-e, Water/Dry/Smudge/Blend e alguns
  óleos mudam entre a foto molhada e a seca (o git guarda o blob uma vez: 810 PNG únicos, 144 MB).
- **Presets que quase não marcam sob o Wine** (fotos válidas; é o oráculo assim): `Pastel Oil/Oil Pastel` (só o
  meio das rampas de pressão; `tamanhos` 0,00 % de tinta — já registado a 11/10) e `Spatter/Spatter` (quase
  transparente, 0,01 % — o doc 52 §1). Os seguintes mais fracos: Spatter 2/3, Metallic/Splats, Charcoal Liner
  (1,5–5 %).
