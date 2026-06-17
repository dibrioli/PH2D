═══════════════════════════════════════════════════════════════════
HANDOFF → Coordenador · GPU pass-graph PRECISA premultiplicar (2 artefatos do Enio)
Autor: Implementador Painter (jornada 2026-06-05) · segue
       `HANDOFF_painter_w4_spatial_wire_impl.md` §3.4 (premul deferred → agora URGENTE)
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
O Enio testou os espaciais em layers com **fundo transparente** e achou 2 artefatos —
**mesma causa-raiz**: o pass-graph roda os kernels em **straight (não-premultiplicado)
RGBA**, então texels transparentes (que carregam RGB lixo) vazam pro resultado.
1. **GaussianBlur fica preso à silhueta opaca** — o miolo borra mas a borda do alpha
   continua dura (sem feather suave pra transparência).
2. **ChromaticAberration solta pontos azuis** — o gather do canal blue puxa cor de
   texels transparentes.
**Fix (TEU, GPU — tu pré-reivindicaste em §3.4):** premultiplicar o below-composite
materializado UMA vez antes do pass-graph; rodar todo kernel premultiplicado (borra
alpha junto); des-premultiplicar; e o combine do `SpatialAdjustment` usar o alpha
**feathered** do kernel, não preservar o base. **Já fiz a referência CPU + fallback**
(commit `458b6d5`) — espelha bit-a-bit. Este é o **caminho visível** (o smoke do Enio
roda GPU); meu CPU sozinho não muda a tela dele.

## §1 — A CAUSA-RAIZ (uma só, 2 sintomas)
Operação espacial em **straight** RGBA lê vizinhos transparentes. Um PNG transparente
guarda RGB arbitrário sob `a=0` (matte/lixo). Borrar/gather straight mistura esse RGB:
- blur: a cor vaza nas bordas E o alpha não-borrado mantém a silhueta dura → sem feather.
- chroma: o shift do blue amostra blue-lixo de texels transparentes → speckle azul.
**Premultiplicado**, todo texel transparente é `(0,0,0,0)` → contribui zero. Aí cor E
cobertura (alpha) borram juntas → feather suave + zero speckle.

## §2 — O QUE MUDAR EM `ph2d-render` (pass-graph)
1. **Materialize premultiplicado:** ao materializar o composite-de-baixo no `Rgba32Float`
   linear pro segmento espacial, **premultiplica** (`rgb *= a`). (Hoje materializa straight.)
2. **Kernels rodam premultiplicados** (já são genéricos 4-canais): o separável H/V borra
   **os 4 canais** (incl. alpha); motion idem; chroma faz gather premult dos R/G/B nos
   shifts + **alpha no gather UNSHIFTED** (a lente desloca cor, não cobertura).
3. **Combine usa o alpha feathered:** o `cs_combine` do `SpatialAdjustment` deve emitir
   a cobertura borrada, **não** preservar `acc.a`. Semântica (espelho do meu CPU):
   - des-premultiplica o resultado borrado → straight RGBA com alpha feathered.
   - `result = (blend_mode==Normal) ? blurred : blend(base, blurred)`.
   - `out = lerp(base, result, t)` nos **4 canais** (`t = opacity·mask`).
   - kernel neutro (radius 0) ⇒ `blurred==base` ⇒ identidade exata.
4. **Intermediário com alpha:** se hoje o `Rgba32Float` ping-pong não carrega alpha,
   passa a carregar (4 canais). (Casava com a tua nota de trocar pra `Rgba16Float` —
   continua ok, só mantém os 4 canais.)

## §3 — PARIDADE (não quebra teus gates se premultiplicares OS DOIS lados)
Teus `gpu_{gaussian,sharpen,motion,chroma}_matches_cpu_reference` comparam GPU vs a TUA
referência CPU em `ph2d-render`. Premultiplica **ambos** (GPU + tua ref) e seguem verdes.
**Melhor ainda:** `ph2d-render` é dev-dep de `ph2d-painter-brush`, então a tua ref CPU
pode CHAMAR a minha canônica direto (sem duplicar a math premul):
`ph2d_painter_brush::adjustments::{apply_gaussian, apply_motion_blur, apply_sharpen,
apply_chromatic_aberration}` — todas agora premultiplicadas, com testes
(`gaussian_feathers_coverage_into_transparency`, `premultiplied_blur_ignores_transparent_texel_colour`,
`chroma_does_not_speckle_transparent_regions`). Os pesos seguem pinados (`21ae78e`).

## §4 — REFERÊNCIA (meu commit `458b6d5`, lê e espelha)
- `crates/ph2d-painter-brush/src/adjustments/spatial.rs`: `premultiply`/`unpremultiply`
  + `separable_blur_premul` (4-ch) + os 4 `apply_*` premultiplicados.
- `crates/ph2d-tool-painter/src/compositor/compose.rs`: o ramo `is_spatial` do combine
  (lerp 4-ch com alpha feathered) vs o per-pixel (preserva base alpha).
Semântica idêntica nos dois → quando ligares o GPU, CPU-fallback e GPU concordam.

## §5 — POSSE
Mexi só nas MINHAS crates (`ph2d-painter-brush`, `ph2d-tool-painter`). `ph2d-render`
(materialize/combine/pass-graph) é TEU — este handoff é o pedido pra ligar o premul lá.
Sem push (tu shipas). Me pinga quando o GPU premul landar que eu confirmo o smoke com o Enio.

**Smoke do Enio (pós teu fix):** GaussianBlur numa layer transparente → borda macia
feathered (não recortada); ChromaticAberration → fringe limpo, zero pontos azuis.
═══════════════════════════════════════════════════════════════════
