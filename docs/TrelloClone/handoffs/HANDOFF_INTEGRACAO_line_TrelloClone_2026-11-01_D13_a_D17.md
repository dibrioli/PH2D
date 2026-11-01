# Handoff de INTEGRAÇÃO — `line/TrelloClone` D13 → D17 + auditoria visual (01/11/2026)

> Para o agente INTEGRADOR (outra janela, integração em lote — `scripts/integrar-rodada.sh`).
> O smoke do dono passou (01/11: «smoke OK»). Integrar e shipar só por ordem do Enio.
> Anterior: [W2b–W11, no main 29/10](HANDOFF_INTEGRACAO_line_TrelloClone_2026-10-28_W2b_a_W11.md).

## 1. Base e HEAD

- Worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-TrelloClone`, ramo `line/TrelloClone`.
- Merge-base = `422fec2027` (o main no momento do fecho, ainda não andou).
- HEAD de código antes do fecho = `0cd4a0fcf9` (33 commits, 199 ficheiros, +22 182 / −1 239). O commit de fecho junta as curas do gate (§8), este handoff e a §5 do CLAUDE.md.
- A árvore está limpa, salvo `project-memory/*`, que são do dono e não desta linha: o integrador não as comite.

## 2. Commits por onda

| Onda | O quê | Commits |
|---|---|---|
| D13 | «Use as Kanban background…» no botão direito da Hierarquia; «Photos» do projecto revertido (ver §9) | `c0abf4ce4e`, `a49dec73c0` (revert), `fa4165be1c` |
| D14 | Campos personalizados: lista suspensa a sério, calendário no campo de data, nomes sem corte; etiqueta +1 px | `d0ef87108a`, `ddf0e70e63`, `1d6615a4a2` |
| D15 | Ícone de calendário; «…» do Kanban no Quadro ao 1.º clique; barras de rolagem; **crate nova `ph2d-kanban-live`** (tecto de LOC) | `7acbc585b7`, `78e596668b` |
| D16 | A barra da lista como a do Trello: margem própria, cartões estreitos, setas na `AxisBar` | `9728dcb96c`, `df700210bc` |
| W10 | O quadro REAL do dono vira fixture e gate, anonimizado (`trello_board_real_anon.json`, `kanban-import/tests/it/real.rs`) | `799cdb1f21` |
| D17 | Ver a lista abaixo | `4d383b99ed` … `0cd4a0fcf9` |

O que a D17 trouxe:

- Desfazer «inserir Kanban» tira só a referência.
- O prazo usa UMA lei (`datepick::due_text`).
- `HoldRepeat`: segurar a seta repete.
- Kanbans e Quadros aparecem nos Assets: abrir, arrastar para o Quadro, apagar.
- Cena 31 do Quadro.
- Uma imagem dos Assets largada no Quadro.
- `salvage` passa para a `ph2d-board-render`.
- Apagar um Kanban esquece o refazer.
- Uma referência órfã fica INERTE e revive.
- A barra horizontal é igual à da lista.
- A **auditoria visual**: `68d386048a` e as fotos `0cd4a0fcf9`.

## 3. Crates

- **NOVA, 1:** `ph2d-kanban-live`. É o Kanban ao vivo pelo transporte do Quadro (`net_io.rs` + `Seen` MOVIDOS da `ph2d-app-kanban`). A pertença ao workspace vai por glob, sem editar o `Cargo.toml` da raiz.
- **Código movido entre crates, para caber no tecto de LOC da `ph2d-app-kanban` (32 175):**

| O quê | De | Para |
|---|---|---|
| `edit.rs` | `ph2d-app-kanban` | `ph2d-kanban-views/src/edit.rs` |
| As contas da capa | `ph2d-app-kanban` | `ph2d-board-render/src/kanban_cover.rs` (re-export em `front/cover.rs`) |
| `salvage` | `ph2d-app-board` | `ph2d-board-render/src/salvage.rs` |

- **Dependências:**
  - `ph2d-app-kanban` ganha `ph2d-kanban-live`.
  - `ph2d-app-board` ganha `ph2d-panel-registry-init`, `ph2d-panel-asset-browser`, `ph2d-asset-index` e `ph2d-kanban-live`.
  - `png`/`zune-*` passam da `ph2d-app-board` para a `ph2d-board-render`.
  - `Cargo.lock` regenera-se.

## 4. Contratos e formatos — NENHUM mudou

Lidos com `git show <rev>:<ficheiro>` na base e no HEAD:

| Item | Base | HEAD |
|---|---|---|
| `PROJECT_SCHEMA` (`shells/desktop/src/project_schema.rs:535`) | 209 | **209** |
| `KanbanSet` `FORMAT_VERSION` (`ph2d-kanban-model/src/set.rs:21`) | 8 | **8** |
| Quadro `FORMAT_VERSION` (`ph2d-board-model/src/board.rs:28`) | 19 | **19** |
| `PROTOCOL` (`ph2d-board-sync/src/session.rs:28`) | 15 | **15** |
| Contratos §6 (`NodeOp`, `Tool`, `VectorOp`…) | — | intactos |

No diff, as únicas linhas com `PROJECT_SCHEMA` são doc-comment. Nenhum gate `architecture_*_contract_surface` foi tocado.

## 5. Foundational tocado, e porque é ADITIVO

- **`ph2d-asset-index`:**
  - `AssetKind::{Kanban, Board}` e `AssetRef::{Kanban{id}, Board{id}}` entram no FIM dos enums e do `ALL`.
  - O índice é derivado do projecto ao abrir, não é gravado.
  - O catálogo tem versão própria dentro dos bytes (`CATALOG_DOC_VERSION`).
- **`ph2d-board-model`:**
  - `kanban.rs`: `install_kanbans`/`installed_kanbans` (fonte `thread_local`) e `Element::kanban_missing()`. Sem fonte instalada, devolve `false`, ou seja, o comportamento antigo.
  - `element.rs`: o filtro de hit ignora a referência órfã.
  - `history.rs`: o refazer esquece a reinserção de um Kanban apagado.
- **`ph2d-editor-core`:**
  - `widget/scroll_list.rs` é NOVO: `AxisBar`, `BarGrab`, `HoldRepeat`. Não substitui nenhum widget.
  - `widget/date_picker`.
  - Menus dos Assets: `ids/menus_asset.rs`, `pointer_down_menus.rs`, `types_menu.rs`.
  - `drag_payload` + `state/asset_drag_ops.rs`.
  - `documents*.rs`.
  - `screens/hero/board_kanban*.rs` e `send_to_board.rs`.
  - Ícone `calendar` (`docs/design/icons/calendar.svg`, `icons.rs`).
- **`ph2d-spring`:** `follow.rs` novo, um módulo aditivo.
- **Shell (`shells/desktop`):** `asset_drag_wire.rs`, `project_catalogs.rs`, `render_loop/fase_*`, `snapshots_*`. É só composição: a família vive nas crates.
- **i18n:**
  - Ficheiros: `asset_browser`, `board`, `board_images`, `board_kanban`, `chrome_menus`, `chrome_panes`, `kanban`, `kanban_front`, `kanban_import`, `shell`.
  - Só chaves novas, salvo dois textos. Em `kanban_import`, `report_intro` e `export_done` passam a usar aspas curvas, e há uma chave nova `loss_item`.
- **Tokens:** nenhum ficheiro tocado.

## 6. Superfície de colisão (ficheiros que outras linhas também tocam)

Corra `collision-surface.sh` pelo caminho ABSOLUTO. São prováveis:

- `ph2d-editor-core/src/screens/hero/{menu_rows,menu_tables,context_menu_overlay,pre_populate,board_keys,document_tabs_menu}.rs`
- `interaction/state/{mod,store_core,store_hierarchy}.rs`
- `action_bus*.rs`
- `ph2d-i18n/src/{shell,chrome_menus,chrome_panes}.rs`
- `ph2d-panel-asset-browser/src/*`
- `ph2d-app-components/src/asset_*.rs`
- `ph2d-panel-code/src/console.rs` (TAB na consola)
- `ph2d-app-code/src/doc.rs`
- `shells/desktop/src/render_loop/*`
- `Cargo.lock`

## 7. Para o integrador

- **Cenas de smoke** (contadas nos roteadores):

| Variável | Valor na linha | Onde se lê |
|---|---|---|
| `PH2D_KANBAN_SMOKE` | `CENAS: u8 = 9` | `ph2d-app-kanban/src/smoke.rs:30` |
| `PH2D_BOARD_SMOKE` | `max_level: 31` (cena 31 nova, «Quadros e Kanbans nos Assets») | `ph2d-app-board/src/smoke.rs:118` |

  Se o MiroClone também somou uma cena, **RECONTE**: a 31 desta linha passa a `N+1`, e mudam as fotos e o texto da parte 13 do tutorial.
- **Tectos de LOC:** os números estão em §8. A cura é MOVER, nunca subir.
- **A suíte completa e o `nextest-impacted` são do integrador.** A linha correu só os testes e gates por nome (§8).
- **§5.1 do CLAUDE.md:** a entrada está no fim deste ficheiro e já foi aplicada no CLAUDE.md desta linha. Na combinação vence a versão desta linha.

## 8. Prova de fecho

Gate de fecho batched, corrido 1× sobre o diff em 01/11 com `PH2D_SUITE=fecho` e `loadavg` 12–20. A suíte completa e o `nextest-impacted` ficam para o integrador.

**Verdes:**

- `cargo fmt --all -- --check`.
- clippy `--all-targets -D warnings` nas 19 crates tocadas e no `ph2d-host-desktop`.
- `censos-da-arvore-combinada.sh`: **129/129**, 12/12 censos. O main não andou.
- Crate inteira (motivo: código movido entre crates e deps):
  - `ph2d-kanban-views`: 41 passaram, 5 ignorados.
  - `ph2d-board-render`: verde.
  - `ph2d-app-kanban`: 28 + 105 passaram.
  - `ph2d-app-board --lib`: 96 passaram, 13 ignorados, em paralelo e também com `--test-threads=1`.
- Shell, por nome:
  - `file_loc_caps`: 4/4.
  - `the_shell_only_shrinks_and_every_app_crate_too`: verde.
  - `arch_safe_clamp_only`: verde.
  - `o_send_to_board_leva_a_seleccao_com_a_pose`: 3/3.
  - `project_catalogs`: 7 passaram.
  - `send_to_board`: 3 + 6 passaram.
- Por nome, no resto:
  - `asset_doc_menu`: 3.
  - `seam_doc_menu`: 4.
  - `scroll_list`: 6.
  - `the_calendar_and_date_picker_answer_the_hand`: 9.

**Três vermelhos que o gate apanhou, todos curados antes deste handoff:**

1. `arch_safe_clamp_only`: havia 7 `.clamp(0.0, max)` nus no `widget/scroll_list.rs` novo. A cura foi trocá-los por `math::safe_clamp`.
2. `duas_janelas::o_verso_abre_no_canvas_por_cima_do_quadro_e_fecha_no_fundo`: era uma REGRESSÃO da auditoria (`68d386048a`). O `back.area` passou a ser o rect acima do HUD e servia também de porta do rato (`click.rs`), por isso o clique no véu, na faixa do HUD, deixou de fechar o verso. A cura separa os dois papéis:
   - `Back.full` e `auto::Panel.full` são o alcance do rato (o véu inteiro).
   - `area` continua a ser onde se pinta e rola.
   - Vale no verso e na Automação.
3. `ph2d-app-board` `smoke::history` e `smoke::w16` só falhavam quando corriam DEPOIS de `smoke::d17_assets::fotos::asset_image_dropped_on_the_board_lands_selected_and_undoes`. Esse teste chamava `register_all_panels()`, um `OnceLock` do PROCESSO, e o registo completo muda o ecrã dos testes seguintes. A cura: o teste usa `staged_bare()`, sem registo; só as fotos `#[ignore]` instalam os painéis.

Smoke do dono aprovado em 01/11, depois da auditoria visual. Fotos em `docs/TrelloClone/smoke/D13..D17/`. Tutorial: `docs/TrelloClone/tutorial/01_kanban.pdf`, 29 páginas, parte 13 nas páginas 26–27.

Smokes (o `cd` inteiro):

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-TrelloClone && PH2D_KANBAN_SMOKE=1 cargo run -p ph2d-host-desktop --profile smoke`. Troque `1` por `2..9`: `5` = Filtro com barra, `4` = verso, `9` = relatório da importação.
2. `cd … && PH2D_BOARD_SMOKE=31 cargo run -p ph2d-host-desktop --profile smoke`. Mostra os Assets com Quadros e Kanbans: duplo-clique abre, arrastar para o Quadro insere, o «…» do cartão dá Open/Delete.
3. `cd … && PH2D_BOARD_SMOKE=30 cargo run -p ph2d-host-desktop --profile smoke`. Mostra o Kanban no Quadro.

Está errado quando:

- uma folha mostra fragmentos cortados sem barra;
- o verso fica por baixo do HUD do rodapé;
- um Kanban apagado deixa um cartão vivo no Quadro;
- segurar uma seta de barra dá um só passo.

## 9. ABERTO

- **Ao vivo entre duas casas (furo de NAT real):** o dono ADIOU (`fefc886666`). Está provado só em 127.0.0.1, a mesma pendência do Quadro ([ADR-0187](../../architecture/decisions/0187-o-quadro-pela-internet-e-quic-sobre-udp-com-encontro-na-dht.md)).
- **«Photos» do fundo a listar as imagens do PROJECTO:** foi revertido no D13 (`a49dec73c0`) e o dono decidiu outro caminho, «Use as Kanban background…» na Hierarquia.
- **HUD de estatísticas da app sobre a aba Kanban:** é chrome da shell, não desta linha. Esconder o HUD nas abas de documento é uma decisão de produto que fica para o dono.
- **Nenhum teste de integração clica a barra de uma folha pela `Mesa`.** A fiação está coberta pela compilação, pelas fotos e pelo smoke do dono, e as contas pelos testes da `sheet` e da `AxisBar`.
- Ao fechar: `rm -rf target/*/incremental` e `bash scripts/agent-loop-profile.sh`.

## Entrada §5.1 (≤ 700 bytes)

- **Kanban (TrelloClone)** — o Kanban é um documento do projeto ([ADR-0195](docs/architecture/decisions/0195-o-kanban-e-um-documento-do-projeto-o-quadro-referencia-o-e-a-aba-abre-o.md)): aba como no Trello (verso, vistas, fundo, Butler, `ph2d.kanban` Luau), o mesmo no Quadro, ao vivo, import do Trello, Planner; Kanbans e Quadros nos Assets. `ph2d-kanban-*` + `ph2d-app-kanban`/`-planner`. Smoke `PH2D_KANBAN_SMOKE=1..9` · `PH2D_BOARD_SMOKE=30|31`; tutorial PDF. Último: [D13–D17, 01/11](docs/TrelloClone/handoffs/HANDOFF_INTEGRACAO_line_TrelloClone_2026-11-01_D13_a_D17.md) · [plano](docs/TrelloClone/01_plano.md) · [docs](docs/TrelloClone/README.md)
