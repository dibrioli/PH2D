//! ⭐⭐ **A corda LÊ o colisor que as suas peças declaram** (doc 125 §1, Bug #16) — o mesmo
//! caminho do corpo mole (doc 124 §6): as peças entram pela porta `shape` (`Shape → Rope`), e com
//! o `Collide` do Shape ligado a corrente traz a DECLARAÇÃO do colisor. As peças distribuem-se
//! pelos nós como o `Cycle` do Duplicator (`nó i ← peça i mod m`): uma forma só veste a corda
//! inteira.
//!
//! O contacto corre DENTRO do laço de restrições, intercalado com as `iterations` (o que o Vellum
//! faz), e sub-tica com o `Substeps` da corda — o relógio do grafo não muda (doc 125 §1).
//!
//! Os pares que o REPOUSO já sobrepõe (os vizinhos da corrente) são da corda, não do contacto
//! ([`ph2d_contact::Ligados`]).

use ph2d_contact::{Colisor, Pecas, Saida};
use ph2d_nodegraph::attr::{Column, Stream};

/// O que as peças declararam, por nó, e o repouso que filtra os vizinhos.
pub(crate) struct Contacto {
    colisores: Vec<Option<Colisor>>,
    repouso: Vec<[f32; 2]>,
    corpo: Vec<f32>,
    /// O menor alcance declarado — a régua do passo seguro.
    alcance_min: f32,
    pub(crate) ritmo: Ritmo,
}

/// ⭐ **O PASSO SEGURO** (doc 125 §1.2): com o contacto declarado, nenhum nó anda mais que MEIO
/// alcance por sub-passo — os sub-passos sobem até isso (a condição CFL do Houdini), entre o
/// `Substeps` do artista e o [`MAX_SUBSTEPS`](super::MAX_SUBSTEPS). Medido no chicote da fixtura:
/// `0,9659·D` na 2.ª metade e o comprimento `1,01×` o da corda nua, a `2,9` sub-passos por quadro
/// em média; `1` alcance segura `0,92·D` mas estica `1,11×`, `¼` custa `1,8×` sem ganhar nada.
pub(crate) const PASSO_SEGURO: f32 = 0.5;

/// ONDE o contacto corre no passo. O produto corre o [`Ritmo::Cfl`]; os outros são as variantes
/// que a rodada do doc 125 §1.2 mediu e recusou, vivas só na sonda que a reproduz.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Ritmo {
    /// Por iteração, e os sub-passos sobem até cada nó andar no máximo `k` alcances; a âncora anda
    /// ao longo deles.
    Cfl(f32),
    /// O [`Ritmo::Cfl`] com a âncora a saltar o tique inteiro no 1.º sub-passo.
    #[cfg(test)]
    CflAncoraInteira(f32),
    /// Uma varredura a cada iteração de restrições, sem passo seguro: segura, mas estica `1,23×`.
    #[cfg(test)]
    PorIteracao,
    /// Uma varredura depois das iterações de cada sub-passo (o corpo mole): não segura (`0,73` a 8).
    #[cfg(test)]
    PorSubPasso,
    /// Por iteração, e o passo de cada nó é cortado a `k` alcances (o `maxSpeed` do FleX): muda o
    /// chicote.
    #[cfg(test)]
    VelocidadeMax(f32),
}

impl Contacto {
    /// `None` quando a porta não declara colisor nenhum — a corda de sempre, ao bit.
    pub(crate) fn de(forma: &Stream, n: usize, repouso: Vec<[f32; 2]>, corpo: f32) -> Option<Self> {
        let declarados = ph2d_contact::colisores(forma)?;
        let m = declarados.len();
        if m == 0 || !declarados.iter().any(Option::is_some) {
            return None;
        }
        let colisores: Vec<Option<Colisor>> = (0..n).map(|i| declarados[i % m]).collect();
        let alcance_min = colisores
            .iter()
            .flatten()
            .map(Colisor::alcance)
            .fold(f32::MAX, f32::min);
        Some(Self {
            colisores,
            repouso,
            corpo: vec![corpo; n],
            alcance_min,
            ritmo: Ritmo::Cfl(PASSO_SEGURO),
        })
    }

    pub(crate) fn por_iteracao(&self) -> bool {
        #[cfg(test)]
        if self.ritmo == Ritmo::PorSubPasso {
            return false;
        }
        true
    }

    /// Quantos sub-passos o tique leva: o do artista, ou mais se o [`Ritmo::Cfl`] o pedir.
    pub(crate) fn sub_passos(&self, artista: usize, deslocamento: f32, tecto: usize) -> usize {
        match self.ritmo {
            #[cfg(test)]
            Ritmo::CflAncoraInteira(k) => self.cfl(k, artista, deslocamento, tecto),
            Ritmo::Cfl(k) => self.cfl(k, artista, deslocamento, tecto),
            #[cfg(test)]
            _ => artista,
        }
    }

    fn cfl(&self, k: f32, artista: usize, deslocamento: f32, tecto: usize) -> usize {
        if self.alcance_min <= 0.0 {
            return artista;
        }
        let pedido = (deslocamento / (k * self.alcance_min)).ceil();
        #[expect(clippy::cast_possible_truncation, reason = "limitado ao tecto")]
        #[expect(clippy::cast_sign_loss, reason = "um quociente nao negativo")]
        let pedido = pedido.clamp(1.0, tecto as f32) as usize;
        artista.max(pedido)
    }

