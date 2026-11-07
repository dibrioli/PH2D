# P10 — Tilemaps: o plano (06/11)

> Linha `line/PixelArt`, HEAD `286ce1ea95`, merge-base `bbfefaf393` (conta-gotas e zoom aprovados no smoke, por integrar).
> Plano do módulo: [02_plano_modulo.md §P10](02_plano_modulo.md). Estudo: [01_estudo_aseprite.md §3](01_estudo_aseprite.md). Molde: [P9_plano.md](P9_plano.md).
> Lei do módulo: [ADR-0196](../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md).
> **Estado: P10a (motor) fechada na linha em 06/11, sem tela e sem smoke; próxima: P10b (tela).** Medido: §6.1. Lista de recusas para a tela: §5.1.

## 0. O que entra

| Grupo | Peças | No Aseprite |
|---|---|---|
| **C — Camada** | camada tilemap, grelha por camada, nova e converter (imagem ⇄ tilemap, com deduplicação) | Layer › New › Tilemap, Convert To |
| **S — Tileset** | tile 0 vazio, flips X/Y/D, partilhado entre quadros, cartão com a lista de tiles | Tileset |
| **D — Desenhar** | Draw Pixels **Manual/Auto/Stack**, **Draw Tiles**, `Space+Tab`, `Space+1/2/3` | barra do tilemap |
| **T — Terreno** | **autotile 47** que resolve um mapa real (quantos terrenos: §9.2) | ❌ além do Aseprite |
| **G — Grelhas** | **iso e hex** em Draw Tiles | ❌ além do Aseprite (Pixelorama tem) |
| **E — Export** | Export Tileset, `--split-grid`/`--trim-by-grid` que a P9 adiou, mapa em JSON | Export Tileset |
| **A — `.ase`** | ler e gravar tilemap de verdade, em vez de achatar com aviso | formato |

**Fora, de propósito (salvo resposta do dono, §9.1):** objeto de tilemap no jogo, colisão por tile, regras à LDtk, mapa em chunks, import `.ldtk` e `.tmx`. Tudo isso é a Fase P do projeto Tiling (§2).

## 1. Conferido no código

| Afirmação | Conferido |
|---|---|
| O plano diz «Smoke 11» | ❌ envelheceu. `smoke.rs:34` tem `max_level: 20`, ⇒ **próxima cena 21** |
| O documento tem tilemap | ❌ `LayerKind` só tem `Image` e `Group` (`sprite.rs:163`); `Op` não tem tile (`op.rs:15-41`) |
| O leitor `.ase` entende tilemap | ✔ `file.rs`: camada tipo 2, cel tipo 3 com máscara e flips, tileset `0x2023`, `expand_tilemap` com flips |
| O app usa isso | ⚠️ `ph2d-pixel-io::abrir` achata em imagem e avisa (`ase.rs:290,353`). `ph2d-aseprite::composite` deixa a camada em branco (`composite.rs:161`) |
| O leitor está inteiro | ⚠️ `tileset` (`file.rs:474`) só usa os bits 1 e 2 dos `flags`: o bit 4 (tile 0 é o vazio) nunca é olhado, e `expand_tilemap` assume sempre o 0 |
| O escritor `.ase` grava tilemap | ❌ `ase.rs:117` só escreve `KIND_GROUP`/`KIND_IMAGE` |
| Geometria iso/hex existe | ✔ `ph2d-grid` (`iso.rs`, `hex.rs`, `staggered.rs`), em `f32`. ⚠️ `ph2d-pixel-core` não depende de nenhuma crate nossa |
| Lei do autotile 47 existe | ✔ fora do git: `docs/Pixel Art/02_spec_engine_e_porte.md §15` e `app/engine/tiles.js`. O MVP só resolve mapa booleano, sem documento (`03_dossie…:411`, o «PARCIAL») |
| Blob aceita campo novo | ✔ `codec.rs` salta etiqueta desconhecida; `an_unknown_field_is_skipped_inside_the_same_major`. ⇒ MINOR 3→4 sem `PROJECT_SCHEMA` |
| Tecto `ph2d-app-pixel` | 11 387 de 11 415: **folga 28**. Lei e tela têm de nascer fora dele |
| O `Tab` | `input_teclas.rs:179` ainda alterna a timeline. `828051653b` **não está no main**. ⚠️ `Space+Tab` mexe neste mesmo sítio: risco de conflito na integração, a anotar no handoff |
| Censo de quem toca o modelo | `Cel {` 25 literais · `LayerKind::Image/Group` 52 · `.is_group()` 30 · `.data` em `src/` de core, io e app 27 |

