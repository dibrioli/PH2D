# Handoff de INTEGRAÇÃO — `line/physics` **W2b + W2c** (DIRETRIZ §1.5.9)

> Para o **agente integrador**, por ordem explícita do Enio. A linha fechou e PAROU:
> não integrei, não pushei.
>
> Estado técnico completo: [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) §W2b e §W2c.
> O *porquê*: [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md) D8.

---

## 1. Identidade

- Branch **`line/physics`**, worktree `Worktrees/line-physics`, árvore limpa.
- Base (merge-base com `main`): **`389676f9`**.
- Commits desta jornada (`git log --oneline 389676f9..HEAD`): **15** (este doc inclusive)
  - `4062529b` docs(physics): baseline pos-integracao -- o que a integracao MUDOU e o terreno re-verificado
  - `2bf00ba4` docs(physics): handoff de CONTINUACAO -- W2b em janela nova
  - `f53fb928` docs(physics): o roteador passa a saber que a fisica existe (TAREFA ZERO)
  - `156d8650` feat(physics): o mundo ganha os defaults de corpo (damping + sleep) -- W2b, a metade do motor
  - `94bb8e62` feat(physics): PhysicsSettings -- a truth-at-rest para o MUNDO, com todo teto MEDIDO
  - `7e51db25` feat(physics): o painel global de MUNDO -- W2b, a autoria (crate ph2d-panel-physics)
  - `6e50419b` feat(physics): as settings de mundo VIAJAM no arquivo (PROJECT_SCHEMA 18->19) + cena de smoke 4
  - `75df6697` docs(physics): W2b FECHADO -- tracker, plano, roteador e handoff de integracao
  - `f939255d` style: cargo fmt --all (alfabetizacao do mod/pub use novo + reflow das listas)
  - `1eeb6f16` docs(physics): a contagem de gates estava errada (22/21 -> 24/22) e a identidade do handoff
  - `a4067033` fix(physics): o painel estava INALCANCAVEL e o "Air Drag" nao era ar -- os 2 achados do smoke
  - `b33ba632` docs(physics): os 2 achados do smoke no tracker, no roteador e no handoff + 2 memorias
  - `86587171` feat(physics): camadas de colisao -- o motor (W2c, a metade que nao e UI)
  - `09c9a8bc` feat(physics): a UI das camadas -- a matriz no painel + a camada no Inspector (W2c fecha)
- A `main` **não** andou desde o fork (`git log HEAD..main` vazio no fechamento).

## 2. Foundational / compartilhado tocado

| Arquivo | O quê |
|---|---|
| `crates/ph2d-physics/` | **meu módulo**, aditivo: `world/drag.rs` (**NOVO**, o modelo de arrasto real) + `BodyDefaults` + `world/defaults.rs` + `set_body_defaults`/`stamp_defaults`. ⚠️ os 4 caminhos de spawn passam a carimbar os defaults — **byte-idêntico** nos defaults (gate), então o hash C9 não se move. |
| `crates/ph2d-physics-ecs/` | **meu módulo**, aditivo: `settings.rs`. O campo `PhysicsBridge.gravity` virou `settings` (privado). |
| `crates/ph2d-editor-core/` | **ids/chrome/physics.rs (NOVO)** + `mod`/`pub use` · `screens/hero/paint.rs` (1 linha na lista de z-order) · `widget/scrollbar.rs` (+id 836 e a linha da auto-checagem) · `widget/mod.rs` (re-export) · `interaction/dispatch/scroll.rs` (1 braço) · `tests/node_id_collisions.rs` (29 linhas) |
| `crates/ph2d-i18n/` | 21 chaves `panel.physics.*` (bloco novo, antes do `panel.vector.title`) |
| `crates/ph2d-panel-registry-init/` | `src/lib.rs` (bloco **GERADO** + `EXPECTED_TYPED` à mão) · `Cargo.toml` (2 blocos gerados + a lista `default` à mão) |
| `crates/ph2d-panel-physics/` | **crate NOVA** (glob `crates/*` — zero edit central) |
| `shells/desktop/` | `Cargo.toml` (+dep, **+feature `panel-physics` e a entrada no `default`**) · `render_loop/mod.rs` (+`mod` +1 call) · `render_loop/physics_panel_bridge.rs` (**NOVO**) · `forwarding.rs` (import + `|| inside(PHYSICS_PANEL)`) · `input_handlers.rs` (tecla `W`) · `project.rs` (schema + campo + save + load) · `project_tests.rs` (tripla-pin + 2 gates + 2 initializers) · `physics_smoke.rs` (cena 4) |
| `CLAUDE.md` | §5 (entrada de física + o W2b) e §1 (linha nova no roteador) |

