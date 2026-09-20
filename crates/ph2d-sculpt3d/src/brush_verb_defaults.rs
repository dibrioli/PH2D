//! **COM QUE NÚMEROS UM VERBO NASCE** — os defaults de fábrica, irmãos do
//! [`super`], que responde *o que um verbo É*.
//!
//! ⚠️ **A divisão não é de tamanho, é de assunto.** O arquivo pai é o
//! CATÁLOGO — quais verbos existem, o que cada um lê (o anel, o plano, a
//! máscara), com que gesto se pega, qual referência declara a lei dele. Aqui
//! ficam só as respostas à outra pergunta: *que números um pincel deste verbo
//! traz quando o artista o escolhe*. As duas crescem por motivos diferentes —
//! a de cima com cada verbo novo, esta com cada canal de fábrica novo — e é
//! isso que as separa.
//!
//! ⚠️ **Toda resposta aqui DELEGA à tabela dos modos** (`ref_mode`): a fábrica
//! é o que a referência `S` declara, tool a tool, e um `unwrap_or` significa
//! *"a fonte não respondeu"*, nunca um valor inventado com a autoridade dela.

use super::*;

impl Verb {
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
    ///
    /// ⚠️ **E o fallback é POR VERBO desde 2026-08-17, porque um deles foi
    /// MEDIDO:** a demão sai `0,7` por veredito de smoke do Enio (*"temos bom
    /// resultado para Layer com Strength 0.7, Hardness 0.4 e Auto Smooth 0.0"*),
    /// e o `0,5` continua sendo a resposta para todo verbo que ninguém julgou
    /// ainda. Os dois são *nossos*; o que os separa é ter um humano tido o
    /// produto na mão, e é essa distinção que o `unwrap_or` guarda.
    #[must_use]
    pub fn default_strength(self) -> f32 {
        self.profile(crate::RefMode::S)
            .and_then(|p| p.strength)
            .unwrap_or(match self {
                // ⭐ O pincel de plano: `0,7` é o perfil *aparar* do alvo, MEDIDO (espec
                // §14.2) — e a referência `S` não o tem, logo o fallback é a fonte.
                Self::Layer | Self::Plane => 0.7,
                _ => 0.5,
            })
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
    /// SCULPTGL**, e a faixa não é dele. O *Clay Strips* da referência mede as
    /// posições locais contra a posição **VIVA**, sempre —, e o que o
    /// *Accumulate* da referência escolhe é a fonte do PLANO (com o Accumulate
    /// desligado a referência lê a pose congelada do pen-down).
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
        // ⚠️ **A DEMÃO sobrescreve TRÊS colunas, e cada uma tem um motivo do
        // *Layer*:**
        //
        // * `coat` — a saturação assintótica, que é a lei dela;
        // * `unit_accum` **falso** — o `accum` da demão **é** o factor de
        //   deslocamento da referência, então o aplicador tem de o
        //   multiplicar (um alvo que já trouxesse o peso levaria a demão inteira
        //   no primeiro dab);
        // * `from_live` **falso sempre** — o *Layer* da referência mede as
        //   distâncias contra as posições ORIGINAIS do pen-down
        //   incondicionalmente, e o [`Self::accumulates`] já tira o interruptor
        //   da tela pelo mesmo
        //   motivo. Sem esta linha um documento salvo com o checkbox armado
        //   noutro verbo mudaria a lei da demão em silêncio.
        if self == Self::Layer {
            law.coat = true;
            law.unit_accum = false;
            law.from_live = false;
        }
        // ⭐⭐ **A PINTURA compõe como uma tinta** — ver [`crate::GripLaw::tint`],
        // onde está a álgebra que mostra que isto **é** o `Paint.js:129-131` e
        // não uma aproximação dele.
        //
        // ⚠️ **E `unit_accum` falso**, pela mesma razão que a demão: o `accum`
        // deste verbo **é** a fracção de mistura que o aplicador interpola. Com
        // ele verdadeiro o primeiro dab levaria a cor inteira e o pincel
        // deixaria de ter borda.
        if self == Self::Paint {
            law.tint = true;
            law.unit_accum = false;
            law.additive = false;
        }
        // ⭐⭐ **OS DOIS QUE LEEM O ANEL compõem por DAB, e não ao longo do
        // traço** — nenhuma das três leis de acumulação serve, e a que fica é o
        // `w` cru.
        //
        // ⚠️ **A razão é que o ALVO deles MUDA a cada dab:** ele é a
        // vizinhança VIVA, e ela é reescrita pelo dab anterior. Um acumulador
        // ao longo do traço (o `tint` da pintura, o `additive` da máscara)
        // guarda *quanto já se andou* para um alvo FIXO; aqui isso misturaria a
        // fracção de hoje com o destino de ontem. ⇒ cada dab puxa a cor viva
        // uma fracção `w` para a vizinhança de AGORA, que é o que uma
        // esfregadela é. Ver [`crate::stroke_cor`].
        if self.le_o_anel_de_cor() {
            law.tint = false;
            law.unit_accum = false;
            law.additive = false;
        }
        // ⛔⛔⛔ **A PROJECÇÃO mede da posição VIVA, SEMPRE** — espec §6.5 com
        // todas as letras: *«não há normalização por área, nem acumulador, nem
        // memória entre dabs — cada dab re-mede a distância a partir de onde o
        // vértice está AGORA»*.
        //
        // ⚠️⚠️ **Sem esta linha o `from_live` ficava preso ao interruptor
        // `Accumulate`** (é a única coluna que ele move num [`crate::Grip::Stamp`]),
        // e com ele desligado — que é o valor de fábrica **e** o das `16`
        // fixturas do oráculo — a queda passava a medir contra o `pre` do
        // pen-down. ⭐ **MEDIDO:** oito fixturas de SEIS dabs saltam de
        // `5,9e-2`–`2,6e-1` para **`8,9e-8`–`2,0e-7`**, ou seja de fora da barra
        // para dentro dela. *A partição um-dab/seis-dabs que este corpus
        // mostrava não era a composição do traço: era esta coluna.*
        //
        // ⚠️ **E ela NÃO colide com a fotografia do pen-down** que o report do
        // dono pagou: aquela congela **onde o dab aterra** (o PICK, um degrau
        // acima), esta diz **de onde a queda mede** dentro do dab. *Duas
        // perguntas com a mesma palavra.*
        if self == Self::SceneProject {
            law.from_live = true;
        }
        // ⭐⭐⭐ **O PINCEL DE PLANO também o prega, e por uma razão que é a mais
        // fina desta família: o interruptor de acumular responde a OUTRA PERGUNTA
        // neste verbo.**
        //
        // Nesta casa o `Accumulate` **é** o `from_live` do [`crate::Grip::Stamp`]
        // — *de onde a curva de queda mede a distância*. Na lei deste pincel
        // (espec §2.6) ele decide **de que superfície o PLANO é lido**, e mais
        // nada; o quadro local da §3 mede sempre da posição **VIVA**.
        //
        // ⇒ **duas perguntas com a mesma palavra**, e deixá-las na mesma coluna
        // faz o interruptor mover as duas ao mesmo tempo. Quem lê a pose do plano
        // é a porta [`Self::le_a_superficie_viva`]; aqui a coluna fica pregada.
        //
        // ⚠️⚠️ **MEDIDO, e o corpus apontou o dedo ao dab exacto:** com a coluna
        // presa ao interruptor, uma cadeia de **2** dabs (ou seja UM dab efectivo)
        // reproduzia o oráculo a `2,980e-08` e as de `4` e `8` dabs desviavam
        // `1,634e-01` e `3,022e-01`. *O primeiro dab estava certo e o segundo já
        // não* — que é a assinatura de uma coluna que governa a coisa errada
        // assim que a superfície se move.
        if self == Self::Plane {
            law.from_live = true;
        }
        // ⭐⭐⭐ **O PINCEL AFIADO prega a coluna do lado OPOSTO, e essa é a lei
        // inteira dele** (espec §2.1): a curva de queda mede a distância a
        // partir das posições do **pen-down** *sempre*, e isso **não depende**
        // do interruptor de acumular.
        //
        // ⚠️ **Sem esta linha ele herdaria o interruptor do carimbo** — e com o
        // Accumulate armado a queda passaria a medir da posição JÁ afundada, o
        // que faz o vinco **alargar enquanto aprofunda**: é exactamente o
        // comportamento do desenho comum, medido na ablação a `+55 %` de largura
        // (§8.1). ⇒ o interruptor sai da tela em [`Self::accumulates`], porque
        // aqui ele deixou de ter o que escolher.
        if self == Self::DrawSharp {
            law.from_live = false;
        }
        law
    }

