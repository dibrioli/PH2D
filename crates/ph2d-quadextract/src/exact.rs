//! ⭐⭐⭐ **A ARITMÉTICA EXACTA DESTA CRATE** — o ponto do domínio, a função de
//! transição, e o predicado de orientação.
//!
//! # ⛔ A lei que decide tudo: NADA de epsilon
//!
//! A extracção inteira é uma cadeia de decisões discretas — *este ponto de grade
//! cai dentro deste triângulo, sobre esta aresta, ou fora?* Com aritmética
//! aproximada duas decisões vizinhas **discordam**, e a malha sai com buracos,
//! faces repetidas ou nada. A alternativa — tolerância com `ε` — obriga a
//! intersectar uma bola em torno de cada ponto com toda a vizinhança e a decidir
//! consistentemente no meio de ambiguidades: é **mais** código, **mais** lento, e
//! não fecha.
//!
//! # ⭐⭐⭐ Por que aqui não há biblioteca de precisão múltipla nem filtro
//!
//! A rota óbvia para um predicado exacto sobre `f64` é *filtro rápido + inteiro
//! grande*. Esta crate não precisa de nenhum dos dois, e o motivo é **estrutural**:
//! o saneamento ([`crate::sanitize`]) trunca **toda** a mantissa do domínio para
//! uma grade comum de passo `2^(M−51)`, onde `M` é o maior expoente binário de
//! qualquer coordenada do mapa. Depois disso:
//!
//! | grandeza | limite | porquê |
//! |---|---|---|
//! | coordenada, em passos da grade | `< 2^52` | `\|x\| < 2^(M+1)` e o passo é `2^(M−51)` |
//! | diferença de duas coordenadas | `< 2^53` | cabe em `i64` |
//! | produto de duas diferenças | `< 2^106` | cabe em `i128` |
//! | o determinante (dois produtos) | `< 2^107` | cabe em `i128` |
//!
//! ⇒ **o domínio inteiro vive em `i64` sem perder um bit**, e o determinante é uma
//! conta `i128` **exacta por construção** — não «exacta até um bound». Não há
//! filtro que possa errar porque não há filtro, e o zero é *o* zero.
//!
//! ⚠️ **Os limites acima são identidades, não medições** — eles saem da definição
//! de `Q` e valem em qualquer malha. O que é medido, e por isso guardado, é a
//! folga: ver [`Q_HEADROOM`].

/// **UM PONTO DO DOMÍNIO**, em unidades de `2^-Q` de célula da grade.
///
/// ⚠️ Não é um `f64` disfarçado: depois do saneamento **toda** coordenada do mapa
/// é um múltiplo exacto do passo da grade, e este é o inteiro de passos.
pub type P = [i64; 2];

/// **A FOLGA do expoente**, em bits — a distância entre o teto estrutural de uma
/// coordenada (`2^52`) e o teto do `i64`.
///
/// ⚠️ **Ela existe para ser CONFERIDA e não para ser acreditada:** a conversão em
/// [`crate::sanitize`] recusa um mapa que a viole, em vez de silenciosamente
/// truncar. *Um limite que só vive num comentário é um palpite à espera de um
/// smoke.*
pub const Q_HEADROOM: u32 = 63 - 52;

/// O maior valor absoluto que uma coordenada do domínio pode tomar — `2^52`.
pub const COORD_MAX: i64 = 1 << 52;

/// ⭐ **O SINAL DO DETERMINANTE `det(b−a, c−a)`, SEMPRE CORRECTO.**
///
/// `+1` anti-horário · `−1` horário · `0` colinear — e o `0` é exacto, que é a
/// metade que um predicado aproximado nunca entrega.
///
/// Dele derivam todos os outros: *dentro do triângulo*, *sobre a aresta*, *para
/// que lado a direcção aponta*, *o sinal da área*.
#[must_use]
#[inline]
pub fn orient(a: P, b: P, c: P) -> i8 {
    let (ax, ay) = (i128::from(a[0]), i128::from(a[1]));
    let (bx, by) = (i128::from(b[0]), i128::from(b[1]));
    let (cx, cy) = (i128::from(c[0]), i128::from(c[1]));
    let d = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
    match d.signum() {
        1 => 1,
        -1 => -1,
        _ => 0,
    }
}

