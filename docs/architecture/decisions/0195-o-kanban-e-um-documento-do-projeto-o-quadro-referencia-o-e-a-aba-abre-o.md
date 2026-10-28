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

1. **Um documento de primeira classe.** `ph2d-kanban-model`: `KanbanSet` → `Kanban` (cabeçalho + tabelas
   `BTreeMap`: listas, cartões, etiquetas, checklists, itens, comentários, anexos, campos, espelhos,
   automações, atividade, eventos). Muda-se só por `KanbanOp` (`Put` · `Delete` · `Header`), que devolve o
   inverso; gestos compostos saem como `Plan`. A crate é pura (deps: `ph2d-frac`, `sha2`, `serde`/`postcard`).
2. **No projecto, um blob irmão do `boards`.** `ProjectFile.kanbans: Vec<u8>`, o último campo
   (`PROJECT_SCHEMA 193 → 194`, `shells/desktop/src/project_schema.rs`), com versão PRÓPRIA:
   `ph2d_kanban_model::FORMAT_VERSION = 6` (`set.rs`). Escada: v1 base · v2 seguir/atividade/origem/ficheiros ·
   v3 modelos, envelhecimento, espelhos · v4 automações · v5 agenda do projecto · v6 ligações do cartão ao
   projecto (objecto/script/macro/timeline). `from_bytes` lê cada degrau antigo por `v1.rs`..`v5.rs`
   (`upgrade()`); versão maior recusa por nome. Fixtures congeladas `tests/fixtures/kanban_set_v1..v5_full.hex`
   (gravadas pelo código do degrau, cabeçalho `#`); ⚠️ **v6 ainda sem fixture própria** (cobertura em
   `links_tests.rs`). Mudança interna sobe a versão dele, nunca o schema. Em execução o dono é
   `ph2d_editor_core::documents::Documents::kanbans` (`SharedKanbans`, o mesmo `Rc` da aba e do elemento).
3. **Duas vistas, um documento.** A aba e o elemento do Quadro têm cada um o estado SÓ da vista
   (`KanbanView` na aba; `KanbanBoardView` no elemento: `scroll_x`, `zoom`, `collapsed`, `scroll_y` por
   lista). `KanbanView::shown(&KanbanSet)` é projecção pura: não há sincronização entre janelas, as duas
   leem o mesmo registo (gate `both_views_show_the_same_state_after_every_op`; `a aba e o canvas do MESMO
   Kanban` em `ph2d-app-kanban/tests/it`).
4. **Um registo por coisa.** Cada registo tem `Meta { version, nonce, deleted }` (molde do `Element`,
   ADR-0181). Comentários, checklists, itens e anexos são registos próprios apontando ao pai, nunca `Vec`
   dentro do cartão. Ordem por `FracKey`. Ids: um espaço por Kanban; em sessão ao vivo
   `Kanban::set_session(Some(semente))` passa a sortear com o bit 63 (porta única `mint`/`mint_n`; o
   `Copier` também passa pelo `Minter`).
5. **Desfazer POR KANBAN, fora do `ProjectState`.** `History` guarda o lote INVERSO e compara **por campo**
   (`Record::fields`, `history_fields.rs`): desfazer não pisa um campo que outro mudou depois; o 2.º desfazer
   seguido no mesmo cartão funciona (`History::rebase` × `meta.version`). `undo_logged`/`redo_logged` põem o
   desfazer na atividade (ele nunca a apaga). A hora é a do `undo_tick` único, passada por quem chama. O
   Planner tem a fila dele (a aba do Kanban não o desfaz).
6. **`FracKey` em `ph2d-frac`** (movido do `ph2d-board-model`, que o reexporta; bytes e caminho público
   iguais). Evita o ciclo `board-model ↔ kanban-model`.
