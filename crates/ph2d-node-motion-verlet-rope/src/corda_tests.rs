use super::{Corda, Feitio};

const F: Feitio = Feitio {
    pontos: 12,
    gravidade: 2000.0,
    iteracoes: 20,
    amortecimento: 0.05,
    flexao: 0.1,
};

#[test]
fn a_ui_rope_hangs_between_its_two_ends_and_follows_them() {
    let mut c = Corda::default();
    let (a, b) = ([0.0, 0.0], [200.0, 0.0]);
    for _ in 0..240 {
        c.passo(a, b, 260.0, 1.0 / 60.0, &F);
    }
    let p = c.pontos();
    assert_eq!(p.len(), 12);
    assert_eq!((p[0], p[11]), (a, b), "as pontas ficam presas");
    let meio = p[6][1];
    assert!(meio < -40.0, "a folga pende para −y (gravidade): {meio}");

    // A ponta anda: o primeiro passo não teleporta o meio (sem saltos, T41).
    let antes = p[6];
    c.passo(a, [260.0, 0.0], 260.0 * 1.3, 1.0 / 60.0, &F);
    let depois = c.pontos()[6];
    let d = (depois[0] - antes[0]).hypot(depois[1] - antes[1]);
    assert!(d < 30.0, "o meio acompanha com inércia, não salta: {d}");
    assert_eq!(c.pontos()[11], [260.0, 0.0]);
}
