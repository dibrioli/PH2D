//! A matemática mínima da lei: vectores de três, e rotações como quaterniões.
//!
//! ⛔ **Zero dependências, de propósito** (ver o cabeçalho do `Cargo.toml`) — e
//! por isso a álgebra vive aqui, no tamanho exacto do que a lei usa.
//!
//! ⚠️⚠️ **Tudo em `f32`, e isso é LEI e não conveniência.** A espec §12.1 mediu
//! que a comparação de região é um `<` estrito em precisão simples, e que um
//! único vértice a atravessar essa fronteira muda a franja ⇒ muda o pivô ⇒ muda
//! a deformação inteira. Fazer a conta em `f64` "para ser mais exacto" degrada a
//! paridade com o oráculo de `1e-7` para `1e-2` em malhas cujos vértices calham
//! sobre o raio. *Aqui, mais precisão dá mais erro.*

/// Um ponto ou vector em espaço de objecto.
pub type V3 = [f32; 3];

pub fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

pub fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

pub fn escalar(a: V3, k: f32) -> V3 {
    [a[0] * k, a[1] * k, a[2] * k]
}

pub fn ponto(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn cruz(a: V3, b: V3) -> V3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// ⚠️ A distância que o predicado de região usa — **raiz da soma dos quadrados
/// em `f32`**, exactamente como a espec §12.1 prescreve. Mudar a ordem das
/// operações aqui move vértices de um lado para o outro da fronteira.
pub fn distancia(a: V3, b: V3) -> f32 {
    let d = sub(a, b);
    ponto(d, d).sqrt()
}

pub fn comprimento(a: V3) -> f32 {
    ponto(a, a).sqrt()
}

/// Normaliza, devolvendo `None` para o vector nulo — e é o chamador que decide o
/// que fazer com isso.
///
/// ⚠️ **Devolver `Option` em vez de um `[0,0,0]` silencioso é load-bearing:** o
/// §11.1 da espec é exactamente o caso em que o pivô cai sobre o cursor e o
/// primeiro segmento tem comprimento nulo. Quem normalizar às cegas ali produz
/// `NaN` ou uma direcção arbitrária, e a malha inteira parte.
pub fn normalizar(v: V3) -> Option<V3> {
    let n = comprimento(v);
    if !n.is_finite() || n <= 0.0 {
        return None;
    }
    Some(escalar(v, 1.0 / n))
}

/// Quantos ulps de folga damos ao chão de ruído de uma diferença de posições.
///
/// ⭐⭐ **Este número sai de um VALE MEDIDO, não de conforto.** Varrido o corpus
/// inteiro, o comprimento do primeiro segmento vale **`0,9` ulps** na única
/// fixtura degenerada (§11.1) e **`176 683` ulps** na mais pequena de todas as
/// outras — *cinco ordens de grandeza de vazio entre os dois lados*. Qualquer
/// barra entre `10` e `10 000` separa-os; `16` está `18×` acima do ruído medido
/// e `10 000×` abaixo do primeiro caso real.
const ULPS_DE_RUIDO: f32 = 16.0;

/// ⭐ **A direcção de `de` para `para`, ou `None` quando a diferença é só
/// RUÍDO DE REPRESENTAÇÃO.**
///
/// ⚠️⚠️ **É esta porta que cura o §11.1**, e o mecanismo é fino: quando o pivô
/// cai em cima do cursor, a diferença entre a cabeça e a origem do primeiro
/// segmento vale **um ulp** das próprias coordenadas — ela é feita inteiramente
/// do arredondamento com que as duas foram escritas, e a direcção que se tira
/// dela é ruído normalizado a comprimento `1`. O alvo devolve ali deslocamento
/// **zero em toda a malha**; quem normalizar às cegas roda a região por um
/// ângulo arbitrário, e o modelo de referência da própria espec errou essa
/// fixtura por `2,5e-1` — *o maior erro de todo o corpus veio de um vector de
/// comprimento quase nulo*.
///
/// ⛔ **O piso é RELATIVO à magnitude das posições**, nunca uma constante
/// absoluta: um ulp perto de `2,2` é `2,4e-7` e perto de `0,002` é `2,4e-10`.
/// *Uma barra absoluta aqui mede o tamanho da peça, não o ruído.*
pub fn direccao_entre_posicoes(de: V3, para: V3) -> Option<V3> {
    let escala = de
        .iter()
        .chain(para.iter())
        .fold(0.0f32, |m, c| m.max(c.abs()));
    let piso = ULPS_DE_RUIDO * f32::EPSILON * escala.max(1.0);
    let d = sub(para, de);
    if comprimento(d) <= piso {
        return None;
    }
    normalizar(d)
}

/// Uma rotação. Guardada como quaternião unitário `(w, x, y, z)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rot {
    pub w: f32,
    pub v: V3,
}

