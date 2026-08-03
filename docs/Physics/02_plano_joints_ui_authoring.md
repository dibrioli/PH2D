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

> **⚠️ ESTE QUADRO É O DIAGNÓSTICO DE 2026-07-25, NÃO O ESTADO DE HOJE.** As sete limitações
> estão **FECHADAS**: L1→W-J1 · L2→W-J2/W-J2b · L3→W-J4/W-J4b · L4→W-J5/W-J6 (+ os tipos que o
> §8 não previa: **Rod** · **Wheel** · **Pulley**) · L5→W-J3 · L6→W-J7/W-J7b · L7→W-J8. O §7
> abaixo carrega o ✅ e a cena de cada uma. O que sobra do plano é o **§8**: **UM** item, e ele
> segue condicionado — **rows de readout tingidas** (⚠️ **condição NÃO satisfeita**: o readout de
> carga vive no OVERLAY, não em row). ⚠️ Os outros dois FECHARAM em 2026-08-03: **params
> keyframáveis** (W-JointAnim, cena `=78`) e **Custom/GenericJoint** (W-JointCustom, cena `=79`),
> este por ordem direta do Enio — que é literalmente o *"se um caso real pedir"* que o item exigia.
> ⚠️ **Duas saíram da lista construídas**: a metade autorável do Pin-to-world (W-JointWorld,
> cena `=65`, §9) e o **copy/paste de propriedades** (W-JointCopy, cena `=66`, §10). Dos
> quatro que restam, três estão condicionados a *"se o uso pedir"*.

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
| `unity_fixedjoint2d_inspector.png` | Fixed: **Damping Ratio + Frequency num WELD** | O weld deles é mola dura (tunável); o nosso era rígido (rapier FixedJoint). ✅ **Construído (W-SoftWeld, `=68`)** como VARIANTE e não default — a chave `Rigid \| Soft`, e a rígida segue sendo um `FixedJoint` de verdade (exato e mais barato) em vez de um motor rigidíssimo |
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
| **W-J8 — Higiene do par** ✅ (2026-07-26, `=50`; o **Swap é preservador de comportamento** — medido, um swap CRU reverte motor/servo e ESPELHA a faixa de limites, e a compensação reproduz o autorado ao 4º decimal ⇒ *ele troca qual ponta se chama A, e nada mais*. E o Active expôs um bug latente: ele e uma RUPTURA escrevem a MESMA flag do rapier, então desarmar um joint o pintaria de vermelho com estouro) | Active · Collide Connected · Swap · nome auto | checkbox Active (JointEnabled) — desabilitado esmaece o glifo; Collide Connected (default off); botão Swap A↔B; joint novo nasce "A : B" (nome editável como sempre) | presença E ausência por kind; swap preserva âncoras (gate); collide-on medido (elos que se tocam) |
| **W-JG — O grupo carrega o rig** ✅ (2026-07-26, `=51`, **smoke aprovado**; **Alt+arrastar** carrega o rig — opt-in por gesto, e o Alt é provadamente inerte no Translate do gizmo; a lei é o componente conexo **INTEIRO** (`jointed_rig`, irmão do `jointed_group` do bake — as duas divergem de propósito, com gate); e os DOIS sítios de Down passaram a semear pela mesma porta, `joint_rig_drag`) | (ex-W2 padrão-ouro) mover corpo em repouso arrasta o grupo articulado | reusa `jointed_group` (W-BakeJoint); modificador para mover só um | gate: mover elo move a corrente em repouso; com modificador, não |

Dependências: W-J3 depende de W-J1 (o arco precisa existir); W-J4 é independente; W-J5/J6 são
independentes entre si; W-J7 depende só da infra de impulso; W-J8 é independente e pequena — candidata a
pegar carona na primeira sessão.

**Toda wave obedece a política de UI do plano-mãe** (4 condições: componente autorável · pintado e
registrado · clique chega ao barramento · a SEQUÊNCIA leva a algum lugar) **+ a 5ª desta linha:** a costura
UI→componente tem gate que dá flush (a lição do W-JointParams).

---

## §8 — Horizonte (nomeado para não re-derivar, sem wave)

- ~~**Params de joint KEYFRAMÁVEIS**~~ — **FECHADO (W-JointAnim, 2026-08-03, cena `=78`).**
  ⚠️ **As três saídas que o tracker nomeava eram todas caras, e havia uma QUARTA já shipada
  na própria `ph2d-timeline`:** o `PropKind::Opacity` dirige um campo do `ph2d_render::Sprite`
  — outra crate que o runtime base não quer conhecer — por **dep OPCIONAL atrás de uma feature**
  que a shell liga. A feature `physics` é a irmã exata dela; o runtime base segue sem física,
  como segue sem wgpu, e nada de novo foi inventado.
  ⚠️ **E a wave é CORREÇÃO antes de ser capacidade:** um parâmetro keyframado é uma **entrada
  por TICK**, do mesmo jeito que a pose de um corpo cinemático (a auditoria do W4b). O
  `reconcile_joints` roda uma vez por DISPATCH e os laços de play e replay dão N passos dentro
  dele, então sem o `drive_joint_params` o número chegaria uma vez por QUADRO e, num replay,
  **nunca**.
- ~~**Custom/GenericJoint**~~ (lock/limit/motor por eixo — o godot-proposals 15126) —
  **FECHADO (W-JointCustom, 2026-08-03, cena `=79`), porque o caso real pediu: o Enio.**
  ⚠️ **A nota de que o Wheel NÃO era esse caso continua CERTA** — ele queria a *plumbing* do
  `GenericJoint`, não a UI por-eixo —, e é por isso que o Custom **não substitui** os presets:
  um Pin diz *"isto é uma dobradiça"*, e é essa frase que o glifo desenha, que o `fk_dof` lê
  para posar e que o `is_rigid_link` lê para montar a árvore de IK.
  ⚠️ **O que a wave descobriu e a nota não previa:** duas perguntas deixam de ser do TIPO num
  Custom — a UNIDADE do motor e a reação ANGULAR —, e a primeira falha em SILÊNCIO (rotulada em
  graus, um alvo que o solver lê em metros faz o artista digitar 90 e a peça andar 1,57 m).
