# Continuação — W1 (modelo + projeto) e W1b (maquete) → W2 (a aba) · 2026-10-27

- **Linha:** `line/TrelloClone` · worktree `Worktrees/line-TrelloClone/` · base `c3662cd8e4` (= main; o rebase foi no-op).
- **Commits (`git log c3662cd8e4..HEAD`):** `86ab320490` modelo + `ph2d-frac` · `da655e8075` projeto, schema 194 · `a09c623648` ADR-0195 · `bf893b2729` maquete · `ed45aff877` clippy (`Put` em caixa).
- **Plano:** [`../01_plano.md`](../01_plano.md) · **W0→W1:** [`HANDOFF_CONTINUACAO_W0_para_W1_2026-10-27.md`](HANDOFF_CONTINUACAO_W0_para_W1_2026-10-27.md) · **ADR:** [0195](../../architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md).
- **Smoke desta onda:** só a maquete, APROVADA pelo dono em 27/10 («Aprovado»). Sem binário de app: a W1 não tem tela.

## W1 feita

### `crates/ph2d-frac` (NOVA)
- `FracKey` movido por `git mv` de `ph2d-board-model/src/frac.rs` (`frac_tests.rs` → `tests.rs`). O `board-model` reexporta `pub use ph2d_frac::FracKey`: caminho público e bytes iguais.
- Motivo: evitar ciclo na W4 (`ElementKind::Kanban` precisa de `KanbanView`, que mora no `kanban-model`).

### `crates/ph2d-kanban-model` (NOVA)
- `KanbanSet`: `FORMAT_VERSION = 1`, bytes postcard `(version, set)`. Vazio = sem kanbans. Versão desconhecida recusa por nome.
- `Kanban` = `Header` + 8 tabelas `BTreeMap` (lists, cards, labels, checklists, items, comments, attachments, fields). `next_id` é um espaço por kanban; o bit 63 fica reservado aos ids de sessão da W5.
- `Meta { version, nonce, deleted }`.
- `KanbanOp { Put(Box<Record>), Delete(RecordId), Header }`, com inverso (construtor `KanbanOp::put`). Mesmo conteúdo = `None`. A lápide conta como conteúdo: defeito achado e corrigido na W1.
- `History` por kanban: lote inverso, hora `at` passada por quem chama (= `undo_tick`), rebase POR REGISTO.
- `KanbanView { kanban, collapsed, scroll_x, scroll_y }` + `shown()` (projecção pura). Estado de vista ≠ documento.
- Cores = chaves `Hue × Shade` (30). `Moment(i64 ms UTC)`. `Member(String)` = identidade de `Who::Person`. Ficheiros por sha-256 `[u8; 32]`.

### Projeto (`.ph2dproj`)
- `ProjectFile.kanbans: Vec<u8>`, ÚLTIMO campo. `PROJECT_SCHEMA` 193 → 194: doc + const em `project_schema.rs`; tripla irmã em `project_schema_tests.rs` = `(194, 13, 23)`.
- `project_load_read.rs`: o braço `193 | 192 | 191` acrescenta 1 ou 2 zeros; o braço 190/189 passa a `[0, 0]` (+1 do `read_before_scripts`). Migrações v128/v95 com `kanbans: Vec::new()`.
- Save recusa se não serializa. Load recusa com i18n `shell.project_load.project_refused_its_kanbans` (`crates/ph2d-i18n/src/shell_media.rs`).
- Dono em runtime: `ph2d_editor_core::documents::Documents { kanbans: KanbanSet }` com `kanbans()` / `replace_kanbans()`. O `editor-core` reexporta `KanbanSet`.
- Teste novo: `shells/desktop/src/project_kanbans_tests.rs` (filho de `project_tests.rs`).

### Shell
- +116 linhas de composição (`TETO_LOC` 189 041 + `FOLGA_DE_COMPOSICAO` 4 000). O integrador reconta; não confie neste número.

### Datas e markdown (decisão no ADR; AINDA NÃO adicionadas)
- Datas: `jiff` 0.2 (Unlicense OR MIT). Markdown: `pulldown-cmark` 0.13 (MIT). Ambas permitidas no `deny.toml`. Entram com `ph2d-kanban-dates` / `ph2d-kanban-md` na W3.

## Premissas derrubadas (não repita)
1. «`ProjectState.kanbans` no desfazer» (W0 §4): errado. `ProjectState` restaura por fotografia e apagaria a edição de um colega ao vivo. O desfazer é POR KANBAN, como o Quadro.
2. «ADR-0192 é o próximo livre»: era 0195. O ADR é PROVISÓRIO: 0192 está em `line/PixelArt` e `line/BlenderConstraints`; 0193/0194 em `line/BlenderConstraints`. O integrador renumera e regenera o índice (`scripts/adr-index.sh`).
3. «Folga zero na shell» (explorador): há `FOLGA_DE_COMPOSICAO` 4 000.

