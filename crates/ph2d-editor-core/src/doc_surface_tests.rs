//! «Quer outro quadro»: uma família com trabalho de fundo diz-o, e o conjunto responde por ela.

use super::*;
use crate::documents::Documents;

/// Uma família mínima: `Busy` tem uma capa a caminho; `Quiet` fica com o `wants_frame` do contrato.
macro_rules! family {
    ($t:ident { $($extra:tt)* }) => {
        struct $t;
        impl DocumentSurface for $t {
    fn tabs(&self) -> Vec<DocTab> {
        Vec::new()
    }
    fn paint(&mut self, _: u64, _: Rect, _: &mut VectorScene, _: &mut TextSystem, _: &Theme) {}
    fn pointer(
        &mut self,
        _: u64,
        _: PointerInput<'_>,
        _: PointerKind,
        _: PointerButton,
        _: f32,
        _: f32,
    ) -> bool {
        false
    }
    fn pointer_move(&mut self, _: u64, _: PointerInput<'_>, _: f32, _: f32) -> bool {
        false
    }
    fn wheel(&mut self, _: u64, _: f32, _: f32, _: f32, _: bool) -> bool {
        false
    }
    fn key(&mut self, _: u64, _: &mut TextSystem, _: KeyInput<'_>) -> KeyOutcome {
        KeyOutcome::default()
    }
    fn compose(&mut self, _: u64, _: Compose<'_>) -> bool {
        false
    }
    fn close(&mut self, _: u64) {}
    fn undo(&mut self, _: u64, _: bool) -> bool {
        false
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    $($extra)*
        }
    };
}

family!(Quiet {});
family!(Busy {
    fn wants_frame(&self) -> bool {
        true
    }
});

/// ⭐ Uma família com trabalho de fundo pede outro quadro pelo conjunto; sem ela, ninguém o pede.
#[test]
fn a_family_with_background_work_asks_for_another_frame() {
    let mut docs = Documents::default();
    assert!(
        !docs.surfaces().wants_frame(),
        "sem famílias, nenhum quadro extra"
    );
    docs.install_surface(Box::new(Quiet));
    assert!(
        !docs.surfaces().wants_frame(),
        "o contrato, por omissão, não pede"
    );
    docs.install_surface(Box::new(Busy));
    assert!(
        docs.surfaces().wants_frame(),
        "a família ocupada pede, mesmo sem aba activa"
    );
}
