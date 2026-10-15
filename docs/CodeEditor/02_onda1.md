# CodeEditor — Onda 1: o editor que já ganha do externo

> Desenho da Onda 1 do [plano](reports/Editor%20de%20scripts%20estado%20da%20arte.md) §6, ajustado às
> [decisões do dono](00_decisoes_do_dono.md), ao [ADR-0188](../architecture/decisions/0188-o-editor-de-scripts-mora-no-ph2d-e-o-script-mora-no-projeto.md)
> e ao [handoff da Onda 0](handoffs/HANDOFF_CONTINUACAO_line_CodeEditor_2026-10-15_ONDA0.md) (§6 aberto, §7, retorno do smoke).
> Estado: **aprovado pelo dono 15/10** — e decidiu: **Ctrl+S no editor aplica o script E grava o projeto** (opção «a»).

## O que o código diz hoje (medido 15/10, explorador)

| Facto | Endereço | Consequência |
|---|---|---|
| `Documents` só conhece quadros: `active: Option<BoardId>`, `None` = Cena | `ph2d-editor-core/src/documents.rs:27-41`, `screens/hero/document_tabs.rs:100-135` | o documento de código é um TIPO novo de aba (foundational, Modo L append-only) |
| As costuras do Board na shell: ~60–80 linhas em 8–9 ficheiros + `keyboard_board.rs` (119) | `input_dispatch.rs:147-165,295-305,475-480,582-591`, `keyboard.rs:165-170`, `despacho_metodos_janela_e_vetor.rs:200-220`, `fase_pointer_subjects.rs`, `project_save.rs:67` | NÃO copiar o padrão: uma costura «documento ativo» genérica, e a shell paga-a movendo código para fora (catraca `the_shell_only_shrinks`, `TETO_LOC 189 041`) |
| Recarga = polling do `blake3` do texto por quadro | `ph2d-script/src/scene.rs:281` (`sync`) → `refresh` (~306) | «salvar» = escrever o buffer na `ScriptLibrary`; recarrega no quadro seguinte sem disco. Falta o RELATÓRIO da recarga para o selo |
| Texto que não compila: `slot.module = None` — o objeto PÁRA | `scene.rs:~343-346`, `module_of` `:451` | o smoke da onda («apagar um `end`, a cena continua») exige trazer a «última versão boa continua» (catálogo #22, era Onda 4) |
| Não há contagem de objetos por `ScriptId` (`live_count` é global; `actors` privado) | `scene.rs:272` | contagem nova para «Reloaded · N objects» |
| `ScriptDiagnostic` não tem coluna nem trecho; o token vem cru na mensagem (`got '='`) | `ph2d-script/src/diagnostic.rs:29-38`; fixtura `tests/it/diagnostico_do_luau_medido.rs` | o lugar exacto do símbolo culpado vem do tree-sitter (nó `ERROR`/`MISSING`), conferido contra o `luau-lsp` como oráculo |
| `print` do script vai para o stdout; não há console | (busca vazia em `ph2d-script/src`) | `print` próprio → buffer de console por script |
| Cache de layout do `ph2d-text` é chaveado por `UiFont`; a mono não passa por ela | `ph2d-text/src/layout_cache.rs:18`, `system.rs:159` | cache por LINHA própria do editor |
| `TextRun`/`Document` existem no accesskit 0.24.1, nunca usados no PH2D | `ph2d-a11y/src/lib.rs:40` | árvore por linha nova; o adaptador do SO continua não linkado (lacuna do app) |
| `no_literal_color` / `no_magic_numeric` / cap 600 só varrem `ph2d-panel-*` | `no_literal_color.rs:314` | a pintura mora em `ph2d-panel-code` para ficar sob os gates |
| `crop`, `tree-sitter`, `tree-sitter-luau` fora do lock; crates.io acessível | `crop` 0.4.3 · `tree-sitter` 0.27.1 · `tree-sitter-luau` 1.2.0 (MIT, traz highlights/folds/indents/locals) | licenças por artefacto no passo 1 (§0.9) |
| `LspClient::request` é bloqueante | `ph2d-code-lsp/src/client.rs:310-334` | a Onda 1 NÃO pede nada ao LSP no quadro (é Onda 2); só o oráculo de posição nos testes |
| Godot 4.7.2 corre `CodeEdit` headless: `(`→`()`, Enter após `func f():` indenta | sonda em `/tmp/claude-1000/godot-probe/probe.gd` | oráculo de auto-indent/auto-brace viável por `push_input` |

## Crates

| Crate | Papel | Gates |
|---|---|---|
| `ph2d-code-buffer` (nova, pura) | `crop::Rope` atrás de trait própria; `Transaction`/`ChangeSet` invertível; `Selection = Vec<Range>` (multicursor); âncoras; linha/coluna/UTF-16; histórico agrupado | testes por traços de edição |
| `ph2d-code-syntax` (nova) | `tree-sitter` 0.27 + `tree-sitter-luau` fixado; realce, dobras, indentação, pares; reparse incremental sobre a transação; posição do erro | fixtures |
| `ph2d-panel-code` (nova) | pintura: linhas visíveis com cache por linha, calha, barra de estado, buscar/substituir, console, lista; a11y por linha | `no_literal_color`, `no_magic_numeric`, cap 600, `hr12` |
| `ph2d-app-code` (nova, família) | estado do documento, teclado, salvar→recarga→selo, smoke `PH2D_CODE_SMOKE=<n>` (`SmokeRouter`) | `the_shell_only_shrinks`; registo por `cargo run -p ph2d-app-sync` |
| aditivo: `ph2d-script`, `ph2d-editor-core` (`documents`), `ph2d-i18n`, `ph2d-tokens` | — | — |

## Itens (prova = o teste vermelho se a propriedade quebrar)

| # | Item | Prova |
|---|---|---|
| **E1** | **Buffer**: rope, transações invertíveis, multicursor, histórico agrupado (palavra contínua = 1 passo; edição multicursor = 1 passo) | traço de edição aleatório: aplicar+inverter = texto original (byte a byte); `Ctrl+D` 3× + digitar = 1 passo de undo; linha↔byte↔UTF-16 contra contagem ingénua |
| **E2** | **Sintaxe**: realce, dobras, indentação, pares, reparse incremental | `bob.luau` → captura por byte em fixtura; reparse incremental = parse do zero (árvores iguais) num traço de edições; controlo: query vazia ⇒ vermelho |
| **E3** | **Aba de documento de código** + botão «Edit script» no cartão Script; a costura «documento ativo» na shell é UMA e genérica | ui-testkit: clique em «Edit script» ⇒ aba ativa com o texto do script do objeto; segundo clique não duplica a aba; gate de costura (7 sites) |
| **E4** | **Pintura**: linhas visíveis só (virtualização), cache por linha (hash da linha, spans, largura, geração da fonte), fonte mono com hint, calha com números | contador: rolar 1 linha remolda ≤ 1 linha nova; editar 1 linha num ficheiro de 7 500 remolda só ela; gate de placa: a mono pinta com hint (o pico das duas barras do `=` igual, mesma régua do `679ce98a93`) |
| **E5** | **Teclado capturado no topo da cadeia** quando o editor está focado (texto, IME, Ctrl+Z/Y, Tab, setas) | teste por nome: Espaço, `P`, `W`, Delete, Ctrl+Z com o editor focado ⇒ ZERO ação da cena e o texto muda; controlo: com o canvas focado, Espaço dá play |
| **E6** | **Salvar → recarga → selo** «Reloaded · N objects»; erro de compilação ⇒ selo de recusa, a versão anterior CONTINUA a correr e a linha fica marcada | teste: 3 objetos no mesmo script, salvar ⇒ relatório N=3 e os 3 com o valor novo; apagar um `end` e salvar ⇒ os 3 continuam com o código antigo, diagnóstico com a linha; controlo: corrigir ⇒ volta a recarregar |
| **E7** | **Erro na linha + símbolo culpado** (retorno do dono): calha marca a linha; o símbolo sublinhado no lugar EXACTO; no cartão o símbolo sai em chip, nunca `'='` cru | fixtura `script_quebrado.luau`: a posição vinda do tree-sitter = o `range` que o `luau-lsp` 1.70.1 dá (oráculo, gravado com cabeçalho); ui-testkit: o cartão não contém `'` à volta do símbolo |
| **E8** | **Fechamento automático e auto-indent** discretos e apagáveis | fixtura do oráculo Godot 4.7.2 (`CodeEdit` headless, entradas nossas, casos portáveis: pares `( [ { " '`, saltar o fecho, Backspace apaga o par, Enter entre pares, Enter mantém indentação, Enter após abridor indenta) — cada caso vira gate contra a nossa saída |
| **E9** | **Buscar/substituir** (no ficheiro; com prévia da contagem) e **Ir para linha** | testes: substituir tudo = 1 passo de undo; busca sem resultado diz-o no ecrã; `Ctrl+G 40` põe o cursor na 40 e a vista mostra-a |
| **E10** | **Console**: `print` do script, erros de runtime, recusas de recarga; abre sozinho no erro de runtime | teste: `print("oi")` num `update` chega ao console do script e NÃO ao stdout; erro de runtime ⇒ console abre com a linha clicável |
| **E11** | **Lista de scripts** da biblioteca (filtro, novo, renomear, apagar sem uso) — fecha os abertos da Onda 0: seletor de existentes, «bob 2.luau» no re-Browse, Inspector a dizer «file» | testes: re-Browse do mesmo ficheiro não cria cópia; apagar um script em uso é recusado com frase no ecrã |
| **E12** | **AccessKit por linha**: nó `Document` com seleção + um `TextRun` por linha visível (≤ 255 chars) | teste: a árvore tem um nó por linha visível com o texto exacto; controlo: rolar troca os nós |
| **E13** | **Comandos no menu ⇒ Ctrl+K** (salvar, buscar, ir para linha, dobrar); i18n + tokens | gates de UI verdes; Ctrl+K lista «Go to line» |
| **E14** | **Bancada tecla→pixel + medições** — UMA rodada intercalada: tecla→pixel (cena tocando/parada × 51/7 500 linhas), encode por linha visível, reparse por tecla, salvar→selo, árvore a11y por tecla | tabela nesta página com min/p50/p99/p99.9; os orçamentos da onda saem dela. **Critério de morte:** se as ~100 linhas visíveis estourarem o quadro com a cena a tocar mesmo com cache, trocar para atlas de glifos antes de seguir |
| **E15** | **Smoke `PH2D_CODE_SMOKE=1`** (cena com FORMAS e simulação, três objetos no mesmo script) — fotografado antes de entregar; a shell encolhe (paga as costuras movendo o código da T7 para a família) | foto; `the_shell_only_shrinks` verde com delta ≤ 0 contra o merge-base |

## Ordem

1. **E1, E2** (puros, em paralelo) — a fundação sem UI.
2. **E3 → E4 → E5 → E6**: devolve o CAMINHO DE EDIÇÃO (a regressão da Onda 0 fecha aqui).
3. **E7–E13** em lote (pastas disjuntas, sobre o caminho já vivo).
4. **E14** numa rodada só, **E15**, fecho batched (suíte imprescindível: crates novas + formato).

Série só em 2 (cada passo precisa do anterior para ser visto no ecrã).

## Fica para a Onda 2 (com o porquê)

- Textos escritos à mão para os erros frequentes (retorno do dono, parte 2): pedem a medição de frequência do plano §6 Onda 2.
- Tudo o que pede ao `luau-lsp` dentro do quadro (diagnóstico ao digitar, completion, hover): o cliente é bloqueante e a onda dele é a 2.
- Marcador da fila global do undo apontando a transação (§5.5): só tem consumidor quando uma ação toca script E cena (renomear propriedade, Onda 2/3); até lá o histórico próprio + «documento ativo vence o Ctrl+Z» é o molde do Board.
