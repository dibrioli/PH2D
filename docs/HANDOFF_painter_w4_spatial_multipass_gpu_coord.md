═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · W4 — infra GPU MULTI-PASS p/ os 8 ajustes ESPACIAIS
(Gaussian/Bloom/Sharpen/Motion/ChromaticAberration/ShadowsHighlights/…)
Autor: Implementador Painter (sessão 2026-06-04) · pós Curves/Levels + BATCH-1
═══════════════════════════════════════════════════════════════════

╔═══════════════════════════════════════════════════════════════════╗
║ PADRÃO-OURO REAL-TIME é o ponto. Esses filtros são o que um editor   ║
║ GPU (Photoshop/Lightroom) e post-processing de jogo fazem em <ms.    ║
║ O alvo é o MESMO da engine de adjustments: 1024² sub-2ms, slider-    ║
║ drag fluido. CPU-first aqui é DÍVIDA (não é o caminho) — a infra GPU ║
║ multi-pass tem que vir PRIMEIRO, e é ARQUITETURA SUA (`ph2d-render`).║
╚═══════════════════════════════════════════════════════════════════╝

───────────────────────────────────────────────────────────────────
§1 — O PROBLEMA (verificado no código, não vapor)
───────────────────────────────────────────────────────────────────
O compositor GPU (`layer_composite.wgsl` `cs_flat`/`cs_grouped`) é **um único
compute pass** com o acumulador **num REGISTRADOR per-pixel** (`acc0` + stack de
grupo pequeno). Cada `textureLoad(layers, coord, slot)` lê UM texel — o do próprio
pixel. **Zero acesso a vizinhos.** Um ajuste só transforma o acc do próprio pixel
(`apply_adjustment(ap, rgb)`).

→ Filtros ESPACIAIS (que dependem de uma vizinhança / raio) são **arquiteturalmente
impossíveis** aqui. Não é "adicionar um case no switch" como foi Curves/Levels —
precisa de um MECANISMO NOVO: ler a composição-de-baixo como TEXTURA amostrável,
rodar o efeito em 1+ passes (render-to-texture, ping-pong), e devolver pro stack.

**Por que NÃO CPU-first:** vizinhança per-pixel é O(r²)/pixel (ou O(2r) separável)
— em 1024² com raio 30 são dezenas-centenas de ms na CPU. O cut-cache (que salvou
Curves/Levels) **não ajuda**: ele evita recompor as camadas de baixo, mas o custo
do filtro É a vizinhança, que continua lá todo frame de drag. Logo CPU-first quebra
o mandato real-time e vira dívida a refazer. A infra GPU é o caminho certo, não o
atalho.

───────────────────────────────────────────────────────────────────
§2 — QUAIS kinds precisam (e quais NÃO — pra não travar o Impl à toa)
───────────────────────────────────────────────────────────────────
**PRECISAM da infra multi-pass / gather (6):**
  - **GaussianBlur** `{radius}` — separável 2-pass (H depois V), O(2r)/pixel.
  - **Sharpen** `{amount, radius, mask_edges}` — unsharp mask = `src + amount·(src −
    blur(src))`; **reusa o Gaussian** + 1 combine pass.
  - **Bloom** `{threshold, intensity, radius, falloff}` — bright-pass → blur-chain →
    add. Mip-chain / dual-Kawase pra raio grande barato.
  - **MotionBlur** `{distance, angle}` — blur direcional (kernel 1-D ao longo do
    ângulo); 1 pass com N taps, ou 2-pass se decompor.
  - **ShadowsHighlights** `{… , shadows_radius, highlights_radius, …}` — contraste
    LOCAL (usa um blur do próprio canal como "mapa de tom") → blur + combine.
  - **ChromaticAberration** `{r/g/b_shift, falloff_center}` — GATHER (amostra a
    textura-de-baixo em coords deslocadas por canal, radial). Single extra pass,
    mas precisa da composição-de-baixo como TEXTURA (não register) — mesma infra.

**NÃO precisam — são PER-PIXEL (Impl faz JÁ pelo caminho escalar/LUT, não bloqueia):**
  - **Noise** `{amount, kind, monochromatic}` — ruído por pixel (hash coord-seeded
    no shader; PRNG determinístico HR-5). Single-pass — vai no switch escalar igual
    Vibrance. ⚠️ avise o Impl: NÃO é multi-pass; pode fazer agora.
  - **Halftone** `{dot_size, angle, shape}` — padrão de retícula por pixel (função
    de tela amostrada na coord). Single-pass — idem.

→ **Desbloqueie Noise+Halftone pro Impl já** (caminho escalar). Os 6 acima são SUA
infra.

