# Fixture — a BATERIA do EXAME no ArtRage 6 (oráculo sob Wine)

| campo | valor |
|---|---|
| app | **ArtRage 6 Demo 6.1.2.0** (Windows, `ArtRage Demo.exe`, FileVersion 6.1.2.0) |
| instalador | `install_artrage_6_demo_windows.exe` (Advanced Installer), sha256 `95f1d58b8627a5be82f048302d8a625be480112fd79357790b3e852b1d50dcaa`, de `https://s3.amazonaws.com/artrage_public/install_artrage_6_demo_windows.exe` (o link da página oficial `artrage.com/demo`) — fica em `~/Apps/artrage/instalador/`, FORA do repo |
| Wine | wine 11.19 do sistema, prefixo próprio `~/Apps/artrage/wineprefix`, tela **Xvfb `:71`** 1920×1080 (só software: Mesa llvmpipe, nenhum ICD Vulkan ⇒ não toca a placa nem a tela do Rebelle) |
| data | 2026-10-19 |
| licença / EULA | Demo **sem prazo** («There is no time limit on the demo software»), sem conta; NÃO grava, NÃO exporta e NÃO grava scripts (os três recusados no próprio app); «may only be used for evaluation purposes». O nosso uso — correr o app sobre entradas nossas e fotografar o ecrã para referência visual — é avaliação. ⛔ Nenhum ficheiro do ArtRage (preset, grão, textura) entra no repo: os presets são lidos do prefixo no momento da corrida |
| expira | **não expira** |
| entrada | `docs/Painter/Pinceis/ferramentas/bateria.json` (a fonte única; 3 cenas 1000×640) |
| comando | `bash ~/Apps/artrage/inicia.sh` e depois, na raiz `docs/Painter/Pinceis/`: `python3 -I ferramentas/fotos_artrage.py ferramentas/lista_artrage.tsv fixtures/artrage_bateria` (≈ 26 s por cena; 105 fotos ≈ 48 min) |
| saída | `<Ferramenta>/<preset>_<cena>_A.png` — a pintura 1000×640 recortada do ecrã (`import -window root -crop 1000x640+460+235`), 8 bit sRGB |

## A porta: o SCRIPT do próprio app (`.arscript`)

O Demo **toca** scripts (File > Play Script…); só GRAVAR é recusado. Um script é texto UTF-16 com cada traço como
`<StrokeEvent>` e cada ponto com `Loc:` e `Pr:` (pressão) — os pontos EXACTOS da bateria entram sem rato nem caneta
simulados. O conversor é `ferramentas/bateria_artrage.py` (o formato e o que foi confirmado estão no docstring).

| op da bateria | no ArtRage |
|---|---|
| `pincel` | `CID_ToolSelect` + `Reset Current Tool` (+ `ToolPreset` com os bytes do bloco 5 do `.prs` instalado) |
| `cor` | `SetForeColour` `Pixel` `0xFFRRGGBB` |
| `traco` | `CID_SetSpecificToolSize` `real` = px/k (tabela abaixo) + `StrokeEvent`, 8 ms entre eventos (125 Hz), Accelerate Playback DESLIGADO |
| `seca` | `Add New Layer` — o ArtRage não tem secagem nem tempo: tinta numa camada nova não arrasta a de baixo |
| `simula` | (nada: nada evolui no tempo) ⇒ **foto B ≡ foto A**, só se guarda a A |

## Tamanho: px = k·s, MEDIDO (pressão 0,999, largura onde a tinta passa 25 % do máximo)

| ferramenta (id) | s → px medidos | k usado |
|---|---|---|
| Ink Pen (4917) | 0,02→3 · 0,16→23 · 0,64→95 · 1→149 · 2→299 | 149 |
| Oil Brush (4900) | 0,16→15 · 0,64→61 · 1→95 · 2,5→236 | 95 |
| Crayon (4902) | 0,16→13 · 0,64→43 · 1→67 · 3→196 | 66 |
| Felt Pen (4904) | 0,32→15 · 0,64→30 · 2→94 · 4→187 | 47 |
| Airbrush (4914) | 0,32→35 · 1→121 · 2→256 | 124 |
| Paint Roller (4915) | 0,04→17 · 0,25→101 · 0,5→199 | 400 |
| Watercolor (4916) | 0,64→35 · 2→107 · 4→217 | 54 |
| Pencil (4901) | 1→5 · 4→17 · 16→41 · 30→41 · 60→41 | tabela `1=5,4=17,16=41` — **o lápis satura em 41 px**: o «200» dele é 41 |
| Custom Brush «Bristle Round» (4925) | 0,16→23 · 0,64→86 | 134 (usado para os 7 Custom Brush) |

## O que foi medido e não se repete

| achado | prova |
|---|---|
| **Pressão 1,0 EXACTA corta o traço** (Crayon s=0,32 pára aos ~80 pontos; Felt Pen s=0,64 sai curto e claro); 0,9 e 0,999 pintam inteiro — o ArtRage lê `Pr: 1` como rato | `t_cut`…`t_cut6` (19/10) ⇒ teto 0,999 no conversor |
| O traço curto do recto de 5 px no Oil Brush é do PRÓPRIO óleo (a tinta acaba num pincel fino), não da porta | as outras ferramentas pintam o mesmo recto inteiro |
| **Chalk**: nenhum id achado pela porta — `4903` dá fotos **byte a byte iguais** às do Crayon (4902); `4907` pinta como o Oil Brush; `4910–4912`, `4924` não pintam nada | por isso a pasta `Chalk/` saiu; o giz do ArtRage fica por fazer (a alternativa é escolher a ferramenta pelo clique na roda, fora do script) |
| Arrasto do rato (`xdotool`) não serve: o tamanho volta ao do utilizador quando o script acaba e o alisamento persegue o cursor (cauda até ao `mouseup`) | `t_cut5` |
| O Palette Knife só se fotografa na cena `sobre` (base = Oil Brush de fábrica): em papel limpo não há o que arrastar | `lista_artrage.tsv`, coluna `cenas` |
| O papel é o de omissão do ArtRage (grão leve, «Canvas» de fábrica) — o grão faz parte do lápis/crayon dele | — |

## Os 35 pincéis (105 fotos)

- **OilBrush**: default · ThickGloss · DryClumps · ThinAndDry · NormalSquare
- **PaletteKnife** (só `sobre`): default · JustBlendColor · HardWetBlender
- **Pencil**: default · BShading · HardAndDark
- **Crayon** (as «Pastel» do ArtRage são presets desta ferramenta): default · HeavyPastel · LightPastel · SolidCrayon
- **FeltPen**: default · SoftMarker · BlendingMarkers
- **InkPen**: default · ThickAndThin · WritingChisel · TechnicalPen
- **Watercolor**: default · WetOnWet · HeavyBleed · DelicateOnDry · DryOnWet
- **Airbrush**: default · SlowFlow
- **PaintRoller**: default
- **CustomBrush** (Artistic): BristleRound · StrongCharcoal · FaintCharcoal · SmearedInk · Watery1 · Pastel · SimplePaint

A lista que os gera é `ferramentas/lista_artrage.tsv` (404 presets existem na instalação — `Content/Tool Presets/`;
acrescentar uma linha basta para fotografar outro).
