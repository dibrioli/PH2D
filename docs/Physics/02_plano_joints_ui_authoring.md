# 02 — Joints: UI, autoria de âncoras e os tipos que faltam (plano pós-pesquisa)

> **Origem:** ordem do Enio (2026-07-25, pós-smoke do W-JointParams): *"pesquisa na UI e no modo de criar
> as âncoras… Unity e Unreal, mas não só… apps mais próximos de nós com Rust… descubra as reclamações e
> sugestões dos usuários de grandes apps e tente criar um sistema de criação de juntas melhor que os
> existentes… investigue os demais tipos de junta que Rust suporta nativamente."*
>
> **Método:** 5 agentes de pesquisa leram os manuais oficiais (Unity 2D Joints · Unreal Physics Constraints
> · Godot Joint2D · Fyrox book · RUBE manual · Algodoo · Newton 3 · Moho · Construct 3 · GDevelop), baixaram
> **44 screenshots verificados** para `~/Documentos/Recursos/UI_Reference/` e colheram **28 reclamações/
> elogios com URL**. A superfície nativa do rapier foi lida do **source local**
> (`~/.cargo/registry/src/…/rapier2d-0.28.0/src/dynamics/joint/`), não de docs da web. Cada imagem foi
> **aberta e lida uma a uma** antes deste plano (§2).
>
> **Este plano ABSORVE as "waves 2-5 do padrão-ouro"** listadas no tracker (grupo carrega o rig · 2 alças +
> snap · limite/motor visuais · break force) — o mapa §7 diz onde cada uma pousou. O tracker aponta pra cá.

---

## §1 — Estado atual (honesto, 2026-07-25)

O que JÁ temos — e que a pesquisa validou como acima da média:

- **Joint é ENTIDADE** (W3): Hierarquia, seleção, nome, delete, undo, save de graça. A pesquisa confirmou
  que este é o modelo vencedor: Godot (joint-como-nó), avian (joint-como-entidade), e é o que o Unity NÃO
  tem (componente assimétrico no corpo A; o corpo B nunca sabe que está jointado).
- **Corpos nomeados, nunca apontados por bits** (`stable_name_id`) + **eyedropper de re-pick por ponta**
  (W-JointAuthoring) com "(missing)" por slot — o RUBE tem o equivalente (Body A/B são BOTÕES de re-pick) e
  o Unreal falha exatamente onde não tem isso (binding por STRING que falha em silêncio, reclamação E2/E3).
- **Âncora body-local semeada do REPOUSO, re-seed por GESTO explícito** (W-AnchorFollow `anchored=false`):
  a pesquisa provou que este é o desenho certo — o "Auto Configure Connected Anchor" do Unity é um MODO
  persistente que silenciosamente possui um campo editável-na-aparência, e gera a maior classe de
  reclamação deles (E1/E6/E7 do Unity: re-deriva sozinho em runtime, fica stale no editor, esconde o gizmo
  da âncora auto). O Fyrox iterou 3× no CHANGELOG até chegar no MESMO modelo que o nosso.
- **Dot de âncora arrastável** (W-JointAnchor), **"Join Selected Bodies" + "Join As"** (W-JointCreate),
  **params vivos mid-play** (W-JointParams), **auto-seleção do joint criado**, jointed não colidem por
  default (o default que Unity/Box2D/RUBE shipam; Unreal shipa o oposto e cobra caro).

As limitações que o Enio nomeou ("grandes limitações que precisam ser totalmente sanadas"), agora com o
diagnóstico da pesquisa:

