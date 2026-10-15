# CodeEditor — Onda 0: fundações medidas

> Desenho da Onda 0 do [plano](reports/Editor%20de%20scripts%20estado%20da%20arte.md) §6, ajustado às
> [decisões do dono](00_decisoes_do_dono.md) e fixado no [ADR-0188](../architecture/decisions/0188-o-editor-de-scripts-mora-no-ph2d-e-o-script-mora-no-projeto.md).
> Mapa do código medido em 15/10 (explorador), com `file:line`, nas notas
> [`ph2d_internal_context.md`](research_notes/Editor%20de%20scripts%20estado%20da%20arte/ph2d_internal_context.md).

**Desvio do plano (medido):** as fixturas do oráculo Godot e a bancada tecla→pixel passam à **Onda 1** — os
seus consumidores (auto-indent, pintura do editor) só nascem lá. Entra na Onda 0 o **script dentro do
projeto** (decisão 1), porque tudo o que o editor faz edita esse texto.

## Itens

| # | Item | Onde | Prova (vermelha se a propriedade quebrar) |
|---|---|---|---|
| T1 | **Diagnóstico estruturado** — `ScriptDiagnostic { kind: Syntax·Decl·Runtime·Budget, line: Option<u32>, message (inteira), traceback }`; o corte na 1.ª linha some do modelo (o cartão decide o que mostra); o cartão Script mostra «line N». Formato real da mensagem do Luau com `set_name("=<nome>")` **medido** e congelado numa fixtura. | `ph2d-script` (`module.rs`, `scene.rs`), cartão em `ph2d-app-components`/`ph2d-panel-inspector` | teste por nome: erro de sintaxe na linha 3 ⇒ `line == Some(3)`; erro de runtime em `update` na linha 7 ⇒ `Some(7)`; o cartão pinta a linha (ui-testkit) |
| T2 | **Tabela da API como fonte única** — `api_table.rs` declarativo (`caminho`, tipo Luau, doc); teste que monta a VM pela MESMA fiação e compara as chaves de `ph2d`/`motion` com a tabela nos dois sentidos; o `ph2d.d.luau` (`declare ph2d: {…}`) sai dela; `ph2d-bindgen --check` passa a acusar a deriva desta metade. | `ph2d-script`, `tools/ph2d-bindgen`, `runtime/luau/` | o teste de introspeção (função nova sem linha na tabela ⇒ vermelho); controlo: chave inventada na tabela ⇒ vermelho |
| T3 | **`luau-lsp` fixado** — manifesto (versão 1.70.1 + sha256 por SO), `scripts/luau-lsp-fetch.sh` (descarrega para `~/.cache/ph2d/luau-lsp/<ver>/`, confere o sha256), resolução `PH2D_LUAU_LSP` → ao lado do executável → cache. Crate `ph2d-code-lsp`: cliente stdio mínimo (initialize, didOpen/didChange, publishDiagnostics, completion, hover), UTF-8↔UTF-16, supervisão. | `crates/ph2d-code-lsp`, `scripts/`, `tools/luau-lsp/` | gate: o `ph2d.d.luau` carrega SEM «Failed to read definitions file»; `bob.luau` limpo em não estrito e estrito; fixtura com erros de propósito dá EXATAMENTE os diagnósticos esperados (controlo de cegueira). Executa a HR-17. |
| T4 | **Spike de latência** — sonda `#[ignore]` que, pelo cliente da T3 e com as NOSSAS definições, mede `didChange→publishDiagnostics`, completion e hover em `bob.luau` e módulos de 200/1 000/3 000 linhas, solver antigo × novo **intercalados no mesmo processo**, resumo = mínimo, p50, p99, p99.9. Tabela gravada aqui. | `ph2d-code-lsp/tests/it` | a tabela; os orçamentos da Onda 1–2 saem dela |
| T5 | **Fonte mono** — JetBrains Mono (OFL-1.1, licença ao lado do ficheiro) embutida como `PH2D JetBrains Mono`; `FONT_MONO` nomeia a família forçada. | `ph2d-text`, `ph2d-tokens` | gate: a família resolve para a embutida (não para o sistema) e é monoespaçada (avanço de `i` = `W` = `0`) |
| T6 | **Pré-edição de IME** — `Ime::Preedit` chega ao campo focado e é pintada sublinhada; `set_ime_cursor_area` no cursor; `´`+`a` insere `á` UMA vez (sem duplicar por `KeyEvent.text`). | shell (`despacho_metodos_janela_e_vetor.rs`, `keyboard.rs`), `ph2d-editor-core` (WidgetStore, pintura do TextInput) | ui-testkit: Preedit ⇒ composição visível e nada inserido; Commit ⇒ inserido 1×; Preedit vazio ⇒ composição some |
| T7 | **Script dentro do projeto** (depois de T1/T2) — crate de dados `ph2d-script-doc` (`ScriptLibrary`, `ScriptId`, blob `PH2DSCRIPTS` major/minor tolerante); `LuauScript` v3 aponta `ScriptId` (ou `Unimported(caminho)` para a recusa legível); o runtime lê o texto da biblioteca (hash por script, sem polling de disco); `ProjectFile.scripts` + degrau de `PROJECT_SCHEMA`; projeto antigo importa o ficheiro no load; Browse = «importar ficheiro»; nome do chunk = nome único na biblioteca. | `ph2d-script-doc` (nova), `ph2d-script`, shell (`project*.rs`), cartão | round-trip gravar→carregar byte-idêntico; projeto v189 com caminho ⇒ texto importado e objeto a correr; caminho em falta ⇒ cartão com recusa e nada vazio; dois objetos no mesmo script editado ⇒ os dois recarregam |

