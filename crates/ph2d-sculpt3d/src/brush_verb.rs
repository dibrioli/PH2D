//! **O CATÁLOGO** — os verbos e o que cada um significa, cortados por ASSUNTO
//! do [`super`].
//!
//! ⚠️ **Quantos eles são não está escrito aqui, e é de propósito:** o cabeçalho
//! já disse *"dezassete"* enquanto a [`Verb::ALL`] listava dezanove, e uma
//! contagem em prosa é a primeira coisa que uma wave esquece. Quem quer o número
//! conta a lista, que é a fonte.
//!
//! O pai responde *que pincel está na mão* (raio, força, curva, os knobs); aqui
//! mora *que OPERAÇÃO ele executa* e as portas que perguntam sobre ela. **Quanto
//! cada família desloca** saiu para o irmão [`super::magnitudes`] — as três
//! crescem por razões diferentes, e foi a terceira que levou este arquivo ao teto
//! de LOC.

use super::*;

/// O que o pincel FAZ. Ver `docs/3D/04.1` para a família de cada um.
///
/// ⚠️ **O [`Verb::Layer`] ESTÁ aqui desde a W8** — esta caixa dizia *"não está,
/// e a premissa que o mantinha fora morreu"*, e ela descrevia o mundo de
/// 2026-08-12. O que sobrevive dela é o argumento, porque é ele que separa os
/// dois verbos hoje: a wave da paridade (2026-08-11) fez o
/// [`crate::Grip::Stamp`] **COMPOR** e **matou o envelope**, então
///
/// - **Draw + Accumulate** (`from_live`) — o vértice e o centro sobem juntos, o
///   pincel **não se esgota**, e um traço demorado empilha sem teto;
/// - **Draw sem Accumulate** — o vértice atravessa `dist >= 1` e **sai da
///   pegada sozinho**, o que auto-limita por GEOMETRIA (*o vértice andou mais
///   que o raio*), um número que muda quando o artista muda o pincel;
/// - **a DEMÃO** para numa **ALTURA escolhida**, que não muda com o raio.
///
/// ⚠️ **E a segunda metade daquela nota foi REFUTADA por medição, não
/// construída:** ela prometia *"estado persistente por vértice, que obriga um
/// plano novo a entrar no `ModelSnapshot` do undo no mesmo commit"*. Medido: na
/// referência o deslocamento acumulado por vértice do *Layer* vive no **cache do
/// traço** — construído no pen-down, **destruído no pen-up** —, logo é estado de TRAÇO e
/// não do documento; e do nosso lado ele nem sequer é um plano novo, porque o
/// `accum` que o motor já guarda **é** ele (ver [`crate::GripLaw::coat`]).
/// *Um custo nomeado num plano é uma afirmação sobre um número que a medição
/// pode remover.*
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Verb {
    /// Empurra ao longo da normal **da ÁREA** (uma direção para o dab inteiro) —
    /// é isso que faz um domo liso em vez de um ouriço.
    #[default]
    Draw,
    /// Empurra cada vértice ao longo da **própria** normal: a forma ENGORDA,
    /// que é uma palavra diferente de "sobe". (A distinção Draw×Inflate é
    /// exatamente esta, no Blender e no ZBrush.)
    Inflate,
    /// Puxa cada vértice para a média dos vizinhos — o laplaciano.
    Smooth,
    /// O laplaciano com o sinal trocado: afia o que o Smooth arredondaria.
    Sharpen,
    /// Projeta no plano ajustado à pegada; move nos dois sentidos.
    Flatten,
    /// O plano, só para CIMA — enche vale sem raspar crista.
    Fill,
    /// O plano, só para BAIXO — raspa crista sem encher vale.
    Scrape,
    /// O plano deslocado para fora: o barro que se ADICIONA (é o Flatten com
    /// Offset > 0; os dois knobs ficam na tela, e por isso não é um verbo que
    /// esconde um número).
    Clay,
    /// Puxa tangencialmente para o centro do dab: afia uma aresta.
    Pinch,
    /// O oposto do Pinch: empurra para fora, alarga.
    Magnify,
    /// Pinch + deslocamento negativo — cava um vinco.
    Crease,
    /// **O BOLO DE BARRO** — o [`Self::Crease`] com o aperto lateral INVERTIDO:
    /// em vez de puxar o barro para o eixo (afiando), ele o empurra para fora
    /// (arredondando), e o depósito sobe.
    ///
    /// ⚠️ **A relação é a da referência, ao pé da letra** — lá o *Crease* e o
    /// *Blob* são **a mesma lei**, com um único booleano a trocar o SINAL do
    /// termo lateral e mais nada; o deslocamento normal dos dois é o mesmo
    /// produto **normal × raio × força**.
    ///
    /// ⚠️ **E é por isso que ele é um VERBO e não um slider negativo no
    /// `pinch`:** o nosso próprio catálogo já decidiu esta pergunta uma vez —
    /// [`Self::Pinch`] e [`Self::Magnify`] são exatamente o mesmo kernel com um
    /// sinal, e são dois chips. Um `pinch` que alcança negativo seria a segunda
    /// resposta a *"como o artista pede o oposto?"*.
    ///
    /// ⚠️ **A DIREÇÃO do depósito é NOSSA, e a §4 é o motivo.** O nosso
    /// [`Self::Crease`] cava por default porque herda o `_negative = true` do
    /// `Crease.js`; o SculptGL **não tem** Blob, então não há `_negative` a
    /// herdar — e inventar um com a autoridade de uma referência que não o
    /// declara é precisamente o que a §4 proíbe. ⇒ a direção é escolha nossa, e
    /// ela é a que o NOME diz: um *blob* é um monte, então ele SOBE. O `Ctrl`
    /// dá o oposto de cada verbo, como em toda a família.
    ///
    /// ⚠️ **O SculptGL NÃO O TEM** — ver [`crate::RefMode`]: o `S` fica
    /// silencioso aqui, como já fica no [`Self::Sharpen`] e no
    /// [`Self::ClayStrips`].
    Blob,
    /// Não move geometria: escreve `mask[v]`, que **todos** os outros respeitam.
    Mask,
    /// **Pega o barro e o traz junto** — ver [`Grip::Hold`].
    Move,
    /// **Estica o barro num espinho**: a pegada ANDA com o cursor e cada dab
    /// entrega ao seguinte — ver [`Grip::Hook`].
    SnakeHook,
    /// **TORCE** o barro em torno do eixo da vista — o redemoinho. Ver
    /// [`Grip::Turn`].
    Twist,
    /// **INFLA ou ENCOLHE** o barro em torno da âncora, radialmente. Ver
    /// [`Grip::Turn`].
    ///
    /// ⚠️ **Não é o [`Self::Inflate`], e a diferença é o CENTRO.** O Inflate
    /// empurra cada vértice pela PRÓPRIA normal (a forma engorda em relação à
    /// superfície dela); este empurra cada vértice para longe de UM ponto — o
    /// que cresce é a região inteira, como um balão que se enche a partir do
    /// lugar onde a mão está.
    LocalScale,
    /// **A FAIXA DE BARRO** — o Clay com a pegada deitada na direção do traço.
    ///
    /// ⚠️ **A lei é a do [`Self::Draw`]; o que muda é a SILHUETA** (ver
    /// [`crate::Footprint`]): miolo chato numa caixa arredondada em vez de um
    /// domo, e um portão parabólico na profundidade que faz a passada
    /// DEPOSITAR barro abaixo do plano em vez de levantar o que já está no
    /// lugar. É a ferramenta de blocagem da referência (o *Clay Strips*), e a que
    /// mais muda o que se consegue fazer numa sessão.
    ///
    /// ⚠️ **O SculptGL NÃO A TEM** — ver [`crate::RefMode`]: a metade
    /// declarativa do `S` fica silenciosa aqui, como já fica no
    /// [`Self::Sharpen`].
    ClayStrips,
    /// **O POLEGAR** — o plano se INCLINA ao longo do traço, e o ângulo CRESCE
    /// enquanto a mão anda.
    ///
    /// ⚠️ **A lei é a do [`Self::Flatten`]; o que muda é QUAL plano** — o
    /// *Clay Thumb* da referência projeta cada vértice num plano *bilateral*,
    /// exatamente como o Flatten, e a ferramenta inteira mora na construção do
    /// plano:
    ///
    /// 1. ele passa pelo **centro do dab**, não pelo centro de área — a
    ///    diferença com os quatro verbos de plano que a
    ///    [`crate::stroke_plane`] serve;
    /// 2. a normal dele é a normal de área **girada** em torno do eixo que
    ///    ATRAVESSA o traço (o produto vectorial da normal com o caminho — o
    ///    mesmo eixo que o *Pinch* monta);
    /// 3. o ângulo dessa rotação **ACUMULA** ao longo do traço
    ///    (`+`[`crate::CLAY_THUMB_TILT_STEP_DEG`]` por dab, teto
    ///    [`crate::CLAY_THUMB_TILT_MAX_DEG`]) — e o efeito que a referência diz
    ///    procurar com isso é **simular o barro a acumular**: quanto mais
    ///    amostras o traço junta, mais inclinado o plano.
    ///
    /// ⚠️ **É o PRIMEIRO verbo cujo alvo depende de quantos dabs já passaram**,
    /// e não só de onde este caiu. O estado mora no [`crate::SculptStroke`], ao
    /// lado do `last_center` de que ele é irmão: os dois são fatos sobre o
    /// GESTO, e nenhum deles cabe num [`crate::Dab`].
    ///
    /// ⚠️ **Sem direção ele não deposita**, como na referência (sem deslocamento
    /// do dab a referência não faz nada): um plano inclinado precisa de um eixo,
    /// e o eixo é o traço. O primeiro dab de todo traço cai nesse caso por
    /// construção — o `path` dele é `[0, 0, 0]` —, que é a mesma recusa que a
    /// referência escreve com uma desistência própria para adiar o primeiro dab.
    ///
    /// ⚠️ **O SculptGL NÃO O TEM** — ver [`crate::RefMode`], como os dois
    /// vizinhos acima.
    ClayThumb,
    /// **A LÂMINA EM V** — o *Multiplane Scrape* da referência. O único verbo com **DOIS**
    /// planos, e é isso que o nome diz: em vez de raspar contra uma superfície,
    /// ele raspa contra um TELHADO, e o que sobra é um sulco de duas facetas
    /// planas com uma aresta viva no meio.
    ///
    /// ⚠️ **A dobradiça é o TRAÇO.** Os dois planos partilham a origem (o centro
    /// do dab, como no [`Self::ClayThumb`]) e as normais deles são a normal de
    /// área girada de `±ângulo/2` em torno do eixo que corre **AO LONGO** do
    /// caminho — a rotação ORTOGONAL à do polegar, que gira em torno do eixo que
    /// o atravessa. Os dois verbos inclinam o mesmo plano; o que os separa é
    /// **em torno de quê**.
    ///
    /// ⚠️ **Qual dos dois um vértice consome é decidido pelo LADO em que ele
    /// caiu** (o SINAL da coordenada dele no referencial do dab), e cada
    /// meio-plano se inclina **para o lado que ele serve** — é isso que abre o V
    /// em vez de o fechar. Num ângulo negativo (a aresta CÔNCAVA) as normais
    /// tombam ao contrário, o telhado vira vale, e a ferramenta **enche** a
    /// dobra em vez de a cavar.
    ///
    /// ⚠️ **E o culling de lado é gateado no SINAL do ângulo** — ele só corre
    /// com o ângulo não-negativo: com o V aberto só o que está ACIMA do próprio
    /// meio-plano é tocado — o que torna o verbo auto-limitado, como o
    /// [`Self::Scrape`] —, e com ele fechado a projeção é bilateral e a dobra é
    /// preenchida dos dois lados.
    ///
    /// ⚠️ **A ponta é deformada ao longo do traço** para não deixar degraus em
    /// traços curvos, e por isso **não é um disco**. É a
    /// [`crate::Footprint::Blade`], e sem ela um traço curvo deixa degraus onde
    /// dois dabs vizinhos raspam com dobradiças que já não são paralelas.
    ///
    /// ⚠️ **Sem direção não há dobradiça, logo não há depósito** — a mesma
    /// recusa do [`Self::ClayThumb`], pela MESMA porta ([`crate::stroke_axis`]),
    /// e a referência a escreve com as mesmas duas desistências: **adiar o
    /// primeiro dab** do traço, e desistir quando o deslocamento do cursor é zero.
    ///
    /// ⚠️ **O SculptGL NÃO O TEM** — ver [`crate::RefMode`].
    MultiplaneScrape,
    /// **O ÚNICO VERBO QUE NÃO MUDA A FORMA** — ele redistribui os vértices
    /// SOBRE a superfície (é o *Slide Relax* da referência).
    ///
    /// Todos os outros vinte respondem *para onde este vértice vai*; este
    /// responde *este vértice está no lugar errado DA MALHA*. Um traço que
    /// esticou um trecho deixa triângulos compridos e finos, e nenhum dos
    /// dezanove verbos de geometria os conserta — o Smooth alisa a FORMA e leva
    /// a estrutura junto.
    ///
    /// **A lei, em duas linhas:** caminhe para a média do anel, e depois
    /// **remova a componente ao longo da normal** — ou seja, projecte o
    /// deslocamento no plano tangente, que é o que a referência faz. O que
    /// sobra é tangencial ⇒ o vértice desliza pela superfície e a silhueta fica
    /// onde estava. É a única linha que separa este verbo do [`Self::Smooth`],
    /// que é a mesma média SEM a subtração.
    ///
    /// ⚠️ **A normal é a VIVA, e o contraste com o [`Self::Inflate`] é o ponto:**
    /// lá ela é congelada porque é uma **DIREÇÃO PARA ANDAR**, e um traço parado
    /// arrastaria o empurrão consigo (medido: 53,4° em 64 dabs). Aqui ela é o
    /// **PLANO EM QUE FICAR**, e o verbo nunca anda ao longo dela — congelá-la
    /// prenderia o vértice ao plano tangente do pen-down e ele sairia da
    /// superfície que o traço acabou de mover. *A mesma grandeza, dois papéis
    /// opostos.*
    ///
    /// ⚠️ **Numa BORDA a normal é outra, e este ramo é o caso NORMAL e não a
    /// exceção:** a referência troca a normal do vértice pela **bissetriz** das
    /// arestas de borda quando um vértice de beira ficou com exactamente dois
    /// vizinhos de borda — e numa malha manifold
    /// a curva de borda é um LOOP FECHADO, logo **todo** vértice dela tem
    /// exactamente dois vizinhos de borda (medido: 12 de 12 no `open_tube3`, o
    /// número que o [`ph2d_mesh::ring_average`] já regista). Sem a bissetriz o
    /// vértice desliza no plano tangente da SUPERFÍCIE, que contém a direção da
    /// corda entre os dois vizinhos — e a beira encolhe para dentro dela a cada
    /// dab.
    ///
    /// ⚠️ **APROXIMAÇÃO NOMEADA, não escondida:** a referência tem um terceiro
    /// filtro que impede um vértice de atravessar a fronteira de um *face set*.
    /// **Nós não temos face sets**
    /// (decisão do Enio, doc 21 §5.2), então esse filtro não tem análogo aqui e
    /// o verbo relaxa através de uma fronteira que o Blender respeitaria.
    ///
    /// ⚠️ **Sem inverso**, e não é omissão: o oposto de *distribuir* não é
    /// *concentrar* — é o estado anterior, que o Ctrl não sabe reconstruir. A
    /// referência também não o oferece.
    ///
    /// ⚠️ **O SculptGL NÃO O TEM** — a quinta vez a mesma frase (ver
    /// [`crate::RefMode`]).
    SlideRelax,
    /// **O ALISAMENTO QUE DEVOLVE O QUE TIROU** — é o alisamento da referência
    /// no modo de deformação de SUPERFÍCIE, que é o **HC** de
    /// Vollmer, Mencl & Müller (EG 1999, *Improved Laplacian Smoothing of Noisy
    /// Surface Meshes*).
    ///
    /// **O defeito que ele existe para curar está MEDIDO neste repo** — o
    /// [`Self::Smooth`] é um laplaciano *umbrella*, e ele **contrai o volume a
    /// cada aplicação, para sempre**: 0,09 % numa passada, **3,58 % em quarenta**
    /// (a tabela vive no [`crate::brush_pass`]). *Alisar até ficar liso é alisar
    /// até sumir*, que é a objeção com que o paper abre.
    ///
    /// **A lei, em duas linhas:**
    ///
    /// ```text
    /// b_i = média(q)_i − [α·o_i + (1−α)·q_i]        // o que o passo laplaciano TIROU
    /// p_i = q_i + w·(média(q)_i − q_i) − w·[(1−β)·média(b)_i + β·b_i]
    /// ```
    ///
    /// — caminhe para a média do anel (o Smooth), e depois **devolva o
    /// deslocamento que isso custou**, suavizado sobre a vizinhança.
    ///
    /// ⚠️ **Ele NÃO é o `l-mode` do [`Self::Smooth`], e a distinção está escrita
    /// no [`crate::brush_pass`] desde a wave do Taubin:** o `o` do HC é a pose
    /// do **pen-down**, então com `α > 0` ele PUXA de volta para ela — o oposto
    /// de *"passar de novo alisa mais"*, que é o que uma pincelada faz. Como
    /// **ferramenta própria** isso deixa de ser um defeito e passa a ser a
    /// feature: o knob chama-se *Shape Preservation* porque é exactamente o que
    /// ele preserva.
    ///
    /// ⚠️ **O `b` obriga a um BUFFER, e é a única parte estrutural do verbo:**
    /// `média(b)` é um operador de SEGUNDA ordem — ele precisa do `b` dos
    /// VIZINHOS, e nenhum deles é derivável da posição depois de o passo
    /// laplaciano ter corrido. Ver [`crate::stroke_hc`].
    ///
    /// ⚠️ **`b-mode ≡ l-mode`, então ele tem UM chip e não um dropdown** — o
    /// Blender **é** o port do paper aqui, e um segundo chip que declarasse a
    /// mesma lei seria o controle morto que esta casa varre a cada wave. É a
    /// mesma coincidência que o plano §5.1 prevê para o Elastic Deform, e ela é
    /// o teste de sanidade da matriz do §3.
    ///
    /// ⚠️ **O SculptGL NÃO O TEM** — a sexta vez a mesma frase (ver
    /// [`crate::RefMode`]).
    SurfaceSmooth,
    /// **A DEMÃO** — uma camada de espessura ESCOLHIDA, saturante e apagável
    /// (é o *Layer* da referência).
    ///
    /// **A lei, em três linhas:**
    ///
    /// ```text
    /// d ← clamp(d + w·força·(1,05 − |d|),  0, 1−máscara)   // satura
    /// alvo = pre + normal_pre · sinal · altura              // a demão CHEIA
    /// pos  = lerp(pre, alvo, d)                             // o aplicador de sempre
    /// ```
    ///
    /// ⚠️ **A propriedade que o define é o PLATÔ, e ela está MEDIDA:** *todo*
    /// peso da pegada converge para `d = 1,0000` — o falloff é uma **TAXA**
    /// (quão depressa cada vértice lá chega), nunca um perfil. Medido em
    /// `measure_layer_law`, dabs até 99 % do teto: peso `1,00` → **1** dab ·
    /// `0,50` → 5 · `0,25` → 10 · `0,10` → 28 · `0,02` → 142. É isso que faz de
    /// uma demão uma demão em vez de um [`Self::Draw`] com teto.
    ///
    /// ⚠️ **E é isso que o separa do [`Self::Draw`], depois de a cerca de
    /// Chesterton ter caído:** o doc do plano 21 §5.1 registrava que os dois
    /// *"colapsavam"* sob a lei do envelope, e a wave do accumulate (2026-08-11)
    /// **matou o envelope** — hoje o Draw com Accumulate empilha sem teto, e sem
    /// ele auto-limita-se por GEOMETRIA (*o vértice andou mais que o raio*), que
    /// é uma grandeza do PINCEL. O teto da demão é uma **ALTURA**, um número que
    /// não se move quando o artista muda o raio.
    ///
    /// ⚠️ **O `accum` É a fração da demão, e é isso que dispensa um plano
    /// novo.** O aplicador já anda `lerp(pre, alvo, accum)`; pondo o alvo na
    /// altura CHEIA, o `accum` que o motor já guarda passa a ser exactamente o
    /// factor de deslocamento da referência. O plano por-vértice que o plano 21
    /// prometia — e a lei do repo que ele arrastava (*ao adicionar um plano,
    /// adicione-o ao snapshot de undo no MESMO commit*) — **não existe**: medido,
    /// o deslocamento acumulado por vértice do *Layer* vive no **cache do traço**
    /// da referência, que ela constrói no pen-down e **liberta no pen-up**, logo
    /// ele é estado de TRAÇO, irmão do nosso `pre` congelado e não da máscara.
    ///
    /// ⚠️ **DIVERGÊNCIA DECLARADA — o segundo `f` do Blender não é portado, e a
    /// medição diz que portá-lo QUEBRARIA o verbo.** Lá a escrita é `pos +=
    /// (alvo − pos)·f` sobre a posição VIVA; o nosso aplicador anda do `pre`
    /// CONGELADO, então o mesmo `·f` seria re-aplicado do base a cada dab e o
    /// platô convergido passaria a valer `f · altura` — o falloff a vazar para
    /// dentro da única propriedade que o verbo entrega (medido: `1,000000`
    /// contra `0,500000` no peso 0,5). As duas recorrências **pousam no mesmo
    /// lugar**; o que difere é o transiente (pior separação `0,26–0,37` da
    /// altura, fechada em 8–55 dabs conforme o peso).
    ///
    /// ⚠️ **E o `f` da referência ali é a atenuação GENÉRICA de borda**, a mesma
    /// que todo pincel do Blender multiplica no deslocamento — não um
    /// amortecimento próprio da demão. No nosso motor essa atenuação **é** o
    /// `accum`, e aplicá-la duas vezes é o perfil-em-dobro que o
    /// [`crate::Grip::Hold`] já documenta ter pago uma vez.
    ///
    /// ⚠️ **O SculptGL NÃO O TEM** — a sétima vez a mesma frase (ver
    /// [`crate::RefMode`]).
    Layer,
    /// **O TECIDO** — a superfície sob o pincel passa a ser pano: a região
    /// escolhida no pen-down simula, o anel de fora fica pregado, e a mão entra
    /// como força. Ver [`Grip::Simulate`] e o `stroke_cloth`.
    Cloth,
    /// **O POLEGAR** — espalma o barro na direcção do gesto **sem o levantar**:
    /// o deslocamento é a componente do puxão no PLANO TANGENTE, e mais nada.
    ///
    /// ⚠️ **É o [`Self::Move`] menos a componente normal, e a subtracção É a
    /// ferramenta:** o agarrar leva o gesto inteiro (e pode inclinar-se para a
    /// normal por um peso próprio); este leva só o que corre paralelo à
    /// superfície. É por isso que ele *espalma* em vez de *levantar* — um
    /// polegar a alisar barro, que é o que o nome público diz.
    ///
    /// ⚠️ **O grip é o [`Grip::Hold`] que já existe**, e com ele vem a
    /// propriedade que define este gesto: o alvo é função do `pre` CONGELADO e
    /// do puxão TOTAL ⇒ **só o último evento conta**. Medido no oráculo, o
    /// mesmo caminho em 12 e em 24 eventos dá o MESMO deslocamento máximo
    /// (`0,600000` nos dois), enquanto o irmão que viaja dá `0,599054` contra
    /// `0,595226`.
    ///
    /// ⚠️⚠️ **A força entra ao QUADRADO, e isto não é herança — é medição:**
    /// com o slider a `0,5` o pico cai para **exactamente `1/4`**. Na nossa
    /// cadeia isso sai de graça e por construção: o alvo leva UM
    /// [`crate::Brush::weight`] e o aplicador multiplica pelo `accum`, que já
    /// carrega o outro. ⛔ **Não generalize:** o agarrar e o gancho são
    /// LINEARES no mesmo slider (`docs/3D/cleanroom/SPEC_pull_brushes.md` §3),
    /// e quem levar o quadrado para lá erra por `2×`.
    ///
    /// ⚠️ **A normal que ele lê NÃO é a do plano do carimbo** — é a
    /// [`crate::stroke_normal_do_gesto`], amostrada num raio próprio e com os
    /// dois lados da silhueta separados.
    ///
    /// ⚠️ **O `Ctrl` não o inverte**: quem dá o sentido é a direcção do gesto.
    /// Ver [`Self::honours_invert`].
    Thumb,
    /// **O EMPURRÃO** — a MESMA conta do [`Self::Thumb`], com a pegada a
    /// VIAJAR: varre matéria ao longo do traço em vez de espalmar um sítio.
    ///
    /// ⭐⭐ **A fórmula é a mesma, letra por letra** (a parte tangencial do
    /// gesto vezes a força efectiva) — o que muda são **três** coisas, e as
    /// três já são vocabulário desta casa: o puxão é o INCREMENTO e não o
    /// total, a pegada anda com o cursor, e o peso mede-se na pose VIVA. Isso é
    /// exactamente o [`Grip::Hook`] ⇒ **zero modelo novo**.
    ///
    /// ⚠️ **Voltar pelo mesmo caminho NÃO devolve o barro** (medido no oráculo:
    /// o pico fica em `0,259160` depois da ida e da volta, não em zero) — é uma
    /// integral de linha, que é o que o doc do [`Grip::Hook`] já promete.
    ///
    /// ⚠️ **A paridade está fechada no PLANO e ABERTA na superfície curva**
    /// (`~10 %`), e a causa **não é a lei**: quatro variantes da normal foram
    /// medidas e dão o mesmo resíduo. O que falta é o centro que o oráculo de
    /// facto usou em cada evento — ele reamostra a superfície VIVA, que se
    /// deforma debaixo do traço. Ver a espec §6.3.
    Nudge,
    /// **A POSE** — o pincel que acha sozinho uma **articulação enterrada** na
    /// forma e dobra a região à volta dela, como um braço. Sem esqueleto.
    ///
    /// ⚠️⚠️ **É o verbo que MENOS se parece com os outros 26, e a diferença não
    /// é de grau:** todos eles são um núcleo por-vértice atenuado pela distância
    /// ao cursor, e este **não tem atenuação radial nenhuma**. Um vértice a dez
    /// raios de distância pode mover-se por inteiro, porque a região é escolhida
    /// pela **ligação** da malha e não pela vizinhança no espaço — a cadeia
    /// cresce por arestas, pára no raio, e o primeiro anel para lá dele dá o
    /// **pivô**. ⇒ ele **desvia antes do `dab_core`**, como o [`Self::Cloth`],
    /// e por uma razão própria: a lei dele já resolve os **oito octantes de
    /// espelho numa passagem só** (a simetria vive nos mapas afins), logo passar
    /// pela expansão de espelho genérica aplicá-la-ia duas vezes.
    ///
    /// A lei inteira vive na crate-folha [`ph2d_pose`], medida contra `69`
    /// traços do oráculo; a espec é `docs/3D/cleanroom/SPEC_pose_brush.md`.
    Pose,
    /// ⭐⭐⭐ **CONTORNO** — a borda ABERTA da malha deforma-se, e a deformação
    /// esmorece **para dentro** da peça.
    ///
    /// ⚠️⚠️ **Ele é o segundo verbo sem atenuação radial, e por outra razão que
    /// a pose:** aqui a região não cresce a partir do cursor — ela cresce a
    /// partir da **BORDA**. O cursor só escolhe *que troço de borda*; a partir
    /// dela a onda entra pela malha em anéis, e o `deslocamento_da_origem`
    /// alonga-a ainda mais. ⇒ *a região que este pincel toca não está contida na
    /// esfera do raio*, e ele desvia antes do `dab_core` como o [`Self::Cloth`]
    /// e a [`Self::Pose`].
    ///
    /// ⛔ **Numa malha fechada ele não faz nada** — não há borda —, e há **duas**
    /// recusas geométricas em que ele recusa o traço inteiro (a quina de uma
    /// grelha, e uma tira de uma fileira onde todo vizinho é de borda).
    ///
    /// A lei inteira vive na crate-folha [`ph2d_boundary`], medida contra `61`
    /// traços do oráculo; a espec é `docs/3D/cleanroom/SPEC_boundary_brush.md`.
    Boundary,
    /// ⭐⭐ **A DENSIDADE — o único verbo que NÃO MOVE UM VÉRTICE.**
    ///
    /// Ele não tem lei por-vértice nenhuma: **todo** o efeito dele é sobre o
    /// passe de TOPOLOGIA. Onde o pincel passa, a malha **afina** — as arestas
    /// curtas colapsam e a densidade desce —, e a forma fica.
    ///
    /// ⛔ **Ele LIGA o colapso e NÃO liga o partir**, e são duas leis, não uma:
    /// um teste que só afirmasse a primeira passaria com um pincel que também
    /// subdivide, que é **outro produto**. Ele nunca ACRESCENTA superfície.
    ///
    /// ⚠️ **Ele desvia antes do `dab_core`** ([`Self::sem_lei_por_vertice`]):
    /// um verbo novo que caísse lá herdaria a cadeia de peso — a dureza, a
    /// curva de queda, a força — que aqui **não existe**. Medido na espec com o
    /// passe desarmado: `0` vértices movidos de `1 681`, e a diferença máxima de
    /// posição é `0` exactamente, não «pequeno».
    ///
    /// ⭐⭐⭐ **ELE CORRE SEM O INTERRUPTOR DA TOPOLOGIA DINÂMICA** — ordem do
    /// dono (2026-09-14): *«independente se Dynamic topology está ligado ou não,
    /// Density faz o seu trabalho. Dynamic topology é para os outros
    /// pincéis.»* ⚠️ **Divergência DECLARADA** da referência, que desarma o
    /// passe inteiro no modo *Manual* (espec §3.2, 1.ª linha): aquele
    /// interruptor pergunta *«o meu traço também muda a topologia?»*, e este
    /// verbo **não tem traço**. Ver [`Self::corre_sem_o_interruptor`].
    ///
    /// ⛔ **DECISÃO DE PRODUTO por decidir, e ela é observável:** o nosso
    /// colapso recusa mexer numa aresta em que *algum dos quatro vértices está
    /// na beira* — mais duro que o alvo, que em vez de recusar **escolhe o
    /// sobrevivente**. O que shipa é a opção conservadora (espec §3.8), e o que
    /// o artista vê é *«o pincel não afina a borda»*. A outra saída herda o
    /// cuidado de mover-ou-não o sobrevivente, e custa re-medir o ponto fixo do
    /// nosso par partir/colapsar.
    ///
    /// A espec é `docs/3D/cleanroom/SPEC_unblocked_brushes.md` §3.
    Density,
    /// ⭐⭐ **O APAGADOR DE DESLOCAMENTO — ele repõe a pele na SUPERFÍCIE-LIMITE.**
    ///
    /// Com uma pilha de multiresolução montada, cada vértice do nível de cima é
    /// *a superfície de base mais um deslocamento*. Este pincel **desfaz esse
    /// deslocamento** onde passa: a escultura fina esmorece e a forma grande
    /// fica exactamente onde estava.
    ///
    /// ⛔⛔ **A referência é o LIMITE e não a PREVISÃO, e essa é a wave inteira**
    /// ([`ph2d_mesh::limit_point`], espec §2.1): `subdivide^k(base)` **não é** a
    /// superfície-limite, e a diferença não tende a zero na densidade que um
    /// artista usa. Medido no canto de um cubo de lado `1`: a previsão de um
    /// passo pousa em `0,2778` e o limite em **`0,2500`** — `11 %` de resíduo,
    /// que um apagador ancorado na previsão deixaria **a cada passagem**. ⇒ ele
    /// *encolheria a peça*, e o artista leria isso como *«o apagador comeu a
    /// forma»*.
    ///
    /// ⚠️ **A lei é uma interpolação LINEAR pura em direcção à referência** —
    /// `p ← p + f·(R − p)` —, sem direcção privilegiada, sem normal e sem
    /// acumulador. ⭐ A prova de que não é *«mover ao longo da normal até à
    /// referência»* é a componente perpendicular a `R − p`, medida no oráculo
    /// em **`3,48e-08`**: ruído de `f32`.
    ///
    /// ⚠️ **O tecto `min(f, 1)` significa que ele NUNCA ultrapassa a
    /// referência** — não há *«apagar demais»*.
    ///
    /// ⛔ **Inverter não faz nada**, e isso não é uma feature em falta: apagar
    /// deslocamento tem um só sentido (o deslocamento zero), e *«apagar ao
    /// contrário»* não nomeia nada. Ver [`Self::honours_invert`].
    ///
    /// ⛔ **Sem pilha de multiresolução ele RECUSA em voz alta** (espec §4.3): ali
    /// não existe o dado de entrada. *O irmão-filtro do alvo estoirou
    /// publicamente por não verificar isto* — e é por isso que o gate desta
    /// fronteira é a **recusa**, não o resultado.
    ///
    /// A espec é `docs/3D/cleanroom/SPEC_unblocked_brushes.md` §4.
    EraseMultires,
    /// ⭐⭐ **O ESFREGÃO DE DESLOCAMENTO — ele arrasta a PELE sobre a forma.**
    ///
    /// Com uma pilha de multiresolução montada, cada vértice do nível de cima é
    /// *a superfície de base mais um deslocamento*. Este pincel **não move o
    /// vértice para onde a mão vai**: ele move o **campo de deslocamento** sobre
    /// a superfície de referência, como quem arrasta uma textura sobre uma forma
    /// fixa. A forma grande fica exactamente onde estava; o que viaja é o
    /// relevo.
    ///
    /// ⚠️ **A média que o faz viajar pesa a vizinhança pela parte NEGATIVA do
    /// cosseno** (espec §5.2): só os vizinhos **a montante** de `d̂` contribuem,
    /// e o vértice entra na própria média com peso **`1` fixo**. Se o peso
    /// próprio fosse normalizado com os outros, a vizinhança dominaria a média
    /// por mais vizinhos que houvesse a montante, e o pincel borrava em vez de
    /// transportar.
    ///
    /// ⛔⛔ **A vizinhança é medida na SUPERFÍCIE DE REFERÊNCIA, nunca nas
    /// posições deslocadas** — é isso que faz esfregar repetidamente não
    /// deformar a topologia. A referência é o LIMITE e não a PREVISÃO, pela
    /// mesma medição que o [`Self::EraseMultires`] carrega
    /// ([`ph2d_mesh::limit_point`]): no canto de um cubo de lado `1` a previsão
    /// de um passo pousa em `0,2778` e o limite em `0,2500`.
    ///
    /// ⚠️ **Três direcções e UMA lei** ([`crate::SmearMode`]): o arrasto segue a
    /// mão, o aperto aponta ao centro do dab, o espalhar aponta para fora.
    /// ⛔ **O arrasto com o cursor PARADO é inerte por construção** — `d̂` é
    /// nulo, nenhum vizinho passa o teste do cosseno, e a média colapsa no
    /// próprio vértice. Os outros dois **não** têm essa degenerescência.
    ///
    /// ⚠️ **A orla COME deslocamento, e é um artefacto que a referência
    /// TOLERA** (espec §5.4): o campo só é posto em dia nos nós tocados, e um
    /// vizinho de fora entra com o valor que tinha — zero, no primeiro dab. ⭐ O
    /// zero não é conveniência: é a cura **publicada** de uma regressão em que
    /// vizinhos sem valor definido propagavam `NaN` pela malha.
    ///
    /// ⚠️ **A conservação alegada pelos autores é boa a menos de `1 %` e NÃO é
    /// exacta** — medida na espec §5.5, a deriva do deslocamento total vai de
    /// `−0,62 %` a `+0,05 %` e **cresce com o comprimento do traço**. Um gate
    /// escrito como igualdade reprovaria o próprio alvo.
    ///
    /// ⛔ **Inverter não faz nada**, pela razão do apagador: o factor de força
    /// deste pincel não tem sinal. Ver [`Self::honours_invert`].
    ///
    /// ⛔ **Sem pilha de multiresolução ele RECUSA em voz alta** (espec §5.6):
    /// ali não existe o dado de entrada.
    ///
    /// A espec é `docs/3D/cleanroom/SPEC_unblocked_brushes.md` §5.
    SmearMultires,
    /// ⭐⭐⭐ **PROJECTAR NA CENA** — o barro é empurrado até **encostar noutra
    /// peça**, como quem molda uma coisa contra a outra.
    ///
    /// Para cada vértice ao alcance lança-se um raio na direcção do dab contra
    /// **as outras peças da cena**, e o vértice viaja a distância que o raio
    /// mediu. ⭐ **Uma direcção para o dab INTEIRO**, nunca uma por vértice — a
    /// versão por normal do vértice foi construída e rejeitada pelo autor do
    /// alvo como inutilizável (espec §9.2: *recusa medida por terceiros*).
    ///
    /// ⛔⛔ **Nenhum acerto ⇒ o vértice NÃO SE MEXE.** Não há «projectar para o
    /// infinito», e é por isso que sem outra peça na cena o pincel é inerte —
    /// a fixtura `projectar_sem_alvo` mede **zero** vértices movidos.
    ///
    /// ⚠️ **Não há acumulador nem memória entre dabs** (espec §6.5): cada dab
    /// re-mede a distância a partir de onde o vértice está **agora**. ⭐ É daí
    /// que sai a assimetria da §6.6 — *a lei é antissimétrica e o processo não
    /// é*: na direcção do alvo a distância encolhe e o vértice **pára**; ao
    /// contrário, ela cresce. ⛔ Um gate que exija espelho exacto sobre um TRAÇO
    /// reprova o próprio alvo (medido: `max |d + d′| = 1,415e-01`).
    ///
    /// A espec é `docs/3D/cleanroom/SPEC_unblocked_brushes.md` §6.
    SceneProject,
    /// ⭐⭐⭐ **BOX TRIM — o corte por uma forma desenhada** (ordem do dono,
    /// 2026-09-15: *«Crie o botão nos tools para box trim»*).
    ///
    /// ⚠️⚠️ **Ele NÃO tem lei por-vértice, e não é «mais um pincel sem
    /// aplicador»:** os outros verbos desta casa carimbam barro, e este
    /// INTERCEPTA o arrasto inteiro — o que o gesto entrega é uma forma de ecrã,
    /// e o que muda a peça é uma booleana sobre a malha toda, no LARGAR.
    ///
    /// ⭐ **Porque é um VERBO e não um botão ao lado da fileira:** um verbo é
    /// **exclusivo** com os outros por construção — escolher um pincel desarma o
    /// corte e escolher o corte larga o pincel, sem uma única regra escrita à
    /// mão. Um botão separado precisaria dessas duas regras, e elas são
    /// exactamente o tipo de coisa que apodrece (o módulo já pagou isso com o
    /// `aim` e a linha da Hierarquia a disputarem a peça activa).
    ///
    /// ⚠️ **Ele herda, do [`Self::Density`], o caminho dos verbos sem lei
    /// por-vértice** — os censos filtram por [`Self::writes_through_applicator`],
    /// e o dab nunca corre para ele.
    BoxTrim,
    /// ⭐⭐⭐ **O PINCEL DE PLANO** — aparar, encher e achatar **esfregando**, com
    /// um pincel só e **dois tectos**.
    ///
    /// Report do dono (2026-09-15): *«o Trim Brush do próprio blender que faz o
    /// trim esfregando o pincel como massinha, modelando»*. A espec é
    /// `docs/3D/cleanroom/SPEC_pincel_de_plano.md`; a LEI e as medições vivem no
    /// [`crate::Footprint::Tectos`] e no estimador dele, que é onde também está
    /// escrito porque ele **não** é um quarto irmão dos verbos de plano da casa.
    ///
    /// ⚠️ **A pegada NÃO é uma esfera:** os dois tectos achatam-na num
    /// **elipsóide**, e isso muda a POPULAÇÃO tocada e não só o peso —
    /// `altura 1 / profundidade 0` move `151` vértices, `0 / 1` move `114`, e
    /// `1 / 1` move `265 = 151 + 114`.
    ///
    /// ⚠️ **Ele exige ESFREGAR** ([`Self::exige_esfregar`]): o primeiro dab de
    /// cada passagem não move nada. ⭐ E a orientação do quadro **dentro** do
    /// plano não alcança a saída (`≤ 8,0e-08` em 18 de 19 configurações usando só
    /// a normal, o centro e o raio) ⇒ simplificação adoptada **sem divergir**.
    /// ⛔ Ela é deste pincel e **não** dos irmãos, que constroem a silhueta A
    /// PARTIR daquele quadro.
    Plane,
    /// ⭐⭐⭐ **O PINCEL AFIADO** — o vinco duro, no lugar do domo do
    /// [`Self::Draw`] (ordem do dono, 2026-09-16). Espec:
    /// `docs/3D/cleanroom/SPEC_pincel_afiado.md`.
    ///
    /// ⭐⭐ **Ele é o [`Self::Draw`] com CINCO valores trocados e NENHUMA lei
    /// nova** (espec §0.1): a curva afiada, a direcção a afundar
    /// ([`Self::afunda_de_fabrica`]), a distância medida sempre do pen-down, o
    /// passo de `5 %` do diâmetro e a atenuação `a` por dab.
    ///
    /// ⚠️⚠️ **A diferença de LEI é UMA, e é a DISTÂNCIA** (espec §2.1): a curva
    /// pesa cada vértice pela posição que ele tinha no **pen-down**, e não pela
    /// de agora — é isso que impede o vinco de ALARGAR enquanto aprofunda. A
    /// coluna é **pregada** em [`Self::grip_law`], nunca pendurada no
    /// interruptor de acumular, que por isso sai da tela.
    ///
    /// ⭐ **E é dela que sai a auto-limitação** (§4.1): o cursor desce com o
    /// vinco e afasta-se das posições do pen-down, logo a profundidade converge
    /// e nunca passa de um raio. Levantar a caneta **recomeça** o limite.
    ///
    /// ⚠️ **A normal da área é a do alvo** (§2.3, decisão D-3) — a mesma lei dos
    /// gestos tangenciais e do pincel de plano, ⛔ e **não** a do estimador de
    /// plano que o [`Self::Draw`] lê.
    DrawSharp,
}

