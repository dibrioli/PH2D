#![forbid(unsafe_code)]
//! ⭐⭐ **O CONTACTO ENTRE PEÇAS** — doc 109, ordem do dono (2026-09-13): *«colidem sozinhas»*.
//!
//! Peças que se sobrepõem são afastadas até apenas se tocarem. É a lei do `motion.collide`
//! (restrição de não-penetração do *Position Based Dynamics*, Müller et al. 2007), com o mesmo
//! **Jacobi com média** que o tornou independente da ordem do stream (Macklin & Müller, *Unified
//! Particle Physics*, 2014): cada varredura lê UMA fotografia das posições, cada peça soma o que
//! os contatos dela pedem, e aplica a MÉDIA.
//!
//! ## As peças têm FORMA — disco ou caixa (doc 109 §5)
//!
//! Report do dono, com foto: *«collider impreciso, o collider não é gerado conforme a forma da
//! Shape»*. Um quadrado declarado como o disco à volta dele deixava `41 %` de ar entre peças que o
//! olho lê como caixas. Um [`Colisor`] é um [`Forma::Disco`] ou uma [`Forma::Caixa`] orientada, com
//! o centro deslocado de `P` quando a arte não está centrada na origem da peça.
//!
//! - **disco × disco** — a lei de sempre, termo a termo.
//! - **caixa × caixa** — o teorema do eixo separador: quatro eixos, e o de MENOR sobreposição dá a
//!   normal e a profundidade.
//! - **disco × caixa** — o ponto da caixa mais próximo do centro do disco; com o centro já DENTRO,
//!   a face de menor penetração (a lei do `SHAPE_BOX` do `sim.collide`).
//!
//! ## E as peças RODAM (doc 109 §6)
//!
//! Report do dono no dia seguinte: *«precisa destravar a rot. e colocar outro botão para travar
//! rotação»*. Todo contacto sabe agora **ONDE** toca ([`Contacto::ponto`]), e a correcção reparte-se
//! entre mover e RODAR, na forma canónica do PBD de corpo rígido:
//!
//! ```text
//!   r = ponto − centro          (o braço)
//!   c = r × n                   (a alavanca da normal naquele braço)
//!   k = w + invI · c²           (a massa efectiva do contacto, por peça)
//!   λ = penetração / (k_a + k_b)
//!   Δp = n · λ · w              Δθ = c · λ · invI
//! ```
//!
//! ⚠️ **`invI = 0` TRAVA a rotação e devolve a lei anterior**: `k = w`, `λ = pen / (w_a + w_b)` e
//! `Δp = n · pen · w / (w_a + w_b)` — o que a versão sem rotação fazia, termo a termo.
//!
//! ⚠️⚠️ **O ponto de contacto de duas caixas é o MEIO do trecho que penetra**, e não o vértice mais
//! fundo: uma caixa pousada de chapa sobre outra tem dois vértices à mesma profundidade, e escolher
//! um deles daria binário a uma pilha parada — ela tombava sozinha, sem ninguém lhe tocar. O trecho
//! sai do recorte da face incidente contra a de referência (Sutherland–Hodgman de dois pontos).
//!
//! ⛔ **O que isto NÃO é:** não há velocidade ANGULAR. A rotação é uma projecção de posição, como o
//! afastamento — uma peça roda enquanto está em contacto e não continua a girar no ar. É o que
//! separa isto de um corpo rígido a sério, e está nomeado no doc 109 §6.
//!
//! ## E as peças têm MATERIAL (doc 109 §7)
//!
//! Report do dono: *«os círculos não rotacionam com a colisão, talvez por falta de atrito.
//! Precisamos de parâmetros do material»* — e ele tinha razão pela conta que o módulo [`atrito`]
//! escreve: **a alavanca da normal sobre um disco é EXACTAMENTE zero**, logo nenhuma lei que só
//! empurre ao longo dela roda um círculo. A metade que faltava é a TANGENTE, cuja alavanca no
//! mesmo disco é o raio inteiro. Ler [`atrito`] antes de tocar aqui.
//!
//! ## A lei do par é calculada na ordem do PAR
//!
//! Cada par resolve-se sempre do índice MENOR para o MAIOR, e o maior recebe a normal simétrica —
//! os dois lados de um contacto são **exactamente** opostos, e dois centros coincidentes separam-se
//! em sentidos contrários sem desempate nenhum a inventar.
//!
//! ## Porque é uma folha
//!
//! Dois integradores a pedem — o `sim.step` (W2) e o `motion.integrate` (W4) — e o `sim.collide`
//! pergunta-lhe que forma uma peça declarou. Copiar a lei seria a segunda resposta à mesma
//! pergunta; um nó a depender de outro nó quebraria o isolamento.
//!
//! ## A grelha dá os MESMOS BITS que todos-os-pares, e isso é uma escolha
//!
//! O `motion.collide` de CPU é `O(n² · varreduras)`. Aqui cada peça procura parceiros só nas 9
//! células vizinhas de uma grelha de lado `2 · alcance_max`, onde o ALCANCE é o raio, a partir de
//! `P`, do círculo que contém o colisor — duas peças cujos colisores se tocam estão a
//! `alcance_i + alcance_j ≤ 2 · alcance_max`, nunca a mais de uma célula.
//!
//! ⚠️ **E os parceiros são somados em ordem CRESCENTE de índice**, porque é essa a ordem em que o
//! laço de todos-os-pares (`i < j`, `i` por fora) entrega as contribuições a uma peça. ⇒ o gate
//! [`tests`] exige **igualdade ao bit** com a referência, e não uma tolerância que esconderia uma
//! ordem trocada.
//!
//! ## O que conta como peça
//!
//! Um elemento só entra com **colisor válido e posição finita**. Os outros não empurram nem são
//! empurrados — é a declaração ausente vista por dentro. Um peso `w = 0` (o `inv_mass` do
//! `motion.pin_constraint`) é um **obstáculo**: não se move, não roda, e os outros contornam-no.

