# Dossiê — Godot 4.x (2D): catálogo completo de nodes traduzido para COMPONENTES estilo Unity

> Levantamento para o PH2D (decisão do dono: componentes estilo Unity, não herança de nodes).
> Pesquisado em 2026-08-20 nas docs oficiais (docs.godotengine.org, stable = 4.x).
> **Nota de tradução:** no Godot, "adicionar um comportamento" = criar um node filho na árvore
> (a posição na hierarquia é parte da semântica). Na tradução para componente Unity-style, o node
> vira um componente do objeto de cena; onde a posição própria do node importa (ex.: CollisionShape2D
> tem offset próprio, RayCast2D tem origem própria), o componente precisa carregar um offset/transform
> local próprio como propriedade. Onde o Godot usa "node filho aponta para pai" (PathFollow2D dentro
> de Path2D), o componente vira uma referência a um recurso/objeto (ex.: componente PathFollower com
> campo "path"). Sinais viram eventos do componente; groups viram tags.
> **Iluminação 2D (PointLight2D, DirectionalLight2D, LightOccluder2D): ADIADO por decisão do dono
> (2026-08-20)** — listado no fim, sem priorização.

---

## A. Visual / Render

Fontes: https://docs.godotengine.org/en/stable/classes/class_sprite2d.html · class_animatedsprite2d.html · tutorials/2d/using_tilemaps.html · class_line2d.html · class_meshinstance2d.html · class_ninepatchrect.html · class_canvasgroup.html · class_canvasmodulate.html · class_backbuffercopy.html

### Sprite2D
- **O que faz:** desenha uma textura no mundo; suporta recorte de atlas e grade de frames de spritesheet sem animação.
- **Propriedades na UI:** `texture` · `hframes`/`vframes`/`frame`/`frame_coords` (grade de spritesheet) · `region_enabled`/`region_rect`/`region_filter_clip_enabled` (recorte de atlas com anti-bleeding) · `centered` · `offset` · `flip_h`/`flip_v` (+ herdado de CanvasItem: `modulate`, `self_modulate`, `z_index`, `y_sort`, material/shader, filtro de textura).
- **Código que elimina:** carregamento/gestão de textura, cálculo de UV de atlas, offset de pivô, espelhamento (sem duplicar assets), seleção de célula de spritesheet.
- **Compõe com:** AnimationPlayer (anima `frame`), corpo físico (visual do corpo), Skeleton2D.
- **Relevância 2D:** **alta** (o componente nº 1 de qualquer engine 2D).

### AnimatedSprite2D
- **O que faz:** sprite com animações de frames nomeadas, definidas num recurso `SpriteFrames` editado visualmente (importa pasta de imagens, define FPS e loop por animação).
- **Propriedades na UI:** `sprite_frames` (editor próprio embutido) · `animation` · `autoplay` · `frame` · `speed_scale` · `flip_h`/`flip_v`.
- **Métodos/eventos:** `play("nome")`, `pause()`, `stop()`; sinais `animation_finished`, `frame_changed`, `animation_changed`.
- **Código que elimina:** toda a máquina de troca de frames (arrays de textura, índice, timer de FPS, loop/one-shot, callback de fim de animação). Um `play("run")` substitui dezenas de linhas.
- **Compõe com:** CharacterBody2D (trocar animação por estado), sinais (encadear lógica no fim da animação).
- **Relevância 2D:** **alta**.

### TileMapLayer (+ recurso TileSet)
- **O que faz:** camada de tiles pintada em grade. O TileSet embute, POR TILE: shapes de colisão física, polígono de navegação, oclusor de luz, camadas de dados customizados (metadata consultável), terrenos/autotiling (modos Connect/Path), tiles animados e **tiles-cena** (um tile instancia uma cena inteira).
- **Propriedades na UI:** tileset, editor de pintura completo (pincel, retângulo, balde, terrenos), Y-sort por camada, múltiplas TileMapLayer reordenáveis.
- **Código que elimina:** instanciar/posicionar milhares de sprites, criar colisão por elemento, gerar navmesh do nível, autotiling manual (a lógica de vizinhança dos terrenos é o maior eliminador), animação de tiles, metadata de tile ("isso é gelo? é lava?") sem tabela paralela.
- **Compõe com:** física (colisão dos tiles é automática), NavigationAgent2D, Y-sort com personagens.
- **Relevância 2D:** **alta** (para jogos de tile é o coração da produtividade).

### Line2D (trail/linha)
- **O que faz:** desenha uma polilinha espessa com juntas, caps, gradiente de cor, curva de largura e textura ao longo — o node de trilhas/rastros/cordas visuais.
- **Propriedades na UI:** `points` (editável no viewport) · `width` · `width_curve` (Curve) · `default_color` · `gradient` · `texture`/`texture_mode` · `joint_mode` (sharp/bevel/round) · `begin_cap_mode`/`end_cap_mode` · `round_precision` · `antialiased`.
- **Código que elimina:** geração de mesh de polilinha (miter/bevel, UVs, caps) — trabalho de gráfica não-trivial. Trail = append de pontos por frame, só isso.
- **Compõe com:** qualquer objeto móvel (rastro), Path2D (visualizar caminho).
- **Relevância 2D:** **alta**.

### MeshInstance2D / MultiMeshInstance2D
- **O que faz:** desenha um mesh 2D arbitrário (geometria custom, ou sprite convertido em mesh para economizar fill-rate em texturas com muita área transparente). MultiMesh instancia N cópias com custo de 1 draw.
- **Propriedades na UI:** `mesh` · `texture`; o editor tem "Sprite2D → Convert to MeshInstance2D".
- **Código que elimina:** vertex buffers e batching manual para geometria custom ou vegetação/props em massa.
- **Relevância 2D:** **média** (otimização e geometria custom).

