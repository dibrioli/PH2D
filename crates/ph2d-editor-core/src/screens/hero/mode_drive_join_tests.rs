//! Os gates dos modos que JUNTAM (Vector ▸ Edit leva as formas seleccionadas) e das PARTES com gizmo —
//! irmão de `mode_drive_tests` pelo tecto de LOC (HR-18); usa a cena dele.

use super::*;

/// Um vetor FALSO: Vector ▸ Edit leva junto as formas seleccionadas (o multi-objecto do Blender).
#[derive(Default)]
pub(super) struct JoinFamily {
    pub(super) held: Vec<u64>,
}

pub(super) const VEC_A: u64 = 50;
pub(super) const VEC_B: u64 = 51;
pub(super) const VEC_C: u64 = 52;
/// Uma FORMA dentro do objecto vetorial — uma parte de outro tipo (spec/06 F3, o contentor).
pub(super) const VEC_SHAPE: u64 = 53;

impl ModeFamily for JoinFamily {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::Vector, ObjectMode::Edit)]
    }
    fn holds(&mut self, _: ObjectMode, e: u64, _: &mut ToolRegistry) -> bool {
        self.held.first() == Some(&e)
    }
    fn enter(&mut self, m: ObjectMode, e: u64, t: &mut ToolRegistry) -> bool {
        self.enter_with(m, e, &[], t)
    }
    fn leave(&mut self, _: ObjectMode, _: u64, _: &mut ToolRegistry) {
        self.held.clear();
    }
    fn joins(&self, _: ObjectMode) -> bool {
        true
    }
    fn parts_take_the_object_gizmo(&self, _: ObjectMode) -> bool {
        true
    }
    fn enter_with(&mut self, _: ObjectMode, e: u64, joined: &[u64], _: &mut ToolRegistry) -> bool {
        self.held = [e].into_iter().chain(joined.iter().copied()).collect();
        true
    }
    fn parts(&mut self, e: u64) -> Option<Vec<u64>> {
        (self.held.first() == Some(&e)).then(|| {
            let mut p = self.held.clone();
            p.push(VEC_SHAPE);
            p
        })
    }
}

pub(super) fn vec_kind(bits: u64) -> ObjectKind {
    match bits {
        VEC_A | VEC_B | VEC_C => ObjectKind::Vector,
        VEC_SHAPE => ObjectKind::Empty,
        _ => ObjectKind::Image,
    }
}

/// ⭐⭐ GATE (spec/06 F3 ▸ Vector, o multi-objecto) — **um modo que JUNTA leva os do mesmo tipo**:
/// com duas formas e uma imagem seleccionadas, o `Tab` entra no Edit do activo COM a outra forma
/// (a imagem sai da selecção); a selecção anda entre as duas, uma terceira forma é recusada; e o
/// `Tab` de volta devolve as duas à selecção (e NÃO as formas de dentro, que também são partes),
/// para o seguinte voltar a juntá-las.
#[test]
fn a_mode_that_joins_takes_the_selected_of_the_same_kind() {
    let mut c = cena();
    let mut fam = JoinFamily::default();
    let quadro = |c: &mut Cena, fam: &mut JoinFamily, req| {
        drive(
            &mut [fam],
            &vec_kind,
            &|_| "Obj".to_string(),
            &mut c.tools,
            &mut c.hero,
            &mut c.toasts,
            req,
        );
    };
    let selected = |c: &Cena| {
        let mut s: Vec<u64> = c.hero.gizmo.selection.iter().copied().collect();
        s.extend(&c.hero.gizmo.extra_selection);
        s.sort_unstable();
        s
    };
    c.hero.gizmo.replace_selection(Some(VEC_A));
    c.hero.gizmo.add_to_selection(IMG);
    c.hero.gizmo.add_to_selection(VEC_B);
    quadro(&mut c, &mut fam, Some(ModeRequest::Toggle));
    assert_eq!(
        c.hero.gizmo.mode.locked_entity(),
        Some(VEC_B),
        "o activo é o último"
    );
    assert_eq!(
        fam.held,
        vec![VEC_B, VEC_A],
        "a outra forma não entrou no Edit"
    );
    assert_eq!(
        selected(&c),
        vec![VEC_A, VEC_B],
        "a imagem ficou, ou uma forma saiu"
    );
    quadro(&mut c, &mut fam, None);
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Edit,
        "as duas derrubaram o Edit"
    );
    chosen(&mut c.hero, Some(VEC_A), false);
    assert!(
        !c.hero.gizmo.mode.take_release(),
        "a forma que entrou tirou o modo"
    );
    c.hero.gizmo.replace_selection(None);
    quadro(&mut c, &mut fam, None);
    assert_eq!(
        c.hero.gizmo.mode.current(),
        ObjectMode::Edit,
        "desseleccionar saiu do Edit"
    );
    quadro(&mut c, &mut fam, Some(ModeRequest::Toggle));
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Object);
    assert_eq!(
        selected(&c),
        vec![VEC_A, VEC_B],
        "o Tab não devolveu as duas"
    );
    assert_eq!(
        object_mode::active_of(c.hero.gizmo.selection, &c.hero.gizmo.extra_selection),
        Some(VEC_B),
        "o activo mudou ao sair"
    );
    quadro(&mut c, &mut fam, Some(ModeRequest::Toggle));
    assert_eq!(
        fam.held,
        vec![VEC_B, VEC_A],
        "o Tab seguinte não as juntou de novo"
    );
}

