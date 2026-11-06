# P9 — Export e Import: o plano (05/11)

> Linha `line/PixelArt`, HEAD `19c36ab8b8`, merge-base `c61f228b05` (P8d aprovada no smoke, por integrar).
> Plano do módulo: [02_plano_modulo.md §P9](02_plano_modulo.md). Molde: [P8_plano.md](P8_plano.md).
> Lei do módulo: [ADR-0196](../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md).
> Referência de produto: menu File do Aseprite ([foto 12](aseprite/12_sub_export_import_paste_rotate.png)), diálogo da folha
> ([foto 31](aseprite/31_camada_tamanho_folha_prefs.png)), CLI ([40](aseprite/40_cli_help_1.3.18.6.txt)).
> **Estado: P9 fechada (P9a, P9b), `Smoke OK` do dono em 05/11 (cena 20 e janela do Import); à espera de integrar. [Handoff de integração](handoffs/HANDOFF_INTEGRACAO_line_PixelArt_2026-11-05_P9.md).**

## 0. O que entra

| Grupo | Peças | No Aseprite |
|---|---|---|
| **E — Export As** | PNG, sequência de PNG, GIF, APNG, WebP, `.ase`; escala, direção, quadros (todos/tag/intervalo), camada (visíveis/uma) | File › Export › Export As, `Ctrl+Alt+Shift+S` |
| **F — Export Sprite Sheet** | abas Layout · Sprite · Borders · Output, com todas as opções do CLI que têm sentido sem slices e tilemaps; prévia ao vivo | File › Export › Export Sprite Sheet, `Ctrl+E` |
| **J — O JSON** | `json-hash` e `json-array`, `meta.frameTags/layers/slices` | Output |
| **R — Repeat Last Export** | repete a última exportação do sprite | `Ctrl+Shift+X` |
| **I — Import Sprite Sheet** | grelha com tamanho, margem e espaço, vira quadros | File › Import, `Ctrl+I` |
| **O — Abrir** | `.ase`/`.aseprite`, PNG (paleta preservada), sequência, GIF, APNG, WebP viram Pixel Sprite editável | File › Open |
| **P — Pendências** | as da P8 (§7) | — |

**Fora, de propósito:** `--batch` do binário (o plano do módulo já o adia) · `--split-slices`, `--slice` e slices no JSON além da lista
vazia (P11) · Export Tileset e `--split-grid`/`--trim-by-grid` com tilemap (P10) · JPEG e QOI (ver §9, «Assumido») · a ponte para o Sprite
Inspector como objeto vivo (P14: aqui só o gate de leitura do ficheiro).

## 1. Conferido no código (o handoff P8d contra a árvore)