### NinePatchRect (9-slice)
- **O que faz:** escala uma textura preservando cantos, esticando/tilando bordas e centro (painéis, balões de fala, molduras).
- **Propriedades na UI:** `texture` · `patch_margin_left/top/right/bottom` · `region_rect` (atlas) · `draw_center` · `axis_stretch_horizontal`/`vertical` (Stretch / Tile / Tile Fit).
- **Código que elimina:** fatiar a textura em 9 e desenhar cada fatia com regra própria.
- **Relevância 2D:** **alta** para UI; média in-world.

### CanvasGroup
- **O que faz:** desenha todos os filhos como UM objeto, aí aplica transparência/blend ao conjunto — resolve o clássico "sobreposição de sprites translúcidos escurece a interseção".
- **Propriedades na UI:** `fit_margin` · `clear_margin` · `use_mipmaps`.
- **Código que elimina:** render-to-texture manual do grupo + composição.
- **Relevância 2D:** **média-alta** (fade de personagem multi-parte é dor universal).

### CanvasModulate
- **O que faz:** tinge TODOS os CanvasItems do canvas com uma cor (dia/noite, mood global). Um por canvas.
- **Propriedades na UI:** `color`.
- **Código que elimina:** propagar tint por toda a árvore de objetos.
- **Relevância 2D:** **média**. (Toca no sistema de iluminação 2D — parte disso está sob o **adiado**.)

### BackBufferCopy
- **O que faz:** copia uma região da tela para um buffer que shaders leem (`hint_screen_texture`) — habilita distorção, refração, heat-haze localizados.
- **Propriedades na UI:** `copy_mode` (Disabled/Rect/Viewport) · `rect`.
- **Código que elimina:** pipeline de captura de tela para efeitos de shader.
- **Relevância 2D:** **média** (infra de efeitos).

---

## B. Esqueleto / Deform 2D

Fonte: https://docs.godotengine.org/en/stable/tutorials/animation/2d_skeletons.html · class_skeletonmodification2d.html

### Skeleton2D + Bone2D + Polygon2D (rig cutout)
- **O que faz:** esqueleto 2D com hierarquia de ossos; `Polygon2D` mapeia um sprite em malha (UV editor, vértices internos em regiões de dobra) e é deformado por pesos pintados por osso (weight painting visual, branco=1/preto=0). Rest pose armazenada.
- **Propriedades na UI:** hierarquia de bones com rest pose; por polígono: skeleton alvo, pesos por osso (pintura), vértices internos, UVs.
- **Código que elimina:** TODO o skinning 2D (deformação de malha por matriz de ossos + pesos) e a ferramenta de autoria — é dos sistemas mais caros de construir do zero.
- **Compõe com:** AnimationPlayer/AnimationTree (animar poses dos ossos), IK.
- **Relevância 2D:** **alta** para animação cutout (estilo Spine); o PH2D já tem sculpt/deform — sinergia direta.
- **IK:** `SkeletonModification2D` (stack com CCDIK, FABRIK, TwoBoneIK, LookAt, PhysicalBones) existe mas está **marcado EXPERIMENTAL** na doc 4.x — o próprio Godot sinaliza API instável aqui.

---

## C. Partículas

Fonte: https://docs.godotengine.org/en/stable/classes/class_gpuparticles2d.html · class_cpuparticles2d.html

### GPUParticles2D (+ ParticleProcessMaterial)
- **O que faz:** sistema de partículas simulado na GPU. O comportamento (velocidade, gravidade, cor ao longo da vida, formas de emissão, turbulência, colisão, sub-emissores) vive no recurso `ParticleProcessMaterial` — zero shader escrito à mão.
- **Propriedades na UI:** `emitting` · `amount`/`amount_ratio` · `lifetime` · `one_shot` · `preprocess` (pré-aquecer) · `speed_scale` · `explosiveness` (0=contínuo, 1=burst) · `randomness` · `fixed_fps`/`interpolate`/`fract_delta` · `local_coords` (partícula segue ou fica no mundo) · `draw_order` · trails (`trail_enabled`, lifetime, sections) · `sub_emitter` · `collision_base_size` · `visibility_rect` · `seed`/`use_fixed_seed` · `texture` · `interp_to_end`. Sinal `finished` (one-shot).
- **Código que elimina:** o simulador inteiro + shader de partículas; burst vs contínuo; pré-warm; determinismo por seed.
- **Compõe com:** one_shot+`finished` para efeitos de impacto; sub-emitter (explosão → fagulhas).
- **Relevância 2D:** **alta**.

### CPUParticles2D
- **O que faz:** o mesmo modelo, simulado em CPU (compatibilidade/low-end); TODOS os parâmetros ficam inline no node (direção, spread, formas de emissão point/circle/rect/ring/pontos custom, acelerações linear/radial/tangencial, orbit, ramps de cor e escala, hue variation, damping). Sem trails/sub-emitters/colisão do GPU.
- **Relevância 2D:** **média** (fallback). Lição para o PH2D: a regra do §0.0 — o fallback CPU não pode definir o teto.

---

## D. Câmera de gameplay

Fonte: https://docs.godotengine.org/en/stable/classes/class_camera2d.html

