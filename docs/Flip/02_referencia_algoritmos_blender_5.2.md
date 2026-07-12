# Flip — Referência de algoritmos (Grease Pencil, Blender 5.2)

> **Consulte este doc antes de cada tópico** e vá ao fonte em `~/Downloads/blender-5.2-grease-pencil-ref/`
> (índice em [`00_README.md`](00_README.md)). **Clean-room:** leia o comportamento, reimplemente do
> zero; nunca copie código GPL. Citações `arquivo:linha` são do recorte 5.2. Cada seção termina com a
> **decisão PH2D** (como adaptar ao engine 2D-ortográfico).

Sumário: §1 Modelo de dados · §2 Render GPU (ultra-perf) · §3 Tween (interpolação) · §4 Ops de curva
(smooth/simplify/fit/resample/fillet) · §5 Draw/Erase/Fill · §6 Reshape (sculpt).

---

## §1 — Modelo de dados

**Hierarquia (GPv3):** `GreasePencil (datablock) → root LayerGroup → Layer(folha) | LayerGroup →
frames: Map<int,Frame> → Drawing → CurvesGeometry (strokes)`. O datablock guarda **um array plano de
drawings** (`drawing_array`, ordem arbitrária) e cada `Frame` referencia um drawing **por índice**;
vários frames podem apontar pro mesmo drawing (instância). Ref: `DNA_grease_pencil_types.h:205-503`,
`BKE_grease_pencil.hh`.

### Frame = keyframe que guarda um desenho; a duração é implícita

`GreasePencilFrame { int drawing_index; flag; int8 type }` (`DNA:255-275`). A **chave do mapa é o
frame-de-início**; o fim é a próxima chave. Dois mecanismos de duração:

- **Implicit hold** (`GP_FRAME_IMPLICIT_HOLD`): o desenho segura até a próxima chave. É o default
  (`insert_frame` com `duration==0`, `grease_pencil.cc:1551`).
- **End-frame sentinela** (`Frame::end()`, `drawing_index == -1`): fecha uma duração fixa. Ex. mapa
  `{0:d0, 5:d1, 10:end, 12:d2}` → d1 aparece de 5 a 9, nada de 10 a 11 (`hh:448-474`).

**Resolver o desenho ativo num frame N** (`Layer::drawing_index_at`, `grease_pencil.cc:1617-1712`):
`upper_bound(sorted_keys, N)` e recua um → maior chave ≤ N; se for end-frame, retorna −1 (nada). É a
semântica de **hold** — é isto que o playhead amostra.
```
it = upper_bound(sorted_keys, frame_number); if (it==begin) → nada; return *prev(it)  // se não é end
```

### Drawings são refcontados pelos frames

`DrawingRuntime::user_count` (atômico, começa 1). `add_user`/`remove_user` nos frames; `is_instanced()
= users>1`; `remove_drawings_with_no_users()` compacta o array e **remapeia todos os `drawing_index`**
(`grease_pencil.cc:3414-3526`, `4538-4549`). Duplicar frame: `do_instance=true` compartilha o mesmo
drawing (+1 user); `false` faz cópia profunda. **Nota:** o operador de duplicar do editor **sempre**
copia (`do_instance=false`, `grease_pencil_frames.cc:810-851`); a instância é usada internamente.

### Atributos — por-ponto e por-curva (com defaults)

O stroke é uma polilinha (curva `POLY`, ou Bezier/Catmull). Atributos em `CurvesGeometry`:

| Domínio | Atributo | Tipo | Default | Nota |
|---|---|---|---|---|
| **Ponto** | `position` | float3 | — | mundo (no PH2D: 2D) |
| Ponto | `radius` | float | `0.01` | espessura em unidade de mundo (`hh:196`) |
| Ponto | `opacity` | float | `1.0` | (=strength) |
| Ponto | `vertex_color` | float4 | `(0,0,0,0)` | cor por-ponto (preto transparente = "sem override") |
| Ponto | `rotation` | float | `0.0` | rotação do dab (dots) |
| Ponto | `.selection` | bool | `true` | |
| **Curva** | `cyclic` | bool | `true`(novos) | traço fechado |
| Curva | `material_index` | int | `0` | |
| Curva | `start_cap`/`end_cap` | int8 | `ROUND(0)` | ROUND/FLAT |
| Curva | `softness` | float | `0.0` | hardness efetiva = `1-softness` |
| Curva | `aspect_ratio` | float | `1.0` | |
| Curva | `fill_color` | float4 | `(0,0,0,0)` | preenchimento |
| Curva | `fill_opacity` | float | `1.0` | |
| Curva | `fill_id` | int | `0` | 0 = não preenchida; agrupa curvas num fill composto |

