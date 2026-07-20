# HANDOFF — Rendering Modes + Wet Mix (PH2D Painter)

> **Para:** o próximo implementador (contexto fresco).
> **Design canônico:** [`docs/Painter/07_rendering_modes_wet_mix.md`](07_rendering_modes_wet_mix.md) — leia-o INTEIRO antes de codar. Este handoff é o **roteiro operacional** (checkpoint, ordem, anchors, gotchas, aceite); o design tem a math e a justificativa de cada decisão.
> **Regra-mãe do projeto:** `cargo check -p` verde é VELOCIDADE, não prova de nada. O que prova é o **teste e2e + a verificação visual no canvas demo**. (feedback_tool_unit_green_integration_dead / project_painter_canvas_res_64.)
> **A CADA passo:** releia [`docs/IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md`](../IntegracaoMultiAgente/DIRETIVA_IMPLEMENTACAO.md).

---

## 1. Objetivo + escopo

Adicionar ao pincel do Painter um **modo de renderização por-traço** + dois efeitos de borda + o grupo **Wet Mix** (smudge/mixer). O *enabler* é um **stroke buffer RGBA premultiplicado-linear por traço**: os dabs acumulam nele e o traço é composto **uma única vez** na camada no pen-up.

### 1.1 As 6 features (design §2)
1. **Uniform Glaze** — filme de cor; auto-overlap NÃO acumula intra-traço (cap MAX). Quase = caminho `stroke_mask` atual + composite-único.
2. **Intense Glaze** — mesma regra, teto de alpha mais alto (additive); só muda escalar.
3. **Uniform Blending** — dab LÊ o destino e faz lerp; acumulação MAX.
4. **Intense Blending** — o mais pesado ("squash and mix"); lê destino + acumulação additive. **É o ponto onde o Wet Mix ganha vida.**
5. **Wet Edges** — feather/blur das bordas do traço (verbatim Procreate), no finalize.
6. **Burnt Edges** — color-burn na banda da borda (`rim = max(0, α − blur(α))`), no finalize.

### 1.2 Decisão Wet Mix (design §1, §4)
Rendering Mode e Wet Mix são **acoplados, sem gate liga/desliga duro**. O RenderingMode **seleciona a regra de acumulação** do stroke buffer (MAX vs additive) e **liga a leitura de destino** (Blending). Wet Mix é um **smudge/mixer state independente**, mas só tem em que "morder" nos modos Blending. **Veredito:** Wet Mix é ESSENCIAL para os 2 modos Blending, DISPENSÁVEL para Glaze e para Wet/Burnt Edges → **Wet Mix vem por último** (Fase 2); Glaze (Fase 1) + Edges (Fase 3) shippam antes dele.
Default RGB-lerp linear (sem Mixbox/K–M — design §6: não há código espectral alcançável, ADR-0096). Mixbox residual fica como Fase 5 opcional atrás de flag.

### 1.3 ⛔ INVARIANTE INEGOCIÁVEL: a engine atual é INTOCÁVEL + duas seções de UI com master toggle

**O comportamento atual do Painter NÃO pode mudar em NADA sem a escolha explícita do usuário.** Esta é a trava acima de todas: nenhuma feature nova altera um único pixel do que o pincel já faz hoje a menos que o usuário **opte explicitamente** por ela na UI. "Default byte-idêntico" (§2) é o piso; o **master toggle** é a garantia visível disso.

Os parâmetros novos vivem em **duas seções novas e separadas** no painel, cada uma com um **master checkbox** que liga/desliga a seção inteira:

| Seção (nova) | Master toggle | Conteúdo |
|---|---|---|
| **Rendering** | `Use Rendering` (default **OFF**) | Mode dropdown (Uniform/Intense Glaze, Uniform/Intense Blending) · `Wet Edges` · `Burnt Edges` · `Burnt Strength` · `Edge Blur` |
| **Wet Mix** | `Use Wet Mix` (default **OFF**) | `Dilution` · `Charge` · `Pull` · `Wet Blur` (o smudge/mixer state) |

