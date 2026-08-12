//! O PINCEL — o verbo, o falloff e os knobs que o artista gira.
//!
//! Ver `docs/3D/04.1`. A divisão que este arquivo faz, e que decide tudo o
//! resto: **o `Dab` é onde e com que força a mão apertou; o `Brush` é que
//! ferramenta está na mão.** Um traço é uma lista de dabs contra UM brush.

use crate::grip::{Amount, Grip};
use crate::{Alpha, AlphaStencil};

/// A curva de peso do pincel, do centro (`t = 0`) à borda (`t = 1`).
///
/// A MESMA família que o Painter 2D já expõe, e de propósito: um artista que
/// aprendeu *Sharper* pintando não devia reaprendê-la esculpindo. A curva
/// **customizada** (o `ParamWidget::Curve` que o repo já possui) é a 6ª e entra
/// quando houver painel — construir aqui um segundo editor de curva seria a
/// segunda resposta que o `04.1` proíbe em letra.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Falloff {
    /// `(1 − t²)²` — **C¹ na borda** (valor e derivada zeram em `t = 1`), e é
    /// isso que faz um traço não deixar degrau na fronteira do pincel.
    #[default]
    Smooth,
    /// `√(1 − t²)` — o perfil de uma esfera: cheio no miolo, tangente vertical
    /// na borda. Deposita mais massa que o Smooth com o mesmo raio.
    Sphere,
    /// `(1 − t²)⁴` — pico estreito, ombro que morre cedo. É o falloff de quem
    /// quer detalhe pequeno com pincel grande.
    Sharper,
    /// `1` até a borda, e nada além. Um disco duro; o degrau é a feature.
    Constant,
    /// `√(1 − t)` — sobe rápido na borda e achata no miolo, o oposto do Sharper.
    Root,
    /// `3t⁴ − 4t³ + 1` — **a curva da REFERÊNCIA**, e a única desta família que
    /// não foi escolhida por desenho: ela é a que as dez tools de geometria do
    /// SculptGL usam, e é o que a paridade bit-a-bit exige que exista aqui.
    ///
    /// ⚠️ **Ela é mais CHEIA que a `Smooth`, e a diferença é visível:** a meio
    /// raio dá `0,6875` contra `0,5625` — **1,22×** —, e a razão cresce até
    /// `1,44×` a `7/8` do raio. Um artista que trocar de uma para a outra vê o
    /// pincel engordar, não um detalhe numérico.
    ///
    /// ⚠️ **Ela é `C¹` nas DUAS pontas** (derivada `12t²(t − 1)`, que zera em
    /// `t = 0` e em `t = 1`) — daí o nome: um platô no miolo e um pouso sem
    /// degrau na borda. A `Smooth` só é plana na borda.
    ///
    /// ⚠️ **E o valor sai da PORTA ÚNICA** [`crate::ref_kernels::falloff`], em
    /// `f64`, arredondado uma vez: uma segunda cópia da quártica aqui seria a
    /// forma exata de a paridade divergir do porte que ela mede.
    Plateau,
}