### Camera2D
- **O que faz:** câmera que segue o dono, com suavização, margens de arrasto e limites de mundo — o "camera follow" completo por checkbox.
- **Propriedades na UI:** `anchor_mode` (centro/canto) · `zoom` · `offset` · `position_smoothing_enabled/speed` · `rotation_smoothing_enabled/speed` · `ignore_rotation` · drag: `drag_horizontal/vertical_enabled`, 4 margens, 2 offsets (dead-zone: câmera só anda quando o alvo sai da janela) · limites: `limit_left/top/right/bottom`, `limit_enabled`, `limit_smoothed` (clamp aos limites do nível, com suavização) · `process_callback` (idle/physics) · `enabled` · `custom_viewport` · gizmos de editor (`editor_draw_screen/limits/drag_margin`).
- **Código que elimina:** lerp de follow, dead-zone, clamp nas bordas do nível, transição suave entre câmeras (trocar `enabled` interpola) — o script de câmera que TODO projeto 2D escreve.
- **Compõe com:** RemoteTransform2D (seguir sem reparenting), Path2D (câmera em trilho), limites lidos do TileMap.
- **Relevância 2D:** **alta**.
- **LACUNA declarada:** **screen-shake NÃO é built-in** no Godot (faz-se via ruído no `offset`, com script/addon). Zonas de câmera e "confiner" por região (estilo Cinemachine) também não existem prontos — só os 4 limites globais. Oportunidade clara para o PH2D: shake com trauma/decay, confiner por polígono, zonas com prioridade e transições — tudo por UI.

---

## E. Física & colisão

Fontes: https://docs.godotengine.org/en/stable/tutorials/physics/physics_introduction.html · class_characterbody2d.html · class_rigidbody2d.html · class_area2d.html · class_animatablebody2d.html · class_raycast2d.html · class_shapecast2d.html · class_joint2d.html · class_pinjoint2d.html · class_groovejoint2d.html · class_dampedspringjoint2d.html

### StaticBody2D
- **O que faz:** corpo imóvel que colide (chão, parede). Tem `constant_linear_velocity`/`constant_angular_velocity`: transmite velocidade SEM se mover — esteiras rolantes e plataformas giratórias de graça.
- **Propriedades na UI:** camadas/máscara de colisão, `physics_material_override` (atrito, quique, rough, absorbent), velocidades constantes.
- **Relevância 2D:** **alta**.

### AnimatableBody2D
- **O que faz:** corpo estático MOVIDO por animação/código (plataforma móvel, porta) que empurra corretamente os outros corpos: estima velocidade linear/angular a partir do movimento e a transmite. `sync_to_physics` sincroniza o movimento animado ao frame de física.
- **Código que elimina:** o notório bug "plataforma móvel que não carrega o jogador" — resolvido por design.
- **Compõe com:** AnimationPlayer/PathFollow2D (mover a plataforma), CharacterBody2D (que herda a velocidade da plataforma via `platform_on_leave`).
- **Relevância 2D:** **alta**.

### RigidBody2D
- **O que faz:** corpo com simulação completa (forças, torque, sono, colisão contínua).
- **Propriedades na UI:** `mass` · `inertia` · `center_of_mass_mode` · `gravity_scale` · `linear/angular_velocity` · `linear/angular_damp` (+modo) · `lock_rotation` · `freeze`/`freeze_mode` · `can_sleep` · `continuous_cd` (anti-túnel) · `contact_monitor`/`max_contacts_reported` (liga sinais de colisão) · `custom_integrator` · `physics_material_override` · `constant_force`/`constant_torque`.
- **Métodos/eventos:** `apply_force/impulse/torque…`; sinais `body_entered/exited`, `body_shape_entered/exited`, `sleeping_state_changed`.
- **Código que elimina:** integração, resolução de contato, resposta a colisão — e o detalhe de UI: `lock_rotation` e `freeze` como checkbox evitam gambiarras comuns.
- **Relevância 2D:** **alta** (o PH2D já tem rapier2d — aqui o valor é o EMPACOTAMENTO em componente com essas propriedades).

### CharacterBody2D — o quase-controller
- **O que faz:** corpo cinemático com API de alto nível para personagens: `move_and_slide()` com detecção de chão/parede/teto, rampas, snap, plataformas móveis.
- **Propriedades na UI:** `motion_mode` (**GROUNDED** = platformer / **FLOATING** = top-down — um dropdown muda a semântica!) · `up_direction` · `velocity` · piso: `floor_max_angle`, `floor_snap_length`, `floor_constant_speed`, `floor_stop_on_slope`, `floor_block_on_wall` · parede/teto: `wall_min_slide_angle`, `slide_on_ceiling` · plataformas: `platform_floor_layers`, `platform_wall_layers`, `platform_on_leave` (herda velocidade ao sair: add / add-upward / nada) · `safe_margin` · `max_slides`.
- **Métodos:** `move_and_slide()`, `is_on_floor/wall/ceiling[_only]()`, `get_floor_normal/angle()`, `get_wall_normal()`, `get_platform_velocity()`, `get_slide_collision*()`, `get_real_velocity()`, `apply_floor_snap()`.
- **Código que elimina:** clasificação chão/parede/teto, matemática de rampa (parar vs deslizar, velocidade constante em subida), snap ao descer, multi-slide, herança de velocidade de plataforma — os 6 problemas que consomem as primeiras semanas de qualquer platformer.
- **O que NÃO elimina:** o loop de input→aceleração→pulo (o usuário ainda escreve ~15 linhas de gravidade/pulo). Ver lacuna F.
- **Relevância 2D:** **alta** — provavelmente o node mais valioso do Godot 2D.

