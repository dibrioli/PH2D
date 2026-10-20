# Fotos do Rebelle 8 sobre a bateria das LACUNAS (2026-10-20)

> Oráculo: **Rebelle 8.3.4 Pro, versão de TESTE** («Trial (17 days left)» a 20/10), sob Wine 11, tela virtual
> `kwin_wayland --virtual` 1920×1080, porta WebSocket (Motion IO) — o mesmo da [`rebelle_bateria`](../rebelle_bateria/README.md).
> Bateria: [`../../ferramentas/bateria_lacunas.json`](../../ferramentas/bateria_lacunas.json) **versão 1**, sha256
> `5ce10f13ea586664353cc43125c7e2293df91517f156af2e2b99dc328bb39f6b` (gerada por `bateria_lacunas.py`; a
> `bateria.json` não mudou: `cff23b71…14fa`). Lista: [`lista_lacunas_rebelle.txt`](../../ferramentas/lista_lacunas_rebelle.txt).
> Dois lotes (quatro sessões curtas, ≤ 30 min cada) com a vez da placa (`PH2D_GPU=1`), fecho só por `gotas_fecha.sh`; **30 cenas · 60 fotos
> · 0 falhas**, todas com a guarda da foto velha (`foto_estavel.py`: tela estável e ≠ da cena anterior).

## O comando (da raiz da árvore)

```bash
BAT=$PWD/docs/Painter/Pinceis/ferramentas/bateria_lacunas.json PH2D_PRAZO=1800 \
  bash docs/Painter/Pinceis/ferramentas/lote_bateria.sh \
  docs/Painter/Pinceis/ferramentas/lista_lacunas_rebelle.txt docs/Painter/Pinceis/fixtures/rebelle_lacunas
python3 -I docs/Painter/Pinceis/ferramentas/leis_lacunas.py <memoria|canto|efeito|velocidade|carimbos_3d|textura|dabs> <png>
```

`<TOOL>/<nome>_<cena>_<A|B>.png` — recorte 1000×640 a 100 %, papel `Default/EX00 White Simple`, semente 7;
A = depois de `SIMULATION 240`, B = depois do `DRY`. As cenas estão no docstring de `bateria_lacunas.py`.
O 6.º campo da lista vai JUNTO ao `SET_BRUSH` do preset (`--params` do `bateria_rebelle.py`).

## Índice (app Rebelle 8.3.4 · data 2026-10-20 · hora no `log.csv`)

| fotos | preset | gesto (cena) | base | params do SET_BRUSH | hora |
|---|---|---|---|---|---|
| `OIL_AND_ACRYLIC/flat_flat_oily_{memoria,canto}_{A,B}.png` | Flat/Flat Oily | memoria · canto | — | — | 04:39 · 04:43 |
| `OIL_AND_ACRYLIC/knife_knife_smooth_{memoria,canto}_{A,B}.png` | Knife/Knife Smooth | memoria · canto | — | — | 04:39 · 04:43 |
| `OIL_AND_ACRYLIC/flat_flat_dry_{memoria,canto}_{A,B}.png` | Flat/Flat Dry | memoria · canto | — | — | 04:39 · 04:43 |
| `EXPRESS_OIL/flat_knife_soft_{memoria,canto}_{A,B}.png` | Flat/Knife Soft (Express) | memoria · canto | — | — | 04:39 · 04:44 |
| `MARKER/effect_{burn,color,dodge,hue,luminosity,overlay,saturation}_efeito_{A,B}.png` | Effect/… (os 7) | efeito | MARKER:Marker/Bullet | — | 04:40–04:41 |
| `INK_PEN/pen_bullet_velocidade_{A,B}.png` | Pen/Bullet | velocidade | — | — | 04:41 |
| `PENCIL/graphite_pencil_hb_velocidade_{A,B}.png` | Graphite Pencil/HB | velocidade | — | — | 04:41 |
| `WATERCOLOR/watercolor_round_velocidade_{A,B}.png` | Watercolor/Round | velocidade | — | — | 04:41 |
| `WATERCOLOR/spatter_splats_carimbos_3d_{A,B}.png` | Spatter/Splats | carimbos_3d (toques a 3·D) | — | — | 04:42 |
| `WATERCOLOR/spatter_splats_esp3_gestos_{A,B}.png` | Spatter/Splats | gestos | — | `{"splat_spacing": 3.0}` | 04:42 |
| `PENCIL/charcoal_charcoal_2{,_ctl2,_ctl3}_tamanhos_{A,B}.png` | Charcoal/Charcoal 2 | tamanhos (controlos) | — | — | 04:42 · 04:48 |
| `PENCIL/charcoal_charcoal_2_semdual{,2}_tamanhos_{A,B}.png` | Charcoal/Charcoal 2 | tamanhos | — | `{"dual_brush_enabled": false}` | 04:42 · 04:48 |
| `PASTEL/pastel_dry_dry_thin{_ctl2,_ctl3}_tamanhos_{A,B}.png` | Pastel Dry/Dry Thin | tamanhos (controlos) | — | — | 04:48 |
| `PASTEL/pastel_dry_dry_thin_semdual{,2}_tamanhos_{A,B}.png` | Pastel Dry/Dry Thin | tamanhos | — | `{"dual_brush_enabled": false}` | 04:42 · 04:48 |
| `PENCIL/charcoal_charcoal_prime_tamanhos_{A,B}.png` | Charcoal/Charcoal | tamanhos (a mesma vizinhança de 19/10) | — | — | 04:48 |

## O que olhar com cuidado (medido)

- **O SET_BRUSH ignora os parâmetros do preset** (`dual_brush_enabled`, `splat_spacing`): `semdual2` ≡ `ctl3` (mesma
  textura, a mesma diferença ao `ctl2` que o controlo repetido: 9,07 % dos px), Splats `esp3` ≡ a foto de 19/10 (dabs/100 px,
  ⊥ sd e passo iguais). O `semdual` da 1.ª sessão saiu com o recto de 40 fino (5 px) e **não se repetiu** na 2.ª — não o use.
- **O recto de 200 px de Charcoal 2 / Dry Thin NÃO reproduz entre dias**: 57–60 px nas 5 corridas de 20/10 (2 lotes),
  170 / 95 px nas fotos de 19/10; o de 40 px é igual ao px. Ver 06 §10.
- **Sem SET_BRUSH (cena `memoria`) o tamanho volta ao do PRESET** (Flat Oily 40 → 64 no horizontal, Flat Dry 40 → 78) —
  a lei REPETE_PARAMS de `gera_eventos.py`; o arranque mede-se ÷ a mediana do próprio traço.
