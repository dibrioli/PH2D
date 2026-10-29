# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P6d (a forma VIVA com os gizmos do Vector) — 2026-10-27

> Leitor: a próxima LLM que assume a linha numa janela nova. A P6d está **feita**; o smoke do dono (cena 12)
> está **POR FAZER**. A **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada integrado, nada pushado.
> Lei do módulo: [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) (§12, emenda P6d)
> · [plano](../02_plano_modulo.md) (§5 P6) · anterior: [P6c](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6c.md) (§6 continua valendo) · [P6b](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6b.md) · [P6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6.md) · [P4](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (tem de dizer `line/PixelArt`).
- Merge-base `c3662cd8e4` (o `main` não andou). Commit da onda: `def03e014a` (feat P6d).
- Origem: o smoke da P6c foi APROVADO pelo dono em 27/10 com o pedido «os ajustes devem ser em tempo real no gizmo igual ao módulo vector» e «use os mesmos gizmos do vector».

## 2. O que a P6d fez

1. **A forma fica VIVA.** Largar o arrasto de Retângulo/Elipse/Polígono/Estrela NÃO grava. `Gesture.viva: Option<Viva>` (+ `Gesture.mods`, os modificadores do último movimento) em `crates/ph2d-app-pixel/src/state.rs`. Módulo novo `crates/ph2d-app-pixel/src/viva.rs`: `Viva` = caixa `w×h` levada por `ph2d_pixel_core::affine::Affine` + pivô (a mesma conta da moldura solta); `geo()` dá a meia-caixa local de centro a centro, `xf` = local (Y para cima, escala 1, giro e inclinação da caixa) → imagem. Sem giro assenta pela caixa inteira via `formas::cells` (bit-idêntico ao gesto antigo); com giro por `formas::cells_mapped` (novo, com `formas::cook_local`; `achata` ganhou um `map`). Esticar a caixa muda w/h da forma, nunca os raios.
2. **Alças = lei do Vector**, só funções puras de `ph2d_tool_vector::shape_handles` (`handles_of`, `handle_grip`, `handle_point`, `HandleDrag::start`, `drag`, `reset`, `track`, `reads_only_the_angle`, `is_arc_angle`) e `shape_handles_face` (`glyph_of`, `radius_ring`); pintura `ph2d_vec_render::draw_shape_handles`; régua `DocPx::new(1.0)` (1 px do Vector = 1 célula); comprimentos (`FieldUnit::Px`) arredondados a células inteiras. Mods: Shift = passos de 15°, Ctrl = pares, Alt = um canto; duplo-clique numa alça repõe o campo (janela `DOUBLE_CLICK_WINDOW_NS`). A caixa: `solta::gizmo_of`/`hit_of`/`drag_opts`/`imagem` (extraídos de `Solta::gizmo`/`hit`, partilhados com a moldura solta). O glue press/drag/draw foi refeito no Pixel (o de `ph2d-app-vec::shape_handles_live` é da família Vector, não importável).
3. **Alça e campo = duas mãos sobre o mesmo número.** Os valores moram em `Options::shapes`; `PixelRequest::Options` chama `PixelApp::reaviva()` (re-cozinha na hora). Fill do painel idem.
4. **Assentar (1 passo do desfazer):** `Enter`, trocar de ferramenta (`set_tool`), clique fora das alças/caixa (assenta e o premir segue a desenhar a próxima), qualquer pedido com `edicao::assenta_antes` (`viva::assenta` em `panel.rs aplica`), sair do Draw (`frame.rs` e `mode.rs leave`). `Esc` a meio de um arrasto repõe caixa/campos do premir; `Esc` sem arrasto larga a forma (nenhum passo). Clique sem arrasto (from==to) grava como antes. `frame.rs`: a forma viva da cena de smoke montada antes do 1.º Draw sobrevive (como a moldura solta da cena 10).
5. **Fora do Pixel (Modo L, append):** `HIT_PX`/`PARK_PX` subiram de `ph2d-app-vec/src/shape_handles_live.rs` para `ph2d_tool_vector::shape_handles` (app-vec re-exporta com `pub use`); `painted(Glyph)->HandleFace` e `RING_N` subiram para `ph2d_tool_vector::shape_handles_face` (app-vec usa-os; o `face_of` local saiu); `ph2d-tool-vector/Cargo.toml`: `ph2d-vec-render` passou de dev-dep a dep normal; `ph2d-app-pixel/Cargo.toml` ganhou `ph2d-tool-vector` e `ph2d-vec-render` (família→ferramenta e →folha, permitidos por `architecture_no_dependency_climbs_a_layer` e `architecture_cycle_prevention`, ambos verdes). `PixelPanelInfo.live_shape: bool` (editor-core `pixel_panel/mod.rs`); dica `pixel.hint.live_shape` (`ph2d-i18n/src/pixel_selecao.rs`), escolhida primeiro em `ph2d-panel-pixel-tools/src/column.rs hint`. `gesture::boxed` agora devolve a caixa normalizada ↖↘ e é `pub(crate)`.

**Cena `PH2D_PIXEL_SMOKE=12`** (`crates/ph2d-app-pixel/src/smoke_viva.rs`, sprite 128×80 RGBA): retângulo de canto 6 e elipse cheia já assentados à esquerda; à direita uma Estrela cheia VIVA girada 18° com a caixa e as alças; Estrela na mão. `max_level` 12; a sonda de nível desconhecido (`tests/it/edit.rs`) passou ao 13. Fotos `smoke_P6d_cena12_2026-10-27.png` (fábrica) e `smoke_P6d_cena12_dono_2026-10-27.png` (`HOME=/home/enio`; `layout.txt` sha256 `ee9932ad…` igual antes/depois; placa livre: 0,7–1,6 de 16 GB). A geometria da foto foi conferida (pontas no giro de 18°).

## 3. Smoke do dono — POR FAZER (a aguardar)

Texto para o Enio (curto):

1. No terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=12 ./target/smoke/ph2d-host-desktop` (binário já compilado: a 2.ª `cargo build -p ph2d-host-desktop --profile smoke` deu `Finished … in 0.23s`, zero `Compiling`).
2. Abre o sprite com a estrela à direita, ainda inclinada, com uma caixa e pontinhas à volta. A ferramenta Estrela já está na mão.
3. Arraste a ponta de dentro da estrela: a estrela muda na hora. Mude um número no painel «Pixel»: a estrela muda na hora também. Arraste um canto da caixa para esticar e o canto de fora para girar. Segure Shift para girar de 15 em 15 graus.
4. Carregue Enter: a estrela fica pintada. Ctrl+Z desfaz a estrela toda de uma vez. Desenhe uma forma nova com o rato: ao largar ela NÃO fica pintada, fica com as pontinhas. Clicar fora ou trocar de ferramenta também a pinta. Esc larga a forma sem pintar.
5. Deu errado se: largar o rato pinta logo; mexer num número do painel não mexe na forma; as pontinhas não aparecem; Enter não pinta; Ctrl+Z desfaz só um pedaço.

## 4. Superfície de colisão NOVA (soma-se à [P6c §4](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6c.md))

- `ph2d-app-pixel`: `viva.rs`, `smoke_viva.rs`, `tests/it/viva.rs` **NOVOS**; `state.rs`, `input.rs`, `gesture.rs`, `formas.rs`, `solta.rs`, `draw.rs`, `panel.rs`, `frame.rs`, `mode.rs`, `smoke.rs`, `lib.rs`, `Cargo.toml`.
- `ph2d-tool-vector`: `shape_handles.rs`, `shape_handles_face.rs`, `Cargo.toml`. `ph2d-app-vec`: `shape_handles_live.rs`.
- `ph2d-editor-core`: `pixel_panel/mod.rs`. `ph2d-i18n`: `pixel_selecao.rs`. `ph2d-panel-pixel-tools`: `column.rs`, `tests/it/seam_pixel_formas.rs`.
- ⚠️ Um merge pode partir: quem constrói `PixelPanelInfo` sem `..Default` (campo novo `live_shape`). Quem usa `ph2d_app_vec::shape_handles_live::{HIT_PX,PARK_PX}` continua a funcionar (re-export). Quem espera que largar uma forma a grave (testes antigos): ajustados com Enter (`desenho.rs` `drag` e `a_rectangle_previews…`; `formas.rs` `the_fill_box_fills…`).

## 5. Prova (por nome, verde)

- `ph2d-app-pixel` `viva` 7: `releasing_a_shape_leaves_it_live_and_enter_settles_it_as_one_step`, `a_panel_field_recooks_the_live_shape_at_once`, `the_corner_handle_rounds_the_rectangle_by_the_vector_law_and_a_double_click_resets_it`, `esc_mid_drag_puts_the_field_back_and_esc_again_drops_the_shape`, `the_box_handle_grows_the_shape_without_stretching_its_corner` (célula a célula contra `formas::cells` na caixa nova), `another_tool_or_a_click_outside_settles_the_live_shape`, `scene_12_shows_a_live_rotated_star_with_its_handles`; `desenho::` 11, `formas::` 8, `edit::` 16, `selecao::` 14, `camadas::` 5, `quadros::` 4.
- `ph2d-tool-vector` `shape_handles` 40; `ph2d-app-vec` `shape_handles` 22; `ph2d-panel-pixel-tools` `the_hint` 2 (novo `the_hint_teaches_the_handles_while_a_shape_is_live`), `seam_pixel` 24; `ph2d-panel-registry-init` `the_pixel` 3; `ph2d-app-macros` `cenas_do_smoke` 2.
- `ph2d-editor-core`: 38 gates de arquitetura (`no_dependency_climbs`, `cycle_prevention`, `workspace_file_loc_cap`, `the_shell_only_shrinks`, `claude_md`, `tool_contract_surface`, `docs_paths_and_smokes_resolve`, …).
- `cargo fmt --all --check` limpo; `cargo-check-narrow ph2d-host-desktop` ✓. **Clippy NÃO corrido** (só no fecho da linha).

## 6. ABERTO

- **O smoke do dono da P6d (§3).**
- A ETIQUETA do arrasto do Vector («Points: 7», `face::readout`) não aparece no Pixel (precisa do `hero` e dos nomes do painel).
- Durante o arrasto de uma alça a caixa não passa a «só contorno» como no Vector (`outline_only`).
- A barra da moldura (X/Y/W/H/ângulo) não mostra a forma viva.
- Gravar o projecto com uma forma viva não a assenta (fica de fora, como um polígono a meio).
- Ctrl+Z com forma viva não faz nada.
- Os abertos do [P6c §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6c.md): Skew só pelo campo, flyout 3+1, [P6 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6.md), [P4 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md).
- **Mutações no fecho da LINHA** (somar às da P6b/P6c): `viva::nasce` (largar não grava), `reaviva` no `Options`, `escape` repõe `at_press`, o arredondar `Px`, `assenta` em `assenta_antes`/sair do Draw, `frame.rs` guarda a viva da cena.

## 7. Próximo item

O smoke do dono da cena 12; depois o que ele pedir. Fotos de fábrica e com `HOME=/home/enio` (copiar antes `~/.ph2d/layout.txt`; `nvidia-smi` antes).

## Emenda (pré-smoke, report do dono): o pan do botão do meio

`0b6bf9c529` — com uma forma viva na tela o pan do botão do meio (e o da mão/Espaço) morria:
`viva::moved` consumia todo movimento e `viva::release` todo largar, e a shell nunca chegava ao
`pan_anchor`. Agora seguem o idioma da `solta`: só um arrasto consome o mover; o largar só quando o
premir foi da forma (`g.held`, marcado em `input::press`). Teste
`viva::with_a_live_shape_a_move_or_release_without_a_drag_goes_on_to_the_middle_button_pan`.

## Veredito do smoke (dono, 27/10): **OK**

Cena 12 aprovada (com o pan do meio corrigido em `0b6bf9c529`). Pedido seguinte do dono, palavra por
palavra: *«Traga todas as outras shapes presentes em vector para pixel mantendo os mesmos gizmos
maravilhosos»* — é a onda **P6e**: o `PixelShape` (hoje 4: Rectangle→`RoundRect`, Ellipse, Polygon,
Star) passa a cobrir TODO o catálogo `ph2d_vec_scene::ShapeKind` que o Vector oferece ao artista
(espiral, linha, arco, pizza, segmento, setas, fluxograma, balões, símbolos…), cada uma VIVA com a
caixa do gizmo do sprite e as alças da lei `ph2d_tool_vector::shape_handles` — o mesmo caminho da
P6d (`viva.rs`, `formas::cook_local`/`cells_mapped`), sem cópia por forma.
