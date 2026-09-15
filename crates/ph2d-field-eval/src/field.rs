//! ⭐⭐⭐ **O DOCUMENTO AVALIADO PONTO A PONTO** — a porta das sondas e dos gates.
//!
//! ⚠️ **Ela NÃO é o caminho do traçado.** Quem desenha é o [`crate::hybrid`], em `f32` e em lote;
//! esta responde **um ponto de cada vez**, em `f64`, que é o que uma régua de `‖∇f‖` por diferença
//! central precisa.
//!
//! ⚠️ **Ela mora aqui e não no `lib.rs` porque o tecto de LOC da workspace o obrigou** (2026-09-15,
//! `708` contra `700`) — e a fronteira que ele forçou é a certa: o `lib.rs` é **o compilador** (a
//! árvore, as booleanas, a pose, o compilador de regiões) e isto é **um avaliador**. *Um tecto por
//! ficheiro cura-se por responsabilidade, nunca por uma isenção.*

use crate::{compile, point_tape, wgsl};
use fidget::context::Tree;
use ph2d_field::FieldDoc;

/// Um documento avaliado ponto a ponto — a porta que as sondas e os gates usam.
///
/// ⚠️ **Ela NÃO é o caminho do traçado.** Quem desenha é o [`hybrid`], em `f32` e em lote; esta
/// responde **um ponto de cada vez**, em `f64`, que é o que uma régua de `‖∇f‖` por diferença
/// central precisa. Ver [`point_tape`] para o preço que essa escolha custava e como ele saiu.
pub struct Field {
    /// ⚠️ `pub(crate)` e não privado: a [`crate::owners_wgsl`] escreve o WGSL de **uma folha por
    /// dono** e lê a fita directamente. Enquanto isto viveu no `lib.rs` eram o mesmo módulo, e a
    /// visibilidade era um acidente do sítio — *mover um item TORNA EXPLÍCITO quem já o lia*.
    pub(crate) tape: point_tape::PointTape,
}

impl Field {
    #[must_use]
    pub fn new(doc: &FieldDoc) -> Self {
        Self::from_tree(&compile(doc))
    }

    /// ⭐⭐⭐ **O MESMO campo, com o escalonamento da fita como PARÂMETRO** —
    /// ver [`tape_schedule`]. ⚠️ Só a sonda passa `false`; o produto é sempre escalonado.
    #[doc(hidden)]
    #[must_use]
    pub fn new_com(doc: &FieldDoc, escalonar: bool) -> Self {
        let tree = compile(doc);
        let mut ctx = fidget::context::Context::new();
        let root = ctx.import(&tree);
        Self {
            tape: point_tape::PointTape::build_com(
                &ctx,
                root,
                &std::collections::BTreeMap::new(),
                escalonar,
            ),
        }
    }

    #[must_use]
    pub fn from_tree(tree: &Tree) -> Self {
        let mut ctx = fidget::context::Context::new();
        let root = ctx.import(tree);
        // ⭐ O `Context` morre aqui: a fita já traz o grafo achatado, e guardá-lo seria lastro.
        Self {
            tape: point_tape::PointTape::build(&ctx, root),
        }
    }

    /// ⭐⭐⭐ **A fita deste documento escrita em WGSL** — ver [`wgsl::TapeWgsl`].
    #[must_use]
    pub fn tape_wgsl(&self) -> Option<wgsl::TapeWgsl> {
        self.tape.to_wgsl()
    }

    /// ⭐⭐⭐ **O retrato da fita deste documento** — ver [`point_tape::TapeShape`]. É ele que diz se
    /// um interpretador de GPU cabe: o `vivos` é o scratch **por thread**.
    #[must_use]
    pub fn tape_shape(&self) -> Option<point_tape::TapeShape> {
        self.tape.shape()
    }

    /// `f(x, y, z)`. `NaN` se a árvore não puder ser avaliada ali.
    #[must_use]
    pub fn at(&self, x: f64, y: f64, z: f64) -> f64 {
        self.tape.eval(x, y, z)
    }

    /// ⭐⭐⭐ **`f` em MUITOS pontos, em faixas** — a porta de quem varre uma grelha.
    ///
    /// ⚠️ **Ela existe por uma medição, e a medição é surpreendente:** numa varredura do censo,
    /// **`79 %`** das avaliações não são gradientes — são o **teste de banda** (*este ponto está
    /// perto da superfície?*), um `at` por ponto de uma grelha de `78³`. Perguntados um a um, cada um
    /// paga a descodificação inteira da fita.
    ///
    /// ⭐ A resposta é **bit-a-bit** a de `at` ponto a ponto, pela mesma razão que a do
    /// `gradient_norm`: faixas independentes, mesmos operandos, mesma aritmética. Há gate.
    pub fn at_many(&self, pts: &[[f64; 3]], out: &mut Vec<f64>) {
        self.tape.eval_slice(pts, out);
    }

    /// `‖∇f‖` por diferença central — a medida de quanto o campo ainda é uma **distância**.
    ///
    /// ⭐⭐ **As seis amostras vão numa passagem só** ([`point_tape::PointTape::eval_many`]): elas
    /// percorrem a MESMA fita, e descodificá-la seis vezes era trabalho repetido. A resposta é
    /// bit-a-bit a de seis chamadas ao [`Field::at`] — há gate.
    ///
    /// ⚠️ **A ordem das seis é load-bearing** e é a mesma de sempre (`+x, −x, +y, −y, +z, −z`): a
    /// subtracção que vem a seguir é em vírgula flutuante, e trocar quem é o minuendo mudaria bits.
    #[must_use]
    pub fn gradient_norm(&self, x: f64, y: f64, z: f64, eps: f64) -> f64 {
        let v = self.tape.eval_many(&[
            [x + eps, y, z],
            [x - eps, y, z],
            [x, y + eps, z],
            [x, y - eps, z],
            [x, y, z + eps],
            [x, y, z - eps],
        ]);
        let gx = (v[0] - v[1]) / (2.0 * eps);
        let gy = (v[2] - v[3]) / (2.0 * eps);
        let gz = (v[4] - v[5]) / (2.0 * eps);
        (gx * gx + gy * gy + gz * gz).sqrt()
    }
}
