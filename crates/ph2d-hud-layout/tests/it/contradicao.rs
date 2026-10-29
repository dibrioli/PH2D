//! O Cassowary NÃO recusa desigualdades `REQUIRED` opostas: aceita as duas e devolve uma caixa que viola uma
//! delas, sem dizer nada. Quem impede é a ferramenta, por esta lei.

use ph2d_hud_layout::{Eixo, Forca, Lado, Relacao, contradiz, resolve};

use crate::{caixa, el};

#[test]
fn the_solver_silently_accepts_opposite_required_sides() {
    let els = [
        el(16.0, 312.0, 100.0, 32.0, None),
        el(200.0, 316.0, 24.0, 24.0, None),
    ];
    let rel = [
        (
            Relacao::NaoInvade {
                a: 0,
                b: 1,
                lado: Lado::Direita,
                folga: 8.0,
            },
            Forca::Requerida,
        ),
        (
            Relacao::NaoInvade {
                a: 0,
                b: 1,
                lado: Lado::Esquerda,
                folga: 8.0,
            },
            Forca::Requerida,
        ),
    ];
    // A resposta que ele dá depende da ordem interna do solver (medido 29/10: 6 em 12 corridas cada): ora viola
    // uma das duas, ora cumpre as duas DEFORMANDO a caixa (largura negativa). As duas são «aceitou calado».
    let r = resolve(caixa(0.0, 0.0, 640.0, 360.0), &els, &rel);
    assert_eq!(
        r.len(),
        2,
        "o solver não recusa: devolve uma caixa por elemento"
    );
    let antes = r[0].x + r[0].w + 8.0 <= r[1].x;
    let depois = r[0].x >= r[1].x + r[1].w + 8.0;
    let deformada = r.iter().any(|c| c.w <= 0.0);
    assert!(
        !(antes && depois) || deformada,
        "as duas não cabem juntas numa caixa sã — e o solver devolveu uma resposta na mesma: {r:?}"
    );
}

#[test]
fn a_cycle_of_sides_is_a_contradiction() {
    let direita = |a, b| Relacao::AoLado {
        a,
        b,
        lado: Lado::Direita,
        folga: 8.0,
    };
    // a à direita de b e b à direita de a.
    assert!(contradiz(&[direita(0, 1)], &direita(1, 0)));
    // Pela outra porta: «0 não invade 1 pela direita» põe 0 antes de 1; «0 à direita de 1» põe-no depois.
    assert!(contradiz(
        &[Relacao::NaoInvade {
            a: 0,
            b: 1,
            lado: Lado::Direita,
            folga: 0.0
        }],
        &direita(0, 1)
    ));
    // Em cadeia: 1 depois de 0, 2 depois de 1, e 0 depois de 2.
    assert!(contradiz(&[direita(1, 0), direita(2, 1)], &direita(0, 2)));
    // Acima/abaixo é outro eixo: não fecha o círculo do X.
    let acima = Relacao::AoLado {
        a: 0,
        b: 1,
        lado: Lado::Acima,
        folga: 8.0,
    };
    assert!(!contradiz(&[direita(1, 0)], &acima));
    assert!(contradiz(
        &[acima],
        &Relacao::AoLado {
            a: 1,
            b: 0,
            lado: Lado::Acima,
            folga: 8.0
        }
    ));
    // Uma cadeia aberta e o centrado não contradizem.
    assert!(!contradiz(&[direita(1, 0), direita(2, 1)], &direita(3, 2)));
    assert!(!contradiz(
        &[direita(1, 0)],
        &Relacao::Centrado {
            a: 0,
            b: 1,
            eixo: Eixo::Y
        }
    ));
    assert!(!contradiz(
        &[direita(1, 0)],
        &Relacao::LarguraDoConteudo { a: 0 }
    ));
}
