# W4 — o mesmo Kanban no canvas do Quadro (plano do coordenador, 28/10)

## Desenho (ADR-0195 §3)
- `ElementKind::Kanban { kanban: KanbanId, view: KanbanBoardView }` — variante no FIM (postcard
  posicional). `ph2d-board-model → ph2d-kanban-model` é acíclico. `KanbanBoardView` = só estado de
  VISTA (rolagem interna, zoom interno, listas recolhidas desta vista) — nunca dados do documento.
- `PROTOCOL` 11 → 12 (`ph2d-board-sync/src/session.rs:23`). `FORMAT_VERSION` do Quadro: conferir se o
  Embed subiu degrau; se a escada exige, subir com degrau de leitura.
- O Quadro não pode depender da família Kanban (`board-render`/`editor-core` ← ciclo). Pintura e
  ponteiro do elemento passam por um GANCHO: `trait LiveElement` (ou equivalente) registado pela
  `ph2d-app-board`/shell, molde = o vídeo (`ph2d-board-render/src/video.rs:107`, `Placement{to_screen}`,
  `Videos.active` para entregar rato/teclas). Sem gancho: LOD baixo (blocos de cor por lista + contagem),
  pintado pelo próprio board-render a partir do `KanbanSet` (board-render pode ler `ph2d-kanban-model`).
- LOD: zoom alto (px por unidade de lista ≥ limiar MEDIDO) = Kanban inteiro interactivo (mesmo código
  da aba numa `Affine`); médio = títulos + cartões só com título/capa; baixo = blocos + contagem.
- A família Kanban expõe `paint_view(scene, to_screen: Affine, rect, &mut Tab, &KanbanSet, lod)` e
  `pointer_view(local, …)` — a aba passa a ser UM chamador (escala 1), o elemento outro.
- Gestos: «Open in tab» na barra do elemento + duplo clique no cabeçalho; na aba «Show on board»
  (escolha se vários) e «Place on board»; entrada «Kanban» na barra de ferramentas/inserção do Quadro
  (não há menu Add: usa-se o `TOOLBAR` de `board_bar.rs:184` + menu de botão direito
  `menu_tables_board.rs`); apagar a referência não apaga o documento; exportação Excalidraw/PDF =
  imagem (LOD médio rasterizado).
- Smoke: Quadro cena 30 (`ph2d-app-board/src/smoke.rs`) e Kanban cena seguinte (contar).

## Lotes
| Lote | Possui | Quando |
|---|---|---|
| M | `ph2d-board-model`, `-geom`, `-render`, `-edit`, `-excalidraw`, `-sync`; `ph2d-editor-core/src/screens/hero/board_*`, `menu_tables_board.rs`, ids/i18n do Quadro; `ph2d-app-board/src/io.rs` | já (∥ W3 A/C) |
| V | `ph2d-app-kanban`: separar pintura/ponteiro da aba numa vista com `Affine` + LOD; implementar o gancho; «Show/Place on board»; ligação em `ph2d-app-board` | depois do lote D da W3 |
