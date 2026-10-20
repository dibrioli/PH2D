# CodeEditor — a página de ajuda (Onda 2, item 5)

> Leitor: a LLM da linha. Decisão do dono no smoke do Acabamento (19/10): **Ctrl+clique numa função da API abre uma
> PÁGINA DE AJUDA numa aba, como o Godot 4.7.2** — o que faz, cada argumento, um exemplo, as irmãs ao lado; o balão
> preso do I6 era redundante com o do rato parado. Plano escrito ANTES do código (`/pd-feature`, 19/10).
> Oráculo: [`ferramentas/oraculo_ajuda/`](ferramentas/oraculo_ajuda/) (`corre.sh` → `saida/amostras.txt` + recortes) ·
> sonda do assistente: [`ferramentas/lspprobe_ajuda.py`](ferramentas/lspprobe_ajuda.py) →
> [`medicoes_onda2/ajuda_bruto.json`](ferramentas/medicoes_onda2/ajuda_bruto.json) ·
> [`ajuda_bibliotecas.json`](ferramentas/medicoes_onda2/ajuda_bibliotecas.json).

## 1. Estado da arte (corrido, não lido)

**Godot 4.7.2 — o produto de referência do dono** (oráculo de 20/10, `amostras.txt`; licença MIT):

| Gesto | O que acontece (medido) |
|---|---|
| Ctrl+clique `sin`, `clamp`, `print` | abre **@GlobalScope** e rola até ao membro (a descrição dele no topo); a MESMA página é reaproveitada |
| Ctrl+clique `p.length()` | página **Vector2** rolada até `length` (o tipo do valor decide a página) |
| Ctrl+clique `add_child` (herdado) | página **Node** (a classe que DEFINE), não a do script |
| Ctrl+clique `Vector2` / `Node` (classe) | a página no topo |
| Ctrl+clique `PI` | **@GDScript** rolada até à secção Constants |
| Ctrl+clique `amplitude` (nosso) | vai à declaração (linha 2), nenhuma página |
| Ctrl+clique `var` | nada (o ponteiro nem vira mão) |
| A página | entra na LISTA de scripts (ícone de doc, ordem alfabética depois dos scripts); barra própria (título, «Open in Online Docs», «Search Help», ‹ ›) |
| Estrutura | «Class: X» · Description (frase em negrito + texto) · Online Tutorials · Properties · **Methods (tabela de TODAS as irmãs: tipo de retorno alinhado à direita + nome(params), tipos são links)** · Constants · Method Descriptions (• assinatura · texto · bloco de exemplo com botão de copiar · «Note:» em negrito · links inline para outros membros) |
| Índice de secções | Top · Description · … · Method Descriptions, clicável (painel de baixo da lista) |
| Link na mesma página (`push_warning()`) | só ROLA; **não entra no histórico** (‹ a seguir volta ao SCRIPT, à linha do Ctrl+clique) |
| Link para outra classe (`Engine.print_to_stdout`) | abre **Engine** rolada ao membro; ‹ volta a @GlobalScope no `print`; › avança |
| Ponteiro | mão sobre link; seta sobre o texto da página |
| Balão | rato parado sobre `void` da assinatura: «No return value.» |
| Teclado | PgDn rola (com o foco na página); **Ctrl+F** abre uma barra de busca no fundo da página («smoothstep» → «2 of 13 matches», setas ↑↓, ×); **Ctrl+W** fecha a página e volta ao script |
| ⚠️ Gesto | o Godot só reconhece o nome se o rato MEXER com o Ctrl já carregado |

