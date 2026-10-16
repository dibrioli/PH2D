# HANDOFF DE INTEGRAÇÃO — line/CodeEditor, Ondas 0 + 1 (15/10/2026)

> Leitor: o agente integrador (só por ordem EXPLÍCITA do Enio; a linha fecha e PARA). Desenho: [01_onda0.md](../01_onda0.md) · [02_onda1.md](../02_onda1.md) · [ADR-0188](../../architecture/decisions/0188-o-editor-de-scripts-mora-no-ph2d-e-o-script-mora-no-projeto.md). Handoff anterior (continuação, Onda 0): [ONDA0](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-15_ONDA0.md).

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` · merge-base com o `main` `0aab2c1cb6` · HEAD `065d40edd8` (antes deste handoff) · **62 commits** · 331 ficheiros (+31 706 / −931).
- **AGORA é integrável.** A Onda 0 sozinha não era (tirava o editor externo sem dar o interno); a Onda 1 devolve a edição DENTRO do app (aba, «Edit script», Ctrl+S → recarga).
- Não integrada, não enviada; o smoke (§8) é do Enio.

## 2. Superfície de colisão (colada de `bash /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor/scripts/collision-surface.sh`, 15/10)

```
PROJECT_SCHEMA 190 (base 189) · tripla (190, 13, 22) (base (189, 13, 22)) · VEC_SCENE 22 · FLIP 13 · DOC_VERSION 20 (= base)
Registos de componentes 108/109/109 (= base) · contrato congelado: node.rs e tool.rs intocados
ADR: cria 0188 (próximo livre 0189) · Cargo.lock: 12 pacotes novos (§4) · marcadores de conflito: nenhum
TETOS DE LOC: ✗ 1060/600 shells/desktop/src/main.rs (a linha troca 1 linha; o excesso vem da base)
```

## 3. Contadores como DELTA e números PROVISÓRIOS a recontar (§5.0)

| Contador | Delta | Quem reconta |
|---|---|---|
| ADR | +1: **0188** (provisório; a `0187` é da `line/MiroClone`) | integrador, contra o `main` do dia; renumerar também os links `0188-…` (CLAUDE.md §5.1, `01_onda0.md`, `02_onda1.md`, handoffs) |
| `PROJECT_SCHEMA` | +1: 189 → **190** (`scripts` em `ProjectFile`; braço 189 lê com o tipo vivo + o `0` do `scripts` vazio) | `python3 scripts/schema-recount.py <valor no main>` (precisa do argumento) — escada e tripla em `project_schema.rs`/`project_schema_tests.rs`. ⚠️ O braço 189 supõe `scripts` o **ÚLTIMO** campo do `ProjectFile`: se outra linha acrescentar campo depois dele, o braço parte |
| Shell (`.rs` de `shells/desktop`) | **−66** linhas contra o merge-base (a Onda 0 sozinha: +217) | `the_shell_only_shrinks` (verde); reconte na fusão |
| Painéis do inspector | `quantas_entradas_tem_cada_painel`: inspector 87 → 88 (o seletor de scripts, capacidade nova) | soma com outras linhas |
| Cenas de smoke `PH2D_CODE_SMOKE` | 2 (nível 1 e 2), escrito em `ph2d-app-code::smoke` e `ph2d-app-components::code_smoke::CENAS` sem gate que os cruze | quem somar um nível, mude os dois |

## 4. O que mudou em FOUNDATIONAL — e porque é ADITIVO

Contratos congelados (CLAUDE.md §6): **NENHUM tocado.** O diff não toca `ph2d-nodegraph`, `ph2d-editor-core/src/tool.rs`, `ph2d-vector-doc`, `ph2d-imageio` nem os gates `architecture_*contract_surface`. Prova (re-corrida 15/10):

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor
git diff --name-only 0aab2c1cb6..HEAD | grep -E "contract_surface|ph2d-nodegraph/|ph2d-editor-core/src/tool.rs|ph2d-vector-doc/|ph2d-imageio/"   # → vazio
git diff -U0 0aab2c1cb6..HEAD -- '*.rs' | grep -E '^[+-]' | grep -cE 'NodeOp|OpResolver|NodeManifest|RasterEditTool|CanvasPaintTool|PanelEvent|VectorOp|MAX_SPIRAL_TURNS|MAX_POLYGON_SIDES|MAX_VERTICES_PER_LLM_GEN'   # → 0
```

