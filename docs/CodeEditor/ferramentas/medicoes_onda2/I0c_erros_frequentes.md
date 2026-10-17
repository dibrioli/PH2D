# I0c — Que mensagens de erro o artista mais vê? (corpus por mutação, 16/10)

> Item I0(c) de [`03_onda2.md`](../../03_onda2.md); alimenta o **I8** (explicações escritas à mão por
> CLASSE de mensagem; a mensagem crua fica no hover e no console). ⛔ Este doc só MEDE — não há aqui
> nenhuma explicação. Medido em 2026-10-16 com **luau-lsp 1.70.1** (o binário fixado,
> `~/.cache/ph2d/luau-lsp/1.70.1/`) e o motor **`ph2d-script`** (mlua 0.12.1 / Luau 736, a VM da casa
> com sandbox). Ficheiros: [`../corpus_erros/`](../corpus_erros/) — gerador, sonda, saídas cruas e as
> tabelas completas ([`tabelas.md`](../corpus_erros/tabelas.md),
> [`classes_estatico.tsv`](../corpus_erros/classes_estatico.tsv),
> [`classes_runtime.tsv`](../corpus_erros/classes_runtime.tsv)).

## Como re-correr (bash, ~10 s mais a compilação da sonda)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-CodeEditor
bash docs/CodeEditor/ferramentas/corpus_erros/corre.sh
```

O `corre.sh` faz, por ordem: (1) `corpus_erros.py gera` — escreve os mutantes em
`~/.cache/ph2d/corpus_erros/mutantes/` (⚠️ **fora do repo**: o `.luaurc` da raiz tem
`languageMode: strict` e punha o luau-lsp em modo estrito — medido: o `bob.luau` original ganhava 4
`TypeError 1030` que o modo do app não dá) e o índice `mutantes.tsv`; (2) `corpus_erros.py estatico` —
o luau-lsp em **modo LSP** (`lsp --definitions:@ph2d=runtime/luau/ph2d.d.luau`, as settings de
`ph2d_settings()`: `platform.type=standard`, `fflags.sync=false`), um `didOpen` + pull
`textDocument/diagnostic` por mutante, com o **mesmo filtro do cliente** (`is_unused_engine_hook`:
some o `FunctionUnused` de um `function init|update|on_signal` global) → `estatico.jsonl`; (3) a sonda
`#[ignore]` [`corpus_de_erros`](../../../../crates/ph2d-script/tests/it/corpus_de_erros.rs) — cada
mutante entra na cena como script da biblioteca (`ProjectScripts` + `LuauScript::library`), `scene_sync`
(corre o topo), 5 × `scene_tick` (o 1.º corre o `init`), um `scene_hear(["ping"])` → `runtime.jsonl`
com fase, tipo, linha e mensagem de cada `ScriptDiagnostic`; (4) `corpus_erros.py tabelas`.

**Modo:** NÃO-estrito (o default do app). Os 6 ficheiros do corpus com `--!strict` têm essa linha
trocada por um comentário (os números de linha ficam). **Base:** cada ficheiro original passa pelas duas
vias; o que ele já dizia (mesma linha e mensagem) é subtraído de cada mutante dele — só sobram 2
`FunctionUnused` estáticos (`task_body`, `tick_alloc`); o motor não diz nada sobre nenhum original.

## O corpus

| ficheiro | linhas | linhas de código | mutantes | entra? |
|---|---:|---:|---:|---|
| `crates/ph2d-code-syntax/tests/it/fixtures/bob.luau` | 53 | 34 | 273 | sim — o único script de CENA completo (`property`, `init`, `update`) |
| `tests/spike/c7_typical_script.luau` | 118 | 99 | 528 | sim — ⚠️ módulo (`return {…}`), sem ganchos: o motor só corre o topo |
| `runtime/luau/examples/ph2d_script_attach.luau` | 58 | 26 | 125 | sim — ⚠️ idem (`local function init/tick` + `return`) |
| `tests/spike/gc_stress.luau` | 27 | 18 | 100 | sim — o topo cria tabelas e corrotinas |
| `tests/spike/hello.luau` | 14 | 8 | 33 | sim — `update(frame)` global |
| `docs/scripting/examples/spike/lsp/case_2_coroutine.luau` | 12 | 6 | 20 | sim — ⚠️ função nunca chamada |
| `tests/spike/coro_timing.luau` | 9 | 3 | 12 | sim — ⚠️ função nunca chamada |
| `crates/ph2d-app-components/src/script_boot.luau` | 2 | 1 | 4 | sim |
| **total** | **293** | **195** | **1 095** | |

