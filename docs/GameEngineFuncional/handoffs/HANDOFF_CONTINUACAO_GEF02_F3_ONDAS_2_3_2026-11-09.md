# HANDOFF DE CONTINUAÇÃO — GEF02 · F3 (corpo físico e gatilho), ondas 2–3 — 09/11/2026

Linha `line/GameEngineFuncional02` (worktree `Worktrees/line-GameEngineFuncional02`). Não é handoff de
integração: a linha NÃO fechou (falta a onda 4), nenhum push. Plano: [f3_corpo_e_gatilho_plano.md](../componentes/f3_corpo_e_gatilho_plano.md).
Anterior: [ondas 0–1](HANDOFF_CONTINUACAO_GEF02_F3_ONDAS_0_1_2026-11-09.md). Decisões do dono:
[memória](../../../project-memory/project_physics_critique_answers_2026_11_09.md).

## 1. Estado (commits desde `304a755700`)

| Onda | Commit | Resumo |
|---|---|---|
| 2 | `c8e004d79d` | a camada de colisão na timeline diz o rótulo do campo Layer (nome / «Default» / «Layer n»), não o número |
| 3 (fix) | `89460ca606` | o aviso da forma redonda esticada estava MORTO desde a opção B; ressuscitado |
| 2–3 (dono) | `985319e9ac` | «resolva agora» + «More»: a curva da física guarda e mostra o número do CARTÃO |
| foto | `1b33fd2a6c` | foto do cartão Body com More fechado |

Smoke do dono 09/11: OK nos três (nome da camada na timeline; aviso da bola esticada + «Set Scale to 1»;
Width na timeline igual ao do cartão). Sem suíte completa (é do integrador).

## 2. Já existia — NÃO se reconstrói

Width/Height no cartão da forma (`ed0a0dcbe6`); Layer do Ray Sensor já é dropdown; dobras fechadas por omissão
(Start moving, More, Joint, Bake); recibo «Physics Body removed — Undo».

## 3. Mecanismo

- **c8e004d79d.** `Especie::CamadaDeColisao` (`ph2d-ecs` `canal.rs`) + resource `NomesDasCamadasDeColisao`
  (`canal_palavra.rs`), publicado por `canais_corpo::publicar` (`ph2d-app-physics`) a partir de `LayerNames` +
  `ph2d_panel_physics::aba::nome_inteiro`, só quando muda. O descritor `canal_fisica::CAMADA` passa a
  `CamadaDeColisao`. Gates ajustados: `canal_componente_donos_tests::compativel` (Int ↔ Numero|CamadaDeColisao),
  `canais_donos_tests::outro`. Foto da cena 120: `PH2D_PHYSICS_SMOKE_CAMADA_NA_TIMELINE=1`
  (`physics_smoke_ficha.rs`: chaves Player layer Player→Props a 0 s/1 s, abre a Timeline). Testes
  `a_camada_de_colisao_diz_o_nome_do_cartao`, `a_timeline_le_a_camada_de_colisao_pelo_rotulo_do_cartao`; 2 mutações sangraram.
- **89460ca606.** `InspectorPhysicsInfo::estica: Option<InspectorTransformInfo>` (`ph2d-editor-core`
  `inspector_model_physics.rs`) = pose do mais próximo (forma ou antepassado) com |escala| desigual quando o
  produto ao longo da cadeia é desigual; calculado por `crates/ph2d-app-physics/src/inspector/estica.rs::quem_estica`.
  `esticada()` (`physics_cartao_forma.rs`) usa-o; «Set Scale to 1» (`recibo_fisica.rs`) repõe a escala DAQUELA
  entidade (espelho mantido); o cartão Body também pinta o aviso quando `shape_on_child`. Testes
  `a_forma_esticada_pelo_corpo_diz_quem_a_estica`, `bola_esticada_avisa_com_as_duas_saidas` (reescrito); 4 mutações sangraram.
- **985319e9ac.** Canal FISICA guarda o número do cartão: MEIO_X/Y = 2·half (Width/Height), CAPSULA =
  (cap_half+radius)·2 (altura inteira), VEL_ANGULAR em °/s; `edicao` passa `v` direto ao funil do painel.
  `ph2d_ecs::canal::unidade`: FISICA RAIO..=CAPSULA e DESLOC_X/Y = Comprimento (a timeline mostra a unidade do
  projeto como o cartão). Rótulo «Capsule Height». `card.insp.phys.advanced` → «More» (a dobra já existente
  renomeada). **Sem bump de `PROJECT_SCHEMA`** (nenhum tipo gravado mudou; CLAUDE.md §0.11). Teste
  `a_curva_diz_o_numero_do_cartao`; 3 mutações sangraram. `canais_donos_tests` converte VEL_ANGULAR rad→deg;
  `a_chave_da_forma_no_corpo_vai_para_a_filha` usa v·0.5 para MEIO_X.

## 4. Superfície de colisão e o que um merge pode partir

- Vocabulário partilhado `ph2d-ecs`: `canal.rs`, `canal_palavra.rs`, `canal_fisica.rs`; i18n `canais.rs`/`cartoes.rs` (`ph2d-i18n`).
- ⚠ `InspectorPhysicsInfo` ganhou o campo `estica`: qualquer outra linha que o construa literalmente deixa de
  compilar — acrescentar `estica: None`.
