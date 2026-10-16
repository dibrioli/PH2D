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

## Medições (E14)

### Tecla→pixel, salvar→selo e a gramática por tecla (15/10)

Sonda `bancada::tecla_pixel` (`crates/ph2d-app-code/tests/it/bancada.rs`, `#[ignore]`). Por tecla, o
caminho do app: `DocumentSurface::key` (buffer + `AutoEdit` + reparse incremental + `reveal`) → [cena
a tocar: `scene_sync` + `scene_tick` do `ScriptHost`, três objectos no MESMO script, que é o documento
aberto] → `ph2d_app_code::frame` → `paint` da aba numa `VectorScene` → `VelloPass::render_to_intermediate`
na placa → cópia de UMA faixa do alvo (a linha do cursor) para a memória. O relógio pára quando a faixa
chega. Fora do relógio, confirma-se que o glifo novo está nos píxeis lidos e que o cursor está onde a régua
lê. Controlo da régua em cada célula: antes da tecla, zero píxeis com tinta na caixa do glifo; depois, ≥ 4.
**Fora da medição:** o compositor do app, a espera do vsync e a apresentação no ecrã do sistema.

- **Perfil:** `smoke` (otimizado). **Placa:** NVIDIA GeForce RTX 5060 Ti (Vulkan), com a exclusão da placa (`PH2D_GPU=1`).
- **`loadavg`:** início `5.46 23.47 33.18` · fim `8.00 21.96 32.26` (outras linhas a compilar). Não se esperou calma.
- **Intercalação:** 12 tarefas (8 células tecla→pixel, 2 de salvar, 2 de gramática), 10 rodadas × blocos de 25
  com a ordem rodada, `n = 250` por célula (o p99.9 é praticamente o máximo). A rodada inteira levou `37,5 s`.
- **Variantes:** cena parada × a tocar · `bob.luau` (51 linhas, o do smoke) × 7 500 linhas (o mesmo juntado;
  só a 1.ª cópia declara as `ph2d.property`, porque o motor recusa a 2.ª declaração: `Refused` na linha 58)
  · vista de ~43 linhas (`1600 × 862` px, ≈ a janela do dono a `1930 × 1040`) × vista de 100 linhas
  (`1600 × 1945` px, a do critério de morte). Escala 100 % (a do dono), `TextSystem::new()` (o do app).
- **Escrita:** cinco letras no fim de `  self.t = self.t + dt` (no grande, a mesma linha na cópia 72: linha
  3 703), depois cinco Backspace fora do relógio. Entre duas teclas há 2 quadros sem tecla, pintados e
  enviados à placa (a cena a tocar dá um passo em cada um).
- Re-correr: `PH2D_GPU=1 bash scripts/ph2d-run.sh cargo test --profile smoke -p ph2d-app-code --test it bancada -- --ignored --nocapture`.

Cada célula: **mínimo / p50 / p99 / p99.9** em ms; as fases são p50.

**ANTES** da cura das dobras (`3bd0ba3a55`, `loadavg` 5–8):

| cena · documento · vista | linhas pintadas | 1.º quadro | **tecla→pixel** | tecla (buffer + AutoEdit + reparse + dobras) | quadro da família (+ passo da cena) | paint | placa + leitura | a11y por tecla | paint sem mudança |
|---|---|---|---|---|---|---|---|---|---|
| parada · 51 · ~43 | 43 | 15.7 | 1.15 / 1.41 / 3.79 / 4.89 | 0.128 | 0.003 | 0.474 | 0.757 | 0.07 / 0.08 / 0.23 / 0.27 | 0.36 / 0.40 / 1.11 / 1.36 |
| a tocar · 51 · ~43 | 43 | 2.7 | 1.16 / 1.35 / 3.37 / 4.67 | 0.127 | 0.030 | 0.463 | 0.707 | 0.07 / 0.08 / 0.24 / 0.29 | 0.37 / 0.41 / 1.08 / 2.76 |
| parada · 7 500 · ~43 | 43 | 8.8 | 6.70 / 9.45 / 20.58 / 21.09 | **7.497** | 0.009 | 0.746 | 1.028 | 0.08 / 0.12 / 0.32 / 0.33 | 0.45 / 0.52 / 1.29 / 2.06 |
| a tocar · 7 500 · ~43 | 43 | 11.5 | 6.82 / 10.02 / 20.30 / 20.38 | **7.939** | 0.118 | 0.801 | 1.034 | 0.08 / 0.13 / 0.29 / 0.31 | 0.45 / 0.54 / 1.14 / 1.15 |
| parada · 51 · 100 | 52 | 4.7 | 1.28 / 1.86 / 3.51 / 4.16 | 0.163 | 0.004 | 0.642 | 0.998 | 0.08 / 0.12 / 0.24 / 0.27 | 0.42 / 0.57 / 0.93 / 1.79 |
| a tocar · 51 · 100 | 52 | 3.3 | 1.32 / 1.62 / 3.40 / 4.37 | 0.137 | 0.033 | 0.559 | 0.835 | 0.08 / 0.09 / 0.21 / 0.27 | 0.43 / 0.49 / 0.89 / 0.92 |
| parada · 7 500 · 100 | 100 | 9.7 | 7.28 / 9.97 / 18.28 / 21.76 | **7.405** | 0.010 | 1.159 | 1.348 | 0.14 / 0.20 / 0.43 / 0.49 | 0.78 / 0.92 / 1.89 / 2.09 |
| a tocar · 7 500 · 100 | 100 | 13.3 | 7.31 / 10.42 / 22.93 / 23.30 | **7.688** | 0.116 | 1.170 | 1.369 | 0.14 / 0.20 / 0.52 / 0.59 | 0.79 / 0.93 / 2.15 / 2.45 |