**Regra dura (sem exceção):**
- **`Use Rendering` OFF ⇒ a engine usa EXATAMENTE o caminho atual** (equivale a `RenderingMode::Direct`), **ignorando** o que estiver selecionado no Mode dropdown. Marcar a seção é o ato explícito que liga o stroke buffer.
- **`Use Wet Mix` OFF ⇒ nenhum smudge/pickup roda** (a cor continua fixa por traço, como hoje). Wet Mix só "morde" nos modos Blending **e** só com `Use Wet Mix` marcado.
- **Ambos OFF (o default de fábrica) ⇒ byte-idêntico ao Painter de hoje**, não importa o valor de qualquer slider/dropdown das seções novas.

**Mecanismo (engine):** dois bools novos no `BrushSpec`, default `false`, que gateiam o modo **efetivo** — o `rendering_mode`/sliders podem ter qualquer valor salvo, mas **só têm efeito** quando o usuário marca o master toggle:
```rust
pub use_rendering: bool,   // default false — master gate da seção Rendering
pub use_wet_mix:   bool,   // default false — master gate da seção Wet Mix

// no dispatcher/finalize:
let effective_mode = if spec.use_rendering { spec.rendering_mode } else { RenderingMode::Direct };
let smudge_on      = spec.use_wet_mix && effective_mode.reads_destination();
```
O golden test (§2.2) prova a invariante: com os dois OFF, **qualquer combinação** de Mode/sliders produz o MESMO hash do baseline. (Detalhe de UI em §4.3; teste adversarial em §6.)

### 1.4 Crates — minhas vs proibidas
| Crate | Papel | Posso editar? |
|---|---|---|
| `ph2d-painter-brush` | engine CPU (spec, dab, blend, novo `wet_buffer.rs`) | **SIM** |
| `ph2d-tool-painter` | host/lifecycle (paint.rs, brush_settings.rs, trait_impls.rs) | **SIM** |
| `ph2d-panel-painter-layers` | UI (populate/event/paint_brush + novo paint_rendering.rs) | **SIM** |
| `ph2d-editor-core` (só `src/ids/chrome/painter_brush_sections.rs`) | IDs de chrome do painel | **SIM** (só esse arquivo de IDs; é o padrão das seções existentes) |
| `ph2d-color`, `ph2d-tokens`, compositor, qualquer contrato congelado | reuso read-only | **NÃO — PARE e reporte ao Coordenador** |

⚠️ Se precisar mexer em `ph2d-color`, no compositor, num contrato congelado (§6 do CLAUDE.md), ou em qualquer arquivo fora da lista acima: **PARE e reporte ao Coordenador.** Não renegocie direto com outro agente.
⚠️ `BrushSpec` é `Copy`, sem serde, sem `SCHEMA_VERSION` no engine (a serialização vive upstream em `BrushSettings`). Adicionar campos inline é seguro e NÃO é "mexer em contrato congelado" — o gate de pintura foi revogado (ADR-0099/§6). Se ao adicionar campos algum arch-gate reclamar, **PARE e reporte** (não suprima gate).

---

## 2. CHECKPOINT + rollout NÃO-DESTRUTIVO

A garantia central (§1.3): **com os master toggles `Use Rendering` e `Use Wet Mix` em OFF — o default de fábrica — a saída é BYTE-IDÊNTICA à de hoje.** Tudo de novo fica atrás dos master toggles + modo/flags; o caminho legado nunca é editado in-place. A engine atual é **intocável** sem opt-in explícito do usuário.

### 2.1 Checkpoint — ✅ JÁ FEITO pelo Coordenador (você só cria a sua branch)

