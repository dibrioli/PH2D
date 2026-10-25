# Constraints geométricas/paramétricas em CAD e sketchers 2D — insumo para o PH2D

> Pesquisa de 2026-10-25. As fontes primárias que consegui ler foram: a referência do SolveSpace, a ajuda do Onshape,
> o blog e a doc da Zoo (ezpz), o README/repositório do Fiksi, o cabeçalho do `GCS.cpp` do FreeCAD e o README do port
> `@salusoft89/planegcs`. O wiki do FreeCAD (wiki.freecad.org) **recusou o acesso** (bot-protection Anubis) e o
> DeepWiki do FreeCAD deu 429, por isso o catálogo do FreeCAD Sketcher saiu menos verificado do que devia. Fusion,
> SolidWorks, Shapr3D, NX/D-Cubed, GeoGebra, Cinderella, Rhino/Grasshopper e Dynamo **não foram verificados nesta
> sessão**: o que sei deles fica nos *Gaps*, marcado como conhecimento prévio sem fonte.

## 1. Que constraints e que recursos de UX tornam o sketch de CAD rápido e agradável?

### Takeaway
O núcleo é o mesmo em todo o lado: coincidente / ponto-na-curva, horizontal/vertical, paralelo, perpendicular, tangente, igual, simétrico, ponto-médio, fixar/travar, mais as dimensões (distância, ângulo, raio/diâmetro). O que dá velocidade é a UX à volta delas: **inferência automática enquanto se desenha** (que recusa criar uma constraint redundante), **um só comando que escolhe a constraint conforme a seleção**, a **dimensão que vira REF sozinha** quando a nova sobre-restringiria, **cores de estado** (azul = livre, preto/branco = totalmente definido, vermelho = conflito) e um **diagnóstico que lista as constraints que se podem apagar**.

### Cited Findings

