# P10 Tilemaps — rodada de oráculos (06/11, M1–M5)

Só medido; o que não foi medido está declarado em cada secção. Ferramentas em `docs/PixelArt/ferramentas/` (`pixelorama_tiles/`, `oraculo_godot_tiles.{py,gd}`, `trial_tiles.py`, `trial_tiles_le.py`). Fixturas com cabeçalho em `crates/ph2d-pixel-tiles/tests/fixtures/oracle_*.json`. Nada do código dos oráculos foi lido: só corridos (Pixelorama pela API pública/reflexão, Godot `--headless`, Trial por fotografia).

## M1 — Pixelorama 1.2.3: Manual / Auto / Stack (a extensão CONDUZ)

Mecanismo: extensão `TilesProbe` (HOME isolado, Xvfb :94). Camada de tilemap construída por `LayerTileMap.new` + `TileSetCustom.new`, modo posto pela variável estática `TileSetPanel.tile_editing_mode` (0 Manual, 1 Auto, 2 Stack; omissão do app = Auto), traço por `Tools.handle_draw` (Pencil 1 px azul / Eraser). Conduziu à 5.ª corrida; as anteriores falharam por 2 erros meus de montagem (célula do tileset como objeto `Cell`; e `selected_cels` não seguia a camada nova, o toque caía na camada 0). Fixtura: `oracle_pixelorama_modos.json` (42 estados antes/depois).

Mapa: 4×4 células, tile 8×8. tile1 = 1 px vermelho em (0,0) usado em 3 células; tile2 = 1 px verde usado em 1 célula; tile0 vazio. Cada cenário é um projeto novo.

| Cenário (traço) | Manual | Auto | Stack |
|---|---|---|---|
| Célula vazia (tile 0), toque | nada muda | nasce tile3 (1 px azul), a célula passa a 3 | nasce tile3, a célula passa a 3 |
| Tile usado 1×, toque | o tile2 é editado NO LUGAR (2 px) | idem, no lugar | tile2 intacto (fica com `times_used` 0, NÃO é apagado); nasce tile3 com o desenho; a célula passa a 3 |
| Tile usado 3×, toque numa cópia | o tile1 é editado no lugar: as 3 cópias mudam | nasce tile3 só para essa célula; as outras 2 ficam com tile1 (`times_used` 3→2) | idem Auto (nasce tile3, tile1 passa a 2 usos) |
| Apagar o único pixel do tile usado 1× (Eraser) | sem mudança observada | célula volta a 0 e o tile2 é REMOVIDO do conjunto (a lista encolhe) | sem mudança observada |
| Apagar o único pixel numa cópia de tile usado 3× (Eraser) | sem mudança observada | a célula volta a 0, tile1 fica com 2 usos | sem mudança observada |
| Dois toques iguais em duas células vazias | nada | UM tile3, `times_used` 2, as duas células apontam para ele (DEDUPLICA) | UM tile3 com `times_used` 2 (deduplica); o tile0 perde 1 uso (0) |

Ressalvas: (a) Manual e Stack com Eraser não mudaram nada nesta sonda — pode ser que o apagar só entre em Auto, ou artefacto do meu toque; só o vi dar «nada» (não testei pincel em modo de posicionar tiles, `place_only_mode`/`Tools.is_placing_tiles`, nem traços com vários pixels/arrasto). (b) Um só desenho por cenário, 1 px. (c) A deduplicação é por imagem idêntica entre tiles existentes; em Manual não nasce tile nenhum, logo não há o que deduplicar. (d) O modo inicial do app foi lido do estático: 1 (Auto).

Pixelorama também foi usado para M3 (abaixo).

## M2 — Godot 4.7.2 (MIT): as 47 máscaras e as 256 vizinhanças (`oracle_godot_autotile.json`)

