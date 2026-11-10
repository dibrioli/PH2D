//! B11.A1 (o resto): o worker da sim MORRE em pânico e leva o motor. Nenhuma porta que o pede de volta entra em
//! pânico, a sessão acaba no guarda seguinte e a tela fica com a última imagem composta — ao byte.

use crate::tool::paint::PainterTool;
use crate::tool::paint::wetpaint::offthread::EngineSlot;
use crate::tool::paint::wetpaint::otica_no_tique_tests::{cena, quadros, tique};

/// A cena do J13 com o motor no worker, e o worker morto com ele.
fn com_o_worker_morto() -> (PainterTool, Vec<u8>) {
    let mut t = cena(false, 0.0, false);
    quadros(&mut t, 4);
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    assert!(
        matches!(s.engine, EngineSlot::Away),
        "premissa: o motor com o worker"
    );
    s.worker.as_mut().expect("o worker").mata_no_teste();
    let antes = t.canvas_rgba.to_vec();
    (t, antes)
}

fn acaba_sem_tocar_a_tela(mut t: PainterTool, antes: &[u8], porta: &str) {
    for _ in 0..3 {
        tique(&mut t);
    }
    assert!(
        t.paint.wetpaint.session.is_none(),
        "{porta}: a sessão sem motor continua viva"
    );
    let n = antes
        .iter()
        .zip(t.canvas_rgba.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert_eq!(
        n, 0,
        "{porta}: {n} bytes da tela mudaram depois da morte do worker"
    );
}

#[test]
fn worker_morto_acaba_a_sessao_sem_tocar_a_tela() {
    let (mut t, antes) = com_o_worker_morto();
    t.paint
        .wetpaint
        .session
        .as_mut()
        .expect("sessao")
        .bring_home();
    acaba_sem_tocar_a_tela(t, &antes, "bring_home");

    let (mut t, antes) = com_o_worker_morto();
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    assert!(
        s.try_bring_home(),
        "try_bring_home: o motor perdido não volta nunca (a água congela)"
    );
    acaba_sem_tocar_a_tela(t, &antes, "try_bring_home");

    let (mut t, antes) = com_o_worker_morto();
    let s = t.paint.wetpaint.session.as_mut().expect("sessao");
    assert!(
        s.try_bring_home_now(),
        "try_bring_home_now: o motor perdido não volta nunca"
    );
    acaba_sem_tocar_a_tela(t, &antes, "try_bring_home_now");
}