| Afirmação | Conferido |
|---|---|
| `ph2d-pixel-io` só tem `frame_rgba`, `.msk`, paletas e pincéis | ✔ `lib.rs:15`, `msk.rs:115,174`, `palette_file.rs:76,284`, `brush_file.rs:148,250`; 939 linhas |
| O documento tem o que o `.ase` pede | ✔ `Sprite` (`ph2d-pixel-core/src/sprite.rs:328`): RGBA/Gray/Indexed, camadas com grupo, blend e opacidade, cels esparsos com `opacity`, `z`, `link`, `duration_ms`, tags com `repeat` e `color` |
| `ph2d-imageio-gif` serve | ⚠️ grava animação, mas **sempre em laço infinito** (`lib.rs:232`); a tag com repeat e o sem-laço pedem extensão |
| `ph2d-imageio-apng` serve | ❌ **só lê** animação; `Animated` no export dá `Unsupported`. O `png` 0.18.1 tem `set_animated`, `set_frame_delay`, `set_palette`, `set_trns` |
| `ph2d-imageio-webp` serve | ❌ **um quadro só**. O `image-webp` 0.2.4 codifica VP8L de um quadro; `ANIM`/`ANMF` é contentor RIFF de spec pública |
| `to_aseprite_json` serve | ❌ como está: sem `duration`, sem `trimmed`, sem tags e camadas, só hash, e ordenado por nome (`aseprite.rs:34`). É o escritor a estender, nunca um segundo |
| `ph2d-aseprite::parse` serve | ⚠️ devolve quadros JÁ compostos (`lib.rs:49,173`). Sem camadas, cels, paleta ou modo. A lei de blend (`blend.rs`) já existe |
| Sem escrita de `.ase` | ✔ existe só o escritor Python de teste (`ferramentas/trial_ase_camadas.py`), a partir da spec pública |
| «O Pixel não tem menu File» | ✔ mas **o app tem** (`hero/file_menu.rs:18`, `menu_tables.rs:174,306`): Save, Save As, Open, Export SVG |
| `.ase` arrastado | Vira folha animada, não Pixel Sprite: `import_router.rs:83,158` → `shells/desktop/src/ase_import.rs:239` |
| Costura de diálogo | ✔ `PixelEditAction` (`pixel_panel/selection.rs:90`) → `filtros::menu` (`filtros.rs:32`) → `PixelDialogKind` → `paint_pixel_edit_dialog` (`context_menu_pixel_edit.rs:73`, 621 linhas: **não cabe a folha aí**) |
| Precedente de ficheiro | `.msk`: `selecao.rs:126`, `ph2d_app_host::modal::pick_file` |
| Fixturas do MVP W13 | ❌ **nunca foram ficheiros de importador.** O gate era um «importador de brinquedo» só com os campos do formato (`docs/Pixel Art/app/tests/sheet.test.js`, pasta fora do git, só no primário). O diário (W13) diz o contrato: forma do Aseprite, `repeat` em STRING e ausente se infinito, cor `#rrggbbaa`, o nosso numa chave própria em `meta`. Trago a ideia, em Rust (§6) |
| Licenças | ✔ `png`, `gif`, `image-webp`, `flate2`, `fdeflate`: MIT/Apache-2.0 · `miniz_oxide`: MIT/Zlib/Apache-2.0. Tudo em Rust, nenhuma biblioteca nova |
| Tecto `ph2d-app-pixel` | 11 193 de 11 415: **folga ~222**. A lei e a UI nova ficam fora dele |

## 2. O oráculo — corrido em 05/11, antes do plano

`libresprite -b` (`SDL_VIDEODRIVER=dummy`), sobre um GIF nosso de 3 quadros (100/200/300 ms), e sobre 5 `.aseprite` reais:

| Pergunta | Medido |
|---|---|
| Grava `.ase`, sequência, GIF? | ✔ Magic `e0a5`; durações preservadas; o GIF sai com retângulos por quadro (16×16, 9×11+4+3, 7×5+4+9) |
| Folha + JSON? | ✔ hash e array, `rows`, `packed`, `--trim`, 3 paddings. Medido: `--inner-padding 1` e `--trim` dão `frame` 7×7 e `spriteSourceSize` 5×5 em (2,2) |
| WebP? | ❌ com imagem indexada recusa («Indexed format… Error saving»). RGBA por medir |
| Nome da sequência | `seq_{frame}.png` sai `seq_1{frame0}.png`: **defeito dele, não lei** |
| Lê `.aseprite` atual? | ✔ tags com direção, grupos, cels ligados. Em tilemap avisa (`chunk 8227`) e segue |

**O que ele NÃO responde** (ausente da ajuda): `--merge-duplicates`, `--extrude`, `--power-of-two-size`, `--split-tags`, `--tagname-format`,
`--trim-sprite`, `--play-subtags`, `meta.frameTags[].repeat`, escala de pixel ratio. Estes têm **lei própria, testada por propriedade**
(§6) e a definição vem do CLI do Aseprite já fotografado (40) e da spec pública. ⛔ Nada de dizer «paridade» neles.

**Correção do plano do módulo e do handoff:** o gate «`.ase` aberto no Aseprite» **cumpre-se sem licença**. O Trial não grava, mas ABRE: foi
por aí que a P3 mediu os modos de mistura (`.ase` escritos por nós, `trial_ase_camadas.py:1`). O Trial entra só para isto, abrir e fotografar; a
referência de saída é o LibreSprite. ⛔ Fixtura «escrita pelo Aseprite» não existe nem existirá, e isso fica dito.