use std::collections::BTreeMap;

use ph2d_nodegraph::attr::{
    COLLIDER_BOX_COLUMN, COLLIDER_COLUMN, COLLIDER_OFFSET_COLUMN, Column, INV_INERTIA_COLUMN,
    SIZE_IDENTITY, Stream, par_build,
};

pub mod atrito;
mod par;
mod trig;

pub use atrito::{Deslize, Material, Pecas, Saida, materiais};
pub use par::{contato, disco_caixa};

/// Abaixo disto dois centros coincidem e a normal não existe (o `EPS` do `motion.collide`).
pub(crate) const EPS: f32 = 1e-9;

/// Radianos → graus, que é a unidade da coluna `rot` (a única unidade de ângulo autorada do app).
/// Público porque o `sim.collide` faz a mesma conta do lado dele, e duas cópias divergiriam.
pub const GRAUS: f32 = 180.0 / std::f32::consts::PI;

/// **A forma de um colisor, já em unidades de MUNDO.**
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Forma {
    /// Um disco de raio `r`.
    Disco(f32),
    /// Uma caixa orientada: as MEIAS extensões e o eixo `x` dela, `[cos, sin]` unitário.
    Caixa { meia: [f32; 2], eixo: [f32; 2] },
}

/// **O colisor de uma peça**: a forma, e onde o centro dela fica em relação a `P`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Colisor {
    pub forma: Forma,
    /// O centro do colisor menos o `P` da peça, em mundo. `[0, 0]` na arte centrada.
    pub desvio: [f32; 2],
}

/// **Um contacto**: a normal unitária de `a` para `b`, a profundidade, e ONDE eles se tocam.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Contacto {
    pub normal: [f32; 2],
    pub penetracao: f32,
    /// O ponto de MUNDO onde a força age — o braço da rotação sai dele.
    pub ponto: [f32; 2],
}

impl Contacto {
    /// O braço deste contacto sobre uma peça centrada em `centro`: `ponto − centro`.
    pub fn raio(&self, centro: [f32; 2]) -> [f32; 2] {
        [self.ponto[0] - centro[0], self.ponto[1] - centro[1]]
    }

