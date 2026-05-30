═══════════════════════════════════════════════════════════════════
HANDOFF — Sprite Inspector v2 · agente SOLO (Coordenador **e** Implementador)
Autor: agente anterior (solo Coord+Impl) · Data: 2026-05-29
═══════════════════════════════════════════════════════════════════

VOCÊ É: o ÚNICO agente. Acumula os dois papéis (Coordenador + Implementador).
Trabalha **sozinho, sem medo de colisão** — não há outro agente na máquina.
Você implementa, audita, conserta, commita, faz ship, baby-sita o CI, e só para
para o smoke visual do Enio **no fim de tudo**.

───────────────────────────────────────────────────────────────────
§0 — MANDATO (governa cada decisão; memorize)
───────────────────────────────────────────────────────────────────
> **Sempre o melhor possível. Sempre o padrão-ouro. A todo custo, sem medo.**

- Dúvida que você consegue resolver sozinho → resolva pela régua acima (a forma
  mais correta/completa/definitiva vence custo de build, cronograma e conforto).
  NÃO pare pra perguntar o que o mandato já responde. Pare só pro **smoke visual**
  (no fim) ou se faltar uma decisão de PRODUTO que só o Enio tem (raro).
- **Proibido:** corner-cut disfarçado de v1; `unwrap`/falha silenciosa onde cabe
  `Result`; assumir paridade/correção sem prova; `TODO: depois` em coisa que dá
  pra fazer certo agora; afirmação técnica/industrial em ADR sem verificar
  (`cargo search`/grep/`ls`/conta) — [[feedback-no-industrial-claims-without-verification]].
- Determinismo (HR-5) onde aplica; contratos minúsculos+gateados; toda superfície
  pública documentada; testes cobrem feliz+edge+a-classe-de-bug.

───────────────────────────────────────────────────────────────────
§1 — FONTE DA VERDADE DA UI (regra dura — zero alucinação)
───────────────────────────────────────────────────────────────────
A UI do Sprite Inspector **só** pode usar o que JÁ EXISTE e é demonstrável:

1. **O próprio Inspector** ([`crates/ph2d-panel-inspector/`](../crates/ph2d-panel-inspector/))
   — o painel real, rodando. É o destino do trabalho e o espelho da verdade.
2. **A Widget Gallery** ([`crates/ph2d-panel-widget-gallery/`](../crates/ph2d-panel-widget-gallery/))
   + o showcase de widgets em `ph2d-editor-core` (gate
   `architecture_widget_showcase_coverage`: todo widget aparece ou opta-out
   explicitamente) — o catálogo canônico de widgets que existem de verdade.

REGRAS:
- **NÃO invente widget novo na cabeça.** Toda seção/controle do Inspector é
  composta de widgets que estão na Gallery/showcase. Precisa de um widget que não
  existe? Então ele é um **deliverable do plano** (crie-o no widget layer + registre
  no showcase + gate) ANTES de usá-lo no Inspector — nunca um desenho imaginado.
- **Zero hex, zero `f32` literal de UI, zero string hardcoded** — tudo via
  tokens (`ph2d-tokens`) / i18n (HR-15). Strings de UI **sempre em inglês**, mesmo
  que o Enio descreva em pt-BR — [[feedback-app-ui-english-only]] (comentário de
  código pode ser pt-BR; UI não).
- Magic-number em widget/screen? gate `no_magic_numeric` rejeita → use token ou
  `// LITERAL-PX-OK: <razão genuína>`. Glifo fora da Inter (→, etc.) → gate
  `no_tofu_glyphs` → use ASCII (`->`) ou `·` (U+00B7).
- Botão novo num painel typed exige register no `populate.rs` do crate do painel +
  hit_index — senão o dispatcher dropa o clique em silêncio
  ([[feedback-panel-populate-register]], [[feedback-hier-companion-dispatch-allowlist]]).