## W1b — a maquete (aprovada)
- Tela publicada: <https://claude.ai/artifact/Bt2sphR8SBYHomJw4JEBsA>.
- Fonte versionada: `docs/TrelloClone/maquete/project/*.dc.html` + `canvas.json`. Telas: `Main` (quadro escuro), `BoardLight`, `Cards` / `CardsLight` (15 variantes da frente do cartão), `Card` (componente), `Back` (verso escuro), `Filter` (claro), `Canvas` (3 zooms), `Palette`.
- Cada smoke da W2 em diante é fotografado LADO A LADO com a maquete.

### Valores a portar para `ph2d-tokens` na W2 (append-only; zero hex no código)
| Grupo | Valor |
|---|---|
| Etiquetas | 30 cores, claro/escuro (tabela `PAL` em `Palette.dc.html`) |
| Tinta da etiqueta | claro `#091E42`, branco no forte · escuro `#DEE4EA`, `#1D2125` no forte |
| Superfícies | lista `#f1f2f4` / `#111520` · cartão `#ffffff` / `#22293a` · fundo Azul `#2c63c9` / `#1b3a6e` |
| Sombras | repouso · hover · voo |
| Cantos | etiqueta 4 · cartão 8 · lista 12 · verso 12 |
| Movimento | hover 150 ms · pegar inclina 4° em 150 ms · largar e abrir o espaço 240 ms (mola) · recolher 240 ms · check 150 ms |
| Larguras | lista 272 · cartão 256 |

⚠️ O contraste é ESTIMADO, não medido. Meça com o gate de contraste do `ph2d-tokens` na W2 antes de aceitar qualquer par.

## Prova de fecho (verdes, por nome)
- `ph2d-kanban-model` 21 · `ph2d-frac` 9 · `ph2d-board-model` 132 · shell `kanbans` + `marcas` 4 (`project::tests::kanbans::os_kanbans_atravessam_o_arquivo`, `um_v193_um_v192_e_um_v190_abrem_sem_kanbans`).
- Gates ⭐ no modelo: `both_views_show_the_same_state_after_every_op` (duas vistas, um documento) e `collapsing_a_list_is_the_views_and_never_the_documents`.
- Clippy `-D warnings` verde em `ph2d-frac`, `ph2d-kanban-model`, `ph2d-board-model`, `ph2d-i18n`, `ph2d-editor-core`, `ph2d-host-desktop`. `cargo fmt --all` limpo.
- A suíte completa foi RECUSADA pela cadência (esperado; é do integrador).

### O que um merge pode partir
- `PROJECT_SCHEMA`: outra linha também sobe o degrau. Reconte por `python3 scripts/schema-recount.py` (três sítios + o braço `193 | 192 | 191` do load, que passa a `N | …`).
- Número do ADR (acima) e links para ele neste handoff, no plano e no README.
- `the_shell_only_shrinks`: o teto soma entre linhas.
- `ph2d-board-model`: toca-o quem mexer em `frac`; o `FracKey` já não mora lá.
- Registo de crates (`Cargo.toml` do workspace) e `Cargo.lock`: duas crates novas.

## ABERTO
- Desfazer compara por REGISTO; o Quadro compara por CAMPO. Passa a campo na W5, com a réplica.
- `History` ainda NÃO tem dono em runtime. Entra na W2 com a aba: `BTreeMap<KanbanId, History>` ao lado de `Documents.kanbans`, como `live.histories` dos quadros.
- `PROTOCOL` 11 → 12 e `ElementKind::Kanban` (variante no FIM) ficam para W4–W5.
- Datas e markdown: crates e dependências na W3.

## A tarefa da W2 (plano §6)
1. Abrir um Kanban numa ABA: `DocumentSurface` em `crates/ph2d-app-kanban` (NOVA). Precedente: `ph2d-app-code/src/docs.rs:42` + `shells/desktop/src/init.rs:374`; trait em `ph2d-editor-core/src/doc_surface.rs:170`. Confira os números de linha antes de citá-los: podem ter andado.
2. Criar listas e cartões; arrastar com a sensação do Trello (mola, inclinação, rolagem nas bordas); cores e recolher listas; arquivo; atalhos de navegação.
3. Tokens do Kanban em `ph2d-tokens` (tabela acima); `History` por kanban em runtime.
4. Roteador de smoke próprio `PH2D_KANBAN_SMOKE=1..` (número da próxima cena contado no roteador da família); seam tests com `ph2d-ui-testkit`.
5. MEDIR o teto de cartões a 60 fps (§0.0) e escrever a tabela ao lado do número; não escolha um `MAX_*`.
6. Shell só composição; código de família na crate. Janela nova, comece por `cd` + `pwd` + `git branch --show-current`.

## Linha do §5 do CLAUDE.md
Ainda NÃO existe entrada para o Kanban no §5.1 (só handoffs de continuação, sem integração). A entrada (≤ 700 bytes) nasce no fecho da linha, com o último handoff de integração.
