//! Gates da LEI DO TRAÇO e dos verbos.
//!
//! O gate que decide a wave é
//! `the_stroke_is_a_fact_of_the_path_not_of_how_finely_it_was_sampled` — o
//! irmão 3D do `the_trench_is_a_fact_of_the_path_not_of_the_dab_spacing` que a
//! `line/Painter` escreveu depois de pagar a mesma doença quatro vezes.

use super::*;
use crate::Falloff;
use ph2d_mesh::{Mesh, shapes};

fn sphere() -> Mesh {
    shapes::uv_sphere(32, 48, 1.0)
}

/// Um dab visto **de fora, olhando direto para o centro dele** — o caso normal
/// de toda fixture desta suíte.
///
/// ⚠️ Ele existe para o conjunto FRONTAL não mudar o sentido de nenhum gate
/// escrito antes dele: com o olho apontando para o centro, a calota inteira sob
/// o pincel é frontal, que é a situação que todas estas fixtures descrevem. Um
/// olho fixo (`[0,0,-1]`) faria a pegada de um dab fora do eixo ficar
/// parcialmente de costas, e o culling passaria a mexer em gates que não são
/// sobre ele.
fn dab_at(center: [f32; 3], radius: f32) -> Dab {
    let l = (center[0] * center[0] + center[1] * center[1] + center[2] * center[2]).sqrt();
    let eye = if l > 1e-6 {
        [-center[0] / l, -center[1] / l, -center[2] / l]
    } else {
        [0.0, 0.0, -1.0]
    };
    Dab::at(center, radius, eye)
}

/// **O dab que ESTE verbo precisa** — a porta que os gates que varrem
/// [`Verb::ALL`] usam.
///
/// ⚠️ Ela existe porque um gate que dá o MESMO dab a todo verbo mede vácuo em
/// quem puxa: sem gesto o Grab não move nada, e a comparação vira *nada contra
/// nada*. O gate do `honours_invert` pegou isso sozinho no dia em que o `Move`
/// entrou no enum — ele se recusa a comparar dois dabs inertes —, e enumerar a
/// exceção em cada gate é o que apodrece quando chega o próximo verbo com
/// gesto.
///
/// ⚠️ **Exaustiva sobre o [`crate::Grip`], e não um `if verb.anchors()`:** cada
/// grip pede um gesto de espécie DIFERENTE (um vetor, um ângulo, uma fração), e
/// um `else` genérico daria a um grip novo o gesto de outro — a fixture mediria
/// vácuo com todos os gates verdes. Assim ela **não compila** até alguém dizer
/// com que gesto o grip novo se exercita.
fn dab_for(verb: Verb, center: [f32; 3], radius: f32) -> Dab {
    let base = dab_at(center, radius);
    match verb.grip() {
        // O carimbo e a pintura de canal exercitam-se com o dab simples: os
        // dois agem no lugar em que o dedo está, e não a partir de uma âncora.
        crate::Grip::Stamp | crate::Grip::Paint => base,
        // Um puxão tangente à superfície: ele move de fato, e não empurra o
        // vértice para dentro (o que confundiria um gate de deslocamento).
        crate::Grip::Hold | crate::Grip::Hook => {
            Dab::pulling(center, radius, base.eye, [0.0, radius * 0.5, 0.0])
        }
        crate::Grip::Turn(crate::Amount::Angle) => Dab::turning(center, radius, base.eye, 1.0),
        crate::Grip::Turn(crate::Amount::Fraction) => Dab::scaling(center, radius, base.eye, 0.5),
    }
}

fn snapshot(mesh: &Mesh) -> Vec<[f32; 3]> {
    mesh.positions().to_vec()
}

