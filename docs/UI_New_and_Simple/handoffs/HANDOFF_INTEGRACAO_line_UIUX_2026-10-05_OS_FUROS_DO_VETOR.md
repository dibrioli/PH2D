# HANDOFF DE INTEGRAÇÃO — `line/UIUX`, 2026-10-05 — os furos do Vector (o Edit é do TIPO)

> Leitor: o agente integrador, e a próxima janela desta linha. **Cita e NÃO substitui**
> [`…_2026-10-05_CADA_FORMA_E_UM_OBJECTO.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-05_CADA_FORMA_E_UM_OBJECTO.md)
> (a onda anterior desta reabertura, smoke APROVADO) nem o
> [`…_2026-10-04_PARA_O_MAIN.md`](HANDOFF_INTEGRACAO_line_UIUX_2026-10-04_PARA_O_MAIN.md). Ordem de
> origem: [`HANDOFF_CONTINUACAO_…_2026-10-05_OS_FUROS_DO_VETOR.md`](HANDOFF_CONTINUACAO_line_UIUX_2026-10-05_OS_FUROS_DO_VETOR.md) §2.

- **Branch** `line/UIUX` · **base (merge-base = `main`)** `5d596eaaf` · **HEAD** = o commit deste
  ficheiro. `git cherry main HEAD` = **23 `+`** com este doc (a 1.ª redacção dele, `8310dc52c`,
  incluída); `HEAD..main` = 0. Nada integrado. ⚠️ O §5b é a 2.ª leva (depois do smoke do dono).

## §0 — Para o `CLAUDE.md` §5.1

- **UI/UX:** «Último:» → este ficheiro; smoke `PH2D_OBJECT_MODE_SMOKE=1|4|6|7|8`; «Image = Object · Paint · Edit (o IMG saiu)».
- **Vector + Esqueleto:** «cada forma é um objecto; o Edit é do TIPO (`Tab`)» (≤ 700 bytes, gate
  `architecture_claude_md_cabe_no_orcamento`).

## §1 — O que esta onda entrega

| commit | o quê |
|---|---|
| `d75a10d76` | o Edit é do TIPO; a herdeira; Width/Trim descem à crate; cena 7; spec/06 |
| `afaa2a887` | fecho: fmt; os gates que lêem a shell seguem a costura; sai o gate do filtro revogado |
| `ececa7c58` | fecho: gate do modo do TIPO na própria fundação; o nascido re-tranca pelo `enter_with` |

**(1) A decisão do dono (05/10)** — perguntado (furo 3) se Shift+clique juntava ao Edit uma forma de
fora, verbatim: *«o modo de edição significa que todos os objetos daquele tipo estão em modo de
edição. Ao clicar num objeto de outro tipo, o objeto deve ser selecionado mas em modo object, saido
do modo de edição do objeto de tipo diferente que estava previamente selecionado»*. ⇒ a pergunta
deixou de existir: toda forma está no Edit.

**(2) Fundação `ph2d-editor-core` (append-only no trait):**
- `ModeFamily::holds_the_whole_kind(mode)` (omissão `false`) → `ModeState::whole_kind` publicado em
  todo quadro (passo 4). Com ele: `mode_drive::refused` não recusa nada (um objecto de outro tipo
  escolhido muda a selecção e a rede `still_holds` volta a Object, sem aviso); o `Step::Leave` deixa
  a selecção como está (vazia ⇒ o trancado), em vez de seleccionar as partes (que são TODAS as formas).
- `ModeFamily::heir(mode, tools)` (omissão `None`): no passo 2, quando o modo deixa de se segurar,
  uma família pode indicar a herdeira — o modo re-tranca nela **sem tocar a selecção** e sem aviso.
- Passo 3, `Step::Stay`: num modo do TIPO, o objecto que NASCE (`wants`) já é parte (a rede não cai),
  logo o modo **passa** a ele pelo `enter_with` da família (sem aviso).
- ⚠️ O Paint (Painter) e o Draw (Flip) **continuam com o cadeado** (não declaram o tipo).

**(3) `ph2d-app-vec::vector_mode`:** `holds_the_whole_kind(Edit)`; `parts` = todas as outras formas
vivas; lei pura `heir(objects, shapes, pen)` = a última viva da caneta, senão uma do Edit, senão
qualquer forma. O `follow` deixou de juntar a caneta ao `objects` (o Edit já tem todas).
- **Furo 1 (Soldar):** o *Weld* escreve a rede no traço mais ao fundo e consome os outros; com o Edit
  trancado num consumido ele caía a Object — agora passa à rede, que fica com o gizmo. (A booleana
  também passa a usar a herdeira quando o resultado ganha entidade no mesmo `sync`: medido na foto.)
- **Furo 2 (caneta):** **já existia** — `PenTool::reopen_endpoint` (premir a ponta de um caminho
  aberto retoma-o, mesmo `VecPathId`) e `join_open_path` (premir a ponta de OUTRO junta-o). O que a
  impedia era o filtro do Edit por forma (o outro traço não se agarrava). Juntar apaga o juntado —
  se era o trancado, a herdeira segura o Edit. Oráculo não corrido: a lei já estava no código e no
  gate `clicking_an_open_endpoint_reopens_the_path_to_continue_and_close` (convenção do Illustrator).
- **Furo 3:** ver (1). O filtro `VecViewState::editing` (+ `in_edit`, `EditTarget::editing`, a cópia
  em `view_state_for_pick`, as duas linhas da shell) **saiu** — controlo morto depois da decisão — com
  os 4 gates dele (`the_pick_view_carries_the_shapes_of_the_edit`,
  `an_edit_with_no_shapes_yet_grabs_nothing_from_outside`, `in_edit_only_the_shapes_of_the_mode_are_reached`,
  `in_edit_a_shape_outside_the_mode_draws_no_nodes`) e o gate de fonte
  `registry-init::the_vector_edit_reaches_the_frame_view`.
- **Furo 4 (Width/Trim):** a costura do Trim (`hit`, `apply`, `piece_world` + `hit_at` novo) desceu de
  `shells/desktop/src/vec_trim.rs` para `ph2d_app_vec::trim` (os testes foram com ela,
  `trim_tests.rs`); o press do Width é `width_handles::press_at`. **Defeito achado pelo gate:** o
  Width pressionava sobre a selecção mesmo TRAVADA (a Hierarquia selecciona uma forma travada) — agora
  só uma forma que se agarra (`PenTool::is_pickable`, novo) ganha paradas.

**(4) Smoke `PH2D_OBJECT_MODE_SMOKE=7`:** Add ▸ Vector Drawing, duas linhas que se cruzam pela
CANETA (o Edit passa à 2.ª), *Weld* pelo botão do painel, seletor aberto.
**Fotos conferidas** (tela virtual, `fotografa_cena.sh`, sobre `ececa7c58`): cena 7 — «Path 0» só na
Hierarquia, Edit Mode, gizmo à volta da rede, o nó no cruzamento; cena 6 (controlo) — «Path 2», Edit
Mode, gizmo. Medido no programa real com registo temporário (retirado, `grep -c` = 0): a cena 7 deu
`soldar: ok (1 rede de 4 arcos, 1 nó)` seguido de `heir -> <a 1.ª linha>`.

## §2 — Foundational tocado e contratos

- `ph2d-editor-core` (`mode_drive`, `object_mode::ModeState`), `ph2d-vec-scene` (`VecViewState::editing`
  removido), `ph2d-vec-edit` (`PenTool::is_pickable`), `ph2d-vec-entities` (`view_state_for_pick`).
- Contratos congelados (§6): **nenhum encostado** (o `ph2d-vector-doc` não entrou).

## §3 — Superfície de colisão

`ph2d-editor-core` (mode_drive + testes, object_mode) · `ph2d-app-vec` (vector_mode + testes +
`vector_mode_furos_tests.rs` novo, trim + `trim_tests.rs` vindo da shell, width_handles) ·
`ph2d-vec-scene` (structure) · `ph2d-vec-edit` (lib, multi_path_tests) · `ph2d-vec-entities`
(entities; `entities_pick_tests.rs` apagado) · `ph2d-vec-render` (overlays + testes) ·
`ph2d-app-registry-init` (tests/it/every_object_mode_has_a_composed_family) · shell (`vec_trim.rs`
encolhe, `despacho_clique_vetor_premido`, `fase_vector_view_and_drives`, `fase_vector_layout_recook`,
`fase_vector_click_previews`, `vec_trim_tests.rs` saiu; tests/it `the_trim_tool_owns_its_gesture`,
`the_width_tool_owns_its_gesture`) · `docs/UI_New_and_Simple/spec/06`.

**Contadores (DELTA):** nenhum (`PROJECT_SCHEMA`, registos ECS e i18n intocados).

**O que um merge pode partir:** quem leia `VecViewState::editing`/`in_edit`, `EditTarget::editing`,
`crate::vec_trim::{hit,apply,piece_world}` na shell **deixa de compilar** (falha ALTO). Uma família
nova de `ModeFamily` não precisa de nada (os dois métodos novos têm omissão).

## §4 — Fecho (DIRETRIZ §1.5.9)

**Gate batched** (`verificador`, BASE `5d596eaaf`, sobre `d75a10d76`, `load` ~35):
- nextest-impacted **17 917 / 17 923**; os 6 vermelhos eram todos desta onda e determinísticos — os
  3+2 gates que lêem a shell como texto (as agulhas `crate::vec_trim::apply(` e
  `crate::width_handles::press(` mudaram com a descida) e o gate do filtro revogado → curados em
  `afaa2a887` (15/15 verdes nos dois ficheiros).
- `CARGO_BUILD_WARNINGS=deny` check `--all-targets` e clippy `--workspace --all-targets -D warnings`:
  verdes; `fmt` (curado em `afaa2a887`); censos da árvore 12/12, 114/114; doc-index em dia.
- Catraca da shell `the_shell_only_shrinks`: **184 685 / TECTO 189 041** (folga 4 356; baixar o tecto
  é opcional).
- Depois de `ececa7c58`: `ph2d-editor-core` `mode` 94/94, `ph2d-app-vec` `vector_mode` 13/13, as
  outras famílias e o registo (`ph2d-app-registry-init`, `-flip`, `-painter`, `-components`)
  1 206/1 206; clippy `-D warnings` da `ph2d-editor-core` verde.

**Mutação** (agente `mutacao`, controlo verde com população 13 e 93/94): **8/8 sangram** — o `heir`
da família, o `holds_the_whole_kind`, o `parts`, o filtro do `press_at`, o re-trancar do `Stay`, o
`heir` no `drive`, o `Leave` do tipo e o `refused` do tipo. ⚠️ Na 1.ª rodada M5–M8 (a fundação) só
sangravam pelos gates do `ph2d-app-vec` → gate novo na própria fundação (`ececa7c58`), re-mutado:
**4/4 sangram** nele. Ele achou também que o re-trancar do `Stay` não avisava a família (agora passa
pelo `enter_with`).

**Gates (nomes):**
- `ph2d-app-vec` `vector_mode::furos_tests`: `the_weld_leaves_the_net_in_edit_with_its_gizmo`,
  `the_pen_in_edit_continues_the_open_path_and_joins_another`,
  `width_and_trim_pressed_in_edit_reach_the_shapes_and_skip_a_locked_one`.
- `ph2d-app-vec` `vector_mode::tests`: `the_edit_holds_every_shape_and_another_kind_leaves_it`
  (substitui `two_shapes_edit_in_one_and_the_other_is_untouched`), `the_pure_laws` (+ `heir`),
  `clearing_the_selection_in_edit_keeps_the_edit` (o controlo mudou: apagar a trancada passa à outra;
  sem forma nenhuma o modo cai).
- `ph2d-editor-core` `mode_drive::tests::a_mode_of_the_whole_kind_never_locks_and_hands_itself_on`.

**NÃO verificado por compilação:** o clique real no canvas (XTest ignorado na tela virtual — provado
por gate); o `Tab`/`Ctrl+Tab`/`Ctrl+Space` pelo teclado real (o despacho vive na `App`,
`input_handlers.rs:122`).

## §5 — Smoke do dono (lista técnica, para o registo)

1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. `+` da Hierarquia ▸ «Vector Drawing»; no painel «Pen»; clicar dois pontos e botão direito; outra
   linha que cruze a 1.ª (dois cliques e botão direito): duas linhas na Hierarquia, «Edit Mode».
3. No painel «Select»; Shift+clique nas duas linhas no canvas; «Weld» no painel: fica UMA linha
   («Path 0»), seleccionada, «Edit Mode», com o gizmo.
4. «Pen»; clicar na PONTA solta de uma linha aberta e depois noutro sítio: a mesma linha cresce
   (nenhuma linha nova na Hierarquia).
5. Em «Edit Mode», com «Select», clicar noutra forma: fica seleccionada e continua «Edit Mode».
6. Com uma imagem na cena (arraste uma para o canvas), em «Edit Mode» com «Select», clicar na imagem: fica seleccionada e o topo
   diz «Object Mode».

**Errado:** depois do Weld o topo diz «Object Mode»; o clique na ponta cria «Path 1»; clicar noutra
forma não faz nada; clicar na imagem dá o aviso de cadeado.

**Smoke do dono: APROVADO (05/10)** sobre `8310dc52c` (*«smoke ok»*).

## §5b — Depois do smoke: o que estava aberto (ordem do dono, 05/10: *«siga para o que está em aberto»*)

| commit | o quê |
|---|---|
| `b92cba820` | o Node que falha o vetor escolhe o objecto de outro tipo; o `Tab` só do modo (nós com `]`/`[`) |
| `1fcdc40e7` | Image ▸ Edit: o IMG vira o modo Edit da imagem; o botão sai |
| `63478a171` | fecho: o controlo do espelho (mutação M1) |

**Duas decisões do dono (05/10), perguntadas com o que se mediu:**
- *«Tab sempre troca o modo»* — o defeito MEDIDO: no Node, um braço `KeyCode::Tab`
  (`keyboard_cadeia.rs`) percorria os nós e consumia a tecla, logo em Edit com o Node o `Tab` não
  saía do modo. Os nós andam agora com `]` (frente) e `[` (trás); o `[`/`]` do Painter (tamanho do
  pincel) só consome com o Painter em mãos, e corre antes.
- O botão IMG *«vira modo da imagem»* — `ph2d_app_painter::image_edit_mode::Family` declara
  (Image, Edit): do TIPO (todas as imagens; `joins` — o *Equalize Sizes* trabalha sobre várias), com
  `heir`. O `image_edit.mode_on` (que a fila das ferramentas de imagem, o `activation_gate` e o canvas
  lêem) é o ESPELHO do modo (`mirror`, na `fase_object_mode` depois do quadro); quem larga a ferramenta
  de imagem fora do Edit continua a ser a `fase_image_tools_mode_and_pills` (que também limpa a
  pré-visualização do Bg Removal — por isso o `leave` da família não larga nada). Saíram: o pill IMG, a
  linha *Image Tools* do menu Window (13 → 12 linhas; `the_window_menu_reaches_every_module`
  actualizado), `ModuleTruth::ImageMode` (`MODULE_TRUTHS` 18 → 17), o toggle, o id `TOPBAR_IMAGE_TOOLS`
  (o `…_BACKDROP` da fila fica), 5 chaves i18n, o parâmetro `active` dos chips da barra (ficou morto) e
  os testes `click_on_image_tools_pill_toggles_mode`, `the_image_tools_chip_shows_the_mode_in_every_family`.
  O split da barra clássica 7 → 6 (nenhum pill muda de lado). O composite smoke entra em Paint pelo modo.

**A regra «outro tipo ⇒ Object» chegou ao Node** (sem pergunta: é a regra do dono):
`vector_mode::another_kind_under` (pura) + `App::vetor_node_escolhe_outro_tipo` pela porta única do
pick (`hover_highlight::pick_objects_at`). Os modos de DESENHAR não perguntam (desenhar por cima de
uma imagem é o uso).

**Fundação:** `ModeFamily::heir(mode, locked, tools)` — recebe a entidade trancada (assinatura nova,
ainda não integrada).

**Gates novos:** `ph2d-app-painter` `image_edit_mode::tests::{the_image_offers_edit_and_edit_is_the_image_tools,
the_image_edit_holds_every_image_and_another_kind_leaves_it}` · `ph2d-app-vec`
`furos_tests::a_node_miss_picks_the_object_of_another_kind_underneath` · shell `tests/it`
`the_mode_keys_are_wired::{only_the_mode_owns_the_tab, ctrl_space_toggles_the_zen}` (fecha o «gate de
teclado precisa da App»: o território do input tem UM braço `KeyCode::Tab`) e
`the_node_miss_picks_another_kind`; `the_node_selection_scale_is_wired::brackets_and_select_all_reach_the_pen_in_node_mode`
(era `tab_and_…`). `mode_drive_tests.rs` passou o tecto de 700 LOC com o gate do tipo → partido em
`mode_drive_kind_tests.rs`.

**Fecho da 2.ª leva:** gate batched (`verificador`, BASE `5d596eaaf`, HEAD `1fcdc40e7`, `load` 15–29):
nextest-impacted **17 927 / 17 927** (com as features dos painéis), check `deny` e clippy `-D warnings`
do workspace verdes, fmt, censos 12/12 (114/114), doc-index, `the_shell_only_shrinks`,
`architecture_workspace_file_loc_cap` — tudo verde. **Mutação: 8/8** depois de `63478a171` (a M1, o
espelho ler só «há modo», sobrevivia — gate de controlo acrescentado, re-mutada ⇒ RED); as outras 7
(o tipo e a herdeira da imagem, o `whole_kind` e o filtro do `another_kind_under`, a chamada no Node,
o `Tab` no Node, o `mode_key` do `Tab`) sangraram à 1.ª.
**Foto** (`PH2D_OBJECT_MODE_SMOKE=8`, conferida): «Canvas» seleccionado, a fila TRIM · SQUAR · BGRMV ·
SIZE · PAD · CEQ · EQSZ · RASTR · UPSC na barra, o seletor aberto com Object Mode · Paint Mode ·
**Edit Mode**.

**Smoke do dono, 2.ª leva (lista técnica):**
1. `cd ~/Documentos/Projetos/PH2D/Worktrees/line-UIUX && ./target/smoke/ph2d-host-desktop`
2. Ctrl+N (imagem nova), seleccioná-la, seletor do topo ▸ «Edit Mode»: aparece a fila das
   ferramentas de imagem; o botão IMG já não existe (nem no menu Window).
3. Seletor ▸ «Object Mode»: a fila some.
4. Vetor em «Edit Mode» com «Node»: `Tab` volta a «Object Mode»; `]`/`[` andam de ponto em ponto.
5. Vetor em «Edit Mode» com «Node», clicar na imagem: fica seleccionada, «Object Mode».

## §6 — O que fica ABERTO

- A booleana com SÓ duas formas cujo resultado nascesse um quadro depois repetiria o aviso «Edit
  Mode»; **medido que não acontece**: o resultado ganha entidade no mesmo `sync` que apaga as duas, e a
  herdeira segura o Edit (cena 6, registo temporário).
- spec/02: **G** (esvaziar os painéis — o censo `quantas_entradas_tem_cada_painel` põe na frente, por
  comandos, `inspector` 314 (grande por direito), `tokens` 110, `physics` 60, `vector` 45; cada um pede
  o destino de cada comando, decisão de produto por painel); **H** (separar layout de paleta) sem trava
  desde 20/09; **I** (temas): a pergunta viva é se os 4 clássicos sobrevivem ao dia em que o
  `PH2D_UI_NEW` sair — do dono, e só nesse dia. O *«como partir o `DrawMode`»* do spec/02 §5.1 ficou
  respondido pelo dono: a partição `EDIT_TOOLS` foi recusada (04/10) e o Edit é do TIPO (05/10) — as
  ferramentas todas vivem no Edit.
- As layouts F4: o campo *«modo ao abrir»* existe; a aba Vector fica `Tool("vector")` (ver spec/06 F4).

## §7 — Perfil do laço do agente

```
  ✗ paralelismo de ferramenta              1.13/passo   alvo: >= 1,5  (9% dos passos com 2+ chamadas)
  ✓ respostas por sessao (mediana)                263   alvo: <= 800
  ✗ cargo test : cargo check                784 : 229   alvo: <= 1,0  razao 3.4x (baseline: 4,3x)
  ✗ edicoes pela ferramenta Edit                  35%   alvo: >= 80%  (1845 por script; baseline: 48%)
  ✗ contexto relido por passo (media)         461 mil   alvo: <= 250 mil
  ✓ contexto no inicio da sessao               63 mil   alvo: <= 80 mil
```

## §8 — Binário de smoke

```
▸ linha line_uiux · CPU ≤ 1600% de 32 núcleos · mem ≤ 24G · prazo 1800s
    Finished `smoke` profile [optimized] target(s) in 0.21s
```
(2.ª corrida, sobre `63478a171` + este doc.)
