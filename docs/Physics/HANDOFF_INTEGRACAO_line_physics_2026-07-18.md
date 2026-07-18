# Handoff de INTEGRAÇÃO — `line/physics` (DIRETRIZ §1.5.9)

> Para o **agente integrador**. A linha está fechada e **não integra nem faz ship** por conta própria.
> Estado técnico completo do módulo: [`HANDOFF_line_physics.md`](HANDOFF_line_physics.md) ·
> decisão: [ADR-0131](../architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md).

---

## 1. Identidade

| | |
|---|---|
| Branch | `line/physics` |
| HEAD | `adfab599` |
| Base do fork (merge-base com `main`) | `389676f9` |
| Commits | **21** |
| Waves cobertas | W0 · W1 · W1.5 · W2a · **W2b** · **W2c** · **W3** |
| ⚠️ `main` andou desde o fork? | **NÃO** — `merge-base == main HEAD == 389676f9` no momento deste handoff |

**Consequência:** enquanto a `main` não andar, isto é **fast-forward puro** (`--ff-only` passa sem
Mergiraf). Se outra linha integrar antes, veja o §3 — os pontos de colisão estão todos listados com valor
literal.

---

## 2. Foundational / compartilhado tocado, e por quê

A linha é **Modo L**, então foundational é permitido sob o protocolo testado (ADR-0107). Nada aqui é
gratuito — cada item existe porque a física precisava de uma porta que não havia.

### 2.1 Crates NOVAS (glob `crates/*` — **zero edit central**)
- **`ph2d-physics-ecs`** — a ponte ECS↔rapier (components + `PhysicsBridge`). É o coração da linha.

### 2.2 `ph2d-physics` (a crate M10, era dormente) — **aditivo**
`spawn_body`/`set_body_pose`/`remove_body` + `BodyDesc`/`ShapeDesc` · `checkpoint`/`restore` +
`PhysicsCheckpointRing` (módulo filho `world/checkpoint.rs`) · knobs de contato/substeps.
⚠️ **Os helpers antigos e `step` NÃO foram tocados**, mas os **defaults de integração mudaram** (§3.6).

### 2.3 `ph2d-editor-core` — **aditivo, exceto uma catraca de LOC**
| Arquivo | O quê |
|---|---|
| `action_bus.rs` | +2 variants (`Transport`, `InspectorPhysicsEdit`) — enum `#[non_exhaustive]`, **não serializado** |
| `ids/inspector.rs` | +14 ids §11 (slugs hasheados) |
| `screens/hero/inspector_model.rs` | +`InspectorPhysicsInfo` +`PhysicsFieldEdit` |
| `lib.rs`, `screens/mod.rs` | re-exports dos dois acima |
| `screens/hero/chrome/transport.rs` | **NOVO** handler (`z=300`) |
| `screens/hero/chrome/mod.rs` | **GERADO** por `ph2d-chrome-sync` (§3.3) |
| `screens/hero/topbar/mod.rs` | tooltips Play/Pause/Reset (eram mentira) |
| `tests/node_id_collisions.rs` | +14 linhas na tabela à mão |
| `tests/architecture_panel_loc_cap.rs` | ⚠️ **allowance `paint_inspector` 431 → 424** (§3.5) |

### 2.4 `ph2d-panel-inspector` — seção §11 + **um split forçado pelo teto de LOC**
Novos: `sections/physics.rs`, `event_physics.rs`, `paint_frame.rs`, `tests/seam_physics.rs`.
⚠️ **`paint.rs` foi REESTRUTURADO** (não só acrescido): o corpo do macro `live_section!` e a fase B
saíram para `paint_frame.rs` porque `paint_inspector` estava congelado em 431 LOC e uma seção custa ~18.
**É o arquivo com maior risco de conflito textual** se outra linha mexeu no Inspector.

