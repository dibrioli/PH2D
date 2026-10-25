# F30 — a lista FORA do Auto Key (plano, 25/10)

Fila: [F29 §6.1](handoffs/HANDOFF_CONTINUACAO_line_UIUX_2026-10-25_F29.md) — ordem do dono (25/10): *«Smoke OK. Vamos voltar
ao plano antes dos consertos realizados»*; lei: [o Auto Key grava ABSOLUTAMENTE tudo](../../project-memory/project_autokey_records_everything_2026_10_19.md).
Base `c3662cd8e4` (ponta da linha `8042c8862a`).

⚠️ **O item 2 da fila do briefing (achados do censo da palavra, F27 F §6.3) já está CURADO** pela F28
([F28_plano.md §A](F28_plano.md)): os sete gates existem (`vocabulario_tests.rs`, `canal_palavra_tests.rs`,
`ph2d-timeline/tests/it/canal_field.rs`), 9/9 mutações sangraram. Não se refaz.

## 0 — Medição: o painel escreve onde · quem lê · o leitor decide?

| item | onde mora | escritor | leitor (decide?) | porque está FORA hoje |
|---|---|---|---|---|
| **Timers** | `Timers(Vec<Timer>)` `ph2d-ecs/src/timer.rs:182` | `inspector_timer.rs:84` por ÍNDICE | `timer_tick.rs:65` a cada tick — **decide** | a `FieldKey` COMPONENTE gasta `i:j` no id do componente: não há onde dizer «o item n» (ADR-0191) |
| **Tweens** | `Tweens(Vec<Tween>)` `tween.rs:52` | `TweenFieldEdit(i,…)` | `fase_tweens.rs:34` por quadro — **decide** | idem; ⚠️ o tween `i` usa o `Timers[i]` (acoplamento por posição, `tween.rs:95-107`) |
| **StateMachine** | `states`/`transitions: Vec`, `from/to/initial: u8` (índices) `state_machine.rs:114` | `statemachine_edits.rs:70` | `state_machine_tick.rs:81` — **decide** (ordem das setas = precedência) | idem; o catálogo não tem descritor de seta (`logic.rs:72-79` mistura os dois) |
| **CounterWatch** | `Vec<CounterWatchRow>` `counter_watch.rs:142` | `counter_watch_edits.rs` | `counter_watch_bridge.rs:53,97` por quadro — **decide** | idem |
| **SignalActions** | `Vec<SignalAction>` `signal_actions.rs:465` | `action_trigger_edits.rs:13` | `signal_actions.rs:560` por sinal — **decide** | idem |
| **ShakeEmitter** | `Vec<ShakeSource>` `camera_shake.rs:178` | `shake_edits.rs` | `shake_bridge.rs:107` por quadro — **decide** | idem |
| **Resistências** | `Health.resistances: Vec<Resistance>` `ph2d-physics-ecs/src/components/health.rs:111` | `vida_inspector.rs:203-224` | `bridge/health.rs:287,354` por dano — **decide** | idem; sem descritor (`logic.rs:617` só esconde) |
| **Âncoras** (`sections/anchors.rs`) | `NamedAnchorList(SmallVec)` `named_anchor.rs:284` | a secção | — | o gate chama-lhe `VecAnchors`: ⚠️ **erro de nome** — `VecAnchors` (`vec_anchors.rs:74`) não é lista e é INERTE (`hud_anchors.rs:1-12`) |
| **Pilha do Vetor** (tinta, efeitos) | `p.paints`/`p.effects` | `paint_stack.rs:221-250`, `fx_bridge.rs:106-131` (`remove`/`swap`) | o funil VETOR — **decide** | ⛔ **DEFEITO NOVO, mesma família:** `i` = posição (`canal_vetor.rs:9-14`); apagar/mover uma entrada **desloca as chaves para a vizinha** e nada toca a timeline (só `canais_topologia::propagar` re-chaveia, e só pontos). Nenhum gate. O Flip já faz certo: `i` = `LayerId` durável (`canal_flip.rs:10`). |
| **Gravidade e mundo da física** | `PhysicsSettings` no `.ph2dproj` (`project.rs:58`) | `PhysicsIntent::SetSettings` → `bridge/settings.rs:38` | `world.set_gravity` — **decide** no passo seguinte, sem guarda de repouso | não há alvo de CENA na timeline (`SCENE = 0` só é dono de CLIP, `doc_owner.rs:42`); o gate dá o painel como `NaoAnimavel` |
| **Mesa de som** | thread-local do painel (`snapshot.rs:15-58`), **não viaja no projeto** nem nas prefs | `event.rs:28` | `fase_audio_mixer.rs:8-89` todo quadro, com rampa no ganho — **decide**; o jogo passa pelos sub-bus (`scene.rs:110,212`) | sem documento e sem alvo de cena |
| **Física a meio** | `BodyDesc` (`ph2d-physics/src/world/desc.rs:239`) | a curva escreve o componente (`canais_corpo.rs:91`) | `bodies.rs:172-176`: só re-descreve **em repouso**, e recriando o corpo | a junta já o faz no lugar (`joints.rs:500-520`, `retune_joint`, sem limpar o anel); rapier 0.35.3 tem setter vivo para quase tudo |
| **Rope Length** (polia) | `PhysicsJoint.max_length` | caixa do Inspector · e a PONTE re-deriva (`joint_wheel.rs:189`, `rope.rs:371`) | — | o amostrador só vê o VALOR (`autokey_amostra.rs:21-26`): excluído para não gravar a ponte como mão |
| **IkBendSide** | `IkGoal.bend` (`Keep/Cw/Ccw`) | chip `ph2d-panel-skeleton/src/state.rs:169` | `goal.rs:495` por quadro — **decide** | fora de `AUTOKEYED` de propósito (`prop.rs:170-182`): «cada troca viraria chave» — que é exactamente o que a lei do dono pede |
| `ph2d-panel-painter-layers` | — | — | — | ⛔ **NÃO É DESTA LINHA** (dono, 23/10: volta com a `line/PainterWatercolor`, que está VIVA a mexer nesse painel hoje) |
| Rack de efeitos de áudio | cozido no buffer do clip (`fx_rack.rs:192-210`, passo de undo) | — | — | **não é defeito**: edita o asset de som, não a cena (`NaoAnimavel` certo) |

