# HANDOFF DE CONTINUAÇÃO — linha Pixel Art, onda P0b (Draw, coluna, roda) — 2026-10-25

> Leitor: a próxima LLM que assume a linha numa janela nova. A onda P0b está **fechada e com o smoke do
> dono APROVADO (25/10)**; a **linha continua** (`/pd-linha-fechar` NÃO foi corrido). Nada integrado,
> nada pushado. Lei do módulo: [ADR-0196](../../architecture/decisions/0196-o-pixel-art-e-um-modulo-proprio-com-documento-inteiro-e-desfazer-por-sprite.md)
> (emenda 9, 25/10) · [plano](../02_plano_modulo.md) · [handoff P0](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md) (§§1–8 = a P0; §9 = o report do dono; §10 = esta onda em resumo).

## 1. Identidade

- Entrada: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && pwd && git branch --show-current` (tem de dizer `line/PixelArt`).
- Branch `line/PixelArt`, worktree `Worktrees/line-PixelArt`, merge-base `c3662cd8e4`.
- **23 commits** na linha (`git log --oneline c3662cd8e4..HEAD`); HEAD `b246313c6c`.
- Desta onda: `5ed4562176` (a cura: Draw no topo, coluna à esquerda, zoom sem avisos) · `afd19d87fd` (handoff §10 + ADR-0196 emenda 9) · `b246313c6c` (2.ª volta do smoke: o vetor ainda cobria o sprite com layout Vector gravado). Antes (docs): `b6ee59351e`, `6304c7144a`.
- `git diff --stat b6ee59351e..HEAD`: 38 ficheiros, +917/−161.

## 2. Estado do módulo

Igual ao do [handoff P0 §2](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md) (núcleo `ph2d-pixel-core`, `-render`, `-io`, `ph2d-app-pixel`, `ph2d-panel-pixel-tools`, diálogo New Sprite, formato `PH2DPIXEL` 1.0, `PROJECT_SCHEMA` 193→194), com **três mudanças**:
- O modo do Pixel Sprite é **Draw** (par D6 `(PixelSprite, Draw)`), não Edit; o seletor Object · Draw do topo aparece como no Imagem/Vetor.
- As ferramentas (Pencil/Eraser/Eyedropper/Hand) vivem numa **coluna à esquerda** (padrão do Painter); o painel `pixel_tools` à direita ficou só com cores e paleta.
- A roda não empilha avisos: o zoom lê-se no rodapé; `input::Roda` acumula (rato = um degrau por linha; trackpad = degrau por soma de píxeis).

## 3. O que a P0b fez

Causa comum dos defeitos 1 e 2 do report (handoff P0 §9), medida no código: com o sprite em Draw a ferramenta activa do registo continuava `vector`, que publicava as SUAS `ToolBars`; a fila de cima só mostra o seletor de modo quando não há barras de módulo (ou são a coluna do pintor). Um bloco só:

1. `mode.rs` declara `(PixelSprite, Draw)`; D6 (`every_object_mode_has_a_composed_family`) actualizado.
2. `ToolBars.pixel_rail` + `ToolBars::is_family_column()`: a fila guarda o seletor para colunas de família; o `bars_publish` do vetor só limpa barras que NÃO são de família; `module_hint` (rodapé) fala também com `pixel_rail`.
3. **O vetor larga a mão:** `WidgetStore::family_owns_canvas` (posto por `ph2d_app_pixel::panel::frame` com o sprite em Draw OU seleccionado em Object), lido pelos DOIS sítios que decidem `vector_active`: `fase_vector_scale` e `ph2d_app_vec::vector_bridge::dispatch` (este também abre o painel Vector).
   - **2.ª volta do smoke:** só o primeiro estava curado. Com o layout *Vector* gravado em `~/.ph2d/layout.txt`, a ponte continuava a publicar barras e a abrir o painel; a foto de fábrica (HOME isolado) não o via. Cura em `b246313c6c`; foto com o HOME do dono: `smoke_P0b2_2026-10-25.png`.
   - ⛔ **Lição:** fotografar SEMPRE também com `HOME=/home/enio`, guardando antes uma cópia de `~/.ph2d/layout.txt` (o app reescreve-o).
4. **A coluna:** `ph2d_panel_pixel_tools::column::{column, hint}` (ids do painel; o clique segue pelo `seam` dele, também com o painel fechado); `ph2d_app_pixel::panel::publish_bars` publica/limpa; a shell compõe em `fase_pixel_panel`. Ícones `VectorPencil`, `Erase`, `Eyedropper`, `Pan`.
5. **Amostras:** `PIXEL_FG`/`PIXEL_BG` mudaram-se para `ph2d_editor_core::ids` (o painel reexporta); `register_picker_swatch` ⇒ Down abre o seletor partilhado; `panel::sync_swatch` lê de volta a escolha do artista. Fecha o aberto «clicar nas amostras não abre seletor» da P0 §6.
6. **Zoom:** sem toast. `Roda`: `Lines` = um degrau por linha; `Pixels` = um degrau quando os píxeis somados dão o factor dele por `Camera2d::wheel_px_for`; inverter o sentido ou bater no fim da escada esquece o resto. `PixelApp::zoom` (lido em `frame`) vai para o `readout` do rodapé.

## 4. Smoke do dono — APROVADO em 25/10

Fotos: [`smoke_P0b_2026-10-25.png`](smoke_P0b_2026-10-25.png) (fábrica) · [`smoke_P0b2_2026-10-25.png`](smoke_P0b2_2026-10-25.png) (HOME do dono).

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-PixelArt && PH2D_PIXEL_SMOKE=1 ./target/smoke/ph2d-host-desktop`
2. No topo aparece **Draw Mode** (com Object ao lado). Na coluna da esquerda: Pencil, Eraser, Eyedropper, Hand, as amostras preta e branca e o botão de trocar.
3. O painel **Pixel** à direita mostra só as cores e a paleta. No rodapé: a dica da ferramenta e «Zoom 1600 %».
4. Clique no quadrado preto (frente): abre o seletor de cor; escolher uma cor muda a amostra e o lápis pinta com ela.
5. Gire a roda do rato: o zoom anda em degraus nítidos, sem mensagens na tela; no trackpad o gesto é suave (um gesto não pula dezenas de degraus).
6. Troque **Draw Mode ▸ Object**: voltam as ferramentas do Vetor e o seletor continua lá, com o sprite seleccionado. `Tab` volta ao Draw.

