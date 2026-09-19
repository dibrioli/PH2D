#![forbid(unsafe_code)]
//! ⭐⭐⭐ **A CAMADA DE ESTILO** — os botões para MENTIR DE PROPÓSITO, por cima de um pipeline
//! honesto (`docs/Render3d/03`, a `W8`).
//!
//! # A tese que põe esta camada no FIM e não no princípio
//!
//! O [`docs/Render3d/01`] mede o alvo do dono e conclui que *«o que faz aquele jogo ser bonito não é
//! um shader — é o pipeline ser fisicamente honesto para a direcção de arte poder depois mentir de
//! propósito»*. ⇒ **primeiro a honestidade física, depois os botões para a trair**, e esta crate é a
//! segunda metade.
//!
//! ⚠️ **Ela não é uma segunda lei de material.** Um material responde *«que luz esta superfície
//! devolve?»* e obedece à física (energia conservada, o *furnace test* de pé). Isto responde *«que
//! luz quero eu que ela devolva?»*, e não obedece a nada — é por isso que mora numa crate à parte,
//! **depois** do [`ph2d_material`] e **antes** do [`ph2d_view_transform`].
//!
//! ```text
//! material + luz  →  [ESTILO]  →  olhar (exposição + vista)  →  sRGB
//!  (a física)        (a mentira)   (a ph2d-view-transform)
//! ```
//!
//! # ⭐⭐⭐ Os quatro botões, e porque são estes quatro
//!
//! | botão | o que ele mente | o que o olho lê |
//! |---|---|---|
//! | [`Rim`] | acrescenta luz onde a superfície **foge** do olhar | a peça descola do fundo — é o que separa um personagem de um cenário |
//! | [`Curvature`] | tinge pelo que a peça **é**: aresta contra cova | o aspecto pintado à mão, sem ninguém pintar um mapa |
//! | [`Zones`] | tinge o que é **escuro** e o que é **claro**, com cores diferentes | a grade de cor de um filme — sombras frias, luzes quentes |
//! | [`Style::indirect_saturation`] | satura (ou lava) **só** a luz que ricocheteia | o sangramento de cor da `W5` deixa de ser tímido, sem tocar nas lâmpadas |
//!
//! ⛔ **E um quinto foi RECUSADO nesta fatia com o mecanismo escrito:** o **contorno** (a tinta na
//! silhueta) **não é uma lei por ponto** — ele precisa dos VIZINHOS no ecrã, logo é um passe, não
//! uma multiplicação. Pô-lo aqui obrigaria esta crate a receber um G-buffer, e ela deixaria de ser
//! a lei que os dois motores partilham. *Ele fica nomeado no [`docs/Render3d/03`] §W8.*
//!
//! # ⭐⭐⭐ A OMISSÃO É A IDENTIDADE, e **por construção** — nunca por um `if`
//!
//! Toda tinta desta crate é uma **cor**, e o valor de fábrica dela é o **branco**; toda soma tem um
//! peso, e o de fábrica é **zero**. As duas formas foram escritas para serem exactas em `f32`:
//!
//! | forma | porque ela é exacta no ponto neutro |
//! |---|---|
//! | `a + (b − a)·w` | com `a == b` o parêntesis é **`0`** e `a + 0·w == a`, seja qual for `w` |
//! | `rgb · tinta` | a tinta acima sai exactamente `1`, e `x · 1 == x` |
//! | `rgb + cor · k` | com `k == 0` a parcela é **`0`** e `x + 0 == x` |
//! | `rgb·s + luma·(1 − s)` | com `s == 1` é `x·1 + y·0 == x` — ⭐ e é a ÚNICA das quatro em que a alternativa falha |
//!
//! ⛔⛔⛔ **E UMA PREMISSA MINHA CAIU AQUI, por uma mutação SOBREVIVENTE.** Esta secção dizia que a
//! forma ingénua `a·(1−w) + b·w` **não serve**, porque `(1−w) + w` não seria `1` para todo `w`.
//! Duas mutações que a instalavam nas tintas passaram os gates todos, e a varredura explicou:
//! **`(1−w) + w` dá `1,0` EXACTAMENTE** em `2 044 824` amostras de `f32` em `[0, 1]` e acima —
//! como tem de dar, porque o erro daquela soma é no máximo **meia ULP** de `1` e o desempate é para
//! o par, que é o próprio `1,0`.
//!
//! ⭐⭐⭐ **O perigo real é outro, e é a mutação que SANGROU que o nomeia: reconstruir `b` a partir
//! de `a + (b − a)` quando `a ≠ b`.** Nas três tintas os dois extremos são iguais no ponto de
//! fábrica (as duas brancas), logo o parêntesis é **zero** e as duas redacções são exactas; na
//! saturação eles são a **luminância** e o **rgb**, que são diferentes, e ali a reconstrução perde
//! bits por cancelamento — `s = 1` deixaria de devolver a entrada.
//!
//! ⇒ **a lei que fica é MULTIPLICAR o valor que se quer de volta, nunca reconstruí-lo por
//! diferença**; e as tintas ficam na forma robusta por ela ser exacta *seja qual for* a faixa do
//! peso — uma propriedade, e não uma coincidência da faixa de hoje. Gate:
//! `a_forma_que_reconstroi_por_diferenca_e_a_que_perde_bits`, com as duas metades.
//!
//! ⇒ um quadro que não pediu estilo nenhum sai **byte a byte** o de antes, sem o caminho de omissão
//! precisar de um ramo — e é isso que faz as paridades já pagas ([`docs/Render3d/08`] §12, a
//! `100,000 %`) ficarem de pé sem serem re-medidas.
//!
//! # ⭐⭐⭐ UM NÚMERO QUE NÃO É FINITO VOLTA AO VALOR DE FÁBRICA — e isso é uma PORTA, não um `if`
//!
//! Toda cerca sobre os **botões** vive em [`Style::sanitized`], que se chama **uma vez por quadro**.
//! ⛔ Ela não está dentro da lei por pixel, e a razão é medida noutro sítio desta casa: *uma conta
//! que só depende de coisas do QUADRO, escrita dentro do laço, corre onde o laço corre.* Um `f32`
//! de painel não muda entre dois píxeis.
//!
//! ⇒ [`Style::apply`] supõe um estilo **já saneado** e guarda apenas as duas entradas que são
//! **geometria por pixel** ([`Point`]) — essas sim mudam a cada amostra, e uma normal degenerada
//! entrega `NaN` a quem o não espera.
//!
//! ⚠️ **As duas portas do produto são o [`Style::sanitized`] (CPU) e o [`wgsl::pack`] (dispositivo),
//! e a segunda chama a primeira** — *o dispositivo não pode receber um bloco sujo por alguém se ter
//! esquecido de uma chamada.*
//!
//! # ⚠️ O que esta crate NÃO sabe, e quem lho diz
//!
//! Ela é por PONTO e não tem cena: quem a chama entrega o [`Point`], e as duas grandezas dele são
//! **adimensionais de propósito**:
//!
//! - [`Point::facing`] é `|N·V|`, que já não tem unidade;
//! - [`Point::curvature`] é `H · raio_da_peça`, e **não** `H`. A curvatura mede-se em `1/comprimento`
//!   — um botão calibrado nela mudaria de sentido ao escalar a peça ou ao trocar de unidades.
//!   Multiplicada pelo raio da bola que envolve a peça ela passa a ser *«quantas vezes esta zona é
//!   mais curva do que a peça inteira»*, que é o que um artista quer dizer. ⭐ Numa esfera ela vale
//!   **`1`** em todo o lado; num filete de `1/10` do raio, **`10`**; numa face plana, **`0`**.
//!
//! # ⛔⛔ NENHUMA conta desta crate usa `f32::mul_add`, e isso é uma LEI
//!
//! O `mul_add` é **fundido**: ele arredonda uma vez onde `a*b + c` arredonda duas. O gémeo desta lei
//! corre em WGSL, onde `a * b + c` é o que a linguagem promete e um `fma` fundido **não** é
//! garantido por nenhum backend. ⇒ escrever a forma fundida de um lado e a solta do outro é uma
//! divergência **por construção**, escrita na única crate cuja razão de existir é os dois motores
//! responderem o mesmo — e ela não apareceria numa paridade de percentagem de pixels: apareceria num
//! **byte, um dia, num pixel**. Há gate a varrer o ficheiro por esse nome.
//!
//! [`docs/Render3d/01`]: ../../../docs/Render3d/01_o_alvo_decomposto.md
//! [`docs/Render3d/03`]: ../../../docs/Render3d/03_o_plano.md
//! [`docs/Render3d/08`]: ../../../docs/Render3d/08_a_luz_indirecta.md

