# HANDOFF DE INTEGRAÇÃO — linha `line/FLIP` (DIRETRIZ §1.5.9)

> **Para o agente integrador.** Este é o documento de fusão da linha FLIP ao `main`.
> Tracker técnico do módulo: [`HANDOFF_flip_impl.md`](HANDOFF_flip_impl.md).
> Guia do próximo implementador do módulo: [`HANDOFF_flip_NEXT.md`](HANDOFF_flip_NEXT.md).
>
> **A linha está FECHADA para integração** (W0+W1+W2 completos, gate batched verde).
> Há **1 bug visual conhecido e ACEITO pelo Enio** (as quinas do traço saem
> "mordidas" com hardness < 1 — §7); ele **não bloqueia a integração** (é cosmético,
> confinado a `flip.wgsl`, e o Flip só renderiza se o usuário ativar a tool).

---

## 1. Identidade

| | |
|---|---|
| **Branch** | `line/FLIP` |
| **HEAD** | o tip da branch — `git rev-parse --short line/FLIP` (os 2–3 últimos commits são **só docs**; o último de código é `0e966416`) |
| **Base (merge-base com `main`)** | `1c7c9a22` |
| **Commits na linha** | ~51 (todos `--no-verify`, fast mode) |
| **Escopo** | ADR-0114 — o 4º meio do PH2D (animação quadro-a-quadro, port 2D clean-room do Grease Pencil 5.2): W0 (modelo) + W1 (render GPU) + W2 (tool + painel + desenho + borracha + Select/gizmo) |

**Ordem dos commits:** linear, sem interdependência frágil (W0 → W1 → W2 → smoke
fixes → rodada 7 do raster). **Integre como UM bloco** — não faça cherry-pick
parcial (o W1 depende do modelo do W0; o shell wiring depende dos dois).

---

## 2. Foundational / compartilhado tocado (49 arquivos fora da pasta do módulo)

Tudo **aditivo** (nada reescrito). Foundational-não-contrato = editável pela linha
sob o gate testado (ADR-0107).

### 2.1 Crates novas (drop-in, entram pelo glob `crates/*` — sem editar `members`)

| Crate | Papel |
|---|---|
| `ph2d-flip` | modelo de documento puro (`FlipDoc → objects → layers → frames → FlipDrawing → FlipStroke`) |
| `ph2d-flip-render` | pipeline wgpu do traço + fill (passe dedicado, NÃO vai pela `vello::Scene`) |
| `ph2d-tool-flip` | a tool (drop-crate ADR-0040) |
| `ph2d-panel-flip` | painel docado (Mode/Brush/Color/Layers) |

### 2.2 Foundational editado (aditivo)

