# Dossiê — GameMaker (atual) & Defold: o catálogo do que o usuário ACRESCENTA a um objeto de cena

**Data da pesquisa:** 2026-08-20 · **Fontes:** docs oficiais (manual.gamemaker.io/monthly, defold.com/manuals) — URLs por categoria no fim de cada seção.
**Escopo:** tudo que a engine dá pronto via editor/UI e o código que isso elimina. Iluminação 2D aparece 1× (Defold `Light`) e está marcada **ADIADO** conforme decisão do dono (2026-08-20).
**Nota metodológica:** só listei o que confirmei na doc oficial nesta sessão. Onde a engine NÃO tem a categoria, está declarado explicitamente.

---

# PARTE 1 — GAMEMAKER (manual "monthly", 2026)

## 1.0 O modelo de composição do GameMaker

GameMaker **não tem** componentes plugáveis: a unidade é o **Object** (template) → **Instance** (cópia na room). O "catálogo de behaviors" do GM é feito de quatro mecanismos:

1. **Checkboxes e propriedades no Object Editor** (Visible, Solid, Persistent, Uses Physics, Managed, Collision Mask, Parent) — cada checkbox liga um subsistema inteiro.
2. **Built-in instance variables** — toda instância JÁ NASCE com um motor de movimento (speed/direction/gravity/friction), um player de flipbook (image_index/image_speed) e 12 timers (alarm[0..11]). O usuário não adiciona o componente; ele só **escreve na variável** e o runtime faz o resto.
3. **Events** — o usuário adiciona *eventos* (Create, Step, Collision com X, Alarm 3, Mouse Enter…) e põe lógica dentro (GML Code ou GML Visual). O evento é o "slot de behavior".
4. **Assets atribuíveis** — Path, Timeline, Sequence, Particle System, Tile Set: criados em editores dedicados e **atribuídos** à instância/room por 1 função ou drag&drop.

A lição estrutural: o GM elimina código **empurrando comportamento para dentro do runtime da instância** (todo objeto já é um "character controller cru") e para **editores de asset dedicados**.
Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Objects.htm

---

## 1.1 Visual / render

### Sprite (atribuição no Object) + built-in sprite variables
- **O que faz:** associa um sprite ao objeto; o Draw event default desenha o sprite animado com todas as transforms, sem 1 linha de código.
- **Propriedades (via built-ins, todas lidas/escritas):** `sprite_index`, `image_index` (frame), `image_speed` (velocidade da animação), `image_angle`, `image_xscale/yscale`, `image_alpha`, `image_blend` (tint), `image_number`, `mask_index`.
- **Código que elimina:** loop de flipbook (avanço de frame por dt), draw com rotação/escala/tint/alpha, flip por escala negativa.
- **Compõe com:** collision mask (o sprite define a máscara por default), Sequences (que sobrescrevem image_* quando animam a instância).
- **Relevância 2D: ALTA.** É o baseline: *renderizar + animar sem escrever draw*.
- Fonte: https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/Asset_Management/Instances/Instance_Variables/Instance_Variables.htm

### Nine Slice (no Sprite Editor)
- **O que faz:** 9-slicing com 4 guias visuais; cantos preservados, bordas/centro esticados ou repetidos ("Tile Mode" por fatia). Uma vez ativado, funciona **em qualquer lugar** onde o sprite é escalado (rooms, sequences, draw_sprite_ext) sem mudança de código.
- **Propriedades UI:** Activate Nine Slice, 4 guias arrastáveis, Tile Mode por fatia, preview redimensionável/rotacionável.
- **Elimina:** todo o código de desenhar painéis/molduras escaláveis em 9 partes.
- **Relevância: ALTA** (UI in-game e level building com paredes esticáveis).
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Sprite_Properties/Nine_Slices.htm

### Skeletal Animation Sprites (Spine)
- **O que faz:** sprites criados no Spine importam como asset de sprite; troca de animação nomeada, mixing/crossfade entre animações, eventos de frame — via funções `skeleton_animation_*` (set, mix, list, get_event_frames…).
- **Elimina:** runtime de esqueleto 2D inteiro (bones, slots, interpolação, blend entre clips).
- **Relevância: MÉDIA-ALTA** (depende de ferramenta externa paga; mas é a única resposta do GM a deform 2D).
- Fonte: https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/Asset_Management/Sprites/Skeletal_Animation/Skeletal_Animation.htm

### Asset Layers (elementos visuais soltos na room)
- **O que faz:** camada da room que recebe **sprites animados, texto, sequences e particle systems** como elementos posicionáveis — sem criar Object nenhum.
- **Propriedades UI por elemento:** nome, blend color, rotação, flip, escala X/Y, posição, image speed e frame inicial; para texto: string multi-linha, wrapping, justificação, espaçamento de caractere/linha, origem do frame; tudo editável no Inspector e mutável em runtime (`layer_text_*`, `layer_sprite_*`).
- **Elimina:** objetos "dummy" só para decorar; código de desenhar texto no mundo.
- **Relevância: ALTA** — decoração e texto de cenário viram dado, não código.
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Room_Properties/Layer_Properties.htm

### Background Layers (com auto-scroll = parallax)
- **O que faz:** camada de cor sólida ou sprite tileável (H/V), com stretch, offset X/Y e **Horizontal/Vertical Speed** — o fundo rola sozinho a N px/step.
- **Elimina:** o step-code de parallax/scroll de fundo (para o caso comum).
- **Relevância: ALTA** (categoria *parallax/scrolling*: o GM cobre o básico por camada; parallax proporcional à câmera ainda é código).
- Fonte: mesma página de Layer Properties acima.

### Filter/Effect Layers (FX)
- **O que faz:** camada que aplica um filtro/efeito visual (ex.: Desaturate, Outline) a **todas as camadas abaixo**, ou como "Single-Layer FX" a uma camada específica — escolhido num dropdown, com preview no editor.
- **Elimina:** shaders de pós-processamento para efeitos comuns e o pipeline de aplicá-los por camada.
- **Compõe com:** qualquer layer (dá para aplicar Outline só na camada de texto, p.ex.).
- **Relevância: ALTA** — é "pós-processamento como dado de cena".
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Rooms.htm (seção Filter/Effect)

### O que NÃO tem em visual/render
- **Mesh 2D deformável, trail/line renderer:** NÃO existem como recurso de editor (desenho custom é código no Draw event).

---

## 1.2 Tilemap — Tile Set Editor + Tile Layers