### Area2D (sensor/zona)
- **O que faz:** região que detecta sobreposição (corpos e áreas) e emite eventos; e SOBRESCREVE física local: gravidade (direcional OU pontual com falloff), damping, prioridade de empilhamento; e troca o bus de áudio de quem está dentro.
- **Propriedades na UI:** `monitoring`/`monitorable` · `priority` · gravidade: `gravity_space_override`, `gravity`, `gravity_direction`, `gravity_point` (+centro, +unit_distance p/ inverso-quadrado) · `linear/angular_damp_space_override` (+valores) · `audio_bus_override`/`audio_bus_name`.
- **Eventos:** `body_entered/exited`, `area_entered/exited` (+ variantes por shape).
- **Código que elimina:** TODOS os triggers de gameplay (coletável, dano, checkpoint, porta, água com empuxo, zona de gravidade invertida, poço gravitacional) — overlap + evento + física local por checkbox.
- **Relevância 2D:** **alta** (com CharacterBody2D, o par que mais elimina código).

### CollisionShape2D / CollisionPolygon2D
- **O que faz:** dá forma aos corpos/áreas: Rectangle, Circle, Capsule, Segment, Separation ray, World boundary, ConvexPolygon, ConcavePolygon; o Polygon edita vértices no viewport (com decomposição automática em convexos). Propriedades: `shape`, `disabled`, `one_way_collision` (+`one_way_collision_margin`) — **plataforma atravessável por baixo é um checkbox**, `debug_color`.
- **Código que elimina:** definição de geometria de colisão e o caso especial one-way (que é chato de escrever à mão).
- **Relevância 2D:** **alta**.

### Sistema de layers/masks (32 camadas nomeáveis)
- `collision_layer` (onde estou) × `collision_mask` (o que vejo), com NOMES definidos no projeto e grade de checkboxes na UI. Elimina os `if` de filtragem de colisão. **Alta.**

### RayCast2D
- **O que faz:** raio persistente atualizado a cada frame de física, configurado no editor (a seta é visível no viewport).
- **Propriedades na UI:** `target_position` · `enabled` · `exclude_parent` · `collide_with_areas/bodies` · `collision_mask` · `hit_from_inside`.
- **Métodos:** `is_colliding()`, `get_collider()`, `get_collision_point/normal()`.
- **Código que elimina:** consultas manuais ao espaço de física; casos: sensor de chão/borda, linha de visão, mira, "tem parede à frente?".
- **Relevância 2D:** **alta**.

### ShapeCast2D
- **O que faz:** varre uma FORMA (não um ponto) ao longo de um vetor; devolve múltiplas colisões e frações safe/unsafe.
- **Propriedades na UI:** `shape` · `target_position` · `margin` · `max_results` · máscara/filtros idem RayCast2D.
- **Relevância 2D:** **média-alta** (detecção "gorda" de personagem, ataques com área).

### Joints: PinJoint2D · GrooveJoint2D · DampedSpringJoint2D
- **Base (Joint2D):** `node_a`/`node_b` · `bias` · `disable_collision`.
- **PinJoint2D:** pino rotativo com `softness`, limites angulares (`angular_limit_enabled/lower/upper`) e **motor** (`motor_enabled`, `motor_target_velocity`) — pêndulo, gangorra, roda motorizada sem uma linha de código.
- **GrooveJoint2D:** trilho/pistão (`length`, `initial_offset`).
- **DampedSpringJoint2D:** mola amortecida (`length`, `rest_length`, `stiffness`, `damping`) — suspensão, ponte de cordas.
- **Código que elimina:** solvers de restrição; correntes/ragdolls/veículos por composição de joints no editor.
- **Relevância 2D:** **alta** (o PH2D já tem joints como entidades — validação da abordagem; o motor no pino é o detalhe a copiar).

### PhysicsMaterial
- Recurso com `friction`, `bounce`, `rough`, `absorbent` — compartilhável entre corpos. **Alta** (barato e útil).

---

## F. Character controllers PRONTOS

- **LACUNA declarada:** Godot **NÃO** tem controllers prontos (platformer completo com pulo/coyote/buffer, top-down, 8-direções, veículo). O CharacterBody2D resolve a METADE de baixo (colisão/rampa/plataforma), mas gravidade, pulo, aceleração, coyote time, jump buffering, dash — o usuário escreve (a doc oficial traz templates de código, não componentes). Engines como Construct/GameMaker (GML Visual) entregam isso pronto.
- **Oportunidade PH2D:** componentes `PlatformerController`, `TopDownController`, `VehicleController` COM UI (curva de pulo desenhável, coyote/buffer em ms, curvas de aceleração) em cima do rapier — o PH2D já tem `ph2d-platformer` (lei pura, 3 modos): está À FRENTE do Godot aqui; falta o empacotamento como componente adicionável + UI.

---

## G. Caminhos / splines

Fonte: https://docs.godotengine.org/en/stable/classes/class_pathfollow2d.html

### Path2D + PathFollow2D
- **O que faz:** Path2D guarda uma `Curve2D` (bézier editável no viewport); PathFollow2D anda pela curva e carrega os filhos, rotacionando-os pela tangente.
- **Propriedades na UI:** `progress` (px) · `progress_ratio` (0–1) · `h_offset`/`v_offset` · `rotates` · `cubic_interp` · `loop`.
- **Código que elimina:** amostragem de curva por comprimento de arco (não-trivial!), orientação pela tangente, wrap de loop. Mover algo num trilho = animar UM float (`progress_ratio` 0→1 num AnimationPlayer/Tween).
- **Compõe com:** AnimatableBody2D (plataforma em trilho), Camera2D (câmera em trilho), inimigos patrulha, cutscenes.
- **Relevância 2D:** **alta**.