- ~~**Wheel preset**~~ — **FECHADO (W-Wheel, 2026-07-27, cena `=57`).** ⚠️ E o nome do item estava
  errado em duas frentes: não é *preset* (não empilha joints — é **UM** `GenericJoint` travando
  `LIN_Y`, e dois joints sobre o mesmo par dariam ao solver duas restrições brigando) e não é
  *prismatic+revolute*: é o **primeiro tipo do kit a deixar DOIS graus de liberdade livres**. A
  suspensão é motor de posição em `LinX` (a mola do artista), o giro é o motor dele em `AngX`, e
  o **curso tem os DOIS batentes** — o que o Rod não conseguiu, porque em rapier o acoplamento é
  EXPLÍCITO (`coupled_axes`) e aqui nada é acoplado.
- ~~**POLIA**~~ — **FECHADA (W-Pulley, 2026-07-27, cena `=58`).** A pesquisa desta lista
  estava certa sobre o motor (rapier não tem polia e o `PhysicsHooks` só fala de CONTATOS)
  e **errada sobre a dependência**: ela dizia que a polia *"quer o conceito de âncora de
  MUNDO, que é exatamente a metade autorável do Pin-to-world"* e que *"as duas andam juntas
  ou nenhuma anda"*. Não andam. Uma polia guarda os **próprios** pontos de mundo
  (`wheel_a`/`wheel_b`), e o primitivo de AUTORIA que a nota queria — `PointGizmoView` +
  `paint_point_gizmo` — já tinha sido construído pela W-JointAnchor e generalizado para uma
  LISTA pela W-J2b. A dependência já estava paga quando a nota foi escrita.
  ⚠️ E o que ela previa como *"polia MOLE (estica sob carga)"* **não é o que shipou**: um laço
  de força PD estica, e por isso não foi usado — o passe é uma **projeção de velocidade com a
  massa efetiva exata do Jacobiano**, então a corda segura igual com 0,1 kg ou 100 kg
  (medido) e o esticamento em regime é **1,1 mm**, menor que a tolerância de repouso do
  próprio rapier. O que ela NÃO faz é **partir** (`JointKind::can_break` — nada mede a reação
  de algo fora do `ImpulseJointSet`), e a §12 não pinta a caixa.
- ~~**Pin-to-world / Target joint**~~ — **FECHADO NAS DUAS METADES.** A do **GESTO**
  fechou na W-Grab (2026-07-26, cena `=52`); a **AUTORÁVEL** fechou na
  **W-JointWorld** (2026-07-30, cena `=65`, §9 deste plano) — e ⚠️ **a objeção
  desta nota estava certa e o preço, menor do que ela dizia**: os `names_two_bodies()`
  eram **dois** sítios de produto a mudar, não quatro (o rig walk já estava certo
  por construção, e o overlay não gateia nada). O texto original, para o registro:

  > arrastar um corpo dinâmico com o relógio andando é a **MÃO** (uma mola macia para um corpo-âncora
  > invisível no cursor). O que resta no horizonte é a metade **AUTORÁVEL** — um joint com UM corpo e um
  > ponto de mundo persistido —, e ela não é mecânica: `names_two_bodies()` gateia o reconcile, o rig walk,
  > o overlay e a §12.
- ~~**IK multibody**~~ — **FECHADO ([ADR-0149](../architecture/decisions/0149-physics-ik-is-a-transient-posing-tree-not-a-second-joint-representation.md),
  W-IK, 2026-07-27, cena `=54`)**, e ⚠️ **a metade "arquitetura separada (multibody set)" desta nota
  foi DELIBERADAMENTE recusada** — leia o ADR antes de reabrir. O multibody **não é estado da cena, é
  uma ferramenta transitória de POSE**: a árvore é construída no press, resolvida, escrita de volta em
  `Transform` e jogada fora no release; nada no `step` a toca e o `MultibodyJointSet` do mundo continua
  **VAZIO**. A alternativa que esta nota sugeria (joints multibody-NATIVOS como representação de
  produção) está avaliada e rejeitada no ADR §*Alternativas*, com o preço: seria uma **ÁRVORE** (um pai
  por corpo — laço fechado inexprimível) e todo elo não-raiz teria de ser `Dynamic`.
  ⚠️ E a metade **"e bakear"** também está paga, por outro caminho: posar escreve `Transform`, e com o
  **AutoKey armado a timeline captura pela máquina que já existe** — não há, nem deve haver, um
  segundo caminho de IK→keyframe (plano 04 §5).
- ~~**Ragdoll wizard** (Fyrox) / **Make chain por protótipo** (RUBE)~~ — **FECHADO (W-Rig, 2026-07-31,
  cena `=67`, §11 deste plano).** ⚠️ E o item era **dois**, com metades de tamanhos muito diferentes: o
  *make chain por protótipo* já estava praticamente pago (a corrente da W-J4 CRIA, a W-JointCopy
  CARIMBA — dois gestos em vez de um), e o que faltava mesmo era o wizard, porque **uma corrente é uma
  FILA e um ragdoll é uma ÁRVORE**. A árvore não precisou ser inventada: o artista já a desenhou na
  Hierarquia.
