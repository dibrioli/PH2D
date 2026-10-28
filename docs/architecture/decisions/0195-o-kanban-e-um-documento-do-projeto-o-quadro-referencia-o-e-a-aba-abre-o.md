# ADR-0195 — O Kanban é um documento do projeto; o Quadro referencia-o e a aba abre-o

- **Status:** Accepted. O produto é ordem do DONO: o mesmo Kanban «nos dois lugares», no projeto, sem Mapa,
  IA no fim, calendário interno (27/10, plano §8). O desenho técnico é delegado.
- **Data:** 2026-10-27
- **Linha:** `line/TrelloClone` (W1)
- **Amparo:** [plano](../../TrelloClone/01_plano.md) §2–§4 e §8 · ADR-0181 (réplica por campo) · ADR-0184
  (ficheiro por conteúdo) · ADR-0185 (o `Embed` como molde de elemento que referencia)

> ⚠️ **Número PROVISÓRIO** (§5.0): o plano dizia `0192`. A contagem de 27/10 contra o `main` (`0191`) e
> todas as worktrees achou `0192` em `line/PixelArt` e em `line/BlenderConstraints`, e `0193`/`0194`
> nesta última. Quem integrar reconta-o.

## Contexto

O dono pediu (27/10) um Trello completo que vive como quadro dentro do canvas do Quadro **e** abre em
aba exclusiva: *«o mesmo Kanban pode ser aberto a qualquer momento das duas formas, nos dois lugares»*.
Medido na W0: nenhum código de kanban no repo. O Quadro já tem `FracKey`, réplica por campo (ADR-0181),
equipa e menções, ficheiro por conteúdo, e um desfazer **por quadro** fora do `ProjectState`.

## Decisão

1. **Um documento de primeira classe.** `ph2d-kanban-model`: `KanbanSet` → `Kanban` (cabeçalho + oito
   tabelas `BTreeMap`: listas, cartões, etiquetas, checklists, itens, comentários, anexos, campos).
   Muda-se só por `KanbanOp` (`Put` · `Delete` · `Header`), que devolve o inverso. A crate é pura:
   sem ECS, sem render, sem UI.
2. **No projeto, um blob irmão do `boards`.** `ProjectFile.kanbans: Vec<u8>`, o último campo
   (`PROJECT_SCHEMA 193 → 194`), com versão PRÓPRIA (`ph2d_kanban_model::FORMAT_VERSION = 1`). Uma
   mudança interna sobe a dele, nunca o schema. Em tempo de execução o dono é
   `ph2d_editor_core::documents::Documents::kanbans`, ao lado dos quadros: a aba (W2) e o elemento do
   canvas (W4) estão ambos ao alcance dele.
3. **Duas vistas, um documento.** A aba e o elemento do Quadro guardam cada um uma `KanbanView`: o
   `KanbanId` e o que é SÓ dessa vista (listas recolhidas, rolagem; na W6, o filtro). O que a vista
   mostra é `KanbanView::shown(&KanbanSet)`, uma projecção pura. Por isso não há sincronização entre
   janelas: as duas leem o mesmo registo. O elemento do canvas (W4) será
   `ElementKind::Kanban { view: KanbanView }`, variante no FIM.
4. **Um registo por coisa, com o que a colaboração pede.** Cada registo tem `Meta { version, nonce,
   deleted }` (o molde do `Element`, ADR-0181). Comentários, checklists, itens e anexos são registos
   próprios, filhos que apontam ao pai, e não `Vec` dentro do cartão: dois colegas a comentar o mesmo
   cartão escrevem dois registos. A ordem é por `FracKey`, então reordenar é uma escrita. Os ids são
   um espaço por Kanban; o bit 63 fica para os sorteados numa sessão ao vivo (W5).
5. **Desfazer POR KANBAN, fora do `ProjectState`** (o handoff da W0 dizia `ProjectState.kanbans`; a
   medição derrubou-o). O `History` guarda o lote INVERSO, como o do Quadro, e desfazer não pisa um
   registo que outro mudou depois. O `ProjectState` restaura por FOTOGRAFIA: numa sessão ao vivo, um
   `Ctrl+Z` apagaria as edições do colega. Também guardaria o documento inteiro a cada passo. A hora
   de cada passo é a do `undo_tick` único, passada por quem chama, para a crate ficar pura.
