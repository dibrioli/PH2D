# 01 — Estudo do Aseprite 1.3.18.6 (a espinha do módulo Pixel Art)

> Data: 2026-10-25. Fonte de cada linha: **[app]** = visto no Trial 1.3.18.6 a correr (fotos em
> [`aseprite/`](aseprite/)); **[doc]** = documentação pública de usuário (aseprite.org/docs, API,
> release notes, [spec do formato](https://github.com/aseprite/aseprite/blob/main/docs/ase-file-specs.md));
> **[NC]** = não confirmado — verificar no app antes de implementar.
>
> ⛔ **Clean-room.** O fonte do Aseprite é proprietário (EULA desde 2016 — o cabeçalho de
> `crates/ph2d-aseprite/src/lib.rs` diz «GPLv2», o que vale só para o fork antigo/LibreSprite). Lê-se
> documentação, spec do formato e o app a correr; **nunca** `.cpp/.h`. Ver
> [SKILL_Cleanroom](../_Skill_Especificações/SKILL_Cleanroom_Reimplementacao.md).

## §0 — O que o Trial deixa fazer (medido 25/10)

| Capacidade | Trial | Como foi medido |
|---|---|---|
| Desenhar, menus, diálogos, timeline | ✅ | Xvfb `:77` + `xdotool`, fotos por `import -window root` |
| **Salvar / exportar** (GUI e `-b --save-as`) | ⛔ *«Save operation is not supported in trial version»* | `-b in.png --scale 2 --save-as out.png` → nenhum ficheiro |
| **`--script` / `--script-param`** | ⛔ *«Invalid option --script»* (a opção nem existe no binário) | idem |
| `--help` (lista do CLI) | ✅ | [`aseprite/40_cli_help_1.3.18.6.txt`](aseprite/40_cli_help_1.3.18.6.txt) |

⇒ **O Trial é referência de COMPORTAMENTO (observar), não oráculo de SAÍDA** (§0.9 do CLAUDE.md
pede um oráculo que se corre por script e grava fixtura). Ver o plano §3 para a decisão do oráculo.

Instalação: `~/.local/opt/aseprite-trial/Aseprite-trial.AppImage`, atalho `aseprite-trial.desktop`.
Para fotografar sem tocar no ecrã do dono: Xvfb + `import -window root` (⛔ nunca `spectacle`).

## §1 — Mapa dos menus [app]

Fotos: [`10_menus…`](aseprite/10_menus_file_edit_sprite_layer.png) ·
[`11_menus…`](aseprite/11_menus_frame_select_view_help.png) · submenus
[`12`](aseprite/12_sub_export_import_paste_rotate.png) · [`13`](aseprite/13_sub_shift_adjust_fx_colormode_canvas.png) ·
[`14`](aseprite/14_sub_layer_frame_tags_select.png).

| Menu | Itens (atalho padrão) |
|---|---|
| **File** | New (Ctrl+N) · Open (Ctrl+O) · Open Recent › · Save (Ctrl+S) · Save As (Ctrl+Shift+S) · Close (Ctrl+W) · Close All · **Export ›** Export As (Ctrl+Alt+Shift+S) · Export Sprite Sheet (Ctrl+E) · Export Tileset · Repeat Last Export (Ctrl+Shift+X) · **Import ›** Import Sprite Sheet (Ctrl+I) · Exit |
| **Edit** | Undo/Redo (Ctrl+Z/Y) · Undo History · Cut/Copy/**Copy Merged**/Paste · **Paste Special ›** as New Sprite · as New Layer · as New Reference Layer · Delete · **Fill (F)** · **Stroke (S)** · Rotate › 180/90 CW/90 CCW · Flip H (Shift+H) · Flip V (Shift+V) · **Transform (Ctrl+T)** · **Shift ›** Left/Right/Up/Down · New Brush (Ctrl+B) · New Sprite From Selection · Replace Color (Shift+R) · Invert · **Adjustments ›** Brightness/Contrast · Hue/Saturation (Ctrl+U) · Color Curve (Ctrl+M) · **FX ›** Outline (Shift+O) · Convolution Matrix (F9) · Despeckle (Median) · Insert Text · Keyboard Shortcuts (Ctrl+Alt+Shift+K) · Preferences (Ctrl+K) |
| **Sprite** | Properties (Ctrl+P) · **Color Mode ›** RGB · Grayscale · Indexed · More Options · Duplicate · Sprite Size (Ctrl+Alt+I) · Canvas Size (C) · **Rotate Canvas ›** 180/90 CW/90 CCW/Flip Canvas H/V · Crop · Trim |
| **Layer** | Properties (F2) · Visible (Shift+X) · Lock Layers · Open Group (Shift+E) · **New ›** Layer (Shift+N) · Group (Alt+Shift+N) · Layer via Copy (Ctrl+J) · Layer via Cut (Ctrl+Shift+J) · Reference Layer from File · Tilemap Layer (Space+N) · Delete · **Convert To ›** Background · Layer · Tilemap · Duplicate · Merge Down · Flatten · Flatten Visible |
| **Frame** | Frame Properties (P) · Cel Properties · New Frame (Alt+N) · New Empty Frame (Alt+B) · Duplicate Cel(s) (Alt+D) · Duplicate Linked Cel(s) (Alt+M) · Delete Frame (Alt+C) · **Playback ›** Play (Enter) · Play Preview (Shift+Enter) · Speed 0.25×/0.5×/1×/1.5×/2×/3× · Play Once · Play All Frames (Ignore Tags) · Play Subtags & Repetitions · Rewind on Stop · **Tags ›** Tag Properties · New Tag · Delete Tag · **Jump to ›** First (Home)/Prev (←)/Next (→)/Last (End)/First in Tag/Last in Tag/Go to Frame (Alt+G) · Constant Frame Rate · Reverse Frames (Alt+I) |
| **Select** | All (Ctrl+A) · Deselect (Esc) · Reselect (Ctrl+Shift+D) · Inverse (Ctrl+Shift+I) · **Color Range** · **Modify ›** Border/Expand/Contract · Load/Save MSK file |
| **View** | Duplicate View · Workspace Layout · Run Command (Ctrl+Space) · Extras · Show › · Grid › · Tiled Mode › · Symmetry Options · Set Loop Section · Show Onion Skin · Timeline (Tab) · Preview · Advanced Mode · Full Screen (F11) · Full Screen Preview · Home · Refresh & Reload Theme |

⚠️ **Advanced Mode** [app]: o layout por omissão esconde a timeline; `Tab` mostra-a
([`32_timeline.png`](aseprite/32_timeline.png): play/navegação, colunas olho·cadeado·continuous, cel = ponto).

## §2 — Ferramentas [app]

Barra (de cima para baixo) e sub-ferramentas (clique-e-segura,
[`21_subferramentas.png`](aseprite/21_subferramentas.png)):

| # | Grupo | Sub-ferramentas | Barra de contexto [app] ([`20_…`](aseprite/20_barra_de_contexto_por_ferramenta.png)) |
|---|---|---|---|
| 1 | Seleção | Rectangular Marquee (M) · Elliptical · Lasso (Q) · Polygonal Lasso · Magic Wand (W) | modo (replace/add/sub/intersect) · tolerância/contíguo (varinha) [doc] |
| 2 | Pencil | Pencil (B) · Spray | forma do pincel · tamanho · **ink** · dinâmica · **Pixel-perfect** |
| 3 | Eraser (E) | — | idem |
| 4 | Eyedropper (I; Alt rápido) | — | **Pick:** Color+Alpha… · **Sample:** All Layers / Current / First Reference |
| 5 | Zoom (Z) | — | 100% · Center · Fit Screen |
| 6 | Move (V; Ctrl rápido) | — | Auto Select Layer |
| 7 | Paint Bucket (G) | Bucket · Gradient [doc] | **Tolerance** · **Contiguous** · conectividade 4/8 · refer layer/visible |
| 8 | Line (L) | Line · Curve (Shift+L) | tamanho · ink |
| 9 | Rectangle (U) | Rectangle · Filled Rectangle · Ellipse (Shift+U) · Filled Ellipse | tamanho · raio de canto (rounded, 1.3.17) · ink |
| 10 | Contour (D) | Contour · Polygon (Shift+D) | — |
| 11 | Blur | Blur · Jumble | tamanho · opacidade · pixel-perfect |
| 12 | Text (T) | — | fonte · tamanho · B/I · antialias · fill/stroke |
| — | Hand (H; Space rápido) · Slice (Shift+C) | sem ícone visível no layout padrão [NC] | — |

**Inks (RGB) [app]** ([`30_ink_pincel_paleta.png`](aseprite/30_ink_pincel_paleta.png)): **Simple** ·
**Alpha Compositing** · **Copy Alpha+Color** · **Lock Alpha** · **Shading** · «Same in all Tools».
⚠️ São **5**, não os ~10 «de memória» (Merge/Hue/Saturation…) — a pesquisa de doc errou aqui; o app manda.

**Modificadores [doc]:** Shift = reta desde o último ponto (pencil) / quadrado-círculo (formas) ·
Ctrl = snap de ângulo / a partir do centro · Alt = rotaciona · Space = move a origem · botão
esquerdo = cor de frente, direito = cor de fundo, `X` troca.

**Simetria [doc]:** horizontal, vertical, **diagonal** (1.3.11), eixo arrastável que pode ficar a
meio pixel. **Dinâmica [doc]:** pressão/velocidade → tamanho, ângulo, gradiente; **stabilizer**.

## §3 — Documento, camadas, frames [doc + app]

- **Modo de cor por sprite:** RGBA · Grayscale · **Indexed** (índice transparente configurável).
  Conversão RGB→Indexed: dithering none / ordered (Bayer 2/4/8 ou matriz) / old.
- **Sprite Size** [app]: px ou %, Lock Ratio, **Interpolation › Nearest-neighbor** (+ outros [NC]).
- **Camadas:** Background (opaca, única, fundo) · normal · **grupo** (colapsável; blend/opacidade) ·
  **referência** · **tilemap**. Propriedades [app] ([`31_…`](aseprite/31_camada_tamanho_folha_prefs.png)):
  Nome · **Mode** (blend) · **Opacity**. Blend no ficheiro: ids 0..18 (Normal…Divide). Visível,
  lock, **solo** (1.3.16), continuous («prefer linked cels»), cor, user data.
- **Frames:** duração própria em ms; **cel** = camada × frame com posição, opacidade, **z-index**
  (1.3); **cel linkado** (partilha imagem); layer continuous liga cels ao copiar.
- **Tags:** nome, cor, intervalo, direção **Forward/Reverse/Ping-pong/Ping-pong Reverse**, **repeat**
  (0 = infinito na UI, 1× no export), subtags e grupos de tags (1.3.18).
- **Onion skin (F3):** N anteriores/N seguintes, modo clássico vermelho/azul [doc]; outros modos [NC].
- **Slices:** retângulo com nome, **9-slice center**, **pivot**, keys por frame, exporta em `meta.slices`.
- **Tilemaps:** tileset (tile 0 = vazio, flags de flip X/Y/D); modos Draw Pixels **Manual/Auto/Stack**
  (Space+1/2/3) e **Draw Tiles**; Space+Tab alterna.
- **User data:** texto, cor e propriedades tipadas em sprite/layer/cel/tag/slice/tileset.
- ❌ **Não existe** [doc]: tween/interpolação, áudio, auto-outline como efeito de camada, smudge.

## §4 — Paleta e cor [app]

Menu da paleta ([`30_…`](aseprite/30_ink_pincel_paleta.png)): **Edit Palette (F4)** · **Palette Size** ·
Select › · Small/Medium/Large Size · seletor **Color Tint/Shade/Tone** · **Color Spectrum** ·
**RGB Color Wheel** · **RYB Color Wheel** · ⭐ **Normal Map Color Wheel** · Load/Save Palette ·
Save as Preset · Save as Default · **New Palette from Sprite**. Presets num botão próprio.
Barra de cor: hex da cor de frente/fundo. Editar o índice em Indexed muda o desenho.

## §5 — Export / Import [app + CLI]

- **Export Sprite Sheet** [app] ([`31_…`](aseprite/31_camada_tamanho_folha_prefs.png)): abas
  **Layout · Sprite · Borders · Output**; Sheet Type (By Rows…), Constraints, Merge Duplicates,
  Ignore Empty, Size=2ⁿ, Preview, Open Sprite Sheet.
- **CLI** [app] ([`40_cli_help…`](aseprite/40_cli_help_1.3.18.6.txt)) — é a lista completa das opções
  de folha: `--sheet-type horizontal|vertical|rows|columns|packed`, `--sheet-width/height/columns/rows`,
  `--power-of-two-size`, `--split-layers|tags|slices|grid`, `--layer`/`--ignore-layer`/`--all-layers`,
  `--tag`, `--play-subtags`, `--frame-range`, `--ignore-empty`, `--merge-duplicates`,
  `--border/shape/inner-padding`, `--trim`/`--trim-sprite`/`--trim-by-grid`, `--extrude`, `--crop`,
  `--slice`, `--filename-format`, `--tagname-format`, `--data` + `--format json-hash|json-array`,
  `--color-mode`, `--dithering-algorithm none|ordered|old`, `--dithering-matrix bayer2x2|4x4|8x8|png`,
  `--scale`, `--shrink-to`, `--palette`, `--list-layers|layer-hierarchy|tags|slices`.
- Formatos [doc]: `.ase/.aseprite` (tudo), PNG, GIF, sequência numerada, QOI, JPEG, WebP/APNG [NC];
  `.psd` só por extensão.

## §6 — UI e ergonomia [app + doc]

Preferências (Ctrl+K, com busca) [app]: General · Files · Color · Alerts · Editor · Selection ·
Timeline · Cursors · Background · Grid · Guides & Slices · Undo · Theme · Extensions · Reset.
Escala de ecrã 200% + escala de UI. Atalhos editáveis (`user.aseprite-keys`), **Run Command**
(Ctrl+Space), painéis dockáveis (1.3.16), layouts de workspace, Data Recovery (sessões).

## §7 — O que os artistas amam e o que pedem [doc/fórum; amostra pequena]

- **Amado:** timeline com onion skin e tags · pencil pixel-perfect · ink Shading · pincel
  personalizado · paleta/indexed · RotSprite · Tiled Mode · CLI + export de folha.
- **Pedido (não existe):** áudio na animação · auto-outline como ajuste de camada · animar
  posição/rotação entre frames (tween) · cel linkado com offset próprio · símbolos estilo Flash ·
  editar pivot/9-slice arrastando · pattern brush aleatório · smudge · color cycling.

⇒ Metade desta lista é exatamente o que a PH2D já tem noutros módulos (áudio, timeline/expressões,
Motion, componentes) ou no MVP (cycling, rampas). É o argumento de produto do §P13 do plano.

## §8 — Lacunas a fechar no app (antes da onda que as usa)

Lista de blend modes do diálogo · métodos de interpolação do Sprite Size · modos de onion skin ·
formatos de paleta aceitos · algoritmo do pixel-perfect (o MVP já tem o nosso, transcrito de
Ricky Han) · onde ficam Hand/Slice no layout · opções exatas de Color Range e Outline.
Receita: Xvfb `:77` + `xdotool` + `import -window root -crop …`.