**`ph2d-ecs` NÃO foi tocado.** Contratos congelados (§6): **NENHUM**.

## 3. Símbolos que podem COLIDIR — grepe

1. ⚠️ **`PROJECT_SCHEMA = 21` + tripla-pin `(21, 8, 8)`.** Se outra linha desta janela também
   bumpar o schema, **o valor se CONTA, não se escolhe**
   ([[feedback_numbers_that_sum_across_lines_count_dont_pick]]): some os deltas e atualize a
   tripla. O gate `a_schema_bump_anywhere_must_bump_the_project_schema` fica vermelho até
   baterem. **E lembre do ponto cego dele:** ele pina CONSTANTES, então uma mudança de UNIDADE
   num campo de layout igual passa VERDE.
2. ⚠️ **`EXPECTED_TYPED = 19`** em `ph2d-panel-registry-init`. Outra linha que adicione painel
   **também** soma 1 — mesma aritmética de contagem. O `panel-sync` **não** regenera essa const
   nem a lista `default`.
3. ⚠️ **`PHYSICS_SCROLLBAR_ID = NodeId(836)`** — inteiro à mão, fora do `node_id_collisions`.
   Se outra linha tiver pegado 836, re-atribua (o próximo livre passa a ser 838) e conserte a
   nota "Next free id" **e** a tabela de auto-checagem em `widget/scrollbar.rs`.
4. **Blocos GERADOS — nunca resolva conflito na mão:** `ph2d-panel-registry-init/src/lib.rs` e
   `Cargo.toml` (marcadores `<ph2d-panel-sync:*>`) ⇒ **re-rode `cargo run -p ph2d-panel-sync`**
   e depois reponha as duas edições manuais (`EXPECTED_TYPED`, lista `default`).
5. **Listas append-only que o Mergiraf funde mas confira:** a lista de fallback de z-order em
   `hero/paint.rs` · `mod physics_panel_bridge;` · o `use` de `forwarding.rs` · o `match` de
   teclas em `input_handlers.rs` (a `W`) · `chrome/mod.rs` (`mod physics;`/`pub use physics::*;`).
6. **Tecla `W`** — auditada livre contra o conjunto inteiro do shell no fechamento. Se outra
   linha tiver reivindicado `W`, as livres restantes eram **H · J · Q · R**.
7. Nomes improváveis de colidir: `PhysicsPanel`, `PhysicsSettings`, `BodyDefaults`,
   `PhysicsIntent`, `physics_panel_bridge`, `panel-physics`, `panel.physics.*`.

## 4. O que só o `ship.sh` / CI pega

- `typos` (os comentários novos são longos e em pt-BR) · `machete` (deps novas:
  `ph2d-panel-physics` no shell e `ph2d-physics-ecs`/`ph2d-i18n`/`ph2d-tokens`/`ph2d-a11y` na
  crate nova — todas USADAS; `ph2d-ui-testkit` é dev-dep) · `deny`/`audit` (**zero crate externa
  nova**).
- A **matriz cross-OS dos dois hashes C9**. ⚠️ **Eles não devem se mover:** os defaults novos
  são os do rapier e há gate provando que aplicá-los não muda um bit. Se um hash mudar na CI, é
  achado, não ruído.

## 5. O que smoke-testar (Enio)

