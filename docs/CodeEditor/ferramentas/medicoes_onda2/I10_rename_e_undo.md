# I10 — rename, referências e o marcador do undo global: medição (18/10/2026)

> Uma rodada (§0.10), antes de codar. Sonda do servidor: [`../lspprobe_i10.py`](../lspprobe_i10.py) →
> [`I10_bruto.json`](I10_bruto.json) (`loadavg` 2,5 → 1,6). Oráculo: [`../oraculo_i10/corre.sh`](../oraculo_i10/corre.sh)
> (VS Code 1.142.0, Xvfb privado, HOME isolado — o `~/.vscode-shared/sharedStorage/state.vscdb` do dono
> ficou com a data de 08:53, conferido antes e depois de cada corrida); fotos em [`I10/`](I10/).
> Código lido por dois exploradores (o desfazer de hoje; as chaves `own` e os caminhos do rename).

## §1 `luau-lsp` 1.70.1 sobre o `bob.luau` (+ uma global, uma sombra, um emoji)

`renameProvider` e `referencesProvider` = `true`; **`prepareRename` e `documentHighlight` = −32601** (não
existem). Resposta do rename: `changes` (nunca `documentChanges`), sempre **só o documento do pedido**.

| Sítio | refs | rename `renomeado` | `end` (reservada) | `1x-y` | nome que já existe |
|---|---|---|---|---|---|
| local (`brilho`, `s`, `d` com emoji antes) | 2 · 6 · 3 | 2 · 6 · 3 edições, UTF-16 certo | **aceita** (código partido) | recusa: «The new name must be a valid identifier…» | **aceita** (`s`→`brilho`: 6 edições, muda o significado) |
| parâmetro (`dt`, `self`) | 2 · 24 | 2 · 24 | aceita | recusa | aceita |
| `local function`, sombra interna/externa | 1 · 2 · 3 | certo, respeita o âmbito | aceita | recusa | aceita |
| global do script (`ajuda`) | 2 | 2 | aceita | recusa | aceita |
| **gancho do motor** (`function update`) | 1 | **1 — renomeia o gancho** (o motor deixa de o chamar) | aceita | recusa | aceita |
| `self.amplitude` · `self.t` · o literal de `ph2d.property("amplitude"` | 0 · 0 · `null` | «Unable to find symbol to rename» | — | — | — |
| global da API / stdlib (`ph2d`, `math`) | 14 · 2 | «Cannot rename a global variable» | — | — | — |
| **membro** da API / stdlib (`ph2d.set`, `math.sin`) | **o servidor MORRE (SIGSEGV)** | **MORRE** | MORRE | recusa (antes de morrer) | MORRE |

12 mortes em 12 pedidos sobre membros (`-11`). Tempo (quente, p50 / máx): 51 linhas `0,02` / `0,3` ms;
7 500 linhas `0,1` / `10` ms (o máximo é o 1.º pedido depois de abrir). Dois documentos abertos: o rename
do `self` de um nunca toca no outro (I0b §7: cada script é um ambiente).

## §2 VS Code 1.142.0 (TypeScript, dois ficheiros `a.ts`/`b.ts`)

- **F2** abre uma caixinha por baixo do nome, **com o nome escrito e todo seleccionado**; rodapé «Enter to
  Rename, Ctrl+Enter to Preview»; as ocorrências ficam realçadas enquanto ela está aberta (`r1`).
- **Enter** aplica nos dois ficheiros e **grava-os** (`files.refactoring.autoSave`, de fábrica): `a.ts` fica
  limpo, `b.ts` nem abre (`r3`).
- **Um Ctrl+Z, no `a.ts` OU no `b.ts`, desfaz os DOIS, sem perguntar**; os dois ficam com ● (`r4`, `r8`).
  O refazer refaz os dois.
- **Conflito** (escrever no `b.ts` depois do rename, desfazer no `a.ts`): aviso no canto «Could not undo
  'Renaming helper to ajuda' across all files becaus…» **e desfaz SÓ o `a.ts`** — o `b.ts` fica com `ajuda`
  e o projecto parte (`b.ts 1` erro, `r9`). ⇒ o oráculo mostra o que NÃO fazer: um passo partido ao meio.
