# HANDOFF DE CONTINUAÇÃO — line/CodeEditor, Onda 2: I9 + I12 + I14 (18/10/2026)

> Leitor: a próxima LLM da linha (e, depois, o integrador). **Não integrado, não enviado.** O último
> handoff de INTEGRAÇÃO é o do [I0–I5](HANDOFF_INTEGRACAO_line_CodeEditor_2026-10-17_ONDA2_I5.md), já no
> `main`; I6/I11, I7, I8, I8b e [I10](HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-18_ONDA2_I10.md) estão na
> linha (links nele); este vem por cima. Desenho e provas: [03_onda2.md](../03_onda2.md) §Estado (linha I9+I12+I14) · medição:
> [I9_I12_I14.md](../ferramentas/medicoes_onda2/I9_I12_I14.md). O mecanismo vive nesses; aqui só o resumo.

## 1. Identidade

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor` · branch `line/CodeEditor` ·
  merge-base com o `main` `100d34c8e0` · HEAD `f4a5305ae8` (antes do commit dos docs). A linha foi
  **rebaseada sobre o `main` nesta sessão** (44 commits da linha, sem conflito).
- Commits da onda (sobre `b3c7623e13`), **7**: `cf4b9ca003` docs (medição) · `e9d876f97f` feat ·
  `a0783807b2` test+fix · `a6bda82c38` test · `10818ac113` test+simplifica · `ba94dce6e2` test (M11) ·
  `f4a5305ae8` style (clippy).

## 2. Superfície de colisão

```
merge-base 100d34c8e0 · 44 commit(s) da linha
PROJECT_SCHEMA = base (sem degrau) · registo de componentes = base · ADR: nenhum · cena de smoke: nenhuma
Cargo.lock +0
```

**Contadores como DELTA desta onda:** nenhum (sem schema, registo, ADR, cena de smoke; `Cargo.lock` +0).
Comandos da tabela `TABLE`: **+2** (9 → 11): `CMD_TYPE_HINTS` «Show Type Hints»/«Hide Type Hints» e
`CMD_STRICT` «Turn On/Off Strict Types (--!strict)». `Ink::ALL` 15 → **16**.

**Shell:** 3 ficheiros, **+11 −16** (encolhe): `command_palette_input.rs` e `global_palette_input.rs` delegam
no `ph2d-editor-core`; `undo_tests.rs` só `cargo fmt`.

**FOUNDATIONAL tocado (`ph2d-editor-core`, aditivo):**

- `doc_surface.rs`: `DocSymbolKind`, `DocSymbol{name, container, kind, line, col (chars)}`,
  `DocPlaces{symbols, lines}`; `DocumentSurface::places(&mut self, id)` e `go_to(id, line, col)` com
  **implementações por omissão**.
- `widget/command_palette/scope.rs` (novo): `PaletteScopes{symbols, line: LineScope}`, `Scoped`, `scoped()`,
  `line_target`, `ScopeState`.
- `WidgetStore.command_palette_scope` (campo novo; `state/mod.rs` **697/700**, `store_core` +1);
  `chrome_ops.rs`: `set_command_palette_scopes` / `command_palette_scopes` / `command_pick_query` /
  `command_palette_has` / `command_palette_top`; `set_command_pick` guarda a consulta (a paleta já está
  fechada quando o pick é drenado); `chrome/command_palette.rs` (clique usa `command_palette_has`,
  **463/500**).
- `screens/hero/global_palette.rs` (`open()`/`drain()` + `route_global_pick` roteia os prefixos primeiro) e
  `global_palette_places.rs` (novo: `PALETTE_GOTO_LINE`, ids de símbolo = `SYMBOL_BASE ^ (i+1)`, máx 2^20,
  `scopes()`, `route()`).

## 3. O que a onda fez (resumo; mecanismo na medição e no código)

- **Medição** (`cf4b9ca003`; `ferramentas/medicoes_onda2/I9_I12_I14.md`, fotos `I9_I12_I14/`, bruto
  `I9_I12_I14_bruto.json`; oráculo `ferramentas/oraculo_i9_i12_i14/corre.sh` — VS Code 1.142.0, Xvfb `:119`,
  `ORACULO_SO` escolhe as cenas; sonda `ferramentas/lspprobe_i9_i12_i14.py`, `luau-lsp` 1.70.1).
  **`luau-lsp`:** `bob.luau` dá 9 dicas de tipo, todas `kind` 1, `padding` falso, e só 4 passam a lei do
  «add type» do I7; **nenhum item de gancho** no topo nem depois de `function ` (0 itens); os esqueletos
  anotados e preenchidos dão zero erros de tipo em normal e `--!strict` (só `FunctionUnused`, já escondido por
  `problems::engine_hook`); o `documentSymbol` não traz `ph2d.property`. **VS Code:** a dica não tem coluna
  (cursor antes, a seta salta-a, a selecção fica depois, escrever empurra-a); snippet: 1.º buraco
  seleccionado, espelhos = cursores, escrever na ponta estica, Tab/Shift+Tab, `$0` acaba, Esc/sair acabam, a
  lista aberta ganha ao Tab, todos os preenchimentos de uma sessão = 1 passo de undo; Ctrl+P: `@` por ordem do
  ficheiro com contentor, `@:` agrupado por tipo, `:N` fixa na última linha, `>` só comandos. **Godot** não
  tem dicas nem paragens.
- **I12 — esqueletos com buracos** (`ph2d-app-code`): `snippet.rs` (gramática de snippet LSP 3.17 completa →
  `Expanded{text, stops, end}`; `Expanded::adapt` = recuo da linha + `\t`→unidade; `Tabstops`: o buraco
  activo cresce nas duas pontas, os inactivos não, os vazios andam; `next/prev/holds/marks`) · `snippet_doc.rs`
  (`insert_snippet` num passo de undo para todos os cursores; `snippet_key` Tab/Shift+Tab/Esc;
  `snippet_follow` acaba a sessão quando o cursor primário sai de todas as inserções) · `skeleton.rs`
  (`params()` lê tipos de função Luau; `hook_snippet` de `api_table::HOOKS` com `${n:param}: Tipo` e corpo
  `$0`; `call_snippet` para toda entrada da API — parâmetros obrigatórios como buracos, tipos textuais
  (`string` ou união de literais, p.ex. `PoseField`) entre aspas; `skeletons()` só no topo, no início da
  instrução ou depois de `function `, só ganchos por declarar + `ph2d.property`; `local_items`; também sem
  assistente via `Completer::offline`). `complete.rs`: `expand` removido; `insertion()` devolve `Expanded`;
  `answer()` recebe os itens locais e troca o snippet do servidor por `call_snippet` nas chamadas da API.
  `ph2d-code-buffer`: `Buffer::jump_selection` + `History::carry` (o salto de buraco não parte o grupo de
  escrita). `edit.rs`: `undo_text` acaba a sessão. `ph2d-code-syntax`: `Syntax::at_top_level` (entre o
  INÍCIO dos parâmetros e o `end` — medido: numa árvore só deslocada o tree-sitter estica o `)` por cima do
  texto escrito depois, `parameters 13..25`; era um flake de 2 em 15, hoje teste unitário), `Syntax::symbols`
  (funções a qualquer profundidade com nome completo `M.soma`/`M:metodo`, locais de topo, `ph2d.property`;
  `Symbol`/`SymbolKind` exportados), `declares_global_function` agora `pub`.
- **I9 — dicas de tipo** (`ph2d-panel-code`): `display.rs` `ExpKind{Tab, Preedit, Inlay}`, expansões de zero
  bytes, preedit antes da dica na mesma posição, `to_disp_after`, `is_inlay`, `inlays()`; `geom.rs`
  `x_of_byte_after`, `byte_at_x` mantém a dica como coluna própria (mesmo dono da coluna seguinte ⇒ clicar em
  qualquer ponto da dica dá a posição dela); `paint.rs` dicas por linha, `display_spans` começa depois da
  dica + spans `Ink::Hint`, pílula (`pal.pill`), `seg_rect` depois da dica, paragens do snippet riscadas
  (`pal.bracket`, fino); `view.rs` `inlay_cols` (`x_in_line` sem cache); `style.rs` `Ink::Hint` (`Text2`); `frame.rs` `pub struct Inlay{at, label}` e `CodeFrame` ganhou `inlays`, `stops`,
  `strict`. `ph2d-app-code`: `inlay.rs` `InlayHeld` (as edições deslocam por `map_byte` — escrever na posição
  empurra a dica; sai quando a posição fica estritamente dentro de um intervalo apagado; a resposta passa por
  `actions::insertable`, agora `pub(crate)`); `intel.rs` `inlays_on` (**desligado por omissão**) +
  `ask_inlays` na pausa do I11 (uma pergunta por (arranque, versão)); `goto.rs` `owns/answer` roteia o bilhete.
- **I14 — paleta `@ : >` e `--!strict`**: o núcleo é o foundational do §2. Prefixos: sem prefixo = tudo +
  símbolos que casam quando se escreve algo; `>` só comandos; `@` só símbolos agrupados por tipo (Enter
  sozinho = o 1.º símbolo MOSTRADO, o cartão Functions primeiro); `:N[:C]` fixa na última linha, senão uma nota
  de ajuda (a paleta mostra-a em vez de «No matches»). `ph2d-app-code`:
  `places.rs` (`tab_places` com `syntax.fresh`, `tab_go_to` em chars), `docs.rs` delega `places/go_to`
  (**693/700**: `commands.rs` ganhou `tab_commands/family_command`), `strict.rs` (`directive_line`:
  comentários do topo antes do 1.º código; `toggle_strict` num passo de undo), barra «Strict types»
  (`StatusLine.mode`), `commands()` recebe `CommandState`. **i18n:** `code.cmd.type_hints(_off)`,
  `code.cmd.strict(_off)`, `code.complete.hook`, `code.complete.skeleton`, `code.status.strict`;
  `chrome.palette.symbols*`, `no_symbols`, `no_lines`, `goto_line*`, `line_prompt`.

## 4. O que um MERGE pode partir

- **`DocumentSurface`** ganhou 2 métodos com implementação por omissão (impls fora da linha compilam).
- **`CodeFrame`** ganhou `inlays`/`stops`/`strict`: construtores fora da linha reprovam a compilar.
- **`commands::commands()`** recebe `CommandState`: chamadores fora da linha quebram; a contagem da `TABLE` é 11.
- **`Ink::ALL`** tem 16 elementos (testes de contagem de tintas).
- **`WidgetStore`** ganhou campo; `state/mod.rs` a **697/700** — outra linha que acrescente campo ali estoura o tecto.
  `command_palette.rs` a 463/500 (tecto do widget).
- **Aceitar um item da lista agora insere buracos**: qualquer teste noutra linha que espere `name()` falha
  (dois desta linha foram actualizados: `completar::a_lista_pinta_se_clica_se_e_rola`,
  `dicas::aceitar_uma_funcao_da_lista_abre_a_assinatura`).

## 5. Testes e mutação

- `crates/ph2d-app-code/tests/it/esqueletos.rs` **8**: `o_gancho_entra_com_o_1o_buraco_seleccionado_e_o_tab_anda` ·
  `escrever_na_ponta_do_buraco_fica_dentro_dele` · `esc_e_sair_do_modelo_acabam_a_sessao` ·
  `so_os_ganchos_por_declarar_e_depois_de_function_so_o_resto` · `cada_gancho_da_tabela_tem_esqueleto` (gate da
  tabela) · `a_propriedade_do_topo_e_a_chamada_da_api_poem_o_texto_entre_aspas` ·
  `com_a_lista_aberta_o_tab_e_da_lista` · `desfazer_pelo_menu_acaba_a_sessao`.
- `dicas_de_tipo.rs` **3**: `desligadas_por_omissao_e_o_comando_liga_e_desliga` ·
  `a_dica_nao_tem_coluna_o_cursor_a_seleccao_e_o_clique` · `escrever_na_posicao_da_dica_empurra_a`.
- `paleta.rs` **4**: `os_simbolos_e_as_linhas_do_script_activo` · `arroba_leva_ao_inicio_do_nome_e_dois_pontos_a_linha`
  · `strict_poe_e_tira_a_linha_num_passo_e_o_rotulo_diz_o_que_faz` · `clicar_num_simbolo_da_paleta_leva_la`.
- `oraculo_lsp::os_esqueletos_preenchidos_passam_limpos_normal_e_estrito`.
- Unitários: `snippet` 2 · `skeleton` 2 · `strict` 1 · `facts` 3 novos (símbolos, topo, árvore só deslocada) ·
  `display` 1 · `scope` 1 · `global_palette_places` 1 · `estado::o_modo_estrito_aparece_na_barra`. Servidor falso:
  `Plano.tipar` (inlayHint). Sondas em `docs_probe.rs`: `selection_of`, `snippet_of`, `inlays_of`,
  `caret_rect_at`, `segments_of`.
- **Mutação: 22 mutações; 20 sangraram na 1.ª rodada.** M11 (`undo_text` sem acabar a sessão) sobreviveu ⇒
  teste novo `desfazer_pelo_menu_acaba_a_sessao` (agora sangra). M21 (`directive_line` com `starts_with("--!")`)
  está coberta pelo unitário `strict::tests::a_diretiva_so_conta_antes_do_codigo` (fora dos filtros da
  rodada). M17/M18 só sangram no teste de lib do editor-core (`scope::tests::os_prefixos_estreitam`).
- Vizinhos verdes por nome: app-code `it` 142+ · editor-core lib 124 + `it` 55 · gates de arquitectura +
  `the_shell_only_shrinks` 111 · host-desktop `file_loc_caps`/`fn_loc_caps`/`the_app_only_sheds_fields`.
- Corrida: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && bash scripts/ph2d-run.sh cargo test -p ph2d-app-code --test it -- esqueletos:: dicas_de_tipo:: paleta::`

