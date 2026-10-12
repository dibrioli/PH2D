# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W15 → `main` (2026-10-11)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem pusha.
> **Integra-se JUNTO com a W12+W13 e a W14** ([W12+W13](HANDOFF_INTEGRACAO_line_MiroClone_W12_W13_2026-10-11.md) ·
> [W14](HANDOFF_INTEGRACAO_line_MiroClone_W14_2026-10-11.md), que seguem válidos para o que é delas): a linha nunca
> foi ao `main` desde `8f20c60de`. Briefing da onda: [W15](HANDOFF_CONTINUACAO_W15_IMAGENS_DA_CENA_2026-10-11.md).
> Oráculo Excalidraw 0.18.1 [`excalidraw_oracle/`](../ferramentas/excalidraw_oracle/README.md) §«W15» (`saidas/w15/`).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| base (merge-base) | `8f20c60de` (o `main` de 11/10) — se o `main` não andou, `--ff-only` |
| commits da W15 | `84a111d63..9bd06c29b` (22) + este handoff; a linha inteira: `git log --oneline 8f20c60de..HEAD` |
| diff da W15 em `crates/`+`shells/` | `git diff --shortstat 3e45504e5..HEAD -- crates shells`: 89 ficheiros, +4 082 / −222 |
| smoke | cena 16 (`PH2D_BOARD_SMOKE=16`): **OK do dono** (11/10); a foto da cena (o binário final, num KWin virtual, com uma imagem largada e a barra dela à vista) está em `ferramentas/soltar_wayland/quadro16_depois.png` |

Ordem do dono (11/10, depois do OK da W14): *«moldura (stroke) para imagem como as shapes · um filtro preto e branco
com botão na barra superior · imagens SVG e Flip também devem poder ser importadas · na Hierarchy, botão direito:
"Enviar para o quadro…" e um modal para escolher o quadro que recebe a img, vetor ou flip»*. Os quatro num bloco
(§0.10):

1. **A moldura** — o `Style` da imagem já tinha `stroke`/`stroke_width`/`dash`/`sketch` (sem formato novo): desenha-se
   o contorno do rectângulo da caixa PELA LEI DAS FORMAS (`cached_outline` + `style_stroke`, ou `sketch::paint_shape`
   à mão), por cima da imagem e fora do recorte, com a tinta do tema; com opacidade < 100 a imagem e a moldura
   compõem-se JUNTAS numa camada (a regra de uma forma). A barra da imagem ganhou cor · espessura · traço; a moldura
   nasce no modo do quadro (`Editor.style.sketch`) e o «Rascunho ↔ Final» alcança-a (`sketchable` inclui a imagem).
2. **O preto e branco** — `Image::filter: ImageFilter { Original, Gray }` (append-only) ⇒ **formato 13** e
   **protocolo 6**. Lei: a luminância relativa Rec. 709 sobre a luz LINEAR (`LinearRgba::luminance` + a curva sRGB do
   `ph2d-color`) — a mesma do `feColorMatrix type="saturate" values="0"` em `linearRGB`. Ecrã/PNG/PDF: a pirâmide é
   por `(FileId, ImageFilter)` (os píxeis já cinzentos; o PDF pÕe-nos noutro XObject e nunca passa um JPEG cinzento
   como chegou). SVG: os bytes ORIGINAIS com o filtro (um SVG embebido fica vector). Botão `Item::Gray` (o círculo
   meio cheio), aceso com todas cinzentas, UM passo.
3. **SVG como imagem** — `mime_of` reconhece um SVG (texto: BOM, `<?xml`, comentários, DOCTYPE com ou sem
   entidades); `ph2d-imageio-svg::raster` (`parse`, `natural_size`, `rasterize` pelo `resvg 0.48`, `plain`,
   `dropped_texts`); o `ingest` guarda os bytes como chegaram (um `.svgz` descomprimido com tecto), o tamanho do
   `<svg>` (senão o `viewBox`); o desenho rasteriza CADA NÍVEL ao tamanho dele (a pirâmide de um SVG começa no lado do
   atlas, 8192, e cada nível pede-se sozinho + a miniatura): nítido a qualquer zoom; no PDF a 300 ppp. Entra por soltar,
   «Open…» (svg/svgz no filtro) e `.excalidraw` com `image/svg+xml`. ⛔ Os `<text>` NÃO se desenham (a feature `text`
   do usvg é uma recusa medida, `ph2d-vec-svg/Cargo.toml`): o aviso na tela conta-os (`board.images.svg_texts`).
   ⛔ Um `.svg` largado na CENA continua desenho vectorial (a escolha é pela aba activa).
