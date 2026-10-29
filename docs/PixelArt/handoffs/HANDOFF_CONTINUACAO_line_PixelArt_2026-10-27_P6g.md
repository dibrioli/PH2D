# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P6g (as pontas soltas das formas e da selecção) — 2026-10-27

> Leitor: a próxima LLM que assume a linha. A P6g está **feita**; o smoke do dono (cena 15) está
> **POR FAZER**. A **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada integrado, nada pushado.
> Lei: [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) §12 (emenda P6g)
> · anterior: [P6f](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6f.md) (smoke OK) · [P6c](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6c.md) e [P6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6.md) (§6: os abertos que esta onda fecha).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (`line/PixelArt`).
- Merge-base `c3662cd8e4` (o `main` não andou; `git rebase main` = «atualizado»).
- Pedido (ordem do dono 27/10): fechar num bloco as pontas soltas das formas e da selecção.
- Commits `a6b5d138ec..9d232ed41a` (9; `git log --oneline 93043a29d1..HEAD`):
  `a6b5d138ec` inclinar pelo gizmo, flyout sem órfão, Tip, Alt no teclado, varinha visíveis, Paste as New Sprite, Auto Select Layer ·
  `0d0d2284ce` bilinear do Sprite Size igual ao Trial em 8463/8463 canais ·
  `385da3e772` o giro Fast fora de 90° e a inclinação pela lei do Trial (76/86 célula a célula) ·
  `588743a3b0` a Fast Rotation do Trial em `affine_fast.rs` (tecto de 700 LOC: `affine.rs` 779 → 643) ·
  `5ef43ebad7` cena `PH2D_PIXEL_SMOKE=15` ·
  `8ea775b037` a elipse da selecção igual ao Trial em 715/715 caixas ·
  `fb2dbe8d81` ler e gravar `.msk` byte a byte como o LibreSprite (45 casos) ·
  `710a2aa7f3` Select ▸ Load from MSK / Save to MSK ·
  `9d232ed41a` dicas do Transform falam de inclinar pelo lado; fotos da cena 15.

## 2. O que a P6g fez