**DEPOIS** da cura (15/10, mesma sonda, rodada de `16.1 s`; `loadavg` início `6.11 4.46 14.82` · fim
`5.89 4.53 14.62`). A fase «tecla» é buffer + `AutoEdit` + reparse + as dobras FECHADAS (as da vista
passaram para a pintura):

| cena · documento · vista | linhas pintadas | 1.º quadro | **tecla→pixel** | tecla | quadro da família (+ passo da cena) | paint | placa + leitura | a11y por tecla | paint sem mudança |
|---|---|---|---|---|---|---|---|---|---|
| parada · 51 · ~43 | 43 | 16.4 | 1.12 / 1.30 / 2.75 / 4.13 | 0.052 | 0.002 | 0.505 | 0.706 | 0.07 / 0.08 / 0.16 / 0.18 | 0.39 / 0.45 / 0.98 / 1.10 |
| a tocar · 51 · ~43 | 43 | 2.6 | 1.12 / 1.29 / 1.88 / 2.94 | 0.050 | 0.027 | 0.491 | 0.690 | 0.06 / 0.08 / 0.16 / 0.18 | 0.40 / 0.44 / 0.92 / 0.94 |
| parada · 7 500 · ~43 | 43 | 3.3 | 1.49 / 1.74 / 2.68 / 3.45 | **0.471** | 0.003 | 0.461 | 0.731 | 0.07 / 0.08 / 0.16 / 0.22 | 0.36 / 0.39 / 0.61 / 0.62 |
| a tocar · 7 500 · ~43 | 43 | 3.1 | 1.54 / **1.77** / 2.81 / **3.05** | **0.482** | 0.057 | 0.464 | 0.724 | 0.07 / 0.08 / 0.13 / 0.15 | 0.37 / 0.40 / 0.63 / 0.91 |
| parada · 51 · 100 | 52 | 4.2 | 1.26 / 1.47 / 2.17 / 3.40 | 0.051 | 0.003 | 0.573 | 0.798 | 0.07 / 0.09 / 0.17 / 0.18 | 0.47 / 0.52 / 0.79 / 0.81 |
| a tocar · 51 · 100 | 52 | 2.8 | 1.27 / 1.44 / 2.25 / 2.92 | 0.051 | 0.027 | 0.567 | 0.786 | 0.08 / 0.08 / 0.14 / 0.14 | 0.47 / 0.51 / 0.81 / 1.06 |
| parada · 7 500 · 100 | 100 | 3.7 | 2.13 / 2.51 / 3.99 / 4.27 | **0.516** | 0.003 | 0.854 | 1.075 | 0.14 / 0.15 / 0.33 / 0.47 | 0.72 / 0.78 / 1.25 / 1.36 |
| a tocar · 7 500 · 100 | 100 | 4.8 | 2.20 / **2.75** / 4.26 / **4.50** | **0.576** | 0.061 | 0.885 | 1.072 | 0.14 / 0.15 / 0.30 / 0.34 | 0.73 / 0.81 / 1.21 / 1.27 |