4. **«Send to board…»** (Hierarchy ▸ botão direito) — só para `ObjectKind::{Image, Vector, Flip}`: a shell publica a
   cada quadro as linhas que podem ir (`WidgetStore::set_hierarchy_board_rows`), o pintor do menu salta as outras ANTES
   do `hit_index` (pintado == clicável). O modal (molde `SceneList`: busca + lista dos quadros abertos + «New board»)
   devolve `HierRequest::SendToBoardChosen { row, board }`; a shell lê o conteúdo na família de cada tipo e entrega-o a
   `ph2d_app_board::send_to_board::deliver` (tudo pela porta `ingest`): **Image** = os píxeis da sprite (alfa reto,
   `PRECISION-READONLY`: cópia de 8 bits); **Vector** = o SVG SÓ desse objecto aos `pixels_per_meter` (`svg_at`,
   `svg_of_path`, `ph2d-app-vec`); **Flip** = o quadro ACTUAL rasterizado fora do ecrã pela placa aos
   `pixels_per_meter` (`ph2d-app-flip::pass::raster::current_frame`, sem onion nem fundo, a caixa dos traços posados
   com o raio). O conteúdo nasce no MEIO da vista do quadro escolhido (ou de um novo), cuja aba fica activa; um quadro
   nunca desenhado usa o centro da câmara (`board_images::insert_images`). Um conteúdo recusado não cria quadro nem
   troca a aba, e diz porquê.

## 2. Foundational / partilhado tocado — e porque é aditivo

