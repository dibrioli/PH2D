//! Os gates das **vistas nomeadas** (W47).

use super::*;

fn close(a: [f32; 3], b: [f32; 3]) -> bool {
    (0..3).all(|i| (a[i] - b[i]).abs() < 1.0e-5)
}

/// ⭐⭐ **CADA VISTA PÕE O OLHO NO EIXO QUE O NOME PROMETE.**
///
/// ⚠️ **É o gate que separa o nome da aritmética.** A orientação é escrita em `yaw`/`pitch` (a porta
/// da casa para um enquadramento nomeado), e um sinal trocado ali dá uma vista chamada *Frente* que
/// mostra as **costas** — e nada, em teste nenhum de forma, notaria: a peça aparece, inteira e
/// enquadrada, do lado errado.
///
/// A régua é a **base da câmera**, que é a única coisa que sabe para onde ela olha.
#[test]
fn every_named_view_puts_the_eye_on_the_axis_its_name_promises() {
    for s in Standard::ALL {
        let cam = Orbit {
            rotation: s.rotation(),
            ..Orbit::default()
        };
        let (_, _, fwd) = cam.basis();
        assert!(
            close(fwd, s.eye_axis()),
            "{s:?}: o olho ficou em {fwd:?} e o nome promete {:?}",
            s.eye_axis()
        );
    }
}

/// ⚠️ **As seis são DISTINTAS** — duas vistas com a mesma orientação seriam dois botões para o mesmo
/// lugar, e o realce acenderia no errado.
#[test]
fn the_six_views_are_all_different() {
    for (i, a) in Standard::ALL.into_iter().enumerate() {
        for b in Standard::ALL.into_iter().skip(i + 1) {
            assert_ne!(
                named_view(&Orbit {
                    rotation: a.rotation(),
                    ..Orbit::default()
                }),
                Some(b),
                "{a:?} foi reconhecida como {b:?}"
            );
        }
    }
}

/// ⭐ **A vista é RECONHECIDA, não guardada** — e o arrasto mais pequeno já a solta.
///
/// ⚠️ A segunda metade é a que importa: se a tolerância fosse larga, o chip ficaria aceso sobre uma
/// vista que já não é aquela — *um espelho de estado a mentir*, que é o defeito que este módulo já
/// pagou no cache do traçado.
#[test]
fn a_named_view_is_recognised_and_the_smallest_drag_lets_it_go() {
    for s in Standard::ALL {
        let mut cam = Orbit {
            rotation: s.rotation(),
            ..Orbit::default()
        };
        assert_eq!(
            named_view(&cam),
            Some(s),
            "{s:?} não se reconhece a si mesma"
        );

        // ⚠️ **E o RUÍDO não a solta**: re-normalizar (o que toda composição faz) tem de deixar a
        // vista reconhecida. Sem esta metade, uma barra apertada demais passaria no teste do pixel
        // e o chip apagaria sozinho a meio de uma sessão parada.
        let n = (cam.rotation[0].powi(2)
            + cam.rotation[1].powi(2)
            + cam.rotation[2].powi(2)
            + cam.rotation[3].powi(2))
        .sqrt();
        let mut renormed = cam;
        renormed.rotation = cam.rotation.map(|c| c / n);
        assert_eq!(
            named_view(&renormed),
            Some(s),
            "{s:?}: re-normalizar soltou a vista — a barra está abaixo do ruído de f32"
        );

        // Um pixel de arrasto — o menor gesto que existe.
        crate::field3d_input::law::orbit(&mut cam, 1.0, 0.0);
        assert_eq!(
            named_view(&cam),
            None,
            "{s:?}: um pixel de arrasto tem de soltar a vista nomeada"
        );
    }
}

/// ⚠️ **`q` e `−q` são a MESMA orientação.** Sem o módulo no produto interno, metade das vistas
/// certas leria como livre — e qual metade depende do caminho pelo qual a câmera lá chegou, que é
/// a assinatura de um defeito impossível de reproduzir a pedido.
#[test]
fn the_negated_quaternion_is_the_same_view() {
    for s in Standard::ALL {
        let q = s.rotation();
        let cam = Orbit {
            rotation: [-q[0], -q[1], -q[2], -q[3]],
            ..Orbit::default()
        };
        assert_eq!(named_view(&cam), Some(s), "{s:?} negada deixou de ser ela");
    }
}