## 1 — As famílias (CLAUDE.md §0.10) e a ordem

São QUATRO mecanismos, não catorze itens. Ordem por dependência: D e C são pequenos e isolados; B dá o alvo de CENA; A é o
maior e o único com formato novo em muitos sítios.

### A — o item de uma lista tem IDENTIDADE (listas dos componentes + pilha do Vetor)

Lei: **o endereço de uma chave é a identidade do item, nunca a posição** (o Flip já o faz). Desenho:

⚠️ Medido (25/10): os componentes gravam-se em **postcard posicional** pelo registo (`scene/registry.rs:65,92,226`) ⇒ um
campo novo no item é **formato**: `PROJECT_SCHEMA` sobe e cada um dos 8 componentes ganha migração (precedente:
`script_project.rs:98-110`). A pilha do Vetor é `PaintEntry`/`FxEntry` em `ph2d-vec-scene` (`VEC_SCENE_SCHEMA_VERSION = 23`,
`schema.rs:90`). A cópia de objecto (`deep_copy_subtree`, `instantiate.rs:231-285`) copia componentes por bytes: os ids de
item viajam com a cópia ⇒ o passe de atribuição tem de REFAZER os ids de item repetidos dentro do projecto (o primeiro na
ordem de `StableId` do dono fica). Criação de itens: o Inspector de cada componente (`vida_inspector.rs:206`, `event_*`),
`anchor_mount.rs:270,366,397`, a migração `signal_actions_v1.rs:54`.

1. **`ItemId(u32)` em cada item** (`0` = por atribuir), tirado do **`StableIdCounter`** do projeto — o
   contador que já é persistido, monotónico por construção e FORA do undo (`stable_id.rs:102-150`): um id nunca se reusa,
   logo uma chave órfã nunca «acorda» noutro item. Redo devolve o item com o MESMO id (o undo guarda o estado).
2. **Atribuição** num passe idempotente gémeo de `assign_missing_stable_ids`, que **reconcilia o contador contra os ids de
   item** antes de alocar. Documento antigo: os itens recebem ids na ordem da lista (determinístico).
3. **Família nova `LISTA = 8`**: `campo = lista << 10 | membro << 2 | faixa` (`lista` ≤ 63 = código DECLARADO, append-only,
   da lista — Timers, Tweens, estados, setas, regras, acções, fontes de shake, resistências, âncoras; `membro` ≤ 255 = o
   `field_id` do item no catálogo); `i:j` = o `ItemId` (32 bits). A entidade é a do binding. Item apagado ⇒ chave órfã
   (a lei existente dos órfãos).
