# Plano operacional pós-spike — implementação real do core

**Status:** In progress (M1-M12 done; M13 active)
**Data abertura:** 2026-05-08 (logo após merge do PR #1, spike fechado)
**Última revisão:** 2026-05-09
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
| **M13 — Polish + features** | 🟢 Library + hero + input pipeline + deep polish + color picker fix done | branches `m13/design-library`, `m13/tool-palette-ui` | Tool palette UI shipada (PR #30). Design system canônico Claude Design importado (`docs/design/`, 89 SVGs + 17 telas + 4 specs + tokens.json). `ph2d-tokens` codegenerado (4 themes OKLCH) + struct `ColorValue { rgba, oklch }` (HR-12 a11y contrast pipe). Biblioteca completa: **32 widgets** em `ph2d-editor::widget`. **NumberInput + TextInput agora interativos** (buffer + caret + dispatch para `[0-9.eE+-]` em NumberInput, char insert + Backspace + arrows em TextInput; commit no Enter/Blur; revert no Escape). **Slider × NumberInput two-way binding** via `WidgetStore::link_slider_number` (drag mirror, type→commit mirror; Inspector wireado). **BlenderColorPicker reescrito** em pasta de 9 arquivos (state/paint/wheel/value_slider/segmented/channels/hex_field/palette/tests): wheel HSV real (peniko sweep + radial gradient), valor vertical com gradiente cor→preto, segmented com labels visíveis, hex parse `#RGB[A]`/`#RRGGBB[AA]`, paletas editáveis; **wheel e value clicáveis via `BlenderHit` shim no `dispatch_pointer`**. **`PH2D_THEME` env funcional** no shell desktop (estava hardcoded). `screens::hero` orquestra 10 sub-módulos. Showcase region (bottom-left) exercita 18+ widgets. 399 testes em ph2d-editor (de 367 → +32). Vide planos: [`2026-05-ui-components.md`](2026-05-ui-components.md), [`2026-05-editor-hero-screen.md`](2026-05-editor-hero-screen.md), [`2026-05-editor-input-pipeline.md`](2026-05-editor-input-pipeline.md), [`2026-05-hero-deep-polish.md`](2026-05-hero-deep-polish.md), [`2026-05-color-picker-fix.md`](2026-05-color-picker-fix.md). Próximo: projeto-piloto pra crates stub + screens 03-17 + segmented/palette/hex dispatch wireado. |

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
