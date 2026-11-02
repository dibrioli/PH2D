# A história dos cabeçalhos do `ph2d-app-vec` (movida do código, F47, 2026-11-02)

Verbatim: o `//!` de cabeçalho de cada módulo abaixo, menos o 1.º parágrafo (que ficou no ficheiro). Saíram
do código pela catraca de LOC por crate (`the_shell_only_shrinks_and_every_app_crate_too`, tecto `93 637`) e
pela lei do CLAUDE.md §2 («o porquê histórico vive no doc, com link; o código diz a lei»). Os links
intra-doc do rustdoc (`[`crate::x`]`) ficaram como texto. Texto à data do commit `d95d9a1bea`.
2.º lote (F48, 02/11, o Expand sobre a pilha: +545 sobre o tecto): de `smoke_bone_paint.rs` em diante, texto à
data do commit `8810c8838d`.
3.º lote (F48b, 02/11, o Offset Path no padrão do Inspector: +386 sobre o tecto): de `state.rs` em diante, texto à data do commit `878fa728b1`.
4.º lote (F48d, 02/11, o Offset Path vira ferramenta: +76 sobre o tecto): `lib.rs` e `smoke_bone_paint_assada_tests.rs`, texto à data do commit `fa1283eec8`.
5.º lote (F48d, 02/11, a ferramenta Offset Path: +8 sobre o tecto): `ui_panel_spec.rs`, texto à data do commit `fa1283eec8`.
6.º lote (F49, 02/11, o Draw do vetor fica nas outras formas: +103 sobre o tecto): de `resize_box_edit.rs` a `svg_import.rs`, texto à data do commit `9362272bff`.
7.º lote (F50, 02/11, Boolean e Convert na ④: +309 sobre o tecto): de `no_orphan_module_tests.rs` a `connector_panel_tests.rs`, texto à data do commit `ca6875143c`.
8.º lote (fecho F46–F50, 02/11, curas da auditoria: +186 sobre o tecto): de `bool_gesture.rs` a `smoke_bone_paint_pixels_tests.rs`, texto à data do commit `55c489600d`.

## `symmetry_live.rs`

> **A SIMETRIA VIVA na shell** — o cozimento das cópias do [`ph2d_ecs::VecSymmetry`].

Espelho do [`crate::offset_live`] / [`crate::contour_live`]: o componente guarda a relação (que
espelho, onde, quantas cópias) e a aparência é uma **função pura** dela, re-cozida aqui e
desenhada por [`ph2d_vec_render::dispatch`] **no z da forma**.

# A simetria é da SESSÃO DE DESENHO, e não da forma seleccionada

*"A linha de simetria deve aparecer logo que se aperta o botão e não quando se inicia o
desenho. A simetria funciona apenas para formas que serão desenhadas com a tool ligada e não
deve fazer simetria de formas que já existem previamente. Com o botão checado pode-se fazer
quantos desenhos desejar que a linha de simetria permanece no lugar"* (Enio, 2026-08-01).

São três exigências e elas decidem o modelo inteiro:

1. **A linha existe sem forma nenhuma.** Logo ela não pode ser propriedade de uma forma: há um
   **eixo de SESSÃO**, em coordenadas de MUNDO, semeado no centro do ECRÃ no instante em que o
   botão liga. É ele que aparece, com a cena vazia e nada seleccionado.
2. **Só o que for DESENHADO com o modo ligado espelha.** Logo armar **nunca** toca a selecção —
   a adopção acontece no nascimento, e o oráculo de *"o artista está a desenhar isto agora"* é
   o [`ph2d_vec_entities::transform::gesture_paths`], a mesma porta que o `settle_origins` usa. Um
   diff de ids apanharia colar, o restore do undo e o próprio *Apply*, e espelharia coisas que
   o artista não desenhou.
3. **O eixo fica no lugar entre desenhos.** Logo ele não segue a câmera depois de semeado nem
   salta para cada forma nova: uma semeadura por LIGAÇÃO, e ela vale para a sessão toda.

E a promessa original continua de pé — *"uma vez que o desenho é feito, a referência para a
linha passa a ser o próprio desenho"*: cada forma **captura** o eixo de sessão no espaço LOCAL
dela, e a partir daí move-se com ela.

# A captura só sela quando o pivô assenta

⚠️ Enquanto a forma está em gesto o [`ph2d_vec_entities::transform::settle_origins`] pula-a, e no frame
em que o gesto acaba ele **translada a geometria e compensa no `Transform`** para pôr o pivô no
centro. Um eixo capturado antes disso ficaria deslocado exactamente por essa translação — e em
silêncio, porque nada falha. Por isso a re-derivação corre enquanto a forma está em gesto **e
mais uma vez no frame seguinte** (o `prev_drawing`), que é justamente o frame em que o pivô
assenta. Depois disso o eixo é DELA e ninguém lhe toca.

# Desarmar esconde, não destrói

*"Se a simetria for desmarcada antes do apply, as cópias somem mas não são destruídas"*. O
interruptor gateia o **COZIMENTO**, não o componente: as cópias desaparecem porque deixam de
ser cozidas, e voltam inteiras ao religar. Elas nunca estiveram no documento, então não há
nada a destruir — e nada a reconstruir.

# O cozimento é LOCAL, e a ORDEM é a espinha

O eixo vive no espaço da geometria autorada (ver o cabeçalho do componente), então a reflexão
corre **antes** de a pose entrar: `reflectir → assar a pose`. A ordem inversa só daria o mesmo
resultado se a pose fosse uma semelhança — sob escala não-uniforme ou skew as duas divergem, e
a que corresponde ao que o artista vê é esta (o espelho é uma propriedade da FORMA, e a pose
leva a forma inteira, cópias incluídas).

⚠️ É por isso que este módulo **não** pode copiar o `offset_live` verbatim: lá a distância é de
MUNDO, então ele assa primeiro e offseta depois. Aqui é o contrário, e a chave do memo tem de
carregar a pose junto (o resultado depende dela).

# Quem lê a DERIVADA e quem lê a FONTE

Lêem a derivada os que respondem *"o que está na tela aqui?"* — o [`ph2d_vec_render::dispatch`]
e o hit-test de canvas. Lêem a FONTE os que respondem *"o que o artista escreveu?"*: o modo
**Node** (as âncoras que ele arrasta são as autoradas — a mesma lei do raio de quina vivo,
ADR-0121) e a **caixa do gizmo**. Nenhum dos dois é esquecimento.

# O memo

O cozimento roda por frame, então sem memo uma cena parada pagaria a reflexão para sempre. A
chave é o que de facto determina o resultado — a geometria LOCAL que entra, a pose, e a spec —
e não um contador de versão que alguém esqueceria de bumpar.

## `widget_icon.rs`

> **O DESENHO É O GLIFO** — a porta única do ícone de um `IconButton` autorado.

Um `IconButton` é o segundo (e último) tipo do catálogo cujo parâmetro nem o retângulo, nem o
rótulo, nem os tokens, nem o estado vivo determinam: *qual* ícone? Num editor VETORIAL a
resposta nativa é a que não inventa canal nenhum — **a forma que veste o botão É o glifo**. O
artista desenha a engrenagem, marca-a `IconButton`, e o app põe a moldura à volta dela.

# Por que isto é UMA função e não duas

As duas metades do W8b precisam do mesmo glifo por motivos diferentes — o **canvas** para o
pintar ao vivo, o **codegen** para o escrever no painel gerado como texto — e é exactamente aí
que nasce a divergência que só uma screenshot revela: um ícone no canvas e outro no painel.
Elas percorrem esta função, e o que muda depois dela é só a saída
([`BezPath`] × [`BezPath::to_svg`]).

# A geometria é a AUTORADA, não a projetada — e a consequência está nomeada

[`build_bezpath`] sobre o [`cooked`](ph2d_vec_scene::VecPath::cooked) do path: a quina viva
(ADR-0121) e a pilha de Live Path Effects entram, a **pose não**. Duas razões, nesta ordem:

- **as duas metades alcançam-na com ZERO argumento novo** — o `ui_panel_spec` tem só `scene`, e
  pedir-lhe `xforms`/`camera` para orientar um glifo que vai ser re-normalizado numa caixa de
  24 px seria churn a troco de nada. Sem argumento em comum, as duas produzem os MESMOS bytes
  por construção, e não meramente por gate;
- **a moldura já ignora a pose**: o chip é axis-aligned na tela. Honrar a rotação só no glifo
  seria honrar meia regra.

⚠️ **O preço, dito em vez de descoberto:** girar a forma pelo gizmo **não gira o glifo** — ele
é o desenho autorado, endireitado. Se um smoke pedir o contrário, é um argumento de distância.

# A normalização é UNIFORME

Um glifo é uma figura, e esticá-la para encher a caixa mudaria o desenho do artista. O lado
maior encosta nos 24, o outro centra-se. É a mesma lei que o [`paint_icon_path`] aplica depois
ao encaixar a viewbox no retângulo do botão — a de lá é sobre a CAIXA, a daqui sobre a FIGURA.

# ⚠️ E ela INVERTE Y, porque os dois espaços discordam

O documento é **Y para CIMA** (é a câmera quem inverte: `world_to_screen_affine` multiplica por
`scale_non_uniform(k, -k)`), e a caixa de 24×24 do ícone é **Y para BAIXO** — ela é a viewbox do
SVG, que é de onde todo glifo do catálogo nasce. O [`paint_icon_path`] mapeia essa caixa para o
retângulo do botão **sem inverter nada**, e está certo: para um `IconId` as duas pontas já
falam Y-down.

Esta função é o ÚNICO ponto em que uma geometria de documento entra naquela caixa, logo é aqui
que a conversão pertence. Sem ela o desenho do artista chega ao botão **de cabeça para baixo**
(report do Enio, 2026-08-09) — e chegava nas DUAS metades, porque as duas passam por aqui.

[`paint_icon_path`]: ph2d_editor_core::paint::paint_icon_path

## `fx_live_memo.rs`

> **A CHAVE do memo do FX raster** — a pergunta *"os pixels desta forma ainda servem?"*, feita num só lugar.

# O defeito que este módulo existe para fechar (medido 2026-07-29)

O memo do [`crate::fx_live`] era `(pilha resolvida, largura, altura)`. Mas o que está na textura
é o resultado de **rasterizar a forma** e correr a pilha sobre esses pixels — e a forma não
entrava na chave. Consequência alcançável: **mudar a cor do preenchimento de uma forma filtrada
não muda a tela.** A pilha é a mesma, a caixa é a mesma, então o memo acerta e o artista
continua a ver a cor antiga, sem erro e sem warning. O mesmo valia para o traço, para a pilha de
Live Path Effects e para qualquer edição de geometria que não movesse a caixa (arrastar um
vértice interior).

⚠️ **O memo não era otimização, era uma afirmação**: *"nada que eu desenho mudou"*. Ela estava
escrita em três números que não sabem nada sobre o desenho.

# A lei, e por que a chave carrega VALORES e não um hash

A chave é **exatamente o que os dois consumidores leem** — o
[`ph2d_vec_render::draw_path_isolated`] (que põe a forma na célula do atlas) e o
[`ph2d_vec_render::silhouette_segments`] (que dá a fronteira aos degraus que a usam). Ela guarda
os **valores** e compara com `==`, como o `ops` já fazia: é **exato** (sem colisão, sem discutir
bits de `f64`) e o clone só é pago no **miss**, isto é, no frame em que já vamos rasterizar —
onde ele é ruído contra um render do Vello.

⚠️ **A TRANSLAÇÃO fica FORA da chave, de propósito.** O conteúdo da célula é a forma desenhada em
`-ex0,-ey0`, então mover a forma (ou panhar a câmera) dá **a mesma arte na mesma posição dentro
da célula** — o que muda é o `rect` onde a célula é desenhada, e esse é recomputado todo frame.
Incluir a translação faria **toda forma filtrada re-cozinhar em todo frame de pan**, que é
precisamente o gesto onde o memo se paga. O que se perde é a **fase sub-pixel**: um pan de meio
pixel reusa um antialiasing calculado para a fase anterior. É diferença de AA, e o preço de a
corrigir é re-cozinhar a cena inteira a cada frame de pan.

A parte **LINEAR** do afim, ao contrário, entra: rodar ou escalar muda os pixels de verdade.
Entram as duas metades dela (a da câmera e a da pose da forma) porque os dois consumidores
cascateiam diferente — a geometria derivada é desenhada pela câmera, a autorada pela pose — e
perguntar por uma só deixaria um vão exactamente onde a outra governa.

⚠️ **Mas `cam`/`screen` são CINTO, não gate, e a diferença está medida:** uma mudança linear que
altera os pixels altera a **caixa** também (girar 45° cresce `w`/`h`; escalar 2× dobra-os), e uma
que não altera a caixa é uma **simetria da forma**, onde os pixels são os mesmos. Não consegui
construir um caso em que estes dois campos decidam sozinhos — então eles ficam por serem 8 `f64`
de graça contra um modo de falha que é *pixels velhos que ninguém vê que são velhos*, e **nenhum
gate afirma que eles estão aqui**. Dizer o contrário seria vender um gate que não pode falhar.

## `svg_export.rs`

> ⭐⭐⭐ **EXPORTAR O DESENHO VECTORIAL COMO SVG** (plano 40).

Pedido do Enio (2026-09-02): *"precisamos de um meio de exportar o path para que vc possa
analisar melhor. Veja se já há exportação no app"*. Não havia: o app exportava **imagem** (16
formatos), **folha de sprites**, **peças 3D** e os **tokens DTCG** — nada que levasse uma curva.

# A lei

> **O que sai é o que se VÊ: a geometria COZIDA, no MUNDO.**

⚠️ **Cozida** — a pilha de Live Path Effects e o raio de quina já correram, e é por isso que um
ficheiro exportado de uma forma com efeitos abre igual ao que está no ecrã. ⚠️ **No mundo** — a
pose de cada objecto está assada na geometria, então não há um `transform` por elemento a
discordar do que a régua do editor mede.

⚠️⚠️ **O `d` sai da MESMA porta que o renderer usa** ([`ph2d_vec_render::build_contours`]) — uma
segunda travessia dos contornos daria um ficheiro que discorda do ecrã em curvas que nenhum
olho apanha, e essa é a classe de defeito que este módulo existe para NÃO ter.

# ⛔⛔ O EIXO Y, e o defeito que esta linha corrigiu em 2026-09-05

Até hoje o cabeçalho deste ficheiro dizia *"em coordenadas de MUNDO (Y para baixo, como o
SVG)"* — e as duas metades da frase **contradizem-se**. O mundo do PH2D mede o Y para **CIMA**
(`ph2d_render::Camera2d::world_to_screen_affine` é `scale(k, **−k**)`), o SVG mede-o para
baixo, e escrever as coordenadas cruas fazia **todo ficheiro exportado sair verticalmente
espelhado**.

⚠️ **Ninguém o viu porque o consumidor era uma LLM a ler números** (o pedido do Enio de 02/09 foi
*"precisamos de um meio de exportar o path para que vc possa analisar melhor"*), e **nenhum
gate media orientação** — os seis que existiam mediam tinta, pose, marca e a nota do cabeçalho.

⇒ A conversão passa pela porta ÚNICA ([`ph2d_vec_svg::world_to_svg`]), que é a inversa exacta da
que o IMPORTADOR usa. *Uma lei escrita em dois sítios ainda não é uma lei — só uma PORTA é.*

# ⛔ O que ele NÃO carrega, e diz

Um exportador que ignora em silêncio é pior do que um que recusa (a lei do importador `.ase`).
Um padrão, um pincel de contorno e um gradiente multi-ponto **não têm equivalente em SVG 1.1**:
saem com a **mesma cor sólida que o renderer usa quando o ladrilho não resolve**
(`Paint::primary_color` / `PatternFill::fallback`), e o cabeçalho do ficheiro **nomeia** cada
forma em que isso aconteceu.

## `shape_build.rs`

> **Shape Builder** — o cursor pinta REGIÕES, e o que ele pinta vira forma.

Com 2+ formas fechadas selecionadas, o plano fica dividido nas faces do arranjo
([`ph2d_vec_boolean::Arrangement`]): "dentro da A e fora da B", "dentro das duas", e
assim por diante. O Shape Builder deixa o dedo passar por cima dessas faces:

- **Arrastar** (ou clicar): as faces tocadas viram **UMA** forma.
- **Alt + arrastar**: as faces tocadas **somem**.