---

## H. Animação

Fontes: https://docs.godotengine.org/en/stable/classes/class_animationplayer.html · tutorials/animation/animation_tree.html · classes/class_tween.html

### AnimationPlayer
- **O que faz:** timeline keyframada que anima QUALQUER propriedade de QUALQUER objeto (posição, cor, frame de sprite, propriedade de shader, bool…), mais trilhas de CHAMADA DE MÉTODO, trilhas de ÁUDIO e trilhas que tocam outras animações. É simultaneamente o sistema de animação E o sequencer de cutscenes.
- **Propriedades na UI:** biblioteca de animações; `autoplay` · `speed_scale` (negativo = reverso) · `playback_default_blend_time` · `movie_quit_on_finish`.
- **Métodos/eventos:** `play()`, `play_backwards()`, `queue()`, `seek()`, `pause()`, `stop()`; sinais `animation_finished`, `animation_changed`.
- **Código que elimina:** interpolação manual de propriedades, sequências roteirizadas (cutscene = animação com trilha de método chamando gameplay), sincronização de som com animação.
- **Relevância 2D:** **alta** — o PH2D já tem timeline própria; o delta relevante do Godot: trilha de método (callback no tempo), trilha de playback aninhado, e "qualquer propriedade de qualquer componente é keyframável por default".

### AnimationTree
- **O que faz:** grafo de blending sobre as animações do AnimationPlayer: **state machine visual** (transições com condição, auto-advance, xfade, e `travel()` que acha o caminho entre estados via A*), **BlendSpace1D/2D** (blend direcional: andar/correr por velocidade; idle-run 8 direções por vetor), Blend2/3, OneShot (ataque por cima do movimento), TimeScale/TimeSeek, root motion.
- **Código que elimina:** a state machine de animação inteira (o espaguete de `if state == ...` que todo jogo acumula), crossfades, blending direcional.
- **Relevância 2D:** **alta**.

### Tween (procedural, por código — mas 1 linha)
- **O que faz:** animação procedural fluente criada em runtime: `create_tween().tween_property(obj, "position", alvo, 0.5).set_trans(Tween.TRANS_ELASTIC)`.
- **Recursos:** PropertyTweener, IntervalTweener (delay), CallbackTweener, MethodTweener, SubtweenTweener, AwaitTweener; encadeamento sequencial por default, `parallel()`; 12 curvas (LINEAR, SINE, QUAD, CUBIC, QUART, QUINT, EXPO, CIRC, ELASTIC, BOUNCE, BACK, SPRING) × 4 eases (IN/OUT/IN_OUT/OUT_IN); `set_loops()`, `set_speed_scale()`; sinais `finished`, `loop_finished`, `step_finished`.
- **Código que elimina:** todo lerp manual em `_process` (juice: pop de coleta, recuo de dano, fade, shake simples).
- **Relevância 2D:** **alta** (para o PH2D: um componente/recurso "Tween preset" com UI eliminaria até a linha de código que o Godot ainda exige).

---

## I. Timeline / sequencer

- Godot **não tem node separado de sequencer**: o AnimationPlayer É o sequencer (trilhas de método + áudio + animação aninhada cobrem cutscenes). Declarado: categoria coberta por H, sem componente dedicado.
- **PH2D:** a timeline própria com sinais (ADR-0143) já é o equivalente — o delta é garantir trilha de callback/método e trilha de áudio.

---

## J. Áudio

Fonte: https://docs.godotengine.org/en/stable/classes/class_audiostreamplayer2d.html

### AudioStreamPlayer2D (posicional) / AudioStreamPlayer (global)
- **O que faz:** toca um stream com atenuação por distância e pan estéreo pela posição na tela.
- **Propriedades na UI:** `stream` · `volume_db` · `pitch_scale` · `autoplay` · `max_distance` · `attenuation` (expoente do falloff) · `max_polyphony` (N sons simultâneos do MESMO player — sem cortar o anterior) · `panning_strength` · `bus` (roteia para o mixer) · `area_mask` (Area2D pode trocar o bus = reverb de caverna por zona) · `playback_type`. Sinal `finished`.
- **Código que elimina:** cálculo de atenuação/pan, gestão de vozes (polifonia), roteamento por zona.
- **Compõe com:** Area2D (bus override por região), AnimationPlayer (trilha de áudio).
- **Relevância 2D:** **alta** (o PH2D tem o rack de 42 efeitos — falta exatamente ESTE componente de cena: emissor posicional com atenuação).

### AudioListener2D
- **O que faz:** move o "ouvido" para fora do centro da tela (`make_current()`); útil com câmera desacoplada do player.
- **Relevância 2D:** **média**.

---

## K. Input (mapeamento de ações)

Fonte: https://docs.godotengine.org/en/stable/tutorials/inputs/inputevent.html

- **InputMap** (nível de projeto, não componente): ações nomeadas ("jump", "shoot") mapeadas a N teclas/botões/eixos com deadzone, na UI de Project Settings. Código consulta `Input.is_action_pressed("jump")`, `get_axis()`, `get_vector()` (este último devolve o vetor 2D normalizado de 4 ações — o movimento 8-direções em 1 linha).
- **Código que elimina:** condicionais por dispositivo, suporte simultâneo teclado+gamepad, remapeamento em runtime.
- **Declarado:** é configuração de projeto + singleton, **não** um componente de objeto. (Unity tem o Input System com PlayerInput component; Godot não tem o componente.)
- **Relevância 2D:** **alta** (fundamento).

---

## L. UI in-game

