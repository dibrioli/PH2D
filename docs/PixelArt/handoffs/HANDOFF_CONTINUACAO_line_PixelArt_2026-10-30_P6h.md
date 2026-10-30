# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P6h (o intervalo da linha do tempo) — 2026-10-30

> Leitor: a próxima LLM que assume a linha. A P6h está **feita**; o smoke do dono (cena 16) está
> **POR FAZER**. A **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada integrado, nada pushado.
> Lei: [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md)
> · anterior: [P6g](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-27_P6g.md) (smoke da cena 15 OK, dono, 30/10) · [P4 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-26_P4.md) (o que esta onda fecha: TagRange/ConstantFrameRate/LayerContinuous, selecção de intervalo).
> Oráculo: [`oraculo_timeline.md`](../ferramentas/oraculo_timeline.md) · condutor `docs/PixelArt/ferramentas/trial_timeline.py`.

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (`line/PixelArt`).
- Linha reaberta sobre o main `422fec2027` (30/10); P0–P6g já estava no main. Ícone do Pixel Sprite na Hierarquia: fica `LayoutGrid` (o dono não pediu outro).
- Commits (`git log --oneline 422fec2027..HEAD`):
  `5a77c467e6` 11 testes fecham 13 sobreviventes das mutações P6d–P6g ·
  `092acc816d` feature P6h ·
  `ebd427b3f9` testes do canal compilam (`PointerEvent` sem `Default`, `active_id`), censo de entradas conhece a `PixelTimelineSurface`, fotos da cena 16 ·
  `17d5a61f68` testes P6h para o crate NOVO `shells/pixel-scenes` (`ph2d-host-pixel-scenes`; precedente `game-scenes`/`code-scenes`): a catraca `the_shell_only_shrinks_and_every_app_crate_too` conta `tests/`; `ph2d-app-pixel` 14 073 → 13 364 linhas, tecto 13 424 ·
  `44bb34a563` import sem uso em `quadros.rs` · + este handoff.

## 2. O que a P6h fez

Lei medida no **Aseprite Trial 1.3.18.6 CORRIDO** em Xvfb próprio. Entrada = `tl.ase` 16×16, 6 quadros 100..600 ms, camada A não contínua, B contínua com cels nos quadros 1 e 3, tag «walk» 2..4. Tabela: `oraculo_timeline.md`; fixtura `shells/pixel-scenes/tests/fixtures/oracle_trial_timeline.json`.

1. **Canal de gestos próprio** `ph2d-editor-core/src/interaction/pixel_timeline_surface.rs`: `PixelTimelineHitKind {Header, Cel, TagEdge, RangeOutline}`, `PixelTimelineGesture`, `InteractiveState::PixelTimelineSurface`, hooks em `dispatch`/`pointer_{down,move,up}.rs`. Canal separado porque cada painel drena o seu canal inteiro. `is_pointer_surface` junta grafo/timeline/Flip/Pixel e substituiu duas condições em `pointer_down.rs` (**699/700 linhas**).
2. **Modelo do painel** (`pixel_panel/timeline.rs`): `PixelRange {kind: None|Frames|Cels|Layers, layers, frames}`; `PixelPanelInfo.range`; `PixelRequest::Range(PixelRange)` e `RangeMove{before}`; `PixelLayerRow.continuous`; `PixelLayerWidget::Continuous` + `TIMELINE` (6) — ALL continua 5; `LayerRowPainter.continuous` (só a linha do tempo pinta o 3.º ícone: `MoreHorizontal` descontínua / `Minus` contínua).
3. **Família `ph2d-app-pixel/src/intervalo.rs`**: `Doc.range` (estado de vista, não passo de undo; ids mortos caem na leitura), `range_frames/range_keys/range_info/alvo_quadros/range_span`, `aplica_intervalo` (Range; RangeMove → `frames::move_block`; LayerContinuous aplicado SEM história via `ph2d_pixel_core::apply`).
   - `quadros.rs`: FrameDuration vai a todos os do intervalo; TagRange com a lei do diálogo (to<from → to=from, presos ao último); Delete = os do intervalo (tag inteira coberta sai — `tags_after_delete` já o fazia); Reverse com intervalo = o span contíguo, 1 quadro ou soltos = nada; LinkPrevious com 2+ = `link_range` (`Op::Batch` por camada).
   - `edicao.rs`: tecla Delete sem selecção e com intervalo = `frames::clear` (cels saem, quadros ficam).
   - `selecao.rs`: Esc limpa o intervalo (depois de moldura/laço, ANTES do Deselect — **ordem não medida no Trial**).