Fora (usam API que **não existe** em `runtime/luau/ph2d.d.luau` — medido pelo luau-lsp sobre o
original, não por leitura): `spike/debug-autonomy/bug1_race_condition` (`ph2d.system`, `ph2d.query`),
`bug2_determinism` (`ph2d.state_table`, tipo `ph2d.Entity`), `bug3_coroutine_leak` (já tem 3
`SyntaxError`), `llm-tests/01_fsm`, `02_query_ecs` (`system`/`query`, tipo `ph2d.Vec2`), `03_coro_anim`
(tipo `ph2d.Entity`, `ph2d.set(…, "Color", …)`), `04_message` (`message_handler`/`message_send`),
`05_dialogue` (`state_table`, `dialogue`), `lsp/case_1_query_ecs`, `case_3_system`, `case_4_message`,
`case_5_fsm`; e `docs/CodeEditor/ferramentas/smoke/script_quebrado.luau` (quebrado DE PROPÓSITO na
linha 3: o erro dele mascara qualquer mutação). Os `.d.luau` não são scripts.

## Os operadores (cada um aplicado em CADA sítio aplicável; um erro por mutante)

Fontes: **A&B 2015** = Altadmri & Brown, *37 Million Compilations: Investigating Novice Programming
Mistakes in Large-Scale Student Data* (SIGCSE 2015, dados Blackbox; as categorias A–R de erros de
novato); **Becker 2016** = Becker, *An Effective Approach to Enhancing Compiler Error Messages*
(SIGCSE 2016; as mensagens mais frequentes do javac entre novatos, à cabeça *cannot find symbol*);
**K&M 2005** = Ko & Myers, *A framework and methodology for studying the causes of software errors in
programming systems* (JVLC 2005; deslizes, lapsos e omissões); **Lua** = a diferença do Lua/Luau para
a família C/Python/JS que o novato traz (manual do Lua 5.1 / *Programming in Lua*).

| operador | o que faz | fonte | sítios |
|---|---|---|---:|
| `identificador_trocado` | troca 2 letras de um nome de variável/função (`ph2d`→`p2hd`, `self`→`slef`) | Becker 2016; A&B 2015 (*cannot find symbol*) | 277 |
| `palavra_chave_trocada` | troca 2 letras de uma palavra-chave (`function`→`funtcion`, `then`→`tehn`) | K&M 2005 (deslize de digitação) | 170 |
| `membro_trocado` | troca 2 letras de um membro/API (`ph2d.get`→`ph2d.egt`, `self.t`) | Becker 2016 (*cannot find symbol*, membro) | 150 |
| `sem_fecha_parenteses` | apaga um `)` | A&B 2015 cat. C (parênteses desequilibrados) | 77 |
| `sem_fecha_aspas` | apaga a aspa de fecho de uma cadeia | A&B 2015 cat. C (aspas) | 55 |
| `argumento_a_menos` | apaga o último argumento de uma chamada | A&B 2015 cat. I; Becker 2016 | 54 |
| `atribuicao_por_igual` | `=` de atribuição → `==` | A&B 2015 cat. A (sentido inverso) | 47 |
| `ponto_por_dois_pontos` | `a.b(` → `a:b(` (chamada ou definição) | Lua (`.` vs `:`) | 45 |
| `sem_end` | apaga um `end` | A&B 2015 cat. C (bloco desequilibrado) | 41 |
| `argumentos_trocados` | troca os 2 primeiros argumentos | A&B 2015 cat. I (tipos errados) | 26 |
| `sem_fecha_chave` | apaga um `}` ou `]` | A&B 2015 cat. C | 25 |
| `sem_then` | apaga um `then` | Lua; K&M 2005 (omissão) | 24 |
| `inicializacao_esquecida` | apaga uma linha `a.b = expr` inteira (fica vazia) | K&M 2005 (passo omitido) | 22 |
| `virgula_em_falta` | apaga uma vírgula entre campos de um construtor `{…}` | Lua; A&B 2015 cat. C/F (separadores) | 19 |
| `igual_por_atribuicao` | `==` → `=` | A&B 2015 cat. A | 16 |
| `dois_pontos_por_ponto` | `a:b(` → `a.b(` (chamada ou definição) | Lua; A&B 2015 cat. M (método como estático) | 15 |
| `logico_estilo_c` | `and`/`or`/`not` → `&&`/`\|\|`/`!` | A&B 2015 cat. D; hábito C | 8 |
| `else_if` | `elseif` → `else if` | Lua; Becker 2016 (*'else' without 'if'*) | 8 |
| `sem_self_no_parametro` | tira o `self` de uma lista de parâmetros | Lua (o `self` implícito, família de `:`/`.`) | 4 |
| `sem_do` | apaga um `do` | Lua; K&M 2005 (omissão) | 4 |
| `indice_zero` | `[1]` → `[0]`, `for i = 1,` → `for i = 0,` | Lua (índices a partir de 1) | 4 |
| `diferente_estilo_c` | `~=` → `!=` | Lua (hábito C/Python) | 1 |
| `concat_com_mais` | `..` → `+` | Lua (hábito JS/Python) | 1 |
| `comparacao_invertida` | `>=`/`<=` → `=>`/`=<` | A&B 2015 cat. L | 1 |
| `mais_mais` | `x = x + 1` / `x += 1` → `x++` | hábito C | 1 |
| `chamada_sem_parenteses` | `f()` → `f` | A&B 2015 cat. J | **0** (o corpus não tem chamada sem argumentos) |