| Foundational | O que entra (aditivo: ficheiro/módulo irmão, variante nova ou default) |
|---|---|
| `ph2d-editor-core` | `doc_surface` (trait `DocumentSurface` + `Surfaces`, `KeyInput`, `a11y`/`commands` com default) · `documents_surface` · `screens/hero/surface_input` · abas de família em `document_tabs` · entrada antes do quadro (`pre_dispatch`) · `script_edits` (`InspectorScriptStatus::BrokenRunsPrevious`, culpado em `InspectorScriptError`) · `ime`/composição do `TextInput` (Onda 0) · `math::safe_clamp_ord`. As abas do quadro (`active: Option<BoardId>`) seguem como antes |
| `ph2d-tokens` | 13 cores `code-*` (`CodeBg` … `CodeComment`) em **12 temas**, gate `the_code_inks_read_and_differ` · `FONT_MONO` · `docs/design/tokens.json` |
| `ph2d-i18n` | `code_editor.rs` (`code.*`/`doc.*`) · `panel.inspector.script.*` |
| `tools/ph2d-panel-sync` | `PAINT_ONLY_CRATES` + `paint_only_census` + `staleness::paint_only_crates_are_honest`: `ph2d-panel-code` é pintura que guarda o prefixo (para os gates de UI); uma lista lida pelo gerador, pelo `staleness` e pela shell |
| `ph2d-script` | recarga (`drain_reloads`), console e tectos, última versão boa (`runs_previous`), `ScriptEditRequests`, `LibraryEpoch`, `ScriptLibrary` (PH2DSCRIPTS 1.0), `LuauScript` v3, `ScriptDiagnostic` com linha |
| `ph2d-script-doc` (nova) · `tools/ph2d-bindgen` · `runtime/luau` | a tabela da API Luau → `ph2d.d.luau`; `luau-lsp` 1.70.1 fixado (sha256 por SO) |
| `ph2d-text` · `ph2d-a11y` · `ph2d-panel-inspector` | JetBrains Mono embutida, `without_prep_table`, `NodeBuilder::value`, `Tree::{ids,get}` · «Edit script», seletor, erro um degrau acima, chip do culpado |
| `shells/desktop` | só composição: instala `CodeDocs`, chama `ph2d_app_code::frame` (`fase_snapshots_publish`), cede a tecla à aba (`keyboard*.rs`, F9/Morph), `PROJECT_SCHEMA` 190 + braço 189 |
| `.config/nextest.toml` · `FLAKES_DE_CARGA.md` | +1 membro candidato (§6) |

### Crates NOVAS e deps novas

Crates: `ph2d-code-buffer` (rope `crop`, transações invertíveis, multicursor, busca) · `ph2d-code-syntax` (tree-sitter Luau, `AutoEdit`) · `ph2d-panel-code` (pintura, console, lista) · `ph2d-app-code` (família: `CodeDocs`, teclado, salvar→selo, smoke) · `ph2d-script-doc`, `ph2d-code-lsp` (Onda 0).

`Cargo.lock` — 12 pacotes novos, **todos MIT ou MIT/Apache-2.0** (`cargo metadata`): `crop` 0.4.3 MIT · `str_indices` 0.4.4 MIT/Apache-2.0 · `streaming-iterator` 0.1.9 MIT/Apache-2.0 · `tree-sitter` 0.27.1 MIT · `tree-sitter-language` 0.1.9 MIT · `tree-sitter-luau` 1.2.0 MIT + as 6 crates internas.
## 5. Itens E1–E15 (commit · teste que fica vermelho se a propriedade quebrar; o desenho de cada um está em [02_onda1.md](../02_onda1.md))

