# Flip — Referência de algoritmos (Grease Pencil, Blender 5.2)

> **Consulte este doc antes de cada tópico** e vá ao fonte em `~/Downloads/blender-5.2-grease-pencil-ref/`
> (índice em [`00_README.md`](00_README.md)). **Clean-room:** leia o comportamento, reimplemente do
> zero; nunca copie código GPL. Citações `arquivo:linha` são do recorte 5.2. Cada seção termina com a
> **decisão PH2D** (como adaptar ao engine 2D-ortográfico).
>
> **Versão pinada:** a referência canônica é o recorte **5.2** vendorado. Exceções anotadas
> explicitamente: *Corner Types* (PR 143688, 2025, pós-5.2 — citado no `03` como ratificação do
> fix), *SSAA de render* (4.5) e *dots ilimitados/Ciallo* (2026, commit `20e256add6`). Se dois
> implementadores divergirem sobre "o que o Blender faz", vale o recorte 5.2.

Sumário: §1 Modelo de dados · §2 Engine de render (pipeline) · §2b Shader do traço · §2c
Antialiasing · §3 Tween · §4 Ops de curva · §5 Draw/Erase · §6 Fill · §7 Reshape (sculpt) ·
§8 Frames/Onion/Primitivas/Undo · §9 Materiais · §10 VFX · §11 Seleção/Multiframe/Cíclicas/UV ·
§12 Tradução GL→wgpu.

---

## §1 — Modelo de dados

**Hierarquia (GPv3):** `GreasePencil (datablock) → root LayerGroup → Layer(folha) | LayerGroup →
frames: Map<int,Frame> → Drawing → CurvesGeometry (strokes)`. O datablock guarda **um array plano de
drawings** (`drawing_array`, ordem arbitrária) e cada `Frame` referencia um drawing **por índice**;
vários frames podem apontar pro mesmo drawing (instância). Ref: `DNA_grease_pencil_types.h:205-503`,
`BKE_grease_pencil.hh` (wrappers C++ com `static_assert(sizeof) == DNA` + `reinterpret_cast`).

### Frames: chave = início; duração implícita; end-sentinel

`GreasePencilFrame { drawing_index; flag; type }` (`DNA:255-275`). Exemplo canônico
(`hh:457-474`): `{0:0, 5:1, 10:-1, 12:2, 16:-1}` = d0 em [0,5), d1 em [5,10), nada em [10,12),
d2 em [12,16). Dois mecanismos de duração: `GP_FRAME_IMPLICIT_HOLD` (segura até a próxima
chave) e o **end-sentinel** (`drawing_index == -1`).

**`add_frame(key, dur)`** (`grease_pencil.cc:1517-1563`):
```
se mapa contém key e NÃO é end-frame → FALHA (overwrite é responsabilidade do chamador)
insere/substitui; end_key = key + dur
remove end-frames órfãos consecutivos a partir da próxima chave
dur == 0 → flag IMPLICIT_HOLD e retorna
se não há próxima chave ou próxima > end_key → insere end() em end_key
# chave REAL dentro da duração trunca silenciosamente (sem erro, sem sentinel)
```

**`remove_frame(key)`** (`cc:1565-1602`) — **deletar pode INSERIR**: se o vizinho anterior tem
duração fixa (não implicit-hold), a chave vira end-sentinel em vez de sumir (senão o anterior
"vazaria" pra frente). Teste "removi ⇒ mapa menor" está ERRADO.

**Lookup pelo playhead** (`cc:1617-1638`): `upper_bound(frame)` e recua um; end-frame → nada.
O Blender paga `Map` + `sorted_keys_cache` lazy + DUAS dirty-flags (chaves vs valores —
trocá-las = cache stale intermitente); **o `BTreeMap` do PH2D elimina a classe inteira de bugs**.

**Tabela de invariantes do mapa (o implementador da W3 vai reinventar errado sem isto):**

| Operação | Invariante |
|---|---|
| iterar `frames()` | end-frames SÃO entradas do mapa — filtrar `is_end()` sempre |
| delete no meio de dur. fixa | chave vira sentinel (preserva a duração do vizinho) |
| insert sobre chave real | falha; overwrite = remover antes (é o que o paste faz) |
| insert com dur. engolindo ends | ends órfãos deletados em cascata; chave REAL trunca sem aviso |
| duplicate | `do_instance=true` compartilha drawing (+1 user); operador do editor SEMPRE copia |
| move/duplicate no dope-sheet | transacional: remove fontes ANTES de inserir; decrementa user só no overwrite |
| refcount | **NUNCA serializado** — recontado no load (`cc:174-186`); validado só em debug |
| compactação | `remove_drawings_with_no_users` (cc:3414-3526): swap in-place O(n) + **remap de TODOS os frames de TODAS as camadas** |

### Atributos — a lista canônica (nomes/domínios/defaults verificados)

- **Point**: `position` (float3) · `radius` (float, **0.01** — desde o 4.3 em unidades de MUNDO,
  por-ponto; não existe mais "thickness" de stroke) · `opacity` (**1.0**) · `vertex_color`
  (rgba, **(0,0,0,0)** = sem override, misturada SOBRE a cor do material) · `rotation` (float
  rad, **0.0**, giro da textura do dab) · `delta_time` (segundos desde o início do traço) ·
  `.selection` (**ausente = tudo selecionado**).
- **Curve**: `material_index` (**0**) · `cyclic` · `fill_color` (rgba, **(0,0,0,0)**) ·
  `fill_opacity` (**1.0**, separado do alpha p/ animar) · `fill_id` (**0** = sem fill; ≠0
  agrupa N curvas num fill composto COM BURACOS) · `start_cap`/`end_cap` (int8, **0=ROUND**,
  1=FLAT) · `softness` (**0.0**; = `1 − hardness` do brush) · `aspect_ratio` (**1.0**) ·
  `u_translation` (**0.0**) / `u_scale` (**1.0**) (deslizar/escalar textura ao longo do arco) ·
  `uv_rotation/uv_translation/uv_scale` (transform da textura do FILL) · `init_time` · `curve_type`.
- **Layer** (domínio no datablock): **ZERO builtins** — `tint_color`/`radius_offset` são
  convenções aplicadas na avaliação (até o Blender trata como gambiarra provisória).
- Colunas têm **default implícito** (coluna ausente = default — não materializa `opacity=1` p/
  5M pontos). `LEGACY_RADIUS_CONVERSION_FACTOR = 1/2000` (`hh:55`).

### Camada, grupos, máscaras, implicit sharing

- Árvore com **grupos aninhados** (layers sempre folha; root group obrigatório; ordem
  bottom-up). Herança **assimétrica**: `is_visible` = próprio E pais; `is_locked` = próprio OU
  pai; onion/masks seguem o visible. Testar só o flag local = camada "editável" dentro de grupo
  trancado.
