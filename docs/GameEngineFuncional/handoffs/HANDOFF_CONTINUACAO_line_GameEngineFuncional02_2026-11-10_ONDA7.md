# HANDOFF de continuação — `line/GameEngineFuncional02`, 10/11, onda 7 (física fluida sob animação: juntas de mundo, formas reafinadas por tique)

Leitor: o próximo agente da linha (ou o integrador). Anterior: [ONDA6](HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA6.md). Último handoff de integração (já no `main`): [OUTLINE_E_F3](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-09_OUTLINE_E_F3.md). Memória: [física fluida sob animação](../../../project-memory/project_physics_fluid_under_animation_2026_11_10.md).

## 1. Identidade

Branch `line/GameEngineFuncional02`, worktree `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02`. Onda 7 = 2 commits sobre o handoff da onda 6. `PROJECT_SCHEMA` = 231, **inalterado**; registos, ADR, contratos congelados §6: 0. Nada integrado, nenhum push.

| SHA | O quê |
|---|---|
| `f7b233f13b` | curas: juntas de mundo no referencial do objeto, formas reafinadas no lugar e por tique |
| `b9a15ad2d6` | cena 66 animada (`PH2D_PHYSICS_SMOKE_ANIMA=1`) + gate + fotos |

## 2. Decisões do dono (10/11)

Smoke da onda 7 = **OK** (cena `PH2D_PHYSICS_SMOKE=66 PH2D_PHYSICS_SMOKE_ANIMA=1`).

**Respostas ao §7 (dono 10/11)** — todas as 5 perguntas respondidas; a lista da ONDA 8 está na §7.
1. Crescer a partir do centro ou de âncora: «Cresce com o pivot da shape como centro se a shape sofre a transformação. Se o objeto (sprite, vector, etc) é transformado, cresce como filho do objeto (já está assim). O que ainda não temos é a possibilidade de mudar o pivot do collision shape. Vc pode implementar a seção Look completa do sprite para o collision shape.»
2. Penetração durante a rampa: «Se conseguir fazer melhor que a Godot faça. Desde que não perca performance e precisão.»
3. Width/Height 1×/quadro: «Meça»
4. Motor da junta na Timeline: «Corrija para graus»
5. Herdados do OUTLINE_E_F3 §11.5: «Corrija»

## 3. O que mudou

**Medição antes de curar.** O caso de UMA forma (caixote no chão; Width/Height pela curva, ScaleY corpo e forma, encolher a dormir, pilha, parede) **NÃO reproduziu defeito**: fundo ≤ 0,7 cm, recriações 0, scrub = Play. Cena 120: a Table salta a 15 m/s ao crescer — causa: o tampo entra na Updraft Zone (física certa, não defeito). **Reproduzido na cena 66:** o Gate A adormece no batente e o rapier 0.35.3 só acorda o dono de um colisor mudado que já tenha contactos (`narrow_phase/pair_management.rs`). Junta de mundo invertida (medido tique 120: [−30,+10]→−10; motor +1 rad/s→−114°; alvo +20°→−20°; Slider [−0,5,+0,2]→+0,5).

**`f7b233f13b` (19 ficheiros, +1163 −220).**
1. `joint_desc(.., lado_b_mundo)` + `no_outro_referencial` (involução: limites [−max,−min], motor −speed/−target exceto Rope, eixos Custom) em `reconcile_joints` e `drive_joint_params`; `views.rs` desenha o cartão no pino de mundo (angle_a = mundo, angle_b = objeto, limites/motor desfeitos). Testes `tests/it/joint_world_frame.rs` (9), mutação 9/9.
2. `PhysicsWorld::retune_part` + `copia_colisor` partilhado (`world/body_build.rs`, `world/parts.rs`); `bridge/parts.rs` reafina no lugar com o mesmo dono (ring sobrevive).
3. `retune_body` faz `wake_up(true)`.
4. `bridge/shape_drive.rs` `drive_shapes` por tique no laço de Play (`dispatch.rs`) e de replay (`rewind.rs`, também no ramo replayed==0); extração `desc_do_corpo` (`bodies.rs`) e `desc_da_peca` (`parts.rs`). O memo rest/local de corpos e peças viaja no checkpoint (`ControllerMemory.formas/pecas`, `tape.rs`, reposto em `seed_player_states`) — reafinar às cegas depois do semeio quebrava o bit-exacto (mesa sem animação 1e-7); não reafinar deixava a forma do checkpoint.
5. «Voltar a zero recria o corpo» cura-se pelo 4. ⛔ **Recusa medida:** a regra «só a pose no tique 0» e um campo `inicio` foram escritos, a mutação não sangrou, e saíram.
- Testes `tests/it/forma_animada.rs` (11), mutação 10/10 + controlo. Diagnóstico novo `#[doc(hidden)]`: `body_sleeping`, `body_handle_bits`, `PhysicsWorld::body_sleeping`.

