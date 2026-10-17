# I0b — o que o luau-lsp 1.70.1 RESPONDE sobre as nossas entradas (Onda 2)

**Reproduzir** (da raiz da worktree, ~20 s): `python3 -I docs/CodeEditor/ferramentas/lspprobe2.py`
→ regrava `caps_luau_lsp_1.70.1.json` e `I0b_bruto.json` (todas as respostas cruas; este texto é
leitura delas). Só stdlib.

- **Servidor:** `~/.cache/ph2d/luau-lsp/1.70.1/luau-lsp` (`--version` = `1.70.1`), lançado como o
  cliente Rust: `lsp --stdio --definitions:@ph2d=runtime/luau/ph2d.d.luau`, `rootUri: null`,
  `workspace/configuration` (secção `luau-lsp`) = `{"platform":{"type":"standard"},"fflags":{"sync":false}}`
  (`ph2d_settings`). Documentos só em memória: `file:///ph2d-project/<nome>.luau`.
- **Entradas:** variantes de `crates/ph2d-code-syntax/tests/it/fixtures/bob.luau` (sem as 2 linhas de
  cabeçalho, 51 linhas, como a `bancada.rs`); 3 000 e 7 500 linhas pelo `juntado()` da `bancada.rs`.
- **Godot (oráculo):** `godot --version` = `4.7.2.stable.arch_linux.ed1daf0bf`; `pacman -Qi godot`:
  `4.7.2-2.1`, **MIT**.
- **Data:** 2026-10-16. **loadavg:** corrida A (só tempos) `5.83 15.04 15.97` → `5.54 14.68 15.84`;
  corrida final (o bruto) `28.43 20.20 17.70` → `28.05 20.52 17.85`. Os tempos das duas batem (tabela §6).

Legenda: **[MEDIDO]** = resposta do servidor/Godot gravada no bruto; **[INFERIDO]** = conclusão minha.

## §1 Capacidades (`initialize`)

[MEDIDO] Ficheiro: `caps_luau_lsp_1.70.1.json`. As capacidades **não mudam** com capacidades de cliente
mais ricas (`caps_extended_igual_caps_rust: true`). Destaques:

| Capacidade | Resposta |
|---|---|
| `textDocumentSync.change` | `2` (incremental) |
| `completionProvider` | gatilhos `. : ' " / \n`; **`resolveProvider: false`**; `labelDetailsSupport: true` |
| `signatureHelpProvider` | gatilhos `(` `,` |
| `codeActionProvider` | kinds `quickfix, refactor.extract, refactor.inline, source, source.organizeImports`; `resolveProvider: true` |
| `semanticTokensProvider` | `full: true`, **`range: false`**; legenda com 23 tipos / 10 modificadores |
| `inlayHintProvider`, `documentSymbolProvider`, `hoverProvider`, `definitionProvider`, `typeDefinitionProvider`, `referencesProvider`, `renameProvider`, `foldingRangeProvider`, `colorProvider`, `callHierarchyProvider`, `workspaceSymbolProvider` | `true` |
| `diagnosticProvider` | pull, `interFileDependencies: true`, `workspaceDiagnostics: true` |
| `prepareRename` | **não suportado** (`-32601 method not found`) |
| `positionEncoding` | `utf-16` |

O servidor pede `workspace/configuration` uma vez, só a secção `luau-lsp`.

## §2 Erros de digitação — diagnósticos e *code actions*

[MEDIDO] Ruído comum a TODO o `bob.luau` (não listado por caso abaixo): `FunctionUnused` (code `8`,
severidade 2, tag `1`=Unnecessary) para `init` e `update` em ambos os modos; e em `--!strict` mais
**4×** code `1030` *«Unknown type used in + operation; consider adding a type annotation to 'self'»*
(o `self` dos ganchos não é tipado pelo `ScriptSelf` — o d.luau só o declara num comentário `@hook`).
Linhas 0-based; em `--!strict` tudo desce 1 linha.