- ~~**Soft weld**~~ — **FECHADO (W-SoftWeld, 2026-07-31, cena `=68`, §12 deste plano).** A premissa
  original (*"frequency/damping no Fixed"*) já estava corrigida aqui em 2026-07-27, e a construção
  corrigiu a **correção**: ela prescrevia *"não travar nada + três molas em alvo 0"*, e isso foi
  construído, medido e **REPROVADO** — com os três eixos moles o braço deriva **0,92 m** para longe da
  parede e balança 104° sem nunca assentar. As peças vêm APART, que se lê como a solda FALHANDO, não
  vergando. O que shipou trava os DOIS eixos lineares e amolece só o **ANGULAR** (separação medida
  `0,0000 m`), que é o que uma solda mole é. E o *"nada o pede hoje"* estava certo sobre a demanda e
  errado sobre o VÃO: este conjunto segurava um ângulo **absolutamente** (Weld, Slider) ou o deixava
  **inteiramente livre** (Spring, Rope, Rod, o giro do Wheel), e não havia nada no meio — o espelho
  exato do vão que o Rod preencheu na distância.
- ~~**Rod**~~ — **FECHADO (W-Rod, 2026-07-27, cena `=56`).** E a construção que esta linha previa está
  **MEDIDA E MORTA**: `set_limits(LinX, [len, len])` não segura, porque o limite linear acoplado do rapier
  é **unilateral** (`// FIXME: handle min limit too.` no solver dele). O que ficou é um **motor de posição
  no eixo acoplado**, `ROD_STIFFNESS = 1e6` medido. Detalhe no [plano de waves](00_plano_waves.md).
- ~~**Copy/paste de propriedades de joint**~~ — **FECHADO (W-JointCopy, 2026-07-31, cena `=66`).**
  ⚠️ E o item mudou de forma ao ser construído: *"Ctrl+C/V"* estava errado. O atalho de
  clipboard deste app **segue a ÁREA SOB O MOUSE** desde a integração da `line/anim-fixes`
  (a regra do Blender: mouse na timeline = keyframes, no canvas = formas), então
  sequestrá-lo para joints seria um TERCEIRO significado da mesma tecla. O gesto virou
  **dois botões na §12** — a seção que já é dona de todo parâmetro de joint — e o Paste é
  **a única edição da §12 que faz fan-out**, que é a razão de ele existir.
- **Rows de readout tingidas** (RUBE verde=estático/vermelho=dinâmico) — se o §12 ganhar readouts vivos.

### §8.1 — A ordem escolhida (2026-07-27, ordem do Enio: *"o item B primeiro"*)

Não é a ordem da lista; é a que sai das dependências **medidas** acima.

| # | item | por que aqui |
|---|---|---|
| ~~1~~ | ~~**Rod**~~ ✅ `=56` | o gap real — hoje **nada** segura dois corpos a distância fixa deixando os dois GIRAREM (o Weld trava o giro, a Rope só o teto). O menor, e estreia a plumbing do `GenericJoint` |
| ~~2~~ | ~~**Wheel preset**~~ ✅ `=57` | o de maior valor visível (um veículo). Precisou da plumbing por eixo, que o Rod abriu |
| ~~3~~ | ~~**Polia**~~ ✅ `=58` | a primeira que precisa de um passe de restrição PRÓPRIO (rapier não a tem). ⚠️ **NÃO puxou o *Pin-to-world*** — ela guarda os próprios pontos de mundo, e o gizmo de PONTO que a nota queria já existia desde a W-JointAnchor |
| ~~4~~ | ~~**Ragdoll wizard / make chain**~~ ✅ `=67` | era um GERADOR, e a previsão se confirmou pelo avesso: o valor não veio do conjunto de tipos, veio da **HIERARQUIA** — a estrutura que uma corrente não expressa e que já estava desenhada |
| ~~5~~ | ~~**Copy/paste de propriedades**~~ ✅ `=66` | e a previsão *"vale mais quanto mais propriedades existirem"* se confirmou: são **doze** campos de afinação a carimbar hoje |
| ~~6~~ | ~~**Soft weld**~~ ✅ `=68` | e o vão era o espelho do que o Rod preencheu: nada segurava um ÂNGULO com folga. A previsão *"caminho de builder próprio"* estava certa; a receita dela (**não travar nada**) foi medida e reprovada — as peças se separam 0,92 m |

---

## §9 — W-JointWorld: a metade AUTORÁVEL do Pin-to-world (2026-07-30)

**O que é:** um joint cujo lado B é um **PONTO DE MUNDO**, não um corpo. A dobradiça na
parede, o pêndulo no teto, a mola presa no cenário.

**O que ele remove:** hoje, para prender algo ao mundo, o artista **inventa um corpo
estático** só para servir de âncora — um objeto a mais para nomear, achar na Hierarquia e
mover por acidente. O mundo não é um objeto da cena, e não devia precisar de um.

### §9.1 — O que a medição mudou nesta nota, antes de uma linha de código

⚠️ **O primitivo JÁ EXISTE.** A nota dizia que a metade autorável *"não é mecânica"*, e a
parte cara — *como se prende um corpo a um ponto de mundo em rapier?* — está construída e
medida desde a **W-Grab**: `grab_body_with` insere um `RigidBodyBuilder::fixed()` no ponto e
jointa contra ele. rapier **não tem âncora de mundo**; todo joint é corpo↔corpo, e a resposta
do repo já é um corpo fixo. Esta wave dá a esse corpo um **ciclo de vida autorado** em vez de
um por-gesto.

⚠️ **São DOIS sítios, não os quatro que a nota lista.** Medido:

