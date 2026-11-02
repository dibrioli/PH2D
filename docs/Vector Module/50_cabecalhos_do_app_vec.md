# A história dos cabeçalhos do `ph2d-app-vec` (movida do código, F47, 2026-11-02)

Verbatim: o `//!` de cabeçalho de cada módulo abaixo, menos o 1.º parágrafo (que ficou no ficheiro). Saíram
do código pela catraca de LOC por crate (`the_shell_only_shrinks_and_every_app_crate_too`, tecto `93 637`) e
pela lei do CLAUDE.md §2 («o porquê histórico vive no doc, com link; o código diz a lei»). Os links
intra-doc do rustdoc (`[`crate::x`]`) ficaram como texto. Texto à data do commit `d95d9a1bea`.

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
