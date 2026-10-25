# Handoff de continuação — `line/GameEngineFuncional`, plano 08 (a lista de regras) + rodada 3 do 07 + teclado no palco (25/10)

**Próxima janela:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` + `pwd` +
`git branch --show-current` ANTES de ler. Base (merge-base com o main) `c3662cd8e4` (já estava em cima);
HEAD antes deste doc `b2a323480d`. **Não integrada, não empurrada.** Antecessor:
[`HANDOFF_CONTINUACAO_2026-10-25_07_ANIMACAO.md`](HANDOFF_CONTINUACAO_2026-10-25_07_ANIMACAO.md). Plano:
[`08_regras_de_animacao_plano.md`](../08_regras_de_animacao_plano.md).

## Estado

**Smoke do dono por fazer. Nada disto foi fotografado no app real** (nem Frames/s, nem as regras, nem o
teclado no palco) — só testes por nome.

| sha | O quê |
|---|---|
| `5c683d0f45` | Frames/s leva os tempos próprios junto (report do dono, rodada 3 do 07) + plano 08 |
| `68519c00ed` | no palco do jogo uma aba aberta não lhe tira o teclado |
| `b2a323480d` | plano 08: a lista de regras no cartão Animation |

### `5c683d0f45` — Frames/s (lei do Godot)
Causa: o herói nascia com `per_frame_ms` = a folha inteira; tempo próprio vence o ritmo, logo mexer em
Frames/s não mudava nada. Cura: `AnimationTag::set_rhythm` escala os tempos próprios pelo multiplicador em
2 casas (a razão crua dava 266→110→265, sem volta igual), `set_own_ms` poda o que é igual ao ritmo,
`frame_factor`/`ms_of_factor` (`ph2d-ecs/src/sprite_anim.rs`). Herói nasce só com o que difere (salto `[0,133,167,133]`).
«This frame» e a faixa do editor mostram/editam o MULTIPLICADOR (`Unit::Times` «x», depois de `px` em
`Unit::ALL`, 13→14); o dado continua em ms. Gates: `mudar_o_ritmo_leva_os_tempos_proprios_e_volta_igual`,
`um_tempo_igual_ao_ritmo_poda_o_vetor`, `mudar_os_quadros_por_segundo_leva_o_tempo_proprio_junto`.

### `68519c00ed` — teclado no palco
Com o editor de animações (qualquer aba de documento) à frente, «Play in a Window»/ecrã cheio não recebiam
teclas: `cena_do_jogo` dava `a_vista=false` por `surface_active()`. A lei saiu da shell para
`ph2d_app_components::posse_do_jogo::Tapa::a_vista` (no palco as abas não tapam; texto e modos tapam
sempre). Gate `no_palco_a_aba_aberta_nao_tapa_o_jogo`. **Só teste, não verificado no app.**

### `b2a323480d` — lista de regras «quando ___ → toca ___»
- **Dado:** `ph2d::physics::AnimRules` (`crates/ph2d-physics-ecs/src/components/anim_rules.rs`), irmão de
  `CharacterAnimations`. `AnimWhen::{Pose(CharacterPose), Action(String), Life(LifeEvent), Signal(String)}`
  append-only; `CharacterPose` ganhou Serialize/Deserialize + `ALL`. **Sem degrau de `PROJECT_SCHEMA`**
  (precedente `SpriteAnimFrames`; ficheiro antigo lê-se igual). ⚠️ O N1 subiu 193→194 para
  `CharacterAnimations` por convenção «registado novo ⇒ sobe»; **o integrador decide se conta**
  (`python3 scripts/schema-recount.py`).
- **Avaliação:** `ph2d-app-game/src/personagem.rs` (`decide`, recurso `RegrasEmCurso` com seguros e
  `SignalReader`); `anima_personagens(world, physics, &premidas, &outbox)`. A shell calcula
  `acoes_premidas(hero.input_map, input_actions)` em `fase_physics_step.rs`. Sinais lidos do outbox têm
  **1 quadro de atraso** (a física publica na fase seguinte). Deps novas em `ph2d-app-game`: `ph2d-input`,
  `ph2d-runtime`, `bevy_ecs` (Cargo.lock).
- **Cartão:** `AnimChooser::Rules` (era `Character`; vale sem Platform Player: num personagem pré-preenche
  via `CharacterAnimations` — parado→idle, correr→run, subir→jump, cair→fall; noutro objecto grava
  `AnimRules` vazio). `AnimWhenInfo`/`AnimRuleInfo`; `AnimCardInfo::{rules, rule_actions (enchido em
  `snapshots_inspector.rs` a partir de `hero.input_map`), rule_signals (nomes autorados: animações, Health
  on_damage/on_heal/on_death, SignalOnHit/Leave, RaySignals e os das regras)}`; `when_options()`.
  Edições `RuleWhen/RulePlay/RuleAdd/RuleRemove/RuleMove`. Ficheiros novos: panel-inspector
  `sections/anim_rules.rs`, `event_anim_rules.rs`, `tests/it/seam_anim_rules.rs` (8 seams).
- **Ids:** SAÍRAM `CharacterState` e `INSP_ANIM_STATE_PICK`; `INSP_ANIM_STATE_OPT`→`INSP_ANIM_RULE_PLAY_OPT`;
  novos `MAX_ANIM_RULE_ROWS=16` (teto da tabela de ids), `INSP_ANIM_RULE_{GRIP,WHEN,PLAY,DEL}[16]`,
  `INSP_ANIM_RULE_ADD`, `INSP_ANIM_RULE_WHEN_OPT[64]`; `PanelRowFamily::AnimRule` (plana).
- **i18n:** novas `rules`, `rules_hint`, `rules_hidden` e rótulos das condições; saíram `character`,
  `character_hint`, `needs_player`, `state_tip`.

## Superfície que um merge pode partir

- `ph2d-physics-ecs` registo de componentes (delta **+2** contra o merge-base: física 45→46; o integrador
  CONTA). Render/script espelhos não mudaram (gates verdes).
- `Cargo.lock` + `ph2d-app-game/Cargo.toml`; `personagem.rs` e `fase_physics_step.rs` (assinatura de
  `anima_personagens` mudou); `snapshots_inspector.rs`.
- Inspector: `anim_card.rs`, `inspector_model_anim.rs`, tabela de ids de animação, i18n do cartão. Outra linha
  que edite o seletor «The Character» conflita.
- Shell: +7 (`snapshots_inspector`) +~3 (`fase_physics_step`) +1 (`posse_do_jogo`) — `the_shell_only_shrinks` verde.

## Prova de fecho (DELTA)

Verdes, por nome: physics-ecs anim_rules, registers_every_physics_component · app-game personagem (7), smoke
(8) · ecs sprite_anim (32) · app-sprite-anim editor::tests (24), inspector (25), card (8) · panel-inspector
seam_anim (40) · editor-core numeric_input_with_unit, a_longer_suffix, inspector_model_anim,
architecture_panel_wiring_parity, hit_indexed_ids_are_registered, architecture_panel_loc_cap,
architecture_the_shell_only_shrinks, architecture_every_editable_field_is_a_channel · app-components
posse_do_jogo, todo_componente_com_nome_tem_a_sua_lei · host-desktop a_posse_do_teclado,
every_physics_component_is_authorable, a_schema_bump_anywhere · render/script registry · i18n
a_tabela_inglesa · panel-registry-init nenhum_nome_carrega, o_inspector_armado. Clippy de fecho de onda: ver
commit seguinte. **Suíte de impacto NÃO corrida** (regra do dono; o integrador TEM de a correr). Smoke build,
2.ª corrida: `Finished smoke in 0.32s`, zero `Compiling`.

## Smokes

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=2 cargo run --profile smoke -p ph2d-host-desktop
```

