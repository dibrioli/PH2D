# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-07 — PARA O `main` (o ponto de entrada único)

> Leitor: o agente integrador (DIRETRIZ §1.5.3; `/pd-integracao`), e só por ordem do Enio. A linha
> FECHOU e PARA: nada integrado, nada enviado. **Este ficheiro é a porta; o mecanismo de cada onda está
> no handoff dela (§1), linkado.** Não substitui nenhum.
>
> **Ordem do dono (07/10)**, depois de fumar o gizmo invisível no Paint: *«smoke OK. Escreva handoff
> para integrar ao main»*.

- **Branch** `line/UIUX` (worktree `Worktrees/line-UIUX`) · **HEAD** `f493a573d` (+ o commit deste doc)
  · **base (merge-base = `main`)** `5d596eaaf` (a linha foi integrada a 04/10, tag
  `integ-rodada-04-10/UIUX`, e REABERTA a 05/10 sobre a poda do 3D).
- **O `main` ANDOU 1 commit:** `a46c4c200` *guarda(R4)* — toca só `.claude/hooks/tecto-de-recursos{,.prova}.sh`
  e `docs/DevOps/TETOS_DE_RECURSO_POR_LINHA.md`, **nenhum** dos 101 ficheiros desta linha ⇒ rebase/merge
  sem conflito esperado. Refaça o §5 sobre o `main` combinado (é a regra, não uma suspeita).
- `git cherry main HEAD` = **26 `+`** antes do commit deste doc. `git diff --shortstat 5d596eaaf HEAD` =
  **101 files, +2 736 / −2 060**.

## §0 — Para o `CLAUDE.md` §5.1 (≤ 700 bytes por módulo; gate `architecture_claude_md_cabe_no_orcamento`)

Medido hoje (com o `\n`): UI/UX **554** B · Vector + Esqueleto **511** B.

1. **UI/UX** — «Último:» → **este ficheiro**; `PH2D_OBJECT_MODE_SMOKE=1|4|6` → `1|4|6|7|8`; e na frase
   dos objectos por TIPO e MODO acrescentar «Image = Object · Paint · Edit» (≈ +40 B ⇒ ≈ 600 B).
2. **Vector + Esqueleto** — acrescentar «criar = *Add ▸ Vector Drawing*; cada forma é um objecto; o
   Edit é do TIPO (`Tab`)» (≈ +85 B ⇒ ≈ 600 B). Substitui a proposta do `…_O_OBJECTO_VETORIAL` (o
   contentor saiu).
3. Confira com `wc -c` e corra o gate.

## §1 — A linha, onda a onda (todas citam-se e NÃO se substituem)

| onda | handoff | smoke do dono |
|---|---|---|
| spec/06 alinhada com a poda do 3D (`fc172c88a`) | — | — |
| **cada forma é um objecto** — o contentor `VecObject` sai; *Add ▸ Vector Drawing* só arma a ferramenta; a forma que nasce pede o Edit; desseleccionar não sai do modo; a booleana deixa a forma em Edit COM gizmo | [`…_2026-10-05_CADA_FORMA_E_UM_OBJECTO.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-05_CADA_FORMA_E_UM_OBJECTO.md) | ✅ 05/10 |
| **os furos do Vector** — o Edit é do TIPO (decisão do dono); a herdeira (Soldar, juntar caminhos); a caneta continua o caminho aberto; Width/Trim descem à crate (§1 dele) | [`…_2026-10-05_OS_FUROS_DO_VETOR.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-05_OS_FUROS_DO_VETOR.md) | ✅ 05/10 |
| **2.ª leva** — o `Tab` só do modo (nós com `]`/`[`); o Node que falha o vetor escolhe o objecto de outro tipo; o botão IMG vira **Image ▸ Edit** (§5b dele) | idem §5b | ✅ 05/10 (com 2 reports) |
| **reports** — o gizmo da imagem no Edit; a sincronia da caneta apagava a escolha do Node (§5c dele, `e87d83373`) | idem §5c | ✅ |
| **o gizmo INVISÍVEL no Paint** (dono, 06/10: *«melhor deixar o gizmo invisível no paint mode»*; `f493a573d`) | idem §6 | ✅ 07/10 (*«smoke OK»*) |

## §2 — O que fica ABERTO depois da linha