Fonte: https://docs.godotengine.org/en/stable/tutorials/ui/index.html

- Árvore separada de **Control** nodes: Button, Label, TextureRect, ProgressBar/TextureProgressBar (vida!), LineEdit/TextEdit, containers (HBox/VBox/Grid/Margin/Scroll/Tab), popups; sistema de âncoras (layouts responsivos) e temas (skinning global).
- **Código que elimina:** layout manual de HUD/menus, escala por resolução, estilização repetida.
- **TouchScreenButton** (Node2D, não Control): botão de MUNDO com multitouch nativo — `texture_normal/pressed`, `bitmask`/`shape` (área de toque), `action` (dispara ação do InputMap!), `passby_press`, `visibility_mode` (esconde em desktop); sinais `pressed`/`released`. Controles mobile sem gerenciar toques à mão.
- **Relevância 2D:** **alta** (HUD é obrigatório); TouchScreenButton **média** (mobile).

---

## M. Navegação & AI

Fonte: https://docs.godotengine.org/en/stable/tutorials/navigation/navigation_introduction_2d.html

### NavigationRegion2D
- **O que faz:** área navegável (NavigationPolygon; com baking geométrico); o servidor COSTURA regiões adjacentes automaticamente num mesh combinado.
- **Código que elimina:** construção e junção de navmesh.
- **Relevância 2D:** **alta**.

### NavigationAgent2D
- **O que faz:** pathfinding + desvio de agentes (RVO) por componente: define `target_position`, consome `get_next_path_position()` — o resto (A*, repath, avoidance recíproco entre N agentes) é automático.
- **Propriedades na UI:** `path_desired_distance`, `target_desired_distance`, `radius`, `max_speed`, `avoidance_enabled`, camadas de navegação.
- **Código que elimina:** A*, suavização de caminho, desvio dinâmico entre agentes — meses de trabalho.
- **Relevância 2D:** **alta**.

### NavigationObstacle2D
- **O que faz:** obstáculo dinâmico que os agentes desviam (sem alterar o pathfinding estático). **Média-alta.**

### NavigationLink2D
- **O que faz:** conecta dois pontos distantes do mesh (pulo, teleporte, escada) — o path passa por ali e o jogo anima a travessia. **Média-alta.**

### Behavior trees / FSM de gameplay
- **LACUNA declarada:** Godot **NÃO** tem behavior tree nem FSM de gameplay no core — o ecossistema usa addons (LimboAI, Beehave). O AnimationTree tem state machine, mas de ANIMAÇÃO.
- **Oportunidade PH2D:** um componente de state machine de GAMEPLAY com UI (estados+transições+condições) seria diferencial real sobre o Godot.

---

## N. Spawn / factory / pooling

- **LACUNA declarada:** não há node de spawner ou pooling single-player. O idioma é `preload("cena.tscn").instantiate()` + `add_child()` — código, sempre. (MultiplayerSpawner é só replicação de rede.)
- **Oportunidade PH2D:** componente `Spawner` (o quê, onde — Marker2D/área/path —, cadência, limite, pool) por UI. Engines de "no-code" (Construct) provam o valor.

---

## O. Timers

Fonte: https://docs.godotengine.org/en/stable/classes/class_timer.html

### Timer
- **O que faz:** contagem regressiva com evento.
- **Propriedades na UI:** `wait_time` · `one_shot` · `autostart` · `paused` · `process_callback` (idle/physics) · `ignore_time_scale`; `time_left` read-only; `start()`/`stop()`; sinal `timeout`.
- **Código que elimina:** acumular delta à mão para cooldowns, respawns, ondas. Trivial de implementar, MUITO usado — o caso perfeito de componente barato de fazer e de altíssimo uso.
- **Relevância 2D:** **alta**.

---

## P. Rede / multiplayer

Fontes: https://docs.godotengine.org/en/stable/tutorials/networking/high_level_multiplayer.html · classes/class_multiplayerspawner.html · class_multiplayersynchronizer.html

### MultiplayerSpawner
- **O que faz:** replica automaticamente spawn/despawn de cenas da autoridade para os peers: adicionar filho no `spawn_path` já replica.
- **Propriedades na UI:** `spawn_path` · lista de cenas spawnáveis · `spawn_limit` · `spawn_function` (spawn custom com dados); sinais `spawned`/`despawned`.
- **Código que elimina:** RPCs de criação/destruição sincronizada de objetos.
- **Relevância 2D:** **alta SE multiplayer estiver no escopo**; senão baixa.

### MultiplayerSynchronizer
- **O que faz:** sincroniza propriedades marcadas (UI de replicação: escolhe propriedades numa lista) da autoridade para os peers; suporta intervalo, delta-updates e **filtros de visibilidade por peer** (interest management).
- **Propriedades na UI:** `replication_config` (editor próprio) · `replication_interval`/`delta_interval` · `public_visibility` · `visibility_update_mode` · `root_path`; sinais `synchronized`, `delta_synchronized`, `visibility_changed`.
- **Código que elimina:** o grosso do netcode de estado (broadcast de posição/vida/etc, com relevância por jogador).
- **Relevância:** idem acima. + RPCs por anotação `@rpc` e autoridade por node.

---

## Q. Persistência / save

- **LACUNA declarada:** **não há componente de save**. Existem APIs (`FileAccess`, `ConfigFile`, `ResourceSaver`, JSON) e um tutorial oficial de padrão (groups + serialização manual) — mas o usuário escreve tudo.
- **Oportunidade PH2D:** componente `Persistent` (marque o objeto, escolha as propriedades) + slot de save por UI. O PH2D já tem `ProjectState` snapshot-based — a infra de captura já existe; falta o recorte "runtime save de jogo".

