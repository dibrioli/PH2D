# HANDOFF DE INTEGRAÇÃO — line/ScriptEditor: acabamento 10/11 (tabelas da casa, caixa de marcar, molduras, divisor, pesos da maquete 14)

> Leitor: o agente integrador. Antecessor: [Onda B + plano 14 Q1–Q4](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-09_PLANO14_Q1_A_Q4.md)
> (já no main desde `bc4a6ce930`, a base desta linha). Mecanismo e diário:
> [handoff de continuação do acabamento](HANDOFF_line_ScriptEditor_2026-11-10_ACABAMENTO.md) (§A e o smoke de lá estão
> **superados** por este: a pílula saiu). Decisões do dono: [00_decisoes_do_dono.md](../00_decisoes_do_dono.md).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor` · branch `line/ScriptEditor` · HEAD de código `40df6a8502`
  (o commit deste handoff vem a seguir: só docs).
- Merge-base com o `main` = `bc4a6ce930`. **55 commits** (`git rev-list --count bc4a6ce930..HEAD`), **129 ficheiros, +4607 −1052**
  (`git diff --shortstat bc4a6ce930..HEAD`).
- Smoke do dono 10/11: as rondas até `e082d45b6d` **APROVADAS («Smoke OK»)**. As rondas seguintes (divisor da coluna, pesos da
  maquete 14, medidas, tamanho da letra) — **smoke do dono PENDENTE**.

## 2. Superfície de colisão

`git diff --name-only bc4a6ce930..HEAD`, por crate (**A** = ficheiro novo):

| Crate / pasta | Ficheiros |
|---|---|
| `ph2d-panel-code` (47) | `Cargo.toml` · `src/`: **A** `peso.rs`, `find_state.rs`; M `splitter.rs`, `bancada/{grid,mod,outline,phrase,popover}`, `cards`, `chrome`, `console`, `console/problems`, `debug_card/{boxes,head,live,live_marks,mod,sheet}`, `debug_marks`, `drop_mark`, `help/{paint,view}`, `lib`, `line_cache`, `metrics`, `options_card`, `paint`, `rename_box`, `script_list`, `status`, `table_text` · `tests/it/`: **A** `peso_das_pecas.rs`, `peso_na_placa.rs`, `bancada/acabamento.rs`; M `bancada{,/grelha,/puras}`, `clique`, `estado`, `main`, `medicoes_do_fecho`, `opcoes_do_script`, `pagina`, `problemas`, `quadro_do_depurador`, `valores_ao_vivo` |
| `ph2d-app-code` (17) | `src/`: **A** `column.rs`; M `console`, `doc`, `docs`, `find`, `folds`, `lib`, `note`, `play_problems`, `problem_list`, `strict`, `surface`, `tab_menu` (vários só encolhem: código movido) · `tests/it/`: **A** `divisor.rs`; M `barra_da_aba`, `main`, `onda_b` |
| `ph2d-code-assist` (4) | `src/`: **A** `folds.rs`, `play_problems.rs`, `strict.rs`; M `lib.rs` (`ClosedFolds`/`PlayProblem`/strict vieram do app-code) |
| `ph2d-code-bancada` (3) | `src/{act,lib}.rs`, `tests/it/acto.rs` |
| `ph2d-text` (4, foundational) | `src/`: **A** `peso_exato.rs`, `peso_exato_tests.rs`; M `lib.rs`, `system.rs` |
| `ph2d-editor-core` (6, foundational) | `src/`: **A** `paint_text_weight.rs`; M `paint.rs`, `paint_text.rs`, `widget/{mod,toggle_classic}.rs` · `tests/it/architecture_the_shell_only_shrinks.rs` (1 linha, abaixo) |
| `ph2d-tokens` (4, foundational) | `src/{lib,state_layer}.rs`, `tests/it/{main,the_hover_layer_lightens_a_dark_surface}.rs` |
| `ph2d-ui-table` (8, foundational) | `src/`: **A** `measure.rs`, `look.rs`; M `lib.rs`, `marks.rs`, `paint.rs` · `tests/it/`: **A** `pesos.rs`, `reserva_e_fios.rs`; M `main.rs` |
| **`Cargo.lock`** | **TOCADO:** `ph2d-panel-code` ganha dev-deps `ph2d-gpu` e `ph2d-render` (teste `peso_na_placa`: tinta medida no píxel). `no_dependency_climbs` e `cycle_prevention` **passaram**. Merge de lockfile: refaça por `cargo` se conflitar. |
| Hook / scripts / roteador | `.claude/hooks/prazo-do-agente.sh`, `scripts/processo.prova.d/I.sh`, `docs/Components/ferramentas/fotografa_cena.sh`, `CLAUDE.md` (só a linha §5.1 do módulo: o link do handoff) |
| **`ph2d-code-scene-arena` (crate NOVA, 2)** | **A** `Cargo.toml`, `src/lib.rs` (a cena 15 e o `STEPS_15`, vindos de `ph2d-code-scenes`/`ph2d-code-smoke`); consumidores `ph2d-app-code` (`smoke_drive.rs`, `Cargo.toml`) e `ph2d-app-components` (`code_smoke.rs`, `Cargo.toml`); linha nova em `TETOS_DAS_CRATES_DA_FAMILIA`; `Cargo.lock` tocado |
| Ronda «stem darkening» (`07d722387f`) | `ph2d-tokens/src/{typography,lib}.rs` (variante 4.ª de `TextRendering`, `ALL` = 4, campo `stem_darkening`, `stem_darkening_px`) · `ph2d-text`: nada · `ph2d-editor-core`: `published.rs` (+**A** `published_embolden_tests.rs`), `paint.rs`, `paint_text.rs` (692/700), `paint_text_rotated.rs`, `ids/menus_settings.rs`, `screens/hero/{chrome/settings_text,menu_row_mark,menu_tables_settings,pre_populate}.rs` · `ph2d-i18n/src/chrome_menus.rs` (`chrome.menu.crisp_stem_darkening`) · `ph2d-vector/src/lib.rs` (re-export `FontEmbolden`, `Diagonal2`; o contrato congelado vive em `ph2d-vector-doc`, intocado) · `ph2d-panel-code/src/glyphs.rs`, `tests/it/nitidez.rs`. ⚠️ Quem faça `match` exaustivo sobre `TextRendering` noutra linha parte aqui. |
| **Ronda 11/11** (`89ac2897e9..c67465ff6e`) | `ph2d-panel-code`: **A** `src/tab_bar.rs`, `src/tab_bar/menu.rs`, `tests/it/{submenu,fundo_real}.rs`; M `mono_label` (`paint` → `paint_on(.., bg, ..)`, ⚠️ quem chame `mono_label::paint` parte), `palette` (`link_ink`, `#[derive(Clone)]`), `code_text` (`choices`), `help/paint`, `script_list`, `bancada/{grid,outline}`, `lib`, `tests/it/main` · `ph2d-app-code`: `tab_menu` (a barra saiu; `Bar`/`BarPainted` = alias genéricos; `menu_items`/`menu_titles`/`ROW_TEXT_RENDERING`), `commands` (os 4 modos só na paleta), `surface`, `smoke_drive`, `tests/it/{barra_da_aba,divisor,ajuda,arrastar,bancada,main,onda_b}` · **`ph2d-editor-core` (foundational, ADITIVO):** `doc_surface.rs` **`PointerInput.primary_held`** (⚠️ campo novo num literal: quem construa `PointerInput` noutra linha não compila — 17 sítios preenchidos aqui), `documents.rs`/`documents_surface.rs` (`note_button`, `primary_held()`), `widget/lista_flutuante.rs` (`Superficie::fundo`), `screens/hero/surface_input.rs`, `published_embolden_tests.rs` · `shells/desktop/src/input_dispatch.rs` (+2 linhas: `note_button` no topo de `pointer_button`) · `shells/game-scenes/tests/it/` (5 ficheiros, o campo) · `ph2d-i18n/src/code_bancada.rs` (5 chaves `code.menu.text*`) |
| Docs / fotos | 2 handoffs em `docs/CodeEditor/handoffs/`; fotos em `docs/CodeEditor/ferramentas/smoke/` (+`cena13/`); `cena14/fotos_q3.sh` (`ARRASTA=`, `DUPLO=`) |