A tabela operador × ficheiro está em [`tabelas.md`](../corpus_erros/tabelas.md) §Sítios.

## Como uma mensagem vira CLASSE

`corpus_erros.py::classe`: espaços colapsados; `got '…'`/`got "…"`/`got malformed string` → `got <tok>`
(`got <eof>` fica: é outro caso); `at line N`/`at column N` → `at <pos>`; `to close '<kw>'` →
`to close <kw>`; um `'…'` com palavra-chave ou pontuação fica (`'end'`, `')'`, `'then'`), qualquer
outro (nome, tipo) → `<x>`; números → `<n>`; as variantes de gramática do *Argument count mismatch*
fundem-se em DUAS (falta: `only <n>`/`none`; sobra); a aritmética perde a operação e a ordem
(`(add) on nil and number` = `(mul) on number and nil`) e guarda os TIPOS; o alvo numérico do mlua
(`u64`, `f64`) → `<número>`. O **código** é o `code` do LSP: lints = o número do `LintWarning`
(`5` SameLineStatement, `4` LocalShadow, `7` LocalUnused, `8` FunctionUnused…), erros de tipo =
`1000 + variante` (`1000` TypeMismatch, `1001` UnknownSymbol, `1002` UnknownProperty, `1008`
CountMismatch, `1014` **SyntaxError**). No motor, o código é o `DiagnosticKind`.

**Colunas:** *sítios* = mutantes que produzem ≥ 1 mensagem da classe (uma vez por mutante); *%* e
*% acum.* sobre a soma dos pares classe × sítio; *1.ª* = em quantos mutantes ela é a PRIMEIRA mensagem
(menor linha — a que o cartão mostra); *% ⚖️* = a mesma partilha com **cada operador a pesar igual**
(o controlo do viés por sítio, abaixo); *grav.* = `severity` do LSP.

## Via estática (luau-lsp, enquanto se escreve) — 1 095 mutantes, 54 classes, 1 661 pares; 180 calados

As 27 primeiras (96,4 % acum.); as 54 em [`classes_estatico.tsv`](../corpus_erros/classes_estatico.tsv).

