# Síntese cruzada — Domínio VISUAL & CÂMERA (componentes canônicos para o PH2D)

> Fontes: `dossie_unity.md`, `dossie_godot.md`, `dossie_unreal.md`, `dossie_construct_gdevelop.md`,
> `dossie_gamemaker_defold.md`, `dossie_cocos_phaser.md`, `dossie_bevy_rust.md`, `inventario_ph2d.md`
> (todos lidos por inteiro, 2026-08-20). Decisão do dono já tomada: **componentes estilo Unity
> (AddComponent)**, não herança de nodes. **Iluminação 2D: ADIADA (decisão do dono, 2026-08-20)** —
> seção L lista tudo sem priorizar.
>
> **Legenda de prioridade:** P0 = espinha de qualquer engine de jogo (sem isso não há jogo) ·
> P1 = diferencial forte de facilidade · P2 = depois.
> **"Já tem":** sim / parcial / não, com a crate (do `inventario_ph2d.md`).
>
> **Nota transversal nº 1 (vale para TODO componente abaixo):** o inventário mediu que no PH2D os
> passos "definir → registrar → persistir → undo" de um componente novo são baratos e mecânicos;
> **o custo real é a seção artesanal no Inspector** (não há UI genérica de AddComponent nem derive
> de painel). Cada P0 abaixo paga esse custo; a mecânica que o amortiza (inspector derivado do tipo
> + "required components" à la Bevy 0.15 — adicionar `CameraFollow` puxa `GameCamera` sozinho) é
> pré-requisito de plataforma que este domínio consome mas não possui.
>
> **Nota transversal nº 2:** referência durável entre objetos (alvo de follow, alvo de socket) é
> **`stable_name_id`** (hash do `Name`), nunca `Entity::to_bits()` — lei já paga pelo undo do PH2D.

---

## A. Sprite & animação de frames

### A.1 `Sprite` — ✅ JÁ EXISTE (registrar como pronto)
- **Entrega:** desenhar imagem com tint em cascata, flip, pivot, atlas/região, sheet inline.
- **Por engine:** Unity SpriteRenderer · Godot Sprite2D · UE PaperSpriteComponent · Cocos/Defold/Phaser/Bevy Sprite · GM built-ins da instância.
- **Já tem:** **sim** — `ph2d-render::Sprite` v4 (tint cascateado + `self_tint` + gradiente por canto + `tint_fill` flash-de-dano + opacity, flip, pivot, `hframes×vframes×frame`, region, KTX2) + `ph2d-ecs` (`TextureFilter/TextureRepeat/UvTransform`, `BlendMode`). Cobre inclusive o "flash de dano" do Phaser e o scroll de UV do TileSprite.
- **Prioridade:** — (feito). O que falta vira A.2 e A.3.

### A.2 `AnimatedSprite` — **P0**
- **Entrega:** animações de frames NOMEADAS num asset (importa pasta/atlas, FPS e loop por animação); `play("run")`, pause, eventos `animation_finished` / `frame_changed` / marker-em-frame ("no frame 7, spawne a hitbox"). Elimina a máquina de troca de frames (array + índice + timer + callback) que todo jogo reescreve — e o Animator pesado que a Unity obriga para um flipbook.
- **Por engine:** Godot AnimatedSprite2D+SpriteFrames (o modelo a copiar) · UE PaperFlipbookComponent · Phaser Anims (chain, yoyo, eventos por frame) · Bevy (ausente no core; `bevy_spritesheet_animation` com markers) · GM `image_index/image_speed` · Cocos Animation · Construct Sprite animations. Unity: **ausência declarada** (flipbook leve não existe — lacuna notória).
- **Já tem:** **parcial** — `Sprite` já tem a grade inline (`hframes/vframes/frame`); `ph2d-timeline::SpriteAnimation` anima só Transform (não frame); e o **módulo Flip é a autoria de flipbook DENTRO do app** (`FlipObjectRef` já liga objeto Flip ↔ entidade).
- **Dependências:** Sprite (pronto) + asset `SpriteFrames`; sinergia única: **tocar os quadros de um documento Flip** — nenhuma engine concorrente tem a autoria do flipbook embutida.
- **Nota de desenho:** eventos de fim/frame devem publicar `Signal` (`ph2d-runtime`, ADR-0143) — vira gameplay autorável quando o R3 (tabela sinal→ação) existir. Frame index keyframável na Timeline (hoje `SpriteAnimation` só escreve Transform — fechar esse canal junto).