É a diferença entre pedir uma operação e **desenhar o resultado**. Um Pathfinder obriga
o artista a traduzir o que ele quer ("a lua crescente") numa sequência de ops ("subtrai
o círculo B do A"); aqui ele passa o dedo na parte que quer e pronto.

## A sobra é POR FORMA — e é isto que o smoke do Enio derrubou

A 1ª versão devolvia a sobra como `união(todas as fontes) − o que foi levado`: **uma**
forma só. O efeito no app real (medido, pela `build_smoke` da shell): um clique numa região
fundia o pentágono, a estrela e o retângulo num BLOB único, com um estilo só — as
silhuetas que o artista tinha acabado de desenhar simplesmente sumiam, e as fronteiras
entre elas (que são justamente o que define as faces seguintes) deixavam de existir.
Um clique destruía a arte.

Agora cada forma de origem sobrevive como **ela mesma menos o que foi levado**, com o
ESTILO dela e no z dela; a forma que ninguém tocou **não é sequer tocada** (mantém id,
entidade, `Transform`, raio de quina e os params de Live Shape — uma booleana que passa
por perto não pode assar uma forma paramétrica que o artista não escolheu). É o que o
Illustrator faz: o Shape Builder divide o que você percorre e deixa o resto em paz.

## A regra do estilo (uma só, e é a que a booleana já usava)

A forma NOVA (as faces pintadas) herda o estilo da forma do **TOPO entre as tocadas** — a
convenção do `apply_many` e do Illustrator. Sem essa regra, a face herdaria o estilo do
último argumento de uma SUBTRAÇÃO, que é justamente uma forma que ela **não** contém.

⚠️ **Mudou-se da shell em 2026-09-12** (`line/render-loop`, A9 da auditoria de arquitectura): a
`App` guardava a sessão num campo solto (`vec_build`) cujo TIPO morava aqui, na shell, e um
campo assim não pode juntar-se ao [`crate::state::VecState`]. O tipo tem `impl` (a lei do
gesto), então desceu o ficheiro inteiro — com os testes — e a shell ficou com o `impl App` que o
conduz
(`shape_build_gesture.rs`).

## `pattern_seam_probe.rs`

> ⛔⛔⛔ **A COSTURA DO LADRILHO — e ela NÃO é do renderer** (2026-08-30, plano 33 W10).

# ⭐⭐⭐ A conclusão vem primeiro, de propósito

*O grampo do amostrador custa **o salto do próprio ladrilho na volta**, e quase nada além
disso.* Um ladrilho que fecha não tem costura em qualidade nenhuma — medido: onda quadrada crua
`107` níveis, a mesma **espelhada** `0`. Um motivo assado de uma FORMA tem a caixa justa (alfa a
zero nos quatro lados) e mede **`0`**; quem tem o defeito é a arte de bordo a bordo que não foi
feita para repetir, e aí o artista já vê uma **aresta dura** muito maior que a banda do filtro.

⚠️⚠️ **Isto está no topo porque a acusação abaixo é convincente e leva à cura errada.** Ela
custou meio dia e três curas construídas em teoria e refutadas por medição — se o próximo leitor
encontrar o mecanismo antes do veredito, refaz a caça. *Uma nota que descreve um defeito sem
dizer de quem ele é manda o leitor seguinte ao sítio errado* — foi exactamente o que a nota do
`fill_path_image` fez comigo.

# O mecanismo (verdadeiro, e insuficiente)

Em `Extend::Repeat` o `fine.wgsl` embrulha a *coordenada* e depois **grampeia os TAPS** contra o
rectângulo da imagem no atlas — porque o atlas do Vello empacota as imagens **encostadas, sem
folga** (`vello_encoding-0.8.0/src/image_cache.rs`: `atlas.allocate(size2(w, h))`, zero
padding), e sem o grampo um tap leria a imagem VIZINHA. Do vello 0.9 em diante o `High` é
Mitchell a sério (16 taps, `+-1.5` texels) e o `image_quality_for` manda `Smooth -> High`, então
a banda passou de ~1 texel para ~3.

⛔ **Não há gutter possível**: qualquer folga acrescentada ao ladrilho entra no `extents` e passa
a fazer parte do **período**. E baixar para `Medium` foi medido e refutado — sob meio pixel de
`pan` os dois filtros chegam ao **mesmo** pico. As tabelas estão no irmão [`sweeps`].

# ⭐⭐⭐ O ORÁCULO, e porque ele não é um modelo do shader

⛔ Reimplementar o `bicubic_sample` em Rust mediria **o meu port**, não o renderer.

A régua aqui é a **própria periodicidade**: uma arte periódica de período `P`, assada como um
ladrilho de `P` px e como um ladrilho de `4P` px, produz **a MESMA imagem infinita** — os dois
buffers têm os mesmos bytes, repetidos. Logo, em toda coluna que é *interior* ao ladrilho largo,
o largo dá a resposta **certa**; e é exactamente aí que o estreito tem as suas costuras.

⇒ `|estreito - largo|` naquelas colunas **é** o defeito, medido em níveis de 8 bits, pelo
renderer de verdade, com a GPU de verdade. Sem port, sem epsilon inventado, sem oráculo externo.

## `smoke_fade.rs`

> ⭐⭐⭐ **A LINHA DO TEMPO DESVANECE UM CAMINHO VETORIAL** — `PH2D_VEC_FADE_SMOKE=1`.

# O defeito que a cena fecha

Até 2026-09-04 o canal `Opacity` existia para **todo** objecto e era **MUDO** num vetor: a row
aparecia no *+ Track*, aceitava chaves, desenhava a curva no editor de gráfico — e não movia um
pixel. O braço da escrita exigia um `ph2d_render::Sprite`, e a entidade de um caminho vetorial
nasce com `(Transform, Name, VecPathRef, RootOrder)` e nunca com um.

# A cena (unidades de mundo, ~±6)

Duas estrelas iguais, com a **mesma** curva de opacidade — `1 → 0 → 1` em 4 s:

- **"Fade"** (esquerda, azul) — uma forma simples. Ela prova a metade nova: a linha do tempo
  alcança a aparência de um vetor.
- **"Fade FX"** (direita, laranja) — a mesma forma **com um brilho** (`FxOp::GLOW`). Ela prova
  a segunda metade, e é a que **falhava em silêncio**: o memo do FX guardava a forma AUTORADA,
  e a opacidade é uma camada de VISTA aplicada depois — então a textura acertava o memo e
  ficava com os pixels opacos da era anterior. *Pixels velhos que ninguém vê que são velhos.*

# O que provar (dê PLAY, ou arraste a régua)

1. **As duas** desvanecem juntas, e voltam juntas. Uma que não desvanece é o defeito.
2. **A da direita desvanece com o brilho** — o halo some junto com a forma, não depois nem
   "por degraus". Se ela ficar cravada opaca enquanto a esquerda desaparece, o vão do memo
   voltou.
3. **No topo (`t = 0` e `t = 4`) as duas voltam a OPACO**, não a um degrau abaixo. Um `254` em
   vez de `255` é invisível numa cor chapada e visível na borda contra o fundo.
4. ⭐ **As DUAS rows da régua mostram um número que ANDA** (2026-09-04) — `1.00` → `0.00` →
   `1.00`, ao lado de *"Fade · Opacity"* e *"Fade FX · Opacity"*. Ele vem do MUNDO e não da
   curva, e é por isso que ele é a régua do ponto 2: **se a da direita ficar opaca com o número
   dela a marcar `0.00`, o desenho é que não está a honrar a opacidade** — foi exactamente esse
   o par de defeitos reportado neste dia.

⚠️ O número só é pintado enquanto a coluna de nomes tiver largura para ele **e** para o nome
(arraste a divisória: no piso ela larga o número, nunca o nome).

⚠️ Se a linha `[vec-fade-smoke]` não aparecer, PARE: a cena não montou e o resto não significa
nada.

## `contour_live.rs`

> **O Contour VIVO na shell** — o cozimento do [`ph2d_ecs::VecContour`] (pesquisa `20_*` item #9).

Generalização direta do [`crate::offset_live`]: onde ele produz UM caminho offsetado, este
produz **N anéis concêntricos com uma rampa de cor**. A forma fonte nunca é tocada — é ela que
o modo Node edita —, e os anéis são desenho derivado, re-cozidos aqui e desenhados por
[`ph2d_vec_render::dispatch`] no z dela.

# A ordem de desenho é UMA regra: por PROFUNDIDADE (distância assinada decrescente)

Cada anel nasce a uma **distância assinada** da fonte: `> 0` é para FORA (maior, atrás dela);
`< 0` é para DENTRO (menor, à frente dela, senão ela o tapa). A fonte está no `0`. Ordenar tudo
por distância DECRESCENTE — o mais para fora primeiro (fundo), a fonte no meio, o mais para
dentro por último (frente) — é UMA regra que serve os três Sides (Outer/Inner/Both) e os dois
sinais do slider sem um `if` por caso.

# O `side` do Contour é DIREÇÃO, não seleção de furos

⚠️ O `OffsetSide` do módulo Expand escolhe QUE contorno anda (a borda de fora × os furos), e
isso é o certo para o *Offset Path*. Mas um Contour age sobre a **silhueta**, e a estrela do
smoke não tem furos — então "Inner = só furos" não movia um pixel (o report do Enio). Aqui o
`side` vira a **direção** do modelo do Corel: **Outer** = para fora (respeita o sinal do
slider); **Inner** = o ESPELHO (a direção oposta); **Both** = os dois lados. O offset é sempre
chamado com `OffsetSide::Outer` (a silhueta) e a direção sai do SINAL da distância, então o
offset direto (`offset_ring`) cobre TODOS os Sides — o FPS e o pisca-pisca não voltam por Inner
nem por Both. Um compound (com furos) cai no `offset_path` da silhueta, ignorando os furos:
um contour segue a borda de fora, que é o que o artista espera.

# O memo é POR ANEL, e é o que torna o slider de passos barato

O anel `k` está em `d · k^accel` — uma função só de `k` e dos parâmetros, **nunca da contagem**
(vide o § do `d` no [`ph2d_ecs::VecContour`]). Então acrescentar um passo não invalida os
anteriores: o memo guarda o `Vec` de anéis já cozidos e só coze o que falta. Medido
(`probe_contour_cost`): um anel custa 0,4–1,4 ms conforme forma e quina, então re-cozer 16 por
frame arrastando os passos seria 5–11 ms — o gesto mais comum do efeito, e o mais caro, se o
memo não existisse.

Quem **invalida** o memo inteiro é o que move todos os anéis de uma vez: a geometria de mundo,
o `d`, a quina, o lado e a aceleração. É a mesma chave do `offset_live` — o que ENTROU —, não
um contador de versão que alguém esqueceria de bumpar.

## `bool_live.rs`

> **A BOOLEANA VIVA** — um grupo cujos filhos se combinam, e continuam editáveis.

O 7º produtor de [`LiveGeometry`], e o **segundo que TRANSFORMA o mapa** em vez de o estender
(o primeiro é o [`crate::align_live`]). A razão é a mesma dele e vale a pena repeti-la: os
cinco produtores que o `render_loop` funde com `extend` são mutuamente exclusivos — uma forma
é offset OU pattern OU contour, cada um é um componente próprio e o painel oferece um de cada
vez. **A booleana não é membro dessa família**: ela é um componente do PAI, então os operandos
dela podem ter, cada um, o seu offset vivo. Fundida por `extend` ela apagaria esses offsets —
ou seria apagada por eles — em silêncio.

# O que entra, e o que sai

A entrada de cada operando é **o que o mapa já diz** (o offset/pattern/contour dele, se
houver), ou a fonte assada em MUNDO. É a lei do `align_live`, e é ela que faz a booleana
operar sobre *o que se DESENHA* em vez de sobre *o que está guardado*.

A saída vai no id da **BASE** — o operando mais ao FUNDO —, e cada um dos demais recebe uma
lista **VAZIA**, que o `dispatch` desenha como nada (`ph2d-vec-render/src/lib.rs:174`: ele
itera os items do mapa, e zero items desenham zero). Não é arranjo: é exatamente a regra que a
booleana DESTRUTIVA já segue (*"a base é a de trás: o resultado ocupa a fatia de z dela"*), e
é o que faz o **Apply** não mover a arte um pixel.

⚠️ **O grupo NÃO pode ser a chave.** Um grupo é *"uma entidade sem geometria própria que tem
filhos"*, então ele não tem `VecPathId` — e a `LiveGeometry` é chaveada por `VecPathId`. Foi
por isso que a base virou a portadora do resultado.

# `Err` e `Ok(vazio)` são coisas DIFERENTES

`Ok(vazio)` é uma RESPOSTA: a interseção de duas formas disjuntas é o vazio, e desenhar nada é
o certo (a mesma lei que o `offset_live` documenta — *"vazio é a ANIQUILAÇÃO"*). `Err` é o
motor a RECUSAR (geometria degenerada no meio de um arrasto), e aí o mapa fica **intocado** —
a arte continua como estava em vez de piscar.

# Aninhamento: o mais FUNDO primeiro

Um grupo booleano dentro de outro compõe naturalmente **se** o de dentro cozinhar antes: o
resultado dele pousa no id da base dele, e o de fora encontra esse id no mapa e o consome como
operando. Por isso os grupos são processados por profundidade decrescente — a ordem é a única
coisa que separa "compõe" de "o de fora lê a fonte crua do de dentro".

## `widget_drive.rs`

> **A ROW DIRIGE A FORMA** — o valor vivo de um controle autorado, projetado no que a cena desenha (plano UI/UX W8b.3).

É a metade que fecha o laço do §2: *o desenho é a PELE, o widget é o COMPORTAMENTO*, e aqui o
comportamento passa a ter CONSEQUÊNCIA. Sem isto o painel gerado responde ao ponteiro e não
muda nada na arte — um controle que se move e não move nada.

# A porta única: o que uma row FAZ é derivado do TIPO dela

[`drive_of`] é a ÚNICA resposta a *"o que este widget dirige?"*, e a resposta sai do
[`WidgetKind`] — nunca de um campo autorado ao lado. Um Slider produz um número e o número que
uma forma tem é a **opacidade**; um Toggle produz um sim/não e o sim/não que uma forma tem é
**aparecer**. Um segundo controle (*"esta row dirige o quê?"*) teria de ser mantido de acordo
com o tipo, e no dia em que discordassem o painel pintaria um slider que apaga a forma.

⚠️ **Quem não RESPONDE não dirige**, e o conjunto é menor que o dos controles: um `Button`
responde ao ponteiro e produz um EVENTO, não um valor — dirigir uma forma com ele exigiria um
vocabulário de VERBOS (*"faz o quê?"*) que nenhum consumidor pede ainda, e um vocabulário sem
consumidor é desenhar no escuro (a lei que o `OverrideSlot` da W5a já escreveu). O painel
**não oferece** o vínculo nesses tipos, em vez de o oferecer e o deixar inerte.

# VISTA, nunca documento

O resultado é publicado no [`VecViewState`] — a projeção por-frame — e **nada aqui escreve a
cena**. Três consequências, e as três são o motivo:

1. arrastar um slider **não vira passo de undo** (o undo deste editor é por DIFF do mundo, e
   uma pose derivada escrita no documento faria cada gesto encher a fila — a lição que o
   ADR-0153 pagou no auto layout);
2. a tinta autorada **sobrevive**: arrastar até zero e voltar devolve a arte AO BIT, porque
   ninguém a reescreveu;
3. e o fato tem UM dono — a `VecViewState` é a mesma lista que o olhinho da Hierarquia e os
   tokens já alimentam.

⚠️ **O preço honesto, nomeado:** o valor de uma row vive no `WidgetStore`, que é de runtime ⇒
ele **não é salvo**. Reabrir o projeto devolve os controles ao default e a arte ao que o
artista autorou. Persistir a POSIÇÃO de um controle é a W4b/W8a, não esta.

## `hier_group.rs`

> ⭐⭐⭐ **AGRUPAR / DESAGRUPAR pela Hierarquia** (Enio, 2026-08-30).

⚠️ **Ele viveu na `shells/desktop` e mudou-se para cá em 2026-09-20**, quando a catraca
`the_shell_only_shrinks` cobrou a wave das âncoras do HUD. ⭐ A mudança é a lei da casa
(ADR-0075: *código de família vive em `crates/ph2d-app-<família>`*) e não um arranjo: o ficheiro
e os gates dele tocavam **só** `ph2d_vec_entities`, `ph2d_ecs` e `ph2d_editor_core` — nenhum
símbolo da shell —, e o modelo de grupo que eles governam já vivia em
[`ph2d_vec_entities::entities`]. O que fica lá é a FASE que resolve a linha em bits e chama
estas portas.

# O que faltava, e o que não faltava

O modelo de grupo **já existia inteiro**: uma entidade sem geometria própria com filhos
([`ph2d_vec_entities::entities`]), com selecção que entra e sai do grupo inteira, gizmo de canvas,
árvore recolhível na Hierarquia e persistência. Até o gesto existia — `Ctrl+G` / `Ctrl+Shift+G`.

⛔ **O que não existia era ALCANCE.** Nenhum menu, botão, rótulo ou entrada de paleta em todo o
app dizia a palavra *Group*. É a lei deste repo aplicada à UI: *uma ferramenta que nenhum passo
escrito chama pelo nome morre* — e esta estava morta para quem não adivinhasse o atalho.

⚠️ **E o atalho tinha três cercas que ninguém veria:** ele só responde com a ferramenta **Vector**
em mãos, lê **apenas** a selecção de caminhos da caneta (uma sprite escolhida na Hierarquia era
invisível para ele, embora o `group_entities` a aceite de bom grado), e falhava para o `stderr`.
O verbo da Hierarquia entra por outra porta e não herda nenhuma das três.

# A lei do SUJEITO — e ela **já estava decidida nesta casa**

O menu é por LINHA, mas agrupar é sobre um CONJUNTO. O *Merge Sprites* enfrentou exactamente
esta pergunta e a resposta dele fica: **se a linha clicada está na selecção, o sujeito é a
selecção**; se está fora e há uma selecção múltipla, o verbo **não age — ORIENTA**
(*"right-click on one of the selected objects"*), porque agir sobre a união traria para dentro
do grupo um objecto que o artista não escolheu, e agir só sobre a linha faria *Group* falhar
por ter um sujeito só.

⛔ Inventar aqui uma terceira lei para a mesma pergunta seria a divergência que este repo paga
sempre: *duas respostas à mesma pergunta, e a que o artista encontra é a que envelhece.*

## `bucket_claim.rs`

> ⭐⭐⭐ **DE QUEM É CADA FACE** (plano 40 §11) — resolvido pelas **ÂNCORAS**, sem memória nenhuma do quadro anterior.

Módulo irmão do [`crate::vec_bucket`], e o corte é por RESPONSABILIDADE: aquele responde
*"quando se recoze"*, este responde *"quem herda o quê"*.

# ⛔⛔ O modelo anterior, e porque ele foi SUBSTITUÍDO

Report do Enio, quatro vezes seguidas (2026-09-01 e 02): a tinta partia-se ao atravessar uma
linha, sumia, deixava resíduo, trocava de área. Cada defeito teve a sua cura, e o report voltava.

⚠️⚠️ **A causa não era nenhum deles: era o modelo.** A receita de um preenchimento era *a região
que ele pintou no quadro anterior*, e o dono de cada face saía de comparar as faces de hoje com
as regiões de ontem — uma votação por amostragem de área. Isso **deriva**: o que um quadro
decide vira a régua do seguinte, e um único quadro de topologia confusa reatribui a tinta **para
sempre**, porque nada puxa de volta. Medido nos SVG que ele exportou: a partir do estado
`drawing01`, o corpo do círculo direito vota **azul** — o app tinha-o **verde**, ganho num quadro
intermédio e nunca devolvido.

# A lei nova

> **Uma região é o lado de um conjunto de pedaços de linha** — e não uma coordenada, nem uma
> área, nem o que estava lá antes.

No clique, o balde grava as **âncoras** da face: para cada arco que a cerca, *de que contorno de
que caminho ele é um pedaço*, *em que fracção*, e *de que lado*. Arrastar um nó move a curva
**sem mudar de que curva o pedaço é** ⇒ a face segue.

⭐⭐⭐ **É STATELESS.** Cada quadro resolve-se do documento sozinho: *o mesmo desenho dá sempre as
mesmas cores, seja qual for o caminho por que se lá chegou.* É a propriedade que os quatro
reports pediam, e a única que nenhuma heurística sobre o quadro anterior podia dar.

⭐ **E o resto cai de graça:** uma região que se PARTE deixa umas âncoras a cercar uma metade e
outras a outra — o preenchimento fica com as duas, sem uma linha de código sobre partir. Uma
FUSÃO põe as âncoras de dois na mesma face — ganha quem tem mais lá. Uma área NOVA não tem
âncora nenhuma — fica por pintar, que é o que o artista espera de uma região que nunca visitou.

## `smoke_bone_par.rs`

> ⭐⭐⭐ **O PAR — a MESMA barra como DESENHO e como IMAGEM** — `PH2D_VEC_BONE_SMOKE=3` (ordem do dono, 2026-09-29: *«monte a cena com apenas dois esqueletos iguais, um com vector outro com img»*).

# Porque esta cena existe

O report anterior dele (*«o osso do meio provoca ondulações discretas, que diferem da deformação
de imagens»*) só se julga com as duas mídias **lado a lado e em condições iguais** — e a cena
`=1` não as dá: lá o braço vectorial tem `7 × 1` m e o pintado `3,2 × 0,96`, com esqueletos de
tamanhos diferentes e sítios diferentes do ecrã. *Duas peças que diferem em tudo não dizem qual
diferença é da mídia.*

# O que a cena monta, e o que é IGUAL por construção

| | em cima | em baixo |
|---|---|---|
| a peça | um `RoundRect` **vectorial** | uma **imagem** pintada da mesma cápsula |
| o tamanho | [`peca`] — UMA porta para os dois | idem |
| a cor | [`COR`] | idem |
| o esqueleto | [`OSSOS`] ossos pelo eixo da cápsula | idem, deslocado só em `y` |
| a pose | [`DOBRA`] em Z, aplicada DEPOIS de prender | idem |

⚠️ **A dobra é aplicada no 2.º tempo, depois de prender**, e é load-bearing: prender fotografa a
pose de REPOUSO, e com o esqueleto já dobrado as duas peças aprenderiam o dobrado como repouso e
abririam RECTAS.

⭐⭐⭐ **E a `=4` é A DOBRA FORTE** (ordem do dono, 2026-09-29: *«a dobra forte do cotovelo:
tente o estado da arte diretamente»*): o MESMO par, empilhado e aberto já a [`DOBRA_FORTE`] por
junta — onde a face de dentro de dois membros se SOBREPÕE. O desenho sai como a SILHUETA dos
membros (canto em «V»), que é o que a imagem de baixo já mostra de graça; com
`PH2D_SKIN_CONTACTO=0` volta o contorno com o «olho» por dentro, e é essa a comparação.

⚠️ **Nada mais na cena, de propósito** — o dono pediu *«apenas»*. Sem folha solta, sem
bifurcação, sem barra por cima: *uma cena de comparação com uma terceira peça obriga o olho a
decidir o que comparar*.

## `fx_silhouette.rs`

> **A silhueta RESOLVIDA das formas com traço** — o que o campo de distância dos FX precisa saber sobre uma forma que o `ph2d-vec-render` sozinho não sabe responder.

# O bug que este módulo fecha

O campo exato do plano 24 é semeado pela GEOMETRIA (o pé da fronteira sai de um laço sobre os
segmentos da silhueta) e cai no JFA sobre o raster quando não há geometria. O caminho do raster
semeia em texels DISCRETOS, então a direção que ele devolve salta na fronteira de célula de
Voronoi e a distância escadeia — renderizado, o bevel sai com um **pente** de hachuras
diagonais finas. Foi o que o Enio fotografou.

E toda forma com TRAÇO caía nesse caminho, porque `silhouette_segments` recusa (com razão) o
caminho autorado: numa forma traçada, a curva que o artista desenhou passa pelo MEIO da faixa
de tinta, e semeá-la poria a fronteira dentro da forma. A silhueta que se vê é
`preenchimento ∪ contorno-do-traço` — e só a booleana sabe uni-los.

# Por que a união mora AQUI e não lá

O `ph2d-vec-render` **não depende** do `ph2d-vec-boolean` (o `Cargo.toml` dele avisa do skew de
versão do kurbo), e essa cerca é boa: o desenhista não precisa saber resolver interseção. Então
a shell — que já conhece as duas — pergunta à booleana e entrega o resultado pronto, do mesmo
jeito que entrega a geometria derivada do offset/pattern/contour.

# O memo é MEDIDO, e o que ele guarda é camera-INDEPENDENTE

`silhouette_paths` custa, em release, **0,19–0,31 ms** numa estrela de 5 pontas (10 âncoras),
**0,46–0,67 ms** numa de 12 (24 âncoras) e **1,67–2,54 ms** numa de 40 (80 âncoras) — linear na
contagem de âncoras, ~0,02 ms cada. O re-cook dos FX roda a cada frame de um arrasto de zoom
(a pilha resolvida em pixels muda), então sem memo um zoom sobre uma forma complexa pagaria
isso por frame.

A chave é o que de facto determina a resposta — a geometria de MUNDO que entra — e não um
contador de versão que alguém esqueceria de bumpar. É o mesmo desenho do [`crate::offset_live`],
e por o mundo ser função da POSE (nunca da câmera) o memo acerta durante todo pan e zoom: a
união é paga por EDIÇÃO, não por frame.

## `align_live.rs`

> **O ALINHAMENTO do traço, vivo** — Inner/Outer desenhados no z da forma.

Espelho dos cinco irmãos (`offset_live`/`pattern_live`/`contour_live`/`symmetry_live`/
`profile_live`): o documento guarda o que o artista autorou e o que se vê é derivado, re-cozido
aqui e desenhado por [`ph2d_vec_render::dispatch`] no z da fonte.

# ⚠️ Este roda por ÚLTIMO, e não é ordem por gosto

O `render_loop` funde os cinco mapas com `extend` e o comentário de lá diz por que isso é
seguro: *"uma forma é offset OU pattern OU contour (nunca dois — cada um é um componente
próprio e o painel oferece um de cada vez)"*. **O alinhamento não é membro dessa família.** Ele
é um campo do `StrokeSpec`, então **compõe** com qualquer um deles: uma forma pode ter offset
vivo *e* traço interno. Fundido por `extend` ele apagaria o offset (ou seria apagado por ele),
e a forma perderia metade do que o artista pediu, em silêncio.

Então ele **não produz um mapa** — ele **TRANSFORMA** o mapa já montado, como o
`fx_silhouette`, que roda depois pela mesma razão (*"a união é do que se DESENHA"*). A entrada
de um caminho é o que estiver no mapa; se não houver nada, é a fonte assada em mundo.

# O que uma entrada vira

Um `VecPath` com traço alinhado é **substituído por dois ou mais**: ele próprio sem traço (o
preenchimento fica onde estava) e as peças da faixa recortada, que já nascem preenchidas com a
cor do traço (ver [`ph2d_vec_boolean::aligned_stroke`]). Um item recusado pelo motor — caminho
aberto, Centre, largura zero — passa **verbatim**; e se NENHUM item alinhou, o mapa não é
tocado, que é o que torna o mundo pré-alinhamento byte-idêntico (uma entrada, mesmo igual à
fonte, já muda a rota de desenho).

# O memo é MEDIDO, e o número decidiu contra ele

`aligned_stroke` custa **0,064-0,069 ms** numa rosquinha, **0,18** numa estrela de 5 pontas e
**1,46** na patológica de 24, contra 16,6 ms de um quadro. Isso é vivo com folga, e o memo
existe pelo caso da CENA CHEIA (dez formas traçadas × por frame), não pelo caso de uma. A
chave é o que de facto determina a saída — a geometria de MUNDO que entra — e não um contador
de versão que alguém esqueceria de bumpar.

## `vec_frame_edit.rs`

> **A moldura da SELEÇÃO** — a projeção que o painel lê, e o chip que a edita.

Duas metades da mesma pergunta (*a seleção é uma moldura, e ela recorta?*), respondidas UMA vez
e no lugar onde o mundo mora. O painel não alcança o ECS — e não deve: se alcançasse, haveria
duas respostas para *"esta seleção é uma moldura?"*, e a que desenha a seção divergiria da que
honra o clique.

# A moldura é a que CONTÉM a seleção

⚠️ **Um GRUPO puro ainda expande** — o [`crate::vec_selection`] entrega ao pen o que
`selection_paths` diz, e um grupo não tem geometria própria, logo empresta os filhos. Então
uma moldura dentro de um grupo chega aqui junto com os irmãos, e uma regra de *"exactamente
um selecionado"* responderia `None` sobre a moldura que interessa. Foi o defeito reportado —
*"não vejo em lugar nenhum a seção Frame"*.

A pergunta certa é o **INVERSO da expansão**, pela mesma relação de parentesco que a produziu:
*qual moldura selecionada contém tudo o que está selecionado?* Ela responde igual pelas duas
rotas de selecção, e é isso que impede a resposta de depender de POR ONDE o artista selecionou.

⚠️ E ela ficou MAIS simples em 2026-08-02, sem mudar de forma: uma **moldura** deixou de
emprestar os filhos (ela é uma forma, e restilizá-la restilizava seis), então hoje a moldura
costuma chegar aqui sozinha — e contém-se a si própria. Continua a ser o inverso da expansão
porque a expansão continua a existir *para grupos*.

O que ela recusa, e é o desenho:

- **um filho sozinho** não oferece a seção — o artista selecionou a forma, não o contêiner;
- **duas molduras irmãs** não têm resposta única (nenhuma contém a outra);
- **moldura + forma solta** também não: o chip editaria o que o artista não está a olhar.

Aninhadas, a de FORA contém as de dentro ⇒ ela é a resposta, que é a moldura em que o artista
de facto clicou.

## `paint_dilate.rs`

> ⭐⭐⭐ **O OFFSET DE CAD DE UMA CAMADA** — a silhueta dela cresce ou encolhe (v22).

Pedido do Enio, 2026-09-05: *"o offset do cad, contraindo e dilatando"*. É o *Offset Path* do
Illustrator aplicado a UM atributo da pilha de aparência, e é o que faz um adesivo, um rótulo
contornado ou um selo **numa forma só** — sem duplicar o objecto, que é a doença que a pilha
existe para curar.

# Por que este módulo existe (e não é feito no renderer)

O renderer é **sem estado** e o `path_tess` corre por forma e por quadro. Medido:

| direcção | motor | custo por camada |
|---|---|---|
| **crescer**, contorno único | [`ph2d_vec_boolean::offset_ring`] (Minkowski, sem booleana) | **~0,5 µs** |
| **encolher**, ou compound | [`ph2d_vec_boolean::offset_path`] (o sweep booleano) | **85–440 µs** |

Tesselar uma forma custa **~0,13 µs** (`encode_cost_by_n`: 10 k formas em 1,323 ms). ⇒ crescer é
da mesma ordem do desenho e **encolher é ~1 000× isso** — é o encolher que obriga ao memo, e é
por isso que ele mora aqui, do lado que tem estado entre quadros.

# ⚠️ A chave é LOCAL, e é isso que faz o memo sobreviver ao arrasto

O [`crate::contour_live`] coze em MUNDO e por isso **recoze a cada quadro** enquanto a forma é
arrastada (o próprio ficheiro dele declara sofrer disso). Aqui a distância é local
([`ph2d_vec_scene::PaintEntry::dilate`]), então a chave é a geometria COZIDA + a distância + a
quina — nenhuma delas muda ao mover a forma pelo canvas.

# ⛔ O cozimento passa pela porta do Contour, não por uma segunda

[`crate::contour_live::cook_piece`] é quem escolhe entre o anel e a booleana, com o domínio de
cada um medido e escrito. Uma segunda escolha aqui divergiria da dele no dia em que o domínio
do anel crescesse.

## `group_gizmo_view.rs`

> ⭐ **O gizmo de um objeto que não desenha nada** — o VAZIO e o GRUPO (Enio, 2026-08-26).

Até aqui `snapshots::build_view` respondia `None` para toda entidade sem geometria própria —
*«grupo/outro: sem gizmo próprio»* — e um objeto sem gizmo **não é agarrável de forma nenhuma**:
nem para mover, nem para girar, nem para escalar. O botão `Add` da Hierarquia (F3) nasce
exatamente assim (`Transform` + `Name` e mais nada), então o objeto que o artista acabou de
criar era o único do app que ele não podia pegar.

# A caixa é a DELE, sempre — e ter filhos não a muda

> *«O objeto vazio deve permanecer com seu gizmo original e não se utilizar do gizmo dos
> filhos.»* (Enio, 2026-08-26, 3.ª volta do smoke.)

⛔ **A 1.ª versão fazia a caixa ser a UNIÃO dos filhos visíveis** quando havia algum — a lei do
container de um `VecEnvelope` (ADR-0129 Fatia 3) generalizada. Está **construída, medida e
REJEITADA por veredito de produto**: um objeto vazio passava a ter um tamanho que não é dele, e
a caixa mudava sozinha sempre que um filho se mexia. ⚠️ *Um controlo cuja moldura muda quando o
artista não lhe tocou lê-se como o app a decidir por ele.* A árvore dessa versão sobrevive no
commit `828bc88f4`; ⛔ uma 2.ª tentativa começa perguntando **o que ficou pior**, não
reconstruindo.

⇒ a caixa é sempre o **marcador** do objeto, com meia-extensão derivada do tamanho da alça (ver
[`EMPTY_HALF_PX`]). Ela move/gira/escala o `Transform` DELE, e os filhos seguem por parentesco —
é isso, e não o tamanho da moldura, que faz o conjunto andar como um objeto só.

# ⛔ Quem já tem alças NÃO ganha caixa

Uma junta e uma roldana também não têm geometria, e chegam aqui pelo mesmo caminho — mas elas
**já publicam alças** (os dots de [`crate::render_loop::point_gizmo`]). Uma caixa por cima
registaria o interior dela como *Translate* no hit-index e **engoliria o clique nas alças**, que
são os controlos que aquelas entidades de facto têm. *É a mesma razão pela qual o conector e o
spine de um Blend não publicam gizmo* (ver o cabeçalho de [`ph2d_app_vec::vec_gizmo_view`]).

## `frame_spans.rs`

> **Os intervalos de RECORTE deste frame**, resolvidos contra a pilha de z que se vai desenhar.

A metade que a shell possui: ela sabe a ÁRVORE (o parentesco e o `ph2d_ecs::VecClipContent`
moram em entidades) e o renderer sabe DESENHAR, então o que atravessa a fronteira é um par de
ids por recorte — `ph2d_vec_scene::VecClipSpan`, dentro do `VecViewState` que já leva *"o que a
árvore diz sobre os paths"*.

⚠️ **"Moldura" aqui é história, não requisito** (2026-08-21): quem recorta é qualquer forma
FECHADA que carregue o `VecClipContent`, e o campo `frame` do `VecClipSpan` guarda o nome de
quando só a moldura podia. O que se lê abaixo vale igual para uma elipse e para uma tela.

# O que este módulo DEIXOU de produzir (2026-08-04)

Ele produzia um intervalo para **todo pai**, recortasse ou não, porque a pilha de z era o DFS
**invertido** — o pai desenhava na FRENTE dos filhos e o renderer tinha de **antecipar** o
desenho dele. Enio: *"o filho é desenhado por trás do pai … em game engines como Godot os
filhos são renderizados acima dos pais"*.

⚠️ A cura não foi um terceiro remendo: foi **inverter a projeção** (`vec_zorder::z_order`). O
filho passou a desenhar sobre o pai em TODA rota — incluindo a INSTÂNCIA de componente, que
percorre a cena e não tem renderer por trás, e por isso não tinha como herdar a antecipação
(*"ao criar a instância, os filhos que no mestre aparecem na frente dos pais vão para trás"*).
Com a antecipação morta, o intervalo voltou a ser o que o nome diz: **recorte**, e só de quem
recorta.

# O intervalo sai da pilha FINAL, e é isso que o Z global exige

O Z sobrepõe a árvore, então a sub-árvore de uma moldura **deixou de ser contígua** na pilha:
um filho com Z alto sai de dentro do intervalo e não é recortado. Resolver o intervalo contra o
DFS o descreveria onde a forma já não está — e um intervalo que aponta para o path errado é um
`pop_layer` no lugar errado.

## `smoke_bone_paint.rs`

> ⭐⭐⭐ **UM CANVAS DO PAINTER PRESO A OSSOS, E DOBRADO** — `PH2D_VEC_BONE_PAINT_SMOKE=1`.

# Por que esta cena existe

⛔⛔ **A cura das guias chatas (item 4 do dono, 2026-09-15) não tinha CENA.** A grelha, os selos de
operação e a caixa do gizmo passaram a seguir a arte dobrada, está tudo gateado — e **nenhuma cena
deste app punha o Painter a pintar por cima de arte dobrada por ossos**, logo o dono não tinha como
olhar para a correcção. *Uma cura que ninguém pode ver é uma cura que ninguém julga.*

⚠️ **Ela é da família do VECTOR e não da do Painter**, e não é arrumação: a `ph2d-app-painter` não
conhece o esqueleto — nem deve. Quem tem as duas metades é esta família, que já possui a cena dos
ossos ([`crate::smoke_bone`]).

# ⛔⛔ A 1.ª REDACÇÃO DESTA CENA FOI REPROVADA PELO DONO («ruim», 2026-09-15), E A CAUSA É ESTA

Ela montava um canvas **QUADRADO** de `512²` com os três ossos deitados ao meio dele, e a arte
saía **RASGADA**: lascas brancas e pedaços do traço pintado deslocados do resto.

⚠️⚠️ **Não era dobra a mais — era arte FORA DO ALCANCE.** O raio de um osso é o **comprimento
dele** vezes a `strength` ([`ph2d_skeleton::SkinBone::new`]), e um ponto fora do raio de TODO osso
é **órfão**: ele salta, em salto seco, para o osso mais próximo ([`ph2d_skeleton::Skin::weights_at`],
o *point binding* do Moho). Um salto seco num mapa contínuo é **um rasgo**, e é exactamente o que a
foto mostrava.

⛔⛔⛔ **A ARITMÉTICA PROÍBE O QUADRADO, e é por isso que afinar o ângulo nunca ia curar:** com `n`
ossos deitados ao longo da largura `W`, cada osso mede `W/n` e portanto **alcança `W/n`**; a arte
sobe `W/2` acima do eixo. `W/2 < W/n` só é verdade com `n < 2`. ⇒ *um quadrado com uma corrente de
três ossos tem uma banda órfã por construção, em qualquer ângulo e em qualquer tamanho.*

⚠️⚠️ **E eu citei a RÉGUA ERRADA ao escolher os `25°`.** O doc dizia que o ângulo estava «bem abaixo
do ponto em que o mapa dobra sobre si mesmo», e isso era **verdade e irrelevante**: a
[`ph2d_skeleton::fold`] mede **INVERSÃO**, e a coluna `inverted` lia `0,00 %` sobre a foto do rasgo.
Quem gritava era a coluna `orphan`, a **`33,85 %`** — a coluna que o doc daquela régua chama de
*anti-vacuidade* e que eu não olhei. *Uma régua com duas colunas responde a duas perguntas, e ler a
errada é ler zero sobre o defeito que está no ecrã.*

## A medição que decidiu a forma (3 ossos, `strength` no default do produto, dobra `25°`)

| canvas | meia-altura | alcance do osso | `det_min` | invertida | **ÓRFÃ** |
|---|---:|---:|---:|---:|---:|
| `512×512` (a 1.ª redacção) | 256 | 170,67 | 0,2622 | 0,00 % | **33,85 %** |
| `512×384` | 192 | 170,67 | 0,2573 | 0,00 % | **12,31 %** |
| `512×336` | 168 | 170,67 | 0,2114 | 0,00 % | 0,00 % |
| **`512×320`** (esta cena) | **160** | **170,67** | **0,2497** | **0,00 %** | **0,00 %** |
| `512×288` | 144 | 170,67 | 0,2510 | 0,00 % | 0,00 % |
| `512×352` | 176 | 170,67 | 0,1241 | 0,00 % | 4,12 % |

⛔⛔ **E a cura «óbvia» — alargar o alcance — foi MEDIDA e é PIOR pelo meio:** com o quadrado de
`512²`, a `strength 1.0` dá `det_min 0,2622` com `33,85 %` de órfãs, a `1.5` dá **`det_min −1,2829`
com `0,32 %` da arte DO AVESSO**, e só a `2.0` volta a limpar (`0,4750`). *Um alcance que cobre
metade da banda órfã mistura um osso que chega com um vizinho que salta, e a mistura inverte-se —
chegar a meio é pior que não chegar.* ⇒ a alavanca desta cena é a **FORMA da arte**, não um knob.

# O que ela monta, e por que cada peça está lá

| peça | o que ela serve |
|---|---|
| um **canvas branco opaco**, largo e baixo | é o sujeito do Painter; o branco é o fundo em que as guias se leem, e é ele que dá tinta à malha |
| **três ossos** deitados ao longo dele | três porque com dois a dobra tem um vinco só, e o que o olho julga é a CURVA |
| a **altura DERIVADA** do alcance de um osso | ver [`ALTURA_PX`] — é isto que impede a banda órfã, e não um número escolhido |
| o canvas **PRESO** a eles | é isto que faz a sprite ser desenhada como MALHA, e sem malha não há o que corrigir |
| os dois últimos ossos **rodados** | ⚠️ a dobra é feita DEPOIS de prender: o repouso é o instante do bind, e dobrar antes não dobraria nada |

⛔ **Nada mais é armado** — nem a ferramenta, nem o pincel, nem a grelha. É a lei que a cena da
máscara já escreveu: *uma cena que arma estado por baixo da mesa salta exactamente a costura que
ela devia provar, e esconde um default mau.*

# ⭐⭐⭐ O QUE ELA DEMONSTRA MUDOU EM 2026-09-15, POR ORDEM DO DONO

*«Inative a possibilidade de pintar sobre malha deformada por ossos. SE o usuário entrar no modo
Painter em imagem deformada por ossos a imagem deixa a deformação para ser pintada. Ao sair do
modo painter, ela retorna a deformação.»*

⇒ a cena passou a demonstrar o **ciclo** (dobrado → achata ao pegar no pincel → pinta → dobra
outra vez ao largá-lo, com a tinta a dobrar junto), e **não** mais as guias a seguir a curva.

⚠️⚠️ **E a consequência está NOMEADA e é do dono decidir:** o chrome do Painter que passou a
seguir a arte dobrada (a grelha, os selos, os gizmos, os contornos das formas — item 4) **deixa
de ter sujeito enquanto se pinta**, porque durante a pintura não há dobra nenhuma. ⛔ Ele não foi
apagado e não custa nada: sem malha o [`ph2d_app_painter::canvas_map::CanvasMap`] degenera no
afim do quad, **ao bit**. Fica **dormente** — e volta a valer no dia em que pintar sobre a dobra
for permitido. *Um roteiro de smoke que continuasse a mandar procurar a curva debaixo do pincel
seria uma cena a ensinar o contrário do que acontece.*

# ⭐⭐⭐ E EM 2026-09-16 A ORDEM VIROU REGRA (fila do esqueleto, F6-s)

*Toda* ferramenta que pinta, apaga ou deforma pixels endireita a imagem (a Remoção de fundo
incluída), **menos o Liquify**, que trabalha sobre a dobra. ⇒ o roteiro passou a ter as TRÊS
metades: o Painter endireita, o Liquify volta a dobrar (e o anel dele segue a dobra — o chrome
dormente acorda ali), a Remoção de fundo endireita outra vez. ⭐ E as de TAMANHO/MARGEM (a
resposta do dono no mesmo dia): o Padding endireita e avisa; o Make Square — o canvas desta cena
não é quadrado — solta a imagem dos ossos, e o Ctrl+Z devolve-a.

⚠️ **Se a linha `[bone-paint-smoke]` não aparecer, PARE:** a cena não montou.

## `morph_machine_drive.rs`

> **A MÁQUINA DE MORPH A CORRER** (plano 32 W5) — quem faz a forma virar a outra.

# ⚠️ Ela só corre num MODO, e isso não é conservadorismo

A condição de uma seta é uma **acção do Input Map**, isto é, uma tecla. Se a máquina escutasse
enquanto o artista edita, carregar em `Z` faria a forma mudar **e** o que quer que o `Z` faça no
editor — os dois, sem que nada na tela explicasse. É o argumento do [`crate::render_loop::ui_preview`]
(*"um hover que animasse a forma enquanto o artista trabalha tornaria o editor inutilizável"*)
com outro dispositivo de entrada, e a resposta é a mesma: **um modo**.

# ⛔⛔ O PLAYHEAD ERA A PORTA E DEIXOU DE SER — e a diferença é uma medição, não um gosto

A W5 escreveu que *"o modo já existe: neste editor, o jogo a correr é o playhead a andar"*, e
era um argumento bom sobre a coisa errada. **O playhead não tranca o teclado do editor.** Com
ele a andar, as teclas continuam a chegar aos atalhos — então a mesma tecla morfa a forma **e**
faz o que ela faz no editor, que é exactamente o que a nota dizia estar a evitar.

Enio, 2026-08-25, depois do smoke: *"precisamos de um modo preview (com botão) como o de states
de animação pois senão temos conflitos de atalhos (como setas do teclado movendo as formas)"*.

⇒ a porta é o **interruptor `Preview`** da seção *Morph States*, e ele **toma o teclado**
(`input_dispatch::keyboard`, logo depois do retrato dos dispositivos e antes de todo atalho).
⛔ **Uma porta, não duas:** deixar o playhead a dirigir também manteria o conflito viva na porta
que não tranca nada — e *duas portas para o mesmo modo divergem em silêncio*.

⚠️ *Um modo cuja entrada não exclui os outros consumidores não é um modo — é mais um produtor.*

# ⚠️ O que ela escreve é PRÉ-VISUALIZAÇÃO, e o undo não a vê

A máquina escreve **dois** campos do `VecMorph` — o par e o `t` —, e os dois passam pelo ledger
([`crate::preview_drive`]). ⛔ **O `Driver::MorphT` sozinho não bastava:** ele cobre o `t` e
**só** o `t`, e sem o `MorphPair` uma transição durante a reprodução entraria no undo como se o
artista tivesse re-ligado as fontes à mão.

# ⚠️⚠️ E parar o relógio NÃO devolve a forma autorada — a nota anterior estava ERRADA

Ela dizia *"ao largar as máquinas a cena volta ao que o artista desenhou"*. **Não volta**, e a
lei que manda é a da [`ph2d_preview_drive::PreviewDrive::settle`]: o ledger repõe o autorado
**dentro da fotografia** enquanto o motor conduz, e no primeiro quadro em que ele **para** a
entrada morre ⇒ a captura seguinte vê o vivo, difere do baseline e regista **UM** passo. É o
*«desfaz a corrida»*, e aqui ele significa: *sair do modo COMPROMETE a forma em que se ficou*.

⚠️ **Isso é o comportamento certo** (o artista carrega em ▶, sai, e o objecto ficou onde ele o
pôs — desfazível num Ctrl+Z) **e tem uma consequência que custou o report de 2026-08-26**: a
máquina seguinte nasce **depois** de o componente já estar noutra forma. Por isso ela é
**semeada pelo mundo** ([`open`]) e não por `graph.start()`.

## `smoke_bone_envelope.rs`

> ⭐⭐⭐ **A CENA DO ENVELOPE** — `PH2D_VEC_BONE_SMOKE=2`, a cena que o dono pediu por escrito
> (2026-09-18: *«Testei aqui e não vi em nenhum dos casos o envelope fazer diferença na
> deformação. […] Melhor montar uma cena específica para me mostrar isso»*).

# ⛔⛔⛔ O report estava CERTO, e mais certo do que a minha resposta lhe deu

Eu respondi-lhe que *«numa forma vectorial o envelope manda como sempre»*, e a medição derruba-o:
o `ph2d_vec_skin::pesos::pesos_do_caminho` resolve os mesmos *Bounded Biharmonic Weights* sobre o
INTERIOR de um contorno fechado desde 2026-09-15 — logo o alcance é inerte ali também. Medido
pela porta do produto, variando o `strength` do osso do meio de `0,1` a `8,0`
([`ph2d_skeleton_live`], `sonda_do_envelope_no_vector_tests`):

| forma | fechada? | amplitude da deformação |
|---|---|---:|
| `Rectangle` · `Ellipse` · `Star` · `Polygon` | fechada | **`0,000000`** |
| `Line` · `Arc` · `Spiral` | ABERTA | `2,03` · `4,25` · `2,05` |

⇒ **a mídia nunca foi a pergunta certa.** O envelope manda onde o padrão-ouro **não resolveu**, e
isso acontece num sítio só: um caminho **ABERTO** não tem interior, logo não tem domínio para a
energia, logo não tem pesos guardados — e a deformação cai na lei euclidiana, que é a que lê o
alcance.

# O que a cena monta, e por que cada peça está lá

Três fileiras, a MESMA corda, a MESMA corrente de três ossos, a MESMA dobra. Só o que está
escrito na coluna da direita muda:

| fileira | o que ela é | o envelope |
|---|---|---|
| **`Corda (alcance 1)`** | um traço ABERTO, com o alcance de fábrica | VIVO — mancha e alça à vista |
| **`Corda (alcance 4)`** | o MESMO traço, com o alcance do osso do meio em `4` | VIVO — e a corda sai **noutro sítio** |
| **`Barra preenchida`** | a mesma faixa, FECHADA | INERTE — sem mancha e sem alça |

⭐⭐ **As duas primeiras são um par, e é ele que responde ao report:** elas diferem num número só,
logo *a diferença que se vê entre as duas É o envelope*. ⛔ Uma cena com uma corda só obrigaria o
dono a confiar na memória da pose anterior, que é exactamente o que falhou no smoke dele.

⚠️ **A terceira fileira é a metade NEGATIVA, e vale tanto como as outras duas:** ela é onde a
segunda queixa dele (*«o gizmo do envelope fica sempre visível mesmo quando não é usado?»*) se
lê curada — ali o osso do meio não tem mancha nenhuma, porque ali o alcance não decide nada.

# ⚠️ Os números desta cena são DERIVADOS, e a tabela está no código

A dobra, o comprimento da corda e o alcance forte saem de uma varredura de legibilidade, não de
gosto — ver [`DOBRA`], [`CORDA_M`] e [`ALCANCE_FORTE`].

## `smoke_appearance.rs`

> ⭐⭐⭐ **A FORMA TEM OPACIDADE E MISTURA PRÓPRIAS** — `PH2D_VEC_APPEARANCE_SMOKE=1`.

# O que a cena fecha

Até 2026-09-05 uma forma vectorial não tinha nem uma nem outra: o painel oferecia duas rows
`Opacity` que são o **alfa da tinta que a ferramenta tem na mão** (elas semeiam a forma seguinte
e re-vestem a selecção), e mistura não havia em lado nenhum — `grep blend_mode` nas crates do
vector dava vazio. É o item 2 do estudo 42, e a metade que faltava ao report do Enio de 04/09
(*"o painel não mostra..."* — não havia o que mostrar).

# A cena (unidades de mundo, ~±9)

Uma **faixa de fundo** com três blocos de cor (frio · quente · claro) e, por cima dela, uma
fileira de discos — cada um com um **modo de mistura** diferente, na ordem em que o dropdown os
oferece. Mais dois quadrados iguais à direita: o de cima **opaco**, o de baixo a **40 %**.

⚠️ **O fundo é load-bearing:** um modo de mistura só existe contra o que está por baixo. Sobre
o fundo da tela todos os modos desenham a mesma coisa, e a cena ensinaria que a feature não
funciona.

# O que provar

1. **Os discos não são todos iguais** — cada um compõe-se com a faixa de outra maneira (o
   `Multiply` escurece, o `Screen`/`Add` clareiam, o `Difference` inverte).
2. **O quadrado de baixo é translúcido** e o de cima não, com a MESMA cor autorada: a
   opacidade é do OBJECTO, e não da tinta.
3. **O painel edita as duas** — pegue a ferramenta Vector, clique numa forma, e a seção
   **Appearance** aparece com *Opacity* e *Blend*. Arrastar o slider muda a forma na hora;
   escolher outro modo na lista muda como ela se compõe.
4. **Ctrl+Z desfaz** o arrasto inteiro num passo só.

⚠️ Se a linha `[vec-appearance-smoke]` não aparecer, PARE: a cena não montou.

## `envelope_live.rs`

> **O host do Envelope vivo** (ADR-0129) — um **container** cujos filhos têm a geometria deformada
> por uma gaiola de 4 cantos comum.

Espelho do [`crate::morph_live`], e diferente no que deforma: o morph interpola DUAS fontes e
cacheia um `Plan`; o envelope deforma N fontes (1..) por um mapa `R2→R2` (a homografia do gesto
Quad, [`ph2d_vec_envelope::QuadWarp`]). A entidade que carrega o [`VecEnvelope`] é um **container**
(sem `VecPathRef`); cada FILHO tem o `VecPath` da forma **cozida** (deformada) na cena, re-escrita
*em lugar* a cada frame.

# Um só modelo: o container sempre (`N=1` inclusive)

Um envelope de uma forma só é o caso `N=1`; um de várias é o *warp group* do Affinity. Modelar o
de-um como path-com-componente e o de-vários como container seriam DOIS modelos do mesmo fato — e
recook/gesto/gizmo teriam de perguntar "qual dos dois é este?" em cada sítio. Aqui há um só: o
**container**, com 1..N filhos.

# A fonte + a gaiola vivem em LOCAL do CONTAINER; a POSE vive no `Transform` (ADR-0111)

Ao criar, cada forma-fonte tem a sua geometria COZIDA **assada em MUNDO** e guardada como a fonte
do filho; o filho é reparentado sob o container e posto na **identidade** (`Transform::IDENTITY`),
e o container nasce na identidade também — então, no nascimento, **container-local == mundo**. O
recook desserializa cada fonte, deforma pela gaiola (uma só, comum) e escreve o resultado LOCAL no
path de cada filho. Quem leva tudo ao MUNDO é o `Transform` do **container** — e é por isso que o
**gizmo de sprite move/gira/escala o envelope inteiro no Select** (Fatia 2): a geometria dos
filhos é estável, só a pose do container se move, e o recook **preserva a pose**.

⚠️ **Por que não dobra (ADR §3.3):** o Blend dobra porque a geometria dele depende de FONTES que
se movem; a do envelope é função pura de `corners`+fontes FIXOS, então no Select nada a muda e o
gizmo age sobre uma bbox estável. Arrastar os cantos (que MUDA a geometria) é Node-only. O
`settle_origins` PULA os filhos (têm `ChildOf` — geometria já em mundo) e o container (não está no
mapa de paths).

## `width_handles.rs`

> **O WIDTH TOOL** — as alças de largura na curva (plano 25 §5, ADR-0148).

O W1c deu a representação (a lista de paradas) e o motor vivo; o W1d deu uma fonte automática
(o gesto do lápis). Este é o terceiro escritor do MESMO perfil: **a mão do artista, ponto a
ponto** — o *Width Tool* do Illustrator.

# O gesto, e por que ele é um MODO

Uma alça por parada, **fora da curva**, à distância que a fita de facto tem ali. Arrastar:
afastar da curva ENGROSSA, aproximar AFINA, e andar ao longo dela MOVE a parada. Clicar na
curva onde não há alça **acrescenta** uma parada ali; o botão direito sobre uma alça a
**apaga**.

É um modo (o pill **Width**) e não uma alça escondida no Node pela razão que o Fillet/Chamfer
já pagou: no Node as alças competiriam com as âncoras — e uma parada de multiplicador pequeno
senta a milímetros da curva, ou seja em cima delas. O Illustrator faz o mesmo (Shift+W).

# Nada aqui inventa uma segunda maneira de guardar a largura

Toda escrita passa por [`crate::profile_live::arm`], a MESMA porta dos quatro sliders e do
lápis. O que muda é só quem calcula a lista — e é por isso que o preview, o Apply e as três
rotas de autoria não podem divergir: há um motor (`power_stroke_layers`) e um lugar onde o
perfil mora (o componente).

# A primeira alça nasce do NEUTRO

Numa forma sem perfil, o 1º clique cria a lista uniforme (duas paradas, `1.0` nas pontas) e
**só então** arrasta — o desenho não salta no toque. É a mesma lei do Fillet, que transforma um
ponto suave em quina antes de arredondá-lo: *a ferramenta faz, dentro do gesto, o passo que o
artista faria à mão*.

## `smoke_svg.rs`

> ⭐⭐⭐ **IMPORTAR SVG** — `PH2D_VEC_SVG_SMOKE=1` (estudo 42, item 3).

# O que a cena fecha

Até 2026-09-05 este app **exportava** uma curva e não sabia **ler** nenhuma: o
`ph2d-imageio-svg` validava o ficheiro e devolvia um documento vazio, *"intentionally empty"*.
Nenhum acervo de artista entrava.

# ⭐ A cena ESCREVE o próprio `.svg`

É a lei do smoke do `.ase`: testar o importador não pode depender de o operador ter um ficheiro
à mão. Ele é escrito no directório temporário, o caminho é impresso (para o Enio o poder
**arrastar outra vez**, ou abrir noutro programa e comparar) e depois entra pela **mesma porta**
que o arrastar-e-largar usa — ⛔ não por um atalho de teste.

# O que o ficheiro carrega, e porquê cada coisa

| No ficheiro | O que prova |
|---|---|
| Um triângulo com a ponta em CIMA | a lei dos eixos (o SVG mede o Y ao contrário do mundo) |
| **Dois** quadrados dentro de dois `<g transform>` | a pose ANINHADA chega certa, e o `<g id>` vira um GRUPO na Hierarquia |
| Uma curva `Q` (quadrática) | a elevação a cúbica é exacta |
| Um `<linearGradient>` | a rampa viaja com a geometria |
| Um traço `stroke-dasharray` | o tracejado entra em múltiplos da largura |
| Um `<g opacity mix-blend-mode>` sobre uma barra | a opacidade e a mistura da v19 do schema |
| Um `<text>` | ⛔ o que NÃO entra sai NOMEADO, e não em silêncio |

⚠️ **O `grupo-aninhado` tem DOIS filhos de propósito.** O verbo de agrupar desta casa exige dois
membros (a mesma regra que o artista lê no menu), então um `<g>` com um filho só é achatado — e
a cena não mostraria grupo nenhum na Hierarquia, que é metade do que ela existe para provar.

## `vec_clip_edit.rs`

> **O RECORTE da seleção** — a projeção que o painel lê, e o chip que a edita.

Módulo irmão do [`crate::vec_frame_edit`], e o corte é por PERGUNTA, não por tamanho: aquele
responde *"qual é a moldura desta seleção?"* (a seção Frame, o auto layout, as âncoras), este
responde *"quem recorta nesta seleção?"*. Foram a mesma pergunta enquanto só uma moldura podia
recortar; deixaram de ser em 2026-08-21, quando o Enio pediu *"a feature Clip Content para
qualquer forma vetorial fechada"*.

# A regra de elegibilidade é a MESMA, com um candidato a mais

*Qual forma selecionada contém tudo o que está selecionado?* — o inverso da expansão de
seleção, herdado do `vec_frame_edit` inteiro (inclusive o `is_within`, que é dele). O que muda
é só o filtro do candidato: era *"tem `VecFrame`"*, é *"o caminho é FECHADO"*.

O que ela recusa segue valendo, e pelas mesmas razões: **um filho sozinho** não oferece o chip
(o artista selecionou a forma, não o contêiner); **dois irmãos** não têm resposta única;
**contêiner + forma solta** editaria o que o artista não está a olhar.

# Por que FECHADO

⚠️ Um caminho aberto não tem interior. O Vello recortaria à mesma — ele fecha a região
implicitamente para preencher — mas por uma silhueta que o artista **não desenhou**, e o
resultado seria arte a sumir atrás de uma fronteira invisível. É o mesmo predicado
(`VecPath::closed`) com que a booleana viva e o Blend recusam caminhos abertos, e por isso o
filtro vive AQUI, na fronteira de autoria, e não no componente: o `VecClipContent` descreve
uma intenção, e quem decide onde ela pode ser expressa é quem oferece o controlo.

## `smoke_bone.rs`

> ⭐⭐⭐ **O DESENHO GANHA OSSOS** — `PH2D_VEC_BONE_SMOKE=1` (estudo 42, item 5).

# O que a cena fecha

Até 2026-09-06 um personagem deste editor só animava como **recorte de papel**: a timeline movia
a POSE de uma forma inteira, e mais nada. Dobrar um braço exigia desenhá-lo outra vez.

# A cena, e o que cada peça prova

| O quê | O que ela prova |
|---|---|
| **O BRAÇO** (barra com 3 ossos, já presa) | girar um osso DOBRA o desenho, e a base fica onde está |
| **O TENTÁCULO** (barra com 6 ossos, já presa) | a cadeia inteira: girar a RAIZ leva tudo, girar a PONTA leva só a ponta — a cinemática é a hierarquia |
| **A FOLHA SOLTA** (uma forma e um esqueleto **não ligados**) | o gesto do *Bind*: seleccionar a forma, carregar no botão, e ela passa a obedecer |
| **A BIFURCAÇÃO** (dois esqueletos pequenos, no meio da metade de baixo) | o `From Chain` curva o osso até ao filho — e com DOIS filhos a ponta fica recta (a pergunta ao dono de 2026-09-16) |

⚠️ **As duas primeiras são ligadas pela cena**, e é de propósito: sem uma peça que já obedece, o
primeiro gesto do artista seria montar um rig do zero para só então descobrir se ele funciona.

⚠️ Se a linha `[vec-bone-smoke]` não aparecer, PARE: a cena não montou.

# ⚠️ Ela monta em DOIS tempos, e não é conforto

Prender uma forma exige a **entidade** dela, e quem a cria é o `vec_entities::sync`, que corre
**depois** deste prólogo. Ligar no mesmo quadro em que se desenha encontraria o mapa vazio e
prenderia zero formas — em silêncio.

## `layout_edit.rs`

> **O AUTO LAYOUT da seleção** — a projeção que o painel lê, e a porta que o edita (plano UI/UX
> W2, ADR-0153).

Irmão do [`crate::vec_frame_edit`], e com a mesma divisão de donos: a verdade mora no ECS
(`VecLayout` no pai, `VecLayoutItem` no filho) e isto é o que a shell publica por frame. O
painel não alcança o mundo — se alcançasse, a resposta que DESENHA o chip aceso divergiria da
que HONRA o clique.

# Uma tabela, duas direções

⚠️ Os três rádios (direção · alinhamento · distribuição) precisam da mesma correspondência
`enum ↔ chip` lida nos dois sentidos: **para publicar** o aceso e **para resolver** o clique.
Escrevê-la duas vezes — um `match` que pinta e outro que honra — é como uma variante nova entra
só numa delas, e o sintoma é um chip que acende e não faz nada. Aqui cada rádio é **um array**,
e as duas perguntas são varreduras dele.

# Quem é o SUJEITO da seção

Duas metades, resolvidas por portas diferentes porque são perguntas diferentes:

- **a moldura** é a que CONTÉM a seleção — a MESMA porta do W0
  ([`crate::vec_frame_edit::frame_of_selection`]), que já sabe desfazer a expansão de seleção
  dos contêineres;
- **o filho** é a seleção quando ela é UMA forma cujo pai flui. Uma moldura aninhada responde
  às duas, e é isso que faz os dois blocos aparecerem juntos nela.

## `ui_state_bridge.rs`

> **A ponte dos ESTADOS de UI** (plano UI/UX W7) — quem faz a cena ANDAR entre duas poses.

# O que esta ponte dirige, e o que ela deliberadamente NÃO dirige

Ela dirige o botão **Show**: o artista pede um papel e a cena **transita** até ele, com a
duração que ele autorou. É assim que ele vê o tween que gravou, em vez de apenas confiar que
ele existe.

⚠️ **Ela não é dirigida pelo mouse DAQUI, e as duas razões continuam de pé:** um hover que
animasse a forma enquanto o artista trabalha tornaria o editor inutilizável (é por isso que o
Figma põe a interação num **modo de apresentação** separado), e o undo deste editor é por
**DIFF do mundo**, então uma pose escrita por hover viraria passo de undo a cada passagem do
rato.

⇒ quem liga o rato é o **MODO DE PREVIEW** ([`super::ui_preview`], W7r): ele resolve as duas
de uma vez — enquanto corre, o gesto de edição não existe e o undo não regista, e ao sair o
mundo volta exactamente ao que era. Ele PEDE por esta mesma porta ([`request`]), então não há
um segundo caminho para *"pôr a cena nesta pose"*.

# O relógio é o do FRAME, e não um relógio próprio

`advance(dt)` recebe o `dt` que o `FixedStep` já produziu. É a lição W4.T7 do Motion, onde o
`MotionTransport` **morreu**: dois relógios divergem, e o modo de falha é a UI a andar noutra
velocidade que a cena.

## `snap_labels.rs`

> **O NÚMERO do smart guide** — quanto vale a distância que a guia desenha
> (plano 25 §9, o último item da tabela da W6).

O segmento tracejado entre o ponto que encaixou e o ponto em que ele encaixou
já dizia *com o quê* a forma alinhou. O que faltava era **quanto** — a
pergunta que faz de um encaixe uma medição, e a razão de a W6 se chamar
PRECISÃO.

# Três donos, e NENHUM deles é este arquivo

- **quais guias merecem número e onde ele pousa** — [`ph2d_vec_render::snap_labels`],
  geometria pura, sem noção de unidade;
- **que número, com que casas** — [`ph2d_editor_core::LengthDisplay`], a porta única
  que a régua e o painel de Grid Snap também usam;
- **como uma ficha é desenhada** — [`ph2d_editor_core::readout`], desde a C3 do estudo da UI viva.

⚠️ O terceiro item dizia *«aqui, e só aqui»* até esta ficha ter deixado de ser a única: o corpo,
a altura, o raio e as cores mudaram-se para a porta comum, e o que sobrou aqui é a largura
MÍNIMA — o único número que é desta ficha e de mais nenhuma. Este arquivo existe porque desenhar
exige `TextSystem` e a cena do frame; ele não decide nada que os outros decidem.

# O sufixo

⚠️ A régua imprime o número **NU** e este imprime **com sufixo** (`px`/`m`), e
a diferença não é gosto: uma régua é entendida pela faixa graduada em que ela
vive, e esta ficha **paira sobre a arte** sem eixo nenhum ao lado que a
explique. É também o que torna a unidade ativa visível sem abrir o menu.

## `pattern_live.rs`

> **O Pattern Along Path VIVO na shell** — o cozimento do [`ph2d_ecs::VecPatternPath`] (plano 23).

Espelho do [`crate::offset_live`]: o componente guarda a **relação** (a que caminho, com que
espaçamento, de que lado) e as cópias são uma **função pura** dela, re-cozidas aqui por frame
numa [`LiveGeometry`] e desenhadas por [`ph2d_vec_render::dispatch`] **no z do motivo**. A curva
do motivo nunca é tocada — é ela que o modo Node edita; o que se vê são as cópias.

# Duas entradas, e onde cada uma vem

Um pattern precisa de DUAS geometrias: o **motivo** (a forma a estampar) e o **caminho-guia**.

- O **guia** entra pela porta única [`crate::guide::guide_arc`] — cozido, assado em MUNDO,
  virado em [`ph2d_vec_scene::arc_path::ArcPath`]. É a MESMA porta do texto em caminho.
- O **motivo** é a `cooked()` do próprio path, em espaço **LOCAL** (a forma, não a pose). O
  motor a recentra pelo bbox e a estampa nos pontos do guia. ⚠️ **A pose do motivo é ignorada
  de propósito** (v1, plano 23 §3): as cópias substituem o desenho do motivo, então mexer no
  gizmo dele não teria o que mostrar — é o mesmo *"mover o objeto vinculado não quer dizer
  nada"* do texto e do conector. Para mudar o tamanho das cópias, edite os nós do motivo (a
  fonte) ou o Spacing; escalar por gizmo é decisão de produto adiada.