| onde | o quê | porque é aditivo / seguro |
|---|---|---|
| `ph2d-board-model` | `ImageFilter`, `Image::filter` (`#[serde(default)]` para o `customData` JSON da W14), **`FORMAT_VERSION` 12 → 13** + `legacy_v12.rs` (só a variante `Image` muda); `Field::Image` leva o filtro (tuplo de 4); `mime_of` + `is_svg` | crate da linha; o 12 lê-se (fixture `format_v12.bin` gravada pelo build `3e45504e5` ANTES da mudança, gate `a_format_12_file_still_opens_with_its_images_unfiltered`); o `.excalidraw` da W14 lê-se igual (`the_excalidraw_round_trip_returns_our_images` sobre a `w14_nosso`) |
| `ph2d-board-sync` | **`PROTOCOL` 5 → 6** | um par do 5 é recusado com a frase de sempre |
| `ph2d-vector` | `ImageSource::gray: bool` | o único construtor é o desenho do quadro; os leitores são o export do quadro |
| `ph2d-imageio-svg` | `pub mod raster`; o importador de imagens passa a usar o `raster::parse` (as MESMAS defesas: tecto de bytes, nenhum `href` ao disco, e agora o gzip — de topo e ANINHADO num `data:` — com tecto); deps `resvg 0.48` (`raster-images`, sem `text`), `flate2 1` | o importador devolve o mesmo `DecodedImage::Vector` vazio; `Cargo.lock`: **3 pacotes novos** — `resvg 0.48.1`, `tiny-skia 0.12.0`, `rgb 0.8.53` (licenças Apache-2.0/MIT, `pacman -Qi resvg` e crates.io). `tiny-skia` fica em DUAS versões (0.11.4 pelo `sctk-adwaita` do winit, que não sobe — `stack-audit --tetos`): o tecto do `deny.toml` (`multiple-versions = "warn"`), nenhum tipo atravessa; o `tiny-skia-path` já estava em duas |
| `ph2d-board-render` | `Decode` ganha `at: Option<[u32; 2]>` (o tamanho de um SVG); `Images` por `(FileId, ImageFilter)`, níveis de SVG; `image_frame.rs`, `gray.rs` (`to_gray`, `gray_of`); `Pixels` deriva `Clone, Debug, PartialEq, Eq`; dep `ph2d-color` | crate do quadro; todos os `Decode` do repo mudaram no diff (app-board, testes) |
| `ph2d-board-export` | SVG: filtro `gray` (definido uma vez, `color-interpolation-filters="linearRGB"`); PDF: a chave do XObject leva `gray` e o tamanho do nível | crate do quadro |
| `ph2d-board-edit` | `Editor::{images_gray, toggle_gray}`; `sketchable` inclui a imagem; `insert_images` nasce no modo do quadro | só o Quadro consome |
| `ph2d-board-excalidraw` | nada no código: o nosso elemento inteiro já viajava no `customData` (o `filter` e a moldura vão nele); testes + a entrada `w15_nosso` do oráculo | — |
| `ph2d-editor-core` | `board_bar.rs` (`Item::Gray` no FIM dos ids, depois do `Reshape`; grupos da imagem), `board_bar_look.rs` (a cara); `send_to_board.rs` (o modal), `ContextMenuKind::SendToBoard`, `HierRequest::{SendToBoard, SendToBoardChosen}`, `CTX_MENU_HIER_SEND_TO_BOARD`, `WidgetStore::{set_hierarchy_board_rows, menu_row_is_hidden}`, `context_menu_overlay` salta a linha escondida; `board_images::insert_images` sem vista usa o centro da câmara | variantes e ids novos no fim; o menu da Hierarchy continua estático na tabela (os portões de menu cobrem a linha); só a linha nova se esconde |
| `ph2d-panel-hierarchy` | a guarda e o braço do id novo (`event.rs`) | o molde do `EXPORT_IMAGE` |
| `ph2d-app-vec` | `svg_at`, `svg_of_path` (UM objecto) pela mesma porta dos eixos (`world_to_svg`) | o export da cena inteira não muda |
| `ph2d-app-flip` | `pass_raster.rs` (`current_frame`); `pass.rs` separa o `compose` do `blit` | o ecrã compõe pelo mesmo `compose`; a leitura usa um compositor PRÓPRIO (as fatias e a frescura do ecrã ficam intactas) |
| `ph2d-app-board` | `images.rs` (SVG/svgz no `ingest`/`decode`, `Placed::{texts, warning}`), `io.rs` (filtro svg/svgz), `send_to_board.rs` (`ask`, `deliver`), a cena 16 (`smoke_frames.rs`); dep `ph2d-imageio-svg` | família do quadro |
| `ph2d-i18n` | `board_images.rs` (`board.style.gray`, `board.images.svg_texts`, `board.smoke.frames.*`), `board_send.rs` (`board.send.*`), `chrome_menus.rs` (`chrome.menu.send_to_board` = «Send to board…») | chaves novas |
| `shells/desktop` | `render_loop/fase_send_to_board.rs` (NOVO, 118 LOC: lê o conteúdo e chama a família), `snapshots_hierarchy.rs` publica as linhas, `fase_bus_hierarchy.rs`/`fase_bus_drain_out.rs`/`fase_hero_commits.rs` drenam os dois pedidos | composição; a shell fica nos tectos (§4) |

## 3. Contratos e schemas — a prova de que NÃO foram tocados

`git diff 8f20c60de..HEAD -- crates/ph2d-vector-doc crates/ph2d-vec-scene crates/ph2d-nodegraph/src/node.rs
crates/ph2d-editor-core/src/tool.rs` → vazio (o contrato vectorial §6 e os de Nodes/Tools intocados). `PROJECT_SCHEMA`
intocado (o quadro viaja como blob com versão própria). O que muda de VERSÃO é só do Quadro: `FORMAT_VERSION` 13 e
`PROTOCOL` 6 (não somam com outras linhas). Nenhum ADR novo nesta onda (as leis vivem no ADR-0184 e neste handoff).

## 4. O portão (1× sobre o diff acumulado)

| portão (árvore `10c66f842`; depois dela só a frase da lição 5, re-verificada abaixo) | resultado |
|---|---|
| `bash scripts/nextest-impacted.sh` (base `8f20c60de`) | **18 601 / 18 602**; a 1 é flake de CARGA da família conhecida: `ph2d-physics-ecs measure_player_budget::the_cost_of_a_player_is_linear_in_their_number` (sem diff na crate; sozinha 3/3 verde, 1,29–1,51 s, `loadavg` 49→38; na corrida do portão 51–65) |
| … dentro dele, pelo nome | `file_loc_caps` (4) · `fn_loc_caps` (2) · `the_shell_only_shrinks` · `the_app_only_sheds_fields` (3) · `arch_safe_clamp_only` (2) · `a_verb_that_costs_precision_says_so` (2): ✓ |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` (a linha do `ship.sh`) | ✓ exit 0 |
| `cargo fmt --all -- --check` · `typos` | ✓ · ✓ |
| depois da lição 5 (`9bd06c29b`): `ph2d-i18n` lib 24/24 + `--test it` 16/16 (`a_tabela_inglesa_fala_ingles`), o gate da cena 16 | ✓ |
| GPU `#[ignore]`: `ph2d-app-flip … --ignored current_frame` | ✓ 1/1 na placa |