    /// **A ALAVANCA deste contacto sobre uma peça centrada em `centro`** — `(ponto − centro) × n`.
    /// Zero quando a normal passa pelo centro: ali o contacto só empurra, nunca roda.
    ///
    /// ⚠️ **Num DISCO ela é zero sempre e exactamente** (`r = ±R·n`), e é essa a conta que diz
    /// porque um círculo não podia rodar antes do atrito — [`atrito`].
    pub fn braco(&self, centro: [f32; 2]) -> f32 {
        cruz(self.raio(centro), self.normal)
    }

    /// **A TANGENTE do contacto** — `perp(n)`, a direcção em que as duas superfícies deslizam.
    pub fn tangente(&self) -> [f32; 2] {
        perp(self.normal)
    }

    /// **A ALAVANCA da TANGENTE** — `(ponto − centro) × t`, que pela identidade `r × perp(n) =
    /// r · n` é a projecção do braço na NORMAL. Máxima (`±R`) exactamente onde a de [`braco`] é
    /// zero: as duas são as duas coordenadas do mesmo vector.
    pub fn braco_tangente(&self, centro: [f32; 2]) -> f32 {
        dot(self.raio(centro), self.normal)
    }
}

impl Colisor {
    /// Um disco centrado em `P` — o colisor de antes do doc 109 §5.
    pub const fn disco(r: f32) -> Self {
        Self {
            forma: Forma::Disco(r),
            desvio: [0.0, 0.0],
        }
    }

    /// Uma caixa centrada em `P`, de meias extensões `meia`, com o eixo `x` em `eixo`.
    pub const fn caixa(meia: [f32; 2], eixo: [f32; 2]) -> Self {
        Self {
            forma: Forma::Caixa { meia, eixo },
            desvio: [0.0, 0.0],
        }
    }

    /// O centro do colisor de uma peça em `p`.
    ///
    /// ⚠️ **Sem desvio devolve `p` tal e qual**, e não `p + 0`: `−0 + 0` é `+0`, e o disco centrado
    /// é a lei que já shipou — os mesmos bits, sem excepção de sinal.
    pub fn centro(&self, p: [f32; 2]) -> [f32; 2] {
        if self.desvio == [0.0, 0.0] {
            p
        } else {
            [p[0] + self.desvio[0], p[1] + self.desvio[1]]
        }
    }

    /// **O mesmo colisor, girado `graus` à volta da origem da peça** — o que uma varredura precisa
    /// depois de a anterior ter rodado a peça. `0` devolve-o intocado, ao bit.
    pub fn girado(&self, graus: f32) -> Self {
        if graus == 0.0 {
            return *self;
        }
        let e = eixo_de(graus);
        let gira = |v: [f32; 2]| [v[0] * e[0] - v[1] * e[1], v[0] * e[1] + v[1] * e[0]];
        Self {
            forma: match self.forma {
                Forma::Disco(r) => Forma::Disco(r),
                Forma::Caixa { meia, eixo } => Forma::Caixa {
                    meia,
                    eixo: gira(eixo),
                },
            },
            desvio: if self.desvio == [0.0, 0.0] {
                self.desvio
            } else {
                gira(self.desvio)
            },
        }
    }

    /// **O ALCANCE**: o raio do círculo centrado em `P` que contém o colisor inteiro — o que decide
    /// o lado da grelha.
    pub fn alcance(&self) -> f32 {
        let longe = match self.forma {
            Forma::Disco(r) => r,
            Forma::Caixa { meia, .. } => meia[0].hypot(meia[1]),
        };
        if self.desvio == [0.0, 0.0] {
            longe
        } else {
            longe + self.desvio[0].hypot(self.desvio[1])
        }
    }