impl Falloff {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 6] = [
        Self::Smooth,
        Self::Sphere,
        Self::Sharper,
        Self::Constant,
        Self::Root,
        Self::Plateau,
    ];

    /// O peso a uma distância normalizada `t`. **Porta única** — todo verbo, a
    /// simetria e o cursor perguntam a esta função.
    ///
    /// Sem transcendental exceto a raiz (HR-5): `sqrt` é instrução de hardware,
    /// não chamada de libm.
    #[must_use]
    pub fn weight(self, t: f32) -> f32 {
        // O NaN é peneirado explicitamente: sem isso ele escorre pelas fórmulas
        // e sai como peso NaN num vértice, que é como uma malha inteira vira
        // `NaN` a partir de um dab com raio zero em algum lugar.
        if !t.is_finite() || t >= 1.0 {
            return 0.0;
        }
        let t = t.max(0.0);
        match self {
            Self::Smooth => {
                let u = 1.0 - t * t;
                u * u
            }
            Self::Sphere => (1.0 - t * t).sqrt(),
            Self::Sharper => {
                let u = 1.0 - t * t;
                let u2 = u * u;
                u2 * u2
            }
            Self::Constant => 1.0,
            Self::Root => (1.0 - t).sqrt(),
            // ⚠️ **Em `f64`, e não uma quártica escrita aqui em `f32`.** É a
            // aritmética do original (um `Float32Array` do JS lê `f32 → f64`,
            // calcula em `f64` e arredonda UMA vez), e é o que faz esta curva
            // servir de peça de paridade em vez de parecer com ela.
            Self::Plateau => crate::ref_kernels::falloff(f64::from(t)) as f32,
        }
    }

    /// O nome que a UI mostra.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Smooth => "Smooth",
            Self::Sphere => "Sphere",
            Self::Sharper => "Sharper",
            Self::Constant => "Constant",
            Self::Root => "Root",
            Self::Plateau => "Plateau",
        }
    }
}

/// O que o pincel FAZ. Ver `docs/3D/04.1` para a família de cada um.
///
/// ⚠️ **`Layer` não está aqui, e a ausência é uma decisão medida.** Sob a lei do
/// traço (`accum` é um ENVELOPE em `[0,1]`, nunca uma soma) o Draw já é limitado
/// a um `reach` por traço, por mais que o artista demore — que é exatamente o
/// que o Layer do ZBrush existe para garantir. Os dois colapsam, e o Painter 2D
/// registrou o mesmo colapso pelo mesmo motivo (`docs/Painter/18` §"Draw Sharp
/// colapsa no Layer"). Quem quiser empilhar solta e desenha de novo — o
/// `pre` se re-congela e o traço seguinte soma.
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
}

