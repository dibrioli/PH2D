# Síntese cruzada — INTERAÇÃO & FLUXO (input · UI in-game · áudio de gameplay · tempo · tween · animação · sequencer · sinais · scripting · mobile)

> Cruzamento dos 7 dossiês (Unity, Godot, Unreal, Construct/GDevelop, GameMaker/Defold, Cocos/Phaser, Bevy)
> contra o `inventario_ph2d.md`. Data: 2026-08-20. Decisão-mãe já tomada: **componentes estilo Unity
> (AddComponent)**, não herança de nodes. Iluminação 2D: **ADIADA** (não há item de luz neste domínio;
> nada a listar). Levantamento EXAUSTIVO — o dono corta, não o levantamento.
>
> Legenda "Já tem": ✅ sim · ◐ parcial · ✖ não — com a crate do inventário.
> Prioridade: **P0** = espinha (sem isso não há jogo) · **P1** = diferencial forte de facilidade · **P2** = depois.

---

## 0. As cinco leis de desenho que atravessam TODO o catálogo deste domínio

Estas não são componentes — são o contrato que faz o catálogo funcionar (destiladas dos dossiês; a
fonte de cada uma está citada):

1. **Todo componente publica VOCABULÁRIO, não só propriedades** (Construct §0): conditions
   ("Is on floor"), actions ("Set max speed") e expressions ("Timer.Progress") que o resto do sistema
   — SignalActions, Luau, timeline — consome. Painel configura; vocabulário compõe. No PH2D o
   vocabulário natural é: **cada componente emite `Signal` nomeados e aceita ações da tabela R3**.
2. **`Default controls` + `Simulate control`** (Construct, lei nº 1 do gênero): todo componente que
   reage a input LÊ `ActionState`, NUNCA dispositivo — e o `ActionState` é escrevível por IA, replay,
   rede e testes. Um clique liga o "anda sozinho com as setas" (demo instantânea); desligar o bool
   transforma o MESMO componente em motor puro. É também o que torna o smoke do Enio auto-play
   (feedback_ready_to_smoke_example).
3. **Toda propriedade de editor tem `Set` em runtime + bool `Enabled` + (quando faz sentido)
   `Preview` no editor** (Construct/Godot): nada é initial-value-only. No PH2D isso é barato: config
   plain-data registrada já persiste e entra no undo de graça (inventário §c).
4. **Marcadores são componentes** (Solid/Persist/No-save do Construct; `Replicated`/`Save` do Bevy):
   componente de ZERO campos que muda como N sistemas tratam o objeto. Baratíssimo, alavancagem enorme.
5. **Required Components** (Bevy 0.15 — o meta-matador): o "Add Component" da UI insere o conceito
   dirigente e a engine completa as dependências em cascata (`AudioSource2D` puxa nada; `UiButton`
   puxa `UiAnchor`+picking; `AnimatedSprite` puxa `Sprite`). Sem isso o usuário reconstitui o grafo
   interno de cabeça — o erro "adicionei X, faltou Y, nada acontece" morre por construção.

⚠️ **Nota de custo transversal (do inventário §c):** os passos 1–4 de um componente novo (definir,
registrar, persistir, undo) são mecânicos e baratos; **o custo real é o passo 5 — a seção artesanal no
Inspector**. Um catálogo de ~40 componentes só é viável com o **inspector derivado do tipo**
(reflection/derive sobre plain-data, como `@export` do Godot / `go.property` do Defold / MonoBehaviour
da Unity) + a **UI genérica de "Add Component"**. Essa infra é pré-requisito de TODO o cardápio abaixo
e deve ser o item nº 0 da ordem de construção (é provável que outro domínio a reivindique também —
é dele que todos dependem).

---

## 1. INPUT — a maior lacuna bruta do PH2D neste domínio

Estado atual (`ph2d-input`): gamepad cru (press/held/released/axis), snapshot por tick, projetado no
Luau. **Sem ações nomeadas, sem teclado de gameplay, sem rebinding, sem touch.** Nenhuma das engines
"code-first" (Phaser, Cocos, GameMaker) tem action mapping — só Unity/Unreal/Godot/Defold têm, e é
consenso nos dossiês que é a categoria mais copiável: puro dado + pipeline valor→modifier→trigger.

### 1.1 `ActionMap` (asset) — P0
- **Entrega:** ações nomeadas ("Jump", "Move", "Fire") com N bindings por dispositivo
  (tecla/botão/eixo/composite WASD→Vec2), deadzone e escala — editadas numa UI de tabela. O gameplay
  nunca mais vê `KeyCode`; rebinding vira editar o mapa (a tela de settings de controles sai DE GRAÇA).
- **Equivalentes:** Unity Input Actions asset · Unreal Input Action + Mapping Context · Godot InputMap
  (projeto) + `get_vector()` · Defold input bindings (`game.input_binding`) · Bevy leafwing
  `InputMap`/`Actionlike`.
- **Já tem:** ✖ (`ph2d-input` é dispositivo cru).
- **Dependências:** nenhuma (é o alicerce da cadeia inteira). Teclado precisa entrar no snapshot de
  gameplay (hoje nem chega ao Luau).
- **Nota de desenho:** valor da ação tipado (bool / Axis1D / Axis2D) como no Enhanced Input; o
  composite "4 ações → vetor normalizado" (o `get_vector()` do Godot) é o movimento 8-direções em
  1 leitura.

### 1.2 `ActionState` (componente por entidade) — P0
- **Entrega:** o leitor do mapa COMO COMPONENTE do objeto (modelo leafwing): `pressed/just_pressed/
  value/axis_pair` por ação. Por ser por-entidade: multiplayer local = 2 entidades com mapas
  diferentes; IA/replay/rede ESCREVEM o estado (lei nº 2 do §0); testes mockam input.
- **Equivalentes:** leafwing `ActionState` · Unity `PlayerInput` · Construct `Simulate control`
  (a mesma ideia por dentro).
- **Já tem:** ✖.
- **Dependências:** 1.1. O `PlatformPlayer` da física e o Luau (`ph2d.input`) migram para ler daqui.

### 1.3 `InputTriggers` & `InputModifiers` (no binding) — P1
- **Entrega:** tap vs hold vs double-tap, **combo** (sequência com janela — fighting game por asset),
  **chord** (Shift+X), pulse; modifiers negate/swizzle/response-curve/smooth. Elimina TODO o parsing
  temporal de input que hoje vira `if` acumulando timestamps.