    /// **A meia-largura do colisor ao longo da direcção unitária `n`** — o suporte de um convexo
    /// simétrico, que é o que um plano precisa para pousar a peça pela FACE e não pelo centro.
    pub fn suporte(&self, n: [f32; 2]) -> f32 {
        match self.forma {
            Forma::Disco(r) => r,
            Forma::Caixa { meia, eixo } => {
                meia[0] * dot(eixo, n).abs() + meia[1] * dot(perp(eixo), n).abs()
            }
        }
    }

    /// **O INVERSO DA INÉRCIA de rotação desta forma**, para uma peça de massa inversa `w`
    /// (doc 109 §6): uma caixa de meias `h` tem `I = m·(hx² + hy²)/3`, um disco de raio `r` tem
    /// `I = m·r²/2`. Um obstáculo (`w = 0`) não roda; um colisor degenerado também não.
    pub fn inv_inercia(&self, w: f32) -> f32 {
        if !w.is_finite() || w <= 0.0 {
            return 0.0;
        }
        let i = match self.forma {
            Forma::Disco(r) => 2.0 * w / (r * r),
            Forma::Caixa { meia, .. } => 3.0 * w / (meia[0] * meia[0] + meia[1] * meia[1]),
        };
        if i.is_finite() { i } else { 0.0 }
    }

    /// **O PONTO do colisor mais avançado na direcção `−n`** — é ali que uma parede plana o toca, e
    /// é dele que sai o braço da rotação (doc 109 §6).
    ///
    /// ⚠️⚠️ **Com uma face PARALELA à parede devolve o MEIO dela**, e isso é a metade que importa:
    /// ali os dois cantos tocam à mesma profundidade, e escolher um deles daria binário a uma peça
    /// pousada de chapa — ela tombava sozinha, sem ninguém lhe tocar. *O suporte é um CONJUNTO, e o
    /// representante honesto dele é o meio.*
    ///
    /// ⛔ E o centro NÃO serve: `centro − n · suporte` cai sempre debaixo do centro, com braço zero —
    /// uma caixa inclinada nunca se endireitava (medido, `sim.collide` a `20°`).
    pub fn ponto_de_suporte(&self, p: [f32; 2], n: [f32; 2]) -> [f32; 2] {
        /// Abaixo disto o eixo é paralelo à parede (`1e-4` ≈ `0,006°` de desvio).
        const PARALELO: f32 = 1e-4;
        let c = self.centro(p);
        match self.forma {
            Forma::Disco(r) => [c[0] - n[0] * r, c[1] - n[1] * r],
            Forma::Caixa { meia, eixo } => {
                let v = perp(eixo);
                let comp = |e: [f32; 2], m: f32| {
                    let d = dot(e, n);
                    if d.abs() <= PARALELO {
                        0.0
                    } else if d > 0.0 {
                        -m
                    } else {
                        m
                    }
                };
                let (a, b) = (comp(eixo, meia[0]), comp(v, meia[1]));
                [c[0] + eixo[0] * a + v[0] * b, c[1] + eixo[1] * a + v[1] * b]
            }
        }
    }

    /// Os quatro cantos de uma caixa com `P` em `p`, no sentido anti-horário a partir de `(+, +)`;
    /// `None` para um disco.
    pub fn cantos(&self, p: [f32; 2]) -> Option<[[f32; 2]; 4]> {
        let Forma::Caixa { meia, eixo } = self.forma else {
            return None;
        };
        let (c, v) = (self.centro(p), perp(eixo));
        let canto = |sx: f32, sy: f32| {
            [
                c[0] + sx * meia[0] * eixo[0] + sy * meia[1] * v[0],
                c[1] + sx * meia[0] * eixo[1] + sy * meia[1] * v[1],
            ]
        };
        Some([
            canto(1.0, 1.0),
            canto(-1.0, 1.0),
            canto(-1.0, -1.0),
            canto(1.0, -1.0),
        ])
    }

    /// Um colisor que ocupa espaço, com números finitos.
    fn valido(&self) -> bool {
        let forma = match self.forma {
            Forma::Disco(r) => r.is_finite() && r > 0.0,
            Forma::Caixa { meia, eixo } => {
                meia.iter().chain(&eixo).all(|x| x.is_finite())
                    && meia[0] >= 0.0
                    && meia[1] >= 0.0
                    && meia[0].max(meia[1]) > 0.0
            }
        };
        forma && self.desvio.iter().all(|x| x.is_finite())
    }
}

