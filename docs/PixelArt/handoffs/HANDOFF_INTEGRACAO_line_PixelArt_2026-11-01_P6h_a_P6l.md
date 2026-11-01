# HANDOFF DE INTEGRAÇÃO — linha Pixel Art (P6h..P6l) — 2026-11-01

> Leitor: o agente integrador (lote, `scripts/integrar-rodada.sh`). A linha **fecha e PARA**: não integra, não faz push.
> Lei: [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md) (emendas P6h–P6k) · anterior de integração: [28/10](HANDOFF_INTEGRACAO_line_PixelArt_2026-10-28.md) (P0–P6g, já no main).
> Mecanismo de cada onda: [P6h](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-30_P6h.md) · [P6i](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-01_P6i.md) · [P6j](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-01_P6j.md) · [P6k](HANDOFF_CONTINUACAO_line_PixelArt_2026-11-01_P6k.md). A P6l não tem handoff próprio: está no §4 daqui.

## 1. Identidade

- Branch `line/PixelArt`; worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt` (`cd` + `pwd` + `git branch --show-current` antes de ler).
- Merge-base `422fec2027` (= `git merge-base HEAD main` à data); HEAD `b4c0efd542`; 30 commits (`git log --oneline 422fec2027..HEAD`); 172 ficheiros, +22 360/−614 (inclui docs/fotos).
- **Smoke do dono OK em 01/11**: P6k (cenas 17 e 3) e P6l (cenas 3, 17, 16: menus da paleta, Preview com pegas/xadrez/menu Window, barra da queda na linha do tempo). P6h–P6j: OK em 31/10–01/11.
- Commits (mais antigo primeiro):
  - P6h: `5a77c467e6` 11 testes fecham 13 sobreviventes P6d–P6g · `092acc816d` intervalo da timeline, pontas da tag, contínua, Constant Frame Rate · `ebd427b3f9` testes do canal + censo · `17d5a61f68` testes P6h para `shells/pixel-scenes` (catraca 14 073 → 13 364) · `44bb34a563` import sem uso · `0260c8292c` handoff · `b621fc4c2d` smoke do dono 31/10 (D1–D3).
  - P6i: `a688bf172c` D1 Enter num campo não toca · D2 o bloco voa com o vão · D3 pega/cursor na ponta da tag · `1a835345c7` nome da tag depois da pega · `7253796620` handoff + emenda ADR.
  - P6j: `2249259e7a` ronda 2 do Trial (Ctrl ao largar copia, cels movem só cels, Reverse dos soltos, Esc, Select All, #8) · `9635d2da9a` sobreviventes #10 #12 #13 #18 #20 · `283323ef45` ids-fantasma na moldura a meio do arrasto + gestos conduzidos · `3b44708025` oráculo ronda 2 · `03d639c72c` sobreviventes A3 A7 A9 B2 A11 B1 · `3604ff40b4` undo devolve o intervalo (`Doc.ranges`) · `f85fef993d` clippy `ParCel` · `05e67d4c88` handoff.
  - P6k: `1da0523dac` docs ronda 3 · `7f7b02db29` P5 pela lei do Trial + paleta do Aseprite · `ccf942d31e` docs recusas MEDIDAS · `6f8750734c` janela Preview + cena 17 · `b65610a617` tags aninhadas em filas + Preview fora da janela · `1fb72bab92` fotos · `8ef4af8660` amostra da paleta voa · `27ded053d5` barra do acento da queda · `111eb70a08` queda = função só do lugar do rato · `2a91de355e` handoff + emenda.
  - P6l: `7d31a2e66e` feat (Preview com pegas/xadrez/menu Window; paleta como no Aseprite; barra da queda na timeline; censos com registo privado) · `b4c0efd542` mutações A18, B9, B10 e as da P6l.

## 2. O que a linha entrega (P6h–P6l)

- **P6h** — intervalo da linha do tempo pela lei do Aseprite Trial 1.3.18 (CORRIDO, fixtura `oracle_trial_timeline.json`): canal de gestos próprio `PixelTimelineSurface`, `PixelRange`, `RangeMove`, Constant Frame Rate, TagRange, camada contínua (`PixelLayerWidget::Continuous`), Delete/Reverse/Link por intervalo; cena 16; crate de testes `shells/pixel-scenes`.
- **P6i** — Enter num campo focado não toca; o bloco arrastado voa com o vão (lei do voo do cartão do Kanban extraída para `ph2d_editor_core::paint_flight`, o Kanban passa a usá-la); pegas e cursores (`PointerLook::{Grab,Grabbing,ResizeEw}`) nas pontas da tag.
- **P6j** — ronda 2 do Trial: Ctrl ao largar copia (`copy_block`), `CelsMove`, Reverse sobre o envelope dos soltos, Esc/Select All; undo/redo restauram o intervalo (`Doc.ranges`); sprite nascido de um recorte leva a paleta em RGBA; gestos conduzidos `PH2D_PIXEL_SMOKE_GESTO=copia|cels`.
- **P6k** — ronda 3: P5 (`play.rs` reescrito: subtags, Play Once/All, velocidades, Rewind), Loop Section, janela Preview flutuante (crate NOVA), paleta indexada com remap pendente e «Remap palette», arrasto da paleta (voo + barra + queda por lugar do rato), elipse > 150×96 e Modify tecto 100 contra o Trial; recusas MEDIDAS (Fast 36→36, RotSprite 1 554→1 554) em `ferramentas/oraculo_rotacao.md`.
- **P6l** — ver §4.

## 3. Superfície de colisão CUMULATIVA (P6h ∪ P6i ∪ P6j ∪ P6k ∪ P6l)

### 3.1 Números que SOMAM entre linhas — o integrador RECONTA (nunca escolhe)

| Onde | Aqui | Nota |
|---|---|---|
| `MODULE_TRUTHS: [(NodeId, ModuleTruth); N]` em `crates/ph2d-editor-core/src/screens/hero/menu_bar.rs:231` | **23 → 24** | +1 linha `TOPBAR_PIXEL_PREVIEW` (l. 246). Outra linha que acrescente módulo ⇒ o N soma; reconte por `git grep`. |
| Linhas do menu Window: `MENU_BAR_WINDOW_ROWS` em `screens/hero/menu_tables.rs` | **+1** (`menu_row(ids::TOPBAR_PIXEL_PREVIEW, "chrome.menu.pixel_preview")`, l. 253) | reconte o tamanho do array. |
| `topbar/mod.rs` (registo, l. 48) · `chrome/pixel_toggle.rs` | +1 id registado; o mapa mapeia as DUAS linhas (Pixel, Pixel Preview) | só a Preview alterna a janela da Preview. |
| Crate NOVA `ph2d-panel-pixel-preview` | `ph2d-panel-registry-init` (`Cargo.toml`, `lib.rs`), `shells/desktop/Cargo.toml`, `Cargo.toml` (workspace), painel-sync/gate `…the_shell_drives_is_in_its_registry` | **conflito provável** com outras linhas que acrescentam crates/painéis. |
| Crate NOVA `shells/pixel-scenes` (`ph2d-host-pixel-scenes`) | workspace `Cargo.toml` (`"shells/pixel-scenes"`) | idem; precedente `game-scenes`. |
| `Cargo.lock` | só 2 `+name` novos: `ph2d-host-pixel-scenes`, `ph2d-panel-pixel-preview` (dep nova `ph2d-panel-pixel-preview → ph2d-pixel-render`; nenhum pacote externo) | regenere/reconte pós-merge. |
| Catraca `crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs:222` | `ph2d-app-pixel` tecto **13 424**, medido ~12 900 nesta árvore (**não subiu**) | tecto SOMA entre linhas; cura por corte. |
| Smoke `PH2D_PIXEL_SMOKE` `max_level` | **17** (cenas 16 e 17 novas; 15 era a última no main) | confira que nenhuma outra linha usa o 16/17 do Pixel. |
| i18n `crates/ph2d-i18n/src/pixel.rs`, `chrome_menus.rs` | ver 3.3 | |
| `ALTURA_DE_ABERTURA` | **intocada** | |
| `PROJECT_SCHEMA` / contratos §6 | **não tocados** (nenhum ficheiro `schema`/`contract_surface` no diff) | |

### 3.2 Caminhos tocados fora do módulo (foundational, ADITIVO salvo marca)

- `ph2d-editor-core`: `interaction/pixel_timeline_surface.rs` NOVO + `interaction/{dispatch,pointer_down,pointer_move,pointer_up}.rs` (⚠️ `pointer_down.rs` a 699/700 linhas — outra linha que o aumente quebra o gate de LOC); `pixel_panel/{mod,timeline}.rs` (**`RangeMove` ganhou o campo `copy`; `CelsMove` é variante nova com `beyond`** — todo `match` exaustivo/construtor sobre `PixelRequest` tem de os tratar); `layer_row` (`LayerRowPainter.continuous`); `paint_flight.rs` NOVO, `doc_surface.rs`, `lib.rs` (+2 linhas); `PointerLook::{Grab,Grabbing,ResizeEw}` e `WidgetStore::pixel_timeline_look` (`board_view::pointer_look`); `ids/chrome/pixel.rs` (`PIXEL_PREVIEW_PANEL`, `TOPBAR_PIXEL_PREVIEW`, `PIXEL_PALETTE_MENU` re-exportado); `screens/hero/paint.rs`.
- **`ph2d-app-kanban`** (`metrics.rs`, `paint.rs`, `paint_card.rs`, de `a688bf172c`): perdeu as constantes do voo, que agora vêm de `ph2d_editor_core::paint_flight` (`FLIGHT_TILT_DEG`, `SHADOW_FLIGHT_DARK/NEAR`). Linha do Kanban que mexa nestes três ficheiros ⇒ conflito; o comportamento do Kanban é idêntico.
- `ph2d-pixel-render`: novo `Checker` partilhado (extraído de `SpriteImage`; `lib.rs`, `view.rs`).
- Shell (`shells/desktop`): `Cargo.toml` (+3), `input_dispatch/despacho_pixel.rs`, tradução de cursor (+3 linhas).
- `ph2d-pixel-core/src/{play,palette,frames}.rs`; `ph2d-pixel-io` sem mudança relevante.
- `ph2d-app-pixel/src/{intervalo,paleta,paleta_remap,quadros,edicao,selecao,object_add,smoke_intervalo,smoke,state,lib}.rs`.
- `ph2d-panel-pixel-timeline/src/{gestos,paint_grid,paint_props,paint_voo,state,seam,ids}.rs`; `ph2d-panel-pixel-tools/src/{ids,paint,paint_paleta,paint_paleta_voo,paint_paleta_menu (NOVO),seam,state}.rs`; `ph2d-panel-pixel-preview` (crate inteira, NOVA).
- Gates/testes: `ph2d-panel-registry-init/tests/it/{quantas_entradas_tem_cada_painel,paineis_armados,the_window_menu_reaches_every_module}.rs`; `shells/pixel-scenes/tests/it/{intervalo,sobreviventes,quadros,paleta,tocar,preview,comum,main}.rs` + fixturas (`oracle_trial_*.json`); `ph2d-pixel-core/tests/it/{fast_tie,…}` + fixturas.
- `docs/PixelArt/` (ferramentas `trial_*.py`, `oraculo_*.md`, fotos `smoke_P6*`); emendas P6h–P6k no ADR-0196.

### 3.3 i18n (P6l; as das ondas anteriores estão nos handoffs de onda)

- Acrescentadas: `chrome.menu.pixel_preview` («Pixel Preview», `chrome_menus.rs`); `panel.pixel.palette_selected` («Palette · {n} selected»), `panel.pixel.palette_menu_tip`, `panel.pixel.presets_menu_tip`, `panel.pixel.sort_by` («Sort by {key}») em `pixel.rs`.
- **Removidas (mortas)**: `panel.pixel.palette`, `panel.pixel.selected`, `panel.pixel.sort`, `panel.pixel.presets`. Outra linha que as use reconta.

## 4. P6l em detalhe

- **(a) Preview**: pegas nos DOIS cantos de baixo (ids `RESIZE`/`RESIZE_BL` em `ph2d-panel-pixel-preview/src/ids.rs`; `clamp_panel_rect` com `panel_resize_delta`); fundo xadrez por baixo do quadro via `ph2d_pixel_render::Checker` (partilhado com `SpriteImage`); linha «Pixel Preview» no menu Window (`ids::TOPBAR_PIXEL_PREVIEW`, `pixel_toggle.rs` mapeia as duas linhas, `MODULE_TRUTHS` 24).
- **(b) Paleta como no Aseprite** (decisão do dono 01/11; a 1040 px o painel PIXEL media ~2,3 ecrãs e os botões da paleta ficavam ~500 px abaixo da dobra): cabeçalho «Palette · n selected» com dois menus de ícone (≡ `PALETTE_MENU` = `ph2d_editor_core::ids::PIXEL_PALETTE_MENU`; `PRESETS_MENU`) cujas linhas REUSAM os mesmos ids de acção (despacho inalterado; clicar numa linha fecha os dois menus — `paint_paleta_menu.rs`, `e_linha`). Sob a grelha só ficam Remap (pendente), Edit color, Palette size, agora ACIMA do seletor. Ficheiro novo: `crates/ph2d-panel-pixel-tools/src/paint_paleta_menu.rs`.
- **(c) Arrasto na linha do tempo**: barra do acento (`StrokeToken::Heavy`) à esquerda da 1.ª coluna do vão (`paint_voo.rs`: `vao → Option<Rect>`, `marca`; getter `drop_marker()`).
- **(d) Cura do teste dependente da ordem `a_altura_de_abertura_de_um_painel_so_encolhe`**: mecanismo = `PANEL_REGISTRY` global do processo; cada `ErasedPanel` guarda memória de `ColunaDoPainel` que só cresce; um vizinho que pintava com `abre_tudo` subia o inspector para 1659 (vs 1488) e o `painter_layers` para 1643 (vs 1593) só quando corria em concorrência. Cura: todo censo de `crates/ph2d-panel-registry-init/tests/it/quantas_entradas_tem_cada_painel.rs` usa `registo_privado` (registo `thread_local` por teste; libtest = 1 fio por teste, nextest = 1 processo por teste); tectos inalterados; o módulo em paralelo verde 3/3; comentário de `paineis_armados.rs` corrigido. ⚠️ Se outra linha mexer nas chamadas `with_registry` desse ficheiro, mantê-las encaminhadas por `registo_privado`.
- **(e) Ganchos de foto**: `PH2D_PIXEL_SMOKE_GESTO=menu` (cena 3) e `=preview` (cena 17) em `smoke_intervalo.rs`.
- **(f) Fotos** `smoke_P6l_*` (cenas 3, 9, 16, 17; fábrica e layout do dono; 1930×1040). `layout.txt` do dono: sha256 `bfb4c314…` igual antes/depois. A foto da cena 9 (aberto P6k §5.5) está refeita.

## 5. Prova (por nome, verde, 01/11)

- `ph2d-panel-pixel-tools`: `seam_pixel` 31 · `every_` 5. `ph2d-panel-pixel-timeline`: `seam` 20. `ph2d-panel-pixel-preview`: `seam` 8 · `every_` 3.
- `ph2d-host-pixel-scenes`: `paleta` 21 · `preview` 4 · `tocar` 5. `ph2d-panel-registry-init`: `window_menu` 4 · `quantas_entradas_tem_cada_painel::` 15 · `nenhum_rotulo_do_app_pinta_nada` 13. `ph2d-editor-core`: `menu_bar` 19.
- `clippy -D warnings` limpo nas 8 crates tocadas (`ph2d-pixel-render`, `-panel-pixel-preview`, `-panel-pixel-timeline`, `-panel-pixel-tools`, `-app-pixel`, `-panel-registry-init`, `-editor-core`, `-i18n`); `fmt` limpo.
- **Mutação P6l**: 18 mutantes + A18/B9/B10 (as só-pintura das ondas anteriores) → **zero sobreviventes**. Testes acrescentados: `mid_ctrl_drag_the_gap_and_the_bar_open_where_the_copies_land`, `the_tag_name_starts_after_the_left_handle`, `the_flying_block_carries_no_dots_on_group_rows`, `o_xadrez_pinta_se_por_baixo_do_quadro_a_escala_e_ao_canto_da_imagem`, `the_pixel_preview_row_exists_and_toggles_only_the_preview_window`, `as_pegas_dos_cantos_redimensionam_a_preview`; getters ocultos `marcas_pintadas()`, `xadrez()`, `drop_marker()`.
- Binário de smoke construído: `cargo build -p ph2d-host-desktop --profile smoke` (2.ª corrida `Finished` em 0,19 s).
- **NÃO corridos na linha** (ordem do dono: lote na integração; suíte completa só lá): suíte completa / `nextest-impacted`, `cargo machete` (deps novas desde a P6i), auditoria, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`.

