# 12 — Os sub-blocos dos cartões compostos viram GRUPOS dobráveis (plano, 24/10/2026)

> Decisão do dono 24/10, pergunta (c) de 22/10: «busque o padrão-ouro do mercado». **Por fazer** — este doc é
> o desenho e a receita; a implementação é a próxima janela.

## 1. O oráculo (Godot 4.7.2, MIT — corrido)

[`ferramentas/oraculo_cartao_composto/`](ferramentas/oraculo_cartao_composto/corre.sh): o editor numa Xvfb com
Camera2D e Parallax2D seleccionados (fotos `foto_01..03`, estrutura em `saida.txt`).

| O Godot faz | O PH2D hoje (`ph2d_hoje_camera.png`, `ph2d_hoje_paralaxe.png`) |
|---|---|
| As propriedades PRINCIPAIS do nó vêm sem cabeçalho (Offset, Zoom …; Scroll Scale/Offset) | igual: o corpo do `GameCamera` e o fator da paralaxe sem subtítulo ✓ |
| Cada parte extra é um **grupo**: rótulo em **negrito** + chevron (›/⌄), clique abre/fecha | «Camera Follow», «Camera Limits», «Parallax Repeat» = texto `Sm` `Text2` (`sections/rows.rs::subtitulo`), sem chevron, sem clique |
| O conteúdo do grupo fica **recuado** (~12 px) | encostado à mesma coluna do resto; o subtítulo encosta à borda do cartão |
| A dobra é lembrada por objecto; a altura do cabeçalho do grupo (23 px) < a da linha | — |
| Uma parte que se liga/desliga leva o interruptor NA linha do grupo («Limit ☑ On», «Position Smoothing ☐ On») | as nossas partes existem pela PRESENÇA do componente — não há interruptor a pôr ali (fora do âmbito) |

O Blender (sub-painéis) e o Unreal (sub-categorias do Details) convergem no mesmo idioma: cabeçalho dobrável,
conteúdo recuado. ⇒ **a lei**: o `subtitulo` passa a ser um cabeçalho de GRUPO dobrável (negrito, chevron, recuo
de `list_indent_px()` = 12 px no conteúdo), aberto por omissão (a parte só existe se o artista a pôs), dobra
lembrada como a dos cartões. O nome do grupo continua a ser o do componente — é o prefixo do script
(`camera_follow.damping`).

## 2. A receita (explorador 24/10, por confirmar ao implementar)

- Os 8 chamadores de `subtitulo` (`sections/rows.rs:401`): `camera.rs` (follow, limits), `parallax.rs` (repeat,
  drift, limits), `hud_corpo.rs` (canvas, rótulo, botão, contador).
- Cabeçalho + dobra que se reusa: `RowCtx::section_header` (`editor-core/src/panel/rows.rs:119`, devolve
  `(y do corpo, fechada)`), `paint_section_header` (`widget/section_header/mod.rs:132`), `SectionFold::begin`
  (`widget/section_header/body.rs:83`), `close_subsection` (`widget/section_cards/mod.rs`, ao lado do `close_section`).
- Estado: `WidgetStore.collapsed`/`fold_live` por `NodeId` (`interaction/state/mod.rs:323,332`); clique →
  `apply_click` (`interaction/dispatch/focus.rs`) → `toggle_collapsed` (`collapse_ops.rs:72`) se o id estiver
  marcado `mark_collapsible_section` (`store_core.rs:365`). Ids por `hash_node_id`, sem variante de evento nova.
- **O precedente CERTO é o `INSP_VIS_LAYER_HEADER`** (sub-cabeçalho dentro de um cartão, marcado em
  `screens/hero/pre_populate.rs:667`). ⚠️ O «Cull Mask» da câmera (`sections/camera.rs:360-400`) regista o hit com
  `hit_index.register` simples e NÃO está marcado — o clique nele pode estar MORTO (o teste abre-o por
  `set_collapsed`). Medir primeiro; se morto, curar no mesmo bloco (§0.10).
- Portões: `every_painted_id_is_reachable.rs:256` (id novo sem marca reprova), `hr12_widgets_a11y.rs`,
  `the_camera_section_is_alive.rs`. Tecto de LOC dos ficheiros de secção.

## 3. Smoke previsto

`PH2D_GAME_CAMERA_SMOKE=1` (Camera → Game Camera) e `PH2D_PARALLAX_SMOKE=1` (Arvores → Parallax): os grupos
abrem e fecham, o conteúdo recuado, a dobra lembrada ao trocar de objecto e voltar. Fotografar com
`ferramentas/olha_cena.sh` antes de mandar.
