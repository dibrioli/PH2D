//! **O DAB QUE NÃO É UM DISCO** — a pegada ganha uma FORMA, e a curva de
//! falloff passa a receber uma coordenada LOCAL em vez de uma distância
//! escalar.
//!
//! Até aqui todo verbo media `dist / raio` contra o centro: um disco, sempre. É
//! o que o SculptGL faz, e é por isso que o catálogo inteiro dele tem a mesma
//! silhueta. A blocagem de verdade pede outra coisa — uma **faixa**, com miolo
//! chato e quinas arredondadas, deitada na direção em que a mão vai.
//!
//! ⚠️ **A entrada da curva continua sendo UM número**, e isso é a espinha: a
//! dureza ([`crate::Brush::shaped_distance`]), as doze curvas do
//! [`crate::Falloff`] e a curva própria da máscara continuam lendo o mesmo `t`
//! normalizado. O que muda é **de onde ele vem** — e é por isso que uma forma
//! nova não pede um segundo falloff.
//!
//! ⚠️ **[`Footprint::Disc`] reduz LITERALMENTE ao mundo que já shipa** (`t =
//! dist · inv_r`, portão `1.0`), então quinze dos dezasseis verbos atravessam
//! esta camada byte a byte. É a mesma âncora que o `rf = 1` da multi-resolução
//! do Wet Paint e o `eye = 0` do estêncil do alpha usam: *a forma nova é a
//! antiga quando o parâmetro dela é neutro*.

/// **A SILHUETA DE UM DAB**, hoisted uma vez por dab.
///
/// ⚠️ Construída fora do laço de vértices, como o [`crate::AlphaFrame`] — a
/// moldura depende do plano ajustado e da direção do traço, que são fatos do
/// DAB, e recomputá-la por vértice pagaria a raiz quadrada de novo em cada um.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Footprint {
    /// O disco de sempre: `t = dist / raio`, sem portão de profundidade.
    Disc,
    /// A **FAIXA** do Clay Strips: caixa arredondada no plano e parábola na
    /// profundidade.
    Strip(Strip),
}

/// A moldura local de uma [`Footprint::Strip`].
///
/// Ela mapeia um ponto de mundo para `(x, y, z)` normalizados: `x` **atravessa**
/// o traço, `y` corre **ao longo** dele, e `z` mede quão fundo o ponto está
/// abaixo do plano, em raios.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Strip {
    /// A origem do frame: o ponto do plano ajustado.
    pub origin: [f32; 3],
    /// O eixo que ATRAVESSA o traço, já dividido pelo raio.
    pub across: [f32; 3],
    /// O eixo que corre AO LONGO do traço, já dividido por `raio ·
    /// comprimento`.
    pub along: [f32; 3],
    /// A normal do plano, já dividida pelo raio e **virada para BAIXO** — um
    /// ponto sob o plano tem `z` positivo.
    pub down: [f32; 3],
    /// Quão quadrada é a ponta: `1` é um disco, `0` é uma caixa de quina viva.
    pub roundness: f32,
}

impl Footprint {
    /// **A coordenada que a curva consome, e o portão que a profundidade
    /// impõe** — `(t, gate)`.
    ///
    /// `t` é `0` no miolo e `1` na borda, exatamente como a distância
    /// normalizada que ele substitui; o `gate` multiplica o peso depois da
    /// curva e vale `1` no disco.
    #[must_use]
    #[inline]
    pub fn at(self, p: [f32; 3], dist: f32, inv_r: f32) -> (f32, f32) {
        match self {
            Self::Disc => (dist * inv_r, 1.0),
            Self::Strip(s) => s.at(p),
        }
    }

    /// **Quantos raios a pegada alcança** — o fator que o raio de consulta
    /// multiplica.
    ///
    /// ⚠️ **Uma caixa não cabe no círculo que a inscreve**, e esquecer isto
    /// corta as QUINAS da faixa: o canto de uma caixa de meio-lado `1 × L` está
    /// a `√(1 + L²)` raios do centro, então uma consulta de raio `r` devolveria
    /// uma faixa com os cantos comidos — e o defeito seria **mudo**, porque a
    /// silhueta continuaria plausível.
    #[must_use]
    pub fn query_factor(self) -> f32 {
        match self {
            Self::Disc => 1.0,
            Self::Strip(s) => Self::strip_query_factor(s.length()),
        }
    }

