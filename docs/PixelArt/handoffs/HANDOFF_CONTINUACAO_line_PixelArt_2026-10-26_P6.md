# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P6 (Seleção e transformação) + gates do P4 — 2026-10-26

> Leitor: a próxima LLM que assume a linha numa janela nova. A onda está **fechada e o smoke do dono
> foi «Restante Smoke OK» com QUATRO correcções de desenho (§3) — elas são a próxima onda, P6b**. A
> **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada integrado, nada pushado. Lei do módulo:
> [ADR-0192](../../architecture/decisions/0192-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) (**§12 = esta onda**)
> · [plano](../02_plano_modulo.md) (§5 P6 com a nota do que entrou) · anteriores: [P4](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md) · [P3](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P3.md) · [P2](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P2.md) · [P0](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (tem de dizer `line/PixelArt`).
- Merge-base `c3662cd8e4` (o `main` não andou: `git rev-list --count HEAD..main` = 0). **81 commits** na linha; os desta janela são `3edee7bbe0..HEAD` (15 + este handoff), 112 ficheiros, +12300/−1163.

## 2. O que esta onda fez

**(a) Gates do P4 curados** (`3edee7bbe0`, `bcb7ed2be5`; cura por mover/cortar, nenhum tecto subiu): `pixel_panel.rs` (807 > 700) virou a pasta `editor-core/src/pixel_panel/{mod,timeline,layers,new_sprite,selection}.rs`; o `paint.rs` do painel PIXEL deu a paleta a `paint_paleta.rs`; uma só fila de botões que só emparelha quando os dois rótulos cabem (14 cortes do degrau estreito saíram); Replace Color: botão «Replace» + frase que quebra; `set_color_mode_*` benignos; `safe_clamp_f64`/`CLAMP-OK`; `LITERAL-PX-OK` com porquê na matemática de cor; o `hash3` do pixel-core era outra lei → `spray_seed`; `ph2d-pixel-tools` em `DIVIDA_CRATE NaoAnimavel` no gate campo-canal; `widget_lab` saiu das isenções de rótulo; «Pixel-perfect» → «Perfect» por decisão do dono (74 px para ~78 no degrau estreito). Fica vermelho só `a_altura_de_abertura_de_um_painel_so_encolhe` (inspector 956/831, painter_layers 1620/1570): passa sozinho e reprova com os vizinhos do módulo — já medido na P0, não é desta linha.

**(b) Passo 1 — MVP** (`faf39b23ba`): `docs/PixelArt/ferramentas/extrai_selecao_mvp.mjs` → `crates/ph2d-pixel-core/tests/fixtures/mvp_selection.json` (380 casos: mask_seq, morph, perm, rotated_size, rotate, xform, affine, offset_wrap).

**(c) Passo 2 — Trial observado** (`2f318ddfa5`): `trial_ase_selecao.py` / `trial_selecao.py` / `trial_fixture_selecao.py` → `oracle_trial_selection.json` (367 casos). Leis: marquee inclusivo nas duas pontas, clique sem arrasto desmarca; elíptico = Zingl `plotEllipseRect` (106/112); laço = par-ímpar nos centros ∪ Bresenham de cada aresta; varinha `max|dR,dG,dB,dA| ≤ tol` (A=0 casa todo A=0), contígua 4/8; Color Range = varinha global; modos Shift / Alt+Shift / Ctrl+Shift (Alt só = conta-gotas); Border = faixa INTERIOR n (o MVP, grow−shrink, acerta 0/60); pincel círculo = máscara elíptica (2n+1)²; Flip/Shift dentro da caixa da selecção; Rotate 90 da selecção centrado com floor; Transform positivo = anti-horário, escala `floor((d+0.5)·S/N)`; Sprite Size nearest `floor(d·S/N)`, bilinear com cantos alinhados ±1; Canvas Size centro `trunc(dw/2)`/`dh−trunc(dh/2)`; Trim tira borda transparente E opaca uniforme; Paste na posição copiada. Peculiaridades do Trial (largar o último mousemove antes do release; máscara lida pelo Edit▸Fill magenta) estão no docstring de `trial_selecao.py`.

**(d) Núcleo `ph2d-pixel-core`** (`623f7a5f06`, `4985ec94f2`, `961b080e4d`): `selection` (Mask, SelectMode, Modify/ModifyBrush, Wand, `wand/color_range`; passos select/deselect/reselect/select_all/invert/modify → `Option<Op>`), `transform` (Image, flips, rot90/180, offset_wrap, resize_nearest/_centred/_bilinear, epx2/epx8, Floating, lift, stamp), `affine` (Affine com drag/fields/with_fields, `Resample` Fast|RotSprite, transform_image), `edit` (flip/rotate/shift/transform/copy/copy_merged/paste/commit), `canvas` (rotate/flip canvas, sprite_size, canvas_size com Anchor, crop, trim). `Sprite.mask`/`last_mask` NÃO são gravados (formato inalterado; gate do round-trip). `Op::Mask`, `Op::Canvas`, `Op::Batch`; `OpError::BadMask`/`BadSize`. Pintar recortado num ponto só: `Stroke::accepts`.
Exacto contra o Trial: rect 10, laço 12, menu 6, modify 90, varinha 45, Color Range 7, modos 7, Flip/Rotate/Shift/Canvas 23, escala+90° 8, Sprite Size nearest 4 + RotSprite@2× 1, Canvas Size 8, Crop 2, Trim 3, Paste 4. **Lacunas medidas:** elipse 106/112 (4x6 5x6 6x5 10x6 10x8 24x16 listadas no gate); bilinear 8392/8463 canais (±1); giro fora de 90° = algoritmos do MVP (Fast = nearest, RotSprite = EPX×8 amostrado no centro) a 10–29 células do Trial (teste ignorado `mede_rotacao`), sem tabelas.

**(e) App** (`0f3453c0f2`, `e23a372cab`, `9c79b2a1b1`, `17fca7ef2b`, `9f6ff3a428`, `f0390b0ab2`): ferramentas M, Shift+M, Q, Shift+Q, W, Move V (Ctrl arrasta); os 4 modos no painel (duas filas sob «Selection mode»); varinha tolerância/contígua/4-8; menu Select (Ctrl+A, Ctrl+D/Esc, Ctrl+Shift+D, Ctrl+Shift+I, Color Range…, Expand/Contract/Border…); Edit (Shift+H/V, Rotate 180/90, Shift ×4 só botões, Delete, Ctrl+C/X, Ctrl+Shift+C, Ctrl+V flutuante); Transform Ctrl+T com moldura PRÓPRIA do overlay (8 alças, giro fora do canto, Ctrl numa alça lateral inclina, pivô, campos X/Y/W/H/Angle/Skew, Fast/RotSprite; Enter/Apply/clique fora = UM passo; Esc ou Ctrl+Z larga); Sprite (Canvas Size C, Sprite Size só por botão — Ctrl+Alt+I não chega: o caminho do teclado só passa Ctrl e Shift —, Canvas 180/90, Flip Canvas, Crop, Trim); formigas em `ph2d-pixel-render/overlay.rs`; o `selected_all` da P3 saiu (Replace Color, Layer via Copy/Cut, Copy usam a máscara); `PixelLayerAction::{Copy, SelectAll, Deselect}` removidos. **Cena `PH2D_PIXEL_SMOKE=9`** (`smoke_selecao.rs`; sonda de nível desconhecido passa ao 10).

## 3. Smoke do dono (26/10) — «Restante Smoke OK», com QUATRO correcções

Fotos: `smoke_P6_cena9_2026-10-26.png` (fábrica) e `smoke_P6_cena9_dono_2026-10-26.png` (`HOME=/home/enio`; `layout.txt` sha256 `5c62c7d1…` igual antes/depois). Verbatim do dono, depois a leitura técnica:

1. «o ícone Magic Hand ficou errado» → o ícone da Magic Wand no painel está errado: achar o ícone certo da casa (`IconId`).
2. «As 4 seleções podem ficar agregadas/sobrepostas um único botão do painel com uma setinha na lateral inferior para indicar que há opções a mais no mesmo botão» → grupo de ferramentas com flyout (Photoshop/Aseprite): um botão mostra a última usada + triângulo no canto inferior. Procurar se a casa já tem um widget de botão com sub-opções antes de criar.
3. «O gizmo de transformar deveria ser o mesmo da sprite que já existe no app, completo com rot. Além do atalho deveríamos ter a Tool de transformar com ícone no painel.» → reusar o gizmo de transformar do objecto Sprite no viewport (com rotação) em vez da moldura do `overlay.rs`; ferramenta Transform com ícone na coluna, além do Ctrl+T. Investigar onde mora (ph2d-editor-core / app de sprites) e se aceita alvo em células inteiras.
4. «Os polígonos deveriam ser os mesmos do módulo vector, deveriam ter opções específicas no painel à direita como fillet.» → as formas do Pixel (retângulo, elipse, polígono…) usam as mesmas do Vector (QuickShape/primitivas `ph2d-vec-*`: N lados, estrela, raio do canto/fillet) com as opções no painel Pixel, rasterizadas pelas leis inteiras do pixel-core. Medir o que o Vector expõe antes de desenhar ([ADR-0108](../../architecture/decisions/0108-vector-reposition-rive-referenced-native-editor-first.md)).

## 4. Superfície de colisão NOVA

Soma-se à [P4 §4](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md). Sem `Cargo.toml`/`Cargo.lock`/shell novos; contadores ECS/schema/`MODULE_TRUTHS` inalterados (blob PH2DPIXEL 1.2 intacto).
- `ph2d-pixel-core`: `selection.rs`, `transform.rs`, `affine.rs`, `edit.rs`, `canvas.rs`, `op.rs` (Mask/Canvas/Batch), `sprite.rs` (mask), `stroke.rs` (accepts/with_values).
- `ph2d-pixel-render`: `overlay.rs`, `view.rs`, `lib.rs`.
- `ph2d-app-pixel`: novos `selecao.rs`, `solta.rs`, `edicao.rs`, `smoke_selecao.rs`; tocados state, input, gesture, draw, panel, camadas, paleta, frame, quadros, smoke, compose.
- `ph2d-panel-pixel-tools`: novos `ids_selecao.rs`, `seam_selecao.rs`, `seam_seletor.rs`, `paint_selecao.rs`, `paint_paleta.rs`; tocados ids, seam, paint, paint_camadas, column, lib.
- `ph2d-editor-core`: `pixel_panel/` (pasta), `ids/chrome/{mod.rs, pixel_edit.rs}`, `interaction/types_menu.rs` (`ContextMenuKind::PixelEditDialog`), `interaction/dispatch/{key.rs, pointer_down_menus.rs, pointer_up.rs}`, `interaction/state/{chrome_ops.rs, pixel_edit_ops.rs}`, `screens/hero.rs` (glob re-export, 700 linhas), `screens/hero/{chrome/mod.rs, chrome/pixel_edit.rs (z=178), context_menu_dialogs.rs, context_menu_overlay.rs, context_menu_pixel_edit.rs, menu_rows.rs}`, `widget/mod.rs`, `lib.rs`.
- `ph2d-i18n`: `lib.rs`, `pixel.rs`, novos `pixel_selecao.rs`, `pixel_dialogos.rs`.
- Gates na casa: `arch_mode_has_reconcile.rs`, `architecture_every_editable_field_is_a_channel.rs`, `nenhum_rotulo_do_app_pinta_nada.rs` (widget_lab saiu), `paineis_armados.rs`.
- ⚠️ Um merge pode partir: o glob de `hero.rs` (rustfmt parte-o em três e estoura as 700 linhas); a pasta `pixel_panel/` contra quem ainda edite o `pixel_panel.rs` antigo; o `reserve_bottom_strip` (P4) com a banda baixa.

## 5. Prova (por nome, verde)

`ph2d-pixel-core`: `selection_oracle` 8, `transform_oracle` 7 (censo dos 367), `selection_parity` 4, `affine_parity` 2, `selection_laws` 10; **6 mutações apanhadas**. App: `selecao::` 14, `seam_pixel_selecao` 7, `the_pixel_edit_dialogs_are_alive` 7; `ph2d-pixel-render` 6. Clippy `-D warnings` limpo nas crates do pixel (`9f6ff3a428`). Binário smoke em dia (`Finished` 0.29 s). A suíte completa é do integrador.

## 6. ABERTO

- **As 4 correcções do §3 (P6b).**
- Load/Save MSK, Paste as New Sprite, Auto Select Layer do Move, amostra «camadas visíveis» da varinha; Alt no teclado (Ctrl+Alt+I); tecto 100 do Modify e o arredondamento %→px por medir no Trial.
- Lacunas medidas: elipse 6/112, bilinear ±1, giro fora de 90°.
- `a_altura_de_abertura_de_um_painel_so_encolhe` com vizinhos (§2a).
- O [P4 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md) que continua aberto: `TagRange`/`ConstantFrameRate`/`LayerContinuous` no painel, selecção de intervalos na linha do tempo, P5.
- **Mutação no fecho da linha** (somar às anteriores): Mask ops, wand, border/contract, `Stroke::accepts`, `Op::Canvas` inverso, `Op::Batch` rollback, `rot90_origin`, canvas anchors.
- Fecho da linha: `/pd-linha-fechar`, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`.

## 7. Próximo item

**P6b = as 4 correcções do smoke, nesta ordem:** (1) ícone da varinha; (2) grupo de ferramentas com flyout; (3) gizmo de transformar da casa + ferramenta Transform; (4) formas do Vector com fillet. Para 3 e 4 ler primeiro o código da casa (`explorador`) e medir o que já existe. Cena de smoke que MOSTRE cada correcção (a próxima é a **10**), fotos de fábrica e com `HOME=/home/enio` (copiar antes `~/.ph2d/layout.txt`; `docs/Components/ferramentas/fotografa_cena.sh` aceita `"HOME=/home/enio PH2D_PIXEL_SMOKE=n"` no 1.º argumento).