**SolveSpace — catálogo e como se aplica**
- Fluxo: primeiro seleciona-se a geometria, depois escolhe-se a constraint no menu **Constrain**. O mesmo item gera constraints diferentes conforme a seleção (Distance/Diameter dá diâmetro num círculo e comprimento numa linha). Se a seleção não casa, aparece um erro com a lista de opções válidas. Na maioria dos casos a ordem de seleção não importa. — [SolveSpace ref](https://solvespace.com/ref.pl)
- **Distance/Diameter**: diâmetro de círculo ou arco, comprimento de linha, ou distância de um ponto a uma entidade. As distâncias ponto-plano e ponto-linha são **com sinal**: o ecrã mostra sempre um valor positivo, e para trocar de lado digita-se um valor negativo. — [SolveSpace ref](https://solvespace.com/ref.pl)
- **Angle**: 0–180°, com o sinal ignorado. O solver escolhe arbitrariamente um dos dois ângulos suplementares e desenha um arco para indicar qual; **Other Supplementary Angle** troca. — [SolveSpace ref](https://solvespace.com/ref.pl)
- **Horizontal/Vertical** (de uma linha ou entre dois pontos): exige um workplane ativo. A doc recomenda usá-las sempre que possível, porque são as mais simples de resolver. — [SolveSpace ref](https://solvespace.com/ref.pl)
- **On Point / Curve / Plane**: ponto coincidente, ponto na curva ou ponto no plano. Restringir em 3D dois pontos que já estão no plano é redundante e dá erro. — [SolveSpace ref](https://solvespace.com/ref.pl)
- **Equal Length / Radius / Angle**: o equal-angle recebe 4 vetores e compara ∠(A,B) com ∠(C,D). Uma linha com um arco iguala ao **comprimento** do arco, não ao raio. — [SolveSpace ref](https://solvespace.com/ref.pl)
- **Length Ratio**: a ordem de seleção importa (a 1.ª linha é o numerador). **Length Difference**: um valor negativo escolhe qual das linhas é a mais curta. — [SolveSpace ref](https://solvespace.com/ref.pl)
- **At Midpoint**; **Symmetric** em relação a um plano ou a uma linha do workplane; se não se indica o eixo, o solver infere o eixo H ou V conforme o desenho. — [SolveSpace ref](https://solvespace.com/ref.pl)
- **Perpendicular** é igual a um Angle de 90°. **Parallel/Tangent**: a tangência só vale onde as curvas já partilham um endpoint (normalmente põe-se primeiro um coincidente). — [SolveSpace ref](https://solvespace.com/ref.pl)
- **Same Orientation** (normais, para assemblies). **Lock Point Where Dragged**: o solver trata o ponto como totalmente restrito, mas o usuário ainda o pode arrastar diretamente. **Comment**: texto sem efeito na geometria. — [SolveSpace ref](https://solvespace.com/ref.pl)
- **Driving vs reference**: por omissão a dimensão é *driving*. **Toggle Reference Dimension** transforma-a em "REF": o valor passa a seguir a geometria, e um duplo-clique nela não faz nada. — [SolveSpace ref](https://solvespace.com/ref.pl)
- **Auto-constraints**: com a opção ligada, uma linha quase horizontal mostra a constraint H sobre o meio e uma quase vertical mostra "V"; a constraint é aplicada no 2.º clique. **Ctrl** enquanto se desenha desativa. A auto-constraint **não é criada se for redundante ou sobre-restringir**, mesmo que tenha aparecido no preview. Uma dimensão nova que sobre-restringiria é criada como **REF**. A opção "edit newly added dimensions" abre logo a caixa de valor. — [SolveSpace ref](https://solvespace.com/ref.pl)
- Atalhos que a página cita: `g` (alterna construção), `W`, `Tab`, `Esc`. A página **não** lista atalhos por constraint. — [SolveSpace ref](https://solvespace.com/ref.pl)

**Onshape**
- Constraints da barra: Coincident, Concentric, Parallel, Tangent, Horizontal, Vertical, Perpendicular, Equal (várias linhas ou vários arcos de uma vez), Midpoint (de linha **ou arco**), Normal (curva⊥linha/plano), Pierce, Symmetric (em relação a uma linha, plano ou aresta), Fix, **Curvature** (continuidade de curvatura entre splines/cónicas e a geometria vizinha, ou seja G2; a página não usa o termo "G2"). — [Onshape help](https://cad.onshape.com/help/Content/constraints.htm)
- Constraints que só nascem automaticamente: Quadrant (elipse), Use (liga a outra sketch/aresta, com fundo azul), Intersection. — [Onshape help](https://cad.onshape.com/help/Content/constraints.htm)
- Aplicação nos **dois sentidos**: ferramenta primeiro e depois as entidades, ou as entidades primeiro e depois a constraint. Há também um modo em que a constraint fica ligada e cada par selecionado a recebe. Pierce = `Shift+G`. — [Onshape help](https://cad.onshape.com/help/Content/constraints.htm); [busca: Onshape Pierce](https://cad.onshape.com/help/Content/Sketch/pierce.htm)
- Inferência: os indicadores da constraint aparecem junto ao cursor quando a curva "snapa". — [Onshape help](https://cad.onshape.com/help/Content/constraints.htm)
- **Cores**: entidade azul = sub-restrita, preta = totalmente restrita, vermelha = problema (sobre-restrita). Ícone de constraint preto sobre cinza = ok; branco sobre vermelho = conflito. O filtro **Errors** do Constraint manager destaca os conflitos e o filtro **Driven** pinta as dimensões driven de amarelo. Para resolver, apaga-se a constraint. — [Onshape help](https://cad.onshape.com/help/Content/constraints.htm)
- **Prioridade de seleção**: com pontos sobrepostos, os não totalmente restritos (azuis ou vermelhos) são escolhidos antes dos pretos. É um detalhe de UX que poupa muitos cliques. — [Onshape help](https://cad.onshape.com/help/Content/constraints.htm)
- Hover mostra as constraints da entidade; **Shift** mantém todas visíveis; "Show constraints" mostra todas. Os ícones podem ser arrastados. — [Onshape help](https://cad.onshape.com/help/Content/constraints.htm)

**Zoo Design Studio (ezpz)**
- Parallel (`B`), Vertical (`V`), Perpendicular, dimensão de comprimento (`D`), ancorar um vértice na origem, equal length. Geometria de construção. A geometria desenhada à mão "snapa para o lugar final" à medida que se acrescentam constraints. Cores: **azul = sub-restrito, branco = totalmente restrito**. — [Zoo blog](https://zoo.dev/blog/announcing-solver)

**FreeCAD (só pelo port planegcs)**
- Primitivas: Point, Line, Circle, Arc, Ellipse, arco de elipse, Hyperbola, Parabola (e arcos destas); B-Spline incompleto no port. As constraints têm as propriedades opcionais `driving`, `temporary` e `scale`, e os valores podem referir parâmetros da sketch. — [planegcs README](https://cdn.jsdelivr.net/npm/@salusoft89/planegcs@1.1.7/README.md)

### Inferences
- A combinação "um comando por intenção que se resolve pela seleção" (SolveSpace) com "a ordem ferramenta↔seleção é livre" (Onshape) é a que exige menos memorização. Para o PH2D, que já tem `Tab`/modos, encaixa bem como menu contextual sobre a seleção.
- As regras "auto-constraint redundante não nasce" e "dimensão que sobre-restringiria nasce REF" evitam que o usuário entre em estado vermelho por acidente. São baratas se o solver já sabe calcular o posto (rank) do Jacobiano.
- A prioridade de seleção para pontos não totalmente definidos é uma boa ideia para um editor vetorial com pontos sobrepostos (endpoints de paths).

### Gaps
- Não consegui ler a **lista do FreeCAD Sketcher** com atalhos (wiki bloqueado). Conhecimento prévio, **não verificado**: Coincident `C`, Point-on-object `O`, Horizontal `H`, Vertical `V`, Parallel `P`, Perpendicular `N`, Tangent `T`, Equal `E`, Symmetric `S`, Block, Lock `K,L`, distâncias `L`/`I`, Radius/Diameter, Angle `A`, Refraction (Snell); constraint "Block"; cores branca/verde para totalmente restrito; mensagens "redundant/conflicting/partially redundant/malformed" no painel do solver.
- **Fusion, SolidWorks, Shapr3D, Siemens NX/D-Cubed DCM**: não pesquisados nesta sessão. Conhecimento prévio sem fonte: o SolidWorks tem Relations (Coincident, Merge, Pierce, ...), "Fully Define Sketch" automático e driven dimensions em cinza; o Fusion tem Fix/Coincident/Collinear/Concentric/Midpoint/Smooth (G2)/Symmetric/Curvature e "Sketch Fully Constrained" (linhas pretas); o Shapr3D tem constraints em parte inferidas por gesto; o D-Cubed 2D DCM (Siemens, proprietário) está por trás de muitos CADs comerciais.
- **GeoGebra / Cinderella**: não pesquisados. São geometria dinâmica *construtiva* (por construção, não por um solver de restrições); o Cinderella é conhecido por tratar a continuidade (evitar saltos) com métodos complexos. Sem fonte nesta sessão.
- **Pattern, block/rigid set, expressions/variables**: só o SolveSpace (Step groups) e o planegcs (parâmetros referenciáveis) aparecem nas fontes lidas; nenhum detalhe de UX de pattern em 2D foi verificado.

## 2. Como os solvers se comportam interativamente (drag) e o que falha?

### Takeaway
O drag resolve-se de novo a cada movimento. O SolveSpace e o planegcs tratam o ponto arrastado como uma constraint **"temporária/suave"**: é cumprida até onde não conflita com as outras e **não consome graus de liberdade**. No FreeCAD isto liga o **SQP**. A diagnose de redundância e conflito é feita pelo **posto (QR) do Jacobiano**. As falhas típicas são a escolha arbitrária de um ramo (ângulo suplementar, lado de uma distância com sinal), a não-convergência em geometria impossível, e diferenças entre algoritmos (DogLeg falha onde o LM converge, e vice-versa).

### Cited Findings
- SolveSpace: arrastar um ponto re-resolve a sketch e as constraints decidem como o resto se move. Um ponto travado (Lock) não é movido pelo solver, mas pode ser arrastado; no exemplo da doc, arrastar o lado oposto a um vértice travado redimensiona o triângulo. — [SolveSpace ref](https://solvespace.com/ref.pl)
- SolveSpace — **Show Degrees of Freedom**: marca com um quadrado ciano grande os pontos não totalmente restritos (os que se moveriam num drag). 1 DOF pode marcar vários pontos. As marcas limpam-se com Regenerate All ou com um drag. — [SolveSpace ref](https://solvespace.com/ref.pl)
- SolveSpace — **inconsistente ou redundante**: o fundo fica **vermelho** e o Property Browser mostra uma lista de constraints cuja remoção restaura a consistência; passar o cursor sobre cada link destaca a constraint. Exemplo de inconsistente: ângulos de 30/50/90; exemplo de redundante: 30/50/100. — [SolveSpace ref](https://solvespace.com/ref.pl)
- SolveSpace — **não-convergência** sem conflito (por exemplo um triângulo de lados 3, 4, 10): erro **sem** lista de remoção; a saída é editar ou desfazer. — [SolveSpace ref](https://solvespace.com/ref.pl)
- SolveSpace — o **ramo** do ângulo é escolhido arbitrariamente entre os dois suplementares, e as distâncias com sinal escolhem o lado pelo sinal. Estas são as fontes de "flip". — [SolveSpace ref](https://solvespace.com/ref.pl)
- planegcs — constraints `temporary` são próprias para o drag do mouse: "cumpridas só até onde não conflitam" e "não reduzem DOF". A presença delas troca o algoritmo para **SQP**, seja qual for o configurado. — [planegcs README](https://cdn.jsdelivr.net/npm/@salusoft89/planegcs@1.1.7/README.md)
- planegcs — algoritmos: **DogLeg (padrão)**, Levenberg-Marquardt, BFGS e SQP. — [planegcs README](https://cdn.jsdelivr.net/npm/@salusoft89/planegcs@1.1.7/README.md). No código-fonte, o enum é `BFGS=0, LevenbergMarquardt=1, DogLeg=2`, e o SQP é a via com subsistema aumentado. — [busca: GCS.h](https://free-cad.sourceforge.net/SrcDocu/d1/de7/GCS_8h_source.html); [commit SparseQR](https://gitea.suzanne.soy/suzanne.soy/FreeCAD/commit/796c9d79d4fab1f4957fa228b2a513fb39af1abe)
- planegcs (`GCS.cpp`): o `System` mantém `redundant`, `conflictingTags`, `redundantTags` e `partiallyRedundantTags`. `initSolution()` chama `diagnose()` e **exclui as redundantes** da resolução. Depois de resolver, verifica se alguma redundante ficou violada. A diagnose usa QR, por omissão **EigenSparseQR** com `autoChooseAlgorithm` e `autoQRThreshold=1000`; o passo de Gauss do DogLeg usa `FullPivLU`. Existe também uma análise de **parâmetros dependentes** (`pDependentParameters`, `pDependentParametersGroups`). — [FreeCAD GCS.cpp](https://raw.githubusercontent.com/FreeCAD/FreeCAD/main/src/Mod/Sketcher/App/planegcs/GCS.cpp)
- Falhas conhecidas no FreeCAD: casos em que o DogLeg falha e o LM resolve; o LM reportou erradamente uma sketch como totalmente restrita onde o DogLeg acertava; uma redundância não foi reconhecida. — [tracker 1865](https://tracker.freecad.org/view.php?id=1865); [tracker 3286](https://tracker.freecad.org/view.php?id=3286); [tracker 3773](https://tracker.freecad.org/view.php?id=3773)
- No planegcs, constraints não-driving (de medição) **não funcionam bem** em vários tipos: CircleDiameter, ArcDiameter, C2CDistance, C2LDistance, P2CDistance, ArcLength. — [planegcs README](https://cdn.jsdelivr.net/npm/@salusoft89/planegcs@1.1.7/README.md)
- Um paper de 2023 compara BFGS, LM e DogLeg no PlaneGCS e acrescenta o DFP. — [Tehnički Vjesnik 2023](https://doaj.org/article/2c76a82e7e4a4783aa7303095e78d33a)
- Zoo/ezpz: numa sketch sub-restrita os pontos arrastam-se e a forma move-se; com as constraints travadas, o drag mantém as relações; totalmente restrita, nada se move. — [Zoo blog](https://zoo.dev/blog/announcing-solver)
- ezpz: `LineTangentToCircle` é **direcional**: a ordem dos argumentos muda a solução (escolhe o ramo). — [docs.rs kcl-ezpz Constraint](https://docs.rs/kcl-ezpz/latest/kcl_ezpz/enum.Constraint.html)
- Escala: o ezpz resolveu um problema de 2000 linhas × 2000 variáveis em **2 iterações, ~2,9 ms** (média de 100 corridas). — [ezpz README](https://raw.githubusercontent.com/KittyCAD/ezpz/main/README.md)
- Pesquisa acadêmica: há um paper da CWI que revisita os solvers por relaxação para sketching interativo, incluindo os casos sub e sobre-restritos. — [CWI](https://ir.cwi.nl/pub/18239/18239B.pdf)

### Inferences
- O padrão "drag = alvo suave, de menor prioridade que as constraints duras, resolvido por mínimos quadrados/SQP a partir da pose anterior" é o que dá o "movimento mínimo". Partir sempre da solução anterior (warm start) é o que evita os saltos de ramo. Para o PH2D, isso casa com "preview ≠ documento": o drag escreve no preview, e o commit grava a pose resolvida.
- Os ramos (lado, ângulo suplementar, tangente interna/externa) devem ser **gravados com a constraint** (sinal ou flag), como fazem o SolveSpace (sinal) e o ezpz (ordem dos argumentos), para que um re-solve não faça a figura saltar.
- A diagnose por QR/posto do Jacobiano dá ao mesmo tempo o contador de DOF (n_params − rank), a lista de redundantes/conflitantes e a regra "auto-constraint redundante não nasce".

### Gaps
- Não encontrei uma descrição primária do algoritmo do **SolveSpace** (conhecimento prévio sem fonte: Newton com Jacobiano simbólico, mínimos quadrados sobre o sistema sub-determinado, e as dimensões arrastadas resolvidas primeiro). Também não li o `diagnose()` completo do FreeCAD (o ficheiro foi truncado aos 100 k caracteres).
- Nenhuma medição de escala para o SolveSpace ou o planegcs com milhares de entidades foi encontrada.
- O algoritmo do ezpz não está descrito no blog nem no README (prometeram um post técnico, que não achei). Há branches com nomes "sparse-svd" no CodSpeed, o que sugere SVD esparsa, mas isso é só inferência pelo nome. — [CodSpeed ezpz](https://app.codspeed.io/KittyCAD/ezpz/branches/marcusklaas%3Asparse-svd)

## 3. Que bibliotecas/algoritmos uma engine Rust 2D pode reaproveitar, e sob que licença?

### Takeaway
Para o PH2D, que só porta código permissivo, há **dois candidatos Rust permissivos**: **ezpz / `kcl-ezpz` (KittyCAD/Zoo, MIT)**, usado em produção no Zoo Design Studio, e **Fiksi (endoli, MIT OR Apache-2.0)**, do ecossistema Linebender (canal #kurbo, a mesma família do kurbo/Vello que o PH2D já usa). O **SolveSpace/libslvs (GPL-3+)** e o **FreeCAD planegcs (LGPL-2.1-or-later)** só servem como **oráculos** que se correm, nunca como fonte.

### Cited Findings
- **ezpz**: MIT, Rust, nativo e WASM. Foi construído porque os solvers existentes eram "caros ou tinham licença restritiva". — [Zoo blog](https://zoo.dev/blog/announcing-solver)
  - Crates: `ezpz` (núcleo), `ezpz-cli` (resolve, analisa, visualiza e faz benchmark de ficheiros de problema) e `ezpz-wasm`. Tem um formato de texto com as secções `# constraints` e `# guesses` (`p roughly (3, 4)`), útil como **fixture**. — [ezpz README](https://raw.githubusercontent.com/KittyCAD/ezpz/main/README.md)
  - Publicado como `kcl-ezpz` (0.2.15, fim de dez/2025). — [rust-digger](https://rust-digger.code-maven.com/crates/kcl-ezpz)
  - 24 variantes de `Constraint` (`#[non_exhaustive]`): LineTangentToCircle, CircleTangentToCircle, Distance, DistanceVar, Vertical/HorizontalDistance, Vertical, Horizontal, LinesAtAngle, Fixed, ScalarEqual, PointsCoincident, CircleRadius, LinesEqualLength, ArcRadius, Arc, Midpoint, PointLineDistance (+ vertical/horizontal), Symmetric, PointArcCoincident, ArcLength, ArcAngle. Construtores compostos: `lines_parallel`, `lines_perpendicular`, `point_bisects_arc`, `parallel_lines_distance`, `circle_arc_coincident`. A doc das variantes Vertical/Horizontal parece trocada (Vertical diz "mesmo Y"). — [docs.rs kcl-ezpz](https://docs.rs/kcl-ezpz/latest/kcl_ezpz/enum.Constraint.html)
- **Fiksi**: "A geometric and parametric constraint solver", dual **Apache-2.0 OR MIT**, MSRV 1.85, discussão no Zulip Linebender #kurbo. O repositório tem `fiksi`, `fiksi_svg`, `solvi` e `colamd_rs` (ordenação COLAMD para fatoração esparsa); 171 commits, 14 estrelas, sem releases no GitHub. — [GitHub fiksi](https://github.com/endoli/fiksi); [README](https://raw.githubusercontent.com/endoli/fiksi/main/README.md)
- **SolveSpace**: "GPL v3 or later". — [SolveSpace README](https://raw.githubusercontent.com/solvespace/solvespace/master/README.md). Há o wrapper Rust `slvs` (binding da lib C++, não é Rust puro). — [docs.rs slvs](https://docs.rs/crate/slvs/0.0.0)
- **FreeCAD planegcs**: `SPDX-License-Identifier: LGPL-2.1-or-later`, © 2011 Konstantinos Poulios. — [GCS.cpp](https://raw.githubusercontent.com/FreeCAD/FreeCAD/main/src/Mod/Sketcher/App/planegcs/GCS.cpp). Tem um port WASM/JS em `@salusoft89/planegcs`; a licença do port não aparece no README. — [planegcs README](https://cdn.jsdelivr.net/npm/@salusoft89/planegcs@1.1.7/README.md)
- Outros resultados no crates.io: `Cadrs` (diz ter um solver de constraints, mas os builds no docs.rs falham e a licença não foi vista), `oxiDRAFT`. Não foram avaliados. — [docs.rs Cadrs](https://docs.rs/crate/Cadrs/0.1.3); [oxiDRAFT](https://github.com/fcoltro/oxiDRAFT)

### Inferences
- Pela regra §0.9 do PH2D (oráculo que se corre): o SolveSpace e o FreeCAD correm-se por script sobre as nossas sketches, e a saída grava-se como fixture. O formato de texto do `ezpz-cli` é uma boa ponte para isso.
- Fiksi tem a vantagem de vir da família kurbo/Linebender; o ezpz tem a vantagem de ser usado em produção e ter benchmark público (CodSpeed). Os dois merecem uma medição A×B no MESMO processo antes de se escolher, e é preciso fazer a triagem de licença por artefacto (o Cargo.toml/LICENSE de cada versão, incluindo as dependências: o ezpz parece usar álgebra esparsa; ver qual).
- Reimplementar é viável: o algoritmo padrão (resíduos por constraint + Jacobiano analítico + Gauss-Newton/LM/DogLeg esparso + QR com pivô para o posto) está descrito publicamente e não depende de copiar código GPL.

### Gaps
- O algoritmo, a cobertura de DOF/diagnose e a estabilidade de API do Fiksi não foram lidos (o docs.rs deu 404 na URL que tentei).
- A licença do libslvs separado e a dos bindings Python (`python-solvespace`) não foi verificada nesta sessão.
- D-Cubed DCM (Siemens) é proprietário e licenciado comercialmente (conhecimento prévio sem fonte nesta sessão).
- A licença das dependências do ezpz (por exemplo uma lib de álgebra esparsa) não foi auditada.

## 4. Há exemplos de constraints de CAD aplicadas a animação ou ilustração vetorial?

### Takeaway
Há pouca evidência primária em ferramentas vetoriais mainstream. A linhagem é Sketchpad (1963) → Juno/CoDraw/Briar → geometria dinâmica (Geometer's Sketchpad, que anima construções arrastáveis) → protótipos acadêmicos como o QuickDraw (CHI 2012). Não achei constraints de CAD no Graphite (que só tem o nó Auto-Tangents). O caso mais próximo do PH2D é o **Fiksi**, nascido no ecossistema de gráficos 2D (Linebender), e o próprio **ezpz**, que se apresenta como "para CAD ou aplicações gráficas".

### Cited Findings
- Sketchpad (Sutherland, 1963) é tido como a primeira ferramenta de desenho por constraints. Juno, CoDraw e Briar vieram depois. Uma fraqueza comum a elas é **mostrar as constraints no desenho**. — [Auckland seminar report](https://cs.auckland.ac.nz/compsci705s1c/archive/2008/assignments/seminarreports/tjud007%20702%20Report%20-%20constraint%20based%20drawing.pdf)
- Geometer's Sketchpad: as figuras arrastam-se mantendo as constraints da construção, e os objetos podem ser animados. — [Wikipedia](https://en.wikipedia.org/wiki/The_Geometer%27s_Sketchpad)
- QuickDraw (Microsoft Research, CHI 2012): reconhece segmentos e círculos esboçados, **infere constraints** entre eles e "embeleza" o diagrama. Foi comparado com PowerPoint, Cabri II Plus, Geometry Expressions e GSP. — [MSR QuickDraw](https://www.microsoft.com/en-us/research/publication/quickdraw-improving-drawing-experience-geometric-diagrams/)
- Graphite (alfa, set/2025): booleanas, gradientes, ferramentas de vetor e um nó "Auto-Tangents"; nenhuma menção a solver de constraints. — [LWN](https://lwn.net/Articles/1051242); [mirror Graphite](https://git.joshthomas.dev/mirrors/Graphite/src/commit/c51967384fee18ea4ea118b78287361cd06aada8)
- ezpz: "a 2D constraint solver, for use in CAD or graphics applications". — [docs.rs ezpz](https://docs.rs/ezpz)

### Inferences
- Para animação, um solver de sketch tem dois usos naturais. O primeiro é o **rig**: constraints como ligações, com o drag com movimento mínimo a funcionar como IK geométrica. O segundo é **chavear os valores das dimensões** (uma dimensão driving animada no Auto Key), com as dimensões REF a servir de medidores para expressões e scripts. Isto conversa com a lei do PH2D "Auto Key grava tudo".
- O problema de "mostrar as constraints no desenho" (citado como fraqueza histórica) é resolvido nos CADs modernos com ícones ao hover, um Shift para mostrar tudo e um gestor com filtros (Onshape). Esse é o modelo a seguir.

### Gaps
- Não verifiquei Figma, Penpot, Inkscape, Illustrator, Vectorworks, Rhino/Grasshopper, Dynamo, Cuttle nem Apparatus: as buscas não devolveram fontes primárias. Conhecimento prévio sem fonte: as "constraints" do Figma e do Penpot são de **layout** (pin/scale em relação ao frame), não geométricas; o Vectorworks e o Rhino têm sketch constraints limitadas; o Grasshopper usa o Kangaroo (solver de dinâmica por projeção) para comportamento de constraint; o Apparatus e o Cuttle são editores paramétricos de nicho.
- Não achei nenhum caso documentado de um solver de sketch de CAD usado num editor de **animação** 2D de produção.