⚠️ **NÃO recrie a tag nem o backup — já existem (Coordenador, 2026-06-27).** Recriar gera artefatos/erros duplicados.
- **Tag de revert:** `painter-pre-rendering-modes` → `e719cdee` (estado pré-feature), **já no GitHub** — confirme com `git ls-remote --tags origin painter-pre-rendering-modes`.
- **Docs** (este handoff + `07` + INDEX) commitados em `main` (`b2128599`), **CI verde**.
- Diff `tag→HEAD` = **só docs**; o **código do sistema de pintura+layers é byte-idêntico** ao do backup. Sua linha de base de regressão é o HEAD atual (golden, §2.2).

Você só cria a sua **branch de trabalho** a partir do `main` atual:
```bash
cd /Volumes/MAC_EXTERNO/PROJETOS/_PH2D_definitiva
git switch main && git pull --ff-only          # esteja no b2128599 (ou mais novo)
git switch -c feat/painter-rendering-modes      # sua branch (você NÃO pusha — CLAUDE.md §3)
git status                                       # rode SEMPRE antes de stage
```
> **Anti-colisão:** se `git status` mostrar `M`/`??` que não são seus (ex.: `docs/Painter/HANDOFF_rake_rewrite.md` — doc de outra feature), **NÃO** stage nem comite. `git add -- <só os seus paths>`, sempre ([feedback-parallel-agent-collision]).

### 2.1b ⛑️ BACKUP COMPLETO — ✅ JÁ FEITO (não refaça)

O backup completo e verificado (SHA-256) do **sistema de pintura + layers** já existe em **4 localizações** (o código não mudou desde então — só docs). **Não refaça** — geraria duplicatas.

| Camada | Local | Conteúdo |
|---|---|---|
| Local | `backups/painting_layers_full_2026-06-27/` + `.tar.gz` + `ph2d-full-2026-06-27.bundle` | 5 crates + ids + docs + repo git completo |
| Off-local 1 | `~/PH2D_safety_backups/2026-06-27/` (disco interno do Mac) | tarball + bundle |
| Off-local 2 | `~/Library/Mobile Documents/.../PH2D_safety_backups/2026-06-27/` (iCloud) | tarball + bundle |
| Off-site | GitHub — tag `painter-pre-rendering-modes` @ `e719cdee` | histórico commitado |

> **Sistema =** `ph2d-painter-brush` · `ph2d-tool-painter` (`tool/paint`, `tool/layers`, `compositor/`, `undo`) · `ph2d-painter-effects` · `ph2d-panel-painter-layers` · `ph2d-color`.
> **Restauração:** `backups/RESTORE_painting_layers_2026-06-27.md` (tarball / `git reset --hard <tag>` / `git clone` do bundle).
> **Ao fechar um marco grande** (ex.: fim da Fase 1), crie uma tag **nova** (`painter-rendering-fase1`) — **não** sobrescreva a `painter-pre-rendering-modes`.

### 2.2 Golden / snapshot PINANDO a saída atual — ANTES de qualquer mudança
Antes de adicionar o enum, escreva um teste que renderiza um traço determinístico e **fixa o hash do canvas resultante** com o engine de hoje. Esse golden é o que pega regressão no caminho Direct.

```rust
// crates/ph2d-painter-brush/tests/golden_direct_path.rs  (ou onde os testes do engine vivem)
// Renderiza um traço fixo (seed fixa, coords fixas, opacity 0.3) e fixa o hash do RGBA8.
#[test]
fn direct_path_byte_identical_baseline() {
    let canvas = render_fixed_stroke(/* rendering_mode default */);
    let h = fnv1a(&canvas);                 // ou qualquer hash determinístico já disponível
    assert_eq!(h, 0x________, "Direct path mudou — regressão no caminho legado!");
}
```
Procedimento: rode o teste **com a placeholder `assert_eq!(h, h)` primeiro para CAPTURAR o hash**, cole o literal, e a partir daí o teste falha se o caminho Direct mudar 1 byte. Mantenha esse teste verde em TODAS as fases.