---

## R. Parallax / scrolling

Fonte: https://docs.godotengine.org/en/stable/classes/class_parallax2d.html

### Parallax2D
- **O que faz:** camada com profundidade falsa amarrada à câmera; repetição infinita e autoscroll.
- **Propriedades na UI:** `scroll_scale` (fator de profundidade por eixo) · `scroll_offset` · `repeat_size`/`repeat_times` (tiling infinito) · `autoscroll` (nuvens/água sem câmera) · `limit_begin/end` · `follow_viewport` · `ignore_camera_scroll` · `screen_offset`.
- **Código que elimina:** rastrear câmera, calcular offsets por profundidade, tilar texturas para loop infinito.
- **Relevância 2D:** **alta** (todo side-scroller usa).

---

## S. Utilitários

Fontes: class_marker2d.html · class_remotetransform2d.html · class_visibleonscreennotifier2d.html · class_visibleonscreenenabler2d.html

### Marker2D
- **O que faz:** posição nomeada com gizmo em cruz no editor (`gizmo_extents`); spawn points, waypoints, ponto da arma. Elimina constantes de posição hardcoded. **Alta** (custo ~zero).

### RemoteTransform2D
- **O que faz:** empurra o transform do dono para OUTRO node (`remote_path`), com flags `update_position/rotation/scale` e `use_global_coordinates` — "pin" sem reparenting (câmera seguindo, item preso na mão do osso).
- **Código que elimina:** cópia de transform por frame e os bugs de reparenting. **Média-alta.**

### VisibleOnScreenNotifier2D
- **O que faz:** avisa quando um rect entra/sai da tela: sinais `screen_entered`/`screen_exited`, `is_on_screen()`; `rect` configurável.
- **Código que elimina:** teste manual contra o viewport; despawn de projéteis/inimigos fora da tela. **Alta.**

### VisibleOnScreenEnabler2D
- **O que faz:** DESLIGA o processamento do alvo (`enable_node_path`, default pai) fora da tela e religa ao entrar (`enable_mode`: inherit/always/when_paused) — culling de lógica por checkbox, "inimigos só acordam quando o player chega".
- **Código que elimina:** gestão manual de ativação por proximidade. **Alta** (perf de graça).

### Utilitários estilo Construct (line-of-sight, drag&drop, wrap, fade, flash, sine, move-to, anchor, solid/jumpthru)
- **LACUNA declarada (parcial):** Godot cobre por composição, não por behavior pronto: line-of-sight = RayCast2D; solid/jumpthru = one_way_collision; fade/flash/sine/move-to = Tween/AnimationPlayer (mas exigem escrever o tween); wrap de tela, drag&drop de objeto de cena, pin = script curto. **Não existem como componentes de 1 clique.** Oportunidade PH2D: uma família de micro-behaviors (Fade, Flash, SineMotion, MoveTo, Wrap, DragDrop, LookAt) — baratos, e são exatamente "componentes que com sua UI reduzem a necessidade de programação".

---

## T. Sinais & Groups (mecanismo transversal)

Fontes: https://docs.godotengine.org/en/stable/getting_started/step_by_step/signals.html · tutorials/scripting/groups.html

- **Sinais:** observer pattern de 1ª classe. TODO componente builtin emite eventos (`body_entered`, `timeout`, `animation_finished`, `pressed`…), conectáveis PELA UI do editor (dock de sinais → método alvo, callback gerado). Sinais custom com argumentos; `await` de sinal. **Elimina:** polling, referências diretas cruzadas, event bus caseiro.
- **Groups:** tags multi-pertencimento por checkbox; `get_nodes_in_group("enemies")`, `call_group("guards", "enter_alert_mode")`. **Elimina:** listas manuais de referências e varreduras da árvore.
- **Tradução PH2D:** eventos ECS já são o idioma (ADR-0075); o delta do Godot é a UI — conectar evento→método CLICANDO. Isso é "reduzir programação via UI" em estado puro. **Alta.**

---

## U. Como o usuário cria um componente próprio

Fontes: https://docs.godotengine.org/en/stable/tutorials/scripting/gdscript/gdscript_exports.html · https://godotengine.org/article/godot-4-will-discontinue-visual-scripting/

