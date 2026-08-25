//! Os gates do sistema dos fechos.

use crate::weld::weld;
use crate::weld_flat::{ClosureSystem, Var};

/// ⭐⭐⭐ **AS DUAS PORTAS DA PROPAGAÇÃO TÊM DE CONCORDAR.**
///
/// A dependente escreve-se de duas maneiras: [`ClosureSystem::apply`] reconstrói-a das
/// livres, e [`ClosureSystem::bump`] soma-lhe o efeito de UMA livre que se mexeu.
///
/// ⛔⛔ **Este gate existe porque elas divergiram em silêncio.** A primeira redacção do
/// `bump` apanhava só o PRIMEIRO termo de cada livre na expressão (`find`), e uma livre
/// pode aparecer lá **duas vezes** — dois caminhos até ela, cujos coeficientes somam.
/// O erro ficava mascarado: o `apply` do fim da cadeia reconstruía tudo e escrevia por
/// cima. *Uma lei em dois sítios não é uma lei enquanto ninguém a confrontar consigo
/// própria.*
#[test]
fn the_incremental_bump_agrees_with_the_full_apply() {
    for mut mesh in [
        ph2d_mesh::shapes::uv_sphere(24, 36, 1.0),
        ph2d_mesh::shapes::torus(64, 32, 1.0, 0.35),
    ] {
        let (cut, combed, _h, _) = crate::round::tests::chain(&mut mesh);
        let (w, _) = weld(&cut, &combed);
        let jumped: Vec<bool> = (0..cut.seams.len())
            .map(|s| combed.jump.get(s).copied().flatten().is_some())
            .collect();
        let (sys, rep) = ClosureSystem::build(&w, cut.seams.len(), &jumped);
        assert!(
            rep.equations > 20 && rep.resolved > 20,
            "a fixtura tem de conter o fenómeno: {} equações, {} eliminadas",
            rep.equations,
            rep.resolved
        );
        let base = crate::solve::GridMap {
            uv: cut
                .origin
                .iter()
                .map(|o| vec![[0.0f32; 2]; o.len()])
                .collect(),
            shift: vec![[0.0; 2]; cut.seams.len()],
        };
        let mut checked = 0usize;
        for i in 0..sys.free().len() {
            let delta = [0.25f32, -0.75];
            // ── porta 1: incremental.
            let mut a = base.clone();
            sys.bump(&w, &mut a, i, delta);
            // ── porta 2: escrever a livre e RECONSTRUIR tudo.
            let mut b = base.clone();
            match sys.free()[i] {
                Var::Shift(s) => {
                    b.shift[s as usize][0] += delta[0];
                    b.shift[s as usize][1] += delta[1];
                    for &cl in w.shift_classes_pub(s as usize) {
                        w.derive(&mut b, cl as usize);
                    }
                }
                Var::Class(c) => {
                    let y = w.value_pub(&b, c as usize);
                    w.set(&mut b, c as usize, [y[0] + delta[0], y[1] + delta[1]]);
                }
            }
            sys.apply(&w, &mut b);
            for (s, (ta, tb)) in a.shift.iter().zip(&b.shift).enumerate() {
                assert!(
                    (ta[0] - tb[0]).abs() < 1.0e-4 && (ta[1] - tb[1]).abs() < 1.0e-4,
                    "livre {i}, costura {s}: incremental {ta:?} contra reconstruído {tb:?}"
                );
            }
            checked += 1;
        }
        assert!(
            checked > 5,
            "poucas livres para provar coisa nenhuma: {checked}"
        );
    }
}
