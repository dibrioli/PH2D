# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P6i (os defeitos do smoke da cena 16) — 2026-11-01

> Leitor: a próxima LLM que assume a linha. A P6i está **feita e aprovada no smoke do dono (01/11)**.
> A **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada integrado, nada pushado.
> Lei: [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md)
> · anterior: [P6h](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-30_P6h.md) (o §3 tem os defeitos D1–D3; o §6 o aberto que continua aqui).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (`line/PixelArt`).
- Sobre o main `422fec2027`; P0–P6g no main; P6h e P6i só na linha.
- Commits P6i (`git log --oneline b621fc4c2d..HEAD`): `a688bf172c` D1–D3 · `1a835345c7` o nome da tag depois da pega (observação do smoke) · + este handoff.

## 2. O que a P6i fez

1. **D1 — Enter num campo dava Play.** Mecanismo: em `keyboard.rs:134` a shell pergunta à família (`pixel_owns_the_keyboard`) ANTES de entregar a tecla ao widget focado (`forward_key_to_hero`), e a guarda (`surface_owns_the_keyboard`) só cedia com aba de documento activa — que já exclui o campo focado. Toda tecla do Draw (Enter, Delete, Backspace, Esc, Ctrl+Z) roubava o campo. Cura na porta da família: `compose::View.typing` (lido em `View::of` de `HeroScreen::text_entry_focused`) e `PixelApp::keyboard` cede a tecla PREMIDA (largar passa: o Espaço do pan). Teste `intervalo::enter_in_a_focused_field_is_the_fields_and_does_not_play` (`shells/pixel-scenes`), com controlo sem foco; vermelho sem a guarda (`(true, true)`).
2. **D2 — o bloco VÊ-SE a mover.** A lei do voo do cartão do Kanban saiu para `ph2d_editor_core::paint_flight` (`shadow`, `flight_shadow`, `fly`, `FLIGHT_TILT_DEG` 4°, pivô 60 %, sombras); `ph2d-app-kanban` re-exporta e chama-a (`metrics.rs`, `paint.rs`, `paint_card.rs`). A ordem de `frames::move_block` saiu para `ph2d_pixel_core::frames::block_order` (genérica); `gestos::bloco_previsto` aplica-a ao `PixelPanelInfo` e a grelha pinta a ordem PREVISTA a meio do arrasto: o vão (`Bg2`) com a moldura onde o bloco cai, e o bloco voa com o rato (`paint_voo.rs`: números, durações e cels originais, sombra, inclinado). A linha de inserção saiu. `Arrasto::Bloco { alvo, de, x }` (perdeu o `Eq`). Teste `seam::mid_drag_the_grid_paints_the_block_where_it_will_land` (gestos empurrados no canal: nenhum `RangeMove` a meio; o cabeçalho do escolhido já na coluna onde cai); vermelho sem a ordem prevista.
3. **D3 — a ponta da tag.** Pega sempre pintada em cada ponta visível (`Text3`, `StrokeToken::Thick`), acesa (`Accent`, `Spacing::Xs`) sob o rato (`hot_id`) ou puxada; a barra puxada leva a moldura `Accent`; o nome recua `Spacing::Sm + Xxs` (smoke 01/11: «walk» caía na pega).
4. **Cursores.** `PointerLook::{Grab, Grabbing, ResizeEw}` (`doc_surface.rs`); `WidgetStore::pixel_timeline_look(hover)` — a captura (`active_id`) manda, senão o que está sob o rato: ponta → ↔, moldura → mão aberta / fechada a agarrar; pedida no topo de `board_view::pointer_look`; a shell só traduz (+3 linhas em `despacho_metodos_janela_e_vetor.rs`, catraca verde). Teste `pixel_timeline_surface::tests::the_tag_end_and_the_range_outline_ask_for_their_pointer`. ⚠️ O cursor não sai nas fotos (X11 `import`); só o teste e o smoke do dono o provam (aprovado 01/11).
5. **Cena 16 conduzida.** `PH2D_PIXEL_SMOKE_GESTO=bloco|ponta` (`smoke_intervalo::conduz`, chamado em `panel::frame`): premir no meio da moldura (ou da ponta direita da «walk») e andar 1,1 × a largura do bloco SEM largar, pelo `dispatch_pointer` do produto — o XTest não chega à Xwayland da tela virtual. Deps novas da `ph2d-app-pixel`: `ph2d-panel-pixel-timeline` (os ids; família → painel é permitido) e `bumpalo`. Fotos `smoke_P6i_cena16_{repouso,bloco,ponta}_{fabrica,dono}_2026-10-31.png` (layout.txt do dono igual antes/depois em cada corrida).

