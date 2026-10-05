//! O gate do modo do TIPO (dono, 05/10), com a sua família falsa — irmão de `mode_drive_tests` pelo
//! tecto de LOC (HR-18); usa a cena dele.

use super::*;

/// Um vetor FALSO do TIPO (dono, 05/10): o Edit é de todas as formas vivas, a trancada que morre
/// passa o modo à herdeira, e a que nasce num Edit re-tranca-o.
#[derive(Default)]
struct KindFamily {
    alive: Vec<u64>,
    held: Option<u64>,
    born: Option<u64>,
}

impl ModeFamily for KindFamily {
    fn modes(&self) -> &'static [(ObjectKind, ObjectMode)] {
        &[(ObjectKind::Vector, ObjectMode::Edit)]
    }
    fn holds(&mut self, _: ObjectMode, e: u64, _: &mut ToolRegistry) -> bool {
        self.held == Some(e) && self.alive.contains(&e)
    }
    fn enter(&mut self, _: ObjectMode, e: u64, _: &mut ToolRegistry) -> bool {
        self.held = Some(e);
        true
    }
    fn leave(&mut self, _: ObjectMode, _: u64, _: &mut ToolRegistry) {
        self.held = None;
    }
    fn holds_the_whole_kind(&self, _: ObjectMode) -> bool {
        true
    }
    fn parts(&mut self, e: u64) -> Option<Vec<u64>> {
        let others: Vec<u64> = self.alive.iter().copied().filter(|b| *b != e).collect();
        (self.alive.contains(&e) && !others.is_empty()).then_some(others)
    }
    fn heir(&mut self, _: ObjectMode, locked: u64, _: &mut ToolRegistry) -> Option<u64> {
        if self.held != Some(locked) || self.alive.contains(&locked) {
            return None;
        }
        self.held = self.alive.last().copied();
        self.held
    }
    fn wants(&mut self, _: &mut ToolRegistry) -> Option<(u64, ObjectMode)> {
        self.born.take().map(|e| (e, ObjectMode::Edit))
    }
}

/// ⭐⭐ GATE (dono, 05/10: *«o modo de edição significa que todos os objetos daquele tipo estão em
/// modo de edição. Ao clicar num objeto de outro tipo, o objeto deve ser selecionado mas em modo
/// object»*) — **um modo do TIPO**, no quadro da fundação: o cadeado não recusa nada (nem a outra
/// forma nem a imagem, e sem aviso); a outra forma seleccionada não larga o modo; o `Tab` de volta
/// deixa a selecção como estava; a forma que NASCE num Edit re-tranca-o sem repetir o aviso; a
/// trancada que MORRE passa o modo à herdeira sem tocar a selecção; e a imagem seleccionada volta a
/// Object. CONTROLO: num modo que não é do tipo o cadeado recusa
/// (`the_lock_door_refuses_and_says_why`, `a_mode_that_joins_takes_the_selected_of_the_same_kind`).
#[test]
fn a_mode_of_the_whole_kind_never_locks_and_hands_itself_on() {
    let mut c = cena();
    let mut fam = KindFamily {
        alive: vec![VEC_A, VEC_B],
        ..KindFamily::default()
    };
    let quadro = |c: &mut Cena, fam: &mut KindFamily, req| {
        let alive = fam.alive.clone();
        let kind = move |b: u64| match b {
            IMG => ObjectKind::Image,
            b if alive.contains(&b) => ObjectKind::Vector,
            _ => ObjectKind::Empty,
        };
        drive(
            &mut [fam],
            &kind,
            &|_| "Obj".to_string(),
            &mut c.tools,
            &mut c.hero,
            &mut c.toasts,
            req,
        );
    };
    let locked = |c: &Cena| c.hero.gizmo.mode.locked_entity();
    c.hero.gizmo.replace_selection(Some(VEC_A));
    quadro(&mut c, &mut fam, Some(ModeRequest::Toggle));
    assert_eq!(locked(&c), Some(VEC_A));
    assert!(c.hero.gizmo.mode.whole_kind());
    let avisos = c.toasts.iter().count();
    assert!(
        !refused(&c.hero, Some(VEC_B), true, &mut c.toasts),
        "a outra forma"
    );
    assert!(
        !refused(&c.hero, Some(IMG), false, &mut c.toasts),
        "a imagem"
    );
    assert_eq!(c.toasts.iter().count(), avisos, "o cadeado avisou");
    c.hero.gizmo.replace_selection(Some(VEC_B));
    quadro(&mut c, &mut fam, None);
    assert_eq!(locked(&c), Some(VEC_A), "a outra forma largou o modo");
    quadro(&mut c, &mut fam, Some(ModeRequest::Toggle));
    assert_eq!(locked(&c), None);
    assert_eq!(
        (c.hero.gizmo.selection, c.hero.gizmo.selected_len()),
        (Some(VEC_B), 1),
        "o Tab de volta seleccionou as partes"
    );
    quadro(&mut c, &mut fam, Some(ModeRequest::Toggle));
    assert_eq!(locked(&c), Some(VEC_B));
    let avisos = c.toasts.iter().count();
    fam.alive.push(VEC_C);
    fam.born = Some(VEC_C);
    quadro(&mut c, &mut fam, None);
    assert_eq!(locked(&c), Some(VEC_C), "o nascido não re-trancou o modo");
    assert_eq!(
        c.toasts.iter().count(),
        avisos,
        "a passagem repetiu o aviso"
    );
    fam.alive.retain(|b| *b != VEC_C);
    c.hero.gizmo.replace_selection(None);
    quadro(&mut c, &mut fam, None);
    assert_eq!(
        locked(&c),
        Some(VEC_B),
        "a trancada morta não passou à herdeira"
    );
    assert_eq!(c.hero.gizmo.selection, None, "a herdeira mexeu na selecção");
    c.hero.gizmo.replace_selection(Some(IMG));
    quadro(&mut c, &mut fam, None);
    assert_eq!(locked(&c), None, "a imagem não voltou a Object");
    assert_eq!(c.hero.gizmo.selection, Some(IMG));
}
