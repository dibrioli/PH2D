# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P6f (a forma viva até ao fim) — 2026-10-27

> Leitor: a próxima LLM que assume a linha. A P6f está **feita**; o smoke do dono (cena 14) está
> **POR FAZER**. A **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada integrado, nada pushado.
> Lei: [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) §12 (emenda P6f)
> · anterior: [P6e](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6e.md) (smoke OK) · [P6d](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6d.md) (§6: os abertos que esta onda fecha).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (`line/PixelArt`).
- Merge-base `c3662cd8e4` (o `main` não andou; `git rebase main` = «atualizado»).
- Pedido: fechar num bloco os cinco abertos da forma viva da P6d §6.

## 2. O que a P6f fez

1. **Etiqueta do arrasto** = `ph2d_tool_vector::shape_handles_face::readout` (a do Vector), com o nome do campo como o painel o pinta: a ponte dos nomes dos campos subiu para `ph2d_editor_core::pixel_panel::{set_field_names, field_name}` (o `seam_formas::set_field_names` do painel injecta-a lá, como a dos nomes das formas na P6e). Comprimento: `pixel.shape.length` («{n} px», 1 px do Vector = 1 célula). Da alça agarrada; sem arrasto, da alça sob o cursor (o F12 do Vector). `viva::readout` (pública, testável) + `viva::ficha` pinta-a com a ficha da casa (`ph2d_editor_core::readout::{chip_width, at_cursor, paint_chip}`) dentro de `hero.last_content`. **Porquê no Pixel e não no `hero.gizmo.handle_readout`:** a fase do Vector (`shape_handles_live::draw`, em `fase_vector_tool_handles`) ESCREVE esse campo a cada quadro (`None` fora dele) e corre DEPOIS da `fase_pixel_panel` — publicar ali seria apagado. O `draw_of` passou a receber `(&mut VectorScene, &mut TextSystem)`; a shell (`fase_selection_highlight.rs`) mudou 2 linhas por 2 (catraca `the_shell_only_shrinks` verde). `ph2d-app-pixel/Cargo.toml` ganhou `ph2d-text`.
2. **Só contorno**: `Viva::outline_only()` (= agarra uma alça da forma); o `draw` pinta então `ph2d_editor_core::gizmo::paint_gizmo_outline` (1,5 px, a do `paint_canvas_overlays` do Vector) em vez do gizmo com alças. As alças da forma continuam.
3. **Barra da moldura**: `PixelPanelInfo.transform` = a da moldura solta OU `Viva::info()`; a conta é UMA — `solta::info_of` e `solta::com_campo` (extraídas de `Solta::info` / `campo_da_moldura`). `campo_da_moldura` sem solta → `viva::campo` (um passo). O painel (`paint_selecao::moldura`) não mostra a amostragem nem a dica de transformar com `info.live_shape` (a forma não reamostra). **Apply** = `E::Commit` assenta também a viva; **Cancel** = `E::Cancel` larga-a (`input::cancel`).
4. **Gravar**: `persist::to_bytes(&mut PixelApp, …)` assenta a forma viva e a moldura solta antes (um passo cada) — a shell (`project_save.rs`) passa `&mut self.pixel`. A moldura solta entrou por ser o mesmo defeito (o ficheiro levava o documento de antes dela).
5. **Ctrl+Z — decisão (medida no Vector):** `ph2d-app-vec/src/shape_handles_live.rs` (cabeçalho e `release`): *«o passo de undo é o do diff do quadro (o botão premido segura-o até soltar: um arrasto = um passo)»* — no Vector cada arrasto de alça, cada campo e a criação são passos, e a forma continua editável depois do Ctrl+Z. Lei no Pixel: **o Ctrl+Z com forma viva desfaz o último AJUSTE dela** (arrasto de alça ou da caixa, duplo-clique que repõe, campo da barra, campo/Fill do painel — pedidos seguidos do painel sobre OS MESMOS campos juntam-se num passo, máscara de bits em `Viva.painel`); **sem ajustes, larga-a** (o passo do nascimento); o documento não muda. `Ctrl+Shift+Z` refaz os ajustes e, sem forma viva, traz de volta a largada (`PixelApp.largada`: o gesto, o `undo_len` do documento nesse instante e a foto) só se o documento está nesse mesmo `undo_len`; o refazer do documento vem antes quando o Ctrl+Z já desceu mais. Qualquer `commit` esquece a largada. Código em `crates/ph2d-app-pixel/src/viva_ajustes.rs` (filho de `viva.rs`, partido pelo tecto de 700 LOC). Dica `pixel.hint.live_shape` ganhou «Ctrl+Z: undo the last adjustment». Diverge do Aseprite de propósito: ele não tem forma viva (o Ctrl+Z com células levantadas larga-as — `PixelApp::step`, P6), e a forma viva é a do Vector.