/// Os pesos da luminância (Rec. 709).
///
/// ⚠️ **Declarados aqui porque esta crate é FOLHA** (zero dependências de produção — ver o
/// `Cargo.toml`), e **gateados** contra a porta canónica da casa
/// (`ph2d_color::linear::LinearRgba::luminance`), que é uma dependência de teste. *Uma lei escrita
/// em dois sítios ainda não é uma lei; aqui ela é uma cópia com um gate a prender as duas.*
pub const LUMA: [f32; 3] = [0.2126, 0.7152, 0.0722];

/// ⭐⭐⭐ **A LUZ DE CONTORNO** — a luz que se acrescenta onde a superfície **foge** do olhar.
///
/// # O que ela é, e o que ela não é
///
/// Ela **não** é uma lâmpada: não tem posição, não lança sombra, não obedece ao material e não
/// aparece no ricochete. É tinta acrescentada à radiância que sai do ponto, escolhida pela
/// **geometria vista do olho** — `(1 − |N·V|)^largura`. É por isso que ela vive aqui e não no
/// [`ph2d_light`]: uma lâmpada é física, isto é direcção de arte.
///
/// ⚠️ **Ela é acrescentada em luz de CENA**, antes do olhar — logo ela **respira com a exposição**,
/// como tudo o resto. Uma luz de contorno somada depois da vista ficaria igual em todas as
/// exposições, e o artista veria o contorno separar-se da peça ao expor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rim {
    /// A cor da luz acrescentada, em linear de cena. **Fábrica: branco.**
    pub color: [f32; 3],
    /// Quanta. **Fábrica: `0`** — e é o `0` que faz o quadro de omissão ser o de antes, ao bit.
    pub strength: f32,
    /// ⭐ **Quão apertado é o contorno** — o expoente de `(1 − |N·V|)`.
    ///
    /// `1` é uma lavagem larga sobre a peça inteira; `8` é um fio na silhueta. **Fábrica: `3`**, que
    /// é o valor que só vale alguma coisa quando a força sair de `0`.
    ///
    /// ⚠️ **A faixa é limitada, e o recurso tem nome:** acima de [`Rim::MAX_WIDTH`] o expoente
    /// entrega uma banda mais fina do que um pixel numa peça enquadrada a `1080p`, logo o que se vê
    /// deixa de ser um contorno e passa a ser o **serrilhado** dele. ⛔ Abaixo de `0` a potência de
    /// uma base zero é `+∞`, e `∞ · 0` é `NaN` — a cerca fecha isso antes da lei.
    pub width: f32,
}