- **Script anexado a node** (1 script por node; GDScript/C#): `extends CharacterBody2D` + código. Com `class_name` + ícone, o tipo aparece no diálogo de criação como um node nativo.
- **`@export`** — o multiplicador: variáveis exportadas viram UI de inspector automaticamente, com tipos ricos: `@export_range` (slider, conversão rad↔graus), `@export_enum` (dropdown), `@export_flags` (bitmask), `@export_file/dir` (browser), cor sem alpha, multiline, NodePath tipado, Resource, e organização `@export_group/subgroup/category`. **Um componente custom com UI decente custa só as anotações.**
- **Resources custom** (dados compartilháveis que aparecem no inspector), **`@tool`** (o script roda NO editor — preview vivo), **EditorPlugin** (ferramentas de editor completas).
- **Visual scripting: NÃO EXISTE no Godot 4** — VisualScript foi **removido do core** no 4.0 por nunca ter ganho tração (artigo oficial). Lição registrada: visual scripting genérico "nodes de fluxo de controle" fracassou; o que funciona no Godot é a UI DECLARATIVA por componente (inspector + sinais + state machines de animação). Para o PH2D — que já tem nodegraph de Motion — o caminho validado é grafos DE DOMÍNIO (motion, shader, partícula), não um "GDScript visual".

---

## Iluminação 2D — ADIADO (decisão do dono, 2026-08-20)

Listado sem priorização: **PointLight2D** (luz com textura), **DirectionalLight2D**, **LightOccluder2D** (+OccluderPolygon2D, sombras 2D), máscaras de luz por camada, `CanvasModulate` como escuridão-base do esquema de luz. Fonte: https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html

---

## ★ MATADORES DE CÓDIGO (ranking dos que mais eliminam programação só com UI)

1. **CharacterBody2D** — rampas, chão/parede/teto, snap, plataformas móveis, one-way: as primeiras semanas de qualquer platformer viram um dropdown (GROUNDED/FLOATING) + checkboxes.
2. **Area2D** — todo trigger de gameplay (coletável, dano, zona d'água, poço gravitacional, reverb por região) = shape + evento + overrides de física locais.
3. **TileMapLayer + TileSet** — nível inteiro COM colisão, navegação, autotiling por terreno, tiles animados e metadata, 100% pintado.
4. **AnimationPlayer** — keyframe de QUALQUER propriedade + trilhas de método e áudio = animação E cutscene sem código.
5. **AnimationTree** — a state machine de animação (o maior espaguete de todo projeto) vira grafo visual com condições, xfade e `travel()`.
6. **Camera2D** — follow suavizado + dead-zone + limites de nível: o script que todo projeto 2D reescreve, por checkbox.
7. **NavigationAgent2D (+Region/Link/Obstacle)** — A* + desvio recíproco de multidão: `target_position` in, `next_path_position` out.
8. **GPUParticles2D + ParticleProcessMaterial** — sistema de partículas GPU completo (trails, sub-emissores, colisão) sem uma linha de shader.
9. **MultiplayerSynchronizer + MultiplayerSpawner** — replicação de estado e de spawns por lista de propriedades marcadas na UI; o grosso do netcode.
10. **Tween** — juice procedural (12 curvas × 4 eases, encadeável) em 1 linha; mata os lerps manuais de `_process`.
11. **Skeleton2D + Polygon2D (rig/weights)** — skinning cutout completo com autoria visual de pesos.
12. **Path2D + PathFollow2D** — mover em trilho = animar um float; arco-comprimento e tangente resolvidos.
13. **Sinais conectados pela UI + Groups** — o "cabeamento" do jogo (evento→reação, broadcast por tag) sem event bus caseiro.
14. **Joints com motor (PinJoint2D etc.)** — pêndulo, roda motorizada, mola, pistão: física composta no editor.
15. **VisibleOnScreenEnabler2D / Notifier2D** — culling de lógica e despawn offscreen por checkbox.
16. **InputMap + `get_vector()`** — abstração de dispositivo + movimento 8-direções em 1 linha (nível projeto, não componente).

## Lacunas do Godot = oportunidades diretas do PH2D

- **Character controllers prontos** (platformer com coyote/buffer/curvas de pulo, top-down, veículo) — Godot dá o alicerce, não o produto; PH2D já tem `ph2d-platformer`, falta empacotar como componente com UI.
- **Câmera:** shake, confiner por polígono, zonas com prioridade/transições — nada builtin.
- **Behavior tree / FSM de gameplay** — só via addons (LimboAI/Beehave).
- **Spawner/pooling** como componente — inexistente.
- **Save/persistência** como componente — inexistente (só APIs).
- **Micro-behaviors de 1 clique** (fade, flash, sine, move-to, wrap, drag&drop, LOS pronto) — Construct tem, Godot não.
- **Visual scripting** — removido no 4.0; a lição é investir em UI declarativa por componente e grafos de domínio, não em fluxo de controle visual genérico.

## Fontes principais (por categoria)

- Física: https://docs.godotengine.org/en/stable/tutorials/physics/physics_introduction.html e class refs (characterbody2d, rigidbody2d, area2d, raycast2d, shapecast2d, joint2d, pinjoint2d, groovejoint2d, dampedspringjoint2d, animatablebody2d)
- Visual: class refs sprite2d, animatedsprite2d, line2d, meshinstance2d, ninepatchrect, canvasgroup, canvasmodulate, backbuffercopy; tilemaps: https://docs.godotengine.org/en/stable/tutorials/2d/using_tilemaps.html
- Esqueleto: https://docs.godotengine.org/en/stable/tutorials/animation/2d_skeletons.html · class_skeletonmodification2d.html (experimental)
- Partículas: class_gpuparticles2d.html · class_cpuparticles2d.html
- Câmera: class_camera2d.html · Paths: class_pathfollow2d.html · Parallax: class_parallax2d.html
- Animação: class_animationplayer.html · tutorials/animation/animation_tree.html · class_tween.html
- Áudio: class_audiostreamplayer2d.html
- Navegação: https://docs.godotengine.org/en/stable/tutorials/navigation/navigation_introduction_2d.html
- Multiplayer: tutorials/networking/high_level_multiplayer.html · class_multiplayerspawner.html · class_multiplayersynchronizer.html
- Utilitários: class_timer.html · class_marker2d.html · class_remotetransform2d.html · class_visibleonscreennotifier2d.html · class_visibleonscreenenabler2d.html · class_touchscreenbutton.html
- Input: tutorials/inputs/inputevent.html · UI: tutorials/ui/index.html
- Sinais/groups: getting_started/step_by_step/signals.html · tutorials/scripting/groups.html
- Componente próprio: tutorials/scripting/gdscript/gdscript_exports.html · remoção do VisualScript: https://godotengine.org/article/godot-4-will-discontinue-visual-scripting/