Deu errado se: faltar o seletor Object · Draw, aparecerem ferramentas do Vetor sobre o sprite em Draw, surgirem avisos «Zoom N %», o quadrado preto não abrir o seletor, ou o zoom passar de 1000 % com um só gesto.

## 5. Superfície de colisão NOVA desta onda

Soma-se à lista de [P0 §3](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md) (contadores ECS/schema/MODULE_TRUTHS/NodeId 852/z=74,176 não mudaram nesta onda).

- `ph2d-editor-core`: `interaction/tool_bars.rs` (`ToolBars.pixel_rail` + `is_family_column()` — **campo novo num struct partilhado**; todo literal de `ToolBars` de outra linha precisa dele); `interaction/state/tool_bars_ops.rs` (`ToolBarsSlot.family_owns_canvas` + `WidgetStore::{set_,}family_owns_canvas`); `screens/hero/tool_bar.rs` (condição dos pulldowns de área) e `bottom_hud.rs::module_hint`; `ids/chrome/pixel.rs` (`PIXEL_FG`/`PIXEL_BG` movidos para cá); testes em `tool_bar_tests.rs`.
- `ph2d-panel-vector/src/bars.rs`: campo novo no literal de `ToolBars`.
- `ph2d-app-vec`: `bars_publish.rs` (só limpa barras que não são de família) e `vector_bridge.rs` (lê `family_owns_canvas`).
- `ph2d-panel-pixel-tools`: `column.rs` novo; `ids.rs`, `paint.rs`, `seam.rs`, `lib.rs`. `ph2d-i18n/src/pixel.rs`. `ph2d-panel-registry-init/tests/it/` (teste novo do ponteiro real + `main.rs`).
- `ph2d-app-pixel`: `input.rs` (`Roda`), `mode.rs`, `panel.rs`, `state.rs`, `compose.rs`, `frame.rs`.
- Shell: `render_loop/fase_vector_scale.rs`, `render_loop/fase_pixel_panel.rs`, `input_dispatch/despacho_pixel.rs`.
- Merge pode partir: outra linha que mexa em `tool_bar.rs` (condição `module.is_none_or(painter_rail)`) ou crie `ToolBars` por literal; outra que decida `vector_active` em terceiro sítio (a lição aqui foi que eram DOIS).

