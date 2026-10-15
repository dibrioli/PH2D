# Godot Engine — o editor de scripts (GDScript) como referência para o editor de código do PH2D

> Pesquisa de 2026-10-15. Estado: **Godot 4.7.x é o estável** (4.7.2 instalado nesta máquina, build
> `2026-08-17`, licença MIT confirmada por `pacman -Qi godot`); **4.8 está em feature freeze** (dev 7,
> ~set/2026). A doc `docs.godotengine.org/en/stable` já é a de 4.7. Cada afirmação versionada leva a versão.
> Medições locais (oráculo) marcadas **[MEDIDO 15/10, Godot 4.7.2]**.

## 1. Lista completa de funcionalidades do editor de scripts (Godot 4.x)

### Takeaway
O editor é um `CodeEdit` (subclasse de `TextEdit`, ambos nós de UI públicos do próprio engine) embrulhado
num `ScriptEditor` com lista de ficheiros + outline + menus clássicos; o conjunto é de IDE leve
(completion, hover-docs, Ctrl+click, regiões, minimap, multi-cursor, find-in-files, debugger integrado,
LSP/DAP para fora), e o diferencial está nos ícones de gutter e na ligação com a cena, não no editor de
texto em si.

### Cited Findings
**Recursos-base (doc do Script Editor, 4.7):**
- Lista oficial: realce de sintaxe e checagem de sintaxe para GDScript e JSON; bookmarks e breakpoints;
  auto-indent e code folding; tema customizável; múltiplos cursores com `Alt+Click`; auto-completion de
  variáveis/funções/constantes; «inline symbol refactoring» seleccionando um símbolo e `Ctrl+D`
  (= seleccionar próxima ocorrência); find/replace em massa nos ficheiros do projecto; visualizador
  offline da referência de classes — [Script Editor (4.7)](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html)
- Abertura: botão **Script** no seletor de workspace; botão «Open Script» ao lado do nó na Scene Tree;
  duplo clique num `.gd` no FileSystem; botão para **destacar o editor numa janela flutuante**
  (multi-monitor); setas de histórico back/forward — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html)
- Barra de estado: contagem de erros e warnings (clicar lista-os com nº de linha e salta para a linha;
  opção «Ignore» por warning), zoom (clicar na % ou `Ctrl+roda`), linha:coluna, tipo de indentação
  (tabs/spaces), seta para ocultar o painel de scripts — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html)
- Painel esquerdo (lista de scripts): ícone por tipo (GDScript, C#, doc, tool script a correr); tooltip
  com caminho relativo; **«Script Temperature»** (realce dos editados recentemente); filtro fuzzy
  (letras em ordem: `btn` casa `button.gd`); `*` no filtro mostra só os não-salvos; `*` no nome = não
  salvo; arrastar reordena; clique do meio fecha; menu de contexto (salvar/fechar, copiar caminho
  relativo, mover, ordenar, Close Docs, Show in FileSystem) — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html)
- Outline/«members»: nome do ficheiro; alternar ordem alfabética × ordem de fonte; lista de métodos (ou
  sumário, numa página de doc); clicar salta; «Filter Methods» fuzzy — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html).
  Em **4.8 dev 4 (26/08/2026)** o outline passa de lista a **árvore** (GH-121534) e a selecção deixa de
  se perder ao salvar — [4.8 dev 4](https://godotengine.org/article/dev-snapshot-godot-4-8-dev-4/)
- Menu File: New Script, New Text File (inclui JSON), Open/Reopen Closed/Open Recent, Save/Save As/Save
  All, **Soft Reload Tool Script**, Copy Script Path (`res://`), History Prev/Next (se o cursor andou >10
  linhas, volta primeiro à posição anterior no mesmo ficheiro), Theme import/save/reload, Close
  All/Other/Docs, **Run** (para `EditorScript`) — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html)
- Menu Edit: Duplicate Selection/Lines; **Evaluate Selection** (calcula expressão matemática pura);
  Word Wrap (visual); Line ▸ Move Up/Down, Indent/Unindent, Delete, Toggle Comment; Folding ▸ Fold/Unfold,
  Fold All, Unfold All, **Create Code Region**; Completion Query (`Enter`/`Tab` aceita; `Tab` também
  substitui o texto à direita do cursor); Trim Trailing Whitespace / Final Newlines; converter
  indentação; Convert Case; escolher Syntax Highlighter (Plain/Standard/JSON/GDScript) — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html)
