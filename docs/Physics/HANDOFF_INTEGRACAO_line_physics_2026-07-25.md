# HANDOFF DE INTEGRAÇÃO — `line/physics` → `main` (2026-07-25)

> Para o **agente integrador**, sob ordem EXPLÍCITA do Enio. A linha está FECHADA,
> todos os smokes aprovados. O implementador **não integra nem pusha** (§0.7); este
> handoff é o que o integrador executa. **PARE antes do push — o ship é do Enio.**

## TL;DR

Integrar a **jornada reaberta** da `line/physics` (2026-07-23 → 25): **44 commits**,
93 arquivos, +8782/−1274. **É um FAST-FORWARD** — o `main` não se moveu desde o
fork, então **não há rebase nem conflito**. O trabalho real do integrador é **rodar
o gate da árvore combinada** (a linha foi feita com `cargo check -p` / testes
por-crate, que NÃO rodam os arch-gates do shell nem a contagem dos três espelhos).

## Estado do git (conferido)

- **worktree:** `/home/enio/Documentos/Projetos/PH2D/Worktrees/line-physics`
- **branch:** `line/physics` · **HEAD:** `13f084207`
- **merge-base == `main` == `origin/main` == `df91ef6ec`** (o main NÃO avançou desde o fork)
- ⇒ `git merge --ff-only line/physics` a partir do `main` é FAST-FORWARD, zero conflito.

⚠️ **SE o `main` tiver avançado** quando você for integrar (outra linha entrou
primeiro), **isto deixa de ser FF**: rebase `line/physics` em `main` e **RE-CONTE**
os números que somam entre linhas — `PROJECT_SCHEMA`, o registro do `ph2d-physics-ecs`,
os espelhos, o hash c9. *O valor certo se CONTA, não se escolhe*
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). Hoje, com o main
parado, os números abaixo valem como estão.

## Como integrar (mecânica)

1. `cd` para a árvore PRIMÁRIA (o worktree do `main`). Confirme `git rev-parse --short main` == `df91ef6ec`.
   - Se ≠, ver o aviso acima (rebase + re-contar) e **PARE e reporte ao Enio** se houver colisão de mesmo-símbolo.
2. **FF:** `git merge --ff-only line/physics` (ou `scripts/foundational-integrate.sh`, que já roda o gate da árvore combinada). Como `line/physics` é superconjunto estrito do `main`, não há conflito.
3. **GATE DA ÁRVORE COMBINADA (o ponto todo):** `./scripts/ship.sh` — paridade EXATA com o CI (fmt, clippy `--all-targets`+features, machete, deny, audit, nextest `--cargo-profile ci-test`, typos). Ele roda o **workspace inteiro**, que é o que pega o que o `cargo test -p` por-crate não pega (abaixo).
4. Corrija todo `✗`. **PARE antes do push.** O Enio faz o push + babysit do CI.

## Escopo — as waves (todas smoke-aprovadas pelo Enio)

Família das ZONAS (fecha o W-Area):
- **W-AreaFrame** (`=34`) — girar o sensor gira o vento; toggle `Force Axes: Zone|World`. Marcador `AreaForceWorldAxes` (registro 19→20). `libm::sincosf` na fronteira, `atan2` barrado do caminho c9.
- **W-AreaFalloff** (`=35`) — a força/torque desvanecem do centro à borda (a régua é a silhueta da zona). Componente `AreaFalloff` (20→21). Sem `hypot`/transcendental no c9.
- **W-AreaMirror** (`=36`) — espelhar a zona espelha o vento (vetor REFLETE) e inverte o giro (pseudoescalar). **Sem componente/id/schema novo** (a lateralidade é função da POSE).

BAKE:
- **W-BakeRange** (`=37`) — o INÍCIO do loop é honrado (`[2s,5s]` assa `[2s,5s]`). Sem schema.
- **Fidelidade do bake** (`=37` re-smoke) — bake **sem simplificação**: uma chave por tick, `Interp::Linear`, exato a 60 fps (o fit foi removido do bake; o `simplify_recorded` do RECORD fica).

