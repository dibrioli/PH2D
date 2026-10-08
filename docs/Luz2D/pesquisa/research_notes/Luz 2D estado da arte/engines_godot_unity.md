# Godot 4 e Unity URP 2D: como fazem luz 2D em tempo real e onde ficam lentos (baseline para o PH2D)

> Pesquisa de 2026-10-08. Fontes primárias: docs oficiais, issues e PRs do GitHub, e tópicos de fórum.
> Para cravar números que a documentação não dá, fiz **grep pontual** de constantes no `master` do Godot
> (sem ler o código linha a linha). Os links apontam para o ficheiro. Todo número leva versão e
> dispositivo quando a fonte os dá. Quando a fonte não os dá, a nota diz isso.

## Q1 — Godot 4: o algoritmo de PointLight2D / DirectionalLight2D / LightOccluder2D, shadow maps, SDF, normal/specular, batching, limites

### Takeaway
O Godot 4 faz a luz 2D como **forward de passada única por canvas item**. Cada item recebe uma lista de até
**15 luzes posicionais** (índices de 8 bits num array de 256 luzes por render) e o fragment shader percorre
essa lista. As sombras são **mapas de sombra 1D**: cada PointLight2D tem 4 quadrantes de 512 texels (atlas
de 2048 × 512 por omissão), filtrados por PCF de 5 ou 13 taps ao longo da linha. O SDF dos oclusores é gerado
à parte e só serve shaders de utilizador e colisão de partículas, nunca a iluminação de base. Não há GI, nem
sombra com penumbra física, nem luz vinda de sprites emissivos.

### Cited Findings