- spec/02 **G** (esvaziar os painéis — por comandos: `inspector` 314, grande por direito; `tokens` 110,
  `physics` 60, `vector` 45; cada painel pede ao dono o destino de cada comando); **H** (separar layout
  de paleta, sem trava desde 20/09); **I** (os 4 temas clássicos no dia em que o `PH2D_UI_NEW` sair — do
  dono, e só nesse dia).
- Clicar num objecto de outro tipo sai do Edit do vetor pelo **Select** e pelo **Node**; os modos de
  DESENHAR não perguntam (desenhar por cima de uma imagem é o uso).
- O Paint e o Draw (Flip) **mantêm o cadeado** (não são do tipo); só o Edit (vetor e imagem) é do TIPO.

## §3 — Foundational tocado e contratos congelados

- **`ph2d-editor-core`** — `screens/hero/mode_drive.rs`: trait `ModeFamily` ganhou
  `holds_the_whole_kind(mode)` e `heir(mode, locked, tools)` (omissões `false`/`None`, append-only);
  `drive`: o passo 0 selecciona SÓ a entidade do `wants`; o aviso não se repete quando o modo só passa
  (`fell_from`); a rede `still_holds` pede a herdeira antes de largar; o `Step::Stay` re-tranca um modo
  do tipo no nascido pelo `enter_with`; o `Step::Leave` de um modo do tipo deixa a selecção;
  `refused` não recusa num modo do tipo; `whole_kind` publicado em todo quadro. `object_mode.rs`:
  `ModeState::{whole_kind, publish_whole_kind}`; `still_holds` aceita a selecção vazia. Chrome: o botão
  IMG saiu (`ids::TOPBAR_IMAGE_TOOLS`, `chrome/image_tools_toggle.rs`, `ModuleTruth::ImageMode`, a linha
  do menu Window) e o parâmetro `active` de `paint_top_bar_cluster`/`paint_topbar_rail_chip`.
- **`ph2d-ecs`** — `VecObject` removido. **`ph2d-component-desc`** — marcador `ObjectKind::Vector` =
  `VecPathRef`.
- **`ph2d-vec-scene`** — `VecViewState::editing` e `in_edit` removidos (o Edit é do tipo).
  **`ph2d-vec-edit`** — `PenTool::is_pickable`. **`ph2d-vec-entities`** — `view_state_for_pick` sem o
  Edit; `entities_object.rs` removido, `entities_duplicate.rs` novo.
- **Contratos congelados (§6 do `CLAUDE.md`): nenhum encostado** (o `ph2d-vector-doc` não entrou).

## §4 — Superfície de colisão

Crates: `ph2d-editor-core` (mode_drive + `mode_drive_tests`/`mode_drive_kind_tests`, object_mode,
chrome/topbar/menu, ids, state, fixture) · `ph2d-i18n` (object_add, component_catalog, chrome,
chrome_menus, chrome_panes) · `ph2d-ecs` · `ph2d-component-desc` · `ph2d-render`/`ph2d-script`
(contagens do registo) · `ph2d-app-components` (component_attach) · `ph2d-app-vec` (vector_mode +
`vector_mode_furos_tests` novo, object_add, trim + `trim_tests` vindo da shell, width_handles,
vec_selection, vec_gizmo_view, group_gizmo_view, svg_import) · `ph2d-app-painter` (paint_mode,
`image_edit_mode` novo, composite_smoke) · `ph2d-vec-scene` · `ph2d-vec-edit` · `ph2d-vec-entities` ·
`ph2d-vec-render` (overlays) · `ph2d-tool-bgremoval` (comentário) · `ph2d-app-registry-init`,
`ph2d-panel-registry-init`, `ph2d-tool-registry-init` (testes). Shell: `vec_trim.rs` (encolhe),
`input_dispatch/{despacho_clique_vetor_premido, despacho_clique_pick, keyboard_cadeia}`,
`render_loop/{fase_object_mode, fase_object_add, fase_vector_view_and_drives,
fase_vector_layout_recook, fase_vector_tree_settle, fase_vector_click_previews, snapshots}`,
`vec_tree_settle`, `hover_highlight`, `despacho_clique_largar`, `hierarchy_duplicate`,
`project_schema*`, `tests/it` (`the_mode_keys_are_wired`, `the_node_miss_picks_another_kind` novos;
trim/width/node_selection_scale actualizados). Docs: spec/06, os handoffs.

