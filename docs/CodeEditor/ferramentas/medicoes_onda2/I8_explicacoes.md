# I8 — as explicações escritas à mão (17/10)

**Reproduzir** (da raiz da worktree): o censo `bash scripts/ph2d-run.sh cargo test -p ph2d-code-explain
--test it -- censo::` lê o corpus do I0c (`ferramentas/corpus_erros/*.jsonl` e `classes_*.tsv`); os
oráculos `bash docs/CodeEditor/ferramentas/oraculo_explicacoes/corre_tudo.sh <pasta-vazia>` regravam
[`I8_oraculos_bruto.txt`](I8_oraculos_bruto.txt) (Godot 4.7.2 MIT, `luau-lsp` 1.70.1 MIT, `rustc` 1.95
MIT/Apache-2.0, Python 3.14.8 PSF-2.0, Lua 5.5 / LuaJIT MIT — corridos sobre entradas nossas, nenhum
fonte lido). `loadavg` no fim `3.48 26.60 25.38` (nenhum tempo aqui decide nada).

## 1. As classes do corpus

[MEDIDO] O corpus do I0c tem **54 classes na via estática** (`luau-lsp`, 1 661 sítios) e **47 na via do
motor** (669), 101 linhas. A classe é a mensagem com nomes, números e posições trocados por marcadores
(`corpus_erros.py::classe`); as tabelas `classes_estatico.tsv`/`classes_runtime.tsv` saem de
`corpus_erros.py tabelas` e reproduzem-se byte a byte. O `I0c_erros_frequentes.md` é escrito à mão e só
mostra as 27 + 24 primeiras. Todo o corpus é NÃO estrito.

- `classe()` não tira o sufixo `caused by: …` nem a vírgula: o `1000` aparece em 5 classes e o `1008` sem
  nome à parte. Para o I8 são variantes da MESMA explicação.
- As linhas de sintaxe repetem-se nas duas vias (o motor dá a mensagem sem o prefixo `SyntaxError:`).
- Classes NOSSAS no corpus: só as três `Decl:*` do motor (`ph2d.property` com default, nome ou dica
  errados). O aviso do I7 (`self.X is never set`) não está no corpus — entra no censo como classe nossa.
- O `1002` (`Key 'x' not found in table '…'`): 25 das 51 mensagens trazem a tabela inteira do `ph2d`
  (até 571 caracteres, `*TRUNCATED*`); a chave é o primeiro `'…'`. O tipo não se lê por aspas (tem
  literais `"rotation"`). As únicas outras com a mesma forma são 25 `1000` (`but got '{ attach_script…`).

⇒ **55 explicações** (`ph2d_code_explain::Kind`): 53 que cobrem as 101 classes juntando o que pede o
mesmo conserto, mais as 2 nossas (`unknown_field`, `unknown_field_near`).

## 2. Onde a mensagem aparece hoje

[MEDIDO, leitura do código] Não há ponto único: sete sítios montam o texto, e **o editor não mostrava a
mensagem em lado nenhum**.

| Superfície | Antes do I8 | Depois |
|---|---|---|
| Calha (`ph2d-panel-code/glyphs.rs`) | um ponto por gravidade, sem texto | o rato parado no ponto: **a explicação** de cada problema da linha (título marcado + porquê); não pergunta ao assistente |
| Sublinhado | só a onda | o rato parado no culpado: **a mensagem crua**, por cima do que o assistente diz da palavra |
| Lista de problemas | **não existe** (o `Problem.message` só alimentava a contagem) | — (ABERTO, ver §5) |
| Cartão do Inspector (`script_avisos.rs`) | a 1.ª linha da mensagem do Luau sem o símbolo + o chip do símbolo (E7) | **o título da explicação** + o chip + o porquê por baixo |
| Console (`console.rs`) | a 1.ª linha crua | igual (o cru) |
| Barra de estado | só as contagens | igual |
| Balão do rato | só o `Hover` do assistente | + as secções dos problemas |