⛔ **O que a 1.ª corrida do portão apanhou** (curado, `10c66f842`): o `rustfmt` de 18 ficheiros da onda, e
`a_verb_that_costs_precision_says_so` — o `fase_send_to_board.rs` lia os píxeis de uma sprite sem declarar o que faz
à precisão (`PRECISION-READONLY`: lê e manda uma CÓPIA de 8 bits; nunca escreve de volta).

**Auditoria de duas lentes** (correcção das leis novas · costura do «Send to board…», ~3 100 LOC lidas, o usvg
0.48.1 incluído): 7 achados, 6 curados em `1af6688bb`, cada um com o teste que fica vermelho sem a cura:

| achado | cura · gate |
|---|---|
| ⛔ **bomba de memória**: um SVG de 1 KB com um `<image href="data:image/svg+xml;base64,…">` em gzip — o usvg descomprime o aninhado SEM tecto (`parser/mod.rs:106`, `decompress_svgz`), e o tecto da W15 só cobria o ficheiro de topo (o largar, um par ao vivo e o importador da cena) | o `parse` passa TODO gzip pelo `plain` com tecto, e o resolvedor de `data:` também (`capped_data_resolver`) · `a_nested_gzip_data_uri_is_bounded` |
| o PDF punha o MESMO SVG desenhado pequeno e grande com os píxeis do primeiro (a chave era o ficheiro) — a própria cena 16 | a chave leva o tamanho do nível · `an_svg_drawn_twice_goes_to_the_pdf_at_each_size` |
| `#[allow(clippy::too_many_arguments)]` e a doc do `paint_item` presos à constante nova | a constante sobe · clippy |
| um SVG com DOCTYPE com entidades (o cabeçalho do Illustrator) contava como «não é imagem» | `is_svg` salta o subconjunto interno · `the_mime_comes_from_the_magic_bytes` (14 casos) |
| a leitura do Flip entrava em pânico se o mapeamento falhava (placa perdida) | `read_rgba8` devolve `None` → `RasterError::Gpu` (sem teste: precisa de uma placa a falhar) |
| no export, uma imagem ilegível perdia também a moldura | a moldura sai · `an_unreadable_framed_image_exports_its_frame` |
| (custo, não correcção) o export SVG rasteriza um SVG ao tamanho de impressão que o SVG não usa (embebe os bytes) | **aberto** (§6): a gravação não sabe o destino |

Testes novos da W15 (`#[test]`): model 4 · imageio-svg 7 · render 12 (3 `#[ignore]`: a régua e os gravadores) · export
5 (+1 gravador) · edit 0 (os da barra cobrem-no) · excalidraw 3 (+1 gravador) · editor-core 4 (barra) + os do modal ·
app-board 6 + os do envio · panel-hierarchy 2 ficheiros · app-vec 1 · app-flip 2 (1 GPU `#[ignore]`, verde na placa).

## 5. Números MEDIDOS