Refs: `grease_pencil.cc:946-1006` (accessors), `draw_cache_impl_grease_pencil.cc:1383-1414` (defaults
do consumidor). `LEGACY_RADIUS_CONVERSION_FACTOR = 1/2000` (`hh:55`).

### Camada: opacity, blend, mask, transform

- `opacity` default `1.0`; `blend_mode` ∈ {None, HardLight, Add, Subtract, Multiply, Divide}
  (`DNA:106-113`) — **conjunto pequeno**; o PH2D já tem 22 modos no compositor do Painter (reusar).
- **Masks** por *nome de camada* (`ListBase<LayerMask>` + flags Hide/Invert). Uma camada é mascarada
  por outra(s) camada(s) referenciada(s) por nome.
- **Transform de camada** = loc/rot/scale (`local_transform()`, `grease_pencil.cc:1845`). No PH2D isso
  vira o `Transform` da entidade (§integração Hierarchy).
- Fill cache: triangulação CDT lazy por `fill_id` (`triangles()` → `GroupedSpan<int3>`,
  `grease_pencil.cc:472-639`): 1 stroke → `BLI_polyfill`, N strokes → `delaunay_2d_calc(CDT_INSIDE_WITH_HOLES)`.

### Onion skinning (defaults canônicos)

`GreasePencilOnionSkinningSettings` (`DNA:409-436`): `opacity=0.5`, `mode=RELATIVE`, `USE_FADE|USE_CUSTOM_COLORS`,
`num_frames_before=1`, `num_frames_after=1`, `color_before=(0.145,0.420,0.137)` (verde),
`color_after=(0.125,0.082,0.529)` (azul). Modos: ABSOLUTE (Δ em nº de frame), RELATIVE (Δ em keyframe),
SELECTED. Filtro por tipo de keyframe.

### ► Decisão PH2D (design do `ph2d-flip`)

Modelo limpo, clean-room, SoA-friendly (pensado pro upload GPU):

```rust
struct FlipDoc { layers: Vec<FlipLayer>, drawings: Vec<FlipDrawing>, onion: OnionSettings, next_id }
struct FlipLayer { id, name, frames: BTreeMap<Frame /*i32*/, FlipFrame>,  // BTreeMap = sorted_keys de graça
                  opacity: f32, blend: BlendMode, visible, locked, mask: Option<LayerMaskRef> }
struct FlipFrame { drawing: DrawingId /*-1 = end*/, hold: Hold, kind: KeyKind }  // Hold::Implicit | Fixed(n)
struct FlipDrawing { strokes: Vec<FlipStroke>, users: u32, fill_tris: Lazy<..> }
struct FlipStroke {                 // SoA por atributo, não AoS
  pos: Vec<Vec2>, width: Vec<f32>, opacity: Vec<f32>, color: Vec<Rgba>,   // POR-PONTO
  closed: bool, cap: (Cap,Cap), hardness: f32, material: MaterialId, fill: Option<Fill>,  // POR-CURVA
}
```

- **`BTreeMap<i32, FlipFrame>`** dá `sorted_keys`/`upper_bound`/hold de graça; a amostragem pelo playhead
  é `range(..=frame).next_back()`.
- Manter **refcount** de drawing (instanciar frames repetidos = economia de memória e edição
  propagada — o "duplicate as instance" é ótimo pra ciclos).
- Cor por-ponto é `Rgba` premultiplicado linear; o "sem override" do GP (preto transparente) vira
  `Option`-por-stroke ou α=0. Preferir **override explícito** (mais intuitivo que o truque do GP).
- Guardar geometria **LOCAL** (a pose vai no `Transform` da entidade, ADR-0111).

---

## §2 — Render GPU (ultra-performance, tempo real)

O GP renderiza **todo o traço na GPU**: os pontos vivem em buffers e o vertex shader expande cada
segmento num quad, com junções em **screen-space**. É o padrão-ouro pro runtime do PH2D. Refs:
`draw_cache_impl_grease_pencil.cc`, `draw_grease_pencil_lib.glsl` (`lib`),
`draw/engines/gpencil/gpencil_frag.glsl` (`frag`), `gpencil_engine_c.cc` (`engine`), `gpencil_cache_utils.cc`.

### Layout de dados (o que subir pra GPU)