impl Rim {
    /// O expoente mais apertado que a lei aceita — ver [`Rim::width`].
    pub const MAX_WIDTH: f32 = 64.0;
}

impl Default for Rim {
    fn default() -> Self {
        Self {
            color: [1.0; 3],
            strength: 0.0,
            width: 3.0,
        }
    }
}

/// ⭐⭐⭐ **A TINTA POR CURVATURA** — a peça tinge-se pelo que ela **é**, e não por um mapa pintado.
///
/// Uma aresta recebe uma cor, uma cova recebe outra, e uma face plana não recebe nenhuma. É o efeito
/// que num pipeline de produção custa um *cavity map* assado por artista — e aqui ele é **de graça**,
/// porque a peça já é um campo de distância e a curvatura sai da MESMA soma que a normal percorre
/// (`ph2d_field_render::curvatura`).
///
/// # ⭐⭐⭐ O SINAL é a wave, e ele já existia — deitado fora
///
/// A curvatura média de um campo de distância é `H = ∇²f / 2`, e ela **tem sinal**: positiva numa
/// bossa, negativa numa cova. O consumidor que a estreou (a subsuperfície maciça,
/// `docs/Render3d/10`) pede um **comprimento** — a referência do MaterialX devolve
/// `length(fwidth(N))/length(fwidth(P))`, que é `≥ 0` por construção — e por isso o porte dela
/// escrevia `abs(…)` **dentro** da lei.
///
/// ⇒ *o sinal estava calculado e era descartado uma linha antes de alguém o poder ler.* Aqui ele é a
/// diferença entre uma aresta e um vinco, que é a diferença entre um contorno e uma sujidade.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Curvature {
    /// A tinta de uma **ARESTA** (curvatura positiva). **Fábrica: branco.**
    pub convex: [f32; 3],
    /// A tinta de uma **COVA** (curvatura negativa). **Fábrica: branco.**
    pub concave: [f32; 3],
    /// ⭐ **Quanta curvatura já conta como tinta cheia** — o multiplicador antes do corte em `±1`.
    ///
    /// Com `1`, uma zona tão curva quanto a peça inteira (uma esfera) recebe a tinta toda. Com `4`,
    /// basta ser `4×` mais curva que a peça — o que faz a tinta ir para os filetes e deixar o corpo
    /// em paz. **Fábrica: `1`**, que é a leitura sem opinião nenhuma.
    pub sharpness: f32,
}

