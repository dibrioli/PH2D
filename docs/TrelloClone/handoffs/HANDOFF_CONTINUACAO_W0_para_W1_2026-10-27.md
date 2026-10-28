# Continuação — W0 (pesquisa e plano) → W1 (fundação do modelo) · 2026-10-27

- **Linha:** `line/TrelloClone` · worktree `Worktrees/line-TrelloClone/` · base `c3662cd8e4`.
- **W0 fechada:** só docs — [`../01_plano.md`](../01_plano.md) (inventário do Trello, decisão
  central, aparência, 11 ondas) e as decisões do dono no §8 (todas fechadas). Sem smoke (era plano).
- **Memória:** `project-memory/project_kanban_trello_clone_owner_decisions_2026_10_27.md`.

## Decisões fechadas (não re-litigar)
- UM documento Kanban do projecto (`KanbanId`, `KanbanSet` em blob próprio no `.ph2dproj`); a aba
  (`DocumentSurface` em `ph2d-app-kanban`) e o elemento do canvas (`ElementKind::Kanban`, variante
  no FIM, só id + estado de vista) referenciam o mesmo recurso. Estado de vista ≠ documento.
- Colaboração pela réplica por campo do Quadro (ADR-0181), ordem por `FracKey`.
- Sem Mapa · IA no fim · Google/Microsoft fora · **Calendário do projecto interno** na W9 (§8.1).
- Toda tela passa pela maquete aprovada (W1b) — frente «bela aparência» (§5).

## A tarefa da W1 (plano §6)
1. **ADR-0192** (próximo livre conferido: 0191 é o último — ⚠️ derrubado na W1: escrito como 0195) — a decisão do §3 com as recusas do §3.5.
2. `crates/ph2d-kanban-model`: tipos do §4 (Kanban, List, Card, Label 30 cores por token, Checklist/
   CheckItem, Comment, Attachment por conteúdo ADR-0184, CustomField, datas como valor neutro até a
   crate de datas), `KanbanOp`, `FracKey` (reutilizar `ph2d-board-model/src/frac.rs` — mover para
   crate partilhada se preciso, nunca copiar), bytes com versão própria + testes de ida-e-volta.
3. Datas: decidir `jiff` vs alternativa por `bash scripts/stack-audit.sh --tetos` + licença do
   artefacto; markdown: `pulldown-cmark` idem. Anotar a decisão no ADR.
4. Ligar ao projecto: blob irmão do `boards` (`project_save.rs:67` / `project_load.rs:88`;
   `PROJECT_SCHEMA = 193` sobe 1 degrau para o campo novo no fim do `ProjectFile`, três sítios —
   contar por `python3 scripts/schema-recount.py`), e `ProjectState.kanbans: Arc<…>` no desfazer
   (`undo.rs:134/207/247`). Shell só composição (`the_shell_only_shrinks`).
5. Medir se duas vistas podem ler o mesmo recurso (não verificado na W0) e escrever o gate
   «aba ↔ canvas mostram o mesmo estado» já no modelo (duas vistas, um documento).
6. Em seguida (mesma janela se couber, senão a próxima): **W1b — a maquete** para o dono aprovar.

## Endereços medidos na W0
`doc_surface.rs:170` (trait da aba) · `ph2d-app-code/src/docs.rs:42` + `init.rs:374` (precedente) ·
`element.rs:63` (`ElementKind`) · `board.rs:23` (`FORMAT_VERSION = 17`) · `session.rs:23`
(`PROTOCOL = 11`) · `smoke.rs:110` (roteador do Quadro, 29) · widgets em `ph2d-editor-core/src/widget/`
(faltam calendário, seletor de data, editor rico) · i18n só `Ingles`+`Teste`.