**Cena `PH2D_PIXEL_SMOKE=14`** (`crates/ph2d-app-pixel/src/smoke_ajustes.rs`, 128×80 RGBA): retângulo azul assentado; estrela amarela cheia VIVA girada (barra: `Angle 15`, W/H 60), nascida com 5 pontas e com UM ajuste guardado para 7 (`viva::ajuste`); a alça das pontas mostrada quente com a etiqueta (`Viva.mostra`, só da cena: o 1.º movimento do rato apaga-a). `PH2D_PIXEL_SMOKE_AGARRA=1` (**só para a foto**, que não manda cliques — XTest ignorado na Xwayland do `kwin --virtual`) deixa a alça agarrada: caixa só-contorno. `max_level` 14; a sonda de nível desconhecido (`tests/it/edit.rs`) passou ao 15. Fotos (1930×1040, placa 1,1 de 16 GB): `smoke_P6f_cena14_2026-10-27.png` e `…_arrasto_…` (fábrica), `…_dono_…` e `…_arrasto_dono_…` (`HOME=/home/enio`; `layout.txt` sha256 `827fb56f…` igual antes/depois).

## 3. Smoke do dono — OK (27/10)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=14 ./target/smoke/ph2d-host-desktop`
2. Estrela viva com a etiqueta «Points: 7»; arrastar uma pontinha → a caixa fica só com a linha e a etiqueta segue o rato.
3. No painel Pixel, Transform: mudar X/Angle → a estrela anda/gira na hora; Apply assenta, Cancel larga.
4. Ctrl+Z: 7 → 5 pontas; outro Ctrl+Z → some (o retângulo fica); Ctrl+Shift+Z ×2 → volta com 7.
5. Com a estrela viva, File ▸ Save, fechar e reabrir: a estrela está lá em pixels.

## 4. Superfície de colisão NOVA (soma-se à P6e §4)

- `ph2d-editor-core`: `pixel_panel/shapes.rs` (`CAMPOS`, `set_field_names`, `field_name`), `pixel_panel/mod.rs` (re-export).
- `ph2d-panel-pixel-tools`: `seam_formas.rs` (injecta a ponte no núcleo), `paint_selecao.rs` (`moldura` sem amostragem com forma viva).
- `ph2d-i18n`: `pixel_selecao.rs` (`pixel.shape.length`, dica).
- `ph2d-app-pixel`: `viva.rs`, `viva_ajustes.rs` NOVO, `solta.rs` (`info_of`, `com_campo`), `state.rs` (`largada`, limpa no `commit`), `input.rs` (Ctrl+Z/Ctrl+Shift+Z), `panel.rs`, `edicao.rs`, `persist.rs` (⚠️ `to_bytes` agora `&mut`), `compose.rs` (⚠️ `draw_of` recebe `(scene, text)`), `smoke.rs`, `smoke_ajustes.rs` NOVO, `lib.rs`, `Cargo.toml` (`ph2d-text`); testes `ajustes.rs` NOVO, `viva.rs` (helpers da `Mesa` `pub(crate)`), `edit.rs`, `main.rs`.
- Shell: `render_loop/fase_selection_highlight.rs`, `project_save.rs`, `project_pixel_tests.rs` (`&mut`).