| o quê | medido |
|---|---|
| o Excalidraw 0.18.1 desenha a moldura de uma imagem? | **não**: 0 píxeis vermelhos (`#e03131`, largura 4, contínua/tracejada/redonda) no PNG e no SVG dele; guarda `strokeColor`/`strokeWidth`/`strokeStyle` na volta (`saidas/w15/`, `07_fidelidade_excalidraw.md`) ⇒ extensão nossa, no `customData`; uma imagem DELE nunca ganha moldura (`a_foreign_image_never_gets_a_frame`) |
| o Excalidraw aceita `image/svg+xml`? | **sim**, devolve o mesmo `mimeType`/`dataURL`; só com `viewBox` o PNG dele sai 80×80 no canto (diverge do SVG dele) |
| o cinzento de `200,40,10` nos destinos (ecrã = **104**) | `rsvg-convert` 104 · Inkscape 103 · ImageMagick 103 · `resvg` 102 · `pdftoppm` 104 · `gs` 104 (`fumo_w15_cinza.sh`, `saidas/w15/fumo_cinza.txt`); a mesma conta em sRGB daria 72 ⇒ gate a ±2 |
| `to_gray` == `SrgbRgba::to_linear().luminance()` re-codificado | bit a bit em 52³ cores + os 256 cinzentos (`the_gray_is_the_canonical_luminance_bit_for_bit`) |
| o custo do preto e branco | **78 ms** numa fotografia 4000×3000 de ruído (`--release`, mínimo de 5, `load 69`), fora do desenho ⇒ nenhuma tabela nem GPU |
| o SVG nítido | o mesmo SVG a 100 e a 4000 px: borda de ≤ 4 píxeis de alfa parcial nos dois (`it_rasterizes_at_the_asked_size_not_upscaled`); no quadro a zoom 40 o nível desenhado tem 4000–8192 px (`an_svg_draws_a_level_rasterized_at_the_screen_size`) |
| o Flip para o quadro | um traço de 2,0 unidades com raio 0,1 a 100 px/unidade sai 222 × 22 px (a caixa + 1 px de margem por lado); escalado 2× sai 442 × 42 (teste GPU) |

## 6. Abertos (nomeados)

- **O passo da shell do «Send to board…»** (`SendToBoard` → modal, `SendToBoardChosen` → `deliver`) não tem teste
  que o atravesse inteiro: os testes do editor e da família encontram-se no valor do pedido. É o smoke do dono que o
  fecha.
- **A imagem vai SEM a pose da cena** (rotação/escala da sprite): vai a sprite. O Vector e o Flip vão posados.
- **Um objecto por vez** (o da linha clicada, não a selecção), sem os filhos de um vector; um objecto com o olho
  desligado na Hierarchy vai na mesma (foi o pedido).
- **Os `<text>` de um SVG** não se desenham (a recusa medida da feature `text`); o aviso diz quantos.
- **O export SVG rasteriza um SVG ao tamanho de impressão** que não usa (só custo: a gravação é a mesma para o SVG e o
  PDF).
- **Memória do Flip grande**: o tecto é o `max_texture_dimension_2d` da placa (8192); um Flip de várias camadas a 8192²
  aloca uma fatia de 256 MiB por camada no compositor da leitura.
- Do W14 continuam: editar o recorte, espelhar pela barra, GIF animado (1.º quadro), `Ctrl+V` de `text/uri-list`.

## 7. O integrador

1. `git -C Worktrees/line-MiroClone log --oneline 8f20c60de..HEAD` — W12+W13+W14+W15 + os handoffs. Se o `main`
   andou: `git rebase main` na linha, conflitos pelos estágios (DIRETRIZ §1.5.5), `Cargo.lock` regenerado (os 3
   pacotes novos da W15 vêm do `resvg`).
2. Reconte os ADR da linha (0182–0184, das ondas anteriores) contra o máximo do `main` e regenere o índice.
3. `FORMAT_VERSION`/`PROTOCOL` são do Quadro.
4. Árvore combinada: `cargo check --workspace --all-targets` + os portões do §4.

## 8. Smoke do dono

O binário está compilado na worktree (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`,
2.ª corrida, depois de `rm -rf target/*/incremental`):

```text
▸ linha line_miroclone · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.21s
```

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=16 ./target/smoke/ph2d-host-desktop`
2. Abre o quadro «Frames & B/W»: em cima três fotos com moldura (preta, vermelha tracejada, azul à mão e redonda); por
   baixo a mesma foto a cores e a preto e branco, e o logótipo SVG pequeno e grande; seis notas amarelas com os passos.
3. Siga as notas: clique numa foto ▸ na barra escolha uma cor (o quadrado riscado = sem moldura), a espessura e o
   traço; o círculo meio preto liga/desliga o preto e branco; «Sketch» na barra da esquerda põe a moldura à mão; largue
   um `.svg` e dê zoom (fica nítido); na aba **Scene**, botão direito numa imagem, vector ou Flip da Hierarchy ▸
   **Send to board…** ▸ escolha o quadro (ou **New board**); exporte (PNG/SVG/PDF).
4. Errado: a moldura não aparece ou aparece na foto errada; o preto e branco não volta às cores; o SVG fica borrado
   ao zoom; a linha «Send to board…» aparece num objecto que não é imagem/vector/Flip, ou o conteúdo não chega ao
   quadro escolhido; o export sem as molduras ou a cores.