| sítio da nota | o que a medição diz |
|---|---|
| reconcile (`bridge/joints.rs:266`) | ✅ muda — é onde a âncora nasce |
| §12 (`inspector_joint.rs:158`) | ✅ muda — `bound` tem de aceitar "A + mundo" |
| **rig walk** (`joint_group.rs:152`) | ⛔ **JÁ ESTÁ CERTO, e não é acidente:** ele contribui uma **ARESTA entre dois corpos**, e um pino no mundo não tem segundo corpo ⇒ nenhuma aresta ⇒ **o mundo é FRONTEIRA por construção**, exatamente como Static/Kinematic já são. Mexer aqui seria fazer o rig andar para dentro do cenário |
| **overlay** | ⛔ **não gateia nada** — `names_two_bodies` não aparece no overlay (grep). Ele desenha do `desc` da ponte, e um joint que a ponte não construiu simplesmente não tem `desc` |

### §9.2 — As três decisões

**D1 — MARCADOR, nunca overload de `body_b == 0`.** Esse estado **já significa
meio-autorado** (*"o artista tem um objeto joint e ainda não escolheu o segundo corpo"*) e o
reconcile o trata explicitamente. Reinterpretá-lo faria **todo joint recém-criado pinar no
mundo** no frame entre o clique e a escolha de B — uma mudança de comportamento silenciosa
num caminho que todo artista percorre. Fica um componente **marcador** (`JointWorldAnchor`,
presença = o booleano), blob-key própria ⇒ **zero bump de `PROJECT_SCHEMA`** — o idioma que
esta linha já usou seis vezes (`Ccd`, `LockRotation`, `LockPositionX/Y`, `OneWayPlatform`,
`AreaForceWorldAxes`, `WestonAxle`).

**D2 — O ponto é o `Transform` DO PRÓPRIO JOINT, não um campo novo.** Ele **já é** a âncora
autorada, **já tem** o dot âmbar arrastável (W-JointAnchor), **já tem** as rows Position, e
**já viaja** no save. Um `world_point: [f32; 2]` no componente seria um segundo lugar para o
mesmo fato — e custaria um bump.
⚠️ **Consequência que é a metade do trabalho:** o `sync_joint_pivots` reescreve
`Transform = bodyA · local_a` em repouso (W-AnchorFollow), e num pino de mundo isso é
**exatamente ao contrário** — quem segue a âncora é o CORPO, não a âncora o corpo. Ele tem de
pular um joint marcado, senão arrastar o dot é desfeito no frame seguinte.

**D3 — A âncora fixa mora no `JointRef`**, criada e destruída **com o joint**. Precedente
direto: o `Grab` guarda a dele no `Grab { anchor, .. }`.
⚠️ **E a lição do Weston é lei aqui:** `rebuild_from_rest` **troca o `PhysicsWorld`** e o
replay roda no MESMO chamado, então a âncora tem de voltar junto — foi exatamente assim que a
tabela de polias sumiu e *"um rewind replayava sem as cordas"*. O gate nasce dessa forma: um
scrub para um tique intermediário, não um Reset (que replaya zero passos e **não vê o bug**).

### §9.2b — O GESTO DE CANVAS (correção de smoke, 2026-07-30)

⚠️ **A wave shipou sem a metade que o artista tenta primeiro.** Relato do Enio:
*"não aceita desenhar a junta a partir do canvas vazio, apenas de um objeto para
outro"* — e ele está certo: eu construí o chip da §12 e deixei o gesto do canvas
(W-J4) recusando um release no vazio. Pior, **a recusa dizia ao artista que pinos
de mundo não existiam**, uma frase que esta mesma wave acabara de falsificar.

Agora **soltar no vazio cria o pino** — o vazio *é* o mundo. A única recusa que
sobra é a que continua verdadeira (um joint não liga um corpo a ele mesmo), mais
a **POLIA**, recusada no GESTO e não só no reconcile: recusar tarde deixaria o
artista criar um objeto que nasce dormente sem dizer por quê.

⚠️ **A política de âncora é COPIADA do irmão de dois corpos, e é o que evita um
TRANCO:** num tipo que compartilha um ponto (Pin/Weld) os dois lados são o MESMO
lugar, então ancorar A no ponto do *press* faria o solver arrancar o corpo até o
ponto do *release* no primeiro passo. `create_world_pin_at` mora em arquivo
próprio porque `create_joint_at` **exige dois corpos por construção** — ele toma
bits de entidade, e `Entity::from_bits(0)` não existe.

⚠️ **O arch-gate disparou sobre o meu próprio comentário**, que citava a frase
morta ao pé da letra. Não é ruído: um grep futuro cairia nele do mesmo jeito. O
comentário passou a parafrasear, e diz por quê.

### §9.2c — As duas metades que o 2º smoke derrubou (2026-07-30)

Relato do Enio: *"arrastar do canvas vazio para o objeto também deveria
funcionar"* e *"ainda não posso mover a âncora colocada no mundo"*.

**(a) O gesto vale nas DUAS direções.** O artista pensa *"prego na parede, agora
ligo a bola nele"* tanto quanto o contrário, e as duas produzem o MESMO joint —
o que muda é qual ponta ele nomeia primeiro. ⚠️ **Os pontos TROCAM de papel**, e
essa troca é uma **porta pura** (`gesture_points`) em vez de escrita nos dois
braços do `match`: uma delas nasceria invertida no dia em que um terceiro braço
aparecer, e a versão errada nasce com a âncora onde a mão *terminou*.

**(b) A âncora não se movia, e o mecanismo era exato.** O dot é desenhado no
`Transform` do joint — que num pino de mundo **É** a âncora —, mas
`set_joint_anchor_world(A)` escrevia `local_a`, isto é *onde no CORPO o pino
prende*. O desenho ficava onde estava e o arrasto parecia não fazer nada.
⚠️ **Num pino de mundo o lado A move a ÂNCORA**, e com o `local_a` intacto o
CORPO vai junto — que é o certo: mover a âncora de um pêndulo o faz pender do
ponto novo. Mudar *onde no corpo* ele prende é outra pergunta, e ela não tem alça
(§9.3).