- `opacity` + `blend_mode` ∈ {None, HardLight, Add, Subtract, Multiply, Divide} (6 modos — o
  PH2D tem 22 no compositor do Painter; reusar).
- **Masks por NOME de camada** (rename varre todas as máscaras + paths de animação) — o PH2D
  usa `LayerId` (rename-safe): **estritamente melhor, manter**.
- **Implicit sharing (CoW)** — a alavanca de perf decisiva do GPv3 (undo 6.6×, save 4.25×,
  keyframe novo 5.3×; arquivo −65%): copiar um Drawing compartilha os arrays de atributo E os
  4 caches computados (`SharedCache` — duplicar não re-triangula). Em Rust = `Arc` +
  `make_mut` por coluna. `clean-duplicates` compara spans **por ponteiro** (O(1) no caso comum).
- **Fills multi-stroke:** `fill_id` agrupa curvas; triangulação lazy: 1 curva → ear-clip;
  N curvas → CDT `CDT_INSIDE_WITH_HOLES` (faces CCW; triângulos tocando vértices de
  interseção são DESCARTADOS — fill auto-intersectante perde área por design). Cache com
  cliff: invalidação granular degrada pra full-rebuild se >50% das curvas mudou.

### ► Decisão PH2D (estado + gaps)

O `ph2d-flip` já espelha o núcleo (BTreeMap + end-sentinel + hold + refcount + SoA por-ponto).
Gaps p/ paridade, por prioridade: (1) **`Arc`-CoW nos buffers dos drawings** (pré-requisito de
duplicação/hold em escala e do undo barato — lição nº 1 do GPv3); (2) `fill_id` por-curva
(fills com buracos de N traços) quando o W4 chegar; (3) colunas laterais opcionais para
`rotation`/`u_translation`/`u_scale`/`aspect_ratio` quando pincéis texturizados chegarem;
(4) grupos de camada (o Flip é plano hoje — ok até a organização doer). NÃO portar: domínio
Layer de atributos (metadado NA camada, nunca em array paralelo), máscara por nome,
DrawingReference cross-datablock. Serialização: **nunca persistir refcount** (recalcular no load).

---

## §2 — Engine de render (o pipeline por cima do shader)

Refs: `gpencil_engine_c.cc`, `gpencil_cache_utils.cc`, `gpencil_draw_data.cc`,
`draw_cache_impl_grease_pencil.cc`, shaders `gpencil_layer_blend/mask_invert/depth_merge`.

### A regra de ouro do custo por camada

```
precisa_blend_pass := is_masked OU blend_mode != NONE OU opacity < 1.0    (cache_utils:386)
```
Se FALSO, a camada desenha **direto** no FB do objeto — o caso comum (N camadas alpha-over)
custa **zero** composites intermediários. Se verdadeiro: desenha num `layer_fb` próprio
(limpo color=(0,0,0,0) reveal=(1,1,1,1)) e compõe com um fullscreen pass por blend-state fixo
(HardLight = 2 passes mul+add — não há blend custom em MRT).

### Dual-target color + reveal

FB principal: depth `SFLOAT_32_DEPTH_UINT_8` + color `RGBA16F` + reveal (`RGB10_A2` unorm no
viewport; **`RGBA16F` com sinal quando `use_signed_fb`**). *Revealage* = 1−α por canal RGB.
Por que 2 alvos: blend-state fixo por pass ⇒ o fragment escreve `frag_color` premult no alvo 0
e `revealColor` no alvo 1; o blend pass da camada lê cor E cobertura; **holdout** escreve
`reveal = aaaa` (perfura a camada). **Tabela única do `use_signed_fb`** (3 relatórios
divergiam; esta é a reconciliação):

| Condição | reveal format |
|---|---|
| viewport, sem SUB/HardLight/glow-under | `RGB10_A2` unorm |
| camada com blend SUB ou HARDLIGHT (`cache_utils:405-408`) | `RGBA16F` (senão o unorm clampa negativo silenciosamente) |
| VFX glow modo Subtract ou glow-under (`shader_fx.cc:453-456`) | `RGBA16F` |
| render final (`engine_c.cc:182`, sempre) | `RGBA16F` |

### Máscaras, stencil, ordenação, depth-merge

- **Máscara**: 256 bits por camada; as camadas-máscara são **re-rasterizadas** para cada
  camada mascarada (O(n²) admitido em TODO); inversão via raster-op `LOGIC_INVERT` (não existe
  em wgpu — ver §12). Camada-máscara invisível ainda mascara (renderiza com opacity 0). Onion
  nunca é máscara.
- **Stencil como cull do blend pass**: geometria escreve 0xFF, o fullscreen blend testa EQUAL —
  só toca pixels rasterizados. Não é limpo entre camadas: a correção depende de os
  clear-colors serem **elementos neutros de todos os blend-states** (não documentado; se
  introduzir blend novo, verificar a neutralidade de (0,0,0,0)/(1,1,1,1)).
- **Ordenação 2D**: "all strokes with uniform depth (increasing with stroke id)"
  (`cache_utils:448`), teste GREATER, **depth clear = 0.0** (o par clear-0/GREATER é
  inseparável — inverter um só = tela vazia). Fill em `sid+1`, traço em `sid+2`. O depth 2D é
  também um **bitmap de cobertura**: o depth-merge com a cena 3D usa `depth != 0` como máscara.
- **Batch cache** (`draw_cache_impl`): **1 VBO/IBO por OBJETO** cobrindo todas as camadas e
  ghosts; camadas = faixas de índice; draw-calls consecutivas fundidas. Vértice = 3 texels
  (`pos+radius` · `mat,stroke_id,point_id,packed` · `uv_fill.xy,u_stroke,opacity`) + 2 de cor.
  Sinais como flags: raio<0 = end-cap flat, opacity<0 = start-cap flat, point_id<0 = cyclic.
  `packed` = aspect(9b) + uv_rot(9b: cos+sinal) + hardness(8b) + corner(6b). Padding de
  adjacência: 1 vértice antes/depois por curva (cíclica +1), `mat=-1`, `stroke_id` apontando a
  ponta OPOSTA (a costura cíclica de graça). A contabilidade `t_offset` engine↔cache é contrato
  frágil: drawings/strokes PULADOS ainda avançam offsets.
- A lição de perf histórica (T57829, pré-2.83): 1-2 fps com estado POR-STROKE → 100% mais
  rápido com 1 batch por objeto + depth-index. **Nunca estado por stroke.**

### ► Decisão PH2D

O Flip já está na arquitetura certa (passe wgpu → camadas → `LayerCompositor` 22-modos do
Painter; premult/linear/16F com resolve pro compositor 8-bit sRGB — decisão ratificada no W1).
**Não portar**: dual-target reveal (nosso compositor faz blend em shader — alpha escalar
premult basta), máscara O(n²) (quando houver layer-mask, textura cacheada com dirty-flag),
depth-merge 3D, luzes, overrides de viewport. **Portar**: a regra de ouro do FB por camada
(já implícita no nosso caminho), fill em `sid+1` (o fill sob o próprio traço sem passe extra),
e a disciplina 1-batch-por-camada.

