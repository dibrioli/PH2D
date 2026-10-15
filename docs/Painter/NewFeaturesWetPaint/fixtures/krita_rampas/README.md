# Oráculo Krita — rampa de cor da PONTA × textura (Grain) × rampa de cor da TEXTURA

> **Oráculo CORRIDO, nunca lido.** Krita é **GPL3** (`pacman -Qi krita`: `krita 6.0.4.1-1.1`,
> `Licenças: GPL3`, `/usr/bin/krita`). Nenhum ficheiro-fonte do Krita foi aberto; só se correu o
> binário instalado sobre ENTRADAS NOSSAS (`entradas/`) e se mediram os píxeis que ele pintou.
> Data da corrida: **2026-10-14**. Máquina: workstation, Xvfb privado `:97` (nunca o `:0` do dono).

## Como se correu (linhas exactas)

```sh
S=<scratch>/krita            # config/dados DESCARTÁVEIS: XDG_CONFIG_HOME=$S/config, XDG_DATA_HOME=$S/data, HOME=$S/home
Xvfb :97 -screen 0 1600x1000x24 -nolisten tcp &
python3 -I scripts/make_assets.py $S/data/krita            # gera entradas/: ponta, padrão, gradiente
# plugin Python (scripts/rampas_oracle.desktop + plugin_rampas_oracle__init__.py → $S/data/krita/pykrita/),
# ligado por scripts/kritarc ([python] enable_rampas_oracle=true); ele faz exec do script em RAMPAS_SCRIPT.
RAMPAS_CASES="$(cat scripts/cases.json)" RAMPAS_OUT=$S/out/run1 RAMPAS_DUMP_EDITOR=1 \
  scripts/run_krita.sh $S/paint_cases.py $S/out/run1.log  # = env -u WAYLAND_DISPLAY DISPLAY=:97 QT_QPA_PLATFORM=xcb … /usr/bin/krita --nosplash
python3 -I scripts/analisa.py $S/out/run1 saidas/ /usr/share/krita/patterns/06_hard-grain.png > saidas/metricas.json
```

**Porta:** `krita --nosplash` completo (o `kritarunner` não foi preciso). A pintura é
`Node.paintLine(QPoint, QPoint, 1.0, 1.0)` com o preset ACTIVO da vista. Preset base:
`b) Basic-5 Size` (pixel engine), XML editado via `Preset.toXML()/fromXML()` (Size/Flow sem curva de
pressão, spacing 0,05, tamanho 64 px). Cor de frente = **verde** `(0,200,0)` (para ver se ela entra),
gradiente corrente = **`RedBlue_oracle`** (vermelho puro → azul puro). Saída: `Node.pixelData` cru
(BGRA não-pré-multiplicado) → PNG em `saidas/` (600×200, fundo transparente).

**Entradas** (`entradas/`):
- ponta `rampa_vertical_tip.png` 64×64: alfa = disco duro r=30; RGB = rampa VERTICAL (escuro em cima
  → claro em baixo, leve tinta azul para o Krita a tratar como ponta «colorida»);
- padrão `rampa_horizontal_64.png` 64×64: cinzento = rampa HORIZONTAL `(x mod 64)/63`, constante em y;
- gradiente `RedBlue_oracle.ggr`.

**Geometria:** dois traços horizontais por caso: `(40,60)→(540,60)` e `(63,140)→(563,140)` — o 2.º
deslocado `(+23, +80)`; 23 não é múltiplo do período 64 do padrão.

**Métricas** (`analisa.py`, região interior: `dy ∈ [-26,26]`, `x_rel ∈ [40,460)`):
`mean_alpha` (0–1), `holes_frac` = fracção com alfa < 0,5; `t = B/(R+B)` = posição no gradiente
(0 = vermelho, 1 = azul); `t_std_across` = variação de t ATRAVÉS do traço (com dy → segue a ponta);
`t_std_along` = variação AO LONGO (com x → vem da textura); `corr_*_canvasRamp` = correlação com o
padrão em x de CANVAS; `*_same_CANVAS_x_maxdiff` vs `*_same_RELATIVE_x_maxdiff` = diferença máxima
entre os dois traços comparados no mesmo x de canvas vs no mesmo x relativo ao início do traço.

### ⚠️ O que a porta exigiu (para quem re-correr)

1. **O «Brush mode» da ponta não pega pelo XML.** `brushApplication="3"` no `brush_definition` é
   ignorado na carga (saiu «Color image» com `ColorAsMask=0`, «Alpha mask» com `ColorAsMask=1`;
   `preserveLightness=1` dá «Lightness map»). Só pega escolhendo no combo `cmbBrushMode` do PRÓPRIO
   editor (`setCurrentIndex` + `activated.emit` + ~1,5 s de eventos) — ver `scripts/probe_ui.py`. O
   XML que a UI grava depois é `ColorAsMask="1" brushApplication="3" AdjustmentVersion="2" md5sum=…`.
2. **`Preset.fromXML` muda o recurso em memória**: reler `toXML()` do mesmo recurso arrasta chaves de
   um caso para o seguinte (o padrão stock de um caso ficou nos dois seguintes). Cura: XML PRISTINO
   capturado uma vez (`BASEXML`).
3. `Node.paintLine` no Krita 6 (PyQt6) quer `QPoint`, não `QPointF`. `Node.save` abre o diálogo de
   PNG mesmo com `setBatchmode(True)` → usou-se `pixelData`.
