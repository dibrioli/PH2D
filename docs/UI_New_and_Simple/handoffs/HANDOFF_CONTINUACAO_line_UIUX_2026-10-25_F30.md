# HANDOFF DE CONTINUAÇÃO — `line/UIUX`, 25/10 — F30: a lista FORA do Auto Key — itens com identidade, a Cena e a mesa de som como alvo, o corpo vivo, o fio do caminho

- Branch `line/UIUX`, worktree `Worktrees/line-UIUX`. Base: `main` `c3662cd8e4` (merge-base). A linha tem os commits da F28 + F29 + F30,
  **NÃO integrados** (à frente do main). Ponta: o HEAD desta linha (ver `git log -1`).
  Anterior: [F29](HANDOFF_CONTINUACAO_line_UIUX_2026-10-25_F29.md).
- Mecanismo medido de cada parte, recusas e cura: [`../F30_plano.md`](../F30_plano.md). **Não está copiado aqui.**
- Estado: construído, gates vistos vermelhos por mutação; **smoke do dono (§4) OK (25/10)** — Teste 1 passos 2, 4, 5 OK; o passo 3 tinha
  um defeito, curado, re-smoke OK; Teste 2 OK. Próxima LLM: janela nova, **F31 = §A2** (§6.1).
- O censo da palavra (F27 F §6.3) já estava curado na F28 ([F28_plano §A](../F28_plano.md)); não se refez.

## 1 — Commits (`git log --oneline 8042c8862a..HEAD`, só a F30, do mais antigo)

| commit | o quê |
|---|---|
| `247cb20038` `6a1996076c` | `docs`: plano — a lista FORA medida em quatro famílias; §C e §D feitos; desenho de §B e §A |
| `2f1442d401` | **§D** `feat`: `IkBendSide` em `AUTOKEYED` + Rope Length da mão (`ComprimentoDaRota`) |
| `419e93f20e` `cd87e0e3f0` | **§C** `feat`+`test`: `PhysicsWorld::retune_body`; a gravidade que volta ao repouso volta ao corpo |
| `7d15c68227` `9dd7673515` `6a1d39621f` `0d29e4387a` | **§E** (report do dono): o fio do caminho · foto da cena 39 · oráculo fixa o ACENTO |
| `25245bb9cc` `8f9a1ffa21` | **§B1** `feat`+`test`: a Cena é alvo da timeline (gravidade e mundo) · o anel do scrub mede-se no quadro da mudança |
| `757502aac7` | **§F** (report do dono): arrastar várias linhas na Hierarquia leva TODAS |
| `a509c21b8a` | **§B2** `feat`: a mesa de som é documento e canal da Cena |
| `b21477c738` `27917418f9` `7ed6648b0f` `fa7cd7600c` `db2df751b3` | **§A1**: `ItemId` nos dez itens + degrau 194→195 · família LISTA · losango por item · posição de agora no dope-sheet · fecha 3 furos da mutação |
| `c51e2eac6f` `4197cc8569` | `docs` §A2: a pilha do Vetor parada; decisão técnica e adiamento para a F31 |
| `c73079bb2e` | `test`: cena `PH2D_AUTOKEY_SMOKE=35` (foto `img_F30/cena35.png`) |
| `59fed12a1e` | `chore`: fecho da onda — fmt e clippy |
| `1037e54bd9` | `fix` (smoke do dono): o rebobinar dos scripts não devolve o que a CURVA conduz |
| (este) | este handoff |

Reports do dono a meio da onda: §E (linha virtual do caminho) · §F (arrastar vários na hierarquia) · smoke (o Long congela, §2.6).

## 2 — O que mudou

### 2.1 §D — IkBendSide e Rope Length
`IkBendSide` entra em `AUTOKEYED`; `Keep`/`Mixed` ficam fora do canal (gravá-los = gravar a AUSÊNCIA de uma escolha). O chip não tem losango:
o painel do esqueleto não tem losango em campo NENHUM (§6.6). Rope Length: a ponte anota o `L0` que semeia da rota (`ComprimentoDaRota`,
`joints.rs:~645`, único sítio que escreve `max_length` da rota) e o amostrador só o ignora enquanto a corda o tem — o que a mão digita grava-se.

