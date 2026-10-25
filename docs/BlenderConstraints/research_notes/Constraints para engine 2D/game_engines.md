# Constraints em game engines e physics engines (foco 2D) — insumo para o PH2D

> Pesquisa de 2026-10-25. Docs consultadas: Godot 4.7 (stable), Unity 6 / 2D Animation 10.1.4,
> Animation Rigging 1.x, rapier2d 0.36.1, Box2D v3 (docs atuais), UE5 (Epic docs).
> Nomes próprios em inglês; notas em PT-BR.

## 1. Catálogo de constraints de animação/transform — o que existe e o que é mais usado/mais fácil

### Takeaway
Todas as engines convergem num núcleo pequeno: **copiar transform (posição/rotação/escala/parent, com multi-fonte ponderada)**, **apontar (Aim/LookAt)**, **IK de dois ossos** e **IK de cadeia (CCD/FABRIK)**, mais **movimento secundário (damped/spring/jiggle)**. Para não-programadores, o modelo mais fácil é o de "behaviors" do Construct (Pin, Sine, Orbit, Tween, Move To): um componente = um comportamento com poucos parâmetros, sem grafo.

### Cited Findings

**Unity — Constraints (componentes nativos)**
- Seis tipos: Aim (roda para encarar a(s) fonte(s)), Look At (Aim simplificado), Parent (move+roda com a fonte), Position, Rotation, Scale — [Unity Manual: Constraints](https://docs.unity3d.com/Manual/Constraints.html)
- Lista de Sources com peso por fonte; com várias fontes usa a média ponderada; a ordem das fontes importa para Parent, Rotation e Aim, mas não para Position/Scale — [Unity Manual: Constraints](https://docs.unity3d.com/Manual/Constraints.html)
- Propriedades comuns: Weight (0 = sem efeito, 1 = total), At Rest (valores usados quando Weight=0 ou eixo não congelado), Offset, Freeze Axes, Is Active, Lock; botões **Activate** (grava o offset atual, ativa e trava) e **Zero** (reseta o transform para casar com as fontes, ativa e trava) — [Unity Manual: Constraints](https://docs.unity3d.com/Manual/Constraints.html)
- Ordem: constraints num mesmo GameObject avaliam do primeiro ao último no Inspector; clips de animação podem mover as fontes, e a constraint segue — [Unity Manual: Constraints](https://docs.unity3d.com/Manual/Constraints.html)

**Unity — Animation Rigging package**
- 12 constraints: Blend Constraint, Chain IK, Damped Transform, Multi-Aim, Multi-Parent, Multi-Position, Multi-Referential, Multi-Rotation, Override Transform, Twist Chain, Twist Correction, Two Bone IK — [Animation Rigging: Constraint Components](https://docs.unity3d.com/Packages/com.unity.animation.rigging@1.3/manual/ConstraintComponents.html)
- Baseadas na C# Animation Jobs API — [idem](https://docs.unity3d.com/Packages/com.unity.animation.rigging@1.3/manual/ConstraintComponents.html)
- O Rig Builder fica junto do Animator e cria um Playable Graph **anexado depois** da state machine do Animator (i.e., constraints rodam sobre a pose animada); Rig Layers empilham vários Rigs ligáveis/desligáveis a qualquer momento — [Rigging workflow 1.4](https://docs.unity3d.com/Packages/com.unity.animation.rigging@1.4/manual/RiggingWorkflow.html)
- Target e hint (joelho) são transforms filhos do rig element (ex.: Left Leg IK com Two Bone IK é pai de LeftFootEffector e LeftLegHint) — [Rigging workflow 1.4](https://docs.unity3d.com/Packages/com.unity.animation.rigging@1.4/manual/RiggingWorkflow.html)
- A posição na pilha muda o resultado: um "Shoulder Correction Rig" no topo da pilha do Rig Builder resolve certo; no fundo, a mão desalinha — [Unity Blog: advanced animation rigging](https://blog.unity.com/es/games/advanced-animation-rigging-character-and-props-interaction)
- Falhas silenciosas relatadas: sem Rig Builder, layer nulo, peso 0 ou fonte nula ⇒ constraint "configurada" que não faz nada (fonte de baixa autoridade) — [bugnet.io](https://bugnet.io/blog/fix-unity-animation-rigging-constraint-not-working)

**Unity — 2D IK (pacote 2D Animation 10.1.4)**
- IK Manager 2D no osso raiz controla todos os solvers; tem Weight mestre, Always Update (desligado = só resolve quando as Sprite Skins estão visíveis), Restore Default Pose — [Unity 2D IK](https://docs.unity3d.com/Packages/com.unity.2d.animation@10.1/manual/2DIK.html)
- **Ordem explícita**: solvers processados de cima para baixo na lista; o solver do tronco deve vir antes do braço; reordena-se arrastando — [Unity 2D IK](https://docs.unity3d.com/Packages/com.unity.2d.animation@10.1/manual/2DIK.html)
- Solvers: **Limb** (2 ossos, cadeia fixa de 3 com o effector), **Chain (CCD)** (melhora por iteração, tem Velocity), **Chain (FABRIK)** (menos iterações que CCD, mais lento por iteração com limites de rotação, adapta-se bem em tempo real) — [Unity 2D IK](https://docs.unity3d.com/Packages/com.unity.2d.animation@10.1/manual/2DIK.html)
- Propriedades: Effector, Target, Constrain Rotation (effector copia rotação do target), Restore Default Pose (resolve a partir da pose original vs. da atual), Weight, Chain Length, Iterations, Tolerance — [Unity 2D IK](https://docs.unity3d.com/Packages/com.unity.2d.animation@10.1/manual/2DIK.html)
- Gizmos de IK ligáveis no menu Gizmos da Scene view; cor e visibilidade por solver no Inspector do manager — [Unity 2D IK](https://docs.unity3d.com/Packages/com.unity.2d.animation@10.1/manual/2DIK.html)

**Unreal Engine 5**
- Constraints de animação divididas em Position, Rotation, Scale, Parent e LookAt; criadas no painel Animation (Animation Mode): seleciona-se o control, escolhe-se o tipo e o cursor vira **conta-gotas** para escolher o alvo no viewport; funcionam em qualquer Actor, não só Control Rig; a maioria adiciona **chaves de compensação** para manter a pose visual do filho ao ativar — [Epic: Animation Constraint Tools](https://dev.epicgames.com/documentation/unreal-engine/animation-constraint-tools-in-unreal-engine)
- API Python: `add_constraint(world, type, child, parent, maintain_offset)` e `bake_constraint` (assa a constraint em chaves num intervalo de frames) — [ControlRigSequencerLibrary](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/ControlRigSequencerLibrary)

**Godot 4 — 3D (SkeletonModifier3D) — contexto para o desenho**
- SkeletonModifier3D introduzido como base em 4.4; entre 4.4 e 4.6 vieram 19 modifiers novos — [Godot: IK returns in 4.6](https://godotengine.org/article/inverse-kinematics-returns-to-godot-4-6/)
- LookAtModifier3D (4.4) ≈ IK de 1 osso, exige definir o eixo do osso pelo rest; SpringBoneSimulator3D (4.5) monta a cadeia sozinho de root a end (compat. VRM); BoneConstraint3D (4.5) com CopyTransformModifier3D, AimModifier3D, ConvertTransformModifier3D; em 4.6 podem referenciar Node3D, não só ossos — [idem](https://godotengine.org/article/inverse-kinematics-returns-to-godot-4-6/)
- 4.6: IKModifier3D → TwoBoneIK3D, ChainIK3D → SplineIK3D, IterateIK3D → FABRIK3D/CCDIK3D/JacobianIK3D; separa o solver do "pós-processamento estético"; BoneTwistDisperser3D (distribui twist) e LimitAngularVelocityModifier3D — [idem](https://godotengine.org/article/inverse-kinematics-returns-to-godot-4-6/)
- Determinismo: TwoBoneIK3D e SplineIK3D sempre determinísticos; IterateIK3D com opção — sem ela carrega estado do frame anterior e converge com o tempo; com ela não carrega estado e com poucas iterações pode nunca chegar; IK determinística pode gerar velocidades angulares grandes ⇒ LimitAngularVelocityModifier3D suaviza — [idem](https://godotengine.org/article/inverse-kinematics-returns-to-godot-4-6/)
- Guia comunitário: TwoBoneIK3D para membros; FABRIK3D para caudas/tentáculos/coluna; CCDIK3D tolera algum esticamento, bom para rigs estilizados; Jacobian como último recurso — [gamefromscratch](https://gamefromscratch.com/inverse-kinematics-ik-return-to-godot/) (fonte secundária)

**Godot 4 — 2D**
- `SkeletonModification2D` continua **Experimental** (não deprecated) na doc 4.7: "may be changed or removed"; subclasses: CCDIK, FABRIK, Jiggle, LookAt, PhysicalBones, StackHolder, TwoBoneIK; propriedades `enabled`, `execution_mode`; pertence a um SkeletonModificationStack2D — [Godot 4.7: SkeletonModification2D](https://docs.godotengine.org/en/stable/classes/class_skeletonmodification2d.html)
- Em 2022 a pilha 2D foi marcada experimental no mesmo lote em que os equivalentes 3D (SkeletonModificationStack3D, SkeletonIK3D) foram deprecated — [mirror de commit (terceiro)](https://git.objectionable.solutions/Sara/behaviour-tree-test/src/commit/0dab11afa4dca6bba7aba554ec4097de30200f73/doc/classes/SkeletonModification2D.xml); o artigo oficial de 4.6 só cobre 3D — [Godot blog](https://godotengine.org/article/inverse-kinematics-returns-to-godot-4-6/)
- RemoteTransform2D: copia o próprio transform para outro Node2D (`remote_path`), com `update_position/rotation/scale` e `use_global_coordinates`; cacheia a referência (`force_update_cache()`), e o caminho relativo quebra se os nós mudam de lugar na árvore — [Godot 4.7: RemoteTransform2D](https://docs.godotengine.org/en/stable/classes/class_remotetransform2d.html)

**Construct (modelo "behaviors")**
- Pin (Construct 2 manual) tem modos: Position & angle (padrão), Position only, Angle only, **Rope style** (distância máxima, sem rodar), **Bar style** (distância fixa, sem rodar) — [Construct 2 manual: Pin](https://www.construct.net/en/construct-2/manuals/construct-2/behavior-reference/pin)
- No Construct 3 o Pin foi quase reescrito: pinar a image point, a tamanho e a Z elevation — [Construct 3 release r193/r197 notes](https://construct.net/make-games/releases/stable/r197) (lista exata de modos do C3 não confirmada)
- Orbit: move em círculo/elipse em torno de um ponto ou de outro objeto (seguindo-o se ele se mover); velocidade e raio em px/graus — [Construct: Orbit](https://www.construct.net/animation-software/manual/behavior-reference/orbit)
- Move To (r166): move para uma posição com aceleração, velocidade máxima e desaceleração; waypoints e caminho de Pathfinding/timeline; diferente do Tween, que anima um parâmetro (ex. opacidade) ou um valor — [Construct r166 beta](https://www.construct.net/make-games/releases/beta/r166)
- Conflito entre behaviors: Sine "para de funcionar" quando o objeto está pinado/seguindo outro, porque o follow sobrescreve a posição — [Construct forum (via busca)](https://www.construct.net/en/forum/construct-3/how-do-i-8/tween-orbit-184611)

### Inferences
- O núcleo mínimo para o PH2D: CopyTransform (pos/rot/scale com máscara por eixo e multi-fonte ponderada), Parent (com "maintain offset" = botão Activate do Unity), Aim/LookAt 2D (só ângulo), TwoBoneIK 2D (com bend direction em vez de hint 3D), ChainIK (FABRIK/CCD), Damped/Spring (jiggle), Path follow, e behaviors procedurais tipo Sine/Orbit/Tween.
- Em 2D, IK de dois ossos tem só duas soluções (cotovelo para um lado ou outro) ⇒ um booleano "flip/bend direction" substitui o pole vector; isso é o principal anti-"IK flip" em 2D.
- O modelo Construct (componente-comportamento com poucos parâmetros) mapeia bem para ECS (cada behavior = component + system), e o conflito "Sine vs Pin" mostra que é preciso ordem/composição explícita quando dois escrevem o mesmo canal.
- O "Activate/Zero" do Unity e as chaves de compensação do Unreal resolvem o mesmo problema: ligar uma constraint sem a peça saltar.

### Gaps
- Não encontrei a página oficial atual do Construct 3 para Pin, Sine, Follow (403 no índice de behaviors).
- GameMaker e Defold: não pesquisados a fundo; Defold não tem sistema de behaviors nativo (conhecimento prévio, não verificado nesta sessão).
- Unity LimbSolver2D "flip": não confirmado na doc consultada.
- Unity Sprite Skin: não consultado nesta sessão.
- Unreal IK Rig / Retargeter / Full Body IK e Godot PathFollow2D: não buscados (tempo); são bem conhecidos, mas sem citação aqui.
- Ranking quantitativo de "mais usados" não existe em fonte confiável; a inferência vem da convergência das engines.

## 2. Juntas de física e como os editores as expõem

### Takeaway
Box2D v3, rapier2d, Unity 2D e Godot 2D oferecem o mesmo conjunto: revolute/pin (com limite angular + motor), prismatic/slider/groove, distance/spring (comprimento min/max, hertz + damping ratio), weld/fixed, wheel, motor/relative, e um "arraste do rato" (target/mouse — em Box2D v3 feito com corpo kinemático + motor joint). Editores expõem âncoras como pontos arrastáveis com "auto-configure connected anchor", limites como arcos, e quebra por limiar de força/torque.

### Cited Findings

**Box2D v3**
- Tipos: distance, filter, revolute, prismatic, weld, motor, wheel; **não há mouse joint dedicado** — os samples usam um corpo kinemático no cursor ligado por motor joint com mola linear — [Box2D v3: Simulation](https://box2d.org/documentation/md_simulation.html)
- Distance: `length`, `enableSpring`, `hertz`, `dampingRatio` (1 = amortecimento crítico); min/max length existem (o "rope" virou distance com max) — [idem](https://box2d.org/documentation/md_simulation.html)
- Revolute: `enableLimit`, `lowerAngle`, `upperAngle`, `enableMotor`, `motorSpeed`, `maxMotorTorque`; Prismatic: `lower/upperTranslation`, `maxMotorForce`; Wheel: mola de suspensão + motor; Motor joint: mola linear e angular com força/torque máximos; Filter joint: só desliga colisão entre os dois corpos — [idem](https://box2d.org/documentation/md_simulation.html)
- Dicas de estabilidade: hertz da mola < metade da frequência do passo (< 30 Hz a 60 Hz); ganho do motor moderado; âncoras prismáticas perto dos centros de massa; cadeias de weld flexionam (solver aproximado); mais sub-steps = cadeias longas esticam menos — [idem](https://box2d.org/documentation/md_simulation.html)
- Quebra: `b2Joint_GetConstraintForce/Torque`; `forceThreshold`/`torqueThreshold` na definição + `b2World_GetJointEvents` após o step, depois `b2DestroyJoint` — [idem](https://box2d.org/documentation/md_simulation.html)
- `collideConnected` padrão false; juntas entre corpos static/kinematic são permitidas mas sem efeito — [idem](https://box2d.org/documentation/md_simulation.html)
- Solver "Soft Step" = soft constraints + sub-stepping, mais estável que 2.4 em razões de massa altas, cadeias longas e pilhas; springs adicionados a muitas juntas e motor na distance joint — [Releasing Box2D 3.0](https://box2d.org/posts/2024/08/releasing-box2d-3.0); fundamento em [Catto, Soft Constraints GDC 2011](https://box2d.org/files/ErinCatto_SoftConstraints_GDC2011.pdf)

**rapier2d 0.36.1**
- Structs: RevoluteJoint, PrismaticJoint, FixedJoint, RopeJoint, SpringJoint, PinSlotJoint ("groove" do Godot), GenericJoint, ImpulseJoint, MultibodyJoint, JointAxesMask, JointLimits, JointMotor; MotorModel decide se stiffness/damping dependem da massa — [docs.rs rapier2d::dynamics](https://docs.rs/rapier2d/latest/rapier2d/dynamics/index.html)
- Em 2D revolute = spherical; Prismatic com `limits([min,max])`; motores PD: `configure_motor_position(target, stiffness, damping)`, `configure_motor_velocity`, `configure_motor`, `configure_motor_model` — [rapier: Joints](https://rapier.rs/docs/user_guides/rust/joints)
- Vários colliders num corpo é mais eficiente e estável que fixed joint; use fixed só quando precisa ler a força para quebrar — [rapier: Joints](https://rapier.rs/docs/user_guides/rust/joints)
- Impulse joints: qualquer grafo (laços ok), add/remove rápido, forças recuperáveis, mas podem ser violados se o solver não convergir e montagens grandes quebram com poucos iterações/passo grande. Multibody (coordenadas reduzidas): não violáveis, estáveis com passo grande, mas só árvore (`insert_multibody_joint` devolve `None` se fecha laço), add/remove lento, sem forças. Recomendação: impulse para jogos/animação; combinar (árvore multibody + laços como impulse) — [rapier: Joint constraints](https://rapier.rs/docs/user_guides/rust/joint_constraints)

**Unity 2D Joints**
- Só ligam a um Rigidbody 2D ou a um ponto fixo no mundo; Hinge (pivô, ex. tesoura), Slider (linha, com motor e limites de translação), Wheel (roda com mola de suspensão), Friction (reduz velocidade relativa), Relative (mantém offset de posição e ângulo), Target (mola até um alvo, ex. pegar e arrastar sob gravidade); também Spring, Distance, Fixed — [Unity Manual: Joints 2D](https://docs.unity3d.com/2023.2/Documentation/Manual/Joints2D.html)
- Hinge Joint 2D: Enable Collision, Connected Rigidbody (None = ponto fixo), **Auto Configure Connected Anchor**, Anchor, Connected Anchor, Use Motor (Motor Speed em graus/s, Maximum Motor Force), Use Limits (Lower/Upper Angle), **Break Action**, Break Force, Break Torque (padrão Infinity) — [Unity: Hinge Joint 2D](https://docs.unity3d.com/Manual/2d-physics/joints/hinge-joint-2d-reference.html)

**Godot 4 2D Joints**
- Joint2D base: `bias` (rapidez de reposição; menor = mais folga), `disable_collision`, node_a/node_b — [Godot: Joint2D](https://docs.godotengine.org/en/4.2/classes/class_joint2d.html)
- PinJoint2D (4.7): `softness`, `angular_limit_enabled/lower/upper`, `motor_enabled`, `motor_target_velocity` (rad/s) — [Godot 4.7: PinJoint2D](https://docs.godotengine.org/en/stable/classes/class_pinjoint2d.html)
- DampedSpringJoint2D: `length` (máximo), `rest_length`, `stiffness`, `damping` 0..1; GrooveJoint2D: movimento linear até distância máxima — [Godot: DampedSpringJoint2D](https://godot.readthedocs.io/en/4.1/classes/class_dampedspringjoint2d.html); [KidsCanCode recipe](https://kidscancode.org/godot_recipes/3.x/physics/joints_2d/index.html)

### Inferences
- rapier2d (já no PH2D) cobre todos os casos: revolute(=pin/hinge), prismatic(=slider), PinSlot(=groove/wheel sem mola → wheel = PinSlot + motor + spring via GenericJoint), Rope(=distance max), Spring, Fixed(=weld); Motor/Relative/Target podem ser GenericJoint com motores ou corpo kinemático + junta, como Box2D faz o mouse.
- Para editor: as fontes convergem em (a) âncora no corpo A + âncora conectada com "auto-configure", (b) limites angulares como arco desenhado, (c) parâmetros de mola em hertz + damping ratio (independentes de massa, mais intuitivos que stiffness bruta — mapear para MotorModel mass-independent do rapier), (d) Break force/torque com ação.
- Debug: desenhar juntas, eixos e limites; mostrar força de reação ao vivo (rapier impulse joints a expõem) ajuda a calibrar thresholds de quebra.

### Gaps
- Não encontrei doc oficial sobre o UX visual de gizmos de juntas 2D no Unity (a página de referência só cobre Inspector).
- Detalhes de min/max length e softness de juntas não-distance em Box2D v3 não estavam no trecho lido; ver API reference.
- Unity Spring/Distance/Friction/Relative/Target/Fixed: propriedades detalhadas não lidas.

## 3. Ordem de avaliação, blend, física × animação e problemas recorrentes

### Takeaway
Problemas recorrentes: (1) **ordem de avaliação** (constraint antes/depois da animação ou de outra constraint — resolvido com pilha explícita e ordenável), (2) **jitter/esticamento de juntas** (sub-steps, mais iterações, razões de massa, nunca teleportar corpo kinemático), (3) **IK flip/velocidade angular alta** (bend direction, limites, limitador de velocidade angular), (4) **salto ao ativar** (maintain offset / chaves de compensação), (5) **falha silenciosa** (peso 0, alvo nulo).

### Cited Findings
- Animation Rigging corre como Playable Graph **depois** do Animator; Rig Layers ponderáveis e ligáveis — [Rigging workflow](https://docs.unity3d.com/Packages/com.unity.animation.rigging@1.4/manual/RiggingWorkflow.html)
- Unity 2D IK: lista ordenada de solvers, pais antes de filhos — [Unity 2D IK](https://docs.unity3d.com/Packages/com.unity.2d.animation@10.1/manual/2DIK.html)
- Unity Constraints: ordem do Inspector; Weight + At Rest — [Unity Constraints](https://docs.unity3d.com/Manual/Constraints.html)
- Godot: IK iterativa com estado entre frames converge com o tempo; versão sem estado pode não chegar; IK determinística gera picos de velocidade angular → LimitAngularVelocityModifier3D — [Godot 4.6 IK](https://godotengine.org/article/inverse-kinematics-returns-to-godot-4-6/)
- Unity ragdoll stability: esticamento quando o solver não segura ⇒ projection; jitter ⇒ solver iterations 10–20; razão de massa > 10× ⇒ jitter; **nunca mover Rigidbody kinemático ligado a juntas por acesso direto ao Transform**; limites angulares mínimos ~5–15° (0 trava o eixo) — [Unity 6.4: Ragdoll stability](https://docs.unity3d.com/Manual/RagdollStability.html)
- Relato: cauda de juntas presa a corpo kinemático estica proporcionalmente à velocidade do personagem; trocar o tipo de junta não resolveu — [Unity Discussions: Joint stretching](https://discussions.unity.com/t/joint-stretching-problem/380770)
- Box2D: sub-steps reduzem esticamento de cadeias; hertz < metade da frequência do passo — [Box2D v3 docs](https://box2d.org/documentation/md_simulation.html)
- rapier: multibody não viola junta (bom para cadeias articuladas), impulse pode explodir com passo grande — [rapier: Joint constraints](https://rapier.rs/docs/user_guides/rust/joint_constraints)
- Unreal: constraints criam chaves de compensação e podem ser **assadas** para chaves (`bake_constraint`) — [Epic docs](https://dev.epicgames.com/documentation/unreal-engine/animation-constraint-tools-in-unreal-engine); [Python API](https://dev.epicgames.com/documentation/en-us/unreal-engine/python-api/class/ControlRigSequencerLibrary)
- Construct: dois behaviors a escrever posição (Sine + Pin) — o último ganha, o outro "não funciona" — [Construct forum](https://www.construct.net/en/forum/construct-3/how-do-i-8/tween-orbit-184611)

### Inferences
- Pipeline sugerido para o PH2D (ECS): animação/keys → pilha de constraints de transform (ordem explícita, peso por constraint, mistura com a pose de entrada) → IK → secundário (spring/jiggle, com estado) → escrita para física: corpos *kinematic* recebem `set_next_kinematic_position` (nunca teleporte), corpos *dynamic* são guiados por juntas/motores → step do rapier → leitura de volta para ossos com "physical bones" ponderados.
- Determinismo (lei "mesmo bit" do PH2D): preferir solvers analíticos (TwoBone, Aim) e iterativos sem estado entre frames no scrub da timeline; estado (spring/jiggle/IK com memória) só no modo simulação, com reset no seek.
- "Bake to keys" (Unreal) é a válvula para exportar/editar à mão depois da constraint.
- Gizmos + avisos de "constraint inerte" (peso 0, alvo nulo, ciclo de dependência) atacam a falha silenciosa reportada.

### Gaps
- Não achei fonte sólida sobre foot placement 2D em engines (só 3D).
- Não há dados quantitativos de dor de UX; os relatos são de fóruns e blogs de baixa autoridade.
- Ciclos de dependência entre constraints (A segue B, B segue A): nenhuma das docs lidas descreve como detectam; a verificar.
