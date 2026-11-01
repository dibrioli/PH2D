# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P6k (ronda 3 do Trial: P5, Preview, paleta, arrasto da paleta) — 2026-11-01

> Leitor: a próxima LLM que assume a linha. A P6k está **feita e aprovada no smoke do dono (01/11, cenas 17 e 3)**.
> A **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada integrado, nada pushado.
> Lei: [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md)
> · anterior: [P6j](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-01_P6j.md) (o §5 tinha o aberto que continua aqui).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (`line/PixelArt`).
- Sobre o main `422fec2027`; P0–P6g no main; P6h–P6k só na linha.
- Commits P6k (`git log --oneline 05e67d4c88..HEAD`, 9):
  - `1da0523dac` docs: ronda 3 do Trial (P5, paletas, elipse > 150×96, Modify, lacunas da timeline).
  - `7f7b02db29` P5 pela lei do Trial e a paleta do Aseprite.
  - `ccf942d31e` docs: giro contra o Trial, recusas MEDIDAS.
  - `6f8750734c` a janela Preview, a cena 17 e as curas da ronda 3.
  - `b65610a617` tags aninhadas em filas e a Preview que pintava fora da janela.
  - `1fb72bab92` fotos da cena 17 depois das curas.
  - `8ef4af8660` arrastar amostra da paleta voa como o cartão do Kanban.
  - `27ded053d5` barra do acento à esquerda da 1.ª célula do vão.
  - `111eb70a08` a queda é função só do lugar do rato.
  - + este handoff e a emenda P6k do ADR-0196.

## 2. O que a P6k fez

1. **Oráculo** (Aseprite Trial 1.3.18, Xvfb :78, HOME isolada): ronda 3 em `docs/PixelArt/ferramentas/oraculo_timeline.md` (Ronda 3 P5 + lacunas), `oraculo_paleta.md`, `oraculo_rotacao.md`. Drivers `trial_p5_play.py`, `trial_paleta.py`, `trial_elipse_grande.py`, `trial_modify_tecto.py`; sondas `sonda_rotacao_fast.py`, `sonda_rotsprite.py`. Fixturas com `_header`: `crates/ph2d-pixel-core/tests/fixtures/{oracle_trial_play_p5,oracle_trial_ellipse_big,oracle_trial_modify}.json`; `shells/pixel-scenes/tests/fixtures/{oracle_trial_palette,oracle_trial_timeline (round3_gaps)}.json`.
2. **P5 (`ph2d-pixel-core/src/play.rs` reescrito)**: `PlayOptions{once, all, subtags (omissão true)}`, `PlaySpeed` em quartos inteiros (0.25..3×: 1,2,4,6,8,12), `Play{pilha, raiz, …}`. Lei: ao começar, a pilha é a cadeia de tags que contém o quadro (de fora para dentro; entre sobrepostas a que começa primeiro; mesmo intervalo → a primeira da lista é a de fora, `dentro` decide por índice); raiz = a mais interna; só se entra numa tag filha da do topo; repeat 0 = para sempre só na raiz, noutra conta 1 passagem; subtags OFF → só a raiz em loop, ignorando repeats; Play All ignora tags; Play Once = uma passagem da raiz (tag interna repete para sempre, senão a timeline direita; Ping-pong ida e volta) e pára no quadro de INÍCIO (o Trial ganha sobre o MVP); velocidade por documento, as outras opções globais; Rewind on Stop volta a `Play.start`. **Tag groups NÃO existem no Aseprite 1.3.18**: as filas de rótulos são só o desenho do aninhamento (`tag_rows`: a fila livre mais baixa por baixo das que a contêm).
3. **UI da timeline** (`ph2d-panel-pixel-timeline`): chip de velocidade na barra (`dropdown_chip_width_for`), `PLAY_TOGGLES` (Once, All frames, Subtags, Rewind, tooltip longo); acções First/Last in Tag, Loop Section (F2; tag «Loop» sobre o envelope, quadro único permitido); Shift+Enter = Play Preview, F7 = Preview. A coluna das camadas mantém `PANEL_MIN_W` enquanto resta uma coluna de quadro («Background» cortava a 1280 — gate que não corria desde a P6h).
4. **Janela Preview flutuante**: crate NOVA `ph2d-panel-pixel-preview` (StableImage, zoom, ×, botão play; fica abaixo da régua); `PIXEL_PREVIEW_PANEL` em `ph2d-editor-core/src/ids/chrome/pixel.rs`; cena 17 (`smoke_tocar.rs`, a entrada s1 do Trial); smoke `max_level` 17.
5. **Frames**: `copy_block` liga as cópias nas camadas contínuas; `move_cels_grow`; `CelsMove` ganhou `beyond`; largar cels numa fila de grupo é RECUSADO (divergência deliberada: o Trial destrói dados).
6. **Paleta (decisão do dono: como o Aseprite)**: em Indexed, reordenar/ordenar/mover/inverter muda só a paleta e deixa um remap pendente (`onde[antigo]=novo`); botão «Remap palette» (só com pendente) = `Op::Batch[Palette{P'}, Remap{perm}]`, 2.º passo de desfazer; undo/redo limpa o pendente; Ctrl+arrastar copia e insere (`palette::copy_insert`); Reverse colors; Gradient trunca por canal; Gradient by hue interpola em HSV com matiz crescente, arredondado. Código: `ph2d-pixel-core/src/palette.rs` (`reverse_permutation`, `gradient`, `gradient_hue`, `copy_insert`, `drag_permutation` movida para cá), `ph2d-app-pixel/src/paleta_remap.rs` (novo), `paleta.rs`. Testes de paleta movidos para `shells/pixel-scenes/tests/it/paleta.rs`.
7. **Arrasto da paleta** (3 commits finais, pedidos no smoke): voa como o cartão do Kanban (`paint_flight`; `ph2d-panel-pixel-tools/src/paint_paleta_voo.rs`), o vão aberto na ordem prevista, com Ctrl o vão da cópia, Esc cancela (`cancel_grid_row_drag`); barra vertical do acento (`StrokeToken::Heavy`) logo à esquerda da 1.ª célula do vão (`palette_drop_marker`); a queda é função SÓ do lugar do rato (`paint::Queda{mover, copiar}`, fn `queda`): célula sob o rato na grelha fixa, intervalos contam para a mais próxima, borda prende, meia célula de folga além da qual cancela; a agarrada fica sob o rato (vão começa em c − j). A pré-visualização grava a `Queda` e `seam::larga` lê-a, em vez do hit-test do despacho (que tinha zonas mortas nos intervalos → o vão saltava; dono: «ainda não ficou estável»). Gate `seam_pixel_paleta_voo.rs` (6 testes, incl. o varrimento píxel a píxel `sweeping_the_cursor_across_a_row_moves_the_gap_steadily_and_drops_what_was_shown`). Memória: `feedback_every_drag_reorder_flies_like_the_kanban_card.md`, `project_pixel_palette_reorder_like_aseprite_2026_11_01.md`.
8. **Selecção**: elipse grande 144/144 contra o Trial; Modify tecto 100 igual ao Trial, «%» ignorado; o campo Modify trunca decimais.
9. **⛔ Recusas MEDIDAS** (`oraculo_rotacao.md`): empates do Fast continuam 36 células (36 → 36); RotSprite 1 554 → 1 554 — NÃO fechados; só reabrir com medição nova.
10. **Fotos** `smoke_P6k_*.png` nesta pasta (cena 17 fábrica/dono/preview, cena 3 paleta). O `layout.txt` do dono mudou durante a sessão por OUTRA linha (UIUX, `PH2D_OBJECT_MODE_SMOKE=42`), não por esta.
11. **Catraca** `ph2d-app-pixel` tecto 13 424 respeitada (código movido, tecto não subiu).

