# `trello_board_real_anon.json` — cabeçalho da fixture

**O que é:** o quadro REAL do dono («Menu → Print, export and share → Export as JSON», trello.com),
exportado e entregue em 2026-10-30 — fecha o bloqueio W10 (`docs/TrelloClone/loop/BLOQUEIOS.md`). Fica
AO LADO da sintética (`trello_board_synthetic.*`), que continua a cobrir os casos que este não tem.

**Anonimizado por ordem do dono** (o repositório é PÚBLICO; o original trazia nomes, nomes de usuário e
e-mails de 6 pessoas e textos internos de trabalho): `docs/TrelloClone/ferramentas/anonimiza_trello.py`.
Cada corrida de letras/dígitos dos campos de texto (`name`, `desc`, `text`, `fullName`, `username`,
`initials`, `url`…, e qualquer string com `@` ou `trello.com`) troca-se por outra do MESMO comprimento,
caixa, dígitos e classe de acento, por HMAC com sal aleatório NUNCA gravado (não se inverte); a mesma
palavra dá a mesma troca em todo o ficheiro, por isso as `@menções` continuam a casar com os
`username`. `descData`/`bioData`/`previews` saem. Ids, datas, `pos`, cores, tipos e a estrutura ficam
intactos. A própria corrida prova: das 652 palavras (≥ 4 letras) do original, nenhuma sobrevive fora
de fragmentos de ids/chaves do formato; zero e-mails reais. O relatório da importação do original e o
do anonimizado são iguais (contagens, perdas, listas), salvo o texto do detalhe.

**O que o real ensinou** (curado em `src/import_actions.rs`):
- `addMemberToCard` de quem já SAIU do quadro: o Trello grava `data.member = {id, name: ""}` (sem
  `username`) e o id não está em `members` — era contado como «ação sem par»; agora é `UnknownMember`.
- 29 `updateCard` «sem par» eram `pos` (20), `idLabels` (7) e `dueReminder` (2): o detalhe da perda
  passou a nomear o campo (`updateCard.pos ×20`).

**Gate:** `tests/it/real.rs` — contagens, cartões por lista (a arquivada à parte), cada perda pelo
nome, e Trello → Kanban → Trello → Kanban igual.