**Provas de contrato:**

- §6 (Nodes, Tools, Vector): `git diff --stat bc4a6ce930..HEAD -- crates/ph2d-nodegraph crates/ph2d-tool-api crates/ph2d-vector` = **vazio**.
- `PROJECT_SCHEMA`: `git diff bc4a6ce930..HEAD -- . ':!*.md' | grep -c PROJECT_SCHEMA` = **0**. Nenhum registo nem ADR; uma crate nova (`ph2d-code-scene-arena`, §8).

**Foundational tocado — ADITIVO** (consumidores dos removidos = 0 em `crates/` e `shells/`):

- `ph2d-text`: escopo **`com_peso_exato`** (guarda thread-local). Dentro dele `effective_weight` **não** soma o reforço do preset de
  nitidez; o desvio Light/Normal/Strong do estilo continua a valer. A chave do cache de layout é o peso efectivo (nada vaza). Repõe ao
  sair, aninhado e em pânico (`peso_exato_tests.rs`). Fora do escopo: byte-idêntico.
- `ph2d-editor-core`: pintores com peso (`paint_text_in_weight` e irmãos, `paint_text_weight.rs`); `title_tracked_width` +
  `TITLE_TRACKED_WEIGHT` (mesmo SEMI_BOLD de antes: Kanban/cartões inalterados). ⚠️ `paint_text.rs` **693 / 700**.
  `paint_switch`/`switch_inks` REMOVIDOS (sem outro consumidor; quem os use parte aqui).
- `ph2d-tokens`: `HOVER_LAYER_ALPHA` (Material 3, 8 %), usado pelo hover da linha de opções.
- `ph2d-ui-table`: `Metrics::twist`/`rules_bottom`; `Column::snug()` (largura fixa, sem puxador); `Look::House` (omissão, pesos da casa)
  / `Look::Exact` (aba de scripts, via `peso::metricas()`); medidas em `measure.rs`. Consumidores: `ph2d-panel-code` e `ph2d-code-debugcard`.
- **Catraca** `crates/ph2d-editor-core/tests/it/architecture_the_shell_only_shrinks.rs`: `ph2d-code-assist` **3 925 → 4 188** (regra
  «a isenção viaja com o código», mesma forma de `8281ac0a43`). ⚠️ **PARA O INTEGRADOR:** o tecto de fonte `ph2d-app-code` 26 628 **NÃO**
  foi baixado (baixar 263 ficaria abaixo do contado) e uma lente de auditoria notou que o teto total da família cresceu: **o integrador
  decide no recontar**.

**O que um merge pode partir:** quem importe `paint_switch`/`switch_inks`/`switch_knob`; quem use `FindState`, `ClosedFolds`,
`PlayProblem` ou o «strict» por `ph2d-app-code` (agora panel/assist); `PH2D_CODE_SMOKE=1..14` (`CENAS`, inalterado); os tectos de LOC
(§5); `paint_bench`/`paint_popover`/`ConsoleView::paint` (levam `&mut CodeView`); `Cargo.lock`. ⚠️ **Hook `prazo-do-agente.sh`:** o que
corre é o do primário; a cura só vale depois da integração.

## 3. O que a linha entrega

| Item | O quê |
|---|---|
| A | Cartão «Script options»: liga/desliga = **caixa de marcar do cartão** (dono 10/11: «o padrão do app é checkbox, não toggle»); hover da linha clareia. Pílula e disco saem. |
| B | Tabelas da casa: números na mono do código; fios param na última linha inteira (`rules_bottom`); reserva do ▸ só se alguma linha expande (`Metrics::twist`). |
| C | Vec2 da grelha aceita a vírgula decimal (`parse_vec2`); `FOTO_TECLAS` recusa alto; hook de subagente (`--ignored`, `bash -c "cargo …"`, `cargo t`); 12 casos em `processo.prova.d/I.sh`. |
| D | Molduras de valor de volta nos temas modernos: `cards::value_outline` (sempre traça; `FRAME-RAW-OK`). |
| E | Rodada do dono: faixa da frase = `ROW_H_PX` (22 px); rótulos e teclas dos botões pela `folga_do_texto` / `top_on_baseline`; «In this script» em `outline_entry_h()` (19 px). |
| F | Auditoria: `parse_vec2` com vírgula à cabeça; lei única de centrar. |
| G | **Divisor da coluna esquerda** (dono 10/11; `splitter.rs` + `ph2d-app-code/src/column.rs`): o vão arrasta-se, cursor `ResizeEw`. **Mínimo** = o que cabe das linhas de Scripts + entradas do outline + títulos dos cartões (medido **118,6 px**, cabeçalho SCRIPTS). **Máximo** = deixa à coluna do documento o seu mínimo (menus + engrenagem da barra da aba): **712,8 px** na área de teste 1000×700. Duplo-clique volta à largura de fábrica (220). `Esc` devolve o estado de antes do carregar. Tremor da caneta abaixo de `DRAG_THRESHOLD_PX` = clique. O 2.º clique tem de estar perto do 1.º. Botão direito/contexto limpa o registo do clique. Largar perdido acaba no carregar seguinte. Largura só da sessão. |
| H | **Peso do texto = maquete 14, só na aba de scripts** (dono 10/11): corpo 400, botões/kbd/h4/th 500, títulos/estado 600, mono do código 400 (só o kbd é 500). **Causa-raiz medida:** o preset de nitidez de omissão (`CrispHeavyPlus`) soma **+300** de peso a ≤ 12 px (o corpo 400 pintava a 700); os testes usavam `Default` e nunca o viram. Cura = `com_peso_exato`. Régua de tinta na GPU, palavra «number»: **112,4** (CSS-400: 112,9; CSS-600: 153,1; antes: 176,9). **Medir = pintar** em todo o lado da aba (`tab_menu` `width_of`, tecla da cabeça, título das opções, cabeçalho da `ui-table` com o espaçamento do preset). Altura de linha = da casa (inalterada). |
| I | Tamanho da letra aplicado **uma vez** (Small 10,22 → 11,08, Large 15,98 → 13,85 para Sm; cursor/selecção do campo na altura mostrada; rename e ajuda centram pela `folga_do_texto`); página de ajuda com teste; coluna do ponto do Problems 44 → 18,6 px; cabeçalho «Name»; `forget_frame` (alvos velhos na aba de ajuda / documento fechado — era alcançável). |
| J | Código movido para fora do `ph2d-app-code` (27 039 → ~26 460 contra o tecto 26 628): `FindState` → panel `find_state.rs`; lógica do divisor → `splitter.rs`; `ClosedFolds`/`PlayProblem`/strict → `ph2d-code-assist`; teste `peso_na_placa` → panel. |

## 4. Prova de fecho (1×; sem suíte nem nextest — regra do dono 23/10, são do integrador)

