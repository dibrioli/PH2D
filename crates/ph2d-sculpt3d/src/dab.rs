//! **O QUE UM TOQUE É** — o [`Dab`] e os construtores que nomeiam a leitura dele.
//!
//! Irmão (`#[path]`) do [`super`], e o corte é o que o doc daquele arquivo já
//! separava em prosa: lá mora **a LEI DO TRAÇO** (o envelope, o `pre` congelado,
//! o que sobrevive a uma lista de dabs); aqui mora **o dado de um toque só**.
//!
//! ⚠️ E é aqui que vivem os quatro construtores, que são a única barreira contra
//! a confusão que este tipo admite: [`Dab::pull`] é um TOTAL para o Grab e um
//! INCREMENTO para o Snake Hook, e [`Dab::amount`] é radiano para o Twist e
//! fração para o Local Scale. Nenhum tipo distingue os dois; quem distingue é o
//! nome no sítio de construção.

/// Um toque de pincel: **onde a mão estava e com que força apertou**. O que a
/// ferramenta É vive no [`Brush`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dab {
    /// Centro, em coordenadas de mundo (tipicamente o `Hit::point` do pick).
    pub center: [f32; 3],
    /// Raio de influência, em unidades de mundo.
    pub radius: f32,
    /// Pressão do dispositivo em `[0, 1]`. Sem tablet, `1.0`.
    pub pressure: f32,
    /// A direção do OLHO no instante do pick — do olho para a superfície,
    /// unitária. É o `dir` do raio que produziu o `center`.
    ///
    /// ⚠️ **Ela é do DAB e não do pincel**, e a razão é a simetria: no original
    /// o espelho é aplicado ao raio **antes** de a direção ser computada
    /// (`Picking.js:211-223`), então a cópia espelhada tem o olho espelhado
    /// junto. Guardá-la no `Brush` daria a MESMA direção às duas cópias, e a
    /// metade espelhada passaria a ser ajustada por um olho que não é o dela.
    ///
    /// ⚠️ **É argumento obrigatório do [`Dab::at`], nunca um builder opcional.**
    /// A lição é do `with_arc_len` do Painter 2D: um campo opcional de dab
    /// chegava em 2 de 7 rotas, e nas outras 5 a feature simplesmente não
    /// acontecia — em silêncio, com o painel dizendo que sim.
    pub eye: [f32; 3],
    /// **O gesto**: o vetor de MUNDO que este dab tem para dar.
    ///
    /// ⚠️ **A leitura é do [`crate::Grip`], e há duas** — o construtor nomeia
    /// qual, e é ele o guarda:
    ///
    /// - [`Dab::pulling`] ([`Grip::Hold`]): o deslocamento **TOTAL** desde o
    ///   pen-down. É o que mantém a lei do traço de pé — o alvo é `base + pull`,
    ///   função do `pre` congelado, então puxar de volta devolve o barro ao
    ///   lugar e re-carimbar não intensifica nada. Com incrementos aqui, cada um
    ///   seria uma soma sobre o resultado do anterior: o produto sobre a lista
    ///   de dabs que este módulo existe para não ter.
    /// - [`Dab::hooking`] ([`Grip::Hook`]): o **INCREMENTO** desde o dab
    ///   anterior. Ali a soma sobre a lista **é** a feature (esticar é
    ///   transportar matéria), e o que a torna um fato do CAMINHO em vez da taxa
    ///   de polling é o walk do espaçamento, que fixa o passo na geometria.
    ///
    /// Um campo com duas leituras é um risco declarado, e a alternativa é pior:
    /// dois campos deixariam **um deles morto em doze verbos**, e o outro morto
    /// no décimo terceiro. Nenhum tipo distingue um total de um incremento —
    /// quem distingue é o nome de quem constrói.
    ///
    /// Só os verbos que respondem `true` a [`Brush::verb`]`.anchors()` o leem;
    /// para os outros ele é zero e inerte.
    pub pull: [f32; 3],
    /// **O gesto ESCALAR**: quanto o [`Grip::Turn`] girou ou cresceu desde o
    /// pen-down. A unidade é a que o [`crate::Amount`] do verbo nomeia — radianos
    /// para o [`crate::Verb::Twist`], fração para o [`crate::Verb::LocalScale`].
    ///
    /// ⚠️ **É o TOTAL desde o pen-down, nunca um incremento**, e é essa escolha
    /// que mantém a lei do traço de pé para os dois verbos que o leem — ver
    /// [`Grip::Turn`]. Os construtores [`Dab::turning`] e [`Dab::scaling`] são
    /// quem nomeia a unidade, exatamente como [`Dab::pulling`] e
    /// [`Dab::hooking`] nomeiam a leitura do [`Self::pull`]: nenhum tipo
    /// distingue um radiano de uma fração.
    ///
    /// ⚠️ **Um campo a mais, e não dois**, pelo motivo que o [`Self::pull`] já
    /// pagou: dois campos deixariam cada um morto em quinze dos dezesseis
    /// verbos, e o dia em que entrasse o terceiro gesto escalar seriam três.
    pub amount: f32,
}