### 2.3 Invariantes de não-destruição (design §3.2, §7.3)
- **(a) MASTER GATE.** `use_rendering: false` + `use_wet_mix: false` por default. O modo **efetivo** = `if use_rendering { rendering_mode } else { Direct }`; `Direct.uses_stroke_buffer() == false` ⇒ o dispatcher (`paint.rs:524`) cai no caminho atual; nenhum dab toca `wet_buffer`, nenhum finalize roda, nenhum smudge roda. **Os dois OFF = Painter de hoje, byte a byte — não importa o valor de Mode/sliders.**
- **(b)** Todo código novo atrás de `if effective_mode != Direct` / `if smudge_on` / `if wet_edges` / `if burnt_edges`. O caminho legado (`stamp_stroke_dabs` → `dab.rs` write em `dst[i..i+4]`) **não muda uma linha** — você ADICIONA branches ao lado, nunca reescreve.
- **(c)** Defaults dos campos novos byte-idênticos ao comportamento atual:
  ```rust
  use_rendering: false, use_wet_mix: false,    // ← master gates: TUDO desligado por default
  rendering_mode: RenderingMode::Direct,
  wet_edges: false, burnt_edges: false,
  burnt_strength: 0.0, edge_blur_px: 0.0,
  wet_dilution: 0.0, wet_charge: 1.0, wet_pull: 0.0, wet_blur: 0.0,
  ```
- **(d)** Alocação do `wet_buffer` **só quando `use_rendering` seleciona um modo ≠ Direct** — o default não paga RAM nem clear.
- **(e) Teste adversarial do gate (§6.1):** o golden roda com os master toggles OFF **e** com Mode/sliders em valores NÃO-default (ex.: `rendering_mode=IntenseBlending`, `wet_dilution=1.0`, `use_rendering=false`) → tem que dar o MESMO hash do baseline. Prova que **só o master toggle**, não o valor salvo do parâmetro, muda o comportamento.

### 2.4 Revert total se preciso
```bash
git reset --hard painter-pre-rendering-modes   # volta ao estado pré-feature (cuidado: descarta WIP da branch)
# ou, para descartar só a branch e voltar:
git switch main && git branch -D feat/painter-rendering-modes
git tag -d painter-pre-rendering-modes          # só quando a feature fechar e for aprovada
```

---

## 3. Ordem de implementação (faseada — design §10)

Cada fase é **independentemente shippável**, fecha com `cargo check -p` verde dos 3 crates, mantém o golden §2.2 verde, e tem um teste self-contained. Inner loop = SÓ `cargo check -p` (test/clippy/audit 1× no fim do módulo).

```bash
CHECK='cargo check -p ph2d-painter-brush -p ph2d-tool-painter -p ph2d-panel-painter-layers'
```

- **Fase 0 — Contrato + UI esqueleto (sem efeito).** `RenderingMode` enum + os bools `use_rendering`/`use_wet_mix` (default false) + os demais campos em `BrushSpec` (default Direct/neutro) + espelho em `BrushSettings` + **DUAS seções UI** ("Rendering" + "Wet Mix"), cada uma com seu **master toggle** (`Use Rendering`/`Use Wet Mix`), nos **5 sites** (master checkbox + dropdown Mode + checkboxes + sliders). Setters mutam o spec mas **nenhum efeito de render ainda**.
  - Gate: `$CHECK` verde; golden §2.2 verde (incl. o caso adversarial §2.3-e); marcar/desmarcar os master toggles e trocar o Mode na UI muta o spec (logue/inspecione).
  - Teste: `use_rendering`/`use_wet_mix`/`rendering_mode` round-trips em `BrushSettings`; defaults byte-idênticos; **com os master toggles OFF, qualquer Mode/slider produz o hash baseline**.
- **Fase 1 — Stroke buffer (foundation) + Glaze.** `wet_buffer`/`wet_bbox` no `Paint`; `stamp_dabs_to_wet` + `composite_wet_to_canvas`; routing em `paint.rs:524`. Implementar **Uniform Glaze** (MAX) e **Intense Glaze** (additive) — source-only (não lê destino).
  - Teste: Glaze em opacity 0.3 com N dabs sobrepostos → `α ≈ min(O, max aᵢ)`, NÃO `1−(1−a)ⁿ`. Composite-único (sem double-composite).
