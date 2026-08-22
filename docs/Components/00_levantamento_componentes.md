# Componentes de objeto — o levantamento das 10 engines e o cardápio canônico do PH2D

> **Pedido do Enio (2026-08-20, verbatim):** *"Vamos fazer uma pesquisa profunda nas mais
> importantes engines do mundo e vamos fazer um levantamento de tudo que podemos implementar para
> tornar nossa engine poderosa, robusta, fácil de usar, intuitiva, extraordinária. Componentes que
> facilitem a programação. Componentes que com sua UI consigam reduzir a necessidade de programação."*
>
> **Método (2026-08-20):** 13 agentes — 7 dossiês de engine lendo as docs oficiais (**Unity 6 ·
> Godot 4 · Unreal 5 · Construct 3 · GDevelop · GameMaker · Defold · Cocos Creator · Phaser ·
> Bevy + ecossistema Rust**), 1 inventário do repo, 4 sínteses cruzadas por domínio, 1 crítico de
> completude com verificação factual por WebSearch (5/5 afirmações decisivas confirmadas na doc
> oficial — [critica.md §3](pesquisa/critica.md)). O material integral (471 KB) está em
> [`pesquisa/`](pesquisa/) — **este doc é o roteador e o veredito; a evidência mora nas folhas.**
>
> **Premissas já decididas pelo dono (não re-litigar):**
> 1. Objetos de cena ganham componentes **no estilo Unity** (AddComponent), NÃO a herança de nodes
>    do Godot. (Os nodes 2D do Godot foram pesquisados e *traduzidos* para componentes —
>    [dossie_godot.md](pesquisa/dossie_godot.md).)
> 2. **Iluminação 2D: ADIADA** (Enio, 2026-08-20 — "ainda não sei o papel do 3D na engine").
>    Está listada (§9) sem prioridade.
> 3. **R1 (shell de jogo / play mode) segue adiado por decisão do Enio** — itens que dependem dele
>    estão marcados `dep. R1`.

---

## 📍 Índice — salte, não leia

| § | assunto |
|---:|---|
| **§0** | O achado central: o custo não é onde parece |
| **§1** | As 8 leis transversais do catálogo |
| **§2** | Catálogo — Movimento & Física de gameplay (36 itens) |
| **§3** | Catálogo — Visual & Câmera (37 itens) |
| **§4** | Catálogo — Interação & Fluxo: input, UI, áudio, tempo, animação, sinais, script (45 itens) |
| **§5** | Catálogo — Estrutura & Inteligência: prefab, spawn, tags, AI, navegação, save, rede (36 itens) |
| **§6** | Catálogo — Combate & o que caiu entre as cadeiras (17 itens do crítico) |
| **§7** | **TOP-20** — a ordem de implementação proposta |
| **§8** | Pontos cegos do levantamento inteiro (diálogo, localização de jogo, acessibilidade…) |
| **§9** | Luz 2D — tudo ADIADO, listado |
| **§10** | As folhas — o que ler para aprofundar |

**≈171 componentes catalogados.** Já existem no PH2D: **12 completos + ~35 parciais** (a fundação
é maior do que parece — §0). Prioridades: **P0** = espinha de qualquer engine de jogo · **P1** =
diferencial forte de facilidade · **P2** = depois · **horizonte** = anotado para não fechar portas.

---

## §0 — O achado central: o custo não é onde parece

Quatro medições do [inventário](pesquisa/inventario_ph2d.md) reordenam tudo:

1. **O PH2D já tem ≈91 tipos de componente persistíveis** (62 no `ph2d-ecs` + 32 na física + Sprite
   + LuauScript…), com um caminho de registro em que **persistência e undo saem DE GRAÇA**
   (registrar no `ComponentRegistry` → entra no snapshot → entra no diff do undo). Os passos
   "definir → registrar → persistir → undo" de um componente novo são mecânicos e baratos.
2. **O custo real é o passo 5: a UI.** Cada componente hoje exige uma seção ARTESANAL no
   `ph2d-panel-inspector`. Não existe UI genérica de "Add Component" nem derive de painel a partir
   do tipo. Um catálogo de ~150 componentes é inviável artesanalmente — e trivial com a infra.
   ⇒ **O item nº 1 do TOP-20 (§7) não é um componente: é o Inspector derivado do tipo + a UI
   "Add Component" + required components.** As 4 sínteses o declararam pré-requisito de forma
   independente.
3. **Módulos inteiros já vencem o mercado e só não têm FACHADA de componente:** a Timeline (só
   Unity Timeline e UE Sequencer competem; Bevy não tem NADA), as partículas (Motion Nodes
   GPU-resident, 4,19 M @ 3,6 ms — nenhuma engine 2D pesquisada chega perto), o platformer
   (`ph2d-platformer` entrega o que **Unity, Godot, GM, Defold, Cocos e Phaser não têm**), o rack
   de 42 efeitos de áudio, os sinais (`ph2d-runtime` com ordem gateada). O trabalho nesses casos é
   **empacotamento + trigger**, não motor.
4. **A lacuna nº 1 nomeada por todas as sínteses é a mesma: a tabela sinal→ação (R3).** Sinais já
   viajam (contato, marker de timeline, player) e NADA autorável reage. Com ela + Timer +
   AudioSource, "sinais viram gameplay sem código" — é o coração do pedido do Enio.

---

## §1 — As 8 leis transversais (destiladas das 4 sínteses; valem para TODO componente)

1. **Todo componente publica VOCABULÁRIO, não só propriedades** (lição Construct): condições,
   ações e expressions que sinais/tabela R3/Luau/timeline consomem. Painel configura; vocabulário
   compõe. No PH2D: cada componente **emite `Signal` nomeados e oferece ações à tabela R3**.
