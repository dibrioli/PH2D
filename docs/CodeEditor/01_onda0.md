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

## Smoke do dono (fim da Onda 0)

O de hoje (`PH2D_SCRIPT_SMOKE=1`), agora sem ficheiro solto: o `bob.luau` vive no projeto; um erro mostra a
LINHA no cartão Script; um acento composto aparece sublinhado enquanto se compõe num campo de texto.

## Medições

*(preenchido pela T4 — mínimo, p50, p99, p99.9 por pedido, com perfil de build e `loadavg` ao lado)*