- **Fase 2 — Blending + smudge state (Wet Mix).** `Smudge` struct (premul linear) + pickup reusando `dab.rs:508–512`; **Uniform/Intense Blending**; sliders Dilution/Charge/Pull/Wet Blur ativos.
  - Teste: pintar B sobre A vs sobre transparente dá resultados DIFERENTES (prova que lê destino).
- **Fase 3 — Wet Edges / Burnt Edges.** `box_blur_separable` (running-sum, O(área)) + `finalize_wet_buffer` (rim, color-burn reusando `BrushBlend::ColorBurn`, overlap-gate). Toggles ativos.
  - Teste: Wet → variância do gradiente de α na borda cai; Burnt → `rim>0` só na banda + reforço onde `layer_alpha>0`.
- **Fase 4 (opcional) — Polish Wet Mix:** `Grade`, `Wet Jitter` (design §4.2, deferidos).
- **Fase 5 (opcional) — Mixbox residual** no smudge mix, atrás de flag `pigment_mode` (design §6).

MVP = Fases 0–3. Fases 4–5 só se sobrar escopo na sessão.

---

## 4. Pontos de integração exatos (anchors verificados no repo)

### 4.1 Engine — `crates/ph2d-painter-brush/src/`
- `spec.rs:10` — após `use crate::falloff::Falloff;`: declarar `pub enum RenderingMode` (design §7.1, com `to_u8`/`from_u8`/`uses_stroke_buffer`/`reads_destination`/`is_additive`/`name`).
- `spec.rs:31` — `pub struct BrushSpec` — adicionar os 9 campos novos (design §7.2) logo após `grain_depth` (campo em `spec.rs:111`).
- `spec.rs:157` — `impl Default for BrushSpec` (campos atuais em `spec.rs:168–184`) — adicionar os defaults byte-idênticos (§2.3-c).
- `dab.rs` — `stamp_band` (~423); destino já decodificado em `508–512` (`prev` — reusar como **pickup** para Blending); cap MAX em `532–541` (`if m >= coverage { continue }` etc.); write na camada em `551` (`dst[i..i+4]`) — **caminho legado, não tocar**.
- `blend.rs:33` (variante `BrushBlend::ColorBurn`) / `blend.rs:186` (dispatch em `blend_rgb`) / `color_burn` helper — **reusar** para o color-burn dos Burnt Edges.
- **NOVO** `wet_buffer.rs` — `box_blur_separable` + `finalize_wet_buffer` + `stamp_dabs_to_wet` + `composite_wet_to_canvas` + `Smudge`. `mod wet_buffer;` no `lib.rs`.

### 4.2 Tool — `crates/ph2d-tool-painter/src/tool/paint.rs`
- `paint.rs:202` — struct `Paint` (campo `stroke_mask: Vec<u8>` já existe) — adicionar `wet_buffer: Vec<f32>` + `wet_bbox: Option<Rect>`.
- `paint.rs:287` — após `self.paint.stroke_mask.clear()` (pen-down): clear/resize-zero do `wet_buffer` + `wet_bbox=None`, **só se `rendering_mode != Direct`**.
- `paint.rs:524` — `fn stamp_stroke_dabs` (dispatcher) — branch: se `uses_stroke_buffer()` → `stamp_dabs_to_wet`; senão caminho atual.
- `paint.rs:301` / `:328` — sites que chamam `stamp_stroke_dabs` (per-event e finish) — herdam o branch automaticamente.
- finalize no pen-up: após o `stamp_stroke_dabs` do `finish()` (próximo de `:328`, antes do snapshot/undo em `:381`) → `finalize_wet_buffer()` (edges) → `composite_wet_to_canvas()` (1 over) → `mark_dirty(wet_bbox)` (`mark_dirty` em `:425`).
- undo (`:381`) — inalterado; `wet_buffer` é transiente, fora do snapshot.
- `paint/brush_settings.rs` — espelhar os campos novos em `BrushSettings` + setters `set_brush_rendering_mode`, `toggle_brush_wet_edges`, `toggle_brush_burnt_edges`, `set_brush_*` (clamp 0..1).
- `tool/trait_impls.rs` — handlers de `Click`/`SetValue`/`SelectOption` para os widgets novos.