/// **O CATÁLOGO na ordem em que a UI o lista** — ver [`catalogo`].
///
/// ⛔ Filho (`#[path]`) e não irmão, como os vizinhos deste módulo: o corte é de
/// ASSUNTO — aqui mora *que verbos EXISTEM e o que cada um significa*, e lá *em
/// que ORDEM o artista os encontra*. Foi o teto de LOC que o forçou, e ele é
/// melhor assim: a lista cresce uma linha por pincel novo, o significado cresce
/// um parágrafo.
#[path = "brush_verb_catalogo.rs"]
mod catalogo;
/// ⭐ **COMO O GESTO É CONDUZIDO** — o [`Grip`] de cada verbo. Ver [`grip_por_verbo`].
#[path = "brush_verb_grip.rs"]
mod grip_por_verbo;

/// ⭐ **QUAL CAMPO ELÁSTICO cada verbo consome** — ver [`campo`].
#[path = "brush_verb_campo.rs"]
mod campo;

/// ⭐⭐⭐ **QUEM MEXE NA TOPOLOGIA em Dynamic Topology** — ver [`dyntopo`].
#[path = "brush_verb_dyntopo.rs"]
mod dyntopo;

/// **OS DEFAULTS** — com que números um verbo nasce. Ver [`defaults`].
#[path = "brush_verb_defaults.rs"]
mod defaults;

/// **QUEM PODE SER UM FILTRO** — a lei e a faixa. Ver [`filter`].
#[path = "brush_verb_filter.rs"]
mod filter;
pub use filter::FilterKind;

/// **AS MAGNITUDES** — quanto cada família desloca. Ver [`magnitudes`].
/// **O nome que a interface mostra** — ver [`brush_verb_label`](self).
#[path = "brush_verb_label.rs"]
mod label;

#[path = "brush_magnitudes.rs"]
mod magnitudes;
pub use magnitudes::{
    BLENDER_REACH_FRACTION, CLAY_PLANE_FRACTION, CLAY_THUMB_TILT_MAX_DEG, CLAY_THUMB_TILT_STEP_DEG,
    CREASE_FRACTION, DEFAULT_MULTIPLANE_ANGLE_DEG, LAYER_HEIGHT_HARD_MAX, LAYER_HEIGHT_UI_MAX,
    MULTIPLANE_ANGLE_MAX_DEG, MULTIPLANE_ANGLE_SMOOTH, MULTIPLANE_TIP_STRETCH, PINCH_GAIN,
    REACH_FRACTION, STRIP_PLANE_FRACTION, TECTOS_CENTRO_MAX,
};
