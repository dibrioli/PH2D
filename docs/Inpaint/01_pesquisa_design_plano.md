# Inpaint — pesquisa, design e plano de ondas

> Decisão registrada em [ADR-0102](../architecture/decisions/0102-inpaint-multiscale-patchmatch-cpu-gpu.md).
> Ferramenta de **correção de defeitos** (riscos, poeira, manchas, remoção de objetos pequenos):
> o usuário pinta uma **máscara** sobre o defeito e o sistema **reconstrói** o buraco a partir do
> resto da imagem. Algoritmo pesado ⇒ **uma** formulação que roda **CPU e GPU** (GPU + fallback CPU).

## 1. Pesquisa — por que PatchMatch multi-escala

| Candidato | Veredito |
|---|---|
| **PatchMatch** (Barnes 2009) + jump-flood NNF + EM-voting | **ESCOLHIDO.** Padrão da indústria (Content-Aware Fill do Photoshop). Clássico (sem pesos ML) ⇒ mesmo algoritmo CPU+GPU, reconciliável. Supera IA em alta-res/texturas repetitivas (copia pixels reais, não alucina). |
| Difusivo (Telea / Navier–Stokes) | Só bom p/ riscos finos; borra texturas grandes. Reaproveitado como *inicializador* do nível grosso. |
| ML (LaMa / diffusion) | Melhor em estrutura semântica grande, mas exige pesos+inferência, **não** vira shader WGSL, perde em textura repetitiva. Fora do pedido (CPU+GPU unificado). |

**Fontes:** Barnes et al. *PatchMatch* (SIGGRAPH 2009); *Parallel-Friendly PatchMatch based on Jump
Flooding* (ResearchGate 278703228); *Accelerating Exemplar-based Inpainting with GPU/CUDA*
(dl.acm.org/10.1145/3457784.3457812); *PatchMatch vs AI Inpainting* (2025); Deep Learning Inpainting
Survey (IJCV 2024).

## 2. Algoritmo

Multi-escala **coarse-to-fine** (pirâmide) + laço **EM** por nível:

1. **Pirâmide:** downsample box-2×2 da imagem; a máscara faz downsample por `max` (o buraco *cresce*
   ao subir, para o nível grosso nunca tratar defeito como fonte válida).
2. **Nível mais grosso:** inicializa o buraco com a **média dos pixels conhecidos**.
3. **Por nível (grosso→fino):**
   - Semente do buraco = **upsample bilinear** do resultado do nível anterior (ou média no topo).
   - **EM × N** (default 6):
     - **E (busca NNF):** para cada patch-alvo (overlapa o buraco), acha o melhor patch-**fonte**
       (100% conhecido) por **SSD**, via PatchMatch com **propagação jump-flooding** (passes `n/2…1`)
       + **random-search** de raio decrescente. O NNF é *double-buffered* ⇒ ordem-independente ⇒ CPU
       e GPU rodam os **mesmos** passes.
     - **M (voting):** cada pixel do buraco = média ponderada (`peso = 1/(1+SSD)`) de todos os patches
       casados que o cobrem.
4. **Saída:** pixels do buraco = reconstrução (alpha opaco); **pixels conhecidos = byte-idênticos** à
   entrada (inpaint só toca a máscara).

**HR-5 (determinismo):** SSD compara distâncias ao quadrado (sem `sqrt`), raio halving por shift (sem
`pow`), média em sRGB-float (sem `pow` de sRGB↔linear), RNG splitmix64 (inteiro). Mesma seed ⇒ saída
byte-idêntica em toda plataforma. A GPU (W2) reconcilia dentro de ε (só difere nos últimos bits das
somas f32).

## 3. Arquitetura (engine + MODO do Painter)