/// **As teclas são as do Blender**, e o `Ctrl` dá o oposto de cada uma.
#[test]
fn the_keys_are_the_reference_ones_and_ctrl_gives_the_opposite() {
    use winit::keyboard::KeyCode as K;
    let pairs = [
        (K::Numpad1, Standard::Front, Standard::Back),
        (K::Numpad3, Standard::Right, Standard::Left),
        (K::Numpad7, Standard::Top, Standard::Bottom),
    ];
    for (code, plain, with_ctrl) in pairs {
        assert_eq!(view_for_key(code, false), Some(plain));
        assert_eq!(view_for_key(code, true), Some(with_ctrl));
        // ⚠️ E o oposto é mesmo o oposto — o eixo do olho invertido, não outra vista qualquer.
        let a = plain.eye_axis();
        let b = with_ctrl.eye_axis();
        assert!(
            close(b, [-a[0], -a[1], -a[2]]),
            "{plain:?} e {with_ctrl:?} não são opostas: {a:?} e {b:?}"
        );
    }
    // ⚠️ O `Numpad5` NÃO é desta porta: ele é a lente (W15), e responder aqui roubaria a tecla.
    assert_eq!(view_for_key(K::Numpad5, false), None);
    assert_eq!(view_for_key(K::KeyF, false), None);
}

/// ⭐⭐⭐ **O RÓTULO DE UM VIEWPORT SAI DA CÂMERA, NUNCA DO QUADRANTE** (W90d).
///
/// ⚠️ **É a lei que impede o rótulo de mentir em silêncio.** Com a divisão aberta, a vista de cima
/// nasce no quadrante de cima-esquerda — mas o artista pode orbitá-la, e a partir daí ela **não é**
/// a vista de cima. Um rótulo preso ao sítio continuaria a dizer *Top* sobre uma vista qualquer, e
/// nada na tela o desmentiria.
#[test]
fn the_viewport_label_follows_the_camera_and_not_the_quadrant() {
    use super::{Standard, label_key};
    use ph2d_field_render::Orbit;
    // Cada vista nomeada diz o próprio nome…
    for v in Standard::ALL {
        let cam = Orbit {
            rotation: v.rotation(),
            ..Orbit::default()
        };
        let key = label_key(&cam);
        assert_ne!(
            key, "viewport.model3d.view.user",
            "a vista {v:?} devia dizer o nome dela"
        );
        assert_ne!(
            ph2d_i18n::tr(key),
            key,
            "a chave {key} não traduz — um rótulo que mostra a própria chave é pior que nenhum"
        );
    }
    // …e as seis chaves são DISTINTAS: duas vistas com o mesmo nome seriam duas respostas erradas.
    let mut chaves: Vec<&str> = Standard::ALL
        .into_iter()
        .map(|v| {
            label_key(&Orbit {
                rotation: v.rotation(),
                ..Orbit::default()
            })
        })
        .collect();
    chaves.sort_unstable();
    let antes = chaves.len();
    chaves.dedup();
    assert_eq!(
        antes,
        chaves.len(),
        "duas vistas nomeadas partilham o rótulo"
    );

    // ⭐ E uma câmera ORBITADA deixa de ser a vista nomeada — é aqui que um rótulo preso ao
    // quadrante mentiria.
    let mut cam = Orbit {
        rotation: Standard::Top.rotation(),
        ..Orbit::default()
    };
    crate::field3d_input::law::orbit(&mut cam, 40.0, 25.0);
    assert_eq!(
        label_key(&cam),
        "viewport.model3d.view.user",
        "uma vista de cima que foi orbitada já não é a vista de cima"
    );
    assert_ne!(
        ph2d_i18n::tr("viewport.model3d.view.user"),
        "viewport.model3d.view.user",
        "a chave da vista livre não traduz"
    );
}