- Contadores: shell intocada, EXACTAMENTE no tecto 189 041/189 041 (`the_shell_only_shrinks`); `PROJECT_SCHEMA` intocado.
- Linhas vivas a evitar: as da §4 do handoff anterior (PixelArt em `ph2d-app-components`, TimelinePrincipal em `ph2d-panel-timeline`, etc.).

## 5. Prova de fecho de onda

Testes por nome verdes: `ph2d-ecs` `canal_` 48 · `ph2d-app-physics` `canais_` 42 / `physics_tests` 29 ·
`panel-inspector` `o_cartao_da_fisica` 20 / `seam_physics` 43 / `o_detail_trava` 2 · `registry-init`
`o_inspector_armado` 3 · `panel-timeline` `tracks_value` 6 · `editor-core` `architecture_panel_loc_cap` 4 ·
`the_shell_only_shrinks` 2. `host-desktop` `check` verde; `fmt` nas crates tocadas. Sem clippy nesta rodada
(o fecho da linha faz). Mutações: 2 + 4 + 3, todas sangraram.

Fotos em `fotos_fisica/camada_timeline/`: `01_camada_na_timeline.png` (faixa recolhida: cliques sintéticos não chegam
à janela virtual), `03_esticada_cartoes.png` (`PH2D_GAME_SMOKE=4 PH2D_GAME_SMOKE_FOTO=fisica_esticada`, 1930×2200),
`04_corpo_more.png` (`fisica_dynamic`).

Smoke (o dono já fez, OK):
1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && PH2D_PHYSICS_SMOKE=120 PH2D_PHYSICS_SMOKE_CAMADA_NA_TIMELINE=1 cargo run --profile smoke -p ph2d-host-desktop`
2. Na Timeline, a faixa de camada do Player mostra «Player» → «Props», não números; Width/Height iguais aos do cartão.
3. Bola esticada: `PH2D_GAME_SMOKE=4` mostra o aviso e «Set Scale to 1» o apaga. Errado = número na timeline, ou aviso ausente.

## 6. ABERTO

- Herdado do handoff 0–1 §6: `from_control` e botões/escolhas do painel autorado fora do censo de sinais; sobras da onda do Outline.
- Canais da família JUNTA ainda guardam radianos (não tocados; mesma pergunta «curva = cartão», outra família).
- Clippy e suíte completa por fazer (fecho da linha / integrador). Integração só por ordem do Enio.

## 7. Onda 4 (próxima janela) — tecla escolhida no cartão do Trigger; o Input Map continua a fonte

Terreno mapeado:
- Componente `ActionTriggerRow` `crates/ph2d-ecs/src/signal_on_action.rs:94` (`SignalOnAction(Vec<ActionTriggerRow>)` :110,
  `ActionEdge` :62); o runtime resolve por NOME da acção a cada quadro: `dispara` :152; leitura do mapa em
  `crates/ph2d-app-components/src/trigger_bridge.rs:20,:63` (⚠ `ph2d-app-components` é linha viva do PixelArt).
- Cartão bespoke: `paint_action_trigger_section` `crates/ph2d-panel-inspector/src/sections/action_trigger.rs:454`
  (`editor` :259; Action = TextInput `INSP_TRIGGER_ACTION` :301); eventos `apply_action_trigger_event`
  `crates/ph2d-panel-inspector/src/event_action_trigger.rs:19` (TextChanged :83-98; `EditorAction::CreateInputAction` :50-57);
  aplicação em `crates/ph2d-app-components/src/action_trigger_inspector.rs:85/:142`; drenos em
  `shells/desktop/src/render_loop/fase_inspector_commits.rs` e `fase_bus_drain_out.rs`.
- Input Map: `InputMap{actions,next_id}` `crates/ph2d-input/src/map.rs:42` (create :111, id(name) :137, get_mut :148,
  actions() :164); `Binding{Key,PadButton,PadAxis}` `crates/ph2d-input/src/action.rs:22`; vive no HeroScreen
  (`hero.input_map`), gravado em `shells/desktop/src/project.rs:152` / `project_save.rs:199` (não está em `project_schema.rs`).
  Janela: `crates/ph2d-editor-core/src/screens/hero/chrome/input_map.rs` (+ `apply.rs:116-131` liga a tecla capturada).
- Captura de tecla existe: `listen_for_binding` `crates/ph2d-editor-core/src/interaction/state/input_map_ops.rs:80`,
  `capture_if_listening` :134-151 (Esc cancela), chamada de `interaction/dispatch/key.rs:58-60`; estado
  `input_map_listening: Option<ActionId>` (`store_fields.rs:333`) é preso ao `ActionId` da janela do mapa — o cartão
  precisa de um braço próprio («a acção da linha do Trigger») no mesmo caminho de aplicação, que vive junto de
  `hero.input_map` (editor-core, não shell) — verificar antes de planear linhas de shell.
- Padrão de dropdown: `Dropdown<T>` (`ph2d_editor_core::widget`), já usado pelo Trigger para a aresta
  (`action_trigger.rs:55/:206/:243`) e por `camada_por_nome.rs:53/92`.
- ⛔ Shell no tecto exacto: qualquer linha de shell exige mover código para fora primeiro.
