# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-11-02 — F46 a F50 (lados CAD pela curva · «Above Effects» · Offset Path como ferramenta · Draw fica nas outras formas · Boolean na barra de ações)

- `line/UIUX` → `main` por `--ff-only`, **só por ordem EXPLÍCITA do Enio**. Sem push, sem ship.
- Ponta `5c0c3740bf` (mais o commit deste handoff). Merge-base com o main = `3df5894489` (`git merge-base main HEAD`; o main estava lá em 02/11) ⇒ fast-forward possível; se o main andar, reconte pela §2.
  Delta (`main...HEAD`): **337 ficheiros, +10 015 −4 601**, 26 commits antes deste; `shells/desktop` **35 f. +257 −423** (encolheu: o Flatten saiu da shell para o `ph2d-app-vec`).
- **Smokes do dono, 02/11:** F46 OK · F47 OK (a 1.ª forma foi REVERTIDA pelo smoke: o traço «Above Effects» aparecia duplicado) · F48/F48b OK · F48d OK (cenas 50 e 51) · F49 OK · **F50 OK com três correções do smoke** (tirar o «⋯» e as booleanas raras; a ④ não tapa o gizmo; a ④ não tapa a coluna ①).
  O fecho (auditoria + mutação, `5c0c3740bf`) **não foi mostrado ao dono**: integrar não é aprovar.
- A linha editou a entrada UI/UX do `CLAUDE.md` §5.1 (smokes `6..53`, «Último» novo): o integrador reescreve a linha inteira, sem merge textual.

## 1 — O que entrou (mecanismo nos handoffs de continuação, `HANDOFF_CONTINUACAO_line_UIUX_2026-11-02_<onda>.md`)

| onda | o quê |
|---|---|
| F46 | Os lados das regras CAD pintam-se, acertam-se e acendem-se pela CURVA da forma, nunca pela corda (`cad_tool_plano.rs`/`cad_tool_lado.rs`). Era a «linha residual» do dono depois do Smooth — o ABERTO 1 do handoff F40–F45. Cena 48. |
| F47 | «Above Effects»: o traço (base e cada camada) sai da imagem dos Filtros e desenha-se UMA vez por cima. `StrokeSpec::above_effects` (**VEC_SCENE 27→28**), canal `TRACO_SOBRE_EFEITOS = 117`, checkbox no ⚙ do traço. Cena 49. |
| F48 | Expand e tools do painel sobre a pilha de tintas F45/F47: `ph2d-vec-scene/src/paint_split.rs` (`split_stack`, `carry_look_from`), booleanas/`cut`/`compound_from` levam o look completo, Inner/Outer numa camada de traço desenha. Cena 50. |
| F48b/c/d | Offset Path vivo (`ph2d-ecs/src/vec_offset.rs`: px, Miter limit, `result` Replace/Behind/Front — «In front»), e FERRAMENTA da coluna ① (`DrawMode::Offset`): opções na fila ②, Holes/Result como listas, alça só na ferramenta; a secção Expand sai do Inspector. Cena 51. |
| F49 | No Draw do vetor, escolher outra forma (ou várias) FICA em Draw. `ModeFamily::stays_on_its_kind` → `ModeState::kind_stays` → `object_mode::switches(.., kind_stays, ..)`. Exceção à lei 2 da W1 (24/10). Cena 52. |
| F50 | Boolean e Convert saem do painel para a barra de ações ④, à Figma: Union/Subtract/Intersect/Exclude (acesa a do grupo; booleana SEMPRE viva, a chave «Live» sai), **Flatten** (`ph2d_app_vec::bool_flatten`: bake do grupo mais de fora + Union destrutiva), **Outline Stroke** na ④. Secções Boolean/Convert fora do painel. **Por ordem do dono no smoke:** «⋯», Minus Back/Trim/Crop/Merge e Make/Release Compound REMOVIDOS da UI (o motor fica). ④ fora do gizmo e fora da coluna ①. Cena 53. |

## 2 — Formato e contadores (DELTA — conte no código)