⚠️ **Um gate meu NÃO PODIA FALHAR:** o das duas direções chamava a porta de
criação com os MESMOS argumentos duas vezes e comparava os resultados — verde por
construção, sobre nada. Ele agora pergunta à porta pura o que a direção de fato
muda.

### §9.3 — Aberto de propósito

- **Um joint de mundo não entra num rig** (D1/§9.1) — arrastar o corpo preso NÃO carrega o
  cenário, porque não há o que carregar.
- **Não há alça para *onde no corpo* o pino prende.** O dot move a ÂNCORA (§9.2c b); o
  `local_a` é semeado no gesto de criação e depois só muda por re-criação. Uma segunda
  alça responderia a outra pergunta, e ela ainda não foi pedida.
- **Um pino de mundo e um pino entre dois corpos leem IGUAL na tela.** O overlay o desenha
  de graça e a geometria está certa; o que falta é o glifo dizer que aquela ponta é o
  cenário. Nomeado, não construído — decisão de desenho.
- **Não parte sob carga** por ora: `can_break` lê a reação do `ImpulseJointSet`, e a âncora
  ESTÁ nele — então é provavelmente de graça, mas *"provavelmente"* não é medição. Fica
  nomeado, não afirmado.

---

## §10 — W-JointCopy: copiar e colar as propriedades (2026-07-31)

O item 5 do §8, e ele **mudou de forma ao ser construído**. A linha do plano dizia
*"Copy/paste de propriedades de joint (Unreal PhAT Ctrl+C/V)"*; o atalho estava errado e o
resto estava certo.

### §10.1 — O gesto NÃO é Ctrl+C/V

O clipboard deste app **segue a ÁREA SOB O MOUSE** desde a integração da `line/anim-fixes`
(a regra do Blender: mouse na timeline copia keyframes, no canvas copia formas). Um
terceiro significado para a mesma tecla é como se descobre que copiar um joint copiou um
desenho. O gesto virou **dois botões na §12**, logo acima do Delete — a seção que já é dona
de todo parâmetro de joint, e onde o artista já está quando pensa *"os outros nove têm de
ficar assim"*.

### §10.2 — O que é uma PROPRIEDADE (a linha já estava na tela)

O corte é o que o `joint_pair_rows` já declara: *"aqui **entre QUAIS DOIS** isto está, e
como eles se tratam; lá **o que a restrição FAZ**"*. Viaja a segunda metade.

| classe | campos | viaja? |
|---|---|---|
| identidade | `body_a` · `body_b` | ❌ colar isto é DUPLICAR o joint, não copiar |
| colocação | `local_a` · `local_b` · `anchored` | ❌ um offset medido no corpo da fonte não significa nada no corpo do alvo |
| o experimento | `active` | ❌ é o *"tente o rig sem este aqui"*, sobre UM joint — e o paste age sobre muitos |
| o que a restrição faz | `kind` + os doze de afinação + `collide_connected` | ✅ |

⚠️ **O TIPO viaja, e é o que torna a colagem SEGURA.** Metade destes números não tem
unidade própria: `limit_min/max` são **radianos** num Pin e **metros** num Slider,
`motor_speed`/`motor_target` são rad/s num eixo e m/s num trilho. Números sem o tipo junto
seriam reinterpretação de unidade em silêncio — o ±0,785 rad de um Pin virando ±0,785
**metro** de curso. E é a mesma razão de o paste **não poder** ser escrito como *um
`Kind(tag)` seguido de quinze edições de campo*: o braço `Kind` re-semeia limites, motor e
mola para os defaults do tipo novo, e as quinze edições seguintes estariam desfazendo esse
re-seed campo a campo. Chegando juntos, **não há re-seed a fazer**.

A **âncora** é a exceção com regra: a *política* de âncora é função do tipo (Pin/Weld
ancoram no ponto compartilhado, Spring/Rope no centro de B), então tipo diferente derruba
`anchored` para pedir UMA re-derivação — o 5º sítio de autoria, ao lado do dot, do commit
de Position, do re-pick e do braço `Kind`.

### §10.3 — Um campo novo QUEBRA A COMPILAÇÃO

`PhysicsJoint::with_properties_of` desestrutura a fonte **exaustivamente**. Não é estilo: é
a resposta ao *enumeração apodrece*, e ela erra para o lado seguro. Uma lista escrita à mão
envelhece nos dois sentidos — um campo de afinação novo que não viaja deixa o paste
incompleto (chato, e visível), e um campo de IDENTIDADE novo que viaja faz o joint apontar
para o corpo errado (catastrófico, e silencioso). Com o destructuring, o campo dezoito não
compila até alguém dizer de que lado ele está.

### §10.4 — O ÚNICO fan-out da §12

Colar num joint por vez seria *digitar quinze campos, dez vezes*. O Paste se espalha sobre
a seleção, e a **contagem entra no rótulo** (`Paste to 3 Joints`) porque um clique que muda
dez objetos tem de dizer isso antes — a lei do `Bake 5.0s to Timeline`.

⚠️ E as duas irmãs estruturais **continuam recusando** o fan-out por motivos que não valem
aqui: um `Join` espalhado criaria N joints entre o mesmo par, e um `Bake` espalhado
re-simularia a cena inteira N vezes pelos MESMOS números, deixando N passos de undo.
Um arch-gate afirma as duas metades — que o Paste espalha, e que nada mais espalha.

### §10.5 — O que a medição corrigiu

