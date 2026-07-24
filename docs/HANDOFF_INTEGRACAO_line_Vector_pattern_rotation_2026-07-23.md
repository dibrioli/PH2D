# Handoff de integração — `line/Vector`: a ROTAÇÃO do motivo no Pattern on Path

> DIRETRIZ §1.5.9. A linha **não integra e não faz ship** — entrega isto e espera ordem do Enio.
> Supersede nada: é a wave seguinte ao `HANDOFF_INTEGRACAO_line_Vector_2026-07-23.md`, que já integrou.

## 1 — Identidade

| | |
|---|---|
| Branch | `line/Vector` |
| HEAD (código) | `4a0d8bb8a8393a8f97131130e04502199864fcc6` — o commit **deste handoff** fica por cima; o tip da branch é `git rev-parse line/Vector` |
| Base (merge-base com `main`) | `df91ef6ec` — a linha foi **reaberta** e rebaseada; os 27 commits da jornada anterior já estavam no main e o rebase os colapsou |
| Commits | **6** (5 de código + este handoff) |

Pedido do Enio: *"Em pattern on Path, criar um parâmetro para rotação da shape na curva"*.

## 2 — Foundational / compartilhado tocado, e por quê

| Arquivo | O quê | Aditivo? |
|---|---|---|
| `crates/ph2d-editor-core/src/ids/chrome/vector_patternpath.rs` | +2 ids no FIM do bloco append-only | sim |
| `crates/ph2d-ecs/src/vec_pattern_rotation.rs` | **arquivo NOVO** (módulo irmão — isolamento por §1.5.2.1) | sim |
| `crates/ph2d-ecs/src/lib.rs` | `mod` + `pub use` ao lado dos irmãos `vec_pattern_path` | sim |
| `crates/ph2d-ecs/src/scene/registry.rs` | registro do componente + contador **35 → 36** | contador! |
| `crates/ph2d-render/src/registry.rs` | contador espelho **36 → 37** | contador! |
| `crates/ph2d-script/src/registry.rs` | contador espelho **36 → 37** | contador! |
| `shells/desktop/src/render_loop/mod.rs` | 4 sítios (pending var · classify · drain · publish) | sim |
| `shells/desktop/src/pattern_live.rs` | `rotation_of` / `set_rotation` / `current_rotation`; `spec_to_motor` +1 param; `detach` remove os 2 componentes | não-aditivo em `spec_to_motor`/`detach` (privados do módulo) |
| `crates/ph2d-vec-scene/src/pattern_path.rs` | `PatternSpec.rotation_deg` + `Rotor` + `motif_bbox` mede no referencial girado | campo apendado |
| `crates/ph2d-panel-vector/src/*` | seam (ids, lib, populate, paint, event, state, state_patternpath) | sim |

## 3 — Símbolos que podem COLIDIR com outra linha

**O item de risco real são os TRÊS contadores de componente.** Qualquer linha que registre um
componente ECS mexe nos mesmos números, e eles **se CONTAM, não se escolhem**
([[feedback_numbers_that_sum_across_lines_count_dont_pick]]). Se outra linha desta janela também
registrou componente, o valor certo **não está em nenhum dos dois lados do conflito**:

| Arquivo | main hoje | esta linha | regra |
|---|---|---|---|
| `ph2d-ecs/src/scene/registry.rs` | 35 | **36** | conte os `reg.register::<…>` da árvore combinada |
| `ph2d-render/src/registry.rs` | 36 | **37** | = ecs + 1 (`Sprite`) |
| `ph2d-script/src/registry.rs` | 36 | **37** | = ecs + 1 (`LuauScript`) |

⚠️ Os dois espelhos **só correm na suíte da própria crate** — é a família que já ficou
vermelho-latente na integração de 21/07. Rode `cargo test -p ph2d-render registry` **e**
`cargo test -p ph2d-script registry` explicitamente na árvore combinada.

Símbolos novos (nome único, colisão improvável mas grepável):

- `ph2d_ecs::VecPatternRotation` — nome canônico de registro `"ph2d::ecs::VecPatternRotation"`
- `VECTOR_PATTERNPATH_ROTATION` = `hash_node_id("vector.patternpath.rotation")`
- `VECTOR_PATTERNPATH_ROTATION_NUM` = `hash_node_id("vector.patternpath.rotation.num")`
- `ph2d_panel_vector::ROTATION_MAX = 180.0` · `rotation_from_track` · `rotation_to_track` (`pub(crate)`)
- `PatternSpec.rotation_deg: f64` (campo apendado, struct pública de `ph2d-vec-scene`)

**Mudança de assinatura pública** (um chamador só, na shell):
`ph2d_panel_vector::set_current_patternpath` passou de **6 para 7 argumentos** (o 7º é `rotation: f64`).

## 4 — Contratos congelados encostados

**NENHUM.** Conferido por gate, não por auto-relato — os três verdes na árvore desta linha:
`architecture_contract_surface` (nós) · `architecture_tool_contract_surface` (tools) ·
`architecture_vector_contract_surface` (o data-model foundational do vetor, que escaneia só
`ph2d-vector-doc`+`-traits`). `ph2d-vec-scene` é o motor novo, cujo contrato não é congelado.

## 5 — Schemas: NADA bumpou, e isso foi a decisão de projeto da wave

`PROJECT_SCHEMA` fica em **29**. `VEC_SCENE_SCHEMA_VERSION` fica em **13**. `DOC_VERSION` intocado.