## 6. O que um merge pode partir

1. `MODULE_TRUTHS` / linhas do menu Window: contagem somada por outra linha ⇒ erro de tamanho do array ou gate `window_menu`.
2. Registo de painéis: `ph2d-panel-registry-init` (`lib.rs`, `Cargo.toml`) e o gate `…the_shell_drives_is_in_its_registry` com a crate nova.
3. `match` exaustivo sobre `PixelRequest` (`RangeMove.copy`, `CelsMove`) e qualquer construtor.
4. Kanban: constantes do voo agora em `ph2d_editor_core::paint_flight`.
5. `pointer_down.rs` 699/700; catraca `ph2d-app-pixel` (tecto soma entre linhas); `Cargo.lock`.
6. Censos de `quantas_entradas_tem_cada_painel.rs`: tectos por painel de outra linha entram no mesmo ficheiro — usar `registo_privado`.
7. `PH2D_PIXEL_SMOKE` max_level 17 e o censo de cenas.

## 7. Smoke (para o Enio, depois da integração)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && cargo build -p ph2d-host-desktop --profile smoke && PH2D_PIXEL_SMOKE=3 ./target/smoke/ph2d-host-desktop`. Cena 3: na paleta, abra o menu «≡» e o das predefinições; arraste uma amostra (voa, barra de cor marca onde cai; Ctrl copia; Esc cancela). Errado: o vão salta, ou os botões da paleta ficam fora da tela.
2. `PH2D_PIXEL_SMOKE=17` (mesmo comando): abra o menu Window → «Pixel Preview»; arraste os dois cantos de baixo da janela; veja o xadrez atrás da imagem. Errado: a janela só estica por um canto, ou o xadrez cobre a imagem.
3. `PH2D_PIXEL_SMOKE=16`: arraste quadros da linha do tempo; uma barra marca onde caem. Errado: a barra some ou aparece fora do vão.

