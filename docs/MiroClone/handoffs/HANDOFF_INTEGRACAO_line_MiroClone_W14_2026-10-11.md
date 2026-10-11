# HANDOFF DE INTEGRAÇÃO — `line/MiroClone` W14 → `main` (2026-10-11)

> Para o **agente integrador** (DIRETRIZ §1.5.3–1.5.4, §1.5.9). A linha FECHOU e PAROU: não integra nem pusha.
> **Integra-se JUNTO com a W12+W13** ([handoff](HANDOFF_INTEGRACAO_line_MiroClone_W12_W13_2026-10-11.md), que segue
> válido para o que é delas): a linha nunca foi ao `main` desde `8f20c60de`. Mecanismo da W14:
> [ADR-0184](../../architecture/decisions/0184-a-imagem-do-quadro-e-um-ficheiro-por-conteudo-que-viaja-sozinho.md) ·
> oráculo Excalidraw 0.18.1 [`excalidraw_oracle/`](../ferramentas/excalidraw_oracle/README.md) §«W14» (`saidas/w14/`) ·
> briefings: [soltar no Wayland](HANDOFF_CONTINUACAO_W14_SOLTAR_NO_WAYLAND_2026-10-11.md) ·
> [a imagem no quadro](HANDOFF_CONTINUACAO_W14_IMAGEM_NO_QUADRO_2026-10-11.md).

## 1. Identidade

| | |
|---|---|
| branch | `line/MiroClone` · worktree `Worktrees/line-MiroClone` |
| base (merge-base) | `8f20c60de` (o `main` de 11/10) — se o `main` não andou, `--ff-only` |
| commits | 43 sobre a base (+ este handoff): W12+W13 até `a269acab2`; a W14 de `109f722af` a `76da061f3` (14) |
| diff em `crates/`+`shells/` | `git diff --shortstat 8f20c60de..HEAD -- crates shells`: 167 ficheiros, +14 705 / −966 (só a W14, `a269acab2..`: 95 ficheiros, +6 405 / −282) |
| smoke | «Forma»: **OK do dono** (11/10). Soltar no Wayland: **OK do dono** (11/10). A imagem no quadro (cena 15): **OK do dono** (11/10) |

O que a W14 trouxe (ordem do dono 11/10: *«ainda não podemos arrastar imagens nem para os quadros nem para o canvas»*):

1. **O botão «Forma»** da barra de contexto troca o TIPO das formas seleccionadas (`935aed878`).
2. **Soltar ficheiros no Wayland** — o winit 0.30 não entrega `DroppedFile` lá: um receptor de DnD ao lado dele
   (drop-crate `ph2d-wayland-dnd`, `f8ee5309b`); na cena a imagem entra no ponto. No fecho o receptor saiu da `App`
   para um `thread_local` do `input_drop.rs` (ligado no 1.º quadro com janela) — a catraca da `App` (§4).
3. **A imagem no quadro**: `ElementKind::Image` (recorte, espelho, cantos, opacidade); os bytes UMA vez por conteúdo no
   quadro; soltar (no ponto), colar (`Ctrl+V` de uma captura), «Open an Excalidraw file or a picture…» (um
   `.excalidraw` COM imagens, ou uma imagem no quadro aberto); redimensionar mantém a proporção; setas prendem-se;
   copiar para outro quadro leva os bytes; a barra mostra cantos + opacidade + organizar; PNG/SVG/PDF/`.excalidraw`
   levam-nas; ao vivo, um ficheiro por mensagem. **Formato 12**, **protocolo 5**. Cena 15 `PH2D_BOARD_SMOKE=15`.

## 2. Foundational / partilhado tocado (W14) — e porque é aditivo

