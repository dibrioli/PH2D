# `trello_board_synthetic.json` — cabeçalho da fixture

**O que é:** um quadro do Trello SINTÉTICO no formato do ficheiro de «Menu → Print, export and share →
Export as JSON» (trello.com). Escrito à mão (por gerador) em 2026-10-28 porque o dono ainda não
exportou um quadro dele — o JSON REAL fica em `docs/TrelloClone/loop/BLOQUEIOS.md` (W10) e, quando
chegar, entra AO LADO desta como fixture e gate (esta não sai: cobre os casos que um quadro real
pode não ter).

**Fonte do formato:** a documentação pública da API REST do Trello (developer.atlassian.com/cloud/trello,
«Objects»: Board, List, Card, Label, Member, Checklist, Action, Custom Field, Attachment) — a exportação
JSON é o objecto `board` com os arrays aninhados, os mesmos campos da API.

**Não é oráculo** (CLAUDE.md §0.9): a fixture mede o NOSSO mapeamento, não a lei do Trello. O JSON
real do dono é o que fecha a fidelidade.

| Campo no JSON | Objecto da API | Para onde vai no Kanban | Caso coberto |
|---|---|---|---|
| `name`, `starred`, `desc` | Board | `Header.name/starred`; `desc` → perda `BoardDescription` | descrição não vazia |
| `prefs.background` | Board.prefs | `Background::Color` (cor) · `Gradient(i)` (`gradient-*`, índice da lista do seletor) · id de foto → perda + azul | `gradient-ocean` |
| `labelNames` | Board (legado) | ignorado (os `labels` mandam) | presente |
| `pluginData`, `boardPlugins` | Board / Card | perda `PowerUp` | Card Aging no quadro; Power-Up num cartão |
| `members[] {id, username, fullName}` | Member | `Member(username)` — o nome das `@menções` | 3 membros + 1 id fantasma |
| `labels[] {id, name, color}` | Label | `Label`; `x_light` = sutil · `x` = normal · `x_dark` = forte | as 30 cores, `null`, `teal` (desconhecida), sem nome |
| `lists[] {id, name, closed, pos, softLimit, color}` | List | `List` (`card_limit`, `color`, `archived`); `pos` float → `FracKey` | arquivada, `pos` 8192.25 e 1e9, `softLimit:"3"`, cor `sky` |
| `customFields[] {type, options, display.cardFront, pos}` | Custom Field | `FieldDef` dos 5 tipos (`text`, `number`, `date`, `checkbox`, `list`→`Dropdown`) | opções fora de ordem, cor `none` |
| `cards[].customFieldItems[] {idCustomField, value, idValue}` | Custom Field Item | `Card.fields` | os 5 tipos; campo inexistente; número inválido |
| `cards[] {id, idList, pos, closed, name, desc}` | Card | `Card`; `created` = os 8 primeiros hex do id (segundos) | arquivado, empate de `pos`, `pos` 0.001, sem lista (órfão) |
| `idLabels`, `idMembers`, `idMembersVoted` | Card | `labels`, `members`, `votes` | etiqueta repetida, membro e voto fantasma |
| `start`, `due`, `dueComplete`, `dueReminder` | Card | `start`, `Due { at, done, remind_before_min }` | lembrete 1440 e −1; `dueComplete` sem `due` |
| `cover {color, idAttachment, idUploadedBackground, size}` | Card | `Cover` (`size:"full"` = inteira); imagem só com os bytes do anexo | cor metade, cor inteira, imagem sem bytes |
| `isTemplate`, `cardRole`, `subscribed` | Card | `template`; `cardRole` → perda; `subscribed` → perda | modelo, separador |
| `locationName` | Card | `location` | com vírgula |
| `attachments[] {id, name, url, isUpload, fileName, date, pos}` | Attachment | `Link::Url`, ou `Link::File` com os bytes de uma pasta (`DirFiles`) | link, PDF e PNG carregados |
| `checklists[] {idCard, pos, checkItems[] {state, idMember, due, pos}}` | Checklist / CheckItem | `Checklist`, `CheckItem { member, due }` | duas por cartão fora de ordem; órfã; membro fantasma |
| `actions[] {type, date, memberCreator, data}` (mais novas primeiro) | Action | `commentCard` → `Comment` (`dateLastEdited` → `edited`); o resto → `Activity` | ver abaixo |

**Ações cobertas:** `createBoard`, `createList`, `createCard`, `updateCard` (`idList`+`name` numa só,
`closed`, `due`+`start` numa só, `dueComplete`, `desc`, só `pos` = sem par), `commentCard` (com
`@menções`, editado, num cartão apagado), `addMemberToCard`, `removeMemberFromCard`,
`addChecklistToCard`, `removeChecklistFromCard`, `updateCheckItemStateOnCard`, `addAttachmentToCard`,
`deleteAttachmentFromCard`, `deleteCard`, `convertToCardFromCheckItem` (→ `Card.origin`),
`updateList` (`closed`, `name`), `updateBoard` (`name`), `addToOrganizationBoard` e `enablePlugin`
(sem par), e uma ação sem autor.

**Gates que a leem:** `tests/it/fixture.rs` (campo a campo + o relatório de perdas), `round_trip.rs`
(Trello → Kanban → Trello → Kanban), `csv.rs`, `markdown.rs` (instantâneo
`trello_board_synthetic.expected.md`), `fuzz.rs`.