pub(crate) fn dot(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[0] + a[1] * b[1]
}

/// A componente `z` de `a × b` — a alavanca de um braço contra uma normal.
pub(crate) fn cruz(a: [f32; 2], b: [f32; 2]) -> f32 {
    a[0] * b[1] - a[1] * b[0]
}

/// O eixo `y` de uma caixa cujo eixo `x` é `e`.
pub(crate) fn perp(e: [f32; 2]) -> [f32; 2] {
    [0.0 - e[1], e[0]]
}

/// O eixo `x` de uma peça girada `graus`, `[cos, sin]` unitário e SEM transcendentais (HR-5).
/// `0°` dá `[1, 0]` ao bit, sem passar pela aproximação.
fn eixo_de(graus: f32) -> [f32; 2] {
    if graus == 0.0 {
        return [1.0, 0.0];
    }
    let (c, s) = trig::cos_sin_cycles(graus / 360.0);
    let inv = 1.0 / (c * c + s * s).sqrt();
    [c * inv, s * inv]
}

/// **O colisor que uma linha DECLARA**, lido dos valores das colunas dela — a porta única que o
/// `sim.step`, o `sim.collide` e o gizmo do cartão da forma perguntam.
///
/// - `caixa` válida (as duas meias finitas, `≥ 0`, uma delas `> 0`) **ganha**: é a declaração de
///   uma forma com `Collider Shape = Box`.
/// - senão `raio` finito e `> 0` é um disco.
/// - senão a peça não colide — e é isto que o `[0, 0]` / `0` preenchido pela união do
///   `motion.combine` quer dizer.
///
/// De geometria para mundo: as meias escalam por `|size|` (um espelho não encolhe), o raio por
/// `max(|sx|, |sy|)` (o disco que contém a arte, a lei do `motion.collide`), e o desvio pelo `size`
/// COM sinal (um espelho leva o centro para o outro lado) e depois pela rotação `graus`.
pub fn declarado(
    raio: Option<f32>,
    caixa: Option<[f32; 2]>,
    desvio: [f32; 2],
    size: [f32; 2],
    graus: f32,
) -> Option<Colisor> {
    let eixo = eixo_de(graus);
    let caixa =
        caixa.filter(|m| m.iter().all(|x| x.is_finite() && *x >= 0.0) && m[0].max(m[1]) > 0.0);
    let forma = match caixa {
        Some(m) => Forma::Caixa {
            meia: [m[0] * size[0].abs(), m[1] * size[1].abs()],
            eixo,
        },
        None => Forma::Disco(
            raio.filter(|r| r.is_finite() && *r > 0.0)? * size[0].abs().max(size[1].abs()),
        ),
    };
    let desvio = if desvio == [0.0, 0.0] {
        [0.0, 0.0]
    } else {
        let d = [desvio[0] * size[0], desvio[1] * size[1]];
        [
            d[0] * eixo[0] - d[1] * eixo[1],
            d[0] * eixo[1] + d[1] * eixo[0],
        ]
    };
    let c = Colisor { forma, desvio };
    c.valido().then_some(c)
}

fn escalares<'a>(s: &'a Stream, nome: &str) -> Option<&'a [f32]> {
    match s.get(nome) {
        Some(Column::Scalar(v)) if v.len() == s.count() => Some(v),
        _ => None,
    }
}

fn pares<'a>(s: &'a Stream, nome: &str) -> Option<&'a [[f32; 2]]> {
    match s.get(nome) {
        Some(Column::Vec2(v)) if v.len() == s.count() => Some(v),
        _ => None,
    }
}

