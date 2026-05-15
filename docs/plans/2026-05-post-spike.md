# Plano operacional pós-spike — implementação real do core

**Status:** In progress (M1-M12 done; M13 active)
**Data abertura:** 2026-05-08 (logo após merge do PR #1, spike fechado)
**Última revisão:** 2026-05-11
**Owner:** Enio Oliveira Dias Brito
**Implementador:** Claude Opus 4.7 (1M context)
**Referências:** [ADR-0019](../architecture/decisions/0019-spike-scripting-output.md) (spike output), [ADR-0003-rev2](../architecture/decisions/0003-ecs-choice.md) (ECS), [ADR-0020](../architecture/decisions/0020-surface-lifecycle.md) (surface lifecycle), [ADR-0021](../architecture/decisions/0021-simulation-presentation-boundary.md) (sim/present), [ADR-0022](../architecture/decisions/0022-no-hashmap-in-simulation.md) (HashMap ban), [ADR-0023](../architecture/decisions/0023-ui-ux-baseline.md) (UI/UX baseline).

## Status em 2026-05-09

| Marco | Status | PR / commit | Notas |
|---|---|---|---|
| M1 — Platform host | ✅ Done | PR #2 | `shells/desktop` abre janela winit; ph2d-host trait |
| M2 — Math/time/budget | ✅ Done | PR #3 | ph2d-core completo (FixedStep + MemoryBudget) |
| M3 — wgpu surface | ✅ Done | PR #4 | ADR-0020 implementada (`SurfaceContext::acquire_frame`) |
| M4 — ECS sim/present | ✅ Done | PR #5 | ADR-0021 enforced via `SimComponent`/`PresentComponent` traits |
| M5 — Sprite renderer | ✅ Done | PR #7 + #21 | 1000-sprite demo verde; perf valida 100k @ 60Hz Mac M-series |
| M6 — Asset hot reload | ✅ Done | PR #8 | blake3 content-addressed + off-thread reload |
| M7 — Luau ScriptHost | ✅ Done | PR #9 + #10 | `ph2d.set/get`, sandbox + reset+restore (Defold-style) |
| M8 — Input + gamepad | ✅ Done | PR #13 | gilrs adapter + Pencil stub + first prod bench em CI |
| M9 — MCP + bindgen | ✅ Done | PR #16 | HR-10 paridade enforced via `ph2d-bindgen --check` em CI |
| M10 — Physics determinístico | ✅ Done | PR #18 | Rapier 0.28 + enhanced-determinism + cross-OS hash check |
| M11 — Vector + text | ✅ Done | PR #20 + #23 | ph2d-vector (Vello 0.8) + ph2d-text (parley 0.6); widget paint pass |
| M12 — Editor base + a11y | ✅ Done | PR #19 + #25-#28 | ph2d-tokens + ph2d-a11y + ph2d-editor (4 zonas + FloatingPanel + ZenMode + ToastQueue + ToolRegistry); BrushTool + MoveTool wired |
| **M13 — Polish + features** | ✅ **Core UI funcional concluído (2026-05-11)** | branches `m13/design-library`, `m13/tool-palette-ui` | Round 1 (até 2026-05-09): biblioteca canônica de **32 widgets**, BlenderColorPicker, hero+input pipeline, deep polish, `PH2D_THEME` funcional. Round 2 (2026-05-09→11, vide [`docs/UI_Bugs/README.md §10`](../UI_Bugs/README.md) + [`AUDIT_M13.md`](../UI_Bugs/AUDIT_M13.md)): IME PT-BR no winit, caret-follows-space via `TextSystem::prefix_width`, painéis Inspector+Hierarchy MÓVEIS (drag pill) + REDIMENSIONÁVEIS (resize gripper) + auto-shrink no viewport bottom, modelo de drag incremental (sem rubber-band), Hierarchy DnD com drop-inside parenting + indent visual + x-aware target, `radius_scale` consumido via thread-local em paint, Cmd+C/V/X via `arboard`, chrome de painel centralizado em `style::panel_*_handle_rect`/`paint_panel_corner_dot`. Snapshot congelado `screens/hero_ref/` ativado via cargo feature `reference-snapshot`; launchers `play.command` (working) + `reference.command` (baseline) — **AMBOS aposentados em 2026-05-14**, substituídos pelo painel **Widget Gallery** embutido no app principal (paleta no TopBar; vide [`docs/UI_Bugs/README.md §10.17`](../UI_Bugs/README.md)). **435 testes verdes; clippy + fmt + deny + audit + bindgen + cross-OS replay hash todos PASSING**. Próximo: projeto-piloto que exercite o editor com cena real (M14+). |

**Updates do SKILL aplicáveis aplicados:** §9/§10/§11/§12/§15 atualizados conforme item "Updates aplicáveis ao SKILL" abaixo (ADR-0020/21/22/23 todas referenciadas inline).

## Princípio de organização

**Cada marco fecha um loop testável.** Não popular crate sem ter como provar que funciona end-to-end (mesmo que end-to-end seja "abre janela e desenha quad colorido"). Marcos posteriores agregam capacidade aos loops fechados, nunca exigem refactor de loops já fechados.

Todos os marcos são **incrementais e mergeáveis** — branch curta (≤ 1 semana), PR auto-revisado, gate de aceitação concreto. Critério de PASS sempre tem (a) build verde em CI matrix Linux+Mac+Windows, (b) fixture executável, (c) HRs explicitamente validadas.

## Tabela de marcos (M1..M13)

| # | Marco | Crate(s) alvo | HRs validadas | Gate de aceitação | Tempo estim. | Depende de |
|---|---|---|---|---|---|---|
| **M1** | Platform host + FFI shape mínima | `ph2d-host`, `shells/desktop` (novo) | HR-1 | `cargo run -p ph2d-host-desktop` abre janela winit, processa close, reporta dimensões via PlatformHost trait. Sem render ainda. | 4-6 dias | — |
| **M2** | Math, time, fixed-step accumulator, MemoryBudget aggregator | `ph2d-core` | HR-3, HR-4, HR-13 | Fixed-step loop a 60Hz com accumulator estável (drift ≤ 1ms em 10s). MemoryBudget::sum_against_platform_max passa em iPad-target (1000MB). | 3-4 dias | M1 |
| **M3** | wgpu Instance/Device/Queue + Surface lifecycle (ADR-0020) | `ph2d-gpu` (sem `interop/` ainda — só safe surface) | HR-1, HR-13 | Janela com clear color animado a 60Hz por 60s sem leak (RSS estável). Recovery testada via fixture mock para todas as variants de SurfaceError (ADR-0020). | 6-8 dias | M2 |
| **M4** | ECS wrapper + SimWorld/PresentWorld + extract! macro (ADR-0021) | `ph2d-ecs` | HR-1, HR-5, HR-7 | 200 entities Position+Velocity em SimWorld; sistema de movement; extract para PresentWorld; render lê PresentWorld read-only. Compile-time error se sistema de render escreve em SimWorld. | 5-7 dias | M3 |
| **M5** | Render esqueleto: clear → quad → spritebatch com atlas | `ph2d-render` | HR-3, HR-4 | 1000 sprites animados a 60Hz no Mac M-series, frame budget < 3.5ms. Bind group hierarchy `frame/material/draw` per toji.dev. PipelineLayout explícito (não `auto`). | 7-10 dias | M4 |
| **M6** | AssetDb sync + blake3 hash + hot reload off-thread | `ph2d-asset` | HR-6 | Fixture: 100 PNG (blake3 ids) → carregar → mostrar; modificar 1 arquivo no disk → reload off-thread → swap atomic em `<frame>` boundary. Path renaming não invalida handles. | 5-7 dias | M5 |
| **M7** | Luau runtime integrado ao tick + ph2d.* API canônica | `ph2d-script` (popular sobre o spike skeleton) | HR-8, HR-9, HR-16 | Script Luau controlando 100 entities (Position via ph2d.set), `Lua::sandbox(true)` ativo, hot reload via reset+restore funcional (medido em C4 do spike). GC step pause ≤ 0.01ms p99. | 5-7 dias | M4, M6 |
| **M8** | Input + Apple Pencil abstrações | `ph2d-input` (novo populated), shells/desktop ganha gamepad | HR-1 | Gamepad button press via gilrs (desktop) → ph2d.input table acessível em Luau. Pencil squeeze/double-tap stub (sem iPad shell ainda). Frame budget bench rodando em CI pela primeira vez. | 4-5 dias | M5, M7 |
| **M9** | MCP server real conectado ao SimWorld + ph2d-bindgen | `ph2d-mcp` (popular sobre spike skeleton), `tools/ph2d-bindgen` (novo) | HR-8, HR-10, HR-11 | `cargo run -p ph2d-mcp-server` aceita JSON-RPC; 5 tools de c6-prompts.md operacionais sobre SimWorld real. ph2d-bindgen gera `runtime/luau/ph2d.d.luau` + `mcp/schema.json` a partir das mesmas `#[lua_export]` annotations (HR-10 paridade enforced em CI). | 7-10 dias | M7 |
| **M10** | Physics — Rapier wrapper + det-physics feature ON desde dia 1 | `ph2d-physics` | HR-5 | Fixture lockstep: 50 corpos rígidos em colisão, fixed-step 60Hz, `enhanced-determinism` ativo. Hash idêntico cross-OS em Linux+Mac+Windows (estende C9). Rapier `parallel`/`simd-stable` desligados (incompatíveis com determinism). | 6-8 dias | M4, CI matrix |
| **M11** | Vetorial + texto (Vello + parley + harfrust + skrifa) | `ph2d-vector`, `ph2d-text` | HR-1 | Editor stub: 1 panel Vello renderiza "Hello PH2D" em parley layout com fallback CJK + emoji color. Vello em alpha — risco aceito (ADR-0004 a escrever quando aparecer regressão). | 8-12 dias | M5, M6 |
| **M12** | Editor base + a11y tree (Procreate-style canvas-first, ADR-0023) | `ph2d-editor`, `ph2d-a11y`, novo `ph2d-tokens` | HR-7, HR-12, **ADR-0023** | Editor com **4 zonas** Procreate-inspired (top-right cria, top-left edita, sidebar modula, center 100% canvas) via taffy + Vello. **`FloatingPanel` primitive** (Procreate-style draggable tool drawer) + 1 demo (Selection panel com tabs + action grid). AccessKit integrado (`ph2d-a11y`); cada widget exporta `accesskit::Node`. Mac VoiceOver + Win Narrator + iPadOS VoiceOver navegam. Modo Zen funcional (Tab toggle). 1 widget de exemplo (Button) usando tokens semânticos. WCAG 2.2 AA enforced via lint. | 10-15 dias | M11, M8, **ADR-0023** ratificada |
| **M13** | Restantes em paralelo ditados por demanda | `ph2d-light`, `ph2d-physics-soft` (CPU first), `ph2d-fluids`, `ph2d-audio`, `ph2d-net`, `ph2d-i18n`, `ph2d-save`, `ph2d-telemetry` | HR-3, HR-4, HR-5, HR-14, HR-15 | Cada um com gate próprio quando ativado. Ordem ditada por projeto-piloto que será escolhido pós-M12. | depende | M12 (loop visível completo) |

**Total estimado M1-M12:** ~10-13 semanas calendário (assumindo 1 marco por semana com folga). M13 é open-ended.

## Estado intencional ao fim de cada marco

- **Após M3:** janela abre, clear animado funciona, recovery de Lost/Outdated testada. **Loop de render fechado.**
- **Após M5:** 1000 sprites animados a 60Hz. **Loop visual fechado.**
- **Após M7:** gameplay scriptável em Luau, hot reload funciona. **Loop de gameplay fechado.**
- **Após M9:** LLM (eu próprio) consegue criar entity + atribuir component + observar via MCP. **Loop LLM-as-developer fechado.**
- **Após M12:** editor canvas-first 4-zonas Procreate-inspired (ADR-0023), AccessKit reportando para Mac VoiceOver + Win Narrator + iPadOS VoiceOver, Modo Zen funcional, tokens semânticos AA-compliant. **Engine começa a ter "cara de engine".**
- **M13+:** features incrementais sobre fundação madura.

## Anti-patterns reforçados (extensão a §15 do SKILL)

Adicionar ao SKILL §15 quando este plano for aprovado e iniciado:

- ❌ `bind_group_layout` derivado por reflection (`layout: 'auto'`) em pipeline novo. Sempre `PipelineLayoutDescriptor` explícito (M5).
- ❌ `get_current_texture()` sem matchear todas as variantes `SurfaceError`. Use `SurfaceContext::acquire_frame()` (ADR-0020) — único caminho público.
- ❌ Criar `RenderPipeline` ou `BindGroup` dentro de `RenderGraph` node execution. Só em init / on-resize. Cache agressivo em `ph2d-gpu::pipeline_cache`.
- ❌ `pairs()` em script ou `HashMap.iter()` em código que serializa state lateral / gera snapshot determinístico. ADR-0022 + HR-16.
- ❌ Acoplar API pública de qualquer crate a tipos `wgpu::*` ou `winit::*`. Único re-export legítimo é `ph2d-gpu` para `wgpu::TextureFormat` quando necessário.
- ❌ GPU compute em qualquer cálculo cujo output entra em `SimWorld` (HR-5; ADR-0021 reforça via tipo).
- ❌ Resize não-coalescido — shell desktop deve descartar resize events intermediários do mesmo frame (wgpu issues #2301/#3868/#5353).
- ❌ Async runtime no core (per HR + LLM2 insight). **Async morre na fronteira da shell.** Único async tolerado é `ph2d-asset::loader` e `ph2d-net::transport`, ambos isolados.
- ❌ `SharedArrayBuffer` assumido no web — requer COOP/COEP headers. Web target sempre tem fallback single-thread (ver §11.12 do SKILL).

## Updates aplicáveis ao SKILL após este plano ser aprovado

1. **§9 HR-5:** adicionar bullet "iteração de `std::HashMap`/`HashSet` proibida em simulation crates — ver ADR-0022".
2. **§10.5 (Shaders):** adicionar bullet "PipelineLayoutDescriptor sempre explícito; `layout: 'auto'` proibido (LLM1 audit + ADR-0020 implícito)".
3. **§11.12 (Web target):** anotar "SharedArrayBuffer requer COOP/COEP headers no servidor; sem isso, `rayon` cai para single-thread fallback".
4. **§12.2 (Concurrency):** adicionar referência a ADR-0021 (SimWorld/PresentWorld separados por tipo, não só por thread).
5. **§12.5 (Observability):** substituir bullet "se falhar 2× consecutivas, panic graceful" por link para ADR-0020.
6. **§15 (Anti-patterns):** anexar os 9 itens acima.
7. **§19 (ADR index):** adicionar ADR-0020, ADR-0021, ADR-0022, ADR-0023 à lista.

## Riscos identificados pré-execução

| Risco | Marco afetado | Mitigação |
|---|---|---|
| wgpu 30+ release durante M3 com breaking changes | M3 | Pin estrito em Cargo.toml; upgrade só via ADR. |
| Vello regression na alpha | M11 | Acompanhar issues do linebender/vello; ADR-0004 escrito ao primeiro problema. |
| Rapier `enhanced-determinism` quebrar cross-OS | M10 | C9 cross-platform CI já valida; estender com fixture rapier específica. |
| iPad shell não pronto até M11 | M11 | Aceitar — Mac é primary; iPad shell pode ser M14+. |
| Editor scope creep em M12 | M12 | Escopo agora canônicamente definido em ADR-0023 (4 zonas, AccessKit, tokens AA). QuickMenu radial + gesture-mapping editor + Single-Touch Companion overlay são M13+ (não M12). |
| AccessKit em iPadOS pode estar imaturo | M12 | Testar early; fallback parcial aceito; abrir issue upstream se necessário. |
| Procreate-inspired confundido com cópia | M12 | Iconography, marca e cores PH2D próprias; só princípios são absorvidos (ADR-0023 §"Inspirações honestas"). |
| LLM-as-dev não acompanhar o ritmo | qualquer | Marcos têm folga (1 semana cada nominal; reescopar se necessário). Plano não é deadline rígido. |

## Definition of done deste plano

- [ ] M1-M12 mergeados em `main` com CI verde matrix.
- [ ] Frame budget bench rodando em CI sem regressão > 5% por marco.
- [ ] HR-13 memory budget validado em iPad Pro M2 (quando shell iPad chegar — provavelmente M11 ou depois).
- [ ] LLM (eu) consegue criar projeto-piloto demo apenas via MCP + Luau, sem editar código Rust diretamente.
- [ ] Pelo menos 1 ADR escrito por marco que tomou decisão arquitetural (sem ADR ≠ sem decisão).
- [ ] Pós-M12: ADR de roadmap pós-v0.1 abrindo M13+ com escopo concreto.

## Próximos passos imediatos

**Concluído (2026-05-08 a 2026-05-09):**
1. ✅ Plano aprovado.
2. ✅ Updates ao SKILL aplicados ao longo dos PRs (commits `5f821ff`, `3515418`, `4e95670` para ADR-0023; outros inline).
3. ✅ M1-M12 mergeados em sequência.

**Em curso:**
1. 🟡 Aguardar output do Claude Design (tokens.json + 17 mockups + icons + specs).
2. 🟡 Iterar com Enio sobre design library até aprovação.
3. 🟡 Implementar widgets em Vello sobre `ph2d-editor` seguindo design canônico.
4. ⏳ Pós-design: escolher projeto-piloto que dite ordem de população dos crates stub do M13 (audio? net? save?).
5. ⏳ Hardening sprint pós-auditoria multi-agêntica (branch `hardening/post-audit` no remote, parallel timeline) — a decidir se mergeia, rebasea ou descarta após M13 estabilizar.

## M14.4d (retrofit M6 atlas) — shipped 2026-05-11

Após M14.4c (image import via Open menu) descobrimos que o atlas
herdado de M5 ainda era o placeholder de 4×4×64 px → toda imagem
importada perdia 99 % da resolução, era forçada a square 1:1, e
renderizava Y-invertida. O retrofit M14.4d (descritivo M14.4c+ no
código) endereçou os 3 bugs sem rebuilds maiores:

| Fase | Arquivos tocados | Resultado |
|---|---|---|
| **1. Y-flip** | [crates/ph2d-render/src/sprite.rs](../../crates/ph2d-render/src/sprite.rs) | `QUAD_STRIP` UV.v invertido pra compensar o Y-flip do `Camera2d::view_proj`. Pinned test `quad_strip_uv_compensates_camera_y_flip`. |
| **2. ProjectSettings** | [crates/ph2d-editor/src/project.rs](../../crates/ph2d-editor/src/project.rs), `hero.rs`, `shells/desktop/src/main.rs` | `pixels_per_meter` (default 100, Godot-style) thread-eado do import path; sprite world size agora derivado do source pixel × `1/px_per_m`. Per-asset override deferred. |
| **3. TopBar Settings cluster** | `topbar.rs`, `fixture.rs`, `ids.rs`, `interaction/{state,dispatch}.rs`, `context_menu_overlay.rs`, `hero.rs` | Gear icon entre Open e Project; click abre `ContextMenuKind::SettingsMenu` com 5 presets (16 / 32 / 100 / 256 / 1024 px/m). |
| **4. Skyline atlas** | [crates/ph2d-render/Cargo.toml](../../crates/ph2d-render/Cargo.toml) (dep `rect_packer = "0.2"`), [atlas.rs](../../crates/ph2d-render/src/atlas.rs) (rewrite), `renderer.rs`, `lib.rs`, `shells/desktop/src/main.rs` (extract + import + demo bootstrap) | Grid fixo substituído por `rect_packer::DensePacker` (4096²). `TextureAtlas::insert(key, w, h, rgba)` reserva região nativa; replace path detecta same-size hot-reload sem re-pack. APIs antigas (`dummy_uv`, `cell_px`, `update_cell`, `from_rgba8`, `DUMMY_*`) removidas. `BTreeMap` (HR-5/ADR-0022). |
| **5. Tests + verification** | `atlas.rs` mod tests (10 novos), `sprite.rs` (1 novo), `project.rs` (4 novos) | `cargo test --workspace`: 783 passed / 0 failed. `cargo clippy --all-targets`: clean. `cargo fmt -- --check`: clean. |

**Decisão de packer**: avaliado `texture_packer = "0.30"` vs `rect_packer = "0.2"`. Escolhido `rect_packer` — mais leve (não força Texture trait sobre nossos rgba buffers), mesmo algoritmo Skyline. Bake substituirá em V2 com algo mais sofisticado se necessário.

**Bug crítico capturado pela auditoria**: NodeIds `CTX_MENU_PPM_16..1024` (`930-934`) colidiam com `CTX_SCENE_SEARCH` (`930`) + `CTX_SCENE_ROW_0..7` (`931-938`). Movido pra range `940-944` antes do ship. Auditoria multi-agente identificou; correção em [ids.rs:220-229](../../crates/ph2d-editor/src/screens/hero/ids.rs#L220).

**Aberto pra V2 do atlas (futuro)**:
- Atlas growth strategy: re-pack + grow para 8192² quando full (hoje retorna `AtlasFull` como toast)
- `remove(key)` API para liberar slot quando sprite é despawned (hoje slot fica reservado até atlas reset)
- Per-asset `pixels_per_meter` override no Inspector (hoje só project-level)
- NumberInput livre no Settings panel (hoje só 5 presets)

## M14.5 — Pluggable sprite source strategies (in progress)

**Status 2026-05-12:**
- ✅ **A — Dynamic Atlas (Skyline)**: shipped M14.4d/4f (commits before this sprint).
- ✅ **C — Individual Textures + draw-call batching**: shipped (commit `9e6c901`).
  - `Sprite::source: SpriteSource` enum; `RenderInstance` ganha `texture_id`;
  - `IndividualTextureStore { acquire/retain/release/bind_group/replace_pixels }`;
  - Renderer sort+walk_runs+multi-draw em `SpriteRenderer::render`.
- ✅ **B — Hand-packed Atlas (loader half)**: shipped (commit `32a9404`).
  - `parse_atlas_meta` Aseprite Hash + TexturePacker JSON parsers em
    `crates/ph2d-asset/src/hand_packed.rs`;
  - `AtlasMeta { regions: BTreeMap<String, AtlasRegion>, image_size, image_filename }`;
  - Renderer integration deferred to M14.5 inspector phase.
- ⏳ **M14.5 inspector**: Render Source dropdown per sprite. Needs the
  selected-entity surface from M14.7 A (now shipped) — picks up next.
- ADR-0026 ratifies the model: `docs/architecture/decisions/0026-sprite-source-strategies.md`.



O Skyline atlas shipado em M14.4d cobre o caso amplo (muitos sprites
pequenos, auto-packing) mas não é o único pattern usado em engines
2D pro. **A meta de M14.5 é expor 3 estratégias lado a lado, com a
escolha surfaceada per-sprite no Inspector.** Cada estratégia tem
trade-off de workflow vs perf que casa com diferentes content
pipelines.

### Estratégia A — Dynamic Atlas (Skyline) — current (M14.4d)

Já implementada. Sprite é empacotado on-demand no atlas compartilhado
de 4096² via `rect_packer::DensePacker`. **Use case**: sprites
diversos importados ad-hoc, sem pipeline de art fora da engine.
**Trade-off**: 1 draw call por frame para todos os sprites; mas
packing decisions são runtime (não-determinístico cross-session se
ordem de import mudar) e atlas pode esgotar.

### Estratégia B — Hand-packed atlas (artist-authored)

Artist usa ferramenta externa (Aseprite, TexturePacker, ShoeBox,
free-texture-packer) pra produzir UM PNG + metadata JSON/XML que
descreve cada sub-sprite (`name → pixel rect`). Engine carrega o PNG
inteiro como uma textura wgpu e usa o metadata pra resolver UVs no
extract.

**Implementação prevista:**
- Asset loader: parser pra Aseprite JSON Hash format (de facto
  standard) + TexturePacker JSON. ~100 linhas em `ph2d-asset`.
- `HandPackedAtlas { texture: wgpu::Texture, regions: BTreeMap<String, AtlasRegion> }`
  paralelo ao `TextureAtlas` existente.
- Sprite component referencia atlas asset id + sprite name; extract
  resolve para UV via lookup do nome.

**Vantagens**: packing ótimo (artist controla), determinístico
cross-session, hot-reload trivial (1 arquivo), formato industry-
standard (compatível com Spine, DragonBones, etc.). **Desvantagens**:
exige tooling externo, workflow manual. **Use case**: jogo com art
pipeline definido, tile-sets, character sheets, UI icons curados.

### Estratégia C — Individual textures + draw-call batching (Godot-style)

Cada sprite tem sua própria `wgpu::Texture` em resolução nativa
(sem packing). Renderer agrupa sprites consecutivos que compartilham
textura num único `instanced draw call` (igual a `RenderingServer`
do Godot 4 — sort por texture/material, batch consecutivos).

**Implementação prevista:**
- Refator do extract: emit `(SimRef, GlobalTransform, RenderInstance, TextureRef)` em vez de só o RenderInstance; TextureRef é Arc da textura ou bind group cacheado.
- Renderer: sort instances por `TextureRef`, group consecutivos, emit 1 draw call por group com seu próprio bind group bound.
- LRU cache de bind groups por texture (rebuild eviction é raro).
- ~300 linhas em `ph2d-render` (sprite pipeline branch + bind group cache).

**Vantagens**: zero atlas-full (cada sprite vive em sua textura),
sempre full resolution, sem packing overhead runtime, mental model
simples (1 sprite = 1 texture). **Desvantagens**: mais state changes
GPU se sprites são todos distintos (mitigado por sort+batch),
overhead de alignment por textura pequena, pior para muitos sprites
diversos numa mesma cena vs atlas único. **Use case**: HD 2D
modern (Cuphead-tier), poucos sprites grandes, ou content
procedural onde packing seria mais custoso que batching.

### Inspector UI (Sprite selected)

Quando uma entity com `Sprite` component é selecionada, a sub-
painel "Render Source" do Inspector mostra:

```
┌─ Render Source ────────────────────┐
│ Strategy:  [Dynamic Atlas ▾]       │
│            Dynamic Atlas           │
│            Hand-packed Atlas       │
│            Individual Texture      │
│                                    │
│ [strategy-specific fields below]   │
└────────────────────────────────────┘
```

**Strategy-specific fields:**
- **Dynamic Atlas**: read-only `Atlas key: 16` + `Region: x=42 y=88 w=256 h=256`
- **Hand-packed Atlas**: dropdown `Atlas asset: [hud_main.json ▾]` + dropdown `Sprite name: [heart_full ▾]`
- **Individual Texture**: file picker `Source: [...] sprites/player.png`

Switching strategy reassigns the source. Atlas slot do dynamic é
liberado (precisa do `remove(key)` API listado em V2 do M14.4d).

### Implementation impact

Estimativa: ~600-900 linhas total entre `ph2d-render` (pipeline
branch + bind group cache), `ph2d-asset` (Aseprite JSON loader),
`ph2d-ecs` (Sprite enum variant), `ph2d-editor` (Inspector widget).
Justifica seu próprio marco (M14.5) em vez de retrofit. Provável
ordem: A já feita → C (Godot-style, menor unknowns) → B (loader
parser).

## M14.6 — Hierarchy panel polish (mostly shipped)

**Status 2026-05-12:**
- ✅ **A — Per-entity hide/show (eye toggle)**: shipped (commit `3e2fdf7`).
- ✅ **B — Drag-to-reparent**: shipped (commit `2519125`).
- ✅ **C — Expand/collapse**: shipped (commit `bd7eb49`).
- ✅ **D — Selection sync bidirecional**: shipped (commit `88d9849`).
  Hierarchy row click sets `gizmo_selection`; canvas pick writes
  `selection.label` so the matching row highlights.
- ✅ **E — Search / filter**: shipped (commit `a8f5e25`). `HIER_SEARCH`
  TextInput + `compute_match_filter` with ancestor-path preservation.
- ✅ **F — Right-click context menu**: shipped (commit `cb65a36`).
  Duplicate / Add Child / Reset Transform / Delete. Rename inline-
  edit deferred to a follow-up.



A Hierarchy panel hoje lista entities em ordem DFS (M14.4a) com seleção
via click, mas é read-only — não permite editar a hierarquia nem
controlar visibilidade. M14.6 leva o painel ao nível do que Unity /
Godot / Blender oferecem em outliners de cena.

### A. Per-entity hide/show (eye toggle)

Cada row mostra um ícone de olho (Lucide [`eye.svg`](../../crates/ph2d-editor/src/icons.rs) /
[`eye-closed.svg`](../../crates/ph2d-editor/src/icons.rs)) na coluna
da direita. Click toggle a visibilidade do sprite no canvas.

**Modelo:**
- Adicionar `Visibility` component em `ph2d-ecs` (ou flag em `Sprite`):
  ```rust
  pub struct Visibility { pub hidden: bool }
  ```
- Extract path: se `hidden == true`, skip o emit de `RenderInstance` em
  PresentWorld (sprite some do canvas sem despawn da entity)
- Children herdam por default (parent oculto → children ocultos visualmente);
  flag explícita por entity overrides (Blender-style)

**UI:**
- Adicionar `IconId::Eye` + `IconId::EyeClosed` em [icons.rs](../../crates/ph2d-editor/src/icons.rs)
  com paths Lucide. EyeClosed tem 5 path segments (rays + eye shape)
- `paint_hierarchy_row` recebe um trailing icon button no extreme-right
  alinhado com a row height
- Hit-test: novo `BlenderHit { parent: row_id, kind: VisibilityToggle }`
- Click dispatch: `apply_event` toggle `entity.Visibility.hidden`
- Visual state: opacity da row reduzida quando `hidden` (Text2 → TextDisabled)

### B. Drag-to-reparent

Drag uma row sobre outra → reparenta a entity arrastada como child da target.

**Behavior:**
- Mouse down em hierarchy row começa potencial drag (threshold 4px)
- Past threshold: render "ghost" da row arrastada flutuando no cursor
- Highlight visual da target row enquanto hovering:
  - **Sobre row**: reparenta como child (border-emph na target)
  - **Acima/abaixo de row**: insert sibling no mesmo parent (linha horizontal indicator)
  - **Sobre área vazia do panel**: detach (root-level child de Scene)
- Mouse up commits o reparent via `ChildOf` component mutation em SimWorld
- ESC cancels mid-drag

**Edge cases:**
- Drop em descendant próprio = no-op + toast `"Cannot parent X to its own descendant"`
- Drop em si próprio = no-op silencioso
- Cycle detection antes do commit

**Implementação:**
- `WidgetStore::active_drag` state machine (já existe pra Inspector, reusar pattern)
- Nova `BlenderHit { kind: HierarchyDragSource }` em cada row
- Paint do ghost: copy do row paint com α=0.6, transform=cursor
- Indicador de drop position: thin line entre rows (sibling) ou border-emph na row (child)
- ECS commit via `world.entity_mut(child).insert(ChildOf(new_parent))`

### C. Expand/collapse de subárvores

Click no chevron `>`/`v` antes do nome → toggle children visibility na lista
(sem alterar o ECS — só esconde rows). Padrão: tudo expandido inicialmente.

- `WidgetStore.hierarchy_collapsed: BTreeSet<NodeId>` — set de parents collapsed
- DFS walk skipa subtrees quando parent in `hierarchy_collapsed`
- Recompute hierarchy_order quando set muda (não é hot, OK na main thread)
- IconId::ChevronRight / ChevronDown já existem

### D. Selection sync bidirecional

Hoje click em hierarchy row dispara `HeroSelection.label` update. M14.6 adiciona:
- Canvas click em sprite (post-M14.5 picking) → seleciona row na hierarchy
- Visualmente: row selected ganha `Selection` background fill + scroll into view
- Multi-select (Cmd/Ctrl-click) — defere pra M14.7

### E. Search / filter

Search bar no header da Hierarchy (similar ao Project chip SceneList):
- TextInput vazio = mostra tudo
- Query string filtra rows cujo `Name` contém substring (case-insensitive)
- Hits highlightados em `Accent` color
- Parents de hits permanecem visíveis mesmo se não matcham

### F. Right-click context menu

Per-row context menu (right-click em row):
- "Rename…" → inline TextInput sobre o label
- "Duplicate" → spawn copy com `_copy` suffix
- "Delete" → despawn entity + descendants (com confirmação se subtree > 5 entities)
- "Reset Transform" → set Translation=0, Rotation=0, Scale=1
- "Add Child" → spawn empty entity as child

### Implementation impact

Estimativa por sub-feature:
- A (visibility toggle): ~150 linhas, ~1h
- B (drag-reparent): ~400 linhas, ~3h (state machine + indicators + cycle check)
- C (expand/collapse): ~120 linhas, ~1h
- D (selection sync): ~80 linhas, ~30 min (post-M14.5 picking dependency)
- E (search/filter): ~150 linhas, ~1h (reuse SceneList TextInput pattern)
- F (context menu): ~200 linhas, ~2h (5 menu actions, rename inline edit)

**Total: ~1100 linhas / ~8h.** Ship em sub-PRs por feature (A-F = 6 sub-PRs)
ou single PR organizada por commits per-feature.

**Dependências:**
- B (drag) precisa do M14.4e (drag-and-drop infra do host) — ou rola próprio
  state machine localmente
- D (selection sync) precisa de canvas picking (M14.5 sprite strategies tem
  parte disso quando AtlasRegion permite hit-testing por sprite)
- F (delete with subtree) precisa de cascade despawn — já existe via ChildOf
  no ECS (test `despawn_root_cascades_via_child_of` em
  [transform_hierarchy.rs](../../crates/ph2d-ecs/tests/transform_hierarchy.rs))

## M14.7 — Sprite gizmo (mouse path shipped)

**Status 2026-05-12:**
- ✅ **A — Selection state + bbox compute + world picking**: shipped
  (commit `7ef6e79`). `pick_sprite_at_world` / `selection_bbox_world` em
  `ph2d-render/src/picking.rs`; `HeroScreen.gizmo_selection: Option<u64>`;
  canvas-click picking no host.
- ✅ **B — Gizmo visual painter**: shipped (commit `1e922c3`). Bbox
  stroke + 8 handles + pivot dot + 4 rotate-hover rings em
  `ph2d-editor/src/gizmo.rs`.
- ✅ **C — Mouse hit-test + state machine + ECS write-back**: shipped
  (commit `2959e97`). `GizmoDragKind` (Translate / ScaleCorner / ScaleEdge
  / Rotate) + `compute_gizmo_transform` pure math + host Down/Move/Up
  wiring com Transform write-back.
- ✅ **D — Mouse modifier keys**: shipped (commit `130a57e`). Shift
  (AR lock + rotate snap), Ctrl/Cmd (translate snap-to-grid), Alt
  reserved for mirror-anchor.
- ✅ **F — `ProjectSettings.snap_*` config**: shipped same commit.
  `snap_move_meters` + `snap_rotate_deg` per-project; `0.0` disables
  the corresponding modifier. UI preset dropdown (Settings cluster
  customization) deferred — keeps current 0.16 m / 15° defaults
  unless a project overrides directly.
- ⏳ **E — Touch gestures (2-finger pinch/twist)**: **deferred** —
  desktop shell has no touch source; lands with iPad shell (M11+ or
  separate phase) using `PointerSource::Touch` events that the
  desktop path already routes through `PointerEvent`.
- ⏳ **G — additional tests + polish**: snap quantize, gesture-
  recognition, etc. Most G-scoped tests already landed inline with
  A-D commits (17 unit tests in `gizmo::tests` + 8 in
  `picking::tests`).



`Transform` shipped em M14.1; M14.4e spawna sprites no canvas via
drag-drop. **Falta o tool de manipulação direta** — gizmo visual sobre
o sprite selecionado pra translate/rotate/scale com mouse e touch.

### Estudo de UX (2D engines de referência)

| Engine | Modelo | Atalhos mouse | Touch |
|---|---|---|---|
| Unity 2D | Modal: Q/W/E/R/T separa ferramentas; gizmo dedicado por modo | Shift = constrain axis; Ctrl = grid snap; V = vertex snap | Limited (touch reuses mouse) |
| Godot 4 | Combined: handles de move+rotate+scale visíveis simultaneamente | Shift = AR no scale; Ctrl = snap; Alt = pivot oposto | Touch native (1 finger move, 2 fingers pinch/twist) |
| Aseprite | Selection-based: corner handles + rotation arrow | Shift = AR; Alt = pivot center | n/a |
| **Figma** | **Bbox 8-handles**: 4 corners (scale uniforme) + 4 edges (axis scale); hover outside corner = rotate | **Shift = AR; Alt = mirror anchor; Ctrl = integer % snap; Cmd = snap to grid** | Touch native (single sprite manipulation; multi-finger zoom view) |
| Affinity Designer | Figma-style, mesmas 8 handles + rotation indicator | Idem Figma | Native (iPad) |
| Spine 2D | Translation handles + dedicated rotation ring | Shift = 15° snap | n/a |
| Procreate | Touch-first: 1-finger move, 2-finger pinch/twist no objeto selecionado | n/a (touch-only) | **Best touch UX**: gesture-recognition combinada |

### Decisão pra PH2D (target: mouse desktop + iPad/tablet touch)

**Modelo: Figma/Affinity bbox combined gizmo, com gesture extension pra touch.**

Razões:
- Não-modal (todos os 3 ops visíveis simultaneamente) — economiza
  toolbar real estate e reduz cliques de modo-switch
- Padrão Figma é familiar pra design crowd, padrão Godot pra game dev
  crowd; ambos convergem em bbox+handles
- Touch funciona elegante (1-finger handle drag = mouse handle drag;
  2-finger pinch+twist no sprite = scale+rotate sem precisar handles)
- AR-preservation é Shift universal — não precisa explicar

### Arquitetura

**Visual layer (canvas overlay, painted após sprites + chrome):**
- Bbox: stroke 1.5 px em `Selection` color, rounded corners 4 px
- 8 handles: 12×12 px squares preenchidos com `Accent`, stroke 1 px
  `BorderEmph`. Cantos = scale uniforme; arestas centrais = axis-only
- Rotation: hover sobre região **outside** corner handles (raio +12 px)
  vira cursor de rotação; drag rotaciona around pivot
- Pivot indicator: dot 6 px em `Accent` no Transform.translation
  (sprite center por default; M14.5+ pode mover pivot)
- Active handle: highlight em `AccentHover` durante drag

**State machine (`GizmoState` em ph2d-editor):**
```
Idle
  → Hovering(handle) — cursor sobre handle/bbox; preview tint
    → DraggingTranslate(start_world, start_transform)
    → DraggingRotate(start_angle, start_transform)
    → DraggingScale(handle_kind, start_size, start_transform)
  → Released (commit ECS write)
```

**Mouse shortcuts:**
- **Click+drag em handle de canto** → scale uniforme (`Shift` mantém AR)
- **Click+drag em handle de aresta** → axis-only scale
- **Hover fora dos cantos + drag** → rotate
- **Click+drag em bbox interior** → translate
- **Shift + scale** → mantém AR
- **Shift + rotate** → snap 15° (configurável: `project.snap_rotate_deg`)
- **Ctrl/Cmd + translate** → snap to grid (`project.snap_move`, default
  16 px ou 1 m)
- **Alt + scale** → pivot oposto (Figma "mirror anchor")
- **Alt + rotate** → rotate around opposite anchor
- **Esc** → cancela drag, restaura transform inicial

**Touch gestures (PointerEvent::source == Touch):**
- **1-finger drag em bbox** → translate (sem snap por default; tap-hold
  ativa snap mode com haptic feedback se disponível)
- **2-finger pinch** → scale uniforme (centroid = pivot)
- **2-finger twist** → rotate (centroid = pivot, angle = relative
  angle change)
- **2-finger pinch + twist simultâneo** → composite scale+rotate
  (Procreate-style)
- Cancel: lift fingers ou tap fora do sprite

**Snap config (ProjectSettings extension):**
```rust
pub struct SnapSettings {
    pub move_meters: f32,      // default 0.16 (16 px @ 100 px/m)
    pub rotate_deg: f32,       // default 15.0
    pub scale_percentages: Vec<f32>, // [50, 75, 100, 125, 150, 200]
}
```

**Implementação dividida em sub-PRs:**
- 7.A: Selection state + Transform-coupled bbox compute (~150 linhas)
- 7.B: Gizmo visual painter (bbox + 8 handles + rotate hover ring,
  ~250 linhas)
- 7.C: Mouse hit-test + state machine + ECS write-back (~400 linhas)
- 7.D: Mouse modifier keys (Shift/Ctrl/Alt for AR/snap/anchor) (~200
  linhas)
- 7.E: Touch gestures (2-finger pinch/twist recognition) (~300 linhas)
- 7.F: ProjectSettings.snap_* + UI in TopBar Settings cluster (~100
  linhas)
- 7.G: Tests (transform math, snap quantize, gesture recognition) (~200
  linhas)

**Total: ~1600 linhas / 10-12 h.** Dependências: Transform já existe
(M14.1), screen_to_world já existe (M14.4b.bis), selection picking
precisa de hit-test em world (M14.5 sprite strategies cobre parte).

## M14.4e v2 — Bugs reportados em uso real (shipped 2026-05-12, commit `956e3bc`)

A v1 do M14.4e shipou drag-and-drop funcional + 3 bugfixes; v2 fechou
dois sintomas pós-commit:

- **Y-axis inversion** (camera/grid/sprite Y consumers desencontrados)
  — view_proj normalizada (sem swap bottom/top), pan_screen_delta +=,
  screen_to_world `cy - ny*half_h`. Testes pinned em `camera.rs`
  (`pan_screen_down_moves_camera_up_world`,
  `y_up_world_maps_to_positive_clip_y`, etc).
- **Imagens importadas "agrupadas" na hierarquia** — root cause em
  [`hierarchy.rs:248`](../../crates/ph2d-editor/src/screens/hero/hierarchy.rs#L248):
  o painter chamava `store.hierarchy_depth_of(id)` que lê estado de
  DnD residual; trocado para `entity.indent` (snapshot autoritativo
  do `build_hierarchy_snapshot`) em live mode. Fixture mode continua
  usando o depth do store.

## Processo do loop de implementação (regra ativa)

A partir de 2026-05-12 o usuário pediu uma camada extra de revisão
antes de cada fase do loop:

1. **Draft**: agente principal escreve plano de implementação da fase
   (modelo de dados, arquivos tocados, sub-passos, teste estratégia)
2. **Pre-review**: spawn de Plan agent recebendo o draft + critical
   files do repo; agent retorna parecer (gaps, simplificações,
   alternativas)
3. **Decision**: agente principal consolida feedback, ajusta plano se
   pertinente
4. **Implementation**: código + tests
5. **Post-audit**: spawn de Explore agent valida implementação contra
   plano + procura bugs/inconsistências
6. **Fix**: corrige gaps reportados, re-build + tests verde
7. **Commit**: PR atômico

Aplica a Atlas V2 (D), M14.5, M14.6, M14.7, Telemetria (F), Inspector
polish (E). Skip pra fixes de bug menores.

## M14.5 inspector phase — Sprite Render Source + Reimport (planned)

A spec adicional pra quando o Inspector ganhar live-binding na entidade selecionada (depende de `gizmo_selection` já estar em produção — feito em M14.7 A).

### Sub-panel "Render Source" (dropdown 3 strategies)

Quando uma entity com `Sprite` é selecionada, surge "Render Source" no Inspector:
```
Strategy: [Dynamic Atlas ▾]
          • Dynamic Atlas (auto)
          • Hand-packed Atlas (manual JSON+PNG)
          • Individual Texture (Godot-style)
```

Cada escolha troca `Sprite::source` na entity:
- **Dynamic Atlas** (M14.4d): `SpriteSource::Atlas { key }` — atlas compartilhado, packing runtime
- **Hand-packed Atlas** (M14.5 B + futuro renderer integration): `SpriteSource::HandPacked { atlas_id, region_index }` — sheet curado
- **Individual Texture** (M14.5 C): `SpriteSource::Individual { texture_id }` — textura própria, Godot-style batching

Trocar de strategy:
1. Strategy atual: chamar `release` se Individual ou liberar slot atlas
2. Strategy nova: `acquire` no store apropriado
3. Atualizar Sprite component

### Botão "Reimport"

Abaixo do dropdown:
```
[ Reimport at current px/m ]
```

Quando o user altera `pixels_per_meter` global (M14.4d Settings) mas o sprite foi importado com o valor anterior, o `Sprite.size` ficou stale. Reimport:
1. Lê o `Asset::ImageRgba8` original do AssetDb (via `atlas_asset_map[key]`)
2. Recalcula `size = source_pixels / current_pixels_per_meter`
3. Atualiza o `Sprite` component em SimWorld (sem reimportar bytes — só recalcula world size)

Implementação estimada: ~200 linhas (Inspector binding + Sprite refactor) + ~80 linhas (reimport handler).

## M14.A — Inspector Transform editor + canonical NumberInput interaction (shipped)

Live Transform editor section in the Inspector + canonical interaction
upgrade applied to every `NumberInput` in the editor (Transform fields,
Widget Gallery showcase, vector3 chips, slider+chip composites, future
Periférico panels).

**Transform editor (Inspector live binding):**
- `InspectorTransformInfo { entity_bits, translation: [f32; 2], rotation_rad: f32, scale: [f32; 2] }`
  in `ph2d-editor::screens::hero` — host snapshot, mirrors the M14.5
  sprite pattern. Loose-coupled (no `ph2d-ecs` types in editor crate;
  shell converts to/from `Transform` at the boundary).
- `paint_transform_section` (5-column grid: label / X tag / X box / Y
  tag / Y box) painted ABOVE Render Source. Position + Scale rows use
  two NumberInputs each (R/G axis tints, no Z — 2D-by-design per SKILL §3 +
  ADR-0025). Rotation row is single NumberInput in degrees (rad ↔ deg
  via `f32::to_degrees`/`to_radians`, HR-5 bit-deterministic).
- Reset-to-Identity button in the section header. Same commit code path
  as a field commit (publishes `pending_transform_edit`).
- Selection-change buffer reset: `last_inspector_entity` tracks the
  entity that the 5 NumberInput buffers belong to; mid-edit selection
  switch force-rewrites all buffers so the edit can't leak across
  entities.
- **First end-to-end consumer of `EditorCommandQueue`**. Inspector
  publishes `pending_transform_edit`; shell drains, encodes the
  `Transform` as postcard, pushes `EditorCommand::SetComponent` to the
  queue, calls `apply_editor_commands` with the `ComponentRegistry`
  pre-loaded with `register_ecs_components` at boot. All prior
  `pending_*` fields bypassed the queue with direct
  `sim.world_mut().get_mut::<…>()` — keep this path canonical for
  MCP / Luau / multi-agent edits in M14.B+.

**Canonical NumberInput interaction polish:**
- **Continuous-hold on stepper arrows**: Down on ▲/▼ does the usual
  single increment AND arms a hold; `dispatch_tick` (new public entry,
  called once per frame from the shell with `Self::timestamp_ns()`)
  fires repeats every 30 ms after a 250 ms initial delay. Released on
  pointer-Up.
- **Drag-slider on body**: Down on the box body (not the stepper)
  records a drag candidate. After the cursor moves ≥ 4 px (the
  threshold), the field flips into slider mode. The **dominant axis is
  decided at the moment of promotion and locked** for the rest of the
  drag — `|dx| vs |dy|` (>= favors horizontal). Subsequent off-axis
  wobble is ignored; only a fresh Down resets the axis.
- **Rates**: horizontal locked = 50 step-units / px (fast); vertical
  locked (cursor up = positive, cursor down = negative) = 5 step-units /
  px (slow); Shift held multiplies by 0.001 (fine adjustment).
- **Buffer realtime**: drag-slider mutates `value` + `buffer` +
  `last_committed` directly (bypassing `set_number_value`'s focus-guard)
  so the focused field shows the new number every frame during scrub.
- **Mouse-up split**: drag past threshold commits and clears focus
  (drag mode ≠ edit mode); drag below threshold keeps the field
  focused for typing (regular click-to-edit). Stepper hold ends
  always.
- **Pointer-event modifiers bridge**: `ph2d-host::PointerEvent` doesn't
  carry modifiers natively; shell pushes Shift state to
  `WidgetStore::set_shift_held(...)` on every `WindowEvent::ModifiersChanged`.
- New state types in `ph2d-editor::interaction::state`:
  `NumberInputDragState`, `NumberStepperHoldState`. Tunable constants:
  `DRAG_RATE_X` (50.0), `DRAG_RATE_Y` (5.0), `DRAG_SHIFT_MUL` (0.001),
  `STEPPER_HOLD_INITIAL_DELAY_NS` (250 ms), `STEPPER_REPEAT_INTERVAL_NS`
  (30 ms), `NUMBER_INPUT_DRAG_THRESHOLD_PX` (4.0).
- Doc: `docs/IntegracaoMultiAgente/03-Agente-Periferico.md` §5.6 +
  SKILL §11.9 widget table updated. Peripheral agents using
  `paint_number_input_with_buffer` get the full canonical behavior for
  free (interaction lives in the dispatcher, not the widget).

**Tests**: 8 regression tests in `interaction::dispatch::tests` cover
horizontal rate, vertical rate + inversion, Shift multiplier,
no-drag-preserves-edit-mode, axis-lock at promotion, axis-lock
persistence through off-axis wobble, buffer-realtime refresh,
continuous-hold initial delay + repeat, hold ended on pointer-Up. 2 in
`screens::hero::tests` cover Transform commit and Reset.

**Workspace check**: 523 lib tests + 20 integration tests pass; clippy
+ fmt clean.

## M14.D — Inspector Visibility checkbox (shipped)

Mirror of the Hierarchy eye toggle (M14.6 A) inside the Inspector.
Single checkbox + "Visible" label painted at the top of the Inspector
body, above the Transform section. Both surfaces (Inspector
checkbox + Hierarchy eye) drive the same
`ph2d_ecs::Visibility { hidden: bool }` component via
`EditorCommand::SetComponent` — same pipeline established for
Transform in M14.A, no new boundary.

- New `InspectorVisibilityInfo { entity_bits, visible }` snapshot +
  `pending_visibility_edit` channel on `HeroScreen`. Host publishes
  the snapshot each frame the gizmo selection has a `Transform`
  component (the "Inspector-worthy" gate).
- `pub const INSP_VISIBILITY_CHECK: NodeId = NodeId(381)` registered
  as `InteractiveState::Checkbox` in `inspector::populate`. Default
  Checked matches the canonical absence-equals-visible invariant.
- `paint_visibility_row` painted BEFORE the Transform section in
  `paint_inspector`. Reads the live `CheckboxValue` from the store
  (host writes it on the frame the snapshot lands).
- Commit path: dispatch toggles `CheckboxValue` and emits
  `WidgetEvent::Toggled(INSP_VISIBILITY_CHECK)`; `HeroScreen::apply_event`
  reads the post-toggle value, raises `pending_visibility_edit`;
  shell drains, encodes `Visibility { hidden: !visible }` as
  postcard, pushes `SetComponent`, runs `apply_editor_commands` —
  same code path as Transform commits.
- The shell **always** writes an explicit `Visibility { hidden: ... }`
  (never removes the component) so the round-trip is unambiguous and
  the eventual audit log captures both directions.
- Cached `visibility_type_id` (alongside `transform_type_id`) in
  `AppGfx` so the per-toggle hash is amortized to one-time at boot.
- Tests: `visibility_toggle_publishes_pending_with_selection` and
  `visibility_toggle_no_pending_without_selection` in
  `screens::hero::tests`.

**Workspace check**: 525 lib tests + 20 integration tests pass;
clippy + fmt clean.

## M14.7 polish — rename mode + long-press (planned)

A hierarchy row's right-click menu currently lists Duplicate / Add Child / Reset Transform / Delete (M14.6 F shipped). Two more interactions remain:

- **Double-click on row** → focus on entity (shipped M14.7 polish — uses `WidgetEvent::DoubleClick`).
- **Long-press (≥ 500 ms hold) on row** → enter inline rename mode. Replaces the row's name label with a TextInput; Enter commits the new name, Esc cancels. Estimated ~250 linhas (state machine + inline TextInput + Name component write-back).

## M14.4g+ telemetry — raw fps (planned)

Status bar currently shows fps + frame_ms derived from wall-clock between frames. With vsync on (default), this caps at the display refresh rate (60 Hz / 120 Hz). Per user feedback, add a "raw" fps reading derived from CPU+GPU work time per frame (excludes vsync wait), so the user can gauge headroom for new sprites/effects.

- `BottomHudStats.frame_cpu_ms: f32` — host measures `Instant::now()` delta between start of `render_frame` and end of `queue.submit` (before `present`)
- Status bar shows: `60 fps · 16.7 ms · ~2400 raw`
- Useful for: "I added 1000 sprites and raw fps dropped to 800 — that's a 0.4 ms cost per sprite"

Estimated: ~30 linhas (instrumentation + new BottomHudStats field + paint format).

## M14 settings UX — cascade submenu (planned)

The TopBar gear icon (`TOPBAR_SETTINGS`) currently opens a flat menu with 5 `pixels_per_meter` presets. As more global settings land (snap_move / snap_rotate / theme tokens / etc.), the flat list won't scale. Planned restructure:

```
[gear icon] → SettingsMenu (top-level categories)
    ├─ Pixels per meter ▸  → submenu (the 5 presets)
    ├─ Snap settings ▸     → submenu (move + rotate steps)
    ├─ Theme ▸             → submenu (4 themes + radius scale)
    └─ Show grid · G       → toggle inline
```

Implementation:
- New `ContextMenuKind::SubmenuRequest { parent_kind, anchor }` variant
- Painter for top-level entries reserves a right-side chevron when item has submenu
- Click on a parent entry opens submenu adjacent (mirrors macOS native menu pattern shown in user's reference)
- Menu items with no submenu fire their action as before

Estimated: ~400 linhas (menu state-machine + painter + dispatch).

## Backlog técnico (sem marco assignado)

### Telemetria de render real (substituir placeholder da status bar)

[`crates/ph2d-editor/src/screens/hero/bottom_hud.rs`](../../crates/ph2d-editor/src/screens/hero/bottom_hud.rs) atualmente paint segments com strings hardcoded (`"60 fps · 16.7 ms"`, `"42 draws"`, `"1.2K sprites"`, etc.). Não há telemetria real plumada. Implementar em duas fases:

**Fase A (CPU-side, ~30 linhas, baixo risco):**
- `Instant::now()` delta entre frames no `App::render_frame` + EWMA (α=0.1) → `frame_ms` real.
- `fps = 1000.0 / frame_ms` derivado.
- `sim.world().entity_count()` real.
- `present.world().query::<&RenderInstance>().iter().count()` real.
- Plumb via `HeroScreen.stats: RenderStats` struct → `bottom_hud` lê.

**Fase B (GPU timestamps, ~1h, mais plumbing):**
- `wgpu::QuerySet` com `wgpu::QueryType::Timestamp` em cada pass (sprite, tonemap, vello, compositor).
- `resolve_query_set` + readback assíncrono (uma frame de latência aceitável).
- `RenderStats.gpu_ms_breakdown: [f32; 4]` → segments do status bar.

**Quando fazer:** após M14.4d ou no próximo hardening sprint. Útil pra responder objetivamente "está pesada?" e detectar regressões perf por marco futuro (CI integration possível via frame budget gate, vide §"Definition of done").
