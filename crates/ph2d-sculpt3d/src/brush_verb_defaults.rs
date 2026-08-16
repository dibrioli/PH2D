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
        // ⚠️ **A DEMÃO sobrescreve TRÊS colunas, e cada uma tem um motivo do
        // `layer.cc`:**
        //
        // * `coat` — a saturação assintótica, que é a lei dela;
        // * `unit_accum` **falso** — o `accum` da demão **é** o
        //   `displacement_factor` da referência, então o aplicador tem de o
        //   multiplicar (um alvo que já trouxesse o peso levaria a demão inteira
        //   no primeiro dab);
        // * `from_live` **falso sempre** — o `calc_faces` do `layer.cc` mede as
        //   distâncias contra `orig_data.positions` incondicionalmente, e o
        //   [`Self::accumulates`] já tira o interruptor da tela pelo mesmo
        //   motivo. Sem esta linha um documento salvo com o checkbox armado
        //   noutro verbo mudaria a lei da demão em silêncio.
        if self == Self::Layer {
            law.coat = true;
            law.unit_accum = false;
            law.from_live = false;
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