| # | classe | código | grav. | sítios | % | % acum. | 1.ª | % ⚖️ | exemplo cru | operadores (top 3) |
|---:|---|---|---|---:|---:|---:|---:|---:|---|---|
| 1 | `TypeError: Unknown global <x>; consider assigning to it first` | 1001 | erro | 317 | 19.1 | 19.1 | 234 | 7.6 | `TypeError: Unknown global 'p2hd'; consider assigning to it first` | identificador_trocado 265, palavra_chave_trocada 35, dois_pontos_por_ponto 6 |
| 2 | `SameLineStatement: A new statement is on the same line; add semi-colon on previous statement to silence` | 5 | aviso | 232 | 14.0 | 33.1 | 0 | 16.2 | `SameLineStatement: A new statement is on the same line; …` | palavra_chave_trocada 103, atribuicao_por_igual 47, sem_fecha_parenteses 27 |
| 3 | `SyntaxError: Incomplete statement: expected assignment or a function call` | 1014 | erro | 151 | 9.1 | 42.1 | 120 | 6.7 | `SyntaxError: Incomplete statement: expected assignment or a function call` | palavra_chave_trocada 115, atribuicao_por_igual 33, sem_fecha_aspas 2 |
| 4 | `SyntaxError: Expected identifier when parsing expression, got <tok>` | 1014 | erro | 112 | 6.7 | 48.9 | 23 | 12.4 | `SyntaxError: Expected identifier when parsing expression, got '=='` | atribuicao_por_igual 47, palavra_chave_trocada 39, igual_por_atribuicao 15 |
| 5 | `SyntaxError: Expected ')' (to close '(' at <pos>), got <tok>` | 1014 | erro | 108 | 6.5 | 55.4 | 76 | 4.3 | `SyntaxError: Expected ')' (to close '(' at line 9), got 'ph2d'` | sem_fecha_parenteses 74, sem_fecha_aspas 31, palavra_chave_trocada 2 |
| 6 | `SyntaxError: Expected 'then' when parsing if statement, got <tok>` | 1014 | erro | 81 | 4.9 | 60.3 | 69 | 6.4 | `SyntaxError: Expected 'then' when parsing if statement, got '='` | palavra_chave_trocada 30, sem_then 24, igual_por_atribuicao 15 |
| 7 | `SyntaxError: Expected 'end' (to close <kw> at <pos>), got <eof>; did you forget to close <kw> at <pos>?` | 1014 | erro | 67 | 4.0 | 64.3 | 29 | 4.9 | `SyntaxError: Expected 'end' (to close 'function' at line 31), got <eof>; did you forget to…` | sem_end 30, palavra_chave_trocada 29, else_if 7 |
| 8 | `LocalShadow: Variable <x> shadows a global variable used at <pos>` | 4 | aviso | 59 | 3.6 | 67.9 | 37 | 4.8 | `LocalShadow: Variable 'self' shadows a global variable used at line 25` | palavra_chave_trocada 24, identificador_trocado 22, dois_pontos_por_ponto 6 |
| 9 | `TypeError: Expected this to be <x>, but got <tok>` | 1000 | erro | 57 | 3.4 | 71.3 | 55 | 6.8 | `TypeError: Expected this to be 'string', but got 'number'` | ponto_por_dois_pontos 39, argumentos_trocados 15, sem_fecha_parenteses 2 |
| 10 | `TypeError: Key <x> not found in table <x>` | 1002 | erro | 51 | 3.1 | 74.4 | 42 | 1.0 | `TypeError: Key 'proeprty' not found in table '{ attach_script: (number, string) -> (), col…` | membro_trocado 42, palavra_chave_trocada 7, identificador_trocado 1 |
| 11 | `SyntaxError: Malformed string; did you forget to finish it?` | 1014 | erro | 49 | 3.0 | 77.3 | 49 | 2.5 | `SyntaxError: Malformed string; did you forget to finish it?` | sem_fecha_aspas 49 |
| 12 | `TypeError: Argument count mismatch. Function <x> expects <n> arguments, but only <n> are specified` | 1008 | erro | 42 | 2.5 | 79.8 | 32 | 2.2 | `TypeError: Argument count mismatch. Function 'ph2d.property' expects 2 to 3 arguments, but…` | argumento_a_menos 32, sem_fecha_aspas 9, sem_fecha_chave 1 |
| 13 | `TypeError: Argument count mismatch. Function <x> expects <n> arguments, but <n> are specified` | 1008 | erro | 32 | 1.9 | 81.8 | 2 | 1.9 | `TypeError: Argument count mismatch. Function 'ph2d.property' expects 2 to 3 arguments, but…` | ponto_por_dois_pontos 31, sem_fecha_parenteses 1 |
| 14 | `LocalUnused: Variable <x> is never used; prefix with <x> to silence` | 7 | aviso | 31 | 1.9 | 83.6 | 23 | 0.3 | `LocalUnused: Variable 'ss' is never used; prefix with '_' to silence` | identificador_trocado 25, palavra_chave_trocada 6 |
| 15 | `SyntaxError: Expected <eof>, got <tok>` | 1014 | erro | 31 | 1.9 | 85.5 | 0 | 0.7 | `SyntaxError: Expected <eof>, got 'end'` | palavra_chave_trocada 30, igual_por_atribuicao 1 |
| 16 | `SyntaxError: Expected 'end' (to close <kw> at <pos>), got <tok>` | 1014 | erro | 24 | 1.4 | 86.9 | 18 | 1.2 | `SyntaxError: Expected 'end' (to close 'then' at line 32), got 'self'` | palavra_chave_trocada 15, sem_end 7, else_if 1 |
| 17 | `SyntaxError: Expected '}' (to close '{' at <pos>), got <tok>` | 1014 | erro | 24 | 1.4 | 88.4 | 13 | 2.5 | `SyntaxError: Expected '}' (to close '{' at column 43), got malformed string` | sem_fecha_chave 15, sem_fecha_aspas 5, virgula_em_falta 4 |
| 18 | `FunctionUnused: Function <x> is never used; prefix with <x> to silence` | 8 | aviso | 22 | 1.3 | 89.7 | 21 | 0.3 | `FunctionUnused: Function 'iint' is never used; prefix with '_' to silence` | identificador_trocado 15, palavra_chave_trocada 6, argumento_a_menos 1 |
| 19 | `GlobalUsedAsLocal: Global <x> is only used in the enclosing function <x>; consider changing it to local` | 3 | aviso | 20 | 1.2 | 90.9 | 4 | 0.4 | `GlobalUsedAsLocal: Global 's' is only used in the enclosing function 'update'; …` | palavra_chave_trocada 15, identificador_trocado 4, sem_end 1 |
| 20 | `SyntaxError: Expected ',' after table constructor element` | 1014 | erro | 20 | 1.2 | 92.1 | 19 | 2.6 | `SyntaxError: Expected ',' after table constructor element` | virgula_em_falta 15, sem_fecha_aspas 3, sem_fecha_chave 2 |
| 21 | `UninitializedLocal: Variable <x> defined at <pos> is never initialized or assigned; initialize with 'nil' to silence` | 20 | aviso | 18 | 1.1 | 93.2 | 0 | 1.1 | `UninitializedLocal: Variable 'brilho' defined at line 44 is never initialized or assigned;…` | atribuicao_por_igual 18 |
| 22 | `SyntaxError: Expected '(', '{' or <string> when parsing function call, got <tok>` | 1014 | erro | 16 | 1.0 | 94.2 | 1 | 0.3 | `SyntaxError: Expected '(', '{' or <string> when parsing function call, got ','` | palavra_chave_trocada 16 |
| 23 | `SyntaxError: Expected 'end' (to close <kw> at <pos>), got <tok>; did you forget to close <kw> at <pos>?` | 1014 | erro | 9 | 0.5 | 94.7 | 5 | 0.2 | `SyntaxError: Expected 'end' (to close 'function' at line 21), got 'elseif'; did you forget…` | palavra_chave_trocada 7, sem_end 2 |
| 24 | `ImplicitReturn: Function <x> can implicitly return no values even though there's an explicit return at <pos>; add explicit return to silence` | 16 | aviso | 8 | 0.5 | 95.2 | 1 | 0.2 | `ImplicitReturn: Function 'update' can implicitly return no values even though there's an e…` | palavra_chave_trocada 6, sem_end 2 |
| 25 | `SyntaxError: Expected 'do' when parsing for loop, got <tok>` | 1014 | erro | 8 | 0.5 | 95.7 | 8 | 2.8 | `SyntaxError: Expected 'do' when parsing for loop, got 'od'` | palavra_chave_trocada 4, sem_do 4 |
| 26 | `UnreachableCode: Unreachable code (previous statement always returns)` | 12 | aviso | 7 | 0.4 | 96.1 | 3 | 0.5 | `UnreachableCode: Unreachable code (previous statement always returns)` | palavra_chave_trocada 4, sem_end 2, else_if 1 |
| 27 | `SyntaxError: Expected ']' (to close '[' at <pos>), got <tok>` | 1014 | erro | 6 | 0.4 | 96.4 | 5 | 0.7 | `SyntaxError: Expected ']' (to close '[' at column 15), got '='` | sem_fecha_chave 6 |