| # | Limitação | O que a pesquisa diz |
|---|---|---|
| L1 | **O joint quase não se DESENHA** — segmento + anéis âmbar e só; limites, motor, rest length, max length são números cegos no §12 | RUBE desenha TODO fato como geometria (glifo por tipo, linhas de posse verde/azul, arco de limites, seta de eixo, violação em vermelho). Godot/Fyrox não desenham nada e são cobrados por isso (proposals #7430, #84468) |
| L2 | **Uma alça só** (âncora A); a âncora B não é agarrável; zero snap | Newton: alças nas duas pontas + snap em vértice do contorno (Ctrl) e centro de massa (X). RUBE: 'T' alterna qual lado move |
| L3 | **Criar exige selecionar 2 ANTES de mirar** — a âncora nasce na pose relativa, nunca onde o artista aponta | A síntese da pesquisa inteira: RUBE cria a âncora NO CURSOR ("don't move the cursor just yet!"); Algodoo attach-on-place (a pilha sob o clique É o par). Nenhuma das duas escolas escolhe corpos antes de mirar |
| L4 | **Faltam tipos que o rapier tem prontos** | Prismatic/slider inteiro; servo (motor de posição); motor na rope (guincho); ver §4 |
| L5 | **Limites se digitam em graus**, sem preview | RUBE: pressiona 'L' e POSA o próprio corpo B no limite com o mouse. Unreal desenha o cone mas não deixa arrastar (readout, não handle) — dá pra ser melhor que os dois |
| L6 | **Sem break force** | Unity: Break Force/Torque com default Infinity=off + Break Action dropdown. Godot nem lê a força de reação (proposal #7672 pede). rapier expõe `ImpulseJoint.impulses` público — construível no padrão do W-ImpactForce |
| L7 | **Sem higiene do par**: desabilitar sem deletar, collide-connected por joint, swap A↔B | `JointEnabled` e `contacts_enabled` são NATIVOS do rapier e não têm row; RUBE tem swap; Newton tem "Active" checkbox (keyframável!) |

---

## §2 — As imagens, uma a uma (o que cada uma ensina)

Pasta: `~/Documentos/Recursos/UI_Reference/` — 44 arquivos, todos verificados imagem real.

### Unity (9 inspectors oficiais, docs 2022.3)

| Arquivo | O que mostra | Lição |
|---|---|---|
| `unity_hingejoint2d_inspector.png` | Hinge: botão **"Edit Joint Angular Limits"** com ícone no TOPO; Enable Collision 1ª row; Motor/Angle Limits como foldouts; Break Action/Force/Torque como bloco final (Infinity) | O único gizmo editável deles é um MODO que se entra por botão — e some se o toggle global de Gizmos estiver off (reclamação U3). Nosso: alça sempre-viva, sem modo |
| `unity_springjoint2d_inspector.png` | Spring: Auto Configure Distance ON, Distance auto-derivada (0.005), Damping Ratio + **Frequency** (Hz) | Eles falam frequência (Hz), nós stiffness — artista entende "mais duro/mais mole"; manter stiffness mas com gizmo (anel de rest) |
| `unity_distancejoint2d_inspector.png` | Distance: checkbox **"Max Distance Only"** | Corda e barra-rígida são UM tipo com um toggle — opção elegante p/ nossa Rope ganhar modo "Rod" (min=max, expressável no rapier) |
| `unity_sliderjoint2d_inspector.png` | Slider: Auto Configure Angle, **Angle** = eixo, Motor (m/s), Translation Limits | O slider deles pede um ÂNGULO digitado; o nosso eixo pode ser a ROTAÇÃO da entidade-joint (gesto que já existe) — zero campo novo |
| `unity_wheeljoint2d_inspector.png` | Wheel: Suspension (Damping/Frequency/Angle) + Motor | Wheel = prismatic+spring+motor num pacote; p/ nós é PRESET futuro, não tipo novo |
| `unity_fixedjoint2d_inspector.png` | Fixed: **Damping Ratio + Frequency num WELD** | O weld deles é mola dura (tunável); o nosso é rígido (rapier FixedJoint). Anotar como variante futura "soft weld", não default |
| `unity_relativejoint2d_inspector.png` | Relative: Max Force/Torque, **Correction Scale 0.3** ("tweak to correct its behavior") | Knob-fudge documentado como "ajuste até parecer certo" — exatamente o que [feedback_ergonomics] proíbe. Anti-exemplo |
| `unity_targetjoint2d_inspector.png` | Target: Anchor + **Target em MUNDO**, sem Connected Body | O joint de "carregar com o mouse". Para nós: interação de PLAY futura, não autoria |
| `unity_frictionjoint2d_inspector.png` | Friction: Max Force/Max Torque | Freio relativo entre 2 corpos; nicho — vocabulário §4, sem wave |

### Unreal (8)

| Arquivo | O que mostra | Lição |
|---|---|---|
| `unreal_constraint_details_constraint_section.png` | Constraint Actor 1/2 com dropdown + browse + **eyedropper**; Component Name 1/2 como STRING | O eyedropper deles valida o NOSSO; a string por baixo é o anti-padrão (falha em silêncio) |
| `unreal_constraint_actor_details_panel.png` | O mesmo painel no fluxo de nível, com Pos 1/Pos 2 numéricos | Âncoras como pares X/Y crus no fim do painel — ninguém arrasta |
| `unreal_constraint_component_name_typing.png` | Digitando "StableMesh" no Component Name; a única confirmação é um bounding box no viewport | Typo = constraint ligado a NADA, sem erro. Nosso "(missing)" por ponta é a resposta certa |
| `unreal_constraint_cone_viewport.png` | O cone/arco de swing desenhado sob o cubo constrangido | Eles DESENHAM limites (melhor que Unity/Godot) — mas é readout: não se arrasta |
| `unreal_phat_constraint_limits_viewport.png` | PhAT: cadeia com cones de swing verdes + arcos de twist laranja/azul, selecionado destacado | O vocabulário visual de limites mais rico da pesquisa — e ainda assim sem handle |
| `unreal_phat_constraints_graph.png` | Grafo: Body → nós "child : parent" → Bodies | "Quem liga quem" como GRAFO nomeado — nosso nome auto de joint deve ser "A : B" |
| `unreal_phat_pin_dragging_create_constraint.png` | Criar constraint ARRASTANDO do pino de um body p/ lista buscável | Criação por arrasto A→alvo — versão em grafo do nosso gesto de canvas proposto (W-J4) |
| `unreal_phat_body_context_menu.png` | Menu: **"Constraint selected bodies" Ctrl+Y**, Copy/Paste constraints, Mirror | O nosso "Join Selected Bodies" é este idioma; copy/paste de PROPRIEDADES de joint é ideia barata e boa |

### Godot + Fyrox (8)

| Arquivo | O que mostra | Lição |
|---|---|---|
| `godot_issue85144_groovejoint2d_editor_gizmo.png` | Editor inteiro: o gizmo do GrooveJoint é UMA linha amarela fina, quase invisível | O chão da concorrência: joint sem corpo visual vira bug report (o issue é exatamente "mexi no length e nada mudou") |
| `godot_issue91691_pinjoint2d_cross_gizmo_editor.png` | PinJoint = cruzinha amarela 20×20 px | Idem — e os limites desse pin NÃO FUNCIONAM (bug #91691 aberto) |
| `godot_kidscancode_joints_demo_scenes.png` | As 3 demos (Pin/DampedSpring/Groove) | O conjunto 2D inteiro do Godot são 3 tipos; rapier nos dá 6+ |
| `godot_kidscancode_pinjoint_example.gif` | Pin girando | Referência de comportamento, nada de UI |
| `godot_kidscancode_springjoint_example.gif` | Spring oscilando | Idem |
| `fyrox_ragdoll_wizard_joint_creation.png` | **Ragdoll Wizard**: tabela osso→nó, Total Mass/Friction/CCD, grades de collision/solver groups, e a árvore com dezenas de `Ragdoll*BallJoint/*HingeJoint` gerados | Criação EM MASSA por wizard — o único app da pesquisa com resposta pra "quero 40 joints de uma vez". Horizonte nosso (§8) |
| `fyrox_ragdoll_result_editor.png` | A árvore corpo+collider por osso | Convenção de nomes gerados — o nosso auto-nome "A : B" cobre o caso unitário |
| `fyrox_editor_physics_colliders_scene.png` | Colliders têm wireframe; joints NÃO têm nada | O editor rapier-nativo de referência não desenha joints — o vácuo que podemos ocupar |

### RUBE + Algodoo (11) — a escola da manipulação direta

| Arquivo | O que mostra | Lição |
|---|---|---|
| `rube_action_menu_add_joint.png` | Spacebar → Add joint → Revolute/Prismatic/Distance/Wheel/Rope/Weld/Friction/Motor | Menu no CURSOR: o lugar do clique é dado do gesto |
| `rube_revolute_creation_at_cursor.png` | Revolute recém-criado NO cursor sobre o centro da roda | **A âncora nasce onde se aponta** — a regra número 1 do plano |
| `rube_joint_nub_types.png` | Glifo distinto por tipo (curl, quadrados no eixo, barra, hachura de corda) | Vocabulário de glifos por tipo — nosso W-J1 |
| `rube_joint_bodyA_bodyB_lines.png` | Linha tracejada VERDE→corpo A, AZUL→corpo B | Posse sempre visível; resolve "qual joint liga o quê" sem abrir painel |
| `rube_joint_disjointed_anchors.png` | Linha VERMELHA quando as âncoras estão separadas (violação) | Estado de violação é DESENHADO, não silenciosamente resolvido |
| `rube_revolute_limit_visual_editing.png` | **'L': o próprio corpo B gira com o mouse para POSAR lower/upper limit** (arco amarelo + linha verde do ângulo vivo) | A killer feature da pesquisa. Limite não se digita; se DEMONSTRA. Nosso W-J3 |
| `rube_prismatic_axis_rotated.png` | Eixo do prismatic = SETA tracejada; limites = tracinhos perpendiculares | O desenho canônico do slider — copiar no W-J5 |
| `rube_make_chain_result.png` | Make chain: elo-protótipo replicado N vezes | Corrente por protótipo — nosso §8 (com o Newton cobrindo o caso simples antes) |
| `rube_properties_panel.png` | Painel F7: rows VERDES=estáticas, VERMELHAS=dinâmicas (mudam na sim) | Distinção visual autoria×runtime — ideia fina p/ nosso §12 (rows de readout tingidas) |
| `algodoo_axle_rightclick_motor_sliders.png` | Painel Axle: Motor ✓ → **Motor speed 15.0 rpm + Motor torque 100 Nm** (sliders com UNIDADE), Reversed/Brake, teclas Forward/Back/Brake, **Break limit ∞ Ns** | Unidades físicas em TODO knob; break com ∞=off; motor dirigível por tecla (play interativo) |
| `algodoo_axles_placed_on_wheel_centers.png` | As rosetas de rolamento NOS centros + cunha escura de direção de giro | O glifo do eixo é legível a metros; motor ligado = setas de giro no próprio glifo |

### Newton 3 + Moho + Construct + GDevelop (8) — o nosso domínio

| Arquivo | O que mostra | Lição |
|---|---|---|
| `newton_ui_overview.png` | A janela inteira: lista de Joints (tabela `# / Type / #A / Body A / #B / Body B`), **Joint Properties em ABAS por tipo** (Distance/Pivot/Piston/Spring/Wheel/Blob), cada row com losango de KEYFRAME + randomize, **Collide Connected ✓ e Active ✓ visíveis**, mini curve editor, Export | O app que motion designers elogiam: seleciona 2 corpos → 1 botão por tipo → âncoras arrastáveis. E: **propriedades de joint são keyframáveis** — para uma suíte de ANIMAÇÃO, é o horizonte certo (§8) |
| `newton_joint_buttons.png` | O header do painel Joints: 6 ícones, um por tipo | Criação = 1 clique por tipo, sem menu fundo. Nosso "Join As" + botão já é isso; o gesto de canvas (W-J4) vai além |
| `newton_anchor_distance.gif` | Âncoras de distance sendo ARRASTADAS ao vivo no viewport | As duas pontas agarráveis — nosso W-J2 |
| `newton_export_panel.png` | Export: range + Render, progresso "Keyframing… 17/32" | O bake deles é o nosso (chaves e nada mais) — já temos melhor (in-scene, range, canais) |
| `moho_layer_settings_physics_tab.png` | A aba Physics INTEIRA do Moho: Enable + gravidade + "Use baked physics" | O concorrente direto de animação 2D **não tem joints** — física é flag de camada. Espaço aberto |
| `moho_bone_physics_tool_options.png` | Bone Physics: region, Motor speed/torque, **Lock Tip** | O "pin joint" deles é um checkbox num osso; reclamações: on/off, playback não-determinístico, sem mixagem com animação à mão — as 3 feridas que nosso toggle Physics + replay bit-exato + bake já curam |
| `construct_add_action_physics_joints.png` | "Add action": Create distance/revolute/limited revolute joint… | Joints por EVENTO, invisíveis no editor ("the connecting pole is not shown!") — anti-exemplo |
| `gdevelop_physics2_behavior_properties.png` | O painel de física do GDevelop: só CORPO (Type/Bullet/Shape) | 11 tipos de joint… todos sem UI. A demanda existe (issue #7295); a oferta visual, não |

---

## §3 — Reclamações dos usuários → nossa resposta

As 28 fontes completas ficam nos relatórios; aqui, cada CLASSE com a resposta deste plano:

| Reclamação (app, fonte) | Nossa resposta |
|---|---|
| Auto-anchor re-deriva sozinho / fica stale / esconde gizmo (Unity discussions 1199008, 1621185, tracker "one anchor gizmo") | **Lei já nossa, agora pinada como princípio P6:** re-seed é GESTO one-shot (`anchored=false`), nunca modo. Nenhuma wave introduz modo persistente que possua campo |
| Espaços de âncora confundem (local×mundo muda com outro campo) (Unity 239431) | Âncora se autora ARRASTANDO no mundo; o local é derivado por porta única. Números no §12 sempre em MUNDO na fronteira do painel (como o °/rad já faz) |
| Gizmo de edição some por toggle não-relacionado (Unity 770824) | Alças sempre-vivas quando o joint está selecionado; sem modo "Edit Limits", sem dependência de flag global |
| "Cadê o hinge?" — 6-DOF cru sem tipos nomeados (Unreal 103357) | Tipos NOMEADOS continuam sendo a porta (Pin/Spring/Rope/Weld/Slider); Generic/Custom só como horizonte §8 |
| Binding por string falha em silêncio (Unreal 37083, 304263) | Já resolvido (eyedropper + nome vigente + "(missing)" por ponta). O plano não regride isso |
| Limites simétricos-only, "gire o frame inteiro" como workaround (Unreal 441151) | Nossos limites já são min/max ASSIMÉTRICOS (rapier). O W-J3 os torna arrastáveis por ponta — cada ponta do arco é sua |
| NodePaths opacos, "de quem é o começo do fio?" (godot-proposals 5778) | Linhas de posse A/B coloridas no canvas (W-J1) + rows Body A/B com nome vigente (já temos) |
| Softness/bias sem unidade, dependentes de timestep (godot-proposals 15126) | Todo knob com UNIDADE no label (já é §12; P5 exige nas waves novas: N·m, °/s, m, N·s) |
| Sem leitura de força de reação → sem breakable (godot-proposals 7672) | rapier expõe `impulses`; W-J7 constrói break com pico por-substep (padrão W-ImpactForce) |
| Sem gizmo de limites / gizmo não bate com runtime (godot 7430, 84468) | W-J1 desenha do MESMO `desc` que o solver consome (porta única — a lei do `scaled_shape`) |
| Joints invisíveis no editor de eventos (Construct manual; GDevelop wiki) | Não nos aplica — mas confirma: a metade VISÍVEL é o produto |
| Física on/off, playback não-repetível, sem mixar com animação à mão (Moho lostmarble 24384) | Já curado (toggle Physics, replay bit-exato, bake). Registrar no plano como vantagem a DEFENDER |
| Round-trip de mão única (Newton, Adobe community 12500948) | Estrutural a nosso favor: joints vivem NA cena/timeline. Idem |
| ID-em-variável órfão (GDevelop 76793) | Joint é entidade com nome; nunca exponha id numérico como interface de gestão |

---

## §4 — O que o rapier 0.28 tem NATIVO e nós não expomos (lido do source)

O `GenericJoint` por baixo de todo tipo: `local_frame1/2` (Isometry — âncora **e rotação de frame**),
`locked/limit/motor/coupled_axes` (2D: LinX·LinY·AngX), `limits[3]` (min/max), `motors[3]`
(**`target_vel` E `target_pos`** + stiffness/damping/max_force + `MotorModel`
AccelerationBased|ForceBased), **`contacts_enabled`**, **`enabled: JointEnabled`**
(Enabled|Disabled|DisabledByAttachedBody), `user_data`. E `ImpulseJoint.impulses: SpacialVector` é
**público** — a força de reação que o Godot não tem.

| Capacidade nativa | Onde está no source | Status nosso | Destino |
|---|---|---|---|
| **PrismaticJoint** (eixo, limites de curso, motor completo) | `prismatic_joint.rs` | ❌ não existe | **W-J5 (Slider)** |
| **Motor de POSIÇÃO (servo)** — `set_motor_position(pos, stiffness, damping)` | `revolute_joint.rs` / `prismatic_joint.rs` | ❌ só velocity | **W-J6** |
| **Motor na Rope** (eixo acoplado) = guincho | `rope_joint.rs` (`set_motor_*`) | ❌ | **W-J6** |
| `contacts_enabled` por joint | todo builder | hardcoded `false` | **W-J8** (toggle, default off — o certo) |
| `JointEnabled` (desabilitar sem deletar) | `generic_joint.rs` | ❌ | **W-J8** (checkbox "Active", idioma Newton) |
| `impulses` (reação, p/ break) | `impulse_joint.rs` | ❌ | **W-J7** |
| `MotorModel` Force×Acceleration | `motor_model.rs` | Acceleration (default) | ficar — knob de engenheiro, não de artista |
| Rope com `limits [min,max]` → **Rod** (barra rígida) | `rope_joint.rs` + `set_limits` | ❌ | opção do W-J5/J8 (checkbox "Rigid" na Rope, idioma "Max Distance Only" invertido do Unity) |
| **GenericJoint direto** (lock/limit/motor por eixo) | `generic_joint.rs` | ❌ | §8 horizonte ("Custom") — o que o godot-proposals 15126 implora |
| **Multibody + `inverse_kinematics_delta`** (IK jacobiano pronto) | `multibody_joint/multibody_ik.rs` | ❌ | §8 horizonte (posar cadeia arrastando a ponta → bake) |

Vocabulário Box2D v3 para calibrar "completo" (o que RUBE/GDevelop espelham): revolute · prismatic ·
distance(+spring+rope-limit+motor) · wheel · weld · motor · filter. Com W-J5/J6 cobrimos o núcleo; wheel =
preset futuro; motor-joint e filter-joint = nicho anotado.

---

## §5 — Os princípios (o estado da arte, destilado)

- **P1 — Mire primeiro, ligue depois.** A âncora nasce onde o artista aponta (RUBE), nunca em (0,0) nem
  derivada de pose. Nenhum fluxo pode exigir escolher corpos ANTES de mirar.
- **P2 — Todo fato do joint é GEOMETRIA no canvas.** Tipo (glifo), posse (linha p/ A e p/ B), limites
  (arco), eixo (seta+trilho), rest/max length (anéis), motor (seta de giro), violação (vermelho). Quem
  desenha lê do MESMO `desc` que o solver consome — porta única, a lei do `scaled_shape`.
- **P3 — Duas vistas de um valor.** Arrastar a alça escreve o número; digitar o número move a alça. Já é a
  lei do app (Dur(s) da timeline); vale para rest length, limites, eixo, curso.
- **P4 — Pose, não digite.** Limite se autora POSANDO o fantasma do corpo B em torno da âncora (RUBE 'L').
  Digitar continua existindo (§12), mas é a segunda via.
- **P5 — Unidade em todo knob.** °, °/s, m, N·m, N·s (Algodoo). Row sem unidade é bug de review.
- **P6 — Re-seed é gesto, nunca modo.** O anti-Unity: nada de "Auto Configure" persistente possuindo campo.
  `anchored=false` one-shot já é a nossa lei — as waves novas não podem regredi-la.
- **P7 — Defaults honestos.** ∞ = off (break), jointed não colidem, tipos nomeados, criado = selecionado.
- **P8 — O par é visível e re-apontável.** Eyedropper (temos) + swap A↔B (RUBE) + nome auto "A : B"
  (Unreal Constraints Graph).
- **P9 — Um gesto do estado atual à corrente.** Seleção ordenada + Join = N-1 joints em cadeia (Newton).

---

## §6 — O desenho: melhor que os existentes, numa frase por peça

**Criação (o gesto novo, W-J4):** com um kind armado no "Join As", **clicar no corpo A e ARRASTAR até o
corpo B cria o joint com âncora A no ponto do press e âncora B no ponto do release** (shares_a_point:
mesmo ponto; spring/rope: as duas pontas do arrasto). Um fio elástico acompanha o arrasto; soltar fora de
corpo = pin-to-world? NÃO — recusa com toast (pin-to-world é horizonte §8, GDevelop o faz com static
oculto). O botão "Join Selected Bodies" FICA (rota por seleção, Newton-style, e é a rota da corrente P9).
Nenhuma escola da pesquisa tem as DUAS rotas; nós teremos.

**Visualização (W-J1):** vocabulário por tipo — Pin: glifo de dobradiça + arco de limites + seta de motor
· Spring: zigzag entre âncoras + anel de rest length (tenso/frouxo pela cor) · Rope: linha reta quando
tesa, curva quando frouxa + anel de max · Weld: glifo de solda · Slider: trilho + tracinhos de curso.
Linhas de posse âmbar→A / teal→B. Violação (âncoras separadas sob carga) em vermelho — o RUBE é o único
que desenha isso; seremos o segundo.

**Autoria de âncora (W-J2):** DUAS alças (A âmbar, B teal), snap com modificador: centro do corpo,
vértices/arestas do collider (Newton Ctrl), grade. O dot atual generaliza.

**Limites/motor/comprimentos (W-J3):** pontas do arco arrastáveis (cada ponta = um limite — assimétrico
por construção, o que o Unreal não expressa) com o FANTASMA do corpo B girando junto (RUBE 'L' sem modo:
arrastar a ponta JÁ posa); anel de rest/max arrastável; seta de motor arrastável = velocidade (comprimento
∝ °/s).

**Tipos (W-J5/J6):** Slider completo (o eixo é a ROTAÇÃO da entidade-joint — o gesto de girar o joint já
existe, Unreal/Godot fazem igual e é coerente com "o Transform é a âncora"); servo Position|Velocity em
chips; guincho na Rope.

**Robustez (W-J7/J8):** break force/torque (∞=off) com pico por-substep; Active checkbox; Collide
Connected; Swap A↔B; nome automático "A : B".

---

## §7 — As waves (cada uma fecha numa sessão, com a metade visível + gates + medição)

> Absorção do padrão-ouro antigo: W2-grupo → **W-JG** · W3-alças → **W-J2** · W4-visuais → **W-J1+W-J3** ·
> W5-break → **W-J7**. A ordem abaixo é a recomendada (visual primeiro: destrava todas as outras e é a
> maior distância entre nós e TODOS os concorrentes exceto RUBE).

| Wave | Entrega | Conteúdo | Gates/medição (esqueleto) |
|---|---|---|---|
| **W-J1 — O joint se DESENHA** ✅ (2026-07-25, `=43`) | vocabulário de canvas por tipo | glifo por kind + linhas de posse A/B + arco de limites (Pin) + zigzag/anel de rest (Spring) + linha tesa/frouxa + anel de max (Rope) + glifo de weld; desenha TAMBÉM no play; tudo lendo do `desc` da ponte (porta única) | overlay-scene gates por kind (mutação: desenhar do componente em vez do desc → RED sob edição mid-play); cena de smoke com os 4 kinds lado a lado |
| **W-J2 — Duas alças + snap** ✅ (2026-07-25, `=44`; refinada pela **W-J2b** — maiores, em TODA joint, z por último) | âncora B agarrável; snap | alça B (teal) via `PointGizmoView` (2º ponto); snap: centro/vértice do collider/grade com modificador; re-seed por gesto (P6) nos DOIS lados; `sync_joint_pivots` ganha o irmão do lado B | gate: arrastar B não move A; snap acerta vértice do collider REAL; mutação: snap→nop |
| **W-J3 — Pose, não digite** ✅ (2026-07-25, `=45`; ⚠️ **motor DEFERIDO com razão** — uma TAXA não tem lugar, e a row não tem faixa de onde tirar a escala) | limites/motor/comprimentos por manipulação direta | pontas do arco arrastáveis (cada uma escreve seu limite; fantasma do corpo B gira junto); anel rest/max arrastável; seta de motor = velocidade; tudo com a via numérica intacta (P3) | round-trip gate (arrasto→número→arrasto bit-estável); assimetria (só uma ponta move); fantasma nunca escreve pose real (gate) |
| **W-J4 — Criar onde se olha** ✅ (2026-07-25, `=46`; **+W-J4b** pós-smoke: o toggle Cancel + Esc, e as alcas ja postas ficam inertes/semitransparentes durante o gesto) | gesto press-A-drag-B no canvas | kind armado ⇒ press num corpo + release noutro cria joint com âncoras NOS pontos; fio elástico durante o arrasto; release inválido recusa com toast; rota por seleção FICA; multi-seleção ordenada → corrente (P9) | seam por ponteiro real (o repro do W-JointParams ensinou); gate: âncoras nascem nos pontos do gesto, não em centros; corrente N corpos = N-1 joints, 1 undo |
| **W-J5 — Slider (Prismatic)** ✅ (2026-07-26, `=47`; o eixo e a ROTACAO da entidade-joint — zero widget novo — e `limit_min/max` passam a carregar a unidade do TIPO, com re-semeadura na troca) | 5º kind completo | `JointKind::Slider` (variant apendado, discriminante 4, sem bump — a lei do Weld); eixo = rotação da entidade-joint; limites de curso; trilho+tracinhos no canvas; rows com unidade (m) | behavioral: porta desliza no eixo e para nos limites (mutação: eixo ignorado → RED); c9 ganha um slider (hash MUDA e é correto) |
| **W-J6 — Servo + guincho** ✅ (2026-07-26, `=48`; `has_motor` = Pin·Slider·Rope e `motor_axis` no wrapper — o motor aplicado UMA vez depois do builder; `MOTOR_TRACKING` RE-MEDIDO 100→1000, o linear perdia 20% da velocidade para a gravidade) | motor Position\|Velocity; motor na Rope | chips de modo no card Motor (Pin e Slider): Velocity (atual) \| Position (target ° + stiffness/damping medidos); Rope ganha card Motor (recolher/soltar, N) | servo: alvo alcançado e SEGURADO contra gravidade (medir tracking); guincho: corda encurta sob carga; mutação: target_pos ignorado |
| **W-J7 — Break force** ✅ (2026-07-26, `=49`; o teto é uma **CARGA** e não um impacto — o pico de uma pancada resolve DENTRO de um sub-passo e não é observável de fora, MEDIDO; e o teto de TORQUE é do **Pin** e de mais ninguém, porque o rapier não reporta reação de eixo angular TRAVADO) · **+W-J7b:** o **readout na tela** (`carga / teto` em âmbar, `max` do pico, e a carga que PARTIU em vermelho) — sem ele ajustar um teto era busca binária feita à mão | joints quebráveis | thresholds força/torque separados (Unity), default ∞=off; leitura de `impulses` com pico por-substep (padrão W-ImpactForce, mesma lição do raspão); ação: Disable (JointEnabled) — não destrói a entidade (undo-friendly); flash no ponto + toast | medir o impulso de quebra numa queda conhecida (fixture com número); gate: ∞ nunca quebra; quebra é evento (canal próprio, lição W-TickContacts) |
| **W-J8 — Higiene do par** | Active · Collide Connected · Swap · nome auto | checkbox Active (JointEnabled) — desabilitado esmaece o glifo; Collide Connected (default off); botão Swap A↔B; joint novo nasce "A : B" (nome editável como sempre) | presença E ausência por kind; swap preserva âncoras (gate); collide-on medido (elos que se tocam) |
| **W-JG — O grupo carrega o rig** | (ex-W2 padrão-ouro) mover corpo em repouso arrasta o grupo articulado | reusa `jointed_group` (W-BakeJoint); modificador para mover só um | gate: mover elo move a corrente em repouso; com modificador, não |

Dependências: W-J3 depende de W-J1 (o arco precisa existir); W-J4 é independente; W-J5/J6 são
independentes entre si; W-J7 depende só da infra de impulso; W-J8 é independente e pequena — candidata a
pegar carona na primeira sessão.

**Toda wave obedece a política de UI do plano-mãe** (4 condições: componente autorável · pintado e
registrado · clique chega ao barramento · a SEQUÊNCIA leva a algum lugar) **+ a 5ª desta linha:** a costura
UI→componente tem gate que dá flush (a lição do W-JointParams).

---

## §8 — Horizonte (nomeado para não re-derivar, sem wave)

- **Params de joint KEYFRAMÁVEIS** (Newton faz; para uma suíte de animação é o degrau seguinte natural —
  motor speed animado = maquinário). Cross-line com a timeline; pedir decisão do Enio quando as waves J
  fecharem.
- **Custom/GenericJoint** (lock/limit/motor por eixo — o godot-proposals 15126): só se um caso real pedir.
- **Wheel preset** (prismatic+spring+motor empacotado, idioma Unity/Box2D).
- **Pin-to-world / Target joint** (carregar no play, corpo→ponto de mundo; GDevelop usa static oculto).
- **IK multibody** (`inverse_kinematics_delta` pronto no rapier): posar corrente arrastando a ponta e
  bakear — diferencial de verdade para animação; arquitetura separada (multibody set), ADR próprio.
- **Ragdoll wizard** (Fyrox) / **Make chain por protótipo** (RUBE) — depois que a corrente por seleção
  ordenada (W-J4) existir e o uso pedir mais.
- **Soft weld** (frequency/damping no Fixed, idioma Unity) e **Rod** (rope rígida, min=max).
- **Copy/paste de propriedades de joint** (Unreal PhAT Ctrl+C/V).
- **Rows de readout tingidas** (RUBE verde=estático/vermelho=dinâmico) — se o §12 ganhar readouts vivos.

---

*Imagens: `~/Documentos/Recursos/UI_Reference/` (44). Relatórios integrais dos 5 agentes: na sessão de
2026-07-25. Superfície rapier conferida em `rapier2d-0.28.0/src/dynamics/joint/` local.*
