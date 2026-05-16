# Wave 2 — Eliminando 100% das colisões multi-agente + Source-of-truth UI rigorosa

**Versão:** 1.0 — 2026-05-16
**Status:** Plano canônico aprovado. Wave 2 da migração convention-by-discovery.
**Audiência:** LLMs (Coordenador, Periféricos, PRCI) + Enio.
**Predecessor:** [`2026-05-convention-by-discovery.md`](2026-05-convention-by-discovery.md) (Wave 1) + [ADR-0027](../architecture/decisions/0027-convention-by-discovery.md).

---

## Context

Wave 1 (PRs 1-10, mergeado 2026-05-16) entregou a infraestrutura mínima: 5 tool-crates,
Registry runtime, shell decomposition parcial (init.rs/input_dispatch.rs/hero_intents.rs),
HR-18 declarada mas inativa. main.rs caiu de 3463 → 2421 LOC.

**Auditoria pós-Wave-1** (registrada na sessão de 2026-05-16) identificou que **10 pontos
de colisão e divergência permanecem**, suficientes para que multi-agente paralelo ainda
gere merge conflicts ou (pior) divergência silenciosa entre design e implementação:

1. `crates/ph2d-editor/src/icons.rs` — 874 LOC, enum IconId com 89 variants + fn cmds()
   715 LOC. SVGs canônicos correspondentes existem em `docs/design/icons/*.svg` (89
   arquivos, paridade 1:1). Port é **manual** e mantém-se à mão.
2. `crates/ph2d-tokens/src/color.rs` — header admite "manual sync — future codegen via
   build.rs". tokens.json é declarada source-of-truth mas não-conectada.
3. `crates/ph2d-editor/src/widget.rs` (98 LOC) + diretório `widget/` (8640 LOC, 30
   widgets). Cada widget novo edita `widget.rs` com `mod X` + `pub use X::*` — colisão alta.
4. `crates/ph2d-editor/src/screens/hero.rs` — impl block de 918 LOC. struct `HeroScreen`
   carrega 20 campos `pending_*`. State centralizado cresce sem teto.
5. `crates/ph2d-editor/src/screens/hero/fixture.rs::topbar_clusters()` — 160 LOC
   hard-coded duplicam manifests dos tool-crates. Registry runtime existe mas chrome não
   o consome.
6. `crates/ph2d-editor/src/screens/hero/ids.rs` — 253 consts em ranges manuais
   (100..199 TopBar, 200..299 LeftRail, etc). Race condition latente entre agentes que
   alocam ids ao mesmo tempo. Gap 500..599 sem alocação documentada.
7. `crates/ph2d-editor/src/screens/hero/topbar.rs::image_action_pills()` — lista
   hard-coded `[Trim, MakeSquare, BgRemoval]` em paralelo aos manifests, sem
   cross-validation. Divergência silenciosa.
8. 14 drains `pending_X` inline em `main.rs::render_frame` (Inspector edits +
   Hierarchy ops + Image edits não migrados em PR 9a).
9. HR-18 inativo: main.rs 2421 LOC excede cap 400.
10. `crates/ph2d-editor/src/lib.rs` com 50+ `pub use` re-exports. Cada widget novo
    precisa adicionar linha.

**Adicionalmente**: a source-of-truth da UI está fragmentada em 6 fontes não-sincronizadas
automaticamente:
- `docs/design/tokens.json` (canonical, OKLCH 4 themes)
- `crates/ph2d-tokens/src/*` (Rust mirror manual)
- `docs/design/icons/*.svg` (89 SVGs Lucide-derived)
- `crates/ph2d-editor/src/icons.rs` (enum manual)
- `docs/design/component-library.html` (mockup visual)
- `docs/design/screens/*.html` (17 mockups full-viewport)

**Outcome pretendido:** **zero divergência possível entre design e implementação**.
Periférico nunca erra sobre onde está a fonte da verdade porque o CI rejeita qualquer
divergência automaticamente. Coordenador deixa de existir como gargalo serial em qualquer
operação (incluindo widget/icon/screen/manifest novos).

---

## Princípios e invariantes preservados

Wave 2 não revoga nada de Wave 1 nem das HRs canônicas. Lista de obrigações abaixo;
qualquer PR que viole é inválido.

### Hard Rules
| HR | Implicação para Wave 2 |
|---|---|
| HR-1 core platform-agnostic | `build.rs` codegen é build-time apenas; runtime das libs continua platform-agnostic |
| HR-3 zero alloc hot path | Action queue usa `VecDeque` pré-alocada + `bumpalo` arena per frame (validação via dhat) |
| HR-5 determinismo | `hash_node_id` FNV-1a const cross-platform; OKLCH→sRGB já determinístico; Registry sort por (cluster, order, id) |
| HR-7 editor=off corta 100% | Tool-crates continuam `optional` deps gateadas por feature `editor`; CI gate `nm + grep ph2d_tool_` (skeleton de Wave 1 ativado em PR 11.9) |
| HR-12 a11y tree | Cada widget continua emitindo `Node`; golden image tests não interferem |
| HR-13 memory budget | Manifest declara budget; CI agrega e checa platform max (Wave 1 cobre) |
| HR-15 i18n | Label keys validadas contra design `.toml` em CI (PR 11.5); estendido a cor literal (no-literal-color em PR 11.6) |
| HR-18 cap LOC | **Ativado em PR 11.9** com main.rs < 400 obrigatório |

### ADRs preservados
ADR-0021 (SimWorld/PresentWorld), ADR-0022 (no-HashMap), ADR-0023 (UI 4-zonas),
ADR-0024 (input pipeline), ADR-0027 (convention-by-discovery). Wave 2 estende ADR-0027
via ADR-0028 (sem revogar).

### Stack inalterada
Wave 2 é estrutural. Zero versão de dependência muda. Tokens.json schema preservado
(consumer adicionado é build.rs, não mudança no JSON).

### Modelo operacional multi-agente preservado
Coordenador continua existindo mas seu escopo encolhe drasticamente para "revisor de
manifest + 1 linha em register_all + revisor de design TOMLs/SVGs". Periférico ganha
ainda mais autonomia (4 zonas de trabalho independentes: design canonical, tool-crate,
widget, test).

---

## Princípio canonical Wave 2 — Isolamento físico total de UI

**Regra forte:** todos os arquivos de UI (painel, render overlay, state, algoritmo) de
uma tool ou feature moram **dentro da pasta da própria tool/feature**. Periférico
trabalhando numa tool nunca toca `crates/ph2d-editor/src/` exceto pelo Coordenador
adicionando 1 linha em `register_init`.

Pós-Wave-2:
- `crates/ph2d-tool-grid-snap/src/` contém `panel/`, `state/`, `render/` próprios
  (move de `ph2d-editor/src/grid_snap/`).
- `crates/ph2d-tool-bgremoval/src/` contém `panel.rs`, `tool.rs`, `params.rs`,
  `scratch.rs`, `algorithm/` próprios (move de `ph2d-editor/src/tools/bgremoval/`).
- `ph2d-editor` deixa de conhecer state interno das tools — comunicação via
  `MANIFEST` (declarativo) + `Box<dyn Tool>` (já existe na `ToolRegistry`) + Action Bus
  (PR 11.8).

### Roadmap evolutivo (decisão fechada)

**Wave 2 adopta Nível 1** (princípio acima): tool-crate hospeda UI fisicamente.
Tool-crate importa `ph2d-editor` diretamente para consumir widgets/paint/FloatingPanel
(sem ciclo — Wave 1 retrofit já garantiu).

**Wave 3 (plano futuro, NÃO Wave 2):** Plugin trait formal — `pub trait EditorToolPlugin`
com hooks (`build_panel`, `handle_panel_event`, `render_canvas_overlay`,
`snapshot_state`, `restore_state`). Define contrato explícito em vez de acoplamento via
campos do `HeroScreen`. Permite hot-reload de tool no futuro.

**Wave 4 (se compile time virar issue):** extrair `ph2d-editor-toolkit` (widgets +
paint + FloatingPanel + interaction) para crate intermediário ~5000 LOC. Tool-crates
dependem do toolkit, não de `ph2d-editor` inteiro. Features não-tool
(Inspector/Hierarchy/AssetBrowser) viram `ph2d-feature-*` crates similares.

**Wave 2 NÃO faz Wave 3 nem Wave 4.** Premature optimization. Quando Wave 2 mostrar
custo concreto (compile time, friction), retoma planejamento Wave 3/4 com dados
empíricos.

---

## Design alvo (estado pós-Wave-2)

### Source-of-truth da UI consolidada

```
docs/design/tokens.json          ── canonical APARÊNCIA cores/sizes
        │ build.rs (PR 11.1)
        ▼
crates/ph2d-tokens               ── Rust consumido sem sync manual

docs/design/icons/*.svg          ── canonical APARÊNCIA glyphs (89 SVGs)
        │ build.rs (PR 11.2)
        ▼
ph2d_editor::icons::*_bezpath()  ── geradas; icons.rs encolhe 874→~50 LOC

docs/design/tools/<slug>.toml    ── canonical FUNCIONALIDADE tool metadata
        │ tests/architecture/tool_manifest_design_sync.rs (PR 11.5)
        ▼
crates/ph2d-tool-<slug>/MANIFEST ── replica do .toml; CI compara
        │ ph2d-tool-registry-init::register_all
        ▼
shells/desktop runtime           ── Registry derivada do .toml

docs/design/component-library.html + screens/*.html ── canonical VISUAL
        │ tests/golden/widget_*.rs (PR 11.10)
        ▼
crates/ph2d-editor/widget/*.rs   ── golden image diff contra mockup
```

**Regra forte (HR-15 estendida):** todo aspecto visual e funcional de UI tem **um único
local de declaração** em `docs/design/`. Implementação Rust é **gerada ou validada
cruzadamente**. Periférico nunca decide sobre aparência — só consome.

### Layout do repositório pós-Wave-2

