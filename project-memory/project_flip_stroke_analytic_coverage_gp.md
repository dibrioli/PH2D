---
name: project_flip_stroke_analytic_coverage_gp
description: "Traço do Flip: o tripé do GP (fita+miter_break, GREATER estrito, discard a<0.001) matou acúmulo/spike/bead/escama — MAS falta p0/p3 no fragment (quina sai mordida)"
metadata: 
  node_type: memory
  type: project
  originSessionId: 9af6224e-0122-415e-9f5d-9b462d7c6128
---

O rasterizador de traço do Flip (`ph2d-flip-render/src/shaders/flip.wgsl`) é o
port clean-room do estado 2D do Grease Pencil. **Rodada 7 (2026-07-11): matou
acúmulo/spike/bead/escama** com o TRIPÉ abaixo — **mas o smoke reprovou por um
artefato NOVO: a quina sai MORDIDA** com hardness < 1 (§ no fim). As 3 pernas do
tripé são obrigatórias; remover qualquer uma ressuscita um artefato conhecido:

1. **Fita CONECTADA por miter + `miter_break`** (`gpencil_vertex`,
   `draw_grease_pencil_lib.glsl:696-724`): segmentos adjacentes compartilham o
   vértice de junção (abutam → sem bead/escama). Virada > 120°
   (`-dot(dir_in, dir_out) > 0.5`) NÃO mitra: offset = perpendicular do próprio
   segmento + o quad **estende `r` ao longo da linha** — nunca dobra (fim do
   bowtie/spike da bissetriz). Miter ≤ 120° estica no máx 2× (sem clamp extra).
2. **Depth GREATER ESTRITO + write-depth, por-STROKE**
   (`gpencil_cache_utils.cc:449`): a 2ª face no mesmo pixel (quina quebrada,
   junção, auto-cruzamento) é DESCARTADA, não misturada → zero acúmulo. Default
   do GP: *"the stroke cannot overlap itself"* (`gpencil_vert.glsl:92-96`) — a
   parte desenhada PRIMEIRO fica por cima no auto-cruzamento (união com cor
   sólida). "Parte nova por cima" (3ª rodada) é INCOMPATÍVEL com zero-acúmulo;
   o GP oferece isso como modo de material (`GP_STROKE_OVERLAP`, depth
   por-ponto, aceita acumular) — não portado, é 1 flag + 1 linha se pedirem.
3. **`discard` de fragmento com alpha < 0.001** (`gpencil_frag.glsl:548`), no
   traço E no fill: sem ele, fragmento ~transparente ESCREVE depth e fura a
   geometria que chega depois. **Era o mecanismo do "escamado" do beco
   GREATER+stadium** — o beco não era o GREATER, era a falta do discard.

**Fragment = cobertura ANALÍTICA** (distância do pixel à linha-de-centro, clampada
ao segmento — junção/tampa redonda de graça), perfil de hardness `pow`+smoothstep
do GP, AA por `fwidth`. NÃO usar `v_perp` por-quad (distorce nas junções).

**🔴 O QUE FALTA (a mordida — bug ABERTO, Enio integrou assim):** numa quina
QUEBRADA os quads dos 2 segmentos se sobrepõem no disco da junção; mesmo depth +
GREATER = **o PRIMEIRO vence todos os pixels compartilhados** e pinta ali a sua
queda RADIAL — mas os pixels sobre o EIXO do 2º segmento deveriam ter cobertura ~1
(são o núcleo dele) → **"mordida" macia no lado interno da quina** (só com
hardness < 1; com 1.0 a máscara é degrau e some). **Fix = p0/p3 no fragment**: com
depth first-wins, o fragmento vencedor precisa da distância à **POLILINHA**
(mín. sobre os segmentos vizinhos), não só ao seu — é exatamente por isso que o GP
passa p0/p3 ao `gpencil_stroke_segment_mask`. (Eu descartei esse refino na rodada 7
argumentando que corner ROUND não precisa dele — **errado**, e o smoke provou.)
Alternativa: 2 passes com blend MAX numa scratch (união sem acúmulo, sem depth).

**Oráculo: paridade CPU↔GPU pixel-a-pixel**
(`ph2d-flip-render/tests/gpu_render.rs::assert_matches_analytic`): replica na CPU a
geometria do vertex (quads miter/break/ext + ponto-no-triângulo como o raster) e a
máscara do fragment, e compara o alvo INTEIRO. **Mutações provadas**: GreaterEqual →
desvio 248; sem discard → desvio 254. **⚠️ Ele modela a IMPLEMENTAÇÃO (first-wins),
não a aparência desejada** — por isso ficou verde com a mordida na tela. Antes do
fix, troque o esperado para o **máximo** da máscara sobre os segmentos (a união
real): aí ele fica vermelho hoje e vira alvo irrefutável. **Lição geral:** oráculo
que espelha o código só pega regressão; alvo de aparência tem de modelar a
APARÊNCIA.

**Brush ABSOLUTO** (Enio 2026-07-11): largura em PIXELS DE TELA; `camera_raw`
passa escala 1.0; `fold_model` sobrescreve por `mean_scale` do objeto (gizmo
engrossa; zoom da câmera não).

Verificado em **GPU real** (adapter Vulkan neste Linux) — 9 testes render + 2
composite, debug e `--release`. Ref viva do Blender:
`/home/enio/Downloads/blender-5.2-grease-pencil-ref`. Handoffs:
`docs/HANDOFF_flip_impl.md` §"Rodada 7" (mecanismo da mordida) +
`docs/HANDOFF_flip_NEXT.md` (é a 1ª tarefa do próximo agente da linha).
Ver [[project_flip_module_grease_pencil_2d]].