As 27 restantes (3,6 %) incluem as três frases com sugestão do próprio Luau — `Unexpected '&&'; did you
mean 'and'?` (4), `'||'` → `'or'` (2), `'!'` → `'not'` (2), `'!='` → `'~='` (1) — que pesam pouco por
sítio e muito por operador (% ⚖️ 1,4 / 0,7 / 0,7 / 2,8), e `TypeError 1010 This function must be
called with self. Did you mean to use a colon instead of a dot?` (1).

**Calados na via estática (180 de 1 095 = 16 %):** `membro_trocado` 108 (o `self` é `any` em
não-estrito: `self.abse_x` não diz nada), `inicializacao_esquecida` 22 (todos), `argumento_a_menos` 21,
`argumentos_trocados` 11, `dois_pontos_por_ponto` 9, `indice_zero` 4 (todos), `identificador_trocado` 3,
`ponto_por_dois_pontos` 2.

## Via do motor (`ph2d-script`, ao gravar/correr) — 1 095 mutantes, 47 classes, 669 pares; 426 calados

Cada mutante dá **no máximo uma** mensagem (o compilador pára no 1.º erro; um objecto que falha pára
de correr), por isso *sítios* = *1.ª*. Fases: 490 no compilador (`topo`, `Syntax`), 80 a correr o topo
(`Runtime`), 7 recusas de declaração (`Decl`), 92 nos tiques (`init`/`update`); nenhuma no `on_signal`
(nenhum script do corpus o define). As 24 primeiras (94,6 %); as 47 em
[`classes_runtime.tsv`](../corpus_erros/classes_runtime.tsv).

