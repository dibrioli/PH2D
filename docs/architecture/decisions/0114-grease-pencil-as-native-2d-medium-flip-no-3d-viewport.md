# ADR-0114 — Grease Pencil vira o meio nativo 2D **"Flip"**; sem viewport 3D

- **Status:** aceito (Enio, 2026-07-11)
- **Escopo:** novo meio de criação (animação desenhada quadro-a-quadro). Norte [ADR-0075](0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md) (drop-crate, desacoplar por ECS), padrão de tool [ADR-0040](0040-tool-as-isolated-feature-crate.md), entidade-na-hierarchy [ADR-0110](0110-vector-nodes-are-ecs-entities-one-hierarchy.md)/[ADR-0111](0111-vector-shapes-have-transforms-and-use-the-sprite-gizmo.md).
- **Plano de implementação:** [`docs/Flip/`](../../Flip/01_plano_waves.md).

## Contexto

O PH2D quer dar ao artista **todos os meios** de criar arte. Hoje temos três: **Painter** (raster
pintado), **Vector** (vetor exato âncora+handles), **Motion Nodes** (procedural). Falta o quarto e
mais tradicional de todos: **animação desenhada à mão, quadro-a-quadro** — o meio do cel/2D clássico
(Cuphead, Skullgirls, Dragon's Crown), coberto por OpenToonz, TVPaint, Toon Boom, Callipeg,
Procreate. Nenhum módulo atual o cobre: o traço do Painter vira pixel na hora; o path do Vector é um
objeto persistente exato, não um desenho descartável por-quadro.

A referência de estado-da-arte é o **Grease Pencil** do Blender (reescrito no 5.x sobre
`CurvesGeometry` — GPv3). Analisamos a fundo se ele cabe num engine 2D e se exigiria um viewport 3D.

**Dois fatos decidiram a questão.**

1. **O engine é estritamente 2D — e de propósito.** Câmera só ortográfica (`Camera2d`, sem FOV/Z),
   **sem depth buffer** (os únicos `depth_stencil` são `Stencil8` de máscara), `Transform` afim 2D
   de **28 bytes congelado, sem Z** (gate `transform_v2_caps_frozen`), oclusão por painter's
   algorithm na CPU. A própria SKILL declara: *"Não é engine 3D. 2.5D sim; cenas 3D completas não."*
   Um viewport 3D quebraria contratos congelados e o Norte [ADR-0075](0075-multiagent-parallelism-ecs-decoupling-not-runtime-plugins.md).

2. **O valor 2D do Grease Pencil não depende do 3D.** O próprio template *2D Animation* do Blender
   trava o plano de desenho em Front e **ignora a profundidade 3D** — camadas empilhadas como folhas
   de papel, câmera estática. O eixo Z vira valor morto. Ou seja: a essência (layer→frame→traço,
   ghost frames, tween, fill, reshape) é 2D-nativa; o 3D é a *casca* onde o Blender o coloca por ser
   um DCC 3D, não a *essência* do meio.

## Decisão

**1. Adotamos o meio.** Um quarto meio de criação: **animação desenhada quadro-a-quadro**, primeiro
cidadão do engine ao lado de Painter/Vector/Motion.

**2. É 2D nativo, reimplementado clean-room.** Portamos a **essência** do Grease Pencil para a
arquitetura 2D existente: `Layer → frames(nº→Drawing) → Stroke(polilinha com atributos POR-PONTO:
posição, largura, opacidade, cor) + Fill`, com Ghost Frames (onion), Tween (interpolação), Fill
(balde) e Reshape (sculpt de traço). **Não** portamos código do Blender (GPL-2.0): só comportamento
(ver §Clean-room).

**3. SEM viewport 3D.** Não construímos câmera perspectiva, `Transform` com Z, depth buffer nem
navegação 3D. O custo seria enorme (um editor 3D inteiro), quebraria contrato congelado, e — como o
próprio Blender descarta a profundidade no modo 2D — construiríamos um motor 3D para não usá-lo. O
único "brilho 3D" que vale capturar, **2.5D multiplane** (paralaxe por-camada sobre a `Camera2d`
ortográfica existente — um float por camada, aritmética 2D), fica **deferido** como opção futura
barata, dentro do "2.5D" que a SKILL já admite.

**4. Nome do meio: "Flip".** A metáfora do **flipbook** — a linguagem nativa do artista para
animação quadro-a-quadro (FlipaClip, Procreate). Curto, amigável, intuitivo na hora. A UX e a
nomenclatura devem ser **mais intuitivas que o Blender** e próximas
dos apps de artista (Procreate/Callipeg/vetor): *Ghost Frames* (não "onion skin"), *Tween* (não
"interpolate"), *Reshape* (não "sculpt"), *Frames*/*Hold* (não "keyframe/exposure"). Tabela completa
no plano. (Nome é proposta — trocar é find-replace nos docs; ver §Gaps.)

**5. Arquitetura = drop-crate, integrada à Hierarchy desde o início.** Seguindo o precedente
exato do Vector ([ADR-0110](0110-vector-nodes-are-ecs-entities-one-hierarchy.md)):

- `ph2d-flip` — modelo de documento puro (layers/frames/drawings/strokes), serializável.
- `ph2d-tool-flip` — a tool (drop-crate, [ADR-0040](0040-tool-as-isolated-feature-crate.md)): desenhar, apagar, selecionar.
- `ph2d-panel-flip` — painel **docado no slot do Inspector**, aparência dos inspetores de Sprite/Painter.
- Cada objeto Flip é uma **entidade ECS** (`FlipObjectRef`, espelhando `VecPathRef`) na Hierarchy
  única, com `Transform` próprio, movida pelo **gizmo de sprite** ([ADR-0111](0111-vector-shapes-have-transforms-and-use-the-sprite-gizmo.md)), capturada no
  `ProjectState` (undo + save de graça).
- As camadas do Flip espelham a UX de camadas do **Painter** (blend/opacity/visibility/lock/grupo) —
  é o que "integrar ao sistema de camadas da sprite" significa na prática (ver §Gaps).

**6. Ultra-performance por wgpu, tempo real no runtime.** Um pipeline wgpu **dedicado** para o
traço, inspirado no draw engine do GP: pontos em buffers SoA na GPU, **expansão da polilinha em fita
de quads no vertex shader** com junções miter/bevel/round em screen-space e largura/opacidade/cor
por-ponto; fragment shader faz a seção transversal com falloff de *hardness*. Fills triangulados
(CDT) num pipeline irmão. **Troca de quadro pelo playhead = rebind de range, zero re-tessellação na
CPU** → milhares de traços animados a 60/120 Hz no jogo. Para o 2D-ortográfico do PH2D, a matemática
do GP (billboard camera-facing, divisão perspectiva) **colapsa**: espessura = raio_mundo × px/mundo.

**7. Timeline principal = integração DEFERIDA ao fim.** A `ph2d-timeline` nasce noutra linha. O Flip
começa com uma **tira de frames própria e leve** no seu painel (transport + ghost + tween), e só na
última wave se pluga ao `Playhead`/dope-sheet globais.

## Clean-room (GPL)

O Grease Pencil é **GPL-2.0-or-later**; o PH2D é proprietário. Vale a mesma regra do Blender Texture
Paint (`reference/blender-texture-paint/`, memória `project_blender_texture_paint_reference`): a
fonte 5.2 é **referência de comportamento, nunca de código**. O recorte cirúrgico do GP 5.2 (data
model, draw engine + shaders, operadores, `geometry/`) fica **fora do repo** em
`~/Downloads/blender-5.2-grease-pencil-ref/` (como `reference/`, per-máquina, gitignorado por
construção). O implementador **consulta os algoritmos ali antes de cada tópico** e reimplementa do
zero. O plano cita `arquivo:linha` do 5.2 como referência.

## Contrato

O motor novo tem **contrato próprio, ainda NÃO congelado** (congelar é follow-up, como no Vector
novo — CLAUDE.md §6). Esta decisão **não toca** contrato congelado algum: implementa os traits
`Tool`/`PanelEvent` (gate `architecture_tool_contract_surface`) sem alterar sua superfície, e não
encosta na superfície do doc vetorial (`architecture_vector_contract_surface`, que escaneia só
`ph2d-vector-doc`/`-traits`).

## Gaps que este ADR ABRE (verificados, não suspeitados)

1. **"Camadas da sprite" — interpretação a confirmar.** Não existe um *sistema de camadas da sprite*
   separado no engine: sprites são entidades na Hierarchy; o `LayerStack` é interno ao Painter (por
   canvas de UMA sprite, efêmero até "Apply"). Interpretamos o pedido como: **(a)** as camadas do Flip
   usam a MESMA UX/idioma de camadas do Painter, e **(b)** um objeto Flip participa da Hierarchy única
   como qualquer entidade e pode ser **assado ("Apply") num sprite**. Se a intenção era outra (ex.:
   Flip como um *tipo de camada* dentro do `LayerStack` do Painter), é ajuste de escopo — decisão do
   Enio.
2. **Persistência.** O `ProjectState` ganha um campo para o `FlipDoc` (como o `VecScene` é o 2º
   campo). Mesmo cuidado do gap de `vec_save` ([ADR-0112](0112-vector-select-node-pen-are-three-tools.md) §gap 1): pose/nome/parentesco vivem
   no ECS, geometria no doc — a captura precisa dos dois.
3. **2.5D multiplane** (paralaxe por-camada) fica deferido.
4. **Rig/skinning** (o LBS do Rive que o Vector adiou pro fim) não entra: o Flip é desenho por-quadro,
   não deformação de armadura. Fora de escopo.
5. **Nome "Flip"** escolhido pelo Enio (2026-07-11, metáfora do flipbook); alternativas consideradas:
   *Ink*, *Anima*, *Cel*, *Nib*.

## Consequências

**Boas.** O quarto meio — o mais pedido por animadores tradicionais — entra reaproveitando ~70% da
infra (entidades unificadas, undo/ProjectState, gizmo de sprite, picker OKLCH, tokens/widgets,
Playhead determinístico). Um pipeline wgpu dedicado dá animação de traço em tempo real no runtime, o
que nenhum sprite-sheet estático entrega.

**Custo.** Duas peças genuinamente novas e de risco: o **modelo de dado por-quadro** (cel com
semântica de *hold*, drawings refcontados) e o **pipeline de render wgpu** de largura variável. Ambas
têm precedente — respectivamente o `VecScene`/`ProjectState` e o draw engine do GP como referência.

**Quebra de hábito.** Nenhuma para o usuário existente (é aditivo). Para o implementador: é um meio
novo inteiro, não um retoque — segue o plano em waves, fecha cada uma com smoke antes da próxima.
