# ADR-0102 — Inpaint: PatchMatch multi-escala (referência CPU + compute GPU reconciliado)

**Status:** Accepted (pesquisa 2026-07-02; decisão aprovada pelo Enio). Implementado: **engine**
`ph2d-inpaint` (W1 referência CPU + W2 compute GPU reconciliado ε, headless-Metal verde) + **integração
como MODO do Painter** — o Inpaint é um **pincel de heal** (content-aware fill), NÃO uma tool
standalone. **Aguarda smoke manual do Enio** (pintar sobre um defeito → soltar → cura) para fechar como
Done. Polish (gating de painel + GPU no heal) aberto.
**Contexto/decisor:** Enio, 2026-07-02 — "ferramenta de correções de defeitos de imagens … algoritmo
naturalmente pesado então busque logo uma versão que pode ser traduzida tanto em GPU como em CPU e já
implemente ambas: GPU com fallback para CPU … pesquise o melhor algoritmo atual e implemente o melhor."
+ (2026-07-02, correção de rumo): **"o Inpaint é ferramenta do Painter"** — o botão já existe no rail
esquerdo do Painter (`PAINTER_RAIL_INPAINT`); ligar esse botão, NÃO criar uma tool nos Image Tools.
**Relaciona:** ADR-0040 amendment-3 (`CanvasPaintTool` do Painter — o heal roda dentro do pipeline de
stroke do Painter). O engine `ph2d-inpaint` segue o padrão "GPU reconcilia bit-aprox contra o CPU"
(Bloom/S-H, [project-painter-w4-spatial-gpu-bloom-sh]).
**NOTA:** uma primeira tentativa criou uma tool standalone `ph2d-tool-inpaint` nos Image Tools — **rumo
errado, revertido** (o Enio esclareceu que é modo do Painter). O engine `ph2d-inpaint` (W1/W2) foi
mantido; a integração virou um `PaintMode::Inpaint`.
**Docs de detalhe:** [`docs/Inpaint/01_pesquisa_design_plano.md`](../../Inpaint/01_pesquisa_design_plano.md).

## Contexto

Falta uma ferramenta de **correção de defeitos** (remoção de riscos, poeira, manchas, objetos pequenos):
o usuário pinta uma **máscara** sobre a região defeituosa e o sistema **reconstrói** o buraco a partir
do resto da imagem. É um algoritmo **naturalmente pesado** (busca de vizinho-mais-próximo sobre patches);
o Enio exige **uma** formulação que rode **tanto em CPU quanto em GPU**, com **GPU + fallback CPU**.

## Decisão

### Algoritmo: PatchMatch multi-escala (exemplar-based), NNF por Jump-Flooding, reconstrução EM-voting

1. **PatchMatch** (Barnes et al. 2009) é o padrão da indústria — é o motor do **Content-Aware Fill** do
   Photoshop. Pesquisa de 2025 ainda mostra que **supera IA em alta-resolução e texturas repetitivas**,
   porque **copia pixels reais** em vez de alucinar. É **clássico** (sem pesos de ML) ⇒ o **mesmo**
   algoritmo roda em CPU e GPU e é **reconciliável**.
2. **Jump-Flooding NNF** resolve a única aresta problemática do PatchMatch (a propagação é normalmente
   sequencial): a variante jump-flood torna a busca do campo de vizinhos (NNF) **totalmente paralela** e
   idêntica entre CPU e GPU. É o que permite a paridade CPU↔GPU dentro de ε (float, não bit-exato).
3. **Multi-escala coarse-to-fine** (pirâmide, ex. 128→256→512…) + laço **EM** por nível (~4–5 iterações):
   **E** = busca NNF (jump-flood) dos patches do buraco contra a região conhecida; **M** = **voting**
   (média ponderada dos patches sobrepostos casados) reconstrói o buraco. O nível mais grosso é
   inicializado por um preenchimento difusivo barato (Telea-like), grátis.
4. **Rejeitamos ML (LaMa/diffusion):** exigem pesos + infra de inferência, **não** traduzem para um
   shader WGSL, e **perdem** para o PatchMatch justamente nos defeitos texturados/repetitivos que
   dominam este caso de uso. (Fica como possível follow-up opcional, fora deste ADR.)

### Arquitetura (engine reusável + MODO do Painter)