- **O que faz:** tile set a partir de um sprite-grade (tile size, offset, separação, **Output Border** para sangria anti-crack em zoom); tile layers pintáveis na room.
- **Ferramentas de pintura (UI):** Pencil, Eraser, Fill (com constraint de seleção), Line, Rectangle, Selection (+copy/cut/paste de blocos de tiles), Flip/Mirror/Rotate de brush ou seleção.
- **Os três editores matadores dentro do Tile Set Editor:**
  - **Brush Builder** — brushes permanentes de múltiplos tiles (carimbos compostos).
  - **Animated Tiles** — tiles animados usando outros tiles como frames; tocam na room automaticamente.
  - **Auto Tiles** — autotiling: pinta 1 tile e a engine escolhe a variante que **conecta com os vizinhos** automaticamente.
- **Bônus:** **Convert Image To Tile Map** — importa uma imagem pronta, **deduplica** células (inclusive rotacionadas), gera sprite + tile set + tile map layer automaticamente.
- **Elimina:** estrutura de grid, render de tilemap, lógica de autotile (a maior parte do código de level building), pipeline de importar mapas desenhados.
- **Compõe com:** collision (colisão com tiles é via funções/máscara), Room Editor.
- **Relevância 2D: ALTA.**
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Tile_Sets.htm

---

## 1.3 Partículas — Particle System Editor

