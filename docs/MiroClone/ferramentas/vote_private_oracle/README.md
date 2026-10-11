# Oráculo Scrumlr — votação anónima entre pessoas e «modo privado»

O **Scrumlr** (servidor Go) é corrido em `127.0.0.1` sem docker e sem root, e é medido **por script**,
pela API HTTP e pelo WebSocket. Cada participante (M = moderador/dono, A, B, e C, que entra a meio) é
uma sessão anónima própria. Por caso fica gravado cada pedido HTTP e **todas** as mensagens que cada um
RECEBE no WebSocket, com o tempo. Não se leu a lógica de negócio do Scrumlr (serviços de
votes/notes/boards, `event_filter.go`, frontend). Leu-se só o que é preciso para o operar: o README do
servidor, as flags do `main.go`, o `swagger.json`, as rotas do `router.go`, o ramo de *upgrade* do
WebSocket em `api/boards.go` e os campos JSON de dois DTO de pedido (`NoteDeleteRequest.deleteStack`,
`BoardUpdateRequest.showVoting`).

Medido a 2026-10-11. São **26 casos** (23 da W12 e os 3 `h*` do histórico, W13), cada um corrido
**duas vezes**, e as duas corridas coincidem nos 26 depois da normalização (ver §5).

## 1. Triagem de licença por artefacto

| Artefacto | Versão / origem | Licença (ficheiro no artefacto) | sha256 |
|---|---|---|---|
| Scrumlr (código e servidor compilado) | `inovex/scrumlr.io`, tag `v5.4.2`, commit `6bae213129563eea946ec58e7a38d068dc53e0c3` | **MIT** (`LICENSE` na raiz da tag: «Copyright (c) 2017-2021 inovex GmbH») | — |
| Go (compilador) | `go1.26.9.linux-amd64.tar.gz` (go.dev) | **BSD-3-Clause** (`go/LICENSE`, «Copyright 2009 The Go Authors») | `42d158b4…f8d`, confere com o JSON do go.dev |
| nats-server | `v2.14.4` linux-amd64 (release do GitHub, a mesma versão do `docker-compose.yml` do Scrumlr) | **Apache-2.0** (`LICENSE` no tarball) | `20f9d6a1…a1e91ea` |
| PostgreSQL | `18.4`, em `io.zonky.test.postgres:embedded-postgres-binaries-linux-amd64:18.4.0` (Maven Central) | ⚠️ **O artefacto não traz ficheiro de licença.** O POM diz Apache-2.0, mas isso cobre o empacotamento. O Postgres é **PostgreSQL License**. O bundle traz ainda `libcrypto`/`libssl` 1.1 (OpenSSL), ICU, libxml2, libxslt, liblzma, zlib e ossp-uuid, todos permissivos | `401d4e69…e212` |
| Dependências Go do servidor | as do `go.sum` da tag (baixadas para `~/Apps/scrumlr-oracle/gopath`) | não triadas uma a uma. Só correm aqui como ferramenta, nada é redistribuído nem copiado para o PH2D | — |

Para o PH2D o oráculo é uma **ferramenta que se corre**. Nada do Scrumlr entra no repositório além das
saídas gravadas, que são as nossas entradas e as respostas observadas.