- **`PROJECT_SCHEMA` 213 → 214** (`shells/desktop/src/project_schema.rs:516`; main = 213). UM degrau: 213→214 = `above_effects` (F47). Constante `PROJECT_SCHEMA_SEM_TRACO_SOBRE_FILTROS = 213`.
  Tripla em `project_schema_tests.rs:354`: **`(214, 13, 28)`** (main: `(213, 13, 27)`). Reconte com `python3 scripts/schema-recount.py 213` DENTRO do conflito, se o main/outra linha subir antes.
- **`VEC_SCENE_SCHEMA_VERSION` 27 → 28** (`ph2d-vec-scene/src/schema.rs:110`). `StrokeSpecV23` congelado para 23..=27 (`reading_stroke_v27`); fixture `vec_scene_v27_traco.bin` gravada pelo código v27 (HEAD `d95d9a1bea`) e gate `vec_v27_fixture`. `DOC_VERSION` intocado (`git diff main...HEAD -- crates/ph2d-timeline` = vazio).
- **Formato do offset (F48c):** `VecOffset.behind:bool → result:u8` (serde default 0; o campo nunca saiu da linha); canal 214 com três escolhas. `VecBoolGroup.op` 4..7 **continua lido** (receitas gravadas desenham igual — só a UI deixou de as criar).
- Cenas: `PH2D_OBJECT_MODE_SMOKE` `1|2|4|6..53` (main `..47`). Quem integrar outra linha com cena nova RECONTA.
- ADR: nenhum novo (`git diff --name-only main...HEAD | grep -c decisions/` = 0). Deps: nenhuma (`Cargo.toml`/`Cargo.lock` fora do diff).
- Registos de componentes: não recontados — o integrador reconta pelos gates de registo.

## 3 — Foundational e contratos congelados (§6)

Tocado, **ADITIVO** (Modo L), salvo as remoções abaixo (ids sem outro consumidor):

- `ph2d-editor-core`: `gizmo::screen_aabb` pública nova (`gizmo/paint_geom.rs`, reexportada); `selection_gap` `Lg`→`Xl`; `tool_bars::beside_column` (privada); `ModeState::publish_kind_stays/kind_stays`; `object_mode::switches` ganha o parâmetro `kind_stays`;
  `mode_drive::object_gizmo_shows`; ícones novos **`IconId::Flatten`, `IconId::OutlineStroke`** (enum cresceu — ⚠️ o integrador reconta se outra linha também acrescentou ícones; ordem alfabética).
  **Removido:** `VECTOR_COMPOUND_MAKE` (`ids/chrome/vector.rs`).
- `ph2d-tool-vector` (REMOVIDOS): `VECTOR_SECTION_BOOLEAN`, `VECTOR_BOOL_MINUS_BACK/TRIM/CROP/MERGE` (`ids/vector_cut.rs`). Novo `DrawMode::Offset`/`VECTOR_MODE_OFFSET`.
- `ph2d-panel-vector` (REMOVIDOS): `VECTOR_BOOL_LIVE_ON/OFF`, `VECTOR_BOOL_APPLY`, `VECTOR_SECTION_CONVERT`, `VECTOR_COMPOUND_RELEASE`. **Novo** `VECTOR_FLATTEN`. `state::set_bool_group(Option<u8>)` substitui `set_bool_group_selected`/`set_bool_live_on`.
  **Prova das remoções:** `git grep -l` de cada um dos 13 nomes (os 12 ids + as 2 funções de `state`, `VECTOR_COMPOUND_MAKE` incluído) sobre `*.rs` da ponta devolve **0 ficheiros**.