4. **`ph2d-pixel-core` `frames.rs`**: `move_block(s, ids, before)`, `clear(s, keys)`.
5. **Painel `ph2d-panel-pixel-timeline`**: `gestos.rs` (Geo da grelha, `Arrasto {Cabecalhos, Cels, Ponta, Bloco}`; Shift/Ctrl+clique ACRESCENTA, nunca preenche o vão nem retira; arrastar cabeçalhos = contíguo; cels = rectângulo; ponta de tag larga no quadro sob o cursor; moldura larga o bloco na fronteira mais próxima). Nome da camada = LayerSelect + Range(Layers). `paint_grid`: intervalo aceso, moldura Accent com 4 lados registados como `RangeOutline`, pontas da tag, linha de inserção do bloco, pré-visualização da tag. `paint_props`: título «Frame n» / «Frames [a...b]» / «Multiple Frames»; campo da duração mostra o 1.º do intervalo; botão Constant Frame Rate (aceso = modo; campo mostra o quadro 1 e envia `ConstantFrameRate`; fecha quando o activo ou o intervalo muda); Tag From/To base 1. Ids `tag_edge_id`, `RANGE_EDGES`, `CONSTANT_RATE`, `TAG_FROM`, `TAG_TO`. i18n novas em `ph2d-i18n/src/pixel.rs`.
6. **Cena `PH2D_PIXEL_SMOKE=16`** (`ph2d-app-pixel/src/smoke_intervalo.rs`; a entrada do oráculo, quadros 2..3 escolhidos). A sonda de nível desconhecido passou ao **17**. Fotos `smoke_P6h_cena16_2026-10-30.png` e `_dono_` (layout.txt do dono igual antes/depois); foto da cena 9 com a lei Fast da P6g, `smoke_P6h_cena9_fast_2026-10-30.png` e `_dono_` (o F do Fast muda poucas células; RotSprite igual).
7. **Mutações P6d–P6g**: 49 corridas, 31 sangraram, 18 sobreviveram; 13 fechadas por testes (`shells/pixel-scenes/tests/it/sobreviventes.rs`). EQUIVALENTES provadas: `escolhe_sob` `!is_group` (`op.rs:282-284` recusa célula em grupo); `carrega_msk` `count()==0` (select vazio em Replace já deselecciona, `selection.rs:528`); `cheia && is_closed()` (formas abertas só dão contornos abertos).