## 2. Como correr (comandos exactos)

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-MiroClone/docs/MiroClone/ferramentas/vote_private_oracle
bash instala.sh          # 1× só: baixa Go/nats/Postgres, verifica sha256, clona a tag e compila (via scripts/ph2d-run.sh)
bash start.sh            # Postgres :55432, nats :54222 (-js), scrumlr :58080 — todos em 127.0.0.1, sob `timeout` (PRAZO=7200 s)
~/.local/bin/node oraculo.mjs --corrida 1 --out saidas/scrumlr                 # todos os casos (~3 min; o v05b espera 65 s)
~/.local/bin/node oraculo.mjs --corrida 2 --out saidas/scrumlr/corrida2
~/.local/bin/node compara.mjs saidas/scrumlr saidas/scrumlr/corrida2 saidas/scrumlr/comparacao_corrida1_corrida2.json
~/.local/bin/node resume.mjs saidas/scrumlr/p01.json     # leitura humana, passo a passo (--cheio = sem cortar)
bash stop.sh             # pára SÓ os PIDs gravados em ~/Apps/scrumlr-oracle/run/pids
```

O servidor arranca com `--insecure --disable-check-origin --enable-swagger --join-rate-limit 1000
--allow-anonymous-board-creation --allow-anonymous-custom-templates`. Corre-se um só caso com
`--so v01,p01`. Não há sudo: o Postgres usa `initdb -U admin --auth=trust` num dir próprio
(`run/pgdata`), o socket em `run/` e a base `postgres`, onde o servidor migra o esquema (versão 28).

## 3. O que cada ficheiro faz

| Ficheiro | Papel |
|---|---|
| `casos.json` | Entradas nossas: 2 colunas (Bom, Mudar), 5 notas com autor (n1–n2 de A, n3/n5 de B, n4 de M), a votação base e a frase de cada caso |
| `meta_scrumlr.json` | Cabeçalho fixo copiado para o `meta` de cada saída (tag, commit, versões, flags) |
| `instala.sh` / `start.sh` / `stop.sh` | Instalação sem root, arranque com PIDs gravados e paragem por PID |
| `oraculo.mjs` | Os casos. Um quadro novo por caso: M cria, A e B entram (`POST /boards/{id}/participants`) e abrem o WS (`GET /boards/{id}` com `Upgrade: websocket` e cookie `jwt`). Depois de cada passo espera 250 ms sem mensagens (no máximo 4 s), para cada mensagem ficar no passo que a causou |
| `normaliza.mjs` | Troca os ids por rótulos (A, n1, VOT1, `col:c1`, BOARD) e as datas por `<ts>`. Tem também a forma canónica usada na comparação |
| `resume.mjs` | Mostra um `<caso>.json` em texto: passos, pedidos (status) e `⇐ quem TIPO dados` |
| `compara.mjs` | Compara duas corridas caso a caso e passo a passo (HTTP, e o WS por participante) |
| `saidas/scrumlr/<caso>.json` | Corrida 1: `meta` (versão, commit, data, corrida n.º), `ids`, `linha_do_tempo` (`t` em ms desde o início do caso) e `recebido[participante]` |
| `saidas/scrumlr/corrida2/<caso>.json` | Corrida 2, no mesmo formato |
| `saidas/scrumlr/comparacao_corrida1_corrida2.json` | Resultado da comparação (`iguais` e `difs` por caso) |

⚠️ A entrada `http` da linha do tempo é gravada quando a **resposta** chega. Por isso as mensagens WS
que o servidor empurra durante o pedido aparecem às vezes ANTES da linha do pedido, no mesmo passo.

## 4. Resultados caso a caso

Configuração base da votação: `voteLimit 5, allowMultipleVotes true, showVotesOfOthers false,
isAnonymous true`. «Nada» quer dizer zero mensagens WS naquele passo.

| Caso | O que A / B / M recebem | Veredito numa frase |
|---|---|---|
| **v01** A vota com a votação aberta | A, B e M: **nada** no WS (nem o próprio A). `POST /votes` → 201 `{voting,note,user}` só a quem votou. `GET /votes`: A vê só os seus, B só os seus, M (que não votou) `null`. `?voting=` e `?note=` também só devolvem os do próprio | Durante a votação **ninguém recebe nada**: cada um só conhece os próprios votos |
| **v01b** idem com `showVotesOfOthers=true` | Igual ao v01, byte a byte, salvo o campo `showVotesOfOthers` | A opção **não muda nada que a API mostre**. Se tem efeito, é só no cliente web, que não se mediu |
| **v02** C entra a meio | A, B e M: `PARTICIPANT_CREATED`, `SESSION_UPDATED(connected)` e um `COLUMNS_UPDATED`+`NOTES_SYNC` de ressincronização. O `INIT` de C traz a votação `OPEN` com a configuração e `votes: []` (nenhum voto dos outros). C vota → 201, e o fecho conta o voto dele (total 2) | Quem chega a meio **vota normalmente** e recebe o estado sem os votos alheios |
| **v03** A vota e fecha o WS | B e M: 2× (`COLUMNS_UPDATED`+`NOTES_SYNC`) e **nenhum** `SESSION_UPDATED connected:false`. No fecho, total 4 (inclui os 3 de A) | Sair (fechar a ligação) **não apaga** os votos: contam |
| **v03b** A apaga a conta | Todos recebem `NOTE_DELETED` das notas de A (n1, n2). No fecho a votação vem **sem `votes`**: os 3 votos de A desaparecem, e também o voto de B em n1, porque a nota morreu | Apagar a conta apaga as notas da pessoa e, com elas, **os votos de todos** nessas notas |
| **v03c** M expulsa A (`DELETE participants/{A}`) | `500 Internal server error`. Nada muda e o fecho conta tudo (4) | Expulsar pela API **não funciona** nesta versão (não medido) |
| **v03d** M bane A (`banned:true`) | Todos: `SESSION_UPDATED banned:true` (A continua `connected:true` e a receber). No fecho, total 4 | Banir **não retira** os votos já dados |
| **v04** M fecha (`CLOSED`) | M, A e B recebem **ao mesmo tempo** um `VOTING_UPDATED {voting:{…status CLOSED, votes:{total, votesPerNote:{nota:{total}}}}, notes:[…]}`. É aqui que o total por nota aparece pela 1.ª vez. `GET /boards/{id}` passa a mostrar `showVoting: VOT1` sem que nenhum `BOARD_UPDATED` tenha sido empurrado. ⚠️ `PUT /boards {showVoting:VOT1}` feito por M **zera** o `showVoting` para `null`, e o `BOARD_UPDATED` vem sem `name`. Uma 2.ª votação não apaga os resultados da 1.ª (`GET /votings` lista as duas) | O resultado chega **de uma vez a todos, só no fecho**, e o servidor marca-o sozinho como «a mostrar» |
| **v04b** M cancela (`ABORTED`) | Todos: `VOTING_UPDATED` só com o objecto da votação, **sem `votes` nem `notes`**. `GET /votings/{id}` fica sem resultado, mas `GET /votes` de B ainda lista os votos dele | Cancelar **descarta o resultado** para todos, e cada um continua a ver os seus votos |
| **v05** relógio por `PUT timerStart/timerEnd` (+3 s) | Todos: `BOARD_UPDATED` e `NOTES_SYNC` ao pôr o relógio. Quando o relógio acaba, **nada**. Aos 6 s A ainda vota (201) e a votação continua `OPEN` | O relógio **não** termina a votação |
| **v05b** cronómetro `POST /timer minutes=1`, espera 65 s | Todos: `BOARD_TIMER_UPDATED` ao ligar. Ao fim de 60 s, **nada**. A vota aos 65 s (201) e a votação continua `OPEN` | **Não**: o cronómetro e a votação são independentes |
| **v06** limites | Sem múltiplos: repetir a mesma nota → **404**, 3.º voto com limite 2 → **404**. Retirar um voto (`DELETE /votes`) → 204 e liberta a vaga. Retirar onde não votou → **204** (silencioso). Com múltiplos: 2× na mesma nota passa, o 3.º → 404 (limite). Votar sem votação aberta → 404. Nenhum destes gera mensagem WS | O limite vale por pessoa e por votação, e a recusa é um 404 mudo |
| **v07** `isAnonymous=true`, o resultado | `votesPerNote: {n3:{total:2}, n1:{total:1}}`, **sem `userVotes`**, igual para M, A e B (WS e REST) | O resultado anónimo **não diz quem votou**, nem ao moderador |
| **v07b** `isAnonymous=false` | `votesPerNote` com `userVotes:[{id:A,total:1},{id:B,total:1}]`, **para todos** (M, A e B) | Não anónimo = todos veem quem votou em quê, mas **só depois de fechar** (durante a votação, igual ao v01) |
| **v08** modo privado + votação | B vota na nota escondida de A (o id veio de fora) → **201**, e o voto conta no fecho | O servidor **aceita votos em notas que o votante não vê** |
| **p01** modo privado, A escreve | A: `NOTES_UPDATED` com as **suas** notas. M: todas. B: **`NOTES_UPDATED []`**, uma lista vazia que chega a cada escrita alheia, sem contagem, sem marcador, sem posição e sem texto. C, entrando depois, recebe no `INIT` `notes: []`. ⚠️ Por **REST** o B obtém tudo: `GET /notes` e `GET /notes/{n1}` devolvem o texto e o autor das notas de A | A privacidade é **só do canal empurrado**: o B recebe um «algo mudou» vazio, e a REST **vaza o texto** |
| **p02** o moderador vê? | M (dono) recebe todas as notas, também as de A. A nota que o próprio M escreve fica escondida de B (`[]`). Promover B a `MODERATOR` faz chegar-lhe de imediato um `NOTES_SYNC` com **todas** as notas, e daí em diante B recebe tudo | O moderador vê tudo. Esconder vale também para as notas do moderador, e o papel é que decide |
| **p03** revelar | Cada um recebe `BOARD_UPDATED(showNotesOfOtherUsers:true)` e **um** `NOTES_SYNC` com todas as notas, num só passo | A revelação é **de uma vez**, um instantâneo completo, nunca nota a nota |
| **p04** revelar e esconder de novo | Ao esconder, B recebe um `NOTES_SYNC` só com as **suas** notas, e as de A **somem** (o cliente substitui a lista). Uma edição posterior de A volta a chegar a B como lista só com as dele | Esconder de novo **volta a esconder** o que já tinha sido mostrado |
| **p05** editar, mover e apagar escondida, desfazer | Editar e mover: B recebe `NOTES_UPDATED []`. ⚠️ Apagar: B recebe `NOTE_DELETED` com o **id** da nota escondida. B a editar uma nota de A → 403, a apagar → 403. Depois de apagada, `GET` → 404. Não há rota de desfazer nem de restaurar | Editar e mover são invisíveis, apagar **vaza o id**. **Desfazer não existe** na API |
| **p06** autor escondido (`showAuthors=false`) | B recebe as notas alheias com `author: "00000000-0000-0000-0000-000000000000"` (o mesmo UUID nulo para todos os autores), e a sua com o próprio id. M recebe os autores reais. ⚠️ `GET /notes` de B devolve os autores **reais**. Mostrar de novo → `NOTES_SYNC` com os autores | O autor é trocado por um id nulo **só no WS**, e a REST vaza-o |
| **p07** partilhar (foco) uma nota escondida | B recebe `BOARD_UPDATED sharedNote:n1` (o **id**), mas `NOTES_SYNC []` (sem conteúdo). `GET /notes/n1` de B dá o texto. ⚠️ `PUT {sharedNote:null}` sozinho põe `showNotesOfOtherUsers:true`, **revelando tudo a todos**, e o quadro perde o `name`. `PUT {sharedNote:null, showNotesOfOtherUsers:false}` **não** limpa o `sharedNote` | Focar uma nota escondida mostra o id e não o conteúdo. Desfocar tem um efeito colateral que **revela** o quadro |
| **p08** coluna escondida (`visible:false`) | Nota de M na coluna escondida: B não recebe. A escreve lá → 201, mas **A não vê a própria nota** (`NOTES_UPDATED []`). M vê tudo. `GET /notes` de B devolve as duas. Revelar a coluna → `NOTES_SYNC` com tudo para todos | A coluna escondida é um 2.º mecanismo de «só o moderador vê», que esconde até do próprio autor |

### O histórico das votações (W13, 11/10: `h01`–`h03`)

| Caso | O que se observa | Veredito numa frase |
|---|---|---|
| **h01** VOT1 `CLOSED`, VOT2 `ABORTED`, VOT3 `CLOSED` seguidas | Abrir uma 2.ª com VOT1 aberta → **400** «only one open voting per session is allowed». `GET /votings` guarda **as três**: as `CLOSED` com `votes{total, votesPerNote}`, a `ABORTED` **sem** resultado. Nenhuma tem nome, data nem autor. `showVoting` passa sozinho à última `CLOSED` (VOT1 → continua VOT1 depois da VOT2 cancelada → VOT3). `PUT {showVoting:VOT1}` com VOT3 a mostrar **zera** o `showVoting` (`null`) | Uma votação de cada vez. Todas ficam guardadas, a cancelada sem resultado. O quadro mostra a **última acabada**, e não há forma de voltar a mostrar uma antiga |
| **h02** a nota mais votada é apagada DEPOIS do fecho; apagar e renomear | Depois de `DELETE notes/n3`, `GET /votings/VOT1` dá `total 1, {n1:1}` (era `total 3, {n3:2, n1:1}`). `DELETE /votings/{id}` → **405** (a rota não existe). `PUT /votings/{id} {name}` → **404** | O resultado guardado **não é um instantâneo**: é recontado a partir dos votos vivos, e apagar uma nota tira-a (e aos votos dela) do resultado antigo. Não se apaga nem se renomeia uma votação |
| **h03** C entra depois de duas votações fechadas | O `INIT` de C traz `votings` só com a **última** (VOT2) e `votes` com os **boletins crus** dela: `[{VOT2, n1, user:B}, {VOT2, n1, user:B}]`, numa votação `isAnonymous:true`. `GET /votings` de C dá as duas, com resultado. `GET /votes` de C → `null` | Quem entra depois recebe o histórico só por pedido, e a mensagem de entrada **vaza quem votou em quê** na última votação «anónima» |

⚠️ **O que isto ensina ao PH2D** (ADR-0183): o resultado de uma sessão guarda-se como **instantâneo**
(totais por elemento no momento do fim), nunca recontado a partir dos boletins, que **não saem do cofre do
anfitrião** (ADR-0182 §7). O h03 mostra a alternativa: guardar os boletins para recontar abre a fuga.

### Outros comportamentos relevantes («é de uma pessoa e só se mostra quando alguém manda»)

- **O canal empurra listas completas, não diferenças.** Cada `NOTES_UPDATED` e `NOTES_SYNC` é a lista
  inteira do que **aquele** participante pode ver, já filtrada por participante. Quem não pode ver
  nada recebe `[]`. Esse vazio ainda revela **quando** alguém mexeu (um evento por escrita alheia).
- **Duas serializações:** M (dono) recebe JSON com as chaves por ordem alfabética, e A/B pela ordem dos
  campos da struct. Entre corridas, o mesmo participante pode receber qualquer das duas.
- **Votar é silencioso** para todos até ao fecho. O fecho é um único evento com o total, e no não
  anónimo com quem votou.
- **Ressincronização a cada entrada ou saída:** quando alguém entra ou fecha o WS, os outros recebem
  `COLUMNS_UPDATED`+`NOTES_SYNC` completos (2× na saída). Não chega `SESSION_UPDATED connected:false`.
- **O `PUT /boards/{id}` com `showVoting`/`sharedNote` a `null`** tem efeitos estranhos: zera
  `showVoting`, liga `showNotesOfOtherUsers`, tira o `name` do evento e a resposta traz
  `lastModifiedAt` em `0000-12-31…`. Os outros campos do `PUT` parcial (`showNotesOfOtherUsers`,
  `showAuthors`, `timer*`) comportam-se como esperado.

## 5. Duas corridas: coincidem?

**Sim, nos 23 casos** (`comparacao_corrida1_corrida2.json`: `iguais: true` em todos, com o mesmo
número de passos, os mesmos pedidos/status/respostas e o mesmo multiconjunto de mensagens WS por
participante e por passo). O `v03d` foi acrescentado depois e corrido também duas vezes, contra o mesmo
servidor. Os `h01`–`h03` também (W13). ⚠️ O `h03` deu `DIFEREM em 5:ws:C` na 1.ª comparação: a
ressincronização que a ENTRADA de C provoca chegou a C depois dos 250 ms de silêncio e caiu no passo
seguinte. A cura é esse passo esperar `1 500 ms` de silêncio (`assenta(1500, 6000)`), não normalizar a
diferença.

A normalização, com o porquê de cada regra medido na 1.ª comparação, que sem ela dava 16 de 22 casos
«diferentes», todos por ruído:
1. **Ids aleatórios → rótulos.** Utilizadores → `M/A/B/C`, quadro → `BOARD`, notas → a chave de
   `casos.json`, colunas → `col:c1`, votações → `VOT1, VOT2` pela ordem de criação, também quando são
   chaves de objecto (`votesPerNote`). Um UUID desconhecido (só o UUID nulo do autor escondido) →
   `UUID`.
2. **Datas ISO-8601 → `<ts>`.** O tempo `t` não entra na comparação.
3. **Chaves de objecto ordenadas**, porque a mesma mensagem sai com as chaves em duas ordens (ver §4).
4. **Arrays de objectos ordenados pelo seu JSON canónico.** Listas de notas, votos, participantes e
   `votesPerNote` saem em ordem não estável entre corridas, e a ordem com significado vive em
   `position.rank`/`index`, que se compara. Arrays de valores simples mantêm a ordem.
5. **As mensagens WS de um participante dentro de um passo** comparam-se como multiconjunto (ordenadas),
   e o passo é a fronteira: cada passo espera 250 ms de silêncio.

**Controlo de que o comparador não é cego:** injectar numa cópia da corrida 2 uma nota visível no
`NOTES_UPDATED []` de B (p01) e +1 num total (v07) dá `DIFEREM` exactamente em `p01 3:ws:B` e
`v07 4:http`.

## 6. O que NÃO se mediu

- **O cliente web (React).** Não se correu o frontend. O que o ecrã faz com `NOTES_UPDATED []`, se
  mostra «há N notas escondidas», o efeito de `showVotesOfOthers`, a animação da revelação, o
  «desfazer» do cliente (se existir) e o ecrã de resultado `showVoting` ficam por medir. Seria preciso
  um navegador sem cabeça contra o build do frontend.
- **Expulsar** (`DELETE /boards/{id}/participants/{id}`) devolveu 500, por isso o «sair à força» só se
  mediu por ban, fecho do WS e apagar a conta.
- **Mexer no `showVoting` à mão:** o `PUT` não o liga (zera-o), por isso «mostrar o resultado de uma
  votação antiga» não se mediu.
- **Montes (`stack`), reacções e `deleteStack:true`**, e o «ready»/«raised hand» como sinal de
  «terminei de escrever», ficaram fora.
- **Concorrência real** (dois votos no mesmo milissegundo, o limite sob corrida) não se mediu: os
  passos são sequenciais.
- **Utilizadores com conta (OAuth):** só sessões anónimas.

## 7. Processos

`stop.sh` pára os três processos pelo PID gravado (`postgres`, `nats-server` e `scrumlr-server`, cada
um sob `timeout`). Depois do fecho, `pgrep -a postgres`, `pgrep -a nats-server` e
`pgrep -a scrumlr-server` não devolveram nada. Ficam em `~/Apps/scrumlr-oracle/`: as fontes, os
binários, o `gocache`/`gopath` (~1 GB) e os dados do Postgres em `run/pgdata`, que o `start.sh`
reutiliza.
