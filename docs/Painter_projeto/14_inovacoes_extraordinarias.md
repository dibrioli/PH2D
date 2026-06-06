# 14 — Inovações extraordinárias (superando o Procreate)

> Doc dedicado às **5 propostas extraordinárias** + **Crítica A** absorvidas após análise do feedback Antigravity / Google DeepMind ([`avaliacao_e_melhorias.md`](avaliacao_e_melhorias.md)). Mandato Enio 2026-05-23: padrão-ouro absoluto, tempo maior aceito como custo de superar Procreate em vez de igualar.

## 14.0 Princípio operacional

O Painter PH2D não é "clone multiplataforma do Procreate" — é o **sucessor**. Cada feature aqui foi escolhida porque:

1. **Procreate não tem** (ou tem versão inferior por limitação histórica do iPad).
2. **É viável** dentro da infraestrutura Rust + wgpu + compute shaders do PH2D.
3. **Cabe nos princípios** PH2D (HR-1..HR-18, multi-plataforma, LLM-first, padrão ouro).

Esta página é a fonte de verdade técnica para **Coordenadores escrevendo ADRs** e **Implementadores construindo as waves específicas**.

## 14.1 Mapa rápido (referência)

| # | Inovação | Origem | Wave | ADR | Veredicto |
|---|----------|--------|------|-----|-----------|
| Crítica A | **Adjustment Layers** Photoshop-style | doc DeepMind §3.A | W4 | ADR-0045 | 12 de 17 adjustments viram non-destructive layers |
| Proposta 1 | **Vetor Oculto** (Stroke Vector History full + Reproject + Inspector retroativo) | doc DeepMind §4.1 | W1 + W12 + W14 | ADR-0046, ADR-0048 | Aceito integral em 3 waves |
| Proposta 2 | **Mixbox pigment mixing** (Kubelka-Munk simplificado) | doc DeepMind §4.2 | W5 | ADR-0044 | Axis ortogonal aos 6 Rendering modes |
| Proposta 3 | **Procedural Grain** (Simplex/Gabor/PaperWeave/SprayDot) | doc DeepMind §4.3 | W5 | ADR-0044 | Híbrido — 4 dos 8 grãos default viram procedural |
| Proposta 4 | **Fluid Brushes Extension** (Shallow Water sim + giroscópio) | doc DeepMind §4.4 | W15 | ADR-0049 | Crate dedicado opt-in com graceful degrade |
| Proposta 5 | **Painter MCP Stroke Engine** (LLM gera strokes reais) | doc DeepMind §4.5 | W13 | ADR-0047 | Aceito integral; baixo custo, alto impacto estratégico |

## 14.2 Crítica A — Adjustment Layers Photoshop-style (W4)

### 14.2.1 Origem da decisão

O spec original W0 descartou Adjustment Layers em [12_fora_de_escopo §12.2](12_fora_de_escopo.md) sob a justificativa de "sabor Procreate lean". O doc DeepMind §3.A apontou — corretamente — que a destrutividade absoluta de Procreate é **herança técnica do iPad 2011** (VRAM/CPU apertados), não tese de design.

### 14.2.2 Por que muda o jogo

- **Workflow profissional** exige reajuste pós-feedback. Cliente pede "muda a saturação dos shadows" — em Procreate o artista revive estado pre-adjustment ou aceita perdas (duplicate antes de cada ajuste explode VRAM em canvases 4K+).
- **Painter PH2D roda em desktop / iPad Pro M1+ / Android top-tier** — orçamento de sobra para adjustment layers sem regredir mobile entry-level.
- **PSD interop** ganha — 5 dos 12 adjustments mapeiam 1:1 para PSD adjustment layer types. Workflow Painter ↔ Photoshop fica bidirectional.

### 14.2.3 Escopo W4 (ADR-0045)

Detalhe completo em [02 §2.10.X](02_layers.md). Sumário:

**12 adjustments com Adjustment Layer support (non-destructive):**
HSB, Color Balance, Curves, Gradient Map, Brightness/Contrast, Gaussian Blur, Motion Blur, Noise, Sharpen, Bloom, Halftone, Chromatic Aberration.

**5 adjustments destructive-only (continuam Layer + Pencil modes):**
Liquify, Clone, Recolor, Glitch, Mesh Warp. Razão técnica documentada em [02 §2.10.X.2](02_layers.md).

### 14.2.4 Custo

- Compositor revisado para recomposition com dirty rect aware (existing infra extends).
- VRAM compositor cache cresce ~30 MB para adjustment intermediate states.
- Tempo de implementação: wave dedicada (W4); ~3-5 semanas estimado.

## 14.3 Proposta 1 — Vetor Oculto (Stroke Vector History + Reproject + Stroke Inspector)

### 14.3.1 Origem da decisão

Doc DeepMind §4.1: gravar todo stroke como vetor de alta precisão; canvas pode ser reprojetado em qualquer resolução re-executando o histórico; usuária pode trocar brush retroativamente.

### 14.3.2 Por que muda o jogo

- **Resolução-independente** (real): Procreate em canvas 1080p → upscale 8K via bilinear blur. Painter em canvas 1080p → **re-renderiza strokes em 8K nativamente**, pixel-perfect.
- **Stroke Inspector** transforma o canvas de **commit final** em **objeto editável até o fim**. "Pintei essa onda com pencil_2b; quero ver como fica com ink_studio_pen" — sem re-pintar.
- **Replay determinístico** para snapshot/sync futuros (HR-5 opt-in).

### 14.3.3 Escopo dividido em 3 waves

#### W1 — Stroke Vector History full (ADR-0046)

- `StrokeRecord` completo gravado **desde dia 1** no `.ph2d-painter` ([01 §1.14](01_brush_engine.md)).
- `StrokeHistory::Full` (não-ring) em desktop / iPad Pro / Android top.
- Fallback `StrokeHistory::Ring(1000)` em web / Android entry (memory tight).
- Undo opera no Vec; snapshots a cada 50 strokes para otimização.

#### W12 — Reproject to Resolution (ADR-0046 cobre)

- Dialog "Reproject canvas to X×Y" com progress bar.
- Operação **offline**, não real-time:
  - Det-mode (CPU fallback): ~5 strokes/segundo; 5000 strokes = ~16 min. Bit-identical cross-platform.
  - GPU mode: ~50 strokes/segundo; 5000 strokes = ~100 segundos. Aproximado em ULPs.
- Resultado: canvas em nova resolução com qualidade superior a bilinear upscale.

#### W14 — Stroke Inspector retroativo (ADR-0048)

- UI dedicada — Actions → Stroke Inspector (`Ctrl+Shift+I`).
- **Lasso temporal**: drag no canvas para selecionar strokes individuais.
- Overlay visualiza path desenhado de cada stroke (pontos numerados; pressure heat-map opcional).
- Operações retroativas:
  - Trocar brush.
  - Trocar primary/secondary color.
  - Ajustar pressure curve (scaling slider).
  - Deletar stroke específico (vs Undo cronológico).
- Compositor re-renderiza **apenas a slice afetada** (não re-roda todo o stack).

### 14.3.4 Custo

- RAM stroke history full: ~80-150 MB em sessões longas (vide [08 §8.2](08_performance_memory.md)).
- VRAM zero adicional (stroke history é RAM-side; só rasteriza na composição).
- Wave 1: infrastructure (1-2 semanas).
- Wave 12: Reproject operation (2-3 semanas + golden tests).
- Wave 14: Stroke Inspector UI (3-4 semanas — UI complexa).

### 14.3.5 Limitações honestas

- "Resolução infinita real-time" do doc DeepMind era **otimista**. Reality: offline operation com progress bar. Não promete instant scaling.
- Replay GPU é aproximado; det-mode (CPU) é exato mas 3-5× lento.
- Stroke Inspector seleção visual de strokes em canvas com milhares é UX challenge — pesquisar W14.