- **O que faz:** asset de sistema de partículas com preview em tempo real; N emitters por sistema, 1 tipo de partícula por emitter.
- **Uso sem código (3 vias):** (1) arrastar para um Asset Layer na room — toca sozinho; (2) arrastar para uma Sequence como track; (3) runtime com `part_system_create(asset)` — ou o botão **"Copy GML to Clipboard"** que gera o código equivalente inteiro.
- **Propriedades do emitter (UI):** Enabled, Preview+cor, **Mode Stream/Burst**, Particle Count (negativo = probabilidade 1/N por step!), Delay min–max e Interval min–max (seg ou frames), **Distribution** (Linear/Gaussian/InvGaussian), **Shape** (rectangle/ellipse/diamond/line) sobre uma região arrastável no canvas.
- **Propriedades da partícula (UI):** textura (built-ins ou sprite custom + frame), **cor em 3 estágios ao longo da vida + alpha + Additive**, Life min–max, Scale X/Y, Size (min/max/**Increment por step**/**Wiggle** aleatório), Speed (min/max/increment/wiggle), Gravity (força+direção), Direction (min/max/increment/wiggle), Orientation (min/max/increment/wiggle/**Relative** à direção), **Subparticles: On Death e On Update** (partícula emite outro preset — explosões encadeadas sem código).
- **Biblioteca de presets:** built-ins + presets do projeto; emitters **linkados** a um preset atualizam juntos entre sistemas diferentes.
- **Elimina:** o sistema de partículas inteiro (pool, integração de velocidade, ramp de cor, spawn burst/stream, encadeamento).
- **Relevância 2D: ALTA.**
- Fontes: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Particle_Systems.htm · https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Particle_System_Properties/Emitter_Properties.htm

---

## 1.4 Câmera de gameplay — Views/Viewports com Object Following

- **O que faz:** por room, até múltiplas câmeras (view = retângulo no mundo; viewport = retângulo na tela; suporta **split-screen** nativo por sobreposição/ordem de ports).
- **Propriedades UI (por câmera):** Enable Viewports, Clear Viewport Background, posição/tamanho do view, posição/tamanho do viewport, e o pacote de follow:
  - **Object Following:** escolhe o objeto a seguir num dropdown;
  - **Horizontal/Vertical Border:** zona morta (buffer em px da borda do view — a câmera só anda quando o alvo entra na zona);
  - **Horizontal/Vertical Speed:** velocidade de perseguição em px/frame (−1 = instantâneo, 0 = parada).
- **Elimina:** o follow-camera de 90% dos jogos 2D: dead zone + clamp + velocidade de scroll, e o setup de split-screen.
- **NÃO tem na UI:** shake, zonas de câmera, transições, lookahead (isso é código sobre `camera_set_*`).
- **Relevância 2D: ALTA** — follow com dead-zone configurado em 4 campos numéricos é referência de "câmera sem código".
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Room_Properties/Room_Properties.htm

---

## 1.5 Física & colisão

### Colisão "tradicional" (sem física)
- **Collision Mask** (no Object/Sprite): máscara vinda do sprite ou de outro sprite dedicado; **Collision Event** contra um objeto-alvo (ou um **parent** cobrindo N filhos); flag **Solid** = a engine reverte a posição antes do evento (resolução ingênua embutida).
- **Elimina:** broadphase/teste de overlap, dispatch de "quem colidiu com quem".
- **Relevância: ALTA** (o caminho de colisão arcade sem solver).
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Object_Properties/Object_Events.htm

### Physics Objects (Box2D) — o checkbox "Uses Physics"
- **O que faz:** liga o objeto ao mundo físico da room (Room Physics: Enable Physics, vetor de **Gravity**, **Pixels To Meters**). Movimento "tradicional" e físico são mutuamente exclusivos.
- **Propriedades UI do fixture:** editor de shape encadeado (**circle / rectangle / polígono convexo de 3–32 pontos**), **Density** (0 = estático), **Restitution**, **Collision Group** (positivo = sempre colidem entre si; 0 = precisa de collision event; negativo = nunca colidem), **Linear Damping**, **Angular Damping**, **Friction**, e flags **Sensor** (colisão-notificação sem resposta física, dispara só na entrada), **Start Awake**, **Kinematic** (denso-zero móvel por variável, imune a forças — plataformas móveis).
- **Joints:** existem via código (funções de física / "The Physics Functions"), não via editor.
- **Elimina:** integração com o solver, criação de bodies/fixtures, classificação de camadas de colisão básica.
- **Relevância: ALTA** — é o exemplo canônico de "física como checkbox + form".
- Fontes: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Object_Properties/Physics_Objects.htm · Room Physics: página de Room Properties acima.

### O que NÃO tem
- **Materiais físicos como asset, shapecast dedicado, character controllers prontos (platformer/top-down/veículo): NÃO existem.** Raycast tradicional = `collision_line/collision_ray`-família (funções).

---

## 1.6 Movimento pronto (a "física arcade" embutida) + Paths + Motion Planning

### Built-in movement variables — o motor de movimento gratuito
- **O que faz:** TODA instância tem `speed` (px/step), `direction` (graus), `hspeed`/`vspeed`, `gravity` + `gravity_direction` (aceleração por step), `friction` (desaceleração por step), `x/y`, `xprevious/yprevious`, `xstart/ystart`. Escrever `speed = 2` já move; `gravity = 0.5` já cai. A engine integra tudo por step, mantendo speed/direction e hspeed/vspeed sincronizados.
- **Elimina:** o "update de movimento" de todo objeto arcade — a razão de o GM ser produtivo para iniciante: um jogo de nave é 4 atribuições de variável.
- **Relevância: ALTA** — é a decisão de design mais copiável: *o objeto de cena já nasce com cinemática*.
- Fonte: https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/Asset_Management/Instances/Instance_Variables/speed.htm

### Path (asset) + path-following embutido na instância
- **O que faz:** editor dedicado desenha o caminho (pontos com **speed % por ponto** — acelera/freia ao se aproximar de cada ponto; conexão **straight ou smooth**, precision 1–8, closed; Reverse/Flip/Mirror; edição também direto na room via Path Layers).
- **Na instância:** `path_start(path, speed, endaction, absolute)` + built-ins `path_position` (0–1), `path_speed`, `path_scale`, `path_orientation`, `path_endaction` — a instância anda sozinha pelo caminho; endaction define o que fazer no fim (parar/loopar/reverter…).
- **Elimina:** interpolação de waypoints, easing por trecho, patrulha de inimigos, movimento de plataformas — "movimento dinâmico bonito sem nenhum código" (texto da doc).
- **Compõe com:** mp_grid (o A* devolve um path), Rooms (path layers), alarms (repatrulhar).
- **Relevância: ALTA.**
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Paths.htm

### Motion Planning (mp_*) — pathfinding e desvio
- **O que faz (funções, não UI):** 3 famílias — `mp_linear_*` (avança e para em obstáculo), `mp_potential_*` (desvio local por campos potenciais, "AI simples" de step), `mp_grid_*` (**A*** sobre grid: marca células proibidas por instâncias/retângulos, computa caminho mais curto e **devolve um Path** que a instância segue com o sistema acima).
- **Elimina:** implementação de A*, steering básico de desvio.
- **NÃO tem:** navmesh, agentes com raio/prioridade, behavior trees.
- **Relevância: MÉDIA-ALTA.**
- Fonte: https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/Movement_And_Collisions/Motion_Planning/Motion_Planning.htm

---

## 1.7 Timers — Alarms + Time Sources

### Alarm Events (12 por instância)
- **O que faz:** `alarm[0..11]` são contadores decrescentes automáticos; ao chegar a 0 disparam o **Alarm Event** correspondente e vão a −1 (sentinela "não está rodando"). Auto-rearme dentro do próprio evento = loop periódico.
- **Elimina:** todo contador manual de frames ("espere 3s e faça X", spawners periódicos, i-frames, cooldowns).
- **Relevância: ALTA** — pelo custo/benefício, um dos maiores redutores de código do GM.
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Object_Properties/Object_Events.htm (seção Alarm)

### Time Sources (o timer moderno)
- **O que faz:** timers hierárquicos criados por código: período em **segundos ou frames**, callback + argumentos, N repetições ou infinito, pause/resume/reconfigure/reset, herdando do relógio "Global" ou "Game" (pausáveis em bloco); atalhos `call_later`/`call_cancel`.
- **Elimina:** agendadores caseiros, e substitui alarms quando precisa de mais de 12/pausabilidade/hierarquia.
- **Relevância: ALTA.**
- Fonte: https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/Time_Sources/Time_Sources.htm

---

## 1.8 Timeline/Sequencer

### Time Lines (LEGADO)
- **O que faz:** asset com "moments" (step N → executa código); atribuível a um objeto (`timeline_index`, `timeline_running`, `timeline_speed`, `timeline_position`, `timeline_loop`). A doc declara: **substituídas por Sequences**, mantidas por legado.
- **Relevância: BAIXA (legado)** — mas o mecanismo (script de tempo atribuível a uma instância) é historicamente o "behavior de cutscene barata".
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Timelines.htm

### Sequences — o sequencer/cutscene editor
- **O que faz:** asset de animação multi-track com canvas + Track Panel + **Dope Sheet**. Anima **sprites, objetos (instâncias reais!), sons, texto, particle systems e sequences aninhadas**.
- **Propriedades/recursos UI:** parameter tracks (posição, escala, rotação, cor…) com keyframes; **Automatically Record Changes** (mexeu no canvas com o playhead em outro frame → keyframes gravados sozinhos); **Animation Curves** (editor de curvas por track); esticar asset key além da animação → loop automático; clip region; playback speed; **loop/ping-pong**; guides/smart guides; máscaras de clipping; texto com formatação.
- **Integração com lógica:** **Moments** (frame N chama função nomeada), **Broadcast Messages** (mensagens no timeline que o jogo escuta), **eventos da sequence** (Create/Destroy/Clean Up/Step begin-normal-end/Async).
- **Instanciação:** arrastada numa room (asset layer) ou criada por código; quando anima instâncias, **sobrescreve** x/y/escala/ângulo delas a cada step.
- **Elimina:** cutscenes codadas, animações de UI/menu, intro de fase, composições animadas (o trabalho de um "timeline runtime" inteiro).
- **Relevância 2D: ALTA** — é o exemplo mais completo de "animação como asset que pode conter objetos VIVOS".
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Sequences.htm

---

## 1.9 Áudio

- **O que faz (nível API, sem componente de editor):** `audio_play_sound(_ext)` com prioridade/loop; propriedades em camadas (asset → emitter → instância): **Gain, Pitch, Offset, Listener Mask**; **loop points** (início/fim de loop mutáveis ao vivo); **Audio Groups** (carga/descarga e gain em bloco); **audio buses + Audio Effects** (reverb, echo, delay… encadeáveis no bus); **Audio Emitters/Listeners** para **áudio posicional** com modelos de falloff (`audio_falloff_set_model`); streams de .ogg; **Sync Groups** (sincronia por sample para música em camadas); evento assíncrono "Playback Ended".
- **Elimina:** mixer com buses/efeitos, atenuação posicional, gestão de vozes (128 default).
- **NÃO tem:** componente "audio source" colocável no editor de cena — tudo por código; o Sound Editor do asset define volume/formato base.
- **Relevância: MÉDIA-ALTA** (rico, mas 100% código).
- Fonte: https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/Asset_Management/Audio/Audio.htm

---

## 1.10 Input

- **O que faz:** input entra como **EVENTOS do objeto**: Keyboard/Key Press/Key Release (por tecla, + "Any Key"/"No Key"), Mouse (down/pressed/released por botão **sobre a máscara da instância**, Mouse Enter/Leave, Wheel, "No Mouse Input", e variantes **Global**), **Gesture events** (tap, drag, flick, pinch, rotate — locais ou globais, com `event_data` de posição/movimento).
- **Elimina:** polling e hit-test de mouse sobre objetos (o "botão clicável" é: objeto + sprite + evento Left Pressed).
- **NÃO tem:** **mapeamento de ações nomeadas/rebinding no editor** (nada tipo input actions). Gamepad é 100% função.
- **Relevância: ALTA no conceito "evento de mouse sobre a máscara"; a ausência de action map é uma lacuna notória.**
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Object_Properties/Object_Events.htm

---

## 1.11 UI in-game — UI Layers + Flex Panels (recurso recente)

- **O que faz:** camadas de UI **globais ao projeto** (o "UI Folder" é compartilhado entre todas as rooms — HUD persiste sem objeto persistente), desenhadas sobre o jogo, no **Display** ou **por Viewport** (HUD por jogador em split-screen).
- **Estrutura:** UI Layer = **Flex Panel** raiz; panels aninhados (layout **flexbox via Yoga**: width/height em % ou px, padding, margin, gap, flexDirection, justifyContent, alignItems, position absolute/relative, flexGrow/Shrink, aspectRatio). Dentro dos panels: **Objects (instâncias interativas!), Sprites, Sequences, Fonts/texto**.
- **Edição visual:** arrastar padding/margin com handles coloridos no canvas, preview com tamanhos de tela presets, outlines dos panels, element list com drag para reordenar draw order.
- **Runtime:** instâncias em UI layer recebem **mouse events em espaço de UI** automaticamente; `layer_get_flexpanel_node` dá acesso à árvore; layout recalcula ao vivo; Debug Overlay tem preview de layout JSON.
- **Elimina:** engine de layout de HUD (ancoragem, redimensionamento responsivo), separação mundo/tela, hit-test de UI.
- **Relevância: ALTA** — a resposta madura para "UI que reduz programação": flexbox + objetos vivos dentro.
- Fontes: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Room_Properties/UI_Layers.htm · https://manual.gamemaker.io/monthly/en/GameMaker_Language/GML_Reference/Flex_Panels/Flex_Panels.htm · https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Room_Properties/UI_Layers_At_Runtime.htm

---

## 1.12 Parent Objects — a "herança" que funciona como grupo

- **O que faz:** objeto pode ter parent; herda eventos (override por evento, com `event_inherited()` para estender); e — o uso matador — **parent como GRUPO**: um Collision Event contra o parent pega todos os filhos; `with(parent)`, `instance_number(parent)` etc. incluem filhos; hierarquias multi-nível.
- **Elimina:** N eventos de colisão duplicados; listas manuais de "todos os inimigos"; duplicação de behavior entre variantes visuais.
- **Relevância: ALTA como lição** — mesmo numa engine de componentes, "grupo semântico consultável" (tag/parent) é o que mata o código de bookkeeping.
- Fonte: https://manual.gamemaker.io/monthly/en/The_Asset_Editors/Object_Properties/Parent_Objects.htm

---

## 1.13 Rede / multiplayer — Rollback Multiplayer

- **O que faz:** netcode de **rollback** pronto (beta, alvo GX.games; Sync Test local em Windows/macOS): a flag **Managed** no Object marca o objeto como "estado rolável" (state save/rollback/prediction automáticos); `rollback_create_game`/`rollback_join_game` (até 4 players), servidores da própria YoYo — "no server set-up is required".
- **Elimina:** sincronização de estado, predição, reconciliação — o netcode inteiro do modelo GGPO.
- **Relevância: MÉDIA (beta/plataforma restrita), mas ALTA como referência de design:** *multiplayer = 1 checkbox por objeto + 2 chamadas*.
- Fontes: https://manual.gamemaker.io/beta/en/GameMaker_Language/GML_Reference/Rollback/Rollback_Multiplayer.htm · flag Managed: página do Object Editor.

---

## 1.14 Persistência / save

- **Persistent (Object):** instância atravessa troca de rooms sem ser recriada. **Persistent (Room):** a room preserva seu estado entre visitas. São os dois únicos "persistes" de UI.
- **Save de jogo: NÃO tem componente** — é código com `ini_*`/JSON/buffers.
- **Relevância dos flags: ALTA** (o "player que atravessa fases" e o "mundo que lembra" viram 2 checkboxes).
- Fontes: Object Editor + Room Properties (acima).

## 1.15 Como o usuário cria comportamento próprio

- **GML Code** (linguagem própria) ou **GML Visual** (visual scripting por blocos de ação, conversível para código por evento) — ambos escritos **dentro de eventos** do objeto. Não há "asset de behavior" reutilizável além de scripts/funções e do parenting.
- Fonte: https://manual.gamemaker.io/monthly/en/Drag_And_Drop/Drag_And_Drop_Index.htm

## 1.16 Categorias SEM cobertura no GameMaker (declaração explícita)
- Character controllers prontos (platformer/top-down/veículo): **NÃO** (o GM Asset "template"/tutoriais não contam como engine).
- Esqueleto/deform nativo (sem Spine): **NÃO**. Mesh 2D: **NÃO**. Trail/line: **NÃO**.
- Action-map de input: **NÃO**. Navegação navmesh/agentes/behavior tree: **NÃO** (só mp_grid A*).
- Tween/animation-player genérico de propriedades de instância: **NÃO** como asset (Sequences cobrem o caso autoral; tween por código é manual).
- Save/load de alto nível: **NÃO**. Áudio como componente de cena: **NÃO**. Camera shake/zonas/transição: **NÃO** na UI.

---
---

# PARTE 2 — DEFOLD

## 2.0 O modelo: Game Objects + Components + Collections + Mensagens

- **Game Object:** contêiner leve com posição/rotação/escala e id estável. **Não tem comportamento próprio** — tudo vem dos **components** que ele contém. Componentes são afetados pela transform do GO pai.
- **Component:** adicionado *in-place* no arquivo do GO ou **por referência** a um arquivo de recurso (script, GUI, particlefx e tilemap são obrigatoriamente arquivos separados). Todos podem ser ligados/desligados por mensagem `enable`/`disable` (inclusive `"."` = todos do GO).
- **Collection:** árvore de GOs e sub-collections; é o **prefab**: o arquivo é o protótipo, instâncias na collection referenciam o protótipo, editar o arquivo atualiza todas.
- **Message passing:** componentes se comunicam por `msg.post(url, message_id, {dados})` **assíncrono**, com endereçamento hierárquico (`/go#comp`, `outra_collection:/go#comp`); receptor implementa `on_message`. A doc justifica: **desacoplamento total** — produtor não conhece o consumidor; mensagens de sistema (enable/disable/collision_response/set_parent…) usam o mesmo canal. É a filosofia inteira da engine: **composição sobre herança + comunicação por evento**.
- **Catálogo oficial de components:** Camera, Collection factory, Collection proxy, Collision object, Factory, GUI, Label, **Light (⚠ ILUMINAÇÃO — ADIADO por decisão do dono; apenas registrado, sem detalhamento)**, Mesh, Model, Particle FX, Script, Sound, Sprite, Tilemap — mais **Spine model** e **Rive model** via extensões oficiais.
- Fontes: https://defold.com/manuals/components/ · https://defold.com/manuals/building-blocks/ · https://defold.com/manuals/message-passing/

## 2.1 Catálogo componente a componente

### Sprite
- **O que faz:** imagem/flipbook de um atlas ou tile source.
- **Propriedades UI:** Image (atlas), Default Animation, Material, **Blend Mode** (Alpha/Add/Multiply/Screen), **Size Mode** (Auto/Manual), **Slice-9** (bordas preservadas no resize).
- **Runtime:** `sprite.play_flipbook()` com callback de fim; `sprite.set_hflip/vflip`; propriedades animáveis/setáveis: `tint`, `cursor` (posição normalizada da animação), `playback_rate`, `scale`, `size`, `image`, `material`.
- **Elimina:** gestor de frames, atlas binding, tint/blend manual, 9-slice de sprites de mundo.
- **Compõe com:** atlas/tile source; go.animate (anima tint/cursor/scale); collision object (mesmo GO).
- **Relevância: ALTA.**
- Fonte: https://defold.com/manuals/sprite/

### Label
- **O que faz:** texto **no espaço do jogo** (não no GUI).
- **Propriedades UI:** Text, Font, Size (caixa; largura = ponto de wrap), Color/Outline/Shadow (RGBA), **Leading**, **Tracking**, **Pivot** (alinhamento), **Line Break**, Material (deve casar com o tipo de fonte: bitmap/distance field), Blend Mode.
- **Runtime:** `label.set_text`; color/outline/shadow/scale/size via go.set/go.animate.
- **Elimina:** render de texto no mundo (nomes sobre NPCs, dano flutuante) sem passar pelo sistema de GUI.
- **Relevância: ALTA** (a separação label-no-mundo vs texto-de-GUI é uma decisão de produto copiável).
- Fonte: https://defold.com/manuals/label/

### Tilemap
- **O que faz:** grade pintável de tiles de um **Tile Source**; múltiplas **layers** no mesmo componente.
- **Editor:** paleta (Space), pick da própria layer (Shift+click), seleção retangular, **rotação (Z) e flips (X/Y) do brush**, borracha.
- **Runtime:** `tilemap.get_tile`/`set_tile` (mapas destrutíveis/dinâmicos).
- **Colisão:** o tile source define **collision shapes por tile**; um Collision Object pode usar a geometria do tilemap direto — física de fase sem desenhar shapes.
- **Elimina:** grid, render em lote, e a parte chata: colisão de cenário derivada dos tiles.
- **Relevância: ALTA.**
- Fonte: https://defold.com/manuals/tilemap/

### Particle FX
- **O que faz:** sistema de partículas com preview no editor; **emitters** (shapes: circle, 2D cone, box, sphere, cone) + **modifiers**: **Acceleration, Drag, Radial (atrai/repele), Vortex** (rotação/espiral).
- **Propriedades UI:** spawn rate, tamanho do emitter, lifetime, speed, cor, rotação, stretch… — quase todas **keyable em CURVAS ao longo da duração** (Curve Editor visual).
- **Runtime:** `particlefx.play("#fx")` / `.stop()` / `set_constant` (tint).
- **Elimina:** partículas codadas; os modifiers eliminam até os efeitos "físicos" (vento, redemoinho, atração) que normalmente viram update custom.
- **Relevância: ALTA.**
- Fonte: https://defold.com/manuals/particlefx/

### Collision Object (física)
- **O que faz:** dá presença física ao GO. Engines: **Box2D (v3 ou legacy) para 2D**, Bullet para 3D — escolhível; unidades MKS, objetos ideais 0,1–10 m.
- **Propriedades UI:** **Type: Dynamic / Kinematic / Static / Trigger**; shapes **Box/Sphere/Capsule**, **convex hull** ou **geometria do tilemap**; Friction (0–1), Restitution, Mass, Linear/Angular Damping, **Locked Rotation**, **Bullet** (CCD), **Group** + **Mask** (matriz de colisão por nomes de grupo), e toggles **Generate Collision/Contact/Trigger Events**.
- **Comunicação:** o mundo físico manda **mensagens** ao GO: `collision_response`, `contact_point_response` (com ponto/normal), `trigger_response` — o script só implementa `on_message`. **Ray casts** por API. **Joints** (constraints) por API. Cada collection proxy cria **mundo físico separado**.
- **Elimina:** integração com Box2D, filtragem por camadas, sensores, dispatch de contatos.
- **Compõe com:** tilemap (shape), factory (spawn de corpos), proxy (mundos), fixed_update no script.
- **Relevância: ALTA.**
- Fontes: https://defold.com/manuals/physics/ · https://defold.com/manuals/physics-objects/

### Camera
- **O que faz:** define a projeção/viewport do mundo; múltiplas câmeras, a habilitada por último vale (ou `render.set_camera`).
- **Propriedades UI:** Aspect Ratio, FOV, Near/Far Z, Auto Aspect Ratio, **Orthographic Projection** + **Orthographic Zoom**, **Orthographic Mode: Fixed / Auto Fit / Auto Cover** (adaptação automática da área de design à janela).
- **Follow:** **não há componente de follow** — o idioma oficial é *parentear a câmera ao alvo* ou atualizar posição por script.
- **Utilidades que eliminam matemática:** `camera.screen_to_world`, `camera.screen_xy_to_world` (input → mundo com pan/zoom/projeção resolvidos).
- **Elimina:** matrizes view/projection, letterboxing/fit manual, conversões de coordenada.
- **Relevância: ALTA (ortho zoom + auto fit/cover), com a lacuna do follow.**
- Fonte: https://defold.com/manuals/camera/

### GUI (+ GUI Script)
- **O que faz:** UI resolução-independente desenhada sobre o jogo, imune à câmera. **Node types: Box** (textura/cor/flipbook, 9-slice), **Text**, **Pie** (circular/progresso radial, fill parcial, inner radius), **Template** (instancia outra cena GUI = prefab de UI), **ParticleFX** (partículas dentro da UI); Spine node via extensão.
- **Propriedades UI por node:** transform, size, color/alpha animáveis, blend, **Pivot** (9 opções), **X/Y Anchor** (fixa a distância proporcional às bordas quando a tela estica), **Adjust Mode: Fit / Zoom / Stretch**, Enabled vs **Visible** (invisível mas animável), Material por node, **Layers** para controlar batching/draw calls.
- **GUI script:** Lua próprio (`gui.*`), com `gui.animate` para qualquer propriedade.
- **Elimina:** engine de layout responsivo (âncora+pivot+adjust resolvem multi-resolução), progress bars radiais (Pie), composição de telas (Template), e a ordenação de draw da UI.
- **Relevância: ALTA.**
- Fonte: https://defold.com/manuals/gui/

### Sound
- **O que faz:** toca um som (WAV/Ogg Vorbis/Ogg Opus).
- **Propriedades UI:** Sound (arquivo), **Looping** (+ loop count), **Gain**, **Pan** (−1..1), **Speed**, **Group** (grupo de mixagem).
- **Runtime:** `sound.play`/`sound.stop`; gain por grupo (mixer básico por nomes de grupo, com "master"); a doc mostra o padrão de **gating** para evitar phasing de sons repetidos.
- **NÃO tem:** áudio posicional/2D espacial (só pan estéreo), efeitos/DSP.
- **Elimina:** mixer simples por grupos, controle de instâncias simultâneas.
- **Relevância: MÉDIA** (funcional e simples; sem spatialization).
- Fonte: https://defold.com/manuals/sound/

### Factory — **spawn como componente**
- **O que faz:** fabrica instâncias de um GO-protótipo em runtime: `factory.create(url, pos, rot, properties, scale)` → id.
- **Propriedades UI:** **Prototype** (arquivo do GO), **Load Dynamically** (recursos só carregam no primeiro create/load — com `factory.load(cb)` assíncrono), **Dynamic Prototype** (trocar o protótipo em runtime via `factory.set_prototype`).
- **Detalhe de design:** a doc manda **não** fazer pooling manual — "the engine handles object pooling internally"; teto por `Max Instances` do projeto.
- **Elimina:** instanciação manual, pré-carga de recursos de spawn, pooling.
- **Compõe com:** script properties (passa `properties` por instância no spawn!), collection factory.
- **Relevância: ALTA** — projéteis/inimigos/pickups = 1 componente + 1 linha.
- Fonte: https://defold.com/manuals/factory/

### Collection Factory — spawn de HIERARQUIAS
- **O que faz:** igual ao factory, mas instancia uma **collection inteira** (inimigo = corpo + arma + sensores, com parent-child preservado). `collectionfactory.create` devolve **tabela id-local → id-runtime**; aceita tabela de properties **por objeto spawnado** (`props[hash("/bean")] = {...}`).
- **Elimina:** montagem procedural de entidades compostas.
- **Relevância: ALTA** — "prefab composto spawnável" é o que diferencia de um spawner simples.
- Fonte: https://defold.com/manuals/collection-factory/

### Collection Proxy — mundos/níveis carregáveis + controle do TEMPO
- **O que faz:** carrega/descarrega **outra collection como um MUNDO separado** (memória e física próprias) via mensagens: `load`/`async_load` → `proxy_loaded` → `init` + `enable`; descarga com `unload`. Uso: troca de fase, telas, minigames.
- **O extra matador:** `set_time_step {factor, mode}` **por proxy** — slow-motion, pause (factor 0) e fast-forward do mundo carregado, afetando física, dt, animações e timers, **sem tocar no jogo que está fora**.
- **Elimina:** streaming/gestão de fases, tela de pause com o mundo congelado, bullet-time.
- **Relevância: ALTA.**
- Fonte: https://defold.com/manuals/collection-proxy/

### Script — o componente de comportamento
- **O que faz:** o "custom component": um arquivo Lua com ciclo de vida (`init`, `final`, `update`, `fixed_update`, `on_message`, `on_input`, `on_reload` — hot-reload!) adicionado ao GO como qualquer outro componente. Variantes: GUI script e Render script.
- **Script Properties (`go.property`):** valores declarados no script aparecem **no editor, por instância** (override em azul, botão reset), tipos: number, boolean, hash, url, vec3, vec4, quat e **RESOURCES (atlas, font, material, texture, tile_source)**; acessíveis por `self.x`, `go.get/set/animate`, e **injetáveis no spawn** via factory.
- **Elimina:** sistema de "inspector de parâmetros" para scripts — reuso do mesmo script com dados diferentes por instância sem herança nem duplicação.
- **Relevância: ALTA** — é o mecanismo nº 1 de "script parametrizável por UI".
- Fontes: https://defold.com/manuals/script-properties/ (script: /manuals/script/)

### Model / Mesh (3D dentro do 2D)
- **O que faz:** Model = malha glTF (.gltf/.glb) com Skeleton, Animations (blending, cursor animável, morph targets), Material/texturas — Defold é "3D no núcleo", render 2D é ortográfico por default (render script precisa de ajuste p/ 3D). Mesh = componente de malha custom.
- **Relevância p/ 2D: BAIXA-MÉDIA** (mistura 2.5D).
- Fonte: https://defold.com/manuals/model/

### Light — **ADIADO**
- Listado no catálogo oficial de components. Iluminação 2D foi adiada por decisão do dono (2026-08-20) — registrado sem detalhamento e sem prioridade.
- Fonte: https://defold.com/manuals/components/

### Spine Model (extensão oficial) — esqueleto/deform 2D
- **O que faz:** componente **SpineModel** (spine scene = JSON do Spine + atlas Defold); `spine.play_anim` (loop/once/pingpong, **blend duration**, callbacks com **timeline events**); **bones viram game objects internos** (`spine.get_go`) — dá para parentear uma arma na mão; **IK** controlável; **Spine nodes no GUI** também.
- **Propriedades UI:** spine scene, default animation, skin, blend mode, material, playback rate, offset.
- **Elimina:** runtime de esqueleto 2D, attachment de itens em ossos, transições entre clipes.
- **Relevância: ALTA** (via extensão de 1 clique; Rive idem para vetorial).
- Fonte: https://defold.com/extension-spine/

## 2.2 Sistemas transversais que eliminam código

### Property Animation — `go.animate` / `gui.animate` (o tween universal)
- **O que faz:** anima **qualquer propriedade numérica** (number/vec3/vec4/quat e **constantes de shader**) de GO, componente ou node GUI: `go.animate(url, prop, playback, to, easing, duration, delay, cb)`.
- **Recursos:** **40+ funções de easing** (linear, back, bounce, elastic, sine, expo, circ, quad→quint, in/out/inout/outin) + **curvas custom por vetor**; playbacks once/loop/pingpong; callback ao fim; cancelamento.
- **Elimina:** biblioteca de tween inteira — fade, knockback, pulsação, shake por animação de posição, barras que enchem.
- **Relevância: ALTA.**
- Fonte: https://defold.com/manuals/property-animation/

### Input Bindings — mapeamento de ações no editor
- **O que faz:** arquivo `game.input_binding` mapeia gatilhos físicos → **ações nomeadas** ("jump", "fire"): Key, Text, Mouse, Touch (multi), Gamepad (com arquivo `gamepads` por dispositivo) + acelerômetro. N gatilhos → mesma ação.
- **Runtime:** componente pede `acquire_input_focus` e entra na **input stack**; `on_input(self, action_id, action)` recebe pressed/released/repeated/posições; **retornar true consome** o input (modais/pause de graça).
- **Elimina:** polling de dispositivos, camada de rebinding, roteamento de input entre UI e jogo.
- **Relevância: ALTA.**
- Fonte: https://defold.com/manuals/input/

### Timer — `timer.delay(delay, repeat, cb)`
- Timers de engine com cancel/trigger/get_info; **morrem sozinhos com o script**; respeitam o time_step do proxy. Elimina contadores de frame manuais.
- Fonte: https://defold.com/ref/stable/timer/
- **Relevância: ALTA.**

### Persistência — `sys.save` / `sys.load` / `sys.get_save_file`
- Serializa **tabelas Lua inteiras** em arquivo no local certo do OS por plataforma. API-only (sem componente). Elimina serialização e path por plataforma.
- Fonte: https://defold.com/manuals/file-access/
- **Relevância: MÉDIA-ALTA.**

## 2.3 Categorias SEM cobertura no Defold (declaração explícita)

- **Character controllers prontos:** NÃO (kinematic + código; a doc de física ensina, não dá pronto).
- **Caminhos/splines (path/path-follow):** NÃO existe componente de path.
- **Timeline/sequencer autoral:** NÃO (só go.animate encadeado/curvas de particle).
- **Navegação & AI (agente/navmesh/behavior tree):** NÃO (extensões de comunidade).
- **Câmera-follow/limites/shake como componente:** NÃO (parenting/script).
- **Rede/multiplayer como componente:** NÃO (APIs http/socket + extensões).
- **Áudio posicional:** NÃO (só pan). **Trail/line:** NÃO. **Parallax:** NÃO como componente (render script/extensões).
- **Visual scripting:** NÃO — comportamento é sempre Lua (script component).

---
---

# PARTE 3 — Checklist de categorias × engine

| Categoria | GameMaker | Defold |
|---|---|---|
| Sprite/flipbook | ✔ built-ins da instância | ✔ Sprite component |
| Nine-patch | ✔ Nine Slice (sprite, universal) | ✔ Slice-9 (sprite + GUI box) |
| Tilemap | ✔✔ autotile, animated tiles, brushes, import de imagem | ✔ paint + colisão derivada do tile source |
| Mesh 2D / deform | ✖ (Spine import p/ esqueleto) | ✔ extensão Spine/Rive; Mesh 3D nativo |
| Trail/line | ✖ | ✖ |
| Partículas | ✔✔ editor + presets + subparticles | ✔✔ curvas + modifiers físicos |
| Câmera gameplay | ✔ follow com dead-zone e speed na UI; split-screen | ◐ projeção/zoom/fit na UI; follow = parenting |
| Física | ✔ Box2D via checkbox + form | ✔ Box2D/Bullet, 4 tipos, group/mask na UI |
| Sensores/areas | ✔ flag Sensor | ✔ tipo Trigger |
| Raycast/shapecast | ◐ funções | ◐ API (ray cast) |
| Joints | ◐ só código | ◐ só código |
| Character controller pronto | ✖ | ✖ |
| Paths/splines | ✔✔ asset + follow embutido + speed por ponto | ✖ |
| Pathfinding | ✔ mp_grid A* → path | ✖ |
| Animação (tween genérico) | ✖ (Sequences p/ autoral) | ✔✔ go.animate 40 easings |
| State machine/anim tree | ✖ | ✖ (mix/blend no Spine) |
| Timeline/sequencer | ✔✔ Sequences (dope sheet, curvas, eventos, broadcast) | ✖ |
| Áudio | ◐ API rica (buses, efeitos, posicional, sync groups) | ✔ componente simples (sem posicional) |
| Input action-map | ✖ (eventos por tecla/mouse/gesto) | ✔✔ input bindings + stack |
| UI in-game | ✔✔ UI Layers + Flex Panels (flexbox, global) | ✔✔ GUI (âncoras, pie, template) |
| Navegação & AI | ◐ mp_* | ✖ |
| Spawn/factory/pooling | ◐ instance_create (função) | ✔✔ Factory/Collection Factory + pooling interno |
| Streaming de fases/mundos | ◐ rooms + persistent | ✔✔ Collection Proxy (+ time_step/pause/slow-mo) |
| Timers | ✔✔ alarms na UI de eventos + Time Sources | ✔ timer.delay |
| Rede/multiplayer | ✔ Rollback (beta, Managed flag) | ✖ |
| Persistência | ◐ flags persistent; save = código | ◐ sys.save/load |
| Parallax/scroll | ✔ background layer speed | ✖ |
| Utilitários (drag&drop, pin, wrap, fade…) | ◐ gesture events; resto = código | ✖ (tudo script) |
| Custom component / visual scripting | eventos + GML Code / **GML Visual** (blocos) | **Script component** + go.property (sem visual) |

Legenda: ✔✔ forte em UI/editor · ✔ presente · ◐ só por código/parcial · ✖ ausente.

---

# PARTE 4 — MATADORES DE CÓDIGO (os que mais eliminam programação só com UI)

1. **Built-in movement variables (GM)** — `speed/direction/gravity/friction` na própria instância: o objeto nasce com cinemática; um jogo de nave são atribuições, não um update.
2. **Alarm events (GM)** — 12 timers por instância com evento próprio; cooldowns, spawners e "espere N e faça X" sem nenhum contador manual.
3. **Path + path-following embutido (GM)** — desenha o caminho, define speed % por ponto e endaction; patrulhas e plataformas móveis com zero código de interpolação.
4. **Camera Object Following (GM)** — follow com dead-zone (H/V Border) e velocidade (H/V Speed) em 4 campos; split-screen por viewports. A câmera de 90% dos 2D sem uma linha.
5. **Auto Tiles + Animated Tiles + Convert Image To Tile Map (GM)** — autotiling pintável, tiles animados e importação de mapa-imagem com dedup automática: o pipeline de level building inteiro.
6. **Sequences (GM)** — cutscenes/animações multi-track com dope sheet, curvas, auto-record, moments (chama função no frame N) e broadcast messages: elimina o runtime de cutscene e a maioria das animações de UI.
7. **Particle System Editor (GM) / ParticleFX com curvas e modifiers (Defold)** — efeitos completos por UI; subparticles (GM) e Radial/Vortex/Drag (Defold) matam até os efeitos que viravam update custom.
8. **"Uses Physics" + fixture form (GM) / Collision Object (Defold)** — física = checkbox + meia dúzia de campos; no Defold, trigger/kinematic/static/dynamic + group/mask na UI e colisão do cenário **derivada do tilemap**.
9. **Collision Event + Parent Objects (GM)** — dispatch de colisão por par de objetos e parent-como-grupo: um evento contra `par_enemy` cobre todos os inimigos; elimina bookkeeping de listas e N eventos duplicados.
10. **Factory / Collection Factory (Defold)** — spawn de objetos e de **hierarquias inteiras** com properties por instância, carga assíncrona e **pooling interno da engine** (a doc proíbe pooling manual).
11. **Collection Proxy (Defold)** — níveis/mundos carregáveis com física isolada e **set_time_step por mundo**: pause, slow-motion e bullet-time viram uma mensagem.
12. **Input Bindings (Defold)** — ações nomeadas mapeadas no editor + input stack com consumo: elimina polling, rebinding e o roteamento UI-vs-jogo.
13. **go.animate / gui.animate (Defold)** — tween de qualquer propriedade (inclusive constante de shader) com 40 easings, pingpong e callback: elimina a biblioteca de tween.
14. **Script Properties (Defold)** — `go.property` expõe parâmetros do script no Inspector por instância (com tipos resource!) e no spawn via factory: o mesmo script vira N comportamentos configurados por UI.
15. **UI Layers + Flex Panels (GM) / GUI nodes (Defold)** — HUD responsivo por flexbox global entre rooms (GM) ou por âncora/pivot/adjust (Defold), com objetos interativos dentro (GM) e Pie/Template (Defold): elimina a engine de layout e o hit-test de UI.
16. *(menção)* **Rollback Multiplayer (GM, beta)** — netcode rollback = flag "Managed" + 2 chamadas, sem servidor próprio: a referência de "multiplayer como propriedade do objeto".

---

# PARTE 5 — Leituras para a decisão de componentes do PH2D (curto)

- **As duas engines convergem em UM ponto:** o que mais elimina código não é o componente exótico — é (a) **estado que a engine integra sozinha** (movement vars, alarms, path-follow), (b) **asset editável com preview** (partículas, sequences, tiles) e (c) **parâmetro de script exposto na UI por instância** (go.property).
- **GameMaker** ganha em *editores de asset* (Sequences, partículas com subparticles, autotile, câmera-follow por form) — o usuário-artista trabalha em ferramentas, não em código.
- **Defold** ganha em *arquitetura de composição* (factory/collection factory/proxy como COMPONENTES, mensagens assíncronas, properties tipadas com resources) — o modelo Unity-like que o dono já escolheu para o PH2D tem no Defold o exemplo 2D-first mais limpo.
- **Buracos comuns às duas** (oportunidade de diferenciação do PH2D): character controllers prontos, camera rig completo (shake/zonas/transições/limites), trail/line, tween autoral em timeline (o PH2D JÁ tem timeline própria), áudio posicional 2D como componente (o PH2D já tem rack de 42 efeitos), navegação/AI, e save/load declarativo.

---

## Fontes principais (por engine)

**GameMaker** (manual.gamemaker.io/monthly/en/…): Objects.htm · Object_Properties/Object_Events.htm · Object_Properties/Parent_Objects.htm · Object_Properties/Physics_Objects.htm · GML_Reference/Asset_Management/Instances/Instance_Variables/Instance_Variables.htm (e speed.htm) · The_Asset_Editors/Paths.htm · Timelines.htm · Sequences.htm · Rooms.htm · Room_Properties/Room_Properties.htm · Room_Properties/Layer_Properties.htm · Room_Properties/UI_Layers.htm (+ UI_Layers_At_Runtime.htm) · Particle_Systems.htm (+ Particle_System_Properties/Emitter_Properties.htm) · Tile_Sets.htm · Sprite_Properties/Nine_Slices.htm · GML_Reference/Flex_Panels/Flex_Panels.htm · GML_Reference/Movement_And_Collisions/Motion_Planning/Motion_Planning.htm · GML_Reference/Time_Sources/Time_Sources.htm · GML_Reference/Asset_Management/Audio/Audio.htm · GML_Reference/Asset_Management/Sprites/Skeletal_Animation/Skeletal_Animation.htm · beta/en/GML_Reference/Rollback/Rollback_Multiplayer.htm · Drag_And_Drop/Drag_And_Drop_Index.htm

**Defold** (defold.com/…): manuals/components/ · manuals/building-blocks/ · manuals/message-passing/ · manuals/sprite/ · manuals/label/ · manuals/tilemap/ · manuals/particlefx/ · manuals/physics/ (+ physics-objects/) · manuals/camera/ · manuals/gui/ · manuals/sound/ · manuals/factory/ · manuals/collection-factory/ · manuals/collection-proxy/ · manuals/script-properties/ · manuals/property-animation/ · manuals/input/ · manuals/animation/ · manuals/model/ · manuals/file-access/ · ref/stable/timer/ · extension-spine/