impl Default for Curvature {
    fn default() -> Self {
        Self {
            convex: [1.0; 3],
            concave: [1.0; 3],
            sharpness: 1.0,
        }
    }
}

/// ⭐⭐⭐ **A GRADE POR ZONA** — uma cor para o que é escuro, outra para o que é claro.
///
/// É a grade de cor de um filme, na forma mais barata que existe e a única que não precisa de um
/// passe: o que é sombra puxa para uma tinta, o que é luz puxa para outra, e o que está no meio
/// mistura-se. Sombras frias com luzes quentes é metade da direcção de arte de um jogo estilizado.
///
/// # ⚠️ A repartição é uma RAZÃO, e não um `smoothstep` entre dois limiares
///
/// `h = l / (l + pivô)`, com `l` a luminância de cena. Ela é **suave em toda a recta**, vale `0` no
/// preto, `½` exactamente no pivô e tende a `1` sem lá chegar — logo não tem tecto e **nunca satura
/// numa cena HDR**, que é onde dois limiares fixos deixam de repartir nada.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Zones {
    /// A tinta do que é **escuro**. **Fábrica: branco.**
    pub shadow: [f32; 3],
    /// A tinta do que é **claro**. **Fábrica: branco.**
    pub highlight: [f32; 3],
    /// ⭐ **Onde fica a fronteira** — a luminância de cena que reparte meio a meio.
    ///
    /// **Fábrica: [`Zones::MIDDLE_GREY`]**, e ele **não é escolhido**: é o cinzento médio da
    /// fotografia, o mesmo `0,18` de que toda a exposição deste mundo é medida.
    pub pivot: f32,
}

impl Zones {
    /// O cinzento médio da fotografia — `18 %` de reflectância, que é o que um fotómetro chama de
    /// «meio». ⛔ **Não é um número afinado**: é a âncora da própria escala de exposição.
    pub const MIDDLE_GREY: f32 = 0.18;

    /// O piso do [`Zones::pivot`].
    ///
    /// ⚠️ Ele fecha uma **degenerescência**, não um gosto: com o pivô em `0` a razão `l/(l+0)` é `1`
    /// em toda a peça (tudo vira luz) e **`0/0` no preto**, que é `NaN`. *Um piso que protege a
    /// aritmética é o mesmo desenho do `POINT_LAMP_MIN_DISTANCE` do traçado.*
    pub const MIN_PIVOT: f32 = 1e-6;
}

impl Default for Zones {
    fn default() -> Self {
        Self {
            shadow: [1.0; 3],
            highlight: [1.0; 3],
            pivot: Self::MIDDLE_GREY,
        }
    }
}

