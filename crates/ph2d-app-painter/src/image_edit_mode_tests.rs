//! Os gates de Image ▸ Edit — a família pelo MESMO quadro que a shell corre (`mode_drive::drive`) e o
//! espelho que a fila das ferramentas de imagem lê.

use super::*;
use ph2d_editor_core::object_mode::ModeRequest;
use ph2d_editor_core::screens::hero::mode_drive::{drive, refused};
use ph2d_editor_core::toast::ToastQueue;

const A: u64 = 10;
const B: u64 = 11;
const VEC: u64 = 50;

fn kind(b: u64) -> ObjectKind {
    match b {
        A | B => ObjectKind::Image,
        VEC => ObjectKind::Vector,
        _ => ObjectKind::Empty,
    }
}

struct Cena {
    hero: HeroScreen,
    tools: ToolRegistry,
    toasts: ToastQueue,
    images: Vec<u64>,
}

fn cena() -> Cena {
    ph2d_editor_core::test_support::ensure_panel_registry();
    Cena {
        hero: HeroScreen::new(ph2d_editor_core::NodeId(1)),
        tools: ToolRegistry::new(),
        toasts: ToastQueue::new(),
        images: vec![A, B],
    }
}

impl Cena {
    /// O quadro da shell: as famílias da imagem (Paint e Edit), e o espelho depois.
    fn quadro(&mut self, req: Option<ModeRequest>) {
        let mut edit = Family::of(self.images.clone());
        let images = self.images.clone();
        let kind = move |b: u64| {
            if images.contains(&b) {
                ObjectKind::Image
            } else {
                kind(b)
            }
        };
        drive(
            &mut [&mut crate::paint_mode::Family, &mut edit],
            &kind,
            &|_| String::new(),
            &mut self.tools,
            &mut self.hero,
            &mut self.toasts,
            req,
        );
        let images = self.images.clone();
        mirror(&mut self.hero, |b| images.contains(&b));
    }
    fn em_edit(&self) -> Option<u64> {
        self.hero
            .gizmo
            .mode
            .active()
            .filter(|a| a.mode == ObjectMode::Edit)
            .map(|a| a.entity)
    }
}

/// ⭐⭐ GATE (dono, 05/10: o IMG *«vira modo da imagem»*) — **o seletor de uma imagem oferece Object ·
/// Paint · Edit**, e o Edit pedido pelo seletor liga as ferramentas de imagem (o espelho); o `Tab`
/// de volta desliga-as. CONTROLO: sem imagem activa o espelho fica desligado e o Edit é recusado.
/// (Mutação: o `mirror` deixar de ler o modo ⇒ RED.)
#[test]
fn the_image_offers_edit_and_edit_is_the_image_tools() {
    let mut c = cena();
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Edit)));
    assert_eq!(c.em_edit(), None, "controlo: sem imagem activa");
    assert!(!c.hero.image_edit.mode_on);
    c.hero.gizmo.replace_selection(Some(A));
    c.quadro(None);
    assert_eq!(
        c.hero.gizmo.mode.available(),
        [ObjectMode::Object, ObjectMode::Paint, ObjectMode::Edit],
        "o seletor da imagem"
    );
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Edit)));
    assert_eq!(c.em_edit(), Some(A));
    assert!(
        c.hero.image_edit.mode_on,
        "o Edit não ligou as ferramentas de imagem"
    );
    assert!(
        ph2d_editor_core::screens::hero::mode_drive::object_gizmo_shows(&c.hero),
        "a imagem em Edit ficou sem gizmo (report do dono, 05/10)"
    );
    c.quadro(Some(ModeRequest::Toggle));
    assert_eq!(c.em_edit(), None);
    assert!(
        !c.hero.image_edit.mode_on,
        "Object não desligou as ferramentas de imagem"
    );
    // CONTROLO do espelho: o Paint da imagem e o Edit de outro tipo não as ligam.
    c.hero.gizmo.mode.enter(A, ObjectMode::Paint);
    mirror(&mut c.hero, |b| b == A || b == B);
    assert!(
        !c.hero.image_edit.mode_on,
        "o Paint ligou as ferramentas de imagem"
    );
    c.hero.gizmo.mode.enter(VEC, ObjectMode::Edit);
    mirror(&mut c.hero, |b| b == A || b == B);
    assert!(
        !c.hero.image_edit.mode_on,
        "o Edit do vetor ligou as ferramentas de imagem"
    );
}

/// ⭐⭐ GATE (dono, 05/10: *«todos os objetos daquele tipo estão em modo de edição. Ao clicar num
/// objeto de outro tipo, o objeto deve ser selecionado mas em modo object»*) — **o Edit da imagem é
/// do TIPO**: as duas imagens seleccionadas entram juntas (o *Equalize Sizes* trabalha sobre
/// várias), a outra imagem não é recusada e não larga o modo; apagar a trancada passa o Edit à
/// outra; um objecto de outro tipo seleccionado volta a Object e desliga as ferramentas.
/// (Mutações: `holds_the_whole_kind` = `false` ⇒ a outra imagem é recusada ⇒ RED; o `heir` = `None`
/// ⇒ apagar a trancada larga o Edit ⇒ RED.)
#[test]
fn the_image_edit_holds_every_image_and_another_kind_leaves_it() {
    let mut c = cena();
    c.hero.gizmo.replace_selection(Some(A));
    c.hero.gizmo.add_to_selection(B);
    c.quadro(Some(ModeRequest::Enter(ObjectMode::Edit)));
    assert_eq!(c.em_edit(), Some(B), "o activo é o último");
    assert_eq!(
        c.hero.gizmo.selected_len(),
        2,
        "a outra imagem não entrou junto"
    );
    assert!(
        !refused(&c.hero, Some(A), false, &mut c.toasts),
        "a outra imagem foi recusada"
    );
    c.hero.gizmo.replace_selection(Some(A));
    c.quadro(None);
    assert_eq!(
        c.em_edit(),
        Some(B),
        "seleccionar a outra imagem largou o Edit"
    );
    c.images.retain(|b| *b != B);
    c.quadro(None);
    assert_eq!(c.em_edit(), Some(A), "apagar a trancada largou o Edit");
    assert!(c.hero.image_edit.mode_on);
    assert!(
        !refused(&c.hero, Some(VEC), false, &mut c.toasts),
        "o vetor foi recusado"
    );
    c.hero.gizmo.replace_selection(Some(VEC));
    c.quadro(None);
    assert_eq!(
        c.em_edit(),
        None,
        "o objecto de outro tipo não saiu do Edit"
    );
    assert_eq!(c.hero.gizmo.selection, Some(VEC));
    assert!(
        !c.hero.image_edit.mode_on,
        "as ferramentas de imagem ficaram ligadas"
    );
}
