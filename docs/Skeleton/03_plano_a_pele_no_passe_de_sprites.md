# 03 — PLANO: a imagem presa ao esqueleto é desenhada como MALHA dentro do passe de sprites

> **2026-09-13 · `line/Vector`.** Plano antes do código (DIRETIVA §1). Fonte dos factos: a fila
> [`01_a_fila.md`](01_a_fila.md) F6-e…F6-h e as sondas citadas; nenhum número aqui é de memória.

---

## §1 — O problema, medido

A 2.ª mídia (F6, 2026-09-09) desenha uma imagem presa como **um recorte do Vello mais um afim por
triângulo**, na cena do CHROME, depois de o passe de sprites ter corrido. Quatro defeitos, **uma
causa** — a imagem presa deixou de ser uma sprite do quadro e passou a ser uma camada por cima dele:

| # | defeito | evidência |
|---|---|---|
| 1 | **Perde a ordem de profundidade** — fica por cima de todas as sprites, de toda a arte do documento e do vidro do prefab; entre imagens presas, ordem de arquétipo | a guarda `drawn && !skinned_image(…)` salta o `emit::sprite`, único sítio onde uma sprite empurra o `SortInput` (`shells/desktop/src/render_loop/sim_extract.rs`); o desenho vai para o `vector_scene` (`fase_vector_edit_overlay.rs`) |
| 2 | **Uma imagem ESCONDIDA continua a desenhar** | o `off_canvas::draws_this_frame` só corre na extracção; o `draw_skinned_images` não o pergunta |
| 3 | **Perde as propriedades da sprite** (tinta, opacidade, espelhamento, mistura, filtro) | zero leituras no `skin_image.rs` |
| 4 | **Costuras** entre triângulos, em todo modo | `1 − a·b` na aresta partilhada; `16 580` px a zoom 4 com `216` peças, alfa mínimo `182` (`ph2d-render::skin_pieces_gpu_cost`) |
| + | o tecto de peças do Vello, partilhado pelo quadro | `11` palavras por peça de `1 << 18`; quadro em branco a `21 600` peças |

---

## §2 — As alternativas, e o número que recusa cada uma

| alternativa | cura | não cura | veredito |
|---|---|---|---|
| **A de hoje:** recorte + afim por triângulo no Vello | — | 1 · 2 · 3 · 4 · + | ⛔ é o defeito |
| **Dilatar cada recorte** `0,5 px` | 4 em arte opaca | 1 · 2 · 3; e em arte translúcida **piora** `10 580 → 40 050` px, pior erro `20 → 111` | ⛔ medida (`measure_seams_against_clip_dilation`) |
| **Rasterizar a malha numa textura própria e desenhá-la na cena do chrome** (o molde do FX: `register_texture` + id estável) | 4 · + | 1 · 2 · 3 | ⛔ deixa a camada por cima do quadro |
| **A mesma textura, mas como participante VECTORIAL numa banda** | 1 · 2 · 4 · + | 3 (reimplementar tinta/mistura/… no shader próprio); e cada imagem presa entre sprites **parte uma banda** — o `bands()` tem `MAX_BANDS = 16` e acima disso cai para *«todas as sprites, depois todos os vectores»* (`draw_bands.rs`), que é perder a ordem outra vez num rig com muitas peças | ⛔ tecto estrutural |
| **Uma instância de sprite por triângulo, com descarte fora do triângulo** | 1 · 2 · 3 · 4 | muda o layout do `RenderInstance` (Pod com gate de tamanho) para **todas** as sprites do app; e a partição por descarte em ponto flutuante não é exacta nas arestas | ⛔ preço em todas as sprites por uma feature |
| **A malha DENTRO do passe de sprites, no lugar da sprite na ordem** | 1 · 2 · 3 · 4 · + | — | ⭐ **o padrão-ouro** |

⚠️ **O comportamento que se pede é o de qualquer motor 2D com esqueleto** (o `Polygon2D` com
`Skeleton2D` do Godot, a `MeshAttachment` do Spine): a peça deformada **é** um item do quadro, com a
ordem, a visibilidade e a modulação de um item do quadro. A diferença é só *quantos vértices*.

---

## §3 — A decisão

**A extracção passa a EMITIR a sprite presa** — com rank, pela porta de visibilidade, com todas as
propriedades que o `emit::sprite` já copia — e **marca-a** com a malha posada. O passe de sprites,
ao chegar a ela na sequência ordenada, **desenha a malha em vez do quad**, com o mesmo material, a
mesma tinta e a mesma mistura. O `draw_skinned_images` do Vello sai do quadro.

⇒ as costuras somem (triângulos rasterizados pela regra de canto, sem AA nas arestas internas: cada
centro de pixel pertence a um triângulo só), e o tecto do Vello deixa de se aplicar à pele — o custo
de uma peça passa a ser um vértice.

⚠️ **O que a malha tem de reproduzir ao pixel na pose de repouso é o QUAD de hoje da sprite** (e
não o desenho do Vello, que é o defeito): a malha em repouso cobre exactamente o quad, e os texels
amostrados são os mesmos.

---

## §4 — As waves

⏳ **Em escrita** — dependem do mapa do passe de sprites (campos da instância e o que o shader faz
com cada um · a partição das chamadas de desenho · os grupos de recorte · os passes que consomem
instâncias · o mecanismo de «desenho extra», se existir).