## 3. Smoke do dono — POR FAZER

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=16 ./target/smoke/ph2d-host-desktop`
2. Na linha do tempo, os quadros 2 e 3 vêm escolhidos: contorno colorido à volta, título «Frames [2...3]» no painel de baixo.
3. Clique no quadro 5, depois Shift+clique no 2: o intervalo cresce. Mude a duração no campo: vale para todos os escolhidos.
4. Arraste a ponta da faixa «walk» para outro quadro: a faixa estica; os campos From/To acompanham.
5. Arraste o contorno do intervalo para a esquerda ou a direita: os quadros mudam de lugar (uma linha mostra onde cairão).
6. Na camada B, clique no ícone de três pontinhos: vira um traço (camada contínua). Clique no botão Constant Frame Rate: todos os quadros ficam com a duração do primeiro.
7. Tecla Delete com quadros escolhidos e sem selecção: o desenho dos quadros some, os quadros ficam. Esc: o contorno some.
- Deu errado se: o contorno não aparece; Shift+clique troca o intervalo em vez de somar; o arrasto do bloco apaga quadros; o ícone da camada não muda.

## 4. Superfície de colisão NOVA (soma-se à P6g §4)

- `ph2d-editor-core`: `interaction/pixel_timeline_surface.rs` NOVO, `interaction/{dispatch,pointer_down,pointer_move,pointer_up}.rs` (⚠️ `pointer_down.rs` a 699/700), `pixel_panel/timeline.rs`, `pixel_panel/mod.rs`, `layer_row` (`LayerRowPainter.continuous`); testes `pixel_timeline_surface` 2.
- `ph2d-panel-pixel-timeline`: `gestos.rs` NOVO, `paint_grid`, `paint_props`, `seam.rs`, ids.
- `ph2d-i18n`: `pixel.rs`.
- `ph2d-pixel-core`: `frames.rs`.
- `ph2d-app-pixel`: `intervalo.rs` NOVO, `smoke_intervalo.rs` NOVO, `quadros.rs`, `edicao.rs`, `selecao.rs`, `state.rs` (`Doc.range`), `smoke.rs`, `lib.rs`; **os testes P6h saíram para `shells/pixel-scenes`**.
- **Crate NOVO `shells/pixel-scenes`** (`ph2d-host-pixel-scenes`): `Cargo.toml`, `tests/it/{intervalo,sobreviventes}.rs`, `tests/fixtures/oracle_trial_timeline.json`; entrada no workspace — **conflito provável com outras linhas que acrescentam crates**.
- `docs/PixelArt/ferramentas/`: `trial_timeline.py`, `oraculo_timeline.md`; fotos em `docs/PixelArt/`.
- Registo de painéis/ids: `ph2d-panel-registry-init` (testes `pixel`).

## 5. Prova (por nome, verde)

- `ph2d-host-pixel-scenes`: `intervalo` 11, `sobreviventes` 11.
- `ph2d-app-pixel` crate inteira 114 (motivo: código movido/dep).
- `ph2d-panel-pixel-timeline`: `seam` 10, `every_word` 3. `ph2d-panel-pixel-tools`: `seam` 26.
- `ph2d-editor-core`: `pixel_timeline_surface` 2, `flip_strip`, `pixel_layer_row`, `icons` 182, `the_shell_only_shrinks` 2, `architecture_no_dependency_climbs_a_layer`, `architecture_every_member_inherits_the_workspace_lints`, `architecture_no_orphan_source_file`.
- `ph2d-panel-registry-init`: `pixel` 5, `paineis_armados`. `ph2d-i18n`: `key` 3. `ph2d-pixel-core`: `frames` 9. `ph2d-host-desktop`: `pixel` 26.
- `cargo machete` e `check-standalone-optional` limpos; fmt limpo. **Clippy NÃO corrido** (fecho da linha).

## 6. ABERTO

- Smoke do dono da cena 16 (§3).
- `a_altura_de_abertura_de_um_painel_so_encolhe` reprova COM VIZINHOS (filtro `quantas`), verde sozinho — o defeito já aberto do P6 §2a.
- **`ph2d-app-pixel` sem folga na catraca** (13 364 / tecto 13 424): a próxima onda tem de mover para `ph2d-pixel-*` ou `shells/pixel-scenes`.
- **Mutações por correr**: P6b e P6c (nunca corridas); as da P6h para o fecho: `acrescenta` sem o `contains`; `fronteira` round→floor; ordem Esc intervalo × Deselect; `range_span` contíguo; `alvo_quadros` sem intervalo; `TagRange` clamp; LayerContinuous pela história; `move_block` `at`.
- **Sobreviventes por fechar**: #8 paleta do `born_from_paste` fora do Indexed (dúvida de LEI → oráculo/dono); #10 sinal de `FAST_TIE_LEFT` (precisa de empate exacto na homografia); #12/#13 `outline_only` e `.or(v.mostra)` no `viva::draw` (só se vêem na cena desenhada); #18 `Px` com floor; metade `mode.rs` do #20 (`viva::assenta` no `leave`).
- **Trial não medido**: Ctrl ao largar o bloco (copiar); mover só cels; Shift+clique em cel depois de arrasto; Esc com selecção E intervalo; Reverse com quadros soltos; undo da tecla Delete.
- O resto do §10 do [handoff de integração 28/10](HANDOFF_INTEGRACAO_line_PixelArt_2026-10-28.md): RotSprite, empates do Fast, elipse > 150×96, tecto do Modify, LengthDisplay, paletas, P5, P7–P10.
- Fecho da linha: `/pd-linha-fechar`, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`.

## 7. Próximo item

Smoke da cena 16 pelo dono; depois o dono escolhe a onda (as mutações P6b/P6c e os sobreviventes podem ir num bloco com o fecho da linha).