/// ⭐⭐⭐ **A CAMADA DE ESTILO DA CENA** — os quatro botões, juntos porque viajam juntos.
///
/// ⚠️ **Ela é da CENA e não de cada material**, pela mesma razão que o [`ph2d_view_transform::Look`]
/// é da cena: é direcção de arte, e duas peças da mesma imagem debaixo de duas grades diferentes não
/// são uma imagem. *O material diz de que MATÉRIA a peça é; isto diz que FILME ela está a fazer.*
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub rim: Rim,
    pub curvature: Curvature,
    pub zones: Zones,
    /// ⭐⭐⭐ **Quanto a luz INDIRECTA satura** — `1` é a que a física entregou.
    ///
    /// Acima de `1` o sangramento de cor da `W5` (`docs/Render3d/08`) fica mais afirmado; abaixo
    /// dele lava-se para cinzento. **Fábrica: `1`.**
    ///
    /// # ⚠️ Porque ela é um campo SOLTO e não mais uma tinta
    ///
    /// As outras três actuam sobre a luz que **sai** do ponto — depois de somadas as lâmpadas, o
    /// céu e o ricochete. Esta escolhe **uma parcela** da soma antes de ela ser somada, e é isso que
    /// a torna útil: saturar tudo saturaria também o realce do sol. ⇒ ela tem um consumidor
    /// diferente ([`Style::saturate_indirect`], chamada dentro do sombreador) e não entra no
    /// [`Style::apply`].
    pub indirect_saturation: f32,
}

/// **O que a lei sabe sobre este ponto da superfície.** Ver o cabeçalho para porque as duas
/// grandezas são adimensionais.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    /// `|N · V|` — `1` de frente para o olho, `0` na silhueta.
    pub facing: f32,
    /// ⭐ `H · raio_da_peça`, **com sinal**: positivo numa aresta, negativo numa cova, `0` no plano.
    pub curvature: f32,
}

impl Default for Style {
    /// ⚠️ **A identidade**, e ela é exacta — ver o cabeçalho. Um consumidor que nunca pediu estilo
    /// nenhum pinta o quadro de antes, byte a byte, sem um ramo pelo caminho.
    fn default() -> Self {
        Self {
            rim: Rim::default(),
            curvature: Curvature::default(),
            zones: Zones::default(),
            indirect_saturation: 1.0,
        }
    }
}

impl Style {
    /// ⭐⭐⭐ **A LEI INTEIRA**, sobre a luz que sai do ponto — em linear de CENA, antes do olhar.
    ///
    /// A ordem é a que o cabeçalho desenha, e ela **não é arbitrária**:
    ///
    /// 1. **a tinta da curvatura** — é uma propriedade da SUPERFÍCIE, logo pinta o que o material
    ///    devolveu e mais nada;
    /// 2. **a luz de contorno** — é luz ACRESCENTADA, logo entra depois de a superfície estar
    ///    pintada (o contorno não é tingido pela cova que ele contorna);
    /// 3. **a grade por zona** — é o olhar do director de fotografia, logo é a última e vê tudo,
    ///    contorno incluído.
    ///
    /// ⛔ **A saturação da indirecta NÃO está aqui** — ver [`Style::indirect_saturation`].
    #[must_use]
    pub fn apply(&self, scene: [f32; 3], at: Point) -> [f32; 3] {
        let rgb = self.curvature_tinted(scene, at.curvature);
        let rgb = self.rim_lit(rgb, at.facing);
        self.graded(rgb)
    }

    /// **(1) A tinta da curvatura.** Ver [`Curvature`].
    ///
    /// ⚠️ **Sem ramo entre aresta e cova, e é de propósito:** os dois pesos são `max(±c, 0)` e só um
    /// deles pode ser diferente de zero, logo somar as duas parcelas dá o mesmo que escolher uma —
    /// e o gémeo em WGSL não fica com um `if` que a placa executaria nas duas vias.
    #[must_use]
    pub fn curvature_tinted(&self, rgb: [f32; 3], curvature: f32) -> [f32; 3] {
        // ⚠️ **A curvatura é GEOMETRIA por pixel** — ver o cabeçalho: os botões já vêm saneados, e
        // esta não vem de botão nenhum.
        let c = (finito(curvature, 0.0) * self.curvature.sharpness).clamp(-1.0, 1.0);
        let (wc, wv) = (c.max(0.0), (-c).max(0.0));
        [0, 1, 2].map(|k| {
            // ⚠️ **`1 + (t − 1)·w`**, e nunca `mix` — ver o cabeçalho: com a tinta em `1` o
            // parêntesis é exactamente `0` e o resultado é exactamente `1`.
            let tint = 1.0
                + (self.curvature.convex[k] - 1.0) * wc
                + (self.curvature.concave[k] - 1.0) * wv;
            rgb[k] * tint
        })
    }