O `Diag` do assistente guarda o `code` (`1014`, `1001`…), mas o `Problem` não; o `source` nem é lido.
A explicação lê-se da MENSAGEM (o motor não dá código), por isso as duas vias usam a mesma porta.

## 3. Idioma

[MEDIDO] O `ph2d-i18n` só tem `Ingles` e `Teste` (o pseudo-idioma derivado do inglês); `PH2D_LANG=pt`
cai no inglês com aviso. Nenhum módulo tem português, e o gate `nenhuma_frase_que_o_artista_le_esta_em_portugues`
cumpre a ordem do dono de 19/09 («tudo em inglês»). O plano dizia «PT/EN».
**Decisão do dono (17/10): as explicações só em inglês, como o resto do app**; o texto mora em
`ph2d-i18n/src/code_explain.rs`, pronto para outro idioma quando o app inteiro o tiver.

## 4. Os oráculos

[MEDIDO] Nenhum põe uma explicação ao lado da mensagem:

| Oráculo | Explicação? | Forma |
|---|---|---|
| Godot 4.7.2 (`--check-only`, execução, LSP do editor headless) | não; o LSP manda `code: 0`, sem `codeDescription`; o `--doctool` sai VAZIO nos 49 avisos | frase técnica «esperava X, recebeu Y»; o único conselho é o do aviso (`prefix it with an underscore`) |
| `luau-lsp` 1.70.1 | só um link por código (`codeDescription.href`): o dos lints vai à secção certa de `luau.org/lint`; o de TODOS os erros de tipo e sintaxe é o genérico `luau.org/types` | mensagem com dica no fim |
| `rustc` 1.95 | **sim**, por código (`--explain E0xxx`, também no JSON) | título de uma frase · exemplo errado · o que fazer · exemplo certo (460–1 000 caracteres) |
| Python 3.14.8, Lua 5.5 | não | mensagem curta; o Python às vezes «Did you mean» (o `pirnt` não teve) |

⇒ a forma do `rustc` em pequeno: **título de uma linha com o nome real · uma ou duas frases do porquê ·
o conserto concreto** (o Alt+Enter quando o I7 o tem). Para os erros de tipo e de sintaxe a nossa
explicação é a única fonte (o link do `luau-lsp` não diz nada deles).

## 5. A lei e as provas

- `ph2d-code-explain` (crate nova, folha sobre `ph2d-i18n`): `explain(mensagem) → Explained{kind, nomes}`;
  normaliza espaços e a posição de outro chunk (`bob.luau:3: …`), tira `TypeError:`/`SyntaxError:`; o
  tipo enorme vira «a table»/«a function».
- **Censo** (`tests/it/censo.rs`): as 2 400+ mensagens do corpus têm explicação; cada linha das duas
  tabelas também (classe nova ⇒ vermelho, com o nome dela); cada `Kind` tem título e porquê sem
  marcador por encher e o título cabe em 90 caracteres; nenhuma `Kind` (não nossa) ficou sem mensagem
  no corpus. O `1002` de 567 caracteres → «There is no “proeprty” here».
- **Ecrã** (`ph2d-app-code/tests/it/explicacoes.rs`, servidor falso): o `=` do smoke 2 explica-se na
  calha e mostra o cru no sublinhado, nenhum antes da pausa de 0,5 s, a calha nunca pergunta ao
  assistente, e só a explicação conta como vista; sem assistente e com ele parado a calha explica.
- **Registo local** (sem rede): `code-explanations-seen.tsv` na pasta de dados (`PH2D_CODE_SEEN_FILE` ·
  `$XDG_DATA_HOME/ph2d` · `~/.local/share/ph2d` · `%APPDATA%\ph2d`), uma linha `<segundos>\t<classe>`
  por explicação que um balão da calha abriu. Os testes ficam na memória (`set_seen_file(None)` na
  `Mesa`). O cartão do Inspector não regista (é outra família).
- **ABERTO:** uma LISTA de problemas (painel) não existe — a decisão de a criar é do dono.
