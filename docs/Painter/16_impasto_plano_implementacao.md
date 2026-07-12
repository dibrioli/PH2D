# #16 — Impasto: PLANO DE IMPLEMENTAÇÃO

> **Aprovado pelo Enio (2026-07-12).** Design + pesquisa: [15_impasto_pesquisa_e_design.md](15_impasto_pesquisa_e_design.md).
> **Ordem do Enio, literal:** *"tudo o que for compatível no painel painter — toda a seção Shape, Grain,
> todo o Stroke e até suas shapes dinâmicas, o tiling, o mirror, o randomize color, os color ramps de
> shape e grain — deve ser integrado ao sistema, e isso já deve ser planejado desde já. O que não for
> compatível deve ser escondido. **Cuidado: Watercolor é uma implementação à parte e não deve ser tocada
> ou ferida.**"*

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

> Handoff da troca de dono: [`docs/HANDOFF_line_Painter_impasto_2026-07-12.md`](../HANDOFF_line_Painter_impasto_2026-07-12.md).

---

## 9. LANDOU (2026-07-12) — e onde a implementação DIVERGIU deste plano

Fases 1–3 fechadas + smoke armado. Commits `217aa592` · `c5878926` · `beb8a631` · `37d2258f` · smoke.
**As divergências abaixo são decisões, não desvios** — cada uma tem motivo e gate.

### 9.1 `depth` → `impasto_depth` (o plano ia cruzar dois sistemas)

