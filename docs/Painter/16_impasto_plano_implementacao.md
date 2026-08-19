# #16 — Impasto: PLANO DE IMPLEMENTAÇÃO

> **Aprovado pelo Enio (2026-07-12).** Design + pesquisa: [15_impasto_pesquisa_e_design.md](15_impasto_pesquisa_e_design.md).
> **Ordem do Enio, literal:** *"tudo o que for compatível no painel painter — toda a seção Shape, Grain,
> todo o Stroke e até suas shapes dinâmicas, o tiling, o mirror, o randomize color, os color ramps de
> shape e grain — deve ser integrado ao sistema, e isso já deve ser planejado desde já. O que não for
> compatível deve ser escondido. **Cuidado: Watercolor é uma implementação à parte e não deve ser tocada
> ou ferida.**"*
>
> ⚠️ **Recorte de 2026-08-18.** Este doc é o **plano e as leis** do Impasto. As fases já
> construídas (§9 · §10 · §11 · §12 · §13 · §14 · §15 · §16 · §18.1-§18.5) foram movidas
> **verbatim** para
> [`docs/archive/docs-2026-08-18/Painter/16_impasto_plano_implementacao.md`](../archive/docs-2026-08-18/Painter/16_impasto_plano_implementacao.md).
> Ficou aqui: a arquitetura (§0), a matriz de compatibilidade (§1), **⚠️ a barreira do Watercolor**
> (§2), as três fases do plano (§3-§5), o que está **fora do 1º corte** (§6), o kill-criterion (§7),
> a ordem (§8), as **constantes que viraram lei** (a lista de divergências, com o
> **`DEPTH_UNIT_PX = 16`**), os **abertos nomeados**, **a FILA do Enio** (§17) e o **🔴 §18.6**.
> ⛔ Nada foi resumido — as duas metades remontam o original byte-a-byte (sha256).

---

## 0. A decisão de arquitetura que faz a ordem do Enio sair de GRAÇA

**A altura é um SEGUNDO OUTPUT do pipeline de dab que já existe — não um pipeline paralelo.**