4. **O catálogo descreve o ITEM**: cada lista ganha um descritor de item com `named_as` (as chaves serde estão vazias hoje,
   `field.rs:183-188`); as setas da StateMachine e as resistências ganham o seu. Gate de censo: todo `Hidden::List` /
   `hiding(…)` de lista tem código de `LISTA` (componente novo com lista reprova até escolher).
5. **Funil** `ph2d-app-components/src/canais_lista.rs` no molde de `canais_script.rs` (publicar · aplicar · `PreviewDrive`).
6. **Losango**: a fileira do item regista o canal explícito (`key_mark::regista_canal`, o caminho do script).
7. **Vetor**: `PaintEntry`/efeito ganham `ItemId`; as chaves VETOR da pilha passam de posição a id. É **formato**:
   `TimelineDoc` 23 → 24 com migração no load (posição `k+1` → id da entrada `k` lida no mundo carregado), fixture v23.

Gates (vermelhos por mutação antes):

| gate | mutação |
|---|---|
| `apagar_um_item_do_meio_nao_muda_a_chave_do_vizinho` (LISTA) | `ItemId` substituído pela posição |
| `apagar_uma_entrada_da_pilha_nao_muda_a_chave_da_vizinha` (VETOR) | volta `i = k+1` |
| `um_id_de_item_nunca_se_reusa` | o passe aloca `max+1` em vez do contador |
| `o_contador_reconcilia_com_os_ids_de_item` | o passe não lê os itens antes de alocar |
| `toda_lista_do_catalogo_tem_codigo` | apaga um código |
| `o_doc_v23_migra_para_v24` | a migração mantém a posição |

### B — a CENA é alvo da timeline (gravidade e mundo da física, mesa de som)

Medido (25/10): todo `AnimTarget` vem de `bind(entity, prop)` (`doc.rs:499`); o binding resolve `wire_id → entity` pelo
`StableId` (`persist.rs:45-134`, órfão = `entity 0, missing`), e só uma purga o apaga. A vista da Cena mostra os bindings
cujas faixas estão em clips de dono `None` (`animation_view.rs:162-177`). A população do Auto Key é selecção + mãos
(`autokey_amostra.rs:60-72`); a chave cai no clip activo (`doc.rs:462-467`).

1. **A entidade CENA**: uma entidade ECS com `StableId::CENA` (reservado, fora da faixa do contador) e o marcador `Cena`,
   SEM `Transform` — por isso nunca entra no save nem na hierarquia (a DFS do snapshot parte das raízes com `Transform`) —
   garantida por um passe idempotente a cada quadro (o undo/load que reconstrói o mundo apaga-a; o passe repõe-na). O
   binding dela tem `wire_id` normal: resolução, cura e persistência são as que existem. A população do Auto Key inclui-a
   sempre (poucos campos). A chave de um campo da Cena cai num clip da Cena (dono `None`) — o caminho a medir no
   `autokey_campos`/`upsert_key`.
2. **Família `MUNDO = 9`** sobre a entidade CENA: os campos de `PhysicsSettings` (`settings.rs:~175`: gravidade X/Y,
   substeps, iterações, contact_hz, amortecimento linear/angular, sono, ar; a matriz de camadas é discreta). ⛔ `set_settings`
   limpa o ring de checkpoints (`bridge/settings.rs:38-50`): a escrita da CURVA é um caminho próprio, só quando o valor
   muda, que reafina o mundo sem limpar (gémeo do `retune_body`). O valor AUTORADO (o que o save e o undo vêem) fica à
   parte, como a `ConducaoFisica` dos corpos: a curva é pré-visualização.
3. **Mesa de som vira documento**: `MixerDoc` (os ~50 campos de `snapshot.rs`) no `.ph2dproj` pelo caminho do `physics`
   (`project_save.rs:88`, `project_load.rs:173`, `project_migrate.rs`), fora do `ProjectState` como ele; o painel lê/escreve
   o documento; família `MESA = 10`. ⚠️ **muda o produto**: a mesa passa a ser guardada com o projeto (hoje volta ao
   padrão em cada sessão) — é o que a lei pede; dito ao dono no smoke.