Duas rodadas a mais da mesma árvore, para ver o ruído. Uma a `load 9` dá a tocar · 7 500: `2.02` ms de p50 a
~43 linhas e `2.81` a 100 linhas. A outra, a `load 0.85`, ainda tinha o `changed_ranges` em cada tecla (ver
abaixo) e dá `1.93` e `2.67`. O mínimo varia pouco entre as três.

Ctrl+S → selo «Reloaded · 3 objects» (cena a tocar; Ctrl+S → `frame` → `scene_sync` + passo → `frame`, até o
`drain_reloads` dar `Reloaded` com N = 3; fora: a gravação do `.ph2dproj` em disco, que é da shell):

| documento | Ctrl+S → selo | dos quais a recarga (compilar + passo) |
|---|---|---|
| 51 linhas | 0.04 / 0.05 / 0.14 / 0.26 | 0.04 / 0.05 / 0.13 / 0.23 |
| 7 500 linhas | 2.60 / 3.07 / 6.46 / 12.09 | 2.56 / 2.95 / 5.76 / 11.59 |

A gramática por tecla, à parte (um `Buffer` + `Syntax` por tamanho, a mesma escrita). **ANTES:**

| documento | reparse incremental | dobras (`Syntax::folds`) | erros | realce de 100 linhas |
|---|---|---|---|---|
| 51 linhas | 0.02 / 0.02 / 0.10 / 0.18 | 0.05 / 0.05 / 0.12 / 0.15 | 0.00 / 0.00 / 0.00 / 0.00 | 0.22 / 0.23 / 0.42 / 0.45 |
| 7 500 linhas | 0.37 / 0.58 / 3.34 / 4.17 | **4.95 / 6.46 / 13.75 / 14.57** | 0.00 / 0.00 / 0.00 / 0.00 | 0.37 / 0.42 / 0.86 / 2.05 |

**DEPOIS** (a mesma rodada da tabela DEPOIS acima). A tecla já não paga o `folds`, que só o «Fold All» chama;
paga as dobras da vista (`folds_in`, na pintura) e, só com dobras fechadas, o `changed_ranges`:

| documento | reparse incremental | estrutura mudada (`changed_ranges`, só com dobras fechadas) | dobras do ficheiro (`folds`, só o Fold All) | dobras de 100 linhas (`folds_in`, a pintura) | erros | realce de 100 linhas |
|---|---|---|---|---|---|---|
| 51 linhas | 0.02 / 0.02 / 0.08 / 0.10 | 0.01 / 0.01 / 0.03 / 0.03 | 0.05 / 0.05 / 0.13 / 0.14 | 0.05 / 0.05 / 0.09 / 0.09 | 0.00 / 0.00 / 0.00 / 0.00 | 0.22 / 0.23 / 0.41 / 0.48 |
| 7 500 linhas | 0.37 / 0.42 / 1.19 / 2.24 | 0.34 / 0.36 / 0.60 / 0.72 | 4.91 / 5.37 / 8.72 / 9.25 | **0.07 / 0.07 / 0.14 / 0.15** | 0.00 / 0.00 / 0.00 / 0.00 | 0.37 / 0.39 / 0.63 / 0.63 |

**De quem é o reparse que cresce?** Mediu-se o MESMO reparse com as teclas intercaladas
(`ph2d-code-syntax` `cronometro::reparse_rope_contra_str`, `smoke`, `loadavg 13.11`, 400 teclas). Uma coluna
é o nosso `Syntax::edit`, que lê a rope por pedaços. A outra é o tree-sitter sozinho sobre um `&str`
contíguo:

| documento | `Syntax::edit` (rope por pedaços) | tree-sitter sobre `&str` |
|---|---|---|
| 51 linhas | 0.016 / 0.018 / 0.032 / 0.043 | 0.015 / 0.017 / 0.029 / 0.036 |
| 7 500 linhas, as `ph2d.property` de cada cópia vivas | 2.007 / 2.130 / 4.418 / 4.645 | 1.926 / 2.044 / 4.202 / 4.630 |
| 7 500 linhas, as da 2.ª cópia em diante comentadas (o texto da bancada) | 0.376 / 0.380 / 0.399 / 1.110 | 0.372 / 0.378 / 0.410 / 0.969 |
| 7 500 linhas, um bloco aberto no fim (texto com ERROR) | 1.959 / 2.099 / 3.933 / 4.183 | 1.881 / 2.027 / 4.143 / 4.494 |

**O reparse que cresce é do tree-sitter, não nosso.** A leitura da rope custa ≤ 4 % a mais. O custo depende
do CONTEÚDO, não só do tamanho: com as ~1 000 chamadas `ph2d.property(…)` de topo vivas, o reparse por tecla
sobe de `0.38` para `2.1` ms de p50; com um bloco aberto no fim, também.