───────────────────────────────────────────────────────────────────
§2 — O LOOP (fase a fase, sem parar até o plano fechar)
───────────────────────────────────────────────────────────────────
Para CADA task/fase do plano, em ordem, **sem parar**:

  1. **Implemente no padrão-ouro** (§0). UI só com widgets reais (§1).
  2. **Auto-verifique** (inner loop rápido): `cargo check -p <crate>` no slot warm.
     (test/clippy completos ficam pro fechamento da fase, não por task — §6.)
  3. **AUDITORIA DUPLA COMPLETA** (a cada fase, NÃO por task):
     ≥2 auditores adversariais **em paralelo**, **lentes rotacionadas**
     ([[feedback-audit-lens-diversity]]): A escopo/spec-fidelity · B ABI/grep ·
     C determinism/HR-5 · D UX/a11y/i18n · E security/perf/coverage. Duros,
     caçando bug/lacuna/alucinação-de-UI, dando severidade, **sem validar por
     cortesia**. Audite sobre o **diff acumulado da fase**.
  4. **CONSERTE TODOS os achados** (Crítico→Baixo). **Nada adiado**
     ([[feedback-perfection-no-deferrals]]). **RE-AUDITE até erro-zero.**
  5. **Commit** escopado em background: `git commit --no-verify -m "msg" -- <paths>`.
     (Você é solo — sem medo; ainda assim commits limpos por fase ajudam o histórico.)
  6. **Próxima fase.** Volte ao 1. **Não pare** entre fases — o plano roda inteiro.

  AO FIM DE TODO O PLANO (Sprite + painel + todos os features novos):
  7. **SHIP**: `./scripts/ship.sh` (paridade-CI: fmt, clippy --all-targets+features,
     machete, deny, audit, typos, nextest --workspace ci-test). Conserte todo `✗`.
  8. **PUSH + baby-sit do CI** até verde (matriz Linux/macOS/Windows + replay + bench).
     Vermelho → diagnostique + conserte + re-push, em loop.
  9. **SÓ ENTÃO**: pare e peça o **smoke visual** ao Enio. É o ÚNICO ponto de parada.

───────────────────────────────────────────────────────────────────
§3 — ESTADO ATUAL (não refaça; construa em cima)
───────────────────────────────────────────────────────────────────
- **W2 EM ANDAMENTO — Transform skew vertical FECHADO (2 commits locais, não pushados):**
  - **T2.2 foundational** (`ph2d-ecs`): ADR-0025-amendment-1. `Transform` v2 (skew_x/y,
    28B cap), `compose`/`from_transform` R·Sk·S via `libm::tanf` (skew=0 ⇒ bit-idêntico
    a v1, golden hash preservado). `transform_versioned.rs` + 3 fixtures v1 (21B) +
    skew golden + 9 contratos + 3-level cascade smoke. **Auditado 2-lentes** (ABI +
    determinism): gold-standard. Gap documentado: wrapper é máquina canônica mas
    ComponentRegistry/cooker serializam Transform BARE (greenfield, simétrico ao Sprite
    FROZEN) — promover = hook `VersionedComponent` cross-cutting = ADR separado.
  - **T2.3 UI** (`90227b5`): Skew X/Y editável no Inspector Transform (graus), wire
    end-to-end (snapshot→sync→paint→event→commit-clamp). editor-core 568 verde.
  - **PRÓXIMO (pré-requisito p/ T2.4+):** NÃO existe path Inspector→Sprite edit genérico
    (só `InspectorSpriteSourceChange` + display read-only). T2.4/T2.5/T2.6/T2.8 precisam
    de um `InspectorSpriteEdit`/`SetComponent::<Sprite>` (análogo ao commit Transform em
    `inspector_commits.rs`). Construir ISSO antes das seções editáveis. Depois: T2.1
    refactor sections.rs · T2.7 = **estender** BlenderColorPicker (OKLCH já existe) ·
    T2.0 BulkSelect (Checkbox::Indeterminate já existe) · GlobalTint cascade · audit W2 · smoke.
  - **Render do skew FECHADO + AUDITADO** (`025bd8c`, ADR-0070-amendment-4):
    `RenderInstance.rotation`→`basis [f32;4]`; shader aplica a base 2×2 (paralelogramo
    real, não retângulo rotacionado). Auditoria 2-lentes: álgebra correta à mão,
    no-skew bit-equivalente (zero regressão), picking inverte a base. **Carry-overs de
    skew-divergence (out-of-scope, pré-existentes, handoff aos donos):**
    F1 (LOW, cosmético/ADR-ack) gizmo `snapshots.rs build_view` ainda decompõe → caixa
    de seleção não casa com o sprite cisalhado; fix = rotear por `world_aabb_half_extents`
    + `basis_apply` (exportar de picking.rs). F2 (MED) `bgremoval_preview.rs` overlay
    ignora skew + ordem anchor×scale divergente → preview desalinha sob skew. F3 (MED,
    data-loss) `rasterize.rs`/`sprite_merge.rs` leem só scale/rotation, **não** zeram
    skew ao assar → sprite assado fica double-sheared. Nenhum é regressão do fix; o fix
    apenas tornou a divergência visível.
