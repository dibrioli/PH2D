# CONTINUAÇÃO — `line/MiroClone` W15: moldura e P&B nas imagens, SVG no quadro, «Send to board…» da Hierarchy (2026-10-11)

> Rota `/pd-linha-assumir`: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. Estado: a W14 FECHOU (smoke OK do dono 11/10) — [handoff de integração W14](HANDOFF_INTEGRACAO_line_MiroClone_W14_2026-10-11.md)
> (integra-se com a W12+W13; **não integre, não pushe**). Lei da imagem: [ADR-0184](../../architecture/decisions/0184-a-imagem-do-quadro-e-um-ficheiro-por-conteudo-que-viaja-sozinho.md).
> O que já existe e NÃO se reconstrói: `ElementKind::Image` (recorte, espelho, cantos, opacidade), `Board::files`, a
> porta de entrada `ph2d-app-board::images::ingest`, `Editor::insert_images`, `board_images::insert_images` (editor-core),
> `Images` (pirâmide fora do fio, `install_decoder`), `DrawOp::Image`, SVG/PDF/`.excalidraw` com imagens, `Write::File`.

## Ordem do dono (11/10, depois do smoke OK da W14)

> *«Gostaria de moldura (stroke) para imagem como as shapes. Gostaria de um filtro preto e branco também com botão na
> barra superior. Imagens SVG e FLIP também devem poder ser importadas. Na hierarchy precisamos de uma opção no menu do
> botão direito: "Enviar para o quadro..." e um modal para escolher o quadro que recebe a img, vetor ou flip.»*

Leitura (a confirmar no smoke, não com perguntas): «barra superior» = a barra de contexto que aparece POR CIMA da
imagem seleccionada (a das formas). O Flip não tem ficheiro próprio (vive no `.ph2dproj`, `FlipDoc::to_bytes`,
`crates/ph2d-flip/src/doc.rs:85`): «importar Flip» = mandá-lo da cena pela Hierarchy (item 4).

## Os quatro itens — num bloco só (§0.10), com o que já se mapeou

### 1. Moldura (stroke) — a imagem com contorno, cor, espessura e traço como as formas
- O `Style` da imagem JÁ tem `stroke`, `stroke_width`, `dash`, `sketch` (e os registos ao vivo `Field::Stroke`…):
  **sem mudança de formato**. Hoje o `foreign`/`write` do Excalidraw zeram o `stroke` (`ph2d-board-excalidraw/src/image.rs`).
- Desenho: contornar a caixa (redonda se `round`, `box_path` em `ph2d-board-render/src/image.rs`) DEPOIS da imagem, fora
  da camada do recorte, com a tinta do tema (`cache.ink.color`) e o traço à mão se `sketch` (como `paint_shape`).
- Barra: `Selected::Images` (`board_bar.rs:286`) ganha os grupos `stroke`, `width`, `dash` (as listas de `style_groups`).
- ⭐ MEÇA primeiro no oráculo: o Excalidraw 0.18.1 desenha `strokeColor` numa imagem? (corra `corre.sh` sobre uma
  entrada `w15_*` com `strokeColor` e `strokeWidth`). Se não desenha, a moldura viaja só no `customData` e o
  `07_fidelidade_excalidraw.md` diz que é extensão nossa.

### 2. Filtro preto e branco — um botão na barra da imagem
- Campo novo `Image.gray: bool` ⇒ **FORMATO 13** (`legacy_v12.rs` com fixture gravada pelo build ANTES da mudança, como
  a `format_v11.bin` desta W14: `git archive HEAD crates/ph2d-board-model` num `CARGO_TARGET_DIR` próprio) e o
  `Field::Image` leva-o (o tuplo `(file, crop, flip)` → `(file, crop, flip, gray)`); `PROTOCOL` 5 → 6.
- A lei da luminância NÃO se inventa: porte a canónica do repo (procure `luminance`/`desaturate`/`grayscale` em
  `ph2d-color`, `ph2d-painter-effects`, `ph2d-blend-mode`) e escreva o gate de paridade (§4 da DIRETIVA).
