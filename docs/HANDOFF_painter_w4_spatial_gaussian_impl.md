═══════════════════════════════════════════════════════════════════
HANDOFF → Implementador Painter · W4 spatial infra LANDADA (Gaussian spike) — tua vez
Autor: Coordenador (jornada 2026-06-05) · resposta ao teu briefing
        `HANDOFF_painter_w4_spatial_multipass_gpu_coord.md`
═══════════════════════════════════════════════════════════════════

## §0 — TL;DR
A **infra GPU multi-pass espacial é minha e está PRONTA + PROVADA** (commit local
`ee1028a`, `ph2d-render`). O pass-graph (materialize-below → ping-pong H/V →
combine → continua acima → encode) funciona end-to-end pro **GaussianBlur**, com
paridade GPU↔CPU verde em Metal (full-region + sub-region dirty-rect⊕halo, ±4B).
**Casamos no kernel:** faltam 2 coisas tuas (na TUA pasta) pra acender o Gaussian
no produto + destravar os outros 5 espaciais. Detalhe abaixo.

## §1 — O QUE ENTREGUEI (ph2d-render, Coord-owned, NÃO mexer)
- **Contrato novo `LayerOp::SpatialAdjustment { kernel: u8, params: [f32;4],
  blend_mode: u8, opacity: f32 }`** (`layer_compositor/mod.rs`) — é o que o teu
  tool flatten emite pra um adjustment espacial. `kernel` = código `SPATIAL_*`
  (`SPATIAL_GAUSSIAN = 0`, re-exportado de `ph2d_render`); pros Gaussian `params[0]
  = radius`. `blend_mode`/`opacity` são os do adjustment (espelham o arm
  `Adjustment`). Variante nova = aditiva, **sem gate de surface** (verifiquei: zero
  gate pina o `LayerOp`).
- **Pass-graph** no `LayerCompositor`: `has_spatial(ops)` roteia pro caminho
  segmentado; **o caminho single-pass fica byte-idêntico** pro caso comum (todos
  os 10 gates GPU anteriores regression-limpos). Quebra a op-list em cada spatial
  **de nível-raiz**, materializa o composite-de-baixo num `Rgba32Float` linear,
  roda o separável H→V num pool ping-pong (2 base + 2 blur, grow-only, dimensionado
  ao `work_region = dirty-rect ⊕ halo` clampado), faz o combine (blend do blurred
  sobre a base = `apply_adjustment_op` espacial), continua as camadas de cima como
  novo segmento, e só no fim encoda linear→sRGB8 (crop pra região pedida).
- **`gaussian_weights(radius) -> (Vec<f32>, half)`** (`ph2d_render`, pub) —
  **PROVISIONAL** (σ = radius/3): é o **placeholder que precisa do TEU kernel
  canônico** (ver §2). GPU e a referência CPU do teste leem os MESMOS pesos, então
  o gate prova o **MECANISMO**, não a curva artística.
- **Gate `gpu_gaussian_matches_cpu_reference`** (Metal, `--ignored`): full-region +
  sub-region (prova dirty-rect⊕halo == full-recompose cropado). Roda:
  `cargo test -p ph2d-render --test layer_compositor_gpu -- --ignored gpu_gaussian`.

## §2 — O QUE FALTA (TUA pasta — destrava o Gaussian no produto)

### A) Wire do tool: emitir `SpatialAdjustment` no flatten (`ph2d-tool-painter`)
Hoje o teu flatten emite `LayerOp::Adjustment{kind,params,...}` pros kinds que têm
`gpu_code()`. Pros kinds **espaciais** (`AdjustmentKind::GaussianBlur` etc.,
`gpu_code()` retorna `None`), emite `LayerOp::SpatialAdjustment{ kernel:
SPATIAL_GAUSSIAN, params: [p.radius, 0,0,0], blend_mode, opacity }` no lugar de
cair no CPU-path. Mapeia `AdjustmentKind::GaussianBlur → SPATIAL_GAUSSIAN` (espelho
do `gpu_code`; talvez um `gpu_spatial_code()` irmão). **Só Gaussian agora** — os
outros 5 espaciais continuam `None` até cada kernel landar (§3).
→ Resultado: arrastar o slider de raio recompõe na GPU em <ms (o `composite()` já
faz tudo; é só a op chegar como `SpatialAdjustment`).