- **O oráculo da cena era periódico.** A primeira sonda mediu o ângulo de cada portão no
  tick 120 e reportou **38,1°** para os três sem batente — um portão livre é um PÊNDULO, e
  `t = 2 s` é um ponto arbitrário do ciclo dele. Esse número teria virado a constante da
  mensagem. Pelo **máximo da trajetória** eles giram **179,7°** (quase dão a volta) contra
  **25,1°** do afinado, e depois da colagem os quatro param em 25,1. É exatamente a fixture
  que o W3 já pagou uma vez (*"o pêndulo é PERIÓDICO … tudo virou TRAJETÓRIA"*), reincidida.
- **Um gate irmão ficou vermelho sobre produto correto**, e a causa é a família que o
  próprio arquivo dele documenta: `the_joint_edit_loop_flushes_the_command_queue` procurava
  `body.find("apply_editor_commands(")` — a PRIMEIRA ocorrência — e o braço do Paste ganhou
  um flush legítimo que aparece antes na fonte. A busca passou a começar NO apply, e o gate
  ganhou a metade que faltava: o flush encontrado tem de estar no **mesmo ramo**, senão
  apagar o flush deste apply ficaria verde no dia em que um irmão com flush próprio
  nascesse depois dele.

### §10.6 — Aberto de propósito

- **A área de transferência é runtime-only** (`App.joint_clipboard`), como o `join_kind` e o
  `joint_body_pick`: um projeto reaberto não oferece um Paste do que alguém copiou semana
  passada.
- **Colar entre duas SESSÕES do app** não existe pelo mesmo motivo, e nada o pede.
- **Não há copy/paste na §13 (a roldana).** A roldana tem três números e um corpo de
  montagem; o gesto se paga onde há doze campos, e inventá-lo ali seria simetria em vez de
  necessidade.


---

## §11 — W-Rig: o rig sai da HIERARQUIA (2026-07-31)

O item 4 do §8.1, e a última rota de criação que faltava.

### §11.1 — A lei, numa frase

**Uma aresta pai→filho da Hierarquia É um joint.**

As duas rotas que já existiam ligam o que o artista **APONTA** — `Join Selected` liga uma sequência
marcada, o arrasto no canvas liga dois corpos por um gesto. Nenhuma das duas expressa uma **ÁRVORE**:
a pelve de um ragdoll tem três filhos, e uma corrente ligaria `Torso→Head→ArmL→ArmR`, uma fila que
descreve um boneco que não existe.

E a árvore não precisou ser inventada. Ela já está desenhada: é a Hierarquia.

### §11.2 — O corte, e o que cada metade sabe

| metade | onde | o que ela responde |
|---|---|---|
| **topologia** | `ph2d-physics-ecs::rig` (`rig_edges` / `subtree_parts`) | *dada uma lista de partes e a árvore, quais são as arestas?* — pura, headless, sobre o ECS **autorado** |
| **quem é parte** | `shells/desktop/src/joint_rig.rs` | precisa de `Sprite`, e a crate de física não conhece `ph2d-render` **nem deve** |
| **como uma parte vira corpo** | a porta da §11 (`PhysicsFieldEdit::Add`) | ela já sabe tirar o collider da **CAIXA DO SPRITE**; uma segunda regra faria um rig cujos colliders discordam dos que o botão *Add Body* produz |

É o mesmo corte do `jointed_group` (W-BakeJoint): função pura sobre o estado autorado, gateável sem
um dispatch no caminho.

### §11.3 — O GRUPO é transparente, e essa é a decisão de desenho

Um nó sem desenho é **organização**, não osso — a metade que o ADR-0133 chama de organizacional. Duas
saídas erradas e a certa:

- **dar-lhe corpo** planta um collider invisível de meio metro no meio do personagem (é o fallback do
  `Add` para entidade sem sprite);
- **pular a aresta** DESCONECTA o filho do rig;
- **deixá-lo passar** — o filho se liga ao **AVÔ**. É por isso que a função pergunta *"qual o ancestral
  mais próximo que TAMBÉM é parte?"* em vez de olhar só o pai.

### §11.4 — O botão mora nas DUAS faces da §11, e a vazia é a que importa

As rotas de LIGAR não aparecem na face vazia de propósito: elas precisam de corpos, e um sprite pelado
não tem nenhum. **O rig aparece porque ele os CRIA** — e a face vazia é o caso NORMAL dele, não uma
borda: um personagem nasce como sprites parenteados, sem corpo em lugar nenhum. Deixá-lo só na face com
corpo faria o gerador exigir o passo manual que ele existe para remover.

A contagem entra no **RÓTULO** (`Rig 6 Parts from Hierarchy`), a lei do `Bake 5.0s to Timeline` e do
`Paste to 3 Joints`: ela é a **divulgação** da expansão de subárvore — se você marcou um tronco
esperando três partes e lê seis, você vê o alcance antes de clicar, em vez de desfazendo.

### §11.5 — Re-executável, e é o que o torna ferramenta

Uma aresta que **já tem joint é pulada**. Acrescente um braço ao personagem, clique de novo, e só o
braço novo ganha joint. ⚠️ Dois joints sobre o mesmo par são **legítimos em geral** (é metade do motivo
de um joint ser ENTIDADE, W3); o que esta wave recusa é o **gerador** produzi-los, porque um rig com
duas restrições no mesmo par é o solver brigando consigo mesmo sobre um par que ninguém autorou duas
vezes.

E como todo o trabalho cai no MESMO frame, **um Ctrl+Z desfaz o rig inteiro** — o undo global é por
diff de fim de frame, ele vê um estado e não N operações.

### §11.6 — ⚠️ A medição achou um DEFEITO PRÉ-EXISTENTE, e ele era grande