Dois buffers-textura lidos por `texelFetch` (NÃO como vertex attributes) — `draw_cache:107-144`, confirmado em `lib:458-471`:

- **`gp_pos_tx`** = **3 texels `vec4<f32>` por ponto**:
  - texel 0: `pos.xyz` + `.w = radius/thickness`
  - texel 1 (lido como `int4` via bitcast): `material_index`, `stroke_id`, `point_id` (**sinal = cyclic**), `packed`
  - texel 2: `uv_fill.xy`, `u_stroke` (z, coord ao longo do traço), `opacity` (w)
- **`gp_col_tx`** = **2 texels por ponto**: texel 0 = `vertex_color` RGBA; texel 1 = `fill_color` (α codifica fill opacity).

`packed` (int32, `draw_cache:229-272`, decode `lib:213-242`): bits[0-8]=aspect, [9-17]=cos(rot)+sinal, [18-25]=hardness(`1-softness`), [26-31]=miter/corner type. No PH2D podemos usar campos diretos (temos storage buffers WGSL, sem o aperto de attribute-count do OpenGL do Blender).

### Expansão do traço (sem index buffer de linha — quads por ID)

Cada ponto → **4 IDs de canto → 2 triângulos**. O índice carrega `(point_id << 2) | (1<<30)`
(`GP_IS_STROKE_VERTEX_BIT`); os 2 bits baixos viram o canto do quad (`x=(id&1)*2-1`, `y=(id&2)-1`,
`lib:525-526`). **Padding de adjacência:** 1 vértice antes + 1 depois por curva (+1 se cyclic), com
`mat=-1` e `stroke_id` cruzado (first↔last) — emula `GL_LINES_ADJACENCY`, o shader busca `id-1,id,id+1,id+2`
(`lib:458-465`; contagem `draw_cache:1326-1340`, alocar `N+2`).

**Espessura em screen-space** (`lib:249-258`) — no Blender depende de perspectiva (`winmat[1][1]*viewport.y`
+ billboard camera-facing). **No PH2D 2D-ortográfico isto COLAPSA:** sem divisão perspectiva (`w=1`),
`thickness_px = radius_mundo * pixels_por_mundo` (o fator de zoom da `Camera2d`). O plano do traço **é**
o plano da tela — nada de bilhete/normal/tangente 3D.

**Junções** (`lib:669-727`): miter tangent `normalize(line_adj + line)`; quebra pra bevel quando o
ângulo passa do `miter_limit` (default `cos 60°`); round via distância clampada no fragment. Caps
round/flat codificados no sinal de strength/thickness.

### Seção transversal do traço (fragment) — a fórmula de hardness

`frag` chama `gpencil_stroke_segment_mask` → `gpencil_stroke_hardess_mask` (`lib:29-41`). **Esta é a
queda de borda** (dá o traço antialiased com dureza variável):
```
dist = clamp(1 - dist_ao_eixo/(thickness*0.5), 0, 1)
hard≈1 → step(1e-8, dist)                                  // borda dura
senão  → smoothstep(0, 1, pow(dist, mix(0, 10, 1-hard)))   // airbrush macio
```
AA global final: `frag_color *= smoothstep(0,1, thickness_unclamped/w)` (afina traços sub-pixel,
`frag:534`). Fill não usa máscara (é a cor chapada da triangulação).

### Passes e ordenação

- **Fill primeiro, depois traço** no MESMO IBO: triângulos do fill (índices sem o bit30) seguidos dos
  quads do traço (`draw_cache:1654-1719`). Triangulação na CPU (CDT/earcut), agrupada por `fill_id`.
- Por objeto → por camada, 1 geom pass. Camada com **blend≠None / mask / opacity<1** → renderiza num
  FB próprio e compõe num **blend pass** (HardLight = 2 passes). Alvos duplos: `color` (pré-mult) +
  `reveal` (revealage = 1−α). Blend em `gpencil_common_lib.glsl:18-71`.
- **Depth 2D (o truque de ordenação):** profundidade constante por traço `= (stroke_id + 2)·2e-7`, teste
  **GREATER** (traço mais novo ganha); fill usa `+1` (fica meio-texel atrás do próprio traço). 3D usa
  `gl_FragDepth` real. `gl_FragDepth` é escrito no fragment (`frag:573-582`).
