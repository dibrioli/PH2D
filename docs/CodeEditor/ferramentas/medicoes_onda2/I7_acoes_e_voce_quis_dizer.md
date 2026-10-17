# I7 — as acções de contexto e o «você quis dizer» (17/10)

**Reproduzir** (da raiz da worktree, ~15 s): `python3 -I docs/CodeEditor/ferramentas/lspprobe_i7.py`
→ regrava `I7_bruto.json`. O `luau-lsp` 1.70.1 corre COMO O APP (definições e `--docs` gerados em
`~/.cache/ph2d/luau-lsp/1.70.1/api/`, dicas de tipo ligadas). Entrada: o `bob.luau` com três erros nossos
(`pirnt(1)` depois de `self.t = self.t + dt`, `ph2d.proprety("speed", …)`, `self.ampiltude` na linha do
`local d`), sem e com `--!strict`. Oráculos corridos por script, nenhum fonte lido: Python 3.14.8
(PSF-2.0, `pacman -Qi`), `rustc` 1.98.0 (MIT/Apache-2.0), Godot 4.7.2 (MIT, `pacman -Qi`). `loadavg`
`112.93 31.58 13.50` → `123.51 35.12 14.74` (nenhum número de tempo aqui decide nada).

## 1. O que o servidor dá a `codeAction`

[MEDIDO] Igual nos dois modos, no range do diagnóstico e no CURSOR (início, meio e fim de cada palavra,
com os diagnósticos que tocam no cursor no `context`, como o Alt+Enter vai pedir):

| Onde | Diagnóstico | Acções do servidor |
|---|---|---|
| `pirnt(1)` | `1001` «Unknown global 'pirnt'» | Sort requires · Extract to function · Sort services (+ Extract to local variable no fim) — **nenhuma corrige** |
| `ph2d.proprety` | `1002` «Key 'proprety' not found in table '{ … *TRUNCATED*» | Sort requires · Extract to local variable · Extract to function · Sort services |
| `self.ampiltude` | **nenhum**, nem em estrito | as mesmas genéricas |
| `--!strict`: `self.t + dt`, `self.tint.r + …`, os dois `ph2d.set` | `1030` «Unknown type used in + operation; consider adding a type annotation to 'self'» | as genéricas + «Inline variable 'self'» — **nenhuma acrescenta o tipo** |
| `function init` / `update` | `8` FunctionUnused | **a única `quickfix`**: «Prefix '<nome>' with '_' to silence» (1 edição) |

- Todas as genéricas vêm SEM edição; o `codeAction/resolve` dá-a: «Extract to function» reescreve a
  linha num `local function extracted()` e pede o comando `luau-lsp.rename`; «Extract to local
  variable» idem. São refactors de SELECÇÃO (I10 é o rename), não correcções. «Sort requires/services»
  é do Roblox (não temos `require` nem serviços).
- ⇒ **O que o I7 mostra do servidor: só `kind: quickfix` com edição.** Hoje isso é o «Prefix '_'» e,
  numa `local function` nunca usada, «Remove unused function: '<nome>'».

[MEDIDO] O cliente (fase `cliente` do bruto, com e sem as capacidades `codeAction`/`inlayHint` no
`initialize`): **as respostas são iguais** — sempre `CodeAction` literais, nunca `Command`. A quickfix
do `FunctionUnused` vem **com ou sem** o diagnóstico no `context` (o servidor decide pelo lint no
cursor), mas o campo `diagnostics` da acção só ECOA o que se mandou: `[8]` com, `[]` sem.
- ⇒ A aba manda no `context` só os diagnósticos que MOSTRA, e só guarda a quickfix que os ecoa: o
  `init`/`update` globais (o aviso que a aba esconde, I3) não ganham «Prefix 'init' with '_'» — que
  partiria o gancho do motor.
- As dicas de tipo só chegam com `inlayHints` na resposta a `workspace/configuration` (I0b §6: com a
  configuração de hoje, `[]`); as capacidades do `initialize` não mudam nada.

## 2. Dicas de tipo (`inlayHint`, para «adicionar tipo»)

[MEDIDO] 42 dicas no texto com erros, 9 de tipo (`kind: 1`); as que trazem `textEdits` inseríveis:
`local s` → `: number`, `local up` → `: boolean`, `local brilho`/`local d` → `: any`, o retorno de
`init`/`update` → `: ()`, o parâmetro `dt` → `: T` (um genérico que o script não declara). O `self`
dos dois ganchos tem rótulo `{+ base_x: number?, … *TRUNCATED*` e **zero** `textEdits` — o `1030` não
tem cura do servidor.
- ⇒ «Adicionar tipo» = a dica de tipo cuja posição é o FIM do nome debaixo do cursor e que traz
  `textEdits`, tirando `any` (não diz nada), o rótulo truncado e o genérico solto (uma letra maiúscula
  que o script não declara — inserido, é um erro novo).