4. `TexturingMode` = índice do combo `cmbTexturingMode`: 0 Multiply · 1 Subtract · 2 Lightness Map ·
   3 Gradient Map · 4 Darken · 5 Overlay · 6 Color Dodge · 7 Burn · 8 Linear Dodge · 9 Linear Burn ·
   10/11 Hard Mix (PS)/Softer · 12–15 Height variants (confirmado lendo o texto do combo após carregar).

## Respostas

| | Pergunta | Caso(s) | Resposta do Krita 6.0.4 (medida) |
|---|---|---|---|
| **A** | Ponta em Gradient Map + textura Multiply/Subtract: a textura ainda CORTA a cobertura? a cor segue a PONTA? | `c1`, `c2` (controlos `c0`, `c6`; stock `c7`) | **Sim e sim.** Cobertura cortada: `mean_alpha` 1,000 (c0, sem textura) → **0,936 / 0,926** (Multiply) e **0,938 / 0,928** (Subtract), `holes_frac` 0 → **0,048 / 0,056**; o mesmo corte que a ponta lisa sofre (c6: 0,940 / 0,930, holes 0,046 / 0,053) — o modo da ponta não muda o corte. A cor é a da PONTA: `t` através do traço = **0,125 / 0,478 / 0,827** em dy = −24/0/+24, **idêntico** nos dois traços (diferença máx. **0,000**), `t_std_along` = **0,000** (a textura não mexe na cor), verde da cor de frente = 0. Com o grão stock `06_hard-grain` (c7) o corte é pequeno (alfa 0,993, holes 0,1 %) porque os ~20 carimbos sobrepostos (spacing 0,05) somam por cima — a textura modula CADA carimbo, não o traço acumulado. |
| **B** | Ponta lisa (máscara) + textura Gradient Map: a cor vem do valor da textura? | `c3` (e `c9`: ponta colorida em «Alpha mask») | **Sim, inteiramente.** `corr(t, padrão em x de canvas)` = **1,000** nos dois traços; `t_std_along` = 0,284 / 0,299, `t_std_across` = 0,000; verde da cor de frente = **0** (a cor de frente é substituída). **Não corta cobertura**: `mean_alpha` = **1,000**, holes = 0. `c9` dá a mesma cor (t idênticas). |
| **C** | Os dois em Gradient Map: quem ganha / como combinam? | `c4` vs `c3` | **A textura ganha e a ponta é IGNORADA.** `c4` é **byte-idêntico** a `c3` (ponta lisa) no interior dos traços (diferença máx. **0** em RGBA); `t_std_across` = **0,000** (a rampa vertical da ponta desaparece). Não há combinação: um só gradiente (o corrente) e a cor sai só do valor da textura. |
| **D** | A textura é ancorada ao CANVAS ou ao pincel? | `c1`, `c2`, `c3`, `c6`, `c7`, `c9` | **Canvas, em todos os modos.** Alfa (Multiply/Subtract): diferença entre os dois traços no **mesmo x de canvas** = **0,0085 / 0,0029 / 0,0059** contra **1,000** no mesmo x relativo. Cor (Gradient Map): **0,000** em x de canvas contra **0,651** em x relativo. Grão stock 2D (c7): o melhor deslocamento do ladrilho contra o alfa é **(0, 0)** nos dois traços (corr 0,373 / 0,408) — origem do canvas. A cor da ponta em Gradient Map segue o PINCEL (A acima). Opções de deslocamento expostas: `OffsetX/Y`, `isRandomOffsetX/Y` (aleatório por traço), `MaximumOffsetX/Y`; não há modo «ancorar ao carimbo». |
| **E** | O editor oferece um gradiente de cor de «papel»/canvas para pincéis? | `editor/*.png`, log | **Não como opção própria.** O que existe: (1) Brush Tip → *Brush mode* `Alpha mask · Color image · Lightness map · Gradient map` (só para ponta colorida; com a ponta cinzenta stock `smoke.png` o combo só oferecia `Alpha mask`); (2) Texture → *Texturing mode* com `Lightness Map` e `Gradient Map` (lista completa na secção acima) — é o mais perto de «rampa de cor do papel», e é ancorado ao canvas; (3) Color → *Source* `Plain color · Gradient · Uniform random · Total random · Pattern · Locked pattern` (cor tirada de um padrão — não medido aqui). **Nenhum dos dois Gradient Map tem seletor de gradiente próprio**: ambos usam o **gradiente corrente da barra** (a saída é vermelho↔azul = `RedBlue_oracle`, o único gradiente com essas cores), por isso ponta e textura não podem ter rampas diferentes. Lista de opções do editor: General, Brush Tip, Blending Mode, Opacity, Flow, Size, Ratio, Spacing, Mirror, Softness, Rotation, Sharpness, Lightness Strength, Scatter, Color{Source, Darken, Mix, Hue, Saturation, Value}, Airbrush, Rate, Painting Mode (Wash/Build up), Texture{Pattern, Strength}, Masked Brush{…}. |

## Ficheiros

- `saidas/c0…c9_*.png` — os píxeis do Krita (fundo transparente); `saidas/metricas.json` — todos os números.
  `c5` (ponta lisa sem textura) e `c8` (ponta lisa + Lightness Map: cor de frente modulada pela
  textura, cobertura cheia) são controlos.
- `editor/` — capturas (`QWidget.grab()` dentro do próprio Krita, reduzidas a 600 px) das páginas
  Brush Tip, Pattern (Texture), Source e Painting Mode.
- `entradas/` — as nossas entradas; `scripts/` — tudo o que correu, tal como correu.