7. **Pessoas e ficheiros PRÓPRIOS, sem depender do Quadro.** `Member(String)` = o nome da Equipa
   (`Who::Person`); o Butler é `Member::butler`. Ficheiros: `FileId` (sha-256 dos bytes, os mesmos 32 bytes
   do `ph2d_board_model::FileId`, mas tipo PRÓPRIO em `files.rs`), `KanbanFile`, guardados UMA vez por
   conteúdo no `KanbanSet` (`put_file`, e `put_file_keyed` quando a chave já foi calculada noutro fio);
   `to_bytes` leva só os que algum registo vivo usa (`files_in_use`). Sem tecto de tamanho por anexo (o
   recurso é a RAM, ~3× ao gravar); até 8 MB lê-se na hora, acima num fio (MEDICOES §W3 E). Cores =
   CHAVES (`Hue` × `Shade`, 30 do Trello), nunca RGB.
8. **Agenda do projecto.** `KanbanId::AGENDA` (= 0; os Kanbans contam de 1) é um `Kanban` reservado dentro
   do `KanbanSet`, fora de `iter()`: eventos próprios, Inbox, blocos do Planner e «lidas». Eventos viajam na
   réplica e os calendários juntam-se numa sessão; Inbox, blocos e lidas são pessoais e NÃO viajam.
9. **Datas:** `Moment` (ms UTC) no documento; «hoje/atrasado/semana», fusos e recorrência em
   `ph2d-kanban-dates` sobre **`jiff` 0.2** (`Unlicense OR MIT`; base IANA e aritmética civil: «mesma hora
   amanhã» atravessa a mudança de horário). API sem tipos do `jiff`; textos saem em enums para a i18n.
10. **Descrição = markdown** (CommonMark + tarefas + riscado) por `ph2d-kanban-md` sobre **`pulldown-cmark`
    0.13** ↔ `RichDoc` de `ph2d-richdoc` (zero deps). `render(parse(md))` é ponto fixo; o que o modelo não
    tem (tabelas, HTML, imagens, títulos 4–6→3) vira texto literal; menção com espaço =
    `[@Ana Silva](mention:Ana%20Silva)`.

## O elemento `ElementKind::Kanban` no Quadro

- `ElementKind::Kanban { kanban: KanbanId, view: KanbanBoardView }`, variante no FIM (postcard posicional).
  Quadro `FORMAT_VERSION` **19** (18: a variante; 19: `scroll_y`; leitores `KanbanBoardViewV18/V19` por
  `reading(version)`); `ph2d_board_sync::session::PROTOCOL` **13** (12: a variante; 13: a vista mudou de
  bytes). `ph2d-board-model → ph2d-kanban-model` é acíclico.
- O Quadro não depende da família Kanban: pintura e entrada passam por GANCHOS, molde do vídeo.
  `ph2d_board_render::kanban::KanbanPainter` (`paint` → `false` = o board-render pinta o nível baixo a
  partir do `KanbanSet`; `paint_over` = o verso aberto, em ecrã) e `ph2d_editor_core::…::board_kanban::KanbanElementInput`
  (rato, teclas, roda, IME, cursor, `wants_frame`). A família implementa-os (`ph2d-app-kanban`), `ph2d-app-board`/shell compõem.
  A aba é UM chamador do mesmo `paint_view`/`pointer_view` sobre uma `Affine` (escala 1), o elemento outro.
- **LOD (limiares MEDIDOS, MEDICOES §W4 V)**: Alto ≥ escala **0,786** (texto do cartão de 14 px chega a
  `TypeToken::Xs` = 11 px; recurso = legibilidade de quem edita — Alto custa o mesmo que Médio, ~4,3 ms);
  Médio ≥ **0,4** (título de lista 15 px × escala ≥ `READ_PX` = 6; o Baixo poupa ~1,1 ms por Kanban
  grande); Baixo = blocos por lista + contagem, nome nunca abaixo de 11 px no ecrã. `force_kanban_lod`
  só para medir.
- Gestos: «Kanban» na barra do Quadro; «Open in tab» na barra do elemento; na aba «Show on board» /
  «Place on board» (Quadro activo → último activo → primeiro → cria). Apagar a referência não apaga o
  documento. Entra-se no Kanban do elemento com 2.º clique, no zoom alto.

## A réplica — `ph2d-kanban-sync`