| # | classe | tipo | sítios | % | % acum. | % ⚖️ | exemplo cru | operadores (top 3) |
|---:|---|---|---:|---:|---:|---:|---|---|
| 1 | `Incomplete statement: expected assignment or a function call` | Syntax | 140 | 20.9 | 20.9 | 12.4 | `Incomplete statement: expected assignment or a function call` | palavra_chave_trocada 110, atribuicao_por_igual 29, mais_mais 1 |
| 2 | `Expected ')' (to close '(' at <pos>), got <tok>` | Syntax | 79 | 11.8 | 32.7 | 5.7 | `Expected ')' (to close '(' at column 14), got malformed string` | sem_fecha_parenteses 74, sem_fecha_aspas 4, palavra_chave_trocada 1 |
| 3 | `Expected 'then' when parsing if statement, got <tok>` | Syntax | 69 | 10.3 | 43.0 | 11.6 | `Expected 'then' when parsing if statement, got '='` | palavra_chave_trocada 30, sem_then 24, igual_por_atribuicao 15 |
| 4 | `attempt to index nil with <x>` | Runtime | 65 | 9.7 | 52.8 | 4.1 | `attempt to index nil with 'property'` | identificador_trocado 58, membro_trocado 5, sem_self_no_parametro 2 |
| 5 | `Malformed string; did you forget to finish it?` | Syntax | 49 | 7.3 | 60.1 | 4.9 | `Malformed string; did you forget to finish it?` | sem_fecha_aspas 49 |
| 6 | `Expected 'end' (to close <kw> at <pos>), got <eof>; did you forget to close <kw> at <pos>?` | Syntax | 36 | 5.4 | 65.5 | 8.7 | `Expected 'end' (to close 'function' at line 31), got <eof>; did you forget to close 'then'…` | sem_end 29, else_if 7 |
| 7 | `attempt to perform arithmetic (<op>) on nil and number` | Runtime | 33 | 4.9 | 70.4 | 1.6 | `attempt to perform arithmetic (add) on nil and number` | identificador_trocado 16, membro_trocado 14, inicializacao_esquecida 3 |
| 8 | `Expected identifier when parsing expression, got <tok>` | Syntax | 23 | 3.4 | 73.8 | 8.4 | `Expected identifier when parsing expression, got '=='` | atribuicao_por_igual 18, sem_fecha_chave 3, comparacao_invertida 1 |
| 9 | `attempt to call a nil value` | Runtime | 22 | 3.3 | 77.1 | 0.8 | `attempt to call a nil value` | membro_trocado 18, palavra_chave_trocada 2, identificador_trocado 2 |
| 10 | `Expected 'end' (to close <kw> at <pos>), got <tok>` | Syntax | 19 | 2.8 | 80.0 | 2.3 | `Expected 'end' (to close 'then' at line 32), got 'self'` | palavra_chave_trocada 10, sem_end 7, else_if 1 |
| 11 | `Expected ',' after table constructor element` | Syntax | 19 | 2.8 | 82.8 | 5.0 | `Expected ',' after table constructor element` | virgula_em_falta 15, sem_fecha_aspas 2, sem_fecha_chave 2 |
| 12 | `Expected '}' (to close '{' at <pos>), got <tok>` | Syntax | 14 | 2.1 | 84.9 | 3.3 | `Expected '}' (to close '{' at column 33), got ')'` | sem_fecha_chave 10, virgula_em_falta 4 |
| 13 | `bad argument #<n>: error converting Lua table to String (expected string or number)` | Runtime | 9 | 1.3 | 86.2 | 1.3 | `bad argument #1: error converting Lua table to String (expected string or number)` | ponto_por_dois_pontos 7, argumentos_trocados 2 |
| 14 | `bad argument #<n>: error converting Lua nil to <número> (expected number or string coercible to number)` | Runtime | 8 | 1.2 | 87.4 | 0.6 | `bad argument #2: error converting Lua nil to f64 (expected number or string coercible to n…` | argumento_a_menos 4, membro_trocado 4 |
| 15 | `Expected 'do' when parsing for loop, got <tok>` | Syntax | 8 | 1.2 | 88.6 | 5.6 | `Expected 'do' when parsing for loop, got 'od'` | palavra_chave_trocada 4, sem_do 4 |
| 16 | `bad argument #<n>: error converting Lua table to <número> (expected number or string coercible to number)` | Runtime | 6 | 0.9 | 89.5 | 0.7 | `bad argument #1: error converting Lua table to f64 (expected number or string coercible to…` | ponto_por_dois_pontos 6 |
| 17 | `Expected ']' (to close '[' at <pos>), got <tok>` | Syntax | 6 | 0.9 | 90.4 | 1.3 | `Expected ']' (to close '[' at column 15), got '='` | sem_fecha_chave 6 |
| 18 | `Expected 'end' (to close <kw> at <pos>), got <tok>; did you forget to close <kw> at <pos>?` | Syntax | 5 | 0.7 | 91.2 | 0.4 | `Expected 'end' (to close 'function' at line 69), got 'ned'; did you forget to close 'then'…` | palavra_chave_trocada 3, sem_end 2 |
| 19 | `attempt to index nil with number` | Runtime | 5 | 0.7 | 91.9 | 0.1 | `attempt to index nil with number` | identificador_trocado 5 |
| 20 | `bad argument #<n>: error converting Lua string to <número> (expected number or string coercible to number)` | Runtime | 4 | 0.6 | 92.5 | 0.8 | `bad argument #1: error converting Lua string to u64 (expected number or string coercible t…` | argumentos_trocados 4 |
| 21 | `Unexpected '&&'; did you mean 'and'?` | Syntax | 4 | 0.6 | 93.1 | 2.7 | `Unexpected '&&'; did you mean 'and'?` | logico_estilo_c 4 |
| 22 | `Expected '(', '{' or <string> when parsing function call, got <tok>` | Syntax | 4 | 0.6 | 93.7 | 0.1 | `Expected '(', '{' or <string> when parsing function call, got ','` | palavra_chave_trocada 4 |
| 23 | `property <x> needs a number, boolean, string, ph2d.vec2 or ph2d.color default` | Decl:BadDefault | 3 | 0.4 | 94.2 | 0.3 | `property 'active' needs a number, boolean, string, ph2d.vec2 or ph2d.color default` | argumento_a_menos 3 |
| 24 | `missing argument #<n> to <x> (number expected)` | Runtime | 3 | 0.4 | 94.6 | 0.3 | `missing argument #1 to 'sin' (number expected)` | argumento_a_menos 3 |