4. O gate tira `ph2d-panel-physics` e `ph2d-panel-audio-mixer` de `NaoAnimavel`.

✅ **§B1 feito** (a Cena + o MUNDO; a mesa §B2 fica). Medido e decidido:
- **A Cena** (`ph2d-ecs/src/cena.rs`): `Cena` + `StableId::CENA = u64::MAX >> 1` (`stable_id.rs`, `reservado()`); a
  reconciliação do contador ignora a faixa reservada e o `take` recusa-a (`o_contador_nunca_alcanca_o_id_da_cena`).
  ⚠️ **Facto que o plano não tinha:** «sem `Transform` ⇒ fora do save» vale para o `world_to_snapshot`, mas a captura
  INCREMENTAL do undo (`scene/incremental.rs:~320-345`) leva TODA entidade com `StableId` — a Cena entraria no passo de
  undo e o restauro (`undo.rs:218`, que só limpa `With<Transform>`) duplicá-la-ia a cada Ctrl+Z. Cura: `Without<Cena>` nas
  duas consultas dela (o único caso especial; gate `a_cena_nao_vai_para_o_ficheiro`, os dois caminhos). O restauro do
  undo NÃO a apaga (medido: só despawna `With<Transform>`); o load sim.
- **O passe** `garante_cena` corre no `ph2d_timeline_persist::upkeep` (depois do `assign_missing_stable_ids`, antes da
  cura): é a função de identidade que a shell chama a cada quadro (`timeline_bridge::run`), e pô-la ali torna IMPOSSÍVEL a
  purga apagar a binding da gravidade num quadro em que a Cena ainda não voltou. Resolução/cura/persistência sem caso
  especial (`a_binding_da_cena_atravessa_o_ficheiro_e_recola`: `wire_id == StableId::CENA`, recola num mundo novo).
- **Família `MUNDO = 9`** (`8` fica para a `LISTA` da §A): `canal_mundo.rs`, 11 campos (gravidade X/Y num grupo,
  sub-passos e iterações `Numero`, contacto, amortecimentos, ar, sono velocidade/demora, «Bodies Sleep» `Ligado`).
  Matches tocados: `canal.rs` `info` e `FieldKey::membro` (os dois únicos `match familia`), + `FieldKey::da_cena`.
  ⛔ A matriz de camadas fica fora (36 células sem controlo de campo; `NaoAnimavel` implícito — o gate não a vê).
- **O autorado** é `PhysicsBridge::settings` (o que `project_save.rs:91` grava e o painel edita); a curva vai para um
  `vivo` novo da ponte por `conduzir_settings` (só quando muda, ⛔ sem limpar o anel; `rebuild`/`rewind` nascem do
  `vivo`). Gates: `a_curva_muda_a_gravidade_do_mundo_sem_limpar_o_scrub` (controlo: `set_settings` limpa),
  `o_save_grava_o_autorado_com_uma_curva_activa`. O painel PINTA o `vivo` e escreve sobre o autorado.
- ⚠️ **A ordem do quadro decide o eco:** o painel aplica `SetSettings` DEPOIS do Auto Key (`fase_tool_mirrors`, após
  `fase_hero_commits`) e o dreno da curva corre ANTES dele no quadro seguinte — um eco ingénuo (`eco(curva, autorado)`)
  esconderia a mão para sempre. Cura: `ConducaoMundo` lembra o autorado do quadro anterior; o quadro em que ele mudou a
  MÃO ganha à curva (no mundo e em `CanaisLidos`) e o Auto Key grava-a (`a_mao_ganha_a_curva_no_quadro_em_que_muda`).
  Sem escrita da curva, o mundo corre o autorado.
- **A chave da Cena** cai no clip ACTIVO (`doc.rs:556`); o clip activo é da animação em vista (`animation_view.rs`):
  com um objecto COM Animação seleccionado, cairia no clip dele. Cura: `autokey_campos` recusa um campo `da_cena()` fora de
  um clip da Cena (`KeyRefusal::SceneNotInView`, toast «show the Scene in the Timeline»; `AutokeyPlan::scene_refused`);
  o losango do painel não grava ali (`o_clique_da_cena_nao_grava_no_clip_de_um_objecto`). Selecção sem Animação ⇒ a vista
  é a Cena ⇒ grava. A dope-sheet da Cena mostra as faixas (`a_dope_sheet_da_cena_mostra_as_faixas_do_mundo`), rotuladas
  «Scene» (`tracks_label.rs`).
