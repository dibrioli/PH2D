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

## 3. Arquitetura (2 crates)

- **`ph2d-inpaint`** (algoritmo, foundational-mas-1-consumidor; standalone porque o algoritmo é pesado e
  tem grande superfície de teste). CPU = referência-ouro (`inpaint_cpu`); GPU atrás de `feature = "gpu"`
  (W2), reconciliado por parity-test. Runtime = GPU→CPU fallback.
  - `plane.rs` (imagem RGB-f32 + pirâmide), `mask.rs` (buraco + `Regions` source/target), `nnf.rs`
    (jump-flood + random-search + SSD), `vote.rs` (M-step), `rng.rs` (splitmix64), `lib.rs`
    (API + orquestração), `tests.rs` (known-answer).
- **`ph2d-tool-inpaint`** (drop-crate ADR-0040, W3): pinta a máscara, botão **Inpaint** → roda → baka no
  layer (RasterEditTool), ícone no rail (IconId), painel (patch size / iterações / toggle GPU), undo.

## 4. Ondas

- [x] **W1 — núcleo referência CPU** (`ph2d-inpaint`): SSD, jump-flood NNF, EM-voting, pirâmide + 20
  known-answer tests (stripes reconstruídas, flat continua flat, gradiente suave, byte-reproduzível).
- [ ] **W2 — compute GPU**: WGSL jump-flood + voting em wgpu/Metal, reconciliado dentro de ε contra W1;
  runtime GPU→CPU.
- [ ] **W3 — tool + UI**: máscara, invoke, bake, ícone, painel, undo.
- [ ] **W4 — polish**: pistas de estrutura/borda, progresso, tuning.
