# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-04 — o OBJECTO VETORIAL (contentor das formas)

> Leitor: o agente integrador, e a próxima janela desta linha. **Cita e NÃO substitui** os sete
> anteriores desta linha:
> [`…_2026-10-02_A_ESCALA.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-02_A_ESCALA.md),
> [`…_2026-10-03_O_MENU_ADD.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MENU_ADD.md),
> [`…_2026-10-03_OS_MODOS.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_OS_MODOS.md),
> [`…_2026-10-03_O_SCULPT.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_SCULPT.md),
> [`…_2026-10-03_O_FLIP.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_FLIP.md),
> [`…_2026-10-03_O_MODEL.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-03_O_MODEL.md) e
> [`…_2026-10-04_O_VETOR.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_O_VETOR.md). Os commits até
> `cc9b45a4c` continuam descritos lá; **do O_VETOR deixou de valer o que o §2 abaixo desfaz**. Este
> cobre os DEPOIS e refaz o gate sobre o diff ACUMULADO.
>
> **Ordem do dono (04/10), depois de RECUSAR no smoke a F3 do O_VETOR:** *«apenas uma opção no
> modal: objeto vetorial. Ao clicar nele cria-se um objeto vazio e entra-se no modo edit do vector
> com o menu exatamente como era antigamente […] as shapes são filhas do objeto vetorial vazio»*;
> e para as formas soltas, *«cada uma ganha o seu objecto»*. Briefing:
> [`HANDOFF_CONTINUACAO_…_2026-10-04_O_OBJECTO_VETORIAL.md`](HANDOFF_CONTINUACAO_line_UIUX_2026-10-04_O_OBJECTO_VETORIAL.md).

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `1ad60a1ce` · **HEAD** = o commit deste
  ficheiro. `git cherry main HEAD` = **67 `+`**, nenhum integrado. Rebase: no-op.

## §0 — Para o `CLAUDE.md` §5.1

- **UI/UX:** trocar o link do «Último:» por este ficheiro. A frase não muda.
- **Vector + Esqueleto:** propõe-se acrescentar «criar = *Add ▸ Vector Object* (o contentor; as
  formas são filhas; Edit = `Tab`)», mantendo a entrada ≤ 700 bytes (gate
  `architecture_claude_md_cabe_no_orcamento`).

## §1 — O que esta onda entrega

| commit | o quê |
|---|---|
| `0ebf864da` | o objecto vetorial inteiro (marcador, regra das soltas, Edit sobre o objecto, Add com uma entrada, clique/laço/gizmo em Object, desfazer a partição) |
| `3b2a5c663` | curas do 1.º gate (o `editing` do `multi_path_tests`, fmt) + 2 gates (caneta velha; anel) + spec/06 |
| `82f436ad5` | o Envelope e o objecto numa porta só do gizmo (`publish_gizmo` acima do tecto de 200) |
| `80fc24730` | os 2 sobreviventes da mutação (a cabeça com uma forma em gesto espera inteira; o gate do fundo) |

**O desenho:**
- **`ph2d_ecs::VecObject`** (marcador de tamanho zero, como o `FieldObject`): uma entidade SEM
  geometria cujas filhas `ChildOf` são as formas. `ObjectKind::Vector` passa a lê-lo
  (`ObjectKind::marker`, `component_attach::kind_of`); as formas passam a `Empty` — são PARTES.
  Catálogo `D::machinery` em `catalog/vector.rs`, i18n `component.vec_object.name`.
- **`ph2d_vec_entities::entities::object`** (ficheiro `entities_object.rs`): `object_of`,
  `object_parent`, `top_within_object`, `spawn_object`, `enclose` e ⭐ **`adopt_loose` — a regra das
  SOLTAS**: toda forma sem objecto por cima, fora das que estão em gesto, entra no objecto do Edit
  aberto (`reparent_keeping_world`; ao FUNDO se nasceu atrás dele — o balde) ou é EMBRULHADA num
  objecto novo no lugar dela (pai, `RootOrder`/`SiblingOrder`, translação). A «cabeça» sobe por todo
  pai só-vetor (`vector_only`: moldura, contentor de envelope, grupo de formas). Corre **no passe do
  desenho** (`fase_vector_tree_settle.rs`, depois do `settle_origins`) **e na rede da captura**
  (`vec_tree_settle.rs`) — a forma entra no objecto no MESMO passo de undo (censo
  `the_net_knows_every_derived_writer`).
- **A fronteira do objecto:** `top_members`/`group_entities` agrupam DENTRO do objecto quando todos
  os membros vivem num (`group_inside`; é o que a booleana viva usa); `ungroup_entities` nunca
  dissolve um `VecObject`; `selection_root` pára no objecto (o clique em Edit escolhe a forma).
- **`VecViewState::editing: Option<Vec<VecPathId>>`** (era `Vec`): `None` = Object; `Some(vazio)` =
  o Edit de um objecto recém-nascido, onde NADA de fora se agarra.
- **`ph2d_app_vec::vector_mode`** reescrito sobre o objecto: `EditTarget { objects, following,
  born }` + `editing(sim, map)` (as formas dos objectos + as soltas); `Family::new(vec, sim)` lê os
  objectos e as soltas; `holds` = alvo + ferramenta `vector` na mão (QUALQUER `DrawMode`); `parts` =
  tudo debaixo dos objectos + os outros objectos + as soltas; `owner_of(forma)` = o objecto; `wants`
  = o `born` do Add, ou a porta antiga (a ferramenta chega à mão com formas seleccionadas ⇒ Edit do
  objecto delas, e SÓ aí a caneta junta — `via_pen`). `lift_to_objects`: em Object o clique, o
  realce (`hover_highlight::pick_objects_at`) e o laço (`despacho_clique_largar.rs`) sobem ao objecto.
- **Gizmo:** `vec_gizmo_view::object_view` (caixa-união das formas no espaço do objecto) e
  `container_or_object_view` (uma porta para o Envelope e o objecto, em `snapshots.rs`).
  `group_gizmo_view::is_empty_object` exclui o objecto COM formas — o anel de vazio fica só no
  objecto ainda vazio (a 1.ª foto mostrou um anel em cada forma).
- **Menu Add:** `ph2d_app_vec::object_add::{VECTOR_OBJECT, ENTRIES = [VECTOR_OBJECT], add}` cria o
  objecto vazio no centro da vista e arma `born`. Na shell sai o `Born::Tool`.
- **SVG importado** = um só objecto (`enclose` no topo do desenho).
- **Fundação:** ao sair de um modo que junta, só as partes do MESMO tipo voltam à selecção
  (`mode_drive.rs`, Step::Leave) — as formas de dentro também são partes.
- **`PH2D_OBJECT_MODE_SMOKE=6`** refeito (`vector_mode::smoke_step`): *Add ▸ Vector Object* →
  rectângulo + elipse → UNION pelo botão do painel → `Tab` → 2.º objecto → estrela → Node → seletor.

## §2 — O que do O_VETOR deixou de valer (desfeito)

- `DrawMode::EDIT_TOOLS`/`object_mode()` (`params_mode_object.rs` **APAGADO**) e o gate
  `the_edit_tools_are_exactly_the_draw_modes_of_edit`.
- O filtro da fileira TOOL do painel (`paint_modes.rs`) e os gates de seam que o seguiam:
  `ph2d-panel-vector/tests/it/seam.rs` voltou ao de `e99d5d72e` (= base); sai
  `the_tool_row_shows_only_the_tools_of_the_current_mode`. `ph2d-tool-vector` e `ph2d-panel-vector`
  ficam **byte-idênticos à base**.
- `object_add::{RECTANGLE, ELLIPSE, POLYGON, STAR, PEN, PENCIL, TEXT, SHAPES, TOOLS, arm, tool_of}`,
  `EditTarget::{paths, armed, last_tool}`, `Born::Tool`, e as i18n `object_add.vector.{rectangle,
  ellipse, polygon, star, pen, pencil, text}`.
- Fica do O_VETOR: o pill VECTOR fora, `joins`/`enter_with`, `in_edit` como porta única do clique e
  das âncoras, o censo de famílias com a tabela D6.

## §3 — Foundational tocado e contratos

- `ph2d-ecs`: `vec_object.rs` NOVO, `lib.rs`, `registry.rs` (700/700 linhas — o registo ficou numa
  linha com comentário de fim de linha). `ph2d-component-desc`, `ph2d-vec-scene` (`editing` Option),
  `ph2d-vec-entities` (aditivo + `entities_group.rs`/`entities_selection.rs`), `ph2d-editor-core`
  (`mode_drive.rs`, 1 filtro).
- Contratos congelados (§6): **nenhum encostado** — `Tool=12`/`PanelEvent=4` intactos; o
  `architecture_vector_contract_surface` lê `ph2d-vector-doc`, intocado.

## §4 — Superfície de colisão (para o integrador)

**Contar, nunca escolher:**
- **`PROJECT_SCHEMA` 178 → 179** (`shells/desktop/src/project_schema.rs` + a tripla `(179, 13, 22)`
  em `project_schema_tests.rs`; a tripla NÃO vê o degrau). Reconte com
  `python3 scripts/schema-recount.py`.
- **Registos:** `ph2d-ecs` `registry_tests` **108 → 109**; espelhos `ph2d-render`/`ph2d-script`
  **109 → 110**. Delta +1 cada.

**Falha ALTO noutra linha (não compila):**
- `VecViewState::editing` é `Option<Vec<VecPathId>>` — todo literal `editing: vec![…]` parte (havia
  um em `ph2d-vec-edit/src/multi_path_tests.rs`, curado).
- `vector_mode::Family::new(vec, sim)` (2 argumentos); `smoke_step(vec, scene, sim, hero)`;
  `object_add::add(entry, sim, vec, at) -> Option<u64>`; símbolos apagados do §2.

**Funde LIMPO e REPROVA depois:**
- `ObjectKind::Vector` = `VecObject`: quem perguntar `kind_of(forma) == Vector` passa a ler `Empty`.
- Toda forma ganha um objecto no 1.º quadro (a regra das soltas): gates de outra linha que contem
  RAÍZES, `RootOrder` de formas ou entidades na Hierarquia de uma cena com formas podem mudar de
  número. As cenas `PH2D_*_SMOKE` ganham uma linha «Vector (n)» por forma (foto da `=44` conferida).
- `group_entities`/`ungroup_entities`/`selection_root` respeitam a fronteira do objecto.
- i18n **NOVAS** `object_add.vector.object`, `object_add.vector.object_name`,
  `component.vec_object.name`; **APAGADAS** as sete do §2.
- Catraca da altura do painel `vector` **1 239 → 1 262** (= a base; a grelha inteira volta).
- **`the_shell_only_shrinks`**: HEAD do O_VETOR **196 909** → HEAD **196 974**; esta onda **+65**;
  tecto **196 990** — folga **16**.

## §5 — Fecho: gate batched, mutação, auditoria

**Gate batched** (verificador, diff acumulado desde `1ad60a1ce`), três corridas:
- 1.ª (HEAD `0ebf864da`): não compilou (`ph2d-vec-edit/src/multi_path_tests.rs:308`, o literal
  `editing: vec![a]`) + fmt em 2 ficheiros ⇒ `3b2a5c663`.
- 2.ª (HEAD `3b2a5c663`): **21 100 / 21 101** — `fn_loc_caps::shell_functions_respect_their_ceiling`
  (`snapshots.rs::publish_gizmo` 211 > 200, o fmt abriu a chamada nova; reprovou 3/3 sozinho, não é
  membro de `FLAKES_DE_CARGA.md`) ⇒ `82f436ad5`.
- 3.ª (HEAD `82f436ad5`):

| portão | resultado |
|---|---|
| `nextest-impacted` (BASE `1ad60a1ce`) | **21 101 / 21 101** |
| `ph2d-panel-registry-init --all-features` | **136 / 136** (2.ª corrida; o 3.º commit não o toca) |
| `CARGO_BUILD_WARNINGS=deny cargo check --workspace --all-targets` | verde |
| `clippy --workspace --all-targets -D warnings` | verde |
| `cargo machete` · `check-standalone-optional.sh` · `check-workflow-packages.sh` | verdes (2.ª corrida) |
| `cargo fmt --all --check` | verde |

O 4.º commit (`80fc24730`) só toca `ph2d-vec-entities`: **46 / 46**, clippy `-D warnings` e fmt
verdes.

**Mutação** (agente `mutacao`; controlo VERDE com população > 0, âncora 1×): **24 / 24 sangram** sobre o
código final. 1.ª passada (HEAD `82f436ad5`): 22 / 24 — dois sobreviventes REAIS, curados em
`80fc24730`: **M2** (tirar o «em gesto» do filtro da forma) mostrou uma LEI em falta, não só um gate —
uma moldura com um filho em gesto seria levada para o objecto a meio do traço ⇒ o «em gesto»
percorre agora a sub-árvore da cabeça (gate `a_head_with_a_shape_in_gesture_under_it_waits`); **M5**
(apagar o `send_to_back`) passava porque o gate comparava `None < Some(_)` ⇒ ordens reais depois do
`assign_missing_sibling_order`. Re-mutados: os dois sangram. Lado INDEPENDENTE: família FALSA que
junta em `mode_drive_tests.rs` (M20, o filtro do mesmo tipo ao sair), a REAL em
`vector_mode_tests.rs`; três gates da shell lêem o fonte (M22–M24).

**Auditoria (DIRETIVA §3), três lentes:**
- **Correção — o que nasce no Edit é do objecto, no mesmo passo de undo.** TRAÇO: gesto da caneta →
  `fase_entity_sync` (`entities::sync` cunha a entidade RAIZ) → captura: `vec_tree_settle.rs`
  (`settle_origins` → `object::adopt_loose(…, self.vec.edit.object(), &drawing, …)` →
  `reparent_keeping_world`) → `ProjectState::capture`. Entretanto a forma solta já é do Edit:
  `EditTarget::editing` (agarra-se) e `Family::parts` (o cadeado não cai). VERMELHAS:
  `with_an_edit_open_the_new_shape_enters_its_object`, `a_shape_in_gesture_waits`,
  `a_shape_born_in_the_edit_belongs_to_it_before_it_settles`,
  `the_loose_shape_rule_runs_in_both_nets_with_the_edit_object`.
- **Fiação — *Add ▸ Vector Object* até ao Edit com o painel inteiro.** pick → `fase_object_add.rs`
  → `object_add::add` (`spawn_object` + `edit.born`) → quadro seguinte `fase_object_mode` →
  `mode_drive::drive` passo 0 (`wants` → `born`) → `enter_with` (`tools.set_active("vector")`) →
  `follow`. VERMELHAS: `the_entry_creates_an_empty_vector_object_that_asks_for_edit`,
  `two_objects_edit_in_one_and_the_other_is_untouched`; foto `=6`.
- **Object — a forma responde pelo objecto.** clique: `despacho_clique_pick.rs` →
  `hover_highlight::pick_objects_at` → `lift_to_objects`; laço: `despacho_clique_largar.rs`; gizmo:
  `snapshots.rs::publish_gizmo` → `container_or_object_view`; `Tab` sobre uma forma: `publish_active`
  → `owner_of`. VERMELHAS: `in_object_mode_a_click_on_a_shape_names_its_object`,
  `a_vector_object_publishes_the_union_of_its_shapes`, `a_shape_answers_for_its_object`,
  `object_mode_picks_and_boxes_the_whole_vector_object`.
- NÃO-CHECADO-PELA-COMPILAÇÃO: o arrasto real do gizmo de um objecto no canvas (a caixa vem do
  `object_view`, o arrasto é o do grupo de sempre) — não fotografado em Object; o dono ensaia no
  smoke (passo 5).

**Fotos** (`fotografa_cena.sh`, conferidas):
- `PH2D_OBJECT_MODE_SMOKE=6` — Hierarchy «Vector ▸ Path 2» (a união) e «Vector (1) ▸ Path 3» (a
  estrela); Edit do 2.º com o Node: nós laranja SÓ na estrela, a união intocada; painel com a grelha
  inteira; seletor aberto. ⛔ As três primeiras fotos apanharam o que nenhum gate via: o 2.º objecto
  entrava em Edit JUNTO com o 1.º (a caneta tinha a forma velha ⇒ `via_pen` + gate); as formas a
  ±160 nasciam fora do ecrã (o mundo é em METROS, 100 px/m); e a estrela lia os valores da forma
  activa da ferramenta (`Points 0` ⇒ `ShapeKind::defaults()`).
- `PH2D_BUILD_SMOKE=44` (cena antiga) — cada forma em «Vector (n)», desenhadas no sítio; a 1.ª foto
  mostrou um anel de vazio em cada uma ⇒ `is_empty_object` exclui o objecto com formas.
- `PH2D_OBJECT_ADD_SMOKE=1` — o Add com **2D · 3**: Image…, **Vector Object**, Flip Drawing.

## §5b — Depois do fecho: report do dono (04/10)

- *«não consigo selecionar as formas vetoriais dentro do objeto»*. **Mecanismo:** em Edit com o
  **Select** a ferramenta não captura o canvas (ADR-0112) — o clique vai ao pick de objectos
  (`hover_highlight::pick_objects_at`), que monta a vista com `view_state_for_pick(…, &view_derived)`.
  O `editing` era escrito na vista do DESENHO (`fase_vector_view_and_drives.rs`) e copiado DENTRO do
  `view_state_for_pick`, mas **ninguém o publicava no `view_derived`** ⇒ o pick lia Object e o
  `lift_to_objects` subia da forma ao objecto inteiro. O Node funcionava (a caneta lê a vista do
  desenho). Defeito herdado da onda O_VETOR (a cópia tinha gate; o escritor não) e que esta onda
  agravou (o `lift`). **Cura:** `fase_vector_layout_recook.rs` publica
  `view_derived.editing` ao lado de `clips`/`poses`/`absorbed`; gate
  `the_vector_edit_reaches_the_frame_view` estendido ao escritor — sangra sem a linha (mutação
  conferida, com controlo verde). Também cura o laço e o realce em Edit.
- 2.º report (foto): *«o gizmo não aparece e não consigo a multiseleção»*. **Dois mecanismos:**
  (1) `mode_drive::object_gizmo_shows` = «só em Object» (D6) — certo para Sculpt/Paint, mas o
  Select do Edit do vetor transforma as formas PELO gizmo. ⇒ fundação: `ModeFamily::
  parts_take_the_object_gizmo` (omissão `false`; o vetor diz `true` em Edit), publicado em
  `ModeState::part_gizmo`; o gizmo aparece sobre uma PARTE seleccionada, nunca sobre o objecto
  trancado. (2) O laço chamava `refused(hero, None, additive=true)`, que a lei recusa SEMPRE — o
  aviso *«Leave Edit Mode (Tab)…»* da foto. ⇒ `mode_drive::lasso_admits`: num modo de partes o laço
  fica só com as partes (e o trancado); num modo inteiro recusa como antes. Gate
  `a_parts_mode_gives_the_part_its_gizmo_and_the_lasso_its_parts` (família FALSA; Sculpt como
  controlo) + a declaração na família REAL; 4/4 mutações sangram. ⚠️ Superfície nova para o
  integrador: um método com omissão no trait `ModeFamily` e um campo no `ModeState` (aditivos).
- **Smoke do dono: APROVADO (04/10)**, sobre o HEAD `798c8e97c` (Select, gizmo da forma, laço, Union, `Tab`).

## §6 — Premissas derrubadas

- *«Uma porta só depois do `sync`»* — são DUAS (o passe do desenho e a rede da captura), e a forma
  em gesto tem de esperar (a mão escreve-a em mundo a cada quadro).
- *«A forma nasce raiz; basta reparentar»* — o Envelope e a booleana viva constroem CONTENTORES na
  raiz com as formas: a «cabeça» solta tem de subir por pais só-vetor, senão cada forma era
  embrulhada dentro do contentor e a gaiola partia-se (gate `an_envelope_made_in_the_edit_stays_whole_in_the_object`).
- *«O `group_entities` serve dentro do objecto»* — normalizava para o topo, que era o próprio objecto
  ⇒ a booleana viva recusava dentro dele (`top_within_object`).
- *«A cópia do `editing` no `view_state_for_pick` leva o Edit ao clique»* — a cópia tinha gate e
  zero escritores do lado de lá (§5b).
- *«Lista `editing` vazia = Object»* — o objecto nasce vazio EM Edit ⇒ `Option`.

## §7 — Smoke do dono (o que ensaiar)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. Botão direito no canvas vazio → **Add Object** → **Vector Object** (a única entrada vetorial, em
   2D). Nasce «Vector» na Hierarchy, o botão do topo diz **Edit Mode** e o painel **Vector** à
   direita mostra a grelha inteira (Select · Node · Pen · Pencil · Shape · Text · Connect · Build ·
   Fillet · Chamfer · Width · Cut · Trim · Bucket · Frame). Errado: diz Object Mode, ou o painel
   mostra só algumas.
3. No painel, **Shape** e arrastar no canvas: um rectângulo. Outra vez, por cima de parte do
   primeiro. Na Hierarchy as duas aparecem **por baixo de «Vector»** (Path 0, Path 1). Errado:
   aparecem como «Vector (1)» ou soltas.
4. **Select**, `Shift`+clique nas duas, e na secção **Boolean** do painel → **Union**: fica UMA forma,
   ainda por baixo de «Vector». Errado: a forma nova sai do objecto.
5. **Tab** → **Object Mode**. Clicar na forma: fica seleccionado o **«Vector» inteiro** (a linha dele
   acende), com uma caixa à volta de tudo; arrastar move tudo junto. Errado: só a forma acende, ou
   aparece um círculo em cima da forma.
6. **Add Object** → **Vector Object** de novo, desenhar uma **Shape** ao lado (escolha Star no painel).
   Escolher **Node**: só a estrela mostra os quadradinhos laranja; clicar na primeira forma não
   agarra nada; clicar «Vector» na Hierarchy é recusado com um aviso. Errado: a outra forma mostra nós.
7. **Tab** → Object; `Ctrl`+clique nas duas linhas «Vector» e **Tab**: as duas entram em Edit (as
   formas de ambas mostram nós com o Node). **Tab** → as duas continuam seleccionadas.

## §8 — O que fica para a próxima janela

- (a) Seleccionar na Hierarquia uma FORMA em Object selecciona a forma (o gizmo move-a dentro do
  objecto); só o canvas sobe ao objecto. Nomeado, não pedido.
- (b) Uma forma arrastada na Hierarquia para fora do objecto ganha um objecto novo (a regra das
  soltas) — coerente com a escolha do dono, a confirmar no smoke.
- (c) ✅ **Decidido pelo dono (04/10):** (a) e (b) ficam como estão. Duplicar a LINHA de uma forma
  deixa a cópia no MESMO pai, logo a seguir à original (`entities::object::place_beside`, chamado
  em `hierarchy_duplicate.rs`; gate `a_duplicated_shape_stays_beside_its_source`, mutação sangra);
  duplicar o OBJECTO copia-o inteiro (`duplicate_subtree`).
- (d) `Ctrl+Tab`; Image ▸ Mask; F4 (layouts); Model em Object não desenha (dos handoffs anteriores).

## §9 — Perfil do loop (`bash scripts/agent-loop-profile.sh`)

```
  ✗ paralelismo de ferramenta              1.14/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                219   alvo: <= 800  (uma janela nova por onda de trabalho)
  ✗ cargo test : cargo check                611 : 289   alvo: <= 1,0  razao 2.1x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  36%   alvo: >= 80%  (1709 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         490 mil   alvo: <= 250 mil  (set/2026: 606 mil — 82% do custo)
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil  (02/10: 380 mil, CLAUDE.md a 710 KB)
```

## §10 — Binário de smoke (último passo: `bash scripts/ph2d-run.sh cargo build -p ph2d-host-desktop --profile smoke`, 2.ª corrida)

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.23s
```
