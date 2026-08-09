//! **O ALPHA** — o padrão que decide ONDE, dentro da pegada, o verbo age.
//!
//! Um pincel de escultura sem alpha deposita a mesma forma lisa em toda parte, e
//! é por isso que uma peça acabada num app que os tem (ZBrush, Blender, Nomad)
//! lê como pele, escama ou casca, e a nossa lê como barro. O alpha é um número
//! em `[0, 1]` que **multiplica o falloff**: onde ele vale 1 o verbo age cheio,
//! onde vale 0 ele não toca o vértice.
//!
//! ⚠️ **Ele multiplica o FALLOFF, não o peso.** A diferença aparece no Crease,
//! cujo expoente cai sobre `curva × máscara × alpha` — e é assim no original.
//! Multiplicar o peso já formado deixaria o alpha *fora* do expoente, e o verbo
//! afiaria a máscara sem afiar o padrão.
//!
//! # O MAPEAMENTO, e por que ele é 3D
//!
//! O [[04.1-Pinceis]] previa *"uma textura projetada no frame do dab (tangente ×
//! bitangente × normal), com rake opcional"* — o **View Plane** do Blender. Este
//! módulo constrói o outro mapeamento da mesma lista, o **3D**: o padrão é uma
//! função pura da POSIÇÃO do vértice, e não há frame nenhum.
//!
//! Não é preferência, é o que a lei do traço desta casa exige. Nosso envelope é
//! `accum ← max(accum, w)` sobre a lista de dabs, e com o espaçamento em `0,15·r`
//! um vértice cai sob **dezenas** de dabs. Num mapeamento projetado cada um deles
//! amostraria o padrão num `(u,v)` diferente, o `max` tomaria o maior de dezenas
//! de amostras, e o padrão **lavaria** até a sua envoltória superior — o pincel
//! ficaria mais forte, não texturizado. Num mapeamento 3D todos os dabs
//! concordam sobre o valor daquele vértice, então o `max` de valores iguais é o
//! valor: **o padrão sobrevive à lei intacto**, e as três propriedades do traço
//! (independência de espaçamento, idempotência, undo trivial) continuam valendo
//! sem uma linha a mais.
//!
//! ⚠️ **A posição é a CONGELADA no pen-down**, não a viva — senão o padrão
//! escorregaria enquanto o próprio traço move a superfície, e dois dabs
//! discordariam de novo. É a mesma frase que o `stroke.rs` já diz sobre a
//! distância no envelope, e aqui ela cai de graça porque o `pre` já está lá.
//!
//! ⚠️ **O padrão é colado ao ESPAÇO DO OBJETO, não à superfície.** Deformar muito
//! a malha faz a superfície deslizar por baixo do padrão — o comportamento do
//! mapeamento 3D do Blender, e é ele que também garante que girar o objeto na
//! cena não faz a pele nadar. A alternativa (colar ao índice do vértice) não
//! sobrevive a uma subdivisão.
//!
//! # OS DIRECIONAIS, e o que a medição mudou na nota que os previa
//!
//! Os seis primeiros padrões são **isotrópicos**: lêem igual de qualquer
//! direção, que é o que os deixa viver sem frame. Um padrão **direcional** — um
//! estrato, um risco, um tecido — tem uma direção de VARIAÇÃO, e a nota que
//! abriu este item dizia que ele *"traz o frame do dab de volta"*.
//!
//! ⚠️ **A sonda `tests/measure_directional_wash.rs` mediu isso e o número
//! reformulou o item.** Quem lava um padrão sob o envelope não é a
//! DIRECIONALIDADE — é o **RE-ANCORAMENTO**:
//!
//! | gesto · âncora | dabs | média (verdade = 0,500) | contraste |
//! |---|---|---|---|
//! | 1 dab (o controle) | 1 | 0,492 | **0,295** |
//! | traço 1,00 · **absoluta** | 27 | 0,490 | **0,285** |
//! | traço 1,00 · por-dab (o `04.1`) | 27 | 0,592 | **0,128** |
//!
//! Uma coordenada medida a partir do CENTRO DO DAB muda de fase a cada dab, e o
//! `max` sobre dezenas dela come **57% do contraste** e deixa o pincel 20% mais
//! forte — a lei 2 do módulo, agora com número. Uma coordenada **absoluta**
//! (o objeto) sobrevive ao envelope INTACTA: 0,285 contra 0,295 de um carimbo
//! único. ⇒ **o frame do dab é exatamente o que não se pode usar**, e um padrão
//! direcional é apenas um padrão cujo eixo o artista aponta.
//!
//! O eixo é autorado em dois ângulos e resolvido pelo [`AlphaFrame`]. **O eixo
//! aponta a direção do padrão**, e o que cada um faz com ela está no doc do
//! variant — o Strata EMPILHA ao longo dele, as faixas do Scratches se SUCEDEM
//! ao longo dele, e uma das duas famílias de fios do Weave CORRE ao longo dele.
//!
//! ⚠️ **Uma frase única e mais forte que essa foi escrita e é FALSA** — *"o eixo
//! é a direção em que o padrão varia"*. Ela vale para o Strata e para o
//! Scratches e quebra no Weave, que é uma trama: ela varia nas DUAS direções do
//! plano dela. Escrevê-la assim mesmo faria o doc mentir sobre um terço da
//! família, e o gate `turning_the_axis_turns_the_pattern` é o que afirma o que os
//! três de fato compartilham — que eles LEEM o frame.
//!
//! ⚠️ E não há imagem para carregar hoje — o mesmo fato que fez os matcaps serem
//! analíticos. Sintetizar uma textura para depois amostrá-la seria a mesma
//! fórmula avaliada duas vezes; uma imagem AUTORADA é outra wave, e o preço dela
//! está medido no handoff (o `Brush` é `Copy` em vinte arquivos).
//!
//! # HR-5
//!
//! Zero transcendental: hash inteiro, `floor`, `sqrt` (instrução de hardware) e
//! aritmética. Esta crate é `libm`-free e continua sendo — ver [`frac`], que
//! existe porque `%` **não** é uma instrução.