### 4.3 UI — `crates/ph2d-panel-painter-layers/src/` — **DUAS seções novas, cada uma com master toggle** (design §8)

Seções (colapsáveis, default colapsadas/OFF):
- **"Rendering"** — `Use Rendering` (master, default OFF) · Mode dropdown · `Wet Edges` · `Burnt Edges` · `Burnt Strength` · `Edge Blur`.
- **"Wet Mix"** — `Use Wet Mix` (master, default OFF) · `Dilution` · `Charge` · `Pull` · `Wet Blur`.

Os **5 sites** por widget (design §8); pular um = clique/drag dropado silenciosamente ([feedback-panel-populate-register]):
1. **IDs:** `crates/ph2d-editor-core/src/ids/chrome/painter_brush_sections.rs`:
   - Rendering: `RENDERING_SECTION`, **`RENDERING_USE`** (master checkbox), `RENDERING_MODE` (dropdown), `RENDERING_WET_EDGES`, `RENDERING_BURNT_EDGES`, `RENDERING_BURNT_STRENGTH`(+`_CHIP`), `RENDERING_EDGE_BLUR`(+`_CHIP`), factory `painter_brush_rendering_mode_option_id(u8)`.
   - Wet Mix: `WET_MIX_SECTION`, **`WET_MIX_USE`** (master checkbox), `WET_MIX_DILUTION`/`_CHARGE`/`_PULL`/`_WET_BLUR` (+`_CHIP` de cada).
2. **Register** (`populate.rs`): os **2 master checkboxes** + dropdown + 2 toggles de edge no array de buttons; os **6 sliders** em `register_brush_slider_chips` (`:219`) — **cada slider chama `store.set_number_range(chip, 0.0, 1.0, 0.01)`** (loop `:316`); as **2 seções** em `register_collapsible_sections` (`:323`). O teste `:474` (todo chip tem range) continua valendo.
3. **Paint** (NOVO `paint_rendering.rs` com as duas seções, ou `paint_rendering.rs` + `paint_wet_mix.rs`): `paint_collapsible_section("Rendering", …)` com **`paint_checkbox_row("Use Rendering")` no topo** + `paint_dropdown_row("Mode", …)` + `paint_checkbox_row`s (Wet/Burnt Edges) + `paint_slider_chip_row`s; `paint_collapsible_section("Wet Mix", …)` com **`paint_checkbox_row("Use Wet Mix")` no topo** + sliders. Integrar em `paint_brush.rs` (após Eraser) + `mod` no `lib.rs`.
4. **Event** (`event.rs` / `event/`): os 2 master toggles + Wet/Burnt Edges no match `Click`; os 6 sliders no `ValueChanged`; opção do Mode via o roteador de dropdown do brush.
5. **Tool:** handlers em `trait_impls.rs` + setters em `brush_settings.rs`: **`toggle_brush_use_rendering`**, **`toggle_brush_use_wet_mix`**, `set_brush_rendering_mode`, `toggle_brush_wet_edges`, `toggle_brush_burnt_edges`, `set_brush_*` (clamp 0..1).

**Mode dropdown labels (English):** `Uniform Glaze`, `Intense Glaze`, `Uniform Blending`, `Intense Blending`. (O `Direct` da engine **não** aparece no dropdown — é o estado quando `Use Rendering` está OFF; o master toggle é o liga/desliga, o dropdown só escolhe QUAL modo ativo.)
**Affordance do master toggle:** com `Use Rendering` OFF, esmaeça (token `Text2`) ou colapse o resto da seção Rendering; idem `Use Wet Mix` OFF para a seção Wet Mix. **NÃO** desabilite o `register` dos widgets internos (evita drop de hit) — só o pintar (estado visual) muda. Dentro de Rendering, os controles de Wet Mix são significativos só nos modos Blending (`reads_destination()`); pode esmaecê-los também, mas o master toggle da seção Wet Mix é o gate real.