- **Nome inválido / reservado**: aceita `1x` e `class` e parte o código (7 e 4 erros, `r10`, `r11`).
- **Referências**: o cursor parado num nome realça as outras ocorrências (`r14`); Shift+F12 abre a lista
  «References (4)» agrupada por ficheiro (`r15`). Atraso do realce: §4.
- Godot 4.7.2: **não tem** rename nem referências no editor de scripts (a lista de atalhos
  `script_text_editor/*` do binário só tem find/replace e «replace in files»).

## §3 O código de hoje (exploradores)

- Ctrl+Z (`shells/desktop/src/undo_route.rs:118`): a aba de família activa responde primeiro
  (`surface_input::undo`), e responde sempre; sem aba, a fila global (`ProjectUndo`, pares estado+selecção,
  `UNDO_CAP` 256) — registo por DIFF a cada quadro em `post_frame_undo` (`undo_app.rs`), sem API de comando.
- O texto dos scripts (`ProjectScripts`) está FORA do `ProjectState`: nem o diff nem o undo o vêem. As
  chaves `LuauScript.own` estão DENTRO (o `LuauScript` está registado) — o Inspector escreve-as por
  `put`/`forget` e o diff apanha.
- O histórico da aba é por documento (`ph2d-code-buffer::History`, grupos); uma aba LIMPA fechada perde-o,
  uma com ● estaciona-o. Uma aba limpa cujo texto da biblioteca muda é RECRIADA (`follow_library`).
- Nenhum passo composto, marcador ou elo entre as duas filas.
- `own` é por NOME; renomear no texto deixa o valor órfão (D2, `props.rs:255`). Não há «mudar a chave».
- `self_fields` não dá os sítios de `self["x"]` (só escreve); `property_declarations` dá nome + bytes.

## §4 Atraso do realce das ocorrências

6.ª cena do oráculo (`I10_SO=6`, `loadavg` 4,7): o pixel do `helper` da linha 3 é o fundo (`#121314`) na
foto ANTES do clique no `helper` da linha 1, e o realce (`#182D35`) já na 1.ª foto depois, a **125 ms** (a
própria foto leva ~20–100 ms). ⇒ o realce não tem pausa de interface: segue o cursor. O nosso pede ao
servidor a cada (revisão, palavra sob o cursor) nova — `0,1` ms a 7 500 linhas, fora do quadro — e um
pedido novo cancela o velho do mesmo género.

## O que isto decide

1. **A validação do nome é NOSSA, antes de perguntar**: identificador Luau, não reservado, diferente do
   actual, e que não exista já no script (o servidor e o VS Code aceitam os três e partem o código).
2. **Nunca perguntar sobre um MEMBRO** (`a.b`, `a:b` que não seja `self.<campo>`): o servidor morre. Recusa
   nossa com a razão. O gancho do motor (`init`/`update`/`on_signal` globais) também recusa: o motor chama-o
   pelo nome.
3. **`self.<campo>`, `self["campo"]` e o literal do `ph2d.property` são NOSSOS** (o servidor não os vê). Se o
   campo é uma propriedade declarada, o rename leva as chaves `own` de todo objecto do script, e a
   biblioteca muda com ele (o Inspector mostra o nome novo na hora: sem isso o valor ficava órfão até ao
   Ctrl+S). Se o `self` foge (o `self_fields` cala-se), recusa: não dá para ver todos os sítios.
4. **O passo é UM só, nas duas filas**: a fila global ganha um **elo** (o «marcador»), a aba marca o grupo
   com o mesmo elo; um Ctrl+Z na aba ou na cena desfaz o texto, a biblioteca e as chaves juntos. Quando a
   outra metade já não está no topo (escreveu-se depois na aba, ou mexeu-se na cena), **recusa com a razão
   e não desfaz NADA** — o contrário do VS Code, que parte o passo ao meio.
5. **Referências** = realce das ocorrências com o cursor parado (servidor para locais, nosso para campos do
   `self`) e Shift+F12 com a lista.
