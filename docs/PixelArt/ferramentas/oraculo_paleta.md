# Oráculo das paletas — Aseprite Trial 1.3.18.6, caixa preta

Medido em 2026-11-01. Condutor: [`trial_paleta.py`](trial_paleta.py) (docstring = uso e arranque; `abre()`
REINICIA o Trial a cada caso). Fixtura: [`oracle_trial_palette.json`](../../../shells/pixel-scenes/tests/fixtures/oracle_trial_palette.json).

**Entradas NOSSAS** (`trial_paleta.py gera`): `idx.ase` 8×8 INDEXADO, Background, pixel (x,y) = índice x,
paleta 0 preto · 1 vermelho · 2 verde · 3 azul · 4 amarelo · 5 ciano · 6 magenta · 7 branco; `rgb.ase` igual em RGBA.
**Leitura**: fotografia (o Trial não grava); coluna k do canvas a 3200 % e amostra i da barra (mapa abaixo).

| Item | Comportamento medido | Caso |
|---|---|---|
| 1. Arrastar-reordenar | Agarra-se a **borda** da selecção (arrastar de dentro só estende a selecção). Largar 1 sobre 4 → paleta `0,2,3,4,1,5,6,7` (o item cai NA célula de destino). **INDEXADO: os pixels guardam o índice** — o canvas muda de cor — e aparece o botão **«Remap Palette»** por cima do seletor; clicá-lo remapeia os pixels para seguirem as cores (canvas volta ao original, paleta fica reordenada) e o botão some. Undo: **arrasto = 1 passo, Remap = outro passo** (1.º Ctrl+Z desfaz o remap, 2.º a ordem; «Undid Drag-and-Drop Colors»). **RGBA**: só a paleta muda, sem botão, 1 undo. | `drag_idx`, `drag_rgb` |
| 1. Ctrl+arrastar | **Copia e INSERE**: largar 1 sobre 4 com Ctrl → `0,1,2,3,1,4,5,6,7` (9 cores); em indexado os índices ≥ 4 passam a apontar para a cor seguinte (canvas desloca); aparece «Remap Palette»; 1 undo. | `drag_ctrl_idx` |
| 4. Gradient | Entre a 1.ª e a última da selecção, por canal: `c_i = a + trunc((b−a)·i/(n−1))` (truncagem para zero). ff0000→00ffff em 5: `255,0,0 · 192,63,63 · 128,127,127 · 64,191,191 · 0,255,255`; 000000→ffff00: `0,63,127,191,255`. 1 undo. | `gradient` |
| 4. Gradient by Hue | Interpolação em HSV, matiz crescente, canais arredondados: ff0000→00ffff = `255,0,0 · 255,191,0 · 128,255,0 · 0,255,64 · 0,255,255`. | `gradient_hue` |
| 4. Presets | Botão-pasta da barra abre uma janela: Search, recarregar, lista com nome + tira de amostras, **Load** e **Open Folder**. Nomes visíveis sem rolar: A64, AAP-64, AAP-Micro12, AAP-RadiantXV, AAP-Splendor128, Apple II, ARNE16, ARNE32, ARQ4, ARQ16, Atari 2600 NTSC, Atari 2600 PAL, BubbleGum16, CGA, CGA0, CGA0 High (há mais). Menu ≡: Load Palette, Load Default Palette, Save Palette, Save Palette as Preset, Save Palette as Default, New Palette from Sprite. | `menus` |
| 5. Menu «↓» | Reverse Colors · Gradient · Gradient by Hue · Sort by Hue/Saturation/Brightness/Luminance · Sort by Red/Green/Blue/Alpha · Ascending/Descending. Todos **só sobre a selecção** (Reverse 1..4 → `0,4,3,2,1,5,6,7`), 1 undo cada; em INDEXADO os pixels guardam o índice (o canvas troca de cor) e aparece «Remap Palette». Sort by Hue (tudo) → preto, branco, vermelho, amarelo, verde, ciano, azul, magenta; Sort by Brightness → preto, branco, vermelho, verde, azul, amarelo, ciano, magenta (os cinzentos à frente; empates não são estáveis pela ordem antiga). | `sort_reverse` |
| 5. Menu ≡ | Edit Palette (A), Palette Size, Select ▸, Small/Medium/Large Size, os 5 seletores (abaixo), Load/Save (acima), New Palette from Sprite. | `menus` |
| 2. Tint/Shade/Tone | Caixa de 136×116 px físicos: o plano **S×V do HSV na matiz da cor** (sup-esq branco, sup-dir a matiz pura, base preta), amostrado ao centro do pixel, `s≈(x+0.5)/136`, `v≈1−(y+0.5)/116`; por baixo, barra de matiz e barra de alfa. O ajuste HSV fica a ≤ 2 em todos os pixels (nenhuma variante testada deu 0). | `seletores` |
| 3. Spectrum | Matiz na horizontal (vermelho→amarelo→verde→ciano→azul→magenta→vermelho), branco em cima, saturada a meio, preto em baixo; por baixo, barra de saturação e alfa. | `seletores` |
| 3. Rodas | RGB: disco de matiz, branco ao centro, verde em cima, vermelho em baixo-direita; RYB: amarelo em cima, vermelho em baixo-direita, azul em baixo-esquerda; Normal Map: fundo azul-lilás plano e disco de normais. As rodas têm barra de valor (preto→cor) e um ícone de menu no canto. | `seletores` |

## Mapa da tela (janela 1600×1000, escala de UI 2, Trial reiniciado, Home + o ficheiro)

Amostra i da barra: `x = 30+24·(i%5)`, `y = 118+24·(i//5)` (centro `x−8, y−8`); borda esquerda da amostra 1 `(35,110)`.
Botões da barra: cadeado `(18,70)`, «↓» `(56,70)`, presets `(84,70)`, ≡ `(112,70)`. «Remap Palette» `(77,712)`.
Canvas a 3200 %: coluna k em `x=412+64k`, `y=700`. Menu «↓»: Reverse `(124,103)`, Gradient `(96,126)`,
Gradient by Hue `(128,148)`, Sort by Hue `(112,177)`, Brightness `(140,221)`. Menu ≡: Spectrum `(183,281)`,
RGB `(188,303)`, RYB `(188,325)`, Normal Map `(222,347)`.

## ⛔ Recusas / limites

| O quê | Porquê |
|---|---|
| Lei exacta do Tint/Shade/Tone | O HSV arredondado erra 1–2 em quase todos os pixels; a grelha de ajustes (offset, largura, trunc/round) não chegou a 0. Pode ser outro espaço (p.ex. mistura com gama) — por medir. |
| Cor de base 200,60,40 (e outras não puras) | O campo hex do popup da cor não recebeu o texto do `xdotool type` (a 1.ª tentativa mudou o fg para `idx-3`/`idx-6`); medido só com matizes puras — não se sabe se a caixa depende de S/V da base (as rodas e a barra mostram só a matiz). |
| Valores das rodas e do Spectrum | Só o layout por fotografia; nenhum ponto lido contra fórmula. |
| Lista inteira dos presets | Só os 16 visíveis sem rolar. |
| O transparente segue o arrasto? | Não medido (o índice 0 nunca foi arrastado). |
| Sort/Reverse no RGBA, Ctrl+arrastar no RGBA, arrastar um bloco de várias | Não medidos. |
| Fechar documentos por teclado entre casos | ⚠️ deixou estado alheio (sprite 1×8 com 2 quadros) — por isso `abre()` reinicia o Trial (mata por PID só os do `DISPLAY=:80`). |