| Medido | Resultado |
|---|---|
| Máscaras válidas pela regra «um canto só conta com as duas arestas dele» | 47 |
| 256 vizinhanças do centro (8 vizinhos ligados/desligados, resto do mapa vazio), conjunto completo de 47 | 256/256 iguais à máscara prevista (0 diferem), gravadas por vizinhança |
| Mapa 376 células com buracos e corredores de 1 célula, encostado a x=0 e y=0 e com coordenadas negativas, 44 máscaras distintas exercitadas | 376/376 iguais (0 diferem). Godot não tem «borda do mapa»: fora do mapa = vazio, igual à regra |
| Conjunto sem o tile isolado (46 tiles) | 8 células pediam o isolado; Godot NUNCA deixa a célula vazia: pôs um tile com 1 bit A MAIS (um lado que o mapa não tem). 368 outras iguais |
| Conjunto só de lados (16 tiles, sem cantos) | 178 células pediam cantos: o escolhido é sempre subconjunto da máscara pedida, 1 a 4 bits de distância (109 / 51 / 15 / 3); nunca vazio. 198 iguais |
| Conjunto sem o miolo 15/15 | 3 células pediam o miolo: escolheu 15/13 (perde um canto) |
| Conjunto menos 5 tiles sorteados (42) | 38 células pediam um tile que falta (escolhas a 1 ou 2 bits, nunca vazio, para cima OU para baixo); e 35 células cuja máscara EXISTE receberam outra: a falta de tiles propaga-se aos vizinhos, o resultado não é local |

Não medido: terrenos múltiplos, `MATCH_CORNERS` e `MATCH_SIDES` puros, a regra de desempate entre candidatos (só vi os resultados); mapas acima de 400 células.

## M3 — Centro e volta, iso e hex

Godot (`oracle_godot_grelhas.json`): 4 formas × 6 `tile_layout` × 2 eixos × 4 tamanhos (16×16, 15×15, 32×16, 33×17) = 192 configurações, 25 células cada. `local_to_map(map_to_local(c)) == c` em 192/192; 4 pontos a ±1/4 do tile à volta do centro ficam na mesma célula em todas. Centros são fracionários em tamanho ímpar (15×15: (7.5, 7.5); 33×17: (16.5, 8.5); hex 15×15 `stacked` célula (0,1) = (15.0, 18.75), ou seja 3/4 da altura, não arredondado).

Pixelorama (`oracle_pixelorama_grelhas.json`): 4 formas × 2 eixos × 4 tamanhos, `get_pixel_coords(célula)` e `get_cell_position(pixel + tile/2)`: volta exata 25/25 em todas. Os pixels são inteiros (15 → 7, divisão inteira) e a convenção de iso/hex NÃO coincide com a do Godot (Pixelorama iso 16×16: (0,0)→(8,8), (1,0)→(16,16), (0,1)→(0,16); quadrada: (n·w, n·h)). Não medi `tile_layout` no Pixelorama, e não confirmei se `get_pixel_coords` devolve o canto ou o centro em iso/hex. Comparação com `ph2d-grid` NÃO feita aqui (a crate não é minha).

## M4 — Trial abre o nosso `.ase` com tilemap e flips (`oracle_trial_flips.json`)

- Escritor Python pela spec pública: tileset 0x2023 (flags 2|4, tile 0 vazio), layer type 2, cel type 3 (32 bits, máscaras 0x1fffffff / X 0x80000000 / Y 0x40000000 / D 0x20000000). Tile 4×4 assimétrico de 16 cores distintas, mapa 4×2 com as 8 combinações.
- O Trial 1.3.18.6 abre sem erro com a ordem de chunks tileset → layer → cel. **Com o chunk da camada ANTES do tileset dá «Error: tileset 0 not found»** (foto não guardada; o caso foi só visto uma vez).
- Lido pela tela a 800 %: os 128 pixels batem com um pixel do tile (0 fora da paleta). A ordem que o Trial aplica é **D primeiro (troca x,y), depois X e Y** — igual à hipótese do plano. Ordens testadas e descartadas por pelo menos uma combinação: XYD, YXD, XDY, YDX (DXY e DYX coincidem, X e Y comutam). As combinações que distinguem: (X,D), (Y,D), (X,Y,D). Foto: `handoffs/P10_flips_trial_800pct.png`.
- Não medido: ficheiro com tile 0 não vazio (flag 4 desligada), `tile_id` acima do tileset, mapa maior que o canvas, cels de tilemap com bits por tile ≠ 32, tilesets externos/links.

