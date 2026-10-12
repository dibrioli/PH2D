# CONTINUAÇÃO — `line/MiroClone` W16: o que ficou aberto das imagens (2026-10-11)

> **FECHADO — ver [HANDOFF_INTEGRACAO_line_MiroClone_W16_2026-10-12.md](HANDOFF_INTEGRACAO_line_MiroClone_W16_2026-10-12.md).**

> Rota `/pd-linha-assumir`: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone && pwd && git branch --show-current`
> ANTES de ler. Estado: a W15 FECHOU com **smoke OK do dono (11/10)** —
> [handoff de integração W15](HANDOFF_INTEGRACAO_line_MiroClone_W15_2026-10-11.md) (integra-se com a W14 e a W12+W13;
> **não integre, não pushe**: integrar é só por ordem do dono). Lei da imagem: ADR-0184.

## O que já existe e NÃO se reconstrói

`ElementKind::Image` com recorte, espelho, cantos, opacidade, **moldura** (o `Style`: `stroke`/`stroke_width`/`dash`/
`sketch`, desenhada em `ph2d-board-render/src/image_frame.rs` pela lei das formas) e **filtro** (`Image::filter`,
formato 13, protocolo 6; `gray.rs` = Rec. 709 linear, paridade bit a bit); a pirâmide por `(FileId, ImageFilter)` com
níveis de SVG rasterizados ao tamanho de cada um (`image.rs`); `ph2d-imageio-svg::raster` (`parse` com tecto — o gzip
aninhado incluído —, `rasterize`, `plain`, `dropped_texts`); a porta `ph2d-app-board::images::ingest`; o «Send to
board…» da Hierarchy (`send_to_board.rs` no editor-core e na app-board, `fase_send_to_board.rs` na shell,
`ph2d-app-vec::svg_at`, `ph2d-app-flip::pass::raster::current_frame`); a barra da imagem (`Item::Gray` no fim dos ids);
as cenas 15 e 16.

## Abertos herdados (W14 + W15), num bloco só (§0.10) — confirme cada um no CÓDIGO antes de construir

1. **Editar o recorte** pelo gesto (duplo clique numa imagem, como no Excalidraw — corra o oráculo para a lei do gesto)
   e **espelhar pela barra** (o modelo, o `.excalidraw` e o desenho já têm `crop`/`flip`).
2. **«Send to board…» da selecção inteira** (hoje vai só a linha clicada) e **a imagem com a pose da cena** (rotação/
   escala da sprite; o Vector e o Flip já vão posados) — e um teste que atravesse a fase da shell inteira
   (`SendToBoard` → modal → `SendToBoardChosen` → `deliver`), que hoje só o smoke cobre.
3. **O export SVG não rasteriza um SVG que não usa** (a gravação é a mesma para SVG e PDF: dê-lhe o destino, ou peça a
   miniatura quando o destino embebe os bytes) — MEÇA antes e depois.
4. **GIF animado** (hoje o 1.º quadro) e **`Ctrl+V` de ficheiros copiados no Dolphin** (`text/uri-list`).
5. **Memória do Flip grande**: a leitura aloca uma fatia de 256 MiB por camada a 8192² — meça um Flip de várias
   camadas e decida pelo recurso medido (§0.0), não por um número escolhido.

⛔ Recusas que seguem de pé: a feature `text` do usvg (segunda stack de fontes, `ph2d-vec-svg/Cargo.toml`) — os
`<text>` de um SVG avisam-se, não se desenham.

## Fecho

Portão batched 1× (`nextest-impacted`, clippy da CI, `fmt`, `typos`, auditoria de 2 lentes), cena de smoke nova
`PH2D_BOARD_SMOKE=17` (conte no roteador `smoke.rs`), fotografada com `docs/MiroClone/ferramentas/soltar_wayland/no_app.sh
<nome> PH2D_BOARD_SMOKE=17`, handoff de integração, §5.1 do CLAUDE.md, binário `smoke` compilado 2×.