A primeira corrida da cena 67 mostrou o boneco **esparramando**: o pescoço separava **1,09 m** em 3 s,
mais do que a geometria permite por rotação.

A causa não era do rig. `create_joint_at` computava o ponto médio a partir de
`Transform.translation` **CRU** — e `Transform` é **LOCAL** e compõe com o pai (W5). Juntar um
corpo-filho punha a âncora no meio entre a pose de MUNDO de um e o **OFFSET** do outro, um lugar que
não é nem um nem outro. Medido na cena: a âncora do pescoço nascia em `y = 1,85` com a emenda em
`y = 3,5` — **1,65 m abaixo** —, e cada membro pendia de um ponto flutuando no ar longe dele.

Isto alcançava as **três** rotas de criação (o botão Join, o arrasto no canvas e o rig), e sobreviveu
desde o W3 porque **toda** fixture, cena e demo desta linha usava corpos-RAIZ, onde local e mundo
coincidem. É literalmente a frase que o W5 escreveu sobre si mesmo, no arquivo ao lado, um ano depois.

Conserto na PORTA (as duas leituras passam por `world_transform`), com gate red-first que nasceu
vermelho em `y = 1,850` — o número exato que a sonda tinha medido. Depois: pescoço **1,09 m → 0,0026 m**.

⚠️ **O `physics_ecs_c9` saiu byte-idêntico** (`7cb7728d…`, 96 corpos) e isso é coerente, não sorte: a
cena de determinismo não tem corpo parenteado sob joint, então o conserto não a alcança — ele só muda o
que já estava errado.

### §11.7 — Números

| grandeza | valor |
|---|---|
| partes do boneco da cena 67 | 6 (mais um grupo, que não vira osso) |
| joints criados por um clique | 5 |
| queda do tronco em 3 s | 3,15 m |
| separação do pescoço em 3 s | **0,0026 m** (era 1,09 m antes do conserto da âncora) |
| gates / mutações | 21 gates novos · 14 mutações, 14 sangram |

### §11.8 — O smoke do Enio, e os dois itens que ele mandou fechar

O smoke aprovou o gesto e reprovou uma coisa: ***"o reset não consegue devolver o conjunto à posição
original"***. E os dois itens acima deixaram de ser abertos por ordem dele — *"busque o melhor, o
perfeito, o padrão-ouro, sem pensar em custos"*.