| # | Commit | Prova (nome) |
|---|---|---|
| E1 buffer | `b3f98d4773` | traços aleatórios aplicar+inverter byte a byte; `Ctrl+D` 3× = 1 passo |
| E2 sintaxe | `661979cca9` | incremental = parse do zero (4 983 passos); fixtura do `bob.luau`; 5 mutações |
| E3 aba + «Edit script» | `760c7a0ff8` `06b292caca` `b1b305cdbd` | `edit_script_pede_a_aba_do_editor`, `sem_script_o_edit_nao_e_clicavel`, `abrir::*` |
| E4 pintura | `2d80b4e71b` `fbf5fed664` | rolar 1 linha molda ≤ 1; editar 1 em 7 500 molda só ela; clique a 1/4 e 3/4 do glifo |
| E5 teclado | `797c072c8d` `cfa452b34e` | `despacho::*` (Esc, IME, acordes alheios, chrome por cima, F9/Morph, Ctrl+Z de campo) |
| E6 salvar→recarga→selo | `16252bed2d` `797c072c8d` | `a_recarga_e_o_console::*` (N=3; quebrado ⇒ continuam com a antiga), `salvar::*` |
| E7 erro + culpado | `9c7ef3aefe` `99246386d9` | `o_simbolo_do_fim_sai_da_frase`, `um_script_partido_pinta_a_linha_do_erro`; oráculo `luau-lsp` 1.70.1 = culpado do tree-sitter |
| E8 auto-fecho/indent | `dc055b96e5` | 95 casos do oráculo Godot 4.7.2 (`ferramentas/godot_oraculo/`), 3 divergências listadas |
| E9 busca · E10 console | `3d523fca04` `e98bf678e7` `16252bed2d` | `tests/it/busca.rs` (substituir tudo = 1 passo); `print` no console e NÃO no stdout |
| E11 lista e seletor | `fd8ad5252e` `9564124e57` | `re_browse_do_mesmo_ficheiro_reusa_o_script`, `lista::apagar_um_script_em_uso_e_recusado_com_frase_no_ecra` |
| E12 a11y · E13 comandos | `2d80b4e71b` `9564124e57` | `a_arvore_do_ecra_tem_um_no_por_linha_visivel_e_rolar_troca_os_nos`; `a_paleta_lista_go_to_line_com_a_aba_activa_e_escolher_abre_o_campo` |
| E14 bancada | `3bd0ba3a55` `eb9a10c2f4` `d91055c97c` | `bancada::tecla_pixel` (`#[ignore]`); `dobras::o_trabalho_das_dobras_por_tecla_nao_cresce_com_o_ficheiro` |
| E15 smoke | `6303a35e30` `9c57816736` | `code_smoke::tests::tres_formas_correm_o_mesmo_script_e_o_quadrado_fica_parado`, `o_nivel_2_abre_o_script_partido_com_a_linha_do_erro` |
| shell encolhe | `5e9ddd17da` | `the_shell_only_shrinks` verde, −66 |

### 5.1 A lei do Ctrl+S (correção do smoke do dono, 16/10, `2a27b4b648`)

Smoke do dono: *«Ctrl+S, em vez de salvar o script já existente, abre o diálogo para salvar»*. **Mecanismo:** o Ctrl+S da aba levantava `hero.file_menu.save` (a bandeira do File ▸ Save); a shell drena-a em `drain_project_io` → `project_save_gesture` → `save_target(None, false)` = `Ask` → seletor nativo (`shells/desktop/src/project_io.rs:165-168`). A cena do `PH2D_CODE_SMOKE` nasce **sem ficheiro de projecto** (`initial_project_path` só lê `PH2D_PROJECT_PATH`) ⇒ o diálogo abria sempre. Com ficheiro (env, Open, Save As anterior) o mesmo caminho já gravava em silêncio — medido: `ctrl_s_com_ficheiro_aplica_e_grava_sem_dialogo` era verde antes da cura; não havia defeito a mais.

**Lei:** o Ctrl+S **da aba de código nunca abre diálogo**. Aplica SEMPRE o texto à biblioteca (recarga + selo, como antes; o ● sai). `ph2d_app_code::frame(.., project_file)` recebe da shell `project_path.is_some()`: com ficheiro pede a gravação (File ▸ Save, silenciosa); sem ficheiro não pede nada e o selo diz **«Script applied · project not saved yet — File ▸ Save As…»** e, quando a recarga chega, **«Reloaded · 3 objects · project not saved yet — File ▸ Save As…»** (tom de aviso; i18n `code.status.project_unsaved` / `_after`; texto composto em `ph2d_panel_code::badge_line`, que a pintura e a sonda `CodeDocs::badge_text_of` partilham). O aviso sai quando o projecto ganha ficheiro. **Ctrl+Shift+S** continua «Save As…» (pergunta). **File ▸ Save / Save As do menu** e o **Ctrl+S com a Cena activa**: iguais (com diálogo sem ficheiro). Provas: `salvar::ctrl_s_sem_ficheiro_aplica_e_nao_abre_dialogo` (vermelho antes: 1 diálogo) · `salvar::ctrl_s_com_ficheiro_aplica_e_grava_sem_dialogo` (o ficheiro em disco tem o texto novo); a `Mesa` ganhou `ficheiro` + `shell_drena` (a política do `save_target`). `salvar::ctrl_s_aplica_o_texto_grava_o_projeto_e_o_selo_conta_os_objectos` passou a correr COM ficheiro (a bandeira só sobe então). Shell: saldo 0 linhas.