O plano batizou o campo **`depth`**. Mas `BrushSpec` **já tem `depth`** — é a profundidade óptica
**Beer–Lambert da AQUARELA** ([spec.rs:195](../../crates/ph2d-painter-brush/src/spec.rs#L195)). Um `depth`
solto teria cruzado impasto com o wash em silêncio, exatamente o que a §2 proíbe. Todos os campos são
`impasto_*`.

### 9.2 `DepthSource`: 3 → **2** (o terceiro era um knob morto)

O plano listava `Uniform` / `Grain` / **`Shape`**. Para **qualquer pincel real** o `Shape` é duplicata
silenciosa do `Uniform`: sem slot Shape a silhueta **é** o falloff, e com Shape `Image` a imagem já
**substitui** o falloff — "só a silhueta" e "grão neutro" são o mesmo número. Seria um knob que não faz
nada: a espécie que a varredura de 2026-07-12 passou o dia inteiro exterminando (BUGS #13). **Cortado
antes de ser escrito.** Gate: `depth_source_uniform_is_level_and_grain_is_not` (vermelho provado nos dois
sentidos — Uniform comendo o grão *e* grão inerte).

### 9.3 T1.7 (Per-Layer Color) saiu **de graça** — o plano dava como o único caso que não sairia

A causa é a colocação do **choke point**: a altura é tomada em `stamp_dabs_inner` **acima de todo o
dispatch de rotas**, da silhueta-**união** em que as N camadas já se achatam. O relevo é **UM corpo
coerente**, não N degraus empilhados (o artefato que o plano temia). Gate:
`impasto_per_layer_color_leaves_one_coherent_relief`.

### 9.4 A REGRA que o plano não previa: **a altura não pode sortear do RNG vivo**

O frame aleatório do grão sai de `tex_rng`, um fluxo **persistente**. Um segundo passe que sorteasse dele
**adiantaria o fluxo** e a **COR** sairia com outro grão — marcar "Impasto" **repintaria o quadro**. O passe
roda numa **cópia** do fluxo e a descarta. O gate de byte-identidade **não pegava isso** (com impasto OFF
o passe nem chega no RNG); o gate que faltava é `impasto_on_does_not_disturb_the_pigment`, vermelho
provado escrevendo o RNG de volta. Documentada como **regra 2** em
[`impasto.rs`](../../crates/ph2d-tool-painter/src/tool/paint/impasto.rs).

### 9.5 A luz é **RELATIVA**, não absoluta (§4.2 do doc 15 ficaria errada)

A resposta do pixel é **dividida pela resposta de uma superfície PLANA**. Onde não há relevo o passe
multiplica por 1 e soma 0 — **byte-idêntico**. O ingênuo (`rgb × N·L`) **escureceria o quadro inteiro** ao
ligar a luz (plano a 45° devolve 0,707, não 1) — o bug de metade dos filtros de emboss já escritos. É essa
propriedade que dá dente ao gate.

### 9.6 Três furos que o código não denunciava (achados na costura, não na leitura)

- O **atalho de stack trivial** (`take_preview_arc`) devolve `canvas_rgba` **cru**. Documento de 1 camada é
  **o caso comum** ⇒ sem guard, o jeito mais ordinário de usar Impasto não mostraria relevo nenhum.
- **`gpu_eligible`** mandava pro compositor GPU, que não sabe da altura ⇒ esculpir sem ver.
- **`run_full`** (Apply) achata no sprite, que é só RGBA ⇒ sem assar a luz, Apply **jogava o relevo fora**.

### 9.7 `Smoothing` quase foi entregue **morto**

Declarado no spec, fiado até o painel, **lido por ninguém**. Agora assenta o depósito no fim do traço (box
separável, HR-5). Gate: parede mais íngreme cai >30% **e o volume se conserva** (a tinta espalha, não
evapora).

### 9.8 Perf — kill-criterion **passa sem otimização**

`@2048² r100`: custo do impasto **1,93 ms/move** médio, **2,13 ms** pior (frame inteiro: 2,98 ms).
Alvo ≤4 ms, kill em 8. **Medido, em `--release`, isolando o delta** (o frame já custa ~1 ms sem impasto —
cobrar isso do impasto o teria lisonjeado).

### 9.9 Smoke pronto

```bash
cd /home/enio/Documentos/Projetos/PH2D/Worktrees/line-Painter && \
  PH2D_IMPASTO_SMOKE=1 cargo run --release -p ph2d-host-desktop
```
Canvas branco 1024² já selecionado + pincel armado (Depth 0.7, source Grain sobre grão Noise). **Pegue o
Painter e arraste.** A seção **Impasto** já aparece no painel Brush com Enable marcado; o card **Lighting**
move a luz ao vivo, e **Depth negativo CAVA** em vez de levantar.

### 9.10 Continua fora do 1º corte (§6, inalterado)

`Plow` (arrastar relevo no Smear) · Composite Depth por camada (Subtract/Replace/Ignore) · passe de luz na
GPU · relevo do PAPEL (acopla impasto↔aquarela — **exige ordem nova do Enio**) · múltiplas luzes ·
persistência do `h` no `ProjectState` (herda o gap conhecido: o save já não persiste pixels de
`SpriteSource::Individual`).

---

## 10. Fase 4 — o CORPO (redesign pós-pesquisa 2, 2026-07-12)

> Fundamentação e fontes: [17_impasto_deposito_pesquisa2.md](17_impasto_deposito_pesquisa2.md).
> A medição que abriu a fase (probe no harness real, r=40, Depth 0.7, defaults): pincel macio default
> = domo puro com pico de shading de 7.3 níveis a 31% da meia-largura e 1 nível na borda visível;
> disco duro = platô com pico de 10.3 níveis a 97% — corpo, mas parede de 1 px. O alvo é o que nenhum
> dos dois é: **platô + ombro de largura orgânica, com a luz morando na borda.**

| Task | O quê | Onde | Gate VERMELHO refutável |
|---|---|---|---|
| **T4.1** | **Curva de corpo** no kernel: `body(w) = smoothstep(W_TAIL=0.10, W_SOLID=0.35, w)` substitui `w` cru na altura (cor intocada; cover intocado). Grain segue entalhando o resultado | `height.rs` | o gate de aparência T4.5; e `depth_source_uniform_is_level_and_grain_is_not` continua verde (platô segue nivelado, em cheio) |
| **T4.2** | **Teto de commit**: o Add entre traços satura em `±H_CEIL = 2.0` ("pressed against glass") | `impasto.rs::commit_stroke_height` | N traços de Depth 1.0 no mesmo ponto ⇒ `h == 2.0`, não `N` (RED: sem clamp, cresce sem teto) |
| **T4.3** | **Inclinação física**: morre `SLOPE_GAIN` (e o `× body` dentro da normal); nasce `DEPTH_UNIT_PX` (quantos px de tinta h=1.0 representa — **medido** no probe). `body_eff = min(1, cover/COVER_SOLID)` pesa só o EFEITO | `impasto_light.rs` | byte-identidade do plano intacta; halo do branco não volta (o h já é 0 na cauda <10% — T4.1); gate T4.5 dá o número |
| **T4.4** | **`Amount` morre** (o gêmeo acoplado do Depth): id, row, setter, rota, reset, snapshot, campo — o desmonte dos 7 sites, com as gates de wiring acusando resto | `painter_impasto.rs` (ids) · `paint_impasto.rs` · `impasto_settings.rs` · `paint.rs` | `architecture_panel_wiring_parity` + seam tests compilam sem o id; grep do id = 0 ocorrências |
| **T4.5** | **Gate de APARÊNCIA (derivado da definição, não do shader):** um corpo tem borda iluminada — no traço soft default, o pico de \|Δ\| fica na **banda do ombro** (fora de 60% da meia-largura) e ≥ um piso de níveis; o interior é platô (`h` a 50% da meia-largura ≥ 0.9× o spine). RED provado com o kernel antigo (domo) | `tests.rs` (promove o probe) | é o próprio gate |
| **T4.6** | Fixture de `impasto_light_reads_as_raised_not_engraved` acha os flancos pelo **gradiente real de h** (com platô, os flancos fixos de hoje caem no interior plano) | `tests.rs` | o teste continua provando claro-do-lado-da-luz/escuro-do-outro |
| **T4.7** | Re-medir perf (§7 continua valendo: ≤4 ms/move, kill 8) + re-rodar o probe e registrar os números aqui | — | `impasto_perf_kill_criterion` |

**Fora do corte (inalterado do §6):** `Plow` (agora com receita barata documentada — o deslocamento
degenerado do Corel Thick Paint, pesquisa 2 §3.6) · Composite Depth por camada · luz na GPU · relevo do
papel (ordem nova do Enio) · persistência do `h`.

### 10.1 LANDOU (2026-07-12) — números medidos do fechamento, e onde a implementação DIVERGIU do §10

Probe re-rodado (mesmo cenário da abertura — soft default r=40, Depth 0.7, elev 45°, diffuse-only,
`--release`; a probe varre **só o lado iluminado**, em tinta escura — o teto físico desse lado a
elev 45° é mul ≤ 1/0.707, ~14 níveis em tinta escura; o range do look mora no lado da sombra, −65%,
e no glint):

| métrica | antes (domo) | depois (corpo) |
|---|---|---|
| pico de \|Δ\| no soft | 7.3 níveis a **31%** da meia-largura | **13.0 níveis a 42%** — em cima da PAREDE |
| \|Δ\| na borda visível (15% cover) | 1.0 nível | **0.0** (o véu não carrega relevo — halo impossível) |
| concentração (linhas com \|Δ\|≥3) | 62% da largura (borrão) | **22%** (borda) |
| perfil de h (spine→50%→75%→90% da meia-largura) | 0.70 → 0.39 → 0.16 → 0.07 | **0.70 → 0.37 (meio da parede) → 0.00 → 0.00** |
| disco duro | pico 10.3 a 97%, interior 0 | **12.0 a 97%, interior 0** (intacto) |

**Perf:** impasto **1.79 ms/move** @2048² r100 (`impasto_perf_kill_criterion`; alvo ≤4, kill 8).
**Suítes:** 239 `ph2d-painter-brush` + 584 `ph2d-tool-painter` + painel + editor-core, tudo verde;
byte-identidade OFF, barreira do Watercolor, Tiling/Symmetry, RNG, eraser, undo — intactos.

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

### 10.2 Fase 4.1 — o dial **Body** (smoke do Enio, 2026-07-12, mesmo dia)

O smoke sobre a Fase 4 aprovou o corpo mas achou a perda: *"parece ter perdido a capacidade de
obedecer toda a suavidade do falloff — não consigo relevos perfeitamente arredondados como antes."*
Ele está certo: a curva de corpo esmagava TODO perfil para platô+parede — a promessa do §0 ("a ramp
vira escultura") morreu junto, e o estado-da-arte não faz isso: o perfil é ESCOLHA (PS `Technique`
Smooth↔Chisel; Blender Draw vs Layer brush — pesquisa 2 §3).

**Fix:** `BrushSpec.impasto_body` (0..1, default **1.0** = o look da Fase 4) — slider **Body** no
card, entre Depth e Smoothing. `a = lerp(w, body_profile(w), body)`: em **0** o relevo obedece a
silhueta por inteiro (falloff, Shape image e Shape-Tone ramp esculpem — o arredondado perfeito; e
sob a LUZ NOVA, sem o mute quadrático antigo, o domo sombreia de verdade — a combinação
domo+luz-física nunca tinha existido); em valores médios, a família mesa (Chisel Soft). Blend
monótono de remaps monótonos ⇒ continua comutando com o envelope. Aplica do PRÓXIMO traço (o
envelope armazenado não carrega o `w` cru para re-derivar — diferente do Depth, que é rescale puro).

Gate: `impasto_body_zero_obeys_the_falloff` (domo sem platô + flanco monótono + cauda com relevo +
o véu quase-invisível segue sem sombra — RED provado por mutação `body_mix→1`). Superfície: brush
agora `Enable · Depth · Body · Smoothing · Source · Draw To` (o Body é percepção de FORMA, não um
segundo ganho — não reabre o acoplamento que matou o Amount). Perf re-medida: **1.87 ms/move**.

### 10.3 Fase 4.2 — TODO parâmetro do relevo é vivo (Enio, 2026-07-12)

> *"Coloque todos os parâmetros vivos em tempo real para ajustes depois do traço."*

**O que impedia:** o traço guardava a ALTURA. Depth era vivo porque é rescale puro dela; **Body** e
**Depth Source** ficavam *assados* no depósito, pixel a pixel — não sobrava nada de onde re-derivá-los.
Fazer cada um "vivo" na mão seria N casos especiais que apodrecem um a um.

**A correção estrutural — o traço guarda os INGREDIENTES, não o resultado:**

| plano | o que é | por que existe |
|---|---|---|
| `stroke_paint` (f32, 0..1) | quanto de TINTA cada pixel recebeu (silhueta × dinâmica), envelope por `max` | a 1ª entrada; é também a cobertura que a luz pesa |
| `stroke_grain` (u8, 255 = sem grão) | o grão que o dab VENCEDOR amostrou naquele pixel | a 2ª entrada; é o que deixa flipar Depth Source depois e re-entalhar os sulcos daquele dab |

E o relevo é **sempre** `derive_height(spec, paint, grain)` — uma função pura. Depósito e edição
passam pela MESMA função, então não podem divergir. Consequência: **Depth · Body · Depth Source ·
Smoothing** editam o último traço ao vivo, e nenhum é caso especial. (Lighting — Show/Angle/Elevation/
Shine — já era vivo: re-ilumina todo frame.)

**Duas decisões que caem disso:**
1. **O envelope passa a ser tomado na TINTA**, não na altura: o dab que depositou mais tinta é o dono
   do pixel. Vencedor escolhido por uma grandeza que *nenhum setting muda* ⇒ re-derivar num Body/Source
   novo não re-embaralha qual dab moldou qual pixel. (Antes: `max |h|`.)
2. **O perfil roda na TINTA (`w × dinâmica`), não na silhueta crua** — a mesma grandeza que a luz usa
   para pesar a sombra. Geometria e sombreamento não podem mais discordar sobre onde a tinta fica
   sólida; e toque leve agora deposita tinta mais fina *e* de borda mais macia, que é o que toque leve
   faz. (Com pressão cheia é idêntico ao anterior — as gates provam.)

**`Draw To` continua NÃO-vivo, de propósito:** não é propriedade da tinta, é *qual canal o pincel
escreve*. A metade da cor é irreversível (o pigmento já está na camada), e tornar só a metade da
altura retroativa apagaria relevo que o artista está vendo — meia-verdade pior que a regra clara.

**Gate (RED provado em 2 mutações):** `impasto_every_body_knob_edits_the_last_stroke_live` — para cada
knob, *girar depois do traço* == *ter pintado com ele desde o início* (worst pixel < 1e-5), com
anti-vacuidade (o knob tem de mover ≥200 px). Mutação D (Body sem `refresh`) → vermelho por 0.218 de
diferença; mutação E (não guardar o grão) → vermelho já na anti-vacuidade (Depth Source vira knob
morto). No kernel: `every_body_knob_is_a_pure_function_of_the_stored_ingredients` (3 body × 2 source ×
3 depth = 18 combinações re-derivadas contra depósitos frescos).

**Perf: 1.66 ms/move** (melhorou — o kernel escreve menos por pixel). `height.rs` 761→511 LOC (testes
p/ `height_tests.rs`, precedente `spec_tests.rs`); `paint.rs` em 700/700.

### 10.4 Fase 4.3 — **Shine estava morto** (smoke do Enio: *"shine não funciona"*)

Ele estava certo, e a causa era **geométrica, criada por mim na Fase 4**: o relevo só tem declive na
faixa de cobertura `W_TAIL..W_SOLID` (isso *é* a parede) — e eu tinha gateado o glint **acima** de
`W_SOLID`, ou seja, permitido só no **platô**, que é plano, onde o passe faz early-out e não adiciona
nada. Medido antes de mexer (regra da casa): **94% dos pixels com declive ficavam abaixo do gate**, e
Shine 0 → 1 movia o pixel mais brilhante **1 nível**. Knob morto por construção — a espécie que a
varredura de 2026-07-12 passou o dia exterminando, e eu a reintroduzi.

**Fix em duas partes** (as duas necessárias — consertar uma sozinha foi como o knob morreu):

1. **O glint usa a MESMA curva de corpo do difuso** (`gloss_body = paint_body`): sobe pela parede e
   senta na crista, que é onde tinta a óleo brilha. Ganho: 1 → **160 níveis**.
2. **O highlight deixa de ser aditivo puro e passa a somar contra o headroom (screen):**
   `lit = lit + add·(1 − lit)`. O aditivo plano era o que trazia o halo de volta pela porta do
   specular: num pixel do véu o canal vermelho já está no teto, então a soma só levantava os OUTROS
   canais e o pigmento colapsava para branco (gate do halo vermelha em **19%** de sobrevivência).
   Screen escala o ganho de cada canal pelo espaço que sobra — canal saturado quase não muda, a tinta
   guarda a cor, e a crista acende. É também o que um highlight real faz: aproxima-se do branco, não o
   ultrapassa.

**Gate novo:** `impasto_shine_glints_on_the_wall_without_bleaching_the_rim` — 3 claims, **cada uma com
vermelho provado**: (a) o glint é visível (≥40 níveis; mutação F = glint no platô → **1 nível**, o bug
do Enio reproduzido); (b) ele pousa em tinta **com declive e com corpo** (não no platô nem no véu);
(c) em Shine **máximo** o passe é **no-op estrito sobre o véu translúcido** (`cover < W_TAIL`) —
mutação H' (piso na curva de corpo) → **900 canais movidos**. A mutação G (aditivo plano) é pega pela
gate do halo (19%), e o comentário do teste diz explicitamente qual gate é dona de qual claim.

**O que o gate deliberadamente NÃO afirma:** que um highlight nunca lava uma parede iluminada em
direção ao branco. Ele lava — *é isso que highlight é*. O pior pixel de uma versão mais estrita desta
asserção era tinta a **70% de cobertura** cujo vermelho o **difuso** já tinha levado a 255; a métrica
de croma não sabe distinguir um glint honesto do halo. O look default fica protegido pela gate do halo
(que roda no Shine default 0.3); Shine alto sobre canvas branco lava mesmo — é escolha do artista.

Perf: **1.67 ms/move**. Byte-identidade preservada (screen com `add = 0` e `mul = 1` devolve o pixel).

### 10.5 Defaults do artista + o relevo viaja com o documento (2026-07-12)

**Defaults (ordem do Enio, dialed-in no smoke):** brush `Depth 1.0` · `Body 0.0` (o relevo obedece o
falloff — o arredondado) · `Smoothing 1.0`; canvas `Angle 230°` · `Elevation 30°` · `Shine 0.7`.
Espelhados nos 4 sites (`BrushSpec::default`, `PaintState::default`, `reset_brush_impasto`,
`brush_fallback` do painel). O `impasto_smoke` deixou de re-armar Depth/Smoothing: **arma só o switch e
o Grain source**, para que o smoke mostre o que um pincel novo faz — arma-lo escondiria um default ruim
atrás de uma demo boa.

**BUG achado no caminho (meu, da Fase 1) — o relevo não viajava com o documento.** `StashedDoc`
guardava `images` mas **não** `heights`/`covers`, e as chaves são `RtLayerId`, que `LayerStack::new()`
reinicia em 1 ⇒ **os ids de dois documentos colidem por construção**. Consequências: (a) trocar de
sprite e voltar **perdia** a escultura (o `is_trivial_stack` chamava de "descartável" um doc de 1 camada
COM relevo — mas o sprite é só RGBA, não reconstrói canal de altura); (b) ir para um sprite **cacheado**
passava por `restore_doc`, que não re-sourceia, então o relevo do sprite anterior **ficava** e iluminava
a tinta do novo. É a espécie do Bug #13.c. Fix: `heights`/`covers` entram no `StashedDoc` (take no
stash, **replace** no restore) + `doc_is_disposable() = is_trivial_stack() && heights.is_empty()`.
Gate `relief_travels_with_its_document_and_is_never_lent_to_another` — as duas barreiras são **defesa em
profundidade** (cada uma sozinha já bloqueia o empréstimo; o gate só fica vermelho quando as DUAS caem,
o que a varredura de mutação mostrou — nenhuma das duas linhas é decoração).

**O halo, de novo — e a gate reescrita (esta é a lição):** com `Body 0` o relevo passa a existir sobre a
tinta translúcida, e ali o realce difuso encontra um pixel cujo canal do pigmento **já está no teto** —
só os outros sobem, e o rosa vira branco. Medido: 21% de sobrevivência do pigmento na orla (e **23% com
Shine 0** — logo **não é o specular**). Tentei pesar o GANHO mais duro que a sombra (`body²`): comprou
5 pontos e **matou o modelado** (o flanco iluminado empatou com a tinta sob ele). Revertido, e a
conclusão é honesta: **clarear tinta translúcida sobre papel branco custa saturação, em tinta como na
física** — não é defeito, é a luz. O que continua indefensável é outra coisa, e é isso que a gate agora
mede: **tinta SEM corpo (`cover < W_TAIL`) não recebe nem um byte de luz, em qualquer setting** (`Depth
1` / `Body 0` / `Shine` no talo). RED por mutação (piso na curva de corpo → 900 canais movidos).

**E o screen ganhou gate EXATO, em álgebra, sem limiar de imagem:** `screen(v) = v(1−add) + add` ⇒
`screen(R) − screen(G) = (R − G)(1 − add)` — o matiz é preservado **exatamente** e o croma só escala,
para todo `add < 1`. O aditivo plano clampa e aniquila. A tentativa anterior (contar pixels lavados)
separava mal (69 vs 131) e teria sido um gate frágil; a álgebra separa sempre.
`the_highlight_scales_chroma_and_never_annihilates_it` (unit em `impasto_light`), RED com `v*mul + add`.

Perf: 1.94 ms/move. 43 suítes verdes, clippy 0.

### 10.6 PERSISTÊNCIA do documento pintado (2026-07-12) — a pintura passa a sobreviver ao Ctrl+S

**O que estava quebrado (pior do que o gap conhecido):** o projeto salvava o mundo e os pixels dos
sprites *importados*, mas **nada** do que o Painter pintava. Um sprite pintado é
`SpriteSource::Individual { texture_id }`, e esse id é de **runtime da GPU** — noutra sessão aponta
para um slot vazio. Pintar → `Ctrl+S` → reabrir devolvia o quadro **em branco**: nem o bake achatado
sobrevivia, muito menos as camadas e o relevo. (O `docs` dizia "Individual fica fora do 1º corte"; a
consequência real nunca tinha sido escrita.)

**A correção tem 3 peças, e a do meio é a que faltava:**

1. **Identidade ESTÁVEL** — componente nova `ph2d_ecs::PaintedDoc(u32)` (módulo irmão
   `painted_doc.rs`, molde do `VecPathRef`), carimbada no sprite **no save**. Os bits de entidade (a
   chave do `doc_cache` do Painter) são id de **alocação**: o restore despawna tudo e recria, então
   morrem. A componente viaja no `WorldSnapshot` ⇒ sobrevive ao arquivo **e ao undo**. Registrada no
   `ComponentRegistry` (26→**27**; o gate de contagem acusou, que é o trabalho dele — componente
   não-registrada é descartada em SILÊNCIO).
2. **O DOCUMENTO no arquivo, não o bake** — `PaintedDocument` (`tool/persist.rs`): `layers` + os
   pixels de cada camada + **`heights`/`covers`**. Salvar só o sprite achatado devolveria uma
   *fotografia* de tinta grossa: sem camadas, sem espessura, sem como continuar esculpindo. Único
   tipo que faltava serde: `LayerImage` (3 campos). Undo/caches **não** entram (histórico é da sessão).
3. **A textura re-materializada no load** — o documento é instalado, **composto pelo caminho NORMAL do
   preview** (`bind_document` + `take_preview_arc`, que já assa a luz do impasto) e sobe para um slot
   novo do `IndividualTextureStore`; o `Sprite.source` re-aponta. **Deliberadamente não existe um
   segundo bake** — escrever um seria criar um segundo caminho para a mesma imagem, e é assim que dois
   caminhos divergem seis meses depois (a lição que esta linha já pagou duas vezes).

`PROJECT_SCHEMA` 2→**3** (postcard é posicional; sem saves publicados, quebra livre).

**Gate:** `a_painted_document_survives_the_disk_with_its_relief` — pinta um traço esculpido numa 2ª
camada, coleta pelo mapa de ids estáveis, **serializa em postcard de verdade** (dev-dep nova; um teste
noutro formato não provaria o arquivo que o artista salva), instala num tool **novo** sob **bits de
entidade diferentes** (o que o restore faz) e afirma: as camadas voltam **e o relevo volta idêntico** —
ainda editável, não uma foto.

**Aberto (nomeado):** o carimbo do id acontece no save, então um documento pintado e **nunca salvo** não
tem `PaintedDoc` — o **undo** já o preserva via `doc_cache` em memória, mas um crash antes do 1º save
perde tudo (é o comportamento de sempre; não regrediu). `SpriteSource::Atlas` (imagens importadas) e
`CookedTexture` seguem no caminho antigo (`collect_assets`), intactos.

#### 10.6.1 Fix do 1º smoke da persistência: **a textura reabria gigante** (Enio)

*"Save funciona mas abre uma textura gigante e não do mesmo tamanho."* Bug meu, de uma linha: no
re-ataque do sprite eu escrevia `sprite.size = [w, h]` — mas `Sprite.size` é a pose em **unidades de
mundo (metros)**, e `w`/`h` são as dimensões da **textura, em pixels**. Um canvas de 1024 px a 100 px/m
mede 10,24 m; o load o reabria com **1024 metros** de lado (inflado pelo fator `pixels_per_meter`).

O `size` correto **já vinha no snapshot** — o documento não tinha nada a dizer sobre ele. A correção é
não tocá-lo: o re-ataque virou `reattach_texture(sprite, texture_id)`, que troca **só** a textura (e a
flag de premultiplicado). *Um documento diz o que está pintado num objeto; ele não redimensiona o
objeto.* Gate `reattaching_a_document_never_resizes_the_sprite` — RED por mutação com o próprio bug
(pose 1024 m vs 10,24 m esperados).

### 10.7 **PLOW** — a espátula (2026-07-12): a última peça que a pesquisa mapeou

Até aqui o Smear arrastava a **cor** e deixava o **corpo** onde estava: tinta grossa era *imexível*
depois de pousada, e a luz seguia sombreando uma crista que o pigmento já tinha abandonado. Plow é o
gesto que faltava — Corel expõe o parâmetro com esse nome (*"a brushstroke with a high Plow value…
displaces the depth of the existing brushstroke"*), e a lâmina chata do ArtRage é o mesmo gesto
(pesquisa 2 §3.6).

**Mecanismo (zero geometria nova):** `plow_dab_height` é o **mesmo lift-and-drag** do smear de cor
(`smear_dab`) — fotografa a região de origem em `from`, funde no footprint em `to`, pesada por
falloff × strength — só que sobre `h` **e** `cover`. Isso não é atalho, é o ponto: a faca move tinta, e
o pigmento e o **corpo** daquela tinta têm de se mover **juntos**, pelo mesmo deslocamento, sob a mesma
máscara. Dar ao relevo uma lei de deslocamento própria seria deixar a luz descolar da cor — sombras
flutuando ao lado da crista a que pertencem. Por isso a rota consome o **mesmo `last_smear_pos`** (e
roda ANTES de a cor avançar a cadeia, senão o corpo atrasaria um dab).

**Deslocamento, nunca depósito:** a faca não usa `Depth` nem o master switch do Impasto — ela move o que
já está lá, *quem quer que tenha posto*. Default `Plow = 0` (o Smear de sempre, byte-idêntico).

**UI honesta:** no Smear o card **Body inteiro some** e entra **um card `Knife` com uma linha (`Plow`)**
— e não o card Body com linhas esmaecidas: uma faca não tem Depth, Draw To nem Depth Source, e mostrar
três controles desabilitados é mostrar ao artista três explicações de por que eles não valem. (Controle
esmaecido ainda hit-registra; a regra da casa é: **o que não se aplica não é pintado**.) O predicado é
publicado pelo tool (`impasto_plow_applies`), não re-derivado pelo painel.

**Gates (RED por mutação, os dois):** `impasto_plow_drags_the_relief_with_the_paint` — (a) sem Plow o
relevo **não se move** (o default preservado); (b) com Plow a faca **carrega o corpo** para onde não
havia tinta grossa; (c) **a cobertura viaja junto** (mutação: relevo sem cobertura ⇒ a crista-fantasma
que o gate da borracha já recusa, entrando por outra porta). E a matriz de §1.2 virou **exclusividade**:
em Paint vale o Body, em Smear vale a faca, **nunca os dois**.

Perf: **1.99 ms/move** (a faca é O(footprint), sem solver). Fica **fora**: conservação de volume real
(IMPaSTo/WetBrush — a crista que se ergue à frente da lâmina). O que existe é a versão degenerada e
barata, que é exatamente o que a Corel faz.

---

### 10.8 **Composite Depth por camada** (2026-07-12) — o relevo vira parâmetro de composição

Fechado o item que o §6 nomeava como o próximo (`Composite Depth por camada` + escala por camada).

**O buraco que ele tapa.** O Depth do pincel é **assado em cada traço** no momento em que ele pousa; o
re-derive vivo (§10.3) alcança **só o último**. Ou seja: no instante em que você dá a 2ª pincelada, a
espessura da 1ª está congelada — e até aqui **nada no produto voltava a tocá-la**. O Depth de camada
alcança: é um parâmetro de **composição**, então age sobre *tudo que já foi esculpido naquela camada*,
para sempre, sem re-esculpir um texel. É a barganha da opacidade, um eixo ao lado: `0` **muda**, não
apaga — e o gate termina subindo de volta e exigindo a escultura de volta **bit a bit**.

**Onde mora.** Linha 3 da row da camada, no painel de Layers — ao lado da opacidade, no mesmo formato
(slider bare + leitura + um chip). A profundidade da tinta de uma camada é a *opacidade da espessura
dela*; não pertence a um painel modal noutro lugar. E ela só é pintada em camadas **que têm relevo**
(`Layer::has_relief`) — documento que ninguém esculpiu não mostra chrome de impasto em lugar nenhum.

**4 modos viraram 2 — e isso é o achado.** O plano pedia `Add`/`Subtract`/`Replace`/`Ignore`. Três deles
são **leituras de um único número com sinal**: `+` empilha, `0` muda, `−` cava. Um enum que duplica um
slider é *exatamente* o segundo-ganho que esta seção já teve de matar uma vez (o antigo "Amount", §10.1).
Sobra o único que a escala **não** consegue dizer:

- **`Level`** — a tinta espessa e opaca desta camada **soterra** a textura debaixo dela em vez de herdá-la,
  pesada pela própria cobertura (`h = h_abaixo·(1−c) + h_meu·c`). É o "composite, don't add" da pesquisa
  ([17_impasto_deposito_pesquisa2.md](17_impasto_deposito_pesquisa2.md)). Onde a tinta é sólida a superfície
  é **desta** camada; onde ela é rala, o que está embaixo aparece intacto — senão uma região vazia de uma
  camada `Level` achataria o quadro inteiro.

**A ordem do fold virou carga.** `Level` **não comuta**. Enquanto o composite era só uma soma, o fold podia
iterar o mapa de alturas em ordem de **chave** — e ninguém percebia, porque soma comuta. Agora ele caminha a
**ordem-z** (`LayerStack::z_order_bottom_up`, de baixo pra cima, recursivo nos grupos), e o traço vivo entra
**no slot da camada ativa** (sob o Depth dela), não empilhado por cima de tudo. O gate `impasto_level_buries_
what_is_under_it_in_the_stacking_order` é construído para que **ordem de id e ordem-z discordem** (duas camadas
criadas numa ordem, empilhadas na outra): folde por chave e a resposta sai errada — mutação provada vermelha
(0.5 em vez de 0.25).

**O seam que quase matou tudo em silêncio.** O painel só reaprende a stack quando `layers_revision` muda — e
**uma pincelada é edit de PIXEL**, não bump de revisão. Esculpir a 1ª crista numa camada acenderia o flag e o
painel **nunca ficaria sabendo**: a linha de Depth só apareceria depois que algum edit de camada não-relacionado
bumpasse a revisão por acidente. Um teste que só lê o flag fica **verde** com esse bug ([[feedback_tool_unit_
green_integration_dead]]) — então o gate lê a **revisão** também, e a mutação (`sync_relief_flags` sem
`invalidate_composite`) derruba.

**Gates (5 mutações, todas provadas vermelhas):** (A) fold ignorando `l.depth` · (B) `Level` = `Add` · (C)
`sync_relief_flags` fora do depósito (flag mente) · (D) fold em ordem de chave · (E) revisão não bumpada
(painel nunca sabe). Mais o gate de persistência estendido: `impasto_depth`/`impasto_composite` atravessam o
disco (**PROJECT_SCHEMA 3 → 4** — postcard é posicional; um arquivo v3 leria os bytes do campo seguinte).

**Perf:** 2.35 ms/move (era 1.99 — o fold ordenado custou ~0.36 ms). Alvo ≤4, kill em 8: passa.

**Débito latente pago no caminho:** os gates de contagem de componentes ECS de `ph2d-render` e `ph2d-script`
estavam **vermelhos desde `0a90ed31`** (o `PaintedDoc` registrado na persistência: 27 → 28) e a linha reportou
verde. `nextest-impacted` não os toca; só `cargo test --workspace` pega. Lição já catalogada
([[feedback_ship_parity_gaps_ci_only]]) — e desta vez ela cobrou.

**Continua fora do 1º corte:** passe de luz na GPU (perf não pede) · conservação de volume real da faca ·
relevo do PAPEL (**exige ordem nova do Enio** — acopla impasto↔aquarela) · múltiplas luzes / IBL.

---

### 10.9 **"Smoothing nem sempre se aplica no fim do traço"** (smoke do Enio, 2026-07-12) — a palavra era *nem sempre*

**O que ele viu.** O Smoothing às vezes funcionava, às vezes não.

**Onde NÃO estava.** Na aritmética. O settle é aplicado num único lugar — `rebuild_live_layer_relief`,
alcançado por `commit_stroke_height` — e **incondicionalmente**. O que varia não é o cálculo: é **se aquele
commit roda**.

**A causa.** Os cinco métodos de **FORMA** (Line · Arc · Ellipse · Polygon · **Free Hand**) mantêm o traço
**ABERTO** no pen-up de propósito — a forma continua editável até o Apply. Então `close_stroke` (e com ele
`commit_stroke_height`) **nunca disparava para eles**. Os métodos de mão livre (Space/Dots/Airbrush/
Anchored/DragDot) comitam no pen-up e assentavam normalmente. **Daí o *nem sempre*.**

Três consequências, e o Smoothing era só a visível:

1. **Smoothing morto** nas 5 formas — a luz lia o **envelope cru**.
2. **O card Body inteiro morto** nelas (Depth/Body/Source ao vivo): os *ingredientes* nunca eram entregues.
3. **Pior, e medido:** o relevo ficava em `paint.stroke_height` **sem dono** — o próximo pen-down o
   **apagava** (`reset_stroke_height`). Aplique uma curva, comece outro traço, e a espessura da primeira
   **evaporava**: o pigmento ficava, o corpo sumia.

**O fix, em dois pontos — e são os dois chokepoints, não cinco call-sites:**

- `commit_drag_preview()` (**onde um desenho vira canvas**) passa a chamar `commit_stroke_height()`. Um
  ponto, todos os métodos, Apply **e** Apply & Keep. Pigmento e corpo são duas saídas da MESMA lista de
  dabs — viram permanentes no mesmo instante. (O caminho de mão livre também passa por aqui, logo antes do
  `close_stroke`, cujo `commit_stroke_height` então acha os ingredientes já tomados e vira no-op.)
- `cancel_open_shape()` / `discard_open_shape()` **largam o envelope**. O Esc devolve os pixels ao pristino,
  então o relevo tem de ir junto — senão sobra **crista sem tinta**, o mesmo fantasma que o gate da borracha
  recusa, entrando pela tecla Esc.

**Gates (mutações provadas vermelhas):** (F) `commit_drag_preview` sem o commit do relevo — derruba
*"Smoothing must SETTLE"* e *"the body evaporated"* · (G) cancel sem largar o envelope — derruba a crista
fantasma. O gate principal é uma **TABELA sobre os 10 métodos**, porque o bug nunca esteve no código escrito:
esteve nos **caminhos que ninguém conectou**. Uma 6ª forma amanhã, sem commit, fica vermelha aqui.

**Lição de fixture (a 6ª desta linha):** o `Line` é uma **polilinha** (clique-a-clique), não um arrasto —
dirigi-lo com um drag não pinta **nada**, pigmento incluso. O gate nasceu vermelho por isso, e a fixture
estava errada, não o tool.

---

### 10.10 **"O 1º traço assenta; do 2º em diante, só quando mexo no slider"** (2ª volta do smoke, 2026-07-12)

O §10.9 fechou **quais** traços comitam. Este fecha **se alguém repinta o que o commit trocou** — e é a
falha que o gate do §10.9 **não continha**, porque a fixture tinha **um traço só**.

**A causa.** No pen-up o relevo debaixo da pintura é **trocado**: o envelope cru com que o traço foi
desenhado vira o campo **assentado**. **Nenhum pixel mudou** — então nada naquele caminho marcava o canvas
sujo, e o cache do composite seguia mostrando a iluminação que desenhou **durante** o traço, feita do relevo
**não-assentado**. Mexer em qualquer knob do Body chama `refresh_live_relief`, que **invalida o composite** —
e aí o smoothing aparece, **atrasado**. Exatamente o que o Enio descreveu.

**Por que o PRIMEIRO traço funcionava — e é aqui que mora a lição.** Ele vira o `has_relief` da camada
(§10.8), e essa troca de flag invalida o composite **de efeito colateral**. O primeiro traço estava sendo
salvo **por acidente**. Logo uma fixture de **um traço** não pode conter este bug — e a que escrevi no §10.9
não continha, e passou **verde por cima de um defeito vivo**. O fenômeno mora no **segundo** traço; o gate
agora pinta **três**.

> **Lição durável (nova):** quando o 1º caso é salvo por um efeito colateral (uma invalidação de cache vinda
> de outra mudança), ele passa e os seguintes falham. **Teste a REPETIÇÃO, não a primeira ocorrência.**

**O fix.** `rebuild_live_layer_relief` suja **exatamente o que moveu**, diferindo o campo novo contra o que
ele substitui (`relief_diff_rect`). Não a bbox das dabs — **o settle é um blur, ele se espalha para fora da
tinta** — e não um `invalidate_composite`, que largaria todo cut-cache de ajuste **uma vez por traço** (o
caminho de 55 ms num documento pesado). O diff conhece o alcance do blur sem ninguém ter de codificá-lo.

**Gates:** (H) o re-derive sem sujar nada — **vermelho** (1574 canais divergem no 2º traço, pior caso 101
níveis). (I) o grow de 1 px — **não gateável, e está escrito assim no código**: apagá-lo deixa a suíte verde
e não consegui construir vermelho (o único chamador re-deriva por um blur, então a mudança decai a
`RELIEF_EPS` na borda da caixa e o vizinho muda menos que um byte). Fica como **margem de correção**, não
como fix observado.

Perf: **2.17 ms/move**.

---

## 11. Fase 9 — **o pen-up era O(canvas)** (2026-07-12): 1010 ms → 12 ms

Fase escolhida **medindo antes** ([[feedback_measure_perf_symptom_scale]]), e a medição mudou a fase: eu ia
fazer conservação de volume, e o relógio disse outra coisa.

### 11.1 O que o kill-criterion não estava olhando

O §7 congelou o critério em **2048², e só o `Move`**. Medido em 2026-07-12, com o `Up`:

| canvas | por-movimento | **pen-up** |
|---|---|---|
| 2048² | 2,2 ms | **146 ms** |
| 4096² | 2,2 ms | **1010 ms** |

**Um segundo inteiro de congelamento no fim de cada traço em 4K** — e invisível para um gate que só
cronometrava o arrasto. *Um orçamento cuja outra metade ninguém gasta não é um orçamento.* O critério agora
mede o pen-up **e** roda em 4096².

### 11.2 Onde o tempo morava (instrumentado, não adivinhado)

Em 4096²: `settle` **258 ms** · `derive` 16 ms · `diff` 6 ms · clone da base **64 MB** · merge de cobertura
O(n) · snapshot de undo 12-16 ms · **full-recompose de 225 ms** disparado pelo flag `has_relief` ao virar.

Tudo O(canvas) — para um traço que tocou **0,2 M de 16,7 M texels**. O `settle` era um blur separável sobre
o canvas inteiro num buffer **zero em tudo menos o traço**.

### 11.3 O corte, e por que ele é EXATO

O commit passa a trabalhar numa **janela**: a bbox das dabs do traço, **crescida pelo alcance do blur**
(`SETTLE_REACH_PX` = `SETTLE_MAX_PX`). Não é aproximação:

- fora da janela a tinta é zero ⇒ o relevo é zero ⇒ **não há o que derivar nem o que escrever**;
- o blur de zeros é zero, e o box blur **clampa na borda do seu buffer** — numa janela cuja borda já é zero,
  o clamp replica **o mesmo zero** que o passe de canvas inteiro leria de fora dela.

⇒ **byte-idêntico**, e o gate diz exatamente isso: `impasto_the_stroke_commit_is_cropped_to_the_stroke_and_
byte_identical` compara o commit cortado contra o **mesmo `derive_height` e o mesmo `settle`** rodados sobre
o canvas inteiro (a referência **é** o caminho que isto substituiu — não um oráculo re-implementado), com o
traço **saindo pela borda do canvas**, que é onde um crop tem mais chance de divergir. Mutação (janela sem o
crescimento) = **vermelha**, pior texel errado por 0,009.

### 11.4 Os outros dois

- **O martelo de 225 ms.** `sync_relief_flags` chamava `invalidate_composite()` para publicar **um booleano**
  ao painel — largando o composite inteiro e todo cut-cache de ajuste, i.e. um recompose completo do canvas,
  na primeira pincelada de cada camada. O painel só precisa da **revisão** (`bump_layers_revision`); o relevo
  recém-deposto já está na tela, porque o re-derive sujou exatamente os texels que moveram. **Não é gateável
  por teste** (trocar um martelo por um bisturi não muda o resultado, só o relógio) — está anotado como tal.
- **`heights`/`covers` viram `Arc<Vec<_>>` (copy-on-write).** O snapshot de undo os clonava **fundo**: 80 MB
  por passo em 4096², com cap de **300 passos** e **os dois extremos por passo**. Agora o snapshot é um bump
  de refcount e os extremos vizinhos **compartilham** ⇒ **metade da memória** da pilha. O tempo por traço não
  cai (uma cópia por traço é *inerente* ao undo por snapshot: o `make_mut` copia quando um passo segura o
  buffer) — mas o 1º traço fica de graça e a memória despenca.

### 11.5 Resultado

| | antes | depois |
|---|---|---|
| pen-up @2048² | 146 ms | **16 ms** |
| **pen-up @4096²** | **1010 ms** | **12 ms** |
| por-movimento | 2,2 ms | 2,2 ms (intacto) |

**Aberto (NOMEADO, e não é meu):** o `ModelSnapshot` clona **fundo os pixels de cada camada não-ativa**
(`images`) por traço — a mesma espécie, pré-existente, fora do escopo desta linha
([[feedback_audit_scope_discipline]]). O `canvas_rgba` já é `Arc`; `images` não.

**Continua fora:** conservação de volume real (a fase que a medição adiou) · luz na GPU (a perf não pede) ·
múltiplas luzes / IBL · relevo do PAPEL (**exige ordem nova do Enio**).

---

## 12. Fase 10 — o undo do Painter copiava as camadas que ninguém tocou (2026-07-12)

**Isto NÃO é um bug do impasto.** É o undo do próprio Painter (`ph2d-tool-painter/src/undo.rs`), e está lá
desde sempre. Foi achado ao fechar a §11, e a correção anterior de `heights`/`covers` só o deixou visível.

Uma pincelada toca **exatamente uma camada** — a ativa, cujos pixels vivem no `canvas_rgba` (já `Arc`). Os
pixels de todas as outras vivem em `images` e **não se mexem**. O `ModelSnapshot` clonava **fundo, todas,
a cada traço**. Medido em 4096², regime permanente:

| documento | pen-up | `images` copiado por snapshot |
|---|---|---|
| 1 camada | 7,6 ms | 0 MB |
| 3 camadas | 31,6 ms | **128 MB** |
| 5 camadas | **56 ms** | **256 MB** |

E um **passo** de undo guarda **dois** snapshots (antes + depois), com cap de **300 passos**. Uma pintura de
5 camadas em 4K gastava **meio giga por pincelada** duplicando camadas em que ninguém pintou. Não sobrevive a
uma pintura real: **acaba a memória muito antes de acabar o undo.**

**Fix:** `images: BTreeMap<RtLayerId, Arc<LayerImage>>` — copy-on-write. O snapshot vira bump de refcount; a
mutação (aplicar máscara, trocar de camada ativa) usa `Arc::make_mut`/`own_image` e só copia se um passo de
undo estiver segurando o buffer. O `LayerImage` em si **não muda** — o `Arc` é um dispositivo de partilha de
RUNTIME, e o formato de disco fica intacto (a conversão mora na fronteira, como o `canvas_rgba` já fazia).
O compositor nunca fica sabendo: o `Arc` desreferencia no `ToolPixelSource`.

**Gate:** `an_undo_snapshot_never_copies_the_pixels_of_a_layer_the_stroke_did_not_touch` — asserta
`Arc::ptr_eq` entre o buffer vivo e o do snapshot: **a MESMA alocação**, não "igual". Um gate de tempo seria
dependente de máquina; um gate de igualdade passaria com uma cópia profunda. Mutação (voltar ao clone fundo)
= **vermelha**.

**Resultado: o pen-up fica PLANO em ~8 ms, seja com 1 ou 5 camadas** (era 7,6 / 31,6 / 56).

**Nota de escopo:** eu havia dito que este problema "não era meu" e que seria reportado a outra linha. **Era
meu** — é a crate do Painter. O que era verdade é que ele **precede o impasto** e é peça de trabalho distinta.
Não há outra linha a quem reportar.

---

## 13. Fase 11 — **conservação de volume** (2026-07-12): a tinta não pode se interpenetrar

A última peça que a pesquisa mapeou e o produto não tinha. Até aqui, um traço sobre tinta grossa
simplesmente **empilhava** sobre ela, como se dois corpos de tinta pudessem ocupar o mesmo espaço. Tinta de
verdade não pode: o pincel **abre um canal** e o material deslocado **se levanta como uma crista nas bordas
do traço**. É a coisa mais reconhecível do impasto, e é o que separa **tinta** de um bump map.

### 13.1 A decisão de projeto que vem ANTES da primeira linha de código

O caminho óbvio é deslocar **por-dab, destrutivamente**, na camada. Ele tem **dois defeitos fatais** aqui:

1. As **shape tools re-estampam a forma inteira a cada pointer-move**. Um operador destrutivo acumularia a
   cada frame: em dois segundos de ajuste de curva, um cânion.
2. Um deslocamento destrutivo **não é re-derivável** — `Push` seria o **único knob morto** num card cujo
   propósito inteiro é que todo knob siga vivo depois do traço (§10.3).

Então o deslocamento é derivado da **PEGADA**, não da trajetória: função pura de `(chão, footprint)`,
aplicada no commit sobre uma **cópia** do chão (o armazenado fica pristino). Logo é **idempotente**, re-deriva
com o resto do card, e o problema do re-stamp **não existe**.

**O preço, nomeado:** o traço desloca como *uma forma*, não como uma lâmina em movimento — **não há bow wave
correndo à frente da ponta**. Isso é deferido, não esquecido.

### 13.2 A aritmética

`push_ground` (em `impasto_settle.rs`): **morde** o chão sob a pegada (∝ cobertura × Push), soma o que
tirou, e **devolve tudo** num **rim** — a pegada borrada para fora, menos ela mesma — normalizado pelo peso
real do rim (tinta empurrada contra a borda do canvas se redistribui no rim que sobra, em vez de sumir).

⇒ **Σh não muda.** Nada é criado, nada é destruído: só se move.

### 13.3 Gates (3 mutações provadas vermelhas)

- **Conservação** — o canvas devolve exatamente a tinta que tinha. É o que faz disto física e não efeito, e
  é o que a versão ingênua quebra em silêncio: um "deslocamento" feito de blend/smear **faz média**, e média
  **perde volume** — arraste o bastante e a escultura derrete. **MUT M** (morde e não devolve) = 14,8% de
  perda. **MUT N** (normaliza por constante) = 1,9%.
- **O percept** — canal onde passou, **crista ao lado**. Conservação sozinha não dá isso: espalhar o
  material uniformemente pelo canvas conservaria perfeitamente e não se veria nada.
- **Vivo e idempotente** — o knob mexe num traço **já dado**, e re-derivar 12× não come o chão duas vezes.
  **MUT O** (mutar a base armazenada) = vermelho.

### 13.4 Dois achados no caminho

- **O pincel seco era um no-op.** O kernel recusava registrar qualquer ingrediente com Depth 0
  (`deposits_height()`), otimização correta enquanto "altura" só significava *depositar*. Com Push, a
  **pegada é ela mesma um ingrediente** — e Depth 0 + Push alto é exatamente a **espátula**, o uso mais
  físico que existe. Agora o gate é `touches_height()` = deposita **ou** empurra.
- **O blur transposto.** O rim usa raio 12, e o passe vertical lia com *stride* — 25 cache-lines por texel:
  **+22 ms** no pen-up em 4096². Transpondo (mesmos taps, **mesma ordem de soma**, zero bits diferentes) cai
  para **+10 ms**. E a **soma exata** foi mantida de propósito: a janela deslizante O(n) foi escrita, e o
  gate de byte-identidade **a rejeitou** — uma soma corrente acumula erro *ao longo da linha*, então o
  resultado passaria a depender da **largura do buffer**, e todo o corte da §11 repousa em o blur de uma
  JANELA ser bit-a-bit o blur do CANVAS. Um blur mais rápido que muda o número não é mais rápido, é errado.

### 13.5 Perf

| @4096² | por-movimento | pen-up |
|---|---|---|
| Push 0 (default) | 2,45 ms | 22,7 ms |
| Push 1 (máximo, arando tinta grossa) | 2,47 ms | 33,2 ms |

Default `0` ⇒ **byte-idêntico** a um build sem Push.

---

### 13.6 Refazendo o Push — o smoke do Enio (2026-07-12): *"não em tempo real"* + *"bordas duras"*

Duas queixas, duas causas **distintas**, e ambas eram consequências diretas do desenho da §13, não descuidos.

**1. Não era em tempo real.** Eu derivei o deslocamento da **pegada inteira**, no commit — o que me deu
idempotência e o knob vivo, mas **uma pegada só existe depois que o traço acaba**. O artista pintava às
cegas e descobria o que tinha feito no pen-up.

**A formulação que resolve as duas sem perder nada.** O campo de deslocamento é **LINEAR em Push**. Então o
que se acumula, **dab a dab e localmente**, é `R₁` — o deslocamento a `Push = 1`: **negativo onde o pincel
tirou, positivo onde bancou, somando exatamente zero**. O relevo comitado é simplesmente `chão + push·R₁`.

⇒ **tempo real** (por-dab, `O(dab)`) · **conservativo** (Σ R₁ = 0 *por construção*) · **e ainda vivo** (uma
multiplicação) · **e ainda idempotente** (o chão nunca é mutado) · **e imune ao re-stamp** das shape tools.

**Três bugs no caminho, cada um com o seu vermelho:**

- **Dois livros que não fechavam.** O commit recalculava a *mordida* da cobertura final enquanto o *banco*
  era acumulado por-dab — e um dab que não achava rim bancava nada, mas o commit cobrava assim mesmo:
  **5% da tinta evaporava**. Um livro só, e ele não pode discordar de si mesmo (MUT R = 14,8%).
- **O banco tem de ser LATERAL.** Bancando radialmente, cada dab deposita **à frente de si** — e a própria
  pincelada atropela aquilo no dab seguinte. Metade da tinta ficava **dentro do próprio canal** (o canal
  saía a 55% da laje). Uma lâmina em movimento não deixa tinta em pé na sua frente. (MUT S = vermelha.)
- **A tinta bancada nunca vai onde o traço já passou** — bancar sob a própria pincelada é bancar no próprio
  canal. Com um fallback: se **não houver onde bancar**, banca no swath mesmo. Tinta sem lugar é tinta
  **destruída**, e a coisa toda existe para afirmar que tinta nunca é destruída.

**2. Bordas duras.** O rim era `blur(pegada) − pegada` com um **box blur**, e um box blur de um degrau é uma
**rampa linear**: contínua, mas de **derivada descontínua** — e a luz lê a derivada. Ele desenhava um vinco
a exatamente `reach` px do traço. O rim agora é um perfil **analítico C¹**, sem blur nenhum.

> **Mas o gate é honesto sobre o que ele prova, e isso importa:** trocar o perfil C¹ por um triangular
> (kinkado) **NÃO** derruba o gate — não pode, porque o banco é acumulado **dab a dab** e dabs vizinhos
> sobrepõem-se, apagando qualquer kink do kernel. O que a luz de fato lia como *borda dura* era o **banco
> ESTREITO**: `reach = 0.35 × raio` = 4 texels segurando a mordida de um pincel de 12 px não é um banco, é
> uma **espícula**. Agora é `0.8 × raio` (cap 24 px). **MUT T** (voltar a 0.35) = **vermelha, 40% de kink**.
> O perfil C¹ é a forma certa a buscar; **a largura é o que os testes defendem**, e está escrito assim no
> código.

**Perf — e uma otimização que era obrigatória, não enfeite.** O `R₁` é gravado **sempre que há chão para
empurrar** (não só com o knob levantado), senão Push seria um knob morto num card cujo propósito é o
oposto. Isso custa. O primeiro corte avaliava `silhouette_at` **duas vezes por texel** (uma para depositar,
outra para tirar) e pôs o impasto em **5,0 ms/move — acima do orçamento, em todo traço**. A mordida passou a
**andar dentro do passe do depósito** (3 operações num laço que já rodava) e o banco ficou só com o **anel**,
com os pesos **cacheados** entre os dois passes e sem `sqrt`:

| @4096² | por-movimento | pen-up |
|---|---|---|
| Push 0 | **3,63 ms** | 28,1 ms |
| Push 1 | **3,74 ms** | 27,3 ms |

Alvo ≤4, kill 8. Em canvas limpo o custo é **zero** (`displaced == 0` curto-circuita antes dos laços): o
preço cai exatamente onde a feature vive — tinta sobre tinta.