```
crates/
  ph2d-icon-codegen/             ⬅ NOVO — build-script helper (SVG parser)
                                    consumido por ph2d-editor/build.rs
  ph2d-editor/
    build.rs                     ⬅ NOVO — gera fn <slug>_bezpath() de SVGs
    src/icons.rs                 ⬅ ENCOLHE 874→~50 LOC (re-exports gerados)
    src/widget.rs                ⬅ MARKER append-only + validate script
    src/grid_snap/               ⬅ DELETADO — movido para ph2d-tool-grid-snap
    src/tools/bgremoval/         ⬅ DELETADO — movido para ph2d-tool-bgremoval
    src/screens/hero.rs          ⬅ ENCOLHE 918→~400 LOC (state extraído)
    src/screens/hero/
      inspector_state.rs         ⬅ NOVO — InspectorState (10 pending_*)
      hierarchy_state.rs         ⬅ NOVO — HierarchyState (10 pending_*)
      image_edit_state.rs        ⬅ NOVO — ImageEditState (6 pending_*)
      ids.rs                     ⬅ MUDA: hash_node_id para todos os 253 consts
      fixture.rs                 ⬅ MUDA: chrome derivado do Registry
      topbar/                    ⬅ NOVO subdir — split de topbar.rs 690 LOC
        mod.rs                       em cluster.rs / image_action_row.rs /
        cluster.rs                   play_chip.rs
        image_action_row.rs
        play_chip.rs
      hierarchy/                 ⬅ NOVO subdir — split de hierarchy.rs 998 LOC
        mod.rs                       em row.rs / dnd.rs / menu.rs / snapshot.rs
        row.rs
        dnd.rs
        menu.rs
        snapshot.rs
    src/lib.rs                   ⬅ ENCOLHE: -50 pub use widget::*
  ph2d-tokens/
    build.rs                     ⬅ NOVO — gera consts de tokens.json
  ph2d-tool-grid-snap/           ⬅ EXPANDE de manifest-thin para UI completa
    src/
      lib.rs                       MANIFEST + register
      panel/                       ⬅ MOVE de ph2d-editor/src/grid_snap/panel.rs
        mod.rs                       (2869 LOC → split por GridKind)
        square.rs
        hex.rs
        iso.rs
        staggered.rs
        tri.rs
        quadtree.rs
        voronoi.rs
        chunks.rs
      state/                       ⬅ MOVE de ph2d-editor/src/grid_snap/state.rs
        mod.rs                       (1250 LOC → split por Cfg)
        cfg_square.rs
        cfg_hex.rs
        cfg_iso.rs
        cfg_quadtree.rs
        cfg_voronoi.rs
      render/                      ⬅ MOVE de ph2d-editor/src/grid_snap/render/
        mod.rs                       (já estava split)
        square.rs
        hex.rs
        ...
      inspect.rs                   ⬅ MOVE
      ids.rs                       ⬅ MOVE (renumerar para hash_node_id)
  ph2d-tool-bgremoval/           ⬅ EXPANDE de manifest-thin para UI completa
    src/
      lib.rs                       MANIFEST + register
      panel.rs                     ⬅ MOVE de ph2d-editor/src/tools/bgremoval/
      tool.rs                      ⬅ MOVE (538 LOC — implementação Tool trait)
      params.rs                    ⬅ MOVE
      scratch.rs                   ⬅ MOVE
      algorithm/                   ⬅ MOVE
        mod.rs
        chroma_flood.rs            (M1)
        grabcut/                   (M2 scaffold; M2 body pending)
          mod.rs
          gmm.rs
          graph.rs
          maxflow.rs

shells/desktop/src/
  action_bus.rs                  ⬅ NOVO — VecDeque<ActionInvocation> + drain
  tool_actions.rs                ⬅ NOVO — dispatcher genérico via Registry
  hero_intents.rs                ⬅ EXPANDE — +14 drains migrados
  main.rs                        ⬅ ENCOLHE 2421→<400 LOC (HR-18 enforced)

docs/design/
  tools/                         ⬅ NOVO subdir
    bgremoval.toml               ⬅ NOVO — canonical FUNCIONALIDADE
    grid_snap.toml               ⬅ NOVO
    make_square.toml             ⬅ NOVO
    trim_transparency.toml       ⬅ NOVO (stub para tool ainda não migrada)
    brush.toml                   ⬅ NOVO (stub)

tests/architecture/              ⬅ ATIVADO (existia parcialmente)
  file_loc_caps.rs               ⬅ NOVO — HR-18 enforcement
  no_literal_color.rs            ⬅ NOVO — HR-15 estendida
  node_id_collisions.rs          ⬅ NOVO — hash global validation
  chrome_manifest_coverage.rs    ⬅ NOVO — chrome ↔ manifest sync
  tool_manifest_design_sync.rs   ⬅ NOVO — .toml ↔ MANIFEST sync

tests/golden/                    ⬅ NOVO subdir
  widget_<name>.rs               ⬅ NOVO × 30 — golden image per widget
  baselines/*.png                ⬅ NOVO × 30 — committed (~300KB total)

docs/architecture/decisions/
  0028-wave-2-codegen-and-design-canonical.md  ⬅ NOVO ADR
```

---

## Fase 0 — Fechar Wave 1 antes de iniciar Wave 2 (OBRIGATÓRIO)

Working tree atual contém **TODOS os arquivos modificados/novos dos PRs 1-10 do
Wave 1 + plano Wave 2** sem commits. Antes de qualquer PR do Wave 2 ser iniciado,
Wave 1 deve ser fechado integralmente.

**Sequência:**

1. **`git status` audit completo** — verificar que TODOS os modificados são meus
   (sem vazamento de slot Periférico paralelo).
2. **Commits semânticos por PR** (10 commits, 1 por PR do Wave 1):
   - `feat(registry): foundation skeleton — ToolManifest + Registry + ActionInvocation (HR-3, HR-5) — PR 1`
   - `feat(registry): NodeId hash FNV-1a + IconHandle + collision detection (HR-5) — PR 2`
   - `feat(ci): registry CI lint stack — HR-12/HR-13/HR-15/HR-7 — PR 3`
   - `feat(tool-make-square): piloto convention-by-discovery + registry-init retrofit — PR 4 + PR 4.0`
   - `chore(registry-init): extract register_all to dedicated crate (PR 6.0 retrofit)`
   - `feat(tool-grid-snap): manifest thin (Stateful Tool, content remains in ph2d-editor) — PR 6`
   - `feat(tool-bgremoval): manifest thin (Stateful Tool, content remains in ph2d-editor) — PR 7`
   - `feat(shell): Registry runtime wired in AppGfx — PR 8`
   - `refactor(shell): init.rs extracted from resumed() (260→17 LOC, HR-18 prep) — PR 9c`
   - `refactor(shell): input_dispatch.rs extracted from window_event() (706→28 LOC) — PR 9b`
   - `refactor(shell): hero_intents.rs extracted from render_frame() (6 drains, main.rs -615 LOC) — PR 9a`
   - `docs(adr-0027): convention-by-discovery + shell decomposition + HR-18 declared — PR 10`
   - `docs(migracao): plano canonical Wave 2 — eliminating all collisions (17 PRs)`

3. **`git push` para `main`** (ou branch dedicada se Coordenador decidir branch policy
   nova) — single push contendo todos os commits.
4. **CI matrix** — workflow `spike.yml` roda automaticamente:
   - Linux + macOS + Windows
   - cargo nextest workspace
   - cargo clippy `-D warnings`
   - cargo fmt --check
   - replay hash cross-OS
   - frame budget bench
5. **PRCI assume monitoramento** — papel canonical per `04-Agente-PRCI.md` §10:
   - Polling 15min até CI conclude.
   - Se falha: PRCI diagnostica + corrige + re-push.
   - Loop fecha em `success` ou 3 ciclos falha consecutiva (escala para Enio).
6. **Merge para main** (se push foi para branch dedicada).
7. **Atualizar `docs/IntegracaoMultiAgente/STATE.md`:**
   - Histórico append-only: `2026-05-<dia> — Wave 1 mergeada (sha <X>); 11 PRs, ADR-0027 Accepted, SKILL 2.4`.
   - "Sha conhecido bom (rollback target)": atualizar para o sha pós-merge.

**Critério de aceite Fase 0:**
- [ ] `git status` clean.
- [ ] CI matrix verde em Linux + macOS + Windows.
- [ ] STATE.md atualizado.
- [ ] `main` local = `origin/main`.

**Risco Fase 0:** baixo. Trabalho já está auditado (1319 testes verdes localmente,
workspace clippy + fmt verdes, 4 smokes visuais confirmados pelo Enio nos PRs 4 / 9c / 9b / 9a).
Risco residual: CI cross-OS pode revelar issue específica de Windows/Linux (ex.: case
sensitivity de path, line endings). PRCI absorve.

**Bloqueio para Wave 2:** Wave 2 PR 11.1 só pode iniciar APÓS Fase 0 fechar com CI verde
+ merge. Se CI revelar bug cross-OS, fix vai em commit no Wave 1 (não Wave 2).

---

## Plano de execução — 17 PRs em 5 janelas

> **Por que 17 e não 12 (versão inicial):** auditoria pós-Buraco-1 (UI de tools stateful
> não isolada) + Buraco-2 (HR-18 vai detonar arquivos legacy ≥ 600 LOC: `grid_snap/panel.rs`
> 2869, `screens/hero/hierarchy.rs` 998, `screens/hero/topbar.rs` 690, `hero_intents.rs`
> vai ser ~1100 pós-PR-11.8) adicionou 5 PRs de decomposição + isolamento. Sem esses,
> HR-18 ativa em PR 11.9 falha CI imediatamente.

> **Restrições globais**: nenhum PR quebra os 1319 testes hoje passantes; nenhum PR
> força slots Periféricos ativos a parar; cada PR é reversível via `git revert <sha>`;
> commits locais até PRCI fazer push pro GitHub no fim da jornada; cada PR cita HR
> aplicável no commit message.

### Janela A — Codegen + Lint (paralela, 5 PRs sem conflito entre si)

#### PR 11.1 — `build.rs` em ph2d-tokens: tokens.json → Rust consts
**Objetivo:** eliminar sync manual entre `docs/design/tokens.json` e `crates/ph2d-tokens/src/*`.