| Arquivo | Mudança | Risco |
|---|---|---|
| `crates/ph2d-ecs/src/flip_object_ref.rs` (NOVO) | componente `FlipObjectRef(u64)` — espelha `vec_path_ref.rs` | — |
| `crates/ph2d-ecs/src/lib.rs` | `pub mod flip_object_ref;` + `pub use` | ⚠️ **Vector também** |
| `crates/ph2d-ecs/src/scene/registry.rs` | `reg.register::<FlipObjectRef>` + `reg.len()` **24→25** | 🔴 **ver §3.1** |
| `crates/ph2d-render/src/registry.rs` · `crates/ph2d-script/src/registry.rs` | count **25→26** (somam `register_ecs_components`) | 🔴 **ver §3.1** |
| `crates/ph2d-editor-core/src/icons.rs` | `IconId::Flip` (+ `ALL_ICONS`, ordem alfabética entre `FitView`/`Folder`) | Mergiraf |
| `crates/ph2d-editor-core/src/ids/chrome/flip.rs` (NOVO) | consts `FLIP_*` + família runtime de ids de camada | — |
| `crates/ph2d-editor-core/src/ids/chrome/mod.rs` · `topbar.rs` | `pub mod flip;` + `TOPBAR_FLIP` | Mergiraf |
| `crates/ph2d-editor-core/src/screens/hero/chrome/flip_toggle.rs` (NOVO) + `chrome/mod.rs` | handler do pill (**z=271**; `mod`+`dispatch_all` GERADOS por `ph2d-chrome-sync`) | ver §5 |
| `crates/ph2d-editor-core/src/screens/hero/{fixture,paint,topbar/mod}.rs` | pill no topbar + z-order walk do painel | Mergiraf |
| `crates/ph2d-editor-core/src/widget/{mod,scrollbar}.rs` | `FLIP_SCROLLBAR_ID = NodeId(835)` + lista do teste | ⚠️ **Vector também toca `widget/mod.rs`** |
| `crates/ph2d-editor-core/src/interaction/dispatch/scroll.rs` | branch `scrollbar_panel_for_id` | — |
| `crates/ph2d-editor-core/tests/{node_id_collisions,arch_mode_has_reconcile,architecture_panel_wiring_parity}.rs` | ids novos na tabela · `set_mode`/`set_erase_mode` na benign-list · `FLIP_STROKE_SWATCH` (é picker) na allowlist | ⚠️ **Vector/anim também** |
| `crates/ph2d-{panel,tool}-registry-init/{Cargo.toml,src/lib.rs}` | feature `panel-flip` + push do painel/tool (**parte GERADA** — panel-sync/tool-sync) | ver §5 |
| `crates/ph2d-tool-flip/tests/tool_manifest_design_sync.rs` + `docs/design/{icons/flip.svg,tools/flip.toml}` | ícone + manifest de design da tool | — |
| `shells/desktop/Cargo.toml` · `Cargo.lock` | 4 deps novas + feature `panel-flip` no `default` | Cargo.lock: §5 |
| `shells/desktop/src/` (10 módulos NOVOS) | `flip_draw` · `flip_smooth` · `flip_erase` · `flip_layers` · `flip_entities` · `flip_transform` · `flip_gizmo_view` · `flip_demo` · `render_loop/{flip_bridge,flip_pass,flip_pass_cache}` | isolados |
| `shells/desktop/src/{app_state,init,main}.rs` | 6 campos novos em `AppGfx`/`App` + `mod` novos | ⚠️ **Vector/anim também** |
| `shells/desktop/src/render_loop/{mod,present,snapshots}.rs` | `flip_entities::sync` + `flip_transform` + passe do traço + `GizmoView` do Flip | ⚠️ **Vector/anim/audio também tocam `render_loop/mod.rs`** |
| `shells/desktop/src/input_dispatch.rs` | ~6 sites de pick/marquee/gizmo (TODOS ao lado do bloco vetorial existente, mesmo padrão) | ⚠️ **Vector também** |
| `shells/desktop/src/undo.rs` | `ProjectState` ganha 3º campo `flip: FlipDoc` | ⚠️ **Vector também** |
| `shells/desktop/src/project.rs` | `PROJECT_SCHEMA` **1→2** (o `flip` mudou o formato do save) | 🔴 **ver §3.2** |
| `shells/desktop/src/forwarding.rs` | `cursor_over_hero_panel` (roda do painel) | — |
| `shells/desktop/tests/architecture_no_downcast_to_concrete_tool_in_shell.rs` | `flip_bridge` na `DOWNCAST_ALLOWLIST` | — |

**Nenhum ponto de extensão central foi editado de forma não-append-only.**

---

## 3. Símbolos que podem COLIDIR (grep-áveis) — e o mapa de sobreposição real

**Rodei a interseção dos arquivos compartilhados entre `line/FLIP` e as outras 5
linhas abertas.** Só **duas** linhas encostam nos meus arquivos:

| Linha | Arquivos compartilhados em comum com FLIP |
|---|---|
| **`line/Vector`** 🔴 | `Cargo.lock` · `ph2d-ecs/src/lib.rs` · **`ph2d-ecs/src/scene/registry.rs`** · `editor-core/src/widget/mod.rs` · `tests/arch_mode_has_reconcile.rs` · `tests/node_id_collisions.rs` · `shells/desktop/Cargo.toml` · `app_state.rs` · `input_dispatch.rs` · `main.rs` · `render_loop/mod.rs` · `undo.rs` |
| **`line/anim`** ⚠️ | `Cargo.lock` · `tests/node_id_collisions.rs` · `shells/desktop/Cargo.toml` · `app_state.rs` · `main.rs` · `render_loop/mod.rs` |
| **`line/audio`** | `Cargo.lock` · `render_loop/mod.rs` |
| `line/Painter` · `line/motion-value` | **nenhum** (só `Cargo.lock` no motion-value) |

Todas essas edições são **aditivas em partes diferentes do arquivo** → o Mergiraf
funde. O que ele **não** decide são os 2 resíduos semânticos abaixo.

### 3.1 🔴 `ComponentRegistry` — FLIP e Vector registram componentes DIFERENTES

**Este é o conflito nº 1 da jornada.** Os dois bumpam o mesmo `assert_eq!`:

| Branch | componente novo | `ph2d-ecs` `reg.len()` | `ph2d-render`/`ph2d-script` |
|---|---|---|---|
| `main` | — | 24 | 25 |
| `line/FLIP` | `ph2d::ecs::FlipObjectRef` | **25** | **26** |
| `line/Vector` | `ph2d::ecs::VecShape` | **25** | 25 *(não bumpou — ver nota)* |

**Ao integrar AS DUAS:** o Mergiraf mantém as duas linhas `reg.register(...)`; o
número do `assert_eq!` é o resíduo — **o correto é 24 + 2 = `26`** em `ph2d-ecs`,
e **`27`** em `ph2d-render`/`ph2d-script` (que somam +1 ao count do ECS). Ajuste os
**3 sites** à mão depois do merge.