- **Equivalentes:** Unreal Enhanced Input (9 triggers + 11 modifiers — a lista de referência) ·
  leafwing chords/clash strategy · GDevelop "Konami Code" (extensão).
- **Já tem:** ✖. **Depende de** 1.1/1.2.

### 1.4 `InputContext` (pilha de contextos com prioridade) — P1
- **Entrega:** contextos empilháveis (gameplay / veículo / menu / diálogo) que resolvem conflito de
  tecla sozinhos; abrir o menu desliga o pulo sem um `if` sequer. O modelo Defold acrescenta o
  detalhe certo: **quem está no topo pode CONSUMIR o input** (modal de graça).
- **Equivalentes:** Unreal Input Mapping Context (prioridade) · Defold input stack + consumo.
- **Já tem:** ✖. **Depende de** 1.1.

### 1.5 `PointerTarget` (picking de gameplay) — P1
- **Entrega:** hover/click/drag/drop em QUALQUER objeto do mundo por presença de componente — a UI
  dele elimina hit-test, captura de ponteiro e o protocolo de drag&drop inteiro. Eventos saem como
  `Signal` (`clicked`, `drag_start`, `dropped_on`).
- **Equivalentes:** bevy_picking (`Pickable` + observers Click/Drag/DragDrop) · Unity Physics 2D
  Raycaster · Phaser `setInteractive`+drag/drop zones · GM eventos de mouse sobre a máscara ·
  Construct Drag & Drop (behavior).
- **Já tem:** ◐ — o editor tem hit-test maduro (`input_dispatch.rs`, `vec_live_drawn` com 6 sítios de
  pick), mas nada exposto a gameplay.
- **Nota:** o colisor da física (`Collider`) e a forma do sprite são os dois backends naturais.

### 1.6 `TouchButton` — P1
- **Entrega:** botão de TELA para mobile: textura normal/pressed, área de toque (shape/bitmask),
  multitouch nativo, e — o detalhe certo do Godot — **dispara uma AÇÃO do `ActionMap`**, não um
  callback: o resto do jogo não sabe que veio de touch. `visibility_mode` esconde em desktop.
- **Equivalentes:** Godot TouchScreenButton (o modelo) · GDevelop Multitouch joystick/botões.
- **Já tem:** ✖. **Depende de** 1.1 + UI canvas (§2).

### 1.7 `VirtualJoystick` — P1
- **Entrega:** joystick virtual multitouch que alimenta uma ação Axis2D do mapa; deadzone e retorno
  ao centro prontos. Par do 1.6 para o movimento.
- **Equivalentes:** GDevelop Multitouch joystick (behavior revisado) · dezenas de third-party em
  Unity/Phaser (a ausência nos cores é a oportunidade).
- **Já tem:** ✖. **Depende de** 1.1/1.6.

### 1.8 `LocalPlayerManager` — P2
- **Entrega:** "join when button pressed": pareamento dispositivo↔jogador, spawn de prefab por
  jogador, split-screen automático. Co-op de sofá sem código.
- **Equivalentes:** Unity `PlayerInputManager` (o único first-party) · GM viewports (split-screen).
- **Já tem:** ✖. **Depende de** 1.1/1.2, spawner (domínio de spawn), múltiplas câmeras.

---

## 2. UI IN-GAME — a fundação já existe; falta a metade de RUNTIME

Estado atual: a família `Vec*` já é uma suíte de UI AUTORADA como componentes ECS (`VecWidget`/
`VecWidgetBind`/`VecWidgetValue`/`VecWidgetIcon`, `VecLayout*` sobre **taffy** — a MESMA taffy do
Bevy UI —, `VecAnchors`, `VecLabel`), com **Smart Animate** (`ph2d-ui-state`: estados idle/hover/press
com tween automático via `ph2d-vec-blend` + OKLab) e "a árvore autorada como painel vivo". Isso é uma
fundação MELHOR que a de Godot/Unity no ponto de partida. O que falta é a metade que o JOGO toca:
texto mutável por script, botão que dispara sinal em gameplay, âncora sobre o `GameRt`, foco de gamepad.

### 2.1 `UiCanvas` — P0
- **Entrega:** raiz de HUD em screen-space sobre o `GameRt`, com resolução de referência e política de
  escala (fit width/height) — o HUD sobrevive a resize/fullscreen sem código. Camada separada do mundo
  (a câmera de jogo não o vê).
- **Equivalentes:** Unity Canvas + Canvas Scaler · Cocos Canvas · GM UI Layers (globais ao projeto —
  ideia boa: HUD atravessa cenas) · Defold GUI (imune à câmera) · Godot CanvasLayer.
- **Já tem:** ◐ — `GameRt` (render target do mundo) e widgets Vec existem; falta a raiz screen-space
  de jogo com escala por resolução.
- **Dependências:** nenhuma dura; âncora (2.2) mora nela.

### 2.2 `UiAnchor` — P0
- **Entrega:** prende bordas do elemento às bordas da tela/pai (px, % ou proporcional; ancorar bordas
  opostas ESTICA). Elimina TODO o reflow de HUD por resolução — o código que todo jogo reescreve.
- **Equivalentes:** Cocos **Widget** (o melhor da categoria: px/%, Target, AlignMode) · Unity
  RectTransform · Construct/GDevelop Anchor (GDevelop soma Window center + Proportional) · Defold
  X/Y Anchor + Adjust Mode.
- **Já tem:** ◐ — `VecAnchors` existe (autoria); estender a semântica ao viewport de JOGO.

### 2.3 `UiButton` — P0
- **Entrega:** botão com estados visuais (normal/hover/pressed/disabled — **o Smart Animate já anima
  a transição**) que ao clicar **publica um `Signal` nomeado** — ligado a uma ação na tabela R3 (§8)
  pela UI, zero listener em código. É o UnityEvent/ClickEvents-no-Inspector traduzido para o idioma
  de sinais do PH2D (ADR-0075: produtor não chama ninguém).
- **Equivalentes:** Cocos Button (Transition COLOR/SPRITE/SCALE + ClickEvents no Inspector — o
  modelo) · Unity uGUI Button + UnityEvents · GDevelop Button states/Labeled button · Godot Button.