A rotação nasceu em **componente próprio** em vez de campo no `VecPatternPath` exatamente para
isto: o blob de um componente é postcard **POSICIONAL**, então apender campo bumparia o schema, e
um bump **RECUSA todo projeto já salvo**. É o critério que a `line/physics` fixou depois de o pagar
(W-Offset apendou ao `Collider` e bumpou 28→29; as três waves seguintes da mesma área —
`AreaDrag`, `AreaBuoyancy`, `AreaTorque` — reverteram e cada uma nasceu componente próprio).

Consequência de graça: **ausência do componente é "sem rotação"**, então todo projeto salvo antes
desta wave carrega inalterado, sem campo em falta a inventar no load.

## 6 — O que só o `ship.sh` pega (rodei tudo aqui, mas na árvore COMBINADA pode reaparecer)

Na minha árvore, com o pin 1.95, **todos exit 0**:

| gate | resultado |
|---|---|
| `cargo fmt --check --all` (pin 1.95) | 0 |
| `typos` | 0 |
| `cargo machete` | 0 |
| `cargo clippy --all-targets -- -D warnings` (7 crates) | 0 |
| suíte do **shell inteira** (`-p ph2d-host-desktop`, tests + bins) | 0 FAILED |
| crates tocadas (vec-scene, ecs, panel-vector, render, script, editor-core) | 0 FAILED |
| `file_loc_caps` · `architecture_workspace_file_loc_cap` · `architecture_panel_loc_cap` | ok |
| `architecture_panel_wiring_parity` · `node_id_collisions` · `arch_safe_clamp_only` · `no_tofu_glyphs` | ok |

⚠️ **Duas notas de fmt/typos que valem para o integrador:**

1. `typos` reprova a palavra pt-BR para *attitude* (sem o `t` dobrado) como near-match do
   inglês — inclusive **dentro desta frase**, se ela a escrever. Reescrevi tudo para
   **`orientação`** nos MEUS arquivos em vez de tocar o `.typos.toml` — lista compartilhada funde
   contra a main de hoje, e reescrever o meu próprio texto tem risco zero de conflito.
2. O fmt reprovou nas minhas quebras de linha manuais; rodei `cargo fmt` **com o pin**, dentro da
   worktree, e **só arquivos meus se moveram** (confirmado por `git status`). Medido: o
   `rust-toolchain.toml` é versionado, então worktree e primário resolvem **o mesmo** rustfmt
   (1.9.0-stable, string idêntica) — a regra que previne o skew é o *pin*, não a árvore.

**LOC no limite:** `crates/ph2d-panel-vector/src/state.rs` está **exatamente em 600/600** (cresceu
+1 nesta linha). A próxima linha adicionada ali obriga o split.

## 7 — O que smoke-testar

**`env PH2D_BUILD_SMOKE=24 cargo run -p ph2d-host-desktop --release`**

A cena imprime o roteiro. O passo **7** é o novo, e os números dele são **MEDIDOS** por sonda
headless (`the_scene_shows_the_attitude_turning_and_repacking`), não afirmados:

> Arraste **Rotation** até 90°: as setas ficam **de pé, atravessadas** na curva e **continuam a
> acompanhar o arco** (a orientação é relativa à TANGENTE, não ao mundo). A contagem **sobe**:
> **15 → 21 cópias (1,40×), virando 87°**.

Se a tela discordar desses números, é a **mensagem** que está errada, não a sua leitura.

Conferir também: **Detach** solta o motivo e **a rotação morre com o vínculo** (re-prender não
ressuscita o ângulo), e o campo numérico ao lado do slider aceita graus digitados.

## 8 — Aberto, nomeado em vez de contrabandeado

- **O caminho do CHIP numérico até o bus não está gateado.** O chip não fala com o bus: ele dirige
  o slider pelo *link* do `WidgetStore` (`link_slider_number_mapped`), e o `set_number_value` do
  testkit **espeta o campo sem acionar o link** — escrevi um gate por cima disso, vi que nascia
  vermelho por artefato de **fixture** e não por defeito, e o troquei pelo gate do **par inverso**
  (`the_track_and_degree_maps_are_exact_inverses`). Fechar de verdade pede um método novo no
  testkit que comprometa um número **através** do link; é foundational e merece decisão própria.
- **O mapa track↔valor ainda tem 3 cópias no Spacing e no Offset** (populate/event/paint). Colapsei
  só o da Rotation numa porta única (`rotation_from_track`/`rotation_to_track`); fazer o mesmo nos
  irmãos é higiene de outro dono e alargaria o diff.
- **Observação, NÃO ação:** o `no_tofu_glyphs` afirma que a Inter não cobre o bloco de setas
  (U+2190..U+21FF), mas ao verificar o `°` eu resolvi os glyph ids reais no `InterVariable.ttf` e
  `→` (U+2192) resolveu para o glifo 1799. Não investiguei (pode ser outro font stack em runtime) e
  **não toquei no gate** — ele documenta três recorrências reais. Fica a nota para o dono dele.

## 9 — Ordem dos commits

Sequencial, sem dependências cruzadas:

1. `ceb980eb9` motor (`rotation_deg` + `Rotor` + bbox girado) — 3 gates, 3 mutações
2. `4a97ff5a8` componente ECS + os 3 contadores
3. `5f93e8129` seam completo (id → componente) — 6 gates, 6 mutações
4. `d0b67453c` cena de smoke com números medidos + fmt/typos
5. `4a0d8bb8a` porta única do mapa track↔graus — 1 gate, 1 mutação

**10 gates novos, 10 mutações, 10 sangram.** As três do motor e as três da ponte matam **um gate
cada, e um diferente de cada vez** — cada gate pina propriedade distinta.
