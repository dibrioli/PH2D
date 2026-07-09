# 04 — Evidências da indústria: estado de simulação em grafos de nós (pesquisa primária)

**Data:** 2026-07-09 · **Método:** duas varreduras web independentes (agentes de pesquisa), só
fontes primárias (manuais oficiais, PRs/design tasks, fóruns oficiais), URL em toda claim.
**Consumidor:** `03_reentrada_integrate_estudo_padrao_ouro.md` (a síntese e a decisão moram lá).
Este doc preserva a evidência bruta para futuras decisões (zonas O4, kill nodes, cache/scrub).

---

## 1. Houdini — POP networks e o Solver SOP

**(a) Ciclos no grafo visível?** Não. Uma POP network é uma cadeia unidirecional processada
"in a top-down manner" até o solver; o HDK descreve a passada por timestep como objetos que
"process in a top-down fashion the DOP_Nodes they encounter en route to the Display flagged
node" ([HDK: DOP data flow](https://www.sidefx.com/docs/hdk/_h_d_k__data_flow__d_o_p.html)).

**(b) Estado frame-a-frame.** NÃO vive no grafo: vive no banco da simulação (`SIM_Object` +
`SIM_Data`); para partículas, é geometria persistente com atributos (`P`, `v`, `force`, `age`,
`id`). "A POP solver updates particles according to their velocities and forces" um timestep por
vez ([POP Solver](https://www.sidefx.com/docs/houdini/nodes/dop/popsolver.html)). O grafo é uma
*receita re-executada contra estado persistente a cada timestep* — o fio significa "esta
transformação se aplica ao stream de partículas neste timestep", não "dado flui uma vez".

**(c) O que o artista liga.** POP Object + POP Source e uma cadeia de microsolvers — POP Force,
POP Wind, POP Drag — ligada nos inputs do solver: "It is intended that its green inputs have POP
microsolvers wired into them to control the particles and accumulate forces" ([POP
Solver](https://www.sidefx.com/docs/houdini/nodes/dop/popsolver.html)); "The output of this node
should be wired into a solver chain… The final wiring should go into one of the purple inputs of
a full-solver" ([POP Force](https://www.sidefx.com/docs/houdini/nodes/dop/popforce.html)). POP
Force "modifies the `force` attribute" — forças ACUMULAM num atributo, o solver integra
(mass-aware). "There can only be one, solid wire from every generator node to the POP Solver"
([Streams](https://www.sidefx.com/docs/houdini/dopparticles/streams.html), [Network
flow](https://www.sidefx.com/docs/houdini/dopparticles/network_flow.html)). Nenhum fio de
feedback é visível, nunca.

**(d) Solver SOP — o padrão subnet.** Para feedback genérico de geometria: "Start building
surface nodes from the output of the `Prev_Frame` node. This network will be cooked every frame,
with the output of the `Prev_Frame` node containing the geometry from the previous frame";
"The `Input_1` node… contains the *original* geometry" ([Solver
SOP](https://www.sidefx.com/docs/houdini/nodes/sop/solver.html)). O delay de 1 frame é um
**nó-fonte dentro do subnet do solver**, não uma aresta no grafo pai.

## 2. Blender Geometry Nodes — Simulation Zone (3.6+)

**(a) Ciclos?** Não. A zona é um bracket: par `Simulation Input`/`Simulation Output`;
"It is not possible to have any link going towards outside. The result of the simulation can
only be accessed via the Simulation Output node. This also allows sub-frame interpolation for
motion blur" ([manual](https://docs.blender.org/manual/en/latest/modeling/geometry_nodes/simulation/simulation_zone.html)).

**(b) Estado.** Lista tipada de itens cacheada no Output. Do PR de merge (Hans Goudey): "On the
first frame, the inputs of the `Simulation Input` node are evaluated to initialize the simulation
state. In later frames these sockets are not evaluated anymore… On every next frame, the
`Simulation Input` node outputs the simulation state of the previous frame" ([PR
#104924](https://projects.blender.org/blender/blender/pulls/104924)). Os dois nós são
"internally linked together by their `bNode.identifier`" — pareamento invisível, não fio.

**(c) Partículas.** Não há solver pronto: o artista integra NA MÃO dentro da zona
(`Set Position` deslocado por `velocity × Delta Time`, velocity como state item).

**(d) Racional.** Workshop out-2022: "just giving access to the previous frame's data would make
so much possible… By design, it is not possible to have any link going towards outside"; zonas
foram escolhidas como primitivo GERAL de controle de fluxo — "Serial loops and Parallel loops…
can be built on top of the simulation design" ([Geometry Nodes Workshop
2022](https://code.blender.org/2022/11/geometry-nodes-workshop-2022/)). Confirmou-se: Repeat
Zone no 4.0, For-Each-Element no 4.3 ([release notes 4.0](https://developer.blender.org/docs/release_notes/4.0/geometry_nodes/),
[4.3](https://developer.blender.org/docs/release_notes/4.3/geometry_nodes/)).

**(e) Críticas de UX documentadas** ([devtalk](https://devtalk.blender.org/t/simulation-nodes/29791)):
1. Inputs congelam após o frame 1 — "Even just a simulation zone that does nothing freezes the
   mesh… I would expect data outside the simulation zone to be fed into it as the simulation
   progresses" (Viktor_smg).
2. Atributos que não mudam precisam tunelar — "attributes can't 'skip' the simulation area…
   I have to unnecessarily pass it through" (komodo).
3. Sem neighbor queries, boids saem "improper" (DanielBystedt).
4. Low-level demais — "it lacks high-level building blocks… in many other DCCs building a
   particle system from nodes means manipulating velocities, positions and other attributes"
   (DamianWinnichenko).

## 3. Cavalry (o espírito-irmão do módulo Motion)

**(a) Ciclos?** Sem mecanismo algum nos docs; o modelo de conexão implica DAG: "An attribute can
have several outputs but can only have one input"
([Connections](https://cavalry.studio/docs/getting-started/key-concepts/connections)). Os docs
**nunca** apresentam feedback como padrão de usuário (ausência verificada, não só não-buscada).

**(b) Estado.** Escondido nos nós donos de sim, com caching explícito voltado ao usuário:
**Particle Shape** tem `Start Frame`, `Seed`, `Use Cache`/`Cache File Path`
([Particle Shape](https://cavalry.studio/docs/nodes/shapes/particle-shape/)); **Forge Dynamics**
tem botão **Cache Solver**, `.sdcache` em disco e **Cache Offset** para retiming
([Forge Dynamics](https://cavalry.studio/docs/nodes/shapes/forge-dynamics/forge-dynamics-shape/)).
Scrub é resolvido por BAKE, não por semântica de grafo.

**(c) O que o artista liga.** **Não há array "Forces" no emitter** (o Particle Emitter só tem
Rate/Duration/Initial Direction/Speed/Time/Seed —
[Particle Emitter](https://cavalry.studio/docs/nodes/utilities/particle-emitter/)). O hub é o
**Particle Shape**, com dois arrays: **`Emitters`** ("Connect Emitters to this attribute to
define where Particles will spawn from") e **`Modifiers`** ("Modifiers can be used to manipulate
the Particles movements and properties. They can be 'stacked' where each Modifier adds to the
one below it") — tipos: Data, Flow Field, **Force**, Goal, Image, JavaScript, Magnetic, Path,
Speed, **Turbulence**, Visual, **Vortex**. Gravity/Drag/Turbulence básicos são atributos do
próprio Particle Shape. No rigid-body, mesmo desenho: `Bodies` + `Fields` (Attractor Field,
Vortex Field, Buoyancy/Direction/Drag/Path Fields)
([Fields](https://cavalry.studio/docs/nodes/shapes/forge-dynamics/fields)).

**(d)** O Duplicator, em contraste, é stateless/procedural — só os nós de sim guardam estado;
todo o resto é função pura do Time ([Duplicator](https://cavalry.studio/docs/nodes/shapes/duplicator/)).

## 4. Autodesk Bifrost e Softimage ICE

**Bifrost — feedback ports (o precedente mais próximo do nosso `pre`-como-portal).** Sem ciclo
desenhado; primitivo de 1ª classe: "Feedback ports on compounds can be used to cache the result
of the last evaluation of a subgraph, and use it as the input for the next evaluation. This is
how simulations work… Feedback ports consist of a pair of input and output ports on a compound
that are matched so that the output value is used as the input the next time"
([custom sims](https://help.autodesk.com/cloudhelp/ENU/Bifrost-Common/files/build-a-graph/Bifrost_Common_build_a_graph_custom_sims_html.html)).
Par de portas **marcado na borda**, nunca laço de fio. Partículas: o artista nem toca nisso —
liga `source_particles → simulate_particles` e encadeia forças no port `influences`
("plug its output into the influences input of a simulate node" —
[wind_influence](https://help.autodesk.com/cloudhelp/2019/ENU/Bifrost-Common/files/reference/geometries/Bifrost_Common_reference_geometries_wind_influence_html.html)).

**Softimage ICE.** Acíclico ("Data flows 'downstream' from left to right" —
[framework](https://download.autodesk.com/global/docs/softimage2014/en_us/userguide/files/ICE_basics_UnderstandingtheICEFramework.htm));
estado = atributos do point cloud na região Simulation do stack ("calculations are based on the
previous frame"). Forças → hub **Add Forces** ("it adds up and blends the effect of all forces…
the order… is not important because they are simply added together") → port `Forces` do
**Simulate Particles**, que integra mass-aware
([ICE Forces](https://download.autodesk.com/global/docs/softimage2014/en_us/userguide/files/ice_simulation_ICEForces.htm)).

## 5. TouchDesigner, Niagara, Notch

- **TouchDesigner:** ciclo de fio = "cook dependency loop"; dev oficial: "things will randomly
  break depending on the cook order… You'll want to use the Feedback TOP and Feedback CHOPs"
  ([forum](https://forum.derivative.ca/t/cook-dependancy-loop-and-creating-feedback/186746)).
  **Feedback TOP/POP referenciam o alvo por PARÂMETRO** (path), servindo o frame anterior
  ([Feedback TOP](https://docs.derivative.ca/Feedback_TOP),
  [Feedback POP](https://docs.derivative.ca/Feedback_POP): Target é "a reference to a POP node
  downstream… whose data is retrieved one frame later"). O Feedback CHOP abençoa um laço
  desenhado só porque "It simply copies its input without cooking it first"
  ([Feedback CHOP](https://docs.derivative.ca/Feedback_CHOP)).
- **Unreal Niagara:** nem é grafo de fios — é **stack**: "simulation flows from the top of the
  stack to the bottom, and executes modules in order"
  ([key concepts](https://dev.epicgames.com/documentation/en-us/unreal-engine/key-concepts-in-niagara-effects-for-unreal-engine));
  estado no payload por-partícula (`Particles.*`); forças = módulos stock no **Particle Update**
  (Gravity, Curl Noise, Drag, Vortex, Wind…)
  ([Particle Update](https://dev.epicgames.com/documentation/en-us/unreal-engine/particle-update-group-reference-for-niagara-effects-in-unreal-engine)).
- **Notch:** hierarquia; o **Particle Root** possui o pool ("their positions, velocities,
  colours and so on, in a pool"); Affectors penduram no Root/Emitters
  ([Particles](https://manual.notch.one/2026.1/en/docs/reference/nodes/particles/)).

## 6. Grasshopper, Nuke/Fusion (DAGs estritos)

- **Grasshopper:** recusa na hora — "Recursive data stream found, this component depends on
  itself". Iteração = **Anemone** (bracket Loop Start/End: "Loop End sends data back to Loop
  Start", [food4rhino](https://www.food4rhino.com/en/app/anemone)); física = **Kangaroo2**, um
  solver-nó com goals declarativos e inputs `Reset`/`Restart`/`On`
  ([Solver](https://grasshopperdocs.com/components/kangaroo2/solver.html)); a variante
  "Zombie Solver" itera tudo internamente e cospe só o resultado.
- **Nuke:** "NUKE works on a 'pull' system" sobre um DAG
  ([NDK](https://learn.foundry.com/nuke/developers/63/ndkdevguide/intro/oparchitecture.html));
  acesso temporal por pull deslocado (FrameHold, TimeEcho), nunca estado no grafo.
- **Fusion:** "the output of the probe cannot be connected to the inputs of anything that feeds
  it. No feedback loops" (Bryan Ray, [blog](https://bryanray.name/2017/12/03/blackmagic-fusion-reflections/)).

## 7. O contra-exemplo: áudio/patching realtime (laço desenhado, com delay explícito)

- **vvvv:** "vvvv prevents you from creating such loops by simply not allowing you to make such
  connections" — EXCETO com um **FrameDelay** no laço: "its output is not depending on its input
  of the same frame but simply returns the value the input had in the last frame"
  ([Creating Feedback Loops](https://beta.vvvv.org/using-vvvv/patching/creating-feedback-loops.html)).
  → O FrameDelay é literalmente o nosso `pre`.
- **Pure Data:** "a DSP loop occurs when a patch needs information which is calculated inside
  the same block"; fix = delay de 1 bloco (`send~`/`receive~`)
  ([FAQ](https://puredata.info/docs/faq/dsp-loop)); erro clássico "DSP loop detected".
- **Max/gen~/RNBO:** feedback exige `tapin~`/`tapout~`; no gen~ "The history operator allows
  feedback… through the insertion of a single-sample delay"
  ([history](https://docs.cycling74.com/reference/gen_dsp_history/)); RNBO: "won't allow you to
  create a feedback connection… without the feedback~ object"
  ([RNBO delays](https://rnbo.cycling74.com/learn/delays-in-rnbo)).
- **Por que confunde:** a latência implícita depende de detalhes escondidos do scheduler (block
  size, até ordem de criação dos objetos). Áudio tolera um smear de 64 samples; motion graphics
  não tolera "este fio está secretamente 1 frame atrasado dependendo da ordem de avaliação".

## 8. Síntese (a que o doc 03 usa)

Padrão dominante para partículas-com-forças: **um nó-solver stateful com forças como
cadeia/hub/stack lateral plugada nele, num grafo que permanece DAG estrito** — POP Solver
(inputs verdes), ICE (Add Forces → Simulate Particles), Bifrost (`influences`), Cavalry
(`Modifiers`), Niagara (stack). O mecanismo GENÉRICO de frame-anterior, quando existe, é
feedback-por-referência ou par-de-portas marcado (TD, Bifrost), zona bracket (Blender, Anemone)
ou subnet com nó `Prev_Frame` (Houdini). **Fio de laço desenhado nunca é a superfície de autoria
de partículas** — só existe em áudio/patching, sempre através de um nó de delay explícito, e é
armadilha documentada. Custos conhecidos de cada lado: zonas → inputs congelados no frame 1 +
tunelamento de atributos + low-level demais sem nós densos; solvers opacos → precisam de
Reset/Restart/cache explícitos para scrub (Cavalry `.sdcache`). O desenho PH2D (emitter
stateless + estado no stream via `pre` determinístico + fingerprint por tick) evita ambos os
custos de scrub por construção.
