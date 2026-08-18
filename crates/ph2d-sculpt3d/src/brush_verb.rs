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
/// plano novo a entrar no `ModelSnapshot` do undo no mesmo commit"*. Medido no
/// `layer.cc`, o `layer_displacement_factor` da referência mora no `ss.cache` —
/// construído no pen-down, **destruído no pen-up** —, logo é estado de TRAÇO e
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
    /// ⚠️ **A relação é a do Blender, ao pé da letra** — `crease.cc` tem UMA
    /// função (`do_crease_or_blob_brush`) e um `bool invert_strength` que troca
    /// o sinal do termo lateral e **mais nada**; o `offset` normal dos dois é o
    /// mesmo `sculpt_normal · raio · força`.
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
    /// lugar. É a ferramenta de blocagem do Blender (`clay_strips.cc`), e a que
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
    /// `clay_thumb.cc` projeta cada vértice num plano *bilateral*, exatamente
    /// como o Flatten, e a ferramenta inteira mora na construção do plano:
    ///
    /// 1. ele passa pelo **centro do dab** (`location_symm`), não pelo centro
    ///    de área — a diferença com os quatro verbos de plano que a
    ///    [`crate::stroke_plane`] serve;
    /// 2. a normal dele é a normal de área **girada** em torno do eixo que
    ///    ATRAVESSA o traço (`x = n × path`, o mesmo `X` que o `pinch.cc`
    ///    monta);
    /// 3. o ângulo dessa rotação **ACUMULA** ao longo do traço
    ///    (`+`[`crate::CLAY_THUMB_TILT_STEP_DEG`]` por dab, teto
    ///    [`crate::CLAY_THUMB_TILT_MAX_DEG`]) — *"simulate the clay accumulation
    ///    by increasing the plane angle as more samples are added to the
    ///    stroke"*, `clay_thumb.cc:170-176`.
    ///
    /// ⚠️ **É o PRIMEIRO verbo cujo alvo depende de quantos dabs já passaram**,
    /// e não só de onde este caiu. O estado mora no [`crate::SculptStroke`], ao
    /// lado do `last_center` de que ele é irmão: os dois são fatos sobre o
    /// GESTO, e nenhum deles cabe num [`crate::Dab`].
    ///
    /// ⚠️ **Sem direção ele não deposita**, e isso é a referência ao pé da
    /// letra (`if math::is_zero(grab_delta_symm) { return; }`): um plano
    /// inclinado precisa de um eixo, e o eixo é o traço. O primeiro dab de todo
    /// traço cai nesse caso por construção — o `path` dele é `[0, 0, 0]` —, que
    /// é a mesma recusa que o *"delay the first daub"* da referência escreve com
    /// um `return` próprio.
    ///
    /// ⚠️ **O SculptGL NÃO O TEM** — ver [`crate::RefMode`], como os dois
    /// vizinhos acima.
    ClayThumb,
    /// **A LÂMINA EM V** — o `multiplane_scrape.cc`. O único verbo com **DOIS**
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
    /// caiu** (`local_positions[i][0] <= 0`, `multiplane_scrape.cc:84`), e cada
    /// meio-plano se inclina **para o lado que ele serve** — é isso que abre o V
    /// em vez de o fechar. Num ângulo negativo (a aresta CÔNCAVA) as normais
    /// tombam ao contrário, o telhado vira vale, e a ferramenta **enche** a
    /// dobra em vez de a cavar.
    ///
    /// ⚠️ **E o culling de lado é gateado no SINAL do ângulo** (`if (angle >=
    /// 0.0f)`, `:405`): com o V aberto só o que está ACIMA do próprio meio-plano
    /// é tocado — o que torna o verbo auto-limitado, como o [`Self::Scrape`] —,
    /// e com ele fechado a projeção é bilateral e a dobra é preenchida dos dois
    /// lados.
    ///
    /// ⚠️ **A PONTA NÃO É UM DISCO**, e a referência diz por quê no comentário
    /// dela: *"deform the local space along the Y axis to avoid artifacts on
    /// curved strokes; this produces a not round brush tip"* (`:101-104`). É a
    /// [`crate::Footprint::Blade`], e sem ela um traço curvo deixa degraus onde
    /// dois dabs vizinhos raspam com dobradiças que já não são paralelas.
    ///
    /// ⚠️ **Sem direção não há dobradiça, logo não há depósito** — a mesma
    /// recusa do [`Self::ClayThumb`], pela MESMA porta ([`crate::stroke_axis`]),
    /// e a referência a escreve com os mesmos dois `return` (*"delay the first
    /// daub"* e `is_zero(grab_delta_symm)`).
    ///
    /// ⚠️ **O SculptGL NÃO O TEM** — ver [`crate::RefMode`].
    MultiplaneScrape,
    /// **O ÚNICO VERBO QUE NÃO MUDA A FORMA** — ele redistribui os vértices
    /// SOBRE a superfície (`SCULPT_TOOL_SLIDE_RELAX`, `relax.cc` +
    /// `sculpt_smooth.cc::calc_relaxed_translations_faces`).
    ///
    /// Todos os outros vinte respondem *para onde este vértice vai*; este
    /// responde *este vértice está no lugar errado DA MALHA*. Um traço que
    /// esticou um trecho deixa triângulos compridos e finos, e nenhum dos
    /// dezanove verbos de geometria os conserta — o Smooth alisa a FORMA e leva
    /// a estrutura junto.
    ///
    /// **A lei, em duas linhas:** caminhe para a média do anel, e depois
    /// **remova a componente ao longo da normal**
    /// (`translation_to_plane(pos, n, smoothed)`, `sculpt_smooth.cc:458`). O que
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
    /// vizinhos (`calc_boundary_normal_corner`, `:471`) — e numa malha manifold
    /// a curva de borda é um LOOP FECHADO, logo **todo** vértice dela tem
    /// exactamente dois vizinhos de borda (medido: 12 de 12 no `open_tube3`, o
    /// número que o [`ph2d_mesh::ring_average`] já regista). Sem a bissetriz o
    /// vértice desliza no plano tangente da SUPERFÍCIE, que contém a direção da
    /// corda entre os dois vizinhos — e a beira encolhe para dentro dela a cada
    /// dab.
    ///
    /// ⚠️ **APROXIMAÇÃO NOMEADA, não escondida:** a referência tem um terceiro
    /// filtro (`filter_boundary_face_sets`, `:553`) que impede um vértice de
    /// atravessar a fronteira de um *face set*. **Nós não temos face sets**
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
    /// **O ALISAMENTO QUE DEVOLVE O QUE TIROU** — o `SCULPT_TOOL_SMOOTH` com
    /// `SCULPT_SMOOTH_DEFORM_SURFACE` (`surface_smooth.cc`), que é o **HC** de
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
    /// (`layer.cc`, o `SCULPT_BRUSH_TYPE_LAYER` do Blender).
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
    /// `displacement_factor` da referência. O plano por-vértice que o plano 21
    /// prometia — e a lei do repo que ele arrastava (*ao adicionar um plano,
    /// adicione-o ao snapshot de undo no MESMO commit*) — **não existe**: medido
    /// no `layer.cc`, o `layer_displacement_factor` mora no `ss.cache`, que o
    /// Blender constrói no pen-down e **destrói no pen-up** (`MEM_delete`), logo
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
    /// que todo pincel do Blender multiplica no `calc_translations` — não um
    /// amortecimento próprio da demão. No nosso motor essa atenuação **é** o
    /// `accum`, e aplicá-la duas vezes é o perfil-em-dobro que o
    /// [`crate::Grip::Hold`] já documenta ter pago uma vez.
    ///
    /// ⚠️ **O SculptGL NÃO O TEM** — a sétima vez a mesma frase (ver
    /// [`crate::RefMode`]).
    Layer,
}