1. **Inclinar pelo gizmo.** 4 zonas de inclinação logo fora do meio de cada lado (ids `GIZMO_SKEW_T/R/B/L` = 974–977, `ph2d-editor-core` `gizmo/hit.rs`); mesma lei de geometria das zonas de rodar da casa (`paint_geom::skew_rects` partilha `local_axes`/`outer_rect` com `rotate_rects`). **Opt-in**: `paint_sprite_gizmo_skewing` e `sprite_gizmo_hit_at(view, body, skew, x, y)` — o gizmo de sprite da casa (`paint_sprite_gizmo_with`) não mudou. Registadas ANTES das de rodar (rodar ganha na sobreposição); as alças de aresta ganham a ambas; com o cursor numa zona a alça da aresta pinta-se como losango (como o canto vira círculo no rodar). Lei de colocação = a do MVP do dono (`affine.js transformHandlePoints`: skew-n/s/e/w fora dos meios dos lados). A conta do arrasto é o `Handle::Skew` já existente do núcleo (paridade MVP), agora alcançado por `solta::alca` (SKEW_T→`Skew(1,2)`, B→`(1,0)`, L→`(0,1)`, R→`(2,1)`); serve a moldura solta e a forma viva (`viva` usa o mesmo `hit_of`/arrasto). Nem o gizmo da casa nem o Vector têm alça de skew (o «Skew» do Vector é parâmetro da forma iso-cubo) — registado.
2. **Flyout 3+1.** Lei `float_bar::flyout_cols(n)` (≤ 4 → uma linha; senão o menor de 3–5 cuja última linha não é um botão sozinho), varrida por CONSUMIDOR: flyout do `ToolBars` (selecções Pixel 4→4 numa linha, catálogo de formas Pixel/Vector 49→5 colunas, formas do Painter 5→3) e `board_bar_layout` (Quadro 18→3, resultado inalterado). Teste `o_flyout_nunca_deixa_um_botao_sozinho_na_ultima_linha`.
3. **«Shape» duas vezes.** A chave do seletor da ponta do pincel `panel.pixel.shape` passou a `panel.pixel.tip` («Tip», o tipo de pincel do Aseprite); «Shape» só no cabeçalho da secção da forma.
4. **Unidade da etiqueta do arrasto — DECISÃO: fica «{n} px» (células), não `LengthDisplay`.** Lei da casa: `LengthDisplay` (`ph2d-editor-core/src/length.rs`) é para comprimentos de MUNDO (metros/px da cena: réguas, etiqueta do Vector, Transform do Inspector, snap da grelha); tamanhos nativos da imagem imprimem px da imagem fixos (Inspector `size_px`, tamanhos de camada do Painter, ferramentas de imagem). A célula do pixel art não tem tamanho fixo no mundo (o sprite pode ser escalado): converter para metros imprimiria um número que muda com a escala do objecto para a mesma edição.
5. **Alt chega à família.** `PixelApp::keyboard(v, key, pressed, m: Mods)` (a shell `despacho_pixel` passa `Mods::of`); `input::key_with` + `com_alt` no novo `input_teclas.rs` (`input.rs` passaria o tecto de 700 LOC); `input::key` mantém a assinatura sem Alt para os testes. Atalhos Alt do Aseprite: Ctrl+Alt+I = Sprite Size (NÃO Inverse — Inverse é Ctrl+Shift+I; o texto do pedido errava, o estudo `01_estudo_aseprite.md:39` diz Sprite Size), Alt+Shift+N New Group, Alt+N New Frame, Alt+B New Empty Frame, Alt+M Duplicate Linked, Alt+C Delete Frame, Alt+I Reverse; qualquer outra combinação com Alt não é da família (Alt+E não é E).
6. **Varinha «Visible layers».** `PixelOptions.wand_sample_visible` (próprio, como as opções de floodfill por ferramenta do Aseprite), ids `WAND_SAMPLE_LAYER/VISIBLE`, pintados na secção da varinha.
7. **Paste as New Sprite.** `PixelLayerAction::PasteAsSprite` (botão depois de Paste as Reference, com clip), `PixelApp.nasce`; a shell `fase_object_add` drena `object_add::born_from_paste` depois de `born_from_dialog`: objecto novo do tamanho do clip, o seu modo e paleta, fundo transparente, clip em (0,0), sem passos de undo no documento novo; este documento não muda.
8. **Auto Select Layer** (Transform, que substituiu o Move): `PixelOptions.auto_select_layer`, caixa na secção Transform; o premir escolhe a camada de imagem visível de cima com célula pintada ali (`camadas::escolhe_sob`) e só então levanta.
9. **Load/Save MSK.** `ph2d_pixel_io::msk` (`read_msk`/`write_msk`/`MaskFile`), byte a byte com o LibreSprite 1.3 em 45 casos de oráculo (`docs/PixelArt/ferramentas/oraculo_msk.py` + `oraculo_msk.md`; fixture `crates/ph2d-pixel-io/tests/fixtures/oracle_msk.json`, 839 KB; o Trial não grava ficheiros). Formato: u32 tamanho, `0x9500`, w/h, origem i16 com sinal, chunk tipo 2, linhas `ceil(w/8)` MSB à esquerda, preenchimento escrito a 1; legado 8000 bytes 320×200. Desvios DECLARADOS: legado lido do offset 0 (o LibreSprite salta 6 bytes); chunks partidos = `BadChunk` (o LibreSprite bloqueia). Menu Select: Load from MSK… (sempre), Save to MSK… (com selecção); `PixelApp::carrega_msk` (um passo; ficheiro mau avisa, sem passo) e `msk_de`.
10. **Lacunas medidas contra o Trial** (oráculo CORRIDO por script, fixtures com cabeçalho):
    - **Elipse da selecção: 715/715 caixas** (era 106/112). Fixture `oracle_trial_ellipse_grid.json` (todas 1..24×1..24 nos dois sentidos do arrasto iguais, lados pares 26..150×26..96, 4 posições), drivers `trial_elipse_grelha.py` + `trial_fixture_elipse.py`. Lei por eixo: lado ímpar, ou par em `EVEN_SIDES_ZINGL={4,8,12,22}` (tabela MEDIDA, sem explicação) = Zingl `plotEllipseRect`; outro n par = Zingl em n+1 sem as duas linhas junto à do meio e repetindo a do meio. ⛔ Recusas na grelha (de 576): elipse contínua (impossível em 13), Zingl∩contínua 276, viés de Zingl ≤ 477, variantes de passo 445, Kennedy ≤ 186, ímpar+1-menos-meio para todo o par 504. Tamanhos além de 150×96 NÃO medidos (tela 160×100).
    - **Bilinear (Sprite Size): 8463/8463 canais** (era 8392): o Trial acumula o passo `(S−1)/(N−1)` num f64 corrente por célula de destino. ⛔ Recusas: ordens de f64 (8392), ponto fixo 2^4–2^20 (8400), f32 (8361).
    - **Rotação/inclinação Fast fora de 90°**: lei nova em `ph2d-pixel-core` `affine_fast.rs` (filho de `affine.rs`, partido pelo tecto de LOC): cantos exactos da moldura, cos/sin guardados em 16.16 arredondados para longe de zero, cantos com floor, fonte esticada sobre esse quad inteiro (paralelogramo exacto inteiro; senão homografia), cada célula copia a fonte sob o seu centro, empate exacto = um fio à esquerda. **76/86** casos Fast célula a célula; resíduo DECLARADO = 36 células de empate em 10 casos (r86 a 60°, 72°, 150°, −60°, −37°, 143°; r75 100°; r58 53°, 60°, 150°). O sinal do skew do Trial é oposto ao nosso. Fixture `oracle_trial_rotation.json` (90 casos), driver `trial_rotacao.py`; gate `rotation_oracle.rs` substitui o `mede_rotacao.rs` ignorado. **RotSprite NÃO resolvido**: lacuna por caso registada em `ROTSPRITE_GAP` (8–264 células). 47/50 casos Fast do MVP agora diferem do MVP (`MVP_FAST_CONFLICTS`): o Trial ganha nos pixels. ⚠️ Na app, rodar/inclinar/escalar-arrastando com Fast coloca células diferentes de antes; o aspecto da cena 9 (giro Fast) pode ter mudado — a foto não foi retirada de novo.

