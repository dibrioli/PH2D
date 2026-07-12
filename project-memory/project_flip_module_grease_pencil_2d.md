---
name: project-flip-module-grease-pencil-2d
description: "Módulo Flip (port 2D do Grease Pencil) — planejado 2026-07-11, meio de animação desenhada quadro-a-quadro"
metadata: 
  node_type: memory
  type: project
  originSessionId: d8fb901b-b539-492a-9c73-174480d16eb1
---

**Flip** = 4º meio de criação do PH2D (ao lado de Painter/Vector/Motion): animação desenhada
quadro-a-quadro (cel), port CLEAN-ROOM da essência do **Grease Pencil** do Blender. Decisão em
[ADR-0114](../../docs/architecture/decisions/0114-grease-pencil-as-native-2d-medium-flip-no-3d-viewport.md);
plano exaustivo em [`docs/Flip/`](../../docs/Flip/01_plano_waves.md) (00_README, 01_plano_waves,
02_referencia_algoritmos_blender_5.2). Planejado 2026-07-11, **ainda não implementado** (linha paralela).

Fatos-âncora (não re-derivar):
- **2D nativo, SEM viewport 3D.** O engine é estritamente 2D (câmera ortográfica, sem depth buffer,
  `Transform` 28B congelado sem Z, SKILL "não é engine 3D"). O valor 2D do GP não depende de 3D — o
  próprio template 2D do Blender ignora a profundidade. 3D viewport = quebraria contrato congelado +
  ADR-0075; rejeitado. 2.5D multiplane (paralaxe por-camada sobre a `Camera2d`) fica deferido/barato.
- **Arquitetura drop-crate** (ADR-0075/0040), integrada à Hierarchy desde o início: `ph2d-flip` (doc) +
  `ph2d-tool-flip` + `ph2d-panel-flip` + componente ECS `FlipObjectRef` (espelha `VecPathRef`), capturado
  no `ProjectState`. Painel com cara de Inspector; camadas no idioma do Painter.
- **Ultra-perf wgpu:** traço expandido no vertex shader (screen-space, ortho colapsa a matemática 3D do
  GP); troca de quadro pelo playhead = rebind de range (zero re-tessellação). Reusar compositor 22-modos
  do Painter e CDT do Vector.
- **Ordem:** dados → render → tool → frames/ghost/tween → fill → reshape → **timeline por último**
  (a timeline nasce noutra linha).
- **Nomes mais intuitivos que o Blender:** Ghost Frames (onion), Tween (interpolate), Reshape (sculpt),
  Hold, Gap Closure, Grow/Shrink, Precision.

**Referência Blender 5.2 (GPL, clean-room, per-máquina, gitignorada):**
`~/Downloads/blender-5.2-grease-pencil-ref/` — recorte sparse do fonte GP (data model, draw engine +
shaders, operadores paint/erase/fill/sculpt/interpolate, geometry/). Script de re-fetch no 00_README.
Mesma regra do [[project-blender-texture-paint-reference]]: comportamento nunca código.

**Why:** o Enio quer todos os meios de arte digital; animação desenhada à mão é o maior buraco do leque
(Painter=raster, Vector=vetor exato; nenhum cobre cel). Alinha com "engine para artistas".

**How to apply:** ao implementar, consultar SEMPRE `docs/Flip/02_referencia` + o fonte 5.2 antes de cada
tópico; seguir os sites de registro exatos listados no 01_plano_waves (tool-sync, IconId, panel-sync,
EXPECTED_TYPED, feature-proxy, z-order walk, bridge, ComponentRegistry). Ver [[project-vector-cutover-adr0108]]
(precedente exato de tool+painel+entidade ECS) e [[feedback-app-ui-english-only]].