O «1.º quadro» é UMA amostra fria: o layout de cada linha e os glifos novos para o Vello. A cauda varia com a
carga, e o primeiro uso da placa no processo (`15.7`) é o maior.

**✅ Defeito achado e corrigido (15/10): uma tecla custava O(ficheiro) nas dobras.** ANTES, `CodeDoc::derive`
corria `Syntax::folds` em cada revisão: a query `folds.scm` sobre a árvore INTEIRA. A tecla chamava-o pelo
`reveal` → `unfold_cursor`. A fase «tecla» ia de `0.13` ms (51 linhas) a `7.4–7.9` ms (7 500), e só as dobras
custavam `6.46` ms de p50 a 7 500 contra `0.05` a 51. **A cura:**

- **As dobras da vista** (`Syntax::folds_in`: a query com `set_point_range`, só pelos nós que tocam a faixa).
  A pintura pede-as por bloco contíguo de linhas visíveis (`CodeFrame::folds` é agora um pedido por faixa),
  e o `fold_marks` deixou de percorrer o ficheiro. A 7 500 linhas custam `0.07` ms para 100 linhas.
- **As dobras FECHADAS são estado do documento** (`ph2d-app-code` `folds.rs`). Guardam o byte do cabeçalho
  e o da última linha escondida, que as edições movem. Só se re-conferem (`Syntax::fold_at`, uma linha) as
  fechadas que a edição tocou ou cuja estrutura o tree-sitter diz que mudou (`changed_ranges`). Uma fechada
  sem estrutura abre-se. O `unfold_cursor` só olha as fechadas, e o «Fold All» é a única pergunta pelo
  ficheiro inteiro.
- **O `changed_ranges` custa O(ficheiro):** `0.36` ms de p50 a 7 500 linhas, medido na tabela acima. Na 1.ª
  versão da cura corria em cada tecla, e a fase «tecla» ficava em `0.73` ms. Agora só se arma enquanto
  há dobras fechadas: sem nenhuma, a tecla não o paga (`0.47–0.58` ms).
- **A suspeita do `paint`** confirmou-se: na mesma vista de 43 linhas a 7 500 o paint caiu de `0.75–0.80` ms
  para `0.46` ms, o mesmo que a 51 linhas (`0.49–0.51`).
- **Gate de custo, sem relógio:** `dobras::o_trabalho_das_dobras_por_tecla_nao_cresce_com_o_ficheiro`
  (`ph2d-app-code`). Conta as capturas da query das dobras (`fold_captures_visited`) por tecla mais o
  quadro pintado, com a MESMA vista de 35 linhas. Hoje dá `40` a 53 linhas e `41` a 7 500. Com o custo
  antigo reposto (a query inteira em cada revisão) dá `90` contra `7 126`, e o gate fica vermelho.

**O que ainda cresce com o ficheiro:** o **reparse incremental do tree-sitter**, de `0.02` para `0.42` ms
de p50 (51 → 7 500, ×21). Não é nosso (tabela rope × `&str` acima). Mais nada da tecla cresce: a tecla a 7 500
(`0.47–0.58`) é o reparse mais os `0.05` ms de 51 linhas.

**O que a tabela decide (orçamentos da Onda 1):**

1. **Por tecla — tecla→pixel ≤ 2 ms de p50 e ≤ 5 ms de p99.9**, com a cena a tocar, vista até 100 linhas, em
   QUALQUER tamanho de ficheiro. O recurso é o quadro: 5 ms deixam ≥ 3 ms de um quadro a 120 Hz (`8.33` ms)
   para o resto do app. Medido a 51 linhas: p50 `1.35–1.86`, p99.9 `4.16–4.89` a `load 5–8`. Dentro dele:
   - a tecla `≤ 0.2` ms (medido `0.13–0.16`);
   - o paint `≤ 1.2` ms a 100 linhas (`0.56–1.17`);
   - a placa com a leitura `≤ 1.4` ms (`0.71–1.37`).
   ANTES, a 7 500 linhas o orçamento **estourava** (p50 `9.5–10.4`, p99.9 `20.4–23.3`) pelo defeito das dobras.
   **DEPOIS (15/10, cena a tocar, 7 500 linhas):**
   - **Vista de ~43 linhas: PASSA.** p50 `1.77`, p99.9 `3.05` a `load 6` (`2.02` / `3.67` a `load 9`).
   - **Vista de 100 linhas: o p99.9 PASSA e o p50 REPROVA.** p99.9 `4.50`; p50 `2.75` (`2.81` a `load 9`,
     `2.67` a `load 0.85`, ainda com o `changed_ranges`).

   O que passa dos 2 ms a 100 linhas é sobretudo a pintura e a placa de 100 linhas pintadas: `0.885` +
   `1.072` = `1.96` ms, cada uma dentro do seu sub-orçamento (`≤ 1.2` e `≤ 1.4`). Isto não depende do tamanho
   do ficheiro. Por cima vem o reparse do tree-sitter (`0.42`). A soma dos três sub-orçamentos
   (`0.2 + 1.2 + 1.4 = 2.8`) já passa dos 2 ms de p50. A tecla (`0.48–0.58` ms) passa do seu (`≤ 0.2`) só pelo
   reparse do tree-sitter.
