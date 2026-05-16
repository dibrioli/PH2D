# ADR-0027: Convention-by-discovery + Shell decomposition + HR-18

**Status:** Accepted
**Data:** 2026-05-16
**Decisor(es):** Enio + LLM Coordenador + 2 pareceres independentes
(registrados em sessão de chat 2026-05-16)

## Contexto

Convention-by-edit em registries centrais — `crates/ph2d-editor/src/lib.rs`,
`tools/mod.rs`, `widget.rs`, `icons.rs` (enum 89 variants), e
`screens/hero/fixture.rs` (`topbar_clusters()` hard-coded) +
`shells/desktop/src/main.rs` (3463 LOC, 19 drains `pending_X` inline
em `render_frame`, arms gigantes em `window_event`) — serializa
multi-agente no Coordenador único autorizado a editar arquivos
compartilhados. STATE.md ledger mostrou 3 slots Periféricos `working`
simultâneos competindo pela mesma fila de integração serial.
Adicionalmente:

1. **`INTEGRATION.md` mecânico**: cada Periférico entrega documento
   de ~180 linhas descrevendo edições mecânicas em N arquivos
   centrais. Exemplo: `tools/make_square/INTEGRATION.md` com 181
   linhas instruindo "edite tools/mod.rs adicionando `pub mod
   make_square`; adicione `IconId::MakeSquare` em icons.rs com SVG
   path M3,3v18H21…; adicione item ao cluster Image Tools no
   topbar_clusters() de fixture.rs; aloque NodeId no range 100..199;
   wire on_make_square_clicked em main.rs". Trabalho que deveria ser
   inexistente, não documentado.

2. **`pending_X` proliferation**: `main.rs::render_frame` continha 20
   drains do tipo `if let Some(_) = hero.pending_X.take() { ... }`,
   crescimento linear no número de features. ~1300 LOC de drains
   inline.

3. **`enum IconId` centralizado**: 89 variants, cada tool nova
   adiciona variant + match arm + entry em `ALL_ICONS`. Conflito de
   merge previsível.

4. **`NodeId` ranges manuais**: docs alocam 100..199 TopBar,
   200..299 LeftRail, etc. Cada const novo = consultar SKILL +
   STATE.md + torcer pra outro agente não ter pego o mesmo número.
   Anti-LLM.

5. **`shells/desktop/src/main.rs` god-file**: 3463 LOC, com
   `render_frame()` de 1825 LOC e `window_event()` de 706 LOC,
   ambos crescendo monotonicamente. Cada feature adiciona 30-100
   LOC em main.rs.

Esses fatores produzem fricção operacional pesada: dois pareceres
independentes (registrados em sessão de design 2026-05-16) confirmaram
convergência sobre o diagnóstico e o caminho de correção.

## Decisão

1. **Tool-as-crate**: cada tool de editor vira crate próprio em
   `crates/ph2d-tool-<slug>/`. Cada crate exporta `pub const
   MANIFEST: ToolManifest` + `pub fn register(reg: &mut Registry)`.
   Crates novos não tocam `ph2d-editor` exceto pelo dep direction —
   tool-crates podem depender de `ph2d-editor` (para reusar widgets,
   FloatingPanel, paint), mas `ph2d-editor` NÃO depende de tool
   crates. A direção do dep grafo evita o ciclo Cargo.

2. **`ph2d-tool-registry` crate dedicado**: hospeda `ToolManifest`,
   `Registry`, `Zone`, `NodeId` hash (FNV-1a 64-bit `const fn`),
   `IconHandle`, `ActionInvocation`. Sem deps em `ph2d-editor`. Tool
   crates dependem deste; `ph2d-editor` re-exporta como
   `ph2d_editor::registry` para preservar API.

3. **`ph2d-tool-registry-init` crate dedicado**: contém `register_all()`
   append-only e os CI lints (HR-13 budget aggregate, HR-15 i18n
   keys, HR-12 a11y role allowlist, HR-7 release-game symbol absence
   skeleton). Depende de TODOS os tool-crates + `ph2d-tool-registry`.
   Shells dependem deste crate; `ph2d-editor` NÃO depende dele —
   quebra o ciclo final.

4. **`linkme`/`inventory` rejeitado**: dois pareceres independentes
   confirmaram que `distributed_slice` em wasm32 perde manifests
   silenciosamente (custom sections strippadas por wasm-bindgen +
   bundlers) e que ordem de iteração é não-determinística
   cross-linker (ld64 vs lld vs link.exe vs wasm-ld), violando o
   espírito HR-5. Adotado **híbrido conservador**: registro
   explícito via `pub fn register_all()` em arquivo append-only.
   Coordenador edita 1 linha por integração; merge 3-way trivial.

5. **NodeId por hash estável**: `hash_node_id("tool.<slug>")` via
   FNV-1a 64-bit em `const fn` (algoritmo trivialmente determinístico
   cross-platform; HR-5 safe). Colisão detectada em
   `Registry::build()` com mensagem clara — astronomicamente rara
   para os 200 ids típicos de um editor maduro mas defendida.

6. **`IconHandle(&'static str)`** substitui `enum IconId`. Ícones
   moram dentro do manifest da tool (`icon_fn: fn() -> BezPath`),
   não em registry central — Agente 1 refino sobre proposta
   original. PR 10 não removeu o enum legacy (consumido por chrome
   fixo); migração gradual mantida.

7. **Shell decomposition**: `shells/desktop/src/main.rs` decomposto em:
   - `init.rs` (boot pipeline, era `resumed()` 260 LOC inline)
   - `input_dispatch.rs` (per-arm `WindowEvent` methods, era
     `window_event()` 706 LOC)
   - `hero_intents.rs` (6 drains de Inspector/Tool Action extraídos
     como fns livres, parte dos ~1300 LOC de `render_frame()`)
   `main.rs` encolhe de 3463 → 2421 LOC (-30%). `resumed()` 260 → 17
   LOC; `window_event()` 706 → 28 LOC.

8. **HR-18 declarada** (sem CI gate ativo neste PR; ativação fica
   para refactor futuro quando `main.rs` baixar para o cap real):
   caps 600/200/400 LOC (arquivo/função/main.rs) em `shells/*/src/`.
   Vide §6.1 do plano `docs/Migracao/2026-05-convention-by-discovery.md`.

## Consequências

### Positivas

- 4 Periféricos paralelos sem colisão. Cada um trabalha em
  `crates/ph2d-tool-<slug>/` 100% isolado.
- Coordenador deixa de ser gargalo serial — vira revisor de manifest
  + edita 1 linha em `register_all` por integração.
- `INTEGRATION.md` mecânicos extinguem (manifest declarativo
  substitui).
- `main.rs` `growth-bounded`: features novas vão para módulos
  pertinentes (init/input_dispatch/hero_intents), não inflam o
  god-file.
- CI lints (HR-12/13/15) garantem coerência cross-source:
  manifest declara budget+key+role; CI valida.
- Registry runtime construído no boot — esqueleto pronto para PR 9
  (dispatcher genérico full) substituir `pending_X` proliferation.

### Negativas

- Workspace cresceu de 24 → 29 crates. Cargo paraleliza bem com
  mais crates, mas cargo-deny audit + Cargo.lock crescem
  trivialmente.
- Tools migradas em **shape "manifest thin"**: o conteúdo
  (implementação) permanece em `ph2d-editor::tools::<slug>` ou
  `ph2d-editor::grid_snap` para tools heavily integrated com
  `screens/hero.rs`. Migração total do conteúdo (move físico de
  5481 LOC do grid_snap) fica para refactor futuro — exigiria
  também refatorar `HeroScreen` para extrair `GridSnapState`.
- HR-18 CI gate inativo: `main.rs` em 2421 LOC excede o cap 400.
  Cap só liga quando PR 9 (dispatcher genérico full) e cleanup
  posterior baixarem `main.rs` para o cap real.

### Neutras

- Dispatch dinâmico em click path (não hot — HR-3 preservado via
  dhat bench em `tests/budget/no_alloc_hot_path.rs` que continua
  passando).
- `enum IconId` legacy permanece para consumers de chrome fixo
  (Save, Open, Settings, Play, Layers, Palette, GridSettings,
  Image, Eraser, MakeSquare, TrimTransparency, BgRemoval). Não
  removido — migração gradual.

## Alternativas consideradas

- **`linkme` distributed_slice**: rejeitado (wasm32 + iOS bitcode +
  MSVC LTO fragilidade; ordem não-determinística cross-linker).
  Confirmado por dois pareceres independentes em sessão de design.
- **`build.rs` codegen**: viável tecnicamente mas perde
  transparência (codegen escondido em `OUT_DIR` atrapalha LLM
  lendo o repo; §18#8 do SKILL favorece compreensibilidade por LLM).
- **`bevy_ecs::App::add_plugins` piggyback**: rejeitado (força ECS
  plugin para papel de UI registry; viola separação ADR-0021
  Sim/Present).
- **Tool-as-subpasta dentro de `crates/ph2d-editor/src/tools/`**
  (status quo): parcial; mantém `Cargo.toml` + `icons.rs` +
  `mod.rs` como pontos de colisão. Não resolve multi-agente.
- **Toolkit crate extraído** (`ph2d-editor-toolkit` com widgets/paint):
  refactor maior; rejeitado para esta migração (extrai ~30 arquivos
  de widget/), defere para se justificar empiricamente.

## Trabalho remanescente (post-PR-10)

- **PR 9 full**: substituir os 14 `pending_X` drains restantes em
  `render_frame()` (Inspector intents que não foram extraídos para
  `hero_intents.rs` no PR 9a — duplicate, add_child, delete,
  rename_seed, rename_commit, visibility_edit, transform_edit,
  name_edit, sprite_source_change, etc.) por dispatcher genérico
  via `Registry::action_for`.
- **Migração full do conteúdo grid_snap + bgremoval** para seus
  tool-crates (atualmente `manifest thin`). Exige extrair
  `GridSnapState` de `HeroScreen` ou abstrair via trait.
- **HR-18 CI gate**: ativar quando `main.rs` cair abaixo do cap
  400 (estimado após PR 9 full + extração de hero_intents
  remanescentes).
- **`enum IconId` removal**: substituir consumers de chrome fixo
  por `IconHandle` ou re-export por path (`ph2d_tool_<slug>::icon`).
- **MCP exposure**: campo `McpExposure` reservado em `ToolManifest`;
  wiring real (auto-expose para `ph2d-mcp::tools`, audit log,
  destructive token gating per HR-11) é sub-projeto separado.

## Referências

- **Plano canônico**: `docs/Migracao/2026-05-convention-by-discovery.md`
  (versão 1.0, 2026-05-16) — diagnóstico completo, plano de 13 PRs,
  shape canônico de tool-crate (Apêndice A), HR-18 lint test
  skeleton (Apêndice H).
- **Pareceres independentes**: registrados em sessão de design
  2026-05-16 (Agente 1 e Agente 2). Convergência forte: 6 de 7
  componentes da proposta sustentados; única decisão divergente
  (cortar `linkme`, usar híbrido conservador) adotada.
- **PRs implementados**: 1 (Foundation), 2 (Helpers), 3 (CI lint
  stack), 4 (piloto make-square), 6.0 (retrofit registry-init),
  6 (grid-snap manifest thin), 7 (bgremoval manifest thin),
  8 (Registry runtime wired), 9a (hero_intents.rs extraído),
  9b (input_dispatch.rs extraído), 9c (init.rs extraído).