## Ordem

T1, T2, T3, T5, T6 em paralelo (pastas disjuntas). T4 depois de T3. T7 depois de T1+T2 (mesmos ficheiros do
`ph2d-script`). O gate de definições da T3 fecha quando a T2 entregar o `ph2d.d.luau` verdadeiro.

## Estado (15/10, todos os itens commitados)

| # | Commits | Desvio / achado medido |
|---|---|---|
| T1 | `0a465aa7c3` `85d17d78eb` | Formatos crus do Luau congelados em `ph2d-script/tests/it/diagnostico_do_luau_medido.rs`. Erro vindo de função Rust (`CallbackError`) não tem posição na mensagem: a linha sai da pilha. O erro do prazo chegava SEM pilha: a interrupção recolhe-a enquanto corre. `ModuleError`/`runtime_message` apagados (sem consumidores fora). |
| T2 | `2ed43ac8c2` `2a804cb81b` | 20 funções, 7 tipos, `HOOKS` (`init(self)`, `update(self, dt)`, `on_signal(self, signal)`) e `ScriptSelf`. Ganchos vão ao `.d.luau` como comentário `-- @hook`: declarados como globais, o `luau-lsp` avisa «built-in global is overwritten» em todo script. `ph2d-bindgen` passa a compilar o Luau C++ (custo no passo de CI). |
| T3 | `0b1d0f8b55` `45cb3177b6` | Manifesto em `crates/ph2d-code-lsp/luau-lsp.toml` (pasta sem `Cargo.toml` em `tools/*` parte todo comando cargo — glob de membros). Solver novo liga-se por `--flag:LuauSolverV2=true`; o `fflags` do `workspace/configuration` não tem efeito. O cliente filtra SÓ o «unused function» de um gancho global com nome exato. Sem rede (`unshare -rn`): comportamento e tempos iguais. |
| T4 | `e55328715a` | Tabela em §Medições. Solver antigo por omissão (novo ≈ 7× a 3 000 linhas). Completion logo após tecla custa uma rechecagem ⇒ pedir assíncrono; hover só na pausa do rato. |
| T5 | `c23e919c4a` | JetBrains Mono 2.304 variável, OFL sem RFN, família `PH2D JetBrains Mono` (+303 KB). Itálico fica para quando o realce o pedir. |
| T6 | `6d4cb154d2` `55898085c3` | Medido (sonda em [`ferramentas/imeprobe`](ferramentas/imeprobe/README.md)): X11+XIM ⇒ `Preedit("")`+`Commit("á")`; sem método de entrada (o caminho do Wayland do dono, sem ibus/fcitx) ⇒ `KeyEvent` com `Dead('´')` e depois texto `"á"` — **nunca há `Preedit`**. Por isso a shell transforma a tecla morta em composição. Nunca há inserção dupla. Só o `TextInput` pinta a composição; Board e texto vetorial recebem o `Commit` (seguimento das linhas donas). |
| T7 | `2c4c1bc4bb` `0f8984a5ce` `0e1cd8bff5` | A biblioteca é o recurso `ProjectScripts` do mundo (fora do `ProjectState`, como o `StableIdCounter`): o `restore` do undo só despawna/respawna entidades, logo nunca a rebobina. `PROJECT_SCHEMA` 190 (provisório); o braço 189 lê o ficheiro acrescentando o byte do `Vec` vazio — depende de `scripts` ser o ÚLTIMO campo. `attach_script(entity, nome)`. Shell **+217** linhas (139 de teste). ⚠️ **Regressão até a Onda 1:** sem o editor interno, o script do projeto não se edita (o editor externo já não recarrega); por isso a Onda 0 **não se integra sozinha**. |