2. **`Default controls` + `Simulate control`** (lição Construct): componente que reage a input LÊ
   `ActionState`, nunca dispositivo — e o ActionState é escrevível por IA, replay, rede e teste.
   Um clique liga "anda sozinho com as setas" (o smoke auto-play que o Enio pediu de todo exemplo);
   desligar o bool vira motor puro.
3. **Marcadores são componentes** (Solid/Persist/NoSave/Replicated): zero campos, alavancagem
   enorme, custo ~zero.
4. **Required components** (Bevy 0.15 — o meta-matador de UX): "Add Component" insere o conceito
   dirigente e a engine completa as dependências em cascata. O erro "adicionei X, faltou Y, nada
   acontece" morre por construção.
5. **Referência durável entre objetos é o NOME** (`stable_name_id`) — lei já paga pelo undo do
   PH2D. Vale para TODO campo alvo/prefab/caminho.
6. **Determinismo é lei da casa:** BTreeMap sempre; Spawner/AI/partícula com **seed explícita**
   exposta na UI (o ring GGPO e o hash 3-OS já pagam esse preço — componente novo não o quebra).
7. **Um dono do transform por vez** (lição Construct, prática PH2D): todo mover declara seu modo
   de posse e o Inspector ACUSA conflito (dois movers ativos = badge, não comportamento indefinido).
8. **Micro-movers vs Motion Nodes: MEÇA a composição antes de construir** (lei §5.0 do CLAUDE.md).
   Sine/Orbit/Boids/Wiggle já existem como nodes GPU. O componente é o atalho de 1 clique que
   instancia o node — nunca um segundo motor para a mesma lei
   ([feedback_two_engines_one_state](../../project-memory/feedback_two_engines_one_state_is_worse_than_a_slow_engine.md)).

---

## §2 — Catálogo: MOVIMENTO & FÍSICA DE GAMEPLAY

> Detalhe por item (equivalentes por engine, notas de desenho, riscos):
> [sintese_movimento_fisica.md](pesquisa/sintese_movimento_fisica.md).
> A família de controllers prontos é a lacuna confessa de Unity/Godot/GM/Defold/Cocos/Bevy — e o
> PH2D **já está à frente no platformer**: completar a família é o maior diferencial do domínio.

| Componente | P | PH2D hoje | O que entrega |
|---|---|---|---|
| `RigidBody` + `Collider` + overrides | ✅ feito | sim (`ph2d-physics-ecs`, 32 comp.) | corpo + forma + o pacote completo de overrides |
| `PlatformPlayer` | ✅ feito | sim (`ph2d-platformer` + ponte) | platformer completo por componente — à frente do mercado |
| `PhysicsJoint` (9 kinds + polia/eixo) | ✅ feito | sim (joint = ENTIDADE) | deltas P2: `Gear`, `Friction`, `Target/Mouse` (medir se Spring+WorldAnchor já compõe) |
| Família `Area*` (7 zonas de força) | ✅ feito | sim | vento/arrasto/empuxo/torque/falloff por Inspector |
| `OneWayPlatform` · `WalkSurface` · `NoWallCling` | ✅ feito | sim | conferir drop-through por input (Construct `Fall through`) |
| `SensorZone` | **P0** | parcial (`is_sensor` sem consumidor; `SignalOnHit/Leave` prontos) | trigger universal: moeda, dano, porta, checkpoint = shape + evento |
| `RaySensor` | **P0** | não (rapier tem as queries) | raio persistente com gizmo: chão, parede, mira — copiar a REFLEXÃO do Construct |
| `TopDownPlayer` | **P0** | não | o 2º controller canônico; dropdown de viewpoint (top-down/isométrico) do GDevelop |
| `ProjectileMotion` | **P0** | não | bullet genérico: gravidade, ricochete pela normal, alcance, **homing embutido** (modelo UE) |
| `ShapeSensor` | P1 | não | shapecast persistente: detecção "gorda", golpe com área |
| `LineOfSight` | P1 | não | "A vê B?": alcance + cone + oclusão por camada + eventos |
| `PathFollow` ⭐ | P1 | parcial (paths JÁ são entidades; falta o seguidor) | **"desenhe a patrulha com a caneta"** — arc-length + tangente; nenhuma engine tem editor de path deste nível acoplado |
| `MoveTo` | P1 | não | vai-até com fila de waypoints e `On arrived`; UM executor, 3 fontes (ponto · A* · path) |
| `WaypointMotion` | P1 | parcial (Timeline exprime) | plataforma vai-e-volta OneShot/Loop/PingPong — o atalho de 30 s |
| `KinematicPlatform` | P1 | parcial (o player já herda plataforma) | corpo animado que EMPURRA e CARREGA — velocidade derivada do movimento autorado (Cocos `Animated`) |
| `TransformInterpolation` | P1 | não (⚠️ medir o present-world antes) | anti-jitter física@60 vs render@144+ |
| `TurretAim` | P1 | não | aquisição + rotação limitada + cadência + **mira preditiva** (a interceptação que ninguém quer escrever) |
| `PhysicsGrab` | P1 | não (medir Spring+WorldAnchor) | agarrar/arrastar corpo com o ponteiro mantendo a física viva |
| `ConveyorSurface` | P1 | não | esteira: velocidade tangencial na superfície (desenho Godot: transmite SEM se mover) |
| `AreaAttractor` → player | P1 | parcial (campo existe; **não alcança player de pose própria** — item aberto §5) | fechar a costura "o ímã puxa o herói" |
| `PinTo` | P1 | não | prender sem reparentar, canal a canal (pos/ângulo/escala) — modos Pull\|Push num só componente |
| `DespawnOutside` | P1 | não | higiene de balas/inimigos fora da tela — par do Projectile e do Spawner |
| `TileMotion` | P1 | parcial (`ph2d-grid` dá 11 grids + A*) | movimento casa-a-casa: roguelike/sokoban destravados |
| `VehiclePlayer` | P1 | parcial (Wheel joint + WestonAxle prontos) | carro arcade (drift do Construct) + montador do rig físico |
| `PhysicsMaterial` (asset) | P2 | parcial (números no Collider) | "Gelo"/"Borracha" nomeado e reusável |
| `CompositeCollider` | P2→P1 c/ Tilemap | não | funde colliders vizinhos — mata o ghost collision |
| `AreaGravityOverride` | P2 | não (medir composição) | zona que SUBSTITUI gravidade (semântica replace + prioridade, Godot) |
| `OrbitMotion` | P2 | parcial (Motion Node exprime) | candidato nº 1 a "preset de node empacotado" |
| `ScreenWrap` | P2 | não | Asteroids em 1 checkbox |
| `RegionBound` | P2 | não | clamp na região + EVENTO ao tocar a borda (Phaser) |
| `DelayFollow` | P2 | não (primo: `TapeWire`) | trenzinho/sombra/ghost-replay serializável |
| `ExplosionImpulse` | P2 | não | impulso radial one-shot acionável por Signal |
| `ConstantThrust` | P2 | parcial (provável campo no RigidBody) | força contínua local |
| `KinematicMotor` | P2 | parcial (Luau + rapier dão o cru) | movimento custom SEM perder depenetração/sub-passos |
| `BoidsFlock` | P2 | parcial (node GPU existe) | empacotar, nunca reescrever (lei 8) |
| `RotatingMotion` | P2 | ⚠️ medir Motion Nodes | a serra giratória (achado do crítico — §6.9) |