## 8. ABERTO (na ordem)

1. Fecho da linha / pós-integração: `cargo machete`, auditoria, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`.
2. RotSprite e empates do Fast: recusas MEDIDAS (`docs/PixelArt/ferramentas/oraculo_rotacao.md`) — só reabrir com medição nova.
3. Outras listas reordenáveis por arrasto do app devem herdar voo + barra + queda por lugar do rato (memória `feedback_every_drag_reorder_flies_like_the_kanban_card.md`).
4. Não medido no Trial (P6j §5): cópia com Ctrl numa camada contínua (nós fazemos cópias próprias); bloco de cels largado fora da grelha; Paste as New Sprite em si.
5. Resto do §10 do handoff de integração 28/10 ainda por abrir: `LengthDisplay` na etiqueta do Pixel, gravar com forma viva a assenta (P6d §6), paletas Tint/Shade/Tone, Spectrum, rodas, `.ase` import/export (P9), OKLCH/cycling (P13), GPU dos 19 modos de mistura, P7–P10. Confira o código antes de acreditar numa ausência (partes já feitas nas P6h–P6l).

## 9. Entrada proposta para o CLAUDE.md §5.1 (≤ 700 bytes; o integrador troca a linha)

```
- **Pixel Art** — editor de pixel art autocontido, no idioma do Aseprite: camadas, quadros e linha do tempo (intervalos, tags aninhadas, Preview flutuante, Play Once/All), selecção, moldura afim, formas do Vector, paleta indexada com remap, `.msk` (crates `ph2d-pixel-*`, `ph2d-app-pixel`, painéis `ph2d-panel-pixel-*`). Smoke `PH2D_PIXEL_SMOKE=1..17`. Último: [P6h–P6l, para o main 01/11](docs/PixelArt/handoffs/HANDOFF_INTEGRACAO_line_PixelArt_2026-11-01_P6h_a_P6l.md) · [docs](docs/PixelArt/README.md)
```
