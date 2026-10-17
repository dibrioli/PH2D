# Fotos do Rebelle 8.3.4 sobre os gestos da bancada (sessões de 2026-10-07 e 2026-10-11)

Recorte 1000×640 da janela a zoom 100 % (`SAVE` não existe por WebSocket), estado A (depois dos traços + `SIMULATION`). Gerado por `ferramentas/fixtures_rebelle.py`; as medidas estão no [03 §2](../../03_tracos_medidos_rebelle_x_ph2d.md). As 12 primeiras linhas são de 07/10; as 13 seguintes (de `wcround` a `markerbullet`), de 11/10: presets de fábrica com a água e a opacidade do próprio preset. «Pastel Oil/Oil Pastel» saiu quase em branco (só as pontas da rampa de pressão) e foi substituído por «Oil Pastel 2».

| ficheiro | ferramenta | preset | size_px | pressão | extras | semente | papel | sha256 (16) |
|---|---|---|---|---|---|---|---|---|
| `oil.png` | OIL_AND_ACRYLIC | Bristle/Bristle Flat | 40 | 100 | — | 7 | Default/EX00 White Simple | `2511e721cecdafde` |
| `express.png` | EXPRESS_OIL | Round/Hairy | 40 | 100 | — | 7 | Default/EX00 White Simple | `67e62e67d16b48c6` |
| `wc.png` | WATERCOLOR | Watercolor/Flat 2 | 40 | 100 | — | 7 | Default/EX00 White Simple | `78cb86a061e354f0` |
| `wcpreset.png` | WATERCOLOR | Watercolor/Flat 2 | 40 | 100 | water 30.0, opacity 50.0 | 7 | Default/EX00 White Simple | `92cce3abcf3d6ffe` |
| `wcwet.png` | WATERCOLOR | Watercolor/Round | 40 | 100 | water 80.0, opacity 60.0 | 7 | Default/EX00 White Simple | `03aa52910b8e320b` |
| `ink.png` | INK_PEN | Pen/Bullet | 40 | 100 | — | 7 | Default/EX00 White Simple | `3039b0e4677d1659` |
| `pencil.png` | PENCIL | Graphite Pencil/8B | 40 | 100 | — | 7 | Default/EX00 White Simple | `cf217f80b3d276f8` |
| `penciltilt.png` | PENCIL | Graphite Pencil/8B | 40 | 100 | pen_tilt [(55.0, 0.0)] | 7 | Default/EX00 White Simple | `79c06a4034df15ac` |
| `pastel.png` | PASTEL | Pastel Dry/Dry Thin | 40 | 100 | — | 7 | Default/EX00 White Simple | `ff6c7a4d852b4e69` |
| `marker.png` | MARKER | Marker/Liner | 40 | 100 | — | 7 | Default/EX00 White Simple | `787e3de32c8e80ff` |
| `airbrush.png` | AIRBRUSH | Default/Spray 2 | 40 | 100 | — | 7 | Default/EX00 White Simple | `608f05039813813f` |
| `oildirty.png` | OIL_AND_ACRYLIC | Bristle/Bristle Flat | 40 | 100 | dirty_brush True, paint_type PAINT_MIX | 7 | Default/EX00 White Simple | `ab12513ff9a30f85` |
| `wcround.png` | WATERCOLOR | Watercolor/Round | 40 | 100 | — | 7 | Default/EX00 White Simple | `8bc948b6c0cfcda1` |
| `wcwash.png` | WATERCOLOR | Watercolor/Wash Round | 40 | 100 | — | 7 | Default/EX00 White Simple | `98ed08e61a7c4c99` |
| `wcflat.png` | WATERCOLOR | Watercolor/Flat | 40 | 100 | — | 7 | Default/EX00 White Simple | `6ae817cf1c8e1efc` |
| `wcliner.png` | WATERCOLOR | Watercolor/Liner | 40 | 100 | — | 7 | Default/EX00 White Simple | `d530d38dd78259f1` |
| `gouache.png` | WATERCOLOR | Gouache/Gouache Round | 40 | 100 | — | 7 | Default/EX00 White Simple | `40659fd8f774e1a1` |
| `gouacheflat.png` | WATERCOLOR | Gouache/Gouache Flat | 40 | 100 | — | 7 | Default/EX00 White Simple | `1f63f5e6248d9896` |
| `sumi.png` | WATERCOLOR | Sumi-e/Sumi-e  soft | 40 | 100 | — | 7 | Default/EX00 White Simple | `ee823c51a4c43312` |
| `dippen.png` | INK_PEN | Pen/Dip Pen | 40 | 100 | — | 7 | Default/EX00 White Simple | `5e7ea8ad683d8867` |
| `inkbrush.png` | INK_PEN | Brush/Brush | 40 | 100 | — | 7 | Default/EX00 White Simple | `3627c75138db8e05` |
| `oilround.png` | OIL_AND_ACRYLIC | Round/Round Hard | 40 | 100 | — | 7 | Default/EX00 White Simple | `11ee4ffc5d614eb2` |
| `charcoal.png` | PENCIL | Charcoal/Charcoal | 40 | 100 | — | 7 | Default/EX00 White Simple | `dda2aebbdeccf670` |
| `oilpastel2.png` | PASTEL | Pastel Oil/Oil Pastel 2 | 40 | 100 | — | 7 | Default/EX00 White Simple | `c7708d9d52c5d92f` |
| `markerbullet.png` | MARKER | Marker/Bullet | 40 | 100 | — | 7 | Default/EX00 White Simple | `132547dd52971561` |

## Pontas texturadas (sessão de 2026-10-13, plano 27 W0)