## 3. A lista de candidatas vem da completion do servidor?

[MEDIDO] Pedida no INÍCIO da palavra errada (logo depois do `.` para os membros):

| Sítio | Itens | A certa está? |
|---|---|---|
| `pirnt` (início) | 55 (globais, a API, `self`/`dt` em âmbito, palavras-chave) | `print` sim |
| `ph2d.proprety` (depois do `.`) | 17 = os 17 membros do `ph2d` | `property` sim |
| `self.ampiltude` (depois do `.`) | 12: todo campo que o script LÊ ou atribui em `self` (`base_x`, `t`, `id`, `wave`, …) — **o próprio `ampiltude` incluído** | `amplitude` **não** (agora só aparece no literal da `ph2d.property`) |

- ⇒ **Globais (`1001`) e membros (`1002`): a completion BASTA** — não se mantém lista nenhuma à mão;
  ela já sabe o âmbito (o `dt` do `update`, os `local`), tirando as palavras-chave (`kind` 14).
- ⇒ **`self.<campo>`: a lista é NOSSA** = `Syntax::declared_properties` ∪ os campos atribuídos
  (`self.x = …`) no script ∪ os campos que o motor dá (`self.id`). O servidor infere o `self` pelo uso
  e por isso aceita o erro como se fosse um campo.

## 4. Que distância usa um oráculo?

[MEDIDO] 177 erros (27 + 15 + 12 à mão, e para cada nome com ≥ 3 letras: trocar as letras 1–2, apagar a
1, substituí-la por `q`) contra os 12 campos do bob, 17 globais e os 17 membros do `ph2d`:

| Oráculo | Sugere | Falha a 1 edição (com troca de letras vizinhas) | `pirnt` · `proprety` · `ampiltude` |
|---|---|---|---|
| `rustc` 1.98 (local / campo «with a similar name») | **173 / 177** | **0** | ✓ ✓ ✓ |
| Python 3.14 (`NameError`/`AttributeError` «Did you mean») | 157 / 177 | 15 (`maht`, `tpye`, `gte`, `emti`, `nxet`, …) | ✓ ✓ ✓ |
| Godot 4.7 (`InputMap` «Did you mean», `String.similarity`) | 137 / 177 | 37 (**`pirnt`**, `paris`, `tbale`, `gte`, …) | ✗ ✓ ✓ |

- O Godot só sugere fora do GDScript (o analisador diz «Identifier "ampiltude" not declared» e
  «Function "pirnt()" not found», sem sugestão); a `similarity` dele (pares de letras) dá `0.25` a
  `pirnt`/`print`: **reprova** no caso-guia do I7.
- [MEDIDO] **A lei do `rustc` explica os 177 sem uma falha**: primeiro o nome igual sem contar
  maiúsculas (`AMPLITUDE` → `amplitude`); senão o de menor distância de Damerau (OSA: trocar duas letras
  vizinhas custa 1) com distância ≤ `⌊max(len, 3) / 3⌋`. As três variantes do comprimento (do erro, da
  candidata, o maior dos dois) dão as mesmas 177. Fica a do ERRO (o que se escreveu).
- Empates (`base_z` → `base_x`/`base_y`, `i` → `id`/`t`, `iairs` → `ipairs`/`pairs`): cada oráculo
  escolhe um por uma ordem própria. ⇒ **Nós oferecemos as empatadas todas**, uma acção cada, pela
  ordem da lista de candidatas (a janelinha é uma lista).

## ⛔ Recusas MEDIDAS

| Ideia | Porque não |
|---|---|
| A `String.similarity` do Godot | `pirnt` → `print` dá 0.25; falha 37 de 177 a uma edição |
| Levenshtein simples, limite 1 | as três provas do I7 são TROCAS de vizinhas (Levenshtein 2) |
| A lei do Python | falha 15 trocas em nomes curtos (`gte`, `tpye`, `emti`) |
| Mostrar as acções genéricas do servidor (Extract/Inline/Sort) | não corrigem nada; Extract é refactor de selecção com rename a seguir (I10) |
| A completion do servidor para `self.<campo>` | não tem a `ph2d.property` que só é declarada e tem o próprio erro |
| «Adicionar tipo» ao `self` (o `1030`) | o servidor não dá edição (rótulo truncado, `textEdits` vazio) |
