# HANDOFF de INTEGRAÇÃO — `line/physics` · jornada REABERTA (2026-07-19)

> Para o **agente integrador** (por ordem EXPLÍCITA do Enio; a linha não integra nem pusha
> sozinha — CLAUDE.md §0.7).
>
> ⚠️ **O nome do arquivo diz só "scale_collider" por acidente histórico — este handoff cobre a
> JORNADA REABERTA INTEIRA:** **W6** (a escala alcança o collider, §1–§7) **+ W7 (sensores /
> triggers, §8)**, e mais o que o Enio pedir antes da ordem de integração. Estado por-wave detalhado:
> [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) §W6/§W7. Resumo de produto: `CLAUDE.md` §5.

## §1 — O que a wave faz (uma frase)

Um sprite escalado agora escala o **collider** junto (antes só o quad crescia): a escala de
**mundo** (`Transform.scale`, composta pela cadeia de pais) alcança o corpo físico. **Ball**
uniforme fica círculo, **Ball** não-uniforme vira **elipse** (decisão de produto do Enio — o
collider casa com o sprite, não colapsa num círculo), **Cuboid** escala per-eixo.

## §2 — Isolamento e riscos de merge

- **Contratos congelados (§6): NENHUM tocado.**
- **Foundational tocado:** `ph2d-physics` (o módulo de física, dono desta wave) — **aditivo**:
  `ShapeDesc` ganhou `Ellipse` no **FINAL** (append-only, a regra do próprio enum). O caminho
  determinístico (`c9`) fica intacto exceto por **+1 corpo** no `physics_ecs_c9`.
- **`ph2d-physics-ecs`** (a ponte, também desta linha) e o **shell** (`physics_overlay.rs`,
  consumidor) — aditivos.
- **⚠️ ZERO bump de schema.** O `ColliderShape` **autorado/serializado** não mudou; a elipse é
  derivada só na plain-data `ShapeDesc`; a escala já viajava no `Transform` persistido.
  `PROJECT_SCHEMA` / `DOC_VERSION` **intocados** — **nada a CONTAR** nesta wave
  ([[feedback_numbers_that_sum_across_lines_count_dont_pick]] não se aplica aqui).
- **Único número que se move:** o `body_count` do `physics_ecs_c9` (51 → **52**) e, por
  consequência, o hash `physics-ecs-c9`. O `spike.yml` compara esse hash **entre os 3 OSes**,
  nunca a um literal nem ao raw `physics-c9` — então mudá-lo é seguro (confirmado lendo o job
  `determinism-compare`). Não há gate que fixe `body_count`.

## §3 — Arquivos (todos no módulo de física + 1 consumidor no shell)

| Arquivo | Mudança |
|---|---|
| `crates/ph2d-physics/src/world/shape.rs` | **NOVO** — `ShapeDesc` + `ELLIPSE_SEGS` + `ellipse_vertices` (split de `world.rs`, que bateu o teto de 700 LOC) |
| `crates/ph2d-physics/src/world.rs` | `pub mod shape;` + `pub use shape::…`; arm `ShapeDesc::Ellipse` em `spawn_body` (`convex_polyline`, fallback a `ball`) |
| `crates/ph2d-physics/src/lib.rs` | re-export `ELLIPSE_SEGS`/`ellipse_vertices` (via `world`) |
| `crates/ph2d-physics/Cargo.toml` | `libm = "=0.2.16"` (pin do workspace — usada por `ellipse_vertices`; machete OK) |
| `crates/ph2d-physics/tests/ellipse_collider.rs` | **NOVO** — AABB da elipse no sim + determinismo da tesselação |
| `crates/ph2d-physics-ecs/src/scale.rs` | **NOVO** — `scaled_shape` (a porta única) |
| `crates/ph2d-physics-ecs/src/bridge.rs` | `body_desc` chama `scaled_shape(col.shape, t.scale)` (removeu o `match` inline) |
| `crates/ph2d-physics-ecs/src/lib.rs` | re-export `scaled_shape` + `ShapeDesc`/`ellipse_vertices`/`ELLIPSE_SEGS` |
| `crates/ph2d-physics-ecs/src/bin/physics_ecs_c9.rs` | +1 bola não-uniformemente escalada (elipse cross-OS); doc `body_count 52` |
| `crates/ph2d-physics-ecs/tests/scale_reaches_the_collider.rs` | **NOVO** — 6 gates (4 pure + 2 behavioral) |
| `shells/desktop/src/render_loop/physics_overlay.rs` | `collider_outline(ShapeDesc, …)` + arm `Ellipse`; `outlines` resolve por `scaled_shape`; +2 gates |
| `shells/desktop/src/physics_smoke.rs` | cena `PH2D_PHYSICS_SMOKE=9` (`physics_smoke_scale`) + linha na tabela + dispatch |
| `CLAUDE.md` · `docs/Physics/{HANDOFF_line_physics,BUGS_physics}.md` | docs (esta wave) |