# Sem memo (v1), e é MEDIDO

O [`crate::offset_live`] memoiza porque `offset_path` mede 0,4–1 ms. O [`pattern_along`] mede
**0,597 ms para 200 cópias** (plano 23 §0), e uma cena típica tem muito menos. Uma cena PARADA
com um pattern pagaria isso por frame — dentro do orçamento (3,6% de um frame de 60 fps), e não
medido como problema. Memoizar exige detectar a mudança do guia (cozido + assado), e adicioná-lo
sem uma medição que o justifique é otimização prematura ([[project_m5_perf_validated]]).

## `widget_value.rs`

> **A POSIÇÃO de um controle autorado sobrevive ao arquivo** (plano UI/UX W8b.4).

Irmão do [`crate::widget_drive`], e o corte entre os dois é por PERGUNTA: ali mora *o que
este controle DIZ à forma*, aqui *que posição este controle GUARDA*. Não são a mesma coisa e
não se derivam uma da outra — um slider que o artista ainda não prendeu a forma nenhuma tem
posição, e a opacidade é quantizada em 255 degraus que não descrevem um slider contínuo.

# O reconcile, e por que ele tem DUAS direções

O valor tem dois escritores legítimos, e ignorar qualquer um deles quebra alguma coisa:

- o **PONTEIRO** escreve o `WidgetStore` (arrastar o slider);
- o **MUNDO** escreve o componente (abrir um projeto · um Ctrl+Z · o smoke a semear a cena).