**Cena `PH2D_PIXEL_SMOKE=15`** (`crates/ph2d-app-pixel/src/smoke_pontas.rs`, 128×80 RGBA): duas camadas, «Figures» (activa, em baixo) e «Layer 2» vazia por cima; o «F» da cena 9 levantado numa moldura solta com Skew 20 (a barra mostra Skew 20); losango verde à direita; o F copiado (Paste as New Sprite visível); Transform na mão com Auto Select Layer LIGADO, varinha com Visible ligado; flyout das selecções aberto. `max_level` 15; a sonda de nível desconhecido (`tests/it/edit.rs`) passou ao 16. Fotos (1930×1040, placa 1,1 de 16 GB): `smoke_P6g_cena15_2026-10-27.png` (fábrica) e `smoke_P6g_cena15_dono_2026-10-27.png` (`HOME=/home/enio`; `layout.txt` sha256 `b31e52909b5b09f3…` igual antes/depois). A foto apanhou as dicas do Transform ainda a dizer «Skew in the field»: corrigido (`pixel.hint` transform + dica do painel) e refotografado.

## 3. Smoke do dono — POR FAZER

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=15 ./target/smoke/ph2d-host-desktop`
2. Canto superior esquerdo da tela: as 4 ferramentas de selecção numa ÚNICA linha.
3. A moldura pequena do «F» está inclinada; ponha o rato logo fora do meio de um lado: o quadradinho vira losango; arraste de lado → inclina mais (o número Skew muda); Apply.
4. Ctrl+Alt+I abre Sprite Size; Alt+N acrescenta um quadro.
5. Escolha Magic Wand (W) e clique na tela vazia junto do losango verde: com «Visible layers» ela selecciona à volta do losango.
6. Painel Layers: Paste as New Sprite → nasce um objecto Pixel Sprite novo só com o F.
7. De volta ao primeiro sprite: clique em Layer 2, escolha Transform (V), clique no losango verde → a camada activa salta para Figures.
8. Select: desenhe uma selecção, Save to MSK…, Deselect, Load from MSK… → ela volta.
9. Escolha uma ferramenta de forma (U): o pincel diz Tip, a secção da forma diz Shape.
- Deu errado se: o flyout tem um botão sozinho na última linha; o losango não aparece; Alt+N não faz nada; Load from MSK não devolve a selecção.

## 4. Superfície de colisão NOVA (soma-se à P6f §4)

- `ph2d-editor-core`: `gizmo/hit.rs` (ids 974–977), `gizmo/paint.rs` (`paint_box`, `paint_sprite_gizmo_skewing`, `paint_diamond`; ⚠️ `sprite_gizmo_hit_at` ganhou o parâmetro `skew`), `gizmo/paint_geom.rs` (`skew_rects`, `local_axes`), `gizmo/mod.rs`, `screens/hero/float_bar.rs` (`flyout_cols`), `tool_bars.rs`, `board_bar_layout.rs` (ficheiro do MiroClone: só a lei, resultado inalterado), `pixel_panel/{mod.rs (opções), layers.rs (PasteAsSprite), selection.rs (LoadMsk/SaveMsk)}`.
- `ph2d-panel-pixel-tools`: `ids.rs` (CHECKS 10, SEGMENTS 8, LAYER_ACTIONS 14), `ids_selecao.rs` (SELECT_ACTIONS 10), `paint.rs`, `paint_camadas.rs`, `paint_selecao.rs`, `seam.rs`.
- `ph2d-i18n`: `pixel.rs` (`panel.pixel.tip`), `pixel_selecao.rs`.
- `ph2d-pixel-core`: `affine.rs`, `affine_fast.rs` NOVO, `selection.rs`, `transform.rs`; testes `rotation_oracle.rs` NOVO, `mede_rotacao.rs` APAGADO, fixtures NOVAS.
- `ph2d-pixel-io`: `msk.rs` NOVO, `lib.rs`, `tests/msk.rs` NOVO, fixture.
- `ph2d-app-pixel`: `input.rs`, `input_teclas.rs` NOVO, `compose.rs` (⚠️ `keyboard(.., Mods)`), `selecao.rs`, `camadas.rs`, `object_add.rs` (`born_from_paste`/`spawn_doc`), `state.rs` (`nasce`), `solta.rs`, `viva.rs`, `draw.rs`, `edicao.rs` (`Prancheta::de` `pub(crate)`), `smoke.rs`, `smoke_pontas.rs` NOVO, `lib.rs`; testes `pontas.rs` NOVO (9), `selecao.rs` (`Palco` agora `pub(crate)`).
- Shell: `input_dispatch/despacho_pixel.rs`, `render_loop/fase_object_add.rs`.

## 5. Prova (por nome, verde)

- `ph2d-app-pixel`: `pontas::` 9, `catalogo` 3, `ajustes` 7, `viva` 8, `selecao` 14, `camadas` 5, `edit` 16, `quadros` 4, `formas` 8, `desenho` 11, `painel_e_dialogo` 6.
- `ph2d-pixel-core`: `rotation_oracle` 2, `affine_parity` 2, `transform_oracle` 7, `selection_oracle` (com os dois gates da elipse), `selection_laws` 10, `selection_parity` 4. `ph2d-pixel-io`: `msk_` 6.
- `ph2d-editor-core`: `architecture` 118 (tecto de LOC, `the_shell_only_shrinks`), `gizmo::` 82, `o_flyout` 2.
- `ph2d-panel-pixel-tools`: `seam_pixel` 26, `the_hint` 2; `ph2d-panel-registry-init` `the_pixel` 4; `ph2d-i18n` `key` 3; `ph2d-host-desktop` `pixel` 26.
- `cargo-check-narrow ph2d-host-desktop` ✓. **Clippy NÃO corrido** (fecho da linha).

## 6. ABERTO

- O smoke do dono da cena 15 (§3).
- RotSprite contra o Trial (`ROTSPRITE_GAP`); as 36 células de empate do Fast; elipse com lados > 150×96 não medida e a tabela `{4,8,12,22}` sem explicação; foto da cena 9 com a lei Fast nova.
- O tecto do Modify (100) e o arredondamento %→px ainda não medidos no Trial ([P6 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P6.md)); `a_altura_de_abertura_de_um_painel_so_encolhe` com vizinhos (P6 §2a); [P4 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md) (TagRange/ConstantFrameRate/LayerContinuous, selecção de intervalo na timeline, P5).
- **Mutações no fecho da LINHA** (somar às da P6b–P6f): `flyout_cols` com `n % c != 1`; zonas de skew registadas antes das de rodar (ordem); o losango `over_skew`; `solta::alca` skew (trocar T/B); `com_alt` com `if m.alt` retorno cedo; `escolhe_sob` com `.rev()` (o de cima primeiro) e `is_shown`; `born_from_paste` paleta só para Indexed; `carrega_msk` com `count()==0` → deseleccionar; conteúdo de `EVEN_SIDES_ZINGL`; bilinear acumulado vs directo; sinal de `FAST_TIE_LEFT`; sentido de arredondamento de `TRIAL_TRIG_ONE`.
- Fecho da linha: `/pd-linha-fechar`, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`.

## 7. Próximo item

Smoke da cena 15 pelo dono; depois o dono decide a onda seguinte.