---

## §3 — Catálogo: VISUAL & CÂMERA

> Detalhe: [sintese_visual_camera.md](pesquisa/sintese_visual_camera.md). **Câmera de gameplay é a
> maior lacuna do PH2D — e de metade da indústria** (Godot sem shake/confiner; Cocos e Bevy sem
> NADA): é a categoria onde o PH2D pode passar todos com componentes + UI.

| Componente | P | PH2D hoje | O que entrega |
|---|---|---|---|
| `Sprite` v4 | ✅ feito | sim | tint cascateado + tint_fill (flash de dano) + sheet inline + pivot + KTX2 |
| Sorting suite (Layer/Group/YSort/…) | ✅ feito | sim (`ph2d-ecs`, 8 comp.) | = Unity + Godot somados |
| `Mask2D` / `ClipChildren` | ✅ feito | sim | máscara/clipping hierárquico |
| `OnScreenEnabler` | ✅ feito | sim | culling de lógica por checkbox (⚠️ escrever O QUE ele desliga quando houver gameplay) |
| `GameCamera` | **P0** | parcial (`Camera2d` é resource; `GameRt` HDR pronto) | a câmera como ENTIDADE: N câmeras, prioridade, transição |
| `CameraFollow` | **P0** | não | damping por eixo + dead-zone + lookahead + **multi-alvo com zoom-to-fit** (co-op de graça) |
| `CameraLimits` / `Confiner` | **P0** ret / P1 polígono | não | a JANELA presa à fase; polígono pode consumir um path do Vector |
| `AnimatedSprite` + `SpriteFrames` | **P0** | parcial (grade inline; **Flip é a autoria de flipbook DENTRO do app**) | clips nomeados + play("run") + eventos → Signal — a lacuna confessa da Unity |
| `TilemapLayer` + `TileSet` | **P0** | não (`ph2d-grid` = fundação; ⚠️ docs/Tilling é MVP gitignored FORA do repo) | nível pintado; por tile: sprite, colisão, metadata, animação, prefab. Import Tiled/LDtk = padrão de adoção |
| `TilemapCollider` | **P0** (junto) | não (física pronta) | colisão FUNDIDA num outline (anti-ghost) |
| `ParticleEmitter` (fachada) | **P0** | **parcial FORTE** (Motion Nodes GPU 4,19M @ 3,6 ms) | painel de módulos no objeto; "abrir como grafo" = poder total. ⚠️ é o módulo do caso §0.0 — cap só com tabela de medição |
| `SpriteDrawMode` (Sliced/Tiled/**Filled**) | P1 | não | nine-patch + barra de vida/cooldown radial como MODO do Sprite (1 enum, não 3 componentes) |
| `SpriteLibrary` + `SpriteResolver` | P1 | não | skins/equipamento/lip-sync por Categoria+Label keyframável |
| `AutotileTerrain` | P1 | não | rule tiles visuais — o maior matador de código da categoria tilemap |
| `ParallaxLayer` + `scroll_factor` | P1 | não | os DOIS idiomas: float por objeto (Phaser) + camada com repeat/autoscroll (Godot) |
| `LineRenderer` | P1 | parcial (baker `VecStrokeProfile` ADR-0148) | polilinha de gameplay — reusar o baker, não reimplementar |
| `TrailRenderer` | P1 | não | rastro com vida/fade — categoria SEM dono na maioria das engines; diferencial fácil |
| `Skeleton2d` + `Bone2d` | P1 | não (Timeline anima Transforms de graça) | **autorar rig + arte no MESMO app** — só Unity e Godot autoram; UE não tem 2D |
| `SpriteSkin` | P1 | não | deform por pesos pintados (grade-de-pontos do Construct = degrau barato) |
| `IkSolver2d` | P1 | parcial (ADR-0149: IK transitório na física) | reusar a lei; expor como componente |
| `CameraShake` + `ShakeEmitter` | P1 | não | modelo trauma-com-decay + fontes posicionais com falloff; casa 1:1 com o idioma Signal |
| `CameraZone` | P1 | não (molde: zonas da física) | troca de sala = entrar na zona; ausência generalizada no mercado |
| `CameraEffects` (fade/flash/pan/zoom-to) | P1 | não | as 4 ações de cutscene com callback |
| `PixelPerfectCamera` | P1 | não | ⚠️ projetar JUNTO do Follow (disputam o ortho size — a Unity precisou de extensão p/ reconciliar) |
| `RenderTargetSurface` | P1 | parcial (`GameRt` interno) | minimapa/retrato/portal — expor como componente + textura p/ Sprite |
| `OnScreenNotifier` | P1 | parcial (Enabler mudo) | dar VOZ: `screen_entered/exited` como Signal |
| `ObjectFx` (pilha por objeto) | P1 | parcial (`fx_stack` por camada) | glow/outline/dissolve por checkbox (benchmark: Phaser Filters) |
| `SpriteShape` (terreno por spline) | P2 | parcial (motor vetorial é base MELHOR que a da Unity) | profile ângulo→sprite + baker de colisão |
| `CanvasGroup` | P2 | não (`GameRt` = infra) | fade de multi-parte sem escurecer interseções |
| `SceneTint` | P2 | não | mood tint (a metade "escuridão de luz" fica ADIADA) |
| `BoneSocket` | P2 | não | arma na mão do osso — `stable_name_id` resolve a metade difícil |
| `CameraDolly` | P2 | não (curvas prontas no Vector) | câmera em trilho |
| Multi-câmera / split-screen | P2 | não | GM Views é a referência de UI |
| `MultiMesh`/static batch | P2 | ⚠️ medir `motion.clone` antes | props em massa (achado do crítico) |
| `BackBufferCopy` | P2 | não | captura regional p/ shader de distorção (achado do crítico) |
| `ShapePainter`/Gizmos de jogo | P2 | não | mira dinâmica, telegraph de ataque, debug draw (achado do crítico) |

---

## §4 — Catálogo: INTERAÇÃO & FLUXO

> Detalhe: [sintese_interacao_fluxo.md](pesquisa/sintese_interacao_fluxo.md). Três fatos mandam no
> domínio: **input é a maior lacuna bruta** (`ph2d-input` é gamepad cru); a família `Vec*` +
> Smart Animate já é fundação de UI **melhor que a de Godot/Unity**; e a Timeline só perde para
> Unity/UE — falta a PONTE para a cena de jogo.

| Componente | P | PH2D hoje | O que entrega |
|---|---|---|---|
| `SignalActions` (tabela R3) | **P0 ⭐** | ✖ tabela; ✅ TODO o transporte | **o item mais importante do levantamento**: "sinal X → ação Y" por dropdown (som, spawn, enable, tween, timer, cena, contador, outro sinal). ⚠️ nascer na crate certa (adjacência `ph2d-runtime`, §5 do CLAUDE.md) · payload tipado BTreeMap desde o dia 1 |
| `Timer` | **P0** | não | **a melhor razão custo/benefício do levantamento** (veredito unânime): nomeados, N por instância, `progress` 0–1, publica Signal |
| `ActionMap` (asset) + `ActionState` | **P0** | não | ações nomeadas multi-dispositivo + rebinding de graça; o ActionState por-entidade é escrevível por IA/replay/rede |
| `AudioSource2D` + `AudioListener2D` | **P0** | não (rack 100% editor-side) | som posicional: copiar `max_polyphony` (Godot) e `non_spatialized_radius` (UE, **verificado** — mata o "pan pulando") |
| `UiCanvas` + `UiAnchor` + `UiLabel` + `UiButton` | **P0** | parcial (`Vec*` + taffy + Smart Animate + `VecTextPath`) | o HUD mínimo: placar, vida, menu; o botão PUBLICA Signal (idioma ADR-0075, não callback) |
| `SpriteFrames`+`AnimatedSprite` | **P0** | parcial | (mesmo item do §3 — o pipeline de personagem começa aqui) |
| `SequencePlayer` | **P0** | parcial GRANDE (`TargetBinding`/WireId religável JÁ SHIPADO) | toca um TimelineDoc na cena: play/pause/seek, "play on signal", os 2 checkboxes da UE (desligar input, esconder HUD) |
| `ScriptProperties` + attach visual do LuauScript | **P0** | parcial (host+persistência prontos; **ZERO UI**) | anexar script pela UI + `go.property`-style: campos por instância no Inspector |
| `InputTriggers`/`Modifiers` | P1 | não | tap/hold/double-tap/**combo** (verificado: UInputTriggerCombo)/chord no BINDING |
| `InputContext` (pilha) | P1 | não | menu desliga o pulo sem um if; topo CONSOME input (modal de graça, Defold) |
| `PointerTarget` (picking de gameplay) | P1 | parcial (hit-test do editor maduro) | hover/click/drag&drop em objeto do mundo → Signal |
| `TouchButton` + `VirtualJoystick` | P1 | não | mobile: disparam AÇÕES do ActionMap (Godot), não callbacks |
| `UiProgressBar` | P1 | não | fill linear E radial como modo — casa com `Timer.progress` |
| `UiLayoutGroup` | P1 | parcial (`VecLayout*`/taffy gateado) | hotbar/inventário/menu sem posicionar à mão |
| `WorldSpaceWidget` | P1 | não | healthbar na cabeça / balão de fala (modelo UE WidgetComponent) |
| `UiFocusNav` | P1 | não | menu com gamepad: vizinhos automáticos + realce via Smart Animate |
| `AudioBus` (mixer runtime) | P1 | parcial (o rack de 42 efeitos vira cadeia de bus) | Master/Música/SFX + snapshots ("combate"/"exploração") + ducking |
| `MusicPlayer` | P1 | não | crossfade + intro/loop points + camadas sincronizadas (GM Sync Groups) — ⚠️ conferir os 3 Chesterton de `docs/Audio/03_o_que_falta.md` |
| `TimeChannel` | P1 | parcial (`Playhead` + porta `time` dos nodes) | pause com menu vivo, bullet-time por grupo — ⚠️ decidir CEDO a relação com o fixed-step do rapier, e gatear |
| `Tween` + `TweenPreset` | P1 | parcial (motor COMPLETO: `ph2d-anim` + vec-blend + OKLab) | anima qualquer propriedade registrada, disparado por Signal; stagger (Phaser) + OKLab = combinação que ninguém tem |
| `Fade` · `Flash` · `Oscillator` | P1 | parcial (`tint_fill` pronto; `motion.oscillator` como node) | presets de 1 clique (lei 8: açúcar sobre o motor, nunca 2º motor) |
| `AnimationNotify` | P1 | parcial (markers→Signal na timeline — estender ao flipbook) | "no frame 7, hitbox" — copiar NotifyStates com DURAÇÃO (janela, não instante) |
| `AnimationStateMachine` | P1 | parcial (`ph2d-ui-state::Machine` = embrião) | grafo estados→clips com travel() — ⚠️ decidir: a MESMA máquina do Smart Animate ou irmã (contagem dupla à espreita) |
| `ControllerAnimator` | P1 | não (`PlayerSignals` JÁ publica os estados) | estado do player → clip em 6 dropdowns — a ideia mais custo-eficiente do dossiê GDevelop |
| `PropertyTrack` | P1 | parcial | timeline keyframa QUALQUER propriedade registrada ("animar o fill da barra") |
| `ActivationTrack` · `AudioTrack` · `ControlTrack` | P1 | parcial (infra pronta) | ligar/desligar por clipe · som na timeline · **scrub de partículas** (Motion Nodes dirigidos pela timeline = demo imensa) |
| `SignalEmitter` family (`OnTimer`/`OnInput`/`OnScreenEnter`/`OnAnimEnd`/`OnClick`) | P1 | parcial (`SignalOnHit` é o molde) | completar o lado produtor com marcadores de 1–3 campos |
| `Tag` (grupos) | P1→**P0 no §5** | não | (ver §5 — a moeda de troca de tudo) |
| `ScriptEventHooks` | P1 | parcial (`MessageBus` é a metade de baixo) | script declara handlers por nome e PUBLICA ações no dropdown da R3 — script vira fornecedor de vocabulário |
| `LocalPlayerManager` | P2 | não | join-when-button-pressed + split-screen |
| `UiScrollView` · `UiModalBlocker` · `SafeArea` | P2 | não (Mask2D dá o recorte) | scroll com inércia · modal por PRESENÇA · notch mobile |
| `BlendSpace1D/2D` | P2 | não | idle→walk→run por velocidade; direcional para flipbook (PaperZD) |
| `TimeDilationTrack` | P2 | não | câmera lenta autorada |
| `AudioZone` | P2 | não (molde: zonas da física) | reverb de caverna / abafado na água |
| `GameplayGraph` (visual scripting) | P2 ⚠️ | infra pronta (`ph2d-nodegraph`) | **a lição medida: VisualScript do Godot foi REMOVIDO no 4.0** — fluxo genérico fracassa; tabela R3 + FSM + Luau-com-properties cobrem o espectro primeiro |
| `UserBehavior` (funil comunidade→catálogo) | P2 | não | empacotar {R3+FSM+Timer+Tween+properties} como componente nomeado — o multiplicador de longo prazo (a cauda do GDevelop é 6× a do Construct por isso) |

---

## §5 — Catálogo: ESTRUTURA & INTELIGÊNCIA

> Detalhe: [sintese_estrutura_ai.md](pesquisa/sintese_estrutura_ai.md). O **Spawner como
> componente** é a categoria que Unity, Godot e UE NÃO têm (só as no-code) — diferencial direto.
> E o trunfo de rede do PH2D já existe: determinismo 3-OS + ring GGPO ⇒ quando rede entrar,
> **rollback** é onde já estamos à frente.

| Componente | P | PH2D hoje | O que entrega |
|---|---|---|---|
| `Tags` (multi-grupo + consulta + broadcast) | **P0** | não (base: interning do MessageBus) | a moeda de troca de TUDO (spawner "onde", percepção "quem", broadcast, save-filter). **Hierárquico estilo Unreal (`enemy.flying`) desde o dia 1** |
| `PrefabAsset` (fluxo de editor) + `PrefabRef` | **P0** | parcial (`PrefabDoc`+`spawn_prefab` prontos, **ZERO UI**) | salvar seleção → browser → instanciar → override por campo (modelo Cocos: revert/apply/unlink) |
| `SpawnPoint` | **P0** | parcial (Name+Transform) | marcador nomeado com gizmo, consultável por tag |
| `Spawner` | **P0** | não (sementes: `SpawnQueue`, `SignalReader`) | o quê · onde · quando (taxa/burst/onda/**ao sinal**) · quanto; seed explícita; publica `on_spawned`/`exhausted` |
| `Lifetime` | **P0** | não | TTL trivial — sem ele Spawner e projétil VAZAM |
| `DestroyOutside` | **P0** | não | (par do §2 — mesma linha) |
| `StateMachine` (FSM de gameplay) | **P0** | parcial-embrião (`ph2d-ui-state::Machine`, `PlayerEvent`) | **o cérebro autorável mínimo** — enquanto o script não tem UI, é o único caminho de lógica para o artista. Estado-É-componente (seldom_state): animação/física/spawner reagem a estado por query |
| `PrefabVariant` | P1 | não | "GoblinArqueiro = Goblin + 4 campos" — o diff do undo já sabe comparar |
| `ObjectPool` | P1 | não | ⚠️ decisão Defold: a ENGINE faz o pool — medir se é sequer necessário com o ECS atual antes de expor knobs |
| `Team` (afiliação) | P1 | não | amigo/neutro/hostil + tabela de atitude — par obrigatório da percepção |
| `OnScreenNotifier` | P1 | parcial | (= §3 — dar voz ao Enabler) |
| `Persistent` (entre cenas) | P1 · dep. R1 | não | o player atravessa a troca; o baú lembra que abriu |
| `WorldRef` (proxy de mundo + relógio próprio) | P1 · dep. R1 · **pede ADR** | não (princípio provado: nesting ADR-0133) | streaming de fases + `time_step` por mundo (Defold) — o mais estrutural da lista: errar contamina save/física/áudio |
| `RaySensor` | P1 | não | (= §2; o módulo `sense` do platformer é o primo interno) |
| `SightSense` + `PerceptionSource` | P1 | não (padrão provado: `SignalOnHit`) | visão com cone + oclusão + **memória com esquecimento** + histerese (LoseSightRadius > SightRadius — os 2 campos que separam brinquedo de produto) |
| `NavSurface` (grade primeiro) + `NavObstacle` | P1 | parcial (`ph2d-grid`: **A\* determinístico PRONTO, sem consumidor**) | onde se anda; obstáculo = collider + auto-rebuild (estado da arte vleue_navigator — **verificado: avian, não rapier**, adaptar) |
| `NavAgent` | P1 | não | target entra, passo sai; RVO; publica `path_found`/`arrived` como Signal. O agente DECIDE; quem anda é o executor do §2 (MoveTo) |
| `BehaviorTree` + `Blackboard` | P1 | não (trunfo: o EDITOR do `ph2d-nodegraph` já existe — ⚠️ contrato congelado §6, decisão nodegraph-vs-folha = **ADR**) | patrulha→persegue→ataca clicável — categoria sem dono fora da UE |
| `SaveGame` (slots) + `SavePolicy` | P1 · dep. R1 | parcial-infra (snapshot canônico + registry + ring GGPO) | o jogador salva; Persist/NoSave por marcador; a tese moonshine (model salva, view recozinha) espelha "config, nunca estado de solver". ⚠️ save de jogo LANÇADO = degrau de compat próprio, além do PROJECT_SCHEMA |
| `PinTo` | P1 | não | (= §2) |
| `HearingSense` + `NoiseEmitter` | P2 | não (substrato: `SignalOrigin` espacial) | o guarda investiga o barulho |
| `UtilityBrain` | P2 | não | scorers + pesos em sliders (big-brain) |
| `TacticalQuery` (EQS-lite) | P2 | não | "ache o melhor ponto": cobertura, flanco, spawn justo |
| `NavLink` · `NavCostArea` | P2 | não / parcial | pulo/teleporte no path · lama custa 4× — p/ platformer, GRAFO DE PULOS como modo do NavSurface |
| `ProximityActivator` | P2 | não | "a caverna só simula com o player a 30 m" — nenhuma engine tem como componente |
| `FollowHistory` | P2 | não (primo: `TapeWire`) | ghost de time-trial serializável |
| `Replicated` · `NetworkStateSync` | horizonte | não (`ph2d-net` vazio) | a UI de replicação do Godot (lista de propriedades marcadas) é o modelo de UX |
| `RollbackManaged` | horizonte · **1ª opção quando rede chegar** | semente rara (GGPO + 3-OS prontos) | multiplayer como propriedade do objeto (GM Rollback) |

---

## §6 — Catálogo: COMBATE & o que caiu entre as cadeiras

> Achados do [crítico de completude](pesquisa/critica.md) — presentes nos dossiês, ausentes das 4
> sínteses. O padrão: **combate/RPG não pertencia a nenhum dos 4 domínios** (se houver uma 5ª
> síntese, ela se chama "combate & formulários"). Estes itens entram no cardápio com o mesmo
> status dos demais.

| Componente | P sugerida | O que entrega |
|---|---|---|
| `Health` | **P1 forte** (dono novo: domínio combate) | vida/escudo/armadura/regen/i-frames — **o par dano→vida→morte é o loop de gameplay nº 1** e não tinha dono. Modelo: GDevelop Health (max, over-heal, shield c/ regen, damage cooldown, `Is dead`/`Is just damaged`) + UE GAS Attributes |
| `GameplayEffect` | P1 (wave 2 do mesmo dono) | buff/debuff/DoT/stack como ASSET (Instant/Duration/Infinite, periodic=veneno, tags que concedem/exigem) |
| `WeaponFire` | P1 | a ARMA do player: cooldown/ammo/reload/overheat/spread — completa o trio WeaponFire + ProjectileMotion + Spawner |
| `GameVariables` (+ eventos de mudança) | P1 | o armazém global/por-objeto com evento `changed` — o chão do placar, da quest flag e do HUD reativo (Phaser Data Manager) |
| `UiTextInput` · `UiToggle` · `UiSlider` · `UiDropdown` · `UiPageView` | P1 | **sem slider e toggle não existe tela de settings; sem text input não existe "digite seu nome"** — a suíte §4 cobria HUD, não formulários |
| `RichText` + `TypewriterText` | P1 | o texto de DIÁLOGO: marcação inline, imagens, links; efeito máquina-de-escrever |
| `DialogueController` | P1–P2 | árvore de conversa com escolhas (Construct Flowcharts) — gênero inteiro (VN/RPG) sem dono; ver §8.1 |
| `Inventory`/`Loot` | P2 | itens/slots/drop tables — família natural do Health; território de asset store nas engines grandes = diferencial se for 1ª classe |
| `RotatingMotion` | P2 ⚠️ medir nodes | a serra giratória — Motion Nodes provavelmente exprimem (lei 8) |
| `VideoPlayer` | P2 | cutscene em vídeo / tela de logo |
| Gestos touch (swipe/pinch/rotate) | P2 | o InputTriggers cobre botão; gesto de TELA é outra família (GM Gesture events) |
| `AppStates` (Menu/Playing/Paused) | nota p/ R1 | o fluxo global do jogo (Bevy States, OnEnter/OnExit) — R1-adjacente, a nota precisa existir para o R1 herdá-la |
| `LookAt`/Aim constraint puro | P2 | "vire para o alvo" isolado — candidato a preset de Motion Node |
| Formation/batch actions | P2 | GridAlign/PlaceOnCircle/cobra (Phaser Actions) — menu circular, formação de inimigos |
| `MultiMesh` · `BackBufferCopy` · `ShapePainter` | P2 | (já incorporados ao §3) |
| `UISkew` | nota | o Transform não tem cisalhamento — só registrar |

---

## §7 — TOP-20: se o PH2D implementasse os primeiros 20 amanhã, nesta ordem

> Do [crítico](pesquisa/critica.md) §4, com as 4 sínteses convergindo. Critério: (custo dado o que
> JÁ existe) × (código que elimina) × (o que destrava os seguintes). Coerência: **1** destrava o
> custo de todos → **2–5** fazem sinais virarem jogo (com som) → **6** destrava controllers →
> **7–8** o olhar e o corpo → **9–12** conteúdo dinâmico → **13–15** gameplay sem código →
> **16** a válvula de escape → **17–20** as fachadas sobre módulos já vencedores + o HUD da demo.

| # | Item | Por quê (1 linha) |
|---|---|---|
| 1 | **Infra: Inspector derivado do tipo + UI "Add Component" + required components** | pré-requisito declarado pelas 4 sínteses: sem ele cada item abaixo paga uma seção artesanal — é o divisor do custo do catálogo INTEIRO |
| 2 | `Timer` | a melhor razão custo/benefício do levantamento; primeiro produtor de Signal barato |
| 3 | `SensorZone` | todo trigger de gameplay vira shape+evento — o rapier já reporta, falta a costura |
| 4 | `AudioSource2D` + `AudioListener2D` | a primeira reação AUDÍVEL; o rack de 42 efeitos ganha seu consumidor de cena |
| 5 | `SignalActions` (tabela R3) ⭐ | o item mais importante: com som(4), timer(2) e enable/spawn no dropdown, sinais viram gameplay sem código |
| 6 | `ActionMap` + `ActionState` | a maior lacuna bruta; pré-requisito do padrão default-controls de TODO controller; rebinding de graça |
| 7 | `GameCamera` + `CameraFollow` + `CameraLimits` | a maior lacuna do PH2D e de metade da indústria — o rig que todo jogo 2D reescreve |
| 8 | `SpriteFrames` + `AnimatedSprite` | "o componente nº 1 de qualquer engine 2D" — com a sinergia única de tocar quadros de um documento Flip |
| 9 | `Tags` | a moeda de troca de tudo que segue; hierárquico desde o dia 1 custa quase nada |
| 10 | `PrefabAsset` (fluxo) + `PrefabRef` | `PrefabDoc`+`spawn_prefab` prontos com ZERO UI — o multiplicador de conteúdo |
| 11 | `Spawner` + `SpawnPoint` | a fábrica como componente (categoria que Unity/Godot/UE não têm); `spawn_on_signal` conecta com a R3 |
| 12 | `Lifetime` + `DestroyOutside` | higiene de ciclo de vida em 2 marcadores triviais — sem eles o 11 e o 14 vazam |
| 13 | `TopDownPlayer` | o 2º controller canônico; reusa o desenho lei-pura+ponte validado no platformer |
| 14 | `ProjectileMotion` | o mover arcade genérico (avança+gravidade+ricochete+alcance+homing) |
| 15 | `StateMachine` | enquanto o script não tem UI, é o único cérebro autorável; estado-como-componente |
| 16 | `ScriptProperties` + attach visual do LuauScript | host/persistência/determinismo prontos, ZERO UI — o mecanismo nº 1 de script parametrizável |
| 17 | `TilemapLayer` + `TileSet` + `TilemapCollider` | a linha longa que começa cedo, sobre `ph2d-grid` (11 grids + A*) e física prontos; autotile = wave 2 |
| 18 | `ParticleEmitter` (fachada) | a melhor simulação da classe JÁ existe — o trabalho é o painel no objeto + a escada "abrir como grafo" |
| 19 | `SequencePlayer` | o difícil (religar bindings por WireId) está SHIPADO — o componente transforma a Timeline em cutscenes de jogo |
| 20 | HUD mínimo: `UiCanvas`+`UiAnchor`+`UiLabel`+`UiButton` | placar, vida e menu fecham o loop de demo — o botão publica Signal para a tabela do item 5 |

**Suplentes (21–25), para quando um jogo-guia puxar:** `RaySensor` · `Tween`+presets Fade/Flash ·
`PathFollow` (a vitrine da caneta) · `Health`+`WeaponFire` (o domínio combate, §6) ·
`CameraShake`+`ShakeEmitter`.

---

## §8 — Pontos cegos do levantamento inteiro (nenhum dossiê cobriu — [critica.md §2](pesquisa/critica.md))

1. **Diálogo/narrativa** (árvore de conversa, escolhas, balões) — para uma engine "para artistas",
   possivelmente a ausência mais cara do levantamento. Só Construct Flowcharts tangencia.
2. **Localização de CONTEÚDO do jogo** (strings por língua, assets por locale) — zero menções em
   7 dossiês. O PH2D tem i18n do EDITOR (HR-15); a do JOGO nasce barata se desenhada junto com
   `UiLabel`/binding.
3. **Acessibilidade de jogo** (subtítulos, daltonismo, reduced-motion de gameplay) — zero.
   Irônico: `~/.ph2d/prefs.txt` já tem `reduced_motion` — a semente existe na casa.
4. **Inventário/itens/loot** — zero em todos (asset-store-território). Candidato a diferencial.
5. **Debug/tuning para o USUÁRIO da engine** (overlay de FPS, debug draw de física por toggle,
   console) — pontas soltas sem tratamento.
6. **Aleatoriedade autorável com seed** — numa engine com determinismo 3-OS e GGPO, o "random do
   usuário" TEM de passar pelo serviço com seed. A regra precisa ser escrita quando o primeiro
   componente com random chegar (Spawner, §5).
7. **Rede e save de jogo** têm dossiê raso em TODAS as fontes — quando entrarem no escopo, o
   levantamento atual NÃO basta para desenhar (nova pesquisa dedicada).
8. Tutorial/onboarding in-game, achievements/serviços de plataforma — fora de escopo agora,
   registrado.

---

## §9 — Luz 2D — **TUDO ADIADO (decisão do Enio, 2026-08-20)** — listado sem prioridade

| Item (adiado) | Quem tem |
|---|---|
| Luz pontual/sprite/spot/global + falloff + blend | Unity Light 2D · Godot PointLight2D/DirectionalLight2D · Phaser Light2D |
| Sombras pela silhueta (occluders) | Unity Shadow Caster 2D · Godot LightOccluder2D · Construct/GDevelop |
| Normal maps em sprites | Unity URP 2D · Phaser |
| Escuridão-base / máscara por camada | Godot CanvasModulate |
| Partícula que emite luz | UE Niagara · Unity |

**Registrar, não priorizar:** `ph2d-light` (rig único de lâmpadas p/ relevo do Painter + relight
3D, não-removível) e `BakedForm` (o sculpt doa a normal à tinta 2D) são EXATAMENTE a matéria-prima
de "normal maps em sprites" — quando o adiamento cair, o PH2D chega nessa categoria com **autoria
de normal que nenhuma engine tem**.

---

## §10 — As folhas (o que ler para aprofundar)

| Arquivo | O que tem |
|---|---|
| ⭐ [01_auditoria_modelo_de_objeto.md](01_auditoria_modelo_de_objeto.md) | **a MÁQUINA por baixo do catálogo** (auditoria de 2026-08-21): o `ComponentRegistry` e o fn-pointer que lhe falta · Entidade × nó de DAG (**duas hierarquias disjuntas**) · o mestre/instância/variant que **já existe** no vetor · o inspector **derivado** que já funciona para os nós · e a medição que manda no teto: a captura do undo custa **6,89 ms a 10 k entidades**. ⚠️ Corrige três leituras do inventário abaixo |
| [02_pesquisa_composicao_e_prefab.md](02_pesquisa_composicao_e_prefab.md) | pesquisa externa (2026-08-21) do que os 7 dossiês NÃO cobriram: composição (Unity/Godot/Unreal/**Bevy required components**/**flecs**/Houdini/Figma/Rive), os **4 modelos de override** e a tabela que os decide, e a UX do inspetor (**Godot `get_property_list`** é o schema-em-dados canônico) |
| [03_pesquisa_sistema_de_assets.md](03_pesquisa_sistema_de_assets.md) | navegador e identidade de asset: Godot `uid://`, Unreal Asset Registry/redirectors, Unity `.meta`, **catálogos por UUID do Blender**, `ArResolver` do USD — e o eixo sem prior art: **asset gerado dentro da própria ferramenta** (Substance `.sbs`×`.sbsar` responde, e confessa o preço) |
| ⭐ [04_decisao_arquitetura.md](04_decisao_arquitetura.md) | **v2 (reconsiderada após veto do Enio à noite): arquitetura MATERIALIZADA E VIVA** — instância = objetos reais (física, filhos, tudo) ligados por id ao mestre, override por campo, sync ao vivo, aninhamento sem limite, e o undo **medido de 23,8 ms → 0,27 ms** a 10 k objetos. Verificada por 3 refutadores; as condições que impuseram são o desenho. ⏸️ Aguarda aprovação — ADR e plano não foram escritos |
| [pesquisa/instancias_2026-08-21/](pesquisa/instancias_2026-08-21/) | a evidência da v2: 6 pesquisas (Unity/Godot **em código**, flecs/Bevy **em código**, Houdini/USD/Rive, undo incremental com Blender por release notes, endereçamento de override, fatos do código do PH2D) + 1 **medição** (spike apagado) + **3 refutações** adversariais, cada uma com `file:line` |
| [pesquisa/inventario_ph2d.md](pesquisa/inventario_ph2d.md) | o que JÁ existe no repo (91 componentes, crate a crate) + o caminho que um componente novo percorre (os 6 passos, com o custo medido em cada um) |
| [pesquisa/sintese_movimento_fisica.md](pesquisa/sintese_movimento_fisica.md) | §2 em detalhe: equivalentes por engine, notas de desenho, ordem de waves |
| [pesquisa/sintese_visual_camera.md](pesquisa/sintese_visual_camera.md) | §3 em detalhe |
| [pesquisa/sintese_interacao_fluxo.md](pesquisa/sintese_interacao_fluxo.md) | §4 em detalhe + as 5 leis do domínio |
| [pesquisa/sintese_estrutura_ai.md](pesquisa/sintese_estrutura_ai.md) | §5 em detalhe + as 7 leis do domínio |
| [pesquisa/critica.md](pesquisa/critica.md) | os buracos, as 5 verificações factuais, o raciocínio do TOP-20 |
| [pesquisa/dossie_unity.md](pesquisa/dossie_unity.md) · [godot](pesquisa/dossie_godot.md) · [unreal](pesquisa/dossie_unreal.md) · [construct_gdevelop](pesquisa/dossie_construct_gdevelop.md) · [gamemaker_defold](pesquisa/dossie_gamemaker_defold.md) · [cocos_phaser](pesquisa/dossie_cocos_phaser.md) · [bevy_rust](pesquisa/dossie_bevy_rust.md) | os 7 dossiês com URLs das docs oficiais e a seção "matadores de código" de cada engine |

> ⚠️ Este doc descreve o mundo em **2026-08-20**. O estado vivo dos módulos é o `CLAUDE.md §5`;
> quando um item deste cardápio for construído, a linha dele aqui ganha o ✅ e o link do handoff —
> não reescreva a narrativa aqui (lição do §5).