- **Gate:** `ph2d-panel-physics` sai de `DIVIDA_CRATE` para o `ESCOPO`; tabela `ph2d-app-physics/src/marcas_mundo.rs`;
  `paint.rs::paint_row` = Canal; `paint_irow`, `seg_row` (ferramentas) e `PHYSICS_SHOW_COLLIDERS` (debug) = `NaoAnimavel`.
- O load repõe o `ConducaoMundo` (`project_load.rs`): o autorado do ficheiro não é a mão.
- ⚠️ **Aberto:** o scrub re-simula do checkpoint com o `vivo` do alvo, sem reler a curva por passo (a mesma nota da §C).

### C — o corpo vivo obedece à curva a meio da simulação

✅ **Feito** (`419e93f20e`): `PhysicsWorld::retune_body` (o colisor copiado do MESMO `build_collider`; desmonte exaustivo do
`BodyDesc` = campo novo é erro de compilação) e o braço `!at_rest` da ponte. Achado pelo gate: travar um eixo a meio deixava
a velocidade nele — a lei do `spawn_body` vale também no retune. Gates: `reafinar_um_corpo_vivo_e_o_mesmo_que_cria_lo`
(19 campos, cada um VISÍVEL na cena — a CCD num tiro de 600 m/s, a camada contra uma matriz que a separa),
`a_escala_de_gravidade_muda_o_corpo_a_meio_da_queda`.

⚠️ **Aberto, comum a juntas e corpos:** o scrub reconcilia UMA vez no alvo e re-simula do checkpoint sem reler a curva por
passo — um material animado entre o checkpoint e o alvo re-simula com o valor do alvo (até `STRIDE` passos).

### D — o chip e o valor derivado

✅ **Feito** (`2f1442d401`):
- **IkBendSide** entra em `AUTOKEYED`. ⛔ `Keep`/`Mixed` continuam fora do canal — recusa MEDIDA no doc do `PropKind`
  (animá-los seria gravar a AUSÊNCIA de uma escolha). O painel do esqueleto não tem losango em campo NENHUM (0 PORTAS no
  gate): o losango do chip é dessa dívida, não desta.
- **Rope Length**: a ponte anota o `L0` que semeia da rota (`ComprimentoDaRota`, `joints.rs:~645`, o ÚNICO sítio que escreve
  `max_length` da rota) e o amostrador só o ignora enquanto a corda tem esse valor — o que a mão digita depois grava-se.

### E — report do dono (25/10): o caminho em construção é INVISÍVEL quando o estilo é transparente

*«As ferramentas vetoriais não desenham uma linha virtual. Se o stroke e o fill estiverem transparentes, nada pode ser
visto enquanto se desenha. Ex.: com o width do stroke em 0, a Pen mostra apenas handles.»*

**Medido:** o path em construção é um path normal da cena, pintado só com o estilo real (`ph2d-vec-edit/src/pencil.rs:9-11`,
`pen_support.rs:14-17`); as alças/nós vêm de `ph2d-vec-render/src/overlays.rs:57` com cores FIXAS (`:151-155`); o único
contorno independente do estilo é o do hover (`hover_outline.rs:18-23`, 1,5 px, cor fixa — ⛔ HR-15) e só com o cursor
em cima; uma forma seleccionada mostra só nós; não há vista Outline.

**Os grandes apps (o idioma):** o guia do gesto é um desenho DA FERRAMENTA, separado do estilo do objecto — o Inkscape
pinta uma linha-guia própria enquanto se desenha (o traço real só no fim); o Figma pinta o contorno na cor de selecção e
uma linha azul da última âncora até ao cursor; o Illustrator mostra o path seleccionado na cor da camada (e o Lápis com
Stroke «None» ficar invisível é reportado pelos utilizadores como defeito).

**Cura (um bloco):** o **fio do caminho** — 1 px de ecrã, cor do token de selecção do tema — pintado (1) sobre o path em
construção de TODA ferramenta de desenho vetorial (Pen, Pencil, Shape, Connect, Curvature…), (2) sobre as formas
seleccionadas, (3) a linha da última âncora ao cursor no Pen; e as cores fixas do `hover_outline`/`overlays` passam a
tokens. Gates: `um_caminho_sem_traco_nem_preenchimento_ve_se_enquanto_se_desenha` (pixels do overlay ≠ fundo com width 0
e alpha 0), `a_forma_seleccionada_mostra_o_fio`, e o censo de cor crua no overlay vetorial.