### 2.2 §C — o corpo vivo obedece à curva a meio da simulação
`PhysicsWorld::retune_body` reafina no lugar (colisor copiado do MESMO `build_collider`; desmonte exaustivo do `BodyDesc`: campo novo = erro de
compilação); braço `!at_rest` da ponte. Achado pelo gate: travar um eixo a meio deixava a velocidade nele (lei do `spawn_body` vale no retune).

### 2.3 §B1 — a Cena é alvo; família MUNDO = 9
- `ph2d-ecs/src/cena.rs`: `Cena` + `StableId::CENA = u64::MAX >> 1` (`reservado()`; o contador a ignora e o `take` recusa-a). Passe
  `garante_cena` dentro de `ph2d_timeline_persist::upkeep`. ⚠️ A captura INCREMENTAL do undo (`scene/incremental.rs:~320-345`) leva toda
  entidade com `StableId`: `Without<Cena>` nas duas consultas (gate `a_cena_nao_vai_para_o_ficheiro`).
- `canal_mundo.rs`: 11 campos de `PhysicsSettings` (matriz de camadas fora). Autorado = `PhysicsBridge::settings`; a curva vai para o `vivo`
  por `conduzir_settings`, só quando muda e **sem limpar o anel** (`set_settings` limpa). `ConducaoMundo` lembra o autorado do quadro anterior
  (a mão ganha à curva no quadro em que muda: o painel aplica `SetSettings` DEPOIS do Auto Key).
- A chave da Cena cai no clip activo; com um objecto de Animação própria em vista cairia no dele ⇒ `KeyRefusal::SceneNotInView` (toast
  «show the Scene in the Timeline»; `AutokeyPlan::scene_refused`; o losango recusa igual). A dope-sheet da Cena rotula «Scene».
- Gate: `ph2d-panel-physics` sai de `DIVIDA_CRATE` para o `ESCOPO`; tabela `ph2d-app-physics/src/marcas_mundo.rs`.

### 2.4 §B2 — a mesa de som é documento; família MESA = 10
`ph2d_audio::Mesa` (`crates/ph2d-audio/src/mesa.rs`; a crate ganhou `serde`+`postcard`); recurso `MesaDoc { autorado, vivo, conducao }` no
`SimWorld` (`ph2d-app-audio/src/mesa_doc.rs`); `ProjectFile.mesa: Vec<u8>` no fim, **`PROJECT_SCHEMA` 193→194** (leitura dos anteriores em
`project_load_read.rs:144`). ⚠️ **Muda o produto:** a mesa passa a guardar-se com o projecto. Sem undo (como o `physics`). Família
`canal_mesa.rs`: 19 campos do master + `bus(b, f) = 32 + 16·b + f`. Funil `canais_mesa.rs`; losango `marcas_mesa.rs`. Medidores e *Play Test*
fora (audição da sessão). A chave do ducking interpola-se como índice com `Hold`.

### 2.5 §A1 — o item de uma lista tem IDENTIDADE; família LISTA = 8
- `ItemId(u32)` (crate folha `ph2d-item-id`) em dez itens (Timers, Tweens, estados, setas, CounterWatch, SignalActions, gatilho, shake,
  resistências, âncoras). Passe `ph2d_ecs::item_id::assign_missing_item_ids` (`item_id.rs:167`) corre DENTRO do `assign_missing_stable_ids`.
- ⛔ **Unicidade por OBJECTO, não por projecto** (contradiz o plano original): o `instance_sync` reescreveria os ids da instância por quadro.
  O repetido dentro da entidade é refeito (`item_id.rs:192`); a cópia guarda os ids.