- **W1 schema-bump FECHADO + CI VERDE** (origin/main em `d15fbaa`). Commits:
  f28db39 (migrator/load_sprite) · e41bff8 (RenderInstance v4 ABI 144B, 11 attrs)
  · 51cca9d (shader §4.2 + extract + arch-gate + bench + ADR-0070-amendment-3
  flip_uv=flags). `Sprite` é v4 (20 campos); `RenderInstance` 12 campos/144B FROZEN.
- **Spec ratificada commitada** (antes estava untracked): `docs/Sprite_projeto/`
  (17 arquivos) + ADRs 0069-0074 + 0070-amendment-2/-3 + 0025-amendment-1.
- Detalhe pleno: [[project-sprite-w1-schema-bump-complete-2026-05-28]] +
  [[project-solo-coord-backlog-ship-2026-05-29]].

───────────────────────────────────────────────────────────────────
§4 — O PLANO (o que você vai executar, em loop)
───────────────────────────────────────────────────────────────────
CANÔNICO: [`docs/Sprite_projeto/15_plano_de_implementacao.md`](Sprite_projeto/15_plano_de_implementacao.md) (W1..W8).
SPEC normativa: [`docs/Sprite_projeto/`](Sprite_projeto/) (anatomia §01, components §02,
inspector-seções §03, color/tint §04, ordering §05, mask §06, anchors §07,
animation §08, sampling §09, schema §10, gates/caps §11, i18n §16).

Roadmap (do plano §, resumido — leia o plano pro detalhe de cada task):
- **W2** Inspector seções 1-6 + OKLCH (estender `BlenderColorPicker` existente,
  §4.8 — NÃO reinventar) + BulkSelect.
- **W3** seções 7-9 + 7 Components ECS + ClipChildren + sorting fixture.
- **W4** seções 10-11 + `SpriteAnimator` fixed-point (μs `u64`, sem f32 accumulator).
- **W5** seção 12 NamedAnchors + `SortedSmallVec` (sorted-by-construction) + validate.
- **W6** widgets foundation (Rect2Editor + VariantEditor) — esses são widgets NOVOS:
  crie no widget layer + showcase + gate ANTES de usar no Inspector (§1).
- **W7** polish + i18n (~155 keys) + a11y + bug bash.
- **W8** Asset Cooker Integration (Aseprite/Linked Cels/PSD) — wave separada;
  código **não existe ainda** em `tools/asset-cooker/src/`.

───────────────────────────────────────────────────────────────────
§5 — CARRY-OVERS / ITENS ABERTOS (resolva no padrão-ouro quando a fase tocar)
───────────────────────────────────────────────────────────────────
- **H-1 premult × opacity** (shader): no branch premultiplicado, `opacity<1`
  escurece alpha mas NÃO o rgb. ZERO impacto hoje (opacity default 1.0). Antes de
  opacity virar autorável (W2), reconcilie: amende `04_color_tint_canais.md §4.4`
  + `rgb * opacity` no branch premult. Flagado em `crates/ph2d-render/src/shaders/sprite.wgsl`.
- **Tint cascade de ancestrais**: hoje `RenderInstance.tint` = `self_tint × tint`
  (per-sprite). O `Π(ancestors.tint)` do §4.3 precisa de um pass **GlobalTint**
  (análogo a `propagate_transforms`) que NÃO existe — é W2 (smoke `smoke_w2_color_tint.scene`
  valida o cascade 3-níveis). Construa-o no padrão-ouro.
- **`paint_hierarchy_body`** (388 LOC) está em `FN_OVERAGE_OK` com split adiado;
  se você tocar Hierarchy paint, faça o split per-section (helpers threading `y`)
  com smoke — não deixe crescer mais.
- **CVE `jxl-grid` RUSTSEC-2026-0151**: ignorado em `deny.toml`/`.cargo/audit.toml`
  (é OOB 32-bit-only; PH2D é 64-bit puro). Decisão do Enio se quer bumpar
  `jxl-oxide`; se não mexer, mantenha ignorado.
- **rustfmt local desatualizado**: o `ship.sh` local passou fmt mas o CI rejeitou
  (rustfmt do CI é mais novo → colapsa `if let Some(S{..})` em 1 linha). Rode
  **`rustup update`** no toolchain 1.95 antes de confiar no fmt local, senão o
  CI vai te pegar (custou 1 ciclo).

