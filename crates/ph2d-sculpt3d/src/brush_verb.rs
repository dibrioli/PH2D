//! **O CATÁLOGO** — os dezassete verbos e o que cada um significa, cortados por
//! ASSUNTO do [`super`].
//!
//! O pai responde *que pincel está na mão* (raio, força, curva, os knobs); aqui
//! mora *que OPERAÇÃO ele executa*, mais as constantes de magnitude que cada
//! família lê. As duas perguntas crescem por razões diferentes: o catálogo cresce
//! quando entra uma ferramenta, o pincel quando entra um controle.

use super::*;

/// O que o pincel FAZ. Ver `docs/3D/04.1` para a família de cada um.
///
/// ⚠️ **`Layer` não está aqui, e a PREMISSA que o mantinha fora MORREU**
/// (conferido em 2026-08-12). A nota antiga dizia, e estava certa no dia em que
/// foi escrita: *"sob a lei do traço (`accum` é um ENVELOPE em `[0,1]`, nunca
/// uma soma) o Draw já é limitado a um `reach` por traço — que é exatamente o
/// que o Layer do ZBrush existe para garantir; os dois colapsam"*.
///
/// **Não há mais envelope.** A wave da paridade (2026-08-11) fez o
/// [`crate::Grip::Stamp`] **COMPOR** — cada dab soma o próprio incremento sobre
/// o que o anterior deixou, que é a estrutura do kernel da referência —, e o
/// doc da [`crate::GripLaw::additive`] diz a frase inteira: *"nenhum grip é mais
/// um envelope"*. Com isso o Draw deixou de saturar num `reach`:
///
/// - **Accumulate ON** (`from_live`) — o vértice e o centro sobem juntos, o
///   pincel **não se esgota**, e um traço demorado empilha sem teto.
/// - **Accumulate OFF** — o vértice atravessa `dist >= 1` e **sai da pegada
///   sozinho**, o que auto-limita por GEOMETRIA. ⚠️ E isso não é a mesma coisa
///   que o Layer: o teto dele é uma **ALTURA escolhida**, o deste é *"o vértice
///   andou mais que o raio"* — um número que muda quando o artista muda o raio.
///
/// ⇒ **os dois deixaram de colapsar, e o Layer é hoje uma ferramenta distinta**
/// (altura constante, saturante, persistente por vértice e **apagável**). Ele é
/// a wave W8 do [`plano dos modos`](../../../docs/3D/21_plano_modos_e_ferramentas.md),
/// e o que ele traz de novo é a ideia 3 do doc 20 §10: **estado persistente por
/// vértice** — que é também o que obriga um plano novo a entrar no
/// `ModelSnapshot` do undo **no mesmo commit** que o cria.
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
}