---

## §2b — Shader do traço (o coração — detalhes no [`03`](03_traco_rasterizacao.md))

Resumo de consulta (fórmulas exatas, a mordida e o fix vivem no 03):

- **Vertex** (`draw_grease_pencil_lib.glsl:424-767`): vertex pulling por `texelFetch` de 4
  pontos (adjacência manual); quad por segmento com `x=(vid&1)*2-1, y=(vid&2)-1`; miter
  compartilhado = bissetriz; `miter_break` quando `-dot(line, line_adj) > miter_limit`
  (clampado a `cos 60°` → esticão ≤ 2r); break/round-cap estendem o quad `line*x` (1 raio).
  Espessura NOPERSPECTIVE (`thickness.x` clampada, `.y` unclamped p/ o fade).
- **Fragment** (`lib:65-146` + `gpencil_frag.glsl:433-583`): cápsula clampada → perfil
  (`hard>0.999 → step`; senão `smoothstep(0,1, d^mix(0,10,1-hard))`); corner ROUND (default)
  ignora p0/p3; BEVEL/MITER usam as cunhas p0/p3 para consistência entre quads; ordem
  alpha-discard → scene-depth → mask → `gl_FragDepth` constante (early-Z morto por design);
  fade sub-pixel `*= smoothstep(0,1, thickness.y)`.
- **Depth** (`gpencil_vert.glsl:81-97`): por-stroke `(sid+2)·2e-7` (GREATER, clear 0); flag
  `GP_STROKE_OVERLAP` → por-PONTO (auto-overlap com acúmulo); fill `sid+1`.
- **Dots/squares** (`lib:283-355`, `frag:337-516`): 1 quad por segmento + sprites resolvidos
  ANALITICAMENTE no fragment (interseção de cápsula desigual; loop back-to-front com early-out
  em α>0.999; randomização por hash do índice). Placement RADIUS tem forma fechada logarítmica
  p/ círculos tangentes em taper.
- **Sentinelas em 3 camadas**: `mat=-1` (CPU) → NDC degenerado (vertex) → `p0==p1` (fragment,
  flat = igualdade exata). Traço de 1 ponto vira dot — nunca descartar segmento degenerado.

---

## §2c — Antialiasing (decisão consolidada no [`03 §7`](03_traco_rasterizacao.md))

O GP usa **SMAA 1x preset HIGH** (3 passes fullscreen + 2 LUTs ~176KB), com pegadinhas: o
"threshold" da UI é na verdade um **ganho do luma** (threshold real fixo 0.1); o edge-detect
roda sobre color E reveal com `max` (tinta escura sobre transparente é INVISÍVEL pro luma da
cor — quem denuncia é o reveal); o resolve é também o COMPOSITOR dual-source (roda mesmo com
AA off); filtro linear nas LUTs é parte do algoritmo. Para render final (F12): **acúmulo**
Halton(2,3) → gaussiana σ=0.284 truncada ±0.93 → jitter da projeção → média corrente em
`log2(c+0.5)` com snap de alpha (buffer sempre resolvido p/ preview progressivo). O 4.5+
adicionou SSAA de render e removeu o min-thickness.

► Flip: analítico (fwidth) no viewport + fade sub-pixel + acúmulo Halton no export; SMAA
opcional futuro só do reference MIT. Detalhe e justificativas: 03 §7.

---

## §3 — Tween (interpolação / inbetween)

Refs: `interpolate.cc` (operador) + `geometry/intern/interpolate_curves.cc` (motor).
O upgrade planejado por cima disto (matching espacial + espiral log) vive no [`04 §2`](04_alem_do_blender.md).

### O algoritmo do GP, exato

1. **Intervalo** (`interpolate.cc:200-241`): prev = keyframe em/antes do atual; next = próximo;
   pula end-frames e (se `exclude_breakdowns`) BREAKDOWNs. Se o frame atual É um keyframe, ele
   vira o extremo A e **é SOBRESCRITO** pela interpolação (snapshot p/ cancel).
2. **Pareamento POR ÍNDICE** (`:244-315`): i-ésima (selecionada) de A ↔ i-ésima de B; seleção
   só vale se AMBOS os lados têm algo selecionado; excedentes de B nunca aparecem; sem par =
   cópia estática (fator 0/1) — **estável, não pisca**.
3. **Auto-flip** (`:427-461`, default FlipAuto): se as cordas ponta-a-ponta (A.first→B.first,
   A.last→B.last) se CRUZAM (teste 2D) → flip, com desempate por distâncias se o ângulo entre
   elas < **15°**; sem cruzamento → flip se `dot(dirA, dirB) < 0`.
4. **Contagem destino = MAX(A, B)** com `sample_curve_padded` (`interpolate_curves.cc:87-156`):
   os pontos da curva MENOR são preservados EXATAMENTE (fator 0) e os extras distribuídos
   pelos segmentos ∝ comprimento de arco — em t=0/1 a forma dos extremos é reproduzida
   pixel-a-pixel (re-amostragem uniforme NÃO tem essa propriedade — não usar). O flip é
   resolvido no espaço do MAPA de amostragem (`reverse_samples` — off-by-one fácil em cíclicas).
5. **Mistura** (`:1006-1267`): todo atributo float por-PONTO interpola (posição, radius,
   opacity, vertex_color...); atributos de CURVA int/bool/cor **copiam de A** (material não
   crossfade; fill_color salta — decidir interpolar no Flip). Lerp **não-clampado**: fator
   global clamp [-1, +2] = **overshoot é ferramenta**, não bug.
6. **Fator**: modal usa `(F−prev)/(next−prev+1)` (note o **+1**); sequência usa `(next−prev)`.
   Dois denominadores DIFERENTES (herança GPv2) — decidir conscientemente no port. O fator é
   único (da camada ativa) para TODAS as camadas — computar POR CAMADA é estritamente melhor.
7. **Sequência + easing** (`:1084-1249`): passo default 1; Linear/CurveMap custom/10 equações
   clássicas com defaults por tipo (Back/Bounce/Elastic → **out**; Circ/Cubic/Expo/Quad/Quart/
   Quint/Sine → **in**); constantes `back=1.702`, `amplitude=0.15`, `period=0.15`. Cada
   inbetween nasce keyframe **BREAKDOWN** → re-rodar com `exclude_breakdowns` re-interpola
   entre os extremos ORIGINAIS (regeneração idempotente — o mecanismo do re-tween).
8. **Smoothing pós** opcional (factor 0..2, steps 1..3, `keep_shape=true`) — amacia a mistura
   de densidades diferentes.