### 2.4b W3 — o que a wave de joints acrescentou (tudo **aditivo**)
| Onde | O quê |
|---|---|
| `ph2d-ecs` | **`stable_name_id`** em `name.rs` + re-export. ⚠️ `shells/desktop/src/timeline_persist.rs::wire_id_for_name` passou a **DELEGAR** — mesma FNV-1a byte a byte (pinada contra valores externos), mas é **edição num arquivo da timeline**: num conflito, mantenha a delegação |
| `ph2d-physics` | módulo `world/joints.rs` + re-export de `ImpulseJointHandle` |
| `ph2d-physics-ecs` | `src/joint.rs` (component `PhysicsJoint` + enum `JointKind`), `src/bridge/joints.rs`, dev-dep `postcard`. ⚠️ **a contagem do registry foi 2 → 3** (o teste `registers_every_physics_component`) |
| `ph2d-editor-core` | `InspectorJointInfo`/`JointFieldEdit`; `InspectorPhysicsInfo.can_join`; `PhysicsFieldEdit::Join`; **23 ids §12** na tabela do `node_id_collisions`. ⚠️ `any_live_section` `[bool; 8] → [bool; 9]` e os slots de nota `10 → 11` — **os dois são rígidos DE PROPÓSITO** |
| `ph2d-panel-inspector` | `sections/joint.rs`, `sections/rows.rs` (helpers **movidos** de `physics.rs`), `event_joint.rs`, `tests/seam_joint.rs`. ⚠️ `paint.rs` perdeu a família de física para `paint_frame::paint_physics_sections` |
| `shells/desktop` | `render_loop/inspector_joint.rs` (+tests), `render_loop/physics_overlay_joints.rs`, `tests/join_is_one_gesture_not_a_fan_out.rs`, cena de smoke **6** |

### 2.5 `ph2d-vector` — **1 linha, aditiva**
`PathEl` acrescentado à lista de re-export do kurbo (`src/lib.rs:58`). **Não é a superfície congelada** —
o gate `architecture_vector_contract_surface` escaneia só `-doc` e `-traits` (verificado, §4).

### 2.6 `shells/desktop`
Novos: `transport.rs`, `physics_smoke.rs`, `render_loop/{physics_bridge,physics_overlay,inspector_physics,inspector_physics_tests}.rs`.
Modificados: `main.rs`/`app_state.rs` (2 campos), `init.rs` (registro + gate do smoke), `input_handlers.rs`
(**tecla `B`**, §3.4), `render_loop/mod.rs` (mod decls + 1 chamada de dispatch + 1 braço de dreno),
`snapshots.rs` (publica o snapshot §11), `inspector_{commits,ordering}.rs`, `project.rs`/`project_tests.rs`
(**schema**, §3.1), `Cargo.toml` (dep na ponte).

### 2.7 `.github/workflows/spike.yml`
+1 job de matriz `physics-ecs-c9` + artifact + 3ª comparação no `determinism-compare` (espelho do
`ph2d_physics_c9` que já existia).

---

## 3. Símbolos que podem COLIDIR (grep isto — §1.5.5)

### 3.1 ⚠️ `PROJECT_SCHEMA = 21` — **o valor se CONTA, não se escolhe**
`shells/desktop/src/project.rs:86` e a tripla-pin `(21, 8, 8)` em `project_tests.rs`.

**Esta linha bumpou CINCO vezes**: 16 (componentes de física no `WorldSnapshot`) · 17
(`restitution`/`friction` apendados ao `Collider`) · 19 (as settings de mundo do W2b viajam no arquivo;
o 18 veio da re-contagem que somou o bump da `line/FLIP` na integração anterior) · 20 (`air_drag`
apendado pós-smoke) · 21 (camada + matriz do W2c).

⚠️ **O W3 NÃO bumpou, de propósito, e isso também é a contagem falando.** Um componente NOVO não move
layout nenhum: o blob é chaveado por `stable_type_id = blake3(nome_canônico)[..8]`, derivado do **NOME**,
então registrar `ph2d::physics::PhysicsJoint` cunha um id novo e todo blob já em disco fica onde estava.
É o oposto do W2c, que apendou um campo **DENTRO** do `Collider`, onde postcard é posicional. E bumpar à
toa não é neutro: schema divergente **recusa o arquivo inteiro**, jogando fora todo projeto já salvo. O
raciocínio está falsificável em `crates/ph2d-physics-ecs/tests/joint_persistence.rs` — se algo mover o
layout, o 1º gate fica vermelho e o bump passa a ser devido. **Se outra linha também
bumpou, o valor certo não está em nenhum dos dois lados: some os bumps.** Escolher um lado faz os saves
da outra passarem na checagem de versão e serem lidos com o layout errado — e postcard não tem nome de
campo para reclamar, ele devolve lixo bem-formado. (É o cenário que a doc do próprio gate descreve, e que
já aconteceu com QUATRO linhas em 2026-07-13.)

### 3.2 `EditorAction` — 3 variants apendados
`Transport(TransportCmd)` · `InspectorPhysicsEdit { entity_bits, edit }` ·
**`InspectorJointEdit { entity_bits, edit }`** (W3).
Enum `#[non_exhaustive]` e **não serializado** (é barramento de frame) ⇒ **a ordem é livre**; num conflito,
**mantenha os dois lados**. Mesma regra para `TransportCmd` (enum novo no mesmo arquivo).