/// O maior deslocamento em relação a `before`.
fn max_shift(before: &[[f32; 3]], mesh: &Mesh) -> f32 {
    before
        .iter()
        .zip(mesh.positions())
        .map(|(a, b)| {
            let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .fold(0.0f32, f32::max)
}

/// Passa o pincel do ponto `a` ao ponto `b` em `events` **eventos de ponteiro**,
/// num traço só — **o laço do shell**, e não uma lista de dabs escrita à mão.
///
/// ⚠️ **Ela dirigia `stroke.dab` direto, um por amostra, e isso deixou de medir
/// o que os gates dizem medir** no dia em que o carimbo passou a COMPOR
/// (2026-08-11). Sob o envelope tanto fazia: repetir dabs era idempotente, então
/// 8 e 64 davam o mesmo número **por dentro da lei**. Sob a composição a
/// invariância vem de outro lugar — do **WALK**, que emite o mesmo conjunto de
/// dabs para o mesmo caminho seja qual for a taxa de polling (a metade 1 do
/// plano de paridade: `6,485 % → 0,000 %`). Uma fixture que pula o walk mede
/// *"somei 8 vezes contra 64"*, que é aritmética, não a lei.
///
/// ⇒ ela virou o laço que o `sculpt3d_input` roda, incluindo a âncora vinda de
/// [`crate::Walk::anchor`] — a mesma porta, pelo mesmo motivo que o
/// `verb_hook_tests::drag_hook` aprendeu antes dela: *um gate com driver próprio
/// mede a re-expressão, não o produto*.
fn sweep(mesh: &mut Mesh, brush: &Brush, a: [f32; 3], b: [f32; 3], events: usize) {
    let mut stroke = SculptStroke::default();
    stroke.begin(mesh);
    let spacing = crate::min_spacing(brush.radius);
    // O caminho é percorrido num parâmetro 1-D (o comprimento da corda), e cada
    // passo do walk é levado de volta ao segmento 3-D.
    let len = {
        let d = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
        (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
    };
    let at = |u: f32| {
        [
            a[0] + (b[0] - a[0]) * u,
            a[1] + (b[1] - a[1]) * u,
            a[2] + (b[2] - a[2]) * u,
        ]
    };
    let mut anchor = [0.0f32, 0.0];
    for e in 1..=events {
        let to = [len * e as f32 / events as f32, 0.0];
        let Some(steps) = crate::walk(anchor, to, spacing) else {
            continue;
        };
        for step in steps {
            let c = at(step[0] / len);
            stroke.dab(mesh, brush, &dab_at(c, brush.radius), Symmetry::default());
        }
        anchor = steps.anchor();
    }
}

#[test]
fn a_masked_vertex_is_not_moved_by_any_verb() {
    for verb in Verb::ALL {
        if verb.paints_mask() {
            continue;
        }
        let mut mesh = sphere();
        mesh.masks_mut().fill(1.0);
        let base = snapshot(&mesh);
        let b = Brush {
            verb,
            radius: 0.5,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(
            &mut mesh,
            &b,
            &dab_at([0.0, 0.0, 1.0], b.radius),
            Symmetry::default(),
        );
        assert_eq!(
            base,
            snapshot(&mesh),
            "{} atravessou a máscara",
            verb.label()
        );
    }
}

#[test]
fn every_verb_inherits_symmetry_from_the_one_place_it_is_expanded() {
    for verb in Verb::ALL {
        let mut mesh = sphere();
        let b = Brush {
            verb,
            radius: 0.4,
            strength: 1.0,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        // Dab fora do plano X = 0, para que o espelho caia noutro lugar.
        s.dab(
            &mut mesh,
            &b,
            &dab_at([0.6, 0.0, 0.8], b.radius),
            Symmetry::MIRROR_X,
        );
        let touched = s.touched().len();
        assert!(touched > 0, "{} não tocou nada", verb.label());
        // O conjunto tocado tem de ser simétrico em X: para cada vértice tocado
        // existe o espelho dele. Um verbo que "esquecesse" a simetria falharia
        // aqui sem precisar de um gate por verbo.
        let mut left = 0;
        let mut right = 0;
        for &v in s.touched() {
            if mesh.positions()[v as usize][0] > 0.0 {
                right += 1;
            } else {
                left += 1;
            }
        }
        assert!(
            left > 0 && right > 0,
            "{}: {left} à esquerda e {right} à direita — o espelho não saiu",
            verb.label()
        );
    }
}

#[test]
fn the_undo_window_is_the_touched_list_and_restoring_the_base_is_exact() {
    let mut mesh = sphere();
    let pristine = snapshot(&mesh);
    let b = Brush {
        verb: Verb::Draw,
        radius: 0.35,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..6 {
        let x = -0.3 + 0.12 * k as f32;
        s.dab(
            &mut mesh,
            &b,
            &dab_at([x, 0.0, 0.95], b.radius),
            Symmetry::default(),
        );
    }
    assert_ne!(pristine, snapshot(&mesh));

    // O undo é *exatamente* isto — não há um segundo sistema a construir.
    let (touched, base) = (s.touched().to_vec(), s.base_positions().to_vec());
    for (&v, p) in touched.iter().zip(&base) {
        mesh.positions_mut()[v as usize] = *p;
    }
    assert_eq!(pristine, snapshot(&mesh), "a janela não cobria o traço");
}

#[test]
fn a_dab_that_touches_nothing_is_a_no_op() {
    let mut mesh = sphere();
    let base = snapshot(&mesh);
    let b = Brush {
        radius: 0.2,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for dab in [
        dab_at([9.0, 9.0, 9.0], 0.2),
        dab_at([0.0, 0.0, 1.0], 0.0),
        Dab {
            pressure: 0.0,
            ..dab_at([0.0, 0.0, 1.0], 0.2)
        },
    ] {
        assert_eq!(s.dab(&mut mesh, &b, &dab, Symmetry::default()), 0);
    }
    assert_eq!(base, snapshot(&mesh));
    assert!(s.touched().is_empty(), "capturou sem mover");
}

#[test]
fn the_normals_after_a_stroke_are_what_a_full_rebuild_would_give() {
    let mut mesh = sphere();
    let b = Brush {
        verb: Verb::Draw,
        radius: 0.3,
        strength: 1.0,
        falloff: Falloff::Sphere,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..8 {
        let x = -0.4 + 0.1 * k as f32;
        s.dab(
            &mut mesh,
            &b,
            &dab_at([x, 0.1, 0.9], b.radius),
            Symmetry::MIRROR_X,
        );
    }
    let incremental = mesh.normals().to_vec();
    mesh.rebuild();
    for (i, (a, c)) in incremental.iter().zip(mesh.normals()).enumerate() {
        for k in 0..3 {
            assert!(
                (a[k] - c[k]).abs() < 1e-5,
                "vértice {i}: incremental {a:?} vs rebuild {c:?}"
            );
        }
    }
}

/// Os verbos, um a um — a LEI mora no arquivo irmão.
#[path = "verb_tests.rs"]
mod verbs;

/// A FORMA do peso no barro — ver o cabeçalho dele.
#[path = "verb_shape_tests.rs"]
mod verb_shape;

/// A LEI DE KERNEL que o MODO escolhe — ver o cabeçalho dele.
#[path = "verb_mode_tests.rs"]
mod verb_mode;

/// O Crease, que tem fixtures próprias e caras — ver o cabeçalho dele.
#[path = "verb_crease_tests.rs"]
mod verb_crease;

/// O conjunto FRONTAL, que tem fixtures de silhueta — ver o cabeçalho dele.
#[path = "verb_culling_tests.rs"]
mod verb_culling;

/// O Grab, que é a exceção ao envelope — ver o cabeçalho dele.
#[path = "verb_move_tests.rs"]
mod verb_move;
#[path = "verb_move_field_tests.rs"]
mod verb_move_field;

/// As TRÊS famílias AFINS do paper (twist · scale · pinch) e o gancho que reusa
/// o agarre — a W5-B. Ver o cabeçalho dele.
#[path = "verb_field_tests.rs"]
mod verb_field;

/// O VINCO, o único COMPOSTO — ver o cabeçalho dele.
#[path = "verb_crease_field_tests.rs"]
mod verb_crease_field;

/// A BORDA, que só uma malha ABERTA revela — ver o cabeçalho dele.
#[path = "verb_border_tests.rs"]
mod verb_border;

/// O Snake Hook, que é a OUTRA LEI e só um caminho arrastado revela — ver o
/// cabeçalho dele.
#[path = "verb_hook_tests.rs"]
mod verb_hook;

/// O Twist e o Local Scale, que giram em torno de uma âncora — e o ESPELHO, que
/// eles obrigaram a alcançar o dab inteiro. Ver o cabeçalho dele.
#[path = "verb_turn_tests.rs"]
mod verb_turn;

/// **A LEI DE UM CARIMBO** — ver [`stroke_law`].
///
/// ⚠️ O doc desta declaração estava PARTIDO AO MEIO: uma inserção anterior
/// meteu `stroke_law` dentro da frase que descrevia o `stroke_apply`, deixando
/// um com a primeira linha de outro e o outro com um *"cabeçalho dele."* órfão.
/// *Um comentário que descreve o vizinho é pior que comentário nenhum.*
#[path = "stroke_law_tests.rs"]
mod stroke_law;

/// **O APLICADOR ÚNICO**, e a identidade em que o peso-no-alvo se apoia — ver o
/// cabeçalho dele.
///
/// ⚠️ **Ele fica AQUI e não sob [`super::apply`]**, embora seja o gate daquele
/// módulo: as fixtures (`sphere`, `dab_for`) moram neste arquivo, e pendurá-lo
/// no sujeito custaria uma SEGUNDA cópia delas — o preço errado para ganhar uma
/// indireção de leitura.
#[path = "stroke_apply_tests.rs"]
mod stroke_apply;

/// ⚠️ **RED-FIRST da W4.2, e o defeito é de COSTURA e não de kernel.** Um traço
/// de máscara escreve um canal por vértice e **não move geometria**, então ele
/// esquece a região refrescada de propósito (não há normal nova a subir). Quem
/// perguntasse *"o que refresquei?"* para decidir o upload não subiria byte
/// nenhum — e a máscara ficaria invisível na GPU com todos os gates de CPU
/// verdes, que é exatamente como ela já era invisível antes desta wave.
#[test]
fn a_mask_dab_publishes_a_gpu_window_even_though_it_refreshes_no_normal() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(32, 48, 1.0);
    let brush = Brush {
        verb: Verb::Mask,
        radius: 0.4,
        ..Brush::default()
    };
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let n = stroke.dab(
        &mut mesh,
        &brush,
        &Dab::at([0.0, 0.0, 1.0], 0.4, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    assert!(n > 0, "o dab tem de tocar alguém");
    assert!(
        stroke.last_refreshed().is_empty(),
        "máscara não move geometria: não há normal recomputada, e isso é correto"
    );
    assert_eq!(
        stroke.last_gpu_dirty().len(),
        n,
        "mas a GPU precisa re-ler exatamente os vértices cuja máscara mudou"
    );

    // E o CONTROLE: para um verbo de geometria a janela continua sendo a das
    // normais, que é ESTRITAMENTE MAIOR que a dos movidos (o anel da borda).
    let mut geo = SculptStroke::default();
    geo.begin(&mesh);
    geo.dab(
        &mut mesh,
        &Brush {
            verb: Verb::Draw,
            radius: 0.4,
            ..Brush::default()
        },
        &Dab::at([0.0, 0.0, 1.0], 0.4, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    assert_eq!(geo.last_gpu_dirty().len(), geo.last_refreshed().len());
    assert!(
        geo.last_gpu_dirty().len() > geo.last_moved().len(),
        "para geometria a janela da GPU é o superconjunto, nunca os movidos"
    );
}

/// **UM DAB TEM DE CAIR NA MALHA EM QUE O TRAÇO COMEÇOU** — e quando não cai, a
/// mensagem diz isso.
///
/// ⚠️ Este é o repro do pânico que a cena-lista trouxe: o pen-down começava o
/// traço na peça ATIVA e o pick escolhia a peça sob o cursor, então tocar um
/// cubo (8 vértices) e depois uma esfera (6050) indexava os planos por-vértice
/// de 8 com índices de 6050. A lei mora no chamador; o `assert` existe para o
/// dia em que alguém a quebrar de novo **não** receber *"index out of bounds"*.
#[test]
#[should_panic(expected = "um dab tem de cair na malha em que o traço COMEÇOU")]
fn a_dab_on_a_bigger_mesh_than_the_stroke_began_on_says_so() {
    let small = ph2d_mesh::shapes::cube(1.0);
    let mut big = ph2d_mesh::shapes::uv_sphere(24, 32, 1.0);
    assert!(
        big.vert_count() > small.vert_count(),
        "a fixture contém o caso"
    );

    let mut stroke = SculptStroke::default();
    stroke.begin(&small);
    stroke.dab(
        &mut big,
        &Brush::default(),
        &Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
}

/// E o CONTROLE: na malha certa o dab passa. Sem esta metade o gate acima
/// passaria com um `assert!(false)` no lugar da comparação.
#[test]
fn a_dab_on_the_mesh_the_stroke_began_on_is_fine() {
    let mut mesh = ph2d_mesh::shapes::uv_sphere(24, 32, 1.0);
    let mut stroke = SculptStroke::default();
    stroke.begin(&mesh);
    let moved = stroke.dab(
        &mut mesh,
        &Brush::default(),
        &Dab::at([0.0, 0.0, 1.0], 0.5, [0.0, 0.0, -1.0]),
        Symmetry::default(),
    );
    assert!(moved > 0, "o dab tem de mover alguma coisa");
}
