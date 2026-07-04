---
name: pixel-center-vs-edge-coord
description: "Quando faz CPU resampling de imagens (composite, merge, warp, etc.), o `img_x = (local/size + 0.5) * W` retorna EDGE-coord; bilinear samplers esperam CENTER-coord. Subtrair 0.5 no fim, OU o resultado terá blur de meio pixel em TUDO."
metadata: 
  node_type: memory
  type: feedback
  originSessionId: 7d5e6481-e38a-41fd-b4ce-ae6413dd4bc6
---

Bug clássico de graphics: ao reimplementar amostragem de textura no CPU (merge de sprites, copy/paste, warp, qualquer composite), há DUAS convenções de "coordenada de pixel" e elas precisam casar:

- **Edge-coord**: `img_x = 0` = **borda esquerda** do pixel 0; `img_x = W` = borda direita do último pixel. É a convenção do forward chain `(local / size + 0.5) * W` que sai naturalmente da derivação geométrica image-to-world.
- **Pixel-index / center-coord**: `img_x = 0` = **centro** do pixel 0; `img_x = W-1` = centro do último pixel. É a convenção que `textureSample` do wgpu/wgsl usa (UV `(0.5/W, 0.5/H)` é o centro do texel 0) e que TODO bilinear sampler natural espera (`x.floor()` = índice de pixel, fractional = peso pra próximo).

**A diferença é meio pixel.** Se você compor as duas convenções sem corrigir, `img_x` cai sempre em `k + 0.5` (fracionário) → bilinear mistura 50/50 dois pixels adjacentes em TODO pixel da imagem → blur uniforme de meio pixel.

**Why:** Enio 2026-05-27, feature Merge Sprites. Eu derivei o forward `(i/W - 0.5) * size_w` (edge-coord) e a inversa correspondente `(local/size + 0.5) * W` (edge-coord), depois passei o resultado direto pro bilinear sampler que espera center-coord. Mesmo com grid perfeitamente snapped no source primário, todo output pixel virava `img_x = k + 0.5` → blur de meio pixel inevitável. O sintoma "borrão geral sem dark fringe" depois de já ter corrigido o premul-sampling é a assinatura desse bug.

**How to apply:** sempre que escrever CPU resampling/composite:

1. Quando derivar `img_x` por inversão geométrica, ele sai em **edge-coord**.
2. Antes de passar pro bilinear, **subtrair 0.5** em cada eixo:
   ```rust
   let img_x = (local_x / size_w + 0.5) * src.w as f32 - 0.5;
   let img_y = (0.5 - local_y / size_h) * src.h as f32 - 0.5;
   ```
3. Isso casa com o GPU `textureSample` e com o sampler natural `x.floor() = index`.
4. **Sampler range** agora é `[-0.5, W - 0.5]` (não `[0, W-1]`). O `-0.5` na borda esquerda é a zona clamp-to-edge de meio pixel. Sem ajustar o range, `floor()` em `-0.5` faz underflow em `u32` → UB ou panic.
5. **Clamp interno**: dentro do range válido, faça `xc = x.clamp(0.0, (w-1) as f32)` antes de `floor()`. Valores em `[-0.5, 0)` colapsam pra pixel 0 (clamp-to-edge).

**Sinais do bug:**
- Output tem blur uniforme em TODA a imagem, não só nas bordas.
- O blur persiste mesmo depois de você corrigir dark-fringe (com premul sampling) e grid alignment.
- Subir tolerância/threshold do composite não ajuda — o problema é no MAPEAMENTO de pixel pro source, não no algoritmo.
- Numa imagem 1:1 sem rotação nem scale, o resultado deveria ser bit-exact e não é.

**Verificação rápida:** para um source axis-aligned em scale=1, escolha um pixel do output que devia mapear pro pixel `k` do source. Calcule `img_x` manualmente. Se der `k.5`, falta o `-0.5`. Se der `k.0`, está correto.

Linka com [[project-painter-t15-complete-2026-05-26]] (que tem stamp scheduler ABI e também precisa de center-convention), [[pipeline-inject-dont-cap]] (outro reaprendizado de "não duplicar — usar a convenção do pipeline downstream").
