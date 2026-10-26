# Handoff de continuação — `line/GameEngineFuncional`, plano 09 (abaixar com corpo baixo) (25/10)

**Próxima janela:** `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional` + `pwd` +
`git branch --show-current` ANTES de ler. Base (merge-base com o main) `c3662cd8e4`. **Não integrada, não
empurrada.** Antecessor: [`HANDOFF_CONTINUACAO_2026-10-25_08_REGRAS.md`](HANDOFF_CONTINUACAO_2026-10-25_08_REGRAS.md).
Plano: [`09_abaixar_plano.md`](../09_abaixar_plano.md).

## Estado

**Smoke do dono por fazer.** Fotografado no app real (`fotografa_cena.sh`, `PH2D_GAME_SMOKE_DRIVE=window`,
1600×900): [`fotos_09/agachado_sob_o_tecto.png`](fotos_09/agachado_sob_o_tecto.png) (debaixo da laje junto à
parede, a tocar «fall») e [`fotos_09/de_pe_fora_do_tecto.png`](fotos_09/de_pe_fora_do_tecto.png). ⚠️ O desenho
NÃO encolhe (sem arte de agachar): a cabeça desenhada sobrepõe-se ~27 px à laje enquanto o corpo físico passa.

| sha | O quê |
|---|---|
| `c17d4983df` | lei + física + pose + fábrica + nível |
| `82924921ea` | smoke `=3` + gates das duas mutações sobreviventes |

## Medido antes de desenhar

O agachar JÁ existia (W15, `ph2d-platformer/src/crouch.rs`), desligado de fábrica
(`CrouchConfig::STARTING_POINT.height = 0`), e o Add ▸ Character não o ligava. Era só perna: o collider nunca
mudava. Cápsula do herói `half_height 0,15 + radius 0,2`, `float_height 0,45` ⇒ a perna só descia 0,10 m
(topo 0,80→0,70).

**Defeito medido a meio que MUDOU o desenho:** perna primeiro (até a base da cápsula tocar o chão) + forma a
encolher pelo resto ⇒ o corpo ENCOSTA no chão e o atrito prende-o: agachado o herói ficou em x = 0,02 m em 300
tiques. Cura = lei do Unreal: **a FORMA encolhe primeiro** (por cima, base no sítio, centro e pés onde a folha
os desenha), a perna só desce o que a forma não der. (O §«Desenho» do plano 09 tem a correcção.)

## O que entrou

### Lei (`ph2d-platformer/src/crouch.rs`)
`CrouchConfig::body_room` (NÃO autorado; a ponte põe-no a cada tique = troço recto do collider: cápsula/estádio
`2·half_height`, caixa `2·half_y`, bola/elipse 0). `body_drop = min(rise, body_room)`, `leg_drop = rise −
body_drop`; `ride_for` desce só `leg_drop`; `body_shrink(cfg, ride, crouched)`. `body_room = 0` ⇒ lei W15 bit a
bit (todos os testes da lei pura iguais). `rise` (o sensor de tecto) não muda: a varredura do corpo ENCOLHIDO
sobe `rise` e cobre exactamente a cápsula de pé.

### Física (`ph2d-physics/src/world/reshape.rs`, novo)
`PhysicsWorld::reshape_body(body, rest, shape, offset)` troca forma+translação do collider PRINCIPAL (1.º do
corpo), mantém handle/camada/sensor e a MASSA (agachado pesa o mesmo); repor a forma de pé repõe a
densidade/massa autorada (bit a bit com o spawn). Idempotente pela translação ⇒ um `rewind_to` (reconstrói de
pé) cura-se no tique seguinte, sem estado novo. `ShapeDesc::{top_shrink_room, shrunk_from_top}` (cápsula sem
troço vira `Ball`, estádio vira `Ellipse`). `collider_build::shape_builder` (o match das formas partido de
`build_collider`).

### Ponte (`ph2d-physics-ecs`)
`bridge/player_probes.rs::{with_body_room, crouch_shape}`; `player.rs` empurra `reshapes` para `PlayerResults`
(`player_apply.rs`, aplicados ANTES do step). `player.rs` ficou em 700 linhas (tecto
`architecture_workspace_file_loc_cap` = 700; o cálculo foi para `player_probes.rs`). Novo diagnóstico
`PhysicsBridge::body_extent(entity)` (caixa do solver com a forma actual).