    /// A âncora anda ao longo dos sub-passos (doc 125 §1.3)?
    pub(crate) fn interpola_a_ancora(&self) -> bool {
        #[cfg(test)]
        if let Ritmo::CflAncoraInteira(_) = self.ritmo {
            return false;
        }
        true
    }

    /// O maior passo de um nó por sub-passo, se o [`Ritmo::VelocidadeMax`] o cortar.
    pub(crate) fn passo_max(&self) -> Option<f32> {
        #[cfg(test)]
        if let Ritmo::VelocidadeMax(k) = self.ritmo {
            return Some(k * self.alcance_min);
        }
        None
    }

    /// Separa os nós no sítio, uma varredura. Um nó de peso `0` (a cabeça, a cauda presa, um pino)
    /// é um obstáculo: não se move, e os outros arrumam-se à volta dele.
    pub(crate) fn separa(&self, pos: &mut [[f32; 2]], pesos: &[f32]) {
        let n = pos.len();
        let inercia = vec![0.0f32; n];
        let mut giro = vec![0.0f32; n];
        ph2d_contact::separate(
            pos,
            &mut Saida { giro: &mut giro },
            &Pecas::novas(&self.colisores, pesos, &inercia).com_ligados(&self.repouso, &self.corpo),
            1,
        );
    }
}

/// Quantos sub-passos o tique leva: o `Substeps` do artista, ou o que o contacto pedir.
pub(super) fn sub_passos(
    pos: &[[f32; 2]],
    prev: &[[f32; 2]],
    anchor: [f32; 2],
    p: &super::Params,
    contacto: Option<&Contacto>,
) -> usize {
    contacto.map_or(p.substeps.max(1), |c| {
        // O maior deslocamento que o tique traz: o da cabeça arrastada e o de cada nó no tique
        // anterior (a velocidade de Verlet).
        let mut d = pos.first().map_or(0.0, |h| dist(anchor, *h));
        for (a, b) in pos.iter().zip(prev) {
            d = d.max(dist(*a, *b));
        }
        c.sub_passos(p.substeps.max(1), d, super::MAX_SUBSTEPS as usize)
    })
}

pub(super) fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// ⭐ **Os alvos dos pinos no sub-passo `s` de `sub`** (doc 125 §1.3): com o contacto a âncora ANDA
/// ao longo dos sub-passos — saltar o tique inteiro no 1.º passava a cabeça através das contas
/// vizinhas de uma corda fina. ⚠️ O ÚLTIMO usa a âncora EXACTA (`c + (a − c)·1` não é `a` ao bit),
/// e sem contacto ela salta como sempre (a corda de sempre ao bit).
pub(super) fn alvos(
    contacto: Option<&Contacto>,
    cabeca: [f32; 2],
    anchor: [f32; 2],
    cauda: [f32; 2],
    s: usize,
    sub: usize,
) -> ([f32; 2], [f32; 2]) {
    if !contacto.is_some_and(Contacto::interpola_a_ancora) || s + 1 >= sub {
        return (anchor, cauda);
    }
    #[expect(clippy::cast_precision_loss, reason = "sub-passos, no maximo 16")]
    let f = (s + 1) as f32 / sub as f32;
    let a = [
        cabeca[0] + (anchor[0] - cabeca[0]) * f,
        cabeca[1] + (anchor[1] - cabeca[1]) * f,
    ];
    (
        a,
        [cauda[0] - anchor[0] + a[0], cauda[1] - anchor[1] + a[1]],
    )
}

/// As colunas que a corda ESCREVE ou LÊ do próprio estado — nunca vêm da aparência das peças.
const DA_CORDA: &[&str] = &[
    "P",
    "rope_prev",
    "parent",
    "sim_t",
    "accel",
    "inv_mass",
    "rest",
    "body",
];

/// ⭐ **A corda passa adiante a APARÊNCIA das peças** (como o corpo mole, doc 124 §6): as colunas
/// da porta `shape` que não são dela — a forma, o tamanho, a cor, o colisor declarado — viajam com
/// cada nó, distribuídas como o `Cycle` do Duplicator, e a corda desenha-se sem um Duplicator.
pub(crate) fn veste(out: &mut Stream, forma: &Stream) {
    let (n, m) = (out.count(), forma.count());
    if m == 0 {
        return;
    }
    for (nome, col) in forma.columns() {
        if DA_CORDA.contains(&nome.as_str()) || out.get(nome).is_some() {
            continue;
        }
        if let Some(c) = distribui(col, n, m) {
            out.set(nome.clone(), c);
        }
    }
}

/// `nó i ← linha i mod m`.
fn distribui(col: &Column, n: usize, m: usize) -> Option<Column> {
    fn ciclo<T: Clone>(v: &[T], n: usize, m: usize) -> Option<Vec<T>> {
        (v.len() == m).then(|| (0..n).map(|i| v[i % m].clone()).collect())
    }
    Some(match col {
        Column::Scalar(v) => Column::Scalar(ciclo(v, n, m)?),
        Column::Vec2(v) => Column::Vec2(ciclo(v, n, m)?),
        Column::Vec3(v) => Column::Vec3(ciclo(v, n, m)?),
        Column::Vec4(v) => Column::Vec4(ciclo(v, n, m)?),
    })
}