**`b9a15ad2d6`.** Cena 66 ganha `PH2D_PHYSICS_SMOKE_ANIMA=1` (`author_limit_and_width_tracks`: Limit min −25→−60 de 2,6 a 3,4 s; largura do Gate A 1,5→2,4 m de 3,5 a 4 s; tudo dentro dos 4 s da régua). Gate `a_cena_66_animada_leva_o_portao_ao_minimo_novo` pelo caminho do app (nasce vermelho sem a cura: −25,00°) em `ph2d-app-physics/src/physics_smoke_joint_copy{,_tests}.rs`. Fotos: [01](fotos_fisica/fluida/01_portao_segue_o_batente.png) · [02](fotos_fisica/fluida/01_portao_segue_o_batente_02.png) · [03](fotos_fisica/fluida/01_portao_segue_o_batente_03.png).

## 4. O que um merge pode partir

- Assinatura pública `joint_desc` ganhou o 5.º parâmetro `lado_b_mundo`.
- Quem lê `JointView` de um pino de mundo recebe agora o referencial do OBJETO.
- `retune_body` acorda sempre.
- `BodyRef`/`PartRef` inalterados em campos; `ControllerMemory` ganhou 2 campos (`formas`, `pecas`).
- `drive_shapes` entra nos laços de dispatch/rewind ao lado de `drive_joint_params`; quem reordenar esses laços perde o bit-exacto do scrub.
- `physics_apply.rs` continua 599/600 (não tocado). Shell não tocada.
- Contadores como DELTA: schema 0 · registos 0 · cenas de smoke 0 (a 66 ganhou uma env, não cena) · ADR 0 · contratos §6 0.

## 5. Prova de fecho (na linha; suíte completa e `nextest-impacted` NÃO corridos: do integrador)

- Testes: os de §3. Mutação: joint_world_frame 9/9 · forma_animada 10/10 + controlo.
- Crates inteiras: `ph2d-physics-ecs` 935 ✓, `ph2d-physics` 360 ✓; `ph2d-app-physics` por nome (joint/overlay/canais/smoke/joint_copy) ✓.
- `cargo fmt` feito nas crates tocadas. **Clippy das crates tocadas NÃO corrido ainda** → fazer no fecho (`PH2D_SUITE=fecho`).

## 6. Smokes (dono: OK em 10/11)

Prefixo: `cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02 && … cargo run --profile smoke -p ph2d-host-desktop`.

1. **Portão segue o batente.** `PH2D_PHYSICS_SMOKE=66 PH2D_PHYSICS_SMOKE_ANIMA=1`, Play. O Limit min do Pin A desce até −60° e o portão desce com ele sem ficar preso; a largura do Gate A cresce a 2,4 m perto do fim. Errado: o portão para em −25° e não acorda. Fotos: §3.

Armadilha das fotos: `docs/Components/ferramentas/fotografa_cena.sh` — env vars TODAS no 1.º argumento; casos de física com prefixo `fisica_`.

## 7. ONDA 8 (por ordem; o dono já decidiu, §2)

1. **Secção Look completa na forma de colisão.** Dar ao objeto-filho Collision Shape ([ADR-0199](../../architecture/decisions/0199-a-forma-de-colisao-e-um-objeto-filho-do-corpo.md)) a secção Look COMPLETA que a Sprite tem no Inspector (inclui Pivot e Align; ver [ALIGN_E_PIVOT](HANDOFF_INTEGRACAO_line_GameEngineFuncional_2026-11-09_ALIGN_E_PIVOT.md)); o tamanho animado da forma cresce em volta do PIVOT dela (objeto transformado: cresce como filho do objeto, já é assim). **Também (dono 10/11):** «implemente também a possibilidade de usar a ferramenta pivot na canvas para o collision shape» ⇒ a ferramenta Pivot da barra do canvas (a mesma que move o pivot da Sprite) funciona sobre a forma de colisão selecionada: arrastar o pivot dela no canvas, com Undo e Auto Key, coerente com o campo Pivot da secção Look. **Primeiro passo:** inventariar a secção Look da Sprite no código (cartão, campos, canais/Auto Key, gates) e a ferramenta Pivot do canvas antes de desenhar.
2. **Penetração de ~0,5–1 cm numa rampa de altura/escala:** melhorar além de Godot/Box2D sem perder desempenho nem precisão. Medir antes; tabela de medição ao lado.
3. **Medir** se Width/Height pelo cartão (canais FISICA) chegarem só 1×/quadro ao componente (o `TimelineScene.put` por tique escreve `CanaisEscritos`, mas `canais_corpo::aplicar` só corre no quadro) fazem diferença visível/medível; curar só se medir.
4. **Graus:** `PropKind::JointMotorTarget/Speed` em graus e no referencial do objeto na Timeline (`ph2d-timeline` `apply_prop` `JointField` + `shells/desktop/src/timeline_bridge_keys.rs`; coordenar com a linha Timeline viva — se o rebase conflitar fora dos ficheiros da linha, PARAR e reportar ao Enio).
5. **Herdados** do [OUTLINE_E_F3](HANDOFF_INTEGRACAO_line_GameEngineFuncional02_2026-11-09_OUTLINE_E_F3.md) §11.5 (censo de sinais, O(pontos²) do traço, ponte de navegação, macro que muda o nº de pontos sem modal, painel `PH2D_CARDS=0`): ler a §11.5 e corrigir cada um com teste nascido vermelho.

