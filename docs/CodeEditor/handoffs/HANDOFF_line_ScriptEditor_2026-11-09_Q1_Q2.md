# Handoff de continuação — `line/ScriptEditor`, plano 14 Ondas Q1 + Q2 (09/11)

> Plano: [14_plano_quadros.md](../14_plano_quadros.md) · decisão do dono 13 · maquete
> [14_quadros_do_depurador.html](../maquetes/14_quadros_do_depurador.html). Base: Onda B fechada
> ([handoff](HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-09_ONDA_B.md), não reaberta). **Não integrado, sem push.**
> Smoke do dono de Q1+Q2: **pendente** (cena 14).

## 1. O que entrou

Q1 e Q2 num só commit: a tabela sem um quadro que a use não se fotografa nem se prova ao dono.

- **Motor (`ph2d-script-debug`)** — `Var { name, value, kind: ValueKind, fields, field_count }`; `ValueKind` = o tipo
  Luau (re-exportado por `ph2d_script`). A paragem desce `Var::MAX_DEPTH = 3` níveis de tabela, `Var::MAX_FIELDS = 24`
  campos por tabela (nomes por ordem, depois `[i]` por número; o recurso é o tempo do gancho com o jogo parado). Os
  valores ao vivo NÃO descem (`locals_at(l, 0, 0)`). ⛔ **Defeito achado e curado:** descer a uma tabela no fio parado
  por um ERRO empurrava valores dentro dos registos da função (o Luau não desenrola a pilha) e pisava as locais
  seguintes (`hp` lia-se como o `self.id`) — a descida corre agora numa cópia no fio PRINCIPAL (`ffi::on_aux`, `lua_xpush`).
  Gate: `depurador_pause_on_error_para_no_erro_com_as_variaveis` (+ `dt == 0.5`); prova de mutação: com `aux = t` reprova.
- **`DebugStop.queued: usize` → `waiting: Vec<Entity>`** (quem espera, pela ordem; `waiting.len()` é a contagem de antes).
- **Crate nova `ph2d-ui-table` (Folha)** — a tabela da casa: `Column`/`Row`/`Cell`/`Ink`, `TableState` (larguras
  arrastadas, colunas escondidas, dobras por chave, rolagem, arrasto), `layout()` → `Layout` (a mesma geometria pinta e
  recebe: `Layout::hit` → `Resize | Header | Toggle | Cell`), `natural_widths`, `natural_height`, `paint` (listra →
  âmbar/rato → texto; divisórias; cabeçalho; barra de rolagem), `paint_spark`/`spark_points` (para a Q3),
  `changed_tint` (o âmbar a `ROW_SELECTED_ALPHA`: o `WarnSoft` é superfície neutra neste tema — foto). O texto vem pelo
  trait `CellText` do anfitrião (`UiText` na casa; `ph2d_panel_code::table_text::CodeCells` na mono do código, com a cor
  do realce e o âmbar por `Palette::tint`). Testes: `ph2d-ui-table/tests/it` (8).
- **Crate nova `ph2d-code-debugcard` (Folha)** — o modelo saiu da `ph2d-code-assist` (`debug_view.rs` → `model.rs`):
  `DebugCard { state, title, line, function, who: WhoTag, waiting: Vec<WhoTag>, error, locals, this: Vec<VarRow>, stack,
  can_step }`, `VarRow` achatada com caminho estável (`self.pos.x`, `self.trail[1]`), `StepMemory` (o «mudou» = valores
  diferentes do MESMO objecto+script; outro objecto recomeça), `CardStyle` (Sheet por omissão), `sheet_rows` (grupos
  `Locals · N`/`self · N fields`; uma tabela com filha mudada abre-se sozinha), `columns`, `value_ink`, `type_text`.
  Tecto na catraca da família: 663 (medido 563).
- **Pintura `ph2d-panel-code::debug_card`** (`mod.rs` estado/clique, `head.rs` faixa, `sheet.rs`, `boxes.rs` Cards e Strip)
  — `DebugCardView { style, table, painted, revealed }`: `height`, `paint`, `press → CardClick (Ask | Jump | Copy |
  Taken)`, `drag_to`, `wheel`, `header_at`, `column_menu`, `chevron_of` (sonda). Um passo novo com uma linha âmbar fora
  de vista rola até ela (`revealed` = `linha:chave`). Quadro até ½ da aba (`CARD_SHARE`), mínimo 4 linhas.