    /// O mesmo fator, perguntado **antes de a moldura existir**.
    ///
    /// ⚠️ **Porta e não uma segunda expressão:** quem dimensiona a consulta
    /// ([`crate::Brush::query_radius`]) tem de a fazer antes de saber a direção
    /// do traço — a moldura nasce depois, dentro do dab —, e duas cópias da
    /// mesma raiz divergiriam no dia em que a faixa ganhasse uma terceira
    /// dimensão de forma.
    #[must_use]
    pub fn strip_query_factor(length: f32) -> f32 {
        (1.0 + length * length).sqrt()
    }
}

impl Strip {
    /// Constrói a moldura, ou devolve `None` quando não há plano em que a
    /// deitar.
    ///
    /// ⚠️ **SEM DIREÇÃO ELA NASCE REDONDA, não deixa de nascer** — e a primeira
    /// versão errava aqui, de um jeito que só um gate de PRODUTO viu. O caminho
    /// decide **a ORIENTAÇÃO da caixa** e mais nada; a profundidade é fato do
    /// PLANO, que existe desde o primeiro dab. Caindo no [`Footprint::Disc`], o
    /// toque perdia o portão parabólico junto com a caixa e depositava como um
    /// Draw — medido, `0,039998` de barro onde a lei manda **zero**.
    ///
    /// ⇒ Sem caminho a ponta é **redonda** (`roundness = 1`, `length = 1`), e
    /// aí a moldura no plano é **irrelevante por construção**: a caixa
    /// totalmente arredondada é a distância euclidiana, que não sabe para que
    /// lado os eixos apontam. Escolher um perpendicular qualquer não é inventar
    /// uma orientação — é escolher entre respostas que são a MESMA.
    ///
    /// ⚠️ **O `None` que sobra é honesto:** sem normal não há plano, e sem plano
    /// não há nem caixa nem profundidade.
    #[must_use]
    pub fn new(
        origin: [f32; 3],
        normal: [f32; 3],
        path: [f32; 3],
        radius: f32,
        length: f32,
        roundness: f32,
    ) -> Option<Self> {
        // ⚠️ **A pergunta é FINITO E POSITIVO, e escrita assim** — negar um `>`
        // sobre `f32` é o que deixa o `NaN` atravessar em silêncio, e é o que o
        // clippy nomeia.
        if !(radius.is_finite() && radius > 0.0 && length.is_finite() && length > 0.0) {
            return None;
        }
        let n = unit(normal)?;
        let down = [-n[0], -n[1], -n[2]];
        // `across = n × caminho` é perpendicular aos dois, e some quando o
        // caminho é paralelo à normal (a mão a andar "para dentro" da
        // superfície) — o mesmo caso do caminho nulo, e com a mesma resposta.
        let (across, length, roundness) = match unit(cross(n, path)) {
            Some(a) => (a, length, roundness),
            None => (any_perpendicular(n), 1.0, 1.0),
        };
        // `along = across × n` fecha a base destra e é a projeção do caminho no
        // plano — o traço deitado na superfície, que é onde a faixa vive.
        let along = unit(cross(across, n))?;
        let inv_r = 1.0 / radius;
        let inv_l = inv_r / length;
        Some(Self {
            origin,
            across: [across[0] * inv_r, across[1] * inv_r, across[2] * inv_r],
            along: [along[0] * inv_l, along[1] * inv_l, along[2] * inv_l],
            down: [down[0] * inv_r, down[1] * inv_r, down[2] * inv_r],
            roundness: roundness.clamp(0.0, 1.0),
        })
    }

    /// Quantos raios a faixa mede ao longo do traço — o inverso da escala do
    /// eixo `along`.
    #[must_use]
    pub fn length(self) -> f32 {
        let a = dot(self.along, self.along).sqrt();
        let d = dot(self.down, self.down).sqrt();
        if a > 0.0 { d / a } else { 1.0 }
    }