impl Verb {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 16] = [
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
        Self::Mask,
        Self::Move,
        Self::SnakeHook,
        Self::Twist,
        Self::LocalScale,
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
    #[must_use]
    pub fn accumulates(self) -> bool {
        matches!(self.grip(), Grip::Stamp)
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
            Self::Draw | Self::Inflate | Self::Clay | Self::Crease | Self::Mask
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
    #[must_use]
    pub fn uses_neighbours(self) -> bool {
        matches!(self, Self::Smooth | Self::Sharpen)
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
            Self::Mask => "Mask",
            Self::Move => "Move / Grab",
            Self::SnakeHook => "Snake Hook",
            Self::Twist => "Twist",
            Self::LocalScale => "Local Scale",
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

    /// A força com que um pincel deste verbo **nasce**.
    ///
    /// ⚠️ **A máscara nasce em 1,0 e a geometria em 0,5, e a diferença é o
    /// significado da força em cada canal.** Para geometria ela é *quão longe ao
    /// longo do trajeto*, e meio caminho é um default são. Para a máscara o alvo
    /// é um PLATÔ (protegido) e a lei do traço é um envelope, então a força vira
    /// o **TETO** que um traço alcança: medido com 0,5, esfregar oito dabs no
    /// mesmo lugar chega a **0,5000 e para** — e `keep = 1 − mask` deixa metade
    /// de todo dab seguinte atravessar a proteção. O artista mascara, esculpe, e
    /// o barro se move debaixo da máscara pela metade, que é indistinguível de
    /// *"a máscara não funciona"*. Traços repetidos convergem geometricamente
    /// (0,75 · 0,875 · 0,969), e depois de DEZ ainda são 31 texels acima de 0,99.
    ///
    /// ⚠️ **Divergência do original, e ela é sobre a LEI, não sobre o número:**
    /// lá a força é uma TAXA (ele acumula sobre o estado vivo e satura dentro do
    /// traço), aqui é um TETO (envelope sobre o `pre` congelado). Trocar a nossa
    /// lei devolveria a dependência de espaçamento que o módulo inteiro existe
    /// para não ter; trocar o DEFAULT entrega o gesto sem tocar em nada.
    /// A proteção parcial continua exprimível — é o slider.
    #[must_use]
    pub fn default_strength(self) -> f32 {
        match self {
            Self::Mask => 1.0,
            _ => 0.5,
        }
    }
}

/// Quanto do RAIO do pincel um dab de força cheia desloca.
///
/// ⚠️ **É fração do raio, nunca uma distância absoluta** — a lição que o impasto
/// do Painter pagou em 2026-07-14: com altura absoluta, um pincel pequeno e um
/// grande picam no mesmo valor, e o grande vira uma poça chata porque a razão
/// *altura ÷ largura* despenca. Amarrando ao raio, a razão de aspecto do domo é
/// constante em toda escala e o falloff lê igual com pincel de 1 mm e de 1 m.
///
/// O NÚMERO é decisão de **smoke**, como o `ORBIT_RAD_PER_PX` da câmera: ele não
/// é teto de recurso nenhum, é o quanto de barro uma pincelada move.
pub const REACH_FRACTION: f32 = 0.1;

/// O ganho do **Crease**, e o do vinco é MENOR que o do Draw.
///
/// ⚠️ **Os três números desta família saem da referência, não de afinação:**
/// `Brush.js` e `Inflate.js` usam `intensidade · raio · 0,1`, o `Crease.js` usa
/// `intensidade · 0,07` e o `Pinch.js` usa `intensidade · 0,05`. Eles são o que
/// separa *"a lei está certa"* de *"a ferramenta responde como a referência"* —
/// e a sonda `measure_reference_divergence` é quem os cobra.
pub const CREASE_FRACTION: f32 = 0.07;

/// O ganho do **Pinch** e do **Magnify** — ver [`CREASE_FRACTION`].
///
/// ⚠️ **Ele não existia, e a ausência valia 20×:** o alvo era `base + tangente`
/// atenuado pelo peso, ou seja o vértice caminhava até `w` da distância inteira
/// ao centro **num dab**. Medido contra a referência, `16,88×`.
pub const PINCH_GAIN: f32 = 0.05;

/// Quanto o plano do **Clay** sobe acima da superfície, em fração do raio.
///
/// ⚠️ **É o literal `0.1` do `Brush.js:52`, e ele não é um knob no original.** O
/// nosso `plane_offset` é um controle do artista e SOMA a este — o default dele
/// (`0`) devolve a referência exata, e girá-lo levanta o plano a mais.
pub const CLAY_PLANE_FRACTION: f32 = 0.1;

/// A ferramenta na mão. Um traço inteiro corre contra um destes.
/// ⚠️ **`Copy` MORREU quando o alpha ganhou IMAGEM** — ver [`crate::Alpha`], onde
/// a decisão e o preço medido estão escritos. Nada no caminho quente copiava um
/// `Brush` por vértice; o que sobrou foram três `derive` e cinco `.clone()` num
/// arquivo de teste, contra a estimativa herdada de *"~20 arquivos"*.
#[derive(Clone, Debug, PartialEq)]
pub struct Brush {
    pub verb: Verb,
    /// **ACUMULAR na mesma pincelada** — o `BRUSH_ACCUMULATE` do Blender, e o
    /// irmão exato do campo de mesmo nome do pincel 2D.
    ///
    /// Desarmado (o default) a lei é o ENVELOPE: cruzar o próprio traço não
    /// intensifica nada, e uma pincelada deposita no máximo a força do pincel.
    /// Armado ela é uma **integral de linha** — ver [`crate::ACCUM_PER_DAB`] —, e
    /// passar duas vezes soma duas vezes.
    ///
    /// ⚠️ **Só os verbos de CARIMBO o leem** ([`Verb::accumulates`]): quem tem
    /// âncora carrega o gesto TOTAL desde o pen-down, e somar totais seria somar
    /// a mesma coisa N vezes.
    pub accumulate: bool,
    pub falloff: Falloff,
    /// **O PADRÃO que decide onde, dentro da pegada, o verbo age** — ver
    /// [`crate::Alpha`]. `None` é o pincel liso, e ele é **byte-idêntico** ao
    /// mundo pré-alpha (a porta devolve `1.0` exato, e `x * 1.0 == x` no
    /// IEEE-754).
    pub alpha: Option<Alpha>,
    /// O tamanho de uma feature do alpha, em unidades de OBJETO — ver
    /// [`crate::DEFAULT_ALPHA_SCALE`].
    pub alpha_scale: f32,
    /// **O AZIMUTE do eixo do padrão**, em graus (`0..360`) — só os padrões
    /// direcionais o leem ([`Alpha::is_directional`]).
    ///
    /// ⚠️ **Graus INTEIROS, e é a convenção da lâmpada** (`ph2d_light::Light`):
    /// o rotor deste app anda de um grau em um grau, então um ângulo fracionário
    /// não teria como ser resolvido sem um segundo caminho. Um número, uma
    /// régua, um rotor.
    pub alpha_az_deg: u16,
    /// **A ELEVAÇÃO do eixo**, em graus (`0..=`[`crate::MAX_AXIS_ELEV_DEG`]).
    ///
    /// ⚠️ **Sem o piso que a LÂMPADA tem.** Lá o `MIN_ELEV_DEG` existe porque
    /// uma luz rasante degenera a resposta plana; um EIXO não degenera em lugar
    /// nenhum — o frame é ortonormal por identidade em qualquer elevação —, e um
    /// piso aqui seria um limite copiado de um vizinho em vez de medido.
    pub alpha_elev_deg: u16,
    /// **ONDE o padrão POUSA**, no plano do frame e em unidades de OBJETO.
    ///
    /// ⚠️ **É a terceira metade de COLOCAR um carimbo** — tamanho
    /// (`alpha_scale`), giro (`alpha_az_deg`, que no zênite ROLA o padrão no
    /// plano) e posição. As duas primeiras já existiam; esta faltava, e sem ela
    /// o artista podia dizer *quão grande* e *para que lado*, nunca *onde*.
    ///
    /// ⚠️ **Ele só alcança o motor com uma IMAGEM armada** — ver
    /// [`Brush::alpha_frame`], que é onde a neutralidade é garantida por
    /// CONSTRUÇÃO em vez de por convenção.
    pub alpha_offset: [f32; 2],
    /// **A VISTA, quando ela é conhecida** — o que faz de uma imagem um
    /// ESTÊNCIL preso ao viewport. Ver [`AlphaStencil`].
    ///
    /// ⚠️ **`None` é o mundo pré-estêncil, ao bit**: sem ele o
    /// [`Brush::alpha_frame`] devolve exatamente o frame autorado de antes. É o
    /// que mantém os nove procedurais — e todo gate de kernel — intocados.
    ///
    /// ⚠️ **Ele é ESTADO DE QUADRO, não estado autorado:** quem o preenche é a
    /// cena, uma vez por peça por quadro, porque só ela conhece a câmera e a
    /// pose. Guardá-lo como ajuste do pincel seria congelar uma vista que o
    /// artista move a cada gesto.
    pub alpha_stencil: Option<AlphaStencil>,
    /// **O tamanho de um ladrilho do ESTÊNCIL, em fração da ALTURA DA TELA.**
    ///
    /// ⚠️ **Um campo PRÓPRIO, e não uma reinterpretação do `alpha_scale`.** Os
    /// dois respondem a perguntas diferentes — *que tamanho tem esta feature no
    /// MODELO* contra *que tamanho tem este carimbo na TELA* — e um número só
    /// com duas unidades trocaria de significado em silêncio no instante em que
    /// o artista trocasse de padrão, que é a doença que este módulo varre a cada
    /// wave. Duas perguntas, dois números, duas rows: cada uma aparece no modo
    /// em que está viva.
    pub alpha_stencil_scale: f32,
    /// Raio de influência, em unidades de MUNDO.
    pub radius: f32,
    /// Intensidade em `[0, 1]` — o que multiplica o falloff para virar o peso.
    pub strength: f32,
    /// O `Ctrl` de todo app de escultura: cava em vez de levantar.
    pub invert: bool,
    /// Desloca o plano ao longo da normal da área, em fração do raio. Positivo
    /// adiciona matéria (é o que faz do Flatten um Clay). Só os verbos de plano
    /// o leem — ver [`Verb::uses_plane`].
    pub plane_offset: f32,
    /// Quanto o `Crease` aperta lateralmente, em `[0, 1]`.
    pub pinch: f32,
    /// **A dureza da borda do canal de MÁSCARA**, em `[0, 1]` — o `_hardness`
    /// da tool `Masking` do original (`Masking.js:14`).
    ///
    /// ⚠️ **Ele existe porque a máscara tem CURVA PRÓPRIA, e isso é da
    /// referência, não nosso.** A afirmação *"o SculptGL tem uma curva para
    /// tudo"* vale para as dez tools que movem GEOMETRIA; o canal usa
    /// `(1 − d)^{2(1 − hardness)}` (ver [`Brush::mask_weight`]), e é por isso
    /// que o [`Falloff`] — o nosso seletor, que é um superconjunto do que a
    /// referência oferece à geometria — **não** o alcança. Foi exatamente o
    /// pedido *"cada tool deve ter seu falloff apropriado"*: aqui ele não é uma
    /// escolha de produto, é o que o modelo já fazia.
    ///
    /// ⚠️ **`0.25` é o valor DE FÁBRICA da tool**, não um número escolhido —
    /// ele dá expoente `1.5`, uma borda visivelmente mais apertada que a
    /// quártica da geometria (a `Plateau` vale `0,6875` a meio raio, esta vale
    /// `0,3536`). Em `1.0` o expoente é ZERO e o canal vira um disco duro.
    pub mask_hardness: f32,
}

/// A dureza do canal em `1.0` dá expoente **zero**, e `x^0 == 1` em toda a
/// pegada — o disco duro. É o topo da faixa da tool do original, e o nome existe
/// para o painel não repetir o literal.
pub const MAX_MASK_HARDNESS: f32 = 1.0;

impl Brush {
    /// **A CURVA DO CANAL DE MÁSCARA** — `(1 − t)^{2(1 − hardness)}`, o
    /// `Masking.paint` do original (`Masking.js:66-69`).
    ///
    /// ⚠️ **A aritmética é `f64` e a arredondada é UMA**, como em todo o porte
    /// (`ref_kernels`): o `Math.pow` do JS trabalha em duplo e o
    /// `Float32Array` guarda uma vez. Computá-la em `f32` acumularia uma
    /// segunda arredondada e a paridade sairia do piso do formato.
    ///
    /// ⚠️ **`t` chega JÁ normalizado pelo raio** e não é clampado aqui: o
    /// original clampa (`if dist > 1 dist = 1`) e a nota do
    /// [`crate::ref_kernels`] mede que esse ramo é **inalcançável** — quem monta
    /// a pegada só admite `d² < r²`. A guarda contra `t > 1` mora onde o
    /// consumo mora, e duplicá-la aqui seria a segunda resposta à mesma
    /// pergunta.
    #[must_use]
    pub fn mask_weight(&self, t: f32) -> f32 {
        let softness = 2.0 * (1.0 - f64::from(self.mask_hardness));
        (1.0 - f64::from(t)).powf(softness) as f32
    }
}

impl Default for Brush {
    fn default() -> Self {
        Self {
            verb: Verb::Draw,
            accumulate: false,
            falloff: Falloff::Smooth,
            alpha: None,
            alpha_scale: crate::DEFAULT_ALPHA_SCALE,
            // ⚠️ **O eixo nasce em +Y — as camadas saem HORIZONTAIS**, que é a
            // leitura que um estrato tem no mundo e a que o olho resolve na
            // primeira olhada. Com `az = 0` o eixo seria +X e as camadas
            // sairiam de pé; com `elev = 90` ele apontaria para a CÂMERA (a
            // vista é `+Z`) e o artista veria uma camada só, o que é
            // indistinguível de *"o padrão não funciona"*.
            alpha_az_deg: 90,
            alpha_elev_deg: 0,
            alpha_offset: [0.0, 0.0],
            alpha_stencil: None,
            // Um quarto da altura da tela por ladrilho — quatro carimbos
            // atravessando o que se vê. ⚠️ E ele **não** é semeado do modelo: um
            // estêncil não sabe o tamanho da peça, e é essa independência que o
            // artista pediu.
            alpha_stencil_scale: 0.25,
            radius: 0.25,
            strength: 0.5,
            invert: false,
            plane_offset: 0.0,
            pinch: 0.5,
            // O `_hardness` de fábrica da `Masking` do original.
            mask_hardness: 0.25,
        }
    }
}

impl Brush {
    /// O deslocamento, com sinal, que um dab deste pincel alcança — em unidades
    /// de mundo. Porta única: Draw, Inflate, Clay e Crease perguntam aqui.
    ///
    /// ⚠️ **O termo `honours_invert()` aqui é INERTE hoje, e fica assim mesmo.**
    /// Depois que o predicado passou a dizer a verdade, *todo* verbo que lê
    /// `reach` está na whitelist ⇒ trocá-lo por um `if self.invert` puro daria o
    /// mesmo número em todos os doze, e uma mutação que o remova **não sangra**.
    /// Ele fica porque é aqui que a pergunta pertence — *o sinal é assunto do
    /// VERBO, não do checkbox* —, e porque o dia em que entrar um verbo que
    /// consome `reach` sem ter oposto é o dia em que ele deixa de ser inerte, sem
    /// ninguém precisar lembrar. Defesa em camadas documentada em vez de gateada,
    /// pelo precedente do ADR-0145.
    ///
    /// ⚠️ **Este bloco estava ÓRFÃO** — colado acima do `alpha_weight`, descrevendo
    /// uma função que não é esta, com o `reach` sem doc nenhum. É a classe que
    /// este módulo já registrou duas vezes (*"minhas linhas `mod` orfanaram
    /// doc-comments"*), e ela não levanta erro: só uma leitura pega.
    #[must_use]
    pub fn reach(&self, radius: f32) -> f32 {
        let s = if self.invert && self.verb.honours_invert() {
            -1.0
        } else {
            1.0
        };
        radius * REACH_FRACTION * s
    }
}

/// **AS PORTAS DO ALPHA** — ver [`alpha_doors`].
#[path = "brush_alpha.rs"]
mod alpha_doors;

/// Os espelhos que multiplicam um dab.
///
/// ⚠️ **A simetria é aplicada na LISTA DE DABS, num ponto único** — nunca dentro
/// de um verbo. É o que faz um verbo novo herdá-la de graça, e é literalmente a
/// lição que o `stamp_dabs_inner` do Painter 2D deixou escrita.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Symmetry {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl Symmetry {
    /// Só o eixo X — o caso que 95% da escultura de personagem usa.
    pub const MIRROR_X: Self = Self {
        x: true,
        y: false,
        z: false,
    };

    #[must_use]
    pub fn any(self) -> bool {
        self.x || self.y || self.z
    }

    /// Os sinais por eixo de cada cópia, incluindo o original. De 1 a 8
    /// entradas; a primeira é sempre `[1, 1, 1]`.
    ///
    /// Devolve um array fixo + comprimento para não alocar por dab — um traço
    /// emite dezenas de dabs por segundo, e este é o caminho quente.
    #[must_use]
    pub fn signs(self) -> ([[f32; 3]; 8], usize) {
        let mut out = [[1.0f32; 3]; 8];
        let mut n = 1;
        for (axis, on) in [self.x, self.y, self.z].into_iter().enumerate() {
            if !on {
                continue;
            }
            for i in 0..n {
                let mut s = out[i];
                s[axis] = -s[axis];
                out[n + i] = s;
            }
            n *= 2;
        }
        (out, n)
    }
}

#[cfg(test)]
#[path = "brush_tests.rs"]
mod tests;