### 3.3 ⚠️ Chrome `z=300` — o dispatch é **GERADO**
`chrome/transport.rs` traz o marcador `// ph2d-chrome-sync:z=300`. Os z ocupados hoje: …240, 270, 271,
280, 290, **300**. Se outra linha também tomou 300, **renumere uma delas** e **re-rode
`cargo run -p ph2d-chrome-sync`** — o `chrome/mod.rs` é saída de codegen, nunca resolva o conflito à mão.

### 3.4 ⚠️ **Tecla `B` global** (`input_handlers.rs:132`) — toggle do contorno de collider
Espaço de teclas é compartilhado e não tem gate de colisão. `B` estava livre desde que o W4.T5 da timeline
aposentou a demo de `SpriteAnimation`. **Se outra linha também tomou `B`, é conflito de verdade** (as duas
compilam e a última ganha em silêncio) — escale ao Enio para escolher a tecla.

### 3.5 ⚠️ Allowance de LOC `paint_inspector` **431 → 424** — também se **RE-MEDE**
`tests/architecture_panel_loc_cap.rs:114`. As allowances **só encolhem** (é catraca). Se outra linha também
mexeu em `paint_inspector`, **o número certo é o medido DEPOIS do merge**, não o de nenhum dos lados —
rode o gate e use o valor que ele reportar. Mesma classe do §3.1.

### 3.6 ⚠️ Os dois hashes C9 **MUDARAM** (esperado, não é regressão)
`physics-c9` → `2f7e2d586d395dd7…` · `physics-ecs-c9` → `54fea29671c866b5…`. Causa: os defaults de
integração (`DEFAULT_SUBSTEPS = 4`, `DEFAULT_CONTACT_HZ = 120`) entram no solver.
**Nenhum é pinado em literal** — o CI compara os 3 OSes entre si (`sort -u | wc -l`), então o gate segue
válido. Não "conserte" o hash.

### 3.7 Listas ordenadas / contagens compartilhadas
- `tests/node_id_collisions.rs` — +14 linhas, **append-only**, num conflito mantenha os dois lados.
- `paint.rs:217` — `notes_per_section: [_; 10]` (era 9). **Se outra linha também adicionou seção, re-conte.**
- `ph2d-vector/src/lib.rs:58` — `PathEl` na lista de re-export: mantenha os dois lados.

### 3.8 Ids novos (slugs hasheados — únicos por construção, mas a TABELA é compartilhada)
`insp_live_physics_{section,color}` · `insp_phys_{add,remove,radius,half_x,half_y,density,restitution,friction}` ·
`insp_phys_kind_{dynamic,static}` · `insp_phys_shape_{ball,box}`.
Sem `NodeId(N)` numérico novo. **`PHYSICS_SCROLLBAR_ID = NodeId(836)` foi RESERVADO no plano mas NÃO
usado** (o painel global não foi construído) — está livre.

---

## 4. Contratos congelados encostados: **NENHUM**

Verificado por grep (nenhum arquivo de `ph2d-nodegraph`, `ph2d-vector-doc`, `ph2d-vector-traits`,
`tool-registry` ou dos 3 `architecture_*_contract_surface` foi tocado) **e** pelos gates rodando verdes
nesta árvore:

```
architecture_contract_surface .......... 3 passed
architecture_tool_contract_surface ..... 4 passed
architecture_vector_contract_surface ... 11 passed
```

Nenhum ADR adicional é exigido além do **0131** (escrito por esta linha como 0130 e renumerado na integração — a `line/gpu-nodes` reclamou o 0130 no mesmo dia).

---

## 5. O que só o `ship.sh` pega — **já rodei todos nesta árvore**

O gate de integração não roda estes; para poupar iterações do integrador
([[project_integration_prefork_lines_ship_drift]]), rodei aqui e o estado é:

| Gate | Resultado nesta árvore |
|---|---|
| `cargo fmt --all --check` | **limpo** |
| `typos` | **limpo** |
| `cargo machete` | **limpo** (nenhuma dep não-usada) |
| `cargo deny check` | **advisories ok · bans ok · licenses ok · sources ok** |
| `cargo audit` | 3 warnings **allowlistados e pré-existentes** (memmap2 etc.); **nada novo desta linha** |
| `cargo clippy --workspace --all-targets` | **limpo** |
| `nextest-impacted` | **3640 passed, 43 skipped** |