## 3. Superfície de colisão NOVA (soma-se à P6h §4)

- `ph2d-editor-core`: `paint_flight.rs` NOVO (+ `lib.rs`), `doc_surface.rs` (`PointerLook` +3 variantes — quem faz `match` exaustivo tem de as tratar; hoje só a shell), `interaction/pixel_timeline_surface.rs`, `screens/hero/board_view.rs`.
- `ph2d-app-kanban`: `metrics.rs` (consts de voo re-exportadas; `FLIGHT_PIVOT_Y`/`SHADOW_FLIGHT_LIGHT` saíram dela), `paint.rs` (`shadow` re-exportada, `fly`), `paint_card.rs` (`flight_shadow` re-exportada). **Conflito provável com a linha TrelloClone se reabrir.**
- `ph2d-pixel-core`: `frames.rs` (`block_order`). `ph2d-panel-pixel-timeline`: `paint_voo.rs` NOVO, `gestos.rs`, `paint_grid.rs`, `lib.rs`, `Cargo.toml` (+`ph2d-pixel-core`, dev `ph2d-host`).
- `ph2d-app-pixel`: `compose.rs`, `panel.rs`, `smoke_intervalo.rs`, `Cargo.toml`, `tests/it/{edit,fecho}.rs` (`typing: false`). **Folga na catraca quase nula** (tecto 13 424; `the_shell_only_shrinks` verde a 01/11).
- Shell: `input_dispatch/despacho_pixel.rs` (doc), `despacho_metodos_janela_e_vetor.rs` (+3).

## 4. Prova (por nome, verde)

- `ph2d-host-pixel-scenes`: `intervalo` 12, `sobreviventes` 11. `ph2d-pixel-core`: `frames` 9, `the_pixel_core_declares_no_dependency`.
- `ph2d-panel-pixel-timeline`: `seam` 11, `every_word` 3.
- `ph2d-editor-core`: `pixel_timeline_surface` 3, `the_shell_only_shrinks` 2, `architecture_no_dependency_climbs_a_layer` 3, `architecture_no_orphan_source_file` 4, `architecture_every_member_inherits_the_workspace_lints` 3.
- `ph2d-app-kanban`: `voo` 4, `arrastar` 4. `ph2d-app-pixel`: `fecho` 4, `edit` 21. `ph2d-host-desktop`: `pixel` 26, `the_key_blocks` 4.
- Mutações à mão: a guarda `typing && pressed` e a ordem prevista sangram os testes novos. **Clippy NÃO corrido** (fecho da linha); `cargo machete` por correr (deps novas).

## 5. ABERTO

- **Mutações por correr**: P6b e P6c (nunca corridas); P6h (`acrescenta` sem o `contains`; `fronteira` round→floor; ordem Esc intervalo × Deselect; `range_span` contíguo; `alvo_quadros` sem intervalo; `TagRange` clamp; LayerContinuous pela história; `block_order` `at`); P6i (a captura em `pixel_timeline_look`; o recuo do nome da tag; o `!row.group` do voo).
- **Sobreviventes**: #8 paleta do `born_from_paste` fora do Indexed (CORRER no Trial); #10 `FAST_TIE_LEFT`; #12/#13 `outline_only` e `.or(v.mostra)` no `viva::draw`; #18 `Px` com floor; metade `mode.rs` do #20.
- **Trial não medido** (`trial_timeline.py`): Ctrl ao largar o bloco; mover só cels; Shift+clique em cel depois de arrasto; Esc com selecção E intervalo; Reverse com quadros soltos; undo da tecla Delete.
- `a_altura_de_abertura_de_um_painel_so_encolhe` reprova com vizinhos (P6 §2a).
- O §10 do [handoff de integração 28/10](HANDOFF_INTEGRACAO_line_PixelArt_2026-10-28.md): RotSprite, empates do Fast, elipse > 150×96, tecto do Modify, LengthDisplay, paletas, P5, P7–P10.
- Fecho da linha: `/pd-linha-fechar`, `cargo machete`, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`.

## 6. Próximo item

O §5 num bloco (CLAUDE §0.10): mutações + sobreviventes + as medições do Trial numa rodada.