```
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-physics && cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-physics && env PH2D_PHYSICS_SMOKE=4 cargo run -p ph2d-host-desktop
```

O painel abre sozinho (a tecla **`W`** o alterna). 12 corpos de 3 tamanhos caem numa pilha. A
ordem que o `eprintln` da cena repete:

1. **Gravity Y → 0** — tudo para de cair, no ar.
2. **Gravity X** — a pilha escorrega de lado.
3. **Air Drag / Density** — os corpos **GRANDES** caem mais rápido (o arrasto escala com a
   secção transversal e é resistido pela massa).
   E logo abaixo: **Damping / Linear** — tudo desacelera **igual**. São dois modelos, e é a
   seção que os separa; rotular o uniforme de "Air Drag" foi o que reprovou o 1º smoke.
4. **Sub-steps** — menos afundamento no impacto (olhe um corpo aterrissar).
5. **Sleep / Delay → 0** — a pilha assentada congela mais cedo.
6. **Show Colliders** — tem de concordar com a tecla **`B`**, sempre e nos dois sentidos.
7. **Reset to Defaults** — tudo volta, num clique.
8. **Ctrl+S, Ctrl+O** — as settings voltam com o projeto.

**E o W2c, cena `PH2D_PHYSICS_SMOKE=5`** (dois grupos, duas camadas, um chão):

1. **Célula (1,0)** — o grupo da DIREITA cai pelo chão, o da esquerda fica, **e o
   da direita continua empilhando em si mesmo**. Esse último detalhe é o que
   separa *"camadas funcionam"* de *"colisão quebrou"*.
2. **Clique de novo** — eles pousam. Tem de funcionar sobre os corpos que JÁ
   estão na cena, não só sobre novos.
3. **Célula (1,1)** — o grupo da direita para de colidir consigo mesmo.
4. **Inspector → Layer** — mova UM corpo de camada e veja-o mudar de grupo.
5. **Ctrl+S, Ctrl+O** — a matriz volta com o projeto.

E confirme que **o app normal (sem a env) segue igual**: o painel nasce fechado
(`DEFAULT_VISIBLE = false`) e a ponte é no-op sem corpos.

⚠️ **O smoke que mais importa é o 3**: é onde o 1º smoke reprovou. Air Drag tem de separar
grande de pequeno; Damping tem de NÃO separar. Se os dois se comportarem igual, os modelos foram
fundidos em algum ponto e os rótulos voltaram a mentir.

⚠️ **E confira que a tecla `W` abre o painel** — no 1º smoke ela não abria porque o painel não
estava no build. O gate `every_panel_the_shell_drives_is_in_its_registry` agora guarda isso, mas
é uma tecla: veja com o olho.

## 6. Resumo

*Linha `physics` W2b pronta — 6 commits sobre `389676f9`, árvore limpa. Foundational tocado:
`ph2d-physics` + `ph2d-physics-ecs` (meus módulos, aditivos, C9 intacto) · `ph2d-editor-core`
(ids/z-order/scrollbar/dispatch) · `ph2d-i18n` · `ph2d-panel-registry-init` (blocos GERADOS) ·
shell (consumidor). Crate nova `ph2d-panel-physics`. Contratos congelados: nenhum. Colisões a
grepar: `PROJECT_SCHEMA=21`+tripla-pin · `EXPECTED_TYPED=19` · `NodeId(836)` · tecla `W` — os
três primeiros **se CONTAM** se outra linha também bumpar. 41 gates novos, 38 mutações, 37 sangram
(1 sobrevive POR PROJETO — o early-out de `k<=0` no drag: o contrato é honrado duas
vezes, pelo ramo e pela aritmética). Gate batched verde: fmt · clippy `--all-targets` · `check --workspace --all-targets` ·
`nextest-impacted` (**4424 testes, 0 falhas**). Smoke pendente: `PH2D_PHYSICS_SMOKE=5` (o `=4` do W2b já foi APROVADO). Aguardo
ordem de integração / W3.*