**Godot 4.7 (MIT):** script `--headless` nosso lê o JSON e a folha, recorta cada quadro, compara com o PNG do quadro. Prova coordenadas e pixels,
não o importador da comunidade. **Unity e Phaser não estão instalados e não instalo:** o gate deles é a forma do JSON igual à do LibreSprite no
mesmo sprite. Fica dito que nenhum dos dois correu.

⚠️ **`~/Downloads/ase-para-testar/` (18 `.aseprite` reais):** licença desconhecida, ⇒ medição local, **não entram no repo**. As fixturas
versionadas nascem de entradas nossas.

## 3. Desenho — a porta ÚNICA de cada pergunta

1. **«Como se lê e grava um `.ase`?»** → `ph2d-aseprite`, sem dependência de `ph2d-pixel-core`: `parse_doc` novo devolve camadas, cels, paleta, tags e
   modo; o `parse` antigo vira composição sobre ele, com a saída byte-idêntica (gate) e `import_ase` intocado. `write` novo, deflate por `miniz_oxide`.
   Tradução `Sprite ⇄ doc` em `ph2d-pixel-io/src/ase.rs`.
2. **«O que o documento põe em cada quadro exportado?»** → `ph2d_pixel_io::render::frames(sprite, &Selecao) -> Vec<QuadroSaida>`: camada (visíveis/uma/uma por imagem),
   quadros (todos/tag/intervalo), direção (as 4 de `sprite.rs`, Forward, Reverse, PingPong, PingPongReverse; a passada reusa a lei do `Play` em `play.rs`, nunca uma 2.ª; `pingpong_reverse` não existe no LibreSprite, sem oráculo aqui), escala. **Tudo o resto consome
   isto**, e o PNG, o GIF e a folha não têm caminho próprio. A composição continua `compose_rgba`.
3. **«Como cada formato grava?»** → os `ph2d-imageio-*`, estendidos e não copiados: GIF com `repeat`; APNG e WebP animados **novos** nos crates deles (WebP = VP8L do `image-webp` num
   `ANIM`/`ANMF`). Indexado em PNG: `ph2d-imageio-png` ganha escrita com `PLTE`/`tRNS`.
4. **«Onde cai cada quadro na folha?»** → `ph2d_pixel_io::sheet::layout(&[QuadroSaida], &OpcoesFolha) -> Folha`: tipos horizontal/vertical/rows/columns/packed, restrições (colunas, linhas, largura, altura), 2ⁿ, bordas (3 paddings), trim, extrude, merge duplicates, ignore empty, split layers/tags. `packed` reusa `rect_packer` (`pack.rs`).
5. **«Que JSON sai?»** → `ph2d_sprite_sheet::aseprite` num struct mais rico (`duration`, `trimmed`, tags, camadas, hash ou array, `filename-format`); o `to_aseprite_json` atual passa a chamá-lo, com saída byte-idêntica (gate). `meta.app` diz a verdade: a URL do PH2D.
6. **«Como um ficheiro vira documento?»** → `ph2d_pixel_io::abrir(bytes, formato) -> Sprite`, um por formato sobre os leitores `ph2d-imageio-*`. PNG com paleta abre Indexed (hoje o leitor converte: a leitura indexada é peça nova). Tilemap do `.ase` vira camada de imagem expandida, com aviso. Slices e user data não têm onde morar até a P11/P12: **aviso ao abrir, nunca perda muda**.
7. **«O que é a Import Sprite Sheet?»** → inverso de `sheet::layout` só na grelha; sugestão inicial por `ph2d_sprite_sheet::guess_grid`.
8. **«O que o Repeat repete?»** → `ExportPreset` guardado por sprite (caminho + opções). Codec `PH2DPIXEL` MINOR 2→3 (append, `PROJECT_SCHEMA` intocado). Caminho relativo ao projeto se couber.

**Trabalho longo:** exportar não pode congelar a tela. Medir (§5) folha 1024²×256 quadros; acima de 100 ms vai a um operário com a barra de progresso da casa.
**Limites:** nenhum `MAX_*` novo. Os de formato (GIF e `.ase` em u16) recusam com mensagem; o resto é a memória, e fica medido.

## 4. A UI — as 4 condições, por peça