## 2. A fronteira com o projeto Tiling — decidida pelo dono em 06/11 (§9.1)

[`project_tilemap_belongs_to_the_tiling_project`](../../project-memory/project_tilemap_belongs_to_the_tiling_project.md): em 16/09 o dono mandou pular o tilemap da linha de componentes, porque ele tem projeto próprio em `docs/Tilling/Tiling/` (fora do git). O porte (Fase P: `ph2d-tilemap`, `TilemapRef`, render por chunk, `.ldtk`) está atrás da Fase R, **suspensa** em 31/07 e reafirmada em 19/08. A premissa do Tiling (`plan.md §0`) é: geometria vem de `ph2d-grid`, sem redefinir `Orientation`; `HashMap` proibido; undo próprio.

São dois produtos, e o plano trata-os assim:

| | Camada tilemap (P10) | Tilemap de jogo (Tiling) |
|---|---|---|
| Serve para | **fazer a arte**, dentro de um sprite | **montar o nível**, no mundo |
| Mora | `Sprite` no blob `PH2DPIXEL` | `TilemapDoc` no projeto, ECS |
| Tamanho | o de um sprite | chunks, mundo aberto |
| Automação | só terreno 47 | motor de regras LDtk |

**Para não haver um segundo desenho:** a P10 não cria componente, não toca no ECS nem no `PROJECT_SCHEMA`, usa a geometria de `ph2d-grid` e entrega ficheiros (tileset e mapa). Ela é uma fonte de arte para o Tiling, sem abrir o portão dele.

## 3. O oráculo — medido em 06/11, antes do plano

**LibreSprite 1.3 (GPL-2.0): NÃO tem tilemap.** Três medições: 0 ocorrências de `tilemap`/`tileset` no binário e nos XML do pacote · ao abrir `3x2tilemap-grayscale.aseprite`, `Unsupported chunk type 8227 (skipping)` · gravado de volta, o ficheiro só tem `0x4` e `0x2004`, sem tileset. ⇒ **Não serve de referência para a P10**, salvo para provar que continua a abrir um `.ase` nosso.

| Oráculo | Licença por artefacto | Responde a | Estado |
|---|---|---|---|
| **Godot 4.7.2** (`pacman`) | MIT; 20 dependências ligadas triadas, todas permissivas ou com exceção de ligação (`freetype2`, `zstd` e `graphite` em licença dupla, `glibc` LGPL). Só corrido, nada ligado ao nosso binário | autotile 47, posição iso/hex, e o gate «remonta o mapa» | ✔ **corrido** `--headless` |
| **Pixelorama 1.2.3** (`~/.local/opt`) | MIT segundo o nosso registo; ⚠️ o pacote instalado não traz ficheiro de licença, só o crédito «Orama Interactive» no `.pck`. Só corrido, e nada dele se copia | Manual, Auto, Stack, Draw tiles, iso, hex: os nomes estão no `.pck` (`LayerTileMap.gd`, `CelTileMap.gd`, `TileSetCustom.gd`) | ⚠️ **por medir** se a extensão consegue conduzir, como em `ferramentas/pixelorama_pressao/` |
| **Aseprite Trial 1.3.18.6** | EULA; só abrir e fotografar | referência de PRODUTO dos três modos e leitura do nosso `.ase` | ⚠️ por fotografar |

**Godot, sonda sobre mapa NOSSO** (10×8, tileset com as 47 máscaras, `TERRAIN_MODE_MATCH_CORNERS_AND_SIDES`, `set_cells_terrain_connect`):

