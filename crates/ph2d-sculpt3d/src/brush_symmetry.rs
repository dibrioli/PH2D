//! **OS ESPELHOS QUE MULTIPLICAM UM DAB** — irmão do [`super::brush`], cortado
//! por ASSUNTO.
//!
//! O pai responde *o que uma ferramenta É* (o verbo, o raio, a força, o alpha, o
//! material do traço); aqui mora *quantas vezes um dab acontece*, que é uma
//! propriedade da SESSÃO e não do pincel — e é por isso que a simetria é
//! aplicada na lista de dabs, num ponto único.

/// Os espelhos que multiplicam um dab.
///
/// ⚠️ **A simetria é aplicada na LISTA DE DABS, num ponto único** — nunca dentro
/// de um verbo. É o que faz um verbo novo herdá-la de graça, e é literalmente a
/// lição que o `stamp_dabs_inner` do Painter 2D deixou escrita.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Symmetry {
    pub x: bool,
    pub y: bool,
    pub z: bool,
}

impl Symmetry {
    /// Só o eixo X — o caso que 95% da escultura de personagem usa.
    pub const MIRROR_X: Self = Self {
        x: true,
        y: false,
        z: false,
    };

    #[must_use]
    pub fn any(self) -> bool {
        self.x || self.y || self.z
    }

    /// Os sinais por eixo de cada cópia, incluindo o original. De 1 a 8
    /// entradas; a primeira é sempre `[1, 1, 1]`.
    ///
    /// Devolve um array fixo + comprimento para não alocar por dab — um traço
    /// emite dezenas de dabs por segundo, e este é o caminho quente.
    #[must_use]
    pub fn signs(self) -> ([[f32; 3]; 8], usize) {
        let mut out = [[1.0f32; 3]; 8];
        let mut n = 1;
        for (axis, on) in [self.x, self.y, self.z].into_iter().enumerate() {
            if !on {
                continue;
            }
            for i in 0..n {
                let mut s = out[i];
                s[axis] = -s[axis];
                out[n + i] = s;
            }
            n *= 2;
        }
        (out, n)
    }
}