/// ⭐⭐ GATE (spec/06 F3 ▸ Vector; report do dono 04/10: *«o gizmo não aparece e não consigo a
/// multiseleção»*; e 05/10: *«ao fazer um boolean o gizmo já não aparece»*) — **num modo que o
/// declara, a selecção tem o gizmo** — a parte E o objecto trancado (o Edit do vetor é sobre a própria
/// forma, e a forma da booleana fica sozinha nele); e **o laço que apanha só partes fica no modo**,
/// o que apanha outro objecto sai para Object (dono 24/10). Num modo de objecto inteiro (o Draw do
/// Flip) o gizmo some e o laço com o outro desenho sai.
#[test]
fn a_parts_mode_gives_the_part_its_gizmo_and_the_lasso_its_parts() {
    let mut c = cena();
    let mut fam = JoinFamily::default();
    let quadro = |c: &mut Cena, fam: &mut JoinFamily, req| {
        drive(
            &mut [fam],
            &vec_kind,
            &|_| "Obj".to_string(),
            &mut c.tools,
            &mut c.hero,
            &mut c.toasts,
            req,
        );
    };
    c.hero.gizmo.replace_selection(Some(VEC_A));
    quadro(&mut c, &mut fam, Some(ModeRequest::Toggle));
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Edit);
    assert!(
        object_gizmo_shows(&c.hero),
        "o objecto trancado (a forma do Edit) perdeu o gizmo"
    );
    c.hero.gizmo.replace_selection(Some(VEC_SHAPE));
    quadro(&mut c, &mut fam, None);
    assert_eq!(c.hero.gizmo.mode.current(), ObjectMode::Edit);
    assert!(
        object_gizmo_shows(&c.hero),
        "a forma seleccionada em Edit não tem gizmo"
    );
    lasso_chosen(&mut c.hero, &[VEC_SHAPE, VEC_A]);
    assert!(
        !c.hero.gizmo.mode.take_release(),
        "o laço de partes tirou o modo"
    );
    lasso_chosen(&mut c.hero, &[VEC_SHAPE, IMG]);
    assert!(
        c.hero.gizmo.mode.take_release(),
        "o laço com outro objecto ficou no modo"
    );

    let mut s = cena();
    s.hero.gizmo.replace_selection(Some(DRAWING));
    s.quadro(Some(ModeRequest::Enter(ObjectMode::Draw)));
    lasso_chosen(&mut s.hero, &[DRAWING, DRAWING2]);
    assert!(
        s.hero.gizmo.mode.take_release(),
        "o laço com o outro desenho ficou no modo"
    );
    assert!(!object_gizmo_shows(&s.hero));
}