**Arquivos:**
- NOVO `crates/ph2d-tokens/build.rs` (~250 LOC)
- NOVO `crates/ph2d-tokens/src/generated.rs` (em `OUT_DIR`, via `include!`)
- EDIT `crates/ph2d-tokens/Cargo.toml` — `[build-dependencies]` vazio (parser JSON ad-hoc)
- EDIT `crates/ph2d-tokens/src/color.rs` — tabela manual em `ColorToken::resolve` (~440 LOC) substituída por lookup em `&'static [(&str, Color)]` gerado
- EDIT `crates/ph2d-tokens/src/spacing.rs` — idem
- EDIT `crates/ph2d-tokens/src/radius.rs` — idem
- EDIT `crates/ph2d-tokens/src/lib.rs` — header perde menção "manual sync"

**Decisão técnica:** parser JSON ad-hoc (sem `serde`). tokens.json tem 79 LOC, estrutura
estável; ~150 LOC de Rust suficientes. build-dep `serde` adiciona ~1s de compile;
ad-hoc evita isso.

**Risco:** BAIXO. build-time only. Runtime API preservada.

**Critério de aceite:**
- `cargo check -p ph2d-tokens` verde.
- `cargo test -p ph2d-tokens` verde (incluindo contrast WCAG 2.2 AA tests).
- Mudar `docs/design/tokens.json` força rebuild + values novos chegam ao widget paint.

**Cita HR:** HR-1 (build-time, zero runtime dep), HR-5 (OKLCH determinístico).

---

#### PR 11.2 — `ph2d-icon-codegen` + `build.rs` em ph2d-editor: SVGs → fn <slug>_bezpath()
**Objetivo:** eliminar enum `IconId` + fn `cmds()` 715 LOC manualmente portados de SVGs.
89 funções geradas a partir de 89 SVGs canônicos.

**Arquivos:**
- NOVO `crates/ph2d-icon-codegen/Cargo.toml` (lib-only)
- NOVO `crates/ph2d-icon-codegen/src/lib.rs` (~400 LOC parser SVG → BezPath)
  - Suporta: `<rect>`, `<circle>`, `<line>`, `<path d=...>`, `<polyline points=...>`.
  - Aceita modifiers `rx`, `ry`, `transform="translate/scale/rotate"`.
  - Reusa `kurbo::BezPath::from_svg(d)` para `<path>` (já existe em vello/kurbo).
- NOVO `crates/ph2d-editor/build.rs` (~80 LOC; varre `docs/design/icons/*.svg`,
  chama `ph2d-icon-codegen`, escreve `OUT_DIR/icons_generated.rs`)
- EDIT `crates/ph2d-editor/Cargo.toml` — `[build-dependencies]` ganha ph2d-icon-codegen + walkdir
- EDIT `crates/ph2d-editor/src/icons.rs` (874→~50 LOC): `include!(concat!(env!("OUT_DIR"), "/icons_generated.rs"))`. Mantém o tipo `IconCmd` e re-exporta as funções geradas com nomes Rust-friendly (`save_bezpath`, `open_bezpath`, etc.). **enum `IconId` desaparece**.
- EDIT consumers em `crates/ph2d-editor/src/screens/hero/topbar.rs`, `fixture.rs`,
  `left_rail.rs`: trocar `IconId::Save` por `save_bezpath` (function pointer).
- EDIT `crates/ph2d-tool-*/src/lib.rs`: manifest `icon_fn` já usa fn pointer; sem mudança.

**Validação build-time:**
- SVG canônico sem consumer em manifest ou chrome fixo: warning amarelo (allowlist via `// icon-codegen: orphan-ok`).
- `icon_fn` referenciado em manifest mas sem SVG canônico: hard fail.

**Risco:** MÉDIO. Refactor consumer-side. Smoke visual obrigatório (golden image
hero antes/depois).

**Critério de aceite:**
- `cargo check --workspace` verde.
- `cargo test --workspace` verde.
- `wc -l crates/ph2d-editor/src/icons.rs` < 100.
- Smoke visual: editor abre, TopBar/LeftRail/Image Tools row → todos ícones
  renderizados idênticos (golden image hero diff SSIM ≥ 0.985).

**Cita HR:** HR-3 (BezPath alocação per call existia; sem regressão), §15 anti-pattern (centralized icons enum).

---

#### PR 11.3 — NodeId hash universal em ids.rs
**Objetivo:** substituir 253 consts NodeId manuais em ranges por hash determinístico.

**Arquivos:**
- EDIT `crates/ph2d-editor/src/screens/hero/ids.rs` (567→~300 LOC; valores absolutos
  mudam mas API const-name preservada):
  ```rust
  // Antes:
  pub const TOPBAR_SAVE: NodeId = NodeId(101);
  // Depois:
  pub const TOPBAR_SAVE: NodeId = hash_node_id("topbar.save");
  ```
- NOVO `tests/architecture/node_id_collisions.rs` — enumera todos os 253 consts via
  inventory manual (lista mantida no test); roda `detect_collisions` global.
- EDIT `crates/ph2d-editor/src/lib.rs` — re-export `hash_node_id` para ergonomia
  (opcional, paths longos OK).

**Decisão técnica:** **Quebra clean** dos valores numéricos antigos. Pre-1.0 (SKILL §12.3
"0.x.y aceita quebras em x"). Nenhum código de produção depende de `NodeId(101)` literal
— apenas const-names. Auditoria pré-PR: `grep -rn "NodeId([0-9])" crates/ shells/` deve
retornar apenas declarações em ids.rs.

**Risco:** MÉDIO. Mudança de valores absolutos pode afetar hit-tests em teste/snapshot
se algum baseline gravado tem NodeId literal. Auditoria do `git grep`.

**Critério de aceite:**
- `cargo test --workspace` verde.
- `node_id_collisions.rs` valida zero collision.
- Smoke visual: clicks em todos os widgets → comportamento idêntico.

**Cita HR:** HR-5 (FNV-1a const determinístico cross-platform), anti-pattern §15
(manual range allocation).

---

#### PR 11.6 — Lint `no-literal-color` (workspace test)
**Objetivo:** garantir que widget/screens não usam hex literais; tudo via
`ColorToken::resolve(theme)`.

**Arquivos:**
- NOVO `tests/architecture/no_literal_color.rs` — cargo-walk `crates/ph2d-editor/src/widget/**/*.rs` + `crates/ph2d-editor/src/screens/**/*.rs`; grep regex `\b0x[0-9A-Fa-f]{6,8}\b`; exception `// LITERAL-COLOR-OK: <razão>`.

**Allowlist explícita** (justificada no header do teste):
- `widget/blender_color_picker/*.rs` — color math interno usa hex de tabela HSV.
- Documentation comments (regex skip linhas que começam com `//` ou estão em `/// ` blocks).

**Risco:** BAIXO. Test-only.

**Critério de aceite:**
- `cargo test -p ph2d-editor --test no_literal_color` verde após cleanup.
- Se grep flag literal não previsto: PR cleanup junto (move para token).

**Cita HR:** HR-15 estendida (zero hardcoded em UI).

---

#### PR 11.10 — Golden image tests por widget
**Objetivo:** validar visualmente cada `paint_X` contra baseline; designer pode
re-emitir mockup e CI detecta divergência.

**Arquivos:**
- NOVO `tests/golden/widget_button.rs`, `widget_slider.rs`, ..., `widget_blender_color_picker.rs` (30 arquivos, ~80 LOC cada)
- NOVO `tests/golden/baselines/widget_*.png` (30 PNGs, 256×128 cada, ~10KB → ~300KB total) committed direto (sem git-lfs — small enough)
- NOVO `tests/golden/Cargo.toml` (se for sub-workspace member) ou tests adicionados a
  ph2d-editor `[dev-dependencies]`
- NOVO `tests/golden/lib.rs` ou helper `golden::compare(scene: VectorScene, baseline_path: &str)`:
  - Renderiza VectorScene via `vello::Renderer` headless em CPU mode (sem GPU).
  - SSIM via implementação local ou crate `image-ssim`; threshold 0.985.

**Risco:** MÉDIO. Setup headless Vello pode ter quirks; baselines committed precisam
de PR review humano em cada update.

**Critério de aceite:**
- Cada widget gera baseline aceito.
- Modificar `paint_button` quebra `widget_button.rs` test; designer aprova nova
  baseline; PR atualizado.

**Cita HR:** ADR-0023 (UI baseline visual lock).

---

### Janela B — Chrome derivado + design canonical (serial: 11.4 → 11.5)

#### PR 11.4 — Chrome derivado do Registry
**Objetivo:** `topbar_clusters()` e `image_action_pills()` consultam Registry; hard-coded
lists para tools migradas morrem.

**Pré-condições:** PRs 11.2 (icon_fn) + 11.3 (ids estáveis) mergeados.

**Arquivos:**
- EDIT `crates/ph2d-editor/src/screens/hero/fixture.rs::topbar_clusters()`:
  ```rust
  pub fn topbar_clusters(reg: &Registry) -> Vec<(NodeId, TopBarCluster)> {
      let mut out = Vec::with_capacity(16);
      // Chrome FIXED (não-tool): Theme, Save, Open, Project, Play, RightLayers, etc.
      out.push((ids::TOPBAR_THEME, TopBarCluster::theme("Forge")));
      out.push((ids::TOPBAR_SAVE, TopBarCluster::single("Save", save_bezpath)));
      // ... 7 itens fixed ...

      // Chrome DERIVADO (do Registry):
      for m in reg.cluster("topbar_top_level") {
          out.push((NodeId(hash_node_id(m.id).0), TopBarCluster::from_manifest(m)));
      }
      out
  }
  ```
- EDIT `crates/ph2d-editor/src/screens/hero/topbar.rs::image_action_pills()`:
  ```rust
  fn image_action_pills(reg: &Registry) -> Vec<(NodeId, fn() -> BezPath, &'static str)> {
      reg.cluster("image_tools")
          .iter()
          .map(|m| (NodeId(hash_node_id(m.id).0), m.icon_fn, m.label_key))
          .collect()
  }
  ```
- NOVO `tests/architecture/chrome_manifest_coverage.rs` — cada pill renderizado tem
  manifest correspondente OU declarado em `chrome_fixed` (allowlist).