- ⚠️ **`VECTOR_SECTIONS`** (`ph2d-panel-vector/src/ids/vector_sections.rs:32`, lista append-only partilhada) **PERDEU entradas**: `VECTOR_SECTION_BOOLEAN` e `VECTOR_SECTION_EXPAND`/Convert (a lista agora tem 32 — F48d tirou a Expand, F50 a Boolean e a Convert). Outra linha que registe uma secção NOVA no mesmo bloco dá conflito textual; resolva mantendo as duas e SEM re-introduzir as saídas.
- `ph2d-ecs`: `canal_vetor` (`TRACO_SOBRE_EFEITOS=117`, `OFFSET_MITER=213`, `OFFSET_ATRAS=214`), `vec_offset.rs` (`RESULT_*`, `MITER_AT_BIRTH`, `keeps_source`), `canal_vetor_lista`/`_escolhas` — constantes e listas apendadas. `ph2d-i18n`: `vector.rs`, `vector_bars.rs`, `canais.rs` (chaves órfãs apagadas, ver §5).
- `ph2d-vec-scene` (`paint_split.rs` novo, `stroke_wire.rs`/`stroke_style.rs` +`above_effects` no FIM), `ph2d-vec-render` (`stack_draw::draw_above_effects`, `standalone::draw_path_for_effects`), `ph2d-vec-boolean` (look completo), `ph2d-vec-edit` (+1 linha), `ph2d-vec-svg` (+1).
- **Contratos §6 NÃO tocados:** `git diff main...HEAD --stat -- crates/ph2d-nodegraph crates/ph2d-tool-registry crates/ph2d-vector` = **vazio**; gate `architecture_tool_contract_surface` verde (§5). `Tool=12`/`RasterEditTool=5`/`CanvasPaintTool=1`/`PanelEvent=4` e `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` intactos; `VectorOp`/`Vertex`/`Segment` (ADR-0056..0068) intactos.

## 4 — O que um merge pode partir

- Quem usar qualquer dos 13 nomes removidos (§3) não compila. Quem chame `object_mode::switches` passa o parâmetro novo `kind_stays` (6.º posicional).
- `IconId`: conflito textual esperado no enum se outra linha acrescentou ícones; `icons_tests` conta-os.
- `VecPath`/`StrokeSpec`: `StrokeSpec { .. }` literal de outra linha precisa de `..Default::default()` (+`above_effects`).
- `VEC_SCENE`/`PROJECT_SCHEMA`: se outra linha subir antes, reconte e o leitor v27 fica atrás de `PROJECT_SCHEMA_SEM_TRACO_SOBRE_FILTROS`.
- **Tecto do `ph2d-app-vec`** (93 637, §5): número que SOMA entre linhas — oito lotes de cabeçalhos `//!` foram movidos verbatim para `docs/Vector Module/50_cabecalhos_do_app_vec.md`; o integrador reconta o tecto na árvore combinada.
- Atalhos `Ctrl+Alt+U/I/D/X` agora ARMAM a booleana viva (já não aplicam destrutiva); a destrutiva é o Flatten (`bool_flatten::pathfinder_selection`; `apply_vec_boolean` saiu da shell).
- Compound paths de SVG/projetos antigos ficam sem «desfazer» pela UI (Release Compound saiu por ordem do dono) — ver ABERTO 2.
- Colisões prováveis (`collision-surface.sh` pelo caminho ABSOLUTO em cada worktree): linhas que toquem `ids/vector_*`, `IconId`, `object_mode.rs`, `tool_bars.rs`.

## 5 — Prova de fecho (02/11) — por NOME, VERDES

- **Medido agora** (soma de linhas de todo `.rs` fora de `target`): `ph2d-app-vec` **93 612** (tecto 93 637, folga 25); `shells/desktop` **188 720** (gate `the_shell_only_shrinks`: `TETO_LOC` 189 041, verde). Delta da shell contra o main: −166 linhas.
- **Testes do fecho:** `ph2d-panel-vector` crate inteira **375 ✓** (3 ign.) · `ph2d-app-vec` **1150 ✓** (59 ign.) · clippy `-D warnings --all-targets` nas **6 crates tocadas** ✓ · rustfmt ✓ · `arch_safe_clamp_only`, `file_loc_caps`, `workspace_src_files_under_loc_cap`, `the_shell_only_shrinks`, `hr12`, `node_id_collisions`, `architecture_docs_paths_and_smokes_resolve`, `architecture_claude_md` ✓.
- **Mutação (worktree separada):** F48d **17/20** (3 lacunas fechadas com teste) · F49 **6/7** (o sobrevivente é equivalente) · F50 **13/14** (M14 lacuna, fechada no fecho por `bool_gesture::group_op` + teste de unidade).
- **Auditoria (2 lentes) → curas em `5c0c3740bf`:** (A1) `bake_ids` re-pendura no pai do grupo os filhos que não são operandos (traços abertos) — o Flatten os guarda · (A2) peças de grupos com `op ≥ 4` (receitas antigas) ficam fora da Union do Flatten · (A3) `outermost_group` devolve o grupo mais de fora QUE TEM plano ·
  (A4) `bool_gesture::group_op` (fn pura); a shell publica por ela e um gate de fonte o exige · (A5) a ④ só une as caixas do gizmo se ele será pintado (`mode_drive::object_gizmo_shows`) · (A6) `beside_column` evita também o flyout aberto · cena 50 (Expand) voltou a acabar (o gesto outline clica pela ④) ·
  `outline-stroke.svg` sem `stroke-dasharray` (o codegen ignora-o) · dicas da ④ em `vector_bars.rs` no formato «Name · hint» · i18n órfãs apagadas (`panel.vector.bar.bool_live`, `panel.vector.section.expand`) · comentários corrigidos.
