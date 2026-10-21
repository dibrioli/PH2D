# O Krita com a caneta DEITADA (família `lapis`, J3, 2026-10-20)

Saída do Krita 6.0.4.1 (`pacman -Qi krita`: GPL3 ⇒ só se CORRE; nenhum `.kpp`, ponta ou textura copiado) sobre
eventos NOSSOS: a bateria deitada [`exames/lapis/bateria_deitado.json`](../../exames/lapis/bateria_deitado.json) — as
linhas dos blocos B, D e A de `NewFeaturesWetPaint/fixtures/rebelle_tilt` (x 60 → 560 a 3 px, pressão 1, 40 px; cada
ponto `[x, y, p, tilt_x°, tilt_y°]`, `(t, 0)` = ao longo do traço, `(0, t)` = através). O Krita corta a inclinação nos
60°: a linha de 63° é a de 60°.

Gerado por [`exames/lapis/krita_bateria.py`](../../exames/lapis/krita_bateria.py) +
[`krita_bateria_dentro.py`](../../exames/lapis/krita_bateria_dentro.py) — as cópias das ferramentas da bancada com a
caneta a levar `xTilt`/`yTilt` no `handleTabletEvent` (o dispositivo declara `XTilt | YTilt`) e a cura de um defeito
do original: o `base` do preset das faixas tapava o `base` do caminho da PNG, e a 2.ª e a 3.ª cena de cada preset
saíam com o nome `<PyKrita.krita.Resource object …>_<cena>.png` na pasta de onde se corre (⚠️ o original em
`ferramentas/` tem o mesmo defeito; fica para quem o mantém).

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PainterWatercolor
bash docs/Painter/Pinceis/ferramentas/vez_do_app.sh krita python3 -I docs/Painter/Pinceis/exames/lapis/krita_bateria.py \
  --presets "c) Pencil-5 Tilted,c) Pencil-6 Quick Shade" --bateria docs/Painter/Pinceis/exames/lapis/bateria_deitado.json \
  --cru /tmp/krita_deitado_cru --saida docs/Painter/Pinceis/fixtures/krita_deitado
```

## Medidas (largura a ¼ do p90 dos picos, x 160..460; `exames/lapis.md`)

| preset | de pé | através 35 · 40 · 45 · 50 · 55 · 63° | ao longo 20 · 35 · 45 · 50 · 55 · 60 · 63° |
|---|---|---|---|
| c) Pencil-5 Tilted | 2 px | 9 · 10 · 11 · 12 · 13 · 17 | 2 · 4 · 4 · 4 · 5 · 6 · 6 |
| c) Pencil-6 Quick Shade | 23 px | 23 em todas (não lê a caneta) | 22–23 |

O Pencil-5 deitado através é uma faixa com a ARESTA dura do lado da mina e o esbatido do outro (a ponta-gradiente virada
pelo azimute) e o grão do papel; ao longo, uma linha mais densa e pouco mais larga.
