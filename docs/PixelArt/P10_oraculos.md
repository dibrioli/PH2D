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

## M9 — Casos de borda (06/11): borracha, traço sobre cópias, encolher, vida do tileset

Só medido. Fixtura: `oracle_tile_edges_m9.json`. Ferramentas: `pixelorama_tiles_m9/`, `trial_tiles_m9*.py`, `tiled_m9/`. Pixelorama 1.2.3 pela extensão (`Tools.handle_draw`), repetido por ponteiro real em 5 casos: mesmo resultado; o ecrã de abertura comia o ponteiro na 1.ª tentativa. Trial 1.3.18.6: HOME limpo, uma sessão por caso, leitura por foto (id da célula inferido pelo bitmap). O Trial perde o último movimento antes do largar; o traço agora assenta no fim, e a 1.ª corrida dos arrastos foi descartada.

**M1 corrigido:** a borracha em Manual/Stack não era defeito da sonda em parte: apagar SÓ PARTE funciona; apagar TUDO muda o canvas sem mudar tile nem célula, e M1 não olhava o canvas.

**Q1. Borracha.** Tile «1×» e «3×» no mesmo mapa; `Ctrl+Z`/undo 1× restaura tudo no Trial; na Pixelorama o histórico conta 1 passo.

| Caso | Modo | Pixelorama | Trial |
|---|---|---|---|
| a 1×, um pixel | Manual, Auto | tile editado no lugar | igual |
| a | Stack | tile novo, célula aponta para ele, original fica (0 usos) | igual |
| b 1×, todos | Manual | tile INTACTO, célula mantém o id, canvas vazio (dessincroniza; persiste ao gravar e reabrir `.pxo`). Lápis seguinte na célula: o tile passa a ter só esse pixel | tile fica VAZIO no conjunto (3→3); lápis seguinte na célula edita esse tile |
| b | Auto | célula 0, tile REMOVIDO, ids seguintes descem | igual (3→2) |
| b | Stack | tile intacto, célula mantém o id, canvas vazio (dessincroniza). Lápis seguinte: tile novo de 1 px | célula vazia, tile fica (3→3) |
| c 3×, um pixel | Manual | tile editado, as 3 cópias mudam | igual |
| c | Auto, Stack | tile novo só nessa célula, original fica (2 usos) | igual |
| d 3×, todos | Manual | tile INTACTO, canvas perde as 3 cópias (dessincroniza). Lápis seguinte nessa célula: não muda nada | tile VAZIO, as 3 cópias somem; lápis seguinte aparece nas 3 |
| d | Auto | célula 0, tile fica (2 usos) | igual |
| d | Stack | célula mantém o id, canvas só dessa célula vazio, tile fica (dessincroniza). Lápis seguinte: tile novo | célula vazia, tile fica |

Não medido: borracha com pincel grande, id real da célula no Trial.

**Q2. Um traço sobre duas cópias.** Pixelorama e Trial, tile `T` = t1.

| Caso | Modo | Resultado | Acordo |
|---|---|---|---|
| a 2×, mudanças diferentes (e o traço ao contrário) | Auto | célula ESQUERDA → tile novo; célula DIREITA → `T` editado no lugar. Total 2 tiles além do 0 | iguais; **é o que o PH2D faz hoje** (mas a ordem é a da posição, não a do traço) |
| a | Stack | as duas → tiles novos, `T` fica (0 usos) | iguais |
| b 2×, iguais | Auto | as duas continuam no id 1 com a imagem nova; 0 a mais. ⚠️ «No lugar» era inferência errada: num mapa de 1 tile não se distingue de remover e criar, e M9b mostra que é remover e criar | iguais |
| b | Stack | 1 tile novo partilhado, `T` fica | iguais |
| c 3×, duas no traço, diferentes | Auto, Stack | 2 novos, `T` fica na 3.ª | iguais |
| c2 3×, duas no traço, iguais | Auto, Stack | 1 novo partilhado, `T` fica na 3.ª | iguais |
| a, c, diferentes | Manual | Trial: `T` recebe a UNIÃO. Pixelorama: `T` recebe só a mudança da célula esquerda; a da direita perde-se | DIFEREM |

Não medido: traço vertical ou diagonal (a ordem por posição só foi vista na horizontal).

**Q3. Encolher e crescer.**

| Caso | Pixelorama | Trial | Tiled (`map.resize`) |
|---|---|---|---|
| 32→16→32 | células voltam | voltam (Trim desligado, a omissão) | em tiles, 4→2→4: células PERDIDAS; undo restaura |
| Com «Trim content outside the canvas» | sem esta opção à vista | células fora PERDIDAS; 2× undo restaura | n/a |
| Não múltiplo (20×20) | aceita, volta | aceita, volta | n/a (unidade é o tile) |
| Deslocar 8 e 3 px | mapa anda (`offset` 8, 3) | mapa anda, igual | 2 tiles: células movem-se; `-1` e volta perde as de fora |
| Crop à selecção 8 e 3 px | aceita, células fora ficam | aceita; crescer de volta devolve tudo | n/a |
| Crop ao conteúdo / Trim | 30×29 | 30×29 | n/a |