| Caso | não-estrito | `--!strict` |
|---|---|---|
| a `pirnt("x")` | `1001` sev1 *«TypeError: Unknown global 'pirnt'; consider assigning to it first»* L31:2-7 | igual |
| b1 `ph2d.proprety(...)` | `1002` sev1 *«TypeError: Key 'proprety' not found in table '{ attach_script: …, color: …, … *TRUNCATED*'»* — **567 caracteres**, despeja o tipo inteiro do `ph2d`; range = `ph2d.proprety` inteiro (L7:0-13) | igual |
| b2 `ph2d.gte(...)` | `1002` idem *«Key 'gte' not found in table …»* L22:16-24 | igual |
| c `self.ampiltude` (com `amplitude` declarada e usada) | **NADA** | **NADA** |
| d1 `ph2d.set(self.id,"x","fast")` | `1000` *«TypeError: Expected this to be 'number', but got 'string'»* só no argumento (L43:25-31) | igual |
| d2 `ph2d.set(self.id,"z",…)` | `1000` *«Expected this to be '"rotation" \| "scale_x" \| "scale_y" \| "x" \| "y"', but got '"z"'; none of the union options are compatible»* | igual |
| d3 `ph2d.vec2(0,"1")` | `1000` *«Expected this to be 'number', but got 'string'»* | igual |
| e1 `if self.wave = "square" then` | `1014` *«SyntaxError: Expected 'then' when parsing if statement, got '='»* (no `=`) + `1014` *«Expected identifier when parsing expression, got 'then'»* + lint `5` SameLineStatement (sev2) | igual |
| e3 `if x = 1 then x = 2 end` | os mesmos 3 (`1014`,`1014`,`5`) | igual |
| e2 falta um `end` | `1014` *«Expected 'end' (to close 'function' at line 29), got <eof>; did you forget to close 'then' at line 47?»* — range **vazio no EOF** (L50:0-0, linha que não existe no texto); as linhas citadas são **1-based** | igual |
| f `local k = 5; k()` | **NADA** | `1021` *«TypeError: Cannot call a value of type number»* |

Todos com `source: "Luau"` e `codeDescription.href` (`https://luau.org/types` ou `…/lint#…`).

**Code actions** (pedido no range do diagnóstico com `context.diagnostics` preenchido; 495 acções
gravadas): o ÚNICO `quickfix` que existe é **«Prefix '<nome>' with '_' to silence»** (insere `_`), só
para o lint `FunctionUnused`. O resto, em todo diagnóstico, é genérico e sem edições (vêm a resolver):
«Sort requires», «Sort services» (`source.organizeImports`), «Extract to function»/«Extract to local
variable» (`refactor.extract`), «Inline variable 'self'» (`refactor.inline`).
- **«Did you mean» nunca aparece** (0 ocorrências em todo o bruto). **Nenhum quickfix corrige um typo**,
  nem `=`→`==`, nem acrescenta `end`.

## §3 Completação

[MEDIDO] **Depois de `ph2d.`**: 17 itens, 0,52 ms, 6 020 bytes. O conjunto de rótulos é **idêntico** aos
17 membros declarados no `ph2d.d.luau` (nenhum a mais, nenhum a menos). Cada item:
`kind: 3` (Function), `detail` = tipo **expandido** (`(number, "rotation" | … | "y", number) -> ()` —
os aliases `Entity`/`PoseField` somem; `StateValue` vira `… where t1 = …`), `labelDetails.detail` =
nomes dos parâmetros (`(entity, field, value)`), **`documentation` PRESENTE** em markdown = o comentário
`---` do d.luau, `insertText: "set()"` (com `snippetSupport: false` já põe os parênteses),
`sortText: "1"` em todos (a ordem fica por nossa conta), `preselect: false`, `deprecated: false`.

**Dentro de argumentos** (`sortText`, conforme a tabela de prioridades do luau-lsp: `2` tipo certo,
`3` função que devolve o tipo certo, `4` resto, `8` palavras-chave, `9` desprioritizado):

| Posição | itens | `sortText` presentes | topo |
|---|---|---|---|
| `ph2d.set(|)` (1.º arg, `number`) | 55 | 2,3,4,8,9 | `s`(number) `2`, `rawlen`/`gcinfo`/`error`/`getmetatable`/`require` `3` |
| `ph2d.set(self.id, "|")` (string literal `PoseField`) | 5 | 2 | **`x y rotation scale_x scale_y`** (`kind: 21` Constant) — só os da união |
| `ph2d.set(self.id, "x", |)` (3.º arg, `number`) | 55 | 4,8,9 | **nenhum** com `2`/`3`; `s` com `4` como os globais |
| `ph2d.set(self.id, |)`, `ph2d.vec2(1, |)`, `local q: number = |` | 55 | 4,8,9 | idem: sem ordenação por tipo |