    /// **(2) A luz de contorno.** Ver [`Rim`].
    #[must_use]
    pub fn rim_lit(&self, rgb: [f32; 3], facing: f32) -> [f32; 3] {
        // ⚠️ **A cerca da largura já foi posta pela porta** ([`Style::sanitized`]), e ela não é
        // decorativa: com um expoente negativo, `0^w` é `+∞` numa silhueta exacta, e `∞ · 0` (a
        // força de fábrica) é `NaN` — um pixel branco onde toda a lei promete um quadro
        // byte-idêntico.
        let grazing = (1.0 - finito(facing, 0.0).clamp(0.0, 1.0)).powf(self.rim.width);
        let k = self.rim.strength * grazing;
        [0, 1, 2].map(|i| rgb[i] + self.rim.color[i] * k)
    }

    /// **(3) A grade por zona.** Ver [`Zones`].
    #[must_use]
    pub fn graded(&self, rgb: [f32; 3]) -> [f32; 3] {
        let l = luma(rgb);
        // ⚠️ O denominador é `≥ pivot > 0` (a porta garante o piso) porque `l` já está saneado a
        // `≥ 0` e finito — a divisão não pode dar `NaN` nem `∞`, e é por isso que ela não leva uma
        // cerca por cima.
        let h = l / (l + self.zones.pivot);
        [0, 1, 2].map(|k| {
            // ⚠️ `s + (t − s)·h`: com as duas tintas iguais o parêntesis é `0` e o resultado é
            // **exactamente** a tinta — em particular exactamente `1` no ponto de fábrica.
            let tint = self.zones.shadow[k] + (self.zones.highlight[k] - self.zones.shadow[k]) * h;
            rgb[k] * tint
        })
    }

    /// ⭐⭐⭐ **A SATURAÇÃO DA LUZ INDIRECTA** — chamada DENTRO do sombreador, sobre a parcela
    /// indirecta, antes de as lâmpadas entrarem na soma. Ver [`Style::indirect_saturation`].
    ///
    /// ⚠️ **A forma é `rgb·s + luma·(1 − s)`** e não `luma + (rgb − luma)·s`: as duas são a mesma
    /// álgebra e só a primeira devolve `rgb` **ao bit** em `s = 1`, que é o ponto de fábrica.
    #[must_use]
    pub fn saturate_indirect(&self, rgb: [f32; 3]) -> [f32; 3] {
        let s = self.indirect_saturation;
        let l = luma(rgb) * (1.0 - s);
        [0, 1, 2].map(|k| rgb[k] * s + l)
    }

    /// ⭐⭐⭐ **ESTA CENA PRECISA DA CURVATURA?** — a porta que decide se alguém paga a amostra.
    ///
    /// ⚠️ **Ela é a irmã da [`ph2d_material::Surface::reads_curvature`]**, e as duas juntas são a
    /// condição: o traçado assa o canal da curvatura quando **algum** consumidor a lê. Sem isto o
    /// botão existiria, o artista mexeria nele e a peça não mudaria um pixel — porque a grandeza que
    /// ele escolhe nunca teria sido medida. *É a forma de knob morto que esta casa mede desde
    /// 30/08.*
    ///
    /// ⛔ **Com as duas tintas em branco a resposta é `false` mesmo com a nitidez alta**, e está
    /// certo: uma tinta branca é a identidade, logo a curvatura seria medida para multiplicar por
    /// `1`.
    #[must_use]
    pub fn reads_curvature(&self) -> bool {
        self.curvature.convex != [1.0; 3] || self.curvature.concave != [1.0; 3]
    }