- **Onion:** `onion_id` = Δ assinado; range `[-before, +after]`; tint `color_before/after`;
  `alpha = clamp(onion_opacity * (fade? 1/|Δ| : 1), 0.1, 1)` (`cache_utils:217-264`). No PH2D é um
  passe barato de re-desenho dos drawings vizinhos com tint uniforme — praticamente de graça na GPU.

### ► Decisão PH2D (pipeline wgpu dedicado)

1. **Buffers SoA por drawing** (WGSL storage buffers): `positions`, `widths`, `opacities`, `colors`,
   + tabela de strokes (offsets, closed, cap, hardness, material). Upload 1× por drawing editado; o
   **playback só troca qual range está bound** — zero re-tessellação por frame (é o segredo do runtime).
2. **Vertex shader** expande ponto→quad com miter/bevel/round em screen-space; **sem** matemática 3D
   (ortho colapsa tudo). Instanced/indexed; um draw por (camada, material).
3. **Fragment shader** = a máscara de hardness acima (portada 1:1) + AA sub-pixel.
4. **Fills:** triangular na CPU (reusar o CDT que o Vector já tem em `ph2d-vec-boolean`/kurbo) num IBO
   irmão, **fill-first**.
5. **Compositing de camada:** reusar o **compositor GPU 22-modos do Painter** (`ph2d-render/layer_compositor`)
   em vez de reimplementar os 6 do GP — ganho grátis de riqueza.
6. **Editor vs runtime = mesmo pipeline.** O editor pode continuar pintando UI/gizmo em Vello por cima;
   o traço do Flip vai por este passe wgpu dedicado (não pela `vello::Scene`), pela performance.
7. **Bench desde o início:** N traços × M pontos animando a 60/120 Hz; medir em `--release` (memória
   `feedback_measure_perf_symptom_scale`).

---

## §3 — Tween (interpolação / inbetween)

O núcleo `geometry/interpolate_curves.cc` recebe tudo pré-computado; o **pareamento e o flip** vivem no
operador `editors/sculpt_paint/grease_pencil/interpolate.cc`. Precisa dos dois.

- **Pareamento por ÍNDICE** (não proximidade): curva *i* do "from" ↔ curva *i* do "to"
  (`interpolate.cc:244-312`). Sobra de um lado = descartada; curva sem par é cópia estática
  (peso 0 ou 1), **sem fade nesta camada** (`interpolate_curves.cc:1059-1082`).
- **Reconciliação de pontos** = contagem **MÁX** dos dois (`interpolate.cc:545`): a curva mais longa
  mantém pontos 1:1; a mais curta é *padded* via `sample_curve_padded` (distribui amostras extras por
  arco, segmentos maiores recebem mais — `interpolate_curves.cc:87-155`). Poly interpola direto; Bezier
  passa por `interpolate_to_evaluated` antes.
- **Auto-flip** (`interpolate.cc:427`): se os segmentos que ligam as pontas (from_first→to_first,
  from_last→to_last) **se cruzam** → flip; ângulo <15° desempata por distância; senão flip se
  `dot(from_dir, to_dir) < 0`. Aplica reverse só na curva "to".
- **O que interpola:** tudo que é Float/Float2/Float3/Float4 por-ponto (position, width/radius, opacity,
  vertex_color, rotation) via `lerp(from[i], to[i], t)` (`mix_arrays`, `interpolate_curves.cc:860-877`).
  Atributos não-float (material_index, cyclic) = copiados do "from" (nearest).
- **Contrato:** o chamador dá `from/to_curve_indices` (-1 = sem par), `dst_curve_flip[]`, `mix_factor`
  global. Testes em `GEO_interpolate_curves_test.cc` mostram a colocação de amostras.

### ► Decisão PH2D

- Tween só faz sentido quando os traços **têm correspondência** — replicar o **pareamento por índice**
  (traço *i*↔*i*) e o **padding ao MÁX**. Reamostrar por arco, lerp de pos/width/opacity/color.
- Expor um **UI simples e intuitivo:** selecionar quadro A e B, "Add Tween" → cria N inbetweens; slider
  de quantidade + curva de easing (reusar `ph2d-anim::Interp`/Easing que já existe!). Auto-flip ligado
  por default (evita a "torção" que assusta o iniciante).
- Requisito honesto: tween morre se as contagens/ordem de traço divergirem muito — documentar e, no
  futuro, oferecer "match strokes" manual.

---

## §4 — Operações de curva (smooth · simplify · fit · resample · fillet)

Usadas no pós-processo do traço (pen-up), no Reshape, e no Simplify. Refs: `geometry/intern/*_curves.cc`.