- Máscaras válidas pela regra do MVP («um canto só conta com as duas arestas dele»): **47**, igual à contagem da spec.
- **25 de 25 células** deram a máscara prevista por essa regra. ⚠️ É um mapa só e com um terreno, e não exercita as 47 máscaras: a rodada M2 cobre todas.

| Forma, tile 16×16 (iso 32×16) | `(0,0)` | `(1,0)` | `(0,1)` | `(1,1)` |
|---|---|---|---|---|
| Quadrada | 8, 8 | 24, 8 | 8, 24 | 24, 24 |
| Iso, *layout* de omissão | 16, 8 | 48, 8 | 32, 16 | 64, 16 |
| Meio deslocada | 8, 8 | 24, 8 | 16, 24 | 32, 24 |
| Hex, *layout* de omissão | 8, 8 | 24, 8 | 16, 20 | 32, 20 |

⚠️ O *layout* de omissão do Godot é em **linhas alternadas**. O `IsoGrid` de `ph2d-grid` é losango (`iso_to_world`), ⇒ os índices não se comparam diretamente. Compara-se o centro depois da conversão de layout (M3).

⛔ Fixtura «escrita pelo Aseprite» continua a não existir: o Trial não grava. Os `.aseprite` de `~/Downloads/ase-para-testar/` têm licença desconhecida (2 com tileset) e são só medição local.

## 4. Desenho — a porta ÚNICA de cada pergunta

1. **«Onde mora o tilemap?»** → `ph2d-pixel-core`: `LayerKind::Tilemap { tileset, grid }` (o compilador aponta cada `match` dos 52), `Sprite.tilesets`, e o mapa **dentro do cel** (`Cel.tiles`). ⚠️ Mudou do desenho inicial, um mapa ao lado de `cels`: dentro do cel, apagar camada, apagar quadro, duplicar, desfazer e o `Op::Layers`/`Frames` já o carregam, e um mapa ao lado teria de ser lembrado em cada um, em silêncio. Só 2 construtores e 2 literais mudaram; os outros usam `..c.clone()`.
2. **«Como o resto do programa vê a camada?»** → **`Cel.data` continua a ser píxeis**, derivados do mapa e do tileset. Assim, compor, exportar, miniatura, onion e os 27 leitores de `.data` não ganham um ramo. Lei: `data == expande(mapa, tileset)`, com gate de propriedade. Grava-se só o mapa e o tileset; ao abrir, reexpande. O custo a medir (M6) é o de editar um tile usado por muitos quadros.
3. **«Quem escreve numa camada tilemap?»** → um funil, `ph2d-pixel-tiles::desenho::aplica` (mais `mapa::carimba` e `terreno::pinta`), sobre `Op::Tiles`. Os outros escritores recusam ou só andam a grelha: §5.1, gate `escritores.rs`. ⛔ Nenhum escreve no `data` derivado. O M5 não executou esses comandos no Trial: a recusa é escolha nossa, não paridade.
4. **«O que faz cada modo?»** → `ph2d-pixel-tiles::desenho`. O traço continua a ser o funil `symmetry`/`stroke`; `Stroke::writes` entrega as células e `Stroke::op` recusa em tilemap. A lei, com o que é extrapolação, está em §6.1. Deduplicação exata, sem flips no traço; com flips só em Convert To, a pedido e com tile quadrado.
5. **«Que tile vai em cada célula do terreno?»** → `ph2d-pixel-tiles::terreno`: `reduz`, `formas`, `casa`, `pinta`, com inteiros; 47 tiles seguidos a partir de `Terrain.first`. Slot por preencher não falha: a célula fica com o tile da casa dela, mesmo vazio (⛔ Recusas MEDIDAS, §6.2).
6. **«Onde cai cada célula iso/hex?»** → conversão inteira em `ph2d-pixel-tiles` (o mesmo traço dá o mesmo byte em qualquer máquina), **com gate contra `ph2d-grid`**. Não se redefine a orientação; a arte do tile continua retangular.
7. **«Como o `.ase` guarda isto?»** → `parse_doc`/`write` de `ph2d-aseprite` ganham tileset e cel tipo 3, e o bit 4 passa a ser lido. `abrir` deixa de achatar (`Aviso::Tilemap` só para tileset externo e grelha sem forma no formato). O `parse` antigo mantém a saída byte-idêntica para o Sprite Inspector. ⚠️ O `.ase` não tem campo de forma de grelha: iso e hex saem como imagem, com aviso.
8. **«Que ficheiros saem?»** → `render::frames` e `sheet::layout` da P9, sem caminho próprio: tileset em PNG, mapa em JSON (índice e flips por célula, por quadro), `--split-grid`, `--trim-by-grid`.
9. **Formato gravado:** `PH2DPIXEL` MINOR 3→4, append. Gates: ficheiro MINOR 3 congelado abre igual; projeto novo reabre igual. **Decidido e medido (M7):** `Cel.data` continua gravado, ⇒ o código real do `main` (`bbfefaf393`) lê o MINOR 4, vê uma camada de imagem com o composto igual, e regravar mantém o desenho (perde só os tiles). Ao ler, o mapa manda: `data` reexpande. Fixtura `tests/fixtures/pixel_minor3_bbfefaf393.bin`, escrita por esse código: abre, e regravar só troca o MINOR.