Isso não é elegância gratuita: é a única forma de a integração ser *automática* em vez de N integrações
manuais que apodrecem uma a uma. O kernel de cor tem **um funil único**
([dab.rs:543](../../crates/ph2d-painter-brush/src/dab.rs#L543)):

```rust
let a = w * g * ctx.coverage;   //  w = silhueta (Shape+Falloff+ramp) · g = Grain (+ramp) · coverage = dyn
```

Tudo o que o Enio listou já entra **acima** desse funil:

| Feature | Onde ela age | Integra ao impasto… |
|---|---|---|
| **Shape** (silhueta · Falloff · imagem · procedural) | é o `w` do funil ([dab.rs:437-452](../../crates/ph2d-painter-brush/src/dab.rs#L437)) | **de graça** — o relevo tem a forma da ponta |
| **Shape Tone (ramp da silhueta)** | remapeia `w` antes de compor ([dab.rs:447](../../crates/ph2d-painter-brush/src/dab.rs#L447)) | **de graça** — e vira **escultura**: a ramp passa a modelar o relevo |
| **Grain + ramp + Depth do grain** | é o `g` do funil ([dab.rs:465-493](../../crates/ph2d-painter-brush/src/dab.rs#L465)) | **de graça** — é a fonte natural das **estrias de cerda** (`Depth Source = Grain`) |
| **Mirror / Symmetry** | `push_symmetric` empurra os dabs espelhados **na MESMA lista** ([stroke.rs:176](../../crates/ph2d-painter-brush/src/stroke.rs#L176)) | **de graça** — dab espelhado carrega altura |
| **Tiling** | `tiled_dabs` **replica a lista** antes do accumulate ([tiling.rs:13](../../crates/ph2d-tool-painter/src/tool/paint/tiling.rs#L13), usado em [stamp_route.rs:56/148/198](../../crates/ph2d-tool-painter/src/tool/paint/stamp_route.rs#L56)) | **de graça** — basta o accumulate de `h` consumir a MESMA lista tilada |
| **Stroke + shapes dinâmicas** (Curve/Line/Circle/Polygon/Free Hand, Offset/Trim/Simplify) | produzem lista de dabs pelo mesmo motor | **de graça** |
| **Jitter Scale / Rotate** | muda a geometria do dab | **de graça** |
| **Randomize Color · color ramps de cor** | mexem em **cor**, não em cobertura | **ortogonal** — e *corretamente*: jitter de cor **não deve** mudar a espessura da tinta |

> **A regra que isso impõe ao código:** o `h` **NUNCA** é acumulado por um caminho próprio de geometria.
> Ele consome **a mesma lista de dabs** (já simetrizada e já tilada) e **a mesma máscara** (`StampMask` =
> silhueta × grain, [stamp.rs:38](../../crates/ph2d-painter-brush/src/stamp.rs#L38)) que a cor consome.
> Qualquer PR que crie uma segunda geração de dabs para altura está **errado por construção** — é o
> mecanismo pelo qual "tiling não funciona no impasto" nasceria seis meses depois.

---

## 1. Matriz de compatibilidade (o que integra × o que ESCONDE)

O painel já tem o idiom exato de esconder por modo
([paint_brush_sections.rs:30-85](../../crates/ph2d-panel-painter-layers/src/paint_brush_sections.rs#L30)):
`!brush.paints_no_color() && !brush.eraser && !brush.is_mask`, `if !brush.is_inpaint`, `if !brush.is_clone`.
O Impasto usa **o mesmo idiom**, nada inventado.

### 1.1 INTEGRA (o card Impasto aparece e o relevo funciona)

Brush normal · Shape (todas as fontes) · Shape Tone ramp · Grain (todas as fontes + ramp) · Falloff ·
Stroke (todos os métodos + shapes dinâmicas) · Symmetry/Mirror · Tiling · Jitter Scale/Rotate ·
Randomize Color (ortogonal) · Per-Layer Color (§1.3) · Eraser (§1.3) · Selection/proteção/alpha-lock
(o `h` respeita a mesma gate de escrita da cor).

### 1.2 ESCONDE (o card Impasto não é pintado — logo, não é hit-registrado, logo é inerte)

| Modo | Flag | Por quê |
|---|---|---|
| **Watercolor** | `brush.watercolor` | **implementação à parte — NÃO TOCAR** (§2). Aquarela é tinta fina; impasto é óleo/acrílico. A Rebelle separa "Oils & Acrylics" de watercolor pelo mesmo motivo |
| **Inpaint** | `brush.is_inpaint` | o heal marca uma máscara de disco duro — **já ignora** Shape/Grain/ramps/stroke/symmetry/tiling ([paint_brush_sections.rs:39](../../crates/ph2d-panel-painter-layers/src/paint_brush_sections.rs#L39)) |
| **Mask** | `brush.is_mask` | máscara é grayscale — não tem relevo |
| **Smear / Blur / Clone** | `brush.paints_no_color()` | não **depositam** tinta nova. Arrastar relevo alheio é o **`Plow`** do Painter → **deferido, nomeado** (§6) |
| **Adjustment / Texture layers** | tipo de camada | não têm canal `h` |

### 1.3 Os dois casos que exigem trabalho EXPLÍCITO (não são de graça)

- **Eraser** — a borracha **tem** que apagar o `h` na mesma pegada, senão fica **relevo fantasma** (a tinta
  some e a luz continua acusando volume). Não é opcional; é correção.
- **Per-Layer Color** — essa rota **desvia** das rotas cacheadas normais
  ([stamp_route.rs:417](../../crates/ph2d-tool-painter/src/tool/paint/stamp_route.rs#L417)). O `h` sai da
  **cobertura-união** das N camadas de shape (uma altura, não N). Precisa de fio explícito na rota
  `stamp_dabs_per_layer_*`. **É o único ponto onde "de graça" não vale** — e por isso tem task própria (T1.7).

---

## 2. ⚠️ WATERCOLOR — a barreira (ordem explícita do Enio)

**Nenhum arquivo `watercolor_*.rs` é editado. Nenhuma linha. Zero.**

E a arquitetura **já garante** isso sozinha: o `stamp_dabs` faz **short-circuit da aquarela ANTES** de
chegar em `stamp_dabs_routed` ([stamp_route.rs:47](../../crates/ph2d-tool-painter/src/tool/paint/stamp_route.rs#L47)).
Com Watercolor ON, **o código do impasto nunca é alcançado** — não é uma promessa de disciplina, é o
fluxo de controle.

- O card Impasto fica **escondido** com `brush.watercolor` (§1.2) ⇒ nem existe estado a divergir.
- **Barreira executável (T3.4):** um gate que afirma que `Watercolor ON + Depth > 0` produz canvas
  **byte-idêntico** ao `Watercolor ON + Depth = 0`. Se algum dia alguém fiar impasto no caminho da
  aquarela, esse teste fica **vermelho**.
- **O relevo do PAPEL** (a ideia do §4.1 do doc 15 — `paper_h` alimentando a mesma luz) **sai do escopo**.
  Ela leria `watercolor_noise::paper_height`, e isso acopla impasto a aquarela. **Deferido, requer ordem
  nova do Enio** (§6). O impasto da Fase 1 é **só a altura do traço**.

---

## 3. Fase 1 — o canal `h` (dados + acumulação)

**Isolamento (regra B' do Modo L):** tudo em **módulos IRMÃOS novos**. Os arquivos que "naturalmente"
receberiam isso estão no teto: `watercolor_render.rs` **699/700** (e é proibido tocá-lo de qualquer forma),
`paint.rs` 653, `dab.rs` 605.

| Task | O quê | Onde (NOVO salvo indicado) | Gate VERMELHO refutável |
|---|---|---|---|
| **T1.1** | `heights: BTreeMap<RtLayerId, Vec<f32>>` — mapa **irmão** de `images`, lazy (ausente = camada sem relevo ⇒ custo zero) | `tool/mod.rs:80` (+1 campo) · `tool/layers/undo.rs:21` (+1 linha no clone) · `tool/documents.rs:20` | undo de um traço de impasto **restaura o `h` anterior** (hoje o snapshot já clona `images` — o teste prova que `heights` entrou junto) |
| **T1.2** | `stroke_height: Vec<f32>` — envelope do traço por **`max`**, lazy, no dirty-rect existente. Espelha `stroke_coverage` ([paint.rs:419](../../crates/ph2d-tool-painter/src/tool/paint.rs#L419)) | `tool/paint.rs` (+1 campo) · **`tool/paint/impasto.rs`** | **um** traço que passa 3× no mesmo ponto tem altura de **uma** passada (envelope, não escadinha); **dois** traços separados **somam** |
| **T1.3** | `BrushSpec`: `impasto: bool` · `depth: f32` · `depth_source: DepthSource{Uniform,Grain,Shape}` · `draw_to: DrawTo{Color,Depth,ColorAndDepth}` · `smoothing: f32`. Defaults = OFF | `spec.rs` (+campos, append) · `brush_settings.rs` · `snapshot.rs` · `brush_fallback.rs` | `impasto=false` ⇒ **canvas byte-idêntico** ao HEAD (igualdade de buffer, não "parece igual") |
| **T1.4** | **A máscara de altura reusa o `StampMask`** (silhueta × grain) — `Grain` = a máscara inteira; `Uniform` = a mesma renderizada com grain neutro; `Shape` = só o sample da silhueta. **Zero mudança no kernel de cor** | **`ph2d-painter-brush/src/height.rs`** (fn pura `height_of`) | `Depth Source = Grain` produz `h` que **varia** dentro do dab (estria); `Uniform` produz `h` **constante** no platô |
| **T1.5** | Accumulate de `h`: irmão de `accumulate_color_stamp_coverage` ([accumulate.rs:22](../../crates/ph2d-painter-brush/src/stamp_color/accumulate.rs#L22)), mas `f32` + `max`. **Consome a lista de dabs JÁ simetrizada e JÁ tilada** | **`ph2d-painter-brush/src/stamp_color/accumulate_height.rs`** | **Tiling ON**: traço na borda produz `h` na borda **oposta** (RED: sem consumir a lista tilada, a borda oposta fica chapada). **Symmetry ON**: `h` aparece espelhado |
| **T1.6** | **Eraser apaga `h`** na mesma pegada | `tool/paint/impasto.rs` | apagar a tinta apaga o relevo — RED: sem isso, `Show Impasto` ON deixa **volume fantasma** onde não há mais cor |
| **T1.7** | **Per-Layer Color**: `h` da cobertura-**união** das N camadas | rota `stamp_dabs_per_layer_*` ([stamp_color_cache.rs](../../crates/ph2d-tool-painter/src/tool/paint/stamp_color_cache.rs)) | com Per-Layer ON, um traço produz **um** relevo coerente (não N degraus empilhados) |
| **T1.8** | `Draw To` = `Depth` ⇒ escreve **só** `h` (pincel que só levanta/cava, sem cor) | `tool/paint/impasto.rs` | `Draw To=Depth` ⇒ o RGBA do canvas fica **byte-idêntico** e o `h` muda |

**Sinal:** `h` é `f32` **com sinal** — negativo = **cavar** (o `Negative Depth`/`Erase` do Painter, o HDR
[-1,1] do Substance). Float mata banding na origem.

---

## 4. Fase 2 — o passe de luz

| Task | O quê | Onde | Gate VERMELHO |
|---|---|---|---|
| **T2.1** | `h_total` = soma em z-order dos `heights` das camadas visíveis (Fase 1 = **Add**; os modos Subtract/Replace/Ignore são §6) | **`compositor/impasto_pass.rs`** (irmão — `compose.rs` está em 566/700) | camada oculta **não** contribui relevo; deletar a camada **remove** o relevo |
| **T2.2** | Normal por **diferença central 4-tap** + **Blinn-Phong** (ambient/diffuse/specular). Expoente de shininess por **LUT construída 1×** — HR-5, precedente literal em `watercolor_lut.rs` | **`tool/paint/impasto_light.rs`** | **oráculo derivado da DEFINIÇÃO**, não do shader ([[feedback_oracle_must_model_appearance_not_implementation]]): uma **rampa de altura analítica** (plano inclinado conhecido) ⇒ normal e shading **calculados à mão** batem com o passe. Um oráculo que espelhe o código fica verde com o bug na tela |
| **T2.3** | `Show Impasto` OFF ⇒ o passe **não roda** | idem | OFF ⇒ composite **byte-idêntico** ao HEAD |
| **T2.4** | Impasto visível ⇒ `gpu_eligible` devolve `None` ⇒ composite CPU (o fallback **já existe e já é usado**) | `render_loop/painter_gpu_preview.rs:132` (+1 guard) | com impasto ON o preview **é o CPU** (e o relevo aparece); sem impasto o caminho GPU **continua** sendo escolhido |

---

## 5. Fase 3 — UI (o card + a matriz VIVA)

Runbook já levantado; os **dois pontos de atrito** estão resolvidos no papel:
`event.rs` tem **dispensa de LOC congelada em 601/600** ⇒ **não recebe linha nova** (o predicado entra no
sibling `event_brush_forward.rs`, que já tem o precedente `is_deform_click`, em **troca LOC-neutra**);
`populate.rs` está em **591/600** ⇒ o Impasto **já nasce** como `populate_impasto.rs`.

| Task | O quê | Onde (NOVO) |
|---|---|---|
| **T3.1** | ids + arrays `PAINTER_IMPASTO_{CLICKS,FIELDS}` | `ph2d-editor-core/src/ids/chrome/painter_impasto.rs` |
| **T3.2** | Card **Impasto** (por brush): `Enable` · `Depth` · `Depth Source` (cycler) · `Draw To` (cycler) · `Smoothing` | `paint_impasto.rs` + `populate_impasto.rs` |
| **T3.3** | Card **Lighting** (canvas-level, 1 por documento): `Show Impasto` · `Light Angle` · `Light Elevation` · `Amount` (height-to-slope) · `Shine`. Precedente de params canvas-level no painel do brush: os da aquarela | idem |
| **T3.4** | **A matriz de §1 vira GATE EXECUTÁVEL** | `tests/` do painel |
| **T3.5** | Rota de eventos + setters + reset (espelho de `watercolor_settings.rs`, **sem tocá-lo**) | `tool/paint/impasto_settings.rs` |

**Gates de T3.4 (o que trava o apodrecimento — checklist em prosa NÃO morde):**
1. **Seam** (2 testes, espelho de `seam.rs:425/452`): o evento **real** de cada id chega no `BrushSpec`.
2. **Visibilidade:** em cada modo de §1.2 (`watercolor`/`is_inpaint`/`is_mask`/`paints_no_color`) o card
   **não é pintado** ⇒ nenhum id do Impasto é hit-registrado.
3. **★ Barreira do Watercolor:** `Watercolor ON + Depth>0` ≡ `Watercolor ON + Depth=0`, **byte-a-byte**.
4. **Integração viva** (o antídoto do "morre em 6 meses"): **Tiling ON** ⇒ `h` na borda oposta ·
   **Symmetry ON** ⇒ `h` espelhado. São os dois que provam que o `h` consome a lista de dabs **compartilhada**.

---

## 6. Fora do 1º corte — NOMEADOS (DEFER nomeia a capacidade exata; não conta como fechamento)

- **`Plow`** — Smear/Smudge **arrastar** o relevo existente (hoje o card é escondido nesses modos).
- **Composite Depth por camada** (`Add`/`Subtract`/`Replace`/`Ignore`) + escala de depth por camada. O
  modelo de dados **já nasce per-layer** ⇒ isto é **só o composite**, nunca reconstrução de topologia
  (regra two-strikes respeitada por construção).
- **Passe de luz na GPU** (`LayerOp` novo — há **8 slots livres** em `AdjustmentKind ≤ 32`, e código
  desconhecido no shader já é no-op identidade). Exige reconciliação **bit-a-bit** contra a CPU (DIRETIVA §4).
- **Relevo do PAPEL** — acoplaria impasto↔aquarela ⇒ **exige ordem nova do Enio** (§2).
- **Múltiplas luzes / IBL** (Krita tem 4; Rebelle tem environment maps).
- **Persistência do `h` no `ProjectState`** — herda o gap conhecido (o save já não persiste pixels de
  `SpriteSource::Individual`); fechar é o mesmo work item.

---

## 7. Kill-criterion (congelado ANTES do build — DIRETIVA §5)

> **Cenário fixo:** canvas 2048², r=100, traço arrastado, `Show Impasto` ON, 1 camada com `h`.
> **Alvo:** o passe (acumulação + normal + shading, sobre o dirty-rect) **≤ 4 ms/move**, com o move inteiro
> dentro de 16,7 ms.
> **Kill:** se após **2 tentativas** de otimização em CPU o passe estourar **8 ms/move**, a feature **não
> existe nesta forma** — vira GPU-only (o `LayerOp` do §6) **antes** de fechar a linha.

**Medir ANTES de otimizar** ([[feedback_measure_perf_symptom_scale]]): número em ms, por-knob, em `--release`.

---

## 8. Ordem de execução

1. **T1.3** (spec + defaults OFF) → prova de **byte-identidade** primeiro. É o alicerce: se o default não
   for byte-idêntico, nada mais importa.
2. **T1.1/T1.2/T1.4/T1.5** (dados + accumulate) → gates de **Tiling** e **Symmetry** (§5 T3.4.4).
3. **T1.6/T1.7/T1.8** (Eraser · Per-Layer · Draw To) — os que **não** são de graça.
4. **T2.\*** (luz) → oráculo analítico.
5. **T3.\*** (UI + a matriz como gate).
6. **Perf** (§7) → **Smoke do Enio** com exemplo pronto ([[feedback_ready_to_smoke_example]]).

Fecho a linha com gate batched + handoff de integração, e **PARO** — não integro nem faço ship (§0.7 do
CLAUDE.md).

---

## ⚠️ ESTADO (2026-07-12, dono novo): pesquisa 2 FEITA — o plano segue no §10

Fases 1–3 fecharam gateadas, mas o veredito do Enio (*"Não sei se melhorou ou piorou. Ficou mais
difícil de ajustar"*) suspendeu o plano até uma pesquisa nova sobre o **modelo de depósito** e a
**superfície de knobs**. A pesquisa foi feita (2026-07-12, 5 varreduras de fontes primárias) e está em
[**17_impasto_deposito_pesquisa2.md**](17_impasto_deposito_pesquisa2.md). Resumo em uma linha: **a
hipótese do handoff era certa — nenhum sistema sério deriva altura da opacidade macia da cor** (o nosso
modelo era o caminho "Smooth" do PS, o documentadamente ruim); a correção é um **perfil de corpo**
(platô + ombro) + **inclinação física** (sem gain mágico) + **matar o knob `Amount`**. O §10 abaixo é o
plano dessa correção.

> Handoff da troca de dono: [`docs/Painter/handoffs/HANDOFF_line_Painter_impasto_2026-07-12.md`](handoffs/HANDOFF_line_Painter_impasto_2026-07-12.md).

---

## As CONSTANTES que viraram lei (divergências do §10, cada uma com motivo e gate)

> Recortadas da §10.1, cujo corpo está no
> [arquivo](../archive/docs-2026-08-18/Painter/16_impasto_plano_implementacao.md).

**Divergências do §10 (decisões, não desvios — cada uma tem motivo e gate):**

1. **Os knots subiram: `W_TAIL/W_SOLID = 0.35/0.75`** (o §10 propunha 0.10/0.35). Com a parede no
   véu translúcido (10–35% de tinta), a iluminação forte dela multiplicava pixels que são
   majoritariamente PAPEL — o gate do halo (`impasto_light_shades_the_paint_not_the_paper…`) ficou
   vermelho em 20% de sobrevivência. O bevel do Photoshop é **inner** (corre da borda do matte para
   dentro, sobre pixels sólidos) pelo mesmo motivo: a parede sobe em tinta pigmentada (35–75%), e o
   véu fica plano com o pigmento intacto.
2. **O glint ganhou curva própria (`gloss_body`): specular SÓ no filme** (cobertura ≥ `W_SOLID`,
   rampa até 1). Com a inclinação sem mute, o Shine default punha o brilho na parede
   semi-translúcida e branqueava o véu — o halo voltando pela porta do specular. O diffuse continua
   modelando a parede; o glint cavalga a crista, que é a frase que já estava no comentário do
   `SHININESS`.
3. **`DEPTH_UNIT_PX = 16`** (Depth 0.7 ≈ 11 px de tinta sobre ~11 px de parede = bevel de 45°).
4. **O corduroy do ViewPlane foi ATENUADO, não morto** (fase-variância 1.0 → 0.70): com o corpo, os
   dabs sobrepostos ofertam platô cheio e o envelope guarda mais grão e menos fase — mas grão
   dab-relativo continua errado para `Depth Source: Grain` (gate re-derivada com os dois números e
   anti-vacuidade de textura na spine, medida 0.043).
5. **`paint.rs` estourou o teto de LOC do workspace (709/700) — dívida herdada** (o gate já estava
   vermelho no HEAD recebido, 712): `union_region` movido para o irmão `region.rs` (697/700).

**Gates novas (vermelho provado por mutação):** `impasto_soft_stroke_reads_as_a_body_with_an_edge`
(4 claims da definição de corpo — platô ≥0.98·spine a 25%, véu 0.0, concentração ≤40%, pico ≥8 na
parede; mutação A = `body_profile→identidade` reprova no platô) ·
`impasto_strokes_pile_up_only_to_the_glass` (3 cargas cheias = 2.0; mutação B = sem clamp reprova
em 3.0).

---

## Os ABERTOS nomeados que sobraram das fases arquivadas

### Persistência do documento pintado (§10.6)

**Aberto (nomeado):** o carimbo do id acontece no save, então um documento pintado e **nunca salvo** não
tem `PaintedDoc` — o **undo** já o preserva via `doc_cache` em memória, mas um crash antes do 1º save
perde tudo (é o comportamento de sempre; não regrediu). `SpriteSource::Atlas` (imagens importadas) e
`CookedTexture` seguem no caminho antigo (`collect_assets`), intactos.

### O pen-up e o `ModelSnapshot` (§11.5)

**Aberto (NOMEADO, e não é meu):** o `ModelSnapshot` clona **fundo os pixels de cada camada não-ativa**
(`images`) por traço — a mesma espécie, pré-existente, fora do escopo desta linha
([[feedback_audit_scope_discipline]]). O `canvas_rgba` já é `Arc`; `images` não.

**Continua fora:** conservação de volume real (a fase que a medição adiou) · luz na GPU (a perf não pede) ·
múltiplas luzes / IBL · relevo do PAPEL (**exige ordem nova do Enio**).

---

## 17. A FILA (ordem do Enio, 2026-07-12)

O Enio priorizou explicitamente. Ordem, e nada fora dela:

| # | Item | Estado |
|---|---|---|
| 0 | 🔴 **A UI do rig de luzes está MORTA** — os chips `1 2 3 4` pintam mas não respondem, nem o checkbox (Enio, 2026-07-12) | ▶ **AMANHÃ, primeiro** (§18.6) |
| 1 | ~~Múltiplas luzes~~ — a **matemática** landou (§18); só o acesso pela UI está morto | ✅ com o item 0 aberto |
| 2 | Passe de luz na **GPU** (`LayerOp` novo; há 8 slots livres em `AdjustmentKind ≤ 32`) | fila |
| 3 | Persistência do `h` no `ProjectState` | fila (herda o gap de `SpriteSource::Individual`) |
| — | **Relevo do PAPEL** | **exige ordem NOVA** do Enio (acopla impasto↔aquarela, §2) |
| **último** | **A TINTA EMPURRADA (Push)** — *"ainda não resolveu"* | ⏸ **FIM DA FILA, por ordem** (2026-07-12) |

**Sobre o Push:** a mecânica está correta (real-time, conservativa, viva, idempotente — §13) e o
**desenho** da tinta deslocada ainda não convence. **Não diagnosticar agora.** Enio: *"Adiar para o final
de toda essa implementação. Fim da fila."*

---

## 18.6 🔴 A UI do rig está MORTA — e o gate que faltava é o de sempre

**Enio, 2026-07-12 (print):** *"UI não funciona, nem o checkbox nem se pode selecionar outra luz."*

Os chips `1 2 3 4` **pintam** (o print mostra `1` selecionado, `2· 3· 4·` apagados) e **não respondem ao
clique**. O checkbox **Enable** também não — mas isso pode ser consequência: ele só é pintado quando a
lâmpada selecionada é ≠ 1, e não dá pra selecionar outra.

**Causa: NÃO IDENTIFICADA.** Duas hipóteses levantadas e **descartadas**:

- **Colisão de id** entre o `group_id` do segmented e o id da opção 1 (passei `PAINTER_IMPASTO_LIGHT_1`
  nos dois papéis) → **descartada**: `paint_segmented_adaptive` **ignora** o `group_id`; ele só mapeia
  `widget.options`.
- **Falta de `store.register` em `populate.rs`** ([[feedback_panel_populate_register]]) → **descartada**:
  os segmentos de **Depth Source** e **Draw To** também não estão lá e funcionam.

**A LIÇÃO, e é a terceira vez:** eu gatei a **MATEMÁTICA** do rig com 6 gates e 3 mutações vermelhas — e
**zero gates no SEAM da UI**. O `ph2d-ui-testkit` existe exatamente pra isto. Um teste headless que
**CLICA no chip** e afirma que `impasto_rig.selected` mudou teria saído **vermelho antes de o Enio abrir
o app**. É [[feedback_painted_is_not_populated_paint_gate]] + [[feedback_tool_unit_green_integration_dead]]
outra vez: *unit-verde ≠ funciona no produto*, e *pintado ≠ populado*.

**Ordem de amanhã (não negociável):**

1. **Escrever o gate do seam PRIMEIRO** — headless, clica o chip 2, afirma `selected == 1`; clica
   Enable, afirma `lights[1].on`. **Ele nasce VERMELHO.** Sem isso, qualquer fix é chute.
2. Só então diagnosticar (candidatos ainda não checados: a altura do `card_frame` — o segmented
   **reflui** em painel estreito e pode empurrar as linhas pra fora do card, e o card seguinte pintaria
   por cima, roubando os hit-rects; e a ordem dos arms em `handle_event`).
3. Consertar. É **UI pura** — não toca a matemática, e nenhum dos 6 gates do rig deve se mexer.