**Deps novas** (para o machete/deny do integrador conferirem pós-merge): `ph2d-physics-ecs` puxa
`bevy_ecs 0.18`, `ph2d-core`, `ph2d-ecs`, `ph2d-physics`, `serde` — todas já no workspace. **A única dep
externa nova em toda a linha é `dhat 0.3` como `[dev-dependencies]` de `ph2d-physics`** (o gate de
memória do ring; mesma dep que a `ph2d-audio-edit` já usa).

⚠️ **`rapier2d` NÃO ganhou features novas.** `parallel`/`simd-*` seguem OFF — ligá-las quebra HR-5.

---

## 6. Ordem, dependências e o que smoke-testar

### 6.1 Ordem
Os 15 commits são **estritamente sequenciais** e não há como reordená-los: W1 cria a ponte, W1.5 depende
dela, o overlay depende do W1.5 (a cena de smoke), o W2 depende dos componentes, e o fix de penetração
depende de tudo. **Integre a linha inteira ou nenhuma parte** — não há corte parcial coerente.

### 6.2 Depois do merge, ANTES de declarar verde
1. `cargo run -p ph2d-chrome-sync` — o `chrome/mod.rs` é gerado (§3.3).
2. `cargo check --workspace --all-targets` — **merge textual limpo pode estar semanticamente quebrado**
   ([[feedback_clean_text_merge_can_be_semantically_broken]]); só a checagem cruzada pega.
3. Se `main` andou: re-medir §3.1 (schema), §3.5 (allowance de LOC) e §3.7 (contagens) — **os três se
   contam, não se escolhem.**

### 6.3 Smoke

Todo comando abaixo é para rodar **de dentro do worktree**, e com `env` porque o shell do Enio é
**fish** (`VAR=valor comando` não funciona lá — a env var seria ignorada em silêncio e a cena nunca
montaria):

```
env PH2D_PHYSICS_SMOKE=<n> cargo run -p ph2d-host-desktop
```

| Cena | O quê | Estado |
|---|---|---|
| `=1` | W1: um corpo cai e assenta | ✅ aprovado |
| `=2` | W1.5: pilha + scrub da régua pra trás | ✅ aprovado |
| (mesma cena) | contorno de collider + tecla `B` | ✅ aprovado |
| `=3` | W2a: Add Physics Body no Inspector | ✅ aprovado |
| (mesma cena) | interpenetração no pouso | ✅ aprovado |
| `=4` | W2b: o painel de mundo (tecla `W`) | ✅ aprovado |
| `=5` | W2c: a matriz de camadas | ✅ aprovado |
| **`=6`** | **W3: pêndulo · corrente · ragdoll** | ⏳ **PENDENTE** |

⚠️ **A cena 6 é a única pendente**, e é a única coisa desta linha que ainda não passou pelos olhos do
Enio. O que ela tem de mostrar está no §W3 do tracker; em resumo: o pêndulo pendura pela **PONTA** da
prancha (não pelo meio), os elos da corrente **sobrepõem** nos pinos sem brigar, e os joelhos do ragdoll
dobram **para um lado só**. Tecla `B` liga o overlay — os joints são os traços **âmbar**.

### 6.4 O que NÃO foi construído (para o integrador não procurar)
- **O painel global de física** (`ph2d-panel-physics`, gravidade/substeps/camadas) — é a **outra metade
  do W2**, deliberadamente adiada: os defaults já são bons, enquanto sem o Inspector a física era
  inalcançável. Nenhum dos 5 sites de registro de painel foi tocado, e a lista de z-order do
  `hero/paint.rs` está **intacta**.
- W3 (joints) e W4 (bake) não começaram.

---

## 7. Resumo para o Enio

> Linha `physics` pronta (HEAD `89be6146`, 15 commits, base `cdc3acc1` = `main` atual ⇒ fast-forward).
> Foundational tocado: `editor-core` (aditivo + 1 catraca de LOC), `panel-inspector` (**`paint.rs`
> reestruturado**), `ph2d-vector` (1 re-export), `shells/desktop`, `spike.yml`. Contratos congelados:
> **nenhum** (3 gates verdes). Colisões a grepar: **`PROJECT_SCHEMA=17`** e a **allowance 424** (os dois se
> CONTAM), **chrome z=300** (re-rodar o sync), **tecla `B`**, `notes_per_section[;10]`, +2 variants de
> `EditorAction`. Os hashes C9 mudaram **de propósito** e não são pinados. `ship.sh` inteiro já roda verde
> aqui (3640 testes). Smoke: **tudo aprovado, nada pendente**. Aguardo ordem de integração.
