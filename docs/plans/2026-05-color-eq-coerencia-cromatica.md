# Plano — Coerência cromática do tool Color Equalization

**Aberto:** 2026-05-26 · **Status:** Tier 1 + Tier 2 fechados. Tier 3 pendente.

Auditoria identificou que a pipeline tem um núcleo perceptual coerente
(`adjust_tonal` em linear sRGB / OKLab; `quantize` em OKLab) cercado de
estágios que operam em **sRGB gamma direto** sem decodificar. Resultado:
três bugs cromáticos concretos (CLAHE / `auto_contrast` / histograma) +
quatro estágios subótimos (denoise / sharpen / auto-WB / posterize).

Pasta exclusiva: `crates/ph2d-tool-color-equalization/`. Sem mudanças
foundational, sem ADR.

## Tier 1 — bugs cromáticos concretos (fechado, commit a seguir)

- [x] **CLAHE em YCbCr (BT.709 full-range)** — equaliza `Y`, preserva
  `Cb/Cr` originais. Substitui a reconstrução `scale = new_L / old_L`
  aplicada uniformemente em R, G, B, que saturava componentes
  individualmente e gerava manchas em áreas suaves +
  shift de matiz em pixels muito escuros (`l_in ∈ {1,2}` → scale > 50×).
- [x] **`auto_contrast` em luminância BT.709 linear** — substitui o
  HSL L = (max+min)/2 anterior, que tratava vermelho (0.21 Y) e azul
  (0.07 Y) como equivalentes e gerava shift de matiz em pixels saturados.
- [x] **`compute_histogram` em BT.709** — substitui o BT.601 anterior
  (0.299/0.587/0.114) para alinhar com CLAHE / `luma_srgb` / canônico
  pra sRGB primaries.
- [x] **Teste `run_pipeline_identity_round_trip_exact`** — assegura
  que `run_pipeline(source, Params::default()) == source` byte a byte
  (valida o fast-path `is_noop` do commit 43fca89 e protege regressão).

Resultado: 132/132 testes verdes (era 131; +1 do round-trip). Nenhum
teste byte-exact existente do CLAHE quebrou — os 4 que pinavam bytes
eram todos sobre input uniforme ou grayscale, onde Cb = Cr = 0 e a
saída YCbCr coincide com a antiga RGB-scale.

## Tier 2 — coerência percebida (fechado)

- [x] **`auto_white_balance` Gray-World em linear sRGB** — média de
  luz é em linear, não em gamma. Em imagens contrastadas (sol+sombra)
  o WB em sRGB gamma é enviesado. Sumas em f64 pra precisão.
- [x] **`bilateral denoise` range similarity em linear sRGB** —
  pré-linearizamos o source uma vez (amortiza sobre `O(r²)` lookups
  por pixel), σ_range mantida numericamente (`(20+50·s)/255`) e
  reinterpretada como linear. Output reencoda sRGB. Removemos o
  hot-path de `clamp8(f32 0..255)`. Mantido o nome do método e a
  assinatura — só o domínio do range mudou.

Resultado: 131/131 verde (perdemos o teste do `luminance_bt709`
removido pelo dead-code cleanup; 132−1=131). Zero quebra em byte-exact.

## Tier 3 — lapidação

- [ ] **`sharpen` Laplacian / Unsharp em linear sRGB** — ringing
  uniforme entre sombras/luzes.
- [ ] **`posterize` Floyd-Steinberg em linear sRGB** — gradientes
  mais suaves.

## Fora de escopo

- **LUT 3D em sRGB** — convenção da indústria (DaVinci / Premiere
  fazem assim); presets foram authored contra a sRGB cube. Manter.

## Notas

- BT.709 YCbCr é trabalhado em **sRGB gamma-encoded** (paridade com
  OpenCV `cvtColor BGR2YCrCb` / Krita). Não decodifica pra linear no
  Tier 1 — o objetivo é coerência de **matiz/chroma** entre tiles, não
  fidelidade radiométrica.
- Defaults agora são identidade (`CLIP_LIMIT_DEFAULT = 1.0`, commit
  `43fca89`), então qualquer mudança cromática só aparece quando o
  usuário move um slider — risco de regressão visual em workflows
  existentes é baixo.
- Testes existentes que pinam bytes específicos de saída CLAHE (e.g.
  `clahe_preserves_alpha`) provavelmente vão precisar atualização — os
  valores RGB mudam porque a reconstrução de chroma muda.