- **`PROJECT_SCHEMA` 194→195**: todo `<= 194` passa os blobs pela forma congelada (`item_id_v194.rs`, `health_v194.rs`,
  `project_load_read.rs:238`); a leitura exige o ÚLTIMO byte (`um_blob_ja_na_forma_nova_passa_intacto`).
- Família: `campo = lista << 10 | membro << 2 | faixa`, `i:j` = o `ItemId`; vocabulário em `ph2d_component_desc::lista`
  (`catalog/listas.rs:248`); funil `ph2d-app-components/src/canais_lista.rs`, chamado de dentro de `canais_componente::{publicar, aplicar}`.
  `FieldKey::grupo` devolve o `i:j` inteiro em `u32` (dois timers = dois grupos). Nome da linha = POSIÇÃO de agora
  (`ph2d-timeline-persist/src/lib.rs:426`).
- Losango: `sections/canal_item::regista`; gate 20 `Divida` → `Canal`.

### 2.6 Smoke-fix — o Long congelava depois de apagar o Short (`1037e54bd9`)
Os funis da curva (COMPONENTE, SCRIPT, LISTA) declaravam a escrita no `FieldDrive` dos SCRIPTS; o rebobinar (`script_bridge::rewind` →
`fields.release`, em TODO quadro parado no início, `fase_fabrica_e_morte.rs:73`) repunha o AUTORADO sobre a curva e o Auto Key gravava-o na
chave de t=0. O Ctrl+Z do apagar é o MESMO mecanismo. Cura: `FieldDrive::conduzido_pela_curva`
(`crates/ph2d-preview-drive/src/fields.rs`) — `release` só devolve o que um script escreveu. 2.º defeito: `autokey_campos::grupo` agrupava itens
pela metade alta do id ⇒ passou à porta única `FieldKey::grupo`.

### 2.7 §E — o fio do caminho (report do dono)
Porta `ph2d-vec-render/src/path_wire.rs` (`stroke_screen`, `draw_path_wires`, `draw_pen_reach`, `token_ink`); quem escolhe os caminhos:
`ph2d-app-vec/src/edit_marks.rs` (`wire_ids`, `draw_wires`), chamado em `fase_vector_edit_overlay`. Token `accent` (1 px de ecrã). *Rubber band*
da Caneta (não existia) e da lâmina do Corte. Hover → `accent`, Trim → `danger`. Cena `PH2D_OBJECT_MODE_SMOKE=39` (`vector_mode_fio.rs`),
foto `img_F30/E_fio_caneta_largura_0.png`. Dívida de cor crua contada (§6.9).

### 2.8 §F — arrastar vários na Hierarquia (report do dono)
A lei saiu da shell: `ph2d-app-components/src/hierarchy_reparent.rs` (`quem_viaja`, `reparentar`, atómico, um passo de undo). O dreno recebe os
bits da selecção (`fase_vector_tree_settle.rs`); o painel apaga as linhas que viajam (`row_rides_the_drag`). Largar antes/depois/na raiz e
*Remove from Sheet* curados juntos. Censo `the_drag_asks_the_door_before_changing_the_parent` re-ancorado.

### 2.9 Provas de mutação (agente `mutacao`)
§C/§D 9/10 → a sobrevivente fechada por `a_gravidade_que_volta_ao_repouso_volta_ao_corpo` · §E 6/9 → oráculo corrigido (2 eram furo, 1
redundância) · §B1 10/11 → teste do anel no quadro da mudança · §B2 11/12 (sobreviventes equivalentes/cobertas) · §F 9/9 · §A1 8/10 → três
furos fechados (`db2df751b3`). Gates do smoke-fix: `apagar_o_primeiro_timer_nao_congela_a_curva_do_segundo`,
`reordenar_os_timers_nao_congela_a_curva`, `desfazer_o_apagar_de_um_timer_devolve_o_mesmo_item_e_a_curva`,
`o_rebobinar_dos_scripts_nao_devolve_o_campo_que_a_curva_conduz`, `editar_um_timer_nao_grava_a_chave_do_vizinho`.