## 6. Prova de fecho da onda

Testes por nome, todos verdes:
- `a_trackpad_gesture_zooms_like_the_viewport_and_not_one_step_per_event` · `a_lone_trackpad_event_keeps_its_remainder_and_a_reversal_forgets_it` · `the_wheel_never_pushes_a_toast_and_the_footer_reads_the_level` · `the_wheel_walks_the_integer_ladder_around_the_cursor`
- `o_seletor_oferece_object_e_draw_e_a_coluna_em_draw_e_a_do_pixel` · `every_tool_and_swatch_of_the_pixel_column_answers_a_real_pointer` · `a_coluna_do_pixel_guarda_o_seletor_de_modo_na_fila` (com controlo) · `the_idle_vector_clears_its_bars_and_never_the_painters_column` · `a_coluna_tem_as_quatro_ferramentas_e_as_duas_cores` · `cada_ferramenta_da_coluna_chega_ao_barramento`
- `every_object_mode_has_a_composed_family` · `architecture_no_authoring_tool_shares_the_canvas` · `the_shell_only_shrinks` · os 26 `pixel` da shell.
- `cargo fmt --all` limpo.
- clippy -D warnings por crate: verde nas nove crates tocadas (ph2d-app-pixel, ph2d-panel-pixel-tools, ph2d-editor-core, ph2d-app-vec, ph2d-panel-vector, ph2d-panel-registry-init, ph2d-app-registry-init, ph2d-i18n, ph2d-host-desktop), `--all-targets`, 25/10

## 7. ABERTO

- **Mutação** das leis novas (`input::Roda`, `ToolBars::is_family_column`, `family_owns_canvas` nos dois sítios) e das do núcleo ([P0 §6](HANDOFF_CONTINUACAO_line_PixelArt_2026-10-25_P0.md): inverso do Patch, canonicalização da linha, um traço = um passo), no fecho da linha.
- O trackpad real só o dono prova (a tela virtual não tem gesto) — o smoke dele foi aprovado.
- Licença do Aseprite (plano §7): pergunta ao dono continua aberta.
- Resto do P0 §6: overlay por cima do mundo (ordem de desenho = P14); conversão RGB→Gray Rec.601 provisória (P2); sem teste de shell do caminho pick→OK→objecto; sem marca de «projecto por gravar».
- Fecho da linha pendente: `/pd-linha-fechar`, `bash scripts/agent-loop-profile.sh`, `rm -rf target/*/incremental`; a suíte completa é do integrador.

## 8. Próximo item

O **dono escolhe** (pergunte antes): **P1** (desenho: pincel, pixel-perfect, Line/Curve/Rect/Ellipse/Contour/Polygon/Spray/Bucket, simetria, Tiled) ∥ **P2** (cor, paleta, inks, Indexed completo — inclui a conversão RGB→Gray contra o oráculo) ∥ **P3** (camadas, 19 blend modes).

Primeiro passo de qualquer onda: extrair os vetores de teste do MVP (`docs/Pixel Art/app/tests/*.test.js`, do **primário** — não versionado) para JSON, como `docs/PixelArt/ferramentas/extrai_linhas_mvp.mjs` fez, e só depois escrever o código.