## 6. Portões da onda (18/10)

- `cargo fmt`; clippy `-D warnings --all-targets` em `ph2d-app-code`, `ph2d-panel-code`, `ph2d-code-syntax`,
  `ph2d-code-buffer`, `ph2d-editor-core`, `ph2d-i18n`: **verde** (curado: ramos iguais no `map` do snippet; a
  `Range` única nomeada do `refs.rs` do I10; testes comparam `(início, fim)`; tipo `Tipos` no servidor falso).
- Suíte `PH2D_SUITE=onda nextest-impacted` (900 s): 19 695/19 698; 3 ✗ curados em `04a42af43f`: `arch_safe_clamp_only` (`.clamp` cru no `:N` e no rename do I10 → `safe_clamp_ord`), `the_undo_preserves_the_vector_selection` (agulha `push_undo(base, mark)` → `push_linked(base, mark,` do I10), `light2d a_luz_recua…` (placa sob carga: sozinho verde, load 6,4 — não é desta linha).

## 7. Binário do smoke

`bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida: `Finished \`smoke\` profile [optimized] target(s) in 0.40s` (zero `Compiling`).

## 8. Smoke para o DONO

> Precisa do assistente instalado (`bash scripts/luau-lsp-fetch.sh`) para as dicas de tipo; os modelos e a paleta funcionam sem ele. ⚠️ Não fotografado.

