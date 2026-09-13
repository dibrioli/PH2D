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

## §4 — O mecanismo, verificado no código antes de ser escolhido

⭐⭐⭐ **O shader de sprites JÁ desenha uma malha, sem pipeline nova.** O `vs_main` calcula tudo a
partir de dois atributos por vértice e da instância: `local = anchor + quad_pos · size` →
`world = world_pos + basis · local`, e a UV sai de `quad_uv` (espelhada, repetida, `uv_xform`,
`mix(atlas_uv)`); a tinta por canto interpola sobre o `quad_uv` NÃO espelhado. ⇒ um vértice da
malha leva

- `quad_uv` = a sua coordenada **de repouso** na imagem (`0..1`, `v = 0` em cima — a convenção do
  `QUAD_STRIP`);
- `quad_pos` = a posição **posada** no quad normalizado da sprite, `(local − anchor) ÷ size`, com o
  `anchor` e o `size` **da instância**;

e o shader devolve a posição posada e o texel certo, com tinta, opacidade, mistura,
pré-multiplicação, repetição, tinta por canto e o shader de marca do recorte — **tudo o que uma sprite
tem, sem uma linha de WGSL**.

| facto | onde |
|---|---|
| as 10 pipelines são `TriangleStrip`, sem culling | `ph2d-render/src/pipeline.rs` `build_variant` |
| ⇒ uma lista de `N` triângulos entra como tira com degenerados de ligação: `5N − 2` vértices, zero pipelines novas | |
| o shader lê só os bits **0–4** do `flip_uv` (`&1`, `&2`, `&4`, `(>>3)&3`); a CPU desempacota a mistura com `& 0b111` | `sprite.wgsl` · `instance.rs::unpack_blend` |
| ⇒ os bits **8–31** do `flip_uv` levam a marca de malha **só da CPU**, sem mudar o layout da instância de nenhuma sprite | |
| as chamadas partem-se em runs por `(texture_id, sampling, clip_group, clip_role, mask_role, blend)` e cada run é `draw(0..4, start..end)` | `renderer.rs::compute_runs` · `renderer_draw.rs` · `clip_pass.rs` |
| a sprite é ordenada por rank e filtrada pela janela de banda na recolha | `sprite_collect.rs` |

⛔⛔ **E um QUINTO defeito, latente, achado ao conferir a repouso:** o quad usa
`Sprite::resolve_anchor(ppm)`, que soma `size/2` quando a sprite **não está centrada** e o
**offset** (px → m); o `skin_image::pixel_to_local` usa o `anchor` **cru** ⇒ uma sprite com
*Centered* desligado ou com *Offset* **salta de sítio ao ser presa**. O smoke não o mostra (a imagem
importada nasce centrada e sem offset).

---

## §5 — As waves

### W1 — o primitivo: `SpriteMesh` no passe de sprites (`ph2d-render`, foundational, aditivo) — ✅ FEITA (2026-09-13, `335fe893a`)

- Componente de apresentação `SpriteMesh { local, uv, tris }` (posições posadas em metros LOCAIS da
  sprite, UVs de repouso, triângulos).
- A **recolha** converte `local → quad_pos` com o `anchor`/`size` da própria instância, costura a
  tira, acumula um buffer de vértices de malha **por chamada de render** e escreve a marca
  `(índice + 1) << 8` no `flip_uv`. ⛔ **Instâncias que chegam de fora** (`extra`, o
  `render_instances_only`) **saem sem marca** — não há malha para elas, e uma marca herdada
  indexaria a malha de outra chamada.
- `compute_runs` parte na marca; o laço de desenho, o recorte e a máscara trocam o buffer do slot 0
  e desenham o intervalo da malha.
- **Gates:** a costura dá `5N − 2` vértices e as janelas não-degeneradas são os triângulos de
  entrada · o run parte na malha · a marca sai das instâncias de fora · a volta `local → quad_pos →
  anchor + quad_pos·size` devolve o `local`, **com sprite não-centrada e com offset** · ⭐ GPU: uma
  malha de 2 triângulos em repouso **é o quad ao pixel**, com tinta, opacidade e espelhamento · ⭐⭐
  GPU: arte **translúcida** numa malha fina em repouso **é o quad** (a lei que o caminho do Vello
  reprovava com `10 580` px).

### W2 — a extracção emite a sprite presa com a malha (shell + `ph2d-skeleton-live`) — ✅ FEITA (2026-09-13)

- A guarda `!skinned_image` sai: a sprite presa passa pelo `emit::sprite` (rank, visibilidade,
  propriedades) e ganha o `SpriteMesh` posado (`Fast` = a malha guardada; `Smooth` = refinada contra a
  tolerância em pixels de ecrã).
- `pixel_to_local` passa a usar o `resolve_anchor` (o quinto defeito).
- O `draw_skinned_images` do Vello sai do quadro; o doc falso da `skinned_image` e o gate de texto
  que o «confirmava» são reescritos contra a lei nova.
- **Gates:** a sprite presa tem rank · escondida não emite · tinta/opacidade chegam à instância · a
  malha em repouso coincide com o quad com *Centered* desligado e *Offset*.

⭐ **Como ficou** (mecanismo, gates e as oito mutações na [fila, F6-i](01_a_fila.md)): a malha é posta
por `ph2d_skeleton_live::skin_image::attach_skin_meshes` DEPOIS do extract, na instância base que
ele emitiu — o rank, a visibilidade e as propriedades vêm do `emit::sprite` por construção, e o gate
é que o braço que emite não pergunta pela pele. ⚠️ **O espelho entrou junto com a âncora:** a régua
espelha a POSIÇÃO e a UV é a do quad (`SpriteMesh::uv_at`), senão uma sprite espelhada lia a tinta
fora da silhueta. ⚠️ **O 9-slice e a folha desdobrada** desenham-se sem deformar, com aviso.

### W3 — os outros consumidores de instâncias — ✅ FEITA (2026-09-13), menos o onion (nomeado, com o preço, na [fila F6-i](01_a_fila.md))

Emissivo · vidro do prefab (`present_frost::lift`) · fantasmas de onion · picking (hoje o quad de
repouso) · *View All*. Cada um: ou lê a malha, ou é uma lacuna **nomeada** com o que custa curá-la.

### W4 — o orçamento, re-medido — ✅ FEITA (2026-09-13): `8 738` → **`1 543`** peças, do TEMPO do quadro

Sem o Vello, uma peça é um vértice: o `SKIN_FRAME_PIECES` (derivado do buffer do Vello) deixa de
descrever o recurso deste caminho. ⇒ medir o custo de CPU por peça (deformar + costurar + enviar) e
o de GPU, e escrever o tecto do recurso que sobra. ⛔ Nunca deixar o número de um caminho morto a
limitar o vivo (§0.0).

### W5 — o smoke que ENSINA a ordem — ✅ FEITA (2026-09-13): a barra que atravessa o braço pintado, e o olho

A cena do osso ganha uma sobreposição (o braço pintado atrás de outra peça) e o olho da Hierarquia a
esconder a imagem presa — as duas coisas que a camada de hoje faz ao contrário.