5. **`ph2d-inpaint`** (nova, foundational, `forbid(unsafe)` no caminho CPU): **o algoritmo**.
   - **Referência CPU** = pura Rust, **determinística** (RNG contador 32-bit hash — WGSL não tem `u64`;
     mesmas draws por-pixel em CPU e GPU), **HR-5** transcendental-free — é o **padrão-ouro** contra o
     qual a GPU reconcilia.
   - **Compute GPU** atrás de `feature = "gpu"`: shaders WGSL (jump-flood NNF + gather-voting) sobre
     wgpu/Metal, 8 storage-buffers (máscaras empacotadas em bits pro teto do device).
   - **Runtime = GPU com fallback CPU** (`inpaint(Option<&GpuContext>, req)` — GPU se presente, senão CPU).
   - Reconciliação GPU↔CPU dentro de ε (parity-test headless-Metal `#[ignore]`).
6. **`PaintMode::Inpaint` no `ph2d-tool-painter`** (NÃO uma crate/tool nova): **pincel de heal**.
   - O botão **Inpaint** já existe no rail esquerdo do Painter (`PAINTER_RAIL_INPAINT`); o rail forwarda
     `SelectOption(PAINTER_PAINT_MODE, "inpaint")` → `set_paint_tool_mode` → `PaintMode::Inpaint`.
   - Pincelar **marca** o defeito (disco duro no `inpaint_mask` + tint vermelho ao vivo no canvas); no
     **pen-up** o `heal_inpaint` **recorta** a bbox da máscara + margem (interativo em layer grande —
     PatchMatch roda na vizinhança do defeito, não no canvas inteiro), roda `inpaint_cpu`, escreve os
     pixels reconstruídos de volta na camada e limpa a máscara — **antes** do `close_stroke`, então o
     undo estrutural do Painter captura pré-stroke → curado em UM passo (Cmd+Z).
   - Reusa TODO o pipeline de stroke do Painter (dabs, tamanho de pincel, undo). Zero fiação de shell.

### Ondas

- **W1 — núcleo referência CPU** (`ph2d-inpaint`): distância SSD de patch, NNF jump-flood, EM-voting,
  pirâmide. Testes known-answer (buraco em textura periódica → reconstrói; gradiente → permanece suave).
- **W2 — compute GPU**: WGSL jump-flood + voting em wgpu/Metal, reconciliado dentro de ε contra W1;
  runtime GPU→CPU.
- **W3 — modo do Painter**: `PaintMode::Inpaint` (heal-on-release recortado) + wire do botão do rail +
  testes (heal de um defeito → branco; 1 passo de undo; rail forwarda "inpaint"). **FEITO.**
- **W4 — polish**: gating do painel (esconder cor como Smear/Blur em modo Inpaint), GPU no heal, pistas
  de estrutura/borda, feedback de progresso, tuning.

## Alternativas rejeitadas

- **Inpainting difusivo puro (Navier-Stokes / Telea):** ótimo só para riscos finos; borra texturas
  grandes. Fica como *inicializador* do nível grosso, não como motor.
- **ML (LaMa, diffusion, stable-diffusion inpaint):** melhor em estruturas semânticas grandes, mas
  precisa de pesos/inferência, não vira shader WGSL, e o Enio pediu explicitamente uma formulação
  **CPU+GPU** unificada. Perde em texturas repetitivas de alta-res.
- **Tool standalone nos Image Tools (`ph2d-tool-inpaint`, drop-crate ADR-0040):** foi a **primeira
  tentativa** — máscara + botão Apply + bake via `RasterEditTool` + fiação de shell própria. **Revertida**
  pelo Enio: "o Inpaint é ferramenta do Painter", com o botão já no rail. Vira `PaintMode::Inpaint`, que
  reusa o pipeline de stroke/undo do Painter (heal-brush estilo Photoshop) — zero fiação de shell, e o
  usuário pinta o defeito como qualquer pincel. (O engine `ph2d-inpaint` é agnóstico e sobreviveu à
  reversão intacto.)
- **Só-CPU agora, GPU depois:** contraria o pedido explícito ("já implemente ambas"). A pirâmide +
  jump-flood foram escolhidos justamente para a GPU sair do mesmo desenho — implementar as duas juntas
  é o caminho de menor retrabalho. (O heal do Painter hoje chama o caminho CPU; ligar o GPU no heal é
  W4 — o engine já expõe `inpaint(Some(gpu), …)`.)

## Referências (pesquisa)

- Barnes et al., *PatchMatch* (SIGGRAPH 2009) — gfx.cs.princeton.edu/pubs/Barnes_2009_PAR/patchmatch.pdf
- *Parallel-Friendly PatchMatch based on Jump Flooding* — ResearchGate 278703228
- *Accelerating Exemplar-based Inpainting with GPU/CUDA* — dl.acm.org/10.1145/3457784.3457812
- *PatchMatch vs AI Inpainting* (2025) — supera IA em alta-res/texturas repetitivas
- Deep Learning Inpainting Survey (IJCV 2024)
