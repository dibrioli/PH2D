# Fotos do MyPaint (libmypaint) sobre a bateria das LACUNAS (2026-10-20)

> Oráculo: **libmypaint 1.6.1 (ISC) + mypaint-brushes 2.0.2 (CC0)** (`pacman -Qi`, 20/10), pelo C nosso
> `mypaint_bateria.c` — a porta de [`../mypaint_bateria/`](../mypaint_bateria/README.md) (dt 10 ms/evento, a caneta paira 0,5 s).
> Bateria: [`../../ferramentas/bateria_lacunas.json`](../../ferramentas/bateria_lacunas.json) v1, sha256
> `5ce10f13…f6b` (o `manifesto.json` guarda-o inteiro). Determinística (um fio, um `MyPaintBrush` por processo).

```bash
B=/usr/share/mypaint-data/2.0/brushes
python3 -I docs/Painter/Pinceis/ferramentas/mypaint_bateria.py --pincel $B/classic/pen.myb $B/deevad/pen-note.myb \
  $B/classic/knife.myb $B/ramon/Marker.myb $B/ramon/Thin_Pen.myb --saida docs/Painter/Pinceis/fixtures/mypaint_lacunas \
  --bateria docs/Painter/Pinceis/ferramentas/bateria_lacunas.json --cenas velocidade,canto,memoria
```

| fotos | pincel | gestos (cenas) | para quê |
|---|---|---|---|
| `classic/pen_{velocidade,canto,memoria}.png` | classic/pen | rectos a 3–50 px/evento · cantos · memória | L12 (a curva da velocidade) |
| `deevad/pen-note_{…}.png` | deevad/pen-note | idem | L12 |
| `ramon/Marker_{…}.png` · `ramon/Thin_Pen_{…}.png` | ramon/Marker · Thin_Pen | idem | L12 (o mais forte; o que só perde cobertura) |
| `classic/knife_{…}.png` | classic/knife (`direction_filter` 2,99) | idem | L3: o controlo de canto de uma ponta REDONDA |

Uma foto por cena (o MyPaint não tem molhado: A ≡ B). Data 2026-10-20; o app (GPL) não corre — só a biblioteca.