- **Cola `ph2d-app-code`** — `View ▸ Card Style: Sheet · Cards · Strip` (`CMD_CARD_SHEET/CARDS/STRIP`, a escolhida com ✓,
  estado `CodeDocs::card_style`, como o Peek/Map: em memória); `debug.rs` monta o `WhoTag` com a cor da Bancada
  (`bancada.who.holders` pelos bits) e a `StepMemory`; o clique na pilha vai à linha; clicar num valor copia
  (`copy_out`); o botão direito no cabeçalho abre o menu das colunas (`context_menu::Row::Column`). `debug_card.rs`
  ficou cola (50 linhas): a `ph2d-app-code` estava a 33 linhas do tecto — está em 26 627 / 26 628.
- **Cena `PH2D_CODE_SMOKE=14`** — o MESMO `depura.luau` da 12 (ganhou `self.pos`, `self.trail`, `self.label`, `self.alive`
  e a linha `self.pos.x = …`; a cena 12 não muda de passos), parado em `self.t = self.t + dt` (`QUADROS_BREAK`).
  `STEPS_14`: 1 Sheet · 2 Next line (`self.t` âmbar) · 3 Next line ×2 (`self.pos` abre, `x` âmbar) · 4 Cards · 5 Strip ·
  6 Sheet + menu das colunas. Gestos novos: `ClickTabMenuRow`, `ClickDebugRow`, `RightClickDebugHeader`; `Until::Changed`.

## 2. Provas

| Prova | Resultado |
|---|---|
| `ph2d-ui-table` it (geometria: dobras, flex, puxador com mínimo, esconder Type, ▸ vs célula, rolagem, traço) | 8/8 |
| `ph2d-code-debugcard` it (caminhos estáveis, «mudou», abrir sozinha, `· 3 fields`) | 2/2 |
| `ph2d-script` `depurador_*` (24, com o novo `…_tipo_e_os_campos_das_tabelas_aninhadas`) | 24/24 |
| `ph2d-app-code` `depurador`, `barra_da_aba`, `onda1`, `botao_direito`, `o_nivel_le_se` — inclui o seam `o_card_style_troca_o_stopped_e_o_cabecalho_esconde_a_coluna_type` (cliques reais: ▸, botão direito, «Type») | verdes |
| `ph2d-app-components` `os_scripts_da_cena`, `every_word`, `a_cena` · `ph2d-panel-code` `bancada` | verdes |
| `ph2d-editor-core`: `shell_only_shrinks`, `workspace_file_loc_cap`, `panel_loc_cap`, `panel_wiring_parity`, `no_dependency_climbs_a_layer`, `every_member_inherits`, `hr15`, `no_literal_color`, `no_magic_numeric` | 23/23 |
| Fotos `ferramentas/smoke/cena14/passo_0..6.png` + `maquete_lado_a_lado.png` (A, B, C da maquete × app) | feitas e conferidas |

Não corrido (fecho da linha): clippy das crates tocadas, `PH2D_SUITE=fecho`, auditoria.

## 3. Diferenças conhecidas da maquete (decididas, não esquecidas)

- O âmbar do tema é pálido (`Warn` = 212,199,158 medido na foto); a maquete usou um amarelo mais quente. É o token.
- A coluna Type diz o tipo Luau; a maquete escreveu `object · Circle` para o `self.id` (um número) — o motor não sabe
  que um número é um objecto. Fica para quando o `ph2d.get`/ids tiverem tipo próprio.
- Strip só mostra o nível de cima (sem `self.pos.x`), como a maquete.

## 4. O que falta (plano 14 §5)

- **Q3** — Live values em tabela (Line · Name · Value · Last 4 s) + anel de histórico por linha+nome no `Debugger`
  (amostrado a 2,5 Hz, `Duration::Slow`, como os fantasmas), pairar = mín/máx/actual, clicar = gráfico grande por baixo
  da linha. Peças prontas: `Column::spark()`, `Cell::spark`, `paint_spark`, `spark_points`. Hoje o Live mostra só a faixa
  (estado verde + objecto).
- **Q4** — balão ◆ (`ph2d-panel-code/src/bancada/popover.rs`), Script options (`ph2d-app-code/src/options_card.rs`) e
  Problems (`ph2d-panel-code/src/console/problems.rs`) na mesma família; a grelha da Bancada (`bancada/grid.rs`) migra
  para a `ph2d-ui-table`.
- ⚠️ **Tecto da `ph2d-app-code`: 1 linha de folga.** Q3/Q4 têm de pôr o código novo no `ph2d-panel-code` (sem tecto de
  crate, 600/ficheiro) ou numa Folha, e cortar na cola.