- ⛔ **Suíte COMPLETA e `nextest-impacted` NÃO corridas** (regra do dono 23/10: são da integração). `architecture_tool_contract_surface`: corrido no fecho deste handoff junto dos dois gates de doc (resultado na resposta que acompanha o commit).

## 6 — Smokes (comando inteiro)

0. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && cargo build -p ph2d-host-desktop --profile smoke`.
1. **F50:** `cd /home/enio/Documentos/Projetos/PH2D && PH2D_OBJECT_MODE_SMOKE=53 cargo run --profile smoke --bin ph2d-host-desktop` — selecione duas formas: na barra de ações (junto da seleção) clique **Union**; a forma funde-se e a ② fica acesa; **Flatten** torna-a permanente; **Outline Stroke** contorna o traço. Errado se: a barra tapar a coluna de ferramentas ou o gizmo, aparecer «⋯», Minus Back/Trim/Crop/Merge ou Make/Release Compound. Foto: `docs/UI_New_and_Simple/img_F50/cena53.png`.
2. **F48–F49:** o mesmo comando com `PH2D_OBJECT_MODE_SMOKE=50`, `=51` (ferramenta Offset Path, 15 px vivo atrás; `PH2D_OBJECT_MODE_GESTO=apply|result`) e `=52` (escolher outra forma em Draw não sai do Draw).
3. F46 `=48` · F47 `=49` (checkbox «Above Effects» no ⚙ do traço).

## 7 — Integração (instruções ao integrador)

- Só por ordem do dono. Reescreva a linha UI/UX do `CLAUDE.md` §5.1 (≤ 700 bytes; gate `architecture_claude_md_cabe_no_orcamento`): «Último» = este handoff, smokes `6..53` (ou o que a contagem der).
- Reconte `PROJECT_SCHEMA`, tripla, `VEC_SCENE`, cenas, `IconId`, `VECTOR_SECTIONS`, tecto do app-vec, registos.

## 8 — ABERTO

1. **Holes «Inner» no canal de animação** (`crates/ph2d-ecs/src/canal_vetor_escolhas.rs` ~237, `OFFSET_LADO`) sem par na ④; `VECTOR_EXPAND_SIDE_INNER` está registado (`populate_ops.rs:100`) mas nunca é pintado.
2. **Release Compound saiu da UI** — compound paths de SVG/projetos antigos não têm «desfazer» pela UI. Decisão de produto a confirmar com o dono (o motor fica).
3. Grupo de formas como parte do Draw (F49 §2): um grupo escolhido na Hierarchy ainda sai para Object.
4. Inner/Outer + «Above Effects» (F48); os quatro W do perfil ainda `slider_row`.
5. `cargo fmt --all --check` alheio (pixel-preview, connector_handles) — não é desta linha.
6. Herdados do F40–F45: Weight corta 2 dígitos no Inspector estreito; Esc não cancela o arrasto de linha de painel; régua flutuante regista botões fora da janela baixa.
7. **Próximo bloco do dono:** Path Effects em lista (Contour, Pattern on Path, Envelope) → Blend + Morph.