## §4 — Gates (todos verde local; mutação-provados)

- **`scale_reaches_the_collider.rs`** (ecs): `a_cuboid_inherits_per_axis_scale` · `a_uniform_scale_keeps_a_ball_a_circle` · `a_nonuniform_scale_makes_the_ball_an_ellipse` · `an_unscaled_body_resolves_byte_identically` (regressão) · `a_scaled_dynamic_ball_rests_on_its_scaled_collider` (behavioral) · `a_parented_bodys_collider_uses_the_parents_world_scale` (behavioral, **pai na fixture**).
- **`ellipse_collider.rs`** (physics): `an_ellipse_collider_has_the_authored_half_extents` (AABB do collider VIVO) · `ellipse_vertices_are_deterministic_and_axis_aligned`.
- **`physics_overlay::tests`** (shell): `a_nonuniform_scaled_ball_is_drawn_as_an_ellipse` · `a_parented_bodys_outline_grows_with_its_world_scale`.
- **7 mutações, todas sangram** (verificadas com `cp`-restore, nunca `git checkout`): dropar a
  escala em `scaled_shape` mata 4 · não-uniforme→círculo mata a resolução · spawn `ball(rx)`≠elipse
  mata a AABB · swap dos eixos da elipse mata AABB+determinismo · overlay elipse→círculo mata o
  desenho · `outlines` sem `t.scale` mata o crescimento com o pai.

## §5 — Paridade CI rodada LOCAL (o que falta é só o que exige a matriz 3-OS)

Verde nesta máquina (pin `1.95`):
- `cargo test -p ph2d-physics -p ph2d-physics-ecs` (31 bins, 0 falhas) + overlay (11/11).
- `cargo clippy -p ph2d-physics -p ph2d-physics-ecs --all-targets` (limpo).
- `cargo fmt --check` (limpo) · `cargo machete` (sem deps mortas — `libm` é usada) · `typos`
  no diff (limpo) · **LOC-cap** (`world.rs` 697 < 700 após o split).

**Falta (só na matriz do CI, no push):** a comparação cross-OS do `physics-ecs-c9` — o
verdadeiro gate HR-5 da elipse (localmente só provei repeatability + o pin de vertex). Rode o
**`scripts/ship.sh` inteiro** no fechamento da jornada (não confie nesta tabela — no W5 o ship
achou um `typos` real que o handoff dizia limpo: [[feedback_ship_parity_gaps_ci_only]]).

⚠️ **Ambiente:** o default do rustup se perdeu nesta máquina; só o pin `1.95` está instalado.
Rode `env RUSTUP_TOOLCHAIN=1.95 bash scripts/ship.sh` (o `ship.sh` chama `cargo` nu).

## §6 — Smoke visual — `PH2D_PHYSICS_SMOKE=9`

Construído (`shells/desktop/src/physics_smoke.rs::physics_smoke_scale`). 4 bolas caem, cada uma
um `Ball` escalado diferente: **círculo** de referência · **2× uniforme** (círculo maior,
repousa mais alto) · **não-uniforme** (ELIPSE, cai deitada e balança — um `Ball` que rola como
elipse) · **parenteada** sob um rig 2× (o collider herda a escala do PAI, prova a escala de
MUNDO). O oráculo é o contorno (tecla `B`, default ON): desenha a forma RESOLVIDA, então um
scale→collider morto traçaria o raio autorado dentro de cada sprite escalado. Os gates
behavioral cobrem a física; a cena é para o olho. **Rodar:**
`cd Worktrees/line-physics && env PH2D_PHYSICS_SMOKE=9 RUSTUP_TOOLCHAIN=1.95 cargo run -p ph2d-host-desktop`
([[feedback_run_command_include_cd]]).

## §7 — Ordem de integração

A linha estava **100% contida na `main`** (integração anterior foi fast-forward puro), então este
W6 é o **único delta** sobre a `main` de hoje. Sobreposição com outras linhas: **nenhuma
esperada** — os arquivos são do módulo de física (mais 1 arquivo do shell, `physics_overlay.rs`,
que nenhuma outra linha viva toca). Se `git rebase main` conflitar **fora** dos meus arquivos
(mesmo-símbolo, DIRETRIZ §1.5.5), PARE e reporte ao Enio.

---

**Resumo (DIRETRIZ §1.5.9):** *Linha `physics` W6 pronta — a escala de mundo alcança o collider,
Ball não-uniforme = elipse (`ShapeDesc::Ellipse`, decisão do Enio). Porta única `scaled_shape`
(ponte + overlay). Foundational tocado: `ph2d-physics` (aditivo, append-only, c9 +1 corpo).
Contratos congelados: nenhum. **Zero bump de schema** (o `ColliderShape` autorado não muda; a
escala já vive no `Transform`). 10 gates novos, 7 mutações, todas sangram; batched gate verde
local; smoke visual `PH2D_PHYSICS_SMOKE=9` pronto. Falta só a matriz cross-OS do `ship.sh`.
Aguardo ordem de integração.*