- **Testes por nome, verdes** (rondas até ao fecho de 10/11): `ph2d-panel-code` `opcoes_do_script`, `valores_ao_vivo`, `problemas`,
  `quadro_do_depurador`, `bancada::` · `ph2d-ui-table` `reserva_e_fios` · `ph2d-code-bancada` `o_par_aceita_a_virgula_decimal_como_o_numero` ·
  `ph2d-app-code` `onda_b` · `ph2d-tokens` `the_hover_layer_lightens_a_dark_surface` · `bash scripts/processo.prova.sh`.
  Fase final: `ph2d-text` `peso_exato` (sem reforço do preset a 11 px; desvio Light; repõe aninhado e em pânico) · `ph2d-panel-code`
  `peso_das_pecas` (cada peça do quadro parado no peso do CSS, escrito à mão 400/500/600), `peso_na_placa` (tinta na GPU) ·
  `ph2d-ui-table` `pesos` e a coluna justa · `ph2d-app-code` `divisor` · o fit do «In this script» conta o título (mutação medida).
- **Mutações:** 18 nas frentes A–C; D e E vermelhas; F vermelhas (vírgula do Vec2; lei antiga do título da grelha e do balão); fase final:
  corpo/CORPO, título, kbd e mono do peso das peças; escopo `com_peso_exato`; termos do mínimo do divisor (títulos, outline). Todas
  vermelhas salvo o x da primeira coluna `x += twist` (sem teste; a foto prova).
- **Portões por nome, verdes:** `file_loc_caps`, `fn_loc_caps`, `arch_safe_clamp_only`, `architecture_the_shell_only_shrinks`,
  `architecture_workspace_file_loc_cap`, `every_frame_goes_through_the_theme_door`, `no_magic_numeric`, `no_literal_color`,
  `hr15_no_hardcoded_ui_strings`, `architecture_widget_showcase_coverage`, `architecture_claude_md_cabe_no_orcamento`,
  `architecture_panel_loc_cap`, `architecture_widget_loc_cap`, **`no_dependency_climbs`, `cycle_prevention`** (Cargo.lock). `fmt --check` ✓ ·
  `clippy -D warnings` nas crates tocadas ✓ (`PH2D_SUITE=fecho`).
- **Auditoria, lentes independentes:** sem violação de regra; correcção curada (item F); a lente de ratchet gerou o aviso do §2.
- **Binário do smoke:** ver o fim do §8.

## 5. Folgas medidas (recontar na integração; `wc -l` no HEAD `d137ddac68`, 11/11)

| Ficheiro / crate | Medido / tecto | Folga |
|---|---|---|
| crate `ph2d-app-code` (catraca `the_shell_only_shrinks`) | **26 584** / 26 628 | 44 (era 2: a barra saiu −259; voltaram testes, o largar perdido, o Card Style e o `tab_leaf`; a mecânica fina do submenu foi testada na `ph2d-panel-code`) |
| crate `ph2d-code-smoke` | 614 / 614 | **0** (não tocada) |
| crate `ph2d-panel-code` | 22 129 | sem tecto de crate |
| `ph2d-editor-core/src/paint.rs` | 700 / 700 | **0** (não tocado) |
| `ph2d-editor-core/src/paint_text.rs` | 692 / 700 | 8 (não tocado) |
| `ph2d-editor-core/src/screens/hero/paint.rs` | 700 / 700 | **0** (não tocado; o teste do `publish_text` vive em `published_embolden_tests.rs`) |
| `ph2d-app-code/src/surface.rs` | 643 / 700 | 57 |
| `ph2d-panel-code/src/tab_bar/menu.rs` · `tab_bar.rs` · `palette.rs` · `glyphs.rs` | 387 · 276 · 331 · 354 / 700 | — |
| `ph2d-editor-core/src/documents_surface.rs` · `widget/lista_flutuante.rs` | 134 · 267 / 700 | — |
| `shells/desktop/src/input_dispatch.rs` | 596 / 700 | 104 |
| crate `ph2d-code-assist` | 4 088 × isenção 4 188 | — |

`the_shell_only_shrinks` e `architecture_workspace_file_loc_cap` verdes (por nome, 11/11). O tecto 26 628 da `ph2d-app-code` não foi
baixado (o integrador reconta).

## 6. Smokes (comando exacto)

Os passos 1–3 vêm das rondas aprovadas (ficam para o próximo testador); **4–6 são novos e PENDENTES do dono**.

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=11 cargo run --profile smoke -p ph2d-host-desktop` →
   engrenagem ⚙ à direita de «line 21» → «Script options»: **três caixas de marcar**, «Format on save» marcada; hover **clareia**. Errado se:
   pílula/disco ou hover a escurecer. Foto: [ad_options.png](../ferramentas/smoke/ad_options.png).
2. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=12 cargo run --profile smoke -p ph2d-host-desktop` →
   parado na linha 18: «Continue F8» / «Next line F10» na mesma linha de base ([ad_botoes.png](../ferramentas/smoke/ad_botoes.png));
   ao continuar, valores ao vivo **emoldurados** nas linhas 17–18. Errado se: valor sem moldura.
3. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=13 cargo run --profile smoke -p ph2d-host-desktop` →
   rato no ◆ da linha 1: balão Hero 220 / Hero 2 180, números em mono ([ad_passo_1.png](../ferramentas/smoke/cena13/ad_passo_1.png);
   o passo 1 agora: [ah_passo_1.png](../ferramentas/smoke/cena13/ah_passo_1.png)). Numa célula Vec2, «1,5; 2» dá (1,5 ; 2).
4. **Divisor (NOVO).** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=11 cargo run --profile smoke -p ph2d-host-desktop` →
   segure o rato no **vão entre a coluna da esquerda (Scripts / In this script) e o código** e arraste para a direita: a coluna cresce
   (220 → 340 com 120 px) e o cursor vira ↔; para a esquerda: pára nos 118,6 px sem cortar título. **Duplo-clique** no vão: volta aos 220.
   `Esc` a meio do arrasto devolve a largura de antes. Errado se: o vão não reage, a coluna passa por cima do código, ou o duplo-clique não
   repõe. Fotos: [antes](../ferramentas/smoke/af_divisor_antes.png) · [depois](../ferramentas/smoke/af_divisor_depois.png) ·
   [duplo-clique](../ferramentas/smoke/ag_divisor_duplo.png) · [+120](../ferramentas/smoke/ah_divisor.png).
5. **Peso do texto (NOVO).** Cena 12 (`PH2D_CODE_SMOKE=12`, mesmo comando): o texto da aba parece **fino como o da maquete 14** (corpo
   regular, títulos e estado mais carregados), não negrito. Errado se: o corpo parecer a negrito como antes. Fotos:
   [ah_final.png](../ferramentas/smoke/ah_final.png) · [ah_parado.png](../ferramentas/smoke/ah_parado.png) ·
   [lado a lado com a maquete](../ferramentas/smoke/ag_peso_lado_a_lado.png).