impl Verb {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 23] = [
        Self::Draw,
        Self::Inflate,
        Self::Smooth,
        Self::Sharpen,
        Self::Flatten,
        Self::Fill,
        Self::Scrape,
        Self::Clay,
        Self::Pinch,
        Self::Magnify,
        Self::Crease,
        Self::Blob,
        Self::Mask,
        Self::Move,
        Self::SnakeHook,
        Self::Twist,
        Self::LocalScale,
        Self::ClayStrips,
        Self::ClayThumb,
        Self::MultiplaneScrape,
        Self::SlideRelax,
        Self::SurfaceSmooth,
        Self::Layer,
    ];

    /// **Este verbo pode ACUMULAR?** — a porta única do `accumulate`.
    ///
    /// Só a família do CARIMBO. Os outros três grips carregam o gesto TOTAL
    /// desde o pen-down (o puxão, o ângulo varrido, a fração de escala) e
    /// carimbam `accum = 1` ou congelam a pegada: somar um total N vezes seria
    /// multiplicar o gesto pelo número de eventos de ponteiro, que é exatamente
    /// a dependência de taxa de amostragem que a lei do traço existe para não
    /// ter.
    ///
    /// ⚠️ Porta e não um `matches!` no sítio de uso: o painel pergunta para
    /// OFERECER o interruptor e o aplicador pergunta para HONRAR o clique, e
    /// duas cópias divergiriam num controle que aparece e não faz nada.
    ///
    /// ⚠️ **A DEMÃO fica de fora, e é a referência que a tira:** o `layer.cc`
    /// mede as distâncias contra `orig_data.positions` **incondicionalmente** —
    /// ele não consulta o `BRUSH_ACCUMULATE`, ao contrário dos irmãos de
    /// carimbo. E há razão para isso: o que o Accumulate compra num Draw é
    /// *deixar o pincel não se esgotar*, e a demão já tem o próprio motor de
    /// saturação no [`crate::GripLaw::coat`]. Oferecer o interruptor aqui seria
    /// um segundo controle sobre a mesma pergunta.
    #[must_use]
    pub fn accumulates(self) -> bool {
        matches!(self.grip(), Grip::Stamp) && self != Self::Layer
    }

    /// Este verbo escreve na MÁSCARA em vez da posição?
    ///
    /// Porta única: o aplicador pergunta para saber onde escrever, e a UI
    /// perguntará para saber que knobs oferecer. Duas listas divergiriam no dia
    /// em que entrar o segundo verbo de canal (Paint, na W7).
    #[must_use]
    pub fn paints_mask(self) -> bool {
        matches!(self, Self::Mask)
    }

    /// O sinal (o `Ctrl` de todo app de escultura) muda o RESULTADO deste verbo?
    ///
    /// ⚠️ **Era uma blacklist, e ela MENTIA.** Ao excluir só `Smooth`/`Sharpen` e
    /// `Pinch`/`Magnify`, ela afirmava sinal para `Flatten`, `Fill` e `Scrape` —
    /// e o `invert` **nunca chega neles**: o alvo dos três é `project(base,
    /// plane)` (`stroke.rs:410-424`), que não lê o `reach`, o único canal por
    /// onde o sinal viaja até um verbo de posição. Três controles mortos, com uma
    /// função afirmando que estavam vivos.
    ///
    /// A lista verdadeira é a de quem CONSOME o sinal: `Draw`, `Inflate` e
    /// `Clay` somam `reach` (`stroke.rs:397,398,427`), `Crease` soma `-reach`
    /// (`:435`), e `Mask` troca o alvo do canal dele de 1 para 0
    /// (`apply_mask`, `:481`).
    ///
    /// ⚠️ **Whitelist e não blacklist, e a direção é o conserto.** Numa
    /// blacklist um verbo NOVO nasce reivindicando um sinal que talvez não tenha,
    /// em silêncio — que é exatamente como este defeito nasceu. Numa whitelist
    /// ele nasce sem sinal, e quem o tem escreve o nome aqui.
    ///
    /// ⚠️ **Isto NÃO é um `uses_reach()`.** O `Mask` não lê `reach` — o alvo de
    /// posição dele é o próprio lugar — e mesmo assim tem oposto. A pergunta é
    /// sobre *o resultado que o artista vê*, e `reach` e `apply_mask::goal` são
    /// duas implementações dela.
    ///
    /// **As três alternativas, e por que cada uma morre:** *"faça o invert
    /// funcionar no Flatten"* — não há o que negar, o Flatten projeta nos dois
    /// sentidos e o oposto dele é ele mesmo; *"Ctrl troca Fill↔Scrape"* — é o
    /// `_negative` do `Flatten.js`, mas ele tem UM tool com um toggle e nós temos
    /// DOIS verbos com dois chips, então o rail destacaria "Fill" enquanto a
    /// ferramenta raspa; *"Ctrl nega o `plane_offset`"* — o slider já tem sinal
    /// nos dois sentidos, com gate provando
    /// (`the_plane_offset_lifts_the_plane_the_verbs_project_onto`).
    ///
    /// ⚠️ **Nenhuma UI pergunta isto hoje** (o shell arma `invert = ctrl`
    /// incondicionalmente, `sculpt3d.rs`): o consumidor é o [`Brush::reach`], e o
    /// chip que decide oferecer ou não o controle é da wave que trouxer painel.
    ///
    /// ⚠️ **Os dois [`Grip::Turn`] ficam de fora, e a razão é que o gesto já tem
    /// sinal:** varrer para o outro lado torce ao contrário, arrastar para a
    /// esquerda encolhe. Um `Ctrl` ali seria a segunda maneira de dizer a mesma
    /// coisa — e uma que **compõe** com a primeira, então varrer ao contrário
    /// com `Ctrl` apertado voltaria a torcer no sentido original.
    #[must_use]
    pub fn honours_invert(self) -> bool {
        matches!(
            self,
            Self::Draw
                | Self::Inflate
                | Self::Clay
                | Self::Crease
                | Self::Blob
                | Self::Mask
                // ⚠️ **O Ctrl VIRA O V**, e o oposto de cavar um vinco é
                // enchê-lo: com o ângulo negativo as duas normais tombam ao
                // contrário, o telhado vira vale e o culling de lado se desliga
                // (`if (angle >= 0.0f)`). É o `if (flip) angle *= -1` do
                // `multiplane_scrape.cc:657`, e não uma força negativa.
                | Self::MultiplaneScrape
                // ⚠️ **A DEMÃO cava, e é o `brush.direction` da referência** —
                // no `layer.cc` o sinal viaja no `cache.bstrength`, que o
                // Blender já entrega negativo. Aqui ele viaja no alvo (o `sign`
                // do `compute_target`), porque o nosso `accum` é a MAGNITUDE da
                // demão e uma magnitude não tem lado.
                | Self::Layer
        )
    }

    /// Este verbo ajusta um plano à pegada do dab? (Quem responde `true` usa o
    /// knob `plane_offset`.)
    #[must_use]
    pub fn uses_plane(self) -> bool {
        matches!(self, Self::Flatten | Self::Fill | Self::Scrape | Self::Clay)
    }

    /// Este verbo lê o anel de vizinhos? (Quem responde `true` custa a
    /// travessia do CSR por vértice, e é o que decide se o `vert_verts` pode um
    /// dia virar preguiçoso.)
    ///
    /// ⚠️ **O [`Self::SurfaceSmooth`] o percorre DUAS vezes** — uma para a média
    /// das posições, outra para a média dos `b` —, e ele nasceu FORA desta
    /// lista: a pergunta que ela responde é *quem precisa da adjacência*, e uma
    /// resposta falsa aqui é como um `vert_verts` preguiçoso deixaria de
    /// construí-la exatamente para o verbo que mais a usa. O gate irmão
    /// `the_families_that_the_ui_asks_about_agree_with_the_verb_list` enumera
    /// os nomes, e ficou VERDE sobre a omissão até alguém a procurar.
    #[must_use]
    pub fn uses_neighbours(self) -> bool {
        matches!(self, Self::Smooth | Self::Sharpen | Self::SurfaceSmooth)
    }

    /// O nome que a UI mostra.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Draw => "Draw",
            Self::Inflate => "Inflate",
            Self::Smooth => "Smooth",
            Self::Sharpen => "Sharpen",
            Self::Flatten => "Flatten",
            Self::Fill => "Fill",
            Self::Scrape => "Scrape",
            Self::Clay => "Clay",
            Self::Pinch => "Pinch",
            Self::Magnify => "Magnify",
            Self::Crease => "Crease",
            Self::Blob => "Blob",
            Self::Mask => "Mask",
            Self::Move => "Move / Grab",
            Self::SnakeHook => "Snake Hook",
            Self::Twist => "Twist",
            Self::LocalScale => "Local Scale",
            Self::ClayStrips => "Clay Strips",
            Self::ClayThumb => "Clay Thumb",
            Self::MultiplaneScrape => "Multiplane Scrape",
            Self::SlideRelax => "Slide Relax",
            Self::SurfaceSmooth => "Surface Smooth",
            Self::Layer => "Layer",
        }
    }

    /// **QUAL campo elástico este verbo consegue consumir** — a metade *qual* da
    /// pergunta cuja metade *se* mora no [`crate::RefMode::field`].
    ///
    /// ⚠️ **Ela é do VERBO porque é um fato sobre ele, não sobre o modo:** um
    /// verbo que gira só sabe consumir uma torção, e o alvo dele nomeia esse
    /// kernel. Enquanto as duas metades viviam na tabela do modo, um par trocado
    /// era **escrivível** e o produto o engolia em silêncio — o alvo caía no
    /// modo que já shipava e a pegada continuava a do campo
    /// ([`Brush::query_radius`] só pergunta `is_some`), ou seja um `l-mode` com
    /// o alcance e sem a lei. A mutação que o instalou passou nos **193** gates.
    ///
    /// ⇒ Com o *qual* aqui, o par deixa de poder discordar: não há segundo sítio
    /// para ele estar escrito.
    #[must_use]
    pub const fn elastic_field(self) -> Option<crate::Field> {
        match self {
            // O agarre (eq. 5). O Snake Hook é o MESMO campo com a âncora a
            // andar — o que os separa é o [`Grip::Hook`], não a lei.
            Self::Move | Self::SnakeHook => Some(crate::Field::Grab),
            Self::Twist => Some(crate::Field::Twist),
            Self::LocalScale | Self::Magnify => Some(crate::Field::Scale),
            // ⛔ **A FAMÍLIA QUE APERTA NÃO TEM CAMPO, e a linha que dizia
            // `Some(Field::Pinch)` foi RETIRADA em 2026-08-15 depois de um
            // report do Enio (*"Blob modo L ruim … em L Pinch ruim"*) e de a
            // medição concordar com ele em três eixos** — sonda
            // `measure_pinch_family_modes`, malha de 64×96, pincel `r = 0,30`,
            // traço de 8 eventos a força 0,75:
            //
            // | verbo | modo | fora do anel | ΔV/V (10⁻⁴) |
            // |---|---|---|---|
            // | Pinch | S | 0,0 % | −0,92 |
            // | Pinch | **L** | **62,4 %** | **−4,43** |
            // | Crease | S | 0,0 % | −9,50 |
            // | Crease | **L** | **43,7 %** | −11,48 |
            // | Blob | B | 0,0 % | +10,52 |
            // | Blob | **L** | **46,5 %** | +11,95 |
            //
            // ⚠️ **Metade a dois terços do gesto caía FORA do anel do cursor** —
            // o `KELVINLET_REACH = 3` é a feature do verbo que AGARRA (o doc
            // dele nomeia o preço: *"o anel do cursor deixa de significar o que
            // eu toco"*) e é o defeito de um verbo que APERTA, que é local por
            // definição.
            //
            // ⚠️ **E o campo PIORAVA justamente o que ele existia para curar.**
            // A nota do [`crate::Verb::Pinch`] afirmava *"com campo ele deixa de
            // REMOVER VOLUME … o que sai de lado sai pela normal: aperta E
            // espirra"*; medido, o Pinch com campo remove **4,8× mais** volume
            // que o sem, e dentro do anel o deslocamento normal é **NEGATIVO**
            // (−0,00078 na banda 0,5-0,75 r contra um lateral de +0,00761): ele
            // AFUNDA, não espirra. O mecanismo é geometria — o traço zero
            // reparte `+s` na normal e `−s/2` no plano, mas numa MALHA os
            // vértices vivem na superfície (`r · n ≈ 0`), então o termo normal é
            // ~zero e não há material fora do plano para receber o que sai de
            // lado. *Uma casca não tem para onde espirrar.*
            //
            // ⛔ **E não há corte honesto que o localize:** o perfil lateral é
            // quase CHATO até o anel (0,00304 · 0,00649 · 0,00761 · 0,00666 nas
            // quatro bandas de dentro) e ainda vale **88 % do pico** em `1,0 r`
            // — cortá-lo ali seria um degrau trinta vezes maior que os 2,90 %
            // que o [`crate::kelvinlet::rim_landing`] foi construído para curar.
            //
            // ⚠️ **A REFERÊNCIA chegou à mesma conclusão, e é isso que fecha:**
            // o `elastic_deform.cc` do Blender porta este paper e declara CINCO
            // famílias — `GRAB`, `GRAB_BISCALE`, `GRAB_TRISCALE`, `SCALE`,
            // `TWIST`. **Nenhuma é o pinch.** O SculptGL não tem Kelvinlets. O
            // paper tem a família afim de traço zero como MATEMÁTICA e nenhum
            // escultor a shipa como PINCEL.
            //
            // ⇒ Um chip `L` aqui era exatamente o que a §4 do plano proíbe: uma
            // LEI inteira vestida com a autoridade de uma fonte que não a
            // declara. O `L` desaparece destes três por construção — o
            // [`crate::RefMode::declares`] pergunta `field(verb).is_some()` —, e
            // não por uma segunda lista a manter.
            _ => None,
        }
    }

    /// **Como este verbo consome o gesto** — ver [`Grip`]. A porta única de que
    /// [`Self::anchors`] é uma leitura, e sobre a qual o kernel e o shell fazem
    /// perguntas diferentes.
    #[must_use]
    pub fn grip(self) -> Grip {
        match self {
            Self::Move => Grip::Hold,
            Self::SnakeHook => Grip::Hook,
            Self::Twist => Grip::Turn(Amount::Angle),
            Self::LocalScale => Grip::Turn(Amount::Fraction),
            // O CARIMBO: a faixa compõe sobre a lista de dabs como o Draw.
            Self::ClayStrips => Grip::Stamp,
            Self::Mask => Grip::Paint,
            _ => Grip::Stamp,
        }
    }

    /// **Este verbo PEGA uma âncora no pen-down** em vez de carimbar? — uma
    /// leitura de [`Self::grip`] em vez de um segundo predicado.
    ///
    /// Os três grips que não são [`Grip::Stamp`] têm em comum que o primeiro
    /// toque **escolhe um ponto e não move nada**: o barro só anda quando o dedo
    /// anda, porque no instante do pen-down o gesto ainda vale zero (o puxão, o
    /// incremento, o ângulo varrido, a fração de escala).
    ///
    /// ⚠️ **O nome era `pulls()`, e ele passou a MENTIR quando o
    /// [`Grip::Turn`] chegou** — um redemoinho não puxa nada. O que a pergunta
    /// sempre quis dizer é *este verbo tem âncora?*, e é essa a palavra que
    /// sobrevive a um quinto grip.
    ///
    /// ⚠️ **Ela era `!matches!(grip, Stamp)`, e o quinto grip a tornou FALSA:**
    /// o [`Grip::Paint`] também não carimba geometria, e um verbo de máscara
    /// não tem âncora nenhuma. A pergunta passou a ser feita pelo lado
    /// POSITIVO — quem de fato pega um ponto no pen-down —, que é a forma que
    /// sobrevive ao sexto grip em vez de o adotar em silêncio.
    #[must_use]
    pub fn anchors(self) -> bool {
        matches!(self.grip(), Grip::Hold | Grip::Hook | Grip::Turn(_))
    }
}
/// **OS DEFAULTS** — com que números um verbo nasce. Ver [`defaults`].
#[path = "brush_verb_defaults.rs"]
mod defaults;

/// **QUEM PODE SER UM FILTRO** — a lei e a faixa. Ver [`filter`].
#[path = "brush_verb_filter.rs"]
mod filter;
pub use filter::{FilterKind, FilterLaw};

/// **AS MAGNITUDES** — quanto cada família desloca. Ver [`magnitudes`].
#[path = "brush_magnitudes.rs"]
mod magnitudes;
pub use magnitudes::{
    BLENDER_REACH_FRACTION, CLAY_PLANE_FRACTION, CLAY_THUMB_TILT_MAX_DEG, CLAY_THUMB_TILT_STEP_DEG,
    CREASE_FRACTION, DEFAULT_MULTIPLANE_ANGLE_DEG, LAYER_HEIGHT_HARD_MAX, LAYER_HEIGHT_UI_MAX,
    MULTIPLANE_ANGLE_MAX_DEG, MULTIPLANE_ANGLE_SMOOTH, MULTIPLANE_TIP_STRETCH, PINCH_GAIN,
    REACH_FRACTION, STRIP_PLANE_FRACTION,
};
