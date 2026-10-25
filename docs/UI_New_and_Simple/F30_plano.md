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