7. **Cena 15 (NOVO).** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-ScriptEditor && PH2D_CODE_SMOKE=15 cargo run --profile smoke -p ph2d-host-desktop` →
   (1) a aba `arena.luau` com «Problems (10)» e a grelha «Who has what» (14 objects × 7 properties), células âmbar;
   (2) rato na célula âmbar `6` do Jumper: o pé diz «Jumper · speed is 6 here (2 in the script)»; (3) roda sobre a grelha: até Rover.
   Errado se: lista vazia, grelha sem âmbar ou sem rolar.
6. Mude o tamanho da letra da interface (Small/Large) e confira que o cursor e a seleção do campo de busca/rename acompanham a altura da letra.

## 7. Aberto

1. A cura do hook só vale depois da integração (corre o do primário).
2. `ph2d-editor-core/src/paint.rs` 700/700 e `screens/hero/paint.rs` 700/700 — **não tocados de propósito**: nada desta linha
   precisa deles, e partir ficheiros foundational que outras linhas editam só fabrica conflitos no merge; quem precisar de
   crescer lá cura movendo. `paint_text.rs` 692/700.
3. O integrador reconta a catraca (`ph2d-app-code` 26 584 × tecto 26 628; `ph2d-code-assist` 4 188).
4. **Memória no primário `project-memory/`, NÃO comitada** (as do dono de 10/11) — o integrador funde-as.
5. **Smoke do dono: PENDENTE** (esta ronda e as anteriores).

Fechados em 11/11 (2.ª ronda, §8): o caminho do herói (`surface_input`) e a linha acesa (`row_bg`) com teste e mutação vermelha; os
fundos reais da lista, do «In this script», da grelha e da ajuda com teste por sítio; o Card Style em submenu; a mira até ao submenu.

Mantém-se: altura de linha = da casa (decisão do dono). O peso da maquete foi **aplicado** (já não fica «de propósito» de fora). Os abertos
de desenho da Onda B/Q3_Q4 continuam nos handoffs deles. O ramo `backup/line-ScriptEditor-2026-11-08` foi apagado com o sim do dono.

## 8. Rondas anteriores e binário do smoke

`603be29c52..e082d45b6d`: alvos velhos (`forget_frame`), `ph2d-ui-table` partido (`measure.rs`), tinta das opções pela superfície do
popover, tamanho da letra 1×, limiar do «In this script», cobertura de `value_outline`, cabeçalho «Name»; fotos `ae_*`. Cada lei com teste
por nome, vermelho por mutação e verde. Mecanismo da fase final (peso exacto, divisor): itens G/H e os doc-comments de
`peso_exato.rs` e `splitter.rs`.

- **Ícones do cabeçalho SCRIPTS (dono 10/11, `cdaddf4cb1`):** o «+» e o «‹» com a altura das maiúsculas do título. Novo
  `ph2d_text::cap_height_px(nominal)`; `chrome::icon_on_caps` (ph2d-panel-code) põe a TINTA (traço incluído, `expand_stroke`)
  em `cap_height_px(LIST_TITLE_PX)` com o meio no meio das maiúsculas como o pintor as põe (topo do texto em `ao_pixel`),
  escolhendo escala e y inteiro do encaixe de menor erro; alvo do clique intacto (22 px). Só este cabeçalho usa o padrão
  (o lixo da linha e os outros cartões ficam). Medido Normal: maiúsculas 9,28 px; tinta antes + 7,23 / ‹ 6,30, depois
  + 9,37 / ‹ 9,33 (Large: 10,71 → 10,92 / 10,76). Teste
  `os_icones_do_cabecalho_scripts_tem_a_altura_das_maiusculas_do_titulo` (Normal e Large, ± 0,5 px altura e meio); mutação
  (o cabeçalho de volta a `chrome::icon_button`) → vermelho nos 4 casos. Foto: [ai_icones.png](../ferramentas/smoke/ai_icones.png) ·
  [zoom 4×](../ferramentas/smoke/ai_icones_zoom.png) — título, + e ‹ com 9 px de tinta a partir da mesma linha.

- **Letras do dono (10/11, `6c6db05d2f`, `4cc19dfcc8`):** o nome do script na lista, as entradas do «In this script» e os
  menus da barra (títulos e nomes do menu aberto) na letra EXACTA de «alive» (mono do código, `TypeToken::Base` mostrado,
  `CODE_WEIGHT` 400; `mono_label.rs` chama o mesmo `paint_mono`; medir = pintar em `fit_w`, `outline_fit_w`, `min_w`);
  o título «Who has what» pelo `card_title` (maiúsculas espaçadas); «Console | Problems» no corpo da faixa «Paused», a
  aba aberta a 600 e a outra a 400. Testes `bancada::letras` (3) e `barra_da_aba::os_menus_da_barra_tem_a_letra_do_codigo`;
  6 mutações vermelhas. Foto `smoke/aj_letras.png`.

- **Cena 15 — muitos problemas e muitos donos (dono 10/11, `0582e4c755`):** contada pelo roteador da família
  (`ph2d_code_scenes::CENAS` e `ph2d_app_code::smoke::CENAS` 14 → 15; `steps(15)` = `STEPS_15`). `ph2d-code-scenes/src/arena.rs`:
  14 formas correm o `arena.luau` (`ferramentas/smoke/arena_crowd.luau`; 7 propriedades: number com min/max, int, bool, text,
  color, vec2, choice), 10 com valor próprio (âmbar), a grelha rola (14 > o terço da coluna). Problemas sem o `luau-lsp` (10):
  `self.X is never set — did you mean` ×6, `No component has a property` ×2, `No master` (`Bulet`), `No object` (`Runer`); o
  `self.sped` do Play (aritmética com `nil`) funde-se na linha 26 com «×14 in Play» e pára no Runner (o cartão Stopped aparece).
  Com o `luau-lsp` somam-se `local` sem uso, sombra de global (`local math`) e argumentos a mais (`clamp`). Teste
  `o_passo_1_da_cena_15_mostra_muitos_problemas_e_a_grelha_com_celulas_mudadas` (`ph2d-host-code-scenes`: o `ph2d-app-code`
  está no tecto, e os testes contam): ≥ 8 problemas, ≥ 8 linhas, ≥ 2 âmbar; mutação (sem valores próprios) → vermelho.
  `cena14/fotos_q3.sh` ganhou `RODA=n`. Fotos [passo 1](../ferramentas/smoke/cena15/passo_1.png) ·
  [passo 2](../ferramentas/smoke/cena15/passo_2.png) · [passo 3](../ferramentas/smoke/cena15/passo_3.png).
- **A cena 15 sai para a crate NOVA `ph2d-code-scene-arena` (`7315e9b6b3`):** o `0582e4c755` pôs a catraca vermelha
  (`ph2d-code-scenes` 755 / 666, `ph2d-code-smoke` 631 / 614). A crate leva `arena.rs` (→ `lib.rs`), o `STEPS_15` e o roteador
  `steps()` (a 15 aqui, o resto `ph2d_code_smoke::steps`; a code-smoke não a pode ver — o roteiro usa o `Gesture` dela, ciclo).
  `ph2d_app_code::smoke_drive::steps` reexporta o da crate nova (ph2d-app-code 26 628 / 26 628, sem crescer: a doc do módulo
  reflui 1 linha); `ph2d_app_components::code_smoke::arena` = reexport. Depois: code-scenes **665**, code-smoke **614**, a nova
  **123** (tecto 223 = medido + folga 100). Teste `o_roteador_serve_a_15_daqui_e_as_outras_da_code_smoke`; a cena 15
  (`ph2d-host-code-scenes`), `the_shell_only_shrinks`, `every_member_inherits`, `cycle_prevention`, `no_dependency_climbs` verdes.
- **Letra de «alive» na frase da grelha vazia e no texto dos problemas (dono 10/11, `916043568c`):** `code.bancada.grid.no_props`
  por `mono_label::paint` (cor `text3`); a coluna Problem da lista `.mono()` (a `CodeCells` mede, corta e pinta pela mono da
  `CodeView`; `Plain` = `Text1` como antes; o cabeçalho fica na letra da interface). Teste
  `letras::a_frase_sem_propriedades_e_o_texto_dos_problemas_tem_a_letra_de_alive`; mutações (grelha de volta a `label_in`; coluna
  sem `.mono()`) → vermelho as 2. Fotos [cena15/passo_1.png](../ferramentas/smoke/cena15/passo_1.png) (refeita) ·
  [ak_letras.png](../ferramentas/smoke/ak_letras.png) (cena 12: a frase inteira na mono).
- **A grelha «Who has what» na letra e na tinta de «alive» (dono 10/11, `0f410f6505`):** todas as colunas `.mono()` (cabeçalhos
  na letra da interface); nomes e valores que não são números em `TEXT_INK = Ink::Name` → `Hint` da mono = `Text2` (os valores
  de texto estavam em `Text1`; os nomes já em `Text2`, só mudam de letra). Números ficam no `value_ink` (fora da rampa de texto),
  o «mudou» no âmbar. Contraste do `Text2` sobre as duas listras, com e sem cartões: Forge ≥ 5,96, Light ≥ 7,86 (o `Text3`
  reprova: 3,39–4,97). Testes `letras::a_grelha_tem_a_letra_e_a_tinta_de_alive` e `a_tinta_da_grelha_tem_contraste_nos_dois_temas`;
  mutações (nomes sem `.mono()`; propriedades sem `.mono()`; `TEXT_INK = Plain`; `changed: false`) → vermelho as 4. Fotos
  [passo_1.png](../ferramentas/smoke/cena15/passo_1.png) · [passo_1_claro.png](../ferramentas/smoke/cena15/passo_1_claro.png) ·
  [al_grelha_zoom.png](../ferramentas/smoke/cena15/al_grelha_zoom.png) (2×, grelha ao lado dos problemas, escuro e claro).
- **A letra do script igual à dos problemas (dono 10/11, `1afce7cb8e`):** MEDIDO antes de mexer: já eram a mesma corrida
  (fonte, índice, corpo, eixo `wght`) — a coluna Problem pinta pela `CodeCells` → `CodeView::paint_mono`, com as `Metrics` do
  próprio código. Corpo / altura da linha: Small 12 / 17 px · Normal 13 / 19 px · Large 15 / 22 px, antes = depois (nada do
  que deriva da letra mudou). O que entra: `CODE_TYPE`/`code_font_px()` em `metrics.rs`, a ÚNICA fonte do corpo da mono
  (`Metrics::measure`, a re-medida do `paint.rs` e do `help/paint.rs`, `mono_label`, `Font::Mono` da ajuda — antes três
  repetiam `TypeToken::Base`). Teste `letras::a_linha_do_codigo_tem_a_letra_do_texto_dos_problemas` (código = problema =
  «alive» = `code_font_px` em Small/Normal/Large); mutações (linhas do código a 0,9×; `CODE_TYPE = Sm`; coluna Problem na
  letra da interface) → vermelho as 3. Fotos [am_codigo.png](../ferramentas/smoke/am_codigo.png) (cena 13) ·
  [passo_1.png](../ferramentas/smoke/cena15/passo_1.png) · [am_codigo_zoom.png](../ferramentas/smoke/am_codigo_zoom.png)
  (2×, linha do código ao lado de um problema).
- **O poço do código no tom das linhas de Problems (dono 10/11, `99620647d7`):** o glifo já era o mesmo; a diferença era o
  fundo. Nos cartões `palette::Well`: fundo `code-bg` → `card_tone()` (`kanban-card`), calha `code-gutter` → `card-field`,
  linha do cursor → `kanban-card-raised`, selecção → `accent` a `BAND_INK` sobre o cartão (o `selection` sobre o painel
  ficava a 1,00:1 do cartão no Oled), ocorrências → `kanban-slot`, pílula e caixa de exemplo da ajuda → `card-field`;
  `PH2D_CARDS=0` = os `code-*` de sempre (o `CodeBg` só tem este consumidor, e o campo clássico do cromo fica nele).
  Fundo: Dark (14,17,23) → (34,41,58) · Forge (39,27,42) → (34,41,58) · Light (255,244,225) → (255,255,255) · Sunstone
  (255,249,239) → (255,255,255). Contraste nos 12 temas: toda tinta ≥ 4,5:1 no fundo e na linha do cursor (pior 4,66
  Forge), comentário/dica ≥ 3:1, `text-3` na calha ≥ 3,51 (Oled), texto na selecção ≥ 5,11, selecção 1,49–2,01 do fundo.
  Fora da régua: `Link` (o acento sob Ctrl, sublinhado) 4,29 Oled e 4,41 Sunstone na linha do cursor (era 4,21 Sunstone e
  3,98 Candy antes). Nitidez na placa (franja sRGB / franja em luz / borda), mesma frase: Dark código antigo 0,387 / 0,469
  / 1,917 · novo 0,395 / 0,465 / 1,920 · problema 0,363 / 0,366 / 1,945 — a geometria do glifo não muda com o fundo (a
  cobertura normalizada é a mesma); o ganho é o par de contraste igual ao dos Problems. Testes
  `palette::tests::{o_poco_do_codigo_e_o_tom_do_cartao, as_tintas_leem_se_no_poco_dos_cartoes}`,
  `nitidez::a_letra_do_codigo_tem_o_fundo_dos_problemas` (placa; sem placa não mede; afirma o fundo lido = `kanban-card`);
  `pagina::cada_cor_pintada_vem_de_um_token` aceita o acento a `BAND_INK`. Mutação (o ramo dos cartões nunca entra) →
  vermelho. Fotos [ao_fundo.png](../ferramentas/smoke/ao_fundo.png) · [ao_fundo_claro.png](../ferramentas/smoke/ao_fundo_claro.png) ·
  [ao_nitidez_zoom.png](../ferramentas/smoke/ao_nitidez_zoom.png) (3×, linha 17 do código sobre a linha 17 da tabela).

- **Quarto modo de Text rendering «Crisp + stem darkening»** (dono 10/11, `07d722387f`): `TextRendering::CrispPlusDarkened` =
  `CrispHeavyPlus` + `stem_darkening` (preset e não toggle: o dono pediu «um quarto item»). `FontEmbolden` do Vello 0.11 por
  `paint::font_embolden_for(px, ink)` em `paint_text` (2 sítios), `paint_text_rotated` e `ph2d-panel-code/src/glyphs.rs`: 0,05 px
  por lado até 16 px, linear a zero aos 20 px, **só tinta clara** (luminância > 0,5). Tabela medida na placa no doc de
  `ph2d_tokens::STEM_DARKENING_PX` (`nitidez::o_escurecimento_aproxima_a_franja`, 12/13 px × Dark/Light × 0–0,4 px): franja-luz
  Dark 12 px 0,479 → 0,444 (Problems 0,413) com tinta +7 %; +0,075 já dá +14 % (lê negrito); no Light o código já está abaixo do
  Problems e não se aproxima ⇒ não alarga. Vale dentro do `com_peso_exato` (borda, não peso); avanços iguais; de fábrica
  byte-idêntico (`FontEmbolden::default`). Em memória como os outros (nenhum persiste). A arte vectorial do documento
  (`ph2d-vector/src/record.rs`) NÃO recebe (é do artista). Testes: `settings_text::tests::every_text_rendering_row_picks_its_mode_and_closes_the_menu`,
  `published::embolden_tests::*` (3), `typography::tests::{crisp_plus_darkened_is_crisp_heavy_plus_with_stem_darkening, stem_darkening_fades_with_size_and_only_in_its_mode}`,
  `nitidez::a_aba_de_scripts_escurece_no_preset_escurecido`.
  Mutações (uma de cada vez, âncora única, `git checkout` depois; exit do `cargo-test-narrow.sh`):

  | Mutação | Teste(s) | Exit |
  |---|---|---|
  | `font_embolden_for` devolve sempre o `default` | `embolden_tests` (2 de 3) · `a_aba_de_scripts_escurece_no_preset_escurecido` | 1 · 1 |
  | a linha nova escolhe `CrispHeavyPlus` | `every_text_rendering_row_picks_its_mode_and_closes_the_menu` | 1 |
  | sem o teste da tinta clara | `dark_ink_is_never_emboldened` | 1 |
  | `stem_darkening_px` ignora a flag | `stem_darkening_fades_with_size_and_only_in_its_mode` | 1 |

  Fotos (cena 13 passo 1, `CLIQUES` Edit ▸ Preferences… ▸ Text rendering ▸ item, o menu fecha):
  [ap_padrao.png](../ferramentas/smoke/ap_padrao.png) (Crisp Heavy +) · [ap_escurecido.png](../ferramentas/smoke/ap_escurecido.png) ·
  [ap_nitidez_zoom.png](../ferramentas/smoke/ap_nitidez_zoom.png) (3×: linha 11 do código padrão/escurecido, linha 17 do Live
  padrão/escurecido). Pixéis com diferença > 1 %: código 913 de 8 400 (luma média 53,4 → 54,2), Live 288 (47,7 → 47,9),
  Hierarchy 147. A 3× o traço do código lê-se um pouco mais cheio; a 1× e no Live é quase imperceptível — o ganho é pequeno
  por desenho (0,05 px).
- **Fio da barra da aba e respiro acima da linha 1** (dono 10/11, `d8922bd4ee` · `1bb232ef94` · `6a05d635f9` · `1dc85ab264`).
  Fio: `cards::bar_rule` = última fila de píxeis da barra, de lado a lado, `BorderStrong` a `StrokeToken::Thin` (o `Border`
  medido somia no tom do cartão: 34→51 de luma no escuro; `BorderStrong` 34→78, o token «visible dividers», como o separador
  da banda). Respiro: `Metrics::top_pad = Spacing::Md` (8 px ≈ 0,4 linha de 19 px; o VS Code/Zed usam ~meia linha; `Md` é a
  margem dos cartões da aba), conteúdo que ROLA: `row_y`/`row_span`/`hit`/`ensure_visible`/`max_scroll_y`/barra lêem
  `content_y`; o clique no respiro cai na linha 1 pela coluna do x; mostrar uma linha a subir deixa o respiro por cima (a
  linha 1 volta à rolagem 0); a calha, o cursor, os sublinhados e a busca seguem `Row.y`. A barra de cima (altura, partição,
  cartão, fio) mudou-se para `ph2d-panel-code::cards` (reexport em `tab_menu`): `ph2d-app-code` 26 628 / 26 628. Testes
  `respiro.rs` (5) e `barra_da_aba::um_fio_separa_a_barra_do_codigo_e_a_linha_1_respira_abaixo_dele`; mudados:
  `a11y::um_no_por_linha_visivel_e_rolar_troca_os_nos` e `virtualizacao::rolar_uma_linha_molda_no_maximo_uma` (rolam
  `n·line_h + top_pad`), `sublinhado::o_culpado_e_sublinhado_nas_colunas_dele` (topo da fila + `top_pad`). Mutações, todas
  exit 1: `row_y` sem respiro · clique sem respiro · `ensure_visible` sem respiro · `row_span` sem respiro · `max_scroll_y`
  sem respiro · fio de área zero · `tab_menu` volta a `cards::close`. Fotos (cena 13 passo 1):
  [aq_separador.png](../ferramentas/smoke/aq_separador.png) · [aq_separador_zoom.png](../ferramentas/smoke/aq_separador_zoom.png)
  (3×) · [aq_separador_claro.png](../ferramentas/smoke/aq_separador_claro.png).
- **View ▸ Text rendering da aba (dono 10/11, `0e1dc18d39`)** — a letra do CÓDIGO tem o seu modo, separado do do app.
  Antes: o código pintava sempre `hint(true)`, sem encaixe X, peso fixo `400` e letter-spacing 0, mas o escurecimento
  vinha do preset do APP (`font_embolden_for`) — o único vazamento. Agora `ph2d-panel-code::code_text` (thread-local
  `TextRendering`, fábrica `Default` = o de sempre, byte-idêntico) dá a dica, o encaixe (`snap_x_apply`, agora `pub`) e
  o escurecimento (`font_embolden_with`) a `glyphs::draw`, ao texto, à calha e aos rótulos de `mono_label`; `geom::walk`
  encaixa o x relativo igual (pintura = cursor = clique). Por modo: Default dica·sem encaixe·sem escurecer · Crisp Heavy
  sem dica·encaixe 1 px · Crisp Heavy + dica·½ px · Crisp + stem darkening dica·½ px·escurece (tinta clara). Nunca o
  reforço de peso nem o espaçamento (grelha da mono). Estado de sessão por app, como o Card Style (não gravado). Menu:
  4 linhas planas `Text rendering: … ✓` no View (idioma do Card Style), não um submenu em cascata — a barra da aba não
  tem cascata e o `ph2d-app-code` está no tecto (26 626/26 628 depois de encurtar `TABLE`/`ALL`). Testes:
  `nitidez::a_letra_do_codigo_segue_o_modo_do_script_e_nao_o_do_app` (troca o antigo `a_aba_de_scripts_escurece_no_preset_escurecido`),
  `nitidez::o_modo_do_script_nao_muda_a_letra_da_interface`, `clique::o_cursor_e_o_clique_seguem_o_x_encaixado_de_cada_modo`,
  `letras::a_linha_do_codigo_tem_a_letra_do_texto_dos_problemas` (× 4 modos, dica e alargamento),
  `barra_da_aba::o_menu_view_escolhe_a_letra_do_codigo`. Mutações exit 1: dica fixa · escurecimento do app · `geom` sem
  encaixe · pintura sem encaixe · linha do menu sem despacho. Fotos `ar_menu.png`/`ar_opcoes_zoom.png`: por tirar.

- **Menu View ▸ Text rendering do script** (foto `smoke/ar_menu.png`): quatro linhas planas «Text rendering: …» (a barra da aba não tem submenu; ph2d-app-code no tecto). Clippy do workspace `-D warnings` verde depois de `0e1dc18d39`.
- **Coluna do lado da Sheet rola (dono 10/11, `4d8c7789bf` + `28c6f53f2d`)** — Call stack · Waiting here passava por cima do pé com muitos à espera. Agora `debug_card/side.rs`: uma rolagem só (linhas inteiras, presa ao conteúdo, volta ao topo noutra paragem `linha:função:objecto`), barra da casa (`OwnBar`/`paint_scrollbar`) só se transborda, só linhas INTEIRAS à vista pintam e entram em `painted.stack`/`painted.waiting` (nada sob o pé recebe clique); roda (`side_wheel`, antes da tabela), arrasto da barra (`side_press`/`side_drag`), ponteiro ↕. `height()` = a de sempre `max` a natural com os respiros. A11y inalterada (o `text_rows` já lista todas as linhas, sem geometria). Testes `coluna_do_lado::*` (5). Mutações exit 1: tudo visível · roda morta · barra não agarra · sem reset na paragem · sem prender ao fim · roda sem linhas inteiras. Fotos (cena 14 passo 1, quadro baixado ao mínimo pelo puxador): `smoke/as_lado.png` (barra, recortada no pé), `smoke/as_lado_rolado.png` (`RODA_DEPOIS`, nova no `fotos_q3.sh`: Star e Hexagon à vista). A cena 15 para num ERRO (sem Waiting here): não serve.
- **«Crisp + stem darkening» do script decide pelo FUNDO (dono 10/11: «não faz nada»)** — a causa medida: a lei
  `Y_tinta > 0,5` deixava de fora, no poço escuro (`Y 0,022`), Keyword `0,398`, Function `0,474`, Number `0,440`,
  Constant `0,378`, Comment `0,356`, Operator `0,486`, Unresolved `0,350`, Link `0,349` (e no Forge também
  Punctuation/Hint `0,412`); só Text1/Property/Type/String/Builtin/Text2 alargavam, a `0,05` px. Agora
  `font_embolden_on(amount, ink, bg)` (editor-core): alarga a tinta MAIS CLARA que o fundo; o código passa o poço
  (`pal.bg`; a calha passa `pal.gutter`). Dark/Forge: as 17 tintas alargam; Light (poço `Y 1,0`): nenhuma. O chrome
  (`font_embolden_for`, `paint_text` não sabe o fundo) fica na lei velha e nos `0,05`. Quantidade própria
  `CODE_STEM_DARKENING_PX = 0,1` (sonda `escurecimento::a_quantidade_do_escurecimento_do_codigo`, placa, 3 linhas de
  Luau realçadas no poço Dark; «mudam» = píxeis da caixa com canal > 4/255; tinta = massa vs `+0`):

  | px | +0,05 | +0,075 | **+0,1** | +0,15 |
  |----|-------|--------|----------|-------|
  | 12 | 7,2 % · +5,7 % | 10,3 % · +10,8 % | **11,1 % · +14,5 %** | 11,9 % · +21,8 % |
  | 13 | 6,3 % · +5,6 % | 8,8 % · +10,2 % | **9,3 % · +13,7 %** | 9,9 % · +20,4 % |
  | 15 | 6,4 % · +5,1 % | 8,3 % · +9,2 % | **8,6 % · +11,9 %** | 9,1 % · +17,8 % |

  Escolha = o maior com tinta ≤ +15 %. Fotos (cena 13 passo 1, menu fechado): `smoke/at_sem.png` (Crisp Heavy +),
  `smoke/at_com.png`, `smoke/at_zoom.png` (3×, 4 linhas), `smoke/at_diff.png` (zoom + diferença ×8): no recorte de
  460×80 mudam 4 356 píxeis > 4/255 (11,8 %), 2 667 > 16/255 (7,2 %), máx. 66/255 — todas as tintas, roxo incluído.
  Veredicto: a 1:1 lê-se como traço um pouco mais cheio/claro lado a lado; isolado é subtil (é o *stem darkening*).
  Testes `escurecimento::*` (6, novo), `embolden_tests::with_a_known_background_the_lighter_ink_is_emboldened`;
  `letras::a_linha_do_codigo_tem_a_letra_do_texto_dos_problemas` passou a ler `code_stem_darkening_px`.
  Mutações exit 1: lei velha `Y > 0,5` (tabela + poço escuro) · `!=` (Light) · fundo = a própria tinta (poço escuro).

- **«Crisp + stem darkening» do CHROME decide pelo TEMA (dono 10/11: «para a UI também não faz diferença», `4a072abdab`)** —
  a lei velha `Y_tinta > 0,5` deixava de fora Text3 (`0,17–0,38`) e Accent (`0,31–0,68`) em todos os temas escuros, e o
  Text2 do Forge/Oled. O `paint_text` não sabe o fundo: a shell publica o tema com o preset (`published::publish_text`,
  no lugar do `set_text_rendering` do `hero/paint.rs`); `font_embolden_for` alarga, num tema `is_dark()`, toda a tinta
  mais clara que `STEM_DARKENING_UI_SURFACE = BgElev` (a superfície mais clara do chrome escuro, `Y 0,009–0,156`); num
  claro, nenhuma (nem a branca). Sem publicação: `Theme::default_for(ui_look())`. Quantidade `STEM_DARKENING_PX`
  `0,05 → 0,075` (sonda `escurecimento_da_ui::a_quantidade_do_escurecimento_da_interface`, placa; rótulo do chrome,
  Text1/2/3/Accent × PanelBg/Bg1/Bg2/BgElev/KanbanList/KanbanCard × Dark/Forge = 48 pares; mudam > 4/255 na caixa · tinta vs `+0`):

  | px | +0,05 | **+0,075** | +0,1 | +0,125 | +0,15 |
  |----|-------|------------|------|--------|-------|
  | 11 | 6,3 % · +6,1 % | **9,4 % · +11,6 %** | 10,4 % · +16,0 % | 10,9 % · +20,3 % | 11,3 % · +24,3 % |
  | 12 | 9,9 % · +4,7 % | **14,1 % · +8,7 %** | 15,1 % · +11,8 % | 15,6 % · +15,0 % | 16,2 % · +18,0 % |
  | 13 | 9,3 % · +5,2 % | **13,0 % · +9,3 %** | 14,0 % · +12,8 % | 14,7 % · +16,3 % | 15,1 % · +19,3 % |
  | 15 | 10,1 % · +4,4 % | **14,3 % · +7,9 %** | 15,1 % · +10,7 % | 15,9 % · +13,6 % | 16,3 % · +16,1 % |

  Escolha = o maior com tinta ≤ +15 % em TODOS os corpos (o `0,1` dá +16 % a 11 px). Pintores: `paint_text` (2 sítios) e
  `paint_text_rotated` já passavam pelo `font_embolden_for`; o `RichEditor` (verso do cartão) não — agora via o campo
  novo `GlyphStyle.embolden` (`ph2d-vector`); o texto do Quadro (`ph2d-board-layout`, conteúdo) fica com `default()`;
  o código tem a lei própria (`font_embolden_on`). Fotos (cena 13 passo 1, Edit ▸ Preferences… ▸ Text rendering;
  `CLIQUES="62 11;105 89;305 205;500 255"` / `…;525 277"`): `smoke/au_ui_sem.png` (Crisp Heavy +), `smoke/au_ui_com.png`,
  `smoke/au_ui_zoom.png` (Hierarchy e Transform, sem | com, 3×). Recortes Hierarchy+Inspector+timeline (360 300 px):
  10 667 mudam > 4/255 (2,96 %), 825 > 16/255 (0,23 %), máx. 56/255 fora do relógio da timeline (186). Veredicto
  honesto: chega a todo o rótulo, mas sobre o Crisp Heavy + (já pesado) a 1:1 e mesmo a 3× é quase imperceptível.
  Testes `embolden_tests::on_a_dark_theme_every_chrome_ink_lighter_than_the_surface_is_emboldened` (novo),
  `dark_ink_is_never_emboldened` (agora com o tema Dark publicado: a preta, abaixo do BgElev); `nitidez::o_escurecimento_aproxima_a_franja`
  só imprime (a quantidade do código e do chrome mede-se nas duas sondas). Mutações exit 1: sem `is_dark()` · sem a
  comparação com a superfície · lei velha `Y > 0,5` · `STEM_DARKENING_PX = 0,1` (sonda de placa). Sem teste: a chamada
  `publish_text` do hero (o tema por omissão é o escuro). Clippy do workspace `-D warnings` verde.

