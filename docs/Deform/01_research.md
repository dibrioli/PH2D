# 01 — Pesquisa: Procreate e além

## 1. Procreate — dois sistemas distintos (não confundir)

No Procreate isto são **duas ferramentas separadas**, com mental-models diferentes.
Nosso ganho de simplicidade vem de **unificá-las sob um kernel só** (ver `02`).

### 1.1 Transform (geométrico / bounding-box) — manipulação de geometria
Opera sobre uma caixa de seleção; move geometria, não "empurra pixels". 4 modos:

| Modo | O que faz | Matemática |
|---|---|---|
| **Freeform** | Escala cada handle **ignorando o aspect ratio** (W/H independentes). | Escala não-uniforme (afim). |
| **Uniform** | Escala **preservando o aspect ratio**. | Escala uniforme (afim, ratio travado). |
| **Distort** | Arrasta **cada canto independente** → perspectiva/skew. | **Homografia** (projetiva 2D, 4 pontos → matriz 3×3). |
| **Warp** | Malha (mesh grid) sobre o conteúdo; arrasta cantos, lados **e o interior**. | **Interpolação de mesh** (bilinear/Coons por célula); sub-modo Advanced Mesh. |

Comum aos 4: rotação, flip H/V, snapping, "Fit to Screen", reamostragem (Nearest/Bilinear).

### 1.2 Liquify (per-pixel) — vive em **Adjustments**, não em Transform
Sem bounding-box. É um **campo de deslocamento por pincel**. Modos:

| Modo | Efeito no campo de deslocamento |
|---|---|
| **Push** | Empurra pixels na **direção do traço** (smudge forte). |
| **Twirl L / R** | **Rotaciona** pixels ao redor do ponto (vórtice horário/anti-horário). |
| **Pinch** | **Suga** pixels para dentro (contração). |
| **Expand** | **Empurra** pixels para fora (balão). |
| **Crystals** | Como Expand, mas **desigual/ruidoso** → cacos afiados. |
| **Edge** | Suga pixels para uma **linha** (não ponto) → dobra duas metades. |
| **Reconstruct** | Pincel de "undo" — repinta o original por baixo, revertendo distorção seletivamente. |

Sliders: **Size** (raio), **Pressure** (intensidade × pressão), **Distortion** (ruído
caótico no campo), **Momentum** (o efeito continua/overshoot após levantar), **Amount**
(pós-aplicação: atenua a força do que já foi aplicado).

## 2. Estado-da-arte **além** do Procreate (o que nos torna superiores)

### 2.1 Handle/skeleton shape-preserving (o diferencial)
| Técnica | Referência canônica | Uso |
|---|---|---|
| **MLS image deformation** (afim/similaridade/**rígido**) | Schaefer, McPhail & Warren, *Image Deformation Using Moving Least Squares*, ACM TOG 25(3):533–540, SIGGRAPH 2006 | Warp por handles de **ponto ou linha**, soluções fechadas, tempo real, preserva rigidez local. O "Puppet" superior. |
| **Puppet / ARAP pin warp** | Igarashi et al. 2005; Sorkine & Alexa (ARAP) 2007 | Posar personagem cravando pinos numa malha triangulada. Procreate **não tem**. |

### 2.2 Freeze / Protect mask (Procreate mobile **não tem**)
Photoshop Liquify tem Freeze/Thaw mask. Nós temos **de graça**: o sistema de
**Selection/Mask que está landando agora** (ADR-0103) já dá cobertura por-texel
feathered (`selection_coverage_at`) — reusar como região congelada.

### 2.3 Warps paramétricos (baratos, viram **nós** → animáveis/não-destrutivos)
Spherize/Bulge, Twirl global, Ripple/Wave, **Polar Coordinates** (rect↔polar),
Lens/Fisheye, **Displace** (deforma A pela luminância/vetor de imagem B). Todos
encaixam em `ph2d-node-*` (FBP) — algo que o Procreate **não consegue fazer**
(deformação animável não-destrutiva num grafo).

### 2.4 Não-destrutivo re-editável
O Painter já tem o padrão Apply/Apply&Keep + acumulador (stroke shape-editors,
`shape_offset_base_px`). A deformação herda: editável até o Apply, ao contrário do
Liquify do Procreate (destrutivo-imediato).

## 3. Fundamento do kernel: **inverse (backward) warping**

Decisão de implementação canônica:
- **Forward (scatter):** cada pixel-fonte é jogado no destino → **buracos** e overlaps.
- **Inverse (gather) ✅:** para cada pixel de **destino**, aplico o deslocamento **inverso**
  e amostro a fonte (bilinear) → **sem buracos**, qualidade superior. É o padrão de todo
  imwarp sério.

Consequência arquitetural: **um só kernel** `sample(dst) = source(dst − D(dst))` com um
**gerador de campo `D`** plugável. Liquify-Push, Twirl, Pinch, Edge, Warp-mesh, MLS,
Polar, Spherize — **todos** são só um `D` diferente. Isso é o coração do anti-redundância.

## Fontes
- [Transform — Procreate Handbook](https://help.procreate.com/procreate/handbook/transform) · [Freeform](https://help.procreate.com/procreate/handbook/transform/transform-freeform) · [Distort](https://help.procreate.com/procreate/handbook/transform/transform-distort) · [Warp](https://help.procreate.com/procreate/handbook/transform/transform-warp)
- [Liquify — Procreate Handbook](https://help.procreate.com/procreate/handbook/adjustments/adjustments-liquify)
- Schaefer, McPhail, Warren 2006 — *Image Deformation Using Moving Least Squares* (TAMU PDF: people.engr.tamu.edu/schaefer/research/mls.pdf)
- Backward vs forward mapping — Towards Data Science, *Forward and Backward Mapping for Computer Vision*

> **Nota de método (memória do projeto):** referências acima verificadas por WebSearch/handbook oficial.
> Antes de qualquer ADR ambicioso, re-verificar cada citação e portar a math canônica
> (DIRETIVA §1) — nada de constante-de-magia inventada.