/// **EM QUE DIREÇÃO** — ver [`frame`].
#[path = "alpha_frame.rs"]
mod frame;
pub use frame::{AlphaFrame, AlphaStencil, MAX_AXIS_ELEV_DEG};

/// **OS PIXELS QUE UM PADRÃO AUTORADO CARREGA** — ver [`image`].
#[path = "alpha_image.rs"]
mod image;
pub use image::AlphaImage;

/// **QUE TAMANHO ESTE MODELO COMPORTA** — ver [`scale`].
#[path = "alpha_scale.rs"]
mod scale;
pub use scale::{
    DEFAULT_ALPHA_SCALE, MAX_ALPHA_SCALE, MIN_ALPHA_SCALE, recommended_scale, sampled_edge,
};

/// **OS NOVE PADRÕES**, na ordem em que a UI os lista: seis isotrópicos e três
/// **direcionais**, que leem o [`AlphaFrame`].
///
/// Cada um é nomeado pelo que ele PARECE, não pelo uso: o mesmo Worley que faz
/// escama faz casco de tartaruga, e um nome de intenção envelheceria no primeiro
/// artista que o usasse para outra coisa.
///
/// ⚠️ **Os direcionais vêm por ÚLTIMO, e a ordem é load-bearing na UI:** o painel
/// desloca a seleção de um (a primeira opção dele é *nenhum*), então um variant
/// no MEIO moveria o índice de todo padrão depois dele. Apender é o que mantém o
/// chip que o artista aprendeu no lugar onde ele estava.
/// ⚠️ **`Copy` MORREU aqui, e o preço foi MEDIDO antes de a decisão ser tomada:**
/// o compilador conta **3 derives** (este, o `Sculpt3dUi` e os dois que o contêm)
/// mais cinco `.clone()` num arquivo de teste — contra a estimativa herdada de
/// *"~20 arquivos"*, que era o que arquivava esta saída. Ver a W17 do
/// `06.1-Waves-riscos-e-alvos.md`.
///
/// ⚠️ **E `PartialEq` é MANUAL, com um motivo de custo:** duas imagens são o
/// mesmo padrão quando são a MESMA imagem ([`std::sync::Arc::ptr_eq`]). A
/// derivada compararia **pixel a pixel**, e a chave do cache do swatch do painel
/// é comparada a cada quadro — um megabyte de `memcmp` por frame para responder
/// *"o artista mexeu?"*.
#[derive(Clone, Debug)]
pub enum Alpha {
    /// fBm de ruído de valor, três oitavas. Irregularidade geral — rocha, massa.
    Noise,
    /// Pontos redondos isolados. Pele.
    Pores,
    /// Domos preenchendo as células, com sulco na fronteira. Réptil, casco.
    Scales,
    /// Só as fronteiras das células, finas. Terra seca, casca, esmalte trincado.
    Cracks,
    /// Mosqueado fino de alto contraste. O *chatter* de uma superfície trabalhada.
    Grain,
    /// fBm dobrado: cristas afiadas em vez de ondas. Ruga, estrato, casca de árvore.
    Ridges,
    /// **DIRECIONAL** — camadas paralelas empilhadas ao longo do eixo, com a
    /// fronteira ondulada. Rocha sedimentar, veio de madeira, estratificação.
    Strata,
    /// **DIRECIONAL** — riscos finos e ESPARSOS, correndo perpendiculares ao
    /// eixo. Metal escovado, marca de garra, desgaste.
    Scratches,
    /// **DIRECIONAL** — a trama de duas famílias de fios que passam uma por cima
    /// da outra; uma delas corre AO LONGO do eixo. Tecido, cesta, malha.
    Weave,
    /// **A IMAGEM AUTORADA** — os pixels que o artista apontou, projetados ao
    /// longo do eixo e ladrilhados. Ver [`AlphaImage`].
    ///
    /// ⚠️ **Os pixels moram AQUI, e é a wave inteira numa linha:** a escolha e a
    /// imagem são o MESMO valor, então `Image` **sem** imagem é inexprimível. As
    /// outras duas saídas (um id numa tabela · a imagem como parâmetro do dab)
    /// deixam esse estado nascer, e ele significa *"liso"* em silêncio.
    ///
    /// ⚠️ **E ela fica FORA do [`Self::ALL`]**, de propósito: aquela lista é o
    /// que a UI oferece como CHIPS, e um chip é um nome. Uma imagem não é um nome
    /// — é uma coisa para a qual se aponta —, então o gesto que a arma é um
    /// BOTÃO. Pôr um chip "Image" na fileira criaria exatamente o estado que o
    /// parágrafo acima torna impossível.
    Image(std::sync::Arc<AlphaImage>),
}