---

## 5. Gotchas do projeto (não pule nenhum)

- **Trap do populate-register (feedback_panel_populate_register / feedback_tool_unit_green_integration_dead):** botão/slider/dropdown NOVO exige TODOS os 5 sites do §4.3. Pintar + hit_index NÃO basta. Se faltar o register no `populate.rs`, o widget aparece mas o clique/drag é **dropado silenciosamente** — e o unit-test passa. Só audit e2e + visual pega.
- **`set_number_range` obrigatório (reference_number_input_register_range):** toda caixa numérica/slider LIMITADA precisa de `set_number_range(id, min, max, step)`. Sem isso, o drag escala por `rate×step` (±1 dispara pra 100) e a setinha pula. Use `0.0, 1.0, 0.01` para os sliders 0..1. O teste em `populate.rs:474` falha se esquecer.
- **Default byte-idêntico é o invariante #1 (§2.2/§2.3):** o golden test pin do caminho Direct DEVE ficar verde em toda fase. Se ele quebrar, você editou o caminho legado in-place — reverta a edição, ponha o código novo atrás do flag.
- **`cargo check` verde ≠ funciona (regra-mãe):** check só prova que compila. A prova é o teste e2e do dispatch + a **verificação visual no canvas demo 64px** (o Enio faz pen-input — só ele tem caneta; você prepara o cenário). project_painter_canvas_res_64: o canvas demo é 64×64; render macio borra por causa da res pequena, não do algoritmo — não caçe bug onde não há.
- **Caveat "só visível em opacity baixa" (design §11 Trap #2):** em opacity=1 os 6 modos CONVERGEM. Um teste em opacity=1 "passa" e prova NADA. Teste de modo SEMPRE em opacity≈0.3.
- **Premul/linear (design §6, §11):** o `wet_buffer` é **premultiplicado + linear-sRGB**. `max`/additive/over são associativos sem divisão em premul; `max` em straight injeta lixo quando `a≈0`. Encode/decode sRGB↔linear só na borda (reuso do compositor — NÃO reimplemente LUT). Teste: dab quase-transparente sobre cor saturada não deve injetar cor (straight falharia).
- **Save/serialização:** o engine NÃO versiona (`BrushSpec` é Copy, sem serde). A persistência vive em `BrushSettings` upstream — espelhe os campos lá. Sem `SCHEMA_VERSION` para bumpar no engine. (Design §7; map:brushspec-config §4.)
- **Tokens/i18n (HR-15):** zero hex, zero f32-literal de UI, zero string hardcoded NOVA. Labels em **English** (Wet Edges, Burnt Edges, Dilution, Charge, Pull). ⚠️ Os labels do brush hoje são hardcoded inline (`paint_brush.rs:220` "Accumulate" etc.) — isso é **dívida pré-existente**; siga o padrão vigente da crate (inline) para consistência, e **registre no handoff de volta** que label→i18n token é follow-up cross-cutting da crate inteira (não isolável a esta feature). Cores/sizes: SEMPRE `ph2d-tokens`/`ColorToken`.
- **Git anti-colisão:** `git status` antes de stage; `git add -- <só meus paths>`; `git commit --no-verify -m "msg" -- <paths>`; NUNCA `-A`/`git add .`/`git stash`. Há WIP alheio na árvore (§2.1) — não o toque.
- **RAM/perf:** `wet_buffer` canvas-inteiro em 4K = 256 MiB premul (proibitivo) → bbox-local é mandatório fora do demo. Meça perf em `--release` (dev=opt0 mente — project_painter_composite_perf).

---

## 6. Critérios de aceite + testes

### 6.1 Testes automatizados (rodam 1× no fim do módulo; cada fase tem o seu)
| Prova | Asserção |
|---|---|
| **Default byte-idêntico (o mais importante)** | mesmo traço com `use_rendering=false`+`use_wet_mix=false` → canvas **bit-a-bit** = baseline pré-feature (golden §2.2). Hash igual. |
| **Master gate adversarial (§2.3-e)** | com os master toggles OFF **mas** `rendering_mode=IntenseBlending` + `wet_dilution=1.0`+`wet_charge=…` setados → MESMO hash do baseline. Prova que o valor salvo do parâmetro NÃO muda nada sem o master toggle. |
| **Master toggle liga** | mesmo traço com `use_rendering=true`+`rendering_mode=UniformGlaze` → hash **DIFERENTE** do baseline (o opt-in realmente ativa o stroke buffer). |
| **Uniform Glaze** | N dabs sobrepostos, opacity 0.3 → `α ≤ O+ε`, NÃO `1−(1−a)ⁿ`. |
| **Intense Glaze** | mesma sobreposição → `α` cresce (additive), composto **1×** (sem double-composite contra a camada). |
| **Uniform/Intense Blending** | pintar B sobre A ≠ pintar B sobre transparente (prova pickup do destino). |
| **Wet Edges** | variância do gradiente de α na borda CAI vs toggle off. |
| **Burnt Edges** | `rim = max(0, α−blur(α)) > 0` só na banda; reforço onde `layer_alpha>0`; rim mais escuro que o interior. |
| **Premul correctness** | `max` em premul não injeta cor em pixel `a≈0`. |
| **e2e dispatch** | clicar o Mode dropdown / arrastar um slider muta o spec (populate↔register vivo). |

### 6.2 Verificação visual e2e (o Enio faz — pen-input)
Prepare o cenário no canvas demo 64px e peça ao Enio para validar com a caneta:
1. **Default (master toggles OFF)** = visualmente idêntico ao build atual — e marcar/desmarcar `Use Rendering`/`Use Wet Mix` alterna entre "Painter de hoje" e os modos novos, sem nunca mudar o de hoje sozinho.
2. **Glaze vs Blending** em opacity baixa: Glaze mantém cor uniforme no cruzamento do X; Blending acumula/mistura.
3. **Wet Edges** = bordas mais suaves; **Burnt Edges** = borda escurecida onde sobrepõe tinta.
4. **Intense Blending** = "squash and mix" mais agressivo (o Wet Mix ganha vida).

### 6.3 Gate batched no fim do módulo (1×, não por task)
```bash
cargo nextest run -p ph2d-painter-brush -p ph2d-tool-painter -p ph2d-panel-painter-layers
cargo clippy -p ph2d-painter-brush -p ph2d-tool-painter -p ph2d-panel-painter-layers --all-targets
```
Auditoria ≥2 lentes (feedback_audit_lens_diversity): (1) correção numérica premul/linear vs straight/sRGB; (2) costura e2e dispatch+visual.

---

## 7. Definição de pronto + handoff de volta ao Coordenador

**Pronto quando:** Fases 0–3 fechadas; golden Direct byte-idêntico verde; todos os testes §6.1 verdes; `cargo check -p` + nextest + clippy verdes nos 3 crates; cenário visual §6.2 preparado.

**Reporte ao Coordenador (você NÃO pusha — CLAUDE.md §3):**
- Commits locais prontos (hashes), na branch `feat/painter-rendering-modes`, com a tag `painter-pre-rendering-modes` como ponto de revert.
- O que landou (Fases 0–3) e o que ficou deferido (Fases 4–5: Grade, Wet Jitter, Mixbox).
- **Follow-up registrado:** migração label→i18n token da crate `ph2d-panel-painter-layers` inteira (dívida pré-existente, cross-cutting, não isolável a esta feature).
- **Pendência de validação:** verificação visual e2e §6.2 precisa do pen-input do Enio antes de declarar 100% vivo (cargo-check verde ≠ funciona).
- Qualquer momento em que você teve que **PARAR** por algo fora dos crates da §1.3 (contrato/ph2d-color/compositor) — reportado, não contornado.