Não medido: pintar depois de um deslocamento não múltiplo; o diálogo Resize Map do Tiled (só o script).

**Q4. Vida do tileset.** Pixelorama pela API (funções do menu, sem clique); Trial lido na lista de New Tilemap Layer.

| Última camada… | Pixelorama | Trial |
|---|---|---|
| apagada | tileset fica, oferecido, sobrevive a gravar e reabrir | AVISA «Deleting the following tilemaps will delete their tilesets»; OK: tileset SOME da lista; undo devolve |
| apagada, com outra no mesmo tileset | não corrido | sem aviso, tileset fica |
| achatada | tileset fica, oferecido, sobrevive a reabrir | sem aviso, fica e oferecido com os 4 tiles |
| convertida | não há «Convert To»; Flatten só nela: camada normal, tileset fica, sobrevive a reabrir (a camada de baixo também entrou no achatado) | Convert To ▸ Layer: sem aviso, fica e oferecido com os 4 tiles |

Merge Down, Trial: cinza com tilemap sobre camada normal. Tiled: remover a camada deixa o tileset no `.tmx` gravado. Não medido: gravar no Trial (não grava), logo não se sabe se o tileset sem camada sobrevive a um ficheiro.

**Q5.** Tile usado 1× em cada uma de 2 camadas, Auto e Stack: as duas apps contam 2 usos, nasce tile novo, a outra camada não muda. Manual: as duas camadas mudam. New Tilemap Layer com tileset existente: vem «New Tileset» nas duas apps. Trial com HOME limpo: abre em Auto (fecha a ressalva de M5); os campos de grelha vêm do tileset, ou 16×16 depois de achatar ou converter.

Só na documentação pública: [Aseprite, Tilemap](https://www.aseprite.org/docs/tilemap/): Auto «will erase unused tiles if they are not found in any tilemap referencing the tileset»; Stack «will create a new tile for each modification»; nada sobre apagar tudo, canvas ou vida do tileset. [Tiled, TileMap](https://www.mapeditor.org/docs/scripting/classes/TileMap.html): `resize(size, offset?, removeObjects?)`, só em mapas vindos de ficheiro (confirmado ao correr).

Fotos: `handoffs/P10_m9_*.png`.

### M9b — ordem da lista e quem guarda o original (Auto)

A=1 (usado 2× ou 3×), B=2, C=3, cada um destes dois 1×. Fixtura: chave `m9b`. Ferramentas: `pixelorama_tiles_m9/` (modo `m9b`), `trial_tiles_m9b.py`. Pixelorama lê ids pela API; Trial infere pelo bitmap e pela ordem do painel. Cada caso correu nos dois sentidos, exceto o 1, e o sentido não mudou nada.

| Caso | Pixelorama | Trial |
|---|---|---|
| 1. A 2×, iguais | lista B=1, C=2, novo=3; células A→3, B→1, C→2 (A removido, ids descem) | igual |
| 2. A 2×, diferentes, horizontal | (0,0)→4 no fim; (1,0) fica 1, editado; B, C mantêm | igual |
| 3. A 2×, vertical | (0,0)→4; (0,1) fica 1, editado | igual |
| 3. A 3×, vertical | (0,0)→4, (0,1)→5; a 3.ª fica 1, intacto | igual |
| 4. A 2×, diagonal | (0,1)→4; (1,0) fica 1, editado | **(1,0)→4; (0,1) fica 1, editado** |
| 4. A 3×, diagonal | (0,1)→4, (1,0)→5; a 3.ª fica 1 | **(1,0)→4, (0,1)→5**; a 3.ª fica 1 |

As apps diferem só na diagonal: Pixelorama percorre por coluna, depois linha; Trial por linha, depois coluna. Não medido: ponteiro real nestes casos da Pixelorama; undo; caso 1 ao contrário.

## O que ficou por medir
M1: Eraser em Manual/Stack, arrasto de vários pixels, modos de «posicionar tiles»; Pixelorama com camadas iso/hex e `tile_layout`. M2: terrenos múltiplos e desempate. M3: `ph2d-grid`, layout no Pixelorama. M4: casos de borda do formato. M5: execução real de filtros, Rotate/Flip Canvas, Sprite/Canvas Size, moldura, Color Mode, Flatten, Merge Down com camada normal; qual o modo ao abrir com `aseprite.ini` limpo. M6–M8 não eram desta rodada.