- EDIT `crates/ph2d-editor/Cargo.toml` — ganha dep `ph2d-tool-registry-init` (somente
  pra testes; runtime continua receivable via param).
- EDIT calling sites: `paint_top_bar`/`paint_image_action_row` ganham `reg: &Registry`
  param. Propagação via `HeroScreen` (que já recebe registry via `paint_hero_screen` arg
  novo).

**Risco:** ALTO. Smoke visual obrigatório. NodeId mudam (hash vs hard-coded).

**Critério de aceite:**
- Smoke: TopBar visual idêntico (ordem dos pills, ícones, labels).
- Click em Trim/MakeSquare/BgRemoval/GridSettings → handler dispatched (mesmo
  comportamento).
- `chrome_manifest_coverage` test verde.

**Cita HR:** HR-5 (Registry sort `(cluster, order, id)`), ADR-0023.

---

#### PR 11.5 — Design canonical TOMLs para tools + cross-validation
**Objetivo:** declarar cada tool em `docs/design/tools/<slug>.toml` como source-of-truth
da FUNCIONALIDADE (label/icon/cluster/zone/role). CI valida que `MANIFEST` no Rust
replica fielmente.

**Pré-condições:** PR 11.4 mergeado (manifests estabilizados).

**Arquivos:**
- NOVO `docs/design/tools/bgremoval.toml` (~20 LOC):
  ```toml
  [tool]
  id = "bgremoval"
  cluster = "image_tools_rail"
  zone = "sidebar"
  order = 30
  a11y_role = "Button"
  icon_slug = "eraser"          # → docs/design/icons/eraser.svg
  touches_sim = false

  [label]
  pt_br_inline = "Bg Removal"   # fallback até Fluent wirar (HR-15 deferred)
  en_us_inline = "Bg Removal"
  fluent_key = "tool.bgremoval.label"  # apontará para .ftl quando ph2d-i18n M13+ landar

  [memory_budget]
  vram_mb = 0
  ram_mb = 16  # Oklab 1024² buffer
  heap_script_mb = 0
  ```
- NOVO `docs/design/tools/grid_snap.toml`, `make_square.toml`, `trim_transparency.toml`
  (stub), `brush.toml` (stub) — 5 arquivos totais.
- NOVO `tests/architecture/tool_manifest_design_sync.rs` — para cada `.toml`:
  - Carrega `MANIFEST` const do crate correspondente.
  - Compara campo-a-campo. Divergência → falha com diff inline.
- EDIT `docs/IntegracaoMultiAgente/03-Agente-Periferico.md` — receita: "criar tool nova"
  começa com `docs/design/tools/<slug>.toml`, depois crate replica.

**Decisão técnica fluent:** ph2d-i18n é stub (SKILL §7). Schema TOML inclui ambos
`*_inline` (string fallback) e `fluent_key` (chave Fluent para quando M13+ ativar).
CI lint atual valida `*_inline` ≡ MANIFEST.label_key string; Fluent activation futura
adiciona key validation contra bundle.

**Risco:** BAIXO. Design canonical + CI gate.

**Critério de aceite:**
- 5 TOMLs declarados, todos validados contra MANIFEST const.
- Mudar `.toml` sem atualizar Rust → falha CI com mensagem clara.

**Cita HR:** HR-15 (i18n preparado), HR-13 (budget canonical).

---

### Janela B' — Decomposição interna de arquivos ≥ 600 LOC (paralela, sem conflito entre si)

> **Pré-requisito crítico para HR-18:** sem isso, PR 11.9 (ativação CI) falha em
> arquivos legacy. Janela B' roda em paralelo com Janela B (chrome derivado) — não
> compartilham arquivos.

#### PR 11.7a — `crates/ph2d-editor/src/grid_snap/` decomposição interna
**Objetivo:** quebrar `panel.rs` (2869 LOC) e `state.rs` (1250 LOC) em sub-files
< 600 LOC cada. Movimentação puramente mecânica — comportamento idêntico.

**Arquivos:**
- EDIT `crates/ph2d-editor/src/grid_snap/panel.rs` (2869) → `panel/mod.rs`
  (orquestrador ~200 LOC) + `panel/square.rs` + `panel/hex.rs` + `panel/iso.rs` +
  `panel/staggered.rs` + `panel/tri.rs` + `panel/quadtree.rs` + `panel/voronoi.rs` +
  `panel/chunks.rs` (1 sub-arquivo por GridKind; cada ~250-400 LOC).
- EDIT `crates/ph2d-editor/src/grid_snap/state.rs` (1250) → `state/mod.rs`
  (~200 LOC, GridSnapState struct + traits) + `state/cfg_square.rs` +
  `state/cfg_hex.rs` + `state/cfg_iso.rs` + `state/cfg_staggered.rs` +
  `state/cfg_tri.rs` + `state/cfg_quadtree.rs` + `state/cfg_voronoi.rs` +
  `state/cfg_chunks.rs` (1 Cfg por GridKind).
- EDIT `crates/ph2d-editor/src/grid_snap/mod.rs` — re-export interno mantém API.

**Risco:** MÉDIO. Refactor mecânico extenso (~4000 LOC redistribuídas). Smoke visual
obrigatório (Grid Settings panel cicla 11 kinds + snap funciona).

**Critério de aceite:**
- `cargo test --workspace` verde.
- Smoke: editor abre, painel Grid Settings cicla todos os kinds, snap funciona.
- Cada sub-arquivo < 600 LOC.

**Cita HR:** HR-18 (pré-requisito).

---

#### PR 11.7b — `crates/ph2d-editor/src/screens/hero/hierarchy.rs` decomposição
**Objetivo:** quebrar 998 LOC em sub-files. Hierarchy panel tem 4 responsabilidades
naturais: row paint, DnD, context menu, snapshot building.

**Arquivos:**
- EDIT `crates/ph2d-editor/src/screens/hero/hierarchy.rs` (998) → `hierarchy/mod.rs`
  (orquestrador ~150 LOC) + `hierarchy/row.rs` (paint por row ~300 LOC) +
  `hierarchy/dnd.rs` (drag-and-drop intents ~250 LOC) + `hierarchy/menu.rs`
  (context menu ~150 LOC) + `hierarchy/snapshot.rs` (build_hierarchy_snapshot
  consumer ~150 LOC).

**Risco:** MÉDIO. Smoke: hierarchy panel render + DnD reparent + menu funcionam.

---

#### PR 11.7c — `crates/ph2d-editor/src/screens/hero/topbar.rs` decomposição
**Objetivo:** quebrar 690 LOC em sub-files. TopBar tem 3 responsabilidades: cluster paint,
image action row, play chip.

**Arquivos:**
- EDIT `crates/ph2d-editor/src/screens/hero/topbar.rs` (690) → `topbar/mod.rs`
  (orquestrador ~150 LOC) + `topbar/cluster.rs` (paint clusters ~250 LOC) +
  `topbar/image_action_row.rs` (image_action_pills + paint ~200 LOC) +
  `topbar/play_chip.rs` (play button + project chip ~150 LOC).

**Risco:** BAIXO. Mecânico.

---

### Janela C — Decomposição final + isolamento UI (serial alto risco: 11.7d → 11.8 → 11.8a → 11.8b → 11.9)

#### PR 11.7d — HeroScreen decomposição em sub-states por domínio
**Objetivo:** `pub struct HeroScreen` perde 20 campos `pending_*`; ganha 3 sub-structs
por domínio. Cresce-se por adicionar campo em sub-state, não em HeroScreen central.

**Arquivos:**
- NOVO `crates/ph2d-editor/src/screens/hero/inspector_state.rs` (~120 LOC):
  ```rust
  pub struct InspectorState {
      pub inspector_sprite: Option<InspectorSpriteInfo>,
      pub inspector_transform: Option<InspectorTransformInfo>,
      pub inspector_visibility: Option<InspectorVisibilityInfo>,
      pub inspector_name: Option<InspectorNameInfo>,
      pub last_inspector_entity: Option<u64>,
      pub pending_transform_edit: Option<InspectorTransformInfo>,
      pub pending_visibility_edit: Option<InspectorVisibilityInfo>,
      pub pending_name_edit: Option<InspectorNameInfo>,
      pub pending_sprite_source_change: Option<(u64, RequestedSpriteStrategy)>,
      pub pending_reimport: Option<u64>,
  }
  ```
- NOVO `crates/ph2d-editor/src/screens/hero/hierarchy_state.rs` (~100 LOC):
  ```rust
  pub struct HierarchyState {
      pub pending_visibility_toggle: Option<NodeId>,
      pub pending_reparent: Option<HierReparentIntent>,
      pub pending_duplicate: Option<NodeId>,
      pub pending_delete: Option<NodeId>,
      pub pending_reset_transform: Option<NodeId>,
      pub pending_add_child: Option<NodeId>,
      pub pending_hierarchy_row_click: Option<NodeId>,
      pub rename_target_row: Option<NodeId>,
      pub pending_rename_seed: Option<NodeId>,
      pub pending_rename_commit: Option<(NodeId, String)>,
  }
  ```
- NOVO `crates/ph2d-editor/src/screens/hero/image_edit_state.rs` (~80 LOC):
  ```rust
  pub struct ImageEditState {
      pub pending_trim_transparency: Option<u64>,
      pub pending_make_square: Option<u64>,
      pub pending_bgremoval: Option<u64>,
      pub pending_activate_bgremoval: bool,
      pub pending_undo_image_edit: bool,
      pub has_undoable_image_edit: bool,
  }
  ```
- NOVO `crates/ph2d-editor/src/screens/hero/view_state.rs` (~40 LOC):
  ```rust
  pub struct ViewState {
      pub pending_view_focus: Option<ViewFocusKind>,
      pub dragging_files: Option<(Vec<PathBuf>, (f32, f32))>,
      pub widget_gallery_visible: bool,
  }
  ```
- EDIT `crates/ph2d-editor/src/screens/hero.rs` (918→~400 LOC):
  ```rust
  pub struct HeroScreen {
      pub inspector: InspectorState,
      pub hierarchy: HierarchyState,
      pub image_edit: ImageEditState,
      pub view: ViewState,
      // chrome + store + interaction state continuam aqui:
      pub store: WidgetStore,
      pub hit_index: HitIndex,
      pub layout: HeroLayout,
      pub grid_snap_state: GridSnapState,
      pub gizmo_selection: Option<u64>,
      pub gizmo_drag: Option<GizmoDragState>,
      // ...
  }
  ```