⇒ ordenação por tipo existe **só no 1.º argumento**; do 2.º em diante tudo vem `4`. Locais `any`
(`brilho`, `d`, `_G`) saíram `2` na sessão com as capacidades estendidas e `4` numa sessão só com as do
cliente Rust — não é estável entre sessões (não investigado).

**Documentação por base JSON (`--docs=<json>`)** [MEDIDO]:
- **Funciona pela flag de linha de comando** (`--docs=` / `--documentation=`). **Pela configuração
  `types.documentationFiles` NÃO** (com `rootUri: null` o servidor regista «No documentation file
  given» e responde com o comentário `---`).
- O símbolo de um membro é **`@ph2d/global/ph2d.set`** (o nome do pacote `@ph2d` da flag
  `--definitions`); `@ph2d/global/ph2d` documenta a tabela; a chave sem prefixo (`ph2d.set`) é ignorada.
- Formato: `{"documentation": "...", "params": [{"name","documentation"}], "returns": [...],
  "learn_more_link": "...", "code_sample": "..."}`. A documentação de **parâmetro** só aparece quando o
  campo `documentation` do parâmetro é o **símbolo de outra entrada** (`"@ph2d/global/ph2d.set/param/0"`
  → `{"documentation": "o objeto"}`); texto directo ali → parâmetro com documentação **vazia**.