- Menu Search: Find (Match Case, Whole Words), Find Next/Prev, Replace (um/todos, «Selection Only»),
  **Find in Files** (resultados no painel inferior «Search Results»), **Replace in Files** (checkboxes por
  ficheiro, só aplica em Replace All), **Contextual Help** (= `F1`/Lookup Symbol) — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html).
  Em 4.8 dev 4: pré-visualização do replace em Find in Files (GH-122659) — [4.8 dev 4](https://godotengine.org/article/dev-snapshot-godot-4-8-dev-4/)
- Menu Go To: Go to Function, Go to Line, Bookmarks (toggle, remover todos, próximo/anterior, lista),
  Breakpoints (idem) — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html)
- Área de código: números de linha; **minimap clicável**; guias de comprimento **soft 80 / hard 100**
  por omissão; ícones de gutter: **override** (abre a doc da função original), **receiver** (mostra a
  origem do sinal ligado à função), fold, reticências na linha dobrada; bookmark azul; breakpoint =
  círculo vermelho esmaecido ao passar o rato, clicar adiciona — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html)
- Configuração em **Editor Settings ▸ Text Editor** (tema, cores, cores de bookmark/breakpoint, guias,
  Script Temperature); atalhos remapeáveis em Editor Settings ▸ Shortcuts — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html)