**Feito (25/10).** Medido antes de construir:
- **Ferramentas com path em construção** (o `DrawMode`, `ph2d-tool-vector/src/params_mode.rs:14`, tem 17 modos; não há
  ferramenta «Curvature» — só o parâmetro de forma `shapes.rs:489`): Caneta e a lâmina do Corte (`PenTool::active_path`,
  `ph2d-vec-edit/src/lib.rs:209`; o caminho cresce sempre pela cauda, `pen_press.rs:76`), Lápis/QuickShape
  (`pencil.rs:124`), Shape/Frame (`shape.rs:143`) e Conector (`ConnectorDrag::path`, `ph2d-app-vec/src/connector_drag.rs:19`).
  Todos põem o path NA cena no press — o fio pinta-o pela mesma projecção das âncoras (`overlay_transform`).
- **O *rubber band* da Caneta NÃO existia** (o único «rubber band» do repo é a caixa de seleção da shell,
  `app_state.rs:864`). Agora: da última âncora (com a alça de saída) até ao cursor, só com a mão solta e no modo Pen/Cut.
- **O token:** `accent` (o das alças do gizmo, `ph2d-editor-core/src/gizmo/paint.rs:248`), e não a `selection`: esta é o
  acento a 27,5 % sobre a base do chrome (`ph2d-tokens/src/derive.rs:541`), um tinto de linha de lista que a 1 px some.
- **Seleção ↔ fio:** `pen.selected_paths()` é o espelho do gizmo (`selection_sync.rs`), então o fio da forma
  seleccionada vale em todos os modos (no Select também, como no Figma); fora do `overlay.edit`, por baixo dos nós.

Construído: a porta `ph2d-vec-render/src/path_wire.rs` (`stroke_screen` — também a do hover e do Trim —,
`draw_path_wires`, `draw_pen_reach`, `token_ink`); quem escolhe os caminhos é `ph2d-app-vec/src/edit_marks.rs`
(`wire_ids`, `draw_wires`), chamado uma vez em `fase_vector_edit_overlay`. O hover passou a `accent`, o Trim a `danger`,
os nós/alças do `overlays.rs` a `accent`/`accent-soft`/`warn`/`text-1`. A shell ficou com **−3** linhas (cortei prosa
histórica nas duas fases tocadas: a catraca `the_shell_only_shrinks` estava no tecto exacto, 189 041).
Cena de smoke `PH2D_OBJECT_MODE_SMOKE=39` (`vector_mode_fio.rs`).

**Gates:** os dois nomeados (`edit_marks_tests.rs`; o oráculo é o buffer do Vello — a tinta do `accent`, que não é
transparente nem a do `canvas`, e a caixa dos pontos em px de ecrã cobre o caminho; controlo: sem gesto nem seleção, nada)
e `o_overlay_vetorial_nao_tem_cor_crua` (`ph2d-vec-render/src/cor_crua_tests.rs`, catraca exacta por ficheiro).
**Dívida contada que fica** (outros overlays com RGBA fixo, cada um com cor semântica própria a decidir):
`connector.rs` 5 · `marquee.rs` 4 · `cut_line.rs` 2 · `gradient.rs` 2 · `guides.rs` 2 · `symmetry_overlay.rs` 1.

### F — report do dono (25/10): arrastar vários na hierarquia leva só um

**Mecanismo.** O gesto inteiro tinha UM sujeito em cada degrau e nenhum deles lia a selecção:
`HierarchyDragState.dragged` (`ph2d-editor-core/src/interaction/drag.rs:182`) → o `WidgetEvent::HierReparent`
que o `pointer_up` emite (`interaction/dispatch/pointer_up.rs:170–219`) → o `HierReparentIntent` do painel
(`ph2d-panel-hierarchy/src/event.rs:203`) → o dreno da shell `drain_reparent`, que resolvia `intent.dragged` e
mexia só nessa entidade (`shells/desktop/src/hero_intents/hierarchy.rs`, antes `:46–48`). A selecção vive no
`GizmoStateGroup` (`selection` + `extra_selection`), que o dreno nunca recebia. ⚠️ Emitir N eventos não curaria: o
barramento guarda um pedido por quadro (`fase_bus_hierarchy.rs:35`, `get_or_insert`).