- Duas vias a MEDIR antes de escolher (régua: igualdade dos três destinos ecrã/SVG/PDF): (a) pirâmide CINZENTA na CPU
  (`Images` chaveada por `(FileId, gray)`; o SVG embebe a PNG cinzenta, o PDF `DeviceGray`) — exacta e simples; (b) uma
  camada `Mix::Saturation` + `Compose::SrcAtop` de cinzento por cima da imagem (GPU, sem cópia) — exige `mix-blend-mode`
  no SVG e `/BM` no PDF (o `svg.rs`/`pdf.rs` de hoje ignoram o `blend` da `DrawOp::Layer`) e o librsvg/resvg/Inkscape a
  respeitá-lo (`fumo_svg_pdf.sh`). Recomendação de partida: (a).
- O botão: um `Item` novo (o molde é o «Forma», `935aed878`: `board_bar.rs` variante + `items()` NO FIM + `apply`,
  `board_bar_look.rs` rótulo e cara, `ph2d-i18n/src/board_images.rs`, teste de costura em `board_images_tests.rs`). Aceso
  quando a selecção é toda cinzenta; UM passo de desfazer.

### 3. SVG como imagem no quadro (soltar, colar, «Open… a picture», `.excalidraw` com `image/svg+xml`)
- Hoje: `ingest` recusa SVG (`ph2d_asset::decode_image` começa por `image::guess_format`, `loader.rs:137`); na CENA o SVG
  vira desenho vectorial (`ph2d-app-vec/src/svg_import.rs:64`, `import_router.rs:82`). `ph2d-imageio-svg` só faz parse
  (`usvg 0.48`, `Cargo.toml:38`; o `resvg` está «adiado» lá).
- Precisa de um RASTERIZADOR: `resvg` da mesma série do `usvg 0.48` — **triagem de licença por artefacto** (§0.9) e
  `bash scripts/stack-audit.sh --tetos` ANTES de o pôr (duas cópias do `tiny-skia`/`usvg` = recusa).
- Um SVG é vector: a pirâmide rasteriza-o à resolução do NÍVEL pedido (nítido a qualquer zoom), não por metades — o
  `Decode` ganha o tamanho-alvo para SVG. Tamanho natural = `width`/`height`/`viewBox` do `usvg::Tree`. `mime_of` aprende
  `image/svg+xml` (texto: `<svg` depois de espaços/`<?xml`/comentários). O SVG exportado embebe o SVG original (o
  `ImageSource` já leva os bytes); o PDF rasteriza ao nível inteiro. O Excalidraw aceita `image/svg+xml` (confirme no
  oráculo com uma entrada `w15_svg`).
- ⛔ Um `.svg` largado na CENA continua desenho vectorial (a escolha é pela aba activa — `input_drop.rs`).

### 4. Hierarchy ▸ botão direito ▸ «Send to board…» + o modal que escolhe o quadro (imagem, vector, Flip)
- Menu: `HIERARCHY_ROW_ROWS` (`crates/ph2d-editor-core/src/screens/hero/menu_tables.rs:450`), ids em
  `ids/menus_hierarchy.rs` (`CTX_MENU_HIER_*`, o molde é `EXPORT_IMAGE` `:118`), pedido `HierRequest`
  (`action_bus_hier.rs:143`), despacho `shells/desktop/src/render_loop/fase_bus_hierarchy.rs:175`, aplicação como o
  `export_image_row` (`fase_sheet_verbs.rs:166`). ⚠️ A shell só ENCOLHE: o corpo vai para `ph2d-app-board` (a shell chama).
  A linha só aparece para `ObjectKind::{Image, Vector, Flip}` (`ph2d-component-desc/src/lib.rs:349`, `kind_of` em
  `ph2d-app-components/src/component_attach.rs:23`).