**Auditoria do fecho** (2 lentes: costura/correção e estado/robustez; 2 críticos/altos de cada, curados com teste visto vermelho): `4713dc6948` (pânico do autoedit com comentário não-ASCII) · `b30dc3431f` (scripts vazando entre projectos: `LibraryEpoch`; textos por gravar viram ficheiros em `PH2D_SCRIPT_RECOVERY_DIR` ou `$XDG_DATA_HOME/ph2d/recovered-scripts`; Save grava as abas fechadas com ●) · `cfa452b34e` (Esc/IME/Ctrl+K/chrome) · `ef2ea4cff7` (tectos) · `99246386d9` · `1328bdb2ed` · `ad839a1f25` · `065d40edd8`.

## 6. Prova de fecho (gate batched sobre o diff acumulado, 15/10)

- `PH2D_SUITE=fecho BASE=0aab2c1cb6 bash scripts/nextest-impacted.sh` → **18 884/18 884 verdes** (`load` 13–18).
- Clippy do `ship.sh` sobre o workspace verde · `cargo fmt --check` verde · `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` 0 avisos.
- Verdes também: `file_loc_caps`/`fn_loc_caps`, `the_shell_only_shrinks`, `arch_safe_clamp_only`, `architecture_workspace_file_loc_cap`.
- ⚠️ **Flake de carga visto**, confirmado sozinho 3×: `ph2d-script` `it a_recarga_e_o_console::o_console_cheio_deita_fora_a_mais_antiga_e_conta` (256 `print` por quadro; estoura o prazo de 16,7 ms do script só a `load 60–127`). Membro **candidato** em [FLAKES_DE_CARGA.md](../../DevOps/FLAKES_DE_CARGA.md), com as DUAS escritas (tabela + `filter` da faixa em `.config/nextest.toml`).

## 7. O que um merge pode partir (leia ANTES de fundir)

1. **`shells/desktop/src/render_loop/motores_do_quadro.rs` foi APAGADO** (238 linhas, de OUTRAS linhas: scripts, partículas, vigias, gatilho, armas, barras) e movido inteiro, pela mesma ordem, para `ph2d_app_components::motores`; os cursores passam como `motores::Leitores { script, particles, weapon }`. A CHAMADA continua no corpo de `fase_signal_outbox`. **Linha que edite `motores_do_quadro.rs` ⇒ conflito modify/delete**: reaplique o diff em `crates/ph2d-app-components/src/motores.rs`. Os gates `o_cerebro_fala_antes_de_a_tabela_ouvir` e `as_barras_de_vida_correm_desenham_e_renascem` leem agora o caminho novo por `include_str!`.
2. **Board — sítios tocados** (a `line/MiroClone` mexe aí): `crates/ph2d-editor-core/src/screens/hero/board_view.rs` (+14), `board_keys.rs` (+5), `document_tabs.rs` (+152/−4; as abas de família partilham a geometria) e, na shell, `keyboard_board.rs` (a porta devolve `false` só sem quadro E sem aba de família), `keyboard.rs`, `keyboard_modal.rs`. Depois de fundir, corra: `only_the_mode_owns_the_tab`, `the_keyboard_goes_through_the_same_door_as_the_menu`, `morph_arrow_seam_tests::the_arrow_click_reaches_the_world`.
3. **`PROJECT_SCHEMA` e ADR-0188**: recontagem em §3.
4. **`ph2d-panel-registry-init` / `ph2d-app-registry-init`**: regere com `cargo run -p ph2d-panel-sync` / `cargo run -p ph2d-app-sync` (a `ph2d-panel-code` NÃO entra: é paint-only). `Cargo.lock`: regere. `tokens.json`/`ph2d-tokens`: reconte os 13 tokens `code-*` nos 12 temas se outra linha acrescentar tokens.

## 8. Smoke para o DONO (a janela abre sozinha com uma cena própria)