| Peça | Existe? | Pintada + registada | Clique → barramento | Leva a algum lugar |
|---|---|---|---|---|
| Linhas no menu **File** (conforme §9.1) | Save/Open/Export SVG | `menu_tables.rs` | `file_menu` → `PixelRequest::File` novo | abre o diálogo |
| Diálogo **Export As** | NÃO | cartões, módulo novo (o `context_menu_pixel_edit.rs` está cheio) | `seam` + testkit | `render::frames` → formato → `pick_file` de salvar |
| Diálogo **Export Sprite Sheet**, 4 abas, prévia ao vivo | NÃO | idem, com a receita de cartões | idem | `sheet::layout` a cada mudança; Export grava |
| Diálogo **Import Sprite Sheet** com grelha sobre a imagem | NÃO | idem | idem | novo Pixel Sprite |
| **Open** por formato | `rfd`, `pick_file` | seletor nativo | shell → `PixelRequest::File` | novo Pixel Sprite, um passo de desfazer |
| **Repeat Last Export** | NÃO | linha do menu, cinza sem preset | idem | grava sem diálogo, toast com o nome |
| Atalhos `Ctrl+E`, `Ctrl+I`, `Ctrl+Shift+X`, `Ctrl+Alt+Shift+S` | `Ctrl+Alt+I` é Sprite Size, `Ctrl+Shift+I` é Inverse; os quatro parecem livres, conferir em `input_teclas.rs` e `selecao.rs` no Draw | — | `input_teclas.rs` | os acima |

Lei de UI: painéis nascem em cartões ([receita 11](../CartoesUI/11_RECEITA_PAINEL_EM_CARTOES.md)), compactos como o Figma, rótulo em campo que não se explica sozinho, unidade no rótulo, zero hex e zero `f32` literal, só inglês (i18n em ficheiro `pixel_export.rs` novo). **Crate nova `ph2d-panel-pixel-export`** para os diálogos e `ph2d-app-pixel` só liga (folga ~222): `Cargo.lock` +1, registo por `architecture_panel_wiring_parity`.

## 5. Medição — UMA rodada (§0.10), antes de escrever lei

Um script, `ferramentas/oraculo_export.py` (`libresprite -b` + API JS, como `oraculo_msk.py`), sprites NOSSOS com **camadas, grupo, blend, opacidade de cel, ligados, tags, paleta indexada e transparência**. Fixturas com cabeçalho em `crates/ph2d-pixel-io/tests/fixtures/oracle_export_*.json`. A mesma corrida responde a tudo:

| # | Pergunta | Como |
|---|---|---|
| M1 | Cada byte que o `.ase` dele grava e relê (blend, opacidades, ligados, paleta, modo, tags) | `--save-as` + dump por chunk |
| M2 | O que ele aceita de um `.ase` nosso, e o composto bate? | leitura de volta + `--sheet` + comparação por pixel |
| M3 | Folha: 5 tipos × larguras × `--trim` × 3 paddings × `--split-layers` × `--frame-tag` × `--frame-range` × `--ignore-empty` | JSON e PNG |
| M4 | Forma exata do JSON: chaves, ordem, `meta`, `filename` por omissão | hash e array |
| M5 | GIF: mais de 256 cores, alfa parcial (limiar?), durações que não são múltiplo de 10 ms, laço, transparente | `magick identify` + `ffprobe` |
| M6 | PNG indexado: `PLTE`, `tRNS`, índice 0 transparente | dump de chunks |
| M7 | WebP RGBA e APNG: abre? anima? | se ele não grava, ffmpeg lê os nossos |
| M8 | Escala `--scale` e `--shrink-to` | píxeis |
| M9 | **Pendência:** alfa de `edges-find*` | convolução dele, ou a rodada do Trial por `fit_convolucao_alfa.py` |
| M10 | Tempo: folha 1024²×256 no NOSSO código (depois de escrito), `--profile smoke`, o mínimo de 3 | define se vai a operário |

## 6. Gates, red-first, e as fixturas