/// **Os colisores de um stream inteiro** — `None` quando ele não declara colisor nenhum (nem a
/// coluna do raio nem a da caixa), que é o que mantém toda cena sem colisor **byte-idêntica**: quem
/// pergunta sai antes de tocar em nada.
pub fn colisores(s: &Stream) -> Option<Vec<Option<Colisor>>> {
    let (raio, caixa) = (escalares(s, COLLIDER_COLUMN), pares(s, COLLIDER_BOX_COLUMN));
    if raio.is_none() && caixa.is_none() {
        return None;
    }
    let (desvio, size, rot) = (
        pares(s, COLLIDER_OFFSET_COLUMN),
        pares(s, "size"),
        escalares(s, "rot"),
    );
    Some(
        (0..s.count())
            .map(|i| {
                declarado(
                    raio.map(|v| v[i]),
                    caixa.map(|v| v[i]),
                    desvio.map_or([0.0, 0.0], |v| v[i]),
                    size.map_or(SIZE_IDENTITY, |v| v[i]),
                    rot.map_or(0.0, |v| v[i]),
                )
            })
            .collect(),
    )
}

/// **Quanto cada peça RODA por unidade de binário** (doc 109 §6) — a coluna
/// [`INV_INERTIA_COLUMN`] quando ela existe (`0` = travada pelo cartão), senão derivada da forma e
/// do peso. A porta única: o `sim.step` e o `sim.collide` perguntam a mesma coisa.
pub fn inv_inercias(s: &Stream, colisores: &[Option<Colisor>], pesos: &[f32]) -> Vec<f32> {
    let coluna = escalares(s, INV_INERTIA_COLUMN);
    (0..colisores.len())
        .map(|i| match coluna.and_then(|v| v.get(i)) {
            Some(x) if x.is_finite() => x.max(0.0),
            _ => colisores[i].map_or(0.0, |c| c.inv_inercia(*pesos.get(i).unwrap_or(&1.0))),
        })
        .collect()
}

/// O que uma varredura decidiu para uma peça: a posição nova, o giro em graus e o salto do par
/// mais vivo que ela tocou.
type Nova = Option<([f32; 2], f32, f32)>;

/// Confere que toda coluna tem o comprimento da nuvem.
fn confere(n: usize, saida: &Saida<'_>, pecas: &Pecas<'_>) {
    assert_eq!(saida.giro.len(), n, "um giro por peca");
    assert_eq!(saida.salto.len(), n, "um salto por peca");
    assert_eq!(pecas.colisores.len(), n, "um colisor por peca");
    assert_eq!(pecas.pesos.len(), n, "um peso por peca");
    assert_eq!(pecas.inv_inercia.len(), n, "uma inercia por peca");
    if let Some(d) = pecas.deslize {
        assert_eq!(d.antes.len(), n, "um antes por peca");
        assert_eq!(d.girou_antes.len(), n, "um giro anterior por peca");
        assert_eq!(d.material.len(), n, "um material por peca");
    }
}

/// Afasta as peças sobrepostas, `varreduras` vezes. `p` é reescrito no sítio; a [`Saida`] ACUMULA
/// em GRAUS o quanto cada peça rodou (doc 109 §6) e recolhe o salto de cada uma (§7).
///
/// # Panics
///
/// Se alguma coluna da [`Saida`] ou das [`Pecas`] não tiver o comprimento de `p` — colunas de uma
/// mesma corrente com comprimentos diferentes não são uma pergunta com resposta.
pub fn separate(p: &mut [[f32; 2]], saida: &mut Saida<'_>, pecas: &Pecas<'_>, varreduras: usize) {
    let n = p.len();
    confere(n, saida, pecas);
    let ativo: Vec<bool> = (0..n)
        .map(|i| ativo(p[i], pecas.colisores[i].as_ref()))
        .collect();
    let alcance_max = (0..n)
        .filter(|&i| ativo[i])
        .filter_map(|i| pecas.colisores[i].map(|c| c.alcance()))
        .fold(0.0_f32, f32::max);
    if alcance_max <= 0.0 {
        return;
    }
    let lado = 2.0 * alcance_max;
    for _ in 0..varreduras {
        let foto = p.to_vec();
        // As formas COMO ESTÃO: o que as varreduras anteriores rodaram já conta.
        let girado = saida.giro.to_vec();
        let agora: Vec<Option<Colisor>> = (0..n)
            .map(|i| pecas.colisores[i].map(|c| c.girado(girado[i])))
            .collect();
        let grelha = grelha(&foto, &ativo, lado);
        let novas: Vec<Nova> = par_build(n, |k| {
            if !ativo[k] {
                return None;
            }
            let (cx, cy) = celula(foto[k], lado);
            let mut parceiros: Vec<usize> = Vec::new();
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if let Some(v) = grelha.get(&(cx + dx, cy + dy)) {
                        parceiros.extend_from_slice(v);
                    }
                }
            }
            parceiros.sort_unstable();
            corrigida(
                k,
                parceiros.into_iter(),
                &foto,
                &girado,
                &agora,
                pecas,
                &ativo,
            )
        });
        aplica(p, saida, novas);
    }
}