/// ⚠️ Ver o `derive` acima: identidade para a imagem, discriminante para os nove.
impl PartialEq for Alpha {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Image(a), Self::Image(b)) => std::sync::Arc::ptr_eq(a, b),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Eq for Alpha {}

impl Alpha {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 9] = [
        Self::Noise,
        Self::Pores,
        Self::Scales,
        Self::Cracks,
        Self::Grain,
        Self::Ridges,
        Self::Strata,
        Self::Scratches,
        Self::Weave,
    ];

    /// O nome que a UI mostra.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Noise => "Noise",
            Self::Pores => "Pores",
            Self::Scales => "Scales",
            Self::Cracks => "Cracks",
            Self::Grain => "Grain",
            Self::Ridges => "Ridges",
            Self::Strata => "Strata",
            Self::Scratches => "Scratches",
            Self::Weave => "Weave",
            Self::Image(_) => "Image",
        }
    }

    /// **Este padrão tem um "para onde"?** — a porta única do eixo.
    ///
    /// Os seis isotrópicos leem igual de qualquer direção, e é isso que os deixa
    /// viver sem frame nenhum. Os três direcionais têm uma direção de VARIAÇÃO, e
    /// é ela que o artista aponta.
    ///
    /// ⚠️ **Porta e não um `matches!` no sítio de uso:** o painel pergunta para
    /// OFERECER as duas pistas de eixo, o motor pergunta para decidir se projeta
    /// no frame, e o gate pergunta para provar a byte-identidade dos isotrópicos.
    /// Três cópias divergiriam em dois controles que aparecem e não fazem nada —
    /// o mesmo mecanismo do `Verb::uses_plane`.
    /// **Este padrão é um CARIMBO?** — a porta única da colocação.
    ///
    /// ⚠️ Ela existe porque *ter um eixo* e *ter uma posição* são perguntas
    /// diferentes: os três procedurais direcionais apontam para um lado e são
    /// homogêneos ao longo dele, e só a imagem tem um *onde*. Um `matches!` no
    /// sítio de uso viraria três cópias — o painel (para oferecer as duas rows),
    /// o `alpha_frame` (para decidir se o deslocamento alcança o motor) e o gate.
    #[must_use]
    pub fn is_image(&self) -> bool {
        matches!(self, Self::Image(_))
    }

    #[must_use]
    pub fn is_directional(&self) -> bool {
        matches!(
            self,
            Self::Strata | Self::Scratches | Self::Weave | Self::Image(_)
        )
    }

    /// **A frequência RELATIVA do padrão**, em células por unidade de escala.
    ///
    /// ⚠️ Ela existe porque a pista de escala é UMA e os padrões têm densidades
    /// naturais diferentes: o Grain é um chuvisco e as Scales são placas. Sem
    /// este multiplicador o artista teria de re-afinar a escala a cada troca de
    /// padrão, e a pista significaria uma coisa diferente por chip — que é a
    /// definição de um controle que não se aprende.
    fn frequency(&self) -> f32 {
        match self {
            // A oitava base do fBm já é a mais grossa das três.
            Self::Noise | Self::Ridges => 1.0,
            Self::Pores | Self::Scales | Self::Cracks => 1.0,
            // Um chuvisco: três vezes mais fino que os demais na mesma escala.
            Self::Grain => 3.0,
            // Uma camada e um fio medem a escala inteira: a `tri` que os desenha
            // já ocupa a célula toda, ao contrário do Worley, cuja semente é um
            // ponto no volume.
            Self::Strata | Self::Weave => 1.0,
            // Um risco é FINO na travessia e o que a escala nomeia é a distância
            // entre faixas — o mesmo raciocínio do Grain.
            Self::Scratches => 2.0,
            // Um ladrilho da imagem MEDE a escala inteira: a pista continua
            // dizendo *"que tamanho tem uma feature"*, e para uma imagem a
            // feature é ela.
            Self::Image(_) => 1.0,
        }
    }

    /// **O PESO em `[0, 1]`** no ponto `p` (espaço do objeto), para um tamanho de
    /// feature `scale` em unidades de objeto.
    ///
    /// **Porta única** — o motor, o gate e a sonda perguntam a esta função.
    ///
    /// ⚠️ **A peneira de não-finito é a mesma do [`crate::Falloff::weight`]**, e
    /// pelo mesmo motivo: um `NaN` que escorre daqui vira peso `NaN` num vértice,
    /// e uma malha inteira sai `NaN` a partir de um ponto ruim em algum lugar.
    #[must_use]
    pub fn weight_at(&self, p: [f32; 3], scale: f32, frame: &AlphaFrame) -> f32 {
        if !p[0].is_finite() || !p[1].is_finite() || !p[2].is_finite() {
            return 0.0;
        }
        // ⚠️ **Só o PISO, e o teto saiu de propósito.** O piso é guarda de
        // RECURSO: `k = frequency / s` explode com um `s` zerado ou negativo, e
        // isso não é opinião de ninguém. O teto de `MAX_ALPHA_SCALE` é a faixa
        // confortável do SLIDER — uma afirmação sobre unidades de OBJETO —, e um
        // ESTÊNCIL mede em fração da VISTA: a régua da vista de um modelo
        // enquadrado vale 2-3 unidades de objeto, então um carimbo de um quarto
        // de tela resolve em 0,5-0,75 e era **saturado** aqui, ficando do mesmo
        // tamanho em toda a metade de cima da pista. *Um teto de UI aplicado no
        // kernel é o mesmo número respondendo a duas perguntas.* A row segue
        // clampando o valor AUTORADO, que é onde essa pergunta mora.
        let s = scale.max(MIN_ALPHA_SCALE);
        let k = self.frequency() / s;
        // ⚠️ **Os seis isotrópicos NÃO passam pelo frame, e é isso que os deixa
        // BYTE-IDÊNTICOS ao mundo pré-direcional.** Eles leem igual de qualquer
        // direção por construção, então projetá-los mudaria os bits sem mudar o
        // que descrevem — e esta wave inteira se apoia em nenhum deles se mover.
        // A pergunta é feita à porta ([`Self::is_directional`]), nunca a uma
        // lista de nomes escrita aqui.
        let q = if self.is_directional() {
            let f = frame.project(p);
            [f[0] * k, f[1] * k, f[2] * k]
        } else {
            [p[0] * k, p[1] * k, p[2] * k]
        };

        let w = match self {
            Self::Noise => {
                // Centrado em 0,5 e esticado: o fBm cru se aperta em torno da
                // média e sairia como um cinza uniforme.
                contrast(fbm(q), 0.18, 0.82)
            }
            Self::Ridges => {
                // O dobramento (`1 − |2f − 1|`) troca ondas por CRISTAS: onde o
                // fBm cruza a média nasce um vinco de derivada descontínua, que
                // é exatamente o que uma ruga é.
                let f = fbm(q);
                let folded = 1.0 - (2.0f32.mul_add(f, -1.0)).abs();
                contrast(folded, 0.45, 0.98)
            }
            Self::Grain => contrast(value_noise(q), 0.42, 0.62),
            Self::Pores => {
                // Um disco em torno de cada ponto-semente, e nada entre eles.
                let (f1, _) = worley(q);
                1.0 - smoothstep(PORE_CORE, PORE_EDGE, f1)
            }
            Self::Scales => {
                // `f2 − f1` é grande no miolo da célula e vai a ZERO na fronteira
                // dela — é a distância à parede, de graça. Isto é o domo.
                let (f1, f2) = worley(q);
                smoothstep(0.0, SCALE_RIM, f2 - f1)
            }
            Self::Cracks => {
                // O complemento exato do Scales, com a parede muito mais fina:
                // uma trinca é a fronteira, não o interior.
                let (f1, f2) = worley(q);
                1.0 - smoothstep(0.0, CRACK_WIDTH, f2 - f1)
            }
            Self::Strata => {
                // A coordenada ao longo do EIXO é o índice da camada, e o fBm a
                // ONDULA: é isso que faz uma camada engrossar e afinar em vez de
                // sair um código de barras. ⚠️ O deslocamento é em CÉLULAS (a
                // régua de `q`), então ele acompanha a escala — em unidades de
                // objeto ele seria um número absoluto que só serve num tamanho
                // de modelo, que é exatamente o defeito que o seed de escala
                // existe para não ter.
                let wob = fbm([
                    q[0] * STRATA_WOBBLE_FREQ,
                    q[1] * STRATA_WOBBLE_FREQ,
                    q[2] * STRATA_WOBBLE_FREQ,
                ]);
                contrast(tri(q[2] + STRATA_WOBBLE * (wob - 0.5)), 0.15, 0.85)
            }
            Self::Scratches => {
                // A FAIXA: o padrão varia ao longo do eixo, então cada faixa é
                // indexada por `q[2]` e o risco corre atravessado, em `q[0]`.
                let lane = q[2].floor();
                let h = hash3(lane as i32, SCRATCH_SALT, 0);
                if unit(h) >= SCRATCH_DENSITY {
                    // ⚠️ **Esparso de propósito:** um risco em TODA faixa é uma
                    // grade de listras, não arranhões. O Pores recusa pelo mesmo
                    // motivo — os dois são padrões de EVENTO, não de campo.
                    0.0
                } else {
                    // A fase e o comprimento saem do MESMO hash por rotação, o
                    // que já é a economia que o `worley` faz nas três sementes:
                    // um segundo avalanche por faixa custaria o dobro para
                    // descorrelacionar o que o olho não distingue.
                    let phase = unit(h.rotate_left(9));
                    let along = frac(q[0] * SCRATCH_STRETCH + phase);
                    let len = SCRATCH_MIN_LEN + unit(h.rotate_left(18)) * (1.0 - SCRATCH_MIN_LEN);
                    // Ao longo: cheio até `len`, desvanecendo na ponta — um
                    // risco termina em ponta, não em parede.
                    let run = 1.0 - smoothstep(len - SCRATCH_TIP, len, along);
                    // Através: o perfil fino, centrado na faixa.
                    let across =
                        1.0 - smoothstep(0.0, SCRATCH_WIDTH, (q[2] - lane - 0.5).abs() * 2.0);
                    run * across
                }
            }
            Self::Image(img) => {
                // ⚠️ **O PLANO é o perpendicular ao EIXO** (`q[0]`, `q[1]` são a
                // tangente e a bitangente; `q[2]` corre ao longo dele — a mesma
                // convenção que o Strata empilha). Ou seja: o artista APONTA o
                // eixo para a superfície e a imagem é projetada por ali. Usar o
                // eixo como uma das coordenadas do plano faria a imagem
                // escorregar quando ele girasse, que é o oposto de apontar.
                img.sample(q[0], q[1])
            }
            Self::Weave => {
                // Duas famílias de fios, no plano `t`–`n`: uma corre AO LONGO do
                // eixo (indexada por `q[0]`), a outra atravessada (indexada por
                // `q[2]`, a coordenada do eixo). O perfil de cada fio é a `tri` —
                // cheio no meio, zero na borda.
                //
                // ⚠️ **O plano é `t`–`n` e NÃO `t`–`b`, e a diferença é o que o
                // artista vê no primeiro traço.** Com `b` a trama vive no plano
                // perpendicular ao eixo; no eixo de fábrica (+Y) esse plano é o
                // do CHÃO, visto de perfil por uma câmera em +Z — e uma trama
                // vista de perfil é um conjunto de listras. Com `n` uma das
                // famílias corre ao longo do eixo, a trama fica de FRENTE no
                // default, e a leitura *"um fio corre na direção que você
                // apontou"* vale para os três padrões desta família.
                let along = tri(q[0]);
                let across = tri(q[2]);
                // ⚠️ **O XADREZ é o que faz disto uma TRAMA e não uma grade.**
                // Quem passa por cima alterna com a paridade das duas bandas,
                // que é literalmente como um tecido é tecido. Somados, o
                // cruzamento sairia mais ALTO que os fios — o oposto de uma
                // trama, onde o cruzamento é onde um fio MERGULHA.
                let over = ((q[0].floor() as i32 + q[2].floor() as i32) & 1) == 0;
                let (top, under) = if over {
                    (along, across)
                } else {
                    (across, along)
                };
                contrast(top.max(under * WEAVE_UNDER), 0.1, 0.9)
            }
        };

        if w.is_finite() {
            w.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

/// Raio do miolo cheio de um poro, em células.
///
/// ⚠️ **Os dois números saíram de MEDIÇÃO e a primeira escolha estava errada por
/// quase 4×** (`0,10` / `0,30`): a sonda de cobertura deu **média 0,039, com
/// 1,1% dos pontos acima de 0,9** — um pincel que o artista leria como
/// *quebrado*, não como texturizado. A causa é geométrica e vale para todo
/// padrão daqui: as sementes são pontos no **VOLUME**, e a superfície corta esse
/// volume — a fração que ela vê é a fração VOLUMÉTRICA, `(4/3)π r³`, que cai com
/// o CUBO. Um raio que pareceria generoso num desenho 2D é diminuto em 3D.
const PORE_CORE: f32 = 0.22;
/// Raio onde o poro morre, em células.
const PORE_EDGE: f32 = 0.42;
/// Largura do sulco entre duas escamas, na régua de `f2 − f1`.
const SCALE_RIM: f32 = 0.30;
/// Largura de uma trinca, na mesma régua. Bem menor: é o que a faz ser um traço.
const CRACK_WIDTH: f32 = 0.08;

/// Quanto a fronteira de uma camada de estrato ONDULA, em células.
///
/// ⚠️ **Meia célula, e o teto é geométrico, não de gosto:** em `1,0` a ondulação
/// vale uma camada inteira e as fronteiras se CRUZAM — o padrão deixa de ser
/// estratificado e vira um fBm com listras. Em `0,5` a camada engrossa e afina
/// até o dobro sem nunca encostar na vizinha.
const STRATA_WOBBLE: f32 = 0.5;
/// A ondulação é de baixa frequência de propósito: ela descreve a DOBRA do
/// terreno, e no mesmo passo da camada ela seria ruído sobre ruído.
const STRATA_WOBBLE_FREQ: f32 = 0.35;

/// Que fração das faixas carrega um risco.
///
/// ⚠️ **MEDIDA, e a primeira escolha errou por 4×** (`measure_alpha_coverage`).
/// Com `0,35` de densidade e `0,35` de largura a cobertura saiu em **0,034, com
/// 1,1% dos pontos acima de 0,9** — os números *exatos* que este módulo já
/// registrou como o pincel que o artista lê como **quebrado** (ver
/// [`PORE_CORE`], onde a primeira calibração dos poros deu 0,039 e 1,1%).
///
/// ⚠️ **Mas a CAUSA é outra, e confundi-las levaria à correção errada.** Lá o
/// erro era volumétrico — sementes são pontos no volume, e a fração que a
/// superfície vê cai com o CUBO. Aqui não há volume: o padrão é função de duas
/// coordenadas e constante na terceira, então a cobertura é simplesmente a área
/// que ele ocupa, e o produto `densidade × perfil-atravessado × perfil-ao-longo`
/// **prevê 0,0337 contra os 0,034 medidos**. O que estava errado eram os três
/// fatores, não a geometria.
const SCRATCH_DENSITY: f32 = 0.55;
/// O tempero do hash da faixa — só para os riscos não caírem sobre as sementes
/// do Worley, que usam o mesmo [`hash3`] com `y = 0`.
const SCRATCH_SALT: i32 = 0x5C_A7;
/// Quantas células um risco percorre antes de o padrão repetir. Ele é > 1
/// porque um risco é LONGO: a razão comprimento÷largura é o que o olho lê como
/// arranhão.
const SCRATCH_STRETCH: f32 = 0.18;
/// O menor comprimento de risco, em fração do período. Ver [`SCRATCH_DENSITY`].
const SCRATCH_MIN_LEN: f32 = 0.5;
/// Quanto da ponta desvanece. ⚠️ **Menor que [`SCRATCH_MIN_LEN`]**, senão o
/// desvanecimento começaria antes do risco e ele nasceria já apagado.
const SCRATCH_TIP: f32 = 0.25;
/// A largura de um risco, em fração da faixa. Ver [`SCRATCH_DENSITY`] — é o
/// fator que mais pesou na re-calibração, porque ele entra na cobertura pela
/// integral do ombro, não pelo valor de pico.
const SCRATCH_WIDTH: f32 = 0.6;

/// Quanto do fio que passa por BAIXO ainda aparece.
///
/// ⚠️ Não é zero, e a diferença é o que faz a trama ter profundidade: com `0` o
/// fio de baixo some e a trama vira um tijolo; com `1` os dois empatam e o
/// xadrez fica invisível. É o mesmo raciocínio do sulco do Scales.
const WEAVE_UNDER: f32 = 0.55;

/// A parte fracionária de `x`.
///
/// ⚠️ **`%` e `f32::rem_euclid` são chamadas a `fmod` da LIBM, não instruções** —
/// medido pela `line/Painter` no doc 28 §5.43 (2,51 ns contra 0,54) —, e esta
/// crate é `libm`-free. O `floor` é instrução de hardware, e é o mesmo caminho
/// que o [`value_noise`] já toma.
fn frac(x: f32) -> f32 {
    x - x.floor()
}

/// Onda triangular em `[0, 1]`: **1 no meio da célula, 0 nas fronteiras**.
///
/// É o perfil de uma banda — uma camada de estrato, um fio de trama —, e ela é
/// simétrica de propósito: um perfil assimétrico daria ao padrão um SENTIDO, e o
/// eixo já diz a direção.
fn tri(x: f32) -> f32 {
    1.0 - 2.0 * (frac(x) - 0.5).abs()
}

/// `smoothstep` — o degrau C¹ que todo remapeamento daqui usa.
fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    if e1 <= e0 {
        return if x < e0 { 0.0 } else { 1.0 };
    }
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * 2.0f32.mul_add(-t, 3.0)
}

/// Estica a faixa `[lo, hi]` para `[0, 1]`, com ombro C¹.
///
/// É o `smoothstep` com outro nome, e o nome é o ponto: aqui ele não é um degrau
/// espacial, é o CONTRASTE de um padrão. Os dois usos merecem duas leituras.
fn contrast(v: f32, lo: f32, hi: f32) -> f32 {
    smoothstep(lo, hi, v)
}

/// Hash inteiro de uma célula da grade → `u32` bem misturado.
///
/// ⚠️ **Os três eixos entram por multiplicadores DIFERENTES antes do `xor`**: com
/// o mesmo multiplicador, `(a, b, c)` e `(b, a, c)` colidiriam, e o padrão sairia
/// espelhado em torno da diagonal — visível como uma simetria que ninguém
/// autorou. O avalanche final é o do `splitmix`.
fn hash3(x: i32, y: i32, z: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ (z as u32).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2c1b_3c6d);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297a_2d39);
    h ^= h >> 15;
    h
}