Para o dono (nomes como na tela):
1. Corra o comando; selecione **Player** ▸ cartão **Animation** ▸ **Who Picks ▸ Rules**: aparecem 4 linhas
   (Standing still→idle, Running→run, Going up→jump, Falling→fall).
2. **+ Rule**; na nova, **When = Key held: down** e **Play** = outra animação; arraste o número dela para o topo.
3. **Play in a Window**; segure ↓: toca a escolhida; solte: volta. Errado se não troca, ou se a de cima não ganha.
4. Frames/s: mude o valor no cartão; a animação tem de acelerar/desacelerar e voltar igual ao repor.
5. Teclado: com **Edit Animations…** aberto, **Play in a Window** e mova com as setas. Errado se o boneco não mexe.

## ABERTO

- `AnimRules` sem nome de script nem lei em `property_laws::LEIS` (o censo
  `todo_componente_com_nome_tem_a_sua_lei` só aceita componentes com nome de script; decisão de desenho).
- Barra de largada das regras sem seam.
- «This frame» mostra `1.330x` (formatter partilhado `format_number`, 3 casas) vs editor `1.33x`.
- Herdado do 07: «abaixar» com corpo baixo · R1 vista dupla ao vivo · View ▸ Focus mode morto ·
  `ph2d.disconnect` · resto da arte (plano 06) · faixa do editor não rola ao arrastar junto à borda · suíte de
  impacto (integrador).

## PRÓXIMO

Smoke do dono das três coisas (Frames/s, teclado no palco, regras); depois o que ele pedir.