## 3 — Superfície de colisão (um merge pode partir)

- **Formato (contratos de fio, append-only):** `PROJECT_SCHEMA` **195** (o main está em 193) — reconte na integração com
  `python3 scripts/schema-recount.py` (blob `mesa` no fim do `ProjectFile`; `item` no fim de dez itens + `Health`); famílias de canal
  **LISTA=8, MUNDO=9, MESA=10** (`ph2d-ecs/src/canal.rs`: `info` e `FieldKey::membro` são os dois `match familia`); `StableId::CENA`.
- **Contadores como DELTA:** `PROJECT_SCHEMA` +2 (193→195) · famílias de canal +3 · `PH2D_OBJECT_MODE_SMOKE` +1 (`..39`) ·
  `PH2D_AUTOKEY_SMOKE` +2 (`..35`; a 34 é da F28) · crates +1 (`ph2d-item-id`).
- `line/GameEngineFuncional` (viva) mexe em `timer_tick.rs`, `signal_actions_bridge.rs`, `ph2d-ecs/src/lib.rs`,
  `ph2d-app-vec/src/{lib,state,vector_mode}.rs`: aqui o `ItemId` entrou nos TIPOS dos itens (`timer.rs`, `tween.rs`, `state_machine.rs`…) — os
  construtores literais dela partem em compilação. `line/Vector` em `ph2d-app-vec` / `ph2d-panel-inspector`.
- Ficheiros quentes: `ph2d-ecs/src/{stable_id,item_id,cena,canal*}.rs` · `ph2d-physics/src/world/bodies.rs`, `bridge/settings.rs`,
  `joints.rs` · `ph2d-audio/src/mesa.rs`, `ph2d-app-audio/src/{mesa_doc,canais_mesa,marcas_mesa}.rs` · `ph2d-preview-drive/src/fields.rs` ·
  `ph2d-app-components/src/{canais_lista,canais_componente,hierarchy_reparent}.rs` · `ph2d-vec-render/src/{path_wire,overlays,hover_outline}.rs`
  · `project_schema.rs`, `project_load_read.rs`, `project_load.rs`, `project_save.rs`, `scene/incremental.rs`.
- Shell: `render_loop/fase_audio_mixer.rs` (−90 linhas, o corpo foi para `mesa_doc`), `fase_audio_panels.rs`, `fase_autokey.rs`,
  `fase_timeline_drain.rs`, `fase_vector_tree_settle.rs`, `hero_intents/hierarchy.rs`; catraca `the_shell_only_shrinks` (agulhas re-ancoradas).
- Gate `architecture_every_editable_field_is_a_channel`: `ph2d-panel-physics` e `ph2d-panel-audio-mixer` saíram de `DIVIDA_CRATE`.
- **Semântica nova:** a mesa guarda-se no projecto; a curva da física não limpa o scrub; a Cena nunca vai ao ficheiro nem ao undo.

## 4 — Smoke do dono (em linguagem simples) — já feito, OK

