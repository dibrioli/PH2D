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

Ctrl+S → selo «Reloaded · 3 objects» (cena a tocar; Ctrl+S → `frame` → `scene_sync` + passo → `frame`, até o
`drain_reloads` dar `Reloaded` com N = 3; fora: a gravação do `.ph2dproj` em disco, que é da shell):

| documento | Ctrl+S → selo | dos quais a recarga (compilar + passo) |
|---|---|---|
| 51 linhas | 0.04 / 0.05 / 0.14 / 0.26 | 0.04 / 0.05 / 0.13 / 0.23 |
| 7 500 linhas | 2.60 / 3.07 / 6.46 / 12.09 | 2.56 / 2.95 / 5.76 / 11.59 |

A gramática por tecla, à parte (um `Buffer` + `Syntax` por tamanho, a mesma escrita):

| documento | reparse incremental | dobras (`Syntax::folds`) | erros | realce de 100 linhas |
|---|---|---|---|---|
| 51 linhas | 0.02 / 0.02 / 0.10 / 0.18 | 0.05 / 0.05 / 0.12 / 0.15 | 0.00 / 0.00 / 0.00 / 0.00 | 0.22 / 0.23 / 0.42 / 0.45 |
| 7 500 linhas | 0.37 / 0.58 / 3.34 / 4.17 | **4.95 / 6.46 / 13.75 / 14.57** | 0.00 / 0.00 / 0.00 / 0.00 | 0.37 / 0.42 / 0.86 / 2.05 |

O «1.º quadro» é UMA amostra fria: o layout de cada linha e os glifos novos para o Vello. A cauda varia com a
carga, e o primeiro uso da placa no processo (`15.7`) é o maior.

**⛔ Defeito achado (aberto): uma tecla custa O(ficheiro) nas dobras.** `CodeDoc::derive`
(`crates/ph2d-app-code/src/doc.rs:125`) corre `Syntax::folds` (`crates/ph2d-code-syntax/src/fold.rs:15-18`:
a query `folds.scm` sobre a árvore INTEIRA) a cada revisão. A tecla chama-o pelo `reveal` → `unfold_cursor`
(`keys.rs:98` → `doc.rs:233`), e a pintura chamá-lo-ia pelo `fold_marks`/`problems` se a tecla não o tivesse
feito. A medição separa a causa: a fase «tecla» passa de `0.13` ms (51) para `7.4–7.9` ms (7 500), e as dobras
sozinhas custam `6.46` ms de p50 (`14.6` no p99.9) a 7 500 linhas contra `0.05` a 51. O reparse incremental
(`0.58`) e o resto da tecla (`~0.4`) fecham a conta. Ficou por corrigir: a cura atravessa três crates (dobras
incrementais pelo `changed_ranges` da árvore, ou só as da vista mais as fechadas), por isso não é local.
**Crescimentos menores, dentro do orçamento:**

- o reparse incremental, `0.02` → `0.58` ms de p50 (×29 para ×147 de ficheiro);
- o `paint` na mesma vista de 43 linhas, `0.47` → `0.75` ms. Hipótese não medida à parte: o `fold_marks`
  percorre todas as dobras do ficheiro a cada quadro.

**O que a tabela decide (orçamentos da Onda 1):**

1. **Por tecla — tecla→pixel ≤ 2 ms de p50 e ≤ 5 ms de p99.9**, com a cena a tocar, vista até 100 linhas, em
   QUALQUER tamanho de ficheiro. O recurso é o quadro: 5 ms deixam ≥ 3 ms de um quadro a 120 Hz (`8.33` ms)
   para o resto do app. Medido a 51 linhas: p50 `1.35–1.86`, p99.9 `4.16–4.89` a `load 5–8`. Dentro dele:
   - a tecla `≤ 0.2` ms (medido `0.13–0.16`);
   - o paint `≤ 1.2` ms a 100 linhas (`0.56–1.17`);
   - a placa com a leitura `≤ 1.4` ms (`0.71–1.37`).
   ⛔ Hoje a 7 500 linhas o orçamento **estoura** (p50 `9.5–10.4`, p99.9 `20.4–23.3`) pelo defeito das dobras.
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
estoura o quadro a 7 500 linhas são as dobras, não a pintura.

**Regra de Fatin (51 × 7 500): REPROVADA hoje.** A latência cresce com o tamanho do ficheiro: ×7 no p50 e ×5 no
p99.9, e a cena a tocar quase não pesa (`+0.1` ms). A causa medida é o defeito acima. O aceite da onda
(«uma edição num ficheiro de 7 500 linhas não piora a latência») só passa quando as dobras deixarem de ser
O(ficheiro) por tecla. A re-medição é esta mesma sonda.

## Fica para a Onda 2 (com o porquê)

- Textos escritos à mão para os erros frequentes (retorno do dono, parte 2): pedem a medição de frequência do plano §6 Onda 2.
- Tudo o que pede ao `luau-lsp` dentro do quadro (diagnóstico ao digitar, completion, hover): o cliente é bloqueante e a onda dele é a 2.
- Marcador da fila global do undo apontando a transação (§5.5): só tem consumidor quando uma ação toca script E cena (renomear propriedade, Onda 2/3); até lá o histórico próprio + «documento ativo vence o Ctrl+Z» é o molde do Board.