## M5 — Trial: o que se vê (nada gravável; só leitura pela tela)

- **New Tilemap Layer** (Layer ▸ New ▸ New Tilemap Layer, atalho Space+N): diálogo «New Layer» com Name («Tilemap 1»), Tileset (lista: «New Tileset» ou «Tileset #0 (4×4): "ts"» o já existente), Name do tileset, Grid Width, Grid Height (vinham 4×4 nesta sessão), caixa «Advanced Options» que abre Base Index (1) e Allowed Flips X, Y, D (botões, nenhum aceso por omissão). Fotos: `handoffs/P10_trial_new_tilemap_layer.png`.
- **Modo inicial**: Auto — o botão do meio ficou aceso tanto ao abrir o nosso `.ase` como logo depois de criar a camada. Atenção: usei um HOME isolado com cópia do `aseprite.ini` do dono; um estado guardado nesse ini não está excluído. Tooltips (foto `P10_trial_modos_tooltips.png`): Manual «modify existing tiles, don't create new tiles automatically (Space+1)»; Auto «modify and reuse existing tiles, create/delete tiles if needed/possible (Space+2)»; Stack «don't modify existing tiles, generate and stack new tiles automatically (Space+3)». O primeiro botão é «Show Tileset / Right-click to Show Only the Tileset». Tile 0 aparece como xadrez (vazio).
- **Menus com camada de tilemap selecionada (só vi ativo/cinza)**: ATIVOS — Edit: Rotate, Flip Horizontal (Shift+H), Flip Vertical (Shift+V), Transform, Shift, Fill, Stroke, Replace Color, Invert, Adjustments, FX; Sprite: Properties, Color Mode, Duplicate, Sprite Size, Canvas Size, Rotate Canvas, Trim; Layer: Flatten, Flatten Visible, Duplicate, Delete Layer (com 2 camadas); Select: All, Inverse. CINZA — Cut/Copy/Paste, Deselect, Crop, Color Range (sem seleção). **Merge Down: cinza** com «Tilemap 1» (vazia) sobre «tm» (tilemap), e cinza com camada única; não o testei com uma camada normal por baixo.
- **O que fizeram**: só Edit ▸ Flip Horizontal e Flip Vertical foram executados (sobre «tm», sem seleção): a tela mudou de forma forte e estranha (um retângulo cinzento com uma faixa de pixels), sem diálogo de erro; desfeito com Ctrl+Z e a imagem voltou ao original. NÃO interpretei esse resultado. Nenhum filtro, Rotate Canvas, Sprite Size, Canvas Size, moldura de seleção (nem selecionar com a moldura), troca de modo de cor ou Flatten foi EXECUTADO: só vistos ativos/cinza. Dois `Ctrl+Z` seguidos desfizeram também a criação da camada «Tilemap 1» (o desfazer conta a criação).

## O que ficou por medir
M1: Eraser em Manual/Stack, arrasto de vários pixels, modos de «posicionar tiles»; Pixelorama com camadas iso/hex e `tile_layout`. M2: terrenos múltiplos e desempate. M3: `ph2d-grid`, layout no Pixelorama. M4: casos de borda do formato. M5: execução real de filtros, Rotate/Flip Canvas, Sprite/Canvas Size, moldura, Color Mode, Flatten, Merge Down com camada normal; qual o modo ao abrir com `aseprite.ini` limpo. M6–M8 não eram desta rodada.