### A.3 `SpriteDrawMode` (Sliced / Tiled / Filled) — **P1**
- **Entrega:** nine-patch (cantos fixos, bordas tile/stretch), tiled (textura repetida com tamanho editável) e **Filled** (linear/radial — a barra de vida e o cooldown radial são um dropdown, não uma feature). Elimina a malha de 9 quads, o tiling manual e o shader de barra radial.
- **Por engine:** Unity SpriteRenderer.DrawMode (lição: **modo do Sprite, não componente separado**) · Godot NinePatchRect · Cocos Sprite Type Sliced/Tiled/**Filled** · GM Nine Slice (universal, no asset) · Defold Slice-9 · Phaser NineSlice/TileSprite · Bevy `SpriteImageMode::Sliced/Tiled` · Construct 9-patch · GDevelop Panel Sprite.
- **Já tem:** **não** (TextureRepeat+UvTransform cobrem só o caso "fundo que rola").
- **Dependências:** Sprite; ⚠️ mudar a FORMA do `Sprite` shipado = bump do `PROJECT_SCHEMA` (3 sítios, lei do repo).
- **Nota:** copiar a decisão da Unity/Cocos — 1 enum no Sprite economiza 3 componentes e 3 seções de Inspector.

### A.4 `SpriteLibrary` + `SpriteResolver` — **P1**
- **Entrega:** troca de sprite estruturada por **Categoria + Label** ("Mouth"/"happy") com herança de variantes; o label é keyframável. Skins, troca de equipamento e lip-sync viram dropdown — mata o dicionário-de-sprites que todo RPG escreve.
- **Por engine:** Unity Sprite Library/Resolver (único com o modelo completo) · Cocos sp.Skeleton skins (só via Spine) · Defold/GM via Spine.
- **Já tem:** **não**.
- **Dependências:** Sprite; Timeline (label como canal animável); esqueleto (A resolve por hierarquia — um library na raiz serve o rig).

---

## B. Tilemap & autotiling

### B.1 `TilemapLayer` + asset `TileSet` — **P0**
- **Entrega:** nível pintado em grade (pincel/balde/retângulo/picking, camadas reordenáveis, Y-sort por camada), com o TileSet embutindo POR TILE: sprite, colisão, **metadata custom** ("é gelo? é lava?"), tiles **animados** e tile-que-instancia-**prefab**. Elimina a estrutura de dados de nível inteira + o editor de nível + instanciar milhares de sprites.
- **Por engine:** Godot TileMapLayer+TileSet (o mais completo: colisão+nav+metadata+cena por tile) · Unity Grid+Tilemap+TilemapRenderer · UE PaperTileMap (+ **import de Tiled .json**) · GM Tile Set Editor (+ **Convert Image To Tile Map** com dedup) · Defold Tilemap (colisão derivada do tile source) · Cocos TiledMap (TMX; oclusão node↔tile por linha) · Phaser Tilemaps (`setCollisionByProperty` + `createFromObjects`; v4 TilemapGPULayer custo fixo/pixel) · Construct Tilemap (TMX) · GDevelop Tilemap (LDtk/Tiled) · Bevy `bevy_ecs_tilemap` (tile-como-entidade).
- **Já tem:** **não** no runtime — `ph2d-grid` (11 tipos de grid + snap + **A\* determinístico pronto**) é a fundação matemática; `docs/Tilling` é MVP gitignored FORA do repo (decisão do dono — não referenciar).
- **Dependências:** ph2d-grid (pronto) · render batched em chunks · B.2 (colisão) · import externo (Tiled/LDtk — o padrão de adoção que UE/Phaser/GDevelop provam; o contrato `bevy_ecs_ldtk` "entidade do editor → entidade com componentes registrados por identificador" é o desenho de referência).
- **Nota de desenho (a decisão cara):** tile-como-entidade (Bevy) unifica o modelo mental mas explode contagem; monolito chunked (todas as outras) é rápido mas cria um segundo sistema de dados. Recomendação: **armazenamento chunked dentro de UMA entidade-mapa + API por célula** — e MEDIR antes de fixar qualquer teto (§0.0: o caso GPU/M5 aconteceu exatamente num sistema massivo deste tipo). Hex e isométrico entram de graça pelo `ph2d-grid`.

### B.2 `TilemapCollider` — **P0** (junto de B.1)
- **Entrega:** colisão gerada automaticamente dos tiles pintados, **fundida num outline único** (mata o "ghost collision" nas emendas — o composite da Unity é a prova de que sem fusão o P0 fica quebrado). Sincroniza colisão↔arte quando o nível muda.
- **Por engine:** Unity TilemapCollider2D+CompositeCollider2D · Godot (embutido no TileSet) · Defold (Collision Object usa a geometria do tilemap) · Phaser (`setCollisionByProperty`) · GM (máscara/funções).
- **Já tem:** **não**; a ponte é `ph2d-physics-ecs` (rapier, colisão por camadas já existe).
- **Dependências:** B.1 + física (pronta). Determinismo: BTreeMap, hash 3-OS (lei da linha physics).

### B.3 `AutotileTerrain` (rule tiles / terrenos) — **P1** (a 2ª onda do TileSet)
- **Entrega:** pintar 1 tile e a engine escolhe a variante que conecta com os vizinhos (matriz 3×3 de regras visual; saída fixa/aleatória/animada; a regra pode fixar colisão). **Rule Override** retroca só os sprites (retheming instantâneo). É o maior matador de código da categoria — o bitmask de 47 casos + a re-pintura manual morrem.
- **Por engine:** Unity Rule Tile (o editor visual de referência) · Godot terrenos (Connect/Path) · GM Auto Tiles.
- **Já tem:** **não**.
- **Dependências:** B.1. Nota: shippar B.1 sem B.3 é usável; B.3 é o que faz artista AMAR o tilemap.

### B.4 `SpriteShape` (terreno orgânico por spline) — **P2**
- **Entrega:** desenhar o nível com uma caneta: spline aberta/fechada + profile que mapeia FAIXAS DE ÂNGULO → sprites (chão/parede/teto/quinas), fill no interior, colisão acompanhando a curva. Elimina mesh procedural de terreno + troca de sprite por inclinação.
- **Por engine:** Unity Sprite Shape (único).
- **Já tem:** **parcial na fundação** — o motor vetorial (`ph2d-vec-*`: paths como entidades, stroke profiles vivos ADR-0148, booleana viva, pattern paths) é uma base MELHOR que a da Unity; falta só o profile ângulo→sprite e o baker de colisão.
- **Dependências:** Vector (pronto) + física. Diferencial real: o PH2D tem o editor vetorial completo DENTRO — nenhuma engine de jogo tem.

---

## C. Parallax

### C.1 `ParallaxLayer` — **P1**
- **Entrega:** profundidade falsa amarrada à câmera: fator de scroll por eixo, **repetição infinita** e **autoscroll** (nuvens sem câmera). Elimina rastrear câmera + offsets por profundidade + tiling para loop.
- **Por engine:** Godot Parallax2D (o mais completo: repeat_size, autoscroll, limites) · Phaser **scrollFactor por objeto** (o desenho mais elegante — 1 float em qualquer objeto) · GM Background Layers (speed) · Bevy bevy-parallax (tabela de camadas) · Construct (propriedade de layer). Unity/UE/Defold/Cocos: **ausência declarada** (todo projeto escreve o script).
- **Já tem:** **não** (UvTransform faz o caso degenerado "fundo fixo que rola").
- **Dependências:** H.1 (GameCamera) — o fator é relativo ao scroll DELA.
- **Nota de desenho:** oferecer os dois idiomas: `scroll_factor` como campo barato em qualquer objeto (Phaser) + `ParallaxLayer` com repeat/autoscroll para o caso fundo (Godot).

---

## D. Partículas de gameplay

### D.1 `ParticleEmitter` (componente de cena) — **P0**
- **Entrega:** o motor de VFX inteiro por Inspector: emission rate/burst, formas de emissão (ponto/círculo/retângulo/borda/**sprite**), vida, velocidade/aceleração/gravidade, **cor e tamanho ao longo da vida por curva/gradiente**, spin, one-shot com evento `finished`, pré-warm, seed determinística, **emitZone/deathZone**, sub-emissores (explosão → fagulhas), trails por partícula, preview no editor. Zero código para 95% dos efeitos.
- **Por engine:** Unity Particle System (o padrão-ouro de módulos colapsáveis; toda propriedade aceita constante/curva/random-entre-curvas/gradiente) · Godot GPUParticles2D+ParticleProcessMaterial (seed, explosiveness, trails, sub_emitter) · UE Niagara (pilha de módulos; ribbon) · Phaser ParticleEmitter (emitZone/deathZone/follow/gravity wells — o benchmark de API 2D) · GM Particle Editor (subparticles On Death/On Update, "Copy GML") · Defold ParticleFX (modifiers Radial/Vortex/Drag, curvas) · Cocos ParticleSystem2D (plist) · Construct Particles (preview, **qualquer objeto como partícula**) · Bevy hanabi/enoki (asset declarativo + hot-reload + editor).
- **Já tem:** **parcial FORTE** — a simulação existe e é a melhor da classe: Motion Nodes (`motion.emitter`, forças, boids, clone) **GPU-resident, 4,19M partículas @ 3,6 ms medido**, ligada a objetos via `motion_object_bake`. O que NÃO existe é o **componente "ParticleEmitter" no objeto** com painel de módulos — hoje é grafo de nós.
- **Dependências:** ponte motion↔objeto (existe) + seção de Inspector espelhando os módulos.
- **Notas de risco:** (1) ⚠️ **é o módulo do caso §0.0** — o teto de 16.384 já foi posto 256× abaixo do medido uma vez; qualquer cap novo entra com a tabela de medição ao lado. (2) Duas audiências, um motor: o componente é a fachada simples; "abrir como grafo" dá o poder total (o VFX Graph da Unity e o Niagara provam a escada simples→avançado). (3) `explosiveness`+seed do Godot e o modo "qualquer objeto como partícula" do Construct são os dois detalhes a copiar.

---

## E. Linhas & rastros

### E.1 `LineRenderer` — **P1**
- **Entrega:** polilinha no mundo com **curva de largura**, gradiente de cor, joints (sharp/bevel/round), caps, textura ao longo (stretch/tile). Elimina o mesh de fita (miter/bevel/UVs) — laser, corda visual, mira de trajetória.
- **Por engine:** Godot Line2D (o mais completo) · Unity Line Renderer · Phaser Rope · Cocos Graphics (parcial). UE/GM/Defold/Construct/GDevelop: **ausência declarada** (categoria órfã na maioria!).
- **Já tem:** **parcial** — o motor vetorial já resolve o problema difícil: `VecStrokeProfile` (largura viva com um baker, ADR-0148), joints/caps do motor de traço do Flip. Falta o componente leve de gameplay (lista de pontos mutável por script/física, sem documento vetorial).
- **Dependências:** decidir a costura com `ph2d-vec-*` (reusar o baker, não reimplementar).

### E.2 `TrailRenderer` — **P1**
- **Entrega:** rastro atrás de objeto em movimento: vida por ponto (`time`), min vertex distance, largura/cor decaindo, **`emitting` liga/desliga sem apagar o rastro**, autodestruir. Elimina o ring-buffer de poses + mesh strip + fade de todo jogo de espada/dash/projétil.
- **Por engine:** Unity Trail Renderer · Cocos **MotionStreak** (6 propriedades, preview — a prova de que é barato) · UE Niagara Ribbon · Godot (via Line2D + script — semi-órfã). Construct/GDevelop/GM/Defold/Phaser/Bevy: **ausentes** — categoria com dono raro; diferencial fácil.
- **Já tem:** **não**.
- **Dependências:** E.1 (mesmo baker de fita).

---

## F. Esqueleto 2D, deform e IK

### F.1 `Skeleton2d` + `Bone2d` — **P1**
- **Entrega:** hierarquia de ossos com rest pose, autorada NO editor (bones são entidades com Transform ⇒ **a Timeline do PH2D já os anima de graça** via Name-binding). Elimina o pipeline de rig 2D — e ataca o maior buraco da concorrência.
- **Por engine:** Unity 2D Animation (Skinning Editor, bones+pesos, PSD importer) · Godot Skeleton2D+Bone2D+Polygon2D (weight painting) · Spine via runtime em Cocos (**Sockets!**)/Defold/GM/GDevelop/Bevy. **UE: NÃO TEM esqueleto 2D nativo** (buraco confesso) · Construct não tem oficial · **todas as engines "consomem Spine"; só Unity e Godot AUTORAM** — o PH2D com Painter/Flip/Vector dentro pode autorar rig + arte no mesmo lugar.
- **Já tem:** **não** como rig; **parcial** nas peças: Timeline anima Transforms; `ph2d-physics-ecs` já tem a árvore de posing IK transitória (ADR-0149); Flip tem deform/tween de traço; sculpt/deform do Painter.
- **Dependências:** nada externo; sinergias: Timeline (animação), F.2 (skinning), A.4 (library por rig).

### F.2 `SpriteSkin` (deform por pesos) — **P1**
- **Entrega:** o mesh do sprite deformado pelos bones, com pesos pintados visualmente (branco=1/preto=0); CPU ou GPU. O usuário riga na UI e anima os bones com a Timeline comum.
- **Por engine:** Unity Sprite Skin (GPU deform) · Godot Polygon2D (weight painting) · Construct mesh distortion (grade de pontos, sem bones — o degrau barato).
- **Já tem:** **não**; a grade-de-pontos do Construct é um P1-meio-passo barato (deform sem rig).
- **Dependências:** F.1; o carimbo GPU do Painter (`ph2d-paint-gpu`) e o pipeline wgpu já dão o chão do skinning em GPU.

### F.3 `IkSolver2d` (Limb / CCD / FABRIK) — **P1**
- **Entrega:** arrasta-se um target e a cadeia segue; **Weight 0–1 mistura FK↔IK**. Pé que cola no chão e mão que segura objeto viram "arraste o alvo".
- **Por engine:** Unity IK Manager 2D + 3 solvers (referência de UI) · Godot SkeletonModification2D (marcado **experimental** — API instável, oportunidade) · Spine IK via runtimes.
- **Já tem:** **parcial** — ADR-0149: a física já tem IK como árvore de posing transitória (não uma 2ª representação de joint). Reusar a lei; expor como componente.
- **Dependências:** F.1.

### F.4 `BoneSocket` — **P2**
- **Entrega:** prender uma entidade externa a um osso (arma na mão) sem reparenting — o "Sockets" do Spine no Cocos, `spine.get_go` no Defold, Pin to image point do Construct.
- **Já tem:** **não** (a lei "referência = stable_name_id" já resolve a metade difícil).
- **Dependências:** F.1.

---

## G. Sorting, grupos, máscara e tint — ✅ CATEGORIA PRATICAMENTE FECHADA

- **Sorting suite:** **sim** — `ph2d-ecs`: `SortingLayer`, `OrderInLayer`, `ZIndexOverride`, `ZAsRelative`, `YSort`, **`SortingGroup`** (o mata-pesadelo do personagem multi-sprite da Unity), `ShowBehindParent`, `TopLevel` (Sprite Inspector v2 W3). Equivale ao pacote Unity+Godot somados; o `YSort` cobre o behavior YSort do GDevelop.
- **Máscara/clipping:** **sim** — `ClipChildren`, `Mask2D`, `MaskInteraction` (= Unity Sprite Mask + Godot clip).
- **Tint em cascata:** **sim** — `Sprite` tint cascateia + `self_tint` (= Godot modulate/self_modulate) + `tint_fill` (flash Phaser).
- **Trabalho restante nesta categoria:**
  - **G.1 `CanvasGroup` — P2:** filhos desenhados como UM objeto antes do alpha/blend (fade de personagem multi-parte sem escurecer interseções). Godot CanvasGroup (único). Já tem: **não**; `GameRt` (render-target interno) é a infraestrutura. Depende de I.1.
  - **G.2 `SceneTint` (CanvasModulate) — P2:** tinge o mundo inteiro (dia/noite, mood). Godot CanvasModulate. Já tem: **não**. ⚠️ tangencia o esquema de luz — a metade "escuridão-base da iluminação" fica **ADIADA** junto com a seção L; a metade "mood tint" é livre.
  - **G.3 interleave tilemap↔sprites:** o modo Individual do Tilemap Renderer da Unity / oclusão por linha do Cocos — nota de requisito para B.1 + YSort (personagem atrás da árvore DO tilemap).

---

## H. Câmera de gameplay (a maior lacuna do PH2D — e da metade da indústria)

> Contexto do inventário: `Camera2d` hoje é **resource do editor**, não componente de cena. Toda a
> família abaixo começa por promover a câmera a ENTIDADE. Benchmarks: Phaser (o pacote completo em
> API), Cinemachine (o pacote completo em componentes), GameMaker (dead-zone em 4 campos). Godot
> não tem shake/confiner/zonas; UE não tem 2D; Cocos não tem NADA; Bevy não tem NADA — categoria
> onde o PH2D pode passar TODOS com componentes + UI.

### H.1 `GameCamera` — **P0**
- **Entrega:** a câmera como componente de cena: ortho size/zoom, offset, prioridade, `cull_mask` por `VisibilityLayer`, ativa/inativa (trocar a ativa = transição), helpers `screen_to_world`/`world_to_screen` (Defold prova que só isso já elimina uma classe de bugs de input).
- **Por engine:** Godot Camera2D · Unity Camera+CinemachineCamera (prioridade+blend) · GM Views · Defold Camera (ortho zoom, Fixed/Auto Fit/Auto Cover) · Phaser Camera · Cocos Camera (visibility bitmask, RT no inspector) · Bevy Camera2d.
- **Já tem:** **parcial** — `Camera2d` (zoom + cull_mask) existe como resource; `GameRt` já é o alvo offscreen HDR. O trabalho é a promoção a componente + N câmeras + prioridade.
- **Dependências:** raiz de TODO o resto de H; toca shell/foundational (linha própria).

### H.2 `CameraFollow` — **P0**
- **Entrega:** o script que TODO jogo 2D reescreve, morto por UI: alvo (por Name), **damping por eixo**, **dead-zone** (janela onde o alvo anda sem mover a câmera), soft zone, **lookahead** (antecipa o movimento), offset; **multi-alvo = centro do grupo + zoom-to-fit** (o Scroll To multi-instância do Construct dá co-op local de graça).
- **Por engine:** Cinemachine Position Composer (dead/soft zone, lookahead — a referência) · Godot Camera2D (smoothing + drag margins) · GM Object Following (H/V Border + Speed em 4 campos) · Phaser `startFollow(lerp)+setDeadzone` · Construct Scroll To (multi-alvo!) · GDevelop Smooth Camera / Smooth platformer camera (estabiliza no pulo — detalhe fino a copiar) · UE SpringArm (lag).
- **Já tem:** **não**.
- **Dependências:** H.1. Alvo por `stable_name_id`.

### H.3 `CameraLimits` / `CameraConfiner` — **P0** (retângulo) / P1 (polígono)
- **Entrega:** a **janela visível** (não só o centro) presa aos limites da fase; retângulo no P0, polígono arbitrário com corte de cantos no P1 (salas não-retangulares). Elimina os clamps manuais e os bugs de borda.
- **Por engine:** Godot Camera2D limits (retângulo, com suavização) · Cinemachine Confiner 2D (polígono — só ela tem) · Phaser setBounds · GM (implícito no view).
- **Já tem:** **não**.
- **Dependências:** H.1/H.2; o confiner polígono pode consumir um path do motor vetorial (a MESMA spline do nível — sinergia que a Unity anuncia como feature).

### H.4 `CameraShake` + `ShakeEmitter` — **P1**
- **Entrega:** shake pelo modelo **trauma-com-decay** (amplitude/frequência/octaves — nunca brusco nem flutuante) e **fontes posicionais**: um `ShakeEmitter` na explosão com raio interno/externo e falloff faz tremer mais perto, sem acoplamento "explosão → achar câmera → Shake()". Perfis prontos (recoil, bump, explosion).
- **Por engine:** Cinemachine Impulse (fonte+listener — a referência de arquitetura) · UE CameraShake assets + **UCameraShakeSourceComponent** (falloff espacial) · Phaser `camera.shake` · Construct Scroll To → action Shake (decay) · Bevy bevy_trauma_shake (trauma, 5 números). Godot: **não tem** (buraco confesso).
- **Já tem:** **não**.
- **Dependências:** H.1. **Nota de desenho:** shake é OFFSET pós-follow, nunca escreve no Transform do alvo nem compete com o follow (o erro clássico). A dupla emitter/listener casa 1:1 com o idioma `Signal`/outbox do ADR-0075.

### H.5 `CameraZone` — **P1**
- **Entrega:** regiões da fase que trocam configuração de câmera (zoom, limites, alvo, offset) com **transição com curva por par** — troca de sala = entrar na zona, zero código. Prioridade entre zonas sobrepostas.
- **Por engine:** Cinemachine (N vcams + Brain com blends — o único modelo completo) · resto do mercado: **ausência generalizada** (Godot/GM/Phaser/UE 2D fazem na mão).
- **Já tem:** **não**; as zonas da física (`AreaEffector` etc.) já provam o idioma "região com prioridade e falloff" no PH2D.
- **Dependências:** H.1–H.3; sensor de área (física, pronta).

### H.6 `CameraEffects` (fade/flash/pan/zoom-to) — **P1**
- **Entrega:** efeitos com duração + callback: fade in/out para cor, flash, pan até um ponto, zoom animado — as 4 ações de toda cutscene/checkpoint/morte, sem coroutine.
- **Por engine:** Phaser (o pacote completo: Shake/Fade/Flash/Pan/Zoom/Rotate com callbacks) · UE Fade Track/`SetViewTargetWithBlend` · GM/Godot: código.
- **Já tem:** **não**; a Timeline do PH2D pode autorá-los (fade como clip) — expor também como ações one-shot.
- **Dependências:** H.1; sinergia com Timeline (marker → Signal → efeito quando o R3 existir).

### H.7 `PixelPerfectCamera` — **P1**
- **Entrega:** pixel art nítida e estável: zoom inteiro derivado de PPU + resolução de referência, **crop/pillarbox**, snapping dos sprites à grade NO RENDER (sem tocar os transforms), upscale da render texture. Elimina a matemática que jogos de pixel art exigem.
- **Por engine:** Unity Pixel Perfect Camera (a referência; + extensão de conciliação com Cinemachine) · Phaser roundPixels · Godot (project settings, parcial).
- **Já tem:** **não** (o `GameRt` HDR já é o lugar do upscale).
- **Dependências:** H.1; ⚠️ **conflita com H.2 por desenho** (os dois disputam o ortho size — a Unity precisou de uma extensão para reconciliá-los; projetar os dois JUNTOS desde o início).

### H.8 `CameraDolly` (câmera em trilho) — **P2**
- **Entrega:** câmera presa a um path com progresso 0–1 animável (cutscene, seção em trilhos).
- **Por engine:** Cinemachine Spline Dolly · Godot Path2D+Camera2D · UE (à mão).
- **Já tem:** **não**; depende do componente Path/PathFollow (domínio de movimento) — o motor vetorial já tem as curvas.

### H.9 Multi-câmera / split-screen — **P2**
- **Entrega:** N câmeras com viewport rect próprio; split-screen por 2+ viewports; câmera de UI que ignora o mundo (`camera.ignore`).
- **Por engine:** GM Views (split nativo) · Phaser multi-camera+ignore · Unity PlayerInputManager (split automático) · Defold (N câmeras).
- **Já tem:** **não**; H.1 + I.1 dão as peças.

---

## I. Viewport / render-target

### I.1 `RenderTargetSurface` (SubViewport) — **P1**
- **Entrega:** renderizar uma subárvore/câmera para uma textura usável como sprite: minimapa, retrato, portal, picture-in-picture, TV no cenário. Elimina o pipeline offscreen manual.
- **Por engine:** Godot SubViewport+ViewportTexture · Unity RenderTexture · Cocos Camera.TargetTexture (no Inspector!) · Phaser RenderTexture · UE SceneCapture.
- **Já tem:** **parcial** — `GameRt` (offscreen HDR + tonemap AgX) é exatamente isso, interno; falta expor como componente + textura consumível por `Sprite`.
- **Dependências:** H.1; habilita G.1 (CanvasGroup), H.9 (split), e o FX por objeto pesado de K.

---

## J. Visibilidade on-screen

### J.1 `OnScreenEnabler` — ✅ **sim, já existe** (`ph2d-ecs`) — desliga processamento fora da tela ("inimigo só acorda quando o player chega"). Equivalente ao VisibleOnScreenEnabler2D do Godot — o PH2D está À FRENTE da maioria aqui.

### J.2 `OnScreenNotifier` — **P1**
- **Entrega:** EVENTOS `screen_entered`/`screen_exited` (+ `is_on_screen` consultável, rect configurável) — despawn de projétil, ativar spawner, música por região. O enabler age; o notifier AVISA.
- **Por engine:** Godot VisibleOnScreenNotifier2D (referência) · Construct "Is on-screen" · GDevelop Is on screen · Phaser (culling interno).
- **Já tem:** **parcial** (a detecção existe no enabler; faltam os eventos).
- **Dependências:** publicar como `Signal` no outbox (ADR-0075/0143) — o consumidor autorável chega com o R3.
- **Nota:** o irmão `DestroyOffscreen` (Construct Destroy Outside — higiene de balas em 1 checkbox) é do domínio utilitários/ciclo-de-vida, mas nasce do mesmo teste — registrar a costura.

---

## K. FX por objeto

### K.1 `ObjectFx` (pilha de efeitos por objeto) — **P1**
- **Entrega:** glow/outline/shadow/pixelate/dissolve/wipe **por objeto ou por câmera**, empilháveis, em 1 chamada/checkbox — o "juice" que exigiria shader (hit-flash de selecionado, glow de item raro, transição de tela).
- **Por engine:** Phaser preFX/postFX → **Filters v4** (14 efeitos, +16× mobile — o benchmark) · GM Filter/Effect Layers (dropdown por camada) · Cocos (materiais custom).
- **Já tem:** **parcial** — o `fx_stack` (glow, drop-shadow, rgb-split) existe como **Motion Nodes por camada**; falta a fachada componente-por-objeto.
- **Dependências:** mesma escada do D.1 (componente = fachada; grafo = poder total).

---

## L. LUZ 2D — **TUDO ADIADO (decisão do dono, 2026-08-20)** — listado sem prioridade

| Item (adiado) | Quem tem |
|---|---|
| Luz 2D pontual/sprite/spot/global + blend styles + falloff | Unity Light 2D (URP) · Godot PointLight2D/DirectionalLight2D · Phaser Light2D |
| Sombras 2D pela silhueta (occluders) | Unity Shadow Caster 2D (+Composite) · Godot LightOccluder2D · Construct Shadow caster/Shadow light · GDevelop Light+Light Obstacle |
| Normal maps em sprites (relevo iluminado) | Unity URP 2D · Phaser (diffuse+normal) |
| Escuridão-base / máscara de luz por camada | Godot CanvasModulate como base do esquema de luz |
| Image-based lighting como filtro | Phaser 4 Filters |
| Partícula que emite luz | UE Niagara Light Renderer · Unity Particle Lights |
| Componente Light no catálogo | Defold Light |
- **Já tem (registrar, não priorizar):** `ph2d-light` é o rig ÚNICO de lâmpadas (relevo do Painter + relight 3D — não é luz de jogo, e **não é removível**); `BakedForm` (sculpt doa normal à tinta 2D) é EXATAMENTE a matéria-prima de "normal maps em sprites" quando o adiamento cair — o PH2D chegará nessa categoria com autoria de normal que nenhuma engine tem.

---

## M. Ordem de construção sugerida (dependências resolvidas)

1. **Onda P0-câmera:** H.1 `GameCamera` (promover a componente) → H.2 `CameraFollow` + H.3 `CameraLimits` na mesma linha (são 3 seções de um rig; `#[require]`-style: follow puxa camera).
2. **Onda P0-conteúdo:** A.2 `AnimatedSprite` (curto, destrava gameplay visual) ‖ B.1+B.2 `Tilemap`+`Collider` (linha longa; `ph2d-grid` e física prontos) ‖ D.1 `ParticleEmitter` (só empacotamento — a sim existe).
3. **Onda P1-juice/câmera:** H.4 Shake+Emitter · H.6 Effects · H.7 PixelPerfect (projetado JUNTO do follow) · C.1 Parallax · J.2 Notifier (via Signal).
4. **Onda P1-arte:** A.3 DrawModes · A.4 Library/Resolver · E.1+E.2 Line/Trail (baker do vetor) · K.1 ObjectFx · B.3 Autotile.
5. **Onda P1-rig:** F.1 Skeleton → F.2 Skin → F.3 IK (reusa ADR-0149) — a aposta de diferencial (autoria de rig + arte no mesmo app).
6. **P2:** H.5 Zones (pode subir se um jogo-guia pedir salas) · I.1 RenderTarget (sobe se minimapa/split pedirem) · B.4 SpriteShape · G.1/G.2 · F.4 · H.8 · H.9.

---

## N. Tabela-resumo (nome · prioridade · já existe?)

| Componente | Prio | Já existe? |
|---|---|---|
| Sprite | — | sim (ph2d-render) |
| Sorting suite (SortingLayer/Group/YSort/…) | — | sim (ph2d-ecs) |
| Mask2D / ClipChildren | — | sim (ph2d-ecs) |
| Tint cascata + tint_fill | — | sim (ph2d-render) |
| OnScreenEnabler | — | sim (ph2d-ecs) |
| GameCamera | P0 | parcial (Camera2d resource + GameRt) |
| CameraFollow | P0 | não |
| CameraLimits | P0 | não |
| AnimatedSprite | P0 | parcial (sheet inline; Flip como autoria) |
| TilemapLayer + TileSet | P0 | não (ph2d-grid = fundação) |
| TilemapCollider | P0 | não (física pronta) |
| ParticleEmitter | P0 | parcial forte (Motion Nodes GPU 4,19M) |
| SpriteDrawMode (Sliced/Tiled/Filled) | P1 | não |
| SpriteLibrary + SpriteResolver | P1 | não |
| AutotileTerrain | P1 | não |
| ParallaxLayer | P1 | não |
| LineRenderer | P1 | parcial (baker do ph2d-vec-*) |
| TrailRenderer | P1 | não |
| Skeleton2d + Bone2d | P1 | não (Timeline anima de graça) |
| SpriteSkin | P1 | não |
| IkSolver2d | P1 | parcial (ADR-0149 na física) |
| CameraShake + ShakeEmitter | P1 | não |
| CameraZone | P1 | não (idioma das áreas da física) |
| CameraEffects (fade/flash/pan/zoom) | P1 | não |
| PixelPerfectCamera | P1 | não |
| RenderTargetSurface | P1 | parcial (GameRt interno) |
| OnScreenNotifier | P1 | parcial (enabler detecta; faltam eventos) |
| ObjectFx (pilha por objeto) | P1 | parcial (fx_stack por camada) |
| CameraConfiner (polígono) | P1 | não |
| SpriteShape (terreno por spline) | P2 | parcial (motor vetorial) |
| CanvasGroup | P2 | não (GameRt = infra) |
| SceneTint (CanvasModulate) | P2 | não (½ adiada c/ luz) |
| BoneSocket | P2 | não |
| CameraDolly | P2 | não (curvas prontas no vetor) |
| Multi-câmera / split-screen | P2 | não |
| Luz 2D (toda a seção L) | ADIADO | ph2d-light = rig p/ relevo/3D; BakedForm = normal autorada |
