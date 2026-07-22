# HANDOFF de INTEGRAÇÃO — `line/physics` (jornada completa, 2026-07-21)

> Para o **agente integrador**, por ordem EXPLÍCITA do Enio. A linha **não integra nem pusha
> sozinha** (CLAUDE.md §0.7): ela fechou, entregou isto, e parou.
>
> **Supersede** [`HANDOFF_INTEGRACAO_line_physics_scale_collider_2026-07-19.md`](HANDOFF_INTEGRACAO_line_physics_scale_collider_2026-07-19.md)
> — aquele cobria W6/W7/Weld/BakeChannels, que são os **quatro primeiros** commits desta lista.
> Este cobre a jornada inteira. O antigo fica como registro; se os dois discordarem, **este vale**.

---

## §1 — O que integrar

| | |
|---|---|
| **Branch** | `line/physics` (worktree `Worktrees/line-physics`) |
| **Base** | `5cc54941` — e `main` **não andou** desde então (`git rev-parse main` == a base) |
| **Commits** | **32**, todos locais, árvore limpa |
| **Merge esperado** | `--ff-only` deve funcionar. Se não funcionar, `main` andou: **PARE e reporte** |

⚠️ **Se `main` tiver andado**, o risco NÃO é textual — é o `PROJECT_SCHEMA` e o registro de componentes
(§4). Confira-os **antes** de resolver qualquer conflito.

## §2 — O que a linha entrega

**Um sprite vira corpo pelo Inspector e cai; Play/Pause/Reset dirigem a sim; a régua para trás re-simula
bit-exato.** Sobre esse alicerce (que integrou em 2026-07-18), esta jornada acrescentou:

| Wave | Entrega | Smoke |
|---|---|---|
| W6 | a escala do `Transform` alcança o collider | `=9` |
| W7 | sensores / triggers | `=10` |
| Weld | o 5º joint (`FixedJoint`) | `=11` |
| BakeChannels | assar um subconjunto dos canais | — |
| W8 | gravity scale por corpo | `=12` |
| Capsule | o collider de personagem | `=13` |
| W9 | velocidade inicial por corpo | `=14` |
| W-CCD | detecção contínua | `=15` |
| W-LockRot | freeze rotation | `=16` |
| W-Offset | offset do collider | `=17` |
| W-LockPos | freeze position X/Y | `=18` |
| W-Mass | massa manual | `=19` |
| W-Dominance | prioridade de colisão | `=20` |
| W-Material | regras de combine (bounce/friction) | `=21` |
| W-Damping | drag por corpo + modo Combine/Replace | `=22` |
| W-OneWay | plataforma jump-through | `=23` |
| W-Area | campo de força (área que empurra) | `=24` |
| W-Contacts | quem toca quem, onde, sob que carga | `=25` |
| W-AreaDrag | a área resiste (vento vs água) | `=26` |
| W-Buoyancy | Arquimedes + a linha d'água no overlay | `=27` |
| W-FormDrag | o arrasto que sabe para onde o corpo aponta | `=28` |

**Todos os smokes foram aprovados pelo Enio.** Detalhe de cada wave (o *porquê*, as medições, as armadilhas):
[`HANDOFF_line_physics.md`](HANDOFF_line_physics.md), uma seção por wave.

## §3 — ⚠️ Superfície de colisão (mesmo-símbolo, DIRETRIZ §1.5.5)

**Foundational tocado** — é aqui que outra linha pode ter mexido:

| Arquivo | O que a linha fez |
|---|---|
| `ph2d-editor-core/src/ids/inspector.rs` | **+40 ids** `INSP_PHYS_*` / `INSP_LIVE_PHYSICS_*` / `INSP_JOINT_*` (append) |
| `ph2d-editor-core/src/screens/hero/inspector_model_physics.rs` | **arquivo NOVO** — `InspectorPhysicsInfo` + `PhysicsFieldEdit` |
| `ph2d-editor-core/src/screens/hero/inspector_model.rs` | re-export do módulo novo |
| `ph2d-editor-core/src/screens/hero.rs` | idem |

⚠️ **Os ids são hashes de string** — dois ids de linhas diferentes colidem em silêncio se as strings
coincidirem. O gate `node_id_collisions` pega isso; **rode-o depois do merge**, não confie no texto.

**Shell tocado** (27 arquivos, mas o único de risco real é `render_loop/mod.rs`: a linha adicionou 3 `mod` e
3 argumentos ao `physics_overlay::draw`). O resto é arquivo novo ou `physics_*`/`inspector_physics*`, que
nenhuma outra linha tem motivo para tocar.

**Crates de física** (`ph2d-physics`, `ph2d-physics-ecs`) são **exclusivas desta linha** — sem risco.

## §4 — ⚠️ Números que se CONTAM, não se escolhem

Se outra linha integrou na mesma janela, **estes três não se resolvem pelo diff — recontam-se**:

| | Valor desta linha | Regra |
|---|---|---|
| `PROJECT_SCHEMA` | **29** | Conte os bumps das DUAS linhas a partir da base comum. Esta linha bumpou de 21 → 29 (8 bumps: is_sensor, offset, mass, dominance, layers, settings, drag do mundo, …). **Componentes NOVOS não bumpam** — os 8 desta jornada (`GravityScale`…`AreaFormDrag`) são keyed por hash de type-name e são puramente aditivos |
| Registro de componentes | **18** | `registers_every_physics_component` — some os dois lados |
| `physics-ecs-c9 body_count` | **75** | some os corpos que cada linha acrescentou ao harness |

⚠️ O hash c9 (`7d55a4ab…`) **não é pinado em literal** — o CI compara os três OSes entre si. Não tente
"corrigir" o hash num merge; corrija o `body_count` e deixe o CI falar.

## §5 — O gate que o integrador roda

```fish
cd <arvore-integrada>
env RUSTUP_TOOLCHAIN=1.95 cargo fmt --all -- --check
env RUSTUP_TOOLCHAIN=1.95 cargo check --workspace --all-targets
env RUSTUP_TOOLCHAIN=1.95 cargo clippy -p ph2d-physics -p ph2d-physics-ecs -p ph2d-panel-inspector -p ph2d-editor-core -p ph2d-host-desktop --all-targets --all-features
env RUSTUP_TOOLCHAIN=1.95 cargo test -p ph2d-physics -p ph2d-physics-ecs -p ph2d-panel-inspector -p ph2d-editor-core
env RUSTUP_TOOLCHAIN=1.95 cargo test -p ph2d-host-desktop --bins
env RUSTUP_TOOLCHAIN=1.95 cargo test -p ph2d-host-desktop --tests
env RUSTUP_TOOLCHAIN=1.95 cargo run -q -p ph2d-physics-ecs --bin physics_ecs_c9
```

**Estado desta linha, medido agora:** fmt limpo · clippy **0** · shell **879 testes** · `body_count: 75` ·
`PROJECT_SCHEMA` 29 · registro 18 · machete e typos limpos · todos os LOC caps.

**Os quatro gates estruturais que mais importam depois de um merge** (eles cobrem exatamente o que um merge
textual limpo pode quebrar em silêncio):

- `node_id_collisions` — dois ids de linhas diferentes hasheando igual;
- `architecture_panel_wiring_parity` — controle pintado que ninguém registrou;
- `every_physics_component_is_authorable` — componente que chegou ao motor e não à UI;
- `architecture_workspace_file_loc_cap` + `file_loc_caps` — o merge somando linhas em arquivo no teto.

## §6 — Smokes para re-conferir depois da integração

Os 21 já foram aprovados **nesta árvore**. Depois do merge, os três que atravessam mais camadas:

```fish
cd <arvore-integrada> && env PH2D_PHYSICS_SMOKE=27 RUSTUP_TOOLCHAIN=1.95 cargo run -p ph2d-host-desktop
cd <arvore-integrada> && env PH2D_PHYSICS_SMOKE=7  RUSTUP_TOOLCHAIN=1.95 cargo run -p ph2d-host-desktop
cd <arvore-integrada> && env PH2D_PHYSICS_SMOKE=6  RUSTUP_TOOLCHAIN=1.95 cargo run -p ph2d-host-desktop
```

`=27` toca Inspector + overlay + empuxo; `=7` toca o **bake** (física vira animação, o único ponto onde esta
linha escreve na timeline); `=6` toca os **joints** (pêndulo, corrente, ragdoll).

## §7 — ⚠️ O que esta linha NÃO fez, de propósito

- **Não integrou, não pushou, não tocou em `main`.**
- **Não bumpou `PROJECT_SCHEMA` por conta dos 8 componentes novos** — componente registrado é aditivo, e um
  bump **recusa todo projeto já salvo** no número antigo. Foi decidido quatro vezes pela mesma razão.
- **Não mexeu em contrato congelado** (§6 do CLAUDE.md) — nenhum ADR novo foi necessário além do 0131, que
  já está em `main`.
- **Não tocou `ph2d-ecs`, `ph2d-core`, `ph2d-render`** — o único foundational alcançado é o `editor-core`,
  e só por append de ids + um módulo novo.

## §8 — Aberto (para o próximo dono, não para o integrador)

Por wave, no [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md). Os três de maior valor:

- **eventos de contato início/fim** — precisa de um consumidor de gameplay; a precedência do W7 diz para
  torná-lo visível primeiro;
- **a escala não alcança a espessura do collider** em alguns casos (pré-existente, wave própria);
- **gizmo de âncora de joint no canvas** — hoje a âncora é autorável em números pelos campos Position.

E a política nova que o plano passou a exigir de toda wave futura: **§ "Toda wave chega à UI"** em
[`00_plano_waves.md`](00_plano_waves.md) — as quatro condições (existe · é pintado · o clique chega ao
barramento · **a SEQUÊNCIA leva a algum lugar**), a metade visível, e a cena com números medidos.