**Teste 1 — a lista e a Cena (cena 35):**
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-UIUX && PH2D_AUTOKEY_SMOKE=35 cargo run -p ph2d-host-desktop --profile smoke`
2. Aparecem um chão e uma bola; o objecto «Clock» tem dois Timers (1 s e 3 s); Auto Key e Physics armados; nada seleccionado (a vista da Cena).
3. No Inspector do Clock, com Auto Key: duração do Timer longo 2 s em t=0 e 6 s em t=2; apague o Timer curto; volte a t=0: a chave de 0 s
   continua 2 s e a curva conduz (não fica fixa). Ctrl+Z do apagar também não a congela (era o defeito do smoke).
4. Gravidade da Physics com Auto Key ligado (nada seleccionado = vista da Cena): inverta-a por uma chave a meio da queda; a bola muda sem
   a simulação voltar ao zero.
5. Deu errado se: a curva do longo fica plana depois de apagar o curto; mexer num timer grava no outro; a gravidade só vale depois de parar.

**Teste 2 — Vetor e Hierarquia:** `cd … && PH2D_OBJECT_MODE_SMOKE=39 cargo run -p ph2d-host-desktop --profile smoke` — a Caneta com traço 0 e
sem preenchimento mostra o fio e a linha até ao cursor. Na Hierarquia, seleccione vários objectos e arraste um: vão todos; Ctrl+Z desfaz num
passo.

## 5 — Verificação

- fmt + clippy `-D warnings` nas 23 crates tocadas (commit `59fed12a1e`), binário `--profile smoke` compila.
- 77 testes da onda por nome verdes; gates `architecture_every_editable_field_is_a_channel`, `the_shell_only_shrinks` e
  `architecture_claude_md_cabe_no_orcamento` verdes. Depois do `1037e54bd9`: os cinco gates do §2.9 e o re-smoke do dono.
- Mutação: §2.9.
- ⚠️ **Suíte inteira NÃO corrida** (dono 23/10: só antes da integração; é do integrador).

## 6 — ABERTO, por ordem

1. **F31 = §A2** (primeiro item): a pilha do Vetor (`PaintEntry`/`FxEntry`), os CONTORNOS (apagar um subcaminho desloca chaves — medido:
   `propagar` = `None`, o contorno `i=1` passa a tocar as chaves do apagado) e os FILTROS da forma (`canais_filtros.rs:100,157`) por posição.
   **Decisão técnica já tomada:** 5.º campo `ItemId` (u32) na `FieldKey` (`0` nas outras famílias; `FieldKey::new` mantém a forma),
   degrau do `TimelineDoc`, `VEC_SCENE_SCHEMA` 23→24, `PROJECT_SCHEMA` 195→196. A migração precisa de um espelho dos ids da pilha publicado
   no `World` (`install_from_project`, `ph2d-timeline-persist/src/lib.rs:505`, não vê a `VecScene`; gémeo de `PosicoesDosItens`). Contornos
   pela lei do dono de 20/10 (propagar a todas as chaves).
2. `TimerRuntime` reconciliado por COMPRIMENTO (`timer.rs:277`) e o tween `i` ↔ `Timers[i]` por posição (`tween.rs:95-107`) — medido; código
   da `line/GameEngineFuncional`: assinalar na integração.
3. As setas da StateMachine apontam estados por índice (`from/to: u8`, `state_machine.rs:105`).
4. `.ph2dmacro` gravado antes do 195 com estes componentes no mundo não migra (`ph2d-app-macros/src/mundo/grava.rs`).
5. `canal_componente.rs:269` lê `View::Toggle(t, f)` como `(off, on)` — invertido face a `property.rs:222`; corrigir troca o sentido de chaves
   gravadas (decisão do dono).
6. Sem coluna de losango: faixas da mesa (fader, pan, tom, corte, M/S) e o painel do esqueleto inteiro; a matriz de camadas da física não é
   animável.
7. O scrub re-simula do checkpoint sem reler a curva por passo (juntas, corpos, mundo).
8. Mundo e mesa fora do undo (como já era o `physics`). O nome da linha é por `FieldKey` sem entidade (duas cópias com ids iguais e posições
   diferentes partilham-no).
9. Dívida de cor crua nos overlays vetoriais (`cor_crua_tests.rs`): `connector.rs` 5 · `marquee.rs` 4 · `cut_line.rs` 2 · `gradient.rs` 2 ·
   `guides.rs` 2 · `symmetry_overlay.rs` 1.
10. `CLAUDE.md` §5 conta-se no fecho da linha/integração: `PH2D_OBJECT_MODE_SMOKE=1|2|4|6..39`, `PH2D_AUTOKEY_SMOKE=1..35`.
11. A suíte inteira antes da integração (do integrador). `ph2d-panel-painter-layers` NÃO é desta linha (da `line/PainterWatercolor`).
12. **NÃO integrar nem fazer push sem ordem do Enio.**
