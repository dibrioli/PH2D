# Flip §04 — Além do Blender: estado da arte, paisagem de apps e lições do GPv3

> O Blender é a **referência de comportamento** (clean-room) — mas não é o teto. Este doc
> reúne o que a literatura, os apps maduros e a própria história do GPv3 ensinam, com as
> recomendações já **decididas** para o Flip (marcadas ►). Fontes primárias linkadas em cada
> item — nenhuma afirmação sem fonte (memória `feedback_no_industrial_claims_without_verification`).

Sumário: §1 Renderização de traço (SOTA) · §2 Inbetweening (SOTA + algoritmo recomendado) ·
§3 Fill/colorização (SOTA) · §4 A paisagem dos apps de animação 2D · §5 Lições do redesign
GPv3 · §6 Tabela de mapeamento GP → Rust/PH2D.

---

## §1 — Renderização de traço: o que existe além da fita do GP

O problema do Flip ("join/overlap artifact" de polilinhas semi-transparentes) tem nome na
literatura — e o levantamento confirmou que **a família do nosso fix é a certa** (detalhe e
spec no `03_traco_rasterizacao.md` §4). O mapa:

| Técnica | O que resolve | Por que (não) serve ao Flip |
|---|---|---|
| [Rougier, JCGT 2013](https://jcgt.org/published/0002/02/08/paper.pdf) — polylines com caps/joins analíticos no fragment | AA e joins de linha 2D | O próprio autor ADMITE não ter resolvido self-intersection com transparência — e especula exatamente o nosso fix (a): *"could be fixed... by considering which segment is responsible for actually painting the join"* |
| [Ciallo, SIGGRAPH 2024](https://research.adobe.com/publication/ciallo-gpu-accelerated-rendering-of-vector-brush-strokes/) (+ [tutorial](https://shenciao.github.io/brush-rendering-tutorial/)) — traços de pincel vetoriais em GPU, destinado ao GP | fita instanciada · stamps NO fragment · airbrush analítico | O trabalho MAIS próximo do Flip. A correção de junta dele (`A' = 1−sqrt(1−A)`) só vale alpha constante (hardness=1). O modo **stamp** e o **airbrush integral** (`A(y)=1−exp(−2αc·√(R²−y²))`) são os nossos pincéis futuros. ⚠️ código GPL-3 — só comportamento |
| [Vello / "GPU-friendly Stroke Expansion"](https://linebender.org/gpu-stroke-expansion-paper/) (Levien & Uguray, HPG 2024) — stroke→fill por espirais de Euler em compute | união exata por winding, watertight, sub-ms | **Só alpha/width uniformes — sem pincel macio.** Referência para um futuro pincel "ink" 100% duro (o Vello já está no binário do PH2D) |
| [Polar Stroking](https://arxiv.org/abs/2007.00308) (Kilgard, SIGGRAPH 2020) — tesselação por passo de ângulo de tangente | erro limitado sem recursão; joins/caps unificados | stroke sólido; não trata transparência nem falloff |
| [Stencil-then-Cover / NV_path_rendering](https://developer.nvidia.com/gpu-accelerated-path-rendering) (Kilgard & Bolz 2012) | self-overlap transparente com alpha constante: stencil marca cobertura, cover pinta 1× | É o **avô binário do nosso caminho (b)** (scratch MAX = StC contínuo p/ cobertura fracionária) |
| Patentes Microsoft Ink ([US10235778B2](https://patents.google.com/patent/US10235778B2/en), [US20110304643A1](https://patents.google.com/patent/US20110304643A1/en)) + [Krita Wash mode](https://docs.krita.org/en/reference_manual/brushes/brush_settings/opacity_and_flow.html) | overlap intra-traço via buffer intermediário + blend MIN/MAX/coverage | Pedigree industrial do caminho (b): "dentro de UM traço, repassar não acumula além do teto" |
| [pygfx line rendering](https://almarklein.org/line_rendering.html) (wgpu) | joins sem sobreposição POR CONSTRUÇÃO (vértices recuados) | Viável em wgpu para largura CONSTANTE; com raio por pressão o recuo interno "engole" segmentos curtos (a mesma doença do miter_break) |
| Skia Graphite | depth por draw p/ ordem de pintura | valida o first-wins ENTRE draws; o nosso problema é intra-traço com gradiente — depth não expressa |

► **Decisões (ratificadas pela análise adversarial, ver 03 §4):** fix (a) janela p0/p3 agora;
(b) scratch MAX como escalada com gatilho K4; airbrush/stamps Ciallo = pincéis futuros;
Vello stroke-expansion = pincel ink futuro; `1−sqrt(1−A)` = registrada e rejeitada.

## §2 — Inbetweening: do "por índice" do GP ao padrão-ouro clássico

**A verdade sobre o GP:** o pareamento é **puramente ordinal** (curva i ↔ curva i, ou i-ésima
selecionada) — zero correspondência espacial (`interpolate.cc:244-315`). O que salva o usuário
é o auto-flip + desenhar na mesma ordem. O W3 porta o GP literal primeiro (previsível, é o que
os animadores de GP conhecem — ver `02_referencia` §3); **este parágrafo é o upgrade planejado
por cima da MESMA estrutura de dados** (mapa de amostragem (índice, fator) desacoplado do lerp).

O que a literatura ensina:

1. **[BetweenIT](https://studios.disneyresearch.com/2010/05/03/betweenit-an-interactive-tool-for-tight-inbetweening/)
   (Whited et al., Disney, EG 2010; matemática completa na [patente EP2315179A2](https://patents.google.com/patent/EP2315179A2/en)):**
   o padrão-ouro clássico. Correspondência por **stroke graphs** (DFS simultâneo respeitando a
   ordem circular nas junções; custo `E = (E_A + E_L/2)/L̄` contra limiar). Interpolação por
   **espiral logarítmica**: interpola-se a TRANSFORMAÇÃO de similaridade, não a posição —
   `θ` = ângulo entre as cordas, `σ` = razão de escala, ponto fixo `F` resolve
   `FJ₁ = R(θ)·S(σ)·FJ₀` (2×2 linear), e `P(t) = F + R(θt)·σᵗ·(P₀−F)`. **É a resposta ao
   encolhimento do lerp** (um braço girando 180° colapsa numa linha com lerp; a espiral preserva
   o arco). Junções soldadas: promediar (θ,σ) das strokes que compartilham endpoint.
2. **[CACANi](https://cacani.sg/cacani-features/)** (comercial, NTU): a lição de PRODUTO —
   *nenhum matcher automático é confiável; a UI de correção é parte do algoritmo* (Re-match
   Stroke Order, Join Strokes, feature points automáticos em pontas/cantos, painel de timing).
   O proxy acadêmico é o [FTP-SC (CGF 2018)](https://dcgi.fel.cvut.cz/home/sykorad/FTP-SC.html):
   âncoras de alta confiança por "fuzzy topology" (vizinhança, não grafo exato) + guloso.
3. **Deep learning ([AnimeInbet ICCV'23](https://arxiv.org/abs/2309.16643),
   [JoSTC TOG'24](https://markmohr.github.io/JoSTC/), [ToonCrafter SA'24](https://arxiv.org/abs/2405.17933)):**
   até os neurais convergem para "vetorizar → casar vértices → reposicionar" — o PH2D JÁ tem os
   vetores; a heurística clássica captura a maior parte do valor sem inferência/GPU/não-determinismo.
   ► **Não perseguir.** Oclusão automática ([Even et al. 2025](https://inria.hal.science/hal-04797216v1))
   idem — o fallback honesto é fade + breakdown manual (a resposta da própria Disney).

► **O algoritmo recomendado para o "Tween v2"** (Rust puro, zero deps novas; W3 entrega o
GP-literal, isto é o upgrade qualificado):

```text
tween(frame_a, frame_b, t):
  1. FEATURES por stroke (cacheável): centróide, arclen L, ângulo do eixo (PCA 2×2),
     aberta/fechada, ordem de desenho, largura média
  2. CORRESPONDÊNCIA (por camada; D = diagonal do bbox da união):
     custo(i,j) = ∞ se aberta/fechada incompatíveis
                | 0.40·|Δcentróide|/D + 0.25·|ΔL|/max(L) + 0.20·Δângulo/(π/2) + 0.15·Δordem/max
     hungarian(custo) (n≤10³ → O(n³) trivial; ou guloso melhor-mútuo)
     par com custo > T≈0.35 → sem par
  3. REPARAM: flip se cruza-cordas (o teste do GP, mantém); detectar cantos (pico de ângulo
     > ~35°) e alinhar trecho-a-trecho; reamostrar por fração de arc-length p/ max(|A|,|B|)
  4. INTERPOLAÇÃO INTRÍNSECA: espiral log por stroke (1 sincos + 1 powf POR STROKE, nunca
     por vértice — disciplina HR-5) + resíduo não-rígido em lerp: P(t) = espiral(t) + ease(t)·resid
  5. ÓRFÃS: só em A → opacity·(1−t), advectada pela espiral da stroke casada mais próxima;
     só em B → nasce com opacity·t
```

Mais duas correções baratas ao GP, já para o W3: **fator por camada** (o GP aplica o fator da
camada ativa a todas — `interpolate.cc:380-385`; computar por camada é trivial e estritamente
melhor) e o **scrub por posição absoluta do mouse** (borda esquerda = 0%, direita = 100% —
UX do GP que vale copiar, `interpolate.cc:974-976`). E a lição CACANi: **overlay de pares**
(linhas coloridas ligando strokes casadas) + re-par manual clique-clique como restrição dura
— construir JUNTO com o algoritmo, não depois.

## §3 — Fill e colorização: o mapa completo

O balde do W4 porta o pixel-solver do GP (`02_referencia` §5) — robusto, validado, o resultado
é GEOMETRIA (anima). O que existe além:

- **[LazyBrush](https://dcgi.fel.cvut.cz/home/sykorad/Sykora09-EG.pdf) (Sýkora et al., EG 2009):**
  colorização por scribbles via multiway cut; a fronteira é ATRAÍDA pro pixel mais escuro
  (dentro do traço) → cor "entra por baixo da linha" com AA de graça; gaps nem precisam fechar.
  **Em produção no TVPaint 11 Pro como [CTG Layer](https://doc.tvpaint.com/docs/colorize-texturize/colorize-with-ctg-layers/quick-overview)**;
  o [Colorize Mask do Krita](https://docs.krita.org/en/reference_manual/tools/colorize_mask.html)
  é a mesma linhagem (e admite na doc: "o algoritmo é lento" — solver denso, botão Update).
  Constantes do paper: `K = 2(w+h)`, scribble soft λ=0.95, pré-filtro LoG p/ lápis; guloso
  um-contra-todos ≈ 9-18× mais rápido que α-expansion com ΔE ≤ 0.04%. **O "onion fill" do
  paper — um scribble atravessando várias poses empilhadas pinta o range de frames inteiro — é
  a feature de flipbook mais valiosa da literatura (só o TVPaint entrega hoje).** ► Wave futura
  ("Colorize"), não o W4.
- **[Trapped-ball](https://cg.cs.tsinghua.edu.cn/papers/TVCG_2009_cartoon.pdf) (Zhang et al., TVCG 2009):**
  segmentação de line-art com gaps por morfologia (flood → erode raio R → dilate; best-first
  com raios decrescentes, R₀ = 8px). ► Candidato ao "colorir tudo" em lote (pré-segmentar o
  frame), antes do LazyBrush.
- **Gap closing — a paisagem:** o Extend/Radius do GP tem paralelo direto no
  [Close Gap do Toon Boom Harmony](https://docs.toonboom.com/help/harmony-22/premium/colour/close-gaps.html)
  — com um twist superior: o Harmony **materializa o fechamento como stroke invisível
  persistente** → o re-fill de frames vizinhos e o refill pós-edição param de depender do
  estado da tool. ► **Adotar o twist do Harmony no W4.** O CSP tem
  [Close Gap raster 1-20px + Area Scaling](http://www.clip-studio.com/site/gd_en/csp/toolguide/csp_toolguide/100_reference/Fill.htm)
  (expandir o resultado ±N px — o "cresce por baixo da linha" barato → ► adotar como
  **Grow/Shrink pós-vetorização** via o offset CAD que o Painter já tem). O
  [ColorDrop do Procreate](https://help.procreate.com/articles/zmlayd-fill-an-area-using-colordrop)
  (drag horizontal = threshold) só faz sentido com referência raster de alpha suave — não é o nosso caso.
- **Semântica de balde de ANIMAÇÃO:** modos **Paint / Paint-Unpainted (paint-behind) / Unpaint**
  ([Toon Boom](https://learn.toonboom.com/modules/drawing/topic/painting-drawings)) — ► adotar
  os 3 modos no W4 (o paint-behind é o fluxo de colorir sem tocar a linha).
- **Flood fill em si:** span/scanline ≥ 10× o BFS por pixel ([Lode](https://lodev.org/cgtutor/floodfill.html),
  [Milazzo](https://www.adammil.net/blog/v126_A_More_Efficient_Flood_Fill.html)); poucos ms em
  ≤4 Mpix na CPU. ► **NÃO fazer fill em GPU**: JFA é o primitivo ERRADO (salta paredes — não é
  geodésico; serve p/ distance-field das PONTAS, útil no modo Radius); CCL GPU só paga em
  segmentação em lote. O fill é operação de clique, não de frame.
- **Vetorização do contorno:** Moore/marching-squares → RDP ε≈1.25px → **fit Schneider (o
  PH2D já tem!)** em contornos > ~64px — supera o pós-processo bruto do GP (smooth 20× +
  decimação 2^n), que produz polylines densas e "moles". Alerta: fill analítico direto no
  vetor esbarra em [patentes](https://patents.google.com/patent/US9256972) e é frágil com
  pontas abertas (o caso NORMAL) — mais um motivo pro raster-then-vectorize.

## §4 — A paisagem dos apps (o que os animadores esperam)

Levantamento primário: Krita, TVPaint, OpenToonz, Toon Boom Harmony, CSP EX, RoughAnimator,
Procreate Dreams, Adobe Animate. O que importa pro Flip:

**Ghost Frames (onion):**
- Convenção de cor NÃO é universal — Krita/Toon Boom: vermelho-antes/verde-depois; TVPaint:
  verde/laranja; GP: verde/azul-roxo. Universal é ser CONFIGURÁVEL. ► Default do Flip = o par
  do GP (é a referência), como **tokens** `FlipGhostBefore`/`FlipGhostAfter` (HR-15) + settings.
- **Onion POR-DESENHO, não por-frame** ([Harmony](https://docs.toonboom.com/help/harmony-22/premium/reference/view/onion-skin-view.html)
  recomenda por-desenho pra frame-by-frame — pula holds). É o modo RELATIVE do GP. ► Default
  Relative; Absolute como opção.
- Fade por slot com sliders LINKADOS (TVPaint/Harmony) e **modo silhueta** (tint 100% — que é
  literalmente o que o GP faz, `tint.a = 1.0`). O [Krita](https://docs.krita.org/en/reference_manual/dockers/onion_skin.html)
  tem slider de opacidade POR offset.
- **Light table** (TVPaint/OpenToonz): além dos vizinhos relativos, frames FIXOS arbitrários
  como referência persistente (bookmarks). ► Fase 2: `Vec<FrameRef>` fixos alimentando o mesmo
  passe de ghost.
- **Shift & Trace** (OpenToonz): transform 2D POR GHOST (exibição-only) + F1/F2/F3 alternando
  qual ghost aparece — o "flip de papel" digital pra checar arcos. ► Fase 2, alto valor,
  escopo pequeno.

**Tira de frames / navegação:**
- Dois paradigmas: timeline horizontal (TVPaint/Krita/CSP/Dreams) e X-sheet vertical (Toonz,
  pipeline japonês). ► Horizontal primeiro; como o modelo é células-referência, o X-sheet vira
  só outra view depois.
- **O modelo TVPaint Instância + Células de Exposição** = exatamente o nosso frames-map com
  hold (desenho ≠ frame; estender duração = repetir referência). Exibir o nº de exposições na
  célula (TVPaint) na tira.
- **Flip é o inner loop do animador** — atalhos dedicados de desenho-anterior/próximo
  ([F/G no Harmony](https://docs.toonboom.com/help/harmony-22/premium/getting-started/animation.html)
  — navega por DESENHO, pula holds; RoughAnimator mapeia até botões de volume do tablet).
  ► Atalhos de flip com latência zero: as cels vizinhas SEMPRE residentes como textura GPU.
- **Ciclos = pre/post behavior por camada** ([TVPaint](https://doc.tvpaint.com/docs/animation-additional-functions/timeline-options/pre-post-behavior):
  None/Loop/PingPong/Hold) — ciclo é função de AMOSTRAGEM no playback, zero duplicação.
  ► Adotar no W3 (o sampler do playhead ganha o wrap-mode).
- Krita: Alt+drag = ripple (empurra o frame e todos os seguintes); duplicar vs frame em branco;
  Insert Hold em massa.

**Playback e cache:**
- [Krita](https://docs.krita.org/en/reference_manual/preferences/performance_settings.html):
  cache de frames COMPOSTOS com 2 válvulas — **cap de resolução** (~2500px, cacheia downscale)
  e **Region of Interest**; preenchimento em background; e o relógio **DROPA frames em vez de
  atrasar**. Números: FullHD@25fps = 200 MiB/s descomprimido.
- TVPaint: raster descomprimido em RAM (8K ≈ 95 MB/frame) — o extremo oposto.
- OpenToonz: cache por NÓ de compositing + framebar pintando o estado do cache (cinza =
  renderizado, vermelho = não).
- ► Flip: ring de texturas GPU (frames compostos) keyed por (frame, escala), invalidação por
  (camada, desenho) sujo, drop de frame no relógio. O compositor 22-modos já é GPU — cachear o
  composto pós-blend por frame é o ponto certo.

**Workflow linha/cor (produção):**
- TVPaint CTG (LazyBrush embutido, §3), OpenToonz ink&paint (linha e paint indexados na
  palette; "auto-paint" pinta a linha junto), CSP "Fill up to vector paths" (preenche até o
  CENTRO da polyline — o mesmo insight do `radius_scale = 0.5` do GP!).
- ► O contrato do fill do W4 já nasce com "fill lê camada de referência" (linha) — é o mínimo
  que produção exige; o CTG completo é wave futura.

**Traço raster vs vetor:** TVPaint/CSP/Krita/Dreams = raster; Toonz vector levels (centerline
+ palette indexada re-colorável) e Animate (vetor puro, miter/round/bevel + width tool) =
nichos. ► O Flip fica no traço vetorial-rasterizado-analítico que já tem (edição + re-render
por frame + resolução-independente); vetor exato é o módulo Vector.

## §5 — Lições do redesign GPv3 (2022→2026, fontes primárias)

A história ([proposta](https://devtalk.blender.org/t/developer-discussion-new-grease-pencil-data-structure-proposal/24368) ·
[anúncio](https://code.blender.org/2023/05/the-next-big-step-grease-pencil-3-0/) ·
[design final](https://hackmd.io/@filedescriptors/HJlP52oCj) ·
[perf tests #105540](https://projects.blender.org/blender/blender/issues/105540) ·
[release 4.3](https://developer.blender.org/docs/release_notes/4.3/grease_pencil/)):

1. **O modelo "pool de drawings + frame-map com ranges implícitos" é a decisão mais validada**
   (2022→2026 sem revisão) — o `ph2d-flip` já nasceu nela. ✓
2. **Implicit sharing (CoW) foi a alavanca decisiva** — números medidos: arquivo −65%, abrir
   3.6×, salvar 4.25×, **undo 6.6×** (831→126ms), keyframe novo 5.3×. E SEM ele o GPv3 chegou a
   ser mais LENTO que o modelo velho. ► **`Arc` + copy-on-write nos buffers dos drawings do
   Flip ANTES de qualquer feature de duplicação/hold em escala** — o undo global do PH2D é
   snapshot-based, exatamente o caso em que CoW brilha. Cuidado: os Arcs não podem quebrar o
   `canonicalize()` do undo (comparação por conteúdo).
3. **Tudo-é-atributo com default implícito** (coluna ausente = default; não materializa
   `opacity=1.0` p/ 5M pontos) — a mesma disciplina postcard append-only do PH2D. ► Colunas
   laterais opcionais (`Option<Vec<f32>>`) para atributos novos (rotation, u_scale...).
4. **A maior dor confessada (meeting 2026-07-06): compositing preso DENTRO do engine.** O Flip
   já está do lado certo (produtor de camadas pro compositor 22-modos compartilhado). ► Nunca
   compor dentro do passe do traço.
5. **O cronograma estourou ~3×** (anunciado nov/2023, shippou nov/2024 + um ciclo de
   estabilização com 91 fixes) — e o tempo sumiu no porte de OPERATORS/ferramentas, não no
   modelo de dados. ► Milestones pequenos com gate executável; esperar que tooling custe mais
   que dados.
6. **Instancing sem UI é anti-padrão**: o modelo suporta desde 4.3, a UI não existe até hoje
   (add-ons usam por Python "por conta e risco"). ► Se instância de drawing entrar, entra com
   o gesto de duplicar-instância + marcador visual na tira JUNTOS.
7. Mudanças de tool do 4.3 que validam escolhas nossas: desenhar DIRETO no drawing (o buffer
   temporário causava "salto"), eraser por CORTE analítico, active smoothing por curve-fitting
   ("menos flutuação"), simplify em PIXELS de tela, spacing em % do raio.

## §6 — Tabela de mapeamento: dependência GP → equivalente Rust/PH2D

| No Blender | Licença/nota | No PH2D |
|---|---|---|
| `curve_fit_nd` (Schneider fit do active smoothing/convert) | lib C externa | **JÁ TEMOS 2**: `ph2d-anim/src/curve_fit.rs` + `curve_refit.rs` (Painter, com corner-split). Falta só o `orig_index_map` (atributos por CÓPIA do ponto original mais próximo, não interpolados) |
| `delaunay_2d_calc` (CDT do fill multi-stroke) | BLI, GPL | crate `spade` (MIT/Apache; confirmar constrained-DT com faces) ou earcut p/ 1-curva |
| `polyfill_2d` (ear-clip 1 curva) | BLI, GPL | o CDT/earcut do `ph2d-vec-boolean`/kurbo que o Vector já usa |
| Perlin do jitter (randomize ao longo do arco) | BLI noise | `jitter.rs` do brush do Painter (splitmix64) — mesma filosofia hash-determinística, replay-safe |
| SMAA (`BLI_smaa_textures.h`) | LUTs geradas no repo GPL | se um dia: [reference implementation Jimenez](https://github.com/iryoku/smaa) (MIT, LUTs incluídas) — nunca os bytes do Blender |
| `BLI_lasso`/gestos | GPL | infra de marquee/lasso própria do shell (input_dispatch) |
| CurveMapping (curvas de pressão/falloff editáveis) | GPL | a falloff curve do Painter (`HANDOFF_painter_falloff_curve`) |
| `BKE_brush_curve_strength` (falloff do sculpt) | GPL | mesma falloff curve; presets Smooth/Linear/Sharp como tabelas |
| Hungarian (matching do Tween v2) | — | ~100 LOC próprias ou guloso melhor-mútuo |
| SMAA/SSAA de export | — | acúmulo Halton+gaussiana (03 §7.3) — só matemática publicada |