impl Dab {
    /// Um dab de pressão cheia, visto de `eye`.
    #[must_use]
    pub fn at(center: [f32; 3], radius: f32, eye: [f32; 3]) -> Self {
        Self {
            center,
            radius,
            pressure: 1.0,
            eye,
            pull: [0.0; 3],
            amount: 0.0,
        }
    }

    /// Um dab que **PUXA** — o gesto do Grab.
    ///
    /// ⚠️ Construtor irmão em vez de um builder opcional: um `with_pull()` é
    /// exatamente a forma que o `with_arc_len` do Painter 2D tinha quando ele
    /// chegava em 2 de 7 rotas e a feature simplesmente não acontecia nas outras
    /// cinco, em silêncio. Quem puxa pede este; quem não puxa não o vê.
    #[must_use]
    pub fn pulling(center: [f32; 3], radius: f32, eye: [f32; 3], pull: [f32; 3]) -> Self {
        Self {
            pull,
            ..Self::at(center, radius, eye)
        }
    }

    /// Um dab que **ARRASTA** — o gesto do Snake Hook, e `step` é o
    /// **INCREMENTO** desde o dab anterior, não o total.
    ///
    /// ⚠️ Irmão de [`Dab::pulling`] e não um flag nele: a diferença entre um
    /// total e um incremento não é visível no tipo, e um `Dab { pull, .. }`
    /// construído à mão pode carregar qualquer um dos dois sem o compilador
    /// piscar. O nome no sítio de construção é a única barreira que existe, e
    /// por isso ela tem de estar lá.
    #[must_use]
    pub fn hooking(center: [f32; 3], radius: f32, eye: [f32; 3], step: [f32; 3]) -> Self {
        Self::pulling(center, radius, eye, step)
    }

    /// Um dab que **TORCE** — o gesto do Twist, e `radians` é o ângulo varrido
    /// **TOTAL** desde o pen-down.
    ///
    /// ⚠️ **O EIXO é o olho, e ele entra unitário daqui**: a rotação é em torno
    /// da reta que passa pela âncora na direção de quem olha, e Rodrigues exige
    /// um eixo de norma 1 (com norma `k` ele devolveria uma rotação *mais uma
    /// escala*, e o barro encolheria com o giro). Um espelho preserva
    /// comprimento, então normalizar no sítio de construção basta para todas as
    /// cópias da simetria.
    ///
    /// ⚠️ **Sem eixo não há giro:** um olho degenerado zera o ÂNGULO em vez de
    /// produzir um eixo inventado. É a diferença entre um dab que não faz nada e
    /// um dab que colapsa a pegada na âncora.
    #[must_use]
    pub fn turning(center: [f32; 3], radius: f32, eye: [f32; 3], radians: f32) -> Self {
        let len = (eye[0] * eye[0] + eye[1] * eye[1] + eye[2] * eye[2]).sqrt();
        let (axis, amount) = if len.is_finite() && len > 1e-12 {
            ([eye[0] / len, eye[1] / len, eye[2] / len], radians)
        } else {
            (eye, 0.0)
        };
        Self {
            amount,
            ..Self::at(center, radius, axis)
        }
    }

    /// Um dab que **INFLA ou ENCOLHE** — o gesto do Local Scale, e `fraction` é
    /// a fração **TOTAL** desde o pen-down (`0` não mexe, `+1` dobra o raio da
    /// pegada, `−1` a colapsa na âncora).
    #[must_use]
    pub fn scaling(center: [f32; 3], radius: f32, eye: [f32; 3], fraction: f32) -> Self {
        Self {
            amount: fraction,
            ..Self::at(center, radius, eye)
        }
    }
}