/// `u32` → `[0, 1)`, pelos 24 bits ALTOS.
///
/// ⚠️ Os altos e não os baixos: um multiplicador ímpar preserva perfeitamente os
/// bits baixos de uma soma, então `hash & 0xFF` de células vizinhas anda em
/// passos regulares — o padrão sairia listrado.
fn unit(h: u32) -> f32 {
    (h >> 8) as f32 * (1.0 / 16_777_216.0)
}

/// Ruído de valor em `[0, 1]`: hash nos vértices da grade, trilinear entre eles,
/// com a fade cúbica que tira a assinatura da grade das derivadas.
fn value_noise(p: [f32; 3]) -> f32 {
    let f = [p[0].floor(), p[1].floor(), p[2].floor()];
    let i = [f[0] as i32, f[1] as i32, f[2] as i32];
    let mut u = [0.0f32; 3];
    for a in 0..3 {
        let t = (p[a] - f[a]).clamp(0.0, 1.0);
        u[a] = t * t * 2.0f32.mul_add(-t, 3.0);
    }
    let c = |dx: i32, dy: i32, dz: i32| unit(hash3(i[0] + dx, i[1] + dy, i[2] + dz));
    let lerp = |a: f32, b: f32, t: f32| (b - a).mul_add(t, a);

    let x00 = lerp(c(0, 0, 0), c(1, 0, 0), u[0]);
    let x10 = lerp(c(0, 1, 0), c(1, 1, 0), u[0]);
    let x01 = lerp(c(0, 0, 1), c(1, 0, 1), u[0]);
    let x11 = lerp(c(0, 1, 1), c(1, 1, 1), u[0]);
    let y0 = lerp(x00, x10, u[1]);
    let y1 = lerp(x01, x11, u[1]);
    lerp(y0, y1, u[2])
}