- **`ph2d-inpaint`** (algoritmo, foundational). CPU = referência-ouro (`inpaint_cpu`); GPU atrás de
  `feature = "gpu"` (W2), reconciliado por parity-test. Runtime = `inpaint(Option<&GpuContext>, …)`
  (GPU→CPU fallback).
  - `plane.rs` (imagem RGB-f32 + pirâmide), `mask.rs` (buraco + `Regions` source/target), `nnf.rs`
    (jump-flood + random-search + SSD), `vote.rs` (M-step), `hash.rs` (counter-hash 32-bit p/ paridade
    GPU), `rng.rs`, `gpu/` (WGSL), `lib.rs` (API + orquestração), `tests.rs` (known-answer).
- **`PaintMode::Inpaint` no `ph2d-tool-painter`** (NÃO uma tool nova): pincel de **heal**. O botão já
  existe no rail do Painter; o rail forwarda `"inpaint"` → `PaintMode::Inpaint`. Pincelar marca o defeito
  (`inpaint_mask` + tint vermelho); no pen-up `heal_inpaint` recorta a bbox + margem, roda `inpaint_cpu`,
  escreve de volta na camada, limpa a máscara — antes do `close_stroke` (1 passo de undo). Reusa dabs /
  tamanho de pincel / undo do Painter; zero fiação de shell. `paint/inpaint.rs`.

## 4. Ondas

- [x] **W1 — núcleo referência CPU** (`ph2d-inpaint`): SSD, jump-flood NNF, EM-voting, pirâmide + 24
  known-answer tests (stripes reconstruídas, flat continua flat, gradiente suave, byte-reproduzível).
- [x] **W2 — compute GPU** (`feature = "gpu"`): 5 kernels WGSL (init/cost/propagate/random-search/vote)
  espelhando a CPU op-a-op — mesmo counter-hash **32-bit** (WGSL não tem `u64`), mesma ordem de passes,
  gather-vote (sem atomics). Driver per-nível (`gpu/mod.rs`) com uniform-por-dispatch. Runtime GPU→CPU
  (`inpaint(Option<&GpuContext>, …)`). 3 testes headless-Metal verdes: **reconcilia com a CPU dentro de
  ε** (mean ≤ 2/255) + stripes/flat standalone. As 3 máscaras empacotadas em 1 buffer `flags` (bit0
  source/bit1 target/bit2 hole) p/ caber no piso de 8 storage-buffers/stage.
- [x] **W3 — modo do Painter** (`PaintMode::Inpaint`): o Inpaint é um **pincel de heal do Painter**, não
  uma tool standalone. O botão `PAINTER_RAIL_INPAINT` (que já existia no rail esquerdo) foi ligado —
  `rail_painter_tools.rs` forwarda `SelectOption(PAINTER_PAINT_MODE, "inpaint")` → `set_paint_tool_mode`
  → `PaintMode::Inpaint`. Pincelar acumula `inpaint_mask` (disco duro) + tinta o canvas de vermelho ao
  vivo (`stamp_dabs_inpaint`); no pen-up `heal_inpaint` recorta a bbox da máscara + margem
  (`hole/2`, clamp 24–128 px → interativo em layer grande), roda `inpaint_cpu`, escreve os pixels
  reconstruídos de volta na camada, limpa a máscara — **antes** do `close_stroke` (undo de 1 passo).
  Testes: heal de um defeito vermelho → branco; 1 undo restaura o defeito; rail forwarda "inpaint".
  Suite do Painter 300 verde, editor-core rail verde, clippy/fmt limpos. **Falta só o smoke manual do
  Enio** (pintar sobre um defeito com caneta/mouse → soltar → cura; headless não roda input winit).
  > **Nota de rumo:** a 1ª tentativa foi uma tool standalone `ph2d-tool-inpaint` nos Image Tools —
  > revertida (o Enio esclareceu que é modo do Painter). Engine `ph2d-inpaint` (W1/W2) mantido.
- [ ] **W4 — polish**: gating de painel (esconder cor como Smear/Blur em modo Inpaint), GPU no heal
  (`inpaint(Some(gpu),…)`), pistas de estrutura/borda, feedback de progresso, tuning.