- **smooth** (`smooth_curves.cc`): **não** é média simples — é **blur binomial/gaussiano 1D** (kernel
  `nCr(n, j+n/2)/2^n`, ~gaussiana). Params: `iterations` (default 10, op de edição), `influence`/factor
  (0..1), `smooth_ends` (default false → endpoints fixos), `keep_shape` (kernel com pesos negativos p/
  não encolher a forma), `is_cyclic`. Opera em float/float2/float3 (position, radius, opacity). O
  **active smoothing do desenho** usa `pre_blur_iterations=3`, `keep_shape=true` (`paint.cc:566`).
- **simplify** (`simplify_curves.cc`): **RDP** (Ramer-Douglas-Peucker) com distância **perpendicular
  generalizada** (lambda pela posição, distância medida no atributo). `epsilon` default `0.01`; preserva
  endpoints; iterativo por stack. Modos alternativos: Fixed (steps), Merge-by-distance, Sample
  (resample len 0.05).
- **fit** (`fit_curves.cc`): **Schneider** (bezier least-squares + Newton-Raphson + corner detect) via
  lib externa `curve_fit_nd` (**não vem no recorte**). Métodos `Split` e `Refit`. **PH2D já tem** refit
  Schneider (`curve_refit.rs` no Painter) — reusar. Corners forçam handle FREE; fallback POLY.
- **resample** (`resample_curves.cc`): modos `to_count`, `to_length` (even-length, `count = len/sample_len + 1`),
  `to_evaluated`. Amostra por **comprimento de arco** (`length_parameterize::sample_uniform`). Sempre
  produz poly.
- **fillet** (`fillet_curves.cc`): arredonda cantos por raio (`displacement = radius·tan(θ/2)`, arco de
  N cortes ou 2 handles bezier `handle_len = 4/3·radius·tan(θ/4)`); `limit_radius` evita sobreposição.

### ► Decisão PH2D

- Portar smooth (binomial, endpoints fixos), simplify (RDP), resample (por arco) — são pequenos e
  determinísticos (cuidado HR-5: `pow`/`exp` do kernel são transcendentais — ver se cabe no gate; o
  binomial exato é polinomial e preferível).
- Fit: reusar o Schneider do Painter. Não reimplementar.
- Estes viram utilitários compartilháveis (talvez `ph2d-flip-geom` ou dentro de `ph2d-flip`), usados no
  pen-up (smooth+fit), no Reshape (smooth local) e no Simplify manual.

---

## §5 — Draw · Erase · Fill

Refs: `editors/sculpt_paint/grease_pencil/{paint,paint_common,erase,fill,draw_ops,trace_util}.cc`,
`editors/grease_pencil/intern/grease_pencil_{utils,randomize}.cc`.

### Desenho (`paint.cc`) — o segredo do "traço que assenta"

Fluxo: `on_stroke_begin` → N× `process_extension_sample` → `on_stroke_done`. Cada amostra = `(pos_tela, pressão)`.

- **Espaçamento** (`paint.cc:756-802`): se a nova amostra está a **< 2 px** da última, **sobrescreve** o
  último ponto (mantém o máx de raio/opacidade) — não cria ponto. Se está longe, subdivide o segmento
  linearmente com passo `max(brush.spacing%/100 · raio_px, 0.25px)` (máx 4 pontos/px).
- **Active smoothing** (`paint.cc:544-621`) — inicia com ≥8 pontos. Por frame: detecta cantos → pré-blur
  gaussiano **3 iter** (sigma = `active_smooth`) → **curve fit** (erro `5px·active_smooth`) → reamostra a
  **32** → "morfa" os pontos originais sobre a curva. **Convergência:** acumula a média dos fits; um ponto
  "congela" quando muda < **0.1 px** e sai da janela. **É isto que faz a cauda assentar enquanto a ponta
  ainda vibra** — a sensação premium de desenho. (Reimplementar isto é o que separa "bom" de "medíocre".)
- **Pressão → raio/opacidade** por **curvas editáveis** não-lineares (`curve_sensitivity`, `curve_strength`;
  `grease_pencil_utils.cc:1645-1669`), não linear cru.
- **Jitter** (`paint.cc:623-665`): offset perpendicular à direção suavizada, `rand·draw_jitter·raio`.
  **Random** de raio/opacidade/rotação (`randomize.cc`): **Perlin signed** ao longo do arco (`scale=1/20`).