## Smoke do dono (fim da Onda 0)

O de hoje (`PH2D_SCRIPT_SMOKE=1`), agora sem ficheiro solto: o `bob.luau` vive no projeto; um erro mostra a
LINHA no cartão Script; um acento composto aparece sublinhado enquanto se compõe num campo de texto.

## Medições

### T4 — latência do `luau-lsp` pelo cliente `ph2d-code-lsp` (15/10)

Sonda `latencia_old_vs_new_solver_interleaved` (`crates/ph2d-code-lsp/tests/it/latencia.rs`, `#[ignore]`):
dois servidores lado a lado (solver antigo × novo) no MESMO processo, 10 rodadas × blocos de 25 com o
solver inicial rodado; cada iteração = inserir uma linha (sincronização incremental) → diagnósticos →
completion → hover, e depois removê-la → completion IMEDIATA (o caminho tecla → popup). `n = 250` por célula
(⇒ o p99.9 é praticamente o máximo). Ms de parede no cliente; o «diagnostics» é contado à chegada na
thread leitora. Máquina de 32 núcleos, **sem esperar calma**.

- **Perfil:** cliente compilado em `smoke` (otimizado); o `luau-lsp` é o binário oficial 1.70.1.
- **`loadavg`:** início `6.81 17.45 30.68` · fim `4.67 15.41 29.36` (outras linhas a compilar).
- **Definições:** o `runtime/luau/ph2d.d.luau` da T2 (`2ed43ac8c2`, gerado do `api_table.rs`).
- **Módulos sintéticos:** propriedades, um helper tipado e N funções-passo com `ph2d.get/set/vec2/emit`,
  laço e tabela, todas chamadas de `update` (gerador na sonda). Os `2` diagnósticos de cada doc eram o lint
  `FunctionUnused` em `init`/`update`, hoje filtrado (item 5; a corrida sem rede já mostra `0`).
- Re-correr: `bash scripts/cargo-test-narrow.sh ph2d-code-lsp --profile smoke -- --ignored latencia`, e o
  binário `target/smoke/deps/it-*` com `--ignored latencia --nocapture` via `scripts/ph2d-run.sh` para ver a tabela.

Cada célula: **mínimo / p50 / p99 / p99.9** em ms.

| doc (linhas) | solver | 1.º check | didChange→diagnostics | didChange→completion | completion (texto já checado) | hover | diag. | RSS MB |
|---|---|---|---|---|---|---|---|---|
| bob.luau (51) | antigo | 3.2 | 0.2 / 0.3 / 0.4 / 0.6 | 0.5 / 0.5 / 0.7 / 1.2 | 0.2 / 0.3 / 0.4 / 0.5 | 0.2 / 0.2 / 0.3 / 0.4 | 2 | 32 |
| bob.luau (51) | novo | 4.1 | 0.9 / 0.9 / 1.7 / 2.0 | 1.0 / 1.1 / 1.7 / 2.0 | 0.2 / 0.2 / 0.4 / 0.4 | 0.0 / 0.0 / 0.1 / 0.2 | 2 | 63 |
| m200 (197) | antigo | 0.7 | 0.6 / 0.6 / 1.2 / 1.5 | 0.8 / 0.8 / 1.1 / 1.3 | 0.2 / 0.2 / 0.3 / 0.4 | 0.5 / 0.5 / 0.7 / 0.9 | 2 | 32 |
| m200 (197) | novo | 2.3 | 2.1 / 2.1 / 2.8 / 3.8 | 2.2 / 2.3 / 3.3 / 3.6 | 0.2 / 0.2 / 0.3 / 0.4 | 0.0 / 0.0 / 0.1 / 0.2 | 2 | 63 |
| m1000 (992) | antigo | 2.9 | 2.6 / 2.6 / 3.5 / 3.8 | 2.8 / 2.9 / 3.5 / 4.1 | 0.2 / 0.3 / 0.4 / 0.5 | 2.3 / 2.4 / 2.7 / 3.8 | 2 | 32 |
| m1000 (992) | novo | 13.2 | 10.7 / 12.2 / 15.6 / 16.2 | 11.0 / 12.5 / 16.3 / 18.0 | 0.2 / 0.3 / 0.5 / 0.5 | 0.0 / 0.1 / 0.2 / 0.2 | 2 | 63 |
| m3000 (2 987) | antigo | 8.7 | 7.6 / 8.0 / 9.3 / 11.0 | 7.9 / 8.7 / 11.0 / 11.5 | 0.3 / 0.5 / 0.7 / 0.9 | 6.7 / 7.2 / 8.6 / 9.3 | 2 | 32 |
| m3000 (2 987) | novo | 59.1 | 47.4 / 55.2 / 63.6 / 66.1 | 47.3 / 55.1 / 66.6 / 78.7 | 0.3 / 0.4 / 0.5 / 0.6 | 0.1 / 0.1 / 0.1 / 0.2 | 2 | 63 |

