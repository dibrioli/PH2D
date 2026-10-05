# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-05 — cada forma é um objecto (o contentor `VecObject` sai)

> Leitor: o agente integrador, e a próxima janela desta linha. **Cita e NÃO substitui**
> [`…_2026-10-04_PARA_O_MAIN.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_PARA_O_MAIN.md) (a integração
> de 04/10, tag `integ-rodada-04-10/UIUX`) nem os anteriores, em particular
> [`…_2026-10-04_O_OBJECTO_VETORIAL.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_OBJECTO_VETORIAL.md)
> (o contentor que esta onda desfaz) e
> [`…_2026-10-04_O_CTRL_TAB_E_O_MASK.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_CTRL_TAB_E_O_MASK.md).

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `5d596eaaf` · **HEAD** = o commit deste
  ficheiro. A linha foi integrada no `main` a 04/10; o `main` recebeu depois a poda do 3D
  ([`HANDOFF_INTEGRACAO_line_poda-3d_2026-10-05.md`](../../Retirados/handoffs/HANDOFF_INTEGRACAO_line_poda-3d_2026-10-05.md),
  ADR-0179) e a linha foi **REABERTA a 05/10**: rebase por fast-forward para `5d596eaaf`.
  `git cherry main HEAD` = **5 `+`** (6 com este doc), `HEAD..main` = 0 commits (o `main` não andou).

## §0 — Para o `CLAUDE.md` §5.1

- **UI/UX:** trocar o link do «Último:» por este ficheiro. A frase não muda (já diz «objectos por TIPO e MODO»).
- **Vector + Esqueleto:** acrescentar «criar = *Add ▸ Vector Drawing*; cada forma é um objecto (Edit = `Tab`)»,
  mantendo a linha ≤ 700 bytes (gate `architecture_claude_md_cabe_no_orcamento`).

## §1 — O que esta onda entrega

| commit | o quê |
|---|---|
| `fc172c88a` | spec/06 alinhada com a poda do 3D |
| `8d344c9de` | cada forma é um objecto; o contentor `VecObject` sai |
| `2c937262c` | spec/06: a 2.ª volta do Vector entregue |
| `37954255f` | fecho: o teste das traduções do Add + clippy (registo das famílias com `SimWorld` imutável) |
| `2cdfc482a` | o Add não entra no Edit de uma selecção velha da caneta (mutação M6) |

**(0) spec/06** (`fc172c88a`): nota de cabeçalho (`ObjectMode` = Object · Paint · Draw · Edit; famílias
Painter/Flip/Vector; cenas de smoke 1|4|6), as entradas F3 Sculpt/Model marcadas HISTÓRIA, o item «Model
in 2D» morreu com o 3D, e a recomendação obsoleta «line/ObjectModes» corrigida.

**(1) A decisão do dono (05/10), verbatim:** «Não precisaremos mais de um objeto vazio como pai de
vetoriais. Quando o usuário criar um desenho vetorial, o painel vector abre e nada aparece no canvas ou
na hierarquia até que o usuário crie alguma forma ou linha. Ao desenhar algo, o objeto aparece na
hierarquia no modo edit». Perguntado se a 2.ª forma desenhada no mesmo Edit entra no 1.º objecto ou é
outro: «Cada forma é um objeto».

**(2) O modelo.** `ObjectKind::Vector` lê `VecPathRef` (a forma). Um GRUPO de formas é um Empty (como o
empty do Blender): o Edit é por forma; várias formas seleccionadas + `Tab` = Edit de todas (juntam-se).

**(3) O contentor sai.**
- `VecObject` removido de `ph2d-ecs` (`vec_object.rs` apagado); registo ECS 106 → 105, espelhos
  `ph2d-render`/`ph2d-script` 107 → 106 (deltas −1, cada um com a nota da casa); entrada do catálogo de
  componentes e chave i18n `component.vec_object.name` removidas.
- `PROJECT_SCHEMA` **183 → 184** (`shells/desktop/src/project_schema.rs` + a tripla `(184,13,22)` em
  `project_schema_tests.rs`). **Sem migração:** um projecto v183 é recusado (projectos gravados com
  contentores só existem desde 04/10).
- Removidos com o contentor: `entities_object.rs` (`adopt_loose`, `enclose`, `spawn_object`,
  `object_of`, `object_parent`, `top_within_object`, `wrap`) e os seus 12 testes;
  `vector_mode::lift_to_objects` (e os 2 chamadores na shell, `hover_highlight.rs` e
  `despacho_clique_largar.rs`); `vec_gizmo_view::object_view` e `container_or_object_view`
  (`snapshots.rs` volta a chamar `container_view`); a fronteira de objecto em `entities_group.rs`,
  `entities_selection.rs`, `group_gizmo_view.rs`, `svg_import.rs` (os hunks de `769939d62` reverteram
  limpos — comportamento anterior ao contentor); as duas chamadas da «regra das soltas» em
  `fase_vector_tree_settle.rs` e `vec_tree_settle.rs`.
- **MANTIDO:** «duplicar ao lado da original» (dono, 04/10) — movido para
  `ph2d_vec_entities::entities::duplicate::place_beside` (`entities_duplicate.rs`; gate com um grupo
  simples como pai).

**(4) Add ▸ Vector Drawing não cria nada.** Chave `object_add.vector.drawing`, texto «Vector Drawing»
(as chaves `object_add.vector.object`/`.object_name` saem). `ph2d_app_vec::object_add::add(entry, vec) -> bool`
chama `EditTarget::arm`; a fase `fase_object_add` empurra `ModeRequest::Enter(Object)` (um Add em Paint não
deixa o Painter a disputar a mão) e o cálculo de `at`/centro saiu da shell. `vector_mode::Family::follow`
põe a ferramenta na mão quando nenhum modo está activo e limpa a selecção da caneta (senão a porta velha
entraria no Edit de uma forma obsoleta — mutação M6).

**(5) Uma forma NASCIDA com a ferramenta na mão pede Edit sobre si.** Lei pura
`vector_mode::newborn(shapes, known, drawing)`: exactamente UMA forma nova fora de um gesto desde o
quadro anterior (`EditTarget::known`, escrito por `follow`). Desenhar a 2.ª forma num Edit faz o modo
PASSAR para ela no mesmo quadro (a rede `still_holds` larga a antiga, o pedido entra na nova). O `leave`
da família já não larga a ferramenta (o `follow` larga-a quando não resta Edit vectorial).
`EditTarget::editing(map)` (assinatura mudou: sem `sim`) = as formas do Edit + as nascentes.

**(6) Foundational `ph2d-editor-core` `mode_drive::drive`.**
(a) passo 0: o `wants` de uma família selecciona SÓ a sua entidade (juntava-se à selecção do mesmo tipo:
a forma anterior entrava no Edit novo — achado pelo gate); (b) o aviso «Edit Mode — Tab returns…» não
se repete quando o modo só passa a outro objecto (`fell_from`).

**(7) Smoke `PH2D_OBJECT_MODE_SMOKE=6`** reescrito: Add ▸ Vector Drawing (nada nasce, ferramenta na mão),
rectângulo (objecto em Edit), elipse (outro objecto, o Edit passa-lhe), Node, selector aberto. O 1.º
passo pendurou porque o quadro do Add leva o pedido Object e o passo do smoke não corre nele: a fase 2
espera agora o espelho da ferramenta activa (`hero.image_edit.active_tool_id == Some("vector")`).
**Fotos conferidas:** após o Add — painel Vector aberto, Hierarquia vazia, canvas vazio; no fim —
Hierarquia «Path 0», «Path 1» (seleccionado), Edit Mode, nós só na elipse, selector «Object Mode / Edit Mode».

## §2 — Foundational tocado e contratos

- `ph2d-editor-core`: `mode_drive::drive` (passo 0 + aviso `fell_from`). `ph2d-ecs`: `VecObject` removido.
  `ph2d-component-desc`: marcador `ObjectKind::Vector`.
- Contratos congelados (§6): **nenhum encostado.**

## §3 — Superfície de colisão

- `ph2d-editor-core` (`mode_drive`), `ph2d-ecs` (`VecObject`; contagens do registo),
  `ph2d-render`/`ph2d-script` (contagens do registo), `ph2d-component-desc` (marcador `Vector`),
  `ph2d-app-components` (`component_attach`),
  `ph2d-app-vec` (`vector_mode`, `object_add`, `vec_gizmo_view`, `group_gizmo_view`, `svg_import`),
  `ph2d-vec-entities` (`entities_object` removido, `entities_duplicate` novo, group/selection),
  `ph2d-i18n` (`object_add`, `component_catalog`),
  shell (`fase_object_add`, `fase_object_mode`, `fase_vector_tree_settle`, `vec_tree_settle`,
  `fase_vector_view_and_drives`, `snapshots`, `hover_highlight`, `despacho_clique_largar`,
  `hierarchy_duplicate`, `project_schema*`).

**Contadores (DELTA):** `PROJECT_SCHEMA` +1 (183 → 184); registo ECS −1; espelhos render/script −1 cada;
i18n: `object_add.vector.drawing` +1, `object_add.vector.object`/`.object_name` −2,
`component.vec_object.name` −1.

**A RECONTAR na integração** (somam entre linhas, contam-se, não se escolhem): o degrau do
`PROJECT_SCHEMA` (`python3 scripts/schema-recount.py`; a tripla `(184,13,22)` é a desta linha sobre
`5d596eaaf`) e as contagens dos registos (ECS −1, espelhos −1).

**O que um merge pode partir:** qualquer outra linha que leia `VecObject`, `object_of`, `adopt_loose`,
`lift_to_objects`, `object_view` ou `container_or_object_view` **deixa de compilar** (falha ALTO). Uma
linha que chame `EditTarget::editing` com `sim` também.

## §4 — Fecho (DIRETRIZ §1.5.9)

**Gate batched** (`verificador`, BASE `5d596eaaf`, sobre `2c937262c`, `load` 10–37):
- nextest-impacted **16 551 / 16 552**; o 1 vermelho era `ph2d-editor-core`
  `object_add::tests::every_label_is_translated` (o teste listava a chave removida) → curado em `37954255f`.
- clippy `-D warnings`: 1 erro (`unnecessary_mut_passed` no teste do registry-init) → curado em
  `37954255f`; depois, clippy do workspace verde.
- check com avisos negados, fmt, machete, doc-index: verdes. `censos-da-arvore-combinada`: 12/12 censos,
  114/114 testes.
- Catraca da shell `the_shell_only_shrinks`: **185 056 / TECTO 189 041**.

**Mutação** (agente `mutacao`): 11 mutações, 9 sangraram à 1.ª.
- **M1** (o `leave` da família larga a ferramenta outra vez) SOBREVIVEU e é **EQUIVALENTE**: `VectorTool`
  não tem ganchos de activação (o estado vive em `VecState`), logo largar+retomar no mesmo quadro é
  invisível; o desenho mantém «a passagem não larga a ferramenta» na mesma.
- **M6** (o `follow` já não limpa a caneta ao armar) SOBREVIVEU → gate novo
  `add_never_enters_the_edit_of_a_stale_pen_selection` (`2cdfc482a`); re-mutado ⇒ VERMELHO.
- Final: **10/11 sangram, 1 equivalente explicada.**

**Gates (nomes):**
- `ph2d-app-vec` `vector_mode_tests`: `the_pure_laws`,
  `add_arms_the_tool_and_each_drawn_shape_becomes_the_object_in_edit`,
  `a_shape_born_without_the_tool_does_not_ask_for_edit`,
  `two_shapes_edit_in_one_and_the_other_is_untouched`, `tab_with_two_shapes_edits_both`,
  `the_vector_tool_arriving_without_the_mode_asks_for_the_selected_shape`,
  `add_never_enters_the_edit_of_a_stale_pen_selection`.
- `ph2d-app-vec` `object_add_tests`: `the_entry_arms_the_tool_and_creates_nothing`,
  `one_entry_and_the_entry_of_another_family_is_not_ours`.
- `ph2d-vec-entities`: `a_duplicated_shape_stays_beside_its_source`.
- `ph2d-app-registry-init`: `the_vector_add_only_arms_the_tool_and_asks_for_object`,
  `the_hierarchy_duplicate_keeps_the_copy_beside_its_source` (substituem os gates do contentor).
- `ph2d-app-components`: `every_marker_derives_its_kind` (Vector = `VecPathRef`).

**NÃO verificado por compilação:** o toque do `Tab`/`Ctrl+Tab`/`Ctrl+Space` pelo teclado real (um gate
de despacho de teclado precisa da App) — deixado ao smoke do dono.

## §5 — Smoke do dono (lista técnica, para o registo)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. `+` da Hierarquia (ou Shift+A, ou botão direito no canvas vazio) ▸ «Vector Drawing»: o painel Vector
   abre e NADA aparece na Hierarquia nem no canvas.
3. No painel escolher «Shape» e arrastar no canvas: aparece um rectângulo, «Path 0» na Hierarquia, e o
   botão do topo diz «Edit Mode».
4. Arrastar outra forma: «Path 1» aparece como linha SEPARADA (não debaixo de Path 0), fica seleccionada,
   continua «Edit Mode»; o aviso «Edit Mode» não volta a saltar.
5. `Tab`: «Object Mode»; clicar Path 0 no canvas e arrastar: só o rectângulo se move.
6. Ctrl+clique nas duas linhas e `Tab`: as duas entram em Edit.
7. Em Edit, clicar no canvas vazio: continua «Edit Mode» e a forma continua seleccionada.
8. Com duas formas sobrepostas: Ctrl+clique nas duas linhas da Hierarquia, `Tab` (Edit das duas) e
   «Union» no painel: fica uma forma («Path 2»), seleccionada, em «Edit Mode», COM o gizmo à volta.

**Errado:** aparece uma linha «Vector» vazia depois do Add; a 2.ª forma vai para dentro da 1.ª; o modo
cai para Object depois de desenhar.

## §5b — Depois do fecho: reports do dono (05/10), curados em `0f3c8bef5`

1. *«em edit mode se clicar no canvas vazio (desselecionar) sai do modo Edit. Não permita isso»* —
   `mode_drive::drive` passo **0b**: num modo, LIMPAR a selecção (clique no vazio, `Esc`, caixa vazia)
   devolve-a ao objecto enquanto o módulo o tiver em mãos; o modo só cai com o objecto apagado ou
   desfeito. O `decide` continua a deixar passar o `None` SEM aviso (a doc diz agora porquê). Gate
   `clearing_the_selection_in_edit_keeps_the_edit` (controlo: a forma apagada ainda larga o modo).
2. *«ao fazer um boolean o gizmo já não aparece»* — **três causas**, medidas com um registo temporário
   da família no programa real (o 1.º gate passava porque fazia tudo num quadro só; a foto não):
   - a caneta a seleccionar as duas formas DENTRO do Edit (a shell copia-a para a selecção) largava o
     Edit — o cadeado lia «troca». ⇒ as formas que a caneta selecciona **juntam-se ao Edit** (`follow`
     e `parts`); ao entrar num Edit a caneta é **podada** às formas dele (uma selecção velha não entra
     no Edit seguinte — gate `a_stale_pen_selection_never_joins_the_next_edit`);
   - o Edit que cai porque as formas dele DESAPARECERAM largava a ferramenta, e a forma da booleana só
     ganha entidade no quadro SEGUINTE ⇒ o `follow` não larga a ferramenta quando todas as formas do
     Edit morreram (o `leave` nem é chamado: uma entidade morta não tem tipo);
   - `object_gizmo_shows`: no modo que declara `parts_take_the_object_gizmo` (só o vetor), a selecção
     tem o gizmo **incluindo a forma trancada** (com o contentor o trancado não o tinha).
   Gate `the_shape_a_boolean_leaves_is_in_edit_with_its_gizmo` (DOIS quadros, pelo caminho da caneta);
   `mode_drive_tests::a_parts_mode_gives_the_part_its_gizmo_and_the_lasso_its_parts` actualizado. A cena
   `PH2D_OBJECT_MODE_SMOKE=6` passa a fazer a UNION (rectângulo + elipse sobrepostos); **foto**: «Path 2»
   seleccionado, Edit Mode, gizmo à volta, painel Vector aberto.
   **Mutação** (agente `mutacao`): **6/6 sangram** — o passo 0b, a caneta no `follow` e no `parts`, o
   `!gone`, o gizmo do trancado (sangra também no gate do quadro) e a poda da caneta no `enter_with`.
   Lint `-D warnings` de `ph2d-app-vec`/`ph2d-editor-core` verde; `ph2d-editor-core` 1 966/1 966.

## §6 — O que fica ABERTO

- As layouts F4 (spec/06).
- Do [`PARA_O_MAIN`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_PARA_O_MAIN.md) §2: «Soldar» em Edit consome
  as formas e cai para Object — **reconferir agora**, porque o resultado booleano é um nascituro e o Edit
  devia passar para ele; a caneta dentro do Edit a acrescentar à forma; o gate de Width/Trim (premir).
- Um gate de despacho de teclado (Tab/Ctrl+Tab/Ctrl+Space) precisa da App.
- spec/02, degraus F/G/H/I.

## §7 — Perfil do laço do agente

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                296   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                678 : 224   alvo: <= 1,0  razao 3.0x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  34%   alvo: >= 80%  (1709 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         461 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## §8 — Binário de smoke

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.27s
```