- EDIT `shells/desktop/src/hero_intents.rs` — paths: `hero.pending_X` →
  `hero.inspector.pending_X` (mecânico, sed-driven).
- EDIT `shells/desktop/src/main.rs::render_frame` — 14 drains restantes usam paths
  novos (preparação para PR 11.8 que vai movê-los).
- EDIT `crates/ph2d-editor/src/screens/hero/inspector_sync.rs`, `hierarchy.rs`,
  `topbar.rs` — paths atualizados.

**Decisão técnica:** sub-state structs são `pub` (não pub(crate)) porque `hero_intents.rs`
na shell consome. Acesso via `hero.inspector.pending_transform_edit.take()`.

**Risco:** ALTO. ~600 LOC re-organizadas em 4 arquivos novos + paths atualizados em
~12 sites. Mecânico mas extenso.

**Critério de aceite:**
- `cargo test --workspace` verde.
- Smoke visual: Inspector edit + Hierarchy reparent + Trim/MakeSquare/BgRemoval +
  Cmd+Z undo → todos idênticos.

**Cita HR:** anti-pattern §15 (god-struct), HR-3 (estruturas POD, zero alloc adicional).

---

#### PR 11.8 — Drains residuais para hero_intents.rs + Action Bus genérico
**Objetivo:** os 14 `pending_*` drains restantes em `main.rs::render_frame` migram para
`hero_intents.rs` (Inspector/Hierarchy) ou viram Action queue (Image edits + Reimport).
main.rs < 400 LOC.

**Pré-condições:** PR 11.7d mergeado (sub-states existem para receber paths).

**Arquivos:**
- NOVO `shells/desktop/src/action_bus.rs` (~120 LOC):
  ```rust
  pub struct ActionBus {
      pub queue: VecDeque<ActionInvocation>,  // cap 256
      pub arena: Bump,                         // reset per frame
  }

  impl ActionBus {
      pub fn enqueue<P>(&mut self, action_id: &'static str, payload: P) { ... }
      pub fn drain(&mut self, registry: &Registry, ctx: ToolCtx<'_>) { ... }
  }
  ```
- NOVO `shells/desktop/src/tool_actions.rs` (~80 LOC) — Action handlers para 4 tool
  actions (Trim, MakeSquare, BgRemoval, Reimport):
  - Cada handler é fn `fn handle_make_square(ctx: &mut ToolCtx)` que faz exatamente o
    que `hero_intents::drain_make_square` faz hoje. Wire via lookup no Registry usando
    `manifest.handler` (que apontava para shadow_handler em Wave 1).
- EDIT `crates/ph2d-tool-make-square/src/manifest.rs` — handler real (substitui
  shadow_handler):
  ```rust
  handler: ToolHandler::OneShot {
      on_click: crate::handler::on_clicked,  // fn(&mut ToolCtx)
  },
  ```
  IGUAL para grid_snap (Stateful → on_panel_event real) e bgremoval (Stateful idem).
- EDIT `shells/desktop/src/hero_intents.rs` — receba 14 drains adicionais (Inspector
  intent drains + Hierarchy ops) como fns livres no padrão existente.
- EDIT `shells/desktop/src/main.rs::render_frame` (~1500→<200 LOC inline drains; ~80
  LOC orquestrador):
  ```rust
  fn render_frame(&mut self) {
      self.frame_bookkeeping();
      hero_intents::drain_all_inspector(app);  // 10 drains
      hero_intents::drain_all_hierarchy(app);  // 7 drains
      tool_actions::drain(app, &app.registry);  // 4+ tool actions via Action Bus
      self.extract_to_present();
      self.paint();
      self.present();
  }
  ```
- EDIT `shells/desktop/src/main.rs` overall — sai de 2421 LOC para < 400.

**Validação HR-3:**
- ActionBus.queue pré-alocada (capacity 256, suficiente para 256 actions/frame que é
  ordens de magnitude além do que usuário gera).
- `dhat-rs` em `tests/budget/no_alloc_hot_path.rs` ganha cobertura para `tool_actions::drain`
  — assert que 0 allocations no enqueue (payload em arena reseted/frame).

**Risco:** ALTO. Maior blast radius do Wave 2. Smoke visual obrigatório de tudo.

**Critério de aceite:**
- `cargo test --workspace` verde.
- Smoke visual exaustivo (todos os fluxos).
- `wc -l shells/desktop/src/main.rs` < 400.
- `dhat` bench valida zero alloc no hot path.

**Cita HR:** HR-3, anti-pattern §15 (god-file), HR-18 (cap enforce).

---

#### PR 11.8a — `shells/desktop/src/hero_intents.rs` split em sub-files
**Objetivo:** após PR 11.8 adicionar 14 drains adicionais, `hero_intents.rs` cresce
para ~1100 LOC violando HR-18. Splitar por domínio (paralelo aos sub-states do
PR 11.7d).

**Arquivos:**
- EDIT `shells/desktop/src/hero_intents.rs` (~1100 LOC pós-PR-11.8) → split em:
  - `hero_intents/mod.rs` (re-exports + `drain_all` orquestrador ~80 LOC)
  - `hero_intents/inspector.rs` (10 drains: transform_edit, visibility_edit,
    name_edit, sprite_source_change, reimport, ... ~400 LOC)
  - `hero_intents/hierarchy.rs` (10 drains: reparent, duplicate, delete,
    rename_seed, rename_commit, ... ~400 LOC)
  - `hero_intents/image_edit.rs` (drain_trim_transparency, drain_make_square,
    drain_bgremoval, drain_undo_image_edit ~250 LOC)
  - `hero_intents/view.rs` (drain_view_focus + dragging files ~80 LOC)
- EDIT `shells/desktop/src/render_loop.rs` ou similar — chamadas a
  `hero_intents::inspector::drain_all`, `hero_intents::hierarchy::drain_all`, etc.

**Risco:** BAIXO. Mecânico (sed-driven path migration).

**Critério de aceite:**
- Cada sub-arquivo < 600 LOC.
- `cargo test --workspace` verde.

**Cita HR:** HR-18 (pré-requisito).

---

#### PR 11.8b — UI isolada: mover `grid_snap/` e `tools/bgremoval/` para tool-crates (**ENDEREÇA PRINCÍPIO CANONICAL Wave 2**)
**Objetivo:** consumar o princípio "isolamento físico total de UI". Tool-crates de
grid-snap e bgremoval ganham SEUS PRÓPRIOS `panel/`, `state/`, `render/`, `algorithm/`
(em vez de manifest-thin com conteúdo em `ph2d-editor`).

**Pré-condições:** PR 11.7a/b/c (decomposições internas), PR 11.7d (HeroScreen state
extraído), PR 11.8 (Action Bus genérico).

**Arquivos:**
- MOVE `crates/ph2d-editor/src/grid_snap/` (sub-arquivos pós-PR-11.7a) → `crates/ph2d-tool-grid-snap/src/`.
  - `panel/` (8 sub-files) → tool-crate.
  - `state/` (9 sub-files) → tool-crate. **`GridSnapState` deixa de ser campo de `HeroScreen`** — passa a viver no tool-crate; editor consulta via Action Bus / trait getter ou Resource externa em PresentWorld.
  - `render/`, `inspect.rs`, `ids.rs` → tool-crate.
- MOVE `crates/ph2d-editor/src/tools/bgremoval/` → `crates/ph2d-tool-bgremoval/src/`.
  - `panel.rs`, `tool.rs`, `params.rs`, `scratch.rs` → tool-crate.
  - `algorithm/` (chroma_flood + grabcut scaffold + M2 body futuro) → tool-crate.
- EDIT `crates/ph2d-tool-grid-snap/Cargo.toml` — ganha dep `ph2d-editor` (para widgets/paint/FloatingPanel; sem ciclo desde Wave 1 retrofit).
- EDIT `crates/ph2d-tool-bgremoval/Cargo.toml` — ganha dep `ph2d-editor` (idem).
- EDIT `crates/ph2d-editor/src/lib.rs` — remove `pub mod grid_snap;`.
- EDIT `crates/ph2d-editor/src/tools/mod.rs` — remove `pub mod bgremoval;`.
- EDIT `crates/ph2d-editor/src/screens/hero.rs` — `grid_snap_state` field substituído por callback/trait acesso ao tool-crate via Registry lookup.
- EDIT `shells/desktop/src/main.rs` + `hero_intents::image_edit::*` — consumers de `GridSnapState` e `BgRemovalTool` usam path novo.
- DELETE `crates/ph2d-editor/src/grid_snap/` (vazio após move).
- DELETE `crates/ph2d-editor/src/tools/bgremoval/` (vazio após move).

**Decisão técnica chave — onde mora o state da tool stateful?**
- Opção 1: campo em `HeroScreen` (status quo). Acoplamento alto; ph2d-editor "sabe"
  do interior da tool. REJEITADA pelo princípio Wave 2.
- Opção 2: state VIVE no tool-crate; ph2d-editor pega referência via `Registry::tool_state_mut::<T>(id)`. Tool-crate exporta `pub struct GridSnapState`; Registry guarda como `Box<dyn Any>`. Editor faz downcast quando precisa. **ADOTADA**.
- Opção 3: Plugin trait formal (Wave 3). Adiada.

**Risco:** ALTO. Move cross-crate massiva (~5500 LOC grid-snap + ~1000 LOC bgremoval).
Smoke visual exaustivo obrigatório.

**Critério de aceite:**
- `cargo test --workspace` verde.
- Smoke: Grid Settings panel funciona idêntico; cicla 11 kinds; snap funciona em gizmo Translate + drag-drop.
- Smoke: BgRemoval tool ativável; preview snapshot push; full Apply funciona.
- `wc -l crates/ph2d-editor/src/grid_snap/` retorna nothing (deleted).
- `wc -l crates/ph2d-editor/src/tools/bgremoval/` retorna nothing (deleted).
- Periférico de grid-snap ou bgremoval pode trabalhar 100% em `crates/ph2d-tool-<slug>/` sem nunca abrir arquivo em `crates/ph2d-editor/`.