───────────────────────────────────────────────────────────────────
§3 — A INFRA (compositing SEGMENTADO / pass-graph) — o coração
───────────────────────────────────────────────────────────────────
Hoje o compositor é monolítico (um pass varre o op-list inteiro). Um ajuste
espacial vira um **"pass break"**: o stream de ops se PARTE nele.

Para cada ajuste espacial no op-list:
  1. **Materializa a composição-de-baixo** (tudo abaixo do ajuste) numa TEXTURA
     `rgba8`/`rgba16f` — isto JÁ é o que o cut-cache materializa pro adjustment-
     cache; reuse o conceito. É o INPUT do efeito.
  2. **Roda o efeito** como 1+ compute/render passes via um **pool de texturas
     ping-pong** (A↔B reusadas; HR-3 sem realloc/frame):
       - separável: pass-H (input→A) + pass-V (A→B).
       - bloom: bright-pass + downsample-chain + upsample/combine.
       - gather/direcional: 1 pass.
  3. **Devolve** o resultado pro acumulador (blend pela opacity/mode do ajuste,
     como o `apply_adjustment_op` já faz) e o compositor monolítico **continua** as
     camadas ACIMA num novo segmento.

Estrutura nova no `LayerCompositor`: um **plano de passes** derivado do op-list
(segmentos single-pass intercalados com efeitos multi-pass) + o pool de texturas +
os pipelines dos kernels. O `LayerOp::Adjustment` ganha (ou um irmão
`LayerOp::SpatialAdjustment`) um discriminante "é efeito espacial, kernel=K,
params=…". Contrato: o tool emite isso; você roda o pass-graph.

───────────────────────────────────────────────────────────────────
§4 — TÉCNICAS PADRÃO-OURO real-time (o que faz ser <ms)
───────────────────────────────────────────────────────────────────
  - **Separável** (Gaussian/Box): 2-pass O(2r) em vez de O(r²). Pesos Gaussianos
    pré-computados (CPU) num uniform/storage — a matemática é do Impl (referência +
    pesos), o pass é seu.
  - **Mip-chain / dual-filter (Kawase)** pro Bloom: downsample progressivo +
    upsample — raio "infinito" a custo ~O(1). Padrão de bloom de engine moderna.
  - **Half-res / quarter-res** nos buffers de efeito quando o raio é grande (blur
    de baixa freq não precisa full-res) → 4× menos texels. Up-sample no combine.
  - **Pool ping-pong reusado** entre efeitos + entre frames (HR-3, igual o
    `scratch_ops`/`adj_params_buffer` já fazem).
  - **Dirty-rect + HALO:** o região suja do efeito = dirty-rect ⊕ raio do kernel
    (um stroke perto da borda do efeito "vaza" raio px). Expanda a região de
    recompose pelo raio — senão fica seam. (O dirty-rect já existe no compositor;
    só precisa do dilate por raio.)
  - **Precisão:** acumular em `rgba16f` nos passes de blur (banding em 8-bit com
    raio grande). Encode final pro `rgba8` straight como hoje.

Alvo: mesmo da engine escalar — 1024² sub-2ms, slider-drag (raio/intensidade)
fluido. Bloom mip-chain e blur separável chegam lá folgado em Metal/M-series.

───────────────────────────────────────────────────────────────────
§5 — DIVISÃO DE TRABALHO + faseamento (espelho de Curves/Levels)
───────────────────────────────────────────────────────────────────
**Impl entrega** (na pasta dele, sob demanda sua): a referência CPU canônica de
cada kernel (`apply_gaussian`/`apply_bloom`/… em `adjustments/compute.rs`) + a
MATEMÁTICA exportada (pesos do kernel, bright-pass threshold curve, etc.) — análogo
a `curves_display_luts`. É o que seu pass-graph consome + o que a paridade GPU↔CPU
checa. **Coord entrega** (ph2d-render): o pass-graph segmentado, o pool ping-pong,
os pipelines/WGSL dos passes, o dirty-rect-halo, e o discriminante no `LayerOp`.

**Faseamento (faça o GAUSSIAN primeiro como o spike da infra):**
  1. **GaussianBlur** = o caso canônico separável. Quando o pass-break +
     materialização + ping-pong + 2-pass H/V + dirty-rect-halo funcionarem pra ele
     (com paridade ±tol no teu Mac), **a infra está provada**.
  2. Daí caem de carona na MESMA infra: **Sharpen** (unsharp = Gaussian + combine),
     **ShadowsHighlights** (local-contrast = blur + combine), **MotionBlur**
     (kernel direcional no mesmo mecanismo), **Bloom** (bright-pass + a blur-chain).
  3. **ChromaticAberration** (gather de 1 pass) — só precisa da textura-de-baixo
     amostrável, que a infra já dá.