| onde | o quê | porque é aditivo / seguro |
|---|---|---|
| `ph2d-board-model` | `image.rs` (`FileId` sha256, `Blob`, `ImageFile`, `Image`, `Crop`, `Files`, `Ingest`, `mime_of`); `ElementKind::Image` no FIM; `Field::Image` no FIM (`Field::ALL` 37 → 38); `Board::files`, `Version::files`; `BoardSet::to_bytes` grava só os ficheiros em uso; `board_bytes_lean`; `insert_after` copia os da versão; **`FORMAT_VERSION` 11 → 12** + `legacy_v11.rs` (e a versão dos formatos 8–11 por `VersionV11`); dep `sha2 0.10` (já no lock pelo `ph2d-board-net`) | crate da linha; todos os formatos anteriores leem-se (fixture `format_v11.bin` gravada pelo build `72b26c071`, gate `a_format_11_file_still_opens_and_rewrites_in_the_current_format`); gates de desenho dos espelhos `BoardOut`/`Lean`/`BoardRef` = `Board` |
| `ph2d-vector` | `DrawOp::Image(Box<ImageOp>)`, `ImageSource`, `VectorScene::draw_recorded_image` (a porta que grava da `draw_stable_image_transformed`) | variante nova no fim; os consumidores de `DrawOp` são só o export do quadro e testes (`cargo check --workspace --all-targets` verde) |
| `ph2d-asset` | `pub fn decode_image(bytes) -> Result<Asset, AssetError>` (a porta pública do `decode_image_bytes` que a cena já usava) | só acrescenta uma `pub fn`; nenhum chamador muda |
| `ph2d-board-sync` | `Write::File` no FIM, `replica_files.rs` (`by_file`, `all_files`, `file_in`), o retrato magro, `rejoin` guarda os ficheiros; **`PROTOCOL` 4 → 5** | um par do protocolo 4 é recusado com a frase de sempre |
| `ph2d-board-render/-export/-excalidraw/-edit/-geom/-route` | crates do quadro: `Images` (pirâmide fora do fio, `install_decoder`), SVG `<image>`/`<use>`, PDF XObject (JPEG por `DCTDecode`), `read_with`, `insert_images`, `geom::shape_of` (a imagem ocupa a caixa dela) | só o Quadro as consome |
| `ph2d-editor-core` (tela do quadro) | `board_images.rs` (`insert_images`, `takes_image_paste`); `Selected::Images`; `board_keys` leva os bytes da cópia; `Documents::import_board(…, files)` | ficheiros do quadro; todos os chamadores no diff |
| `ph2d-app-board` | `images.rs` (a porta de entrada `ingest`, `drop_files`, `paste`, `install`), `live_send.rs` (cortado do `live.rs`), a cena 15; deps `ph2d-asset`, `png 0.18`, `image 0.25` (todas já no lock) | família do quadro |
| `ph2d-i18n` | `board_images.rs` (chaves `board.images.*`, `board.io.filter_*`, `board.io.image_added`, `board.live.too_big`, `board.smoke.images.*`); `board.menu.import` → «Open an Excalidraw file or a picture…» (as duas lições que o citam, cenas 10 e 11, acompanham); **sai** `shell.input_drop.board_takes_no_files_yet` | chaves novas; a removida não tem leitor |
| `shells/desktop` | `init.rs` instala o descodificador (1 linha); `input_drop.rs`: o quadro recebe as imagens (a recusa sai) e o receptor do Wayland em `thread_local`; `keyboard_board.rs`: `Ctrl+V` com imagem; `main.rs`/`app_state.rs`: o campo `wayland_drops` SAI | composição; a shell fica nos tectos (§4) |
| `docs/architecture/decisions/` | ADR **0184** + `README.md` regenerado (`adr-index.sh`) | o número SOMA entre linhas — reconte contra o `main` do dia (esta linha cria 0182, 0183, 0184) |

## 3. Contratos e schemas — a prova de que NÃO foram tocados

`bash scripts/collision-surface.sh` (11/10, HEAD `5a0dba3b2`) — referência, não prova:

```text
▸ SCHEMAS            PROJECT_SCHEMA 188 (base 188) · tripla (188, 13, 22) · VEC_SCENE 22 · FLIP 13 · DOC_VERSION 20
▸ REGISTRO           ph2d-ecs 107 · ph2d-render 108 · ph2d-script 108 (base iguais)
▸ CONTRATO CONGELADO ph2d-nodegraph/src/node.rs intocado · ph2d-editor-core/src/tool.rs intocado
▸ ADR                último no disco 0183 (antes do 0184) — a linha cria 0182 0183 (+0184)
▸ Cargo.lock         1 '+name' novo: "ph2d-wayland-dnd" (a drop-crate da W14)
▸ MARCADORES         nenhum
```

`git diff 8f20c60de..HEAD -- crates/ph2d-vector-doc crates/ph2d-vec-scene crates/ph2d-nodegraph/src/node.rs
crates/ph2d-editor-core/src/tool.rs` → vazio (o contrato vectorial §6 e os de Nodes/Tools intocados). O que muda de
VERSÃO é só do Quadro: `FORMAT_VERSION` 12 (campo próprio no `.ph2dproj`, versão própria) e `PROTOCOL` 5.

## 4. O portão (1× sobre o diff acumulado, árvore `5a0dba3b2` + `d359bddac`)

| portão | resultado |
|---|---|
| `BASE=8f20c60de bash scripts/nextest-impacted.sh` | **18 543 / 18 544**; a 1 é flake de CARGA: `ph2d-physics-ecs measure_player_budget::the_cost_of_a_player_is_linear_in_their_number` (sem diff na crate; sozinha 3/3 verde, 0,15 s, com `loadavg` 83/60/60 na corrida do portão). Na 1.ª corrida, outra de carga: `ph2d-app-light2d o_oclusor_fino` (sozinha 3/3, 60 s, `loadavg` 6→4) |
| `cargo clippy --workspace --all-targets --features ph2d-spike/bevy_ecs -- -D warnings` (a linha da CI) | ✓ (exit 0, `--keep-going`) — depois de corrigir 4 lints do clippy 1.98 |
| `cargo fmt --all --check` | ✓ |
| `ph2d-host-desktop --test it` (inclui `file_loc_caps`, `fn_loc_caps`, `the_app_only_sheds_fields`) | ✓ 796/796 |
| `ph2d-editor-core --test it` (`architecture_*` + `arch_safe_clamp_only`) | ✓ 113/113 |
| `board_paint_never_bypasses_the_recording` · `typos` no diff | ✓ · ✓ |
| `cargo check --workspace --all-targets` (`CARGO_BUILD_WARNINGS=deny`) | ✓ |

⛔ **O que o portão apanhou (e se curou movendo, nunca subindo):** a W14-soltar (`f8ee5309b`) tinha deixado a shell
vermelha em três catracas que só correm na varredura impactada — `main.rs` 1066/1061 LOC, `App::new` 214/212, a
`App` 174/173 campos. Cura: o receptor saiu da `App` (`thread_local` em `input_drop.rs`). Fotos depois da cura:
`ferramentas/soltar_wayland/{quadro,cena}_depois.png`.

**Auditoria de duas lentes** (correcção/costura · desempenho/recursos, ~2 100 LOC lidas): 11 achados, os 11 curados
em `5a441a44a`, cada um com o teste que fica vermelho sem a cura — o texto da área de transferência ganha à imagem
(uma folha de cálculo traz as duas); a cache guarda só os níveis do pedido; descodificador em pânico não pendura; o
nível de reserva é o fino; o PDF põe cada ficheiro uma vez e o JPEG como chegou; o tamanho pelo cabeçalho; ao vivo só
os ficheiros referidos; mensagem acima do `MAX_FRAME` diz-se no quadro; o 1.º filtro do «Import» abre tudo; o
`Ctrl+seta` não cai sobre uma imagem; o descodificador do app lê-se ao descodificar.

Testes novos da W14-imagem: 53 (`#[test]`, 3 deles réguas/gravadores `#[ignore]`) — model 11, excalidraw 7, render 10,
export 5, edit 6, sync 3, editor-core 4, app-board 7.

## 5. Números MEDIDOS

