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

1. **`ItemId(u32)` em cada item** (`#[serde(default)]`, `0` = por atribuir), tirado do **`StableIdCounter`** do projeto — o
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

1. **Alvo de cena**: o binding com `wire_id = SCENE_WIRE` (sentinela reservada, como `SCENE = 0` do dono de clip) e
   entidade-sentinela nos `CanaisLidos/Escritos`; a dope-sheet mostra-os sob a raiz Cena (que já existe no outliner).
2. **Família `MUNDO = 9`**: os campos de `PhysicsSettings` (gravidade X/Y, ar, amortecimento, substeps, iterações, sono) e a
   matriz de camadas (discretos). `set_settings` limpa o anel de checkpoints (`settings.rs:38-50`) — **medir** se a curva
   escrevendo por quadro destrói o scrub; se sim, a escrita da curva é um caminho próprio que não limpa (o valor é função
   do quadro, o replay reproduz-se).
3. **Mesa de som vira documento**: `MixerDoc` (os ~50 campos de `snapshot.rs`) no `.ph2dproj` pelo caminho do `physics`
   (`project_save.rs:88`, `project_load.rs:173`, `project_migrate.rs`), fora do `ProjectState` como ele; o painel lê/escreve
   o documento; família `MESA = 10`. ⚠️ **muda o produto**: a mesa passa a ser guardada com o projeto (hoje volta ao
   padrão em cada sessão) — é o que a lei pede; dito ao dono no smoke.
4. O gate tira `ph2d-panel-physics` e `ph2d-panel-audio-mixer` de `NaoAnimavel`.

### C — o corpo vivo obedece à curva a meio da simulação

Gémeo de `retune_joint`: `bodies.rs` deixa de exigir repouso para os campos com setter vivo no rapier (densidade, atrito,
restituição, escala de gravidade, amortecimento, sensor, CCD, camadas, tipo) e aplica-os NO LUGAR, sem limpar o anel; a forma
(`set_shape`) idem. Gates: `a_curva_muda_o_atrito_a_meio_da_queda`, e o `scrubbing_backwards_reproduces_the_tick_exactly`
com uma curva a mudar a densidade (o replay reproduz-se porque o valor é função do quadro).

### D — o chip e o valor derivado

- **IkBendSide** entra em `AUTOKEYED` e o chip ganha losango; a faixa passa a três escolhas (`< -0.5` Cw · `> 0.5` Ccw ·
  resto Keep) — as chaves antigas (±1) mantêm o sentido.
- **Rope Length**: a mão e a ponte distinguem-se na FONTE: a edição da caixa marca o campo como «da mão» neste quadro
  (`maos_do_quadro`, que o amostrador já une), e a exclusão da polia passa a valer só quando não há mão.

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