**Limites:** nenhum `MAX_*` novo. O formato limita (o índice de tile no `.ase` usa a máscara gravada no cel; o tamanho exato confere-se na spec em M4, com recusa e mensagem); o resto é a memória, medida em M6.

## 5. A UI — as 4 condições, por peça

| Peça | Existe? | Pintada + registada | Clique → barramento | Leva a algum lugar |
|---|---|---|---|---|
| **Layer › New Tilemap Layer** + diálogo (Grid W/H, forma) | menu sim, linha não | cartões, crate nova `ph2d-panel-pixel-tiles` | testkit | camada nova com tileset |
| **Convert To › Tilemap / Image** | NÃO | idem | idem | deduplica; inverso achata |
| Cartão **Tileset** (lista, tile 0, flips, Add/Delete) | NÃO | idem | idem | escolhe tile, escreve no documento |
| Seletor **Pixels / Tiles** e **Manual/Auto/Stack** | NÃO | barra de opções da ferramenta | `Space+Tab`, `Space+1/2/3` | muda o destino do traço |
| Cartão **Terrain** (criar gabarito, pintar terreno) | NÃO | idem | idem | resolve o mapa a cada traço, um passo de desfazer |
| **Grid** Square/Iso/Hex | NÃO | diálogo e Layer Properties | idem | a grelha na tela e o clique seguem a forma |
| **File › Export › Export Tileset** + aba da folha | P9 tem a folha | `ph2d-panel-pixel-export` | idem | bytes no disco |

Lei de UI: cartões compactos, rótulo em campo que não se explica sozinho, unidade no rótulo, zero hex e zero `f32` literal, só inglês em `ph2d-i18n`. `ph2d-app-pixel` só liga. Com folga 28, a P10a **move** o que for preciso antes de ligar; o número do tecto não sobe. `Space` já é mover a vista (`compose.rs:179`), ⇒ `Space+Tab` e `Space+1/2/3` não podem interromper esse gesto; gate próprio.

### 5.1 O que recusa numa camada de tilemap (a P10b deixa cinzento, nunca mudo)

Conferido com a camada ativa: `crates/ph2d-app-registry-init/tests/it/tilemap_recusa.rs` manda cada pedido real, com e sem seleção, e o documento fica igual, sem pânico. ⚠️ **O teste do app não manda:** o gesto do rato (lápis, formas, balde, Gradient), Text, Despeckle, Outline, Convolution, Hue/Saturation, Color Curve, Trim, Crop nem arrastar cels; esses têm prova só no núcleo (`escritores.rs`) ou por leitura do chamador. **Coluna «hoje»:** o que o artista vê agora, antes da tela.