**Cita HR:** ADR-0023, princípio canonical Wave 2.

---

#### PR 11.9 — HR-18 ativação efetiva (CI gate)
**Objetivo:** `tests/architecture/file_loc_caps.rs` ativo e enforced via CI workflow.

**Pré-condições:** PR 11.7a/b/c (decomposições internas) + PR 11.7d (HeroScreen) +
PR 11.8 (Action Bus) + PR 11.8a (hero_intents split) + PR 11.8b (UI isolada).
Todos arquivos > 600 LOC eliminados; main.rs < 400 obrigatório.

**Arquivos:**
- NOVO `tests/architecture/file_loc_caps.rs` (~180 LOC; implementação tipo Apêndice H
  do plano canônico Wave 1):
  - cargo-walk `shells/*/src/**/*.rs` + `crates/ph2d-*/src/**/*.rs`.
  - Conta LOC por arquivo.
  - Detecta funções top-level via parse com `syn` (single-pass) — mede LOC de cada fn.
  - Aplica caps: file ≤ 600, fn ≤ 200, main.rs ≤ 400.
  - Exception via `// ph2d-loc-cap: <razão>` no topo do arquivo.
  - Falha com mensagem específica linha-a-linha.
- EDIT `.github/workflows/spike.yml` ou NOVO `.github/workflows/architecture.yml`:
  ganha job `loc-caps` que roda `cargo test -p ph2d-editor --test file_loc_caps` (ou
  workspace tests dir setup).

**Risco:** BAIXO. Test-only; depende de Wave 2 PRs anteriores.

**Critério de aceite:**
- `file_loc_caps` test verde.
- CI workflow job verde.
- Qualquer regressão futura (e.g., agente adiciona 100 LOC numa fn) → CI red.

**Cita HR:** HR-18 (enforce ativo).

---

### Janela D — Cleanup (paralela final: 11.11, 11.12)

#### PR 11.11 — lib.rs trim agressivo
**Objetivo:** eliminar zona de merge `crates/ph2d-editor/src/lib.rs` reduzindo `pub use`
re-exports drasticamente.

**Pré-condições:** PR 11.10 mergeado (golden tests usam paths atualizados).

**Arquivos:**
- EDIT `crates/ph2d-editor/src/lib.rs` (96 LOC →~30 LOC):
  - Remove `pub use widget::{... 50+ items ...}`.
  - Mantém apenas re-exports load-bearing: `paint_hero_screen`, `HeroScreen`, `Layout`,
    `Zone`, `Theme`, `ZenMode`, `ToastQueue`, `NodeId`.
- EDIT consumers em `shells/desktop/src/*.rs` — paths trocados:
  `ph2d_editor::Button` → `ph2d_editor::widget::Button`.
- EDIT `tests/golden/widget_*.rs` (gerados em PR 11.10) — usam paths longos desde início.

