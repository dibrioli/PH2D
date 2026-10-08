# Radiance Cascades redefinem a luz 2D em tempo real

> Relatório de pesquisa, 2026-10-08, linha `line/Luz2D`. Ele resume sete notas de pesquisa em
> `docs/Luz2D/pesquisa/research_notes/Luz 2D estado da arte/`. As contas marcadas **[derivado]** são
> aritmética deste relatório sobre números citados, **não medições**. As marcadas **[inferência]** são
> juízo de engenharia.

**Em linguagem simples (para o dono).** Hoje, no PH2D, os sprites que brilham não iluminam nada à volta.
A melhor técnica de luz 2D que existe surgiu entre 2023 e 2025. Com ela, qualquer coisa que brilha vira
uma lâmpada, as sombras ficam suaves do jeito certo e a luz rebate nas paredes. O custo é o mesmo com uma
ou com mil luzes. Nem o Godot nem o Unity têm isso. Ninguém publicou ainda quanto ela custa num celular,
então o PH2D vai ser o primeiro a medir. A recomendação é fazer dela o padrão do PH2D, com um modo mais leve
que dá a mesma imagem nos aparelhos fracos e um caminho à parte para luzes muito pequenas e nítidas. Antes de
prometer qualquer número, vamos medir em iPhone, iPad e Android, contra o Godot rodando a mesma cena.

**BLUF técnico.** No fim de 2026, a família **Radiance Cascades (RC)** é o estado da arte em luz 2D em tempo
real. Ela tem duas formas:

- **RC clássico de Sannikov**, com o *bilinear fix* e a *pré-média*. É o caminho barato.
- **Holographic RC (HRC, maio de 2025)**, que é a referência de correção.

É a única família que entrega ao mesmo tempo quatro coisas: penumbra que depende do tamanho da fonte, GI com
rebote, sprites emissivos que iluminam os vizinhos, e custo **independente do número de luzes e de
oclusores**. O HRC mede **1,85 ms a 512² e 7,67 ms a 1024² numa RTX 3080 Laptop**, com RMSE menor que um
path tracer de 600 amostras por pixel. Fora dessa família, cada técnica abre mão de pelo menos uma das quatro
propriedades.

O Godot 4 é um piso baixo:

- até **15 luzes por item**, descartadas sem aviso;
- penumbra feita por filtro PCF ao longo de um mapa polar 1D;
- nenhuma GI;
- uma única PointLight2D já levou um Adreno 612 de **60 para 42 FPS**.

O ponto crítico é que **não existe nenhum número publicado de GI 2D em GPU móvel**. A única prova de RC em
celular é 3D (Love and Deepspace, GDC 2026) e não traz milissegundos.

No celular, o recurso escasso é a largura de banda da DRAM, não o ALU. O wgpu 29 **não dá acesso à memória
de tile** (sem input attachments, framebuffer fetch nem imageblocks). Por isso a arquitetura precisa evitar
buffers de tela cheia gravados e relidos.

A recomendação é uma arquitetura em camadas:

- RC 2D como solver padrão, ancorado no mundo e em resolução reduzida;
- HRC como camada «Ultra» e referência de qualidade;
- um caminho analítico de luz direta para fontes pequenas e nítidas;
- uma composição *forward* dentro do passe dos sprites;
- um path tracer de referência na CPU como juiz.

Tudo isso só fica de pé depois de um plano de medição com critérios de abandono explícitos.

## O piso a vencer: Godot e Unity descartam luzes e falsificam a penumbra

