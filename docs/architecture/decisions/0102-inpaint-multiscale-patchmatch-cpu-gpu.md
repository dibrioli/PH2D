# ADR-0102 — Inpaint: PatchMatch multi-escala (referência CPU + compute GPU reconciliado)

**Status:** Accepted (pesquisa 2026-07-02; decisão aprovada pelo Enio; implementação W1 iniciada).
**Contexto/decisor:** Enio, 2026-07-02 — "ferramenta de correções de defeitos de imagens … algoritmo
naturalmente pesado então busque logo uma versão que pode ser traduzida tanto em GPU como em CPU e já
implemente ambas: GPU com fallback para CPU … pesquise o melhor algoritmo atual e implemente o melhor."
**Relaciona:** ADR-0040 (tool = drop-crate isolada) para a crate de ferramenta; padrão "GPU reconcilia
bit-aprox contra o `apply_*` da CPU" já usado em Bloom/S-H ([project-painter-w4-spatial-gpu-bloom-sh])
e no compositor. Efeitos/adjustments em `ph2d-painter-effects`.
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

### Arquitetura (2 crates, espelha o precedente Painter)

5. **`ph2d-inpaint`** (nova, foundational, `forbid(unsafe)` no caminho CPU): **o algoritmo**.
   - **Referência CPU** = pura Rust, **determinística** (RNG seeded, splitmix64), **HR-5**
     transcendental-free onde praticável — é o **padrão-ouro** contra o qual a GPU reconcilia.
   - **Compute GPU** atrás de `feature = "gpu"`: shaders WGSL (jump-flood NNF + voting) sobre wgpu/Metal.
   - **Runtime = GPU com fallback CPU** (`Inpainter::run` escolhe GPU se disponível, senão CPU).
   - Reconciliação GPU↔CPU dentro de ε via **dev-dependency** (padrão Bloom/S-H), gate no fechamento.
6. **`ph2d-tool-inpaint`** (nova, drop-crate ADR-0040): **a ferramenta**.
   - Pinta a **máscara** do defeito (reusa a infra de brush/máscara), botão **Inpaint** → roda o
     algoritmo → **baka** no layer (via `RasterEditTool`), ícone no rail (novo `IconId`, ordem
     alfabética — [feedback-new-tool-icon-needs-iconid]), painel (patch size / qualidade-iterações /
     toggle GPU), **undo** estrutural.
   - Registro via **tool-sync codegen** (fan-out drop-in) — [project-tool-isolation-freeze-2026-05-22].

### Ondas

- **W1 — núcleo referência CPU** (`ph2d-inpaint`): distância SSD de patch, NNF jump-flood, EM-voting,
  pirâmide. Testes known-answer (buraco em textura periódica → reconstrói; gradiente → permanece suave).
- **W2 — compute GPU**: WGSL jump-flood + voting em wgpu/Metal, reconciliado dentro de ε contra W1;
  runtime GPU→CPU.
- **W3 — tool + UI**: máscara, invoke, bake, ícone, painel, undo.
- **W4 — polish**: pistas de estrutura/borda, progresso, tuning.

## Alternativas rejeitadas

- **Inpainting difusivo puro (Navier-Stokes / Telea):** ótimo só para riscos finos; borra texturas
  grandes. Fica como *inicializador* do nível grosso, não como motor.
- **ML (LaMa, diffusion, stable-diffusion inpaint):** melhor em estruturas semânticas grandes, mas
  precisa de pesos/inferência, não vira shader WGSL, e o Enio pediu explicitamente uma formulação
  **CPU+GPU** unificada. Perde em texturas repetitivas de alta-res.
- **Painter como host (modo de pintura em vez de tool própria):** Inpaint não é pintura (é correção
  algorítmica com máscara + bake); tool isolada (ADR-0040) mantém o isolamento multi-agente.
- **Só-CPU agora, GPU depois:** contraria o pedido explícito ("já implemente ambas"). A pirâmide +
  jump-flood foram escolhidos justamente para a GPU sair do mesmo desenho — implementar as duas juntas
  é o caminho de menor retrabalho.

## Referências (pesquisa)

- Barnes et al., *PatchMatch* (SIGGRAPH 2009) — gfx.cs.princeton.edu/pubs/Barnes_2009_PAR/patchmatch.pdf
- *Parallel-Friendly PatchMatch based on Jump Flooding* — ResearchGate 278703228
- *Accelerating Exemplar-based Inpainting with GPU/CUDA* — dl.acm.org/10.1145/3457784.3457812
- *PatchMatch vs AI Inpainting* (2025) — supera IA em alta-res/texturas repetitivas
- Deep Learning Inpainting Survey (IJCV 2024)