    /// **Esta camada não faz nada?** — o predicado inteiro, para quem quiser saltar trabalho.
    ///
    /// ⚠️ **Nenhum caminho de produto PRECISA dele**, e essa é a propriedade: a lei já é a identidade
    /// exacta no ponto de fábrica (ver o cabeçalho). Ele existe para uma sonda poder afirmar a
    /// identidade **sem** depender de a aritmética a dar — *um gate que mede a saída e um predicado
    /// que a prevê são duas testemunhas, e é quando elas discordam que se aprende alguma coisa.*
    #[must_use]
    pub fn is_identity(&self) -> bool {
        *self == Self::default()
    }

    /// ⭐⭐⭐ **A PORTA DOS BOTÕES** — chama-se **uma vez por quadro**, e depois dela a lei é
    /// aritmética pura. Ver o cabeçalho para porque ela não vive dentro do laço de píxeis.
    ///
    /// # A lei, numa frase
    ///
    /// **Um número que não é finito volta ao valor de FÁBRICA daquele controlo**, e os dois que têm
    /// domínio próprio são apertados nele: a largura do contorno a `0..=`[`Rim::MAX_WIDTH`] e o pivô
    /// ao piso [`Zones::MIN_PIVOT`].
    ///
    /// ⚠️ **Fábrica e não zero**, e a diferença importa no pivô: `NaN → 0 → o piso` poria a peça
    /// inteira do lado das LUZES, que é uma grade a sério a partir de um controlo ilegível. *O
    /// valor de fábrica é o único fallback que se lê igual em todos os cinco.*
    ///
    /// ⭐ **Ela é IDEMPOTENTE e o ponto de fábrica é ponto FIXO** — as duas coisas com gate. Sem a
    /// segunda, a identidade exacta que o cabeçalho promete morreria na porta.
    #[must_use]
    pub fn sanitized(&self) -> Self {
        let d = Self::default();
        let cor = |v: [f32; 3], f: [f32; 3]| [0, 1, 2].map(|k| finito(v[k], f[k]));
        Self {
            rim: Rim {
                color: cor(self.rim.color, d.rim.color),
                strength: finito(self.rim.strength, d.rim.strength),
                width: finito(self.rim.width, d.rim.width).clamp(0.0, Rim::MAX_WIDTH),
            },
            curvature: Curvature {
                convex: cor(self.curvature.convex, d.curvature.convex),
                concave: cor(self.curvature.concave, d.curvature.concave),
                sharpness: finito(self.curvature.sharpness, d.curvature.sharpness),
            },
            zones: Zones {
                shadow: cor(self.zones.shadow, d.zones.shadow),
                highlight: cor(self.zones.highlight, d.zones.highlight),
                pivot: finito(self.zones.pivot, d.zones.pivot).max(Zones::MIN_PIVOT),
            },
            indirect_saturation: finito(self.indirect_saturation, d.indirect_saturation),
        }
    }
}

/// A luminância de cena, **nunca negativa e sempre finita**. Ver [`LUMA`].
///
/// ⚠️ O tecto não é um gosto: ele é o que impede a razão da [`Style::graded`] de calcular `∞/∞`, que
/// é `NaN` — e um `NaN` ali pintaria a peça inteira de preto num quadro em que uma única amostra
/// estourou.
fn luma(rgb: [f32; 3]) -> f32 {
    let c = [0, 1, 2].map(|k| finito(rgb[k], 0.0).max(0.0));
    finito(LUMA[0] * c[0] + LUMA[1] * c[1] + LUMA[2] * c[2], 0.0).max(0.0)
}

/// **Um número que não é finito volta ao NEUTRO que o chamador nomeia.**
///
/// ⚠️ A cerca da [`ph2d_view_transform`] é parecida e tem **uma diferença deliberada: esta NÃO corta
/// o sinal.** Ali toda entrada é luz, e luz negativa é luz nenhuma; aqui a entrada mais importante é
/// a **curvatura**, cujo sinal é a wave inteira (ver [`Curvature`]). *Uma cerca copiada de um
/// vizinho que trata de outra grandeza apaga exactamente o que esta grandeza tem de próprio.*
fn finito(v: f32, neutro: f32) -> f32 {
    if v.is_finite() { v } else { neutro }
}

pub mod wgsl;

#[cfg(test)]
mod tests;