⚠️ Um sentido só não basta, e as duas metades falham de formas diferentes: sem *store →
componente* nada é salvo; sem *componente → store* o Ctrl+Z devolve o componente antigo e o
painel continua a mostrar a posição nova — o controle e a arte discordariam sobre o mesmo
número, na tela, sem nada dizer por quê.

A desempate é um memo do que **já foi propagado** (`applied`), e a ordem é lei: **o artista
ganha**. Se o store difere do memo, foi o ponteiro; senão, se o componente difere do memo, foi
o mundo. Sem o memo os dois lados se sobrescreveriam em frames alternados e o slider tremeria.

⚠️ **Escrever o componente é uma EDIÇÃO** — o undo global é por DIFF do mundo, então arrastar o
slider vira um passo de undo. É o correto: mover um controle é um ato do artista, e o
`post_frame_undo` já suprime enquanto o botão está preso ⇒ **um passo por gesto**, não por
frame. (O que continua fora do undo é a ARTE: ela não é escrita por ninguém aqui.)

## `connector_panel.rs`

> **O painel edita o conector** — módulo irmão de [`crate::connector_live`] (que cuida do
> COZIMENTO da rota) e espelho exato de [`crate::shape_params`] (que faz o mesmo pela
> forma viva).

O Enio pediu para calibrar `jetty` e `spread` **com o olho**. Isso impõe três coisas, e
nenhuma delas é decoração:

1. **O painel mostra o valor EFETIVO.** Os dois campos são `Option<f32>` no componente:
   `None` = automático (o jetty acompanha o tamanho das caixas; o spread, o índice
   paralelo). Um campo que nascesse vazio — ou em zero — faria o número SALTAR no
   primeiro toque, para longe da linha que está na tela. Então a shell **resolve** o
   efetivo aqui ([`effective`]) e publica; o painel só desenha.
2. **Editar FIXA** (`None` → `Some(v)`): o auto→override de qualquer editor.
3. **A edição atinge TODOS os conectores selecionados** ([`edit_selected_connectors`]) —
   é o que permite calibrar o diagrama inteiro de uma vez, em vez de linha por linha.

# A regra que este módulo existe para honrar: semente = amostra

O número que o painel EXIBE e o número que o cozimento USA têm de sair da **mesma
função**. Duas cópias da fórmula do automático = o painel mentindo no dia em que alguém
ajustar uma delas (é a lição `feedback_derived_coordinate_seed_must_match_sample` — o
tempo remapeado quebrou três vezes exatamente assim). Por isso a derivação vive em
`ph2d_tool_vector::connector` (`auto_jetty` + `SPREAD_STEP`) e AMBOS os lados a
importam: aqui, para exibir; o `connector_live`, para cozinhar.

## `smoke_bone_media.rs`

> ⭐⭐⭐ **AS TRÊS MÍDIAS PRESAS AO MESMO GESTO** — `PH2D_VEC_BONE_MEDIA_SMOKE=1` (F11, ordem do
> dono de 2026-09-17: *«vamos lá: imagens em 9 fatias e folhas de quadros»*).

Três imagens lado a lado, cada uma presa à corrente dela e dobrada pelo MESMO ângulo:

1. **simples** — o CONTROLO, que já dobrava antes desta wave;
2. **folha de quadros** (`4×1`, a mostrar o quadro `1`);
3. **9-slice** esticado ao triplo da largura.

# ⛔ Porque as três, e não só as duas que a wave curou

⚠️ **O que a `2` e a `3` faziam antes era INVISÍVEL sem o controlo ao lado.** A folha dobrava —
**errado**, e sem aviso nenhum (a silhueta era a dos quatro quadros espremida no sítio de um), e
o 9-slice não dobrava de todo. *Sem a imagem simples ao lado, «dobrou» e «dobrou certo» leem-se
igual.*

# ⚠️ A arte de cada uma é escolhida para tornar o defeito LEGÍVEL

- A folha tem **a MORDIDA num lado diferente por quadro**, e a união das quatro é o disco
  inteiro. Com quatro formas iguais a silhueta errada seria indistinguível da certa — *uma cena
  cuja arte não distingue os dois estados não prova nenhum*.
- O 9-slice é uma **moldura oca**: o miolo é transparente, logo vê-se se o pedaço do meio foi
  desenhado onde não devia.

## `shape_params.rs`

> O painel edita a forma VIVA — módulo irmão de [`crate::shape_live`] (que cuida do
> COZIMENTO e do nascimento da forma; separados pelo teto de 600 LOC, HR-18).

O mesmo modelo do texto (`vec_text_object` (na shell)): uma config do painel tem um
**ALVO**. Aqui o alvo é a forma paramétrica SELECIONADA — os campos do painel deixam
de ser só o *default de desenho* e passam a editar a forma que está na tela, **mesmo
na ferramenta Select**. É o que fecha o ciclo Live Shape: desenhar um polígono de 5
lados e depois torná-lo de 7.

**Tudo aqui é genérico sobre o catálogo.** Nenhuma função conhece estrela, seta ou
balão: elas falam `(ShapeKind, ShapeValues)` e consultam o descritor. Uma forma nova
entra no catálogo e ganha painel, semente, edição, undo e save sem tocar neste arquivo.

Três peças, na ordem em que a shell as usa a cada frame:

1. [`shape_field_target`] — de quem são os campos (a 1ª forma paramétrica da seleção, ou
   NINGUÉM quando o artista está armado para desenhar). A porta única; [`panel_shape_target`]
   é a metade dela que só sabe olhar a seleção.
2. [`shape_seed_focus`] + [`seed_shape_fields`] + [`ui_values_of`] — quando o PAR
   `(alvo, tipo)` muda, os campos são semeados e a tool os ADOTA (painel, tool e objeto
   passam a concordar; e a próxima forma desenhada os herda — modelo Figma).
3. [`is_shape_field_id`] + [`apply_shape_field`] + [`edit_selected_shape`] — o
   `SetValue` de um campo muta o `VecShape` e RE-COZINHA a geometria in-place.

## `live_memo_probe.rs`

> SONDA (`--ignored`): **quanto custa mover uma forma que tem geometria VIVA?**

# A pergunta

Todo produtor de `LiveGeometry` memoiza, e a §11 do plano 25 afirma que **a chave é o
MUNDO — que é exatamente o que a animação move**. Se a afirmação valer, então arrastar (ou
animar) uma forma com Contour/Offset/Perfil/Simetria re-cozinha o efeito **em todo quadro**,
e o memo protege só a cena PARADA.

⚠️ **A sonda mede pela porta do PRODUTO** (`recook`, o que o frame chama), nunca por um laço
próprio sobre o kernel: esta casa já pagou três vezes por medir uma peça isolada e chamar o
número de produto (doc 28 §5.40 do Painter, e a decomposição do `build_flow_field`).

# As duas colunas, e por que a comparação é honesta

- **PARADO** — `recook` N vezes com a MESMA pose. É o que o memo promete proteger.
- **ANIMADO** — `recook` N vezes com a pose a TRANSLADAR. É o que a timeline faz a 60 Hz.

Uma translação **não muda a forma do efeito**: o contorno de uma estrela deslocada é o
contorno da estrela, deslocado. Então toda diferença entre as duas colunas é trabalho que a
resposta não precisou.

Rode: `cargo test -p ph2d-host-desktop --release live_memo -- --ignored --nocapture`

## `ui_state_edit.rs`

> **Os verbos dos ESTADOS de UI** (plano UI/UX W7) — irmão do [`crate::widget_edit`], mesma
> divisão: o painel PEDE, a shell FAZ, e o que muda é o documento.

# O que um estado GRAVA, e por que a sub-árvore inteira

Um botão não é uma forma: é um retângulo, um rótulo e talvez um ícone. Gravar só o hospedeiro
deixaria de fora justamente o que se move num hover. Então um estado captura o hospedeiro **e
cada descendente que é uma forma**, que é o mesmo conjunto que a booleana, o gizmo e o z-order
já chamam de *a sub-árvore* — [`ph2d_vec_entities::entities::subtree_paths`], nunca uma travessia
própria.

# O `Transform` LOCAL, nunca o de mundo

Uma pose é relativa ao hospedeiro: arrastar o botão inteiro para outro canto da tela **não pode
invalidar os estados dele**. O local é exatamente essa relação, e é o que a hierarquia já
guarda — pedir mundo aqui obrigaria a re-derivar contra a pose do pai a cada gravação, e a
primeira vez que alguém movesse o pai os dois números discordariam.

⚠️ **`skew` fica de fora, e a ausência é decisão:** a pose interpola por T/R/S decompostos
([`ph2d_ui_state::ObjectPose`]) justamente para não lerpar matriz; um skew autorado sobrevive
porque nada aqui o escreve, mas ele **não anima** entre dois estados. Nomeado em vez de
descoberto.

## `smoke_bone_losango.rs`

> ⭐⭐ **O LOSANGO DO PAINEL DO ESQUELETO** — `PH2D_VEC_BONE_SMOKE=8` (F40).

O que está na tela, e o que cada coisa é:

| O quê | O que é |
|---|---|
| **A barra laranja** (o braço da cena `=1`, já presa a 3 ossos) | o desenho que os ossos dobram e esticam |
| **O osso do MEIO, já escolhido** | o «antebraço»: o painel *Skeleton* mostra os números DELE |
| **A coluna de losangos** à direita de *Length*, *Segments* e das 4 alças da curva | cada losango diz se aquele número tem animação: cheio = há chave neste quadro · contorno = animado entre chaves · vazio = sem animação |
| **A animação já gravada** | *Length* tem chave no quadro 0 (o comprimento desenhado), no 24 (1,5×) e no 48 (volta): no Play o osso azul do meio cresce e encolhe |

⭐ **A LEI (dono 30/10, como o Blender/Spine):** no Play o osso do meio cresce, EMPURRA o osso
seguinte (a raiz dele fica na ponta nova) e ESTICA a barra presa ao longo do eixo. A mão
(`ph2d_app_skeleton::comprimento::aplica`) e a curva (`canais_osso::aplicar`) correm a MESMA lei
(`ph2d_skeleton_live::comprimento::empurra_filhos`); a pele estica por `agora / bind`
(`Tendon::comprimento`). ⛔ Até F40 o filho e a barra ficavam — a cena ensinava a lei velha.

⚠️ O osso do meio nasce com 4 segmentos: com 1 a curva é inerte e o painel esconde as alças
(`section_campos`), e a cena não as mostraria.

Monta em dois tempos como a `=1`: prender exige a entidade da forma (`vec_entities::sync`).

## `smoke_stack.rs`

> ⭐⭐⭐ **N PREENCHIMENTOS E N CONTORNOS NUMA FORMA** — `PH2D_VEC_STACK_SMOKE=1` (estudo 42, item 4).

# O que a cena fecha

Até 2026-09-05 uma forma tinha **um** preenchimento e **um** contorno. Cada camada de estilo a
mais obrigava a **duplicar o objecto** — e duas cópias de uma forma são duas geometrias que
divergem no primeiro ponto que o artista mexe: ele corrige uma e a outra fica para trás.

# A cena, e o que cada peça prova

| O quê | O que ela prova |
|---|---|
| **A ETIQUETA**: uma estrela com contorno branco largo por baixo de um preto fino | a largura é POR CAMADA (é isto que o Figma não faz: lá as N tintas partilham uma geometria de traço) |
| **O CARRIL**: uma linha com três contornos de larguras decrescentes | a pilha ordena-se, e o de cima desenha por último |
| **A MISTURA**: um disco com um 2.º preenchimento em `Multiply` a 60 % | opacidade e mistura são de CADA camada, e compõem-se DENTRO da forma |
| **A SOMBRA**: um cartão com o 2.º preenchimento DESLOCADO | ONDE cada camada desenha é da CAMADA (v21) — sem isso dois preenchimentos ocupam os mesmos pixels |
| **O ADESIVO**: uma estrela branca cujo 2.º preenchimento ENCOLHE | o *offset de CAD* (v22) — a silhueta de UMA camada contrai/dilata, e é isso que faz um adesivo numa forma só |
| O par de referência ao lado | a mesma arte feita à moda antiga (duas formas empilhadas) — para se ver que agora é **uma** |

⚠️ Se a linha `[vec-stack-smoke]` não aparecer, PARE: a cena não montou.

## `shape_live.rs`

> Live Shapes — o nascimento e o re-cozimento de uma forma paramétrica.

Uma forma desenhada **nasce viva**: ao soltar o gesto, a geometria é RE-COZIDA
CENTRADA no local 0 (o pivô nasce no centro dela), o `Transform` recebe o centro do
retângulo autorado, e a entidade ganha o componente `VecShape::Param` com o `kind` +
os parâmetros. A partir daí a geometria é uma função pura deles — mudá-los re-cozinha
a forma (é o que o painel faz, em [`crate::shape_params`]).

**"Convert to Curves"** de uma forma paramétrica é só **descartar o `VecShape`**: a
geometria já assada na cena vira um path cru, editável com a caneta. (O texto é o
único que também explode — num grupo de paths por-letra.)

O `w`/`h` vem do retângulo AUTORADO do gesto (`ShapeTool::bounds`), não da bbox da
geometria: num polígono/estrela elas diferem (a bbox de um triângulo é menor que a
elipse que o circunscreve), e é o retângulo que o usuário desenhou.

Nada aqui sabe QUE formas existem — o cozimento é o do catálogo
(`ph2d_vec_scene::cook`), a mesma porta que o preview do arrasto usa.

## `stroke_paint.rs`

> ⭐⭐ **COM QUE TINTA O TRAÇO DESENHA** — a porta única da fileira *Type* da secção *Stroke*
> (plano 35, wave D).

# Por que é um módulo próprio, e irmão do [`crate::stroke_present`]

Aquele responde *"esta forma TEM traço?"*; este, *"com que tinta ele desenha?"*. São duas
perguntas sobre o mesmo objecto, e a resposta de uma **pressupõe** a da outra — sem traço não há
tinta de traço. Mantê-las juntas num ficheiro faria a segunda herdar o `Option` da primeira por
acidente; separadas, cada uma declara o próprio `None`.

# ⛔ Por que a lista tem DUAS variantes e não as cinco do preenchimento

O renderer de traço não desenha gradiente. Um chip que produzisse um `StrokePaint::Linear`
gravaria estado que **nada pinta** — o documento leria de volta uma tinta inalcançável, e o
sintoma seria uma forma que abre diferente de como fechou. A recusa está escrita no
[plano 35 §2.1](../../docs/Vector%20Module/35_plano_padrao_no_traco.md); quando um gradiente no
traço for pedido, o `StrokePaint` ganha uma variante e esta lista ganha um chip.

## `bool_shape.rs`

> **O PAPEL DE CADA FORMA dentro de uma booleana viva** — a porta única de *"o que esta forma faz
> aqui?"*, e o chip que a edita.

Irmão do [`crate::vec_clip_edit`] no padrão: uma projeção que a shell publica por frame (o
painel não alcança o mundo ECS e não deve) mais o escritor que o clique dispara.

# Por que UMA porta, e não duas respostas parecidas

