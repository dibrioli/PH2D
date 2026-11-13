# HANDOFF de continuação — GEF 06 Projectile Motion (13/11/2026)

Linha `line/GameEngineFuncional`, worktree `Worktrees/line-GameEngineFuncional`. Onda fechada: versão B do dono, smoke do dono OK em 13/11. Ficha: [06_projectile.md](../componentes/06_projectile.md). Plano: [23_refundacao_plano.md](../23_refundacao_plano.md). Não integrada (a integração é do integrador, por ordem do Enio).

## O que a onda fez

Commits: `5142bb156b` (cartão B) e `96c68552ba` (cena `PH2D_GAME_SMOKE=25`, Target mostra o nome, gates, `fotos_06`).

- **Cartão** (`crates/ph2d-panel-inspector/src/sections/projectile.rs`): avisos; Speed, Gravity, Range, «Faces path» (rótulo do catálogo, índice 7; era Face Velocity); checkbox Bounce → Max Bounces (mín. 1) + Bounciness; checkbox Homing → Target (texto + conta-gotas `INSP_PJ_TARGET_PICK` dentro da caixa, molde da câmera) + aviso «No object…» + Pull (catálogo 8, era Homing; mín. 1); dobra «Speed change» fechada, resumo None · Speeds up · Slows down · Capped · «X · capped».
- **Checkboxes gravam números:** `ProjectileFieldEdit::Bounce(bool)` → `max_bounces` = `BOUNCES_AO_LIGAR` (3) / 0; `Homing(bool)` → `homing_accel` = `PULL_AO_LIGAR` (30 m/s²) / 0. `PickTarget(bool)` arma o recurso de sessão `ph2d_app_components::projectile_inspector::AlvoArmado` (sem passo de undo).
- **Clique no palco:** `escolher_alvo` pega o objeto nomeado do topo sob o cursor (nunca a bala) e empurra `HomingTarget` pelo bus (um passo de undo). Esc desarma (`camera_frame_escape`); a mira vem de `camera_target_armed`.
- **Strings:** `crates/ph2d-i18n/src/projectile.rs` (saíram de `inspector_game.rs`).
- **Target sem nome (defeito achado):** o nome nunca chegava ao campo; agora sincroniza por `sync_fabrica.rs` (`escreve_texto`).
- **Cena 25** (`crates/ph2d-physics-scene/src/cena_projectile.rs`): Missile na fila de cima, lançado para baixo, Pull 80, Max Speed 9 (resumo «Capped»), Target em (2,5; 2,6); abaixo Straight, Bouncy, Arc, Short Range. Enquadramento `ViewFocusKind::All` via `smoke::foco`. Fotos: `docs/GameEngineFuncional/componentes/fotos_06/25_cartao.png`, `25_speed_change.png`, `25_play*.png`.

## Gates e contadores (como delta)

- Novos: `seam_projectile` (8); testes de `projectile_inspector` (10); testes unitários de `cena_projectile`; `shells/game-scenes` `jogo_projectile` (Missile termina a 0,55 m do Target; Arc > 2 m de distância); `smoke_tests` com a 25 como última cena.
- Provas de mutação: 13/13 mortas (cartão + sync do Target + teto de velocidade do Missile).
- Ajustes de gate: `architecture_every_editable_field_is_a_channel` (`projectile::numero`/`voo` = Canal; `ricochete`/`perseguicao` = NaoAnimavel); `every_word…` lista `projectile.rs`; carga de comandos do inspector 104 → 105 (conta-gotas = capacidade nova); o controlo balão passou a `INSP_RAY_ORIGIN_X`; `ph2d-app-components` 80 848 ≤ 80 869 (cortando comentários históricos); shell −2 linhas líquidas; `ph2d-game-smoke-fotos` 1075/1077.
- O que um merge pode partir: a cena 25 (número da próxima cena = 26); a carga de comandos do inspector (105) e o teto de LOC de `ph2d-app-components`, que somam entre linhas; `INSP_RAY_ORIGIN_X` se outra linha usar o mesmo id de balão.

## Smoke do dono (feito, OK 13/11)

1. `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional && cargo build --profile smoke -p ph2d-host-desktop && PH2D_GAME_SMOKE=25 ./target/smoke/ph2d-host-desktop`
2. O cartão da bala aparece com Target «Target». Marque Bounce: surgem Max Bounces 3 e Bounciness.
3. Clique no conta-gotas do Target e depois em «Straight»: o Target vira «Straight». Ctrl+Z volta. Esc cancela o conta-gotas.
4. «Speed change» abre.
5. Play: Straight bate na parede, Bouncy quica, Arc cai, Short Range para, Missile acerta o Target.
6. Deu errado se: o Target fica vazio, o conta-gotas pega a própria bala, ou o Missile passa longe do Target.

Nota: no Play a bala que acabou some (RunOnCopy, `crates/ph2d-ecs/src/lifetime.rs:204` `removable_in_run`; Stop restaura). Regra da casa (moedas), não defeito.

## ~~ABERTO~~ FEITO (13/11, onda b) — [HANDOFF_CONTINUACAO_2026-11-13b_CORRIGIR_TUDO.md](HANDOFF_CONTINUACAO_2026-11-13b_CORRIGIR_TUDO.md)

Os 5 itens abaixo estão FEITOS (texto original mantido); o que sobra (smoke, Ray Sensor) está no handoff novo.

1. **FEITO** — Checkbox do APP INTEIRO dentro de uma MOLDURA, como o Godot; o losango de keyframe segue o padrão dos campos (acima do canto superior direito).** Central: `ph2d_editor_core::widget::paint_card_check` / `card_check_rect` (T64) e `property_row::paint_check_row`; varrer por CONSUMIDOR; fotos antes/depois de vários cartões. Memória: `project-memory/project_checkbox_in_a_frame_like_godot_2026_11_13.md`.
2. **FEITO** — **Homing mantém a velocidade constante** (como o ProjectileMovement do Unreal). Hoje `homing_accel` só puxa: a velocidade cresce, a curva abre e a bala erra sem Max Speed. Lei em `crates/ph2d-projectile/src/lib.rs` `advance()`. Depois a cena 25 tira o Max Speed do Missile (resumo volta a «None»). O dono aceitou.
3. **FEITO** — **Arrastar um objeto da Hierarquia para o campo Target** (regra do dono 02/11: arrastar, digitar ou conta-gotas). Conferir se o Target da câmera já aceita o arrasto e fazer os dois.
4. **FEITO** — **Cena 25:** a bala Arc cai para fora da cena no Play (o Floor é só imagem).
5. **FEITO** — **Comentário velho:** `crates/ph2d-physics-ecs/src/bridge/projectile.rs` (~48-50) e `components/health.rs:248` dizem que a bala de documento «pára e fica»; no Play (RunOnCopy) ela é removida e o Stop restaura.

Depois destes: **Ray Sensor** (fila 23, «Corpo e movimento»).
