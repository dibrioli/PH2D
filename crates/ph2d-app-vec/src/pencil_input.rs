//! **A MÃO do lápis** — o estabilizador, entre o ponteiro e o lápis.
//!
//! O porquê e a história: `docs/Vector Module/50_cabecalhos_do_app_vec.md` § `pencil_input.rs`.

/// A mão filtrada do lápis: a posição corrida do estabilizador.
///
/// Vive na shell porque é a shell que possui a ENTRADA (o `input_dispatch` é quem vê o ponteiro), e
/// porque o filtro mora numa crate que o lápis não pode ver. O lápis continua a receber pontos e a
/// não saber de onde vêm — é o que o mantém puro e testável sem janela.
#[derive(Default)]
pub struct PencilHand {
    /// A posição FILTRADA corrente, em px de tela.
    stab: [f32; 2],
    /// O gesto é um nó do Edit Shape (F7e): o nó segue o ponteiro CRU — o estabilizador é do
    /// traço, e um nó atrasado do dedo é um nó que mente onde está.
    raw: bool,
}

/// **O press do lápis** (a shell chama-o com o ponteiro em mundo): com os nós do Edit Shape à
/// vista, um nó ou o contorno debaixo dele é AGARRADO; senão o Edit Shape fecha e é um traço novo.
pub fn press(
    vec: &mut crate::state::VecState,
    scene: &mut ph2d_vec_scene::VecScene,
    w: [f64; 2],
    px_to_world: f64,
    dyn_in: ph2d_vec_edit::pencil_width::PenDynamics,
    shift: bool,
) {
    let xf = vec
        .pencil
        .shape_edit_path()
        .map_or(ph2d_vec_scene::Xform::IDENTITY, |id| vec.pen.xf(id));
    let grabbed = vec
        .pencil
        .shape_edit_press(scene, w, px_to_world, xf, shift);
    vec.pencil_hand.raw = grabbed;
    if !grabbed {
        vec.pencil.on_press(scene, w, px_to_world, dyn_in);
    }
}

impl PencilHand {
    /// **O press semeia a mão.** Sem isto o 1º move mistura a partir da posição em que o gesto
    /// ANTERIOR acabou, e o traço nasce com um salto vindo do outro lado da tela.
    ///
    /// Semear (em vez de guardar `Option` e limpar no release) é deliberado: o release e o cancel
    /// teriam de lembrar-se de limpar, e um 3º caminho de morte nasceria sem a limpeza. Um valor
    /// obsoleto aqui nunca é lido, porque o filtro só corre com um gesto vivo e todo gesto começa
    /// por um press.
    pub fn begin(&mut self, px: (f32, f32)) {
        self.stab = [px.0, px.1];
    }

    /// **A amostra que o lápis de facto vê.** `strength` 0 devolve o ponteiro cru (a igualdade é
    /// exata — o `lazy_mouse_step` faz early-return), então o slider no mínimo é o produto de
    /// antes desta wave, ao bit.
    pub fn filter(&mut self, px: (f32, f32), strength: f32) -> (f32, f32) {
        if self.raw {
            self.stab = [px.0, px.1];
            return px;
        }
        self.stab = ph2d_painter_brush::lazy_mouse_step(self.stab, [px.0, px.1], strength);
        (self.stab[0], self.stab[1])
    }
}

// ⚠️ **O gate do QUADRO do lápis** — ele vivia solto no `main.rs` da shell (o subject dele é *«o
// gesto contra a ordem REAL do quadro»*) e do lado de cá precisa de um pai: pende do módulo cuja
// lei ele exercita.
#[cfg(test)]
#[path = "pencil_frame_tests.rs"]
mod pencil_frame_tests;

#[cfg(test)]
#[path = "pencil_input_tests.rs"]
mod tests;