| Gate | O que prova | Base |
|---|---|---|
| `o_ase_nosso_volta_igual` | `Sprite → write → parse_doc → Sprite` igual em todos os campos | propriedade |
| `o_libresprite_le_o_nosso_ase` | M2: composto dele = o nosso, quadro a quadro | fixtura |
| `o_ase_dele_abre_igual` | fixturas M1 lidas dão o composto que ele exporta | fixtura |
| `o_parse_antigo_nao_mudou` / `o_json_antigo_nao_mudou` | saídas byte-idênticas | regressão |
| `a_folha_bate_com_o_oraculo_*` | layout, trim, paddings, JSON (menos `packed`) | M3–M4 |
| `packed_nao_sobrepoe_e_preserva` | sem sobreposição, recorte = quadro. ⛔ Posição não é lei: heurística dele | propriedade |
| `merge_extrude_2n_split_tags_*` | merge: duplicado partilha rect; extrude: borda repete; 2ⁿ; split-tags | propriedade + CLI (40) |
| `o_godot_recorta_igual` | script `--headless` sobre a nossa folha. `#[ignore]` só se precisar de placa | corrido |
| `o_sprite_inspector_le_a_folha` | `parse_atlas_meta` lê o nosso JSON, regiões certas | costura |
| `um_importador_so_com_os_campos_do_formato` | importador de teste, sem tipos nossos, remonta a animação pelas tags, incluindo `repeat` em string e ausente se infinito | ideia do MVP W13 |
| `gif_apng_webp_voltam_iguais` | export → nosso leitor + `ffmpeg`: quadros e durações | propriedade + fixtura |
| `export_as_*`, `a_folha_ao_vivo_*`, `import_*`, `repeat_*` | evento real no `ph2d-ui-testkit` → **bytes no disco** ou documento | DoD §5 |
| `abrir_nao_perde_em_silencio` | slices/tilemap ⇒ aviso | costura |

Mutações por `mutacao`: lei do `.ase` (flags de cel, ordem de chunks), trim, direção, `repeat` em string, durações GIF. Portões por nome no fecho: `the_shell_only_shrinks`, `no_magic_numeric`, `no_literal_color`, `architecture_panel_wiring_parity`, `node_id_collisions`, `architecture_workspace_file_loc_cap`, `panel_functions_under_loc_cap`. Crate nova e formato gravado ⇒ crate inteira, com `motivo:`.
**Aceitação congelada:** sprite de teste com todos os recursos do §5 sai pelos 6 formatos e pela folha, e `.ase`/folha voltam iguais. **Kill-criterion do WebP animado:** se o `ffmpeg` e o LibreSprite recusarem o contentor depois de 2 tentativas, o WebP sai só de um quadro e o dono é avisado.

## 6b. O que a P9a mediu e mudou neste plano (05/11)

- **Pixel ratio saiu:** o documento não tem pixel ratio (`grep` em `ph2d-pixel-core`); o `.ase` grava 1:1.
- **O JSON é um escritor novo ao lado do antigo** (`aseprite::sheet_json`), não uma extensão: `to_aseprite_json` ordena por nome e a saída dele tem de ficar byte-idêntica para quem a usa.
- **`.ase` nosso aberto no Trial: feito** ([foto](handoffs/trial_abre_o_nosso_ase_P9a_2026-11-05.png): grupo, paleta, cinza). Sem limite, By Rows dá uma linha (5 quadros: 120×16; 16 quadros: 128×8), igual ao nosso.
- **Curas do leitor antigo, que mudam o que `parse` devolve:** índice transparente lia o byte 24 em vez do 28 (`bg-index-3.aseprite`); paleta só no chunk `0x0004` dava rampa de cinza (`file-tests-props.aseprite`); opacidade de camada sem o bit 1 do cabeçalho vale 255. Os outros 16 `.aseprite` reais saem iguais.

### ⛔ Recusas MEDIDAS (LibreSprite 1.3, `oracle_export.json`)