Duas corridas anteriores com definições provisórias (a API de `host.rs`, antes da T2; `loadavg` `9.8`–`33`)
deram os mesmos mínimos/p50 (±0.6 ms; m3000 novo p50 `50.7`–`58.8`) e caudas até `4×` maiores (m3000 novo
p99 `215`, m1000 antigo p99 `17.9`): **as caudas são carga, os mínimos são o custo**. O «1.º check» é UMA
amostra (o primeiro quadro é um evento próprio) e variou `47`–`263` ms no m3000 novo entre corridas.

**O que a tabela decide (orçamentos da Onda 1–2):**

1. **Solver antigo por omissão** (= a omissão do `luau-lsp`): até 3 000 linhas o re-check custa `≤ 8` ms
   de p50; o novo custa `~55` ms a 3 000 linhas (`7×`) e `12` ms a 1 000, com o dobro da RSS. O novo fica
   como opção do projeto, nunca omissão, até nova medição.
2. **A completion pedida logo após a tecla paga o re-check** (didChange→completion ≈ didChange→diagnostics);
   sobre texto já checado ela custa `≤ 0.6` ms. ⇒ o editor **nunca bloqueia o quadro à espera do LSP**: o
   pedido é assíncrono e o popup aparece quando chega (orçamento: um quadro a 60 Hz até ~3 000 linhas no
   solver antigo, p50).
3. **No solver antigo o hover custa um re-check** (`7.2` ms de p50 a 3 000 linhas, contra `0.1` no novo) — pedir
   hover só em pausa do rato, não a cada movimento.
4. **Interruptor do solver:** `--flag:LuauSolverV2=true` (ou `initializationOptions.fflags`). A opção
   `fflags.enableNewSolver` respondida por `workspace/configuration` **não muda nada** (medido: 12–16 ms
   igual ao antigo, contra 260–330 ms com a flag, num módulo de 3 000 linhas).
5. **Os `2` diagnósticos de cada doc** eram o lint `FunctionUnused` (código `8`) em `init`/`update`: globais
   que o MOTOR chama, que o `luau-lsp` não sabe. Medido (15/10): declará-las no `.d.luau` piora
   (`+BuiltinGlobalWrite`); `globals` num `.luaurc` não silencia; `--base-luaurc` com
   `{"lint":{"FunctionUnused":false}}` silencia, mas apaga também o aviso de função LOCAL não usada.
   **Resolvido:** `api_table::HOOKS` (init · update · on_signal) sai no `ph2d.d.luau` como `-- @hook`, e o
   `ph2d-code-lsp` tira SÓ o `FunctionUnused` de uma `function <gancho>` GLOBAL com o nome exacto (gate
   `ganchos::*`: `helper` local, `updte` e um `local function on_signal` continuam acusados). O `self`
   estrito tipa-se com `ScriptSelf = { id: Entity, [string]: any }`, exportado nas mesmas definições.
6. **Sem rede (`unshare -rn`, 15/10):** os 17 gates passam iguais (3 + 3 corridas intercaladas, `0.03` s cada) e a
   sonda dá os mesmos números (m3000 antigo `7.4 / 7.8` com rede × `7.4 / 7.7` sem; novo `44.5 / 47.0` ×
   `44.8 / 47.2` ms, min / p50, `loadavg` `1.8`–`2.4`) ⇒ com `fflags.sync=false` o `luau-lsp` não usa a rede.