    /// `(t, gate)` para um ponto de mundo.
    #[must_use]
    #[inline]
    pub fn at(self, p: [f32; 3]) -> (f32, f32) {
        let d = [
            p[0] - self.origin[0],
            p[1] - self.origin[1],
            p[2] - self.origin[2],
        ];
        let z = dot(d, self.down);
        // ⚠️ **A PARÁBOLA `z·(1−z)` é o que faz a faixa DEPOSITAR em vez de
        // levantar um domo**: ela vale zero na superfície do plano, tem o pico a
        // meio raio abaixo dele e volta a zero um raio abaixo. Fora de `(0, 1)`
        // o ponto simplesmente não é barro que esta passada tenha de mover —
        // quem está ACIMA do plano já está no lugar, e quem está fundo demais
        // está longe.
        //
        // ⚠️ **É a SUBIDA desta parábola que faz a faixa NIVELAR** — mais fundo
        // recebe mais, até o pico. Onde a superfície em repouso pousa nela é o
        // [`crate::STRIP_PLANE_FRACTION`], e é ele que decide se a passada fecha
        // um vale ou o exagera (a tabela medida está no doc dele).
        //
        // ⚠️ **O ganho é uma CALIBRAÇÃO, não parte da lei:** ele repõe o pico na
        // superfície em repouso para que mover o lift mude a FORMA sem mudar a
        // força. Ver [`crate::STRIP_DEPTH_GAIN`].
        let gate = crate::STRIP_DEPTH_GAIN * (z * (1.0 - z)).max(0.0);
        if gate <= 0.0 {
            // Fora da faixa em profundidade: o `t` não importa, mas devolvê-lo
            // fora do alcance mantém a leitura honesta para quem inspecionar.
            return (1.0, 0.0);
        }
        (
            rounded_box(dot(d, self.across), dot(d, self.along), self.roundness),
            gate,
        )
    }
}

/// **A DISTÂNCIA DE UMA CAIXA ARREDONDADA**, normalizada: `0` no miolo chato,
/// `1` na borda.
///
/// A caixa tem meio-lado `1` nos dois eixos e quinas de raio `roundness`; o
/// miolo (o quadrado de meio-lado `1 − roundness`) é **chato**, que é o que
/// separa uma faixa de um domo — dentro dele todo ponto recebe o peso cheio.
///
/// ⚠️ **`roundness = 1` devolve a distância EUCLIDIANA, exatamente**: o miolo
/// chato colapsa num ponto e toda consulta cai no ramo da quina, cujo centro é a
/// origem. É essa identidade que torna a forma nova uma generalização em vez de
/// um segundo caminho — e há gate a pinar o valor, não a prosa.
///
/// ⚠️ **E fora da caixa devolve `1` em vez do valor cru**: uma curva de falloff
/// vale zero em `t = 1` e o consumidor filtra por `>= 1`, então saturar aqui é o
/// que impede um ponto do canto de aparecer com peso por acidente de aritmética.
#[must_use]
#[inline]
pub fn rounded_box(x: f32, y: f32, roundness: f32) -> f32 {
    let (x, y) = (x.abs(), y.abs());
    if x > 1.0 || y > 1.0 {
        return 1.0;
    }
    if roundness <= 0.0 {
        // Quina viva: dentro é dentro, e a borda é um degrau.
        return 0.0;
    }
    let flat = 1.0 - roundness;
    let inv = 1.0 / roundness;
    if x.min(y) > flat {
        // A quina: a distância ao centro do arco que a arredonda.
        let (dx, dy) = (x - flat, y - flat);
        ((dx * dx + dy * dy).sqrt() * inv).min(1.0)
    } else if x.max(y) > flat {
        // O lado: a distância à aresta do miolo, ao longo de um eixo só.
        ((x.max(y) - flat) * inv).min(1.0)
    } else {
        0.0
    }
}

/// Um versor perpendicular a `n`, qualquer um.
///
/// ⚠️ **"Qualquer um" é literal e é seguro só porque quem o pede já forçou a
/// ponta REDONDA** — ver [`Strip::new`]. Num perfil arredondado a distância não
/// depende de para onde os eixos apontam, então as infinitas escolhas são a
/// mesma resposta; num perfil de caixa, seria uma faixa deitada num ângulo que
/// ninguém pediu.
fn any_perpendicular(n: [f32; 3]) -> [f32; 3] {
    // O eixo de mundo em que `n` é MAIS FRACA — o produto vetorial com ele é o
    // mais bem condicionado dos três.
    let a = if n[0].abs() <= n[1].abs() && n[0].abs() <= n[2].abs() {
        [1.0, 0.0, 0.0]
    } else if n[1].abs() <= n[2].abs() {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    unit(cross(n, a)).unwrap_or([1.0, 0.0, 0.0])
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn unit(v: [f32; 3]) -> Option<[f32; 3]> {
    let len = dot(v, v).sqrt();
    if len.is_finite() && len > 1e-9 {
        Some([v[0] / len, v[1] / len, v[2] / len])
    } else {
        None
    }
}

#[cfg(test)]
#[path = "footprint_tests.rs"]
mod tests;