- O modal: a lista dos quadros abertos (+ «New board»), no idioma do `ContextMenuKind::SceneList`
  (`interaction/types_menu.rs:184`, com busca). Escolher ⇒ a imagem entra no MEIO da vista desse quadro
  (`board_images::insert_images(…, None)` depois de o activar) e a aba dele abre-se.
- Os píxeis/bytes de cada tipo:
  - **Imagem (sprite):** o RGBA do `AssetDb` pelo `atlas_asset_map` (`hero_intents/texture_edit.rs:61`
    `read_sprite_source`; o `image_export.rs:111` `source_for` já dá alfa reto) → PNG → `ingest`.
  - **Vector:** há SVG (`shells/desktop/src/vec_svg_export.rs:27`, crate `ph2d-vec-svg`) — mande-o como SVG (item 3:
    fica nítido), não como píxeis.
  - **Flip:** NÃO há raster de um quadro do Flip para ficheiro (`ph2d-flip-render` compõe na placa,
    `composite.rs:219/380`; leitura de textura genérica em `ph2d-render/src/individual_read.rs` `readback_rgba8`). MEÇA
    as vias: o quadro ACTUAL do Flip lido da placa (`readback_rgba8`) ou um SVG dos traços (se o traço percorrido tiver
    contorno exportável). Um quadro (o actual), não a animação — diga-o no smoke.

## Portões e réguas a desenhar ANTES de codar

- Costura (`ph2d-ui-testkit`/o arnês `T` de `board_bar_tests.rs`): a moldura e o P&B pelo clique real na barra; a linha
  «Send to board…» pelo clique real no menu da Hierarchy e a escolha no modal (o arnês dos menus do hero).
- Paridade: P&B no ecrã = no PNG/SVG/PDF (os rasterizadores do sistema: `grava_svg_e_pdf_com_imagens` é o molde).
- O SVG nítido: a mesma imagem SVG a 100 px e a 4000 px desenha níveis rasterizados a ESSES tamanhos (não ampliados).
- Formato 13 lê o 12 (fixture), e o oráculo corre de novo sobre o `w14_nosso` (o escritor muda com `gray`/moldura).
- Cena de smoke nova `PH2D_BOARD_SMOKE=16` (conte no roteador `smoke.rs`, `max_level`): uma moldura de cada estilo, uma
  P&B ao lado da colorida, um SVG nítido ao zoom, e uma nota por lição — incluindo «na aba Scene, botão direito num
  objecto da Hierarchy ▸ Send to board…». Fotografe-a (`docs/MiroClone/ferramentas/soltar_wayland/no_app.sh <nome>
  PH2D_BOARD_SMOKE=16`, sem embrulhar o roteiro num `PH2D_GPU=1`: o comando de dentro fica à espera do cadeado).

## Armadilhas MEDIDAS na W14 (não repita)

| o quê | medido |
|---|---|
| `nextest-impacted` é o único que corre as catracas da shell (`file_loc_caps`, `fn_loc_caps`, `the_app_only_sheds_fields`) | a W14-soltar deixou-as vermelhas sem ninguém ver; cura = MOVER (o receptor saiu da `App`) |
| clippy 1.98 da CI (`--workspace --all-targets --features ph2d-spike/bevy_ecs -D warnings`) | `chunks_exact_to_as_chunks`, `cloned_ref_to_slice_refs`, `manual_checked_ops` — o `-p` local não os mostrava todos |
| `Element::shape` (forma de função) escapa a um grep de `.shape()` | o `missing()` do roteador deixava as setas soltas da imagem |
| um texto de menu citado numa lição | mudar «Open an Excalidraw file…» obrigou as lições das cenas 10 e 11 e um teste |
| `live.rs` a 690 LOC, `pdf.rs` ~650, `render/lib.rs` ~600, `board-edit/lib.rs` ~655, `gesture.rs` ~690 | tecto 700 por ficheiro: código novo em módulos irmãos |
| `no_app.sh "nome K=V"` entre aspas | vira UM argumento: a foto sai da abertura, sem quadro |