**(a) O Reset — e não era do rig.** Isolado num par pai+filho **sem joint nenhum**: a raiz voltava
exata e o filho aterrissava a **4,910 m**, que é `½·g·t²` do segundo simulado — a queda do PAI
aparecendo dentro do local do filho. O `readback` escreve o filho ANTES do pai (a ordem de `Entity`
num `BTreeMap` não é a de spawn), então ele converte a pose de mundo contra o pai do frame anterior.
Durante o play isso vale **3,2 mm** e some no movimento; num rewind vale o salto inteiro. Pré-existente
desde o W5. Agora o readback escreve em ordem de PROFUNDIDADE. Detalhe: [BUGS #5](BUGS_physics.md).

**(b) A âncora vai para a EMENDA.** A regra antiga era uma aproximação — o próprio doc dela dizia o
que aproximava —, e ela só vale entre formas do mesmo tamanho. A emenda é o meio entre os dois pontos
em que as SILHUETAS cruzam a linha dos centros, **reduz ao ponto médio no caso que o desenho antigo
acertava**, e não precisou de geometria nova: a `radial_fraction` do falloff de área é uma
função-calibre, então o alcance da silhueta é `1/f(d)` em forma fechada — com a escala do W6 e o
offset do collider dentro. Medido na cena 67: pescoço `3,35 → 3,50` (a junta), pernas `2,58 → 2,50`.
Vale para as TRÊS rotas de criação, porque *"onde estes dois se encontram"* é uma pergunta só.

**(c) O rig nasce com BATENTES, e o número é medido.** Sem eles o boneco dobra a cabeça **176°** para
dentro do peito — o ragdoll-macarrão. A varredura (3 s de queda, pior ângulo relativo por junta) está
no doc do `RIG_LIMIT_DEG`; **±60°** é a maior faixa em que TODA junta é de fato limitada, e o
desabamento fica mais vivo, não menos (3,42 m contra 3,30 sem batente). ⚠️ A faixa é simétrica em
torno da pose **AUTORADA**, e isso sai de graça do `axis_locals`. ⚠️ E ela **não** vale para um tipo
cujo limite é uma DISTÂNCIA: num Slider `±60°` viraria **±1,05 metro** de curso.

⚠️ **A decisão de (c) é do RIG, não do Pin:** o botão *Join* faz um joint e o artista já está olhando
para a §12; este faz cinco de uma vez, e afinar N juntas à mão é a labuta que o wizard existe para
remover. As duas rotas continuam duas respostas para duas perguntas diferentes.

### §11.9 — Três defeitos MEUS que a medição pegou nesta rodada

1. **O oráculo da cena virou mentira quando a âncora mudou.** Ele media a distância entre os CENTROS
   como proxy de *"o joint segura"*; com o pivô no pescoço a cabeça GIRA em torno dele e essa distância
   varia de 0,3 a 0,7 m por geometria pura. Reportou 0,35 m e lia-se como *"soltou"*. O oráculo certo é
   a **violação da restrição** (a distância entre as duas âncoras), invariante sob rotação porque ela
   **É** a restrição: medida, **0,00002 m**.
2. **Uma afirmação minha ficou falsa no mesmo commit.** O doc do gerador dizia que a ordem entre dar
   corpo e ligar não importava (componentes disjuntos) — verdade até a emenda passar a medir o
   `Collider`, que ainda estava na fila. O sintoma não seria erro: seria a emenda caindo no fallback do
   ponto médio, em silêncio. O flush entre as duas metades mudou-se para dentro do gerador.
3. **Um gate meu não podia falhar pelo motivo que alegava.** O do Slider pedia `limit_max < 2.0`, e 60°
   em radianos é **1,047** — que passa folgado, e 1,047 metro de curso É o defeito. A mutação o
   encontrou. Virou a PROPRIEDADE: um trilho rigado não tem batente ligado.

### §11.10 — Aberto

- Um ragdoll de verdade quer limites **por parte do corpo** (um joelho não é um pescoço). O Fyrox
  adivinha porque conhece nomes de osso; nós não temos esse vocabulário, e inventá-lo seria adivinhar.
  O que o rig entrega é uma faixa uniforme medida, e a W-JointCopy torna a afinação por-junta um gesto
  de dois cliques.

## §12 — W-SoftWeld: a solda que CEDE (2026-07-31)

O item **6** do §8.1, e o último numerado da lista.

### O vão, e por que ele é o espelho do Rod

Este conjunto sabia segurar um ângulo de dois jeitos e só dois: **absoluto** (Weld, Slider) ou
**livre** (Spring, Rope, Rod, o giro do Wheel). Não havia nada no meio. Um poste que balança no vento,
um pescoço que resiste mas cede, uma placa que treme sob impacto — nenhum era exprimível, e a
justificativa do Rod (*"nada segurava dois corpos a distância fixa deixando os dois GIRAREM"*) é a
mesma frase no outro eixo.

### As duas medições que decidiram o desenho

**(1) Que eixos ficam moles.** A receita que o §8 prescrevia — *"não travar nada + três molas em alvo
0"* — foi construída e **reprovada pela medição**: sob o próprio peso o braço deriva **0,92 m** para
longe da parede e balança **104°** pico-a-pico sem nunca assentar. As peças vêm APART, e isso se lê
como a solda **falhando**, não vergando. O que shipou trava `LIN_X`+`LIN_Y` e põe UM motor de posição
no `AngX`: separação medida **`0,0000 m`** em toda a varredura, e só o ângulo cede.

**(2) O ganho angular.** A rigidez linear é N/m e a angular é N·m/rad — não são a mesma grandeza, e o
knob do artista é um só. Varrido com os defaults dele (`k=30`, `d=0,5`):

| ganho | pendor | balanço |
|---|---|---|
| 1 | 31,6° | **77,2°** ← nunca assenta |
| 10 | 10,0° | 0,0° ← o joelho, sem margem |
| **20** | **5,3°** | **0,0°** |
| 200 | 0,5° | 0,0° ← indistinguível de rígido |

`SOFT_WELD_ANGULAR_GAIN = 20` é o dobro do joelho, e com ele a **faixa inteira** do artista assenta
(stiffness 1 → 65,5° · 1000 → 0,16°, balanço 0,000 em todos).

⚠️ **O pendor NÃO depende da carga** — o motor de posição do rapier é normalizado pela massa, e uma
varredura de 100× (0,05 kg → 5 kg) mediu os MESMOS 31,6°. Gravidade e torque restaurador escalam
juntos e a massa cancela.

### Onde o flag mora, e por que é um campo

`PhysicsJoint.soft: bool`, apendado (**`PROJECT_SCHEMA` 46→47**), e **não** um componente-marcador — que
seria o idioma desta linha (`Ccd`, `LockRotation`, `OneWayPlatform`) e custaria zero bump. O que decidiu
foi o **copy/paste de propriedades** (W-JointCopy): ele desestrutura `PhysicsJoint` **exaustivamente**,
então um campo novo **não compila** até ser classificado — e um marcador escaparia do paste em silêncio.
A dureza reusa a `stiffness`/`damping` que a mola já carregava, então é UM bool e não três campos.

### A consequência que a medição achou de carona

O **break torque** passa a alcançar uma solda mole: rapier publica a reação de um eixo *motorizado* e
nada de um *travado*, e o `soft` é exatamente o que troca um pelo outro — medido, **0,9619 N·m** contra
**0,0000**. A nota do `JointKind::breaks_on_torque` que dizia *"uma Weld lê 0,0000"* ficou meia-verdadeira
e foi **reconferida**; a pergunta que o painel e a ponte fazem passou a ser a da INSTÂNCIA
(`PhysicsJoint::breaks_on_torque`). É o caso do Wheel outra vez: *quem manda é o estado em que a row
pode ser alcançada*.

### O que NÃO mudou, de propósito

- **A solda rígida continua um `FixedJoint` de verdade** — exata e mais barata que um motor rigidíssimo.
  A chave seleciona restrições genuinamente diferentes, não um modo de UI.
- **`is_rigid_link` (IK/FK) segue incluindo o Weld**, e o `soft` não a muda: posar escreve `Transform`s,
  e uma solda mole segura os dois corpos JUNTOS — a mola governa o quanto o ângulo cede sob CARGA, não
  se as duas peças são a mesma peça.

### Números, gates e a mutação que sobreviveu

`physics_ecs_c9` **96 → 98 corpos**, hash `58b0bae0…` (debug ≡ release) — a lane nova é um caminho de
solver próprio. **8 mutações, 7 sangram**; a que sobreviveu (o `can_be_soft` da ponte) **acusou uma
afirmação minha, não um buraco**: `desc.soft` tem UM leitor no wrapper inteiro, então o guard é higiene
e o doc-comment foi corrigido para dizê-lo.

---

*Imagens: `~/Documentos/Recursos/UI_Reference/` (44). Relatórios integrais dos 5 agentes: na sessão de
2026-07-25. Superfície rapier conferida em `rapier2d-0.28.0/src/dynamics/joint/` local.*