/// **O DOBRO DA ÁREA COM SINAL** do triângulo-imagem, em unidades de `(2^-Q)²`.
///
/// ⚠️ Devolve `i128` porque o valor **não** cabe em `i64`; o que cabe em toda a
/// parte é o [`orient`], e é por isso que ele é a primitiva e este o derivado.
#[must_use]
#[inline]
pub fn area2(a: P, b: P, c: P) -> i128 {
    let (ax, ay) = (i128::from(a[0]), i128::from(a[1]));
    let (bx, by) = (i128::from(b[0]), i128::from(b[1]));
    let (cx, cy) = (i128::from(c[0]), i128::from(c[1]));
    (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
}

/// **O PONTO `q` ESTÁ NO SEGMENTO `[a, b]`, extremos EXCLUÍDOS?**
///
/// ⚠️ **Pressupõe `orient(a, b, q) == 0`** — quem chama já o sabe, e repetir o
/// determinante aqui seria pagá-lo duas vezes. O que falta depois da colinearidade
/// é só *estar entre*, e isso é comparação de inteiros.
#[must_use]
pub fn strictly_between(a: P, b: P, q: P) -> bool {
    let by_x = a[0] != b[0];
    let (lo, hi, v) = if by_x {
        (a[0].min(b[0]), a[0].max(b[0]), q[0])
    } else {
        (a[1].min(b[1]), a[1].max(b[1]), q[1])
    };
    lo < v && v < hi
}

/// ⭐ **UMA FUNÇÃO DE TRANSIÇÃO** entre duas cartas: `g(x) = R(r)·x + t`, com
/// `r ∈ {0,1,2,3}` (múltiplos de 90°) e `t` **inteiro** em células.
///
/// ⭐⭐⭐ **É essa forma — quarto de volta mais translação inteira — que faz a grade
/// inteira de uma carta casar com a da vizinha.** Uma translação que não seja
/// inteira desalinha as duas grades, e o saneamento passa a *arredondar o erro
/// para dentro* em vez de o remover: ver [`crate::ExtractReport::shift_residual`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Xf {
    /// Quartos de volta, `0..4`.
    pub r: u8,
    /// A translação, em unidades de `2^-Q` (sempre um múltiplo de uma célula).
    pub t: P,
}

impl Default for Xf {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Xf {
    /// A transição que não faz nada.
    pub const IDENTITY: Self = Self { r: 0, t: [0, 0] };

    /// ⭐ **ROTAÇÃO DE `r` QUARTOS DE VOLTA** — exacta, porque é troca e negação de
    /// componentes e nunca uma multiplicação.
    #[must_use]
    #[inline]
    pub const fn rot(r: u8, p: P) -> P {
        match r & 3 {
            0 => p,
            1 => [-p[1], p[0]],
            2 => [-p[0], -p[1]],
            _ => [p[1], -p[0]],
        }
    }

    /// Aplica a transição a um ponto.
    #[must_use]
    #[inline]
    pub const fn apply(self, p: P) -> P {
        let q = Self::rot(self.r, p);
        [q[0] + self.t[0], q[1] + self.t[1]]
    }

    /// ⭐ **A COMPOSIÇÃO `other ∘ self`** — primeiro `self`, depois `other`.
    ///
    /// ⚠️ A ordem está no nome porque trocá-la é o erro que dá coordenadas locais
    /// impossíveis na extracção de células, e o compilador não o vê.
    #[must_use]
    pub const fn then(self, other: Self) -> Self {
        let rt = Self::rot(other.r, self.t);
        Self {
            r: (self.r + other.r) & 3,
            t: [rt[0] + other.t[0], rt[1] + other.t[1]],
        }
    }

    /// A transição inversa.
    #[must_use]
    pub const fn inverse(self) -> Self {
        let r = (4 - (self.r & 3)) & 3;
        let t = Self::rot(r, self.t);
        Self {
            r,
            t: [-t[0], -t[1]],
        }
    }