> **Nota (latente da linha Vector, não minha):** `line/Vector` registra `VecShape`
> mas **não** bumpou `ph2d-render`/`ph2d-script` (ainda em 25). Se o Vector for
> integrado sozinho, esses 2 testes ficam **vermelhos** — é o tipo de latente que
> só o `cargo check/test` da árvore combinada (ou o `ship.sh`) pega. Considere isso
> ao ordenar as integrações.

### 3.2 🔴 `PROJECT_SCHEMA` (save do projeto) — bump ÚNICO, não dois

`shells/desktop/src/project.rs`: `main` = 1, **FLIP = 2** (o campo `flip` mudou o
formato do postcard). O Vector **não** bumpou (mas o `VecShape` novo também muda o
`WorldSnapshot`). **Depois de fundir as duas linhas, o valor certo continua `2`** —
uma quebra de formato para o par. **Não coloque 3.**

### 3.3 Ids / consts / variants novos (valores literais, pra grep de mesmo-símbolo)

- **Componente ECS:** string canônica `"ph2d::ecs::FlipObjectRef"`.
- **`NodeId(835)` = `FLIP_SCROLLBAR_ID`** (`widget/scrollbar.rs`). Verifiquei: **nenhuma
  outra linha aberta usa 835** (todas param em 834) → sem colisão. Se uma linha nova
  aparecer, o próximo livre é 836.
- **`IconId::Flip`** — variant novo (o `main` tem 138; a linha, 139). **Nenhuma outra
  linha adiciona IconId** → sem colisão. Ordem alfabética é obrigatória (gate
  `enum_order_matches_svgs`); o slug é `flip` (`docs/design/icons/flip.svg`).
- **`TOPBAR_FLIP`** = `hash_node_id("flip")`; chrome handler `flip_toggle` com **z=271**.
- **Ids do painel** (`ids/chrome/flip.rs`): todos por `hash_node_id("flip.*")` — namespace
  próprio, sem número mágico. Família runtime: `flip_layer_widget_id(layer_u64, kind)` +
  `flip_layer_blend_option_id(layer_u64, mode)`.
- **Cluster de tool:** `"flip_tools"`. **Feature:** `panel-flip`.
- **Const:** `ph2d_flip::FLIP_SCHEMA_VERSION = 1`.
- **Campos novos em `AppGfx`/`App`:** `gfx.flip` · `gfx.flip_compose` · `gfx.flip_composite` ·
  `app.flip_entities` · `flip_active` · `flip_style` · `flip_draw` · `flip_active_layer` ·
  `flip_erasing`. (Aparecem nos **3 sites de destructure**: `present.rs`,
  `render_loop/mod.rs`, e o literal em `init.rs` — se o Mergiraf errar aqui, é o
  primeiro lugar a olhar.)
- **`ProjectState.flip`** — 3º campo (`undo.rs`).

---

## 4. Contratos congelados encostados (§4/§6)

**NENHUM.** `NodeOp`/`OpResolver`/`NodeManifest` (=2/1/8), `Tool`(=12)/`RasterEditTool`/
`CanvasPaintTool`/`PanelEvent`(=4), e a superfície `ph2d-vector-doc`/`-traits` — **todos
intactos**. O `ComponentRegistry` **não é** contrato congelado (é ponto de extensão
append-only). **Não exige ADR.**

---

## 5. Arquivos GERADOS / regen obrigatório pós-rebase

Se der conflito nestes, **NUNCA resolva na mão** (DIRETRIZ §1.5.5) — aceite um lado e
re-rode o sync; o staleness gate confirma:

| Arquivo | Regenerador |
|---|---|
| `ph2d-{tool,panel}-registry-init/src/lib.rs` (blocos gerados) | `cargo run -p ph2d-tool-sync` · `cargo run -p ph2d-panel-sync` |
| `screens/hero/chrome/mod.rs` (`mod` + `dispatch_all`) | `cargo run -p ph2d-chrome-sync` |
| `Cargo.lock` | `git checkout main -- Cargo.lock` + `cargo check -p ph2d-host-desktop` |

**Gates hand-maintained que o sync NÃO regenera** (o `foundational-integrate.sh` os
roda; se vermelhos, é aqui): `node_id_collisions` (tabela de ids + dynamic tables) ·
`arch_mode_has_reconcile` (benign-list) · `architecture_panel_wiring_parity`
(allowlist do swatch) · `widget/scrollbar.rs` (lista de ids) · a ordem do cluster de
tools · `EXPECTED_TYPED` do panel-registry.

---

## 6. O que SÓ o `ship.sh` pega (o gate de integração NÃO roda)