---

# §8 — W7: SENSORES / TRIGGERS (o primitivo)

Item (B) do cardápio, pedido pelo Enio. Detalhe: [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) §W7.

**O que faz:** um `Collider.is_sensor` que **atravessa** (sem forças de contato) mas o solver
**reporta o que o sobrepõe**. A DETECÇÃO é contida na física; o **consumidor de gameplay**
(colisão→sinal) é outra camada, cross-line, decisão do Enio. O consumidor VISÍVEL desta wave: o
overlay **acende** o sensor (magenta idle→bright) + estado consultável (`bodies_inside`) + toggle
"Solid | Sensor" no Inspector §11.

**Isolamento / riscos de merge:**
- **Contratos congelados: NENHUM.** **Foundational tocado:** `ph2d-physics` (aditivo: `BodyDesc.is_sensor`
  APENDADO ao FINAL; `intersecting_body_pairs` novo em `world/sensors.rs`; `world/desc.rs` split de LOC).
- **`ph2d-physics-ecs`** (aditivo: `Collider.is_sensor`, trigger state em `bridge/triggers.rs`,
  `body_desc` mudou-se pra `scale.rs`) · **`ph2d-editor-core`/`ph2d-panel-inspector`** (o toggle) ·
  **shell** (overlay acende, Inspector, smoke, persistência).
- ⚠️ **`PROJECT_SCHEMA` 26 → 27** (o W6 NÃO bumpou; o W7 sim, porque `is_sensor` é campo APENDADO
  a um component serializado — mesmo padrão do v21 `layer`). **Tripla-pin `(27, 8, 13)`** no
  `project_tests.rs`. ⚠️ **O número se CONTA:** se outra linha bumpar o schema antes da integração,
  re-conte ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]).
- **C9:** o hash **não** inclui o trigger state (só poses), então o `intersecting_body_pairs` não
  move o hash; mas o `world/desc.rs`/`world/sensors.rs` são novos módulos do mesmo crate.

**Arquivos (aditivos):** `ph2d-physics`: `world/desc.rs` (novo) · `world/sensors.rs` (novo) ·
`world.rs` (mods + `.sensor()` no spawn) · `lib.rs` re-export. `ph2d-physics-ecs`: `components.rs`
(`is_sensor`) · `bridge.rs` (field + call, `body_desc` saiu) · `bridge/triggers.rs` (novo) ·
`bridge/hold.rs` (clear) · `scale.rs` (`body_desc`). `ph2d-editor-core`: `inspector_model.rs`
(`is_sensor` + `Sensor(bool)`) · `ids/inspector.rs` (`INSP_PHYS_SENSOR`). `ph2d-panel-inspector`:
`sections/physics.rs` · `populate.rs` · `event_physics.rs` · `tests/seam_physics.rs`. shell:
`render_loop/physics_overlay.rs` (cores + `triggered`) · `render_loop/mod.rs` (coleta `triggered`) ·
`render_loop/inspector_physics.rs` (build + apply) · `physics_smoke.rs` (`=10`) · `project.rs`
(schema 27) · `project_tests.rs` (tripla-pin). Testes: `ph2d-physics/tests/sensors.rs`,
`ph2d-physics-ecs/tests/sensors.rs`, `+persistence.rs`.

**Gates (10 novos, 3 mutações provadas — ver §W7 do tracker).** Verde local: 33 bins de física,
overlay 13/13, panel seam 11/11, inspector 7/7, project 14/14, clippy `--all-targets` (física +
editor + panel + shell), machete, fmt, LOC-cap.

**Smoke: `PH2D_PHYSICS_SMOKE=10`** (bola bloqueada por sólido × bola atravessa o sensor que acende).

**Aberto (a próxima camada, decisão do Enio):** o **sinal de gameplay** — colisão→ação (um
`Marker` da timeline / um callback do `ph2d-script`), cross-line. O primitivo é o pré-requisito.

**Resumo (DIRETRIZ §1.5.9):** *W7 pronta — o primitivo de trigger: `Collider.is_sensor` atravessa
e o solver reporta os overlaps (`bridge/triggers.rs`), o overlay acende, o Inspector autora, e o
estado é consultável (`bodies_inside`). Foundational tocado: `ph2d-physics` (aditivo). Contratos:
nenhum. `PROJECT_SCHEMA 26→27` (tripla-pin `(27,8,13)`). 10 gates, 3 mutações. Smoke `=10`. O sinal
de gameplay é a próxima camada, cross-line, aguardando o Enio.*