6. **O `FracKey` mora em `ph2d-frac`**, movido (`git mv`) do `ph2d-board-model`, que o reexporta: os
   bytes e o caminho público `ph2d_board_model::FracKey` não mudam. Sem o movimento, a W4 teria um
   ciclo: o Kanban dependeria do Quadro para a ordem, e o Quadro do Kanban para o `KanbanView`.
7. **Pessoas e ficheiros sem depender do Quadro.** `Member(String)` é a mesma identidade de
   `Who::Person` (o nome na Equipa). Imagens e anexos guardam o sha-256 dos bytes, a chave de
   `ph2d_board_model::FileId` (ADR-0184). As cores são CHAVES (`Hue` × `Shade`, as 30 do Trello),
   nunca RGB: a paleta vem dos tokens.
8. **As datas são um instante neutro** (`Moment`, ms desde a época, UTC) no documento. O que «hoje»,
   «atrasado» e «esta semana» significam, os fusos e a recorrência vivem em `ph2d-kanban-dates`
   sobre **`jiff` 0.2** (BurntSushi, `Unlicense OR MIT`, as duas na lista do `deny.toml`). Escolhido
   sobre o `time` 0.3 (já transitivo, `MIT OR Apache-2.0`) porque o `jiff` traz a base de fusos IANA
   e a aritmética civil do Temporal: «mesma hora amanhã» atravessa a mudança de horário sem erro, e
   o `time` deixa isso a quem chama. O `stack-audit --tetos` não o segura (crate nova, sem pais no
   lock).
9. **A descrição é markdown** (CommonMark), lida e escrita por `ph2d-kanban-md` sobre
   **`pulldown-cmark` 0.13** (`MIT`) — o mesmo formato que o Trello exporta (W10 importa-o sem
   tradução). O `RichText` do Quadro é a vista do editor, não o formato gravado.

## Alternativas recusadas

| Alternativa | Por que não |
|---|---|
| O Kanban como **elementos soltos** do Quadro (lista = frame, cartão = nota) | não existe sem o canvas; abrir em aba duplicaria; Tabela/Calendário impossíveis; a ordem e o arrasto do Trello não cabem em geometria livre |
| Blob do Kanban **dentro** do elemento do Quadro | dois quadros com o mesmo Kanban seriam duas cópias que divergem |
| Ficheiro **fora** do projeto | quebra o projeto como unidade (desfazer, salvar, partilhar); fica como exportação |
| CRDT externo (automerge/yrs) | o repo já tem réplica por campo provada em LAN e internet; duas leis de fusão = duas classes de defeito |
| Desfazer no **`ProjectState`** (o handoff da W0) | restaura por fotografia: apaga o trabalho de um colega ao vivo e guarda o documento inteiro por passo (§5) |
| Kanban **depende de `ph2d-board-model`** (para `FracKey`, `Who`, `FileId`) | ciclo na W4 (`ElementKind::Kanban` precisa do `KanbanView`); só a ordem tinha lei a partilhar, e saiu para `ph2d-frac` |
| Comentários e checklists como `Vec` **dentro** do `Card` | dois colegas no mesmo cartão escreveriam o mesmo registo, e a fusão por registo perderia um deles |

## O preço

- O desfazer da W1 compara **por registo**: quando outro mexeu num registo depois do gesto, o passo
  deixa esse registo inteiro como o outro o deixou. O Quadro compara campo a campo. Antes da W5 não
  há outro escritor, então não muda nada para o artista. A W5 passa a regra a campo a campo, junto
  com a réplica.
- `PROTOCOL` 11 → 12 e a variante nova de `ElementKind` ficam para as W4–W5.
- Mais uma crate no grafo (`ph2d-frac`, ~230 linhas, só `serde`).

## Gates

`ph2d-kanban-model`: `the_bytes_round_trip_every_record_kind` · `a_newer_format_is_refused_by_name` ·
`every_op_returns_the_op_that_undoes_it` · `a_delete_is_a_tombstone_and_its_inverse_revives_it` ·
`reordering_writes_one_record_and_renumbers_nobody` · `undo_leaves_a_record_someone_else_changed_since` ·
⭐ `both_views_show_the_same_state_after_every_op` · `collapsing_a_list_is_the_views_and_never_the_documents`.
Shell: `project_kanbans_tests` (ida-e-volta; v193/v192/v190 abrem sem Kanbans).