Dois consumidores fazem a mesma pergunta sobre a mesma forma: o **painel** (*"que verbo mostro
aceso?"*) e a **linha da hierarquia** (*"que selo pinto?"*). Se cada um a derivasse por si,
bastaria um deles usar a ordem de z e o outro a ordem da árvore para o selo dizer `BSE` numa
forma que o motor não trata como base — e a UI passaria a mentir sobre a receita sem uma linha
vermelha em lado nenhum.

⚠️ **A resposta vem do PLANO que o `bool_live` acabou de cozinhar**, não de uma segunda
caminhada da árvore. O plano é o que está na tela: quem é a base, quem são os operandos e em
que ordem. É a mesma disciplina do Apply, que materializa o plano em vez de re-chamar o motor.

## `state.rs`

> ⭐⭐ **O estado da família `vec`, com um dono só** — W2/L4 A2 (os primeiros 19) + A9 (os outros
> 50), ADR-0075.

# ⭐ A9 (2026-09-12): a `App` já não tem campo `vec_*` solto

O campo da `App` chama-se `vec` (era `vec_state`), e cada campo daqui perdeu o prefixo:
`app.vec_pen` passou a `app.vec.pen`. Os 50 que faltavam entraram pela régua da auditoria de
arquitectura, que pergunta pelo **assunto** de cada campo e não por quem o lê:

- **morto** ⇒ apagado antes de agrupar (o `vec_history` era escrito e nunca lido);
- **assunto da família + tipo de crate** ⇒ aqui;
- **assunto da família + tipo da shell** ⇒ o TIPO desceu primeiro, depois o campo. Eram oito
  (`VecTextEdit`, `ConnectorDrag`, `HandleDrag`, `TrimHit`, o `Grab` da largura, `VecSelSync`,
  `LabelPoses`, `BuildSession`); os outros doze «tipos da shell» eram FACHADAS de módulos desta
  crate — medido, não suposto.

⚠️ **A régua de baixo dizia que `vec_entities` e `vec_pen` não eram estado da família** («são o
documento e a árvore, que a shell inteira conduz»). A A9 mediu o preço que essa frase temia: com
os campos numa struct só, o que parte é emprestar o `VecState` **inteiro** ao lado de um campo
dele — e isso acontecia num sítio só, o `smoke_bone::bind`, curado lendo o mapa de dentro do
`st`. Os empréstimos por campo (`&mut app.vec.pen` ao lado de `&app.vec.entities`) são
disjuntos como sempre foram. *Muitos leitores não mudam o assunto de um campo.*

Dois grupos têm tipo próprio porque nascem com valores que não são os do `Default` dos tipos:
[`TextKnobs`] (o estilo de texto corrente) e [`ExpandKnobs`] (o par que o frame viu no quadro
anterior, que tem de nascer igual ao do painel).

# A régua da W2/L4, que escolheu os primeiros 19 (história)

`App` tem ~401 campos públicos e **75** deles chamam-se `vec_*`. Quantos podem sair não é uma
escolha — mede-se por duas perguntas, nesta ordem:

1. **quem os lê?** `app_state.rs` (a declaração) e `main.rs` (o construtor) não contam. Restam
   **30** campos cujos únicos consumidores de fora da família são os dois roteadores da shell
   (`render_loop/mod.rs`, `input_dispatch.rs`) — ou ninguém. Os outros 45 estão espalhados, e
   três deles são o substrato do módulo: `vec_entities` (189 ficheiros), `vec_scene` (131),
   `vec_pen` (36). ⛔ Esses não são «estado da família»: são o documento e a árvore, que a shell
   inteira conduz.
2. **o TIPO deles vem de uma crate?** Dos 30, **19** sim (11 de `ph2d_vec_*`, 8 primitivos) e
   **11 não** — o tipo mora na shell (`crate::blend_live::BlendSpines`,
   `crate::vec_bucket::BucketCache`, `crate::morph_live::MorphPlans`, …). Um campo cujo tipo
   está na shell **não pode** viver aqui, e agrupá-lo num `VecState` da shell seria trocar 11
   campos por 1 sem mover uma linha de lugar.

⇒ estes 19. Os 11 bloqueados estão nomeados no handoff de integração: cada um espera que o
TIPO dele saia, não que alguém o agrupe.

# ⚠️ Porque não é um recurso do ECS

O ADR-0075 manda desacoplar por ECS, e a primeira leitura disto é «ponha num componente». ⛔
Errado aqui, e o mecanismo está escrito no [`undo.rs`](../../../shells/desktop/src/undo.rs): o
`ProjectState` que o undo fotografa é `{WorldSnapshot + VecScene + FlipDoc + …}`, e o
`WorldSnapshot` **cobre toda entidade com componente registado**. Um `ShapeTool` ou um `Pencil`
como componente entraria na fotografia ⇒ **cada nudge do lápis viraria um passo de undo**.

Estado de FERRAMENTA não é estado de documento. O precedente da casa é o mesmo: `prop_state`,
`walk_state`, `z_walk_state` e `input` já são sub-structs num campo de `App`.

# ⚠️ `Default` é derivado, e isso foi VERIFICADO campo a campo

Os 19 inicializadores do `main.rs` eram `false` / `0` / `None` / `Vec::new()` /
`Pencil::default()` / **`ShapeTool::new()`** — e esse último é literalmente `Self::default()`
([`shape.rs`](../../ph2d-vec-edit/src/shape.rs)). *Um `derive(Default)` sobre um campo cujo
`new()` faz mais que o `Default` muda o arranque do app em silêncio* — aqui não faz, e é por
isso que se conferiu antes de derivar.

Na A9 a mesma conferência correu sobre os 50: `PenTool::new()`, `MorphPlans::new()`, os
`BTreeMap::new()` (`SideCache`, `BlendSpines`, `LabelPoses`, `LiveGeometry`) e o
`PencilHand::default()` são o `Default` deles. Os únicos que não eram — os oito números do
texto e o par do Expand — ganharam `Default` manual nos grupos acima, e o `VecState` continua a
derivar o seu.

## `pencil_input.rs`

> **A MÃO do lápis** — o estabilizador, entre o ponteiro e o lápis.

O gesto de mão livre grava o que a mão fez, e a mão TREME. O decimador (RDP) não resolve isto e
nunca vai: ele preserva **extremos locais** de propósito (é o que faz uma quina desenhada
sobreviver a qualquer tolerância), e um tremor é exactamente um extremo local. São duas
perguntas — *que detalhe eu guardo?* (Fidelity, na SAÍDA) e *que mão eu escuto?* (Stabilizer, na
ENTRADA) — e ter dois controles é o que impede um deles de mentir.

# A porta é a do Painter, não uma segunda

O filtro é **`ph2d_painter_brush::lazy_mouse_step`**, a MESMA função que o pincel do Painter e o
Free Hand dele usam. Uma segunda suavização divergiria do que o artista já aprendeu com o
raster, e o app passaria a ter duas respostas para *"o que o slider de estabilização faz?"*.

⚠️ **A aresta de dependência custa ZERO, e isto foi MEDIDO** (`cargo tree -i`): a shell já
recebe a `ph2d-painter-brush` transitivamente pela `ph2d-tool-painter`, então a crate já é
compilada em todo build da shell e a aresta direta só acrescenta uma linha ao `Cargo.toml`. A
objeção de 23.250 LOC que a W1a mediu era contra pôr a aresta na **`ph2d-vec-edit`** — uma crate
leaf com UMA dependência, onde ela seria uma sub-árvore NOVA. É por isso que o filtro é aplicado
aqui (a shell, que possui a entrada) e não dentro do lápis.

# Em que espaço se filtra, e por quê

Em **pixels de TELA**, antes de converter para mundo. O tremor é um fato da mão sobre a mesa, e
é em px que ele tem tamanho; o `lazy_mouse_step` é uma mistura proporcional (não um limiar de
distância), então filtrar em mundo daria a mesma forma — mas o número que o artista arrasta
passaria a significar coisas diferentes em zooms diferentes se algum dia o filtro ganhasse um
termo absoluto. Filtrar onde o tremor vive mantém o slider honesto.

# O default, MEDIDO

**MEDIDO** (`measure_pencil_stabilizer`, o S trémulo de 300 px com ±1,5 px de tremor sintético,
Fidelity 2 px) — o tremor que sobra, o atraso no fim do gesto, e os nós que o traço ganha:

| stabilizer | tremor residual | atraso final | nós |
|---|---|---|---|
| 0,00 | 2,06 px | 0,00 px | **97** |
| 0,25 | 1,41 px | 1,02 px | 25 |
| **0,50** | **0,89 px** | **2,63 px** | **14** |
| 0,75 | 0,58 px | 6,33 px | 14 |
| 0,90 | 1,00 px | 13,08 px | 15 |
| 0,97 | 2,39 px | 21,72 px | 13 |
| 1,00 | 4,23 px | 29,05 px | 14 |

⚠️ **O tremor residual NÃO é monotónico, e é isso que escolhe o teto útil:** ele tem MÍNIMO em
0,75 e depois PIORA — a 1,00 o traço está 2× mais longe da curva pretendida que a mão crua. Um
lazy mouse é um passa-baixa, e passado o joelho ele deixa de remover o RUÍDO e começa a comer o
SINAL: o ponto filtrado atrasa tanto que corta as curvas que o artista quis desenhar. Isto não é
defeito a corrigir — é o que "lazy" custa —, mas mata a leitura ingénua de *"mais é mais liso"*.

⚠️ **E o número que decide o default é a CONTAGEM DE NÓS, não o tremor:** num lápis VETORIAL o
que o artista herda é a curva que vai editar, e com a mão crua o RDP guarda **97 nós** para um S
— porque o tremor É um extremo local e a Fidelity preserva extremos por desenho. A 0,50 são
**14**, com 2,63 px de atraso (um quarto da largura de traço default, imperceptível). O default é
**0,50** por esse par; 0,75 compra 0,3 px de tremor por 2,4× o atraso, e não paga.

⚠️ **Isto CORRIGE uma nota minha da W1a**, que anunciava *"nove nós descrevem um S inteiro"*: era
medido sobre uma fixture de ±0,4 px de tremor — uma mão mais firme do que a real. Com ±1,5 px a
Fidelity sozinha dá 97, e é o estabilizador que torna a contagem editável.

⚠️ **O VALOR vive no tool** (`ph2d_tool_vector::params::PENCIL_STABILIZER_DEFAULT`), que é quem
o autora e semeia o slider; aqui fica a MEDIÇÃO, ao lado do filtro que ela mede. Uma segunda
const com o mesmo número seria a segunda porta que esta wave passou a tarde a evitar.

⚠️ **Preço, e é intrínseco a qualquer lazy mouse:** com estabilização alta o ponto filtrado
ATRASA o cursor, então o traço termina um pouco antes de onde a mão levantou (medido na tabela
acima). O Painter e o *Stabilize Stroke* do Blender têm exactamente a mesma
propriedade — é o que "lazy" quer dizer.

## `fx_live.rs`

> **A PILHA de FX raster VIVA na shell — 100% RESIDENTE NA GPU** (`ph2d_ecs::VecFilter`, plano 24).

Irmão do [`crate::offset_live`]/[`crate::contour_live`], mas de OUTRA natureza: o offset produz
GEOMETRIA (uma `LiveGeometry` que o `dispatch` desenha), e o FX produz PIXELS. A costura do
plano 24 §2: um FX raster não é `PathEffect` (não é `VecPath -> VecPath`) nem `LiveGeometry` —
ele isola a forma na própria textura, roda a pilha e recompõe.

# Por que GPU-resident (Enio: "tudo é para o game em runtime, total performance")

O 1º corte foi CPU-first (render → **readback GPU→CPU** → Gaussiana na CPU → **re-upload**) — um
padrão de PREVIEW de editor. Em runtime a forma filtrada ANIMA, então esse roundtrip roda por
frame por forma: readback bloqueia o pipeline e o re-upload cresce sem fim (o Vello cacheia
imagem por id de Blob, e um Blob novo por frame vaza o atlas). Não há "total performance" com
CPU no caminho.

Agora é **tudo na placa**:
1. **todas** as formas que erraram o memo são rasterizadas num [`VelloPass`] scratch, **numa
   passagem só**, cada uma na célula que o [`crate::fx_atlas`] lhe deu (renderer próprio, **sem
   readback**);
2. o [`FxStackPass`] roda a PILHA (op₁ → op₂ → … → resolve) na GPU para cada forma, lendo a
   célula dela, para uma textura de SAÍDA;
3. essa textura é registrada no renderer PRINCIPAL por um **id ESTÁVEL**
   ([`VelloPass::register_texture`]); re-cozinhar escreve NA MESMA textura, o Vello reusa o slot
   do atlas (zero churn de id, zero upload de CPU) e o `dispatch` a desenha no z da forma.
   ⛔⛔ **E avisa-se o Vello, com [`VelloPass::mark_texture_dirty`].** Até à `vello` 0.8 o atlas
   era **limpo a cada render** e este passo não existia — a frase acima bastava. A **0.10 tornou
   o atlas persistente**: sem a marca, os pixels novos **não chegam à tela** e a forma congela na
   primeira cozedura, refrescando só quando muda de TAMANHO. *Uma afirmação que era verdadeira
   por construção passou a ser um defeito, sem uma linha nossa mudar.*

# Por que UM render, e não `n`

⚠️ Um render do Vello custa **~0,12 ms antes de desenhar coisa alguma** — ele corre a cadeia de
binning/tiling inteira e submete, seja qual for o conteúdo. Com um render por forma, uma cena
de 32 formas filtradas gastava **4,0 ms só em raster**; a MESMA área de arte numa passagem só
custa **0,39 ms** (medido na RTX: `ph2d-render/tests/it/fx_scene_scale_cost.rs`). Num editor isso
é invisível — o artista mexe numa forma de cada vez —, mas o eixo é o do Enio: *"performance em
tempo real em runtime para games"*, e num jogo as formas filtradas **animam**, logo erram o
memo **todas, todo frame**. O que multiplica é o fixo, não o filtro.

O memo vira OTIMIZAÇÃO — pula o re-cook quando nada muda —, não requisito de correção: mesmo
re-cozinhando todo frame (cena inteira animada), é UM render + `2n+1` passes por forma. Vazio =
nenhum FX (byte-idêntico ao mundo pré-FX). ⚠️ **A CHAVE dele mora no [`crate::fx_live_memo`]** e
carrega o que é DESENHADO, não só a pilha resolvida e o tamanho — era essa a falta que fazia
*mudar a cor do fill de uma forma filtrada não mudar a tela*.

# A conversão MUNDO → PIXEL mora aqui, e só aqui

O componente fala MUNDO (resolution-crisp: dar zoom aumenta o borrão na tela, como deve). O
passe fala PIXEL. A câmera é conhecida por esta função e por mais ninguém — uma segunda
conversão noutro sítio seria um segundo sítio a errá-la.

## `widget_live.rs`

> **A PELE de widget, viva** — o degrau 2 do §2 do plano UI/UX (W6.2).

Uma forma marcada com [`ph2d_ecs::VecWidget`] deixa de ser desenhada como vetor e passa a ser
pintada pelo **pintor REAL** do catálogo, no retângulo dela. Este módulo é a ponte, e ele mora
na shell por uma razão de arquitetura: só aqui as duas metades se encontram — a
`ph2d-editor-core` (que sabe o que é um botão) e a `ph2d-vec-render` (que sabe onde a forma
está no z). A crate de render recebe um fragmento **opaco** e nunca aprende o que é um widget.

# Três perguntas, três donos, e nenhuma resposta duplicada

- **ONDE** — [`ph2d_vec_render::path_screen_bounds`], a MESMA função que o produtor de FX
  raster usa para dimensionar o scratch dele. Uma segunda projeção descreveria a forma noutro
  lugar da tela no primeiro zoom.
- **O QUÊ** — o `kind` do componente, traduzido UMA vez por
  [`ph2d_editor_core::widget::WidgetKind::from_code`]; código desconhecido devolve `None` e a
  forma **desenha como sempre** (é a compatibilidade, não um erro).
- **O RÓTULO** — o [`ph2d_ecs::Name`] da entidade, o nome que o artista já digita na
  Hierarquia. Sem `Name`, rótulo vazio: o widget desenha a moldura e mais nada, que é a
  resposta honesta para um objeto sem nome.

⚠️ **E o GLIFO de um `IconButton` é a QUARTA resposta, dada pelo próprio desenho** — a forma
que veste o botão *é* o ícone, normalizada por [`crate::widget_icon::icon_face`], a MESMA porta
que o painel gerado percorre. Ver o doc dela para o que a pose não faz.

# A APARÊNCIA não vem daqui, e isso é decisão

Cor, raio e tipografia saem dos **tokens** (`ph2d_tokens::resolve`, que a W6.1 tornou
autorável) — o desenho responde ONDE e O QUÊ. Um mapeamento por-tipo (*"o preenchimento da
forma vira a cor da swatch"*) é uma tabela de casos especiais e está deliberadamente **não
construído**; ver o doc de [`ph2d_editor_core::widget::paint_widget_skin`].

⭐ **F18 (17/10): o DESENHO passou a ser o corpo.** A forma desenha-se como sempre e a pele vai
por cima com o conteúdo, num escopo do tema que torna as superfícies transparentes e escolhe a
tinta legível sobre a cor PINTADA da forma ([`ph2d_tokens::skin_body`]). Os tokens continuam a
responder COMO — agora pelo papel de cada um.

# O widget é pintado em pixels de TELA

Um token é um número em **px** — `Radius::Md.px()`, `ICON_BTN_SIZE_PX`, o corpo do texto em pt.
Pintar a pele numa escala inventada mostraria ao artista um número que não é o número que o
app usa, que é exactamente a falha que a W6.1 existe para impedir. ⚠️ A consequência honesta:
**o zoom do canvas amplia a MOLDURA, não os detalhes do controle** — a caixa cresce, os cantos
ficam do tamanho que o token diz. Um widget é autorado em px porque um token é px; magnificação
fiel exigiria uma constante px↔mundo, e essa constante já tem dono noutro lugar
(`ProjectSettings::pixels_per_meter`) — puxá-la para cá seria uma segunda resposta a *"quanto
mede um pixel"*.

## `offset_live.rs`

> **O Offset VIVO na shell** — o cozimento e a materialização do [`ph2d_ecs::VecOffset`].

Espelho do [`crate::blend_live`] / [`crate::connector_live`]: o componente guarda a relação
(quanto, com que quina, de que lado) e a aparência é uma **função pura** dela, re-cozida aqui
e desenhada por [`ph2d_vec_render::dispatch`] **no z da forma**.

# A promessa que este módulo existe para cumprir

*"os botões Miter, Round e Bevel são previsualizações em tempo real dos efeitos, mas para
consolidar a curva deve-se apertar Apply Offset ou Convert to Curves"* (Enio, 2026-07-21).

Antes disto o preview **substituía** a forma no documento: o resultado era um objeto com id
próprio, e o modo Node mostrava as ~238 âncoras dos arcos do Round. O artista lia, com razão,
*"a curva já tem todos os vertex novos criados antes de apertar apply"*. Agora o documento
**nunca é tocado** enquanto o offset é preview: o `VecPath` continua sendo a curva autorada, e
o resultado é DESENHO.

# A ordem do cozimento: quina → pilha de efeitos → offset

A fonte que entra aqui é o [`ph2d_vec_scene::VecPath::cooked`] — ou seja, o raio de quina
(estágio zero, ADR-0121) e a pilha de Live Path Effects (ADR-0132) correm ANTES. É a única
ordem que se sustenta: o offset precisa de uma REGIÃO regularizada, e tanto a quina como os
efeitos produzem geometria plana que ele pode consumir. O inverso (offsetar e depois ondular)
continua alcançável pelo caminho destrutivo — Apply Offset e então o efeito.

# Quem lê a DERIVADA e quem lê a FONTE

Lêem a derivada os que respondem *"o que está na tela aqui?"* — o
[`ph2d_vec_render::dispatch`] e, desde 2026-07-21, o **hit-test de canvas** e o **marquee**
([`crate::vec_gizmo_view`]): clicar no que se vê é a definição de apontar, e enquanto o pick
apalpava a curva autorada a forma crescida era pintada num lugar e clicável noutro.

Lêem a FONTE os que respondem *"o que o artista escreveu?"*: o modo **Node** (as âncoras que
ele arrasta são as autoradas — é a mesma lei do raio de quina vivo, ADR-0121) e a **caixa do
gizmo** (o porquê está no cabeçalho do `vec_gizmo_view`). Nenhum dos dois é esquecimento.

# O memo não é otimização prematura: é MEDIDO

`offset_path` custa, em release, **0,40–0,86 ms** no donut e **0,55–1,07 ms** na estrela com
quina Round (Miter/Bevel ficam em 0,1–0,35). O cozimento roda por frame, então sem memo uma
cena parada pagaria isso para sempre. A chave é o que de facto o determina — a geometria de
MUNDO que entra e os três parâmetros — e não um contador de versão que alguém esqueceria de
bumpar.

## `profile_live.rs`

> **A LARGURA VIVA na shell** — o cozimento e a materialização do
> [`ph2d_ecs::VecStrokeProfile`] (ADR-0148).

Espelho exato do [`crate::offset_live`]: o componente guarda a **relação** (que largura, onde)
e a aparência é uma **função pura** dela, re-cozida aqui e desenhada por
[`ph2d_vec_render::dispatch`] **no z da forma**. O documento **nunca é tocado** enquanto o
perfil é preview — o `VecPath` continua sendo a curva autorada (é ela que o modo Node edita), e
a fita de largura variável é DESENHO.

# A promessa que este módulo cumpre

Antes disto os quatro sliders `W Start/Mid/End/Pos` eram **parâmetros de um comando**: nada
acontecia até o clique em *Power Stroke*, e o clique assava. O artista escolhia quatro números
às cegas e via o resultado depois de a curva já ter sido consumida. Agora eles autoram um
perfil vivo — o traço engrossa e afina **enquanto o slider anda** —, e o botão MATERIALIZA, o
mesmo par que o Offset já tinha (*"previsualizações em tempo real … para consolidar a curva
deve-se apertar Apply"*, Enio 2026-07-21).

# Uma porta produz o desenho, e é a MESMA que assa

[`crate::expand::power_stroke_layers`] — o preview desenha o que ela devolve e o Apply
insere o que ela devolve. Uma segunda rota ("um aproximador só para o preview") faria a forma
**SALTAR** no instante do Apply, que é o defeito que o ADR-0128 pagou cinco vezes. Há gate
comparando as duas saídas byte a byte.

# O neutro é a AUSÊNCIA

Perfil uniforme ⇒ o componente é **removido**, não guardado com todos os multiplicadores em
`1.0`. Um documento não acumula relações invisíveis que não desenham nada, e é isso que faz
*"arrastar os sliders de volta ao neutro"* devolver a forma ao estado limpo — a mesma lei do
`VecOffset` com `d = 0`.

# O memo é medido, como o do irmão

`power_stroke` custa **0,26–0,55 ms** por forma (`measure_power_stroke`, release). O cozimento
roda por frame, então sem memo uma cena parada pagaria isso para sempre. A chave é o que de
facto o determina — a geometria de MUNDO que entra (que já carrega a largura do traço) e as
paradas —, e não um contador de versão que alguém esqueceria de bumpar.

## `vec_gizmo_view.rs`

> A forma vetorial vista pelo **gizmo de sprite** (ADR-0111).

Não há gizmo vetorial próprio. Havia — 492 linhas que mutavam a geometria — e
ele foi removido. Um path com `Transform` é um objeto como qualquer outro, e a
matemática de mover/girar/escalar já existe, testada, em `ph2d-editor-core`.

A tradução é exata, não uma imitação. O gizmo enquadra um sprite como

```text
centro = translation + R·(anchor ⊙ scale)     meia-extensão = half_intrínseco ⊙ scale
```

e uma forma vetorial tem a MESMA forma se lermos `anchor` como o **centro da
bbox local da curva** e `half_intrínseco` como a **meia-extensão dessa bbox**.
Por isso `opposite_anchor_translation` (que fixa o canto oposto ao escalar) vale
sem uma linha nova: ela só depende desses dois números.

Aqui moram também o **picking** de canvas — clicar numa forma fora da ferramenta
vetorial a seleciona como se fosse um sprite — e o marquee.

# O pick segue o DESENHO; a CAIXA segue a fonte

Com um offset vivo (`ph2d_ecs::VecOffset`) o documento guarda a curva autorada e a tela
mostra a derivada. O **hit-test** pergunta ao mesmo `live` que o renderer consome — clicar
no que se vê é a definição de apontar, e sem isso a forma crescida era pintada num lugar e
clicável noutro.

A **bbox do gizmo** (`gizmo_view_for_entity`, acima) fica na FONTE, e é decisão, não
esquecimento. Dois motivos: (a) o `d` do offset é uma distância de MUNDO, então escalar a
forma não escala a banda — uma caixa que a incluísse pediria `s = (alvo − 2d)/meia_fonte`,
e o gizmo derivaria do dedo durante o arrasto, que é a mesma armadilha das 5 tentativas
revertidas do Blend Object (ADR-0128: *um gizmo sobre geometria que se move DOBRA*);
(b) é o default do Illustrator — o bounding box ignora traço e efeitos de aparência, e
"Use Preview Bounds" é preferência **desligada**.

## `component_edit.rs`

> **O QUE UM CLIQUE NA SECÇÃO *Prefab* PEDE** — a tabela `id → verbo`, e mais nada.

# ⛔⛔ Este ficheiro tinha 473 linhas e um MOTOR dentro (F4.6c, 2026-09-07)

Ele foi a shell do sistema de instâncias **do vetor**: `create_main`, `place_instance`,
`detach`, `reset_overrides`, `instance_count`, `cascade_offset`, `arm_instance`,
`arm_detached`, `main_content_box` — um segundo motor de instância, com o `VecInstance` no ECS
por trás dele. O modelo GERAL (ADR-0164) responde hoje por todos, e *dois motores para o mesmo
estado é pior que um motor lento*: o que sobra aqui é a **tradução do clique**, que é a única
coisa que era de facto do painel vetorial.

Quem executa é [`crate::vec_component_general`], que passa o verbo ao dreno único
(`instance_verbs::drain`) — onde vivem a voz de cada recusa, a cascata da cópia e a lei *«a
selecção segue para a cópia»*.

# ⚠️ As duas variantes que morreram com o motor

`PieceVisible(row)` e `Variant(axis, value)` endereçavam controlos que só o motor vetorial
pintava (a lista de peças e a fileira de eixos). **As capacidades não se perderam** — esconder
uma peça de UMA cópia e pintá-la são hoje o `Visibility`/`Sprite` da própria peça
([`crate::instance_piece_override_tests`], escrito como pré-condição desta fatia), e a versão
que uma cópia é vive no cartão do Inspector (F5).

⚠️ *Um verbo que não tem executor não fica no enum a fingir* — enquanto ficava, o
`general_verb` devolvia `None` e o clique morria em silêncio, que é o defeito que esta secção
caçou três vezes.

## `blend.rs`

> **Blend** — os passos intermediários entre duas formas (o Blend do Illustrator).

O motor é a [`ph2d_vec_blend`]; aqui mora só a **política**: quem são as duas formas, o que
acontece com elas, e o que a seleção vira.

# A sessão existe por causa do ESCAPE, e o escape existe porque o automático erra

A correspondência entre duas formas — *que ponto de A vira que ponto de B?* — é o problema
que **ninguém** resolveu (`docs/Vector Module/20_*` §1.3: o GSAP tem `shapeIndex` manual **e**
uma ferramenta de debug que admite o erro; o Corel pede pro usuário clicar um nó em cada
forma). A correspondência é 100% AUTOMÁTICA — o motor escolhe o sentido de menor custo e o
melhor casamento de quinas sozinho.

Houve **dois** botões de escape manual, **Rotate Match** e **Reverse Match**, e os dois foram
removidos (2026-07-14) por serem bugs de design: o Reverse invertia o winding e colapsava a
forma; o Rotate rodava a correspondência às cegas e produzia torção. **O ajuste, no modelo vivo,
é editar as formas-fonte** — girar/mover/escalar uma adapta os intermediários —, não um botão.

> ⚠️ Este é o modelo DESTRUTIVO (a `BlendSession` produz paths reais). O **painel já NÃO o
> usa** — o botão "Blend" cria o **Blend Object VIVO** (ADR-0128, `crate::blend_live`): um objeto
> único, não-destrutivo, com as fontes sempre editáveis. Este `apply` sobrevive só para os
> smokes de correspondência (`PH2D_BUILD_SMOKE=7/8/9`, que mostram star→circle etc.); a remoção
> completa (com os smokes repontados ao vivo) é uma limpeza posterior.

As **fontes sobrevivem** (ao contrário da booleana, que consome os operandos) — é o Blend do
Illustrator: os passos nascem ENTRE elas, no z delas.

## `connector_live.rs`

> **Conectores vivos** — a linha que gruda em duas formas e as SEGUE.

Espelho exato do padrão da *Live Shape* ([`crate::shape_live`]): o componente
[`VecConnector`] guarda a **relação** (a quem cada ponta se prende, e como a rota é
desenhada) e a geometria é uma **função pura** dela, re-cozida a cada frame. Ninguém
"move" um conector: move-se uma forma, e a linha se refaz.

Consequência de graça (a mesma da Live Shape): **undo e save cobrem o conector sem uma
linha a mais** — os dois capturam o mundo ECS + a cena vetorial, e o `VecConnector` está
registrado no `ComponentRegistry`.

# Os três detalhes que decidem se isto funciona

1. **Escreve a polilinha EM LUGAR** (`scene.path_mut(id)`), preservando o id. Remover e
   re-empurrar o path daria um id novo a cada frame — e a entidade, a seleção e o gizmo
   piscariam junto (o invariante documentado da Live Shape).
2. **O lado de saída lembra do frame anterior** ([`SideCache`]). Sem o `prev` da
   histerese, um alvo passando pela diagonal da caixa faz a linha piscar entre sair pelo
   topo e sair pela lateral, quadro sim, quadro não.
3. **O conector vive na IDENTIDADE.** A geometria que escrevemos aqui é MUNDO; se a
   entidade tivesse pose, o afim a aplicaria por cima e a rota sairia deslocada. Por isso
   `vec_transform::settle_origins` o **pula** (como já pula as formas vivas) e este
   módulo devolve o `Transform` à identidade se alguém o mexeu — o que torna o conector
   **não-arrastável pelo gizmo**, que é o comportamento certo: arrastar um conector não
   quer dizer nada; o que se move são as pontas dele.

## `blend_live.rs`

> **Blend Objects vivos** (ADR-0128) — o objeto único que interpola 2..=5 formas e as SEGUE.

Espelho exato do padrão do [`crate::connector_live`]: o componente [`VecBlend`] guarda a
**relação** (quais formas, na ordem, e quantos passos) e os passos intermediários são uma
**função pura** dela, re-cozidos a cada frame. Ninguém "desenha" um passo: move-se uma forma
fonte, e a transição se refaz.

Consequência de graça (a mesma do conector): **undo e save cobrem o blend sem uma linha a
mais** — os dois capturam o mundo ECS + a cena vetorial, e o `VecBlend` está registrado no
`ComponentRegistry`.

# Os passos são VIRTUAIS — o que está na cena é só o SPINE

A entidade do blend carrega um `VecPathRef` como qualquer forma; o `VecPath` dela é o **spine**
(a linha que une as fontes). Os N passos NÃO entram na cena — a shell os coze aqui, num
`Vec<VecPath>` de MUNDO, e um passe de render ([`ph2d_vec_render::draw_blend_overlay`]) os
desenha. É o que torna o blend **um objeto**, e não N formas (o pedido do Enio). Consequência:
os passos não são pickáveis (igual ao Illustrator — pega-se o objeto, não um passo).

# O blend vive na IDENTIDADE (como o conector)

O spine e os passos são geometria de MUNDO; uma pose na entidade os deslocaria. Por isso
`vec_transform::settle_origins` o **pula** e este módulo devolve o `Transform` à identidade —
o que o torna (corretamente) não-arrastável pelo gizmo: mover o blend não quer dizer nada; o
que se move são as formas-fonte, e a transição as segue (ADR-0128, o idioma do Illustrator).

## `lib.rs`

# ⚠️ Porque ela tem TRINTA E TRÊS ficheiros e não cento e vinte e sete

O conjunto que pode sair **tem de ser fechado sob toda aresta de compilação**. A Fase A mediu-o
quatro vezes (71 → 29 → 15 → **8** ficheiros), cada resposta menor que a anterior e **as três
primeiras a favor de mover demasiado**; a Fase B mediu-o outras quatro, e o padrão repetiu-se:

| a régua da Fase B corrigiu-se assim | efeito |
|---|---|
| `^impl App` perdia os **10** `impl crate::App` — são **13**, não 3 (HOWTO §2.1) | subestimava |
| `\bApp\b` no ficheiro CRU acusa 23 e **7** mencionam-no só em doc-comment (§2.12) | 69 → **85** livres |
| `crate::render_loop::vector_bridge` é ficheiro **desta** família na pasta do laço | +6 |
| um movido não pode referir um da família que **FICOU** — `vec_entities` prendia **30 de 50** | 50 → **21** |

⛔⛔ **E o bloqueador de fundo NÃO é a `App`.** Medido: curar todo o acoplamento a `App` move
**+4 ficheiros / 615 LOC** e nada mais, porque os 9 que ela prende batem imediatamente no
`vec_entities`. O grafo da família tem **uma raiz só**:

```text
vec_entities  ──blocked by──▶  name_unique::unique_name
     │                         morph_set::is_set_member
     │                         render_loop::off_canvas::is_off_canvas
     ├──#[path]──▶ vec_zorder ──▶ vec_zorder_fixpoint_tests ──▶ undo · hero_intents ·
     │                                                          project_library · preview_drive
     └──referido por 42 ficheiros da família
```

Os três predicados são **puros sobre o ECS** e pertencem a outras famílias ⇒ tomá-los violaria o
[HOWTO §1.2] (*duas famílias que partilham código partilham uma FOLHA, nunca uma delas à outra*).

⭐⭐⭐ **RESOLVIDO em 2026-09-12 pela `line/shell-folhas`, e exactamente como este diagrama
prescrevia:** os três predicados viraram folhas ([`ph2d_unique_name`],
[`ph2d_entity_visibility`]) e o `vec_entities` inteiro — com o `vec_transform` e o `morph_set`,
que fazem ciclo com ele — mudou-se para a [`ph2d_vec_entities`]. ⚠️ **O diagrama acima fica
como está de propósito:** ele é a medição que abriu aquela linha, e apagá-lo apagaria a razão.
⚠️ **O `name_unique` e o `preview_drive` são DOIS dos três que a `line/app-physics` nomeou** como
folhas partilhadas de linha própria — esta família confirma-os **independentemente**.

⛔ **E a cura barata é pior que a doença, com número:** dar a `vec_entities::sync` um parâmetro
para o nome único custa **168 sítios de chamada**, espalhados por `bool_live`, `blend_live`,
`connector_live`, `envelope_live`, `instance_*`, `label_live` — as famílias da 3.ª rodada. Um
parâmetro que atravessa 168 sítios de cinco famílias não é uma assinatura: é uma wave.

⇒ **o que falta desta família não é trabalho desta linha.** A lista, com o preço de cada item,
está no handoff da Fase B.

# O molde que estes ficheiros provam

Um `Cargo.toml` medido dependência a dependência (**quatro** eram invisíveis até a crate
existir — `ph2d-core`, `ph2d-tool-vector`, `ph2d-vec-blend`, `ph2d-vec-fill`; o piloto achou
quatro também), a **re-exportação com alias** que mantém os ~190 `crate::vec_x` da shell byte a
byte iguais, e as duas armadilhas de caminho que só a corrida revela (§2.6): o `include_str!`
que falha alto e o `CARGO_MANIFEST_DIR` que **não** falha.

[HOWTO §1.2]: ../../../docs/IntegracaoMultiAgente/HOWTO_partir_uma_familia_da_shell.md

## `smoke_bone_paint_assada_tests.rs`

# Porque esta é a pergunta DECISIVA, e não uma entre várias

A direcção inteira da F9 — a densidade sai do QUADRO e vai para o BIND, e a deformação passa
para o *vertex shader* — assenta numa única propriedade: **uma topologia só tem de servir todas
as poses**. Se ela servir, o resto é engenharia (enviar poses, costurar os leitores da malha).
Se não servir, a direcção cai e as ondas W1–W4 medem outra coisa.

⚠️ **E ela mede-se sem inventar lei nenhuma.** Assar *no pior caso* (a dobra mais forte, no zoom
mais fino) usa o refinamento que já existe e responde à pergunta da topologia. O critério
proposto na fila — refinar onde o CAMPO DE PESOS curva — é uma forma de escolher ONDE partir com
menos peças, e só faz sentido medir depois de se saber que uma topologia fixa chega.

# ⛔ O CONTROLO que torna a comparação válida

Re-posar uma malha assada noutra cena só significa alguma coisa se o BIND das duas cenas for o
mesmo — mesmas posições de repouso, mesmos pesos. Se a `cena_dobrada` mexesse no bind ao dobrar,
a tabela compararia duas artes diferentes e leria isso como qualidade. ⇒ a sonda **afirma-o
primeiro**, e pára se não for verdade.

# O que cada coluna é

* **peças** — triângulos entregues. Na assada é um número só, o mesmo em toda a linha: é ele que
  vira memória de GPU (a W4 da fila).
* **faceta** — `skinned_deviation`, o desvio da malha contra o campo que ela segue, em pixels de
  ECRÃ (já multiplicado pelo zoom).
* **canto** — o maior canto da silhueta desenhada (`vai_e_volta`), que é a régua do OLHO e a que
  apanhou o report de 2026-09-16. ⚠️ Ela não sabe que campo existe, de propósito.

## `ui_panel_spec.rs`

> **QUE PAINEL ESTA ÁRVORE DESCREVE** — a porta única do W8b (plano UI/UX §4).

Uma moldura autorada com filhos vestidos é a descrição de um painel. Esta função responde
*qual*, e o resultado (`PanelSpec`) é **dado simples**: quem o consome escreve código-fonte
(`ph2d-ui-codegen`), sem ECS, sem documento e sem janela.

# A ordem das rows é a ordem dos FILHOS, e é uma decisão

A moldura com auto layout **flui os filhos na ordem em que eles estão na árvore** — é por isso
que arrastar um filho dentro de um fluxo **REORDENA** em vez de o mover (ADR-0153: a pose é
derivada, então um arrasto não tem onde pousar). Logo, a ordem que o artista vê no painel e na
Hierarquia **é** a ordem dos filhos, e ler o z (que é outra pergunta) daria um painel cujas
rows saem noutra ordem que a moldura mostra.

# Só quem VESTE vira row

Um filho sem [`VecWidget`] é desenho — um fundo, uma divisória decorativa, um ícone. Ele
continua a ser desenhado pela pele da moldura; o que ele **não** é é um controle. Transformar
todo filho em row daria um painel com linhas que não fazem nada, que é o item-de-menu-morto
deste repo na sua forma mais cara.

⚠️ E um `kind` que este build **não conhece** também não vira row: o `from_code` devolve
`None` de propósito (um documento autorado por um build mais novo), e inventar um tipo aqui
seria gerar código para um widget que não existe.

## `resize_box_edit.rs`

> **RESIZE BOX da seleção** — a projecção que o painel lê, e a porta que a edita (plano UI/UX
> W3b, decisão do Enio 2026-08-03).

Irmão do `vec_anchor_edit` (na shell) e do `vec_frame_edit` (na shell), com a mesma divisão de
donos: a verdade mora no ECS ([`ph2d_ecs::VecResizeBox`], um **override** sobre o default
derivado da hierarquia) e isto é o que a shell publica por frame. O painel não alcança o mundo
— se alcançasse, a resposta que DESENHA a caixa marcada divergiria da que HONRA o arrasto.

# O que o checkbox decide

Marcado, arrastar a alça do gizmo reescreve a **CAIXA** do objeto (a geometria). Desmarcado,
escala a **POSE**, que é herdada por todo descendente — o comportamento correto para um
objeto de **game**, e o que este editor sempre fez.

# O NEUTRO destaca, e é o que mantém o ficheiro honesto

O default é uma função da hierarquia ([`ph2d_ecs::resize_box_default`]): molduras e os filhos
delas nascem marcados. Voltar ao valor de fábrica **remove** o componente em vez de gravar um
bool redundante — o precedente literal do `VecLayoutItem` e de todo override da física.

⚠️ Isso é mais do que higiene: sem o destacamento, uma forma marcada em `true` *por já ser
filha de moldura* levaria o `true` consigo ao ser arrastada para FORA, e passaria a
redimensionar num sítio onde o default diz escalar — a regra deixaria de seguir a hierarquia
sem nada na tela a dizer porquê.

## `bucket.rs`

> ⭐⭐⭐ **O BALDE na shell** (plano 40) — a costura entre o ponteiro e a lei ([`ph2d_vec_fill`]).

# As três coisas que esta camada decide

1. **QUE contornos são PAREDE.** Os visíveis do documento, cozidos e assados no MUNDO — **menos
   os preenchimentos** ([`ph2d_ecs::VecBucketFill`]). ⛔ Um preenchimento tem por fronteira os
   mesmos arcos que as linhas; deixá-lo voltar à rede punha lá arestas **coincidentes**, com
   direcção de saída idêntica, e o passeio de faces passava a escolher entre duas meias-arestas
   indistinguíveis: *"ao usar o balde nas áreas coloridas, ele para de funcionar nas áreas não
   coloridas"* (Enio, 2026-09-01).
2. **QUANDO a rede é reconstruída.** ⚠️⚠️ Medido:

   | contornos | arcos | montar a rede | achar a face |
   |---|---|---|---|
   | 4 | 8 | `0,06 ms` | `0,01 ms` |
   | 20 | 280 | **`3,80 ms`** | `0,08 ms` |
   | 80 | 1293 | **`188 ms`** | `0,35 ms` |

   ⇒ montar por quadro está refutado; achar a face por quadro é de graça. A rede é **guardada**,
   com a chave a ser o **CONTEÚDO** (âncoras e alças) e não a contagem de caminhos.
   ⭐ **E o upkeep inteiro não corre se não houver preenchimento nenhum e o balde não estiver na
   mão** — quem não usa a ferramenta não paga nada.
3. **QUANDO os preenchimentos são RE-COZIDOS.** Sempre que a rede muda, e **em qualquer
   ferramenta**: o artista arrasta um nó com a seta branca e a área tem de acompanhar.

## `vector_bridge.rs`

> Vector tool ⟷ shell bridge (ADR-0108 cutover).

Per-frame jobs (mirror of the Padding / Painter panel bridges):

1. **Panel visibility** — show the docked `ph2d-panel-vector` Style panel
   (right dock) iff the `vector` tool is active; hide the real Inspector
   (edge-triggered) so they don't both claim the slot.
2. **Picker read-back** — a Down on the Stroke / Fill swatch opened the
   shared OKLCH picker (generic `is_picker_swatch` dispatch). While the
   picker targets one of our swatches, feed the live picked colour into the
   tool via [`VectorTool::set_stroke_rgba`] / [`VectorTool::set_fill_rgba`].
3. **Style sync** — copy the tool's stroke / fill / width into the Pen so
   newly drawn paths honour the Style.
4. **Restyle selected** — the fields the panel changed (`take_authored`, plano 31),
   recolour the selected path. (The undo step is the GLOBAL queue's, by state
   diff; the per-gesture `RECOLOR_PRE` snapshot died with the vector `History`
   on 2026-09-12 — it fed a stack nobody undid.)
5. **Publish** — sync the swatches' `widget_color` to the live colour (seeds
   the picker on open) + publish the Style snapshot the panel paints.

The concrete-tool downcast lives HERE (allowlisted:
`architecture_no_downcast_to_concrete_tool_in_shell`), so the central render
loop stays downcast-free — mirror of `painter_bridge`.

## `driven_style.rs`

> ⭐⭐⭐ **A APARÊNCIA CONDUZIDA CHEGA À PROJECÇÃO DO QUADRO** — a segunda metade da ponte que o
> [`ph2d_ecs::VecDrivenStyle`] abre.

A linha do tempo escreve o componente (ela só sabe falar com o mundo ECS); este módulo lê-o e
funde-o na **mesma** entrada de estilo que os tokens de design e as rows autoradas já produzem.
É o irmão exacto do [`crate::vec_widget_drive`], e de propósito: o corte entre os dois é *quem
produz o número* (um motor de animação · um controlo que o artista está a segurar), nunca o que
se faz com ele.

# ⚠️⚠️ FUNDIR, nunca acrescentar ao lado

O consumidor lê **uma** entrada por forma (`bound_style(id)` devolve a primeira), então uma
segunda entrada para a mesma forma seria descartada **em silêncio** — e qual das duas some
dependeria da ordem de iteração de um mapa. É a lição que o `vec_widget_drive` já pagou, e este
módulo é o **terceiro** produtor da mesma lista.

# A ordem no quadro, e por que ela é esta

`tokens → ESTE → rows autoradas`. A linha do tempo corre **antes** do controlo que o artista
está a segurar, e é o precedente que o passe de estados já escreve: *se as duas coisas escrevem
o mesmo objecto, quem manda é o gesto que o artista acabou de fazer; o motor é o estado de
fundo*. Arrastar um slider de opacidade durante uma reprodução mostra o slider — se mostrasse a
curva, o controlo pareceria partido.

## `snap.rs`

> ⭐ **A LEI do snap vectorial, fora da shell** — W2/L4, A2 (desfazer o acoplamento).

Aqui mora o que o encaixe DECIDE; na shell fica só o que ele precisa de PERGUNTAR à janela.
O corte é exactamente esse, e é verificável: nada neste ficheiro menciona `App`, `gfx`,
painel ou câmera.

# ⚠️ O módulo chama-se `vec_snap` de propósito

Ele era `shells/desktop/src/vec_snap.rs`, e o nome do módulo entra no **nome de cada teste**.
Mantê-lo faz a prova de que nenhum teste se perdeu (`scripts/nextest-list-diff.py`) passar
mesmo a `--depth 2` (`módulo::fn`), e não só no `--depth 1` por omissão. *Renomear um módulo
de testes durante uma extracção é indistinguível, na lista, de perder os testes dele.*

# O que FICOU na shell, e porquê (a lista que o substrato tem de cobrir)

| método | o que ele precisa e não é da família |
|---|---|
| `vec_px_to_world` | `gfx.surface` + `gfx.camera` — e ⚠️ ele não é do vector: **11 ficheiros** de outras famílias o chamam |
| `vec_snap_point` · `vec_snap_move` | `gfx.hero_screen.grid` — estado de PAINEL |
| `vec_rebuild_snap_targets` | `gfx.guides` + a orquestração do gesto |
| `dragged_entity_bits` · `dragged_vec_path_ids` · `snap_dragged_vec_during_drag` | `gfx.hero_screen.gizmo` |
| `sprite_snap_points` · `vec_move_sources` | `crate::transform`, que **não pôde sair** (declara um `#[path]` de um teste que precisa de `crate::profile_live`) |

## `envelope_gesture.rs`

> O gesto de **arrastar os cantos da gaiola** do Envelope (ADR-0129, Fatia 1) — o lado do HOST.

A parte pura (que canto, e até onde) mora em `ph2d_vec_envelope::{nearest_corner,
move_corner_convex}`; aqui é o adaptador fino que lê a seleção, o componente ECS [`VecEnvelope`]
(no **container**, ADR-0129 Fatia 3) e o cursor, e ESCREVE `corners` de volta no componente. É o
padrão do [`crate::blend_live`] (um gesto de Node que toca o ECS), **não** o do `PenTool`.

O alvo do gesto é uma **entidade** (os bits do container), não um `VecPathId`: o container não tem
path. O host passa a seleção do gizmo (`hero.gizmo.selection`), que — pela regra
*seleciona-só-o-container* de [`crate::vec_selection`] — é o container quando um envelope está
selecionado. Um alvo que não é envelope (sprite, path comum, grupo) é ignorado, e o pen segue dono
do clique.

A alça é PRÓPRIA e vive no modo Node (ADR-0129 §3.3): o gizmo de sprite não a toca. Um gizmo sobre
a geometria de mundo que o [`crate::envelope_live::recook`] reescreve a cada frame dobraria — a
lição de 5 tentativas revertidas do Blend (ADR-0128).

# Undo sai de graça

O arrasto roda com o botão pressionado, e `App::post_frame_undo` suprime passos enquanto
`held_button` está `Some`. Os N frames do arrasto não viram N passos; ao soltar, o [`VecEnvelope`]
alterado (que viaja no `WorldSnapshot`) vira **um** passo no diff global. Nada a instrumentar aqui.

## `svg_import.rs`

> ⭐⭐⭐ **UM `.svg` LARGADO NA JANELA VIRA FORMAS EDITÁVEIS** (estudo 42, item 3).

A tradução vive na crate-folha [`ph2d_vec_svg`]; aqui mora o que é da SHELL — onde o desenho
aterra, que nome cada forma leva na Hierarquia, e como os `<g>` do ficheiro viram grupos.

# Três leis, e as três já existiam nesta casa

1. **Um px é um px** — o mesmo divisor (`pixels_per_meter`) que dimensiona uma sprite
   importada. Um `.svg` de 512 unidades entra do tamanho de um `.png` de 512 px.
2. **Um path ⟺ uma entidade** — quem as cria é o [`ph2d_vec_entities::entities::sync`], e não este
   módulo: pôr aqui um segundo criador de entidades seria a segunda porta pela qual um path
   órfão nasce.
3. **Agrupar é o verbo que já existe** ([`ph2d_vec_entities::entities::group_entities`]) — ele
   põe o grupo entre os filhos, compensa a pose de cada um e ordena a lista. ⚠️ Ele exige
   **dois** membros, então um `<g>` com um filho só é achatado; é a mesma regra que o artista
   lê no menu (*"Select at least 2 objects to group"*).

⚠️ **O ficheiro inteiro vira UM objecto** quando traz mais de uma coisa de topo: sem isso um
logótipo de 40 formas aterraria como 40 raízes na Hierarquia, e mover o desenho seria
impossível sem antes o seleccionar todo.

## `no_orphan_module_tests.rs`

> ⛔⛔ **NENHUM FICHEIRO DESTA CRATE FICA SEM `mod`** — o gate de um defeito que é MUDO por natureza.

# O que aconteceu, e porque nada o apanhou

Na W2 Fase D o `bool_reach_tests.rs` mudou-se da shell para cá. Na shell ele era um
`#[cfg(test)] mod` de TOPO, declarado no `main.rs`; a declaração foi apagada de lá e **nenhuma
foi escrita aqui**. Um ficheiro `.rs` que nenhum `mod` declara **não é compilado** — ele fica no
disco, no git, no diff, e fora do build.

⇒ os **três** gates que ele contém evaporaram-se, e:

| instrumento | o que ele disse |
|---|---|
| `cargo check -p ph2d-app-vec --all-targets` | **verde** |
| `cargo check -p ph2d-host-desktop --all-targets` | **verde** |
| `cargo clippy` | **verde** |
| a suíte da crate | **verde** — com três testes a menos, e ninguém conta os que faltam |
| `cargo test --test it` | **verde** |

*Não há nada para verificar num ficheiro que não entra no build.* O que o acusou foi o `ONLY-A`
da prova de `nextest-list-diff` — é literalmente para isto que aquele número existe.

# ⚠️ Porque este gate vive aqui e não na shell

Ele mede a **árvore desta crate**, e o sujeito é o `src/` dela. É a mesma forma do censo de
órfãos que o `CLAUDE.md` §5.0 descreve (*«o ÓRFÃO e o DUPLICADO são as duas metades do MESMO
audit, os dois mudos»*), aplicada ao sítio onde o movimento acontece.

## `symmetry_live_tests.rs`

> Gates da **SIMETRIA DE DESENHO** — arquivo irmão de `symmetry_live.rs`.

As exigências do USUÁRIO que esta família existe para pinar (Enio, 2026-08-01):

> *"A linha de simetria deve aparecer logo que se aperta o botão e não quando se inicia o
> desenho."*

> *"A simetria funciona apenas para formas que serão desenhadas com a tool ligada e não deve
> fazer simetria de formas que já existem previamente."*

> *"Com o botão checado pode-se fazer quantos desenhos desejar que a linha de simetria permanece
> no lugar."*

> *"Uma vez que o desenho é feito, a referência para a linha passa a ser o próprio desenho … se
> o usuário mover o objeto no canvas a linha de simetria acompanha."*

> *"Se a simetria for desmarcada antes do apply, as cópias somem mas não são destruídas."*

⚠️ **O oráculo NUNCA é "há duas formas desenhadas"** — isso é verdade em qualquer implementação
que reflicta alguma coisa em algum lugar. Os oráculos daqui são: (a) o desenho inteiro TRANSLADA
com a pose, ponto a ponto; (b) uma forma que ninguém desenhou continua **sem componente**; (c) o
eixo capturado, levado de volta ao MUNDO, coincide com o de sessão; (d) a curva autorada fica
byte-idêntica através do ciclo ligar→desligar; (e) uma forma já simétrica no próprio eixo
coincide com a própria cópia sob QUALQUER pose — a propriedade que só a ordem *reflectir → assar*
tem.

## `pencil_frame_tests.rs`

> **O lápis desenha onde a mão está** — o gesto contra a ordem REAL do frame.

Estes gates nasceram VERMELHOS num defeito reportado pelo Enio (*"pencil está com offset do
mouse"*). A causa não estava no lápis: estava na lista de gestos que o `settle_origins` pula.
Ela enumerava *"a caneta e a ferramenta de forma"* — as duas que existiam quando foi escrita —
e o lápis chegou como o TERCEIRO. Assentar um path no meio de um gesto que reescreve a
geometria em MUNDO a cada frame soma geometria + `Transform`, e a tinta sai deslocada do cursor
**pelo ponto onde o arrasto começou**.

# Por que os gates da wave não o apanharam

Os 22 gates de motor provam o AJUSTE (a curva passa pelas amostras) e os 5 arch-gates provam a
ORDEM do dispatch. Nenhum dos dois vê a POSE: a geometria estava certa e o afim da entidade
desmentia-a um passe depois. O que faltava era um gate que corresse os passes do frame **na
ordem do produto** e comparasse o pixel com o dedo — que é o que este arquivo é.

⚠️ **O 1º frame do gesto sai CORRECTO**, e é isso que torna o defeito difícil de ver num teste
de unidade: o `settle` corre ANTES do render, então no frame em que ele assenta o path a
geometria já é local e o afim já a leva de volta ao lugar. A divergência começa no frame
SEGUINTE, quando o lápis reescreve mundo por cima do local. Um gate que medisse um frame só
ficaria verde sobre o produto vermelho — daí a asserção correr **sobre o gesto inteiro**.

## `cut_line.rs`

> **A LINHA DE CORTE** — o caminho que a tesoura usa como lâmina (plano 25 §7).

Espelho exato do `connector_live::upkeep` e do `blend_live::upkeep`: um caminho novo já está
na cena desde o press, mas a **entidade** dele só nasce no `vec_entities::sync`, mais adiante
no mesmo frame — então o componente que o marca como lâmina não pode ser pendurado no press.
A fila de um item (`VecState::cut_pending`) é o que atravessa esse vão.

# Uma lâmina de cada vez

Desenhar uma linha nova **substitui** a anterior. A alternativa — recusar enquanto houver uma
— deixaria o modo Cut morto exatamente quando ele parece estar funcionando (o pill aceso, a
caneta a não desenhar nada), e o botão *Discard* existe para o outro caso: tirar a lâmina da
tela quando se acabou de cortar.

# A lâmina não é obra

Ela perde `fill` e `stroke` ao ser adotada, e quem a desenha é o **overlay** (hachurada, com
a tesoura na ponta). Duas razões, e as duas são de produto: uma lâmina que herdasse a cor e a
espessura do traço corrente seria indistinguível de um desenho, e uma lâmina com estilo sairia
no export — que é o mesmo que dizer que ela não é uma ferramenta.

## `morph_live.rs`

> **O host do Morph vivo** — o `t` animável (a forma única entre duas outras).

Espelho do [`crate::connector_live`], e irmão do [`crate::blend_live`]. A diferença entre os
dois irmãos é o que cada um põe na CENA: o blend põe o *spine* (invisível) e desenha os N passos
como overlay virtual; o morph põe **a forma**, porque é a forma que o artista quer — para
pintar, exportar e, sobretudo, **animar**.

Ninguém desenha um morph: move-se uma fonte, ou mexe-se no `t`, e a forma se refaz. A geometria
é **função pura da relação**, re-cozida a cada frame.

# O `Plan` é CACHEADO aqui, e no blend não é — a diferença não é gosto

O `blend_live::cook_links` monta um [`Plan`] por frame e ninguém reclamou: o blend só re-coze
quando o artista mexe em alguma coisa. No morph o `t` muda **todo frame** — é essa a feature —,
e sem cache a busca de fase (256×256) rodaria por frame: os **5,9 ms** que o `Plan` foi
inventado para matar, de volta, agora dentro do orçamento de quadro.

A correspondência é função do **par**, não do `t`. Então a chave do cache é a **geometria em
MUNDO das duas fontes** — geometria *e* pose, não uma delas: sem a pose, o plano descreveria a
forma onde ela **estava** (a mesma lição da chave de sessão do Shape Builder).

## `bool_live_morph_tests.rs`

> **A BOOLEANA VIVA A MEIO DE UMA TROCA DE VERBO** — os gates do que o artista vê quando um
> estado de UI muda a operação (Enio, 2026-08-23).

# A fixture é a DONUT, e a escolha é a metade do trabalho

Um retângulo grande com outro **inteiramente dentro**. `Union` desenha **1** contorno (área
400); `Subtract` desenha **2** — o de fora e o buraco de 64. É o único par em que a TOPOLOGIA
muda, e portanto o único que decide se o morph serve para alguma coisa.

⚠️ **A fixture do irmão (dois retângulos que se cruzam) NÃO contém o fenômeno:** ali os quatro
verbos dão 1 contorno, e um gate escrito sobre ela ficaria verde sem nunca exercitar um buraco
a nascer. Medido antes de escrever uma linha destes gates.

# ⛔ O que foi MEDIDO e recusado (sonda `probe_boolean_morph.rs`)

| tentativa | por quê não |
|---|---|
| Saltar o verbo (Blender / After Effects / Rive) | move **64,0** de tinta num quadro com a peça PARADA, contra 3,1 do morph |
| Crossfade das duas formas (o recuo do Figma) | mostra as duas ao mesmo tempo; o morph não tem fantasma |
| Perseguir a partir do que está na tela | cura o único quadro que salta e paga com o desenho a FICAR PARA TRÁS do movimento: 793,0 de tinta de afastamento numa peça que viaja |

## `smoke_bone_paint_smooth_tests.rs`

> ⭐⭐⭐⭐ **O BOTÃO `Smooth` ENTREGA — os gates do controlo que estava MORTO.**

Irmão do [`super`] por RESPONSABILIDADE: *a arte obedece à lei da pele* é uma pergunta (a dele),
*o segundo segmento do par `Fast`/`Smooth` faz alguma coisa* é outra.

# O defeito (auditoria de 2026-09-16)

O painel *Bones* tem um par de segmentos `Fast`/`Smooth` ([`ph2d_tool_vector::SkinDeform::ALL`],
registado e pintado). Medido nesta cena, o `Smooth` entregava **a malha do `Fast`, ao bit**: a
lei de refinamento de então era o `k` GLOBAL, cujo tecto é `⌊√(orçamento / peças)⌋`, e toda malha
acima de `orçamento / 4` peças só admite `k = 1`.

⛔⛔ **É a terceira espécie de controlo morto do `CLAUDE.md` §5.0, e a sonda de registo é cega a
ela:** o id existe, é pintado, é registado, o clique chega à ferramenta e o valor chega ao
consumidor — *e o consumidor devolve a entrada*. Nenhum censo deste repo pergunta se a saída
MUDA.

⇒ é essa a pergunta que estes gates fazem, e o controlo da resposta é a lei antiga a reprovar.

## `stroke_present.rs`

> ⭐ **DAR (e TIRAR) o traço de uma forma** — a porta única da caixa *Stroke* do painel (plano 34).

# Por que isto existe

Toda forma **autorada** nasce com traço (`shape.rs` escreve `path.stroke = Some(..)`
incondicionalmente), e o `restyle_selected_strokes` **recusa** quem não tem um. ⇒ uma forma que
chegou ao documento por outro caminho — um importador, uma cena montada por código, uma booleana
— ficava **sem forma nenhuma de ganhar um traço**, com a secção *Stroke* do painel a oferecer
controlos que não a alcançavam. Foi o report do Enio de 2026-08-27.

# ⛔ Por que a cura NÃO é no `restyle_selected_strokes`

Aquela função corre **por quadro** sobre a selecção, sempre que o estilo da tool difere do dela
([`crate::render_loop::vector_bridge`]). Criar ali daria traço a **toda** forma sem traço que
estivesse selecionada, **sem ninguém pedir** — e o comentário que lá está (*"ganhar um traço do
nada seria a UI inventando geometria"*) é a cerca certa pelo motivo certo.

⇒ **a criação é um GESTO explícito**, nunca um efeito colateral de um espelhamento.

## `vector_bridge_opacity_tests.rs`

> ⭐⭐⭐ **A OPACIDADE ALCANÇA AS TRÊS TINTAS** (plano 36, W6) — os gates da PONTE.

O motor já responde (`ph2d-vec-scene/src/brush_opacity_tests.rs`); aqui prova-se o outro lado da
costura: que a barra *Opacity* do painel **chega** a cada espécie de tinta, e que quem DETECTA a
mudança concorda com quem a GRAVA.

# Uma opacidade, uma casa

| tinta | onde a opacidade vive | quem a escreve |
|---|---|---|
| `Solid` (traço e preenchimento) | a alfa da cor | o caminho de sempre |
| `Pattern` (traço) | `PatternFill::alpha`, `fallback.a` em sincronia | [`super::StrokeStyle::onto`] |
| `Pattern` (**preenchimento**) | idem | [`super::style::apply_fill_colour`] ⬅ **a W6** |
| `Brush` (traço) | `fallback.a`, e mais nada | [`super::StrokeStyle::onto`] |

⚠️ **As duas metades de um preenchimento de padrão são independentes**, e uma sem a outra é
pior que nenhuma: escrever sem SEMEAR faz a primeira mexida jogar na forma a alfa da forma
**anterior**; semear sem escrever deixa a barra a mostrar o valor certo e a não o mudar.

## `vector_bridge_publish.rs`

> **O que o painel é TOLD** — o passo 5 do bridge do vetor (o `dispatch` faz os passos 1-4).

# Por que este corte, e não outro

O doc-header do pai já enumerava cinco trabalhos, e os quatro primeiros MEXEM no documento
(visibilidade do dock · leitura do picker · o Style que a caneta herda · o restyle da
seleção) enquanto o quinto só **conta ao painel o que já é verdade**. Quando o arquivo
cruzou o teto de 600 LOC (HR-18) essa era a linha de corte que já estava escrita.

⚠️ **Uma publicação ficou no pai, e não por acidente:** `set_current_vector_style` lê o
`ui_snapshot()` da TOOL, e a tool é um empréstimo mútuo do registry que este módulo não
recebe. Ela roda **antes** desta função; a ordem entre as duas é irrelevante (uma escreve
num estático do painel, a outra no store e noutros estáticos), e é por isso que separá-las
é seguro.

⚠️ **A fronteira de DISPLAY mora aqui** (doc 88 do Motion, e a lição que este painel pagou
por ser a QUARTA superfície a responder *onde está esta coisa?*): todo comprimento sai
UMA vez pela `LengthDisplay`, e a TAXA de arrasto cruza a mesma porta que o valor.

## `blend_live_tests.rs`

> Testes do [`crate::blend_live`] — módulo irmão (teto de 600 LOC por arquivo da shell).

O que eles provam, em ordem de importância:

1. **A transição SEGUE a forma** (`moving_a_source_reflows_the_blend`) — a feature inteira numa
   asserção: mover uma fonte re-coza o overlay, sem re-clicar "Blend".
2. **Girar QUALQUER ponta refaz os passos** (`rotating_either_endpoint_reflows_the_blend`) — a
   simetria; o "a última não influencia" do smoke era o CÍRCULO (rotacionalmente simétrico).
3. **A última forma fica POR CIMA** (`the_last_source_is_drawn_on_top`) — a pilha de z do
   Illustrator: fonte0 embaixo → passos → fonte1 em cima.
4. A cadeia é pairwise; o blend vive na identidade; o `settle` o pula; um elo morto é pulado.

# O `out` do recook é o OVERLAY ORDENADO (passos + fontes reempilhadas)

Não é "os passos": é o que o passe de render desenha, em z. Para cada elo, os passos e **a
fonte de cima** (redesenhada por cima deles) — só a fonte0 fica no z da cena. Por isso o tamanho
é `(elos)·(passos+1)`, e o último item é sempre a última fonte.

## `fx_dump.rs`

> **O DESPEJO dos pixels do FX** — a sonda que só o app pode dar.

Irmão de [`crate::fx_live`] pelo teto de LOC, e existe por um motivo medido: o smoke reporta
dentes no feather DENTRO do app, e a sonda headless (`fx_look_probe`) **não os reproduz sob
nenhuma condição** — geometria esparsa, densa, rasterizador próprio ou Vello, tudo dá ripple no
nível do controle. Cada diferença entre as duas foi eliminada por medição; o que resta é o que
só o app tem, que são os pixels dele.

`PH2D_FX_DUMP=<dir>` escreve, por forma cozida e **uma vez só**:

- `path<id>.ppm` — a saída do FX, RGB composto sobre o cinza do app (é a foto);
- `path<id>.pgm` — o ALFA cru dessa saída (é onde a rampa do feather vive);
- `path<id>.txt` — os segmentos da silhueta e os parâmetros da pilha.

Com isso a distância prevista pela GEOMETRIA e o alfa que a GPU de facto escreveu passam a ser
comparáveis offline. Se discordarem, o campo está exato para uma forma diferente da que o Vello
desenhou naquela textura — que é a única hipótese que sobrou.

## `connector_walls.rs`

> **O que uma linha ENXERGA** — de que ela desvia, e a que ela se prende.

Módulo filho do [`crate::connector_live`] (teto de 600 LOC por arquivo da shell) e o lugar das
duas únicas perguntas que ligam o DOCUMENTO à rota: *o que barra a linha?* (as paredes) e *a
que uma ponta se prende?* (o alvo). Estar num arquivo só não é arrumação — as duas respostas
são a mesma regra vista de dois lados, e é essa regra que fecha o laço:

> **A rota de um conector NÃO PODE depender da pose de um rótulo.**

Se depender — por qualquer das duas arestas —, o sistema tem realimentação. O rótulo de um
conector mora **em cima da rota dele** (é onde ele nasce, no meio por comprimento de arco);
então a rota reage ao rótulo, o rótulo se re-centra na rota nova, a rota reage de novo. Foi o
que o Enio viu: *"linha e texto pulando sem parar"*.

Um laço de realimentação não se amortece nem se clampa: **corta-se**. As duas arestas morrem
aqui — [`is_annotation`] (o rótulo não é parede) e [`anchor_target`] (o rótulo não é alvo).

## `fx_live_memo_tests.rs`

> Gates da **CHAVE do memo** do FX raster — arquivo irmão de `fx_live_memo.rs`.

O oráculo é a decisão que o `recook` toma na 1ª varredura, e ela é literalmente **um `==` entre
duas chaves**: igual ⇒ os pixels que estão na textura servem; diferente ⇒ re-coza. Por isso os
gates comparam CHAVES em vez de espiarem a GPU — a decisão inteira mora aqui, e mora aqui de
propósito (ela vivia dentro do `recook`, que precisa de dispositivo, então nenhum teste headless
a alcançava e o defeito abaixo atravessou a wave inteira com 18 gates verdes ao lado).

O defeito medido (2026-07-29): **mudar a cor do preenchimento de uma forma filtrada não mudava a
tela.** A pilha resolvida era a mesma, a caixa era a mesma, então o memo acertava.

⚠️ **Um gate foi escrito e DESCARTADO em vez de shipado:** *"armar um Live Path Effect re-cozinha
a forma"*. Ele passa com a chave ANTIGA (a `(pilha, w, h)`), porque **todo efeito de caminho move
a caixa** — então ele não podia falhar pelo motivo que alegava, e a `effects` viaja para dentro da
chave pelo `VecPath` inteiro de qualquer modo (não há lista de campos a esquecer). Verde por
construção; a razão fica escrita aqui para ninguém o re-escrever achando que prova algo.

## `guide.rs`

> **A porta única de "onde está o caminho-guia, e como se percorre por arco?"**.

Um caminho-guia é um `VecPath` normal — geometria LOCAL + pose no `Transform` (ADR-0111). Para
algo o *cavalgar* (o texto em caminho, o Pattern Along Path, e o que vier), ele é lido **cozido
e assado em MUNDO** e virado num [`ArcPath`]. Essa resolução tem três cuidados que discordar é
bug:

1. **cozido**, não a fonte crua — senão o que cavalga ignora as Live Corners e a pilha de
   efeitos do guia (ADR-0121/0132);
2. **assado pela pose de mundo** do guia — senão o padrão assenta onde o caminho NÃO está, e
   mover o caminho deixa de mover o que o cavalga (a metade visível da feature);
3. `total > 0` — um guia degenerado (menos de dois vértices, comprimento zero) não tem por onde
   ser percorrido, e quem divide pelo total tem de o saber.

O texto e o pattern faziam (ou fariam) esta MESMA pergunta. Duas cópias divergem no dia em que
uma ganha um cuidado e a outra não — por isso mora aqui, e o `vec_text_ride::guide_of` delega.

## `offset_live_tests.rs`

> Gates do **Offset VIVO** — arquivo irmão de `offset_live.rs`.

O fato do USUÁRIO que esta família existe para pinar (Enio, 2026-07-21):

> *"no momento que aperto Round a curva já tem todos os vertex novos criados antes de
> apertar apply … a idéia é bevel desfazer round e aplicar bevel. Só apply aplica
> definitivamente o efeito."*

Ou seja: enquanto o offset é PREVIEW, o documento guarda a curva AUTORADA — e é ela que o
modo Node mostra. O resultado é DESENHO. `Apply Offset` (ou `Convert to Curves`) é o único
momento em que os vértices do offset passam a existir.

⚠️ **O oráculo NÃO é "os três joins diferem"** — isso é verdade mesmo compondo, e foi por
isso que a rodada anterior ficou verde sobre o produto que o Enio reprovou. Os oráculos
daqui são: (a) a curva autorada BYTE-IDÊNTICA através da cadeia de previews; (b) a
identidade com um resultado FRESCO da forma pristina, âncora a âncora.

## `snap_sprites.rs`

> ⭐ **A caixa de um SPRITE como alvo de encaixe — a parte PURA**, fora da shell (W2/L4, A2).

⚠️ **Nenhum concorrente tem isto, e a razão é estrutural:** nenhum deles mistura raster e
vector na mesma hierarquia. Nós misturamos (ADR-0110), então alinhar uma forma à borda de um
sprite é um gesto tão comum quanto alinhá-la a outra forma — e não havia alvo nenhum.

# ⚠️ O módulo mantém o nome, e o irmão que ficou

`sprite_snap_points` — quem varre o mundo à procura de sprites — **ficou na shell**, e não por
tocar `gfx`: ele chama `crate::transform`, que não pôde sair porque **declara** por
`#[path]` um teste que precisa de `crate::profile_live`. *A aresta que prende um ficheiro pode
estar num módulo que ele declara, não no que ele usa.*

O nome do módulo (`vec_snap_sprites`) é preservado porque ele entra no nome de cada teste: numa
lista de `cargo nextest`, um módulo renomeado durante uma extracção é indistinguível de testes
perdidos.

## `appearance.rs`

> ⭐⭐⭐ **A APARÊNCIA DO OBJECTO, do lado da shell** (estudo 42 item 2, v19 do schema) — o que o
> painel mostra da selecção, e o que um gesto dele escreve no documento.

Duas propriedades, e as duas são da FORMA e não da tinta: a **opacidade** (a forma inteira
compõe-se uma vez e desvanece) e o **modo de mistura** (como ela se compõe com o que está por
baixo dela na pilha de z).

# A lei da selecção múltipla: MOSTRA «misto» onde diferem, ESCREVE o campo tocado em todas

⭐ **Plano 31 da UI nova (decisão do dono, 18/10)** — revoga o «mostra o primário» que aqui
estava. O primário mentia sobre as outras formas, e com o restyle a gravar o estilo INTEIRO ele
APAGAVA: mexer na Width de duas formas pintava as duas com a cor do primário. O terceiro estado
já existia nos widgets (amostra «misto», número em branco do BulkSelect do Inspector), e a
objeção registada — *«o artista fica sem saber o que vai acontecer ao arrastar»* — cai com a
escrita POR CAMPO: arrastar dá a todas esse valor, e nada mais muda. O slider fica no primário.
A leitura mora em `paint_stack::published` (`mixed_opacity`, `mixed_blend`).

## `autokey_smoke_bloco2.rs`

> **As cenas do smoke «o Auto Key grava o bloco 2»** (F25) — `PH2D_AUTOKEY_SMOKE=11..13`, todas
> a mexer sozinhas (a sonda mede) e a parar no quadro 18 para a foto.

- `=11` VETOR: uma estrela com um FILTRO (Drop Shadow) e um perfil de largura (afina nas pontas),
  e um TEXTO ao lado; Auto Key ligado. No quadro 12: o raio e a cor da sombra, o perfil (engrossa
  no meio), o tamanho e a FAMÍLIA da fonte do texto e a quebra para Fixed; no 24 tudo volta; pára
  no 18 com a estrela seleccionada e o painel rolado até aos filtros.
- `=12` a mesma, a acabar com o TEXTO seleccionado e o painel rolado até ao texto.
- `=13` INSPECTOR: uma sprite; no quadro 12 o skew, o tint e a emissão; no 24 voltam; pára no 18
  com o Inspector à frente.
- `=14|15` as juntas — [`crate::autokey_smoke_juntas`].

⚠️ O texto nasce de um rectângulo cuja forma vira `VecShape::Text`: o desenho dos glifos coze-se
na shell (a fonte do sistema), e a porta que a cena alcança é a mesma da curva — um tamanho
escrito em `CanaisEscritos` ANTES de o Auto Key ligar, que o `canais::aplicar` devolve em
`textos` e a shell re-coze. As mudanças do quadro 12/24 vão ao componente, como o painel faz.

## `overlay.rs`

> O que o frame vetorial desenha POR CIMA da cena, por modo de desenho (ADR-0112).

Há duas famílias de overlay, com regras de visibilidade DIFERENTES — confundir as
duas foi o bug do P1 (as guias de snap sumiam no modo Select):

- **Overlays de edição** (âncoras, handles bézier, alças de gradiente, marquee de
  box-select): só FORA do Select. No Select quem transforma a forma é o gizmo de
  sprite; as alças de nó comeriam o clique dele (ADR-0112).
- **Guias de snap** (as linhas que explicam o encaixe vivo): valem em QUALQUER
  modo. O gizmo-move do Select encaixa a forma igual ao pen/shape-tool/node-edit,
  então a guia precisa aparecer lá também — era o que faltava.

A decisão vive aqui, fora do `render_loop`, para ser testável sem gfx/câmera: é o
ponto que trava o P1 (uma regressão que re-agrupasse a guia sob o guard de modo
quebra o teste abaixo, não só o smoke).

## `overlay_diag.rs`

> **Diagnóstico do overlay vetorial** — `PH2D_VEC_OVERLAY_DIAG=1`.

Existe por um artefato que eu **não** consegui explicar por dedução: uma linha reta longa saindo
de uma forma envolvida por envelope (Enio, 2026-07-18). Três hipóteses foram MEDIDAS e refutadas
— a jacobiana do MLS é exata até o pino (não é cancelamento catastrófico), o kurbo não desenha
nada numa haste de comprimento zero, e um segmento reto/degenerado atravessa o `warp_path` sem
produzir ponto disparado.

Quando a dedução falha três vezes, o passo certo não é a quarta hipótese: é **instrumentar o
produto** ([[feedback_harness_reproduces_mechanism_not_context]]). Este módulo imprime, do frame
REAL, quem tem geometria fora do lugar — e nomeia o dono do artefato em vez de o adivinhar.

O sinal que ele procura é o que a foto mostra: um ponto de controle **muito** mais longe do que a
própria forma mede. Uma alça sã vive perto da curva; uma disparada é a assinatura de uma tangente
inválida a chegar ao fitter.

## `paint_stack.rs`

> ⭐⭐⭐ **OS VERBOS DA PILHA DE APARÊNCIA, do lado da shell** (estudo 42 item 4, v20).

O que o painel MOSTRA da selecção, e o que cada clique dele escreve no documento.

# A lei da selecção múltipla é a MESMA da aparência do objecto

**Mostra o primário, escreve em todos** — a lei que a fileira de verbos booleanos já paga, e que
veio de um report: *tocar num filho selecciona o GRUPO*, então o sujeito de um readout é a forma
que o artista apontou.

⚠️ **E a ESCRITA é por ÍNDICE**, o que tem uma consequência declarada: acrescentar uma camada
acrescenta-a a **todas** as formas seleccionadas, e mexer na camada `2` mexe na `2` de cada uma.
Isso é o que o artista espera quando escolhe cinco formas e carrega em *+ Stroke*; ⛔ o que ele
**não** espera é que uma forma sem camada `2` ganhe uma — e por isso um índice que não existe é
**saltado**, nunca criado.

## `vec_bool_pick_tests.rs`

> **O OPERANDO ABSORVIDO** — os gates do pick sobre a booleana viva, módulo irmão de [`super`]
> pelo teto de LOC.

Um grupo booleano vivo dá a tinta do resultado ao operando mais ao FUNDO e escreve uma lista
**VAZIA** para todos os outros (`bool_live`). A lei do hit-test é *nada desenhado, nada pego* —
logo, até 2026-08-22, cada um desses operandos era **inalcançável pelo canvas**: o artista
configurava a booleana e passava a só conseguir agarrar UMA das formas (report do Enio).

A cura não fura a lei, dá ao operando a porta que ele de facto tem: **a tinta do GRUPO que o
absorveu**. É por isso que estes gates vêm aos pares — um que prova o alcance, e o seu gêmeo
que prova que a tela limpa continua limpa. O par que interessa mais é o do `Subtract`: o
cortador ocupa exactamente o buraco, então uma cura que o alcançasse pelo próprio footprint
passaria no gate do alcance e **falharia** no da tela limpa.

⚠️ Mutação canônica: fazer `VecViewState::absorbed_door` devolver sempre `None` (o produto de
antes desta wave) tem de SANGRAR os gates de alcance e deixar VERDES os de regressão.

## `vector_mode_fill_stroke.rs`

> ⭐ **As cenas 45 e 46** (`PH2D_OBJECT_MODE_SMOKE=45|46`, F44 → F44b — Fill e Stroke no layout do
> Figma, todas as linhas iguais). Um rectângulo com degradê linear (azul → roxo) de base, traço
> escuro de base, e três camadas extra: uma SOMBRA (preenchimento deslocado), um BRILHO radial
> (branco → transparente) e um contorno amarelo TRACEJADO por fora — selecionado, com o Inspector
> rolado até à secção Fill. A 45 abre o ⚙ do BRILHO (uma camada extra com tipo e degradê, como a
> base); a 46 esconde o traço de base pelo olho e abre o ⚙ do contorno amarelo (Position, pontas,
> junções, tracejado — o «Stroke settings» de uma camada). F44c: a 45 acaba na seta branca, com
> as alças do degradê do brilho no canvas.

⭐ **A cena 47** (F45 — Fill & Stroke numa só pilha, na ordem livre): o mesmo rectângulo com o
traço de base GROSSO, e a pilha fora da ordem antiga — a SOMBRA por baixo de tudo (o que antes
era impossível), o PREENCHIMENTO de base por CIMA do traço de base (cobre a metade de dentro
dele) e o contorno amarelo no topo (um traço por cima do preenchimento). Nenhum painel aberto: o
dono arrasta os ⚙.

⭐ **A cena 48** (os lados CAD curvos): a 47 com um canto puxado e outro preso por uma regra CAD — ver
[`com_regra_e_um_ponto_puxado`].

⭐ **A cena 49** (F47, «Above Effects»): sombra e desfoque nos Filtros; o traço de base marcado por
cima deles e o amarelo não — ver [`por_cima_dos_filtros`].

⭐ **A cena 50** (o Expand sobre a pilha, dono 02/11): a «etiqueta» — ver [`etiqueta`].

## `connector_panel_tests.rs`

> Testes do [`crate::connector_panel`] — módulo irmão (teto de 600 LOC por arquivo).

O que eles provam, em ordem de importância:

1. **A seção existe SSE há conector na seleção** (`selection_snapshot`): sem isso o
   painel mostraria campos de um objeto que não é conector — ou esconderia os de um que
   é. O teste é bilateral: inverter a condição sai vermelho dos dois lados.
2. **A edição atinge TODOS os selecionados** — é o pedido do Enio (calibrar o diagrama
   inteiro de uma vez). Editar só o primeiro passaria num teste de unidade do componente
   e falharia na mão do usuário.
3. **O campo nasce no valor EFETIVO** (o automático), não em zero — senão o número salta
   no primeiro toque, longe da linha que o olho vê.
4. **Editar FIXA** (`None` → `Some`) e o valor fixado **não é re-derivado** no frame
   seguinte, nem quando as caixas mudam de tamanho.

## `bool_gesture.rs`

> **Os gestos da BOOLEANA VIVA** — armar, re-mirar e consolidar.

O motor é o [`crate::bool_live`]; aqui mora o que é de DOCUMENTO (o grupo, o z, o undo) e a
decisão que os oito botões da seção Boolean tomam quando são clicados.

# Um clique, três destinos — e a ordem deles é a lei

1. **A seleção já está dentro de um grupo booleano** ⇒ o clique RE-MIRA aquele grupo. É o que
   faz os oito botões serem também o seletor de operação de um grupo vivo: uma lista de oito
   verbos, e clicar num deles diz *"passe a ser este"*. Um dropdown separado seria uma segunda
   porta para a mesma pergunta, e ela divergiria no dia em que a nona operação entrasse só num
   dos dois.
2. **O modo `Live` está ligado** ⇒ o clique CRIA um grupo com os selecionados.
3. **Senão** ⇒ o caminho destrutivo de sempre, byte-idêntico.

⚠️ **(1) vem antes de (2) de propósito.** Sem essa ordem, re-mirar um grupo com o modo
desligado o CONSUMIRIA — o artista clicaria "Intersect" esperando trocar a operação e perderia
os operandos.

## `vector_mode.rs`

> ⭐⭐ **VECTOR ▸ EDIT** — cada FORMA é um objecto, e o Edit abre a ferramenta do vetor sobre ela
> (spec/06 F3 ▸ Vector, 2.ª volta; escolha do dono, 05/10: *«nada aparece no canvas ou na hierarquia
> até que o usuário crie alguma forma ou linha. Ao desenhar algo, o objeto aparece na hierarquia no
> modo edit»*, e a 2.ª forma é OUTRO objecto).

- **Add ▸ Vector Drawing** não cria entidade: arma a ferramenta ([`EditTarget::arm`]) — ela chega
  à mão em Object, com o painel inteiro.
- **A forma que NASCE com a ferramenta na mão pede o Edit sobre ela** ([`ModeFamily::wants`]): a
  1.ª depois do Add, e cada nova num Edit (o Edit PASSA a ela — a rede `still_holds` larga a
  anterior e o pedido entra na nova, no mesmo quadro).
- **Edit** = a ferramenta `vector` na mão (qualquer `DrawMode`). ⭐⭐ **O Edit é do TIPO** (dono,
  05/10: *«todos os objetos daquele tipo estão em modo de edição»*, [`ModeFamily::holds_the_whole_kind`]):
  toda forma se agarra e é parte do Edit; escolher um objecto de outro tipo volta a Object, e
  escolher outra forma — ou várias — FICA no Draw ([`ModeFamily::stays_on_its_kind`], dono 02/11:
  as booleanas precisam de várias formas).
- **A forma trancada que MORRE** (o Soldar consome os traços, a caneta junta dois caminhos) passa o
  Edit à [herdeira](ModeFamily::heir) — a última forma viva da caneta.
- **Object** = a ferramenta sai da mão.
- ⭐ **O ALVO** mora no [`crate::state::VecState`], fora da `VecScene` (que entra no undo).

## `lib.rs`

> ⭐⭐⭐ **A família `vec` da shell, fora da shell** — W2/L4 (auditoria de velocidade de
> 2026-09-10 §4-C2, DIRETRIZ §6.7 item 5).

# O que esta crate é

A `shells/desktop` é um `bin` de **493 k linhas** que dobrou em quatro semanas e é a **última
unidade de todo build grande** (34–45 s sozinha no fim do gate de fecho). Dentro dela vivem
famílias inteiras que só ali estão por inércia — e a metade-shell do módulo vectorial é uma
delas: **129 ficheiros, 17 527 LOC de produto e 17 933 de teste**.

Esta crate é o destino dessa família. O sentido é **sempre shell → crate**: a shell é um
binário, logo nada aqui pode referir `crate::` da shell. Quem precisa do que só a shell tem —
janela, `gfx`, painéis, a captura do undo — **fica lá**, e isso é o ADR-0075 outra vez (estado
de família é recurso/componente do ECS; o resto é composição da raiz).

A história da extracção (o conjunto fechado, o molde): `docs/Vector Module/50_cabecalhos_do_app_vec.md`.

## `vec_offset_pick_tests.rs`

> **A forma que se VÊ é a que se PEGA** — os gates do pick sobre o Offset vivo
> (`ph2d_ecs::VecOffset`), módulo irmão de [`super`] pelo teto de LOC.

O ADR-0121 separou a curva AUTORADA da COZIDA e o offset vivo (2026-07-21) levou a costura ao
nível do OBJETO: o documento guarda o `VecPath` que o artista desenhou e a tela mostra a
derivada. Quem desenha já pergunta ao `offset_live` — o hit-test passou a perguntar à MESMA
porta, senão a forma fica **pintada num lugar e clicável noutro**, sem erro e sem aviso: o
artista clica no meio do que vê e nada acontece.

⚠️ **Os dois sentidos são gates separados de propósito.** Um remendo que apenas SOMASSE a
derivada à fonte (união) passa no gate de crescimento e falha no de encolhimento — e o
encolhimento é onde mora o modo de falha oposto, *pegar a forma onde a tinta já não está*.

Mutação canônica: fazer o `contains_path`/`world_bbox` ignorarem o `live` (o produto de antes
de 2026-07-21) tem de SANGRAR os quatro primeiros gates e deixar o de regressão VERDE.

## `bool_reach_tests.rs`

> **A BOOLEANA VIVA ALCANÇA OS ESTADOS** — os gates de ROTA da auditoria de 2026-08-23.

# A pergunta que estes gates fazem

*Com uma booleana em mãos, o artista consegue chegar às poses dela?*

⚠️ Ela não é a mesma que *"o widget existe, é pintado e o clique chega ao barramento"* — essas
três estavam **verdes** enquanto a resposta a esta era **não**. Tocar um operando seleciona o
GRUPO inteiro (lei deliberada do `object_selection_for`), o `publish` exigia forma ÚNICA, e a
seção STATES — com o interruptor **Preview** dentro dela — não era sequer pintada. Não dimmed,
não vazia: ausente, sem uma palavra a dizer o que faltava.

É um gate de alcance: o que se mede aqui é a SEQUÊNCIA.

## `shape_build_tests.rs`

> Gates do Shape Builder — o gesto inteiro, sem gfx.

## A fixture é PARTE do gate, e era ela que estava errada

A 1ª versão destes testes usou quadrados eixo-alinhados construídos à mão, na identidade.
Dezesseis gates verdes, e o Enio reprovou o smoke no primeiro clique. **Um gate só prova
o que a fixture dele contém**: mutar o código e ver vermelho dentro de um universo de
quadrados só prova coisas sobre quadrados.

Aqui a fixture é a do produto: formas do **catálogo** (curvas), **centradas no local 0**,
com a pose num `Transform` (ADR-0111) — que é o que a Shape tool deixa na cena. A sessão
é aberta pelo `BuildSession::open` de verdade, e o resultado é aplicado pelo `commit` de
verdade: o que se mede é **a cena que o artista fica**, não o retorno de uma função.

## `vec_gizmo_view_hit_tests.rs`

> **O que se VE, se CLICA** — os gates da area de captura de um traco.

Enio: *"selecionar a seta esta dificil pois a area da seta e fina"*. A causa nao era o raio
de captura: era que METADE DO DESENHO nao existia para o mouse. A ponta de seta e construida
a partir do caminho + do `StrokeSpec` (`stroke_head`) e nunca entrou no `VecPath`, entao o
hit-test jamais a viu — clicar no triangulo, que e a parte gorda e a que o olho mira, nao
selecionava nada.

**A calibragem que estes testes exigiram, e que vale registrar:** a cabeca e proporcional a
LARGURA do traco. Com uma linha curta e um traco grosso, o triangulo cobre metade da cena — e
a primeira versao destes testes ficou vermelha nas asserções NEGATIVAS, porque os pontos que
eu supunha "longe da seta" caiam dentro dela. O bug era do teste. Uma linha longa e um traco
fino poem cada coisa no seu lugar.

## `canais_topologia.rs`

> ⭐⭐⭐ **Acrescentar ou apagar um ponto numa forma ANIMADA vale em TODAS as chaves** (decisão do
> dono, 20/10: a forma inteira numa chave, AE/Spine — plano 35 §2 escolha 4).

Sem isto, a forma deste quadro passava a ter `n ± 1` vértices e as chaves dos outros instantes
continuavam com `n`: o vértice `j` de uma chave casava com o `j` de outra que já era outro ponto,
e a forma TORCIA entre elas — o defeito do shape tween do Flash.

**Sem adivinhar a correspondência:** as âncoras que a edição não tocou ficam IGUAIS bit a bit
(inserir num segmento só mexe nas alças dos vizinhos; apagar só tira), então o alinhamento é a
maior subsequência comum de âncoras EXACTAS entre o quadro anterior e este. Um ponto novo acha o
seu parâmetro `u` projectando-se no segmento de ontem, e em cada chave esse segmento divide-se no
mesmo `u` (de Casteljau) — a forma de cada chave não muda de desenho, ganha um ponto. Um ponto
apagado sai de todas. Funciona para TODA porta que edita pontos, sem que nenhuma o declare.

## `expand_scale_tests.rs`

> Gates da **LEI DA FORMA** do slider de Offset (report de 2026-07-20: *"se selecionar
> Round, não consegue mudar"*) — arquivo irmão de `vec_expand_tests.rs` pelo teto de LOC.

A faixa antiga (±4 unidades de MUNDO, constante) entregava o gesto natural — arrastar
até o fim do track — a regimes **join-inertes**: à esquerda a forma ANIQUILA (os três
joins produzem o mesmo nada), à direita ela estoura a tela e as quinas saem de vista. A
janela de retune funcionava; o dial é que apontava para onde o join não pode aparecer.
A lei nova: `d = fração × offset_scale(seleção)`, com `offset_scale = maxdim/2` — o
extremo esquerdo é **morte garantida** (inradius ≤ maxdim/2) e o direito é **dobrar a
forma** (o eixo maior cresce exatamente 2×, quinas na vizinhança da tela).

⚠️ Mutação canônica: fazer [`offset_scale`] devolver a constante `4.0` (a faixa velha)
tem de SANGRAR os dois primeiros gates — foi exatamente o produto que o Enio reportou.

## `pick.rs`

> O **Picker de caminho-guia** — o gesto de duas mãos partilhado pelo Pattern on Path e pelo Text
> on Path (Enio 2026-07-23: *"um botão Picker onde o usuário primeiro seleciona a shape, depois
> aperta o botão e seleciona o path. Dessa forma é mais correto"*).

A disambiguação por seleção (`link_candidate`) tem de ADIVINHAR qual dos selecionados é a fonte
e qual é o guia — e adivinha errado quando as premissas não valem (o *"escolhendo a si mesmo"* que
o bbox só resolve no caso comum). O Picker remove a adivinhação: a **fonte** é a que estava
selecionada quando o botão foi apertado, o **guia** é o que o clique seguinte acerta. Um é
capturado, o outro é apontado — nunca há dois candidatos para a mesma vaga.

Este módulo é só o TIPO do pedido + o realce do hover; a captura (no `render_loop`), o clique (no
`input_dispatch`) e o vínculo em si (`pattern_live::link` / `vec_text_ride::link_explicit`) moram
onde já moram os seus irmãos, porque o Picker só troca a PROCEDÊNCIA do par — a inserção do
componente é a mesma que a auto-ligação já fazia.

## `smoke_bone_efeitos.rs`

> ⭐⭐⭐ **OS EFEITOS NA PELE** — `PH2D_VEC_BONE_SMOKE=5` (F50, ordem do dono de 2026-10-02:
> formas vetoriais presas a ossos COM efeitos).

# O que a cena monta

Seis barras iguais, cada uma com um esqueleto igual de [`OSSOS`] ossos, todas dobradas em S a
[`DOBRA`] por junta. A de cima à esquerda NÃO tem efeito (o controlo); cada uma das outras tem
um ([`EFEITOS`]). ⭐⭐ Desde 2026-10-03 o Bind COZE os efeitos no desenho (ordem do dono: *«ao
aplicar os bones, os efeitos são cozidos antes»*, ver `skin_live::bind`) — o efeito desenhado em
repouso dobra com a barra, e a barra presa já não recebe efeitos (a secção Effects diz porquê).

⚠️ **A dobra é aplicada DEPOIS de prender** — prender fotografa a pose de repouso (ver o
[`crate::smoke_bone_par`]).

## `smoke_bone_paint_silhueta_tests.rs`

> ⭐⭐⭐ **A SILHUETA DESENHADA VAI E VOLTA?** — a régua que o olho usa, e o gate que a pôs no
> produto (report do dono, 2026-09-16, foto com cinco setas: *«Smooth parece ter resultado
> discretamente inferior, gerando micro irregularidades»*).

Filho do [`super`] por RESPONSABILIDADE: *o `Smooth` refina onde a dobra pede* é a pergunta
dele; *o que ele desenha é mais liso que o `Fast`* é esta.

⛔⛔ **O gate do irmão estava VERDE sobre o defeito, e a razão é que a régua dele partilhava a
lei do produto:** ele mede o desvio da malha contra o CAMPO que ela segue, e o campo P1 tinha um
vinco em cada aresta do bind — o `Smooth` seguia os vincos fielmente e a régua aprovava. *Um
espelho não acusa.* A régua daqui não sabe que campo existe: ela olha só a polilinha desenhada.

⚠️ **A arte desta cena é um rectângulo OPACO**, logo a silhueta é a fronteira da malha e os
quatro lados dela são os quatro lados do rectângulo de repouso.

## `vector_bridge_fields.rs`

> ⭐ **Campo a campo** — a escrita e a leitura da seleção múltipla (plano 31,
> `docs/UI_New_and_Simple/vetor_nova_ui/31_plano_F20_misto_e_espessura_do_documento.md`).

Módulo irmão do [`super::style`] (teto de LOC). Duas metades da MESMA máscara
([`StyleFields`]):

- **escrever** ([`StrokeStyle::onto_fields`], [`fill_merge`], [`restyle_selected_fields`]): um
  gesto escreve em cada forma selecionada só os campos que o artista mexeu. ⛔ Antes a ponte
  gravava o estilo INTEIRO da tool (= o do primário) em todas: mexer na Width de uma forma
  vermelha e de uma azul pintava as duas de vermelho.
- **ler** ([`mixed_of`]): os campos em que as selecionadas diferem, que o painel pinta como
  «misto». ⛔ Revoga a lei «mostra o primário» do `appearance.rs` (decisão do dono, 18/10): o
  Inspector do mesmo app já mostrava «misto», e o primário mentia sobre as outras.

## `blend_live_edit.rs`

> As operações de **edição e interação** do Blend Object (ADR-0128 C1/C2b) — módulo irmão do
> [`super`] (teto de 600 LOC do shell). O núcleo do objeto vivo (component, `recook`, os helpers de
> geometria) fica em `blend_live.rs`; aqui moram as ações que o painel e o canvas disparam:

- escolher as fontes (por z ou por clique) e ajustar os passos ao vivo;
- **Reset Spine** (volta ao automático);
- a **prévia do Pick Shapes** (contorno + polilinha da ordem);
- as reconciliações âncora↔forma: arrastar o SPINE move todas as fontes (Select), arrastar uma
  ÂNCORA move a forma dela (Node) — o que faz "editar a curva ≡ mover a forma".

`use super::*` traz os privados do pai (`center_of`/`world`/`spine_verts`/`spine_stroke`/
`shift_vertex_to`/`MAX_BLEND_SOURCES`/`BlendSpines` + os `use` dele), como os módulos de teste já
fazem.

## `blend_tests.rs`

> Os gates da POLÍTICA do Blend (a math já está gateada no motor, `ph2d-vec-blend`).

O que se prova aqui é o que o artista vê acontecer com o DOCUMENTO:

1. as **fontes sobrevivem** (é um Blend, não uma booleana — ela é que consome os operandos);
2. os passos nascem **entre** elas, no z delas;
3. cada chamada é **UM** passo de undo;
4. sem duas formas fechadas, não faz nada (e não corrompe a cena).

(Este é o modelo DESTRUTIVO legado — só os smokes `PH2D_BUILD_SMOKE=7/8/9` o chamam. O painel
usa o Blend Object VIVO; os gates dele estão em `blend_live_tests.rs`.)

## `marquee.rs`

> **O gesto de REGIÃO do modo Node** — o retângulo de sempre e o LAÇO, numa estrutura só.

O marquee era um par de cantos (`((f32,f32),(f32,f32))`). O laço precisa do CAMINHO, e a
tentação é um segundo campo ao lado — que seriam dois lugares para limpar, dois para pintar e
dois caminhos de release, com o par a divergir no primeiro refino. Aqui é **um gesto**, e a
forma dele é um campo.

⚠️ **A forma é decidida no PRESS e congela até soltar** ([`ph2d_tool_vector::params::MarqueeShape::for_gesture`]).
Relê-la por movimento faria largar o Ctrl no meio do arrasto morfar a região sob a mão: o
artista veria o caminho que desenhou virar um retângulo entre dois pontos que ele nunca
escolheu. É a mesma lei da régua congelada no `Begin` do arrasto de exposição da tira do Flip.

## `smoke_bone_media_bracos.rs`

> ⭐⭐⭐ **AS CENAS DO BRAÇO** — `=2`, o TESTE NULO, e `=3`, o braço que ANIMA. Filho do
> [`super`] para herdar as fixturas dele.

Ordem do dono (2026-09-18): *«crie os 3 tipos do mesmo tamanho e mesma largura para eu ver como
se dobram, se se dobram igual uma a outra»*.

⚠️ **A pergunta é OUTRA e por isso o ficheiro é outro.** O pai monta a cena `=1`, que pergunta
*que arte cada mídia desenha* — artes diferentes de propósito. Aqui a arte, o tamanho, o
esqueleto e a dobra são os MESMOS, e o certo é **não se ver diferença nenhuma**.

⚠️ Ele é filho e não irmão porque o motor de que precisa — `uma`, `tamanho`, `eixos_em`,
`corrente_em`, `dobra` — é do pai, e **duplicá-lo poria as duas cenas a correr leis diferentes.**

## `smoke_bone_paint_pixels_tests.rs`

> ⭐⭐⭐ **O QUE O OLHO VÊ, EM PIXELS DE ECRÃ** — filho da [`super`] (a assadura) para herdar as
> fixturas dela.

⛔⛔⛔ **Este módulo nasceu do report do dono de 2026-09-17** (*«Como eu já havia dito muitas
vezes: Fast e Smooth estão sempre idênticos. Nada mudou»*), e a primeira coisa que ele mediu foi
que **o dono tinha razão**.

⚠️ **A régua de toda esta linha era o desvio ao CAMPO, em pixels da ARTE** — uma propriedade da
APROXIMAÇÃO. Ninguém vê uma aproximação: vê-se a tinta no ecrã. Medido aqui, as duas leis punham
cada texel a `0,04 px` uma da outra na mediana e `0,34 px` no pior ponto, na dobra que a cena
ship. *Uma régua que mede a fidelidade ao modelo não responde «isto muda alguma coisa aos
olhos?».*