impl Verb {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 18] = [
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
            Self::Draw | Self::Inflate | Self::Clay | Self::Crease | Self::Blob | Self::Mask
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
            Self::Blob => "Blob",
            Self::Mask => "Mask",
            Self::Move => "Move / Grab",
            Self::SnakeHook => "Snake Hook",
            Self::Twist => "Twist",
            Self::LocalScale => "Local Scale",
            Self::ClayStrips => "Clay Strips",
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
    /// ⚠️ **E ela DELEGA à tabela dos modos** (`ref_mode`, 2026-08-12): a força
    /// de fábrica é o que a referência `S` declara, tool a tool, e não uma
    /// segunda cópia dela aqui. Antes disto o app shipava `0,5` em tudo — o
    /// **D3** do doc 20 —, e o número que sobrevive à delegação é o do Draw
    /// (`Brush.js:12`), o único que já batia.
    ///
    /// ⚠️ **Onde a fonte é SILENCIOSA o nosso número fica** (`0,5`): o
    /// `Drag`/`Twist`/`LocalScale` não declaram `_intensity` e o SculptGL não
    /// tem Sharpen. Um `unwrap_or` aqui é *"a referência não respondeu"*, nunca
    /// um valor inventado com a autoridade dela.
    #[must_use]
    pub fn default_strength(self) -> f32 {
        self.profile(crate::RefMode::S)
            .and_then(|p| p.strength)
            .unwrap_or(0.5)
    }

    /// **O Accumulate nasce ARMADO neste verbo?** — e a resposta é da
    /// referência, tool a tool, não uma afinação.
    ///
    /// ⚠️ **O pedido original do Enio dizia isto e eu li como descrição de UI:**
    /// *"Brush:Checkbox:Clay com accumulate **checado por padrão**"*. É o
    /// `Brush.js:16` — `this._accumulate = true` —, e a tool `Brush` do original
    /// é a nossa **Draw E Clay** (o `_clay` é um checkbox dela, ligado de
    /// fábrica). Nós shipávamos os dois **desarmados**, então o artista pegava o
    /// Clay e tinha outra ferramenta na mão.
    ///
    /// ⚠️ **E a família do PLANO é mais forte que um default:** o `Flatten.js`
    /// **não declara `_accumulate`**, e o kernel dele pergunta
    /// `this._accumulate === false` — que em `undefined` é FALSO. Ou seja
    /// `Flatten`/`Fill`/`Scrape` leem o vivo **sempre**, sem checkbox. Nós temos
    /// o interruptor, então o honesto é nascerem armados: é o comportamento
    /// que a referência não deixa desligar.
    ///
    /// Os que ficam DESARMADOS não são omissão — são os que a referência não
    /// arma: o `Smooth` e o `Mask` não têm o campo e não o leem, o `Pinch`, o
    /// `Crease` e os quatro grips de gesto tampouco.
    /// ⚠️ **E ela DELEGA à mesma tabela** que a força (`ref_mode`, 2026-08-12) —
    /// duas portas para *"o que a referência arma neste verbo?"* divergiriam na
    /// primeira wave que mexesse numa delas. O resultado é **byte-idêntico** ao
    /// `matches!` que ela substituiu, e há gate afirmando isso.
    /// **A LEI DE GRIP QUE GOVERNA ESTE VERBO** — a porta do produto.
    ///
    /// ⚠️ **[`crate::Grip::law`] responde outra pergunta:** *qual é a lei deste
    /// grip*. É o mesmo par do [`crate::RefMode::kernel`] / `kernel_for`, e pela
    /// mesma razão — um verbo pode ter uma referência que o grip não conhece.
    ///
    /// ⚠️ **O `from_live` do [`crate::Grip::Stamp`] é o Accumulate do
    /// SCULPTGL**, e a faixa não é dele. O `clay_strips.cc::calc_faces` chama
    /// `calc_local_positions(position_data.eval, …)` — a posição **VIVA**,
    /// sempre —, e o que o `accum` da referência escolhe é a fonte do PLANO
    /// (`sculpt.cc`, `!ss.cache->accum` ⇒ pen-down congelado).
    ///
    /// ⚠️ **E é a combinação que dá o auto-limite:** posição viva contra plano
    /// congelado faz o `z` do portão `z·(1−z)` **encolher** à medida que o barro
    /// sobe, até fechar no plano. Com as duas congeladas o `z` não se move e a
    /// faixa cresce para sempre — medido, `27 → 81 dabs` dava `1,52×` em vez de
    /// saturar.
    #[must_use]
    pub fn grip_law(self, accumulate: bool, carries_field: bool) -> crate::GripLaw {
        let mut law = self.grip().law(accumulate, carries_field);
        if self == Self::ClayStrips {
            law.from_live = true;
        }
        law
    }

    #[must_use]
    pub fn default_accumulate(self) -> bool {
        self.profile(crate::RefMode::S)
            .and_then(|p| p.accumulate)
            .unwrap_or(false)
    }
}

impl Verb {
    /// A **CURVA** com que um pincel deste verbo nasce — o D1 do estudo, e o
    /// único achado dele que o artista encontra sem tocar em nada.
    ///
    /// ⚠️ **Onde a fonte não tem resposta o nosso default fica** (o Sharpen, que
    /// o SculptGL não tem) — a mesma lei do `unwrap_or` da força.
    #[must_use]
    pub fn default_falloff(self) -> Falloff {
        self.profile(crate::RefMode::S)
            .and_then(|p| p.falloff)
            .unwrap_or(Falloff::Smooth)
    }

