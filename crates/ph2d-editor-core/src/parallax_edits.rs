//! ⭐⭐⭐ **O VOCABULÁRIO DA PARALAXE** (plano 24, W7) — o instantâneo e a edição, num módulo abaixo
//! do [`crate::action_bus`] e do [`crate::screens`].
//!
//! # ⛔ Porque ele não está no `screens::hero`, com os irmãos mais velhos
//!
//! Pela mesma razão do [`crate::ray_edits`], do [`crate::tags_edits`] e do
//! [`crate::factory_edits`]: a catraca do DAG tolera a aresta `action_bus → screens` num **tecto**,
//! e escreve a cura ao lado dela. *É mais um degrau da mesma migração, e cada um torna o resto mais
//! barato.*
//!
//! # ⭐⭐⭐ UMA secção para QUATRO componentes, e a razão não é arrumação
//!
//! O `ScrollFactor`, o `ScrollRepeat`, o `ScrollLimits` e o `ScrollMotion` são **quatro populações**
//! (quase todo objecto com paralaxe não repete, quase nenhum tem cerca) — é por isso que eles são
//! quatro componentes, e o descritor do catálogo já o escreve. ⚠️ **Mas o ASSUNTO é um só**: *quanto
//! do movimento do mundo este objecto guarda*. Quatro secções entregariam quatro cabeçalhos a dizer
//! a mesma palavra, e o artista teria de dobrar três para ver o quarto.
//!
//! ⇒ **a secção existe se houver `ScrollFactor`** (que é o que liga a lei) e as outras três
//! aparecem como BLOCOS dentro dela, cada um só quando o componente está lá. *É a ADR-0166 lida
//! como ela está escrita: o Inspector mostra o que o objecto TEM.*
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `there is no game camera` | ⛔ a lei não corre de todo — ela mede contra a vista do JOGO | pôr uma `Game Camera` na cena |
//! | `this layer moves with the world` | `k = 1` nos dois eixos: o passe **salta** este objecto | baixar o `Scroll Factor` |
//!
//! ⚠️⚠️ **Os dois são de espécies DIFERENTES e é por isso que são dois:** no primeiro nenhuma
//! camada da cena se mexe; no segundo só esta. *Dizer «esta camada anda com o mundo» a quem não tem
//! câmera nenhuma é mandá-lo resolver a metade errada* — a lei da recusa dos pincéis.
//!
//! ⛔ **E o neutro NÃO é um defeito** — é o valor de fábrica do componente, e é exactamente por isso
//! que ele precisa de voz: o artista anexa a paralaxe, nada muda, e sem esta linha ele lê *«a
//! paralaxe não funciona»* sobre um motor que está a obedecer.

/// Snapshot da secção PARALLAX da entidade selecionada.
///
/// ⚠️ **Os três blocos opcionais são `Option` e não valores com um `bool` ao lado:** um par
/// `(tem, valor)` é dois campos que têm de concordar, e eles divergem.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorParallaxInfo {
    pub entity_bits: u64,
    /// Quanto do movimento do mundo esta camada guarda, por eixo. `1` = o mundo.
    pub factor: [f32; 2],
    /// O ladrilho, em metros, se o objecto tiver `ScrollRepeat`. `0` num eixo = sem repetição.
    pub repeat: Option<[f32; 2]>,
    /// A deriva própria, em metros por segundo, se o objecto tiver `ScrollMotion`.
    pub motion: Option<[f32; 2]>,
    /// A região de que a vista não sai, para efeito desta camada, se houver `ScrollLimits`.
    pub limits: Option<([f32; 2], [f32; 2])>,
    /// ⭐ **Há uma câmera do jogo na cena?** Derivado do MUNDO e não de um campo — sem ela a fase
    /// não recebe rectângulo nenhum e **nenhuma** camada se mexe.
    pub tem_camera_do_jogo: bool,
    /// ⭐⭐ **Esta camada é NEUTRA?** — e ele é um campo por uma razão de CAMADA, não de gosto.
    ///
    /// ⛔⛔ **Esta crate não pode perguntar ao motor:** ela vive ABAIXO do `ph2d-ecs` no DAG, e um
    /// `use ph2d_ecs::ScrollFactor` aqui é a aresta que a catraca `architecture_no_dependency_
    /// climbs_a_layer` recusa. ⚠️ E escrever `k == [1,1]` aqui seria a SEGUNDA resposta a *«esta
    /// camada é neutra?»* — ela divergiria da do passe no dia em que o neutro mudasse.
    ///
    /// ⇒ quem responde é o CONSTRUTOR do instantâneo, que vive na crate da família e chama a porta
    /// [`ph2d_ecs::ScrollFactor::e_neutro`] — a mesma que o passe lê.
    pub e_neutra: bool,
    pub selected_count: usize,
}