Pura, como `ph2d-board-sync`, com a MESMA lei de fusão (`ph2d_board_sync::wins`; duas leis seriam duas classes
de defeito). Cada registo divide-se em campos (`Record::fields`, a divisão do desfazer), cada campo tem carimbo
de Lamport; apagar é lápide e vence-se como no Quadro (apagar × editar: ficam a lápide E a edição). Listas de
pessoas/etiquetas (membros, votos, quem segue) e campos personalizados do cartão são CONJUNTOS: um registo por
elemento, ordem de entrada. Ficheiros viajam uma vez por conteúdo. O transporte é o do Quadro
(`ph2d-board-net`/`-internet`), por quem compõe. **Retrato de quem entra:** carimbos compactos e SEM perda
(`snapshot::pack/unpack`, 5,2× menores; retrato 2,6×) em blocos de 1 MiB (`SNAPSHOT_BLOCK`, `KMsg::SnapshotPart`);
escritas a meio dos blocos ficam guardadas e aplicam-se depois (anfitrião 38 ms / convidado ~47 ms com 20 000
cartões nascidos na sessão; MEDICOES §W5 S2). Provado: LAN e QUIC em loopback convergem. UI de Share/avatares
e o retrato do anfitrião num fio ficam por fazer.

## Mapa das crates (sem ciclos)

```
ph2d-frac ← ph2d-kanban-model ← ph2d-board-model ← ph2d-board-{render,sync,…}
ph2d-richdoc ← ph2d-kanban-md                      ph2d-kanban-dates (jiff; folha)
kanban-model + dates + md → ph2d-kanban-filter → ph2d-kanban-auto → ph2d-kanban-script (+ ph2d-script)
kanban-model → ph2d-kanban-import                  kanban-model + ph2d-board-sync → ph2d-kanban-sync
dates+filter+model+editor-core → ph2d-kanban-views → ph2d-app-planner (+ ph2d-app-kanban)
model+md+richdoc+dates+filter+board-model+board-render → ph2d-app-kanban → ph2d-app-board → shell
```

`ph2d-kanban-model` depende só de `ph2d-frac`; `ph2d-board-model`/`-render` dependem dele, nunca ao
contrário. Puras (sem UI): richdoc, md, dates, filter, import, auto, sync, script (+ `ph2d-script`).

## Automação — `ph2d-kanban-auto`

O Butler: regras, botões de cartão e de quadro, comandos de calendário e de prazo, executor determinístico
(`Seen::plan` → aplicar → `react` → aplicar o `Outcome::plan` com o autor `Member::butler`), agenda, frase para a
i18n e sugestões. Puro: «agora», fuso e autor vêm de quem chama. **Anti-laço:** uma regra nunca dispara por
acontecimentos da sua PRÓPRIA cadeia (A→B→A pára em A; gate `a_ping_pong_pair_stops_when_a_rule_meets_its_own_chain`,
mutação medida). Limites de SEGURANÇA, não de hardware: `MAX_DEPTH = 16` (cenário real mais fundo: 2),
`MAX_RUNS = 1000` por gesto (= 8 ms medidos < 1 quadro); parar regista `Stop::Depth`. `react` não corre ao
desfazer; `run_scheduled` fica fora do desfazer; calendário perdido corre UMA vez ao abrir, prazo uma vez por
cartão; `SCAN_DAYS = 367` não é tecto. Sugestões: `SUGGEST_MIN = 3`, janela de 5 min. (MEDICOES §W8 U.)

## Scripts — `ph2d-kanban-script`

`ph2d.kanban.*` (ADR-0188: o script mora no projecto). Linhas da API, tipos do `ph2d.d.luau` e ajuda na fonte
única `ph2d_script::api_table_kanban`/`api_help_kanban`; a fiação e `KanbanScripts` (`run`, `react`) aqui. Escreve
por `apply_logged` com autor «script: <nome>», UM passo de desfazer por corrida, numa cópia (um script falhado
não deixa nada), mesmos acontecimentos e mesmo anti-laço do Butler. snake_case (`add_card`, `press_button`);
etiqueta desconhecida é erro que lista as existentes. ⚠️ Por ligar: o `app-kanban` chamar `run`/`react` e o
Butler após cada gesto.