As 23 restantes (5,4 %): `<x> is not a valid property name` (Decl:BadName, 3), `invalid argument #<n> to
<x> (number expected, got table)` (3), `table index is nil` (3), mais conversões do mlua (`nil`/`boolean`
→ `String`), `attempt to compare number < nil`, `attempt to concatenate string with nil`, `attempt to
perform arithmetic (<op>) on string and number` (1 — o único sítio de `..` do corpus), `missing/invalid
argument … (thread|function expected …)` das corrotinas, `property <x>: min/max/step …` (Decl:BadHint).

**Calados no motor (426 = 39 %):** `identificador_trocado` 190, `membro_trocado` 109,
`argumento_a_menos` 40, `ponto_por_dois_pontos` 27, `inicializacao_esquecida` 19,
`argumentos_trocados` 15, `dois_pontos_por_ponto` 15, `palavra_chave_trocada` 5, `indice_zero` 4,
`sem_self_no_parametro` 2. ⚠️ Grande parte é **código que o motor nunca corre**: `c7_typical_script`,
`ph2d_script_attach`, `case_2_coroutine` e `coro_timing` não definem ganchos globais (só o topo corre),
e no `bob` os ramos `square`/`still` e o `emit` (sinal vazio por omissão) não correm em 5 tiques. Por
ficheiro, mutantes com mensagem de tipo `Runtime`/`Decl`: `bob` 128 de 273, `gc_stress` 30/100,
`c7_typical_script` 15/528, `hello` 4/33, `ph2d_script_attach` 1/125, `script_boot` 1/4.

## O mesmo erro, as duas vias (1.ª mensagem de cada)

Tabela completa (top 3 pares por operador) em [`tabelas.md`](../corpus_erros/tabelas.md) §Mapa. O que
ela mostra:

| erro (operador) | estática (luau-lsp) | motor (`ph2d-script`) | n |
|---|---|---|---:|
| sintaxe (`sem_*`, `=`/`==`, `else_if`, `&&`, `!=`, `virgula_em_falta`, `palavra_chave_trocada`) | `1014 SyntaxError: <texto>` | `Syntax`: **o mesmo texto e a mesma linha** que o 1.º `1014` (o mesmo *parser* do Luau), sem o prefixo `SyntaxError: ` | 490 de 490 |
| sintaxe — quantas | mais de um `1014` em 206 dos 490 (o *parser* recupera e continua), e `SameLineStatement` (5) ao lado em 232 mutantes | **uma** (o compilador pára) | — |
| nome de variável trocado | `1001 Unknown global <x>` (diz o NOME) | `attempt to index nil with <x>` (diz o CAMPO seguinte, não o nome) — ou nada se a linha não correr | 54 / 150 calados |
| membro/API trocado (`ph2d.egt`) | `1002 Key <x> not found in table <x>` — com a tabela INTEIRA de `ph2d` (truncada em `*TRUNCATED*`) | `attempt to call a nil value` (**sem nome nenhum**) | 18 |
| `self.campo` trocado | nada (o `self` é `any`) | `attempt to perform arithmetic (<op>) on nil and number` ou `attempt to index nil with <x>` — se correr | — |
| `ph2d.get(` → `ph2d:get(` | `1000 Expected this to be <x>, but got <tok>` / `1008 … but <n> are specified` | `bad argument #<n>: error converting Lua table to String`/`<número>` (mlua, **sem o nome da função**) | 13 |
| argumento a menos | `1008 … but only <n> are specified` | `bad argument #<n>: error converting Lua nil to <número>`, `missing argument #<n> to <x>`, `Decl:BadDefault` — ou nada | 4 + 3 + 3 |
| argumentos trocados | `1000 Expected this to be <x>, but got <tok>` | `bad argument #<n>: error converting Lua string to <número>` | 4 |
| nome de gancho trocado (`function iint`) | `8 FunctionUnused` (o filtro do cliente só some o nome EXATO) | nada — o gancho simplesmente não corre | 14 |
| linha de inicialização esquecida | nada | `attempt to perform arithmetic (<op>) on nil and number` (3) ou nada (19) | 22 |
| `..` → `+` | `1000 Expected this to be <x>, but got <tok>` | `attempt to perform arithmetic (<op>) on string and number` | 1 |

## O viés, dito sem rodeios

1. **Os operadores fomos NÓS que escolhemos** (das fontes acima), e o peso é **uniforme por sítio**: um
   operador que tem muitos sítios (`identificador_trocado` 277, `palavra_chave_trocada` 170) pesa mais
   do que um que tem poucos (`concat_com_mais` 1, `diferente_estilo_c` 1, `mais_mais` 1), sem nenhuma
   prova de que os artistas os cometam nessas proporções. A coluna **% ⚖️** refaz a conta com cada
   operador a pesar igual: o topo da via estática muda (`SameLineStatement` 16,2, `Expected identifier
   when parsing expression` 12,4, `Unknown global` 7,6) e o da via do motor também (`Incomplete
   statement` 12,4, `Expected 'then'` 11,6, `Expected 'end' … got <eof>` 8,7). As duas ordens são
   palpites diferentes da MESMA ignorância; a resposta certa é o registo local do I8.
2. **O corpus é pequeno e quase não é de cena**: 195 linhas de código; só o `bob` tem `init`/`update`
   com `self`. O `c7_typical_script` (48 % dos mutantes) é um módulo de jogo genérico que o motor não
   corre além do topo — por isso a via do motor ali é quase só sintaxe.
3. **Um erro por mutante**: erros em cascata, que o novato faz (A&B), não estão medidos.
4. **5 tiques e 1 sinal**: um erro que só aparece depois (um ramo raro, um `nil` ao fim de 10 s) é
   contado como calado.
5. **Não-estrito**: com `--!strict` a via estática dá outras classes (p.ex. `1030 Unknown type used in
   + operation` no `bob` original); medir o estrito é outra corrida.
6. **O texto da tipagem traz a tabela inteira** (`Key 'egt' not found in table '{ attach_script: … *TRUNCATED*'`)
   — a classe é a mesma, mas o exemplo cru é ilegível; isto é um achado do formato, não do corpus.

## Achado lateral (fora do âmbito deste item)

O doc-comment de `ph2d_code_lsp::Diagnostic::code` diz que os erros de sintaxe vêm com
`code = "SyntaxError"`; **medido aqui, o luau-lsp 1.70.1 em modo LSP manda `"1014"`** (a variante
`SyntaxError` dos erros de tipo, `1000 + 14`) — `"SyntaxError"` aparece em 0 das 1 103 respostas. Quem usar o código
para escolher a explicação (I8) tem de casar `1014`.