## 5. Prova (por nome, verde)

- `ph2d-app-pixel` `ajustes` 7 NOVOS: `scene_14_shows_the_points_readout_the_frame_numbers_and_one_adjustment_to_undo`, `dragging_a_shape_handle_shows_its_readout_at_the_cursor_with_an_outline_only_box_as_one_step` (sem etiqueta longe; F12 sob o cursor; a meio, `outline_only` e a etiqueta no cursor com o valor do campo; largar = 1 passo), `the_frame_bar_shows_the_live_shape_and_a_field_moves_it_at_once_as_one_step` (pelo `panel::frame`: `transform == Viva::info()`, X+30 move o contorno 30 células, Ctrl+Z repõe, Apply assenta num passo), `saving_with_a_live_shape_settles_it_first_as_one_step_and_the_file_carries_it`, `ctrl_z_undoes_the_last_adjustment_then_drops_the_shape_and_ctrl_shift_z_brings_both_back` (incl. o documento desfeito/refeito no meio), `a_committed_step_after_dropping_the_shape_forgets_it`, `panel_edits_of_the_same_field_join_in_one_step_and_another_field_is_its_own`. E `viva` 8 (pan do meio incl.), `catalogo` 3, `formas` 8, `edit` 21, `selecao` 14, `desenho` 11, `quadros` 4.
- `ph2d-panel-pixel-tools` `seam_pixel` 26, `the_hint` 2; `ph2d-panel-registry-init` `the_pixel` 4; `ph2d-i18n` `key` 3; `ph2d-editor-core` `architecture` 118 (incl. `workspace_src_files_under_loc_cap` — `viva.rs` 623 + `viva_ajustes.rs` 240 —, `the_shell_only_shrinks`, `no_dependency_climbs_a_layer`); `ph2d-host-desktop` `pixel_blob`/`the_pixel_sprites` 3.
- `cargo fmt --all --check` limpo; `cargo-check-narrow ph2d-host-desktop` ✓. **Clippy NÃO corrido** (fecho da linha).

## 6. ABERTO

- ~~O smoke do dono da P6f (§3)~~ — OK 27/10.
- A etiqueta no Pixel não segue a `LengthDisplay` do projecto (o Pixel mede em células: «px» fixo pela chave i18n).
- O painel Pixel mostra «Shape» duas vezes (pincel e secção da forma) — herdado.
- Os abertos anteriores: [P6c §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6c.md) (Skew só pelo campo, flyout 3+1), [P6 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6.md), [P4 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md).
- **Mutações no fecho da LINHA** (somar às da P6b–P6e): o `agarra.is_some()` que congela o Ctrl+Z a meio de um arrasto; o `undo_len == n` da largada; o `self.largada = None` do `commit`; a máscara `v.painel == Some(m)` (trocá-la por `is_some()` junta campos diferentes); o `viva::assenta` no `to_bytes`; o `outline_only` no `draw`; o `.or(v.mostra)` da etiqueta; a ordem `refaz || stroking || redo` no Ctrl+Shift+Z.

## 7. Próximo item

Smoke OK (27/10). Ordem do dono 27/10: **P6g — as pontas soltas das formas e da selecção, num bloco**:
inclinar pelo gizmo (hoje só o campo Skew, P6c §6); o flyout 3+1 (widget partilhado — varrer por
consumidor); «Shape» duas vezes no painel; a unidade da etiqueta (células × `LengthDisplay`: decidir e
escrever); Load/Save MSK, Paste as New Sprite, Auto Select Layer do Move, Ctrl+Alt+I (Alt no teclado),
a amostra «camadas visíveis» da varinha (P6 §6); as lacunas medidas contra o Trial (elipse 6/112,
bilinear ±1, giro fora de 90°). Cena 15.