## 14.4 Proposta 2 — Mixbox pigment mixing (W5)

### 14.4.1 Origem da decisão

Doc DeepMind §4.2: substituir lerp linear de cor por mistura física subtrativa baseada em Kubelka-Munk.

### 14.4.2 Por que muda o jogo

Em Procreate / Photoshop:
- Azul + amarelo = cinza-esverdeado morto.
- Vermelho + verde = marrom escuro/cinza.

Em Painter com Mixbox:
- Azul + amarelo = **verde vibrante natural**.
- Vermelho + verde = **marrom quente orgânico**.

Quem pinta com mídia tradicional **espera esse comportamento**. Procreate e Photoshop falham (blend linear). Adobe Fresco tem aproximação parcial.

> **Honestidade competitiva (2026-06-06):** mistura de pigmento subtrativa **não é nova** — Rebelle (Escape Motions), apps que licenciam o Mixbox, e o spectral.js (MIT) já entregam. Contra Procreate/Photoshop é vantagem real; contra Rebelle/Mixbox-licensees é **paridade**, não dianteira. O valor diferenciado do PH2D aqui é ser **clean-room shippável** (ver §14.4.6) — não a novidade do algoritmo.

### 14.4.3 Algoritmo (implementado — clean-room, NÃO o LUT da scrtwpns)

> **Correção 2026-06-06:** NÃO portamos o Mixbox da scrtwpns (dados CC-BY-NC, ver §14.4.6). O que ship é um **modelo espectral próprio** (`mixbox.rs`), validado contra os alvos artísticos públicos:

1. **sRGB linear → reflectância** sobre uma **base Gaussiana de 3 canais** (reconstrução larga; integração desacoplada mais estreita → sem vazamento verde→azul).
2. Mistura por **Kubelka-Munk single-constant** (`K/S=(1−R)²/2R`, blend linear em K/S, inverte) — a física de 1931, domínio público.
3. **Re-anchor** por-cor do round-trip → endpoints e self-mix EXATOS apesar da base "vazada".
4. Integra de volta a sRGB; blend de endpoint `4t(1−t)` (glaze fino quase não desvia; mistura 50/50 = pigmento pleno).

Modelo wash separado (`apply_stamps_wash`): opacidade limita a cobertura do stroke (sem build-up→amarelo). Paridade CPU (path ao vivo) + WGSL.

**Teto clean-room** (2026-06-06): o absoluto-literal (dados de pigmento real medido, método do próprio Mixbox) está **bloqueado por licença comercial** — destravar exige permissão escrita do Berns/RIT + Golden, e aí é troca de tabela drop-in. Detalhe da investigação em `avaliacao_e_melhorias.md`.

### 14.4.4 Onde aplica

`pigment_mode` é axis ortogonal aos 6 Rendering modes:

| Brush categoria | `pigment_mode` default |
|---|---|
| Pencils, Inks, Markers, Airbrushes | `Linear` (cor previsível) |
| **Paints (oils), Watercolors, Gouache** | **`Mixbox`** (mistura subtrativa orgânica) |

Usuária override em Brush Studio.

### 14.4.5 Custo

- ~30 ops adicionais por stamp pixel (vs ~5 do Linear). Dentro do budget (≤0.2 ms adicional em 4K M2).
- 0 VRAM adicional (polinomial direto).
- Tempo de implementação: ~1 semana embedded em W5.

### 14.4.6 Licença — **CORREÇÃO CRÍTICA (2026-06-06)**

A afirmação anterior ("Mixbox MIT, sem fricção") estava **factualmente ERRADA**. Verificado:

- **Mixbox da scrtwpns é CC BY-NC 4.0** ([github.com/scrtwpns/mixbox](https://github.com/scrtwpns/mixbox)) — uso **não-comercial**; comercial exige **licença paga** (mixbox@scrtwpns.com). **Não pode ser embutido** num produto que vende.
- Por isso ship é **clean-room**: Kubelka-Munk (domínio público) + base espectral própria. Zero dado/código deles. Auditado: nenhuma dependência da crate, nenhum LUT, nenhum coeficiente de pigmento medido deles (`mixbox.rs` só referencia o github deles em comentário de comparação).
- **Pendência de higiene de marca:** o nome interno `PigmentMode::Mixbox` (não-user-facing) deve ser renomeado p/ neutro (`Pigment`/`Subtractive`) — adiado (toca contrato congelado + `ph2d-color`, requer Coord).

Alternativas de dados com licença comercial-OK investigadas: Mallett-Yuksel (MIT, sintético), spectral.js (MIT, base derivada do Burns CC-BY-SA ☣), dados de pigmento real (Berns/RIT, Golden, FORS) — **todos não-comercial ou cortesia, nenhum redistribuível**. Detalhe em `avaliacao_e_melhorias.md`.

## 14.5 Proposta 3 — Procedural Grain híbrido (W5)

### 14.5.1 Origem da decisão

Doc DeepMind §4.3: substituir atlas bitmap (64 MB) por geração procedural via compute shader. Tiling zero, resolução infinita, VRAM econômico.

### 14.5.2 Por que muda o jogo

- **Tiling zero**: pincel size 2048px sobre canvas 8K. Bitmap grain repete visivelmente; procedural não.
- **Zoom infinito**: usuária dá zoom 400% para detail work. Bitmap grain fica pixelizado; procedural fica perfeito.
- **VRAM economy**: 64 MB → ~32 MB. Crítico em web (cap 200 MB) e Android entry.

### 14.5.3 Por que **híbrido** (não 100% procedural)

Nuance crítica que o doc DeepMind subestima: **grãos com assinatura visual específica** (canvas Belga vs Cotton, charcoal pesado vs vine, watercolor cold-pressed vs hot-pressed) **não são reproduzíveis com Perlin/Simplex/Gabor**. Esses vêm de **escaneamentos reais** e dão ao Procreate brushes o "weight" de mídia tradicional.

Substituir todos perde isso. **Híbrido preserva o melhor dos dois mundos.**

### 14.5.4 Mapeamento dos 8 grãos default

| Grain | Decisão | Por quê |
|-------|---------|---------|
| `paper_subtle` | **Procedural::SimplexNoise** | Noise genérico; escala melhor |
| `charcoal_heavy` | Bitmap (mantém) | Assinatura de carvão escaneado |
| `marker_streak` | Bitmap (mantém) | Streak pattern específico |
| `canvas_weave` | Bitmap (mantém) | Assinatura Belga vs Cotton |
| `watercolor_paper` | Bitmap (mantém) | Assinatura cold-press vs hot-press |
| `spray_grain` | **Procedural::SprayDot** | Dots benefit de escala infinita |
| `noise_white` | **Procedural::SimplexNoise** | Noise puro |
| `noise_pink` | **Procedural::SimplexNoise (pink filter)** | Idem |

### 14.5.5 ProceduralGrain enum

Detalhe em [01 §1.3.5.1](01_brush_engine.md). 4 variants em W5:
- `SimplexNoise { scale, octaves, persistence, seed }`
- `GaborNoise { frequency, orientation, anisotropy, seed }`
- `PaperWeave { fiber_density, fiber_anisotropy, crossweave, seed }`
- `SprayDot { dot_density, dot_size, dot_jitter, seed }`

### 14.5.6 Custo

- VRAM: -32 MB (atlas cai de 64 → 32 MB).
- GPU compute: ~50-150 ops adicionais por sample em procedural; dentro do budget.
- Tempo de implementação: ~2 semanas embedded em W5.

### 14.5.7 Determinismo

Procedural grain é **determinístico** se `seed` é estável (não usar `time` como input). HR-5 mantido em det-mode.

## 14.6 Proposta 4 — Fluid Brushes Extension (W15)

### 14.6.1 Origem da decisão

Doc DeepMind §4.4: simulação de Shallow Water / Lattice Boltzmann para mídias úmidas, com capilaridade, sangramento, gravidade via giroscópio.

### 14.6.2 Por que muda o jogo

Procreate / Photoshop / Fresco fazem aproximação visual de aquarela (smudge + blur + wet edges fake). **Painter PH2D faz simulação física real**:

- Pintura aquarela próxima a área molhada → **tinta sangra organicamente pelas fibras**.
- Wet edges são **acumulação física de pigmento** (não fake darkening).
- Mais úmido + mais tilt do device → **tinta escorre fisicamente** (giroscópio do iPad/iPhone/Android com sensors).
- Óleo espesso vs aquarela diluída → comportamentos físicos distintos (viscosidade).

### 14.6.3 Arquitetura

Crate dedicado `crates/ph2d-painter-fluid/`. **Opt-in por brush** via flag `brush.fluid_enabled = true`.

Pipeline GPU (per-frame quando ativo):

```
┌─────────────────────────────────────────────────────┐
│ Stamp pipeline (brush) deposita tinta              │
│   ↓ writes pigment density texture (1/4 canvas res)│
│ Fluid solver pass (compute shader)                  │
│   ↓ Shallow Water solver: density + velocity +     │
│     pressure + viscosity                            │
│   ↓ Gravity vector vem do PlatformHost::gyroscope() │
│ Composite fluid → layer texture                     │
│   ↓ aplica wet edges físicos, sangramento          │
└─────────────────────────────────────────────────────┘
```

Resolução fluid texture: **1/4 do canvas** (tradeoff perf vs detail). 4K canvas → 1024² fluid texture. Suficiente para detail visual em maioria dos casos.

### 14.6.4 Mídias suportadas

Brushes que ativam fluid sim por default (W15):

- `watercolor_wash` ✓
- `watercolor_detail` ✓
- `oil_round` (viscosidade alta)
- `oil_bristle` (idem)
- Custom brushes que usuária habilita Brush Studio

Brushes secos (`pencil_*`, `ink_*`, `marker_*`, `airbrush_*`) **não ativam fluid** — não faz sentido.

### 14.6.5 Giroscópio integration

```rust
// PlatformHost extends:
fn gyroscope(&self) -> Option<Vec3>;
```

iPad/iPhone/Android com sensors → `Some(gravity_vector)`.
Desktop sem sensors → `None`. Fluid sim usa default gravity (downward, magnitude=0 = sem gravidade).

Toggle "Use device gyroscope" em Painter Preferences → Fluid Brushes:
- ON (default em devices com sensor): tinta escorre conforme inclinação física do device.
- OFF: gravity fixed = downward com magnitude configurável.

### 14.6.6 Graceful degrade

Devices que **não cabem** no budget fluid (web, Android entry, iPhones antigos):

```rust
if memory_budget.fluid_capable() && perf_budget.fluid_headroom_ms > 3.0 {
    activate_fluid_pass();  // ~20-30 MB VRAM, 3-5 ms per frame
} else {
    use_traditional_wet_mix();  // §01 §1.3.7 Wet Mix matemático
}
```

**Identidade do brush mantida.** Usuária ainda vê `watercolor_wash` se comportando como aquarela — só não tem o "wow" da fluid sim em low-end.

### 14.6.7 Custo

- VRAM: 20-30 MB adicional **apenas quando fluid sim ativo** (brush wet selecionado).
- GPU: 2.5-3.5 ms por frame **apenas quando ativo**.
- Budget Painter total quando ativo: ~6.7 ms (vs 4.2 ms sem). Cabe em 120 Hz (8.3 ms) com folga; 60 Hz (16.7 ms) com folga grande.
- Tempo de implementação: wave dedicada (W15); ~6-8 semanas (sim research + tuning + cross-platform).

### 14.6.8 Determinismo

Fluid sim em GPU é **não-determinístico** (reduções, atomic ops). Painter está em `PresentWorld` (HR-5 não vale) então OK em modo normal. Em det-mode (W12 replay determinístico), fluid sim cai para CPU fallback — 10× mais lento mas exato.

### 14.6.9 Risk profile

- Risco técnico mais alto das 5 propostas. Fluid sim em GPU em devices heterogêneos é desafio.
- Wave 15 = late no roadmap (após Painter v1.0 estável em ~W12-W14). Risco isolado de comprometer earlier waves.
- Se W15 falha ou estoura prazo, Painter v1.0 fica entregue sem fluid sim — degradação aceitável.

## 14.7 Proposta 5 — Painter MCP Stroke Engine (W13)

### 14.7.1 Origem da decisão

Doc DeepMind §4.5: LLM via MCP gera **sequências de strokes reais com brushes nativos**, não pixels colados de modelo generativo externo.

### 14.7.2 Por que muda o jogo

**Estado atual da IA + painting tools:**
- Photoshop Firefly: LLM gera PNG → cola como layer. Pixels colados, não-editáveis.
- Procreate: sem AI integration nativa.
- Affinity Photo 2: sem AI integration.

**Painter PH2D Proposta 5:**
- LLM via MCP recebe prompt: *"Aplique hachura cruzada a grafite na seleção, seguindo o contorno da luz."*
- LLM gera **lista de 50-200 strokes** com `pencil_2b` brush, paths apropriados, pressure curves.
- Engine executa via `painter_paint_strokes`.
- Resultado: 50-200 stroke records **reais, editáveis traço-a-traço** pelo artista humano.

**O controle criativo permanece humano**, enquanto IA atua como assistente técnico de execução.

### 14.7.3 API MCP (ADR-0047)

Detalhe completo em [01 §1.13](01_brush_engine.md). Resumo:

```rust
painter_paint_strokes(canvas, layer, brush, strokes, token) → Vec<StrokeId>
painter_modify_stroke(stroke_id, mods, token) → ()
painter_query_strokes(canvas, layer, filter) → Vec<StrokeRef>
painter_inspect_stroke(stroke_id) → StrokeRecord
```

### 14.7.4 Casos de uso

1. **"Hachura cruzada a grafite seguindo contorno da luz"** — LLM analisa selection + light + brush characteristics, gera strokes.

2. **"Preencher fundo desta seleção com aquarela_wash, gradiente azul claro → escuro"** — LLM gera strokes wash com brush wet, color modulada por posição. Mixbox (Proposta 2) produz transições orgânicas.

3. **"Refaça este line art que pintei com pencil como ink_studio_pen"** — LLM via `query_strokes` lista strokes existentes, chama `modify_stroke` para cada com `new_brush: ink_studio_pen`. Engine recompõe.

4. **"Sugira 3 estilos de finish para este sketch"** — LLM gera 3 variantes de strokes (oils, watercolor, ink) em layers separadas para artista comparar.

### 14.7.5 Custo

- **~200 LOC** wrapper MCP + ~50 LOC governance.
- **0 nova infra GPU.** Stamp pipeline já é cliente lógico desses dados.
- Tempo de implementação: ~2-3 semanas em W13.

### 14.7.6 Quality emerges via prompting

Tool MCP exposta ≠ LLM gera strokes esteticamente bons. **Qualidade emerge** via:
1. **System prompts** com exemplos de bom hachuring/washing/lineart.
2. **Painter brush characteristics** documentados no system prompt (cada brush e o que produz).
3. **Fine-tunes específicos** (Anthropic / OpenAI / locais) — futuro.

ADR-0047 cobre o **contrato técnico**. Quality engineering é trabalho contínuo pós-W13.

### 14.7.7 Governance (HR-11)

`painter_paint_strokes` e `painter_modify_stroke` são `destructive: true`. Token humano necessário (5 min, single-use) OU flag `--unsafe-mcp` no servidor (CI/dev mode).

Audit log per-stroke: `audit.log` (JSON Lines) com agent, brush, layer, strokes count, blake3 antes/depois.

## 14.8 Resumo de impacto

### 14.8.1 Antes vs depois

| Capability | Antes da revisão | Depois da revisão |
|------------|------------------|-------------------|
| Adjustment Layers | Não suportadas | **12 non-destructive layers + 5 destructive** (Crítica A) |
| Pigment mixing | Linear (cinza lamacento) | **Linear + Mixbox** (verde vibrante) |
| Grain | 8 bitmap (atlas 64 MB) | **4 bitmap + 4 procedural** (atlas 32 MB) |
| Stroke history | Ring 250 | **Full vetorial** + Reproject + Inspector |
| LLM painting | (não no spec) | **MCP Stroke Engine** completo |
| Mídia úmida | Smudge matemático | **Mídia úmida + Fluid Brushes Extension** (opt-in) |

### 14.8.2 Memory budget (HR-13)

| Plataforma | Antes (W0 original) | Depois (W17 stable) | Δ |
|------------|---------------------|----------------------|---|
| iPad/iOS | 390 MB | **523 MB** | +133 MB |
| Android | 390 MB | **523 MB** | +133 MB |
| Desktop | 1249 MB | **1560 MB** | +311 MB |
| Web | 291 MB | **352 MB** | +61 MB |

Todos ainda dentro de platform totals do SKILL_Stack §12.1.

### 14.8.3 Roadmap

| Antes (W0 original) | Depois (W0 revisado) | Δ |
|--------------------|----------------------|---|
| 12 waves | **17 waves** | +5 waves (W4 Adj Layers + W12 Reproject + W13 MCP + W14 Inspector + W15 Fluid + Polish W17) |
| ~12 meses estimados | **~18-20 meses estimados** | +50% tempo |

### 14.8.4 ADRs

| Antes (W0 original) | Depois (W0 revisado) | Δ |
|--------------------|----------------------|---|
| 2 ADRs (0041, 0042) | **7 ADRs (0041–0047)** | +5 ADRs |

## 14.9 Como ler este doc

Por papel:

- **Coordenador escrevendo ADRs (0041–0047)**: este doc é fonte de verdade técnica. Cada ADR aprofunda **um item** da §14.1.
- **LLM Implementador de W4 (Adjustment Layers)**: §14.2 + [02 §2.10.X](02_layers.md) + [06 §6.3.1](06_selection_transform_adjustments.md).
- **LLM Implementador de W5 (Brush Studio + Mixbox + Procedural Grain)**: §14.4 + §14.5 + [01 §1.3.5.1 + §1.5.4 + §1.6.8 + §1.6.9](01_brush_engine.md).
- **LLM Implementador de W12 (Reproject)**: §14.3.3 (W12) + [01 §1.14.4](01_brush_engine.md) + [08 §8.2.1](08_performance_memory.md).
- **LLM Implementador de W13 (MCP Stroke Engine)**: §14.7 + [01 §1.13](01_brush_engine.md).
- **LLM Implementador de W14 (Stroke Inspector)**: §14.3.3 (W14) + [01 §1.14.5](01_brush_engine.md).
- **LLM Implementador de W15 (Fluid Brushes Extension)**: §14.6 + nova crate `ph2d-painter-fluid` — exige ADR-0049 aprovado primeiro.
- **Enio reviewing**: §14.0 + §14.8 (impacto) + tabela §14.1.

## 14.10 Reconhecimentos

As 5 propostas + Crítica A vêm do feedback recebido em [`avaliacao_e_melhorias.md`](avaliacao_e_melhorias.md) (Antigravity / Google DeepMind). Esse doc original é **referência histórica** — fica intacto. Esta página é a **síntese técnica** absorvida no spec PH2D, com nuances de implementação dimensionadas honestamente.

**Próximo passo:** escrever os ADRs 0043, 0044, 0045, 0046, 0047 (junto com os já planejados 0041 + 0042) e iniciar W0 freeze.
