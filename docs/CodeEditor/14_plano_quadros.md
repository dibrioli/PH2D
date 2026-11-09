# Plano 14 — os quadros acessórios do editor de scripts (09/11)

> Decisão do dono 13 ([00_decisoes_do_dono.md](00_decisoes_do_dono.md)). Maquete aprovada no rumo:
> [maquetes/14_quadros_do_depurador.html](maquetes/14_quadros_do_depurador.html) (abrir no navegador; tem a captura de hoje e as três variantes).
> Base: `line/ScriptEditor` com a Onda B fechada ([handoff de integração 09/11](handoffs/HANDOFF_INTEGRACAO_line_ScriptEditor_2026-11-09_ONDA_B.md)).

## 1. O pedido

O dono (captura do cartão «Stopped at line 14 — on Circle»): *«quadros como esse podem ser muitíssimo melhor organizados e
desenhados… quem sabe um aspecto de planilha?»* → depois da maquete: *«implementar as 3 com escolha da aparência no menu. (A)
deve ser o padrão inicial»*; tipo e traço: *«busque o estado da arte sem olhar custos»*; âmbar para «mudou»: *«serve»*.

## 2. Os quadros e onde moram hoje (inventário de 09/11)

| Quadro | Dados | Pintura |
|---|---|---|
| Stopped | `ph2d-code-assist/src/debug_view.rs:23` `DebugCard` (`vars: Vec<(String,String)>` já escritos, sem tipo; `stack: Option<String>`; `waiting: Option<String>`) | `ph2d-app-code/src/debug_card.rs:57` (`var_grid` :32, colunas iguais com quebra) |
| Live values | o mesmo `DebugCard` com `stopped=false` (só o título) + os fantasmas no fim da linha (`debug.rs::ghosts`) | `debug_card.rs:78-91` |
| Balão ◆ «em quem» | `ph2d-code-viewmodel/src/bancada.rs:54` `Who` | `ph2d-panel-code/src/bancada/popover.rs:29` |
| Script options | `ph2d-app-code/src/options_card.rs:37` | `options_card.rs:91` |
| Problems | `ph2d-panel-code/src/console/problems.rs:21` `ProblemRow` | `problems.rs:76` |

Molde comum: `ph2d-panel-code/src/cards.rs` (`open`/`close`/`floor`). Grelha da casa: `bancada/grid.rs` (`Cols::of`, realce de
linha/coluna, célula alterada) — sem cabeçalho fixo nem colunas redimensionáveis. Não há tabela genérica em `ph2d-editor-core`:
**construí-la** (`ph2d-editor-core/src/widget/table.rs` ou crate `ph2d-ui-table`), com a grelha da Bancada a migrar para ela.

## 3. Estado da arte (decisões técnicas, delegadas ao padrão-ouro)

| Pergunta | Referências | Decisão |
|---|---|---|
| Coluna Type | Visual Studio (Locals/Autos/Watch: Name · Value · Type sempre, colunas redimensionáveis e ocultáveis); JetBrains (`{type}` cinzento na linha); VS Code (tipo só no hover); Chrome DevTools (sem coluna, valor colorido por tipo) | **Sempre visível, ocultável pelo botão direito no cabeçalho, largura arrastável** (VS) **e valor colorido por tipo** (DevTools: número, string entre aspas, booleano, `nil`, tabela `{N fields}` com ▸). O tipo vem do motor (`ph2d-script-debug` passa a dar `kind` por `Var`, não só texto) |
| Traço dos últimos segundos | Godot Debugger ▸ Monitors (gráfico por grandeza); Xcode debug gauges; Arduino Serial Plotter; Bret Victor *Learnable Programming* (valores ao longo do tempo ao lado do código) | **Sim, para valores numéricos**: coluna «Last 4 s» no Live values (e no Stopped, o histórico até à paragem); pairar dá mín/máx/actual; clicar abre o gráfico grande por baixo da linha. Histórico por linha+nome num anel no `Debugger` (amostrado no mesmo ritmo de 2,5 Hz dos fantasmas, `Duration::Slow`) |
| Mudou | — | **Âmbar** (`ColorToken` warn/warn-soft) com o valor antigo riscado ao lado; «mudou» = desde o passo anterior (Stopped) ou desde a amostra anterior (Live) |
| Escolha da aparência | — | `View ▸ Card Style: Sheet · Cards · Strip` no menu da aba (molde dos `Peek at Scene`/`Scene Map` da decisão 12), padrão **Sheet**; a escolha vale para todos os quadros e guarda-se como as outras opções da aba |

## 4. As três variantes (ver maquete)

- **A · Sheet (padrão):** faixa de cima (estado `Paused`/`Live` com cor, `line N · fn()`, objecto com o ponto da cor dele, botões à direita com a tecla);
  tabela Name · Value · Type com grupos dobráveis (**Locals**, **self**, tabelas aninhadas com ▸), números à direita em mono com dígitos
  alinhados, linhas listradas, «mudou» em âmbar; coluna lateral com **Call stack** (clicável → vai à linha) e **Waiting here** (nome e cor
  de cada objecto à espera); muitas linhas rolam por dentro.
- **B · Cards:** grupos em cartões como o Inspector, cada valor numa caixa de campo.
- **C · Strip:** uma célula por variável (nome pequeno em cima, valor grande em baixo), a altura de hoje.
- **A mesma família** nos outros quadros: Live values em tabela (Line · Name · Value · Last 4 s); balão ◆ em tabela (objecto · valor ·
  default/changed); Script options com interruptor + frase do que faz; Problems com colunas fixas (gravidade · problema · em quem · linha).

## 5. Ondas

| Onda | O quê | Prova |
|---|---|---|
| Q1 | Tabela da casa (cabeçalho, grupos dobráveis, colunas arrastáveis/ocultáveis, alinhamento numérico, listras, rolagem) + `kind` por `Var` no motor + `Card Style` no menu | testes de geometria da tabela; `hr15`, `no_magic_numeric`, `no_literal_color` |
| Q2 | Stopped nas três variantes (Call stack clicável, Waiting com nomes, «mudou» com o antigo) | cena `PH2D_CODE_SMOKE=14` (parada com tabela aninhada e um valor que muda no Next line); fotos das três |
| Q3 | Live values em tabela + traço (anel de histórico, mín/máx, gráfico grande) | teste do anel; foto em rajada |
| Q4 | Balão ◆, Script options e Problems na mesma família | fotos; a grelha da Bancada migra para a tabela |

Cada onda: foto e maquete lado a lado ANTES do smoke do dono (`project-memory/feedback_a_mockup_is_compared_side_by_side_before_the_smoke.md`).