| Comando | Recusa | Hoje |
|---|---|---|
| Lápis, borracha, balde, formas, Text, Gradient (o traço de píxeis em si) | `Stroke::op` = `None` até a P10b ligar `desenho::aplica` | mensagem `pixel.layer.refuse.tilemap` |
| Edit: Flip H/V, Rotate 90/180, Shift, Clear (Delete), Cut, Fill, Stroke, Transform/moldura, Paste | `transform::stamp` = `None` | mensagem |
| Edit: Invert, Despeckle, Outline, Convolution, Brightness/Contrast, Hue/Saturation, Color Curve | `filter::patch` = `None` | mensagem; com «All» saltam só o tilemap |
| Replace Color | recusa no chamador | mensagem |
| Sprite: Rotate Canvas 90/180, Flip Canvas H/V, Sprite Size (todos os métodos) | `None` com qualquer camada de tilemap no sprite | ⚠️ **mudo** |
| ~~Sprite: Canvas Size, Crop, Trim fora de múltiplo do tile~~ | **Saiu da lista em 06/11:** Pixelorama e Trial aceitam (M9); a grelha vai com o canvas | — |
| Layer: Merge Down com tilemap em qualquer lado | `None` | mensagem `pixel.layer.refuse.action` |
| Layer via Cut | `None` | mensagem `pixel.layer.refuse.action` |
| Timeline: arrastar cel para camada de outro tileset ou de imagem | `None` | ⚠️ **mudo** |
| Convert To Tilemap: grupo, referência, Background, já tilemap | `None` | sem tela |

Continuam a valer: Layer via Copy, Copy, Copy Merged, seleção, Flatten, Duplicate, New/Delete/Move/Link/Unlink de quadros e cels, Color Mode, paleta (Remap).

## 6. Medição — UMA rodada (§0.10), antes de escrever lei

Três scripts, `ferramentas/oraculo_godot_tiles.py`, `ferramentas/pixelorama_tiles/` e `ferramentas/trial_tiles.py`, sobre entradas NOSSAS. Fixturas com cabeçalho em `crates/ph2d-pixel-tiles/tests/fixtures/`.

| # | Pergunta | Como |
|---|---|---|
| M1 | A extensão conduz Manual, Auto e Stack no Pixelorama? O mesmo traço, nos 3 modos: que tiles nascem, e quem muda? | `Tools.handle_draw`, como na pressão |
| M2 | As 47 máscaras e as 256 vizinhanças dão a mesma resposta no Godot? E borda do mapa, buraco, slot por preencher? | sonda de 06/11 ampliada |
| M3 | Centro e célula de volta de iso e hex, com tamanhos pares e ímpares: Godot × `ph2d-grid` × Pixelorama | `map_to_local` e `local_to_map` |
| M4 | O Trial abre o nosso `.ase` de tilemap, com flips X/Y/D em todas as 8 combinações? A ordem dos flips é a de `expand_tilemap`? | escritor de teste + foto |
| M5 | Trial: tile 0, modo inicial, e o que filtro, moldura, Sprite Size, Rotate, troca de modo, Merge Down e Flatten fazem à camada | fotos; ⛔ nada de paridade onde só há foto de um caso |
| M6 | Editar um tile usado em 1024×1024 células × 256 quadros: ms e memória. Acima de 100 ms, muda-se o desenho (§4.2) para reexpandir só o que está à vista | `--profile smoke`, o mínimo de 3 |
| M7 | Projeto MINOR 4 no binário do `main`, e tamanho do blob com e sem píxeis expandidos | binário real |
| M8 | Godot remonta o mapa a partir do nosso PNG e JSON, píxel a píxel | `--headless` |

**Kill-criterion do Pixelorama:** se a extensão não conduzir os modos em 2 tentativas, a lei dos modos fica só com as fotos do Trial e com propriedades, e o handoff diz que não há oráculo corrido para ela.

### 6.1 O que M1–M7 deram (detalhe de M1–M5: [P10_oraculos.md](P10_oraculos.md))