**Cura** (uma porta): a lei saiu da shell para `ph2d-app-components/src/hierarchy_reparent.rs` —
`quem_viaja(agarrado, seleccionados)` (agarrada fora da selecção ⇒ só ela; um seleccionado com antepassado
seleccionado viaja dentro dele; ordem = `ph2d_ecs::list_order`, não a do clique) e `reparentar(quem, queda)`
(atómico: ciclo ⇒ `RecusaCiclo`, peça de receita ⇒ recusa com a voz de sempre; o bloco entra no lugar do alvo contando
só os que ficam; `RootOrder`/`SiblingOrder` reescritos; pose de MUNDO re-resolvida por peça). O dreno passa a receber
os bits da selecção (`fase_vector_tree_settle.rs`). Num quadro ⇒ um passo de undo. O painel apaga todas as linhas que
viajam (`paint.rs`, `row_rides_the_drag`). De graça: a ordem entre irmãos passa a ler `ordered_children` (o dado), não a
lista `Children` do bevy, e largar DENTRO escreve o `SiblingOrder` (antes ficava o do pai antigo).

**A família (§0.10):** largar antes/depois (reordenar) e na raiz (des-parentar) — o mesmo intent e o mesmo dreno, curados
juntos (testes `reordenar_varios_*`, `largar_varios_na_raiz_*`) · *Remove from Sheet* (menu) — mesmo dreno; leva as
peças seleccionadas que estão numa folha, só essas (`fase_sprite_precision_emissive.rs`) · arrastar para o canvas/outro
painel — não reparenta (`hierarchy_drop_in_own_panel`), fora da família · camadas do Painter (`PanelRowReparent`) — outra
estrutura, sem selecção múltipla de linhas · modo fixture do store (`pointer_up` move só o agarrado no `hierarchy_order`)
— é o espelho visual sem ECS, re-sincronizado da árvore no quadro seguinte.

**Testes:** `hierarchy_reparent_tests.rs` (7: `largar_varios_seleccionados_num_objecto_torna_os_todos_filhos`,
`largar_varios_mantem_a_ordem_e_o_lugar_no_mundo`, `nao_se_larga_um_objecto_dentro_de_si_proprio`,
`agarrar_uma_linha_nao_seleccionada_leva_so_ela`, `um_seleccionado_filho_de_outro_seleccionado_vai_junto_com_o_pai`,
`reordenar_varios_entre_as_raizes_leva_o_bloco_para_o_alvo`, `largar_varios_na_raiz_tira_os_todos_do_pai`) ·
`largar_varios_e_um_passo_de_undo` (shell, `hierarchy_order_tests.rs`, pelo dreno real + `ProjectState::restore`) ·
`arrastar_uma_seleccionada_apaga_todas_as_seleccionadas` (painel). O censo
`the_drag_asks_the_door_before_changing_the_parent` re-ancorado na casa nova (3.ª mudança).

## Smoke

`PH2D_AUTOKEY_SMOKE=35` (a 34 é da F28; conta-se no despachante): um objecto com dois Timers e uma forma vetorial com três
preenchimentos animados — apagar o do meio e ver que os outros continuam a mudar como antes; a gravidade a inverter-se a meio
da queda por uma chave. Fotografada antes de ir ao dono.

## ⛔ Recusas MEDIDAS

- **Posição + re-chavear ao apagar** (evento «item movido» que a timeline consome): todo caminho estrutural (Inspector,
  script, macro, undo) teria de o emitir, e o undo da timeline é OUTRA pilha (`project.rs:30-49`, `undo.rs:247`) — um
  apagar desfeito deixaria as chaves re-chaveadas. Falha silenciosa por omissão.
- **Id de item `max+1` por lista**: reusa o id do último apagado e a chave órfã dele acorda no item novo.
- **Pôr o id do item em `i:j` da família COMPONENTE**: `i:j` já são o id do componente (ADR-0191).
- **Mexer em `ph2d-panel-painter-layers`**: é da `line/PainterWatercolor` (dono, 23/10), viva.

## Colisão

`line/GameEngineFuncional` (viva, base `c3662cd8e4`) mexe em `timer_tick.rs`, `signal_actions_bridge.rs`, `ph2d-ecs/src/lib.rs`,
`ph2d-app-vec/src/{lib,state,vector_mode}.rs`: os LEITORES das listas não se tocam nesta onda; o `ItemId` entra nos tipos.