O fecho da linha (clippy, handoff de integração) e a integração continuam só por ordem do Enio.

**Riscos:** ficheiros `.log` das fotos por rastrear em `docs/GameEngineFuncional/handoffs/fotos_fisica/` — NUNCA `-A`; comite por path. `physics_apply.rs` 599/600. Integração só por ordem do Enio; smoke do dono não é aprovação.

## 8. Prompt de continuação (colar numa janela nova)

```
/pd-linha-assumir line/GameEngineFuncional02

Worktree: /home/enio/Documentos/Projetos/PH2D/Worktrees/line-GameEngineFuncional02
Antes de ler: cd lá, `pwd`, `git branch --show-current` (tem de dar line/GameEngineFuncional02).
HEAD = o commit do handoff docs/GameEngineFuncional/handoffs/HANDOFF_CONTINUACAO_line_GameEngineFuncional02_2026-11-10_ONDA7.md
(confira com `git log --oneline -3`). Leia esse handoff (§2 respostas do dono e §7 ONDA 8) e CLAUDE.md.

Estado: onda 7 feita; smoke do dono OK em 10/11 (cena PH2D_PHYSICS_SMOKE=66 PH2D_PHYSICS_SMOKE_ANIMA=1):
a física ficou fluida com limites de junta e tamanhos de forma animados (junta de mundo no referencial
do objeto, formas reafinadas no lugar e por tique, corpo acorda). O dono já respondeu às 5 perguntas
do ABERTO (estão na §2/§7 do handoff). Nada integrado, nada com push. PROJECT_SCHEMA = 231.

Missão (onda 8), os 5 itens da §7 por ordem, sem pedir de novo a decisão ao Enio:
(1) começando pelo INVENTÁRIO da secção Look da Sprite no código (cartão, campos, canais/Auto Key,
gates), depois dar à forma de colisão a secção Look completa (Pivot e Align incluídos), com o tamanho
animado a crescer em volta do pivot dela, e a ferramenta Pivot do canvas a funcionar sobre a forma de
colisão selecionada (dono: «implemente também a possibilidade de usar a ferramenta pivot na canvas para
o collision shape»; arrastar, com Undo e Auto Key, coerente com o campo Pivot do Look); (2) penetração na rampa: melhorar além de Godot sem perder
desempenho nem precisão; (3) medir se Width/Height por tique fazem diferença, curar só se medir;
(4) motor da junta (JointMotorTarget/Speed) em graus e no referencial do objeto na Timeline;
(5) herdados do OUTLINE_E_F3 §11.5, cada um com teste nascido vermelho. Medir antes de curar; o fecho
da linha (clippy, handoff de integração) só por ordem do Enio.

Regras de sempre: UI só em inglês, por tokens/i18n (zero hex, zero string solta); teste por NOME,
nascido vermelho, com prova de mutação; inner loop = cargo-check-narrow; comando pesado por
scripts/ph2d-run.sh; commit só por path (`git commit --no-verify -m … -- <paths>`), nunca -A,
não comite os .log de fotos; physics_apply.rs está em 599/600 — cure por MOVER; suíte completa é do
integrador; SEM push, integração ou fecho sem ordem do Enio; smoke ao Enio em passos numerados com
foto tirada antes via docs/Components/ferramentas/fotografa_cena.sh (env vars todas no 1º argumento,
casos de física com prefixo fisica_) e imagem como link markdown relativo à raiz do repo principal
(Worktrees/line-GameEngineFuncional02/docs/…png). Respostas ao Enio: curtas, sem jargão.
```