- **Já tem:** ◐ — `VecWidget` + `ph2d-ui-state` (estados + tween) + gate de hover (`hover_live`);
  falta o clique publicar `Signal` em modo jogo. ⚠️ Cerca conhecida: superfície `Plain` nova que lê
  `hover_live` sem estar no mapa nasce muda (§5 do CLAUDE.md) — o censo é por componente, não global.
- **Depende de** 1.5 (picking) + 8.1 (SignalActions dá o "o que acontece ao clicar").

### 2.4 `UiLabel` (texto dinâmico) — P0
- **Entrega:** texto de HUD/mundo mutável em runtime (`set_text`) — placar, contador, diálogo, dano
  flutuante. A lacuna nº 13 do inventário, verbatim.
- **Equivalentes:** Defold **Label** (a separação label-no-mundo vs texto-de-GUI é a decisão certa) ·
  Godot Label · Cocos Label (+LabelOutline/LabelShadow como componentes EMPILHÁVEIS — padrão elegante
  de copiar) · Phaser BitmapText (barato).
- **Já tem:** ◐ — `VecTextPath` (texto vetorial AUTORADO, fonte vetorial própria); falta o
  componente com string mutável e binding ("mostra a variável X" via `VecWidgetBind`, que já existe
  para autoria).
- **Nota:** o par com `VecWidgetValue`/`VecWidgetBind` sugere o desenho: Label = widget cujo valor é
  um binding; o "Rolling counter"/"Animated score counter" do GDevelop vira um tween do valor (§5).

### 2.5 `UiProgressBar` — P1
- **Entrega:** barra linear E radial (cooldown circular) como MODO do sprite/widget (fill H/V/radial,
  fill start/range), não como feature — a lição do Sprite Filled do Cocos: vida, mana, recarga sem
  shader nem código. Casa com a expression `progress` do Timer (§4.1).
- **Equivalentes:** Cocos Sprite Filled + ProgressBar · Unity Image Filled · Defold GUI Pie ·
  GDevelop Resource bar (contínua/unidades).
- **Já tem:** ✖ (o `Sprite` v4 não tem modo fill).

### 2.6 `UiLayoutGroup` — P1
- **Entrega:** expor o auto-layout taffy (`VecLayout*`, ADR-0153) como componente de HUD:
  horizontal/vertical/grid, padding/gap, resize CHILDREN/CONTAINER — inventário, hotbar e menu sem
  posicionar um filho à mão.