/// **A referência**: a mesma lei por todos-os-pares, na ordem do laço `i < j`. `O(n²)`.
///
/// Pública para que o gate de cada cliente possa comparar-se com ela; nenhum caminho de produto a
/// chama.
pub fn separate_all_pairs(
    p: &mut [[f32; 2]],
    saida: &mut Saida<'_>,
    pecas: &Pecas<'_>,
    varreduras: usize,
) {
    let n = p.len();
    confere(n, saida, pecas);
    let ativo: Vec<bool> = (0..n)
        .map(|i| ativo(p[i], pecas.colisores[i].as_ref()))
        .collect();
    for _ in 0..varreduras {
        let foto = p.to_vec();
        let girado = saida.giro.to_vec();
        let agora: Vec<Option<Colisor>> = (0..n)
            .map(|i| pecas.colisores[i].map(|c| c.girado(girado[i])))
            .collect();
        let novas: Vec<Nova> = (0..n)
            .map(|k| {
                if ativo[k] {
                    corrigida(k, 0..n, &foto, &girado, &agora, pecas, &ativo)
                } else {
                    None
                }
            })
            .collect();
        aplica(p, saida, novas);
    }
}

/// Escreve o que uma varredura produziu.
fn aplica(p: &mut [[f32; 2]], saida: &mut Saida<'_>, novas: Vec<Nova>) {
    for (k, nova) in novas.into_iter().enumerate() {
        if let Some((q, g, s)) = nova {
            p[k] = q;
            saida.giro[k] += g;
            saida.salto[k] = saida.salto[k].max(s);
        }
    }
}

fn ativo(p: [f32; 2], c: Option<&Colisor>) -> bool {
    p[0].is_finite() && p[1].is_finite() && c.is_some_and(Colisor::valido)
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "uma celula de grelha; uma coordenada fora de i64 satura, e so' agrupa pior"
)]
fn celula(p: [f32; 2], lado: f32) -> (i64, i64) {
    ((p[0] / lado).floor() as i64, (p[1] / lado).floor() as i64)
}

/// As peças activas por célula, cada lista em ordem crescente de índice.
fn grelha(foto: &[[f32; 2]], ativo: &[bool], lado: f32) -> BTreeMap<(i64, i64), Vec<usize>> {
    let mut g: BTreeMap<(i64, i64), Vec<usize>> = BTreeMap::new();
    for (i, q) in foto.iter().enumerate() {
        if ativo[i] {
            g.entry(celula(*q, lado)).or_default().push(i);
        }
    }
    g
}

