# Constraints e rigging em software de animação 2D especializado (insumo para PH2D)

> Pesquisa de 25/10/2026. Fontes primárias (docs oficiais) quando possível; afirmações sem fonte foram
> para «Gaps». Orçamento: ~19 chamadas de ferramenta — Moho, DragonBones, Creature, Adobe Animate,
> Synfig e Grease Pencil ficaram com cobertura fraca ou nenhuma (ver Gaps).

## Q1 — Catálogo: o que cada sistema oferece (parâmetros, UI, elogios/queixas)

### Takeaway
Spine e Rive têm o catálogo mais limpo e documentado: poucos tipos de constraint, cada um com **mix/strength
animável 0–100 %**, e uma **ordem explícita** da pilha. Harmony modela constraints como **nós** num grafo
(Transform Limit, Transformation-Gate, Constraint Switch, Dynamic Spring), o idioma mais próximo do Motion
Nodes do PH2D. Live2D não tem «constraints» de osso: tem **parâmetros** que dirigem deformadores, e a
**física é um pêndulo multi-estágio de parâmetro para parâmetro**.

### Cited Findings

**Spine (Esoteric) 4.2 / 4.3**

- *IK — criação:* selecionar 1 osso (IK de 1 osso: aponta para o alvo) ou um osso + um filho (IK de 2 ossos:
  a ponta do filho alcança o alvo); `New... > IK Constraint` entra em modo alvo; clicar num osso existente ou no
  vazio cria o alvo; clicar num osso restrito cria o alvo na ponta dele. O alvo não pode ser descendente dos
  ossos restritos. **3+ ossos não é suportado** (usar várias IK ou FK). — [Spine IK](https://esotericsoftware.com/spine-ik-constraints)
- *IK — parâmetros:* **Positive** (só 2 ossos; o filho dobra no sentido anti-horário relativo ao pai; animável)
  = a direção de dobra; **Compress** (só 1 osso; encolhe quando o alvo está mais perto que o comprimento);
  **Stretch** (estica quando o alvo excede o alcance; em 2 ossos zera o Y local do filho, não se aplica com
  softness > 0 nem com escala não-uniforme do pai); **Uniform** (escala X e Y juntos ao comprimir/esticar);
  **Softness** (só 2 ossos; desacelera perto do alcance máximo — o valor é a distância antes do alcance máximo onde
  a desaceleração começa; evita o «estalo» do joelho esticado); **Mix** FK(0)↔IK(100) animável. — [Spine IK](https://esotericsoftware.com/spine-ik-constraints)
- *IK — queixa documentada:* com mix entre 0 e 100, mover o alvo pode **inverter subitamente** o sentido da
  rotação porque a interpolação toma o caminho angular mais curto. Limitações: não dá para desligar herança de
  rotação/escala/shear nos ossos restritos; o shear local do pai é zerado. — [Spine IK](https://esotericsoftware.com/spine-ik-constraints)
- *Transform constraint:* **Local** (modifica a transformação local vs. a do mundo); **Relative** (soma a
  transformação do alvo à do osso em vez de a substituir); **Offset** (sliders somados aos valores do alvo, ex.
  +90°); **Match** (copia a pose atual do 1.º osso restrito para os offsets — fluxo: mix a 0, ajustar, Match);
  **Mix** por canal, animável, com «Link sliders». Nota 2D: com Local desligado e Relative ligado, **inverter
  o alvo com escala negativa pode girar os ossos restritos 180°**. — [Spine Transform](https://esotericsoftware.com/spine-transform-constraints)
- *Spine 4.3 — Transform constraint generalizado:* propriedades **mapeiam entre tipos** (mover um osso pode
  rodar ou escalar outros, qualquer combinação), valores restritos podem ser **clampados** a uma faixa, origem e
  destino em mundo ou local. — [Spine 4.3](https://esotericsoftware.com/blog/Spine-4.3-released)
- *Spine 4.3 — Slider constraint (novo):* aplica uma **animação** e controla qualquer propriedade animável; o
  quadro é posto à mão ou **dirigido pela rotação/translação/escala/shear de um osso de controlo**; uma animação
  pode animar o slider ou o osso de controlo, reutilizando a animação sem copiar chaves. (É o equivalente Spine
  dos smart bones / drivers.) — [Spine 4.3](https://esotericsoftware.com/blog/Spine-4.3-released)
- *Spine 4.3 — outros:* IK e física ganham **scaleY que preserva volume** (scaleY cresce/encolhe ao contrário de
  scaleX — squash & stretch); «Key Constrained» grava em chaves o resultado depois das constraints (inclui
  física → bake); «improved constraint sorting» (sem detalhe). — [Spine 4.3](https://esotericsoftware.com/blog/Spine-4.3-released)
- *Path constraint:* **Position mode** Fixed (distância) / Percent (fração do comprimento); fora de um caminho
  aberto, extrapola em linha reta. **Spacing** Length (comprimento do osso anterior + espaçamento), Fixed,
  Percent, **Proportional** (a 100 os ossos enchem o caminho na proporção dos seus comprimentos). **Rotate**
  **Tangent** (aponta na tangente; em curvas fechadas a ponta sai do caminho), **Chain** (cada osso vai à
  ponta do anterior e aponta para a próxima posição — esteira de tanque), **Chain Scale** (aponta e escala para
  a ponta cair no caminho — corda; exige pai comum). **Rotate offset**, **Rotate mix** e **Translate mix**
  animáveis; a ordem dos ossos na lista é a ordem ao longo do caminho. — [Spine Path](https://esotericsoftware.com/spine-path-constraints)
- *Physics constraint (4.2):* criado com `New... > Physics Constraint` sobre o osso (não existe no Spine
  Essential). Canais de efeito 0–100: **Translate X/Y, Rotation, Shear X, Scale X** (o comprimento do osso
  influencia); **Limit** (velocidade máxima de translação a que a física responde); **FPS** (taxa de passo;
  doc sugere 30–60, baixo = saltitante e instável). Propriedades animáveis: **Inertia** (quanto do movimento
  vira atraso), **Strength** (mola de volta à pose não-restrita), **Damping**, **Mass**, **Wind** (força
  constante em X do mundo), **Gravity** (força constante em Y do mundo), **Mix**; **Global** (vários
  constraints num só trilho da timeline). Editor: **Simulate**, **Deterministic** (re-simula do quadro 0 a cada
  quadro para que arrastar a timeline rebobine — só no editor, não no runtime), **Reset / Reset All**,
  **Warm Up** no export para loops começarem assentados. Para parâmetros diferentes por aspecto, **vários
  physics constraints no mesmo osso**. Avisos: osso de comprimento zero com rotate/scaleX/shearX ligado. — [Spine Physics](https://esotericsoftware.com/spine-physics-constraints)
- *Spine 4.3 — física:* mais correta e responsiva em taxas baixas, «deterministic behavior»; runtimes ganham
  **vento e gravidade ao nível do esqueleto** e herança de posição/rotação da física. — [Spine 4.3](https://esotericsoftware.com/blog/Spine-4.3-released)

**Rive**

- *Tipos:* IK, Distance, Transform, Translation, Scale, Rotation, Follow Path, Scroll. Translation/Rotation/Scale
  **limitam e/ou copiam** a propriedade de um alvo, com cada propriedade ligada independentemente; Transform
  copia posição, rotação e escala **independente da hierarquia**. — [Rive overview](https://rive.app/docs/editor/constraints/constraints-overview); [Rive docs index](https://rive.app/docs/llms.txt)
- *IK:* **Bone Count** (quantos ossos acima na cadeia; destacados no palco), **Invert Direction** (inverte o
  ângulo de solução = direção da dobra), **Strength** animável (blend FK↔IK ou entre várias IK). Criação: `B`
  cria ossos, `G` cria grupo, o grupo recebe Style = **Target**; selecionar o último osso, Inspector →
  Constraints → IK; escolher o alvo no flyout. — [Rive IK](https://rive.app/docs/editor/constraints/ik-constraint)
- *Ordem:* constraints sobrepostos num osso — a 100 % o **de baixo sobrepõe o de cima**; com strength menor,
  mistura; **arrastar no Inspector reordena**. — [Rive IK](https://rive.app/docs/editor/constraints/ik-constraint)
- *Distance:* **Strength**, **Distance** (círculo vermelho no palco), **Mode** Closer (dentro; padrão) /
  Further (fora; empurra) / Exact (na distância exata). — [Rive Distance](https://rive.app/docs/editor/constraints/distance-constraint.md)
- *Follow Path:* **Distance** em % ao longo do caminho (pode passar de 100 %; o runtime C++ faz módulo positivo),
  **Orient** (rotação pela tangente; com ele ligado não se roda à mão), **Strength**, **Offset** (segue a partir
  da posição atual). — [Rive Follow Path](https://rive.app/docs/editor/constraints/follow-path-constraint); [rive-cpp follow_path_constraint.cpp](https://skia.googlesource.com/external/github.com/rive-app/rive-cpp/+/80c037c2d72c6d936244ebbae1ef6e757bb77f1e/src/constraints/follow_path_constraint.cpp)
- *Scroll constraint (UI):* Direction (Vertical/Horizontal/All), **Scroll Percent X/Y** e **Scroll Index**
  animáveis, **Physics** Elastic (estilo iOS, rubber band) / Clamped, **Snap**, Wheel/Trackpad, **Virtualize**,
  **Carousel** (infinito; exige Virtualize). Layouts (responsivos, N-Slicing, Layout Animation) ficam numa
  secção separada, não nas constraints. — [Rive Scroll](https://rive.app/docs/editor/constraints/scroll-constraint.md); [Rive docs index](https://rive.app/docs/llms.txt)

**Moho (Lost Marble)**

- *Angle constraints:* liga «Angle constraints» no popup **Bone Constraints** (com a ferramenta Select Bone) e
  define min/max; Manipulate Bones para nos limites — impede joelho/cotovelo a dobrar ao contrário. — [Moho Tutorial 3.2](https://www.lostmarble.com/moho/manual/tut03/02/)
- *Bone dynamics:* simulação de **mola** que move o osso sozinho (cabelo, carne). Parâmetros (descritos por
  utilizador do fórum, não pela doc): **Torque** (força no filho quando o pai mexe), **Spring** (força do
  vai-e-vem), **Damping** (0 = nunca para). — [Moho Tutorial 3.3](https://lostmarble.com/moho/manual/tut03/03/index.html); [fórum Lost Marble](https://lostmarble.com/forum/viewtopic.php?p=216398)
- *Queixas Moho 14:* osso dinâmico preso por target/lock e depois solto gera mola **maior que a esperada**;
  simulação erra do quadro 0 para o 1; script de bake de dinâmica parou na 14.1 porque a 14 não atualiza a
  dinâmica quando o script muda o quadro. **Target bones são liga/desliga, sem rampa** (pedido: easing como
  noutras constraints, ex. Independent Angle). — [fórum Lost Marble](https://lostmarble.com/forum/viewtopic.php?p=216398); [MR Bake Bone Dynamics](https://lostmarble.com/forum/viewtopic.php?p=217271); [fórum p=183307](https://lostmarble.com/forum/viewtopic.php?p=183307)
- *Smart bones:* um osso cuja rotação dirige uma **Action** (deformação/troca autorada); para limitar, faz-se o
  ângulo máximo do smart bone casar com a última chave da action. Moho 14.4 exporta **Smart Bones como morph
  targets em glTF**, junto com Dynamic Bones e Actions. — [fórum p=183305](https://lostmarble.com/forum/viewtopic.php?p=183305); [Digital Production, Moho 14.4](https://digitalproduction.com/2025/11/14/moho-14-4-goes-on-a-side-quest-2d-meets-game-engines/)
- *Pedido no fórum Spine:* fazer os ossos mexerem com física «para follow-through automático no fim de uma
  virada de cabeça» é o que torna o conceito smart-bone poderoso (opinião de utilizador). — [fórum Spine 25654](https://fr.esotericsoftware.com/forum/d/25654-request-new-timeline-constraint-linking-art-and-animations-to-bone-positions/18)

**Toon Boom Harmony (Premium 22)**

- *Família de nós Constraint:* **Two-Point** (prende o desenho entre os pivots de duas pegs; estica/espreme
  automaticamente; **Distance Min/Max** em % da distância de repouso, além disso pára; **Volume Min/Max**
  controla a largura ao esticar; **Skew Modifier** escolhe rodar vs. cisalhar), **Three-Point**, **Multi-Point**
  (com quadmap/perspetiva opcional), **Dynamic Spring** (passa o movimento do pai ao filho com atraso de mola;
  fatores de inércia e tensão), **Transform Limit** (escala cada tipo de transformação 0–100 % antes de passar ao
  filho), **Transformation-Gate** (escolhe um de vários pais ou interpola entre dois), **Constraint Switch**
  (liga-se à 1.ª entrada de outro nó de constraint para temperar 0–100 % e escolher a porta ativa do Gate),
  **Static Constraint** (assa a transformação de um pai como pose de repouso para outro elemento). — [Harmony 22 Constraint nodes](https://docs.toonboom.com/help/harmony-22/premium/reference/node/constraint/constraint-node.html); [Harmony 22 Two-Points](https://docs.toonboom.com/help/harmony-22/premium/reference/node/constraint/two-point-constraint-node.html)
- O grupo serve para «limitar, alternar, ligar/desligar ou interpolar transformações aplicadas por pegs». —
  [Toon Boom Learn](https://learn.toonboom.com/modules/adding-constraints-to-an-rig/topic/about-constraints-nodes)
- **Kinematic Output** e **Point Kinematic Output** existem como entradas no Reference Guide (a segunda com
  «sampling points» sobre o deformador). — [Harmony 22 Reference PDF](https://docs-stg.toonboom.com/download/harmony/22.0/premium/Harmony_22_Premium_Reference_Guide.pdf); [Point Kinematic Output](https://docs.toonboom.com/help/harmony-20/premium/deformation/about-point-kinematic-output-sampling-points.html)

**Live2D Cubism**

- *Deformadores:* contentor que move vértices de ArtMeshes juntos; **Warp** (curvas, grelha bezier) e
  **Rotation** (roda e escala por ângulo); aninhados pai-filho («Sequential Creation»); «Validate Deformer»
  verifica que filhos não saem do warp pai. — [Live2D Glossary](https://docs.live2d.com/en/cubism-editor-manual/glossary/); [About Deformers](https://docs.live2d.com/en/cubism-editor-manual/deformer/); [Modeling Menu](https://docs.live2d.com/en/cubism-editor-manual/modeling-menu)
- *Glue:* cola vértices de **duas** ArtMeshes (três ao mesmo tempo não). Queixa de utilizador: o glue é
  **linear** de A para B; pedido de colar deformadores para seguir um caminho (não existe). — [Toolbar](https://docs.live2d.com/4.2/en/cubism-editor-manual/toolbar/); [community.live2d.com](https://community.live2d.com/discussion/comment/2756)
- *Física:* **Input** = parâmetros que são o ponto de suspensão do pêndulo; **Physical model** = peso e reação
  (pêndulo **multi-estágio**, presets ex. 10 estágios); **Output** = escreve o balanço de volta em parâmetros (o
  número de saídas deve casar com os estágios); 60 fps por padrão, mudar altera o resultado. No SDK o
  `.physics3.json` é lido e **gravidade/vento são sobrescritos pelas opções do runtime**. — [About Physics](https://docs.live2d.com/en/cubism-editor-manual/physics-operation/); [How to Set Up Physics](https://docs.live2d.com/en/cubism-editor-manual/physical-operation-setting/); [SDK physics](https://docs.live2d.com/en/cubism-sdk-manual/physics)

**OpenToonz — Plastic tool**

- Esqueleto de referência + **malha triangular** gerada dos contornos do desenho exposto, numa coluna Mesh
  ligada à coluna de origem (deformada e recortada pela malha no render). Cada **vértice do esqueleto** tem
  posição, **limites de ângulo**, **rigidez** e **ordem de empilhamento (SO)**. Densidade da malha decide a
  suavidade das curvas ao rodar. — [OpenToonz manual, Plastic](https://opentoonz.readthedocs.io/en/latest/create_animations_using_plastic_tool.html)

### Inferences
- Os três parâmetros que todo sistema maduro partilha: **peso/mix animável**, **espaço (local/mundo)** e
  **ordem explícita**. Spine e Rive expõem os três por constraint; Harmony fá-lo por fios no grafo.
- Spine 4.3 convergiu para o modelo «constraint genérico de mapeamento + clamp» (from/to entre tipos) e
  «slider dirigido por osso» — dois primitivos que cobrem a maioria dos drivers sem expressões.

### Gaps
- Moho: não encontrei a página oficial atual de Smart Bones, Target bones, Independent Angle/Scale, Follow Path
  nem da física de corpo rígido do Moho; parâmetros de dynamics só por fórum.
- Harmony: parâmetros de Kinematic Output, IK e Auto-Patch e Master Controllers não foram lidos.
- Rive: parâmetros de Transform/Translation/Rotation/Scale (origem, min/max, espaço local/mundo) não lidos;
  interação com State Machine não pesquisada.
- DragonBones, Creature (Kestrel Moon), Adobe Animate (bone tool/pin), Synfig (Skeleton layer, link/exported
  values) e Grease Pencil: **sem pesquisa nesta passagem** (orçamento).
- Live2D: parâmetros numéricos do pêndulo (comprimento, resposta, atraso, aceleração) não lidos.
- Ordem de avaliação Spine (a lista «Constraints» na árvore com ordem arrastável) não confirmada nas páginas
  lidas.

## Q2 — Ideias 2D que faltam nas ferramentas 3D e valem a adoção

### Takeaway
As mais valiosas para o PH2D: (1) **física como constraint sobre canais de osso** (Spine) com
inércia/força/amortecimento/massa/vento/gravidade e um modo **determinístico** para scrub; (2) **osso que
dirige uma animação/forma** (Moho Smart Bones, Spine 4.3 Slider); (3) **física de parâmetro para parâmetro**
(Live2D) — não exige osso; (4) **Two-Point com limites de distância e preservação de volume** (Harmony) para
squash & stretch de membros; (5) **portão/switch de pais** como nó (Harmony Transformation-Gate).

### Cited Findings
- Spine Physics: canais por osso (Translate X/Y, Rotation, Shear X, Scale X), Inertia, Strength, Damping, Mass,
  Wind, Gravity, Limit, FPS, Deterministic, Warm Up. — [Spine Physics](https://esotericsoftware.com/spine-physics-constraints)
- Spine 4.3: scaleY que preserva volume em IK/física; Slider dirigido por osso; Key Constrained (bake). — [Spine 4.3](https://esotericsoftware.com/blog/Spine-4.3-released)
- Moho Smart Bones exportados como morph targets glTF em 14.4 — prova de que «osso → forma» é um conceito
  portável. — [Digital Production](https://digitalproduction.com/2025/11/14/moho-14-4-goes-on-a-side-quest-2d-meets-game-engines/)
- Live2D: pêndulo multi-estágio entre parâmetros, vento/gravidade sobrescritos no runtime. — [About Physics](https://docs.live2d.com/en/cubism-editor-manual/physics-operation/); [SDK physics](https://docs.live2d.com/en/cubism-sdk-manual/physics)
- Harmony Two-Point: Distance Min/Max, Volume Min/Max, Skew Modifier; Transformation-Gate; Dynamic Spring. — [Harmony 22](https://docs.toonboom.com/help/harmony-22/premium/reference/node/constraint/constraint-node.html); [Two-Points](https://docs.toonboom.com/help/harmony-22/premium/reference/node/constraint/two-point-constraint-node.html)
- OpenToonz Plastic: **ordem de empilhamento por vértice do esqueleto** — draw order como propriedade do rig. — [OpenToonz Plastic](https://opentoonz.readthedocs.io/en/latest/create_animations_using_plastic_tool.html)
- Problema 2D específico: espelhar com escala negativa gira 180° em Transform relativo de mundo (Spine), e a
  dobra da IK é um **booleano de sentido** (Positive / Invert Direction), não um pole target 3D. — [Spine Transform](https://esotericsoftware.com/spine-transform-constraints); [Spine IK](https://esotericsoftware.com/spine-ik-constraints); [Rive IK](https://rive.app/docs/editor/constraints/ik-constraint)
- Rive Scroll constraint (física elástica, snap, carrossel) — constraint de UI que não existe em DCC 3D. — [Rive Scroll](https://rive.app/docs/editor/constraints/scroll-constraint.md)

### Inferences
- Em 2D, a dobra da IK reduz-se a um **bit de sinal**; ao espelhar um personagem (escala X negativa) esse bit
  tem de inverter junto, senão o joelho dobra para o lado errado — o PH2D deve tratar o sinal como relativo ao
  determinante da transformação do pai (inferência; nenhuma fonte lida o descreve assim).
- A troca de desenho por ângulo (sprite/drawing swap) é um caso particular do «Slider dirigido por osso» com
  uma trilha de troca de imagem — no PH2D, o Flip poderia ser o alvo desse slider.
- O «Deterministic» do Spine (re-simular do quadro 0) é a resposta a um problema que o PH2D já conhece no
  Motion (scrub ≠ playback); um cache de estados por quadro seria a versão barata.
- O Constraint Switch / Transformation-Gate do Harmony casa naturalmente com o ECS do PH2D (componente de peso
  e de porta ativa, sem sistemas a se chamarem).

### Gaps
- Não encontrei fonte primária descrevendo como Spine/Moho/Rive tratam a dobra da IK sob flip (só a nota de
  180° do Transform do Spine).
- Troca de desenho por ângulo: não confirmada em nenhuma doc lida (Moho «Switch layers» + smart bones é
  conhecido, mas não verifiquei nesta passagem).
- Draw-order constraints de Spine (Draw order timeline / skins) não pesquisados.

## Q3 — O que torna as constraints de Spine/Rive fáceis de usar

### Takeaway
Criação **em contexto no palco** (seleciona-se o osso, `New... > tipo`, clica-se no alvo — ou cria-se o alvo
clicando no vazio), **um slider de mix por constraint**, avisos visíveis no ícone, ordem **arrastável** e
gizmos no palco (círculo vermelho da Distance, ossos destacados do Bone Count).

### Cited Findings
- Spine: o clique no vazio cria o alvo; clicar no osso restrito cria o alvo na ponta. — [Spine IK](https://esotericsoftware.com/spine-ik-constraints)
- Spine: «Match» copia a pose atual para os offsets (sem conta à mão). — [Spine Transform](https://esotericsoftware.com/spine-transform-constraints)
- Spine: aviso no ícone da árvore quando o constraint não faz nada (canais a 0, osso de comprimento 0, fora da
  skin). — [Spine Physics](https://esotericsoftware.com/spine-physics-constraints)
- Spine: Link sliders para mexer todos os mixes juntos. — [Spine Transform](https://esotericsoftware.com/spine-transform-constraints); [Spine Path](https://esotericsoftware.com/spine-path-constraints)
- Rive: constraints vivem no Inspector do objeto; ordem por arrastar; a regra «o de baixo vence a 100 %, abaixo
  disso mistura» é uma frase. — [Rive IK](https://rive.app/docs/editor/constraints/ik-constraint)
- Rive: Distance mostra o raio no palco. — [Rive Distance](https://rive.app/docs/editor/constraints/distance-constraint.md)
- Contra-exemplo (Moho): target bones sem rampa — liga/desliga brusco é queixa recorrente. — [fórum Lost Marble](https://lostmarble.com/forum/viewtopic.php?p=183307)

### Inferences
- O peso animável em **toda** constraint é o que dá a transição FK↔IK sem truques; Moho, que não o tem em todas,
  recebe a queixa. O PH2D deve tornar o mix animável obrigatório (e gravável pelo Auto Key, lei do dono).
- A queixa de «flip» súbito no mix intermédio (Spine) sugere interpolar o resultado no espaço do ângulo
  desembrulhado, não pelo caminho mais curto.

### Gaps
- Não recolhi citações de animadores (fóruns/YouTube) a elogiar explicitamente a UX de Spine/Rive.

## Q4 — Licenças das implementações abertas

### Takeaway
**Só OpenToonz é porta permissiva** desta lista (BSD-3, segundo o CLAUDE.md do projeto). **Spine Runtimes
License não é permissiva**: exige licença do Spine Editor para cada utilizador do produto — não ler nem portar.
Synfig GPL (só docs). Rive e DragonBones: não verificado nesta passagem.

### Cited Findings
- Spine Runtimes License (atualizada 05/04/2025): integração e obras derivadas permitidas **desde que cada
  utilizador dos produtos tenha a sua própria licença do Spine Editor**; redistribuição leva a licença e o aviso
  de copyright. — [Spine Runtimes License](https://esotericsoftware.com/spine-runtimes-license)
- OpenToonz BSD-3 listado como porta permissiva aberta (junto de Godot MIT e libmypaint ISC) na regra 9 do
  CLAUDE.md do PH2D (fonte interna; verificar `pacman -Qi`/LICENSE do artefacto antes de portar).

### Inferences
- Para o PH2D, Spine e Live2D (SDK proprietário) são **oráculos a correr**, nunca fontes a ler (regra 9). O
  Plastic do OpenToonz (malha + esqueleto + SO por vértice + limites de ângulo + rigidez) é o candidato a
  leitura de código.
- O código C++ do runtime do Rive foi citado acima só pelo comportamento (módulo da distância); a licença dele
  tem de ser triada antes de qualquer leitura de código.

### Gaps
- Licença do runtime Rive (`rive-runtime`), do DragonBones e do Creature runtime: não verificadas.
- Licença do OpenToonz não confirmada por fonte externa nesta passagem (a busca não devolveu o LICENSE).
- Synfig GPL: afirmado no briefing, não verificado.