- **Equivalentes:** Cocos Layout · Unity Layout Groups · GM Flex Panels (flexbox/Yoga — o mais rico).
- **Já tem:** ◐ — taffy integrada e gateada (a lei "o passe publica onde as coisas ficam, não escreve
  onde estão" já protege o undo).

### 2.7 `WorldSpaceWidget` — P1
- **Entrega:** um widget PRESO a um objeto do mundo (healthbar sobre a cabeça, balão de fala, placa):
  projeta mundo→tela ou vive no mundo com oclusão. Elimina a projeção + render-to-texture + roteamento
  de clique.
- **Equivalentes:** Unreal WidgetComponent (World/Screen Space — o modelo) · Unity World Space
  Canvas · Cocos UICoordinateTracker.
- **Já tem:** ✖ (a referência durável por `stable_name_id` resolve o "preso a quem").

### 2.8 `UiFocusNav` — P1
- **Entrega:** navegação de foco por gamepad/teclado entre widgets (vizinhos automáticos + override),
  com o realce vindo do Smart Animate. Sem isso, menu com gamepad é código artesanal.
- **Equivalentes:** Unity uGUI navigation (automática) · Godot focus neighbors.
- **Já tem:** ✖. **Depende de** 1.1 (ações de navegação) + 2.3.

### 2.9 `UiScrollView` — P2
- **Entrega:** rolagem com física (inércia, brake, bounce elástico) + máscara de viewport — a física
  de scroll que todo mundo escreve errado.
- **Equivalentes:** Cocos ScrollView (Inertia/Brake/Elastic — a referência) + Mask + ScrollBar ·
  Unity Scroll View.
- **Já tem:** ✖ (`Mask2D`/`ClipChildren` já dão o recorte).

### 2.10 `UiModalBlocker` + `SafeArea` — P2
- **Entrega:** dois marcadores (lei nº 4 do §0): um componente VAZIO que impede o input de atravessar
  o fundo do diálogo (o `BlockInputEvents` do Cocos — modal correto por PRESENÇA); e o ajuste a
  notch/safe-area mobile.
- **Equivalentes:** Cocos BlockInputEvents/SafeArea (únicos com nome próprio).
- **Já tem:** ✖. **Depende de** pilha de input (1.4).

---

## 3. ÁUDIO DE GAMEPLAY — categoria NOVA (o rack de 42 efeitos é todo editor-side)

Estado atual: `ph2d-audio*` tem rack com 42 efeitos + 23 presets, espectral, export, streaming, AI
denoise — **tudo documento do editor; nada é componente de cena**. Não existe emissor posicional, nem
listener, nem "toca este clip quando este sinal chega" (o R3). `SimRef` já prevê "audio spatial" no
doc-comment. É a lacuna nº 4 do inventário.

### 3.1 `AudioSource2D` — P0
- **Entrega:** emissor posicional no objeto: clip, volume, pitch, loop, autoplay, bus de saída,
  atenuação por distância (curva), pan pela posição na tela. O som segue o objeto; morrer o objeto
  mata o som (ou o modo fire-and-forget do Bevy: `Despawn` ao terminar limpa sozinho).
- **Propriedades que os dossiês elegeram como as CERTAS:** `max_polyphony` (Godot — N sons simultâneos
  do MESMO emissor sem cortar o anterior: o tiro rápido), **`non_spatialized_radius`** (Unreal — perto
  da fonte o som vira 2D suavemente, matando o "pan pulando" quando o player passa por cima — o bug
  clássico do posicional 2D), atenuação como **asset compartilhável** (Unreal Sound Attenuation:
  formas esfera/cápsula/box/cone, falloff custom) — 1 asset serve 200 sons.
- **Equivalentes:** Godot AudioStreamPlayer2D (o modelo 2D) · Unity AudioSource (Spatial Blend) ·
  Unreal AudioComponent + Attenuation asset · Bevy AudioPlayer + PlaybackSettings · Phaser spatial
  audio · Defold Sound (só pan — o contra-exemplo).
- **Já tem:** ✖. **Dependências:** ponte mixer→cena (3.3); posição via `SimRef` (já previsto).
- **Risco/desenho:** a fronteira dura já gateada ("nenhum codec alcança o mixer RT") continua valendo
  — o componente é CONFIG; o streaming de vozes (ADR-0118) é quem toca.

### 3.2 `AudioListener2D` — P0
- **Entrega:** o "microfone" (1 ativo por cena; default = câmera de jogo). Sem ele não há posicional.
  `make_current` para câmera desacoplada do player (Godot).
- **Equivalentes:** Unity AudioListener · Godot AudioListener2D · Bevy SpatialListener.
- **Já tem:** ✖. Par do 3.1; depende da câmera de gameplay (domínio vizinho).

### 3.3 `AudioBus` (mixer de runtime) — P1
- **Entrega:** grupos hierárquicos (Master/Música/SFX/UI) com volume, e **a ponte para o rack de 42
  efeitos como cadeia de bus em runtime** — o ativo mais subaproveitado do PH2D vira o mixer de jogo.
  **Snapshots** (Unity) — estados de mixagem nomeados com transição ("combate"/"exploração") — e
  ducking são a metade de cima.
- **Equivalentes:** Unity Audio Mixer + snapshots (o modelo) · Godot buses (+ `Area2D.audio_bus_override`) ·
  GM audio buses + effects.
- **Já tem:** ◐ — a cadeia de efeitos existe (editor-side, invariante no-op no ponto neutro, painel
  auto-populado da tabela `KINDS`); falta o grafo de buses em runtime e o roteamento por componente.
- **Depende de** 3.1 (o campo `bus` do source).

### 3.4 `MusicPlayer` — P1
- **Entrega:** música global (não-posicional) com crossfade entre faixas, intro+loop points (GM), e
  **camadas sincronizadas por sample** (GM Sync Groups — música adaptativa: entra a camada de
  percussão quando o combate começa). Zero código de DJ.
- **Equivalentes:** GM (loop points + sync groups — a referência técnica) · Unity/Godot via código.
- **Já tem:** ✖ (streaming de vozes ADR-0118 é a infra).
- **Nota:** os 3 itens de Chesterton do backlog de áudio (seek/scrub em stream, pitch ao vivo em
  stream, toggle "Streamed") esperavam exatamente este consumidor — conferir
  `docs/Audio/03_o_que_falta.md` antes de abrir.

### 3.5 `AudioZone` — P2
- **Entrega:** área que troca o bus/aplica efeito a quem está dentro (reverb de caverna, abafado
  debaixo d'água = LowPass no listener) — por shape + dropdown de bus.
- **Equivalentes:** Godot `Area2D.audio_bus_override` (o desenho mais limpo) · Unity Reverb Zone +
  Filters empilháveis · Unreal occlusion/reverb send.
- **Já tem:** ✖ — mas a família de ZONAS da física (`AreaEffector`…) é o molde exato: mesmo shape,
  outro payload. **Depende de** 3.3.
- (O "toca som quando o sinal chega" NÃO é componente próprio — é uma AÇÃO da tabela R3, §8.1.)

---

## 4. TEMPO

### 4.1 `Timer` — P0
- **Entrega:** contagem com evento — one-shot ou periódico, autostart, pausável — que ao disparar
  **publica `Signal`**. Elimina a contabilidade de dt de cooldowns/respawns/ondas: o caso perfeito de
  componente barato de fazer e de altíssimo uso (veredito unânime dos dossiês).
- **Os detalhes CERTOS a copiar:** timers **NOMEADOS, N por instância** (Construct — tags) ·
  expression **`progress` 0–1** (Construct — alimenta a barra de recarga de graça, §2.5) · escolha do
  relógio (idle/physics + `ignore_time_scale`, Godot) · o timer morre com o dono (Cocos scheduler —
  sem leak).
- **Equivalentes:** Godot Timer (node→componente 1:1) · Construct Timer (behavior, o mais rico) · GM
  alarms (12/instância — a prova de demanda) · Defold/Phaser/UE timers por API (a UX de componente é
  o diferencial).
- **Já tem:** ✖. **Dependências:** Signal (existe). É possivelmente o item de melhor razão
  custo/benefício de TODO o levantamento.

### 4.2 `TimeChannel` — P1
- **Entrega:** relógios locais: um grupo de objetos com `time_scale` próprio (pause de mundo com menu
  vivo, slow-motion de área, bullet-time) — o `set_time_step` por proxy do Defold e o `timeScale` por
  timer do Phaser, como componente/recurso. Sem ele, "pausar o jogo" vira um `if` em cada sistema.
- **Equivalentes:** Defold collection proxy `set_time_step` (a referência) · Phaser timeScale por
  timer/cena · Unity `Time.timeScale` (global — o contra-exemplo: tudo ou nada).
- **Já tem:** ◐ — `Playhead` global (`ph2d-core`) + a porta `time` opcional dos nodes
  (`oscillator`/`noise`/`wiggle` já aceitam relógio por elemento — o precedente interno).
- **Risco:** interage com o determinismo da física (o fixed-step do rapier é global); a física
  provavelmente só respeita pause/scale do canal do MUNDO, não por-objeto — decidir cedo e gatear.

---

## 5. TWEEN & JUICE — o Godot ainda exige 1 linha de código; o PH2D pode exigir ZERO

### 5.1 `Tween` (componente + `TweenPreset` asset) — P1
- **Entrega:** anima QUALQUER propriedade de QUALQUER componente registrado (posição, escala, cor,
  opacity, um campo do usuário) com easing, duração, delay, loop/ping-pong, destroy-on-complete —
  **autorado no Inspector e disparado por `Signal`** (da tabela R3, de um botão, de um timer). O
  dropdown "propriedade a animar" é a lista de lenses (Bevy) = o `ComponentRegistry` que já existe.
- **Equivalentes:** Construct Tween (24 actions — o mais completo) · GDevelop Tween (cor em HSL,
  escala exponencial — dois acertos técnicos a copiar) · Godot Tween (código, 12 curvas × 4 eases) ·
  Defold go.animate (40 easings, anima até constante de shader) · Phaser (stagger com grid! —
  cascatas "cada moeda 50ms depois da anterior") · bevy_tweening (lenses) · UE TimelineComponent.
- **Já tem:** ◐ — `ph2d-anim` (curvas/easing/extrapolação target-agnóstico) + `ph2d-vec-blend` +
  OKLab + Smart Animate: TODO o motor existe; falta o empacotamento componente+preset+trigger.
- **Depende de** 8.1 (trigger por sinal) + inspector derivado (§0).
- **Nota:** `stagger` (Phaser) e a interpolação perceptual (OKLab — já é a do Smart Animate) são os
  dois diferenciais que nenhuma engine entrega juntos.

### 5.2 `Fade` — P1
- **Entrega:** o ciclo de vida visual universal: fade-in → wait → fade-out → (opcional) destruir.
  Poeira, popup de dano, corpo de inimigo — o "aparece-espera-some" sem timeline nem código.
- **Equivalentes:** Construct Fade (o modelo, com preview) · GDevelop Fade/Tween into view.
- **Já tem:** ✖ (açúcar sobre 5.1 — implementar como preset nomeado).

### 5.3 `Flash` — P1
- **Entrega:** pisca visível/invisível por N segundos — o feedback de dano/i-frames padrão da
  indústria, com a condition "acabou de piscar" (fim da invencibilidade) publicada como `Signal`.
- **Equivalentes:** Construct Flash · GDevelop Flash object. O `tint_fill` do Sprite v4 (flash de
  dano estilo Phaser) **já existe** — o componente só o anima.
- **Já tem:** ◐ (`tint_fill` pronto; falta o oscilador temporal).

### 5.4 `Oscillator` — P1
- **Entrega:** oscila uma propriedade (posição/escala/ângulo/opacity/"value only") com forma de onda,
  período e magnitude — **com `random` de fase/período embutido**: as 100 moedas da fase NÃO dançam
  em fase (o bug estético resolvido por default — lei nº 6 do Construct). O idle-motion de um jogo
  inteiro sem eventos.
- **Equivalentes:** Construct Sine (o modelo exato, 5 formas de onda + preview) · GDevelop Sway.
- **Já tem:** ◐ — `motion.oscillator`/`noise`/`wiggle` EXISTEM como Motion Nodes (com relógio por
  elemento!). O componente é o ATALHO de 1 clique que instancia o node por baixo — decidir se é
  açúcar sobre o grafo ou componente irmão (evitar dois motores para a mesma lei —
  feedback_two_engines_one_state).

---

## 6. ANIMAÇÃO DE PERSONAGEM — flipbook, máquina de estados, notifies

Estado atual: `Sprite` v4 tem sprite-sheet inline (hframes×vframes×frame); `SpriteAnimation`
(Component) amostra um `Clip` no Playhead escrevendo Transform; a Timeline é um sequencer completo
(clips, composição, nesting, curvas weighted). O que NÃO existe: clips de FRAME nomeados com play(),
máquina de estados de animação, blend direcional, evento em frame de flipbook.

### 6.1 `SpriteFrames` (asset) + `AnimatedSprite` — P0
- **Entrega:** animações de frames NOMEADAS ("idle", "run", "attack") com FPS e loop por animação,
  editadas visualmente; `play("run")` troca; `Signal` em `animation_finished`. Elimina a máquina de
  troca de frames que o dossiê Godot chama de "o componente nº 1 de qualquer engine 2D" — e que a
  Unity notoriamente NÃO tem em versão leve (o Animator é pesado demais para flipbook — lacuna
  declarada no dossiê).
- **Equivalentes:** Godot AnimatedSprite2D + SpriteFrames (o modelo) · UE PaperFlipbook · Phaser anims
  (generateFrameNames — a ergonomia de atlas) · bevy_spritesheet_animation (markers em frame) · GM
  image_index/image_speed (built-in).
- **Já tem:** ◐ — a grade inline do Sprite é a metade de baixo; falta o asset de clips nomeados + o
  player. **Dependências:** nenhuma dura. É o degrau para 6.2–6.4.

### 6.2 `AnimationNotify` (evento em frame) — P1
- **Entrega:** marcador num frame/tempo do clip que **publica `Signal`** ("footstep", "spawn_hitbox",
  "frame_7") — som do passo, janela de hitbox e spawn de efeito sincronizados sem polling. O PH2D já
  tem o mecanismo EXATO na timeline (markers → Signal, ADR-0143): isto é estendê-lo ao flipbook.
- **Equivalentes:** Unity Animation Events · UE/PaperZD AnimNotifies (+ NotifyStates com duração —
  a janela de hitbox como INTERVALO, não instante: copiar) · Cocos frame events · Spine events.
- **Já tem:** ◐ (mecanismo pronto na timeline; falta no flipbook). **Depende de** 6.1.

### 6.3 `AnimationStateMachine` — P1 (candidato a P0 do pipeline de personagem)
- **Entrega:** grafo visual estados→clips com transições condicionais sobre PARÂMETROS (float/bool/
  trigger), exit time, crossfade, e `travel()` (Godot — acha o caminho entre estados sozinho). Mata o
  espaguete `if state ==` que todo jogo acumula — o maior eliminador de código de animação segundo
  3 dossiês (Unity, Godot, UE/PaperZD).
- **Os detalhes certos:** Transitional States (PaperZD — "aterrissando"→"parado" flui sozinho) ·
  JumpNodes (interrupção de qualquer estado: dano/morte) · layers com máscara (tronco atira enquanto
  pernas correm — P2 dentro do item).
- **Equivalentes:** Unity Animator Controller · Godot AnimationTree state machine · UE PaperZD (o
  espelho 2D exato) · seldom_state (Bevy — estado-como-componente: queries filtram por estado, o
  desenho ECS certo).
- **Já tem:** ◐ — `ph2d-ui-state::Machine` (Smart Animate) é o embrião declarado no inventário: FSM
  sem relógio próprio, com tween automático entre estados. **Depende de** 6.1 (clips) + parâmetros
  vindos de 1.2/física.
- **Risco de desenho:** decidir cedo se é a MESMA máquina do Smart Animate (uma FSM, dois consumidores)
  ou irmã — duas máquinas de estado com semânticas quase-iguais é a receita de contagem dupla.

### 6.4 `ControllerAnimator` — P1
- **Entrega:** o componente-ponte que troca a animação pelo ESTADO do controller
  (idle/run/jump/fall/land) **sem nenhum evento nem grafo** — o caso comum do platformer em 1
  componente de 6 dropdowns (estado → clip). A ideia mais custo-eficiente do dossiê GDevelop
  (Platformer character animator): fecha o ciclo controller→animação por UI pura.
- **Equivalentes:** GDevelop animators (único com nome próprio) · nos demais, é o primeiro script que
  todo tutorial manda escrever.
- **Já tem:** ✖ — mas `PlayerSignals`/`PlayerEvent` do `ph2d-platformer` JÁ publicam os estados: o
  componente é uma tabela sinal→clip. **Depende de** 6.1 + player (existe).

### 6.5 `BlendSpace1D/2D` — P2
- **Entrega:** blend direcional (idle→walk→run por velocidade; 4/8 direções por vetor) — o top-down
  sem switch-case. Só faz sentido depois de 6.3.
- **Equivalentes:** Unity Blend Trees · Godot BlendSpace1D/2D · PaperZD animações direcionais (para
  flipbook, a variante certa: N variantes por direção, o sistema escolhe pelo vetor).
- **Já tem:** ✖.

### 6.6 `PropertyTrack` (timeline anima qualquer componente) — P1
- **Entrega:** qualquer propriedade de qualquer componente REGISTRADO é keyframável na Timeline (não
  só Transform) — o que o dossiê Cocos chama de "a cola entre animação e componentes" e o Sequencer
  da UE de Property Track. Com o `ComponentRegistry` + reflection do inspector derivado (§0), o custo
  marginal é baixo e o retorno é o produto inteiro ("animar o `fill` da barra de vida", "animar o
  `volume` do AudioSource").
- **Equivalentes:** Godot AnimationPlayer ("qualquer propriedade é keyframável por default" — a
  régua) · UE Property Tracks · Cocos Animation Editor sobre `@property`.
- **Já tem:** ◐ — TimelineDoc anima canais próprios + `SpriteAnimation` escreve Transform; a
  generalização é a wave.

---

## 7. SEQUENCER EM RUNTIME — a Timeline já vence o mercado; falta a PONTE para o jogo

A Timeline do PH2D (dope-sheet, curvas weighted, clips+composição+NESTING, motion path, retiming,
extrapolação, sinais ADR-0143, expressões) está À FRENTE de Godot (AnimationPlayer), Construct
(Timelines), GM (Sequences) e do ecossistema Bevy inteiro (AUSENTE) — só Unity Timeline e UE
Sequencer competem. O delta não é o editor: é o que falta para uma cena de JOGO tocá-la.

### 7.1 `SequencePlayer` — P0
- **Entrega:** o componente que TOCA um TimelineDoc sobre objetos da cena em runtime: bindings por
  referência durável, play/pause/seek/wrap-mode, "play on signal" (cutscene dispara da tabela R3),
  desligar input do player e esconder HUD enquanto toca (os dois checkboxes do Level Sequence Actor
  da UE — exatamente os dois `if` que todo dev de cutscene escreve). Sem ele, o módulo Timeline
  inteiro é editor-only.
- **Equivalentes:** Unity Playable Director · UE Level Sequence Actor · GM Sequences (instanciável
  na room — anima instâncias VIVAS) · Godot (o AnimationPlayer É o sequencer).
- **Já tem:** ◐ GRANDE — `TimelineDoc` + `TargetBinding` com **`WireId` = hash do Name, religado no
  upkeep do frame**: a parte DIFÍCIL (religar bindings a entidades respawnadas) já está resolvida e
  shipada. O componente é empacotamento + trigger.
- **Depende de** 8.1 (o trigger autorável) e habilita 7.2.

### 7.2 Tracks novas de gameplay — P1 (lista, por valor decrescente)
- **`ActivationTrack`** — liga/desliga objetos por clipe (Unity Activation Track): a porta que abre no
  segundo 3 da cutscene. `Visibility`+`OnScreenEnabler` já são os alvos.
- **`AudioTrack`** — clip de som na timeline (todas as engines com sequencer têm; o PH2D tem o módulo
  de áudio inteiro esperando). Depende de 3.1/3.3.
- **`ControlTrack`** — a timeline manda no TEMPO de um sub-sistema (partículas com scrub! — Unity
  Control Track): os Motion Nodes GPU-resident com playhead dirigido pela timeline seriam um
  diferencial de demo imenso. Depende de 4.2 (relógio local).
- **SignalTrack** — ✅ JÁ EXISTE (markers → Signal, ADR-0143). Listado para completude.
- **Equivalentes:** Unity Timeline tracks · UE Sequencer (com **Customizable Sequencer Tracks** — o
  usuário cria tipos de track: P2 distante, mas anotar).
- **Já tem:** ◐ (dope-sheet e infraestrutura prontas; cada track é uma wave pequena).

### 7.3 `TimeDilationTrack` — P2
- **Entrega:** câmera lenta autorada na timeline (UE Time Dilation) — juice de cutscene.
  **Depende de** 4.2.

---

## 8. SINAIS ENTRE OBJETOS — a peça-chave do domínio inteiro

Estado atual: `ph2d-runtime` (Signal/SignalOrigin/SignalOutbox/SignalReader) com ordem no quadro
GATEADA e custo medido (8 consumidores = 1,00× o de 2). Produtores prontos: timeline (markers),
física (`SignalOnHit`/`SignalOnLeave`), player (`PlayerSignals`). **O que falta é exatamente o R3,
nomeado no inventário: NADA autorável reage.** Hoje o consumidor visível é toast/log.

### 8.1 `SignalActions` (a tabela nome→ação) — P0 ⭐ o item mais importante desta síntese
- **Entrega:** a tabela autorável "quando chegar o sinal X → faça Y", editada por dropdowns: tocar
  som (3.1) · spawnar prefab · ativar/desativar objeto · setar propriedade · disparar Tween (5.1) ·
  iniciar Timer (4.1) · tocar animação (6.1) / trocar estado (6.3) · tocar sequência (7.1) · trocar
  cena · incrementar contador/variável · emitir OUTRO sinal (encadeamento). É o UnityEvent + o dock
  de sinais do Godot traduzidos para o idioma event-sourced do PH2D (ADR-0075: o produtor publica,
  a tabela consome com cursor próprio — o desacoplamento já está garantido por construção).
- **Equivalentes:** Godot sinais conectados PELA UI (o padrão-ouro de "reduzir programação via UI" —
  dossiê Godot §T) · Unity UnityEvents/Signal Receiver · UE Event Track/Blueprint events · Construct
  event sheet (a fronteira onde behaviors acabam) · Bevy Observers (a espinha técnica: reação
  imediata, direcionável a entidade).
- **Já tem:** ✖ a tabela; ✅ TODO o transporte. O trabalho é "conteúdo autorado + UI", verbatim do §5.
- **Dependências:** cada AÇÃO da tabela é fornecida por outro componente do cardápio (o vocabulário
  da lei nº 1) — construir a tabela CEDO com 3 ações (som, spawn, enable) e crescer o dropdown a cada
  componente novo. **Ordem: logo depois de 3.1 e 4.1.**
- **Risco/desenho:** ⚠️ a adjacência de nome `ph2d-runtime` vs "runtime de UI" está ABERTA no §5
  (decisão do Enio; a linha recomenda crate irmã — senão o gate `the_event_core_is_a_leaf` é
  revogado deliberadamente). A tabela R3 deve nascer na crate CERTA desde o dia 1.
- **Payload:** hoje `Signal` é nome (String). Argumentos tipados (quem bateu, quanto de dano) são a
  extensão natural — decidir o formato cedo (BTreeMap determinístico, HR-5) porque o undo/replay
  serializa.

### 8.2 Família `SignalEmitter` — P1
- **Entrega:** completar o lado PRODUTOR com marcadores baratos (lei nº 4): `SignalOnTimer` (açúcar
  do 4.1) · `SignalOnInput` (ação do mapa → sinal: "apertou Interact perto de mim") · `SignalOnScreenEnter/Exit`
  (o VisibleOnScreenNotifier2D do Godot — despawn de projétil, "inimigo acordou") · `SignalOnAnimEnd`
  (6.1) · `SignalOnClick` (1.5). Cada um é um componente de 1–3 campos que publica no outbox.
- **Equivalentes:** Godot signals por node (o catálogo a espelhar) · Construct conditions.
- **Já tem:** ◐ (`SignalOnHit`/`OnLeave` são o molde exato).

### 8.3 `Tag` (grupos/famílias) — P1
- **Entrega:** pertencimento múltiplo por checkbox ("enemies", "coins") consultável e endereçável:
  broadcast de sinal por tag ("todos os guards → alert"), filtro de colisão/turret/ações. Elimina as
  listas manuais de referências — o que GM (parent-como-grupo), Godot (groups) e Construct (families)
  provam por três caminhos diferentes.
- **Equivalentes:** Godot Groups · Construct Families (+ behaviors por família — P2) · GM parent ·
  Unity tags (fraco — 1 só por objeto: o contra-exemplo).
- **Já tem:** ✖ (`stable_name_id` é referência 1:1 durável — a infra de hash de nome serve de base).
- **Depende de** nada; potencializa 8.1 (ação "enviar a todos com tag X").

---

## 9. SCRIPTING & A FRONTEIRA COMPONENTE-VS-SCRIPT

Estado atual: `LuauScript` (Component registrado/persistível, bytecode compartilhado, estado
por-instância determinístico que sobrevive a hot-reload, `ScriptHost` com set/get por entidade,
input, spawn, MessageBus) — **wired no boot com placeholder e ZERO UI** (lacuna nº 7 do inventário).

### 9.1 `ScriptProperties` + attach visual do LuauScript — P0
- **Entrega:** (a) anexar um script a um objeto pela UI (Add Component → Script → escolher asset);
  (b) **parâmetros declarados no script viram campos no Inspector, POR INSTÂNCIA** — o mesmo script
  vira N comportamentos configurados por UI, sem herança nem duplicação. É o mecanismo nº 1 de
  "script parametrizável" em TODAS as engines maduras, por unanimidade dos dossiês.
- **Equivalentes:** Defold `go.property` (o modelo mais limpo: tipos incluem RESOURCES, override em
  azul com reset, injeção no spawn) · Godot `@export` (range/enum/flags/file → UI rica só com
  anotações) · Unity `[SerializeField]` · Cocos `@property` · UE Instance Editable.
- **Já tem:** ◐ — componente, host, persistência e determinismo PRONTOS; falta 100% da UI.
- **Dependências:** inspector derivado (§0) — os campos exportados usam a mesma infra.
- **A fronteira (regra de produto sugerida):** componente para o que 80% dos jogos usam igual
  (catálogo); script para a regra ÚNICA do jogo do usuário; e o script se liga ao mundo pelos MESMOS
  sinais/ações da tabela R3 (o vocabulário da lei nº 1) — nunca por referência direta a outro objeto.

### 9.2 `ScriptEventHooks` — P1
- **Entrega:** o script declara handlers por NOME (`on_signal("door_open")`, `on_action_pressed("Jump")`,
  `on_timer("respawn")`) e publica ações próprias no dropdown da tabela R3 — o script vira um
  FORNECEDOR de vocabulário, indistinguível de um componente nativo na UI.
- **Equivalentes:** GDevelop events-based behaviors (funções custom viram conditions/actions no
  vocabulário — o funil que gerou 200+ behaviors) · Construct Custom Actions.
- **Já tem:** ◐ (`MessageBus` com interning + handlers é a metade de baixo). **Depende de** 8.1 + 9.1.

### 9.3 `GameplayGraph` (visual scripting) — P2, com a lição gravada
- **Entrega:** SE vier, um grafo DE DOMÍNIO sobre o `ph2d-nodegraph` que já existe (o motor dos
  Motion Nodes): nós de sinal/condição/ação de GAMEPLAY — não um "Luau visual" de fluxo genérico.
- **A lição medida (dossiê Godot §U):** o VisualScript do Godot foi **REMOVIDO no 4.0 por nunca ter
  ganho tração** — visual scripting genérico de fluxo de controle fracassou; o que funciona é (a) UI
  declarativa por componente + sinais conectáveis (Godot), (b) event sheet como TABELA (Construct/
  GDevelop), (c) grafos de domínio (shader/partícula/motion). O PH2D já apostou certo com os Motion
  Nodes. **A tabela R3 (8.1) + FSM visual (6.3) + Luau com properties (9.1) cobrem o espectro; este
  item só se justifica se, depois delas, sobrar demanda real.**
- **Equivalentes:** Unity Visual Scripting (Script/State Machine) · UE Blueprints (o único sucesso
  pleno — e é uma linguagem inteira, não um item de catálogo) · GDevelop event sheets.
- **Já tem:** ◐ a infraestrutura (`ph2d-nodegraph`, contrato congelado §6 — mexer = ADR).

### 9.4 `UserBehavior` (funil comunidade→catálogo) — P2
- **Entrega:** empacotar {tabela R3 + FSM + Timer + Tween + properties} configurados como um
  COMPONENTE nomeado reutilizável (o "behavior custom sem código" do GDevelop — a razão estrutural
  de a cauda longa deles ser 6× maior que a do Construct). O multiplicador de longo prazo do
  catálogo inteiro.
- **Já tem:** ✖. **Depende de** praticamente tudo acima — é o teto do domínio, não o começo.

---

## 10. Ordem de construção recomendada (dependências resolvidas)

```
Fase 0 (infra, pré-requisito de tudo):
  Inspector derivado do tipo + UI "Add Component" + Required Components
Fase 1 (o mínimo que transforma sinais em jogo):
  Timer → AudioSource2D + AudioListener2D → SignalActions (R3, com 3 ações) → ActionMap + ActionState
Fase 2 (personagem completo por UI):
  SpriteFrames/AnimatedSprite → AnimationNotify → ControllerAnimator → SequencePlayer
  → UiCanvas/UiAnchor/UiLabel/UiButton (HUD) → ScriptProperties (Luau UI)
Fase 3 (facilidade que vira demo):
  Tween/Fade/Flash/Oscillator → InputTriggers/InputContext → PointerTarget → Tag + SignalEmitters
  → AnimationStateMachine → AudioBus/MusicPlayer → ProgressBar/LayoutGroup/WorldSpaceWidget/FocusNav
  → TouchButton/VirtualJoystick → TimeChannel → PropertyTrack + ActivationTrack/AudioTrack/ControlTrack
Fase 4 (teto):
  BlendSpace → ScrollView/Modal/SafeArea → AudioZone → LocalPlayerManager → TimeDilation
  → ScriptEventHooks → GameplayGraph? → UserBehavior
```

---

## 11. Tabela-resumo (nome · prioridade · já existe?)

| # | Componente | P | Já tem? (crate) |
|---|---|---|---|
| 1 | `ActionMap` (asset) | P0 | ✖ (`ph2d-input` é cru) |
| 2 | `ActionState` | P0 | ✖ |
| 3 | `InputTriggers`/`InputModifiers` | P1 | ✖ |
| 4 | `InputContext` (pilha) | P1 | ✖ |
| 5 | `PointerTarget` (picking) | P1 | ◐ (hit-test do editor) |
| 6 | `TouchButton` | P1 | ✖ |
| 7 | `VirtualJoystick` | P1 | ✖ |
| 8 | `LocalPlayerManager` | P2 | ✖ |
| 9 | `UiCanvas` | P0 | ◐ (`GameRt` + Vec widgets) |
| 10 | `UiAnchor` | P0 | ◐ (`VecAnchors`) |
| 11 | `UiButton` | P0 | ◐ (`VecWidget` + `ph2d-ui-state`) |
| 12 | `UiLabel` | P0 | ◐ (`VecTextPath` autorado) |
| 13 | `UiProgressBar` | P1 | ✖ |
| 14 | `UiLayoutGroup` | P1 | ◐ (`VecLayout*`/taffy) |
| 15 | `WorldSpaceWidget` | P1 | ✖ |
| 16 | `UiFocusNav` | P1 | ✖ |
| 17 | `UiScrollView` | P2 | ✖ |
| 18 | `UiModalBlocker`+`SafeArea` | P2 | ✖ |
| 19 | `AudioSource2D` | P0 | ✖ (rack é editor-side) |
| 20 | `AudioListener2D` | P0 | ✖ |
| 21 | `AudioBus` (mixer runtime) | P1 | ◐ (rack 42 efeitos, `ph2d-audio-edit`) |
| 22 | `MusicPlayer` | P1 | ✖ |
| 23 | `AudioZone` | P2 | ✖ (molde: zonas da física) |
| 24 | `Timer` | P0 | ✖ |
| 25 | `TimeChannel` | P1 | ◐ (`Playhead`; porta `time` dos nodes) |
| 26 | `Tween` + `TweenPreset` | P1 | ◐ (`ph2d-anim` + `ph2d-vec-blend` + Smart Animate) |
| 27 | `Fade` | P1 | ✖ (preset de 26) |
| 28 | `Flash` | P1 | ◐ (`tint_fill` pronto) |
| 29 | `Oscillator` | P1 | ◐ (`motion.oscillator` como node) |
| 30 | `SpriteFrames`+`AnimatedSprite` | P0 | ◐ (grade inline do `Sprite`; `SpriteAnimation`) |
| 31 | `AnimationNotify` | P1 | ◐ (markers→Signal na timeline, ADR-0143) |
| 32 | `AnimationStateMachine` | P1 | ◐ (`ph2d-ui-state::Machine` embrião) |
| 33 | `ControllerAnimator` | P1 | ✖ (`PlayerSignals` já publica os estados) |
| 34 | `BlendSpace1D/2D` | P2 | ✖ |
| 35 | `PropertyTrack` (timeline genérica) | P1 | ◐ (TimelineDoc + registry) |
| 36 | `SequencePlayer` | P0 | ◐ (TimelineDoc + `TargetBinding`/WireId prontos) |
| 37 | `ActivationTrack`/`AudioTrack`/`ControlTrack` | P1 | ◐ (infra da timeline) |
| 38 | `TimeDilationTrack` | P2 | ✖ |
| 39 | `SignalActions` (tabela R3) | **P0 ⭐** | ✖ tabela; ✅ transporte (`ph2d-runtime`) |
| 40 | `SignalEmitter` family | P1 | ◐ (`SignalOnHit`/`OnLeave` são o molde) |
| 41 | `Tag` (grupos) | P1 | ✖ (base: hash de `Name`) |
| 42 | `ScriptProperties` + Luau attach UI | P0 | ◐ (`ph2d-script` completo; ZERO UI) |
| 43 | `ScriptEventHooks` | P1 | ◐ (`MessageBus`) |
| 44 | `GameplayGraph` | P2 | ◐ infra (`ph2d-nodegraph`); lição: VisualScript do Godot morreu |
| 45 | `UserBehavior` (funil) | P2 | ✖ |

Iluminação 2D: **nenhum item deste domínio** (adiada por decisão do dono, 2026-08-20 — nada a listar aqui).