impl Rot {
    pub const IDENTIDADE: Rot = Rot {
        w: 1.0,
        v: [0.0, 0.0, 0.0],
    };

    /// A rotação de `angulo` radianos em torno de `eixo` (que tem de vir
    /// normalizado; um eixo nulo devolve a identidade).
    pub fn eixo_angulo(eixo: V3, angulo: f32) -> Rot {
        match normalizar(eixo) {
            None => Rot::IDENTIDADE,
            Some(e) => {
                let (s, c) = (angulo * 0.5).sin_cos();
                Rot {
                    w: c,
                    v: escalar(e, s),
                }
            }
        }
    }

    /// ⭐ **A rotação que leva `de` em `para`** — e o item **1** da lista de
    /// verificação da espec (§17): *devolve **identidade** quando um dos dois é
    /// nulo*.
    ///
    /// ⚠️⚠️ **Isto não é defensividade: é a fixtura `_origem_no_cursor`.** Ali o
    /// pivô coincide com o ponto de aplicação, o primeiro segmento nasce com
    /// comprimento `4,8e-8`, e o alvo devolve deslocamento **zero** em toda a
    /// malha. O modelo de referência da própria obra produziu ali uma rotação
    /// arbitrária e errou a fixtura por `2,5e-1` — *o maior erro de todo o
    /// corpus veio de um vector de comprimento quase nulo*.
    pub fn entre(de: V3, para: V3) -> Rot {
        let (Some(a), Some(b)) = (normalizar(de), normalizar(para)) else {
            return Rot::IDENTIDADE;
        };
        let c = ponto(a, b).clamp(-1.0, 1.0);
        // Anti-paralelos: o eixo é indeterminado, e qualquer perpendicular
        // serve. Escolhemos uma determinadamente (a de menor componente) para
        // que a saída não dependa da ordem dos bits — o determinismo desta lei
        // é uma promessa que fazemos ao alvo (§12).
        if c < -1.0 + 1e-6 {
            let eixo = perpendicular_qualquer(a);
            return Rot::eixo_angulo(eixo, core::f32::consts::PI);
        }
        let eixo = cruz(a, b);
        // `w = 1 + cos`, `v = a × b`; normalizado dá a rotação de meio ângulo.
        let q = Rot { w: 1.0 + c, v: eixo };
        q.normalizada()
    }

    pub fn normalizada(self) -> Rot {
        let n = (self.w * self.w + ponto(self.v, self.v)).sqrt();
        if !n.is_finite() || n <= 0.0 {
            return Rot::IDENTIDADE;
        }
        let k = 1.0 / n;
        Rot {
            w: self.w * k,
            v: escalar(self.v, k),
        }
    }

    /// Aplica a rotação a um vector.
    pub fn aplicar(self, p: V3) -> V3 {
        // p + 2w(v × p) + 2(v × (v × p))
        let t = escalar(cruz(self.v, p), 2.0);
        add(add(p, escalar(t, self.w)), cruz(self.v, t))
    }

    /// ⭐ **Espelhar a rotação num eixo** (§6): *negar a componente do eixo no
    /// vector de rotação **e** o ângulo*.
    ///
    /// ⚠️ Escrito em quaternião isso é **negar todas as componentes MENOS a do
    /// eixo** — as duas negações cancelam-se na componente espelhada. É a
    /// conjugação da rotação pela mesma reflexão, e é fácil de escrever ao
    /// contrário: negar **só** a componente do eixo dá a rotação errada, e o
    /// erro só aparece em fixturas com simetria activa.
    pub fn espelhada(self, eixo: usize) -> Rot {
        let mut v = self.v;
        for (i, c) in v.iter_mut().enumerate() {
            if i != eixo {
                *c = -*c;
            }
        }
        Rot { w: self.w, v }
    }
}

/// Uma perpendicular determinada a um vector unitário.
fn perpendicular_qualquer(a: V3) -> V3 {
    let menor = if a[0].abs() <= a[1].abs() && a[0].abs() <= a[2].abs() {
        [1.0, 0.0, 0.0]
    } else if a[1].abs() <= a[2].abs() {
        [0.0, 1.0, 0.0]
    } else {
        [0.0, 0.0, 1.0]
    };
    cruz(a, menor)
}