    /// ⭐ **O PONTO FIXO de `g(x) = R(r)·x + t`**, em forma fechada.
    ///
    /// | `r` | onde ele cai |
    /// |---|---|
    /// | `2` | metade de cada componente da translação |
    /// | `1`, `3` | uma combinação de **metades** das duas, com os sinais trocados |
    ///
    /// ⛔ **`r == 0` devolve `None`**: a transição é só translação e não tem ponto
    /// fixo nenhum (a menos que `t` seja zero, e aí *todo* ponto é fixo). É por isso
    /// que uma singularidade de valência múltipla de 4 precisa de outra lei — o
    /// inteiro mais próximo — e não desta.
    ///
    /// ⚠️ **O resultado é um múltiplo exacto de meia unidade interna**, logo não
    /// perde um bit: `t` é um múltiplo de uma célula e a grade tem sub-célula.
    #[must_use]
    pub const fn fixed_point(self) -> Option<P> {
        let (tx, ty) = (self.t[0], self.t[1]);
        match self.r & 3 {
            1 => Some([(tx - ty) / 2, (tx + ty) / 2]),
            2 => Some([tx / 2, ty / 2]),
            3 => Some([(tx + ty) / 2, (ty - tx) / 2]),
            _ => None,
        }
    }

    /// **A DIRECÇÃO CARDINAL levada por esta transição** — só a rotação a alcança.
    #[must_use]
    #[inline]
    pub const fn dir(self, d: u8) -> u8 {
        (d + self.r) & 3
    }
}

/// **AS QUATRO DIRECÇÕES CARDINAIS**, em ordem **anti-horária** no domínio:
/// `+u`, `+v`, `−u`, `−v`.
///
/// ⚠️ O índice **é** o nome da direcção (uma transição roda-o somando `r`), então a
/// ordem desta tabela é contrato e não conveniência.
pub const CARDINALS: [[i64; 2]; 4] = [[1, 0], [0, 1], [-1, 0], [0, -1]];

/// O passo de uma direcção cardinal, em unidades de `2^-Q`.
#[must_use]
#[inline]
pub const fn step(d: u8, one: i64) -> P {
    let c = CARDINALS[(d & 3) as usize];
    [c[0] * one, c[1] * one]
}

/// A direcção oposta.
#[must_use]
#[inline]
pub const fn opposite(d: u8) -> u8 {
    (d + 2) & 3
}

/// ⭐ **A ORIENTAÇÃO DE UMA DIRECÇÃO CARDINAL contra um raio**, exacta.
///
/// Devolve o sinal de `det(ray, cardinal)`: `+1` se a cardinal está **à esquerda**
/// do raio, `−1` à direita, `0` se são colineares (mesmo sentido ou oposto).
///
/// ⚠️ **Ele não chama [`orient`] de propósito.** Com uma cardinal o determinante
/// colapsa numa componente só do raio — a conta some, e com ela some a única via
/// pela qual um `i128` poderia transbordar num caminho quente.
#[must_use]
#[inline]
pub fn side_of_ray(ray: P, d: u8) -> i8 {
    // det(ray, cardinal) = ray.x·c.y − ray.y·c.x
    let c = CARDINALS[(d & 3) as usize];
    let v = match (c[0], c[1]) {
        (1, 0) => -ray[1],
        (0, 1) => ray[0],
        (-1, 0) => ray[1],
        _ => -ray[0],
    };
    match v.signum() {
        1 => 1,
        -1 => -1,
        _ => 0,
    }
}

/// **A CARDINAL APONTA NO MESMO SENTIDO DO RAIO?** — só faz sentido quando
/// [`side_of_ray`] já devolveu `0`.
///
/// ⚠️ Colinear **não** é o mesmo que *para o mesmo lado*: o raio oposto também dá
/// determinante zero, e tratá-los como iguais duplica saídas na fronteira do leque.
#[must_use]
#[inline]
pub fn same_sense(ray: P, d: u8) -> bool {
    let c = CARDINALS[(d & 3) as usize];
    ray[0] * c[0] + ray[1] * c[1] > 0
}