- **fmt:** rodei `rustup run 1.95 cargo fmt --check` nas 7 crates tocadas → **limpo no pin**.
  (Cuidado com o skew: `cargo fmt` plain ≠ pin 1.95.)
- **machete:** sem dep órfã. As **dev-deps** de `ph2d-flip-render` (`ph2d-render`/`ph2d-gpu`/
  `ph2d-painter-effects`) são usadas no `tests/composite_blend.rs`; as 4 deps de runtime no
  shell são todas usadas.
- **deny / audit (RUSTSEC):** **zero crate EXTERNA nova** (só path-deps + `serde`/`postcard`/
  `bytemuck`/`wgpu`, já no workspace) → sem superfície nova de advisory.
- **typos:** comentários em **pt-BR** (mesma convenção de `vec_entities.rs`, que passa CI).
  Baixo risco; o ship confirma.
- **`nextest-impacted`:** rodou verde na linha (957 passed no W0; suites por-crate depois).

---

## 7. 🟡 Bug visual conhecido, ACEITO pelo Enio (NÃO bloqueia a integração)

**As quinas afiadas do traço saem "mordidas" com hardness < 1** (smoke do Enio,
2026-07-11 — zigzag: um bocado reto arrancado do lado interno de cada virada).

- **Não é regressão de integração** — é o estado do rasterizador do traço, confinado a
  `crates/ph2d-flip-render/src/shaders/flip.wgsl`.
- **Não afeta nenhuma outra linha nem o main:** o Flip só rasteriza se o usuário ativar
  a tool; sem a tool, o passe é no-op.
- **Diagnóstico + caminho do fix** (pro próximo agente da linha, não pro integrador):
  `HANDOFF_flip_impl.md` §"Rodada 7" + `HANDOFF_flip_NEXT.md` §3.
- **O resto do módulo foi smokado e aprovado** pelo Enio: desenho, painel (Mode/Brush/
  Color/Layers + blend por-camada), borracha (Soft/Hard/Stroke), Select/gizmo, blend em
  tempo real, render por-quadro.

---

## 8. Gate batched da linha (rodado no HEAD `0e966416`)

- **`ph2d-flip`** 29 · **`ph2d-flip-render`** 23 (12 unit + 2 composite e2e GPU + **9 GPU
  de raster**, verdes em debug E `--release`) · **`ph2d-tool-flip`** 10 · **`ph2d-panel-flip`**
  seam 2 → **verde**.
- **Arch-gates** (editor-core + shell): `node_id_collisions`, `architecture_panel_wiring_parity`,
  `architecture_panel_loc_cap`, `no_magic_numeric`, `arch_mode_has_reconcile`,
  `architecture_interactive_crate_has_behavioral_test`, `no_tofu_glyphs`,
  `scrollable_panels_intercept_the_wheel`, `architecture_no_downcast_to_concrete_tool_in_shell`,
  `file_loc_caps` (600), registry-init counts → **verde**.
- `cargo clippy --all-targets` (5 crates + shell) → **limpo**. `cargo fmt --check` (pin 1.95) → **limpo**.
- Build `--release` do shell → **OK**.

*(O resultado completo do gate está no fim desta sessão; se algo divergir depois do
rebase, é resíduo da árvore combinada — exatamente o que o `foundational-integrate.sh`
existe para pegar.)*

---

## 9. Ordem sugerida de integração + smoke pós-merge

1. **Integre o FLIP DEPOIS do Vector** (ou antes — tanto faz), mas **reconcilie o
   `reg.len()` do §3.1 na 2ª das duas** — e, se o Vector entrar primeiro, **conserte o
   count latente dele** em `ph2d-render`/`ph2d-script` (§3.1, nota) senão a árvore
   combinada fica vermelha por culpa dele, não do FLIP.
2. `bash scripts/foundational-integrate.sh` de dentro de `Worktrees/line-FLIP` (ele faz
   rebase → re-sync → staleness → **`cargo check --workspace`** → `nextest-impacted` →
   `merge --ff-only`). A linha tocou foundational → o `--workspace` é obrigatório.
3. **Smoke pós-merge (o que provar):** ativar a pill **FLIP** num doc vazio cria um
   objeto e o traço desenha; o painel troca Mode/Brush/Color/Layers; a borracha apaga;
   o gizmo de sprite move/gira/escala o objeto Flip; **e as outras tools seguem normais**
   (o pill do Flip não pode roubar o canvas quando inativo).
   ```
   cd /home/enio/Documentos/Projetos/PH2D && cargo run -p ph2d-host-desktop --release
   ```
4. **Docs de planejamento untracked no primário** (`docs/Flip/`, ADR-0114,
   `project-memory/project_flip_*.md`): **não estão commitados nesta linha de propósito**
   (untracked no primário quebraria o `merge --ff-only` com "untracked working tree files
   would be overwritten"). **O Enio commita esses por fora**, antes ou depois da integração.