    /// O **RAIO** de fábrica em pixels de tela — o D4/E3.
    ///
    /// ⚠️ **A fração é da REFERÊNCIA e a base é NOSSA:** o perfil guarda
    /// `_radius / 50` (o Crease do original é `25`, metade; o Move é `150`,
    /// três vezes) e a base é o raio que o app oferece de fábrica. Guardar
    /// pixels no perfil congelaria a escolha do artista no dia em que a base
    /// mudasse — a mesma lei do `sss_scatter` e do próprio falloff.
    #[must_use]
    pub fn default_radius_px(self, base_px: f32) -> f32 {
        base_px
            * self
                .profile(crate::RefMode::S)
                .and_then(|p| p.radius_factor)
                .unwrap_or(1.0)
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

/// **Quanto do raio um dab da FAIXA desloca** — e não é o [`REACH_FRACTION`].
///
/// ⚠️ **`clay_strips.cc:327`, verbatim:**
///
/// ```text
/// const float3 offset = plane_normal * ss.cache->bstrength * ss.cache->radius;
/// ```
///
/// O deslocamento é `raio · força`, **fração 1,0** — o `0,1` do
/// [`REACH_FRACTION`] é o `deform = intensidade · raio · 0,1` do `Brush.js`, do
/// SculptGL, que **não tem esta ferramenta**. É a mesma classe do defeito que a
/// §7.21 curou na lei de kernel, uma camada abaixo.
///
/// ⚠️ **MEDIDO, e é o que a foto do Enio mostra:** com `0,1` a faixa era
/// **7,5× mais fraca por dab** que a referência, e por isso deixava estrias
/// macias que ACOMPANHAM a forma em vez das placas chatas que a CORTAM.
///
/// ⚠️ **É por causa dele que o `STRIP_DEPTH_GAIN` morreu:** aquele ganho existia
/// para preservar uma magnitude que era ela própria errada.
pub const STRIP_REACH_FRACTION: f32 = 1.0;

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

/// **Quanto o plano da FAIXA sobe acima da superfície**, em fração do raio.
///
/// ⚠️ **Este número decide se a faixa NIVELA ou COPIA o relevo, e é a coisa
/// inteira que o report do Enio de 2026-08-15 nomeia** (*"num vale a tool
/// correta tende a fechar o vale, na nossa tende a aumentá-lo"*).
///
/// O portão de profundidade da [`crate::Strip`] é `z·(1−z)`, que **sobe** de
/// `z = 0` (o plano) até `z = 0,5` e **desce** depois. Logo o depósito só cresce
/// com a profundidade enquanto o ponto está a menos de meio raio abaixo do
/// plano; passado o pico, quanto mais fundo **menos** barro. Como a superfície
/// em repouso fica a `lift` raios abaixo do plano, a faixa **enche** relevo até
/// `(0,5 − lift)` raios abaixo da média e **exagera** o que passa disso.
///
/// ⚠️ **O valor anterior era `0,5`, e ele punha a superfície EXATAMENTE no pico
/// — folga de enchimento ZERO.** Era o único valor da família em que nenhum vale
/// enche: tudo abaixo da média recebia menos que a média, então a passada
/// devolvia o relevo amplificado. ⚠️ E o defeito era MEU pela mesma via da
/// `tip_roundness`: eu derivei `0,5` de *"pôr o pico na superfície em repouso"*,
/// que é conveniência interna (o máximo depósito em chapa plana), e não da
/// propriedade que decide.
///
/// **MEDIDO** (`tests/measure_valley.rs`, vale de 0,40 de profundidade, pincel
/// `r = 0,8`, nove dabs; a varredura prendeu a magnitude pela FORÇA, para que
/// a única coisa a mover fosse a forma):
///
/// | lift | vale (Δ profundidade) | miolo ÷ aro numa CÚPULA |
/// |---|---|---|
/// | 0,10 | −0,212 | **0,009** (o miolo não recebe nada) |
/// | 0,18 | −0,121 | 0,393 |
/// | **0,25** | **−0,073** | **0,649** |
/// | 0,30 | −0,048 | 0,756 |
/// | 0,50 | **+0,027 (AUMENTA)** | 0,971 |
///
/// ⚠️ **A tabela foi medida com o `reach` do SculptGL, que era ele próprio
/// errado** (ver [`STRIP_REACH_FRACTION`]). Com o `raio · força` da referência o
/// vale FECHA muito mais forte — `0,4000 → 0,0406` em nove dabs a `r = 0,5` —,
/// mas a FORMA da tabela é a que decide o lift e ela não muda: o enchimento
/// segue monótono no lift, e o miolo segue a esvaziar-se quando ele baixa.
///
/// ⚠️ **As duas colunas puxam em sentidos OPOSTOS, e as duas são a mesma lei.**
/// Numa cúpula o miolo da pegada está acima do plano ajustado e o aro abaixo,
/// então nivelar É depositar mais no aro — o *"displaces vertices toward the
/// brush plane"* do kernel da referência. Baixar o lift nivela mais forte e
/// esvazia o miolo; subi-lo faz a banda ficar uniforme e parar de nivelar.
///
/// ⇒ **`0,25` é o MEIO da subida da parábola** (o plano em `z = 0`, o pico em
/// `z = 0,5`), o que dá à ferramenta folga igual para acrescentar num calombo e
/// para encher um buraco. É um marco da própria lei, não um gosto, e as duas
/// medições o confirmam longe de qualquer extremo.
///
/// ⚠️ **NÃO é citável da referência.** O `clay_strips.cc` lê `brush.plane_offset`
/// e o genérico do DNA é `0.0`; quem declara o valor por-tool é o
/// `BKE_brush_sculpt_reset`, **ausente do clone** — a mesma lacuna da §7.1 do
/// plano que bloqueou a W1 e o Draw Sharp. O número acima é NOSSO, e a tabela
/// ao lado é a razão dele.
///
/// ⚠️ **E ele existe porque sem ele a ferramenta nasce MORTA:** com o plano
/// rente, `z = 0` em toda parte e o portão fecha — quatro varreduras da suíte
/// (o alpha, o invert, os dois do aplicador) reprovaram exatamente assim, cada
/// uma dizendo *"dab inerte"*. O [`Verb::Clay`] resolve o mesmo problema pela
/// mesma via ([`CLAY_PLANE_FRACTION`]), e o `plane_offset` do artista SOMA a
/// este em vez de o substituir.
pub const STRIP_PLANE_FRACTION: f32 = 0.25;

/// Quanto o plano do **Clay** sobe acima da superfície, em fração do raio.
///
/// ⚠️ **É o literal `0.1` do `Brush.js:52`, e ele não é um knob no original.** O
/// nosso `plane_offset` é um controle do artista e SOMA a este — o default dele
/// (`0`) devolve a referência exata, e girá-lo levanta o plano a mais.
pub const CLAY_PLANE_FRACTION: f32 = 0.1;