| # | Resultado | Vira |
|---|---|---|
| M1 | A extensão conduziu (5.ª corrida; os falhanços foram montagem minha). 42 estados | gate `os_tres_modos_batem_com_o_pixelorama_corrido`: 17 cenários batem em mapa, número de tiles, píxeis cheios e usos |
| M2 | 256/256 e 376/376 iguais à regra; slots em falta o Godot troca de tile | gates `o_terreno_bate_com_o_godot_*`, lendo o JSON; recusa em §6.2 |
| M3 | Godot e Pixelorama fecham a volta, com convenções iso/hex diferentes entre si | só P10c; ⚠️ comparação com `ph2d-grid` por fazer |
| M4 | O Trial abre o nosso `.ase`; ordem D, depois X e Y | gates de flips; ordem de chunks tileset → layer → cel |
| M5 | Modo inicial Auto (ini do dono copiado, não limpo). Filtros, Rotate, Sprite Size, moldura e Flatten **não foram executados**; Flip do Edit foi, sem interpretação | as recusas de §5.1 são escolha nossa |
| M6 | Abaixo | ⚠️ acima de 100 ms |
| M7 | Ver §4.9 | gate `a_minor_3_file_written_by_the_old_code_opens_the_same` |
| M8 | Por fazer | P10c |

**Lei dos modos: o que é medido e o que é extrapolação.** Medido (M1, um pixel por vez): os 17 cenários. **Extrapolado, sem oráculo:** um traço de vários píxeis e células (aplicado célula a célula, com a contagem de usos viva: um tile usado 2× com as duas células no mesmo traço dá tile novo à primeira e edita o original na segunda) · só sai do conjunto o tile que **este** gesto largou, nunca um que já estava sem uso · a célula com flip mantém os flips no tile novo ou deduplicado · escrita fora do canvas ignora-se · dedupe não entra em tile de gabarito (lei nossa: a célula viraria terreno no próximo `pinta`). **Contra o que a sonda viu, por decisão do briefing da onda:** Eraser em Manual edita o tile e em Stack dedupa com o tile 0; a sonda viu «nada muda», sem saber se é a lei ou o toque dela. ⚠️ A confirmar no smoke do dono.

**M6, `--profile smoke`, load 1,5–1,9, tile 16×16, editar um tile em Manual, o MÍNIMO (`tests/medida_m6.rs`):**

| Documento | Imagens | Gesto inteiro | Só aplicar | Desfazer | Amostras |
|---|---|---|---|---|---|
| 256×256 células × 64 quadros, sem ligados | 4 096 MB | **1 693 ms** | 1 687 ms | 1 687 ms | 5 |
| 1024×1024 células × 1 quadro | 1 024 MB | **426 ms** | 423 ms | 422 ms | 3 |

(Uma corrida anterior a load 13–16 deu 1 927 e 476 ms.) O pedido do §6 (1024×1024 × 256 quadros) são **256 GB**: não cabe em 24 GB, recurso = a memória da fatia; cabem ~23 quadros, e não os corri: o número é por quadro. O caso é o pior possível (todas as células do documento são cópia do tile editado, em todos os quadros); o custo escala por célula afetada (~0,40 µs por célula de 256 píxeis nos dois casos). Só o `desenho::aplica` e o `History` estão medidos, sem preview nem GPU. **Passa dos 100 ms, ⇒ proposta, por decidir com o Coordenador:** (a) `blit` por linha com a imagem de cada combinação de flips já pronta, em vez de `place` por píxel (barato, cá dentro; ganho por medir, ⚠️ não chega sozinho a 100 ms no 1.º caso se o ganho for menos de 17×); (b) os quadros em paralelo; (c) o desenho do §6: reexpandir só o que está à vista e o resto a pedido, o que parte a lei `data == expand` e os 27 leitores dela, logo só se (a) + (b) não chegarem.

**M6 depois da cura (P10b, `--profile smoke`, load 0,7–1,1, o MÍNIMO; o caminho antigo corre no MESMO processo, intercalado: `px_a_px` em `tests/medida_m6.rs`):**

| Documento | Píxel a píxel (antes) | Por linha + só as linhas que mudaram | Desfazer |
|---|---|---|---|
| 256×256 × 64 quadros, um píxel editado | 2 531 ms | **38 ms** | 33 ms |
| 1024×1024 × 1 quadro, um píxel editado | 801 ms | **10 ms** | 7 ms |

Por degrau, no pior caso (256×256 × 64): só por linha, 178 ms; com fios por quadro, 142 ms. A parada é a banda da memória (4 GiB escritos), não a CPU: 16 fios deram só 20 %. ⇒ Os fios saíram (o núcleo continua sem fios nem dependências) e entrou reescrever só as linhas sujas. ⚠️ Mudar TODAS as linhas de um tile nesse documento continua em ~178 ms (mesmo caminho, sem o ganho das linhas): só nesse caso extremo. ⚠️ Ainda não medido: preview e GPU.