| O que ele faz | O que fazemos | Porquê |
|---|---|---|
| Modo de mistura sobre destino transparente dá preto | A fonte | Trial, P3, 6 144/6 144 |
| `--scale 0.5` escala cada cel antes de compor | Escala o composto | Sai o que o artista vê; nenhum deslocamento único explica o dele |
| `--layer` de camada escondida dá folha vazia | Sai desenhada | Quem a escolheu quer vê-la |
| GIF trunca a demora: 7 ms → 0, 15 → 10 | Arredonda, mínimo 10 ms | Demora 0 vira 100 ms em muitos leitores |
| `--sheet-type vertical` devolve a horizontal | Uma coluna | Defeito dele |
| `rows --sheet-height` e `columns --sheet-width` ignoram o limite | O limite vale | O diálogo do Aseprite oferece-o (foto 31) |
| `{frame01}` não existe | Largura e início pelos dígitos | CLI do Aseprite |
| `addition`, `subtract`, `divide` saem `unknown` no JSON | Os nomes da spec | Defeito dele |
| Grava WebP só de um quadro e recusa paleta/cinza | WebP animado RGBA | Lido por `image-webp`, Pillow e ffprobe |

## 7. Pendências da P8 — resolvidas neste bloco

| Pendência | Destino |
|---|---|
| Texto mono em fontes do sistema contra FreeType (1–2 células) | **Fecha registrando:** o produto já não oferece fontes do sistema. Nota em `crates/ph2d-pixel-text/fonts/README.md`; `FontChoice::System` e `rasterize_outline` ficam só para os oráculos, conferir se saem do caminho do app |
| AA3 sem fixtura | **Fecha:** sem Antialias no painel. Registro na mesma nota |
| `edges-find*`, alfa nunca medido | **Feito, e a lei mudou:** eram matrizes reais, o «128» era o xadrez sob alfa 0. `trial_edges_alfa.py`, teste `o_alfa_do_edges_find_bate_com_o_trial`. |
| `curve_a11y`/`range_a11y` sem publicar | **Não se faz nesta linha (medido na P9a):** `sync_a11y` (`surface_input.rs:224`) só publica a janela e as abas de família; **nenhum** widget de painel chega à árvore, nem os sliders. Ligar só estes dois não muda nada para quem usa leitor de ecrã. É uma lacuna do app inteiro: pede a sua própria onda. |
| Mistura parcial | **Feito:** saíram `carimbo::tinge`, `over` e o `below` do `stamp`; `texto::mistura` virou `na_cor`. |

## 8. Ordem e contratos

Um bloco (§0.10), com **uma** janela nova entre as ondas:
- **P9a — motor:** rodada M1–M9 → `.ase` ler e gravar → `render::frames` → formatos (GIF, APNG, WebP, PNG indexado) → `sheet::layout` + JSON → `abrir` + pendências. Série só onde M1–M9 mudam a lei.
- **P9b — tela:** crate do painel → Export As → folha com prévia → Import → Open → Repeat → cena **20** (`smoke.rs:32`, `max_level` 19→20), com fotos. **Smoke do dono.**

Contratos §6 e `PROJECT_SCHEMA`: não encosta (só o MINOR do blob). `PH2DPIXEL` é formato gravado: leitura do antigo com gate. Stack: `png`, `image-webp` e `miniz_oxide` na versão do lock (`STACK_VERSOES`), nenhuma cópia nova, conferir `miniz_oxide` 0.8 × 0.9.

## 9. Perguntas ao dono (antes de codar)

1. **Onde ficam os comandos?** Recomendo o menu **File** de sempre, ao lado de Export SVG, visível com um Pixel Sprite em Draw, e com os atalhos do Aseprite. Alternativa: um cartão no painel Pixel.
2. **O que faz arrastar um `.ase` para o app?** Hoje vira folha animada, a que o Game Engine usa. Recomendo manter, e abrir para editar pelo `File › Open`. Alternativas: passar a virar Pixel Sprite, ou perguntar a cada vez.
3. **GIF com mais de 256 cores?** Recomendo reduzir sozinho às cores mais próximas, sem pontilhado, e avisar. Alternativas: com pontilhado, ou recusar.

**Assumido, salvo veto:** o desenho mora no projeto: `Ctrl+S` grava o projeto, e `.ase` é cópia exportada · abrir ou importar cria um Pixel Sprite novo · JPEG (borra o pixel) e QOI ficam fora.

**Respostas do dono (05/11):** 1. Menu File. · 2. Arrastar `.ase` fica como está; editar é pelo File › Open. · 3. GIF: reduz às cores mais próximas, sem pontilhado, e avisa.
Plano inteiro e «Assumido» aprovados, sem veto («comece», 05/11).