## Import/export e vistas

`ph2d-kanban-import`: JSON do Trello ↔ `Kanban` (`import_trello`, `export_trello_json`), CSV, markdown; pessoa =
`username`; anexos do PH2D saem como `ph2d://`. `ph2d-kanban-views`: Tabela, Calendário (mês/semana/dia/agenda),
Linha do tempo, Painel; estado da VISTA, gestos voltam em `Reply::plan` que o chamador aplica; a `Source`
genérica do calendário serve o Kanban (`KanbanSource`) e o projecto (`ProjectSource`, `ph2d-app-planner`).

## Alternativas recusadas

| Alternativa | Por que não |
|---|---|
| O Kanban como **elementos soltos** do Quadro (lista = frame, cartão = nota) | não existe sem o canvas; abrir em aba duplicaria; Tabela/Calendário impossíveis; a ordem e o arrasto do Trello não cabem em geometria livre |
| Blob do Kanban **dentro** do elemento do Quadro | dois quadros com o mesmo Kanban seriam duas cópias que divergem |
| Ficheiro **fora** do projeto | quebra o projeto como unidade (desfazer, salvar, partilhar); fica como exportação |
| CRDT externo (automerge/yrs) | o repo já tem réplica por campo provada em LAN e internet; duas leis de fusão = duas classes de defeito |
| Desfazer no **`ProjectState`** (o handoff da W0) | restaura por fotografia: apaga o trabalho de um colega ao vivo e guarda o documento inteiro por passo (§5) |
| Kanban **depende de `ph2d-board-model`** (para `FracKey`, `Who`, `FileId`) | ciclo na W4 (`ElementKind::Kanban` precisa do `KanbanBoardView`); a ordem saiu para `ph2d-frac`, pessoas e ficheiros são próprios |
| Quadro a pintar o Kanban directamente | `board-render`/`editor-core` ← família Kanban seria ciclo; ganchos `KanbanPainter`/`KanbanElementInput` (molde do vídeo) |
| Deitar fora carimbos iguais ao do nascimento no retrato | menor, mas pede uma prova de fusão a mais; o sem-perda não pede nenhuma |
| `time` 0.3 em vez de `jiff` | deixa a quem chama a mudança de horário e não traz a base de fusos |
| Kanban **depende de `ph2d-board-model`** só para `FileId` | o `FileId` é PRÓPRIO (mesma chave sha-256); o ciclo já custou o `FracKey` |
| Comentários e checklists como `Vec` **dentro** do `Card` | dois colegas no mesmo cartão escreveriam o mesmo registo, e a fusão por registo perderia um deles |

## O preço

- O desfazer compara por campo (como o Quadro) desde o lote F; antes disso era por registo.
- `PROTOCOL` 13 e Quadro `FORMAT_VERSION` 19: quem integrar reconta contra o `main` (§5.0).
- Mais crates no grafo: `ph2d-frac` e as dez desta linha; deps novas `jiff` 0.2, `pulldown-cmark` 0.13.
- Aberto: fixture v6; UI de Share/avatares; scripts/Butler não ligados ao `app-kanban`; células CSV sem
  protecção contra fórmulas (paridade com o Trello; ver `DECISOES_SEM_O_DONO.md`).

## Gates

`ph2d-kanban-model`: `the_bytes_round_trip_every_record_kind` · `a_newer_format_is_refused_by_name` ·
`every_op_returns_the_op_that_undoes_it` · `a_delete_is_a_tombstone_and_its_inverse_revives_it` ·
`reordering_writes_one_record_and_renumbers_nobody` · `undo_leaves_a_record_someone_else_changed_since` ·
⭐ `both_views_show_the_same_state_after_every_op` · `collapsing_a_list_is_the_views_and_never_the_documents`.
Família: `ph2d_app_kanban::project::tests` (ida-e-volta do blob, vazio = sem Kanbans, blob ilegível recusa). Shell: `project_marcas_tests` (o campo atravessa o `.ph2dproj`; v193/v192/v191 abrem sem Kanbans; v189/v190 pelos testes dos scripts).