**API do widget (`CodeEdit`, 4.7) — o que é reutilizável por qualquer ferramenta, não só pelo editor:**
- Auto brace (`auto_brace_completion_pairs`, highlight do par); completion (`code_completion_prefixes`,
  `CodeCompletionKind`, virtuais `_request_code_completion`/`_filter_code_completion_candidates`/
  `_confirm_code_completion`); **code hint** (assinatura, `set_code_hint`, pode desenhar por baixo);
  delimitadores de comentário/string configuráveis (`is_in_comment/is_in_string`); folding + **code
  regions** (`set_code_region_tags`); gutters (números, fold, breakpoints, bookmarks, **executing line**);
  auto-indent por prefixos; `line_length_guidelines`; **symbol lookup on click** (sinais `symbol_validate`
  + `symbol_lookup`); **symbol tooltip on hover** (`symbol_hovered`); duplicate/move/delete/join lines —
  [CodeEdit class ref](https://docs.godotengine.org/en/stable/classes/class_codeedit.html)
- Code regions: `#region` / `#endregion` depois de um delimitador de comentário de linha; existem desde
  **4.2** (a página da classe 4.2 já as documenta) — [CodeEdit 4.2](https://docs.godotengine.org/en/4.2/classes/class_codeedit.html)
- 4.8 dev 1 (06/07/2026): **suporte a toque** em TextEdit/CodeEdit; `FuzzySearch` exposto à API —
  [4.8 dev 1](https://godotengine.org/article/dev-snapshot-godot-4-8-dev-1/) (via resumo de busca; ver Gaps)

**Linguagem/tipagem visível no editor:**
- **Safe lines**: quando o tipo é verificável, o **número da linha fica verde**; cor configurável ou
  desligável nas Editor Settings; a doc avisa que «safe» ≠ «melhor» (um `as` cast falha silencioso para
  `null`) — [Static typing](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html)
- Typed GDScript «usa opcodes optimizados quando os tipos são conhecidos em compilação»; código
  dinâmico dá poucas sugestões de completion — [Static typing](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html)
- `Array[int]`, `Dictionary[String, int]` (typed dictionaries em **4.4**), sem coleções tipadas aninhadas —
  [Static typing](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html) · [4.4](https://godotengine.org/releases/4.4/)
- Setting **Text Editor ▸ Completion ▸ Add Type Hints** faz o completion/código gerado escrever tipos —
  [Static typing](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html)

**Warnings:**
- Configurados em Project Settings ▸ (Debug ▸) GDScript (exige «Advanced Settings»); cada warning pode
  virar **erro** (impede compilar); `@warning_ignore("nome")`, `@warning_ignore_start` /
  `@warning_ignore_restore` (sem restore = até ao fim do ficheiro); o link «ignore» ao lado do warning
  insere a anotação; o editor autocompleta nomes de warning; nome do warning = nome do setting
  (`debug/gdscript/warnings/unused_variable` ↔ `unused_variable`) — [Warning system](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/warning_system.html)
- Warnings de tipagem desligados por omissão: `UNTYPED_DECLARATION` (exige tipos em tudo),
  `INFERRED_DECLARATION` (proíbe `:=`), família `UNSAFE_*` (`UNSAFE_PROPERTY_ACCESS`,
  `UNSAFE_METHOD_ACCESS`, `UNSAFE_CAST`) que «não cobrem todos os casos das unsafe lines» — [Static typing](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html)
- **[MEDIDO 15/10, Godot 4.7.2]** valores testados: `1` (warn) e `2` (erro; `0` presumido = ignore, não testado); com `untyped_declaration=2` o `--check-only`
  imprime `Parse Error: Variable "x" has no static type. (Warning treated as error.)` e o de retorno
  `Function "_init()" has no static return type.` — ver §7.
- 4.8 dev 3: **sublinhado preciso** do trecho que causa o warning (era a linha toda) — [4.8 snapshots (resumo)](https://www.linuxcompatible.org/story/godot-48-dev-snapshot-3-released-176-fixes-editor-polish-and-high-polling-mouse-support/); proposta original #14875 (fechada) — [godot-proposals #14875](https://github.com/godotengine/godot-proposals/issues/14875)

**Documentação embutida:**
- Doc comments `##` antes do membro (ou no topo do script = descrição da classe; inline depois da
  declaração); `@tutorial(Título): URL`, `@deprecated [msg]`, `@experimental [msg]`; BBCode:
  `[ClassName]`, `[member]`, `[method]`, `[signal]`, `[param x]`, `[codeblock lang=gdscript]`, `[b]` etc.;
  membros com `_` são privados e somem da doc salvo se tiverem doc comment; aparecem no Help do editor,
  em tooltips, e o tooltip de variáveis `@export` no Inspector; exportáveis a XML — [Doc comments](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_documentation_comments.html)
- **4.4**: hover sobre função/variável/classe mostra a doc integrada, inclusive a escrita pelo usuário com
  `##`; UIDs mostram caminho legível no hover — [4.4](https://godotengine.org/releases/4.4/)
- **4.6**: sinais com prefixo `_` somem do autocomplete e da doc (PR 112770) — [4.6](https://godotengine.org/releases/4.6/)
- **4.7**: layout da doc embutida espelha a doc online (PR 107597); propriedades dinâmicas documentadas
  (PR 115253); nomes de métodos/sinais/propriedades de script em fonte monoespaçada (PR 112219,
  setting «Use Monospace Font for Editor Symbols») — [4.7](https://godotengine.org/releases/4.7/)
- 4.7 dev: atalho para mostrar o tooltip de doc da palavra sob o cursor (GH-115767) — [busca: 4.7 changelog](https://godotengine.org/releases/4.7/)

**Debugger / runtime (integrado):**
- Breakpoints no gutter; **4.4**: aba «Expression evaluator» no painel inferior que avalia expressões com
  o estado local parado num breakpoint; janela do jogo **embutida no editor** (processo separado; Linux,
  Windows, Android em 4.4); edição interactiva in-game (clicar elementos, mover câmara com o jogo a
  correr/pausado) — [4.4](https://godotengine.org/releases/4.4/)
- **4.5**: script backtracing e custom loggers; backtraces até em release builds com
  *Debug ▸ Settings ▸ GDScript ▸ Always Track Call Stacks* — [4.5](https://godotengine.org/releases/4.5/)
- **4.6**: botão **Step Out** (PR 97758); profiling de script em Tracy/Perfetto/Instruments (PR 113279);
  **ObjectDB snapshots** comparáveis (PR 97210); controlo de velocidade do jogo no editor; erros/warnings
  do Output clicáveis abrem o script (PRs 108473, 111805) — [4.6](https://godotengine.org/releases/4.6/)
- **4.7**: Remote Inspector mostra o nome do enum em vez do inteiro (PR 115705); folding de grupos no
  Remote Scene Inspector (PR 117357) — [4.7](https://godotengine.org/releases/4.7/)

**Externo (LSP/DAP):**
- Editor externo: Editor Settings ▸ Text Editor ▸ External, flags com `{project} {file} {line} {col}`
  (VS Code: `{project} --goto {file}:{line}:{col}`); **desde 4.5** as flags são auto-detectadas para os
  editores listados; «Auto Reload Scripts on External Change» — [External editor](https://docs.godotengine.org/en/stable/tutorials/editor/external_editor.html)
- LSP porta **6005**, DAP porta **6006**; exige uma instância do Godot a correr com o projecto aberto;
  settings em Network ▸ Language Server / Debug Adapter — [External editor](https://docs.godotengine.org/en/stable/tutorials/editor/external_editor.html)
- **4.6**: o LSP converte BBCode das docstrings para Markdown com mais fidelidade (PR 113099) — [4.6](https://godotengine.org/releases/4.6/)

**Acessibilidade:**
- **4.7**: navegação por landmarks para leitores de ecrã (PR 114449) — [4.7](https://godotengine.org/releases/4.7/)

### Inferences
- O editor é «simples por fora» porque a superfície visível é um editor de texto + 2 listas; a riqueza
  está em ícones passivos (override/receiver/fold/safe-line verde) que **informam sem pedir acção**.
- Muitas features são propriedades do widget `CodeEdit` (genérico, reutilizado pelo editor de shaders,
  por plugins e por jogos), não lógica do «ScriptEditor» — bom modelo para o PH2D: um widget de código
  com ganchos (completion/lookup/hover/gutters) e o idioma (Lua/expressões) pluga por sinais.

### Gaps
- Color picker inline: confirmado em 4.5 (ver §4), mas a página do Script Editor 4.7 não o descreve.
- Profiler/monitores/remote scene tree em detalhe (página de debugger não lida). Hot reload de GDScript
  em jogo a correr («live editing» de script) não confirmado por fonte nesta pesquisa.
- Fontes com ligaduras e text-to-speech: não encontrei fonte oficial nesta pesquisa.
- Lista exaustiva dos ~50 warnings e o seu nível por omissão: não extraída.

## 2. Integração com cena/nós — o que faz «parecer fácil»

### Takeaway
A facilidade vem de o editor **conhecer a cena aberta**: arrastar nó gera `$Caminho` (com Ctrl gera a
`@onready var` tipada), arrastar recurso gera `preload`/(4.6) `@export`, o gutter mostra que sinal liga
a esta função, `@export` vira campo no Inspector com tooltip vindo do `##`, e `@tool`/`EditorScript`
correm dentro do editor.

### Cited Findings
- Arrastar nó da Scene Tree para o script insere `$NomeDoNo`; segurando **Ctrl** ao soltar insere
  `@onready var node_name: NodeType = $NodeName` — [forum.godotengine.org](https://forum.godotengine.org/t/super-noob-problem/134816) · [forum (sem Ctrl)](https://forum.godotengine.org/t/existing-scene-returns-null-when-referenced/135381)
- O tipo só aparece se «Add Type Hints» estiver ligado; relatos de que o modificador depende de o nó
  estar seleccionado no início do arrasto; no Mac é `Cmd` (a partir de 4.2 beta) — [gamedev.tv](https://community.gamedev.tv/t/ctrl-dragging-variables-doesnt-work-on-windows/246011) · [gamedev.tv (Mac)](https://community.gamedev.tv/t/currently-on-mac-how-do-i-drag-the-node-to-make-the-onready-variable-appear/234416)
- **4.5**: soltar recurso com Ctrl faz `preload` **por UID** em vez de caminho — [4.5](https://godotengine.org/releases/4.5/)
- **4.6**: arrastar um recurso para o editor **cria uma variável `@export`** automaticamente; scripts
  novos abrem imediatamente; menu na barra de abas listando cenas/scripts abertos; trocar de aba ao
  pairar durante o arrasto (PR 103478); placeholders `%s`, `%d`, `{name}` realçados em strings (PR 112575) — [4.6](https://godotengine.org/releases/4.6/)
- Gutter «receiver»: ícone na função ligada a um sinal mostra a origem do sinal; «override» abre a doc
  da função herdada — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html)
- `@export_tool_button` (4.4) cria botões no Inspector a partir de `@tool` scripts — [4.4](https://godotengine.org/releases/4.4/)
- **4.5**: `EditorScript` nomeados aparecem automaticamente na **Command Palette** — [4.5](https://godotengine.org/releases/4.5/)
- UIDs (4.4): recursos referenciados por UID para que mover ficheiros não parta referências — [4.4](https://godotengine.org/releases/4.4/)
- Um usuário de fórum: o editor interno «controla tanto o reload, a análise e a avaliação» que editores
  externos ficam mais difíceis com GDScript; e conflitos: o Godot salva o conteúdo do painel por cima do
  que foi editado no VS Code ao relançar o debugger — [gdscript.com / forum (resumo da busca)](https://forum.godotengine.org/t/godot-and-vs-code/72896)

### Inferences
- O par **nó-arrastado → código-gerado** e **código → Inspector (`@export` + `##`)** é a costura que o
  PH2D deve copiar: objecto da Hierarchy arrastado para o editor gera a referência por `StableId`/nome
  (a lei do §5 do PH2D), nunca por bits.
- O gutter «receiver» é a forma barata de mostrar ligações ECS (quem emite o evento que este handler lê).

### Gaps
- Não há doc oficial do Ctrl-drag; o comportamento vem de fóruns (oráculo pode medir — §7).
- Autocomplete de `$Node`/`get_node()` com caminhos da cena aberta e de strings de input actions:
  a proposta #8962 está fechada, mas não confirmei a versão em que entrou — [busca de propostas](https://github.com/godotengine/godot-proposals/issues/8962)

## 3. O que os usuários elogiam e de que se queixam

### Takeaway
Elogio: integração e «já está lá». Queixas: **linguagem** (nullable, structs, interfaces, traits),
**ergonomia de IDE** (abas, split/múltiplas janelas, formatter, rename — este só chega em 4.8),
e o editor interno que briga com o externo. Vim mode foi recusado.

### Cited Findings (contagem de 👍 lida pela API do GitHub em 15/10/2026)
- Mais votadas ligadas a GDScript/editor: **#162 nullable static types (+878, aberta)**; #7329 structs
  (+708); #4872 interfaces (+591); **#899 refactor/rename (+475, FECHADA como completed em 28/09/2026)**;
  #6416 traits (+420); #737 union types (+417); **#622 Vim mode (+378, fechada «not_planned» em
  05/06/2021)**; #157 f-strings (+332); #641 public/private + autocompletion (+322); #1571 overloading
  (+286); #1566 namespaces (+255); **#3630 formatter de GDScript no editor (+225, aberta)** — [godot-proposals](https://github.com/godotengine/godot-proposals/issues/162) · [#899](https://github.com/godotengine/godot-proposals/issues/899) · [#622](https://github.com/godotengine/godot-proposals/issues/622) · [#3630](https://github.com/godotengine/godot-proposals/issues/3630)
- Só do editor: **#4081 scripts e docs como abas acima do texto, «como outros IDEs» (+114)**; #984 dividir
  o editor em várias janelas flutuantes (+54); #8403 regiões no outline e no minimap (+31); #15415 hover de
  diagnóstico com ícone de severidade, botão copiar e **quick-fix** (+17); #13634 impedir o editor de
  roubar foco (+16); #2059 desligar o editor interno ao usar externo (+14); #10199 indicador de TODO
  (+12); #15596 outline com todos os símbolos (+11); #12863 **inlay hints** (+10); #5759 editor interno e
  externo em simultâneo (+8); #15303 RegEx no find (+4); #15228 separar docs dos scripts (+4) — [godot-proposals](https://github.com/godotengine/godot-proposals/issues/4081) · [#984](https://github.com/godotengine/godot-proposals/issues/984) · [#15415](https://github.com/godotengine/godot-proposals/issues/15415)
- Autocomplete: já fechadas (feitas) — ordenar por hierarquia de classe (#99, +37), enum values primeiro
  (#7913, +26), não pôr parênteses dentro de `signal.connect` (#6728, +26), ordenação mais útil (#4189,
  +17); abertas — sugestões pelo **tipo esperado** (#10665), tipos literais de string estilo TS (#14239) —
  [#99](https://github.com/godotengine/godot-proposals/issues/99) · [#10665](https://github.com/godotengine/godot-proposals/issues/10665)
- Rename: PR #102380 (F2, menu de contexto) ficou por fechar sem merge; o **PR #117949 «Add simple symbol
  renaming»** foi mergeado em 28/09/2026, milestone 4.8 — abordagem simples: lookup do símbolo + Find in
  Files exacto + filtro pelos resultados do lookup; distingue identificador de string/comentário e locais
  homónimos em métodos diferentes; renomeia entre ficheiros; o autor admite falsos negativos (bug
  #117948) mas «não deveria haver falsos positivos» — [PR 117949](https://github.com/godotengine/godot/pull/117949) · [PR 102380](https://github.com/godotengine/godot/pull/102380)
- O anúncio de 4.8 dev 7 (feature freeze) chama-o de maior adição do editor; cobertura independente diz
  que é **experimental/opt-in** — [4.8 dev 7](https://godotengine.org/article/dev-snapshot-godot-4-8-dev-7/) · [warp2search](https://www.warp2search.net/story/godot-48-dev-7-enters-feature-freeze-symbol-renaming-and-more/)
- Enquanto não vinha, a comunidade fez plugin de refactor (ago/2026) — [itch.io plugin](https://quanty-bandit.itch.io/gdscript-refactoring-plugin)
- Comunidade: VS Code escolhido por rename, vim bindings, split view, extensões e git; built-in escolhido
  por hábito e integração; muitos usam os dois — [forum: Godot and VS Code](https://forum.godotengine.org/t/godot-and-vs-code/72896) · [godotforums](https://godotforums.org/d/37797-new-to-godot-but-not-code-hows-your-vscode-based-gd-editing-workflow) · [gdscript.com](https://gdscript.com/articles/godot-code-editors/)

### Inferences
- A dor maior **não é o editor** — é o sistema de tipos (nullable/union/interfaces). Para o PH2D com Lua
  (Luau-like tipagem gradual?) isto diz: a qualidade da completion depende de tipos; investir no
  modelo de tipos rende mais que features de editor.
- Abas/split/janelas flutuantes e formatter são os gaps de editor mais votados — baratos de não errar
  desde o começo.

### Gaps
- Reddit directamente (r/godot) não foi lido; os resumos vêm de fóruns e de um espelho. Sem números de
  sentimento agregados.

## 4. O que mudou recentemente (4.2 → 4.8 dev)

### Takeaway
Ritmo constante de polimento: regiões (4.2), typed dicts + hover-docs + evaluator + jogo embutido (4.4),
color picker + abstract/variadic + backtraces (4.5), drag-to-@export + Step Out + LSP Markdown (4.6),
monospace/doc layout/acessibilidade (4.7), rename + outline em árvore + sublinhado preciso + toque (4.8 dev).

### Cited Findings
- **4.2**: code regions `#region/#endregion` — [CodeEdit 4.2](https://docs.godotengine.org/en/4.2/classes/class_codeedit.html); tipar a variável do `for` — [Static typing](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html)
- **4.4** (4.4.1 em 26/03/2025): typed dictionaries, `@export_tool_button`, UIDs, tooltips de doc no
  hover, expression evaluator, jogo embutido, edição in-game — [4.4](https://godotengine.org/releases/4.4/)
- **4.5** (meados de set/2025): **color preview + picker ao lado de todo `Color(...)`**; EditorScripts na
  command palette; preload por UID com Ctrl; argumentos variádicos `...numbers: Array`; **classes e métodos
  abstratos**; qualificador `required` na doc de virtuais; backtracing — [4.5](https://godotengine.org/releases/4.5/) · [W4 Games](https://www.w4games.com/blog/w4-games-news-1/how-the-release-cycle-works-godot-engine-4-5-82)
- **4.6**: drag de recurso → `@export`; scripts novos abrem; menu de abas; placeholders realçados; Step
  Out; tracing profilers; ObjectDB snapshots; LSP BBCode→Markdown; sinais `_` ocultos — [4.6](https://godotengine.org/releases/4.6/)
- **4.7** (4.7.2 em ago/2026): duplo-clique no nome do script revela-o na lista (PR 112796); símbolos em
  monoespaçado; orientação no editor Android; doc embutida com layout da online; landmarks para leitores
  de ecrã; enums nomeados no Remote Inspector — [4.7](https://godotengine.org/releases/4.7/) · [Wikipedia (4.7.2)](https://en.wikipedia.org/wiki/Godot_(game_engine))
- **4.8 dev 1–7** (jul–set/2026): toque em TextEdit/CodeEdit; sublinhado preciso de warnings (dev 3);
  outline em árvore + replace preview + «strings como comentário» proibidas (GH-121833) (dev 4); realce de
  sintaxe GDScript mais rápido (dev 6); **rename de símbolo** (dev 7, freeze) — [4.8 dev 4](https://godotengine.org/article/dev-snapshot-godot-4-8-dev-4/) · [4.8 dev 7](https://godotengine.org/article/dev-snapshot-godot-4-8-dev-7/) · [gamedev.net dev 6](https://gamedev.net/news/5756-dev-snapshot-godot-48-dev-6/)

### Inferences
- Em 2026 o Godot ainda **não** tem formatter, inlay hints, abas de IDE, split view nem quick-fixes
  oficiais — esses são os gaps a fechar no PH2D sem perder a superfície simples.

### Gaps
- Datas exactas de 4.6 e 4.7 estáveis não lidas nas páginas (a de 4.6 não traz data). Itens de 4.8 dev
  1/3/6 vêm de resumos de busca/cross-posts, não li os artigos oficiais desses três.

## 5. Como o Godot consegue a simplicidade

### Takeaway
Um único workspace «Script» com 3 zonas fixas (lista · outline · código), menus clássicos em vez de
painéis, tudo útil ligado por omissão, o rigor (tipos/warnings estritos) **opt-in** e escondido em
«Advanced Settings», e o feedback passivo no gutter/números de linha.

### Cited Findings
- Warnings de tipagem desligados por omissão; aparecem só com Advanced Settings — [Static typing](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/static_typing.html) · [Warning system](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/warning_system.html)
- Warnings nunca impedem correr o jogo, salvo se promovidos a erro — [Warning system](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/warning_system.html)
- Acções de correcção são **um clique no sítio** (link «ignore» insere a anotação; clicar no contador
  da status bar salta à linha) — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html) · [Warning system](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/warning_system.html)
- Filtros fuzzy em todas as listas (scripts e métodos) com o mesmo comportamento — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html)
- Desde 4.5 o editor externo funciona sem configurar flags (auto-detecção) — [External editor](https://docs.godotengine.org/en/stable/tutorials/editor/external_editor.html)
- Docs offline do engine e do usuário no mesmo visualizador, abertas como «ficheiros» na mesma lista — [Script Editor](https://docs.godotengine.org/en/stable/tutorials/editor/script_editor.html) · [Doc comments](https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_documentation_comments.html)

### Inferences
- Misturar docs e scripts na mesma lista é simples mas é exactamente o que a #4081/#15228 contestam:
  simplicidade de modelo que vira atrito com muitos ficheiros.
- O «extraordinário por baixo» do Godot é o **analisador** partilhado (editor, `--check-only`, LSP, doc
  tool usam o mesmo parser), não o widget.

### Gaps
- Número total de settings de Text Editor não contado (mediria-se por `EditorSettings` via script no
  oráculo — §7).

## 6. Editores integrados comparáveis (contraste)

### Takeaway
Roblox = o mais completo integrado (tipagem gradual por ficheiro, análise, IA inline); GameMaker =
linter com quick-fix (Feather); PICO-8 = restrição como design (contador de tokens); Defold = editor
básico que a comunidade troca por LSP externo; Unity = não tem (IDE externa).

### Cited Findings
- **Roblox Studio**: autocomplete ligado ao data model; doc-comment em bloco antes da declaração vira
  tooltip; Go to Declaration (Ctrl-click); Script Function Filter (`Alt+F`); find/replace com regex e
  Find All entre scripts; **Script Analysis** (erros/warnings estáticos); **Code Assist** IA (sugere
  linhas/funções ao pausar, `Alt+\`, aceita com `Tab`); multi-cursor (Alt-click, Ctrl+Alt+↑/↓, Ctrl+D);
  on-type formatting que completa blocos ao Enter — [Roblox Script Editor](https://create.roblox.com/docs/studio/script-editor)
- **Luau**: modos `--!nocheck` / `--!nonstrict` / `--!strict` na 1.ª linha do script; default do lugar
  por `Workspace.LuauTypeCheckMode`; erros de tipo como linhas realçadas + janela Script Analysis — [Luau type checking](https://create.roblox.com/docs/luau/type-checking)
- **GameMaker Feather** (desde 2022.1): ícone de pena no gutter + **Quick Fix** com `Ctrl+Q`; modo
  **Strict × Relaxed** de tipos (2022.8); JSDoc usado nas dicas de função e gerado por quick-fix;
  renomear um asset pode actualizar as referências no código — [GameMaker 2022.1](https://gamemaker.io/en/blog/release-2022-1) · [2022.8](https://gamemaker.io/en/blog/release-2022-8) · [Feather Settings (LTS)](https://manual.gamemaker.io/lts/en/Setting_Up_And_Version_Information/IDE_Preferences/Feather_Settings.htm)
- **PICO-8**: limites simultâneos 8192 tokens, 65535 caracteres, 15360 comprimidos; contador de tokens
  no canto inferior direito do editor (fonte: fórum, não o manual) — [lexaloffle BBS](https://www.lexaloffle.com/bbs/?pid=67844)
- **Defold**: há language server embutido configurável por anotações; completion falha às vezes com
  erros no ficheiro; comunidade usa Neovim/IntelliJ, mas LSPs externos não vêem as dependências em
  `.zip` — [Defold forum](https://forum.defold.com/t/lua-code-completion-in-editor-for-defold-built-in-types/82831) · [Defold forum: drop the code editor](https://forum.defold.com/t/defold-editor-should-drop-the-code-editor/72368) · [IntelliJ plugin](https://forum.defold.com/t/new-defold-plugin-for-intellij/82044)

### Inferences
- Luau é o paralelo mais directo do stack PH2D (Lua via mlua): tipagem **gradual por ficheiro** com
  pragma na 1.ª linha é a versão «simples» do sistema de warnings do Godot.
- O quick-fix de Feather é precisamente o que a #15415 pede ao Godot.

### Gaps
- Unity: não pesquisado (premissa do pedido: sem editor). Roblox Assistant (chat) além de Code Assist
  não detalhado. Feather actual (2025–26) não verificado — doc lida é de 2022/LTS.

## 7. O que medir correndo o Godot como ORÁCULO (headless)

### Takeaway
Godot 4.7.2 (MIT) **já está instalado** (`/usr/bin/godot`); `--check-only` dá diagnósticos e exit code
determinísticos; `--doctool --gdscript-docs` dá a doc gerada de `##` em XML; o LSP (6005) responde a
completion/hover/definition se houver um editor headless aberto no projecto.

### Cited Findings
- Flags (doc + `godot --help` do 4.7.2 local): `--headless`, `--path <dir>`, `-s/--script`,
  `--check-only` (só parse, usar com `--script`), `--doctool [path]`, `--no-docbase`,
  `--gdscript-docs <path>` (com `--doctool`), `--lsp-port`, `--dap-port`, `--debug-server <uri>`,
  `-e/--editor`, `--quit`, `--quit-after`, `--import`, `--dump-extension-api(-with-docs)`; argumentos
  desconhecidos são **ignorados em silêncio** (cuidado com typo num script de oráculo) — [Command line](https://docs.godotengine.org/en/stable/tutorials/editor/command_line_tutorial.html)
- **[MEDIDO 15/10, Godot 4.7.2]** projecto mínimo em scratchpad, `godot --headless --path P --check-only -s f.gd`:
  - erro de tipo → `SCRIPT ERROR: Parse Error: Cannot assign a value of type "String" as "int".` +
    `at: GDScript::reload (res://bad.gd:5)`, **exit 1**;
  - script limpo → **exit 0**, sem saída além do banner;
  - warning em nível 1 (warn) → **nada impresso, exit 0** (o `--check-only` não emite warnings);
  - mesmo warning em nível 2 → `Parse Error: … (Warning treated as error.)` com linha + `Failed to load
    script … "Parse error"` (o exit code desta corrida não foi capturado — presumivelmente 1 como o erro de tipo).
  ⇒ para capturar warnings como fixture, promovê-los a `2` no `project.godot` da fixture.
- LSP só funciona com uma instância do Godot com o projecto aberto — [External editor](https://docs.godotengine.org/en/stable/tutorials/editor/external_editor.html)

### Comandos concretos (oráculo; entradas NOSSAS, saída gravada como fixture com cabeçalho)
- Diagnósticos: `godot --headless --path <fix> --check-only -s res://x.gd; echo $?` (por warning: um
  `project.godot` por fixture com `debug/gdscript/warnings/<nome>=2`).
- Doc de `##`: `godot --headless --path <fix> --doctool <out> --gdscript-docs res://` → XML por classe
  (mede: o que vira descrição, `[param]`, `@deprecated`, privados `_` ocultos).
- LSP: `godot --headless -e --path <fix> --lsp-port 6005` e um cliente JSON-RPC por script (Python
  `socket`) enviando `initialize`, `textDocument/didOpen`, `completion`, `hover`, `definition`,
  `documentSymbol`, `signatureHelp` em posições escolhidas → fixtures de completion/hover por posição
  (mede ordenação: hierarquia, enums primeiro, etc.). DAP idem em 6006.
- Contagem/lista de settings do editor de texto: `EditorScript`/`-s` com `--editor` iterando
  `EditorInterface.get_editor_settings().get_property_list()` filtrando `text_editor/` (inferência:
  exige contexto de editor; não testado).
- Comportamentos de UI (Ctrl-drag `@onready`, color picker, auto-brace, auto-indent): o próprio `CodeEdit`
  é um nó — um script `-s` pode instanciá-lo headless, injectar `InputEventKey` e ler `text` →
  fixture de auto-indent/auto-brace/comentário (inferência; não testado).

### Inferences
- O oráculo mais rico e barato para o PH2D é o **LSP headless**: dá completion/hover/definition por
  posição sem tocar no código-fonte do Godot, compatível com a regra «correr, não ler».

### Gaps
- Não testei o LSP nem `--doctool` nesta sessão (só `--check-only`). Não confirmei se `--check-only`
  sem `--path` carrega o `project.godot` da cwd. Sublinhado preciso (4.8) só é medível com o 4.8, não
  instalado.