Gates W15/W-ShapeCast que liam o topo como «centro + meia-altura» (representação, não produto) passaram a ler
`body_extent`: `platform_crouch.rs` (3 testes) e `platform_pillar_rig.rs::crouch_then_release` (3 testes de
`platform_headroom_sweep`). O scrub agora também compara o topo (a forma baixa volta depois do `rewind_to`).

### Pose
`CharacterPose::Crouch` append-only (`ALL` = 5, `DERIVED` = as 4 do `CharacterAnimations`; `tag()` →
`Option<&str>`, `Crouch` → `None`). Não é exclusiva: `CharacterPose::of` dá a base, «Crouching» vale junto
(`Factos::agachado = PlayerView::crouching`, `ph2d-app-game/src/personagem.rs`) ⇒ as 4 regras derivadas de
ficheiros antigos dão a mesma tag agachado ou não (gate `as_quatro_derivadas_dao_a_tag_de_cada_pose` corre as
duas). Cartão: `ANIM_POSES = 5`, `ANIM_POSE_CROUCH = 4`, `AnimCardInfo::can_crouch` (preenchido em
`ph2d-app-sprite-anim/src/card.rs` pelo `crouch.armed`); a opção só aparece com o agachar armado. i18n
`panel.inspector.anim_card.when_pose_4` = «Crouching»; `POSE_KEYS` 5.

### Fábrica e nível
Sem campo gravado novo ⇒ **sem degrau de `PROJECT_SCHEMA`**; sem componente novo. `object_add.rs`
`AGACHADO = HERO_SHEET_SOLE_BELOW_CENTRE − 2·CAPSULA_MEIA = 0,15` (topo 0,80 → 0,50); velocidade agachado = 2
m/s (de pé 6). Os parâmetros já estavam no cartão Platform Player ▸ Crouch (`Height` m, `Speed` m/s) — nenhum
controlo novo. Nível 2: `BLOCOS_2` 5→6, «Low Ceiling» centro `[-10.5, 0.85]` meia `[1.5, 0.25]` (fundo 0,60,
encostado à Wall Left). Smoke novo `PH2D_GAME_SMOKE=3` (`CENAS` 2→3): o nível 2 com a regra «Crouching → fall» no
topo (não há células de agachar na folha — tags só idle/run/jump/fall; arte é o plano 06) e `HORARIO_3` (← ↓ até
à parede, solta ↓ debaixo do tecto, sai para a direita). `Gesto::{Esquerda, Abaixa}`, `Seta::{Esquerda, Baixo}`
append-only (a shell indexa as teclas pela ordem das variantes, `shells/desktop/src/janela_do_jogo.rs`; shell −2
linhas nesse ficheiro).

## Superfície que um merge pode partir

- `ph2d-physics`: `world.rs` (mod), `collider_build.rs`, `reshape*.rs` novos.
- `ph2d-physics-ecs`: `bridge/player*.rs`, `diagnostics.rs`; testes `it/platform_crouch*.rs` e
  `platform_pillar_rig.rs`.
- **`CrouchConfig` ganhou campo:** todo literal `CrouchConfig {…}` noutra linha parte a compilar — o integrador
  acrescenta `body_room: 0.0`.
- **`CharacterPose` +1 variante:** match exaustivo noutra linha parte.
- `ANIM_POSES` 4→5, `BLOCOS_2` 5→6, `CENAS` 2→3 (números que SOMAM entre linhas: contar, não escolher).
- `shells/desktop/src/janela_do_jogo.rs` (−2).
- Registos/schema: nenhum delta (sem componente novo, sem degrau).

## Prova de fecho (DELTA)