/// As duas razões para nada acontecer, **da mais geral para a mais específica**.
///
/// ⚠️ A ordem é a ordem em que o painel fala, e é a lei da recusa: a primeira que se aplica é a
/// única que se diz.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParallaxQueixa {
    /// Não há `GameCamera` na cena — a lei não corre para ninguém.
    SemCamera,
    /// `k = 1` nos dois eixos: o passe salta este objecto, por construção.
    Neutra,
}

impl InspectorParallaxInfo {
    /// ⭐ **A queixa que o painel mostra, ou `None` quando não há nenhuma.**
    ///
    /// ⚠️ **O neutro chega no campo [`Self::e_neutra`]** e nunca é comparado aqui — ver o doc dele.
    #[must_use]
    pub fn queixa(&self) -> Option<ParallaxQueixa> {
        if !self.tem_camera_do_jogo {
            return Some(ParallaxQueixa::SemCamera);
        }
        if self.e_neutra {
            return Some(ParallaxQueixa::Neutra);
        }
        None
    }
}

/// Uma edição de um campo da secção PARALLAX.
///
/// ⚠️ **Cada variante carrega o PAR inteiro e não um eixo** — é a forma que as irmãs desta casa já
/// usam (`CameraFieldEdit::Offset`), e ela existe porque o dreno escreve um `[f32; 2]`: com um eixo
/// de cada vez o drenador teria de ir buscar o outro ao mundo, e leria o valor de um quadro atrás.
#[derive(Clone, Debug, PartialEq)]
pub enum ParallaxFieldEdit {
    Factor([f32; 2]),
    Repeat([f32; 2]),
    Motion([f32; 2]),
    LimitsMin([f32; 2]),
    LimitsMax([f32; 2]),
}

/// ⭐⭐⭐ **A DISTÂNCIA, como LEITURA derivada do factor** (plano 24, W7) — nunca um segundo campo.
///
/// ⛔ **Guardá-la ao lado do `k` é a recusa medida do plano §5:** são o MESMO número (`z/z₀ = 1/k`,
/// §1), e dois campos que têm de concordar é o defeito que esta casa já pagou. ⇒ ela é calculada
/// para a LEITURA, e o artista continua a escrever um número só.
///
/// ⚠️ **Cinco casos e não uma fórmula**, porque a fórmula mente nas pontas: `1/0` não é «muito
/// longe», é *preso ao ecrã*; `1/k` com `k > 1` é uma camada À FRENTE do mundo (o primeiro plano de
/// um jogo de plataformas); e `k < 0` não tem distância nenhuma — anda contra a câmera.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Profundidade {
    /// `k = 1` — é o mundo.
    Mundo,
    /// `k = 0` — infinitamente longe: presa ao ecrã.
    Ecra,
    /// `0 < k < 1` — `n` vezes mais longe que o plano do mundo.
    Longe(f32),
    /// `k > 1` — à frente do plano do mundo, a `n` da distância dele.
    Frente(f32),
    /// `k < 0` — anda CONTRA a câmera; não é uma distância.
    Contra,
}

impl Profundidade {
    /// ⚠️ Um `k` não-finito lê-se como o MUNDO — é o que o passe faz com ele (não o conduz).
    #[must_use]
    pub fn de(k: f32) -> Self {
        if !k.is_finite() || k == 1.0 {
            Self::Mundo
        } else if k == 0.0 {
            Self::Ecra
        } else if k < 0.0 {
            Self::Contra
        } else if k < 1.0 {
            Self::Longe(1.0 / k)
        } else {
            Self::Frente(1.0 / k)
        }
    }
}