O Godot 4 faz a luz 2D como **forward de passe único por canvas item**. Cada item recebe uma lista de luzes
que o fragment shader percorre. A contagem é guardada em 4 bits e o shader lê `light_count = … & 15u`. Por
isso o teto real é **15 luzes por item** e 256 por render. As primeiras 15 na ordem da lista ficam e as
outras são **descartadas sem aviso** ([canvas.glsl](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/shaders/canvas.glsl),
[renderer_canvas_render_rd.cpp](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/renderer_canvas_render_rd.cpp)).
Usuários da 4.2 e da 4.3 confirmam o corte e dizem que não há como escolher quais luzes ficam
([Godot Forum](https://forum.godotengine.org/t/working-around-godots-2d-light-limitations-for-a-dark-game/81311)).
Esse teto é um **limite de formato**, não de hardware: viola diretamente o §0 do PH2D.

As sombras são **mapas polares 1D**. Cada PointLight2D usa 4 quadrantes de 512 texels num atlas de 2048 × 512,
filtrados por PCF5 ou PCF13 ao longo da linha
([renderer_canvas_render_rd.cpp](https://github.com/godotengine/godot/blob/master/servers/rendering/renderer_rd/renderer_canvas_render_rd.cpp)).
A penumbra resultante tem **largura angular constante**, dada por «Filter Smooth». Ela não depende do tamanho
da luz nem da distância ao oclusor. A documentação avisa que o PCF13 «should only be used for a few lights
at once» e que as sombras direcionais são sempre infinitas
([Godot docs](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)).
O SDF dos oclusores existe, mas só serve shaders de usuário e colisão de partículas. A «2D global
illumination» aparece como algo que o *usuário* pode escrever, não como recurso do motor
([Linietsky, 2020](https://godotengine.org/article/godots-2d-engine-gets-several-improvements-upcoming-40/)).

Os números de celular são ruins e têm nome:

- **Godot 4.1.1, Redmi Note 7 Pro (Adreno 612, 2340×1080)**: uma única PointLight2D sobre um ColorRect de
  tela cheia levou o jogo de **60 para 42 FPS** ([#81152](https://github.com/godotengine/godot/issues/81152)).
  São pelo menos ~7 ms por quadro só para essa luz **[derivado]**.
- **Godot 4.0, Snapdragon 860, 665 e 662**: com qualquer luz 2D, o jogo caiu para **9 FPS**
  ([#73805](https://github.com/godotengine/godot/issues/73805)).
- **PR #100302 (Godot 4.4)**: o PR atacou o custo de CPU de `4 × luzes × oclusores` draw calls e levou um
  **Pixel 4 de 17 para 60 FPS** numa cena de pior caso
  ([PR #100302](https://github.com/godotengine/godot/pull/100302)).
- O próprio clayjohn descreve o laço de luz como «heavily GPU ALU bound». Desenrolá-lo rendeu menos do que
  ele esperava ([PR #100501](https://github.com/godotengine/godot/pull/100501)).

Há portanto dois gargalos distintos: CPU/driver nas passadas de sombra, e ALU por pixel × luzes. O PH2D
precisa medir os dois separadamente.

O Unity URP 2D tem outra forma, *deferred* em espaço de tela:

1. desenha as sombras em texturas;
2. desenha cada luz em *light textures* a **0,5× da tela por omissão**;
3. desenha os sprites amostrando essas texturas.

As fontes são o [Unity 6 Manual](https://docs.unity3d.com/6000.0/Documentation/Manual/urp/Lights-2D-intro.html)
e o guia de [otimização de luzes 2D](https://docs.unity3d.com/6000.1/Documentation/Manual/urp/2d-lights-optimize-methods.html).
O custo cresce com estilos de blend × batches de sorting layers × área das luzes. Com normal map, entra ainda
um pré-passe de tela cheia por batch, que a própria Unity chama de «very expensive». O Unity declara a luz
2D **não physically based**.

Nenhum dos dois motores tem GI 2D, rebote ou sprite emissivo que ilumine os vizinhos. O nicho que o PH2D pode
ocupar está vazio nos dois incumbentes. O middleware mais popular do Unity, o Smart Lighting 2D, só oferece a
alegação do vendedor de ser «3–5 times faster than Unity URP 2D lights», sem dispositivo
([Asset Store](https://assetstore.unity.com/packages/slug/112535)). Essa é a medida de quão baixo está o
piso.

## Só a família Radiance Cascades entrega GI com custo independente das luzes

O RC parte o campo de radiância em faixas de distância. As faixas próximas guardam resolução espacial fina
e angular grossa; as distantes, o contrário. A justificativa é a «condição de penumbra»: o espaçamento entre
sondas cresce ∝ D e o passo angular ∝ 1/D
([Sannikov, RC paper](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf)).

Cada cascata traça só o seu intervalo curto. Depois as cascatas são fundidas de cima para baixo, como alfa
pré-multiplicado: `⟨r_n + t_n·r_f, t_n·t_f⟩` ([HRC, eqs. 7–8](https://arxiv.org/pdf/2505.02041)).

Em 2D, a variante usada na prática tem **ramificação 4**: 2× de espaçamento por eixo e 4× de direções. Assim
cada cascata custa o mesmo que a c0, e o custo total fica ∝ ao número de cascatas, que é logarítmico na
resolução ([m4xc.dev](https://m4xc.dev/articles/fundamental-rc/)).

A propriedade decisiva foi **medida**:

- O Radiant 2d iluminou «2, 10², 100² particles in the same amount of time of about 12ms»
  ([RC paper, Fig. 13](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf)).
- O HRC «does not branch based on the scene data, it has constant time for a given size»
  ([HRC](https://arxiv.org/pdf/2505.02041)).

Todo pixel emissivo é uma luz. A penumbra sai do transporte, não de um filtro.

O RC clássico tem defeitos conhecidos, e cada um tem cura:

- **Anéis concêntricos em volta das luzes.** A cura é o *bilinear fix*: 4× raios por fusão, um para cada
  sonda bilinear da cascata de cima ([Yaazarai](https://github.com/Yaazarai/GMShaders-Radiance-Cascades),
  [Split RC §2](https://arxiv.org/pdf/2607.20384)). As correções do tipo «costumam custar o dobro do tempo
  de quadro» ([jason.today](https://jason.today/rc)).
- **Custo de fusão e de memória.** A **pré-média** corta as leituras de fusão de 16 para 4 e a memória em
  75 % ([GM Shaders RC 2](https://mini.gmshaders.com/p/radiance-cascades2)).
- **Layout *direction-first*.** Foi anunciado como 2× mais rápido e depois **retratado**: o ganho vinha de um
  problema do GameMaker (mesma fonte). Vale pela depuração, não pela velocidade.
- **Penumbras pequenas longe da luz.** Esta não tem cura barata. Na cena de penumbra de 512² do paper HRC,
  o RC clássico otimizado deu **RMSE 0,0354 em 1,96 ms** contra **0,0184 em 1,85 ms** do HRC. A cascata que
  apanha a luz «não tem resolução espacial» para a penumbra ([HRC Fig. 8](https://arxiv.org/pdf/2505.02041)).

O **Holographic RC** (Freeman, Sannikov e Margel; arXiv 2505.02041) usa grades de sondas que perdem
resolução numa só direção, por quadrante. Isso remove os buracos entre raios e dá a mesma precisão a todas as
cascatas. É determinístico e sem ruído, e por isso dispensa cache temporal. Os números saem da Tabela 1 do
[paper](https://arxiv.org/pdf/2505.02041), numa RTX 3080 Laptop com fp16:

| Tamanho | Merge Up | Merge Down | Total |
|---|---|---|---|
| 256² | 0,30 ms | 0,25 ms | **0,55 ms** |
| 512² | 1,00 ms | 0,85 ms | **1,85 ms** |
| 1024² | 4,10 ms | 3,57 ms | **7,67 ms** |
| 2048² | 18,1 ms | 15,8 ms | **33,9 ms** |

A 1024², o HRC dá RMSE **0,0063 em 7,67 ms**. Um path tracer com 600 spp dá **0,0096 em 1080 ms**. O HRC
tem limites:

- borra a sombra em ~4 px;
- mostra xadrez nas bordas das sombras;
- mostra Moiré com luzes **menores que 8× o espaçamento base**;
- traça só 20 direções de saída, o que borra o especular;
- custa **≈ N·X² de memória** na estrutura de aceleração.

Os autores recomendam calcular as sondas em resolução reduzida e interpolar para uma tela 2–4× maior
([HRC §5](https://arxiv.org/pdf/2505.02041)). Não há código público.

A memória decide o que cabe no celular **[derivado]**:

- **RC clássico**, ramificação 4, c0 com 1 sonda por pixel e 4 direções, RGBA16F, pré-média e duas
  cascatas vivas em ping-pong: ~16,6 MB por cascata a 1080p, ~33 MB o par. Com c0 a cada 2 px, ~8 MB o par.
- **HRC a 1024²**: ~126 MB só na estrutura de aceleração.

O orçamento de luz do PH2D em iPad e iPhone é de 80 MB. O HRC fica fora dele, salvo na variante com BVH que
os autores mencionam ([HRC §4.3.2](https://arxiv.org/pdf/2505.02041)).

A tabela abaixo compara todas as famílias pesquisadas. «Mobile» resume a melhor evidência existente, e
«sem número» quer dizer que nenhuma fonte publicou tempo em dispositivo nomeado.

| Família (exemplo) | Penumbra | GI / rebote | Emissivo ilumina vizinho | Nº de luzes | Custo cresce com | Mobile | Maturidade | Fontes |
|---|---|---|---|---|---|---|---|---|
| Formas de luz aditivas / pintadas (Hollow Knight; recomendada pela doc do Godot) | sem sombra | não | só visual | ilimitado | área pintada | excelente | produção | [Godot docs](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html), [Unity: Hollow Knight](https://unity.com/case-study/hollow-knight) |
| Godot 4: forward por item + polar 1D + PCF | filtro de largura angular fixa | não | não | **15/item**, 256/render | pixels × luzes × taps; ≥1 passe de sombra por luz | **medido ruim**: Adreno 612 60→42 FPS com 1 luz | produção | [#81152](https://github.com/godotengine/godot/issues/81152), [PR #100302](https://github.com/godotengine/godot/pull/100302) |
| Unity URP 2D: light textures a 0,5× | «Softness» artística | não | não | sem cap por item | estilos × batches de layers × área; normal map = pré-passe de tela cheia | anedótico | produção | [Unity 6.1](https://docs.unity3d.com/6000.1/Documentation/Manual/urp/2d-lights-optimize-methods.html) |
| Mapa polar 1D por luz (mattdesl) | blur que cresce com a distância | não | não | ~20 numa GeForce 9400 | ≥2 passes por luz | fraco no Android (autor) | antigo | [lwjgl-basics](https://github.com/mattdesl/lwjgl-basics/wiki/2D-Pixel-Perfect-Shadows) |
| Geometria: quads duros / SFSS (Slembcke) | quase correta para luz-disco; escurece demais com oclusores separados sobrepostos | não | não | muitas | luzes × arestas (vértice) + preenchimento; ¼ res aceitável | alegação: 60 Hz num Raspberry Pi 4 | produção desde 2015 | [SFSS](https://www.slembcke.net/blog/SuperFastSoftShadows/) |
| Máscara + atlas (Celeste) | dura | não | não | 256 num passe | luzes × quads de parede | «fastest code in the game» | produção | [Noel Berry](https://noelberry.ca/posts/celeste_lighting/) |
| Raycast na CPU / polígono de visibilidade (box2dlights, Red Blob) | dura ou blur | não | não | 5–128 raios por luz | CPU: luzes × raios | preso à CPU | obsoleto | [box2dlights](https://raw.githubusercontent.com/wiki/libgdx/box2dlights/Performance-Tuning.md), [Red Blob](https://www.redblobgames.com/articles/visibility/) |
| Sombra por luz sobre SDF (Quilez/Aaltonen) | aproximada (k·h/t), vaza em cantos | não | não | por lista de tile | JFA 1× por quadro + luzes × pixels × passos; não depende do nº de oclusores | sem número | madura em 3D | [Quilez](https://iquilezles.org/articles/rmshadows/), [Golus (JFA)](https://web.archive.org/web/2023/https://bgolus.medium.com/the-quest-for-very-wide-outlines-ba82ed442cd9) |
| Muitas luzes: tiles (Filament) / estocástico (MegaLights) | a da sombra escolhida | não | não | centenas a milhares | sobreposições luz × tile; MegaLights ∝ pixels × K | MegaLights no Mali-G1: **41 vs 8 FPS** (~195 luzes) | produção em 3D | [Filament](https://github.com/google/filament/blob/main/filament/src/Froxelizer.h), [Arm MegaLights](https://developer.arm.com/community/arm-community-blogs/b/mobile-graphics-and-gaming-blog/posts/lighting-at-scale-bringing-hundreds-of-dynamic-lights-to-mobile-with-unreal-megalights) |
| Propagação em grade (Terraria) | nenhuma; a luz contorna cantos | difusão grosseira | sim, como fonte | muitas | células × fontes | trivial | produção | [0fps](https://0fps.net/2018/02), [Terraria wiki](https://terraria.wiki.gg/wiki/Lighting_mode) |
| GI por SDF com raios por pixel / sondas (Bigos, The Crypt, bevy_magic_light_2d) | física, mas com ruído | sim | sim | ilimitado | pixels × raios × passos (32–1000 raios, sempre ≤ 360×240) | sem número; o ruído exige filtro temporal | experimental | [Bigos](https://samuelbigos.github.io/posts/2dgi1-2d-global-illumination-in-godot.html), [The Crypt](https://builderbot.itch.io/the-crypt), [bevy_magic_light_2d](https://github.com/zaycev/bevy-magic-light-2d) |
| **RC clássico 2D** + bilinear fix + pré-média | correta, exceto luz pequena ou penumbra fina longe (RMSE 0,0354) | sim (realimentação) | **sim, nativo** | **ilimitado, custo constante** | pixels × nº de cascatas (log); não depende de luzes nem de oclusores | **sem número**; plausível com c0 ≥ 2 px | produção (PoE2, em espaço de tela) | [RC paper](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf), [GM Shaders](https://mini.gmshaders.com/p/radiance-cascades2) |
| **Holographic RC** | ~indistinguível da referência; borra ~4 px | sim | sim | ilimitado | O(X² log X); memória ≈ N·X² | provavelmente pesado (~126 MB a 1024² **[derivado]**) | paper de 2025, sem código | [HRC](https://arxiv.org/abs/2505.02041) |
| Ray query em hardware | dura e exata | não | não | por raio | raios × pixels | só A17 Pro+/M3+, Immortalis-G715+, Adreno 740+; experimental no wgpu | experimental | [Arm MegaLights](https://developer.arm.com/community/arm-community-blogs/b/mobile-graphics-and-gaming-blog/posts/lighting-at-scale-bringing-hundreds-of-dynamic-lights-to-mobile-with-unreal-megalights), [wgpu CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md) |
| Luz autoral (decals do INSIDE, mapas de direção do Ori) | pintada | falsa (decal de rebote) | pintada | — | custo de arte | excelente | produção | [Playdead](https://blog.playdead.com/articles/the_lighting_of_inside/lighting_of_inside.html), [TheGamer](https://www.thegamer.com/ori-and-the-will-of-the-wisps-artists-hand-painted-30000-light-maps/) |

A tabela mostra uma assimetria. Toda técnica com sombra por luz paga um termo **luzes × algo**:

- arestas no SFSS;
- passes no polar 1D;
- passos de march no SDF;
- taps de PCF no Godot.

Toda técnica que não paga esse termo abre mão de outra coisa: o ruído na GI por SDF, a correção física na
grade de propagação, a sombra nas formas aditivas. Só o RC remove o termo «luzes» sem perder a penumbra. Seu
ponto fraco é o oposto do das técnicas por luz: a fonte **pequena e nítida**, onde uma sombra analítica por
luz é exata e barata.

## Os jogos mais admirados pintam a luz; a única GI em produção é RC

Os jogos 2D e 2.5D com a luz mais elogiada são, na maioria, **dirigidos por arte e falsificados**:

- **INSIDE** usa decals colocados à mão para rebote, AO e sombras projetadas. «There is no global
  illumination or even automated AO at play here», e sombra de point light «would have been much too
  expensive» ([Playdead, 2016](https://blog.playdead.com/articles/the_lighting_of_inside/lighting_of_inside.html)).
- **Ori and the Will of the Wisps** pintou à mão ~30 000 mapas de direção de luz para ~7 000 assets. A equipe
  rejeitou as normais geradas por script como «plasticky»
  ([TheGamer, 2020](https://www.thegamer.com/ori-and-the-will-of-the-wisps-artists-hand-painted-30000-light-maps/)).
- **Celeste** trocou malhas por luz por máscaras recortadas por quads de parede, com até 256 luzes num atlas
  de 2048² desenhado num passe ([Noel Berry, 2017](https://noelberry.ca/posts/celeste_lighting/)).
- **Graveyard Keeper** pinta a luz em quatro lados de cada sprite e um script funde tudo numa normal. Cada
  objeto tem até quatro sprites de sombra girados no vertex shader, e o dia é feito com 10 LUTs
  ([Cherkasov, 2018](https://www.gamedeveloper.com/programming/graveyard-keeper-how-the-graphics-effects-are-made)).
- **Dead Cells** renderiza modelos 3D em pixel art com normais por quadro e um toon shader
  ([Vasseur, 2018](https://www.gamedeveloper.com/production/art-design-deep-dive-using-a-3d-pipeline-for-2d-animation-in-i-dead-cells-i-)).
- **Eastward** reconstrói os cenários em 3D e pinta os bump maps um a um, num motor próprio com iluminação
  deferred ([Game Developer](https://www.gamedeveloper.com/art/eastward-s-creators-share-insights-on-making-pixel-art-adventures)).

A lição é dupla. O teto *percebido* vem de canais autorais por asset, não só do solver. E nenhum desses
jogos tem GI.

A única GI de tempo real confirmada em produção nesta família é o **Path of Exile 2**. Ele usa RC em espaço
de tela com cascatas de sondas 2^i×1, a mesma escala do flatland. A marcha de tela roda 4 raios em
paralelo com máscara de oclusão de 128 bits. Sannikov reporta **3 ms numa GTX 1050** para a luz indireta de
um personagem brilhante, sem dizer a resolução. O mesmo texto admite que a variante de tela «can't capture
offscreen light sources»
([RC paper §4.2 e §4.5](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf)).
Nos consoles, a GI só entra no preset de fidelidade a 4K e sai do preset de 100+ FPS
([ONE Esports](https://www.oneesports.gg/gaming/path-of-exile-2-console-graphics-settings)). Até a
Grinding Gear trata a GI como custo premium em hardware de PS5.

O paper Split RC cita Core Keeper e Kyora como jogos 2D com RC
([Split RC §1](https://arxiv.org/pdf/2607.20384)). Uma fonte secundária de 2026 descreve a luz do Core Keeper
como buffer de células, não RC ([foro3d](https://foro3d.com/en/2026/mayo/iluminacion-dinamica-en-pixel-art-el-caso-tecnico-de-core-keeper.html)).
O ponto fica em aberto.

No celular, o estado da arte de 2026 é **3D, não 2D**. **Love and Deepspace** (Infold, GDC 2026, apresentado
com a Arm) entrega GI totalmente dinâmica no telefone:

- usa **surfels pré-computados por asset na importação**;
- faz ray tracing em software contra os surfels;
- usa **RC 3D** para acelerar esse traçado;
- alega performance «comparable to mainstream PC approaches».

Não há milissegundos nem dispositivo publicados
([GDC schedule](https://schedule.gdconf.com/session/the-evolution-of-real-time-global-illumination-in-love-and-deepspace-presented-by-arm/917336),
[Arm](https://newsroom.arm.com/blog/takeaways-from-gdc-festival-of-gaming-2026)).

O resto da GI que roda em celular pré-computa a geometria: Roco Kingdom usa PRT, e o APV da Unity precisa de
modo por vértice nos aparelhos fracos
([Roco](https://schedule.gdconf.com/session/illuminating-rocos-legacy-scalable-global-illumination-from-theory-to-practice/915519),
[APV](https://activation.unity3d.com/roadmap/1972-adaptive-probe-volumes-apv-per-vertex-quality)).
O Lumen no celular é experimental, só Android, e **não roda em iOS**
([Epic](https://dev.epicgames.com/documentation/unreal-engine/using-lumen-global-illumination-on-mobile-in-unreal-engine)).
O padrão que sai em produção é: **pré-computar o proxy geométrico, avaliar a luz ao vivo**. O análogo 2D é
um proxy de oclusão e emissão por sprite, gerado na importação.

As bibliotecas abertas confirmam a escassez de números:

- solis_2d (RC em Bevy, parado desde 2024) ([lib.rs](https://lib.rs/crates/solis_2d));
- bevy_magic_light_2d (SDF, sondas 8×8 e acumulação temporal de 8 quadros)
  ([GitHub](https://github.com/zaycev/bevy-magic-light-2d));
- Firefly, bevy_light_2d e Bulb, que só fazem luz direta com sombra.

Nenhuma publica tempo. Os números de RC 2D que existem são de desktop e frágeis:

- **0,3 ms numa GTX 970**, sem resolução ([Mytino](https://x.com/MytinoDev/status/1722365866979774530));
- 60 FPS a 1200×900 com 6 cascatas numa GTX 1660, provavelmente preso ao vsync
  ([Hybrid46](https://github.com/Hybrid46/RadianceCascade2DGlobalIllumination));
- ≤ 8,33 ms a 1080p numa RTX 3080, preso ao limite de 120 FPS
  ([GM Shaders](https://mini.gmshaders.com/p/radiance-cascades2)).

## No celular o custo é banda, e o wgpu 29 não alcança a memória de tile

Todas as GPUs móveis relevantes renderizam em tiles sobre memória no chip: Apple TBDR, Mali com tiles de
16×16 e Adreno com GMEM. O que custa é cada `load`, cada `store` e cada passe que lê a saída do anterior
([Apple](https://developer.apple.com/documentation/metal/tailor-your-apps-for-apple-gpus-and-tile-based-deferred-rendering),
[Arm Best Practices r3.4](https://documentation-service.arm.com/static/67a62b17091bfc3e0a947695)).

A regra da Arm é **~100 mW por GB/s** de tráfego de DRAM
([Arm](https://developer.arm.com/community/arm-community-blogs/b/mobile-graphics-and-gaming-blog/posts/memory-limits-with-vulkan-on-mali-gpus)).
A Samsung enquadra o celular em **~5 W e ~30 GB/s** compartilhados. Ela também mediu, num Mali-G76, que
trocar um alvo RGBA16F por **R11G11B10F** levou o Forza Street de **53 para 60 FPS**
([Vulkanised 2023](https://vulkan.org/user/pages/09.events/vulkanised-2023/vulkanised_2023_battle_tested_optimisations_for_mobile_v2.pdf)).

O custo dos buffers de tela cheia **[derivado]**:

- Um buffer RGBA16F a 1080p, gravado e relido uma vez por quadro, custa **2 GB/s a 60 Hz** (~0,2 W).
- Um JFA em resolução cheia a 1080p tem ~11 passes e movimenta ~180 MB por quadro, ~11 GB/s. Isso é da
  ordem da banda total alcançada num Adreno antigo
  ([Chips and Cheese](https://chipsandcheese.com/p/inside-qualcomms-adreno-530-a-small-mobile-igpu)).
  O SDF tem de ser feito em resolução reduzida e só nas regiões sujas.

Para o RC clássico **[derivado]**, com c0 a cada 2 px a 1080p, RGBA16F e pré-média:

- ~7 cascatas × (gravação + leitura) × 4,1 MB ≈ **58 MB por quadro, ~3,5 GB/s a 60 Hz**, se nada ficar no
  cache;
- com c0 a cada 4 px, ~0,9 GB/s.

O espaçamento da c0 é, portanto, o principal botão de energia, além de ser o de qualidade.

O desempenho sustentado é outro corte. Nos testes de estresse sem resfriamento, o Dimensity 9300 terminou
com **44 % de estabilidade** e o Snapdragon 8 Gen 3 com **67 %**
([Android Authority](https://www.androidauthority.com/snapdragon-8-gen-3-dimensity-9300-benchmarked-3395385/)).
A Unity recomenda usar só **~65 % do quadro**, cerca de 10,8 ms a 60 Hz
([Unity](https://unity.com/how-to/best-practices-for-profiling-game-performance)).
O orçamento de luz tem de ser cumprido com o clock sustentado, não no pico.

O que o wgpu 29.0.4, versão fixada no workspace, expõe ou não para este trabalho:

| Capacidade | wgpu 29 | Consequência para a luz 2D | Fonte |
|---|---|---|---|
| Memória de tile: input attachments, subpasses, framebuffer fetch, imageblocks, raster order groups | **Não.** Input attachments: [#10148](https://github.com/gfx-rs/wgpu/issues/10148) aberta em ago/2026, segundo rascunho «will be a while»; naga [#5233](https://github.com/gfx-rs/wgpu/issues/5233) aberta. Nada sobre framebuffer fetch ou tile shader. | Um G-buffer de normais seguido de passe de luz *deferred* paga sempre a ida e volta à DRAM. Prefira a composição *forward* no passe dos sprites. | [features.rs v29](https://github.com/gfx-rs/wgpu/blob/v29.0.0/wgpu-types/src/features.rs) |
| Anexos transientes (memoryless / lazily-allocated) | Sim, desde v28 (`TextureUsages::TRANSIENT`, exige `StoreOp::Discard`). Renomeado `TRANSIENT_ATTACHMENT` na v30. | Só serve para depth/stencil e MSAA que nunca são relidos; não guarda o buffer de luz no tile. | [PR #8247](https://github.com/gfx-rs/wgpu/pull/8247), [PR #9568](https://github.com/gfx-rs/wgpu/pull/9568) |
| `LoadOp::DontCare` | Sim, desde v28, atrás de um token `unsafe` | Use em todo alvo reescrito por inteiro. `LOAD` desnecessário custou 600 MiB/s num Mali-G76. | [PR #8549](https://github.com/gfx-rs/wgpu/pull/8549), [Khronos](https://github.com/KhronosGroup/Vulkan-Samples/tree/main/samples/performance/render_passes) |
| f16 em shader | `SHADER_F16` em Vulkan, Metal, DX12 e WebGPU; **não em GLES**. Polyfill de IO no Vulkan desde v27. A v30 proíbe derivadas em f16. | Cascatas e marcha em f16 vetorizado (`vec4<f16>`). Precisa de um caminho f32 equivalente para GLES. | [features.rs v29](https://github.com/gfx-rs/wgpu/blob/v29.0.0/wgpu-types/src/features.rs), [CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md) |
| Storage `rgba16float` | Só escrita ou só leitura no core. Leitura e escrita só via `TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES`, nativo. | Cascatas em ping-pong (escreve numa, lê a outra com `textureLoad`). | [W3C WebGPU](https://www.w3.org/TR/webgpu/) |
| Storage `r16float`, `rg16float`, `rg11b10ufloat`, `rgb10a2unorm` (tier1) e leitura-escrita de 16 bits (tier2) | **Não implementado** ([#8122](https://github.com/gfx-rs/wgpu/issues/8122) aberta desde 2025-08; «PRs welcome») | Formato estreito em compute só empacotando à mão em `r32uint` (leitura-escrita no core). É uma variante a medir **[inferência]**. | [#8122](https://github.com/gfx-rs/wgpu/issues/8122) |
| `r32float` / `r32uint` leitura-escrita | Sim, no core | Sementes do JFA empacotadas, acumulação atômica de listas por tile | [W3C WebGPU](https://www.w3.org/TR/webgpu/) |
| Subgroups | `Features::SUBGROUP` só nativo (Vulkan, DX12, Metal) | Otimização opcional; nunca o caminho base | [features.rs v29](https://github.com/gfx-rs/wgpu/blob/v29.0.0/wgpu-types/src/features.rs) |
| Ray query | `EXPERIMENTAL_RAY_QUERY` no Vulkan (`VK_KHR_ray_query`) e no Metal (iOS 18+/macOS 15+, com `supportsRaytracingFromRender`). AABB e geometria não opaca no Metal só na v30 ([#9304](https://github.com/gfx-rs/wgpu/pull/9304)). | Acelerador opcional de sombra dura em A17 Pro+/M3+, Immortalis-G715+ e Adreno 740+. Nenhum custo móvel publicado. | código vendorizado `wgpu-hal-29.0.4`, [CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md) |
| Timestamps | `TimestampWrites` nas bordas de passe; dentro do passe é uma feature à parte | Meça cada passe pela borda, que é o que os tilers suportam | [CHANGELOG](https://github.com/gfx-rs/wgpu/blob/trunk/CHANGELOG.md) |

Duas regras práticas da Arm completam o quadro. Para processamento de imagem simples, prefira **fragment a
compute**: o fragment ganha compressão AFBC, eliminação de transações e write-back em paralelo. Compute só
ganha quando funde vários passes num dispatch ou quando compartilha dados entre vizinhos. A *shared memory*
do Mali **não está no chip**: é RAM atrás do cache de load/store
([Arm Best Practices](https://documentation-service.arm.com/static/67a62b17091bfc3e0a947695)).

No Adreno, o upscaler GSR 2 mediu **0,905 ms em fragment de 2 passes contra 1,801 ms em compute**, a
1260×2800 num Snapdragon 8 Gen 3 ([SGSR 2](https://github.com/SnapdragonStudios/snapdragon-gsr/tree/main/sgsr/v2)).
A escolha entre fragment e compute para as fusões do RC é empírica e entra no plano de medição.

## Arquitetura recomendada: cinco camadas e um juiz de referência

O desenho segue o pedido do dono: o teto mais alto disponível, e um caminho mais barato que converge para a
**mesma resposta**. Como a luz é apresentação, «mesma resposta» quer dizer **o mesmo transporte de luz,
medido por RMSE contra uma referência**, não igualdade bit a bit. O juiz é um **path tracer 2D de
referência na CPU**: determinístico, em f64, com milhares de amostras por pixel. Ele faz o papel que os 600
spp têm no paper HRC e serve de fixture para os gates de cada camada.

| Camada | O que faz | Técnica recomendada | Por quê (evidência) |
|---|---|---|---|
| **C0 Campo** | Rasteriza a cena num campo de luz: emissão RGB, opacidade/extinção e albedo, em grade **ancorada no mundo** e em resolução de luz, com **margem fora da tela** | Um passe de fragment barato a partir de proxies por sprite (máscara de opacidade com mips, emissivo colorido) gerados na importação | Precompute do proxy por asset (Love and Deepspace); o RC de tela do PoE2 não vê emissores fora da tela; o Godot usa SDF com *oversize* de 120 % ([ProjectSettings](https://github.com/godotengine/godot/blob/master/doc/classes/ProjectSettings.xml)) |
| **C1 Visibilidade** | Acelera os intervalos longos | SDF por JFA em resolução reduzida; camada estática em cache, dinâmica só nas regiões sujas; sementes em `r32uint`. Variante sem SDF: DDA no campo, como o HRC | O JFA é limitado pela unidade de textura (~67–75 µs por passe a 1080p numa RTX 2080S, [Golus](https://web.archive.org/web/2023/https://bgolus.medium.com/the-quest-for-very-wide-outlines-ba82ed442cd9)); em resolução cheia no celular custa ~11 GB/s **[derivado]** |
| **C2 Solver de radiância** | GI, rebote, emissivos como luz, penumbra física | **RC 2D clássico**, ramificação 4, *bilinear fix*, pré-média, f16, trace+merge fundidos por cascata. Degraus: **Ultra = HRC** (desktop) · **Alto** = c0 a 1 px · **Médio** = c0 a 2 px com cascatas altas amortizadas em 2–4 quadros · **Baixo** = c0 a 4 px | Custo constante nas luzes ([RC paper](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf)); amortização é o trabalho futuro do próprio HRC ([HRC §6](https://arxiv.org/pdf/2505.02041)); trace+merge num passe ([GM Shaders](https://mini.gmshaders.com/p/radiance-cascades2)) |
| **C3 Luz direta analítica** | Fontes pequenas e nítidas que o RC resolve mal; normal e especular nítidos; muitas luzes explícitas | Lista de luzes por tile 2D (binning na CPU para centenas, compute para milhares) + sombra pelo **mesmo SDF** da C1 com o estimador de Aaltonen. Ray query só como acelerador medido, nunca como base | RC falha com «a point light that is just too small» ([RC paper](https://github.com/Raikiri/RadianceCascadesPaper/blob/main/out_latexmk2/RadianceCascades.pdf)); HRC falha abaixo de 8× o espaçamento base; tiles ([Filament](https://github.com/google/filament/blob/main/filament/src/Froxelizer.h)) |
| **C4 Sombreamento e composição** | Ilumina o sprite | **Forward dentro do passe dos sprites**: amostra a fluência com upsample bilateral conjunto guiado por opacidade e normal; termo direcional da c0 para normal map; modo pixel-art (snap ao texel, rampa toon, dither); canais autorais (formas aditivas, mapas de direção, decals); depois o bloom e o tonemap atuais | Sem memória de tile no wgpu 29; bilateral ([Kopf 2007](https://johanneskopf.de/publications/jbu/paper/FinalPaper_0185.pdf)); snap ([Godot docs](https://docs.godotengine.org/en/stable/tutorials/2d/2d_lights_and_shadows.html)); normal map via RC ([solis_2d](https://lib.rs/crates/solis_2d)); canais autorais (Ori, INSIDE) |
| **C5 Governador** | Escolhe o degrau por dispositivo e por folga térmica; amortiza no tempo | Perfil medido por classe de GPU + sinal de folga térmica (ADPF no Android) controlando espaçamento da c0, nº de cascatas e taxa de atualização | Estabilidade sustentada de 44–67 %; ADPF ([Arm](https://community.arm.com/arm-community-blogs/b/graphics-gaming-and-vr-blog/posts/getting-started-with-adpf)) |

A C2 e a C3 se dividem por uma regra que **se mede, não se escolhe**. Toda luz entra no RC como disco
emissivo. Assim a penumbra sai física e o custo não depende da contagem. As luzes **abaixo do tamanho em que
o RMSE do RC passa o da C3**, para aquele degrau, são desviadas para o caminho analítico. O limiar é uma
função do espaçamento da c0, levantada pela medição, nunca uma constante escrita à mão (§0 do PH2D).

O rebote vem de realimentar a fluência do quadro anterior × albedo na emissão da C0: um rebote por quadro,
convergindo ao longo dos quadros. É a forma que o bevy_magic_light_2d e o The Crypt usam
([bevy_magic_light_2d](https://github.com/zaycev/bevy-magic-light-2d), [The Crypt](https://builderbot.itch.io/the-crypt)).
Ela tem de ser confirmada contra a referência com 2+ rebotes **[inferência]**.

Ancorar a grade no mundo, com encaixe ao texel de luz, evita o tremor sob pan da câmera. O RC clássico é
determinístico e sem ruído. A acumulação temporal fica restrita à amortização e ao rebote, não à remoção de
ruído **[inferência]**.

O mapeamento para o PH2D atual tem consequências concretas:

- **`SpriteEmissive`**, hoje um escalar que alimenta o bloom, passa a ser emissão colorida com máscara
  opcional e alimenta a C0. O bloom continua como efeito perceptivo à parte.
- **`GameRt` em Rgba16Float** continua sendo o alvo da composição. Vale medir no celular um formato
  estreito para o buffer de luz e, separadamente, para o próprio `GameRt`, pelo resultado de 53 → 60 FPS
  da Samsung.
- **A crate `ph2d-light`** é o rig de 4 lâmpadas do impasto, sem oclusores nem atenuação. Ela fica intocada,
  e a luz 2D nasce como **crate nova da família**, em drop-crate, componentes ECS e shell como composição.
  O doc de stack que diz que ela é Radiance Cascades está **errado** e deve ser corrigido.
- **Não há render graph declarativo.** A cadeia C0 → C1 → C2 → (C3) → C4 tem 4 a 6 passes com dependências.
  A crate de luz deve declarar a própria lista ordenada de passes, com entradas e saídas explícitas. Assim
  as decisões de `DontCare`, `Discard`, `TRANSIENT` e aliasing de texturas ficam num único lugar, e a shell
  só chama a crate.
- **O orçamento do doc de stack (2,5 ms a 60 Hz, 80 MB em iPad/iPhone)** é uma meta, não uma medição. Pela
  conta de memória acima, ele exclui o HRC no celular e exige c0 ≥ 2 px em telas de iPad. A 2732×2048 com
  c0 a 1 px, o par de cascatas pré-médias já soma ~90 MB **[derivado]**. O plano abaixo transforma a meta
  num número medido.

## Plano de medição: o PH2D publicará os primeiros números de GI 2D em celular

Nenhuma técnica de GI 2D tem número publicado em Apple A-series, Adreno ou Mali. O plano mede tudo numa
**única rodada intercalada**, como mandam o §0.10 e o `MEDIR_VELOCIDADE.md`. Todas as variantes são
alternáveis em execução no mesmo processo, intercaladas em blocos rodados. Para comparar A × B, o resumo é o
**mínimo**. Para cumprir o orçamento, vale o **regime sustentado** após 10 minutos de carga, com clock
travado onde o dispositivo permitir, como faz a Samsung.

Os tempos saem de `TimestampWrites` por passe. A banda e a energia saem dos contadores do fabricante: Arm
Streamline, captura de GPU do Xcode e o profiler da Snapdragon.

O Godot (MIT) entra como **oráculo rodado por script** sobre as mesmas cenas, exportadas para ele. Ele é
medido, não copiado (§0.9).

Há um pré-requisito não verificado neste relatório: se o PH2D ainda não tem build iOS/Android, a primeira
entrega é um **harness mínimo de benchmark em wgpu**, fora da shell do editor, nas duas plataformas.

As cenas de entrada são nossas, determinísticas e fotografadas antes de cada rodada:

| Cena | Conteúdo | O que isola |
|---|---|---|
| S1 «uma luz» | Réplica da #81152: retângulo de tela cheia + um sprite + uma luz pontual | Custo fixo; comparação direta com o Godot no mesmo aparelho |
| S2 «enxame» | 1 / 10 / 100 / 1 000 / 10 000 sprites emissivos | Planura da curva luzes → custo (RC) contra o crescimento do Godot e da C3 |
| S3 «labirinto» | 0 / 10 / 100 / 1 000 oclusores poligonais e por alfa | Custo de oclusor; cache estático da C1 contra regiões sujas |
| S4 «penumbra» | Réplica da HRC Fig. 8: 512², oclusor de 14 px, luz de 10 px | RMSE contra a referência CPU, por degrau (C2 e C3) |
| S5 «faísca» | Luzes de 1–16 px a distâncias crescentes do oclusor | Limiar de desvio C2 → C3 (onde o RMSE do RC passa o analítico) |
| S6 «pan» | S2 com câmera em pan e zoom contínuos | Tremor temporal (variação quadro a quadro de pixels estáticos) |
| S7 «resoluções» | S2 a 540p / 720p / 1080p / 1440p / nativa do aparelho | Escala com pixels e nº de cascatas; memória alocada |
| S8 «rebote» | Sala fechada com uma fonte e paredes coloridas | Convergência do rebote por realimentação contra a referência com 2+ rebotes |

As variantes medidas na mesma rodada são:

- espaçamento da c0 em 1, 2 e 4 px;
- *bilinear fix* contra *nearest-interlaced*;
- RC clássico contra HRC;
- com e sem SDF (DDA puro);
- fusões em fragment contra compute;
- formato das cascatas: RGBA16F contra empacotado em `r32uint`;
- formato do buffer de luz e do `GameRt`: RGBA16F, RG11B10 ou RGB10A2;
- f16 contra f32;
- C3 com SDF contra ray query, onde existir.

Os aparelhos formam uma matriz mínima por fabricante, um de gama média e um topo de linha:

| Fabricante | Gama média | Topo de linha |
|---|---|---|
| Apple | iPhone A15/A16, sem RT | A17 Pro ou mais novo (RT) + um iPad M-series, alvo do doc de stack |
| Qualcomm | Adreno 6xx, classe do Adreno 612 da #81152 | Adreno 740/750 |
| Arm | Mali-G57/G68 | Immortalis-G715/G720 ou mais novo |

Como referência entram a GPU da workstation e um iGPU.

**Aparelhos que o dono TEM (2026-10-08):** iPhone 13 (A15, sem RT — gama média Apple) · iPhone 16 (A18,
com RT — topo Apple) · dois POCO, «5» e «8» (modelo exacto a confirmar: X5/F5/M5… decide se há Adreno e/ou
Mali na matriz). Fase de celular **adiada pelo dono** («ainda é cedo»): o desenvolvimento e a primeira
rodada correm no desktop, com o tráfego de memória por quadro **contado** por passe (ver abaixo), que não
depende do aparelho. ⚠️ Simulador (iOS Simulator / emulador Android) corre na GPU do HOSPEDEIRO: serve para
CORREÇÃO (a imagem, o shader compila no backend), nunca para tempo nem banda.

Os critérios de abandono são decisões fixadas **antes** de medir. Eles vêm do orçamento declarado e da
evidência citada. Não são limites no código.

| # | Critério | Se falhar |
|---|---|---|
| K1 | O degrau padrão (RC, c0 a 2 px) cabe em **≤ 2,5 ms sustentados** na S7 a 1080p de saída em **todos** os topos de linha | Padrão desce para c0 a 4 px com amortização; se ainda falhar, a C2 deixa de ser padrão nessa classe e a C3 assume, com o RC como opcional |
| K2 | Memória total de luz **≤ 80 MB** na resolução nativa de cada aparelho | Configuração rejeitada para o aparelho; o espaçamento mínimo da c0 vira função medida da resolução |
| K3 | Na S2, o custo do RC entre 1 e 10 000 emissores varia **≤ 5 %** | Defeito: algo no caminho depende do conteúdo e deve ser encontrado antes de seguir |
| K4 | Na S1 e em cada ponto da S2, o degrau escolhido para o aparelho é **mais barato que o Godot** no mesmo aparelho, e a imagem tem RMSE menor contra a referência | O degrau está errado para a classe; o governador tem de escolher outro |
| K5 | O HRC entra no celular só se custar ≤ 2,5 ms com sondas a 512² **e** cortar o RMSE da S4 em ≥ 40 % contra o RC de mesmo tempo (o paper mostra ~48 % no desktop) | O HRC fica como «Ultra» de desktop e juiz secundário |
| K6 | O *bilinear fix* custa ≤ 2× o RC sem correção | Testar *nearest-interlaced*; se o RMSE da S4 piorar > 20 %, aceitar o custo e descer o espaçamento |
| K7 | O tráfego de DRAM da luz é **≤ 3 GB/s a 60 Hz** (~0,3 W pela regra da Arm) no topo de linha | Reduzir resolução da C1/C2, fundir passes, formato estreito |
| K8 | O tremor na S6 fica abaixo do limiar visual combinado no smoke com o dono | Ancorar a grade com encaixe mais fino ou adicionar estabilização temporal só nas cascatas altas |
| K9 | O rebote da S8 converge em ≤ 10 quadros e com RMSE ≤ ao da luz direta contra a referência com 2+ rebotes | Rebote vira opção de degrau Alto/Ultra apenas |

Os limiares K3, K5, K6 e K7 são pontos de partida e ficam registrados com a tabela da rodada ao lado. Quem
mover um deles reconfere a nota (§0). O resultado da rodada escolhe o degrau padrão por classe de GPU, o
limiar C2 → C3 em função do espaçamento, e o formato de cada buffer. Esse resultado vira a primeira tabela
pública de GI 2D em celular.

## Conclusão

A pesquisa muda a pergunta. «Que técnica de luz 2D é a melhor?» já tem resposta no papel: RC para o
transporte de luz, HRC como teto de correção, sombra analítica para as fontes minúsculas, e canais autorais
para o que a física não pinta. A pergunta aberta é se essa resposta **cabe na banda de um celular**, e
ninguém a respondeu publicamente. O orçamento real não é «ms de GPU». São **bytes de DRAM por quadro sob
clock sustentado**, e o espaçamento da cascata 0 é o botão que governa os dois.

Vencer o Godot é a parte fácil: ele descarta a 16.ª luz e falsifica a penumbra. A parte extraordinária é
publicar o primeiro número de GI 2D medido em Apple, Adreno e Mali, com uma referência que diz quanto cada
degrau erra.

Três lacunas ficam explícitas:

- não há código público do HRC;
- o Love and Deepspace não publicou milissegundos;
- não se verificou se o PH2D já compila para iOS/Android.

A última é a primeira a fechar, porque sem ela nenhum critério de abandono pode ser avaliado.