/// A posição, o giro e o salto de `k` depois desta varredura, ou `None` se nada lhe tocou. Os `parceiros` têm
/// de vir em ordem CRESCENTE — ver o cabeçalho.
fn corrigida(
    k: usize,
    parceiros: impl Iterator<Item = usize>,
    foto: &[[f32; 2]],
    girado: &[f32],
    colisores: &[Option<Colisor>],
    pecas: &Pecas<'_>,
    ativo: &[bool],
) -> Nova {
    let (pesos, inv_inercia) = (pecas.pesos, pecas.inv_inercia);
    let mut delta = [0.0_f32; 2];
    let mut giro = 0.0_f32;
    let mut salto = 0.0_f32;
    let mut contatos = 0_u32;
    for j in parceiros {
        if j == k || !ativo[j] {
            continue;
        }
        let (lo, hi) = (k.min(j), k.max(j));
        let (Some(clo), Some(chi)) = (colisores[lo], colisores[hi]) else {
            continue;
        };
        // O par na ordem do PAR (ver o cabeçalho): a normal vai do menor para o maior.
        let Some(c) = contato(&clo, foto[lo], &chi, foto[hi], (lo + hi) % 2 == 0) else {
            continue;
        };
        // O centro de cada lado, e daí a massa efectiva no PONTO do contacto (doc 109 §6).
        let (ck, cj) = if k == lo {
            (clo.centro(foto[lo]), chi.centro(foto[hi]))
        } else {
            (chi.centro(foto[hi]), clo.centro(foto[lo]))
        };
        let (bk, bj) = (c.braco(ck), c.braco(cj));
        let massa = |w: f32, inv_i: f32, b: f32| w + inv_i * b * b;
        let soma = massa(pesos[k], inv_inercia[k], bk) + massa(pesos[j], inv_inercia[j], bj);
        // Dois obstáculos (ou dois pesos infinitos) não têm correcção a repartir.
        if soma <= 0.0 {
            continue;
        }
        let lambda = c.penetracao / soma;
        let sinal = if k == lo { -1.0 } else { 1.0 };
        let empurra = lambda * pesos[k] * sinal;
        delta[0] += c.normal[0] * empurra;
        delta[1] += c.normal[1] * empurra;
        giro += bk * lambda * inv_inercia[k] * sinal * GRAUS;
        // ⭐⭐⭐ **A METADE TANGENCIAL** (doc 109 §7) — o deslize desfeito, limitado por Coulomb.
        // ⚠️ Sem `sinal`: o deslize já é medido **de `k` para `j`**, então a correcção dele é
        // simétrica por construção e os dois lados do par concordam sem desempate nenhum.
        if let Some(d) = pecas.deslize {
            let (mk, mj) = (pecas.material(k), pecas.material(j));
            salto = salto.max(atrito::salto(mk.salto, mj.salto));
            let t = c.tangente();
            let (tk, tj) = (c.braco_tangente(ck), c.braco_tangente(cj));
            // Quanto o PONTO de contacto de cada lado andou desde o início do passo: o centro
            // MAIS o que a rotação do corpo lhe acrescentou (`dθ × r`, em 2D `dθ · perp(r)`).
            let andou = |i: usize, centro: [f32; 2]| {
                let dtheta = (girado[i] + d.girou_antes[i]) / GRAUS;
                let r = c.raio(centro);
                [
                    foto[i][0] - d.antes[i][0] - dtheta * r[1],
                    foto[i][1] - d.antes[i][1] + dtheta * r[0],
                ]
            };
            let (ak, aj) = (andou(k, ck), andou(j, cj));
            let desliza = dot([ak[0] - aj[0], ak[1] - aj[1]], t);
            let soma_t = massa(pesos[k], inv_inercia[k], tk) + massa(pesos[j], inv_inercia[j], tj);
            let lt = atrito::lambda(desliza, soma_t, atrito::mu(mk.atrito, mj.atrito), lambda);
            delta[0] -= t[0] * lt * pesos[k];
            delta[1] -= t[1] * lt * pesos[k];
            giro -= tk * lt * inv_inercia[k] * GRAUS;
        }
        contatos += 1;
    }
    (contatos > 0).then(|| {
        #[expect(
            clippy::cast_precision_loss,
            reason = "uma contagem de contatos de uma peca, muito abaixo de 2^24"
        )]
        let inv = 1.0 / contatos as f32;
        (
            [foto[k][0] + delta[0] * inv, foto[k][1] + delta[1] * inv],
            giro * inv,
            salto,
        )
    })
}

#[cfg(test)]
mod tests;