1. Cole no terminal: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor && PH2D_CODE_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`
2. Clique no **Circle** e, no cartão **Script**, em **Edit script**: abre a aba do `bob.luau`.
3. **Dicas de tipo:** carregue **Ctrl+K**, escreva `hints` e clique em **Show Type Hints**. Num instante aparecem, em cinzento sobre um fundo suave, `: number` depois de `local s`, `: boolean` depois de `local up` e `: ()` no fim de `function init(self)` e `function update(self, dt)`. Clique em cima de uma dica: o cursor fica logo ANTES dela; a seta → salta-a inteira. Ctrl+K → **Hide Type Hints** esconde-as.
4. **Modelos com buracos:** carregue **Ctrl+End** (fim do script), **Enter**, e escreva `on_`. Na lista aparece **on_signal** («engine hook»); carregue **Tab**: entra `function on_signal(self: ScriptSelf, signal: string)` com `self` seleccionado e o `signal` numa caixa fina. **Tab** de novo: `signal` seleccionado. **Tab** outra vez: o cursor vai para dentro da função; escreva `print(signal)`. Na linha 1 (Ctrl+Home, Enter, seta ↑) escreva `prop` e Tab: entra `ph2d.property("name", default)` com `name` seleccionado dentro das aspas.
5. **Paleta com atalhos:** **Ctrl+K** e escreva `@upd` + **Enter**: o cursor salta para o nome `update`. **Ctrl+K**, `:30` + **Enter**: vai à linha 30. **Ctrl+K**, `@` sozinho: os cartões **Functions**, **Properties** e **Variables** do script. **Ctrl+K**, `>` sozinho: só comandos (sem os símbolos).
6. **Strict:** **Ctrl+K**, escreva `strict` e clique em **Turn On Strict Types (--!strict)**: a 1.ª linha passa a `--!strict` e a barra de baixo diz **Strict types** (podem aparecer avisos novos — é o modo estrito a falar). **Ctrl+Z** tira a linha.
7. **Deu errado se:** clicar numa dica põe o cursor no meio do nome ou depois do espaço; o Tab do passo 4 escreve espaços em vez de saltar; o `Ctrl+Z` depois de preencher precisa de vários toques para tirar o que escreveu nos buracos; `@upd` não sai do sítio; o `--!strict` não aparece ou a barra não muda.

> ✅ **Smoke do dono OK a 18/10.**

## 9. ABERTO

- O interruptor das dicas de tipo vale **por sessão** (o app não tem armazém de preferências).
- A pintura do I9 (pílula, tinta `Text2`) e a caixa fina das paragens **não foram fotografadas**; contraste por medir.
- `@` não tem prévia atrás da paleta (a nossa é um cartão modal sobre um véu — diferença declarada do VS Code).
- `DocSymbol.container` é sempre `None` para Luau (o nome já leva a tabela: `M.soma`).
- Um snippet aninhado **substitui** a sessão de fora (o VS Code mantém as duas).
- O hit-test de linhas sem cache ignora as dicas (só linhas pintadas são clicáveis; o `x_in_line` já as conta).
- Dos I10/I8b: contraste `Bg3`; Shift+F12 vazio calado; botões do balão sem foco de teclado; Ctrl+rato em
  mão e lista de problemas (acabamento).
- Integração: falta `clippy --workspace` e a suíte do fecho.

## 10. Próximo

**I13** (StyLua fixado: crate `ph2d-code-format`, manifesto; o kill-criterion já está declarado em
[03_onda2.md](../03_onda2.md) I0d: Ctrl+S aplica e dá o selo no quadro, o StyLua corre fora e entra como UM
passo de undo só se o texto não mudou) → **I15** (cena `PH2D_CODE_SMOKE=3` fotografada; bancada cheia com a lista
E os balões abertos; solver velho × novo). Acabamento: Ctrl+rato em mão e lista de problemas. Antes de integrar:
o smoke do dono desta onda e `/pd-linha-fechar`.