**Contadores (DELTA sobre `5d596eaaf`)** — somam entre linhas, **contam-se**:
- `PROJECT_SCHEMA` **+1** (183 → 184; tripla `(184,13,22)`) — `python3 scripts/schema-recount.py`.
- Registo ECS **−1** (`VecObject`); espelhos `ph2d-render`/`ph2d-script` **−1** cada.
- i18n: `+object_add.vector.drawing`; `−object_add.vector.object`, `−object_add.vector.object_name`,
  `−component.vec_object.name`; `−chrome.menu.image_tools`, `−chrome.topbar.name.image_tools`,
  `−chrome.topbar.tip.image_tools`, `−chrome.topbar.pill.img`, `−chrome.topbar.image_chip`.
- `MODULE_TRUTHS` 18 → 17; menu Window 13 → 12 linhas; barra clássica split 7 → 6.
- Cenas `PH2D_OBJECT_MODE_SMOKE`: **+7** (duas linhas pela caneta + Weld) e **+8** (Image ▸ Edit).

**O que um merge pode partir (falha ALTO):** quem leia `VecObject`, `object_of`, `adopt_loose`,
`lift_to_objects`, `object_view`, `container_or_object_view`, `EditTarget::editing`,
`VecViewState::editing`/`in_edit`, `crate::vec_trim::{hit, apply, piece_world}` (agora
`ph2d_app_vec::trim`), `ids::TOPBAR_IMAGE_TOOLS`, `ModuleTruth::ImageMode`, ou passe `active` aos chips
da barra. ⚠️ **Em silêncio:** uma família `ModeFamily` nova noutra linha compila (omissões), mas se
escrever `image_edit.mode_on` à mão perde a escrita — ele é o ESPELHO do Image ▸ Edit
(`image_edit_mode::mirror`, depois do quadro do modo).

## §5 — Gate final (HEAD `f493a573d`, BASE `5d596eaaf`)

Atestado do `verificador`, `load` 11–27, nenhum vermelho de tempo:
- nextest-impacted (`NO_FAIL_FAST=1`, com as features dos painéis) **17 928 / 17 928**.
- `CARGO_BUILD_WARNINGS=deny` check `--workspace --all-targets`, clippy `--workspace --all-targets -D
  warnings`, `fmt --check`: verdes. Censos da árvore 12/12 (i18n 114/114); doc-index em dia;
  `architecture_workspace_file_loc_cap` verde.
- Catraca da shell `the_shell_only_shrinks`: **184 838 / TECTO 189 041** (folga 4 203).
- `PROJECT_SCHEMA`: `main` 183, a linha 184 — tripla `(184,13,22)` em `project_schema_tests.rs:300`. ⚠️ O
  `scripts/schema-recount.py` só corre DENTRO de um conflito de rebase (pede o degrau em `argv[1]`); se
  outra linha entrar antes com 184, o degrau desta reconta-se aí.
- Mutação por onda (nos handoffs das ondas): CADA_FORMA 10/11 + 6/6; OS_FUROS 8/8 + 4/4 (fundação) +
  8/8 (2.ª leva) + os três dos reports (sincronia, gizmo do Paint e do Edit) a sangrar.

## §6 — Smoke do dono no `main` integrado (o que reconferir)

1. `cd ~/Documentos/Projetos/PH2D && ./target/smoke/ph2d-host-desktop`
2. `+` da Hierarquia ▸ «Vector Drawing», «Shape», arrastar: «Path 0», «Edit Mode»; outra forma: «Path 1»,
   o Edit passa a ela; clicar no vazio: desselecciona e fica «Edit Mode».
3. Duas linhas cruzadas pela «Pen», «Select», Shift+clique nas duas, «Weld»: uma linha, «Edit Mode», gizmo.
4. «Node»: `]`/`[` andam nos pontos; `Tab` volta a «Object Mode»; em Edit, clicar numa imagem com «Node»
   ou «Select»: a imagem fica seleccionada, «Object Mode».
5. Ctrl+N, seletor ▸ «Edit Mode»: a fila das ferramentas de imagem e o gizmo; ▸ «Paint Mode»: o pincel
   pinta e o gizmo NÃO aparece. O botão IMG não existe (nem no menu Window).

## §7 — Binário de smoke (2.ª corrida)

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.19s
```