Não precisa fazer os 6 de uma vez — o spike do Gaussian destrava todos. Alinhe com
o Impl: ele faz a referência CPU + UI (sliders de raio/intensidade — slider-rack já
existe) enquanto você faz a infra; vocês casam no kernel.

───────────────────────────────────────────────────────────────────
§6 — PARIDADE / GATES / DETERMINISMO
───────────────────────────────────────────────────────────────────
  - GPU↔CPU ±tolerância (blur é `pow`/soma ULP-bounded; ±1-4 B como os outros).
    Espelhe `gpu_adjustment_matches_cpu_reference_each_kind` por kernel em
    `tests/layer_compositor_gpu.rs` — RODE no Mac/Metal (`--ignored`).
  - Pesos do kernel: literais pinados bit-idênticos Rust↔WGSL (mesma disciplina do
    `srgb_lut`/`shader_*_bit_identical`). A matemática (Gaussian σ↔radius, etc.)
    é a do Impl.
  - HR-3: pool ping-pong sem realloc/frame (gate de no-alloc como o `scratch_ops`).
  - Naga-valida os pipelines novos no `--lib layer_compositor`.

───────────────────────────────────────────────────────────────────
§7 — REFERÊNCIAS
───────────────────────────────────────────────────────────────────
  - Compositor atual (single-pass): `crates/ph2d-render/src/shaders/layer_composite.wgsl`
    (`cs_flat`/`cs_grouped`, `apply_adjustment_op`) + `layer_compositor/{mod,compositor}.rs`.
  - Precedente da divisão Impl↔Coord (matemática exportada + binding GPU):
    Curves/Levels — `HANDOFF_painter_w4_bespoke_kinds_coord.md` §2 (`adj_luts`).
  - Kinds + params + estado: `HANDOFF_painter_w4_remaining_kinds_impl.md` §3 +
    `adjustments/mod.rs` (`GaussianBlurParams`/`BloomParams`/… já existem, congelados).
  - Mandato real-time + a engine que provou o alvo: memória
    [[project-painter-composite-perf-2026-06-03]] (GPU 1.7ms vs 55ms CPU).
═══════════════════════════════════════════════════════════════════

## RESPOSTA DO COORDENADOR (2026-06-04)

**ACEITO — a infra multi-pass é minha (ph2d-render, foundational).** Análise correta: vizinhança
espacial é arquiteturalmente impossível no single-pass per-pixel atual; CPU-first quebra o mandato
real-time. Pass-graph segmentado + ping-pong é o caminho.

**🟢 DESBLOQUEIO IMEDIATO — Noise + Halftone (faça AGORA):** são per-pixel (hash/screen-function na
coord), single-pass — vão no switch escalar via teu `gpu_code()/gpu_params()` igual Vibrance. **NÃO
dependem da minha infra. Não espere por mim** — entrega esses 2 no caminho que já domina.

**DIVISÃO (espelho Curves/Levels) + FASEAMENTO (Gaussian spike primeiro):**
- **TU entregas** (sob demanda, na tua pasta): a referência CPU canônica `apply_gaussian` em
  `adjustments/compute.rs` + a MATEMÁTICA exportada (pesos Gaussianos σ↔radius normalizados, análogo
  a `curves_display_luts`) — é o que meu pass consome + a paridade GPU↔CPU checa.
- **EU entrego** (ph2d-render): pass-graph segmentado derivado do op-list, pool ping-pong (HR-3
  sem realloc), materialize-below→textura, passes H/V WGSL, dirty-rect⊕raio (halo), o discriminante
  `LayerOp::SpatialAdjustment{kernel,params}`, e o gate de paridade `--ignored` no Metal.
- **Casamos no kernel Gaussian.** Provado ele (pass-break+ping-pong+H/V+halo+paridade), os outros 5
  caem de carona (Sharpen=Gaussian+combine; ShadowsHighlights=local-contrast; Motion=kernel direcional;
  Bloom=bright+blur-chain; ChromaticAberration=gather 1-pass).

**⚠️ SEQUÊNCIA (recomendação ao Enio):** há **~67 commits locais não-pushados** (Vector W1-W5 + SDF
+ Painter Curves/Levels/tokens). Vou **recomendar shipar esse marco ANTES** de eu landar a infra
espacial (mudança grande de arquitetura do compositor merece base CI-validada limpa). Enquanto o
ship roda, tu fazes Noise+Halftone + a referência CPU do Gaussian; eu arranco a infra logo depois.
Se o Enio mandar arrancar a infra já, arranco — a fiação do mecanismo (pass-graph/ping-pong) é
independente do teu kernel até o ponto de casamento.
═══════════════════════════════════════════════════════════════════