JOINTS (o miolo desta entrega):
- **W-JointAnchor** (`=38`) — dot de âncora de joint arrastável no canvas (`PointGizmoView`/`paint_point_gizmo` em editor-core). Sem schema.
- **W-BakeJoint** (`=39`) — assar UM corpo de um rig articulado puxa o componente conexo DINÂMICO inteiro (`ph2d_physics_ecs::jointed_group`). Sem schema.
- **W-JointAuthoring** (`=40`) — §12 redesenhada: linha por corpo (Body A/B + nome vigente + eyedropper que ARMA um canvas-pick). Sem schema. Ids só de painel (`INSP_JOINT_PICK_A/B`).
- **W-AnchorFollow** (`=41`, padrão-ouro W1) — a âncora vira **body-local por corpo** (`PhysicsJoint.local_a/b/anchored`, a rep nativa do rapier), então segue o corpo em vez de deslizar. **`PROJECT_SCHEMA` 29→30** (campos apendados; único bump da jornada). `bridge.rs` → `bridge/readback.rs`.
- **W-JointCreate** (`=40` atualizado) — escolher o tipo na criação (seletor "Join As" na §11, `INSP_PHYS_JOIN_KIND` + `App.join_kind`) + auto-seleção da joint nova + re-seed da âncora na troca de tipo.

Fora do domínio da linha, pegou carona (uma linha):
- **fix(timeline) `a0d429305`** — a `Dur(s)` aceita QUALQUER duração (o teto `u16::MAX` era palpite). Toca `ph2d-timeline`, não física. Inofensivo, mas registrado aqui por honestidade.

## Números a confirmar (o gate da árvore combinada os checa)

| O quê | main (`df91ef6ec`) | após integração | nota |
|---|---|---|---|
| `PROJECT_SCHEMA` | 29 | **30** | único bump (W-AnchorFollow apendou 3 campos ao `PhysicsJoint`); gate `a_schema_bump_anywhere` = tripla `(30, 8, 13)` |
| registro `ph2d-physics-ecs` | 19 | **21** | W-AreaFrame (→20), W-AreaFalloff (→21); os JOINTS não adicionam componente |
| espelhos (ecs count em `ph2d-render`/`ph2d-script`) | — | **em sincronia** | verificado verde no worktree; é o eixo que só a árvore combinada vê |
| c9 `physics-ecs-c9` | (07-22) | **83 corpos** | hash determinístico debug≡release; **MUDA** vs a base — CORRETO (torque/frame/falloff/mirror mudam a POSE; os readouts não) |
| cenas de smoke | ≤ `=33` | **até `=41`** | novas: `=34..=41`; a `=40` foi reescrita (Join As) |
| ADRs novos | — | **nenhum** | nada em `docs/architecture/decisions/` |

## Classes de vermelho-latente a vigiar (a lição da `line/Vector`)

O `cargo test -p <crate>` da linha NÃO roda:
1. **Arch-gates do shell** (`shells/desktop/tests/*.rs`) — só correm na varredura impactada. `./scripts/ship.sh` (workspace) os roda. Inclui `file_loc_caps` (600), `architecture_panel_wiring_parity`, `no_tofu_glyphs`, `every_panel_the_shell_drives_is_in_its_registry`.
2. **Contagem dos TRÊS espelhos** (registro ecs afirmado em `ph2d-physics-ecs` + `ph2d-render` + `ph2d-script`) — **verificado verde** aqui, mas re-confira no gate combinado.
3. **Caps de LOC** — crates (`architecture_workspace_file_loc_cap`, 700), shell (600), painel (`architecture_panel_loc_cap`, 200/fn + 600/arquivo). Todos verdes no worktree.

Tudo verde no worktree por-crate + os gates de painel/editor-core/shell rodados por-crate nesta sessão; o `ship.sh` é a confirmação final.

## Smokes (o Enio já aprovou todos; re-rodar é opcional)

```
env PH2D_PHYSICS_SMOKE=<n> cargo run -p ph2d-host-desktop --release
```
`=34` frame · `=35` falloff · `=36` mirror · `=37` bake range/fidelidade · `=38` anchor gizmo · `=39` bake joint · `=40` **criar joint (Join As + auto-seleção + re-pick)** · `=41` **a âncora segue o corpo**.

## O que NÃO fazer

- **NÃO pushe.** FF + `ship.sh` verde, e **PARE** — o push + babysit do CI são do Enio ([[feedback_ship_only_enio_end_of_all_lines]]).
- Contrato congelado (§6): **nada tocado** — `NodeOp=2`/`OpResolver=1`/`NodeManifest=8` intactos; a física não é contrato congelado.