Gerado por `ferramentas/ponta_textura_rebelle.sh` (+ `ponta_textura_rebelle.py`: eventos, montagem, cabeçalhos e a
régua, que mede SÓ estas fixtures). Watercolor, azul-escuro (20, 30, 120), opacidade 70 em todos, traço recto de
220 px a pressão 1. `ponta_textura_<preset>_<20|40>.png` = grelha 2×2 de células 320×100 (colunas: molhado ~2 s ·
SIMULATION 400 + DRY; linhas: água 10 · 50). `ponta_textura_opacidade_*_40.png` = a tela inteira, opacidade
100 · 85 · 70 · 55 · 40 de cima para baixo. Porque cada ponta é texturada: estatística da imagem `Shape_0` do
preset, calculada em memória (nada extraído). Cabeçalhos completos em `cabecalhos.json`.

| ficheiro | preset | ponta |
|---|---|---|
| `ponta_textura_round_{20,40}.png` | Watercolor/Round | controlo: disco liso |
| `ponta_textura_fan_{20,40}.png` | Watercolor/Fan | leque de cerdas (miolo média 0,44, 50 % vazio) |
| `ponta_textura_gouachedry_{20,40}.png` | Gouache/Gouache Dry | manchas de pincel seco (média 0,12, 74 % vazio) |
| `ponta_textura_granulation_{20,40}.png` | Granulation/Granulation | grão (média 0,57, 28 % vazio); grão com brilho −0,65 |
| `ponta_textura_flatdry_{20,40}.png` | Watercolor/Flat Dry | forma cheia + ruído de média 0,49 (o seco vem do papel, que não se desenha sob Wine) |
| `ponta_textura_opacidade_granulation_40.png` | Granulation/Granulation | varredura da opacidade |
| `ponta_textura_opacidade_gouachedry_fan_round_40.png` | Gouache Dry · Fan · Round | varredura da opacidade (colunas) |

## O cruzamento SECO (sessão de 2026-10-16, prompt 42)

Os mesmos gestos (`ferramentas/gera_eventos.py --seca_cruz 240`): depois do azul recto da linha 3, `SIMULATION 240` + `DRY`, e só então o vermelho que o cruza — o azul está SECO quando a tinta de cima chega. Estado A, recorte 1000×640. `p42dippen.png` é o CONTROLO da sessão: o `dippen` de 11/10 refeito igual (cruzamento 0,283 contra 0,223 — o ruído da sessão).

| ficheiro | ferramenta | preset | size_px | pressão | extras | semente | papel | sha256 (16) | SIMULATION |
|---|---|---|---|---|---|---|---|---|---|
| `inkseco.png` | INK_PEN | Pen/Bullet | 40 | 100 | — | 7 | Default/EX00 White Simple | `08963f7881df3971` | [240, 240] |
| `dippenseco.png` | INK_PEN | Pen/Dip Pen | 40 | 100 | — | 7 | Default/EX00 White Simple | `e958ab1fcd18be1d` | [240, 240] |
| `inkbrushseco.png` | INK_PEN | Brush/Brush | 40 | 100 | — | 7 | Default/EX00 White Simple | `d58c09cefbbbab4c` | [240, 240] |
| `markerseco.png` | MARKER | Marker/Liner | 40 | 100 | — | 7 | Default/EX00 White Simple | `d853f7c8e22ebfdd` | [240, 240] |
| `markerbulletseco.png` | MARKER | Marker/Bullet | 40 | 100 | — | 7 | Default/EX00 White Simple | `5c3ca3c2024a0f00` | [240, 240] |
| `wcroundseco.png` | WATERCOLOR | Watercolor/Round | 40 | 100 | — | 7 | Default/EX00 White Simple | `0498037ee88c3b56` | [240, 240] |
| `p42dippen.png` | INK_PEN | Pen/Dip Pen | 40 | 100 | — | 7 | Default/EX00 White Simple | `62bf5cb965252cdf` | [240] |

## A MESMA tinta (sessão de 2026-10-16, prompt 42)

`gera_eventos.py --cor_base/--cor_cruz` (e `--seca_cruz 240` onde diz SECO). A cena «marcador azul sobre o azul SECO» saiu em branco (o 1.º envio antes da tela) e não entra.

| ficheiro | ferramenta | preset | cruzamento | sha256 (16) |
|---|---|---|---|---|
| `mbazulmolhado.png` | MARKER | Marker/Bullet | azul sobre azul, molhado | `1d03cc57ef24d25b` |
| `inkazulazul.png` | INK_PEN | Pen/Bullet | azul sobre azul SECO | `2405591a661caff4` |
| `mblaranja.png` | MARKER | Marker/Bullet | amarelo sobre laranja SECO | `c3f7b18037f82c73` |

## O aerógrafo a três tamanhos (sessão de 2026-10-17, prompt 43)

Os mesmos gestos (`gera_eventos.py AIRBRUSH "Default/Spray 2" … --size_px N`), estado A, recorte 1000×640. `airbrush40b.png` é o CONTROLO da sessão: o `airbrush` de 07/10 refeito com eventos byte-iguais. Comando exacto no campo `comando` de `cabecalhos.json`.

| ficheiro | ferramenta | preset | size_px | Size (slider) | semente | papel | sha256 (16) | SIMULATION |
|---|---|---|---|---|---|---|---|---|
| `airbrush20.png` | AIRBRUSH | Default/Spray 2 | 20 | 9.3 | 7 | Default/EX00 White Simple | `26ef6c5cddb0be7a` | [240] |
| `airbrush80.png` | AIRBRUSH | Default/Spray 2 | 80 | 23.6 | 7 | Default/EX00 White Simple | `ecf51447dd38c69c` | [240] |
| `airbrush40b.png` | AIRBRUSH | Default/Spray 2 | 40 | 14.8 | 7 | Default/EX00 White Simple | `151a0ab747a781ac` | [240] |