- **Pen-up** (`paint.cc:1673-1799`): trim de pontas com raio~0 (`1e-5`) → subdivide → smooth
  (position+opacity+radius) → **simplify RDP** em screen-space (`simplify_px`) → trim de auto-interseção →
  opcional outline / converte poly→bezier. Automerge de endpoints a **20 px**.

### Borracha (`erase.cc`) — 3 modos

- **Hard** (`:508`): corta a curva nas interseções com o círculo (interseção em **inteiros/pixel-space**
  pra estabilidade) e remove o span interno; `keep_caps` controla as pontas.
- **Soft** (`:704`): **reduz opacidade** por rings concêntricos (passo 2px), `opacity -= strength·falloff(dist)`;
  no pen-up, remove pontos com opacidade < `1e-4` e divide. `opacity_threshold = 0.05`.
- **Stroke** (`:868`): apaga o **traço inteiro** se qualquer segmento estiver a < raio. Materiais travados
  são preservados em todos os modos.

### Fill / balde (`fill.cc` + `draw_ops.cc`) — pipeline raster (NÃO é potrace)

1. **Fit to view** (`fill.cc:826`): buffer RGBA8 = tamanho da região × `fill_factor` (Precision), min 128²,
   zoom "fit" limitado a 5×, margem 20px.
2. **Render offscreen** (`:894`): ponto-semente verde no clique (4px); traços de fronteira em vermelho com
   **raio ×0.5** (fecha micro-gaps); extension lines em vermelho (1px).
3. **Flags + flood-fill** (`:606`): R>0=Stroke, G>0=Seed; moldura=Border. Flood 4-conexo do Seed, com
   **leak filter de 3px** (bloqueia propagação num eixo se houver Stroke a ≤3px naquela direção — fecha
   vazamentos finos). Encostar em Border = falha (ou inverte, no modo invert).
4. **Dilate/erode** (`:383`): `dilate_pixels` iterações 8-conexo = crescer/encolher o preenchimento.
5. **Traçado da fronteira** (`:462`): **Moore neighborhood** (8-dir, horário) → curva POLY; projeta de
   pixel→região→mundo.
6. **Pen-up** (`draw_ops.cc:1643`): smooth **20 iter** (influência 1.0) + `simplify_fixed` (mantém 1 a
   cada `2^fill_simplylvl`).
7. **Fechamento de gap por EXTENSÃO** (`draw_ops.cc:1023`): prolonga pontas e pontos de alta curvatura por
   `fill_extend_fac·(1/2000)` de mundo (modo Extend) ou põe círculos (modo Radius) que se conectam quando
   se sobrepõem; ajuste modal do gap com passo 0.02. É o "Gap Closure" interativo.

> Nota: `trace_util.hh` (potrace: `size_threshold=2`, `alpha_max=1.0`, `optimize_tolerance=0.2`) é do
> operador **Trace Image** (`trace.cc`), separado — não do balde.

### ► Decisão PH2D

- **Draw:** portar o loop de amostragem + **active smoothing com convergência** é a prioridade #1 da tool
  (é o que dá a mão boa). Curvas de pressão editáveis reusam a **falloff curve** que o Painter já tem
  (`HANDOFF_painter_falloff_curve`). Cuidado HR-5: Perlin e gaussiano usam transcendentais — checar gate.
- **Erase:** os 3 modos; o soft-erase (reduz opacidade) é o mais "pintura-like" e intuitivo — default.
- **Fill:** o pipeline raster (offscreen → flood com leak-filter → Moore → smooth+simplify) é robusto e
  GPU-friendly (o flood pode ir pra compute shader no futuro). O **Gap Closure por extensão** é a feature
  que faz o balde "funcionar de primeira" em line-art com aberturas — implementar cedo. Preview do fill
  antes de confirmar (o Blender mostra) = ganho de UX enorme.
- Nomes intuitivos: **Precision** (não "fill_factor"), **Gap Closure** (não "extension"), **Grow/Shrink**
  (não "dilate/erode").

---

## §6 — Reshape (sculpt de traço)  *(detalhe na W5)*

Brushes em `editors/sculpt_paint/grease_pencil/sculpt_*.cc`: **Smooth** (iterations=2 fixo),
**Push/Grab** (desloca pontos sob o pincel), **Thickness** (raio), **Pinch**, **Twist**, **Randomize**,
**Strength** (opacity), **Clone**. Todos = pincel com raio + força + queda, operando nos pontos dentro
da região. No PH2D vira a tool "Reshape" (mais intuitiva que "Sculpt").