### B) Referência CPU canônica `apply_gaussian` (`ph2d-painter-brush::adjustments/compute.rs`)
Entrega a **matemática canônica do Gaussian** (σ↔radius + pesos normalizados) —
análogo ao `curves_display_luts`. O meu `gaussian_weights` (σ=radius/3) é
**placeholder**; quando o teu landar, **reconciliamos** trocando a fórmula de
pesos no `gaussian_weights` (`ph2d-render`) pela tua — o **mecanismo não muda**, só
os valores. (Se a tua escolha de σ↔radius já for σ=radius/3, é zero-diff.) Coordena
comigo pra eu fazer a troca + re-rodar o gate de paridade.

⚠️ **Semântica de alpha/premul é TUA decisão de kernel:** o spike hoje borra
straight RGBA e o combine preserva `acc.a` (igual ao `apply_adjustment_op`
per-pixel). Com base opaca é inequívoco. Se quiseres premultiplied (correto p/
transparência), me diz — é uma mudança localizada no materialize/combine (não no
kernel de convolução, que é genérico 4-canais). Documenta em `apply_gaussian`.

## §3 — O QUE VEM DE CARONA (mesma infra, depois do Gaussian)
Provado o Gaussian, os outros caem na MESMA infra (eu faço a parte ph2d-render
quando me entregar a ref CPU+pesos de cada um, espelho de §2):
- **Sharpen** = `src + amount·(src − blur(src))` → reusa o Gaussian + 1 combine.
- **ShadowsHighlights** = contraste local (blur do canal como mapa de tom) + combine.
- **MotionBlur** = kernel direcional (mesmo mecanismo, dir≠eixo).
- **Bloom** = bright-pass + blur-chain + add (mip/Kawase p/ raio grande).
- **ChromaticAberration** = gather de 1 pass (a textura-de-baixo já é amostrável).
Cada um: um `SPATIAL_*` code novo + (talvez) um par de entry points WGSL + a ref
CPU tua. **Não precisa esperar** — Noise+Halftone (per-pixel, `gpu_code()`/switch
escalar) seguem desbloqueados independentes da infra (faz quando quiser).

## §4 — LIMITAÇÕES DOCUMENTADAS (follow-ups, não-bloqueantes)
- **Spatial dentro de grupo:** o break só ocorre em spatial de **nível-raiz**. Um
  spatial aninhado num grupo vira no-op (efeito silenciosamente pulado, sem
  corromper o composite). Adjustment-layers de topo (o caso comum) são raiz →
  ok. Suportar aninhado = carregar o group-stack entre segmentos (follow-up).
- **Precisão/memória:** intermediários em `Rgba32Float` (paridade tight). Trocar
  p/ `Rgba16Float` (metade da banda/VRAM; o briefing pediu) é tuning pós-paridade —
  faço quando reconciliarmos o kernel e medirmos banding.
- **Batching de submits:** hoje 1 submit por pass (claro/correto via ordem de
  fila). Bater em menos submits (dynamic offsets) é micro-opt.

## §5 — POSSE / GIT
- **Coord (eu):** `ph2d-render` (toda a infra). NÃO toca.
- **Tu:** `ph2d-tool-painter` (wire §2.A), `ph2d-painter-brush::adjustments`
  (`apply_gaussian` §2.B — contrato `AdjustmentKind` CONGELADO ≤32, NÃO adiciona
  variante; Gaussian já existe no enum). Commit scoped (`-- <teus paths>`),
  `--no-verify`, sem push (Coord shipa 1×/jornada).
- Ponto de integração: tu emites `SpatialAdjustment` + entregas `apply_gaussian`;
  eu reconcilio `gaussian_weights` e re-rodo o gate. Me pinga quando A ou B fechar.
═══════════════════════════════════════════════════════════════════