    #[must_use]
    pub fn default_accumulate(self) -> bool {
        self.profile(crate::RefMode::S)
            .and_then(|p| p.accumulate)
            // ⭐⭐ **O pincel de plano nasce a ACUMULAR** — os perfis de achatar, encher, raspar e
            // aparar do alvo nascem todos assim (espec §14.2), e é a 2.ª alavanca medida (§14.7:
            // sem ele o plano não desce e o aparar pára em `0,287`).
            .unwrap_or(self == Self::Plane)
    }
}

impl Verb {
    /// A **CURVA** com que um pincel deste verbo nasce — o D1 do estudo, e o
    /// único achado dele que o artista encontra sem tocar em nada.
    ///
    /// ⚠️ **Onde a fonte não tem resposta o nosso default fica** (o Sharpen, que
    /// o SculptGL não tem) — a mesma lei do `unwrap_or` da força.
    ///
    /// ⚠️ **O MODO é parâmetro, e até 2026-08-16 ele era `S` cravado** — então
    /// escolher `b-mode` no painel deixava o pincel com a quártica do SculptGL,
    /// e a curva que o Blender de facto veste era **inalcançável pelo produto**
    /// por mais que o perfil dele a declarasse. É o mesmo defeito que o
    /// [`crate::RefMode`] existe para não ter: *um modo que governa a LEI do
    /// kernel e não governa o que o pincel nasce vestindo escolhe metade*.
    #[must_use]
    pub fn default_falloff(self, mode: crate::RefMode) -> Falloff {
        self.profile(mode)
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

    /// **SÓ AS FACES DE FRENTE** — o que o pincel deste verbo nasce vestindo.
    ///
    /// ⚠️ **Na referência isto é uma opção do artista, e nós aplicávamo-la
    /// incondicionalmente.** O facto, a varredura que mostra que **nada lá o
    /// LIGA**, e a lei que ele governa vivem numa casa só:
    /// [`crate::Brush::front_faces_only`]. O [`crate::KernelLaw`] continua
    /// dizendo QUAL lei o modo aplica; este flag diz **se** ela é aplicada.
    ///
    /// Medido em `crates/ph2d-sculpt3d/src/verb_layer_front_face_tests.rs`, cujo
    /// doc de módulo explica por que a fixture ali é uma ESFERA e não a grade
    /// plana: na grade o facing vale `1,0` em todo vértice e os dois mundos são
    /// byte-idênticos.
    ///
    /// ⚠️ **O preço de não ter o flag estava MEDIDO no report do Enio** (*"se
    /// aumentar hardness, Layer fica muito ruim"*), e o mecanismo é a etapa de
    /// dureza: com `hardness = h` toda distância normalizada `t < h`
    /// vira zero, e zero é onde a curva vale **um** — a `0,90` a curva satura em
    /// **90,5 %** do raio. Ali `shape = curva · alpha · facing · keep` colapsa
    /// em `facing`, o cosseno da CÂMERA, e a demão passa a vestir o perfil de
    /// quem olha em vez de subir à meta. Medido numa esfera, dab de raio `0,9`,
    /// hardness `0,90`, **1 dab** — a coluna é o deslocamento radial por `t`:
    ///
    /// ```text
    ///                t=0,1   t=0,3   t=0,5   t=0,7   t=0,9   borda/centro
    ///   com facing  0,0260  0,0248  0,0221  0,0179  0,0099       0,3793
    ///   sem facing  0,0262  0,0262  0,0262  0,0262  0,0205       0,7828
    /// ```
    ///
    /// O artista pede uma MESA de bordas duras e recebe uma RAMPA; a borda perde
    /// **2,1×**. A sonda é a `measure_layer_front_face`.
    ///
    /// ⚠️ **`false` é a regra — cada `true` é uma DÍVIDA nomeada contra a
    /// referência**, não uma escolha de desenho. Os defaults por-tool do Blender
    /// moram num `.blend` **binário** (o §7.0 do plano já o mediu, e é por isso
    /// que a W1 e o Draw Sharp são decisão de produto), então esta coluna é a
    /// mesma tabela que [`Self::default_falloff`] e [`Self::default_strength`]
    /// já mantêm: o mecanismo vem da fonte, o número vem de quem o mediu.
    ///
    /// ⚠️ **E a única exceção de hoje foi MEDIDA, não herdada:** com o flag
    /// desligado em toda parte, a suíte sangra em **três** gates e dois são do
    /// [`Self::ClayStrips`] — `the_strip_does_not_lay_clay_on_what_the_artist_cannot_see`
    /// mede a faixa a depositar **117,2 nas costas contra 97,8 de frente**, e
    /// aquele desenho foi smokado. Desligá-lo é fiel à referência **de fábrica**
    /// e re-decide uma wave vizinha dentro de um fix de outra: fica ligado, com
    /// o motivo escrito, e a mudança é wave própria com smoke próprio.
    #[must_use]
    pub const fn default_front_faces_only(self) -> bool {
        matches!(self, Self::ClayStrips)
    }

    /// A **DUREZA DO DAB** com que um pincel deste verbo nasce.
    ///
    /// ⚠️ **A referência é MUDA aqui, e a mudez é dos DOIS lados:** o
    /// `hardness` é uma etapa do Blender que o SculptGL não tem — logo o
    /// [`crate::ref_profiles`] nunca poderia respondê-lo —, e os defaults
    /// por-tool do Blender moram num `.blend` **binário** (o §7.0 do plano mediu
    /// isso). Então este número **não pode** vir de tabela nenhuma: ele vem de
    /// quem olhou para o barro.
    ///
    /// ⚠️ **`0` é o NEUTRO e é o default de todo verbo menos um** — com dureza
    /// zero a etapa de dureza da referência não corre, então o zero não é
    /// *"pouca dureza"*, é *"a etapa não corre"*.
    ///
    /// ⚠️ **O `0,4` da demão é veredito de SMOKE** (Enio, 2026-08-17: *"temos
    /// bom resultado para Layer com Strength 0.7, Hardness 0.4 e Auto Smooth
    /// 0.0"*), e ele carrega o mecanismo que o report anterior tinha nomeado:
    /// com `hardness = h` toda distância `t < h` vira zero, e ali a demão pousa
    /// na altura CHEIA — é o platô que faz uma demão parecer uma demão em vez de
    /// um domo. O que a tornava *"muito ruim"* com dureza alta era o `facing` do
    /// [`Self::default_front_faces_only`], já corrigido; o número que sobrou é o
    /// que o artista aprovou depois disso.
    #[must_use]
    pub const fn default_hardness(self) -> f32 {
        match self {
            Self::Layer => 0.4,
            // ⭐ O perfil *aparar* do alvo, MEDIDO (espec §14.2) — a 3.ª alavanca (§14.7).
            Self::Plane => 0.6,
            _ => 0.0,
        }
    }

    /// O **ALISAMENTO AUTOMÁTICO** com que um pincel deste verbo nasce.
    ///
    /// ⚠️ **Zero em toda parte, e isso é uma afirmação e não uma omissão:** o
    /// `auto_smooth` é o vizinho do `hardness` no RNA do Blender e nasce em `0`
    /// lá também; um verbo que quisesse outro número teria de o MEDIR, como a
    /// demão mediu a dureza dela. A função existe — em vez de o slot ler
    /// [`crate::Brush::default`] — porque é ela que torna a pergunta
    /// *"com que alisamento este verbo nasce?"* respondível **por verbo** no dia
    /// em que o primeiro deles precisar de outra resposta, sem que a tabela do
    /// [`crate::VerbSlot`] tenha de aprender um caso especial.
    #[must_use]
    pub const fn default_auto_smooth(self) -> f32 {
        let _ = self;
        0.0
    }
}

#[cfg(test)]
#[path = "brush_verb_defaults_tests.rs"]
mod tests;