- **Ronda 11/11 — submenu, largar perdido, link, buracos de teste (`89ac2897e9`, `70aea86b73`, `da68460004`, `c67465ff6e`).**
  (1) **`View ▸ Text rendering ▸`** em cascata: a barra da aba mudou-se para `ph2d_panel_code::tab_bar` (`TabBar<M>`, genérica; a
  `ph2d-app-code` monta `MenuItem::{Cmd, Note, Sub}` e corre o comando). Pairar na linha-mãe abre o submenu (a mãe fica acesa),
  outra linha fecha-o, clicar na mãe também abre, escolher fecha tudo, Esc fecha um nível; ✓ (`IconId::Check`, acento) à esquerda
  do em vigor; à direita, senão à esquerda, senão encostado à borda da aba. Os 4 modos ficam também na paleta Ctrl+K. Sem teclado
  (a barra não o tem). (2) **Largar fora da aba**: `PointerInput.primary_held`; a shell anota o primário no topo de `pointer_button`
  (`Documents::note_button`); na aba, um movimento sem o botão com carregar/arrasto vivo é o largar perdido — divisor e selecção
  acabam ali (pelo caminho do Up), o nome arrastado da lista cancela-se (marca apagada, nada cai). (3) **Link sob Ctrl**:
  `palette::link_ink` = Accent, senão o 1.º degrau a ≥ 4,5:1 no poço e na linha do cursor (medido, linha do cursor: Oled Accent 4,29 →
  Hover 5,10; Sunstone Accent 4,41 · Hover 3,59 → Press 5,43; os outros 10 no Accent, 4,66–8,60); sintaxe intacta; vale também com
  `PH2D_CARDS=0`. (4) Buracos: `the_hero_painter_publishes_the_preset_and_the_theme`, `rotated_text_is_emboldened_only_in_the_darkened_preset`,
  `mono_label::paint_on` com o fundo real (lista: linha corrente; «In this script»: entrada acesa; grelha/cartão; barra: cartão,
  acento, popover = `Superficie::fundo`; ajuda: a caixa do exemplo). Testes novos: `barra_da_aba::{o_submenu_text_rendering_do_view_escolhe_a_letra_do_codigo,
  o_submenu_fecha_por_outra_linha_pelo_esc_e_por_fora}`, `submenu::*` (3), `fundo_real::*` (2), `divisor::{o_primeiro_movimento_sem_botao_acaba_o_arrasto,
  o_largar_perdido_acaba_a_seleccao_e_cancela_o_nome_da_lista}`, `documents_surface::tests::o_botao_primario_e_anotado_ate_largar`,
  `palette::tests::o_link_le_se_no_poco_e_na_linha_do_cursor_nos_12_temas`, os 2 `embolden_tests`. Crates inteiras (motivo: código
  movido): `ph2d-app-code` 280 ✓ · `ph2d-panel-code` 149 ✓; `ph2d-host-game-scenes` compila (scrub 5 ✓); clippy `-D warnings` das 6
  crates tocadas ✓; fmt ✓. **Mutações 13/15 vermelhas** (uma a uma, `git checkout` depois): submenu (hover, mãe, Esc, posição, ✓),
  `pal.bg`, `lost = false`, sem cancelar a lista, `note_button` sem guarda, sem `publish_text`, rodado sem escurecimento, sem
  `link_ink`, ordem dos degraus → exit 1. Sobrevivem `row_bg` e `surface_input` (§7 itens 1 e 7). Fotos (cena 13 passo 1, rato
  real): [av_submenu.png](../ferramentas/smoke/av_submenu.png) (`CLIQUES="752 42;790 195"`) ·
  [av_submenu_escolhido.png](../ferramentas/smoke/av_submenu_escolhido.png) (escolheu «Crisp + stem darkening» e reabriu: ✓ mudou) ·
  [av_submenu_zoom.png](../ferramentas/smoke/av_submenu_zoom.png) · [aw_divisor_largado_fora.png](../ferramentas/smoke/aw_divisor_largado_fora.png)
  (`ARRASTA="541 300 1750 300" RATO_DEPOIS="1200 300;900 300;700 300"`: largado no Inspector, o rato volta sem botão e a coluna
  não o segue; `RATO_DEPOIS` novo no `fotos_q3.sh`).