### 6.1b Decisões com o M9 (06/11; o dono delegou, «o estado da arte»)

Medição e fixtura: [P10_oraculos.md](P10_oraculos.md) M9 e M9b, gates `as_bordas_batem_com_o_m9_corrido` e `a_lista_e_a_copia_que_fica_batem_com_o_m9b`.

| Pergunta | Lei | Porquê |
|---|---|---|
| Borracha | Igual aos outros traços. Apagar um tile inteiro: Manual esvazia o tile e as cópias, Auto põe a célula a 0 e tira o tile sem uso, Stack põe a célula a 0 e o tile fica | Os dois oráculos concordam, menos Manual e Stack a apagar tudo: o Pixelorama deixa o tile intacto com o canvas vazio, e o mapa volta ao reabrir. Vale o Trial |
| Um traço por várias cópias | Pela posição da célula, linha primeiro, nunca pela ordem do traço: em Auto, cada cópia ganha um tile novo no fim da lista e a última que restar edita o original. Todas as cópias com a mesma imagem: o original sai, os de cima descem um número, e o novo fica no fim | Os dois oráculos (M9b). Na diagonal discordam, e vale o Trial; o Pixelorama vai por coluna. Manual: a união (Trial). ⛔ Editar no lugar para manter os números foi feito e desfeito: nenhum oráculo o faz |
| Encolher o canvas | Não corta o mapa; crescer devolve as células. Canvas Size, Crop e Trim aceitam tamanho fora do múltiplo, a grelha vai com o canvas | Pixelorama e Trial. O Tiled corta, mas é um editor de mapas. O Trial também tem a opção de cortar: não há nesta linha |
| Tileset sem camada | Fica no sprite, sempre oferecido | Flatten e Convert: todos ficam. Delete: Pixelorama e Tiled ficam; o Trial avisa e apaga, recusado pela ordem do dono («disponíveis para qualquer camada desde sempre») |
| Camada nova | Traz o tileset da camada activa escolhido | Ordem do dono; o Trial traz «New Tileset» |

### 6.2 ⛔ Recusas MEDIDAS

| Recusa | Medição | Porquê |
|---|---|---|
| Quadros em paralelo para o M6 | 178 → 142 ms com 16 fios, ver acima | A banda da memória manda; só as linhas sujas chegam a 38 ms |
| Copiar o «tile mais próximo» do Godot num slot em falta | M2C: 4 corridas; sempre um vizinho a 1–4 bits, e 35 células com máscara existente receberam outra (a falta propaga-se) | Um mapa onde um tile em falta muda os vizinhos esconde o buraco. A célula fica com a casa dela, mesmo vazia, e o artista vê o que falta. Gate `slot_em_falta_nao_escolhe_outro_tile` |
## 7. Gates, red-first

| Gate | O que prova | Base |
|---|---|---|
| `o_derivado_e_o_mapa_expandido` | depois de qualquer sequência de ops, `data == expande` | propriedade |
| `os_tres_modos_*` | Manual muda todas as instâncias; Auto e Stack criam conforme M1/M5 | fixtura + foto |
| `flips_xyd_nas_oito_combinacoes` | cada combinação, célula a célula | M4 |
| `tile_zero_e_vazio_sempre` | tile 0 não se edita nem se apaga; o bit 4 é respeitado | spec |
| `desfazer_um_gesto_um_passo` | traço, terreno e convert voltam ao byte, com o tileset | propriedade |
| `o_terreno_bate_com_o_godot` | 256 vizinhanças e um mapa real | M2 |
| `iso_hex_batem_com_godot_e_ph2d_grid` | centro e clique de volta | M3 |
| `o_ase_de_tiles_volta_igual` / `o_trial_abre_o_nosso` | ida e volta, incluindo flips | propriedade + foto |
| `o_parse_antigo_nao_mudou` | Sprite Inspector byte-idêntico | regressão |
| `o_blob_minor_3_abre_igual` | ficheiro congelado de hoje | **formato gravado** |
| `o_godot_remonta_o_mapa` | PNG e JSON nossos | M8 |
| `abrir_nao_perde_em_silencio` | ⚠️ muda de propósito: `tests/ase.rs:229` passa a exigir camada tilemap real | costura |
| `space_tab_nao_parte_o_mover_vista` | teclado | costura |
| `tilemap_*` no testkit | evento real → documento ou bytes no disco | DoD |