**Bugs latentes do original (NÃO copiar):** o agrupamento por frame-pairs é morto (nunca
atualiza `prev_from_frame` — cada par vira grupo de 1; "consertar" só o agrupamento estoura um
array dimensionado por grupos); e há `fill(0.0)` de array INTEIRO onde deveria ser por-fatia
(wipe latente de fatores de pares anteriores quando contagens diferem). Num port: fills por
fatia, iteração direta por par.

### ► Decisão PH2D

W3 = o GP literal (por-índice + padding-ao-MAX + auto-flip + easing reusando
`ph2d-anim::Interp`), com 3 correções baratas: fator POR CAMADA, fills por fatia, e o scrub
por posição absoluta do mouse (UX boa do GP). Órfãos de B: fade-in opcional (gate por flag) em
vez de "pipocar". O Tween v2 (matching espacial + espiral logarítmica + UI de correção) está
especificado no `04 §2` — mesma estrutura de dados, sem refactor.

---

## §4 — Operações de curva (smooth · simplify · fit · resample · fillet · trim · merge · outline)

Refs: `geometry/intern/*_curves.cc` + `grease_pencil_edit.cc` + `grease_pencil_segments_geom.cc`.

- **smooth** (`smooth_curves.cc:20-148`): **não é média iterada** — é UM passe com pesos
  binomiais por recorrência (`w *= (n_half+offset)/(n_half+1-offset)`), custo linear em
  `iterations`. `keep_shape` = diferença de duas gaussianas (kernel parcialmente NEGATIVO — um
  unsharp disfarçado que não encolhe a forma; só em position). Endpoints fixos
  (`smooth_ends=false`); bordas clampadas com peso ESCALADO (não zerado); normalização final
  soma `w−w2` do ponto central. NÃO é in-place (buffer por curva; in-place = feedback).
  Seleção parcial: cada range roda como curva aberta independente.