───────────────────────────────────────────────────────────────────
§6 — OPERACIONAL (velocidade + ship + CI)
───────────────────────────────────────────────────────────────────
- **Slot warm (CoW)**: `bash scripts/slot-seed.sh impl-sprite` → imprime
  `CARGO_TARGET_DIR=.../target-slots/slot-impl-sprite`. **PREFIXE TODO cargo** com
  esse path (o Bash-tool NÃO persiste env entre chamadas). NÃO use o `target/` default.
- **Inner loop = só `cargo check -p`** (ou `scripts/cargo-check-narrow.sh <crate>`).
- **Fechamento de fase**: `cargo nextest run -p <crate>` + clippy `--all-targets` +
  fmt + auditoria dupla. Determinism golden: `scripts/nextest-impacted.sh` (já
  corrigido — usa `binary(transform_determinism)`, não `test(...)`).
- **HARDWARE (8 GB RAM + drive externo)**: cada `clippy --all-targets`/`nextest
  --workspace` é ~10 min com swap. Truque-chave: **`cargo clippy --workspace
  --all-targets --keep-going`** enumera TODOS os lints numa passada só, em vez de
  revelar crate-a-crate (economiza N ciclos). ≤3 cargos simultâneos.
- **Gates Sprite**: `architecture_sprite_inspector_surface` (ph2d-render:
  Sprite==20 / RenderInstance==12 / size==144) · `vertex_attr_offsets_match_struct`
  · `inspector_section_count_canonical==12` · `inspector_paint_no_alloc` (HR-3) ·
  `inspector_paint_budget_hr4_p95`. Lista: `docs/Sprite_projeto/11_arch_gates_e_caps.md`.
- **CI**: matriz em `.github/workflows/spike.yml` (Linux/macOS/Windows + C9 replay +
  bench). Link da run: `gh run list --workflow=spike.yml --limit=1`. Cooker ISPC
  flaca no macOS (vendored, retries=6 + 2 testes double-cook skipados em CI-macOS —
  não mexa, é caracterizado). Windows: dav1d só linka com o MSVC env + remoção do
  coreutils `link.exe` (já no spike.yml). NÃO regrida esses.

───────────────────────────────────────────────────────────────────
§7 — CRITÉRIO DE DECISÃO AUTÔNOMA (quando bater dúvida)
───────────────────────────────────────────────────────────────────
1. O mandato §0 resolve? → resolva você, padrão-ouro, sem medo. Não pergunte.
2. É contrato congelado (ADR-0070/0071 etc.)? → você É o Coord: escreva o
   amendment (`ADR-XXXX-amendment-N`) com verificação real, e siga.
3. É UI? → §1: só widget que existe na Gallery/showcase; se falta, crie-o como
   deliverable (widget + showcase + gate) antes de usar. Nunca invente.
4. É decisão de PRODUTO genuína (o que o usuário quer, não o que é tecnicamente
   melhor)? → ÚNICO caso de parar e perguntar. Raro.

───────────────────────────────────────────────────────────────────
§8 — REFERÊNCIAS
───────────────────────────────────────────────────────────────────
- Plano: [`Sprite_projeto/15_plano_de_implementacao.md`](Sprite_projeto/15_plano_de_implementacao.md)
- Spec: [`Sprite_projeto/`](Sprite_projeto/) · i18n: `Sprite_projeto/16_i18n_catalog.md`
- ADRs: `architecture/decisions/0069..0074` + `0070-amendment-2/-3` + `0025-amendment-1`
- Inspector: [`crates/ph2d-panel-inspector/`](../crates/ph2d-panel-inspector/) ·
  Widget Gallery: [`crates/ph2d-panel-widget-gallery/`](../crates/ph2d-panel-widget-gallery/)
- Núcleo operacional: [`CLAUDE.md`](../CLAUDE.md) · stack/HR: `SKILL_Stack_PH2D_Definitiva.md`
- Memória: índice em `MEMORY.md` (leia antes de agir).

**Confiança:** W1 fechado, spec versionada, CI 100% verde na matriz inteira. O
substrato (schema v4 + ABI + shader + gates) está pronto. Daqui é construir o
Inspector e os features W2→W8 em loop, no padrão-ouro, com a UI ancorada no que
existe de verdade. Boa — sem medo.
═══════════════════════════════════════════════════════════════════