Mutações por `mutacao`: máscara do canto, ordem dos flips, deduplicação, bit 4, funil único (escrever no `data` tem de sangrar). Portões por nome no fecho: `the_shell_only_shrinks`, `no_magic_numeric`, `no_literal_color`, `architecture_panel_wiring_parity`, `node_id_collisions`, `architecture_workspace_file_loc_cap`, `panel_functions_under_loc_cap`. Formato gravado e crates novas ⇒ crate inteira, com `motivo:`. `PH2DPIXEL` é formato gravado: `ph2d-pixel-core` e `ph2d-pixel-io` inteiras uma vez.

**Aceitação congelada:** sprite animado com camada tilemap desenhado nos 3 modos e em Draw Tiles, com flips e terreno 47 sobre um mapa real, grava, reabre igual, vai a `.ase` que o Trial abre e a PNG + JSON que o Godot remonta. Mapa real = mapa nosso de jogo com pelo menos 200 células, buracos e corredores de 1 célula.

## 8. Ordem e contratos

Três ondas de um bloco (§0.10), com uma janela nova entre elas:
- **P10a — motor:** rodada M1–M8 → modelo, codec e `Op::Tiles` → modos → `.ase` → testes. Sem tela e sem smoke. Série só onde M1/M5 mudam a lei.
- **P10b — tela:** crate do painel → camada, Tileset, modos, Draw Tiles, atalhos → cena **21** (`max_level` 20→21) fotografada. **Smoke do dono.**
- **P10c — além do Aseprite:** terreno 47 → iso/hex → Export Tileset → cena **22**, fotografada. **Smoke do dono.**

Contratos §6 e `PROJECT_SCHEMA`: não encosta (só o MINOR do blob). `Cargo.lock` +2 crates. A sonda de clique e roda usada para fotografar as cenas fica fora do commit.

⚠️ Base: `828051653b` fora do main. Nos passos de smoke, sair do Draw pelo seletor de modo, nunca pelo `Tab`.

## 9. Perguntas ao dono (antes de codar)

1. **Até onde vai agora?** Recomendo: fazer a arte em tiles dentro do Pixel e exportar, sem objeto de mapa no jogo (que continua no projeto Tiling, suspenso por você). Alternativas: já criar um objeto simples de mapa no jogo; ou reabrir o Tiling primeiro.
2. **Terrenos do pincel automático:** recomendo vários por tileset, sem transição entre eles (terreno encosta em terreno como em vazio). Alternativas: um só por tileset; ou com transições, bem mais caro.
3. **Iso e hex entram agora?** Recomendo sim, por último. O formato `.ase` não guarda essa forma: lá saem como imagem, com aviso. Alternativa: deixar para depois.
4. **Formato de saída do mapa:** recomendo PNG + JSON, com prova no Godot. Alternativa: acrescentar também o arquivo nativo do Godot ou do Tiled.

**Assumido, salvo veto:** comandos, nomes e atalhos do Aseprite · modo inicial Auto, a conferir no Trial · editar um tile muda todas as cópias no modo Manual · o tileset é por sprite · nada de `.ldtk`/`.tmx`.

**Respostas do dono (06/11):** as quatro e o «Assumido» como recomendado, sem veto. **Abordagem escolhida pelo dono (06/11): carimbo à mão (Draw Tiles) + gabarito de 47** («vamos conforme vc sugere»). Comparou com marcar tiles, regras escritas, dual grid de 16 e descoberta por exemplo. A descoberta por exemplo do MVP Tiling, que ele achou pouco intuitiva e imprecisa, **não entra**; regras escritas ficam para o Tiling; o dual grid fica para depois do smoke do gabarito, se ele pedir.