**Decisão técnica:** Quebra API pública. SKILL §12.3 marca PH2D como pré-1.0 ("0.x.y
aceita quebras em x"). Wave 2 é boundary correto.

**Risco:** MÉDIO. Mecânico mas extenso. ~30-50 sites de import.

**Critério de aceite:**
- `cargo check --workspace` verde.
- `cargo test --workspace` verde.

**Cita HR:** anti-pattern §15 (merge zone eliminado).

---

#### PR 11.12 — widget.rs marker + cleanup + ADR-0028 + SKILL update + Multi-agente docs
**Objetivo:** fechamento canonical de Wave 2.

**Arquivos:**
- EDIT `crates/ph2d-editor/src/widget.rs` (98 LOC, mantido como mod tree):
  - Adicionar header marker `// AUTO-GENERATED: append only — see Wave 2 ADR-0028`.
  - Manter `mod X;` em ordem alfabética estrita.
- NOVO `scripts/validate_widget_mod_order.sh` (~30 LOC bash):
  - Checa que linhas `mod X;` em `widget.rs` estão alfabeticamente ordenadas.
  - Roda em CI como pre-commit guard.
- NOVO `docs/architecture/decisions/0028-wave-2-codegen-and-design-canonical.md` (ADR
  completo, ~250 LOC; vide seção §11 abaixo).
- EDIT `SKILL_Stack_PH2D_Definitiva.md`:
  - §1.4 versão: 2.4 → **2.5** com nota Wave 2.
  - §7 layout repositório: adicionar `ph2d-icon-codegen` + `tests/architecture/` ativo +
    `docs/design/tools/`.
  - §9 HR-18: status atualizado ("ativo desde Wave 2 PR 11.9").
  - §14 receita "Adicionar uma tool" reescrita: começa com `docs/design/tools/<slug>.toml`
    e `docs/design/icons/<slug>.svg`, depois crate replica.
  - §15 anti-patterns: 2 novos itens (manual icon porting, manual color hex literal).
  - §19 ADR-0028 na tabela.
- EDIT `docs/IntegracaoMultiAgente/02-Coordenador.md`:
  - Receita de integração nova (Wave 2 era): edita 1 linha em `register-init/lib.rs` +
    revisa `.toml` e `.svg` canônicos.
- EDIT `docs/IntegracaoMultiAgente/03-Agente-Periferico.md`:
  - Receita "criar tool": (1) escrever `docs/design/tools/<slug>.toml`, (2) drop SVG em
    `docs/design/icons/`, (3) criar crate `ph2d-tool-<slug>/` replicando `.toml`,
    (4) CI valida cross-source. Coordenador adiciona linha em registry-init.

**Risco:** BAIXO. Doc + cleanup.

**Critério de aceite:**
- ADR-0028 mergeado, `Accepted` status.
- SKILL versão 2.5.
- `validate_widget_mod_order.sh` roda verde.
- Periférico/Coordenador docs atualizados.

---

## ADR-0028 a criar — escopo canonical

```markdown
# ADR-0028: Wave 2 — Codegen pipeline + Lint guards + Design canonical sources

**Status:** Accepted
**Data:** 2026-05-<dia>
**Decisor(es):** Enio + LLM Coordenador

## Contexto
Wave 1 (ADR-0027) entregou Registry + tool-crates + shell decomposition parcial mas
deixou 10 pontos de colisão (icons.rs gigante, hero.rs god-struct, ranges manuais de
NodeId, source-of-truth UI fragmentada em 6 fontes não-sincronizadas). Auditoria
pós-Wave-1 em 2026-05-16 identificou que multi-agente paralelo ainda gera divergência
silenciosa entre design canonical (`docs/design/`) e implementação Rust.

## Decisão
PH2D adota *codegen-from-design-canonical* como mecanismo permanente:

1. **`docs/design/` é canonical** — toda aparência (tokens.json, icons SVG) e
   funcionalidade declarativa (tools .toml) origina aqui.
2. **`build.rs` codegen** lê design → gera Rust consts/fns. Eliminação de sync manual.
3. **CI cross-validation** garante zero divergência: `tool_manifest_design_sync`,
   `chrome_manifest_coverage`, `no_literal_color`, `node_id_collisions`.
4. **HR-18 ativado**: `file_loc_caps` ≤ 600/200/400 enforced.
5. **HeroScreen decomposed** em sub-states por domínio (Inspector/Hierarchy/ImageEdit/
   View).
6. **Action Bus genérico** substitui últimos `pending_*` drains de tool actions; chrome
   derivado do Registry.
7. **lib.rs trim**: pub use re-exports drasticamente reduzidos; consumers usam paths
   explícitos.

## Consequências
Positivas: zero divergência silenciosa design ↔ Rust; Periférico tem 4 zonas
independentes (design, tool-crate, widget, test); Coordenador edita 1 linha por tool
nova; HR-18 enforced; main.rs growth-bounded < 400 LOC.

Negativas: build.rs adiciona ~3-5s ao primeiro cargo build (subsequent incremental
~0); 5 crates novos (ph2d-icon-codegen + 4 design TOMLs); golden image baselines
adicionam ~300KB ao repo (committed direto, sem git-lfs); quebra API pub use em
lib.rs (pre-1.0 OK per SKILL §12.3).

Neutras: design canonical TOMLs em paralelo a Fluent bundles (HR-15 deferred);
schema TOML inclui `*_inline` (fallback) e `fluent_key` (ativação futura).

## Alternativas rejeitadas
- **Custom clippy lint** (`ph2d-clippy::no-literal-color`): alto setup cost (cargo
  plugin), baixo ROI vs cargo-walk workspace test.
- **Crate-per-widget**: radical, alto blast radius (30 widgets × movimentação);
  defere até pressão real demandar.
- **proc-macro `derive(ToolManifest)`**: incompatível com `const`.
- **`linkme`/`inventory`**: rejeitado em Wave 1 por fragility cross-platform (mantido).

## Referências
- Plano canônico Wave 2: `docs/Migracao/2026-05-wave-2-eliminating-all-collisions.md`.
- Predecessor: ADR-0027 (Wave 1).
- HRs: HR-3, HR-5, HR-7, HR-13, HR-15, HR-18.
```

---

## Ordem de execução com dependências

```
JANELA A — Codegen + lint paralelo (5 PRs sem conflito):
  PR 11.1  (tokens codegen)
  PR 11.2  (icons codegen)
  PR 11.3  (NodeId hash)
  PR 11.6  (no-literal-color)
  PR 11.10 (golden images)

JANELA B — Chrome derivado serial:
  PR 11.4  (chrome derivado)    ← deps: 11.2 (icon_fn) + 11.3 (ids)
  PR 11.5  (.toml design sync)  ← deps: 11.4 (manifests estáveis)

JANELA B' — Decomposições internas (PARALELO com Janela B; sem conflito):
  PR 11.7a (grid_snap/ split panel+state em sub-files)
  PR 11.7b (hierarchy.rs split)
  PR 11.7c (topbar.rs split)

JANELA C — Refactor pesado SERIAL (Coordenador-only):
  PR 11.7d (HeroScreen state extract) ← deps: 11.4 (chrome estável)
  PR 11.8  (drains + Action Bus)      ← deps: 11.7d (sub-states existem)
  PR 11.8a (hero_intents split)       ← deps: 11.8 (file expandiu p/ ~1100 LOC)
  PR 11.8b (UI isolada — MOVE tools)  ← deps: 11.7a + 11.7d + 11.8
  PR 11.9  (HR-18 CI gate)            ← deps: 11.7a+b+c + 11.8a + 11.8b
                                         (TODOS arquivos < 600 LOC)

JANELA D — Cleanup paralelo final:
  PR 11.11 (lib.rs trim)        ← deps: 11.10 + 11.8b
  PR 11.12 (ADR-0028 + SKILL)   ← deps: TUDO
```

**Distribuição multi-agente:**
- **Janela A** (5 PRs paralelos): 4 Periféricos + Coordenador (PR 11.6 trivial).
- **Janela B** (serial 11.4→11.5): 1 Periférico.
- **Janela B'** (3 PRs paralelos): 3 Periféricos em decomposições independentes
  (grid_snap/, hierarchy/, topbar/) — **PARALELO com Janela B** porque tocam arquivos
  distintos.
- **Janela C** (serial 5 PRs, alto risco): **Coordenador-only** por força do blast
  radius (cross-crate moves em PR 11.8b).
- **Janela D**: 1 Periférico (11.11) + Coordenador (11.12) em paralelo.

**Janelas A + B' simultâneas = pico de 7 Periféricos paralelos** (5 da Janela A + 3 da
Janela B' — mas Coordenador faz Janela B com 1 deles, então até 7 slots em uso).
Pre-Wave-1 esse paralelismo seria impossível.

---

## Critérios de aceite globais (DoD Wave 2)

- [ ] 12 PRs mergeados em `main` local.
- [ ] `cargo test --workspace` verde (1319+ existentes + ~80 novos: golden 30 + lints 5 +
      manifests 4 + ...).
- [ ] `cargo clippy --workspace -- -D warnings` verde.
- [ ] `cargo fmt --check` verde.
- [ ] **HR-18 CI gate verde** (`file_loc_caps` test).
- [ ] **Cross-validation lints todos verdes**: `tool_manifest_design_sync`,
      `chrome_manifest_coverage`, `no_literal_color`, `node_id_collisions`.
- [ ] `wc -l shells/desktop/src/main.rs` < 400.
- [ ] `wc -l crates/ph2d-editor/src/icons.rs` < 100.
- [ ] `wc -l crates/ph2d-editor/src/screens/hero.rs` impl block < 500.
- [ ] **TODOS arquivos `.rs` em `crates/ph2d-*/src/**/*.rs` e `shells/*/src/**/*.rs` < 600 LOC** (HR-18 enforce). Sem exceções.
- [ ] `crates/ph2d-editor/src/grid_snap/` deletado (movido para `ph2d-tool-grid-snap`).
- [ ] `crates/ph2d-editor/src/tools/bgremoval/` deletado (movido para `ph2d-tool-bgremoval`).
- [ ] **Princípio canonical Wave 2 validado**: Periférico pode trabalhar em
      `crates/ph2d-tool-<slug>/` sem nunca abrir arquivo em `crates/ph2d-editor/`
      (confirmado por checagem manual em sessão de teste com slot novo).
- [ ] Smoke visual: `PH2D_HERO_LIVE=1 cargo run -p ph2d-host-desktop`:
  - Editor abre normal, boot mensagens incluem `[Xms] PR 8: tool registry built (3 manifests)`.
  - TopBar: chrome fixed + 3 derivados ordenados por manifest.order.
  - LeftRail: BgRemoval ativável; Brush/Move/BgRemoval funcionam.
  - Image Tools row: Trim/MakeSquare/BgRemoval pills funcionam.
  - Inspector edit Transform/Visibility/Name → SimWorld update.
  - Hierarchy DnD reparent/duplicate/delete → comportamento idêntico.
  - Cmd+Z undo do último image-edit funciona.
  - Grid Settings panel cicla 11 kinds + snap.
  - Color picker (BlenderColorPicker) funciona.
  - Zen mode toggle.
- [ ] ADR-0028 mergeado e linkado em SKILL §19.
- [ ] SKILL §1.4/§7/§9/§14/§15/§19 atualizados (versão 2.5).
- [ ] `docs/IntegracaoMultiAgente/{02,03}.md` atualizados com receita Wave 2.
- [ ] Periférico chegando consegue criar tool nova em ≤30min (drop SVG +
      TOML + crate + 1 linha registry-init).

---

## Riscos, mitigação, rollback

### Risco: PR 11.2 — parser SVG omite construct usado em algum dos 89 SVGs
**Mitigação:** Auditoria pré-PR: `grep -l "<g " docs/design/icons/*.svg` etc. para
elementos SVG não-suportados. Adicionar suporte no parser ou normalizar SVG. Fixture
test no `ph2d-icon-codegen` parsing cada um dos 89 SVGs antes do PR.

**Rollback:** `git revert <sha>` — icons.rs antigo retorna.

### Risco: PR 11.7 — Hero state decomposition quebra paths em testes existentes
**Mitigação:** sed-driven path migration (alta confiabilidade); `cargo test` cobre
regressão.

### Risco: PR 11.8 — Action Bus introduz latência de 1 frame em image edits
**Mitigação:** Drain happens ANTES do extract-to-present no mesmo frame. Comportamento
idêntico ao inline drain atual.

### Risco: PR 11.10 — Golden image baselines diferem cross-platform (Linux CI vs Mac dev)
**Mitigação:** Headless Vello em CPU mode (sem GPU); SSIM 0.985 threshold absorve
sub-pixel drift. Se Linux/Mac divergem além disso, separar baselines por OS via cfg.

### Risco: PR 11.11 — lib.rs trim quebra consumer externo (plugin futuro)
**Mitigação:** Pre-1.0 (SKILL §12.3); CHANGELOG.md anotado. Não há consumers externos
conhecidos hoje (PH2D ainda interno).

### Risco geral: cargo build mais lento por causa de build.rs
**Mitigação:** 2 build.rs adicionados; cada um <5s no clean build, ~0 incremental
(sccache na CI). Aceitável; benefício > custo.

---

## Updates canônicos a serem feitos

### SKILL_Stack_PH2D_Definitiva.md versão 2.5

- **§1.4 versão**: `2.5 — 2026-05-<dia> (Wave 2 convention-by-discovery: codegen
  pipeline + design canonical TOMLs + HR-18 ativado; ADR-0028 Accepted)`.
- **§5 stack pinada**: zero versão muda; nota sobre `build.rs` em ph2d-tokens +
  ph2d-editor (build-time only, runtime zero-dep).
- **§7 layout**: novo crate `ph2d-icon-codegen`; `tests/architecture/` ativo;
  `docs/design/tools/` populated.
- **§9 HR-18**: status atualizado "Ativo desde Wave 2 PR 11.9 — `file_loc_caps` test
  enforced via CI".
- **§11.9 Editor UI**: nota sobre Registry como source-of-truth de chrome (PR 11.4
  finalizou).
- **§14 receita atualizada** "Adicionar uma tool":
  1. Escrever `docs/design/tools/<slug>.toml` (label, icon_slug, cluster, zone, order,
     a11y_role, touches_sim, memory_budget).
  2. Drop SVG em `docs/design/icons/<slug>.svg` (Lucide-derived, 24×24 viewBox).
  3. Criar `crates/ph2d-tool-<slug>/` replicando `.toml` no `MANIFEST` const.
  4. Coordenador adiciona linha em `register_init::register_all` (alfabético).
  5. CI valida cross-source: `.toml` ↔ `MANIFEST`, `.svg` ↔ `icon_fn`, label_key ↔
     bundle Fluent (quando ativo).
- **§15 anti-patterns**: 2 novos itens:
  - **Manual SVG → BezPath porting**: `crates/ph2d-editor/src/icons.rs` enum gigante era
    antipattern. Wave 2 ADR-0028 substituiu por `build.rs` codegen automático.
  - **Color hex literal em widget/screens**: usar `ColorToken::resolve(theme)`;
    enforced em CI por `no_literal_color`.
- **§19 ADR-0028** na tabela.

### ADR-0028
Vide §11 acima.

### Diretrizes Multi-Agente
- **`02-Coordenador.md` v1.2**: receita "integrar tool nova" — 1 linha em
  `register_init::register_all`. Plus revisa `.toml` + `.svg` canônicos. Não toca
  Cargo.toml, icons.rs, fixture.rs, ids.rs (todos automatizados).
- **`03-Agente-Periferico.md` v1.2**:
  - Periférico de tool: começa em `docs/design/tools/<slug>.toml` + drop SVG.
  - Periférico de widget: cria widget novo em `crates/ph2d-editor/src/widget/<slug>.rs`;
    adiciona ao `widget.rs` (append-only via script de validação); golden image baseline
    auto-gerada pode ser commit junto.
  - Periférico de screen: trabalha em `crates/ph2d-editor/src/screens/<name>/` (subpasta
    própria por screen).

---

## Apêndices

### A — Shape canonical de `docs/design/tools/<slug>.toml`

```toml
# Schema canonical Wave 2.
# Validado contra MANIFEST const por tool-manifest-design-sync test.

[tool]
id = "<slug>"               # snake_case; matches MANIFEST.id
cluster = "<cluster_id>"    # matches MANIFEST.cluster
zone = "TopLeft|TopRight|Sidebar|Center"  # ADR-0023
order = <u32>               # sort within cluster
a11y_role = "Button|Switch|MenuItem|CheckBox"  # accesskit Role variant
icon_slug = "<svg_filename>" # docs/design/icons/<svg_filename>.svg
touches_sim = <bool>        # HR-5 marker

[label]
pt_br_inline = "<string>"   # fallback até Fluent wirar
en_us_inline = "<string>"
fluent_key = "tool.<slug>.label"  # planned Fluent key (HR-15)

[memory_budget]            # HR-13
vram_mb = <u32>
ram_mb = <u32>
heap_script_mb = <u32>

[mcp]                       # HR-8/HR-11 — reserved
exposed = false
destructive = false
handle_only = true
```

### B — Shape `tests/architecture/file_loc_caps.rs`

```rust
//! HR-18 enforcement: caps de LOC em shells/ + crates/ph2d-*/.

const FILE_CAP: usize = 600;
const FN_CAP: usize = 200;
const MAIN_RS_CAP: usize = 400;

#[test]
fn shells_and_crates_respect_loc_caps() {
    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new("shells")
        .chain(walkdir::WalkDir::new("crates"))
        .into_iter().filter_map(Result::ok)
        .filter(|e| e.path().extension().map(|x| x == "rs").unwrap_or(false))
        .filter(|e| !e.path().to_string_lossy().contains("/tests/")
            && !e.path().to_string_lossy().contains("/target/"))
    {
        let path = entry.path();
        let content = std::fs::read_to_string(path).unwrap();
        if has_exception(&content) { continue; }

        let loc = content.lines().count();
        let cap = if path.file_name().unwrap() == "main.rs" {
            MAIN_RS_CAP
        } else { FILE_CAP };
        if loc > cap {
            violations.push(format!("{}: {} LOC > {} cap (HR-18)",
                                    path.display(), loc, cap));
        }

        for (fn_name, fn_loc) in find_top_level_fns(&content) {
            if fn_loc > FN_CAP {
                violations.push(format!("{}::{}: {} LOC > {} fn cap (HR-18)",
                                        path.display(), fn_name, fn_loc, FN_CAP));
            }
        }
    }
    assert!(violations.is_empty(),
            "HR-18 violations:\n{}", violations.join("\n"));
}

fn has_exception(content: &str) -> bool {
    content.lines().take(5).any(|l| l.contains("// ph2d-loc-cap:"))
}

fn find_top_level_fns(content: &str) -> Vec<(String, usize)> {
    // syn::parse_file + syn::Item::Fn iteration; medir span LOC.
    // (Impl detail; ~30 LOC.)
    use syn::Item;
    let file = syn::parse_file(content).ok();
    file.map(|f| f.items.into_iter().filter_map(|item| {
        if let Item::Fn(f) = item {
            let name = f.sig.ident.to_string();
            // Approx LOC: span tokens count / heuristic. Or use proc_macro2::Span.
            let loc = f.block.stmts.len() * 2;  // rough proxy; refine.
            Some((name, loc))
        } else { None }
    }).collect()).unwrap_or_default()
}
```

### C — Estimativa de tempo (sessões agente)

- **PR 11.1** tokens codegen — 1 sessão.
- **PR 11.2** icons codegen — 2 sessões (parser SVG + 89 validações).
- **PR 11.3** NodeId hash — 1 sessão.
- **PR 11.4** chrome derivado — 2 sessões (smoke visual crítico).
- **PR 11.5** .toml design sync — 1 sessão.
- **PR 11.6** no-literal-color — 0.5 sessão.
- **PR 11.7a** grid_snap/ decomposição interna — 2 sessões (~4000 LOC reorganizados).
- **PR 11.7b** hierarchy.rs split — 1 sessão.
- **PR 11.7c** topbar.rs split — 0.5 sessão.
- **PR 11.7d** HeroScreen state extract — 2 sessões (mecânico extenso).
- **PR 11.8** drains + Action Bus — 2-3 sessões (highest blast radius).
- **PR 11.8a** hero_intents split — 0.5 sessão.
- **PR 11.8b** UI isolada (MOVE tools) — 3 sessões (cross-crate massiva,
  smoke visual exaustivo).
- **PR 11.9** HR-18 CI gate — 1 sessão.
- **PR 11.10** golden image tests — 2 sessões (baselines + designer review).
- **PR 11.11** lib.rs trim — 0.5 sessão.
- **PR 11.12** ADR-0028 + SKILL + docs — 1 sessão.

**Total estimado: 23-24 sessões.** ~50% mais que Wave 1. Paralelização multi-agente
em Janelas A + B' simultâneas (até 7 Periféricos) reduz tempo wall-clock para ~2-3
jornadas humanas. Janela C serial inevitável (Coordenador-only).

---

## Roadmap pós-Wave-2 (Wave 3 + Wave 4) — NÃO em escopo desta migração

Wave 2 adopta **Nível 1** do isolamento (tool-crate hospeda UI fisicamente). Próximos
níveis ficam como plano futuro, ativados se Wave 2 mostrar limitação concreta.

### Wave 3 — Plugin trait formal (futuro)

**Quando ativar:** se acoplamento entre tool-crate e ph2d-editor virar fricção real
(ex.: tool-crate precisa mexer em campo público interno de `HeroScreen`).

**Escopo:**
- Definir `pub trait EditorToolPlugin` em `ph2d-tool-registry`:
  ```rust
  pub trait EditorToolPlugin {
      fn build_panel(&mut self, ctx: &mut PanelCtx) -> FloatingPanel;
      fn handle_panel_event(&mut self, event: PanelEvent, ctx: &mut PluginCtx);
      fn render_canvas_overlay(&self, scene: &mut VectorScene, ctx: &OverlayCtx);
      fn snapshot_state(&self) -> Vec<u8>;
      fn restore_state(&mut self, bytes: &[u8]);
  }
  ```
- Manifest passa a aceitar `plugin_factory: fn() -> Box<dyn EditorToolPlugin>`.
- `PanelCtx` / `PluginCtx` / `OverlayCtx` são structs leves que ph2d-editor passa
  para o plugin — APIs explícitas em vez de acoplamento via campos públicos.
- Permite hot-reload de tool no futuro (snapshot/restore primitivos prontos).

**Custo:** definição do trait + refactor cross-crate dos pontos onde editor consome
state da tool. ~1-2 sessões.

### Wave 4 — Toolkit + Features-as-crates (futuro condicional)

**Quando ativar:** se compile time de tool-crate ficar dolorosamente alto (mudança
trivial em `hero.rs` recompila grid-snap/bgremoval). Medir via `cargo timings` antes
de comprometer.

**Escopo:**
- Extrair `ph2d-editor-toolkit` (widgets + paint + FloatingPanel + interaction —
  ~5000 LOC) para crate intermediário separado.
- Tool-crates passam a depender de `ph2d-editor-toolkit` em vez de `ph2d-editor`
  inteiro.
- `ph2d-editor` encolhe para "app shell" (chrome + hero + composição) ~2000 LOC.
- Features não-tool viram crates próprios: `ph2d-feature-inspector`,
  `ph2d-feature-hierarchy`, `ph2d-feature-asset-browser` (futuro).
- Cada feature implementa `EditorFeature` trait similar ao `EditorToolPlugin`.

**Custo:** extração massiva ~30 arquivos cross-crate. ~5-7 sessões.

### Por que Wave 2 NÃO faz Wave 3/4

Premature optimization. Wave 2 já é ambicioso (23-24 sessões). Wave 3/4 só fazem
sentido se Wave 2 falhar em algum aspecto mensurável:
- Compile time dolorosamente alto → Wave 4.
- Acoplamento via campos públicos virou bug → Wave 3.

Sem dados empíricos, Wave 3/4 são over-engineering.

---

## Pós-Wave-2: estado ideal observável

Periférico novo chega no projeto. Quer criar **tool stateful de Filter Blur com painel**.
Sequência exata pós-Wave-2:

1. Escreve `docs/design/tools/filter_blur.toml` (15 LOC).
2. Cria SVG `docs/design/icons/filter-blur.svg` (Lucide-derived ou novo glyph).
3. Cria `crates/ph2d-tool-filter-blur/` 100% isolado:
   ```
   crates/ph2d-tool-filter-blur/
     Cargo.toml          deps: ph2d-tool-registry + ph2d-editor (para widgets)
     src/
       lib.rs            MANIFEST const + register fn
       panel.rs          paint do painel (consome widgets de ph2d_editor::widget)
       state.rs          FilterBlurState (params: radius, sigma, mode)
       algorithm.rs      lógica pura (separable Gaussian)
       icon.rs           BezPath gerado em build.rs de SVG (PR 11.2 cobre)
       handler.rs        on_click + handle_panel_event
     tests/
       algorithm.rs      testes unitários puros
       manifest.rs       smoke MANIFEST válido
   ```
4. **Periférico jamais abre qualquer arquivo em `crates/ph2d-editor/`** durante o
   trabalho. Toda a UI vive na pasta da tool. State da tool vive na pasta da tool.
5. Crate compila isoladamente; CI cross-source valida `.toml` ↔ MANIFEST,
   `.svg` ↔ icon_fn.
6. Periférico reporta "pronto" ao Enio.
7. Coordenador adiciona linha em `register_init::register_all` (~30 segundos de
   trabalho — única edição central).
8. Workspace recompila; chrome derivado (PR 11.4) mostra Filter Blur na TopBar
   automaticamente; golden image baseline gerada para o ícone.
9. Smoke visual: editor abre, click em Filter Blur → painel aparece com widgets
   declarados em `panel.rs`; handler dispatched via Action Bus.
10. Integração completa em < 5 minutos de Coordenador. **Zero colisão possível com
    outros agentes** (4 Periféricos podem criar tools simultâneas sem nunca tocar
    o mesmo arquivo).

**Compare com pré-Wave-1**: ~30 minutos de Coordenador editando 6-8 arquivos centrais,
~180 LOC de INTEGRATION.md guidance, alto risco de merge conflict com slots paralelos.

**Compare com pós-Wave-1**: ~5 minutos de Coordenador (Wave 1 já entregou tool-crate +
register_all). Mas ainda colisão se 2 Periféricos criam widgets simultaneamente
(widget.rs). Source-of-truth não rígida.

**Pós-Wave-2**: ~30 segundos de Coordenador (uma linha). Zero colisão possível. Source-
of-truth rígida com CI hard-fail em divergência.

---

## Decisão necessária do Enio antes de iniciar Wave 2

Aprovação implícita pelo escopo "100% sem colisão" + decisões fechadas neste plano:
- Quebra clean de NodeId numéricos (legacy values discarded).
- Quebra de API pub use em lib.rs (pre-1.0 OK).
- Golden image baselines committed direto (~300KB, sem git-lfs).
- TOML schema com `*_inline` + `fluent_key` (HR-15 deferred).
- ADR-0028 escopo conforme §11.

Aprovação explícita necessária:
- **Próxima ação**: registrar Wave 2 ativa em STATE.md + iniciar PR 11.1.