- Com a base, a entrada **substitui** o comentário `---` em completação, hover e signatureHelp, e o
  servidor acrescenta `[Learn More](link)` e o `code_sample` num bloco ```` ```luau ````.
- Sem base, o comentário `---` do d.luau já chega à completação e ao hover, mas **não** ao signatureHelp.

## §4 signatureHelp

[MEDIDO] `ph2d.set(self.id, |)`: 1 assinatura, `activeParameter: 1` (no topo e na assinatura),
`label` = `function ph2d.set(entity: number, field: "rotation" | "scale_x" | "scale_y" | "x" | "y", value: number): ()`,
`parameters[].label` = **offsets** `[início, fim]` dentro do label (`[18,32]`, `[34,87]`, `[89,102]`).
`documentation` da assinatura e dos parâmetros = **`""`** sem base JSON; com a base JSON (§3) a da
assinatura e — com símbolos — as dos parâmetros vêm preenchidas. 0,4–1,1 ms.
`ph2d.property("k", 1, |)` → `activeParameter: 2`, `options: PropertyOptions?` (aqui o alias fica).

## §5 Hover e definição

[MEDIDO]
- `ph2d.set`: ```` ```luau\nfunction ph2d.set(entity: number, field: …, value: number): ()\n``` ```` +
  `----------` + o comentário `---` (ou a entrada da base JSON).
- `ph2d` (a tabela): `type ph2d = { … *TRUNCATED*` + o `---` da declaração.
- local `brilho`/`d`: `local brilho: any` (não-estrito infere `any` através de `self.tint.*`).
- `self.amplitude`: `number`. `function update`: assinatura genérica inferida pelo uso
  (`update<T, U, …>(self: { active: T, amplitude: number, base_x: U, … }, dt: D)`); `dt` → `local dt: T`.
  ⇒ o `self` dos ganchos é **inferido estruturalmente** pelo que o corpo lê — por isso `self.ampiltude`
  não é erro (vira só mais um campo inferido).
- `definition` num uso de local → a declaração (`L41:8-14`); em `ph2d.set` → **`[]`** (o d.luau não
  é navegável); em `self.t` → a própria escrita `self.t` da linha.

## §6 inlayHint, semanticTokens, documentSymbol — conteúdo e tempo

[MEDIDO] **inlayHint é DESLIGADO por omissão**: com a configuração do cliente Rust a resposta é `[]`.
Ligado com `inlayHints: {parameterNames: "all", variableTypes: true, parameterTypes: true,
functionReturnTypes: true}` (na mesma resposta de `workspace/configuration`): 45 dicas no bob — 36
`kind: 2` (nomes de parâmetro `name:`, `default:`, `value:`, `signal:` antes dos argumentos) e 9
`kind: 1` (tipos, `label` em partes `[": ", "boolean"]`, com `textEdits` inseríveis).
⚠️ **O `range` do pedido é IGNORADO**: pedir 50 linhas no meio do ficheiro de 7 500 devolve as mesmas
**3 257** dicas das linhas 6–7 493 (497 718 bytes) que pedir o documento inteiro.
documentSymbol (hierárquico): no bob só `init` e `update` (kind 12) com filhos `self`, `dt`, locais
(kind 13). semanticTokens/full: 106 tokens no bob (sem `range`).

Tempos em ms, 20 repetições intercaladas (doc × pedido × modo) por cliente Python, a contar do envio
à leitura da resposta. *Quente* = documento sem mudança; *após edição* = `didChange` (troca um
carácter num comentário) imediatamente antes de cada pedido (inclui o re-check). Corrida final
(load 28):

| Documento | Pedido | quente min / p50 | após edição min / p50 | bytes da resposta |
|---|---|---|---|---|
| bob 51 l. | inlayHint (doc todo) | 0,14 / 0,88 | 0,53 / 0,87 | 6 933 |
| | inlayHint (janela 50 l.) | 0,14 / 0,27 | 0,35 / 0,62 | 6 933 |
| | semanticTokens/full | 0,06 / 0,11 | 0,26 / 0,46 | 1 267 |
| | documentSymbol | 0,06 / 0,09 | 0,09 / 0,15 | 2 176 |
| 3 000 l. | inlayHint (doc todo) | 6,66 / 12,02 | 7,79 / 11,10 | 200 239 |
| | inlayHint (janela 50 l.) | 2,90 / 5,32 | 7,64 / 10,31 | 200 239 |
| | semanticTokens/full | 2,14 / 3,86 | 6,56 / 8,12 | 34 809 |
| | documentSymbol | 2,08 / 3,53 | 2,98 / 4,20 | 129 238 |
| 7 500 l. | inlayHint (doc todo) | 21,56 / 28,67 | 21,81 / 33,68 | 497 720 |
| | inlayHint (janela 50 l.) | 10,88 / 16,43 | 22,82 / 31,16 | 497 720 |
| | semanticTokens/full | 12,62 / 16,01 | 24,49 / 30,37 | 86 239 |
| | documentSymbol | 7,49 / 11,05 | 10,48 / 15,85 | 324 280 |

Corrida A (load 5,8, sem a coluna «janela»), p50 quente/após edição no 7 500: inlayHint 29,65 / 32,08 ·
semanticTokens 15,74 / 33,67 · documentSymbol 11,66 / 11,60 — a mesma ordem que a corrida final.
Primeiro diagnóstico (pull) depois de abrir: 0,4–0,6 ms (bob) · 8–11 ms (3 000) · 18–24 ms (7 500).
A «janela» devolve o mesmo payload; a diferença no *quente* é do lado do servidor, não do tamanho.

## §7 Referências e rename entre dois documentos em memória

[MEDIDO] A = `function helper(x) … end` + `local function so_aqui`; B = `update` que chama `helper(3)`.
- B tem **`1001 Unknown global 'helper'`**: uma global de outro script **não existe** em B (cada script
  é um módulo com o seu próprio ambiente).
- `references` de `helper` em A → só as 2 ocorrências de A. Em B → só a de B. `rename` idem: edições
  **só no ficheiro do pedido**. `definition` do `helper` em B → `[]`.
- Local `so_aqui`: references/rename ficam em A (2 sítios) — correcto.
- `prepareRename` não existe (−32601).

## §8 Godot — atraso do tooltip e da completação

[MEDIDO] Plugin de editor num projecto temporário, `godot --headless --editor` com `XDG_*` temporários
(valores **de fábrica**, sem tocar no `~/.config`):
- `ProjectSettings gui/timers/tooltip_delay_sec = 0.5`.
- EditorSettings: `text_editor/behavior/documentation/enable_tooltips = true` ·
  `text_editor/completion/code_complete_delay = 0.3` · `idle_parse_delay = 1.5` ·
  `idle_parse_delay_with_errors_found = 0.5` · `put_callhint_tooltip_below_current_line = true` ·
  `code_complete_enabled = true` · `add_type_hints = true` · `colorize_suggestions = true`. Não existe
  chave própria de atraso para o tooltip de símbolo do editor de scripts.
- **Tooltip de símbolo de um `CodeEdit`** (`symbol_tooltip_on_hover = true`, movimento do rato
  injectado sobre `brilho`, tempo até ao sinal `symbol_hovered`): **510 ms** de fábrica; **1 207 ms**
  com `tooltip_delay_sec = 1.2`; **111 ms** com `0.1` ⇒ segue `gui/timers/tooltip_delay_sec`.
- A config do dono (`~/.config/godot/editor_settings-4.7.tres`, só leitura) tem os mesmos valores
  (`code_complete_delay = 0.3`, `enable_tooltips = true`, `idle_parse_delay = 1.5`).

## O que isto decide

- [MEDIDO] **«Você quis dizer…?» é NOSSO.** O luau-lsp não sugere nem corrige typos (zero «did you
  mean», zero quickfix para 1001/1002/1000/1014/1021). As candidatas existem: os membros do d.luau (os
  17 rótulos da completação = os 17 declarados) e os locais/globais do documento.
- [MEDIDO] **A mensagem 1002 tem de ser reescrita por nós**: 567 caracteres com o tipo inteiro do
  `ph2d` e `*TRUNCATED*`. A chave errada está entre aspas simples no início — dá para extrair.
- [MEDIDO] **`self.ampiltude` é invisível ao servidor** nos dois modos (o `self` é inferido pelo uso;
  o `ScriptSelf` tem `[string]: any`). [INFERIDO] Só um verificador nosso que cruze os `ph2d.property`
  declarados com os `self.<campo>` lidos e nunca escritos apanha este caso.
- [MEDIDO] O caso do dono (`if x = 1 then`) dá **dois** `1014` e um lint `5` na mesma linha, sem acção;
  [INFERIDO] a mensagem e a sugestão `==` têm de ser nossas, agrupando os três num só aviso.
- [MEDIDO] Falta de `end`: range vazio no EOF e linhas 1-based no texto. [INFERIDO] Ancorar o aviso na
  linha do bloco citado («did you forget to close 'then' at line 47») e não no fim do ficheiro.
- [MEDIDO] `--!strict` acrescenta 4 avisos sobre o `self` não tipado no bob canónico. [INFERIDO] Não
  dá para recomendar strict ao artista sem tipar o `self` dos ganchos (o `@hook` é só comentário).
- [MEDIDO] `k()` com número só é erro em strict.
- [MEDIDO] **Documentação**: o `---` do d.luau já chega à completação e ao hover sem nada a fazer. Para
  signatureHelp (assinatura e parâmetros), `learn_more_link` e `code_sample`, gerar uma base JSON a
  partir do `api_table.rs` com chaves `@ph2d/global/ph2d.<membro>`, parâmetros por símbolo
  `…/param/<i>`, e passá-la por **`--docs=`** na linha de comando (a configuração não serve aqui).
- [MEDIDO] `detail` da completação perde os aliases (`Entity`, `PoseField`); `labelDetails.detail` dá os
  nomes dos parâmetros. [INFERIDO] Mostrar nomes (do `labelDetails`) e os tipos de alias vindos da
  nossa tabela, não do `detail`.
- [MEDIDO] `sortText` é `1` para todos os membros e só ordena por tipo no 1.º argumento. [INFERIDO] A
  ordem (prefixo, frequência, tipo esperado nos outros argumentos) fica para nós; os literais da união
  `PoseField` vêm prontos dentro de `"`.
- [MEDIDO] `resolveProvider: false` ⇒ a documentação vem na própria lista (6 KB para 17 itens; 13 KB
  para 55); não há segundo pedido.
- [MEDIDO] **inlayHint está desligado por omissão** e **ignora o range**: no 7 500 linhas são 3 257 dicas
  / ~0,5 MB por pedido, 22–34 ms. [INFERIDO] Se ligarmos, pedir só depois de pausar a escrita, guardar a
  resposta e filtrar a janela visível do nosso lado; ou ligar só os nomes de parâmetro.
- [MEDIDO] semanticTokens só `full` (sem range/delta): 86 KB e 25–30 ms após edição no 7 500.
  [INFERIDO] Os nossos tokens sintácticos continuam a pintar a cada tecla; os semânticos entram depois.
- [MEDIDO] **Globais não cruzam scripts** e references/rename ficam no ficheiro. [INFERIDO] «Renomear em
  todo o projecto» não vem do servidor; com o modelo actual de um ambiente por script, nem é preciso.
- [MEDIDO] Godot: tooltip de símbolo **0,5 s** (segue `gui/timers/tooltip_delay_sec`), completação
  automática **0,3 s**, re-análise **1,5 s** (0,5 s quando já há erros). [INFERIDO] São os números de
  partida do nosso hover e da nossa completação; o hover do luau-lsp responde em < 1 ms no bob, então o
  atraso é escolha de interface, não limite do servidor.