/// Três oitavas de [`value_noise`], normalizadas para `[0, 1]`.
///
/// ⚠️ **As razões de frequência são 2,03 e 4,07 e não 2 e 4**, e é essa metade
/// que carrega o peso: com razões inteiras as três oitavas caem sobre os MESMOS
/// vértices de grade em `p` inteiro, e a rede de pontos onde as três concordam
/// vira uma trama visível com o período da oitava mais grossa. Os deslocamentos
/// são a segunda camada — eles tiram as três da origem comum — e ficam por serem
/// de graça, não porque sozinhos bastariam.
fn fbm(p: [f32; 3]) -> f32 {
    const OCTAVES: [([f32; 3], f32, f32); 3] = [
        ([0.0, 0.0, 0.0], 1.0, 0.571_428_6),
        ([19.7, 7.3, 31.1], 2.03, 0.285_714_3),
        ([51.3, 27.9, 11.5], 4.07, 0.142_857_15),
    ];
    let mut acc = 0.0;
    for (off, freq, weight) in OCTAVES {
        let q = [
            p[0].mul_add(freq, off[0]),
            p[1].mul_add(freq, off[1]),
            p[2].mul_add(freq, off[2]),
        ];
        acc = value_noise(q).mul_add(weight, acc);
    }
    acc
}