| o quê | medido |
|---|---|
| a entrada de uma fotografia JPEG 4000×3000 (3,9 MB) no fio da janela | **1,8 ms** (cabeçalho) contra **99,7 ms** da descodificação inteira, que corre fora dele — perfil `smoke`, mínimo de 5 intercaladas, `loadavg` ~36 (`mede_a_entrada_e_a_descodificacao_de_uma_fotografia`) |
| PDF com a MESMA fotografia JPEG 400×300 em 2 páginas | 1 XObject `DCTDecode` com os bytes do JPEG; o PDF < JPEG + 20 KB (gate `a_jpeg_goes_into_the_pdf_once_and_as_it_arrived`) |
| uma imagem 4096² vista a 100 px | desenha o nível de 128 px e guarda só ele (gates `far_away_a_big_image_draws_a_small_level`, `a_small_view_keeps_only_the_levels_it_asked_for`) |
| o Excalidraw a abrir o NOSSO `.excalidraw` com imagens | as três imagens no sítio, recorte+espelho e redonda rodada certos (`saidas/w14/w14_nosso.png`); `restore` → o nosso leitor = o mesmo quadro (gate `the_excalidraw_round_trip_returns_our_images`) |
| rsvg-convert · resvg · pdftoppm sobre o nosso SVG/PDF | as duas imagens iguais nos três (verificado à mão, `grava_svg_e_pdf_com_imagens`) |
| `ph2d-board-render` a desenhar o cinzento enquanto descodifica | o 1.º quadro não espera (gate `the_screen_shows_the_placeholder_until_the_pixels_arrive`) |

## 6. Abertos (não são da W14 — nomeados para a próxima)

- **Editar o recorte** (o modelo, o `.excalidraw` e o desenho já o têm; falta o gesto — duplo clique como no Excalidraw)
  e **espelhar pela barra** (o modelo tem `flip`).
- **Imagens SVG** (precisam de rasterizar por nível) — hoje um `.svg` largado num quadro conta como «não é imagem».
- GIF animado mostra o 1.º quadro; `Ctrl+V` de ficheiros copiados no Dolphin (`text/uri-list`) não entra como imagem.
- Uma imagem com o cabeçalho certo e os dados estragados entra e fica no cinzento (o desenho falha-a fora do fio).
- O export descodifica a mesma imagem uma vez por página (tempo, não tamanho: o PDF já a põe uma vez).

## 7. O integrador

1. `git -C Worktrees/line-MiroClone log --oneline 8f20c60de..HEAD` = 43 + este handoff. Se o `main` andou: `git
   rebase main` na linha, conflitos pelos estágios (DIRETRIZ §1.5.5), `Cargo.lock` regenerado.
2. **Reconte o ADR** (0182, 0183, 0184 desta linha contra o máximo do `main`) e regenere o índice (`adr-index.sh`).
3. `FORMAT_VERSION`/`PROTOCOL` são do Quadro (não somam com outras linhas).
4. Árvore combinada: `cargo check --workspace --all-targets` + os portões do §4.

## 8. Smoke do dono

O binário está compilado na worktree (`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`,
2.ª corrida, depois de `rm -rf target/*/incremental`):

```text
    Finished `smoke` profile [optimized] target(s) in 0.33s
```


1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && PH2D_BOARD_SMOKE=15 ./target/smoke/ph2d-host-desktop`
2. Abre um quadro «Pictures» com uma moldura «Moodboard» (uma fotografia, o sol recortado, a mesma redonda e
   rodada, um ícone redondo laranja) e seis notas amarelas por baixo — cada nota é um passo a experimentar.
3. Siga as notas: largue uma imagem do Dolphin no quadro (cai onde a larga); arraste um canto (não estica); clique numa
   imagem (a barra mostra cantos e opacidade); copie uma captura de ecrã e `Ctrl+V`; `Ctrl+C` numa imagem, «+» para um
   quadro novo, `Ctrl+V`; botão direito na aba ▸ Export… (PNG/SVG/PDF/Excalidraw levam as imagens).
4. Errado: uma imagem que fica um quadrado cinzento mais de um segundo; a imagem largada longe do ponto; a colagem
   noutro quadro aparecer cinzenta; um ficheiro exportado sem as imagens.