- **Ronda 11/11 (2.ª) — os abertos (`c5c350b0ee`, `d6a0d145df`, `4d2b1bff1f` e os commits de teste até `d137ddac68`).**
  (1) **`View ▸ Card Style ▸`** (Sheet · Cards · Strip, ✓ = `CommandState.card_style`) antes do Text rendering; as linhas soltas
  saem do View (ficam na paleta). O smoke conta as escolhas com os submenus abertos (`CodeDocs::tab_leaf`): `ClickTabMenuRow(3/4/2)`
  da cena 14 continuam a dar Cards/Strip/Sheet (teste `onda_b::o_roteiro_escolhe_o_estilo_dentro_do_submenu_pelo_indice_de_sempre`).
  (2) **Mira** (`TabBar.last` + `aims_at`): a caminho do submenu aberto (triângulo ponto anterior → borda do submenu) o rato passa
  por cima da mãe vizinha sem trocar; fora do triângulo troca (`submenu::a_caminho_do_submenu_a_linha_de_baixo_nao_o_troca`).
  (3) Herói: `surface_input_tests::o_botao_anotado_chega_a_aba_no_carregar_e_no_mover` (aba falsa `Grava`; `[true, true, false,
  false]`). (4) `row_bg` pública + `fundo_real::a_linha_acesa_assenta_no_acento_e_a_apagada_no_popover`. (5) `glyphs::bg_probe`
  (gravador de testes, desligado = nada) + `fundo_por_sitio::*` (4: nome na lista pela linha corrente, entrada do «In this script»
  pelo realce, frase da grelha pelo cartão, exemplo da ajuda pela caixa e código da prosa pelo poço; cartões e clássico). (6) A
  catraca: os testes do submenu da aba juntaram-se em `barra_da_aba::os_submenus_do_view_escolhem_o_estilo_e_a_letra`; a mecânica
  fina (outra linha, clicar fora, a mãe, posição, mira) vive em `ph2d-panel-code` `submenu::*` (6). **Mutações, todas exit 1:**
  `primary_held: true` no `pointer` e no `pointer_move` do herói · `row_bg` ignora a linha acesa · lista/«In this script»/grelha/ajuda
  pelo fundo errado · gravador mudo · mira desligada · ✓ do Card Style trocado · `tab_leaf` desalinhado · a mãe não abre ao clicar ·
  Esc fecha tudo. Crates inteiras: `ph2d-app-code` 280 ✓ · `ph2d-panel-code` 156 ✓; clippy `-D warnings` (4 crates) ✓; fmt ✓;
  `the_shell_only_shrinks` e `architecture_workspace_file_loc_cap` ✓. Fotos (cena 13 passo 1, rato real):
  [av_card_style.png](../ferramentas/smoke/av_card_style.png) (`CLIQUES="752 42;790 121"`) ·
  [av_submenu.png](../ferramentas/smoke/av_submenu.png) (`"752 42;790 146"`) ·
  [av_submenu_escolhido.png](../ferramentas/smoke/av_submenu_escolhido.png) · [av_submenu_zoom.png](../ferramentas/smoke/av_submenu_zoom.png).

Binário de smoke depois de `rm -rf target/*/incremental` (11/11, HEAD `d137ddac68`; 1.ª: 14,27 s); a 2.ª:

```
▸ linha line_scripteditor · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.24s
```