**Passada única e atlas único (4.0, nov/2020)**
- O Godot 4.0 trocou o multi-pass do 3.x por uma passada única: «The main one was performance due to every light being rendered in a separate draw pass. This is no longer a problem in 4.0, as all lights are drawn in a single pass.» Também: «all 2D shadows and light textures use a single atlas.» — [Juan Linietsky, 28/11/2020, *Godot's 2D engine gets several improvements for upcoming 4.0*](https://godotengine.org/article/godots-2d-engine-gets-several-improvements-upcoming-40/)
- No 3.x cada luz redesenhava os itens que tocava. Um tópico de fórum da era 3.x diz que «very large sprites ... will have to be redrawn entirely every time a light touches them». — [Godot Forum (arquivo, 3.x): *Performance issues using Light2D with Normal-Maps*](https://forum.godotengine.org/t/perfomance-issues-using-light2d-with-normal-maps/7096)

**Lista de luzes por item e limites** (grep no `master`, 2026-10-08)
- `MAX_LIGHTS_PER_ITEM = 16` e `MAX_LIGHTS_PER_RENDER = 256`, iguais no renderer RD (Forward+/Mobile) e no GLES3 (Compatibility). — [renderer_canvas_render_rd.h](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/renderer_canvas_render_rd.h) · [rasterizer_canvas_gles3.h](https://github.com/godotengine/godot/blob/master/drivers/gles3/rasterizer_canvas_gles3.h)
- O limite que vale é **15**. A contagem guarda-se em 4 bits (`INSTANCE_FLAGS_LIGHT_COUNT_SHIFT = 0 // 4 bits`), o shader lê `light_count = read_draw_data_flags & 15u; //max 15 lights`, e o laço da CPU pára em `light_count == MAX_LIGHTS_PER_ITEM - 1`. Os índices de luz vão empacotados em `uint32_t lights[4]`, a 8 bits cada. — [canvas.glsl](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/shaders/canvas.glsl) · [renderer_canvas_render_rd.cpp](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/renderer_canvas_render_rd.cpp)
- Uma luz entra na lista de um item se passar `item.light_mask & light.item_mask`, a faixa de z (`z_min`/`z_max`) e a interseção de retângulos (`global_rect_cache.intersects(light->rect_cache)`). As **primeiras 15 na ordem da lista ficam e as outras descartam-se sem aviso**. Há ainda um comentário TODO no código: «consider making lights a per-batch property and then baking light operations in the shader for better performance». — [renderer_canvas_render_rd.cpp](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/renderer_canvas_render_rd.cpp)
- Do lado dos utilizadores (4.2.1 e 4.3): «Godot's cap of 15 for lights. Once a sprite renders lights to this cap it ignores any additional lights», e «doesn't seem to be a way to select which lights are rendered». Outro utilizador fala em «16». — [Godot Forum: *Working around Godot's 2D light limitations for a dark game*](https://forum.godotengine.org/t/working-around-godots-2d-light-limitations-for-a-dark-game/81311)
- Um roguelite relata artefactos com mais de 16 luzes sobrepostas numa área, mesmo com a maioria ocluída. — [godot-proposals discussion #9336](https://github.com/godotengine/godot-proposals/discussions/9336)
- A página 2D oficial não declara limite nenhum. Só avisa o custo do PCF13 («should only be used for a few lights at once»). — [Docs: 2D lights and shadows (stable = 4.7)](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)
- No Compatibility, os `rendering/limits/opengl/max_lights_per_object = 8` e `max_renderable_lights = 32` valem **só para 3D**: a descrição fala em «omnilights and spotlights … per surface». — [ProjectSettings.xml (master)](https://github.com/godotengine/godot/blob/master/doc/classes/ProjectSettings.xml)

**Shadow maps 1D polares**
- Por omissão `rendering/2d/shadow_atlas/size = 2048`: «Higher values result in more precise Light2D shadows, at the cost of performance and video memory usage». — [ProjectSettings.xml](https://github.com/godotengine/godot/blob/master/doc/classes/ProjectSettings.xml)
- O atlas mede `shadow_texture_size` × `MAX_LIGHTS_PER_RENDER*2` (2048 × 512). Cada luz com sombra ocupa **2 linhas**, `Rect2i(0, p_shadow_index*2, size, 2)`, e a PointLight2D divide-as em **4 sub-retângulos de `size/4` = 512 texels**, um por quadrante de 90°. A DirectionalLight2D usa a linha inteira com uma projeção ortográfica. — [renderer_canvas_render_rd.cpp](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/renderer_canvas_render_rd.cpp)
- Os filtros são None, **PCF5** (5 taps, ±2 texels) e **PCF13** (13 taps, ±6 texels), sempre **ao longo da linha 1D**. O passo `shadow_pixel_size = (1/shadow_texture_size)*(1+shadow_smooth)` cresce com o «Filter Smooth», e a doc avisa: «High values can cause banding, especially with PCF5». — [canvas.glsl](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/shaders/canvas.glsl) · [Docs 2D lights](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)
- Antes da 4.4 (PR #100302): cada luz na tela gerava 4 draw lists, com **`4 * lights_on_screen * occluders_on_screen` draw calls** e mais 4 chamadas de API por draw. O PR passou a cortar os oclusores por luz, criar a projeção na GPU, usar culling de viewport em vez de um render pass por direção, guardar as transformações dos oclusores num SSBO partilhado e fazer o culling no fragment shader. Ficou por fazer o vertex pulling, que daria 4 draws por luz, «added complexity … risk». — [PR #100302 (clayjohn, milestone 4.4)](https://github.com/godotengine/godot/pull/100302)
- A DirectionalLight2D tem sombras sempre infinitas. Quem quer sombra finita precisa de um shader próprio que leia o SDF. «Max Distance» ignora o zoom da Camera2D. — [Docs 2D lights](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)

**SDF dos oclusores**
- Os LightOccluder2D com «SDF Collision» (ligado por omissão) entram num SDF de tempo real que shaders de utilizador e as GPUParticles2D usam. A doc diz que ele «has no performance cost» quando nenhum shader o lê. — [Docs 2D lights](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html) · [ProjectSettings.xml](https://github.com/godotengine/godot/blob/master/doc/classes/ProjectSettings.xml)
- `rendering/2d/sdf/scale` fica a 50 % por omissão: «renders at half the resolution of the viewport size on each axis … 25% of the viewport's pixel count». `rendering/2d/sdf/oversize` fica a 120 %, para cobrir oclusores fora do viewport. — [ProjectSettings.xml](https://github.com/godotengine/godot/blob/master/doc/classes/ProjectSettings.xml)
- Na 4.0 o SDF foi vendido para circle tracing, sombras longas, névoa, distorção de calor, «2D global illumination» feita por shader de utilizador, chuva e colisão de partículas. Não é a base da iluminação embutida. — [Linietsky 2020](https://godotengine.org/article/godots-2d-engine-gets-several-improvements-upcoming-40/)
- O RD tem modos `SHADOW_RENDER_MODE_DIRECTIONAL_SHADOW / POSITIONAL_SHADOW / SDF`: o SDF é rasterizado a partir dos mesmos polígonos de oclusão. — [renderer_canvas_render_rd.h](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/renderer_canvas_render_rd.h)

**Normal e specular**
- O `CanvasTexture` junta Diffuse, Normal Map e Specular (cor e Shininess). PointLight2D e DirectionalLight2D aceitam ambos. «Height» só tem efeito com normal map. A iluminação é calculada à **resolução do viewport**, não do texel, e o pixel-art pede um shader que faça snap de `LIGHT_VERTEX`/`SHADOW_VERTEX`. — [Docs 2D lights](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)

**Batching**
- Na 4.4 o batching de canvas chegou ao Forward+ e ao Mobile; antes só existia no Compatibility. — [Godot 4.4 release](https://godotengine.org/releases/4.4/) · [PR #92797 (Stuart Carnie)](https://github.com/godotengine/godot/pull/92797)
- `draw_texture/rect/line` entram em batch. `draw_arc/circle/polygon` geram um draw cada. — [PR #92797](https://github.com/godotengine/godot/pull/92797)
- O batch quebra quando muda `use_lighting` entre itens (com luz / sem luz). — [renderer_canvas_render_rd.cpp](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/renderer_canvas_render_rd.cpp)
- `rendering/2d/batching/item_buffer_size = 16384` («Maximum number of canvas item commands that can be batched into a single draw call») e `uniform_set_cache_size = 4096`. — [ProjectSettings.xml](https://github.com/godotengine/godot/blob/master/doc/classes/ProjectSettings.xml)
- O PR #100501 de clayjohn transformava o nº de luzes numa specialization constant para desenrolar o laço: «the lighting code is heavily GPU ALU bound». Ficou estacionado: «This breaks batching when light count changes», «CPU cost from reduced batching», «Pipeline stutters from lack of ubershader», «The performance improvement wasn't as big as I expected». — [PR #100501](https://github.com/godotengine/godot/pull/100501)

**Alternativa barata recomendada pela própria doc**
- Sprites aditivos (CanvasItemMaterial em modo Add) «renders faster», mas misturam mal em zonas totalmente escuras, não fazem sombra e ignoram normal/specular. — [Docs 2D lights](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)

### Inferences
- O custo de GPU do Godot cresce em **pixels cobertos × luzes na lista do item × taps de PCF**. Cada luz paga a iluminação e, com sombra, 1 (None), 5 ou 13 leituras de textura por pixel em cada item que toca. Um sprite grande num cenário com muitas luzes paga todas as luzes no ecrã inteiro, porque o teste é de retângulo e não de pixel.
- O custo de CPU das sombras era **O(luzes × oclusores)** em draw calls até a 4.4 o reduzir. Depois dela, continua a haver ≥ 1 passada de sombra por luz, com 4 quadrantes.
- O teto de 15 luzes por item e de 256 por render é um **limite de formato** (4 bits e índices de 8 bits), não de hardware. O descarte arbitrário na ordem da lista viola o §0 do PH2D (o teto vem do dispositivo).
- A resolução angular é de 512 texels por 90° (≈ 0,18° por texel) com o atlas por omissão. O PCF ao longo da linha dá uma penumbra de **largura angular constante**, que não depende do tamanho da luz nem da distância ao oclusor. Não é penumbra física.

### Gaps
- Não achei texto oficial que descreva o polar 1D como «polar» nem a projeção exata por quadrante. A descrição acima vem do grep de constantes e retângulos.
- Não achei medição separada do custo do SDF quando um shader o lê.
- Não confirmei se o Mobile (RD) e o Compatibility diferem no algoritmo das sombras. As constantes são as mesmas nos dois headers.

## Q2 — Godot: problemas de performance MEDIDOS (sobretudo mobile), regressões, o que os devs disseram

### Takeaway
Os números públicos mostram que uma única PointLight2D já derrubava telemóveis Adreno de gama média/baixa no
4.0 e no 4.1: de 60 para 42 FPS num Adreno 612, e para 9 FPS num Snapdragon 860/665/662. O PR #100302, da 4.4,
levou um Pixel 4 de 17 FPS para 60 FPS (vsync) numa cena de pior caso. As explicações dos mantenedores apontam
duas causas: o overhead de CPU/driver das sombras (draw calls × oclusores) e o ALU do laço de luzes.

### Cited Findings
- **#81152** (4.1.1.stable, Vulkan Mobile): **Redmi Note 7 Pro, Snapdragon 675 / Adreno 612, Android 13, 2340×1080**. Uma única PointLight2D sobre um ColorRect de ecrã inteiro e um Sprite2D leva a **60 → 42 FPS** enquanto a luz está ativa. Ficou com milestone 4.3 e fechado, sem fix declarado na página. — [Issue #81152](https://github.com/godotengine/godot/issues/81152)
- **#73805** (4.0 rc3, «GL/Mobile»): **SD 860, SD 665, SD 662** (POCO X3 Pro). «When any kind of 2D light is added to the scene, the frame rate drops to 9fps.» Milestone 4.4, ligado ao PR #100302. — [Issue #73805](https://github.com/godotengine/godot/issues/73805)
- **PR #100302** (4.4), cena de pior caso com muitas luzes e oclusores:

  | Dispositivo | Build | Antes | Depois |
  |---|---|---|---|
  | Windows, RX 3600 | release | 330 FPS (master); dev3 ~380 FPS | 430 FPS |
  | M2 MacBook Pro | debug | 160 FPS (dev3) | 400 FPS («tiling architecture») |
  | **Pixel 4** | — | **17 FPS** | **60 FPS (vsync)** |

  Antes do PR, o thread guard era verificado `5*4*lights*occluders` vezes, «over 100,000» na #99420. — [PR #100302](https://github.com/godotengine/godot/pull/100302)
- **#99420**, regressão 4.4 dev3 → dev4 vinda do PR #98652 (Forward+, jogo 2D, RTX 4090 / i7-13700K): editor **383 → 200 FPS**, exportado **402 → 240 FPS**. Fechado com milestone 4.4 e atribuído a clayjohn. — [Issue #99420](https://github.com/godotengine/godot/issues/99420)
- Um relato de regressão no Forward+ diz que, com luzes, o Compatibility fica com FPS «much lower than even dev 4 when using Forward+». Mudar de renderer não é cura universal. *(Visto só no resumo da pesquisa, sem abrir a página. Não verificado.)*
- O glow no renderer Mobile leva um Android de ~120 para 37 FPS, e o Compatibility não sofre. É pós-processamento, não luz 2D, mas mostra o peso da escolha de renderer no telemóvel. — [Issue #98531](https://github.com/godotengine/godot/issues/98531)
- Era 3.x (multi-pass): duas luzes com normal map derrubam o FPS e o sprite grande é redesenhado por cada luz. — [Godot Forum (arquivo)](https://forum.godotengine.org/t/perfomance-issues-using-light2d-with-normal-maps/7096) · [Godot Forum: *Light2D slowing down the game in low end devices*](https://forum.godotengine.org/t/light2d-slowing-down-the-game-in-low-end-devices/16926)
- clayjohn, no #100501, diz que o laço de luz é «heavily GPU ALU bound» e que desenrolá-lo deu um ganho «wasn't as big as I expected». — [PR #100501](https://github.com/godotengine/godot/pull/100501)

### Inferences
- Há dois gargalos com mecanismos diferentes, e o PH2D deve medir os dois separados. (a) **CPU/driver**: as passadas de sombra cresciam com luzes × oclusores, e é isto que o salto do Pixel 4 (17 → 60) mostra. (b) **GPU/ALU por pixel**: um quad de ecrã inteiro com 1 luz custa ~6 ms num Adreno 612 (16,7 → 23,8 ms/quadro, calculado a partir de 60 → 42 FPS), e os tiles móveis pagam pelo overdraw.
- Não achei número pós-4.4 para Adreno/Mali de gama baixa. O caso do #81152 pode continuar aberto no seu mecanismo de fundo (ALU por pixel), mesmo com a issue fechada.

### Gaps
- Não achei medição publicada em iOS nem em Mali para luz 2D do Godot 4.
- Não achei números da 4.5/4.6/4.7 especificamente sobre luz 2D, nem notas de release dessas versões com mudanças nesta área.
- Não achei explicação escrita de clayjohn ou reduz para os casos Adreno (#73805, #81152). As páginas em cache não mostraram os comentários.

## Q3 — Unity URP 2D Renderer: tipos de Light2D, Shadow Caster 2D, light textures, passes, batching, custos mobile, Unity 6

### Takeaway
O URP 2D trabalha por **light render textures em espaço de ecrã**. A ordem é: (1) desenhar as sombras em
shadow textures, (2) desenhar cada luz em light textures, uma por blend style e por batch de sorting layers,
a **0,5× a resolução do ecrã por omissão**, (3) desenhar os sprites a amostrar essas texturas. O custo cresce
com **blend styles × batches de sorting layers × luzes**, mais um pré-passe de normal map em ecrã inteiro por
batch. Os números de mobile publicados são anedóticos e antigos (2020–2022).

### Cited Findings
- **Tipos**: Spot (cone, que substitui a antiga Point), Sprite (a luz tem a forma de um sprite), Freeform (polígono editável) e Global («no attenuation»; «You can only use one global light per blend style and sorting layer»). — [Unity 6 Manual: Introduction to the 2D lighting system in URP](https://docs.unity3d.com/6000.0/Documentation/Manual/urp/Lights-2D-intro.html)
- **Pipeline**: «Draws the shapes of 2D shadows into one or more shadow textures» → «Draws the color and shape of each 2D light into one or more light textures» → «Renders the lit GameObjects onto the screen, using the information from the textures». «By default, Unity tries to render all the light and shadow textures before it renders any GameObjects into the scene.» — [Unity 6 Manual](https://docs.unity3d.com/6000.0/Documentation/Manual/urp/Lights-2D-intro.html)
- **Não é físico**: a página diz que a luz 2D não é physically based, ao contrário da 3D. — [Unity 6 Manual](https://docs.unity3d.com/6000.0/Documentation/Manual/urp/Lights-2D-intro.html)
- **Light Render Texture**: «screen space Light Render Texture to capture light contribution», a **0,5× do ecrã por omissão**. «Half screen size resolution provides a good performance with almost no noticeable artifact in most situations». Textura menor dá «better performance at the cost of visual artifact». — [Unity 6.1 Manual: Optimize 2D lights](https://docs.unity3d.com/6000.1/Documentation/Manual/urp/2d-lights-optimize-methods.html)
- **Blend styles**: cada um acrescenta uma Render Texture. «The easiest way to increase rendering performance is to reduce the number of blend styles used.» — [Unity 6.1 Manual: Optimize 2D lights](https://docs.unity3d.com/6000.1/Documentation/Manual/urp/2d-lights-optimize-methods.html)
- **Normal maps**: «Using normal maps to simulate depth is currently a very expensive operation». Ligar normal map cria uma Render Texture de **tamanho cheio** num pré-passe de profundidade, **uma vez por batch de Layers**. — [Unity 6.1 Manual](https://docs.unity3d.com/6000.1/Documentation/Manual/urp/2d-lights-optimize-methods.html)
- **Batching por sorting layer**: camadas consecutivas partilham light textures só se tiverem exatamente o mesmo conjunto de luzes (e de shadow casters). Se a sombra de um caster atinge só uma camada, o batch parte-se. Existe o *Light Batching Debugger* para isto. «Layers that are batched together share the same set of Light Render Textures», e fazê-las batcháveis é «crucial». — [Unity 6.1 Manual: Introduction to 2D light batching](https://docs.unity3d.com/6000.1/Documentation/Manual/urp/2d-light-batching-intro.html) · [Optimize 2D lights](https://docs.unity3d.com/6000.1/Documentation/Manual/urp/2d-lights-optimize-methods.html)
- **Max Light Render Textures**: pré-renderizar todos os sets nem sempre é possível. «In a very complex setup with many distinctly lit Layers, it may not be practical to pre-render all Light Render Textures», e subir o limite gasta mais memória. — [Optimize 2D lights](https://docs.unity3d.com/6000.1/Documentation/Manual/urp/2d-lights-optimize-methods.html)
- **Sombras**: as propriedades de sombra da luz são Strength, **Softness** («softness of the edge of the shadow») e **Falloff Strength**. O Shadow Caster 2D define a forma e o Composite Shadow Caster 2D junta várias. — [Unity 6.7 docs: 2D light properties](https://docs.unity.com/en-us/engine/6000.7/manual/unity2d/2d-urp/2d-index/2dlight-properties)
- **Profiler (Unity 6.6)**: o módulo «2D Graphics» conta por quadro as normal textures, light textures, shadow textures, shadow caster draw calls, light batches e as luzes desenhadas individualmente por não caberem num batch. — [Unity 6.6 Manual: 2D Graphics profiler module](https://docs.unity3d.com/Manual/urp/2d-graphics-profiler-module-reference.html)
- O PR «Batching 2D Lights» (venkify) existe no repo Graphics. Só vi o título, não o conteúdo. — [Unity-Technologies/Graphics PR #2569](https://github.com/Unity-Technologies/Graphics/pull/2569)
- **Números de mobile, todos anedóticos**:
  - Galaxy S6, OpenGL ES3, Unity 2021 alpha, 1 global e 2 point lights, 10 sorting layers, alguns shadow casters: **Built-in 60 fps contra URP 2D 20 fps**. O próprio autor atribuiu a causa a «HDR and the global light in the scene», e depois de corrigir conseguiu «stable 60fps». Não há resposta de staff. — [Unity Discussions #820962 (dez/2020)](https://discussions.unity.com/t/performance-problems-with-urp-2d-lighting-shadow-on-mobile-devices/820962)
  - 150 luzes 2D (Unity 2020.3.21f1, desktop): ligar ou desligar o normal map não fez «significant difference». Outro utilizador avisa que 150 luzes podem ser «too much on mobile». Também lá: «Avoid using volume lights, these are quite slow» (mar/2020), e dicas de poucos blend styles, render texture scale menor e poucas sorting layers. Ninguém mediu FPS. — [Unity Discussions: How to optimise 2D lights?](https://discussions.unity.com/threads/how-to-optimise-2d-lights.852601/)
  - Um indie mediu ~6 fps num telemóvel com Light2D e 40–50 fps com uma luz própria feita de SpriteRenderer. Tinha sombras próprias, por isso não é benchmark limpo. *(Visto só no resumo da pesquisa, sem abrir.)* — [itch.io devlog gamba04: *2D Mobile Optimized Lighting System*](https://gamba04.itch.io/my-personal-devlog/devlog/145714/2d-mobile-optimized-lighting-system)
  - A frase atribuída a staff da Unity, «each unique sorting layer adds significant overhead … more than 25 layers is a big no go», apareceu no resumo da pesquisa associada a [este tópico](https://forum.unity.com/threads/unity-universal-render-pipeline-causing-massive-slow-down-on-2d-mobile-project.826647/). **Não a verifiquei**: o fetch caiu num tópico errado.
- **Unity 6 / Render Graph**: o URP com Render Graph «automatically merges and creates native render passes to reduce memory bandwidth … especially on tile-based (mobile) GPUs». A fonte fala do URP em geral, não do caminho de luz 2D. — [Unity 6 Preview blog](https://unity.com/en/blog/engine-platform/unity-6-preview-release)

### Inferences
- O custo do URP 2D tem forma diferente do do Godot. É **deferred-ish em espaço de ecrã**: cada luz é rasterizada na sua light texture a 0,5× e acumulada por blending. Isso é barato por luz em ALU, mas paga largura de banda por light texture × blend style × batch de layers, e cada batch que se parte multiplica passes e resolves (caros num TBDR móvel). O sprite paga pouco: amostra 1 textura por blend style. Não há cap de luzes por item, o custo cresce com a área das luzes.
- O 0,5× por omissão é a admissão da própria Unity de que o fill-rate móvel é o recurso escasso. Em pixel-art isso dá iluminação «borrada» e serrilhado nas sombras: nos tópicos de 2020 a cura sugerida para sombras serrilhadas foi subir para 1×.
- O normal map exige um pré-passe de ecrã inteiro por batch, o que multiplica o custo de cena por batches. Num TBDR isso significa mais um alvo de render a ir e voltar da memória.

### Gaps
- Não achei descrição oficial do algoritmo do Shadow Caster 2D (extrusão de malha, shadow volume, stencil?) nem de como Softness e Falloff funcionam por dentro.
- Não achei medição oficial da Unity, em ms por dispositivo, para luz 2D em mobile, nem nenhum número recente (Unity 6.x) com dispositivo nomeado.
- Não achei notas de release do Unity 6.x com mudanças específicas no 2D Renderer de luz (apenas a menção genérica de que a 6.5 trouxe atualizações nos sistemas 2D, numa fonte secundária).

## Q4 — O que nenhum dos dois faz: GI/bounce, sprite emissivo que ilumina vizinhos, penumbra física

### Takeaway
Nenhum dos dois tem GI 2D, bounce, nem sprites emissivos que iluminem a vizinhança. A penumbra de ambos é
um blur de filtro, não física. O Godot entrega um SDF para quem quiser escrever a sua GI num shader; o Unity
declara a luz 2D não-física.

### Cited Findings
- A página de luz 2D do Godot não fala de GI nem de emissivo. O substituto que ela oferece, sprites aditivos, não faz sombra nem respeita normal maps. — [Docs 2D lights](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)
- A «2D global illumination» do Godot 4.0 está listada entre os usos que o *utilizador* pode fazer do SDF em shader. Não é uma feature do motor. — [Linietsky 2020](https://godotengine.org/article/godots-2d-engine-gets-several-improvements-upcoming-40/)
- A GI 2D em Godot existe como projeto de terceiros, explicitamente «not a plug-and-play package … best used as a reference». — [sambigos: 2D Global Illumination in Godot](https://sambigos.itch.io/2d-global-illumination-in-godot)
- A GI do Godot (SDFGI) é **só 3D e só Forward+**, e mesmo lá «does not handle dynamic occluders or dynamic emissive surfaces». — [Docs: SDFGI](https://docs.godotengine.org/en/stable/tutorials/3d/global_illumination/using_sdfgi.html)
- A penumbra do Godot é PCF5/PCF13 na linha 1D, com largura dada por «Filter Smooth», não pelo tamanho da luz. — [canvas.glsl](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/shaders/canvas.glsl) · [Docs 2D lights](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)
- O Unity declara a luz 2D não physically based, e a sombra é controlada por «Softness»/«Falloff Strength», que são parâmetros artísticos. — [Unity 6 Manual](https://docs.unity3d.com/6000.0/Documentation/Manual/urp/Lights-2D-intro.html) · [Unity 6.7 2D light properties](https://docs.unity.com/en-us/engine/6000.7/manual/unity2d/2d-urp/2d-index/2dlight-properties)
- Um estudo académico sobre Radiance Cascades nota que a GI é «simpler and easier to do in 2D». É uma proposta universitária, não do Godot. — [Heriot-Watt proposal](https://projects.hw.ac.uk/Proposals/Details/KQY8I33OVEYQ)

### Inferences
- O espaço livre que o PH2D pode ocupar é: GI 2D em tempo real (radiance cascades ou campo de distância), emissivos como fontes de luz, penumbra que depende do tamanho da fonte e da distância ao oclusor, e nenhum cap de formato por item. Os dois incumbentes deixam tudo isto a shaders de utilizador ou a assets de terceiros.

### Gaps
- Não achei afirmação oficial do Unity do tipo «2D lights do not support GI/emissive». A ausência foi inferida das páginas de tipos e das propriedades.
- Não achei proposta oficial em godot-proposals para GI 2D no motor. A pesquisa não a encontrou, o que não prova que não exista.