- **simplify** (`simplify_curves.cc:27-118`): RDP com pilha explícita, **genérico sobre
  atributo** (λ pela posição, distância medida no atributo interpolado — dá "simplify por
  radius/opacity" de graça). Epsilon default 0.01. Cíclica: só o último ponto é testado contra
  a corda (não é RDP cíclico completo). Máscara invertida (`points_to_delete` in/out no mesmo
  buffer). Modos: FIXED (`i % 2^steps`, port = `1 << step`), SAMPLE (resample 0.05 — **traço
  curto colapsa em 1 ponto**, surpresa de QA), MERGE.
- **resample** (`resample_curves.cc`): `count = int(len/sample_len)+1`; amostra uniforme por
  arc-length; interpola TODOS os atributos de ponto; **tudo vira POLY** (handles descartados).
- **fit** (`fit_curves.cc`): delega à lib C `curve_fit_nd` (Schneider; **o Blender não tem um
  inline**) — método Refit + HIGH_QUALITY; endpoints de abertas = quinas forçadas; **atributos
  pós-fit são COPIADOS do ponto original mais próximo via `orig_index_map`** (não interpolados
  — preserva picos de pressão). ► Reusar o Schneider do PH2D (`curve_refit.rs`) + adicionar o
  `orig_index_map`.
- **fillet** (`fillet_curves.cc`): `displacement = r·tan(θ/2)`; arco por cortes ou 2 handles
  bezier `4/3·r·tan(θ/4)`; `limit_radius` considera o raio do VIZINHO. Não exposto como op no
  GP (é infra de GN) — candidato a round-corners do Reshape.
- **trim/cutter** (`segments_geom.cc`): interseções todas-contra-todas O(N²) (TODO admitido —
  usar grid/BVH 2D) com **paddings LOAD-BEARING** (bbox +2px; ±1px ao longo da aresta; snap de
  fator a 0/1 com eps 1e-4 — a religação depende de igualdade float EXATA que só funciona por
  causa do snap). Curva = lista de `Segment(ponto+fator)`; segmentos tocados pelo laço saem; a
  religação caminha um grafo (±(index+1)) juntando sobreviventes com interpolação de atributos
  no corte; pontas cortadas = cap FLAT. **O cutter é um reconector de grafo, não uma borracha.**
  A MESMA infra serve dash (modificador, fora do recorte) e eraser vetorial futuro.
- **merge by distance** (`grease_pencil_geom.cc:181-344`): **KD-tree 1D com a distância-ao-
  longo-da-curva como coordenada** — só vizinhos na ordem do traço fundem (nunca
  auto-interseções). Atributos promediados (posição também). Em 2D um sweep linear resolve.
- **outline** (`geom.cc:447-932`): 100% 2D por dentro. Aberta → UMA cíclica (cap + lado
  direito + cap + lado esquerdo reverso); cíclica → DUAS (perímetros interno/externo, que
  viram UM fill com furo via remap de fill_id). Corner por ponto: round = arco `2^(subdiv+1)+1`
  pontos; bevel; miter com guard anti-divisão. `radius_efetivo = max(radius + offset, 0)`.

### ► Decisão PH2D

Portar fielmente: o kernel binomial completo (Reshape/polish — irreproduzível com blur comum),
o RDP genérico com semântica de máscara, o resample por arco, o motor de `Segment` do trim
(base do cutter E dash futuro), o KD-1D (como sweep), o outline. Reusar: Schneider próprio.
HR-5: o binomial é polinomial (preferir à forma `exp` da aproximação).

---

## §5 — Draw · Erase (a mão do artista)

Refs: `paint.cc`, `paint_common.cc`, `draw_ops.cc`, `paint_cursor.cc`, `erase.cc`,
`grease_pencil_utils.cc`.

### Amostragem e espaçamento (`paint.cc:667-988`)

```
dist < 2.0 px → SOBRESCREVE o último ponto; radius/opacity = max(novo, anterior)  # "engorda parado"
senão: subdivide com passo max(spacing% · raio_px, 0.25 px)   # denso e uniforme em qualquer velocidade
```
Cada ponto grava `delta_time`; a curva grava `init_time` (base pro futuro "build" — barato
agora, caro de retrofitar). Pressão → raio via `curve_sensitivity`, → opacity via
`curve_strength` (CurveMappings EDITÁVEIS — nada de gamma fixo). Jitter: **só perpendicular**
à direção suavizada (EMA fator 0.3), armazenado POR PONTO e re-somado a cada frame (não
destrutivo). Randomize de raio/opacity/rotação ao longo do arco: **Perlin 1D contínuo**
(`scale=1/20`), não ruído branco.

### Active smoothing — o "assentar" premium (`paint.cc:544-621`)

**Não é blur — é curve fitting com janela convergente** (o PH2D hoje faz binomial-only; a
diferença aparece em "S" rápido e cantos de letra):

```
janela = pontos[start..fim], roda com ≥ 8 pontos
1. detecta QUINAS (raio 5→30px, 64 amostras, threshold 0.6 rad) — cantos não derretem
2. pré-blur binomial 3 iterações (influence = active_smooth)
3. fit cúbico com erro_max = 5px · active_smooth, quinas pinadas
4. reamostra a bézier a 32 pts/segmento
5. "morph": re-parametriza os pontos originais por arc-length sobre a curva
6. cada ponto acumula o HISTÓRICO de fits e usa a MÉDIA (anti-pop por integração temporal)
7. converge quando muda < 0.1px (contando do início, para no 1º não-convergido) → janela encolhe
```

O parâmetro do usuário entra em DOIS lugares (influência do pré-blur E tolerância do fit).
A janela reescreve geometria "commitada" a cada sample — o render precisa tolerar a cauda
mudando por frame (► manter o traço em curso num buffer "wet" separado; materializar no pen-up).

### Pen-up (ordem exata, `paint.cc:1673-1799`)

trim de pontas raio<1e-5 (o "rabo de pressão zero" — sem isso todo traço de tablet termina com
cauda invisível) → subdivide (só se simplify==0 — antagonistas) → smooth → **simplify RDP em
px de TELA do momento do desenho** (congela as posições de tela num atributo temporário; RDP
no espaço do doc muda de significado com o zoom) → trim de auto-interseção (opc.) → conversão
de tipo → **automerge de endpoints a 20 px** (fechar formas em 2 traços).

### Borracha (`erase.cc`) — corte analítico, não delete de pontos

Interseção segmento×círculo em **INTEIROS** (int64 na quadrática; pixel-space) — estabilidade
na classificação dentro/fora/borda (em float, pontos na borda oscilam e o traço "pisca"; se
portar em float, epsilon explícito). Reconstrução por `PointTransferData` + `compute_topology_change`:
- **Hard**: remove o interior, insere pontos exatos na borda do círculo.
- **Soft** (`:587-866`): amostra o falloff como **anéis** (passo 2px) → poda a 0.05 (anel que
  cruza o threshold é DESLOCADO pro raio exato — senão o corte "pulsa" 2px) → RDP (raio,
  opacity) eps 0.1 → interseção por anel, opacity subtraída com clamp; anel < 0.05 vira corte
  duro. Pen-up: RDP das opacities inseridas (0.01) + remove <0.0001. **Um único algoritmo
  topológico serve os dois erasers.**
- **Stroke**: remove a curva se QUALQUER segmento < raio.
- **Cíclica cortada**: vira aberta com o array ROTACIONADO pro pivô da última interseção (o
  fecho sobrevive como miolo contíguo); pontas de corte = cap FLAT salvo `keep_caps`.
- **Oclusão NÃO existe** (apaga tudo que projeta no círculo, mesmo coberto); reconstrói a
  geometria INTEIRA por sample (► broadphase por bbox de curva num doc grande).

### Autokey (`grease_pencil_frames.cc:344-378`) — semântica POR TOOL

Garantido no **invoke** (1× por gesto; undo agrupa frame+traço): desenho → frame EM BRANCO
(ou duplicata com "Additive Drawing"); **borracha/tint/sculpt → SEMPRE duplicata do anterior**
(errar isso faz a borracha "apagar" quadros que o usuário nem via). Desenhar em cima de
keyframe existente NÃO cria chave; desenhar no "rabo" do hold cria. Freehand **não cancela**
(`test_cancel` = false — commit incremental); primitivas/interpolate cancelam (restore de
snapshot) — regra: modais paramétricos cancelam, tinta não.

### ► Decisão PH2D

Prioridade nº 1 da mão: portar o active smoothing COMPLETO (com as constantes tunadas) — é o
que separa "bom" de "medíocre". Spacing/override/pressure já temos parcialmente; conferir os
`max()` do override. Borracha: portar a discretização por anéis do soft (a nossa reduz
opacidade por-ponto sem inserção — fica "blocada" em segmentos esparsos). Autokey por-tool na
W3 (quando frames chegarem à UI). Estabilizador (lazy mouse): estágio separado do GP (fora do
recorte) — implementar como filtro do InputSample (string-pulling raio+fator), 1 lugar só.

---

## §6 — Fill / balde (pipeline completo)

Refs: `fill.cc`, `draw_ops.cc`, `trace.cc`, `grease_pencil_image_render.cc`. SOTA além do GP:
[`04 §3`](04_alem_do_blender.md).

**Dois solvers** (por brush): **pixel** (default, robusto, com overlay/UX completos) e
**Delaunay** (5.2, vetorial, zoom-independente, sem overlay). **O resultado dos dois é
GEOMETRIA**: um stroke cíclico novo com `fill_id=1` e `hide_stroke=true` (só o preenchimento
aparece) — apagar/mover/animar o fill é manipular um stroke comum.

### Pixel solver

1. **Fit-to-bounds** (não à tela!): buffer = bbox de TODOS os strokes visíveis + clique, margem
   20px, `pixel_scale` = Precision do brush, mín 128², zoom clampado 5× — determinístico em
   qualquer zoom de câmera.
2. **Render offscreen**: semente verde 4px no clique (~13px de tolerância — seed sobre Stroke é
   ignorado); strokes de fronteira em vermelho com **`radius_scale = 0.5`** — meia-espessura, A
   LINHA MAIS IMPORTANTE do subsistema: o contorno traçado fica DENTRO do corpo visual da linha
   e o fill entra por baixo (o mesmo insight do "fill up to vector paths" do CSP; espessura
   cheia = halo; zero = vazamento); linhas de extensão em 1px. Threshold: **qualquer pixel com
   r ≥ 1/255 é boundary** (o AA do render dilata ~½px e fecha micro-frestas — parte do algoritmo).
3. **Flood fill** DFS 4-conexo com **leak filter CRUZADO de 3px** (Stroke próximo na VERTICAL
   bloqueia expansão HORIZONTAL, e vice-versa — inverter a semântica faz o filtro AJUDAR o
   vazamento). Tocar a borda da imagem = **falha total** ("No fill created"); modo invert usa
   Ignore + inversão do bitmap.
4. **Dilate/erode** 8-conexo (`dilate_pixels` do brush, ±).
5. **Moore trace** (horário, offset de direção `+5` = inverte+avança — errar trava em
   ping-pong); inícios em transições vazio→cheio (buracos saem NATURALMENTE como contornos
   separados → mesmo `fill_id` = furos preservados); loops abertos descartados.
6. **Pós**: smooth 20 iterações + decimação `2^simplify` (► trocar por RDP + fit Schneider —
   ver 04 §3); projeta imagem→região→mundo.

### Gap closure (`draw_ops.cc:1023-1150`) — a feature que faz o balde "funcionar de primeira"

- **Extend**: prolonga PONTAS na tangente E detecta quinas mid-stroke onde o raio de curvatura
  < espessura do traço (rearranjo `dist_prev + dist_next < 2·|Δtan|·raio` — é por isso que o
  GP fecha cantos em "V" que outros baldes não fecham), com **corte por colisão** via BVH 2D
  (raycast em DOIS passes contra as extensões ORIGINAIS — ordem-independente; 3 exclusões:
  própria linha, segmento de origem, adjacente).
- **Radius**: círculos nas pontas; pares cujo centro cai dentro do outro geram **linha
  centro-a-centro** (os círculos NUNCA tocam o raster — são overlay); ajuste modal ao vivo
  (scroll ±0.02, helpers visíveis só nos gaps pendentes — a killer feature de UX).
- Cíclicas nunca ganham extensão. Unidades de MUNDO (`fator legado 1/2000`).

### Delaunay solver (resumo)

CDT_FULL sobre os pontos avaliados + 4 cantos de bbox com pad 1.1× (garante que o "oceano"
tenha gargalos maiores que qualquer vão interno — não é estético); watershed por gargalo de
aresta a partir do triângulo do clique; `gap_factor` semeia hints extras; fronteira encadeada
em O(N). Sem overlay, sem smooth pós. ► v2 do Flip; começar no pixel.

### Multiframe + atributos

`target_frames = {atual} ∪ {selecionados}`; o pipeline RODA POR FRAME (N fills independentes —
a região pode mudar de forma). Fill entra ATRÁS (`paint_onback`) ou no fim; material/vertex
color do brush; z-order + matriz de textura ancorada no clique.

### ► Decisão PH2D (W4)

Portar o pixel solver fielmente (constantes acima) com 4 upgrades já decididos (04 §3):
buffer de flags dedicado `Vec<u8>` (não abusar do canal R — TODO do próprio Blender);
vetorização por RDP+Schneider; **fechamentos materializados como strokes invisíveis
persistentes** (twist do Harmony — re-fill sobrevive); modos Paint/Paint-Unpainted/Unpaint +
Grow/Shrink pós-vetorização (offset CAD do Painter). Nomes: **Precision** (fill_factor),
**Gap Closure** (extend), **Grow/Shrink** (dilate/erode). Fill é operação de CLIQUE — span
fill CPU, sem GPU (JFA não é geodésico; readback é inevitável pro trace).

---

## §7 — Reshape (sculpt de traço) — a W5 inteira

Refs: `sculpt_paint/grease_pencil/sculpt_*.cc` + `paint_common.cc` + `grease_pencil_intern.hh`.

### A infra comum (o trait do pincel)

Contrato de 3 callbacks `on_stroke_begin/extended/done` com `InputSample {mouse_position,
pressure}` — casa direto com o `CanvasPaintTool` do ADR-0040-am3. Por amostra:

```
raio = raio_brush · curve_sensitivity(pressure)         # se size-pressure
base = alpha_brush · (alpha_pressure ? pressure : 1) · multi_frame_falloff
influence = base · falloff_curve(dist_ao_cursor, raio)  # CurveMapping presets Smooth/Linear/Sharp...
```

**Sem normalização temporal**: a "dose" é 1 aplicação por SAMPLE de input — mover devagar
aplica mais (um fork que gere samples por timer muda a sensação de TODOS os pincéis).
Invert = `BRUSH_DIR_IN XOR Ctrl`. Auto-masking congelado no DOWN (seleção / material ativo /
stroke-layer-material sob o cursor com threshold **20px** / camada ativa) — arrastar pra fora
nunca pega strokes novos (deliberado: a máscara define O QUE, o traço define QUANTO).

### Os pincéis (matemática + constantes que fazem o GP "sentir" como GP)

| Pincel | Fórmula | Constantes |
|---|---|---|
| **Smooth** | kernel binomial (§4) em position/opacity/radius/rotation, influence = peso de mistura | `iterations = 2` HARD-CODED; projeta TODOS os pontos (o kernel lê vizinhos fora da máscara) |
| **Push** | `pos += delta_mouse · influence` | — |
| **Grab** | máscaras+pesos congelados no DOWN (`pressure=1.0` fixo!); por sample: `pos += delta · peso_congelado` | o conjunto capturado nunca é reavaliado |
| **Pinch** | `s = influence²/25; pos += (cursor−pos)·(±s)` | quadrático ÷25 — deliberadamente lento e "cremoso" |
| **Twist** | rotação rígida `±1° · influence` ao redor do cursor (matriz 2D em tela) | 1°/sample |
| **Thickness** | `radius = max(radius ± influence·0.001, 0)` | aditivo, NUNCA multiplicativo (efeito absoluto) |
| **Strength** | `opacity = clamp(opacity ± influence·0.125, 0, 1)` | idem |
| **Randomize** | hash determinístico re-semeado POR SAMPLE; posição SÓ perpendicular à direção do mouse | random-walk browniano se parado; seeds por canal |
| **Clone** | paste do clipboard 1× no DOWN, centrado no cursor, camada ativa | não é um pincel — é um comando (modos contínuos do GPv2 admitidamente quebrados/removidos) |

**Multiframe** (a feature matadora pra animação): com frames selecionados, o MESMO gesto
esculpe N quadros com falloff por distância temporal — UMA CurveMapping com o frame ativo em
X=0.5, antes em [0,0.5), depois em (0.5,1] (atenuação assimétrica de graça). Só brushes
respeitam o falloff; ops discretas usam 1.0.

### ► Decisão PH2D (W5)

Trait `ReshapeBrush` com o contrato de 3 callbacks; portar os pesos/constantes exatos da
tabela. Em 2D-orto a projeção colapsa (`delta_canvas = delta_tela / zoom`; crazyspace =
identidade) — os 9 pincéis portam quase sem mudança. Randomize com splitmix64 (mesma família
do `jitter.rs`; replay-safe). Multiframe-falloff entra quando a tira de frames tiver seleção
de frames (W3) — reservar o `falloff` na assinatura desde o início. Clone = comando (Ctrl+C/V
de strokes), não brush.

---

## §8 — Frames · Onion · Primitivas · Undo (editor)

Refs: `grease_pencil_frames.cc`, `grease_pencil_layers.cc`, `grease_pencil_utils.cc`,
`grease_pencil_primitive.cc`, `grease_pencil_undo.cc`, `gpencil_cache_utils.cc`.

### Onion skinning — o algoritmo exato (pro W3)

**Seleção dos vizinhos é função pura no editor** (não no engine): `get_frame_id`
(`utils.cc:534-603`):

```
para cada key ≠ key_corrente (identidade de KEY, não de drawing — com hold, o key ativo
                              pode estar N quadros atrás; ele é o Δ=0):
  filtro por tipo: S.filter != 0 e bit (1<<frame.type) fora → pula   # bitmask VAZIA = tudo passa
  modo SELECTED e frame não-selecionado → pula
  Δ = ABSOLUTE ? key − F              # quadros de cena
    : RELATIVE ? index − index_F      # contagem de keys (pula holds — o modo "por desenho")
  se F antes da 1ª chave: Δ += 1      # senão a 1ª chave (futura) sairia como "corrente"
  SHOW_LOOP: wrap circular somando/subtraindo (last+1)  # FIXME upstream: assume início em 0
  ABSOLUTE/RELATIVE: |Δ| fora de [num_before, num_after] → pula   # SELECTED IGNORA o range!
corrente entra por último com Δ=0
```

**Tint/alpha** (`cache_utils:217-264`): cor = custom ? (Δ>0 ? after : before) : tema;
**tint.a = 1.0 — o ghost é a silhueta 100% RECOLORIDA** (não um blend; é o look clássico);
`alpha = (fade ? 1/|Δ| : 1) · opacity`, clamp **[0.1, 1]** (ou [0.01,1] se opacity==0 — ghost
nunca zera). Fade satura no 3º ghost (1, 0.5, 0.33, ... piso 0.1). No shader:
`rgb = mix(rgb, tint.rgb, tint.a)`, alpha do passe multiplica ANTES do discard.
Defaults DNA: opacity 0.5 · RELATIVE · FADE+CUSTOM_COLORS · 1 antes/1 depois ·
before **verde (0.145, 0.420, 0.137)** · after **azul-roxo (0.125, 0.082, 0.529)**.
**Gates**: `do_onion = show && !hide_overlay && !playing` (some no play — regra de produto) e
NUNCA no render final. Per-layer: flag herdável pela árvore. "Current frame only" = before=after=0
(mas SELECTED ignora). Ghosts desenham ANTES do corrente (mesmo VBO, 1 draw + 2 uniforms por
ghost — custo ~zero).

### Ops de frame e camada

- Duplicar no dope-sheet é **transacional** (buffer de transformação; consome no fim do drag;
  remove fontes ANTES de inserir).
- Camada nova ganha keyframe imediato no frame atual.
- `insert_duplicate_frame(do_instance)` — instância = +1 user (ciclos de graça); o operador do
  editor sempre COPIA (instância só existe no modelo — anti-padrão do GP: 2 anos sem UI).
- Clean-duplicates: comparação por IDENTIDADE de span primeiro (O(1) com CoW).

### Primitivas interativas (Line/Polyline/Arc/Curve/Box/Circle)

Um operador modal com state machine (Idle/Extruding/Grab/Drag/DragAll/RotateAll/ScaleAll/
ChangeRadius/ChangeOpacity). **A matemática já é 2D de tela** (De Casteljau quadrático/cúbico,
círculo paramétrico, box por cantos lerpados) — port literal. A curva provisória vive NO
documento (última curva, reescrita por evento; preview = resultado por construção); cancel =
remove a última curva. Subdivisões default POR TIPO: Line/Polyline 6, Arc/Curve **62**, Box 3,
Circle **94** (círculo denso = "desenhado à mão" com pressão — subamostrar muda a cara).
Snap Shift: 8 ângulos (`sin 22.5°`); Alt = crescer do centro; hit 20px (fallback 600px);
automerge de saída 30px. ► Portar no padrão de tool do PH2D (gesto + snapshot único no
release, como o shape-editor do Painter) — o gesto precisa estar dentro da janela "gesto em
curso" do undo global (diff só sem gesto ativo).

### Undo

GP: undo type próprio com snapshot completo (viável só por implicit sharing; decode recria a
árvore inteira e re-acha o ativo POR NOME). ► Flip: **já resolvido** — o doc está no
`ProjectState` do undo global (snapshot+diff). A lição a importar é o **Arc-CoW** (§1) para o
snapshot custar O(frames tocados); o truque de comparar spans por ponteiro é o mesmo que
barateia o nosso diff.

---

## §9 — Materiais (a superfície de render; o Flip NÃO tem materiais — mapeamento abaixo)

Refs: `gpencil_shader_shared.hh` (struct gpMaterial), `gpencil_frag.glsl` (get_color),
`gpencil_draw_data.cc` (CPU→UBO).

- UBO de pools de **255 materiais**; `gpMaterial` = stroke_color, fill_color, fill_mix_color
  (gradiente), fill_uv_rot_scale/offset, alignment_rot (cos/sin), texture_mix, u_scale, flags.
  **O material id viaja nos bits ALTOS (≥19) do próprio `mat_flag`** — flags novas têm que
  caber abaixo.
- Flags: ALIGNMENT (dots: eixo do sprite = Path/Object/Fixed) · OVERLAP (depth por-ponto) ·
  DOTS (círculo vs quadrado) · TEXTURE_USE/PREMUL/CLIP (stroke e fill) · GRADIENT_USE/RADIAL ·
  HOLDOUT (stroke e fill) · placement COUNT/DENSITY/RADIUS · DOTS_RANDOMIZATION.
  (`GP_STROKE_TEXTURE_STENCIL` é flag MORTA — zero usos.)
- **O composite universal em 2 varyings** (a peça mais portável): todo o pipeline de cor
  (solid/texture/gradient/vertex-color/tint/opacity) colapsa em `col·color_mul + col.a·color_add`
  — UM FMA no fragment; a textura age sempre como STENCIL (alpha nunca somado). Gotcha de
  hardware (#156278): 1.0 interpolado entre vértices chega ≠1.0 — clamp perto da identidade.
- Gradiente de fill: `fac = RADIAL ? |uv·2−1| : uv.x`; as 2 cores re-buscadas NO fragment pelo
  matid (não cabem nos varyings). Fill-UV = projeção planar POR CURVA (frame no 1º ponto) ×
  transform do material com pivô no centro.
- `u_stroke` (textura ao longo do arco): comprimento acumulado × `500/texture_pixsize` (o 500
  é herança do legado); por-curva `u_translation`/`u_scale` deslizam/escalam.

### ► Mapeamento pro Flip (cravado)

| gpMaterial | No Flip |
|---|---|
| stroke_color / fill_color / gradiente+flip | **campos do `FlipStroke`** (cor pertence ao traço — modelo Procreate; gradiente de fill = decisão do traço, por-stroke) |
| softness/hardness, caps, cyclic, opacity/radius por-ponto | já são do stroke ✓ |
| modo Line/Dots/Squares + alignment + placement + randomização + texturas Shape/Grain + hardness default | **brush preset** (o "material" do Flip) — randomização casa com o `jitter.rs`; texturas reusam os conceitos ADR-0100 |
| HOLDOUT | fora (se um dia: blend mode de camada, não flag de traço) |
| vertex_color × material mix | fora (nossa cor por-ponto JÁ é a final) — **mas implementar o par mul/add desde já** (1 FMA; deixa o slot de textura/tint pronto) |
| OVERLAP | flag *Self Overlap* futura (03 §8) |

---

## §10 — VFX (efeitos por objeto) — referência adormecida

Refs: `gpencil_shader_fx.cc`, `gpencil_vfx_frag.glsl`. **Nenhuma wave atual usa isto** — está
aqui para quando uma wave "FX" for aberta; NÃO é pendência.

- Arquitetura: cadeia de passes fullscreen com **ping-pong** entre object_fb/layer_fb (o
  layer_fb reusado como scratch), aplicada APÓS o blend das camadas do objeto e ANTES da
  composição na cena; composição final em 2 draws (mul+add — sem blend custom em MRT).
- Efeitos (kernel gaussiano comum: `peso = exp(−x²/(2·0.35²))`, x∈[−1,1] — o **σ=0.35 do raio**
  é o que faz um blur "bater" visualmente com o GP): **Blur** (separável rotacionado, elíptico;
  modo DOF = 3D, ignorar) · **Colorize** (grayscale/sepia/duotone/tint/transparent — duotone
  usa factor como LIMIAR; typo de 15 anos no luma: coef do azul 0.723 em vez de 0.0722 — ►
  corrigir e documentar a divergência) · **Rim** e **Shadow** (= a alma do GP: efeitos de
  SILHUETA — luz de recorte e sombra transformada; usam o truque "stash no attachment irmão"
  entre passes; convenções de out-of-bounds OPOSTAS: rim=0, shadow=1) · **Glow** (threshold
  DURANTE o blur, com o normalizador contando amostras rejeitadas — o falloff suave vem disso;
  "renormalizar direito" muda o look; glow-under exige alpha extra) · **Pixelate** (grade
  ancorada no OBJETO, não na tela — senão "nada" quando anima; meio-texel de correção) ·
  **Wave/Swirl/Flip** (UV remap; flip espelha em torno do centro da TELA — surpresa a decidir).
- ► Quando portar: cadeia sobre UM RGBA16F premult (sem reveal — nosso compositor blenda em
  shader); blend programável no lugar dos multi-draws; raios em px de canvas × zoom (sem o
  fator 2000). Sobreposição com o Painter: colorize/blur/glow ≈ 40% cobertos (estender bloom
  p/ glow; blur ganhar anisotropia) — rim/shadow/pixelate/wave/swirl são novos e pequenos.

---

## §11 — Seleção · Multiframe · Cíclicas · UV (transversais)

Refs: `grease_pencil_select.cc`, `grease_pencil_utils.cc`, `grease_pencil_edit.cc`.

### Seleção (base do Edit Mode futuro e do Reshape)

- Atributo `.selection` em domínio **Point OU Curve** (nunca ambos; ausente = tudo
  selecionado). Modo da toolbar → domínio: point→Point, stroke→Curve, **segment→Point + pós-processo**.
- **Segment mode** = corte por interseção VISUAL: raycast de cada segmento contra BVH 2D do
  frame (ignorando 3 vizinhos); hit = início de segmento; **cíclica sem corte tem ZERO
  segmentos → fallback "1 ponto seleciona a curva toda"**; o último segmento de cíclica enrola
  em DOIS ranges.
- Trocar de modo: point→stroke promove fill com QUALQUER ponto selecionado; o atributo é
  materializado no domínio novo (half-selected só existe em Point).
- `SEL_OP_SET` desmarca o complemento explicitamente.
- ► Flip: `Vec<bool>`/bitset paralelo + conversão de domínio explícita (any/broadcast); o
  segment mode é 100% screen-space — port natural. Transform de seleção = op comum consumindo
  a mesma lista (não existe transform específico de GP).

### Multiframe (ortogonal aos ops — pegar de graça)

`alvo = keyframes selecionados (dedup por Drawing*!) + frame atual como fallback`. Resolvida
ANTES; os ops só iteram `(drawing, frame, falloff)`. Falloff (curva única, ativo em 0.5) SÓ
multiplica influência de brush. Inserir keyframe ao desenhar LIMPA a seleção de frames de
todas as camadas (proteção contra sculpt multiframe acidental).

### Cíclicas (seção unificada — era lacuna)

- **Nascimento**: draw grava `cyclic = use_fill` (material com fill fecha sozinho); fill-tool
  gera tudo cíclico; rasterização trata `cyclic || fill_id` como fechada.
- **Toggle** (`cyclical_set`): CLOSE/OPEN/TOGGLE + **"Match Point Density"** (default true:
  subdivide o segmento novo por densidade média — senão aparece uma reta comprida).
- **Corte** (erase): interseções incluem o segmento de fechamento; a curva abre NO PIVÔ da
  última interseção com ROTAÇÃO do array (não no índice 0!); caps FLAT.
- **Render**: `is_cyclic = cyclic && points > 2`; vértice extra = re-emissão de p0 com
  `u_stroke` do comprimento FECHADO (uv contínuo no seam); flag no SINAL do point_id.
- **Set Start Point**: rotaciona o array pro selecionado virar índice 0 (só cíclicas).
- Subdivide/extrude/stroke-eraser respeitam o fecho (extrude: curvas novas nunca nascem cíclicas).

### UV / arc-length (o 3º texel — reservar as colunas desde já)

`u_stroke`: LINE = arclen em unidades de objeto × u_scale + u_translation; DOTS = índice /
densidade / soma-de-raios (RADIUS: progressão geométrica com forma fechada log). Fill-UV =
projeção planar por curva (§9). `fill_opacity` empacotada em dígitos decimais do alpha
(`int(a·10000)·10 + op`) — truque de 1 float, evitar no PH2D (campo próprio).

---

## §12 — Tradução GL→wgpu (as fronteiras que o port cruza)

| No Blender (GL) | No PH2D (wgpu/WGSL) |
|---|---|
| Buffer textures + `texelFetch` (limite de 14 attrs do macOS GL) | storage buffers + `@builtin(vertex_index)` — JÁ FEITO no `flip.wgsl`; sem o aperto de attrs (campos diretos em vez do `packed` int32) |
| Dual-source blending (resolve do SMAA/composição na cena) | **não precisamos**: o Flip resolve premult→compositor do Painter (decisão W1, byte-idêntica ao Painter). Se um dia: `Features::DUAL_SOURCE_BLENDING` é OPCIONAL em wgpu — reformular como pass extra |
| `gl_FragDepth` | `@builtin(frag_depth)` — mesmo custo (mata early-Z; aceito) |
| `DRW_STATE_LOGIC_INVERT` (máscara invertida) | não existe: desenhar branco com `BlendFactor::OneMinusDst/Zero` (out = 1−dst) OU amostrar `1−mask` com uniform de inversão (melhor: elimina os passes intercalados) |
| Reveal `RGB10_A2`/RGBA16F (tabela §2) | alpha escalar no premult RGBA16F (nosso caminho) — a tabela só importa se um dia houver blend dual-buffer |
| `NOPERSPECTIVE` varyings | 2D-orto: linear == perspective; só `flat` precisa ser preservado (sentinelas dependem de igualdade EXATA) |
| MSAA/SMAA | 03 §7 (analítico + acúmulo; SMAA opcional do reference MIT) |
| depth clear 0.0 + GREATER (2D) | idêntico no nosso `pipeline.rs` ✓ |

Mapa completo de libs GP → crates Rust: [`04 §6`](04_alem_do_blender.md).