**Outros:**
- **VS Code**: não tem página. O rato parado mostra o balão, e o F12 sobre a API abre o ficheiro de declarações (`.d.ts`): é o «balão preso» que o dono recusou.
- **Roblox Studio** ([Script Editor](https://create.roblox.com/docs/en-us/studio/script-editor)): faz balões com exemplos «parecidos com os da referência» e manda a referência inteira para o site. O *Object Browser* de dentro do Studio ficou com a queixa de que «a maioria das classes não tem descrição» ([DevForum](https://devforum.roblox.com/t/object-browser-improvements/544459)). ⇒ **Recusa: uma página sem texto não serve, e o gate do §5 exige texto em todo membro.**
- **Rive** (scripts em Luau, [blog](https://rive.app/blog/why-scripting-runs-on-luau)): gera os tipos do Luau a partir do motor. Não encontrei página de ajuda no editor dele, e a ajuda ao passar o rato vem de uma extensão da comunidade para o VS Code ([rive-luau-lsp](https://github.com/ivg-design/rive-luau-lsp)).
- **Luau** ([library](https://luau-lang.org/library)): o código é MIT. A licença do TEXTO do site não está confirmada ⇒ **o texto é NOSSO** (escrito por nós, verificado contra o runtime). Crédito ao Luau na página `Luau`, como o README do Luau pede aos integradores.

## 2. O que o assistente diz (sonda `lspprobe_ajuda.py`, luau-lsp 1.70.1 fixado)

`definition` é `[]` em TODA a API (só o gancho `update` dá «aqui»). O `hover` **NÃO serve de porta única**:

| Nome | Hover (1.ª linha) | Diz a página? |
|---|---|---|
| `math.sin`, `string.format`, `buffer.create`, `os.clock`, `coroutine.wrap`, `ph2d.set`, `motion.fire` | `function lib.nome(…)` | sim |
| `m.cos` (`local m = math`) | `function math.cos(…)` | **sim — só o hover o sabe** |
| `s:upper()` / `("x"):upper()` | `function s:upper(string)` / `function ('x'):upper(string)` | só pelo TIPO do 1.º parâmetro |
| `table.insert`, `debug.traceback` | `(<V>(t: {V}, …) -> ()) & (…)` (interseção, sem nome) | não |
| `math.pi`, `math.huge`, `self.id` | `number` | não |
| `print`, `tostring`, `typeof` | `function print(string, string, …)` — especializado aos argumentos DESTA chamada | o nome sim, a assinatura não |
| `math`, `string`, `ph2d` | `type math = { abs: …, … }` cortado | a biblioteca |
| `PoseField` | `type PoseField = "rotation" \| …` | o tipo |

⇒ A porta é NOSSA (§3.1): a **árvore** primeiro, o hover só como desempate (o pseudónimo e o `:`). E a **assinatura da página é nossa**, conferida contra a do assistente onde ele a dá limpa (gate G3).

## 3. Desenho — a porta ÚNICA de cada pergunta

1. **«Que página e que membro nomeia este sítio?»** → `ph2d_code_help::resolve(&Query) -> Option<Anchor>` (crate nova,
   pura). `Query { word, qualifier: Option<&str>, colon: bool, defined_here: bool, hover: Option<&str> }`. Ordem:
   `defined_here` ⇒ `None` (o `goto` vai ao sítio — um `local print` sombreia) · `lib.membro` com `lib` página ·
   global solta da página `globals` · `x:m` com o 1.º parâmetro do hover = página que tem `m` · `function lib.m(` do
   hover (pseudónimo) · `type T =` do hover com `T` na página `types` · a própria `lib` ⇒ o topo. **Chamada por TRÊS
   consumidores e mais nenhum:** a mão do Ctrl (`reach.rs`, substitui `documents`), a chegada do Ctrl+clique/F12
   (`goto.rs::arrive`, substitui `hints.pin`) e o link dentro da página (`[math.floor]` no texto). O `documents` e o
   `Hints::pin` SAEM (o balão preso morre).
2. **«O que diz a página?»** → `ph2d_code_help::pages()` — uma fonte. Linguagem: dados estáticos em
   `ph2d-code-help/src/luau/<biblioteca>.rs` (inglês, decisão de 17/10). PH2D: gerada da `api_table` + o texto
   novo `ph2d-script/src/api_help.rs` (parâmetros, retorno, exemplo de cada entrada, tipo e gancho), **chaveado por
   nome** (não campo novo em `ApiEntry`: outra linha que acrescente uma entrada não quebra a compilação — quebra o
   gate G1, alto). O `luau_docs` passa a ler a MESMA tabela ⇒ os 24 parâmetros hoje com `""` no JSON do assistente
   ganham texto também no balão.
3. **«Como se dispõe a página?»** → `ph2d-panel-code/src/help_page.rs`: `HelpLayout::of(page, width, metrics)` puro
   (blocos → linhas partidas, caixas de link, caixas de exemplo, índice), cacheado por (página, largura, escala);
   pintura só dos blocos visíveis; `hit(x, y) -> HelpHit {Link(anchor), Copy(i), Section(i), Text(offset)}`.
   Formato do texto: parágrafos por linha em branco, `` `código` ``, `[lib.membro]` link, `Note:` em negrito,
   exemplo = código Luau realçado pelo MESMO `ph2d-code-syntax` + as linhas `--> resultado`.
4. **«Onde vive a aba?»** → na MESMA família `CodeDocs` (como no Godot, a página é irmã dos scripts): id
   `HELP_TAG | página` (bit alto; `ScriptId` nunca o usa — `debug_assert` + gate), `tabs()` lista-as depois dos
   scripts, ordem alfabética; `take_focus_request` abre-a. Estado em `ph2d-app-code/src/help.rs`
   (`HelpTabs { open: BTreeMap<PageId, HelpTab{scroll, find, selection, focus}> }`) — `docs.rs` está a 678/700:
   só a composição entra lá.
5. **«Para onde volta o ‹?»** → `ph2d-app-code/src/nav.rs`: UM histórico da família, `Vec<Place>` + índice
   (`Place::Script{id, caret}` · `Place::Help{page, scroll}`). Empilha: Ctrl+clique/F12 que muda de aba e link para
   OUTRA página. Não empilha: link na mesma página, índice, Ctrl+F (medido no Godot). Teclas Alt+← / Alt+→ e os
   botões ‹ › do cabeçalho da página. Fechar uma aba tira as suas entradas.
6. **Teclado na página** (`help.rs::key`): ↑↓ PgUp PgDn Home End rolam; Tab/Shift+Tab andam pelos links, Enter
   segue; Ctrl+F = a `FindBar` existente (contagem «2 of 13»); Ctrl+C copia a seleção (arrastar seleciona);
   **Ctrl+W fecha a aba ACTIVA da família** (página: fecha; script: o fechar de hoje, com ● estaciona).
7. **A família do Ctrl+letra que cai num atalho de letra solta** (§0.10, um bloco): a shell casa `KeyCode::KeyM`
   (tema) e `KeyCode::KeyW` (painel da física) sem olhar modificadores ⇒ guarda única «letra solta = sem
   Ctrl/Alt/Super» no despacho da shell, com gate que varre TODOS os atalhos de letra solta.
8. **O balão com o Ctrl carregado** (aberto do Acabamento §9): o Godot fecha-o ao carregar Ctrl ⇒ `Hints` larga o
   balão no keydown do Ctrl (a mão e o sublinhado dizem o que o clique fará).

**Não persiste** no `.ph2dproj` (a página não é documento do projeto). Diferença declarada: o Godot reabre as
páginas da sessão anterior; nós não temos armazém de sessão do editor.

## 4. Contratos congelados e schema — não encosta

- §6 congela Nodes (`NodeOp`/`OpResolver`/`NodeManifest`), Tools (`Tool`/`RasterEditTool`/`CanvasPaintTool`/
  `PanelEvent`) e Vector. Esta onda toca `ph2d-code-help` (nova), `ph2d-script` (`api_help.rs` + `luau_docs`),
  `ph2d-app-code`, `ph2d-panel-code`, `ph2d-i18n` (rótulos da página) e a guarda de letra solta da shell. Prova no
  fecho: `scripts/collision-surface.sh` («contrato congelado (§6): intocado») + `grep -rn
  'NodeOp\|OpResolver\|NodeManifest\|RasterEditTool\|CanvasPaintTool\|PanelEvent\|VectorOp' <diff da onda>` = 0.
- `DocumentSurface` (foundational, não congelado): **nenhum método novo** — a página usa `tabs`, `paint`,
  `pointer*`, `wheel`, `key`, `close`, `a11y`, `places`, `pointer_look`, `take_focus_request`, que já existem.
- `PROJECT_SCHEMA` / registos / `DOC_VERSION`: intocados (nada gravado). Cena de smoke **+1** (`CENAS` 4 → 5).

## 5. A UI — as quatro condições, cada uma com o seu gate

| Condição | Prova |
|---|---|
| o componente EXISTE | `HelpLayout`/`paint` em `ph2d-panel-code` (unitários: partir, caixas de link, índice) |
| é PINTADO e REGISTADO | a aba aparece em `CodeDocs::tabs()` e a pintura sai na cena montada como a shell (gate da cena 5) |
| o clique CHEGA | `pointer` → `HelpLayout::hit` → `Link(anchor)` → `resolve` (teste: clique no link muda a página) |
| a SEQUÊNCIA leva a algum lado | Ctrl+clique → página → link → outra página → ‹ → volta ao membro → Ctrl+W → script na linha de partida |

A11y: `Document` (nome = título), `Heading` por secção e por membro, `Paragraph`/`TextRun` visíveis, `Link` com
acção `Click`, `Button` «Copy»; o foco do Tab é o `active_descendant`.

## 6. Gates (vermelho primeiro) e a fixture

| # | Gate | A fixture que contém o fenómeno | Vermelho se |
|---|---|---|---|
| G1 | `ph2d-code-help` `toda_global_do_runtime_tem_pagina` | o **runtime REAL** (`ScriptHost` com `sandbox(true)`): percorre `_G` e cada tabela de biblioteca | um nome do runtime sem membro, ou um membro sem nome no runtime (doc morta), ou um membro sem texto/parâmetro sem texto |
| G2 | `cada_exemplo_corre_e_da_o_que_diz` | cada exemplo da linguagem corrido no runtime com `print` capturado | a saída ≠ as linhas `-->` |
| G3 | `oraculo_lsp::a_assinatura_da_pagina_e_a_do_assistente` | `luau-lsp` 1.70.1 fixado, hover de cada função | assinatura nossa ≠ a dele (onde ele a dá com nome) |
| G4 | `resolve::a_tabela_da_sonda` | as 29 respostas gravadas em `ajuda_bruto.json` | uma linha da tabela do §2 decide outra página |
| G5 | `ph2d-script` `toda_entrada_da_api_tem_ajuda` | `API`, `API_TYPES`, `HOOKS`, `API_GLOBALS` | entrada/parâmetro sem texto ou sem exemplo |
| G6 | `ajuda::` (app, servidor falso como em `dicas::`) | script com `math.sin`, `m.cos`, `s:upper()`, `ph2d.set`, `local print`, `local` | página/membro errado; aba duplicada; `local print` abre página; palavra-chave abre algo |
| G7 | `ajuda::navegacao` | o roteiro do oráculo (link na mesma página não empilha; outra página empilha; ‹ › ; Ctrl+W) | a sequência diverge do Godot |
| G8 | `ajuda::teclado_e_leitor` | PgDn, Tab/Enter num link, Ctrl+F «N of M», Ctrl+C da seleção, a11y | qualquer um |
| G9 | `despacho::ctrl_letra_nunca_cai_no_atalho_solto` (shell) | TODOS os `KeyCode` de atalho de letra solta × Ctrl/Alt/Super | um deles dispara |
| G10 | `a_cena_5_conduzida_passo_a_passo` | a cena do smoke montada como a shell | um passo não confere |

Mutação no fecho da linha: `resolve` (cada ramo), o «não empilha na mesma página», o bit da aba, a guarda da
letra solta, o largar do balão no Ctrl.

## 7. Cena de smoke 5 (`PH2D_CODE_SMOKE=5`, `ajuda.luau`)

`math.sin`, `local m = math; m.cos`, `s:upper()`, `ph2d.set(self.id, "x", …)`, uma palavra reservada. Passos:
Ctrl sobre `sin` (mão) → clique: página **math** no `sin` (o que faz, argumento, exemplo) · clique `[math.cos]` no
texto (rola, mesma página) · clique no tipo `number` (página **types**) · Alt+← (volta ao `sin`) · Ctrl+W (script, na
linha de partida) · F12 em `ph2d.set` (página **ph2d**: cada argumento, exemplo) · Ctrl+F «entity» (contagem) ·
Ctrl sobre `local` (sem mão). **Números a MEDIR antes da mensagem ao dono** (sonda `bancada`): quadros entre o
clique e a página pintada; `HelpLayout::of` da maior página (frio e em cache); pintura de um quadro com a página
aberta contra o quadro do editor a 7 500 linhas (I15) — fotos `ferramentas/smoke/cena5/`.

## 8. Ordem

1. `ph2d-code-help` (crate) + G1/G2 vermelhos → texto da linguagem até verde (≈ 170 nomes; a contagem exacta é a do
   G1 — o hover do assistente corta a lista: `ajuda_bibliotecas.json` lê 93) · `api_help.rs` + G5.
2. `resolve` + G4 (fixture da sonda).
3. `help_page.rs` (dispor/pintar/acertar) + unitários.
4. `help.rs` + `nav.rs` + ligação em `goto`/`reach`/`hint` (sai o `pin`) + G6–G8.
5. Guarda da letra solta + G9; o balão no Ctrl.
6. Cena 5 + G10 + bancada + fotos → smoke do dono → `/pd-linha-fechar`.

## 9. Achados do runtime ao escrever o texto (20/10, medidos com a bancada `censo::corre`)

O censo do G1 contou **225 nomes** (12 páginas: `globals` 29 · `bit32` 15 · `buffer` 26 · `coroutine` 8 · `debug` 2 ·
`integer` 41 · `math` 44 · `os` 4 · `string` 17 · `table` 17 · `utf8` 6 · `vector` 16) — o hover do assistente cortava a
lista (93). Com a sandbox, os globais de um script são uma tabela VAZIA com `__index` para os congelados (o censo segue a
cadeia). Defeitos nossos achados:

| Achado | Estado |
|---|---|
| `print(integer.maxsigned)` escrevia `9223372036854776000` (o `integer` voltava ao `tostring` como `number`) | **curado** em `console.rs::wire_print`; gate `o_print_de_um_integer_grande_nao_arredonda` (vermelho com a cura desligada) |
| **`ph2d.wait` não funciona**: o `Scheduler` de `ph2d-script/src/lib.rs` não tem chamador no app (nada retoma a corrotina) e, dentro de um gancho, dá `attempt to yield across metamethod/C-call boundary` | **ABERTO** — controlo morto da API; a página `coroutine` di-lo claramente |
| `loadstring`: o código carregado escreve os globais no ambiente PARTILHADO da thread, não no do script ⇒ dois scripts podem trocar globais por aí | **ABERTO — por medir com dois scripts reais** (isolamento) |
| `require` falha sempre (`require is not supported in this context`) | declarado na página |
| `collectgarbage` só aceita `"count"` | declarado na página |
| O `eval` do `ScriptRuntime` não dá nome ao pedaço: as mensagens apanhadas por `pcall` começam por `crates/ph2d-script/src/lib.rs:89:N:` | só a bancada (os scripts reais têm nome, `module.rs`) |
| `string.rep("x", 1e10)` devolve `""` em vez de erro | por investigar |

**A API nossa** (lida no código ao escrever `api_help.rs`, 20/10; a página diz cada um às claras, as linhas `doc` da
`api_table` ficaram como estavam):

| Achado | Estado |
|---|---|
| `ph2d.find_by_name` devolve sempre `nil` no editor: ninguém fora de `ph2d-script` chama `provide_name` | **ABERTO** — controlo morto |
| `spawn`, `spawn_named`, `despawn`, `attach_script` nunca acontecem: ninguém chama `drain_spawns()` (já registado em `the_intent_drain_reaches_every_variant.rs`); a fila nunca esvazia, sobrevive à reconstrução da VM e, passadas 1024 chamadas na sessão, cada nova chamada é erro que pára o objecto | **ABERTO** — controlo morto que acaba em erro |
| `ph2d.get` só lê objectos que correm um script (a fotografia só os inclui); `ph2d.set` escreve em qualquer objecto com `Transform` | **ABERTO** — assimetria |
| `ph2d.input` só vê o comando de jogo (`gamepad.held.*`, `gamepad.axis.*`); um eixo é `nil` até se mexer uma vez; teclado e rato nunca chegam aos scripts | **ABERTO** |
| Estado lateral: `state_set(e, k, nil)` deixa `k` em `state_keys`; listas voltam com chaves de texto (`#t` = 0); não há como tirar um campo; sobrevive ao rebobinar | **ABERTO** |
| `ph2d.set` com campo desconhecido ou NaN não falha na chamada: o objecto pára quando o gancho acaba | declarado |
| `emit`: o `on_signal` ouve-o no quadro SEGUINTE, em todo objecto com o gancho, o emissor incluído | declarado |
| O Inspector só escreve em `self.<prop>` quando o valor do Inspector muda | declarado |