/// As distâncias aos dois pontos-semente mais próximos, em unidades de célula.
///
/// Uma semente por célula, jogada num canto aleatório dela. ⚠️ Varrer as **27**
/// células vizinhas é o que garante que a mais próxima está entre elas: uma
/// semente pode encostar na parede da sua célula, então a vencedora pode morar a
/// uma célula de distância em cada eixo.
fn worley(p: [f32; 3]) -> (f32, f32) {
    let base = [p[0].floor(), p[1].floor(), p[2].floor()];
    let bi = [base[0] as i32, base[1] as i32, base[2] as i32];
    let (mut f1, mut f2) = (f32::INFINITY, f32::INFINITY);
    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cell = [bi[0] + dx, bi[1] + dy, bi[2] + dz];
                let h = hash3(cell[0], cell[1], cell[2]);
                // Os três deslocamentos saem de ROTAÇÕES do mesmo hash: um
                // segundo hash por eixo custaria três avalanches por célula, e
                // rotacionar já descorrelaciona o suficiente para o olho — o
                // gate mede a distribuição, não a minha palavra.
                let seed = [
                    cell[0] as f32 - base[0] + unit(h),
                    cell[1] as f32 - base[1] + unit(h.rotate_left(11)),
                    cell[2] as f32 - base[2] + unit(h.rotate_left(22)),
                ];
                let d = [
                    seed[0] - (p[0] - base[0]),
                    seed[1] - (p[1] - base[1]),
                    seed[2] - (p[2] - base[2]),
                ];
                let d2 = d[2].mul_add(d[2], d[0].mul_add(d[0], d[1] * d[1]));
                if d2 < f1 {
                    f2 = f1;
                    f1 = d2;
                } else if d2 < f2 {
                    f2 = d2;
                }
            }
        }
    }
    (f1.sqrt(), f2.sqrt())
}

#[cfg(test)]
#[path = "alpha_tests.rs"]
mod tests;