## 3. Superfície de colisão NOVA (soma-se à P6j §3)

- `ph2d-editor-core`: `ids/chrome/pixel.rs` (`PIXEL_PREVIEW_PANEL`), `screens/hero/paint.rs`; `CelsMove` ganhou `beyond` (`match`/construtores sobre `PixelRequest`).
- Crate NOVA `ph2d-panel-pixel-preview` + registo: `ph2d-panel-registry-init` (Cargo.toml, lib.rs), `shells/desktop/Cargo.toml` e o gate `…the_shell_drives_is_in_its_registry`.
- `ph2d-pixel-core/src/{play,palette,frames}.rs`.
- `ph2d-app-pixel/src/{paleta,paleta_remap}.rs`.
- `ph2d-panel-pixel-timeline/src/{paint_grid,paint_props,state}.rs`; `ph2d-panel-pixel-tools/src/{ids,paint,paint_paleta,paint_paleta_voo,seam,state}.rs`.
- `shells/pixel-scenes/tests/it/{paleta (movido),tocar,preview,intervalo,quadros,comum,main}.rs` + fixturas acima.

## 4. Prova (por nome, verde, 01/11)

- `ph2d-panel-pixel-tools`: `seam` 31 (incl. `seam_pixel_paleta_voo`). `ph2d-host-pixel-scenes`: `paleta` 21, `tocar`, `preview`. `ph2d-panel-registry-init`: `nenhum_rotulo_do_app_pinta_nada` 14. `ph2d-pixel-core`: `play_oracle`, `selection_oracle`.
- `fmt` e `clippy -D warnings` limpos nas crates tocadas. Provas de mutação NÃO corridas (ver §5).

### Smoke (cenas 17 e 3)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=17 ./target/smoke/ph2d-host-desktop`
2. Cena 17: abrir Preview (F7), tocar com Shift+Enter; mudar velocidade no chip e as opções Once/All frames/Subtags/Rewind. Errado: a Preview pinta fora da janela, ou tags aninhadas sobrepostas na mesma fila.
3. Cena 3 (`PH2D_PIXEL_SMOKE=3`): arrastar uma amostra da paleta — voa, o vão abre sem saltar, a barra do acento marca a queda; Ctrl copia; Esc cancela. Em Indexed, aparece «Remap palette» só depois de reordenar. Errado: o vão salta nos intervalos entre células.

## 5. ABERTO (na ordem)

1. Fecho da linha (`/pd-linha-fechar`, DIRETRIZ §1.5.9): `cargo machete`, auditoria, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental` — ou continuar ondas se o dono quiser.
2. Provas de mutação **A18, B9, B10** não corridas (só-pintura), mais as desta onda.
3. Preview: sem pegas de redimensionar, sem fundo xadrez, sem entrada no menu Window.
4. Botões da paleta abaixo da dobra a 1040 px (o painel rola).
5. Foto da cena 9 não refeita.
6. `a_altura_de_abertura_de_um_painel_so_encolhe` depende da ordem (vizinhos) — aberto conhecido.
7. RotSprite / empates do Fast: recusas medidas — só reabrir com medição nova.
8. Outras listas reordenáveis por arrasto do app devem herdar voo + barra + queda por lugar do rato (memória `feedback_every_drag_reorder_flies_like_the_kanban_card.md`).

Se o dono preferir integrar: `/pd-linha-fechar`. Confira o código antes de acreditar numa ausência.