1. Cole no terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Aparecem quatro formas: três a subir e descer (círculo azul, estrela amarela, hexágono roxo) e um quadrado cinzento parado. Na lista da esquerda (Hierarchy) clique em **Circle** (ou Star, ou Hexagon). À direita, no Inspector, procure o cartão **Script** e clique no botão **Edit script**.
3. O que tem de acontecer: abre uma aba **bob.luau** com o texto do script. Na linha de `amplitude`, troque `1.0` por `2` e carregue **Ctrl+S**: **não abre janela nenhuma**; as três formas passam a subir mais alto sem reiniciar, o ● some do nome da aba, e a barra de baixo, à direita, diz «Reloaded · 3 objects · project not saved yet — File ▸ Save As…» (esta cena ainda não foi gravada num ficheiro: o script já está aplicado, o projeto inteiro grava-se pelo menu File ▸ Save As…; depois disso o Ctrl+S grava tudo sem perguntar e o aviso some). Depois apague um `end`, **Ctrl+S**: a cena continua a mexer com a versão anterior, a linha fica marcada com um ponto vermelho e o símbolo culpado fica sublinhado.
4. Deu errado se: o Ctrl+S abre uma janela a perguntar onde salvar; o botão não abre nada; as formas param; o app fecha; as teclas que escreve fazem coisas na cena (play, mover, apagar); ou o `end` apagado faz as formas pararem.
5. Segundo nível, `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=2 cargo run -p ph2d-host-desktop --profile smoke`: já abre a aba **script_quebrado.luau**, com a linha 3 marcada, o segundo `=` sublinhado e «1 errors» na barra de baixo; a lista de scripts fica à esquerda dentro da aba (bob.luau e o quebrado). Deu errado se a aba não abrir sozinha ou o `=` não estiver sublinhado.

Fotos: [code_smoke_1.png](../ferramentas/smoke/code_smoke_1.png) · [code_smoke_2.png](../ferramentas/smoke/code_smoke_2.png); nomes conferidos com `crates/ph2d-app-components/src/code_smoke.rs`.

## 9. Números MEDIDOS (tabelas em [02_onda1.md](../02_onda1.md) §Medições (E14) e §Medições do fecho)

- **Tecla→pixel** (RTX 5060 Ti, `smoke`, cena a tocar): a 7 500 linhas, ANTES da cura das dobras p50 `9.5–10.4` ms / p99.9 `20.4–23.3`; DEPOIS **p99.9 ≤ 4.50 ms em todas as células** (~43 linhas `3.05`). Orçamento: **p99.9 ≤ 5 ms; recurso = quadro a 120 Hz (8.33 ms)**.
- **Critério de morte NÃO disparou** (paint de 100 linhas `0.56–1.17` ms): cache por linha, sem atlas de glifos.
- **Fatin (51 × 7 500):** ANTES reprovada (×7); DEPOIS ~43 linhas p50 `1.29 → 1.77` (×1.4), p99.9 `2.94 → 3.05` — o resto é o reparse do tree-sitter. ABERTO (§10).
- **Ctrl+S → selo:** `0.05` ms (51 linhas) / `3.07` p50, p99.9 `12.1` (7 500).
- **Tectos do fecho:** linha do console 4 KiB · console 16 MiB · linha longa 2 000 B · histórico 32 MiB/documento · busca fechada 0 B por tecla (antes `215 289`).

## 10. ABERTO (a Onda 2 e além; conferido contra `02_onda1.md` e o handoff da Onda 0 — o fechado não se repete)

Da Onda 1:
- O reparse do tree-sitter ainda cresce com o ficheiro: `+0.48` ms p50 a 7 500 linhas; `2.1` ms com ~1 000 chamadas `ph2d.property` de topo (**causa por medir**). Cura prevista: reparse fora da tecla com o LSP assíncrono sobre um instantâneo da rope (síncrono só no Enter e no `end`).
- Mensagens escritas à mão para os erros frequentes (retorno do dono, parte 2) — pede a medição de frequência.
- LSP no quadro (diagnóstico ao digitar, completion, hover): o cliente é bloqueante.
- Marcador do undo global apontando a transação (só tem consumidor quando uma acção toca script E cena).
- O adaptador de SO do AccessKit **não está linkado** (lacuna do app): `sync_a11y` não tem chamador em produção.
- O seletor do cartão mostra no máximo 32 scripts; renomear/apagar/criar pela lista ficam fora do Ctrl+Z; Fold All sem atalho.
- Linha muito longa: o custo do quadro cresce até ao tecto de 2 000 B (a fila preparada devia ficar em cache entre quadros); a coluna da barra de estado numa linha de 1 MB conta-se por tecla.
- O número de cenas do smoke (2) escrito em dois sítios sem gate.

Pendentes da Onda 0 que continuam: IME no Board (`TextEdit` sem `Commit` em X11), no texto vetorial e no Combobox/NumberInput · catálogo MCP `attach_script` (descreve bytecode) · `drain_spawns` sem chamador · `tokens.json` com `font-mono` velho · `ph2d-text` com `tests/` solto · `pcall` vs orçamento (Onda 4) · falhas de `ph2d.set` sem linha · `luau-lsp`: ficar no solver antigo, re-medir a cada versão.

Fechado nesta onda (saiu da lista da Onda 0): seletor de scripts existentes, re-Browse que criava «bob 2.luau», cartão que dizia «file», o `=` cru na mensagem (símbolo em chip, sublinhado no sítio).