Gates novos (verdes): `crouch::tests::the_body_shrinks_first_and_the_leg_takes_the_rest`;
`world::reshape::tests::{reshaping_keeps_the_mass_and_standing_again_is_the_spawn, the_sweep_sees_the_new_shape,
shrinking_from_the_top_keeps_the_base}`; `platform_crouch_body::{the_low_body_passes_under_a_ceiling_that_stops_him_standing,
under_the_ceiling_he_stays_low_and_under_open_sky_he_stands}`; `personagem::tests::agachado_ganha_e_levantar_devolve_a_pose`;
`smoke::tests::{no_nivel_2_agachado_passa_sob_o_tecto_baixo, o_nivel_3_agacha_sob_o_tecto_e_a_regra_toca}`;
`card_tests::o_cartao_sabe_se_o_personagem_agacha`; seam `seam_anim_rules::agachado_so_se_oferece_com_o_agachar_armado`;
`object_add` assere o agachar armado.

**Prova de mutação 8/8 sangra** (as 2 sobreviventes da 1.ª ronda — `agachado: false` no `anima_personagens` e
`can_crouch: true` — mortas pelos 2 gates do `82924921ea`).

Corridas: crates inteiras `ph2d-physics`, `ph2d-physics-ecs`, `ph2d-platformer` com `motivo:` (ponte de que
muitas dependem): verdes depois de reler 3 gates por `body_extent`. Por nome verdes: platformer crouch/brink,
physics-ecs crouch/anim/probe/registers, app-game personagem/smoke/object_add, panel-inspector seam_anim (43),
editor-core inspector_model_anim + architecture_workspace_file_loc_cap / panel_wiring_parity / hit_indexed /
panel_loc_cap / the_shell_only_shrinks, app-sprite-anim card (9), i18n a_tabela, app-physics
player/probes/crouch, panel-registry-init o_inspector_armado / nenhum_nome_carrega, host-desktop
every_physics_component_is_authorable / a_schema_bump_anywhere, app-components
todo_componente_com_nome_tem_a_sua_lei. `cargo fmt --all` limpo; clippy `-D warnings --all-targets` das 9
crates tocadas limpo. **Suíte de impacto NÃO corrida** (é do integrador). Smoke build, 2.ª corrida: `Finished
smoke profile [optimized] target(s) in 0.32s`, zero `Compiling`.

## Smokes

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && PH2D_GAME_SMOKE=2 cargo run --profile smoke -p ph2d-host-desktop
```

Para o dono (nomes como na tela):
1. Corra o comando; selecione **Player** ▸ cartão **Animation** ▸ **Who Picks** ▸ **Rules** ▸ **+ Rule**;
   When = **Crouching**, Play = **fall**; arraste o número dela para o topo.
2. **Play in a Window**; segure ↓ e ← até à parede da esquerda: passa debaixo do teto baixo, mais devagar, e
   toca «fall». Errado se pára na entrada da laje.
3. Solte ↓ debaixo da laje: continua baixo. Errado se o boneco salta/levanta para dentro da laje.
4. Ande para a direita para fora: levanta e volta a «idle/run».
5. **Platform Player ▸ Crouch**: **Height** e **Speed** mudam o quão baixo e o quão depressa. (O desenho não
   encolhe: falta a arte de agachar.)

Atalho só para ver sem mãos: `PH2D_GAME_SMOKE=3 PH2D_GAME_SMOKE_DRIVE=window` (mesmo `cd` e `cargo run`).

## ABERTO

- Herdado, não desta onda: `AnimRules` sem nome de script nem lei em `property_laws::LEIS` · barra de largada
  das regras sem seam · «This frame» `1.330x` vs `1.33x` · R1 vista dupla ao vivo · View ▸ Focus mode morto ·
  `ph2d.disconnect` · resto da arte (plano 06 — agora inclui células de agachar) · a faixa do editor não rola
  ao arrastar junto à borda · suíte de impacto (integrador).
- **NOVO:** com `leg_drop > 0` numa forma sem troço suficiente (bola, ou altura pedida abaixo do troço) o corpo
  ainda pode encostar no chão e travar (o defeito W18 do piso, agora só fora do caso da fábrica) — sem gate.

## PRÓXIMO

Smoke do dono; depois o que ele pedir (candidato: arte de agachar no plano 06).

## Smoke do dono: OK (25/10)

Os seis passos (agachar sob o tecto, soltar lá debaixo, sair e levantar, Height/Speed no cartão) aprovados. PRÓXIMO: o que o dono pedir; candidato natural = células de agachar na folha do herói (plano 06), para o desenho encolher com o corpo.