2. **Por quadro sem tecla:** paint `≤ 1` ms de p50 a 100 linhas (`0.40–0.93`); a árvore a11y `≤ 0.2` ms de
   p50 (`0.08–0.20`); reparse `≤ 1` ms de p50 e realce das 100 linhas `≤ 0.5` ms (`0.58` e `0.42` a 7 500).
3. **Ao salvar: Ctrl+S → selo dentro de UM quadro a 60 Hz** (`16.7` ms) até 7 500 linhas. Medido: `0.05` ms de
   p50 a 51 e `3.07` (p99.9 `12.1`, a compilação do Luau) a 7 500. No app soma-se até um quadro de espera,
   porque a recarga é lida no `scene_sync` seguinte, e soma-se a gravação do projeto.
4. **O 1.º quadro de uma aba** (frio) é um evento à parte: `2.7–15.7` ms. Cabe num quadro a 60 Hz, nunca num
   quadro a 120 Hz.

**Critério de morte: NÃO disparou — fica a cache por linha, sem atlas de glifos.** Com 100 linhas visíveis e a
cena a tocar, o paint custa `0.56` ms (51) / `1.17` ms (7 500) de p50. Com a placa e a leitura (`0.84` / `1.37`)
fica abaixo de `2.6` ms, um terço do quadro a 120 Hz. O pior p99.9 do paint sem mudança é `2.45` ms. O que
estourava o quadro a 7 500 linhas eram as dobras, não a pintura. Depois da cura o paint de 100 linhas a 7 500
custa `0.885` ms.

**Regra de Fatin (51 × 7 500).** ANTES: **REPROVADA**. A latência crescia com o ficheiro, ×7 no p50 e ×5
no p99.9, pelas dobras O(ficheiro). **DEPOIS: o defeito saiu, mas a regra estrita ainda não passa.** O
único par que pinta o mesmo número de linhas é a vista de ~43 linhas (o `bob.luau` só tem 52 linhas, logo a
vista de 100 a 51 pinta 52). Nele, com a cena a tocar:

- p50: `1.29` → `1.77` ms (`+0.48`, ×1.4; antes `+8.7`, ×7);
- p99.9: `2.94` → `3.05` ms (antes ×4.4).

O que sobra de crescimento é todo o reparse incremental do tree-sitter (`0.02` → `0.42` ms). O resto da
tecla, a pintura, a placa e a a11y ficaram iguais entre 51 e 7 500. O aceite («uma edição num ficheiro de
7 500 linhas não piora a latência») passa dentro do orçamento, mas não ao pé da letra.

**A próxima fase que cresce com o ficheiro é o reparse do tree-sitter**: `0.42` ms de p50 no texto da
bancada, e `2.1` ms com as ~1 000 `ph2d.property` de topo vivas. Ainda não se mediu porque é que as
chamadas de topo o encarecem, e esse é o primeiro passo. Só depois se escolhe a cura: tirar o reparse da
tecla, ou o que essa medição apontar.

## Fica para a Onda 2 (com o porquê)

- Textos escritos à mão para os erros frequentes (retorno do dono, parte 2): pedem a medição de frequência do plano §6 Onda 2.
- Tudo o que pede ao `luau-lsp` dentro do quadro (diagnóstico ao digitar, completion, hover): o cliente é bloqueante e a onda dele é a 2.
- Marcador da fila global do undo apontando a transação (§5.5): só tem consumidor quando uma ação toca script E cena (renomear propriedade, Onda 2/3); até lá o histórico próprio + «documento ativo vence o Ctrl+Z» é o molde do Board.
