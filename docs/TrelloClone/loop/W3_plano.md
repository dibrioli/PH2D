# W3 — o verso do cartão (plano do coordenador, 28/10)

Alvo: o modal do Trello completo (plano §1.2): título · lista onde está · seguir · descrição rica
(WYSIWYG, guarda markdown) · etiquetas · membros · datas (início, entrega com hora, lembrete,
concluída) · checklists (vários; item com responsável e data; esconder marcados; item → cartão) ·
anexos (ficheiro/link/cartão; definir como capa) · capa (cor/imagem, metade/inteira) · campos
personalizados (texto, número, data, caixa, lista; mostrar na frente) · localização · atividade
(comentários com @menção, reacções, editar/apagar; histórico intercalado) · ações (mover, copiar,
arquivar, apagar, seguir, votar, partilhar link). A frente ganha capas de imagem, datas com cor de
estado e campos «na frente».

## Decisões técnicas
- **`ph2d-richdoc`** (crate nova, zero deps): documento rico em BLOCOS (parágrafo, título 1–3, lista,
  lista numerada, checklist, citação, código, régua) com marcas inline (negrito, itálico, riscado,
  código, link, menção); operações de edição puras com selecção (inserir, apagar, alternar marca,
  mudar bloco, atalhos markdown ao escrever: `# `, `- `, `1. `, `[] `, `> `, ``` ``` ```, `**x**`).
  `RichText` do Quadro é só inline e o formato dele é gravado — não se estende.
- **`ph2d-kanban-md`**: markdown (CommonMark + tarefas + riscado, `pulldown-cmark` 0.13) ↔ `RichDoc`;
  ida-e-volta estável (md → doc → md é ponto fixo após a 1.ª normalização).
- **`ph2d-kanban-dates`** sobre `jiff` 0.2: estado de prazo (atrasado/hoje/amanhã/semana/sem data/
  concluído) com a fronteira à meia-noite LOCAL, texto curto do distintivo, lembrete, grelha do mês
  (`CalendarMonth`), início da semana por locale (segunda por omissão).
- **Widgets no `ph2d-editor-core`** (com showcase, ≤ 500 LOC cada): `calendar` (grelha de mês dada
  pelo chamador — o widget não faz contas de datas), `date_picker` (calendário + hora + lembrete +
  intervalo início→entrega), `rich_editor` (pinta e edita um `RichDoc`, barra de formatação).
- **Modelo v2** (`FORMAT_VERSION` 1→2 com degrau de leitura v1): `Card.watchers`, registo `Activity`
  (quem, quando, o quê — gerado pelas `KanbanOp`), `CheckItem` → cartão convertido; ficheiros por
  conteúdo do Kanban (`files: BTreeMap<FileId, ImageFile>` reutilizando `ph2d-board-model::image`).
- Membros: a Equipa vive no `Board`; o Kanban lê a união das equipas dos quadros do projecto + os
  `Member` já usados nos cartões (sem quadro: só estes e «eu»).

## Lotes (partição por caminho)
| Lote | Possui | Depende de |
|---|---|---|
| A | `crates/ph2d-richdoc/**`, `crates/ph2d-kanban-md/**`, `crates/ph2d-kanban-dates/**`, deps `jiff`/`pulldown-cmark` (Cargo.toml raiz só essas linhas + `Cargo.lock`) | — |
| B | `crates/ph2d-editor-core/src/widget/{calendar,date_picker,rich_editor}*`, showcase, `widget/mod.rs`, `crates/ph2d-editor-core/Cargo.toml` (dep `ph2d-richdoc`) | A (richdoc) para o rich_editor |
| C | `crates/ph2d-kanban-model/**` (v2) | — (depois da W2b) |
| D | `crates/ph2d-app-kanban/src/back/**` + frente (capas, datas, campos) + i18n/tokens | A, B, C, W2b |

Ordem: W2b ∥ A ∥ B → C → D (D partido em 2 agentes por ficheiro se couber) → verificador → revisor → fecho.
