//! Gates do Repeater. O que este efeito acrescenta às propriedades da pilha é a **multiplicação
//! de contornos** — e é aí que ele pode estragar um compound, que os outros dois não podem.

use super::*;
use crate::FillRule;

/// Um quadrado de lado 40 na origem — números do produto, não `1.0`.
fn square() -> VecPath {
    VecPath {
        verts: [[0.0, 0.0], [40.0, 0.0], [40.0, 40.0], [0.0, 40.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

/// Uma fileira em x: `n` cópias, `dx`% de passo, com `spin` graus por cópia.
fn spec(copies: f64, dx: f64, spin: f64) -> RepeatSpec {
    RepeatSpec {
        copies_x: copies,
        move_x: dx,
        spin,
        ..RepeatSpec::default()
    }
}

/// A caixa de todos os contornos — a assinatura visível do array.
fn bbox(p: &VecPath) -> ([f64; 2], [f64; 2]) {
    let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
    for v in p.verts_all() {
        for k in 0..2 {
            lo[k] = lo[k].min(v.anchor[k]);
            hi[k] = hi[k].max(v.anchor[k]);
        }
    }
    (lo, hi)
}

/// **Uma cópia é a forma** — o ponto neutro é no-op byte-idêntico, o que a pilha exige.
#[test]
fn a_single_copy_is_a_byte_identical_no_op() {
    let p = square();
    assert_eq!(repeat_path(&p, &spec(1.0, 50.0, 30.0)), p);
}

/// **Um passo que não move nada também é neutro** — `n` cópias exatamente sobrepostas seriam
/// invisíveis e caras. É o neutro pela outra porta, e sem este gate ele passa despercebido.
#[test]
fn copies_with_no_transform_are_still_a_no_op() {
    let p = square();
    assert_eq!(repeat_path(&p, &spec(8.0, 0.0, 0.0)), p);
}

/// **`n` cópias produzem `n` contornos**, e o original continua a ser o primário.
#[test]
fn the_copies_land_as_contours_and_the_original_stays_primary() {
    let p = square();
    let out = repeat_path(&p, &spec(4.0, 100.0, 0.0));
    assert_eq!(out.contour_count(), 4, "1 original + 3 cópias");
    assert_eq!(out.verts, p.verts, "o original não pode ter-se mexido");
}

/// **`Move X = 100` desloca por uma largura-média da forma** — a lei da percentagem, a mesma do
/// `Size` do Zig Zag. O quadrado de 40 tem `ref_size` 40, então 4 cópias medem 4×40 = 160.
#[test]
fn move_is_a_percentage_of_the_shape() {
    let p = square();
    let out = repeat_path(&p, &spec(4.0, 100.0, 0.0));
    let (lo, hi) = bbox(&out);
    assert!(
        (hi[0] - lo[0] - 160.0).abs() < 1e-6,
        "4 cópias a uma largura de distância medem 160, mediram {}",
        hi[0] - lo[0]
    );
}

/// **O MESMO número desenha o mesmo array em qualquer escala** — é o ponto inteiro de a
/// distância ser percentagem. O oráculo é a razão entre a extensão do array e a da forma, que é
/// adimensional.
#[test]
fn the_same_move_looks_the_same_at_any_scale() {
    let scaled = |k: f64| -> VecPath {
        let mut p = square();
        p.for_each_vert_mut(|v| {
            v.anchor = [v.anchor[0] * k, v.anchor[1] * k];
            v.in_handle = [v.in_handle[0] * k, v.in_handle[1] * k];
            v.out_handle = [v.out_handle[0] * k, v.out_handle[1] * k];
        });
        p
    };
    let ratio = |k: f64| -> f64 {
        let p = scaled(k);
        let out = repeat_path(&p, &spec(5.0, 60.0, 0.0));
        let (lo, hi) = bbox(&out);
        (hi[0] - lo[0]) / (40.0 * k)
    };
    let (small, big) = (ratio(1.0), ratio(7.0));
    assert!(
        (small - big).abs() / small < 1e-9,
        "a mesma percentagem deu {small} na forma pequena e {big} na grande"
    );
    assert!(small > 2.0, "e o array tem de existir de facto ({small})");
}

/// **A transformação é CUMULATIVA**: a cópia `k` leva o passo `k` vezes.
///
/// Com só translação isto é uma progressão aritmética, e é o que se mede: as distâncias entre
/// cópias consecutivas são todas iguais. Um passo aplicado UMA vez a todas daria três cópias
/// empilhadas no mesmo sítio.
#[test]
fn the_transform_compounds_copy_by_copy() {
    let p = square();
    let out = repeat_path(&p, &spec(4.0, 50.0, 0.0));
    let x_of = |c: usize| out.contour(c).expect("contorno").0[0].anchor[0];
    let steps: Vec<f64> = (1..4).map(|c| x_of(c) - x_of(c - 1)).collect();
    for s in &steps {
        assert!(
            (s - 20.0).abs() < 1e-6,
            "os passos deviam ser todos 20 (50% de 40), foram {steps:?}"
        );
    }
}

/// **Rodar sozinho gira em torno do CENTRO da forma**, não da origem do mundo.
///
/// O quadrado está em `0..40`, então o centro é `(20,20)` e a origem do mundo NÃO é o centro —
/// se a âncora fosse a origem, meia-volta mandaria a cópia para o terceiro quadrante. É o gate
/// que separa as duas, e por isso o fixture não pode estar centrado na origem.
#[test]
fn rotation_turns_about_the_shapes_centre_not_the_world_origin() {
    let p = square();
    let out = repeat_path(
        &p,
        &RepeatSpec {
            copies_x: 2.0,
            orbit: 180.0,
            ..RepeatSpec::default()
        },
    );
    let (lo, hi) = bbox(&out);
    // Meia-volta em torno de (20,20) devolve o quadrado a si mesmo: a caixa não cresce.
    assert!(
        (lo[0] - 0.0).abs() < 1e-6 && (hi[0] - 40.0).abs() < 1e-6,
        "a caixa foi de {lo:?} a {hi:?}; em torno da ORIGEM ela teria ido a −40"
    );
}

/// **Um compound é copiado INTEIRO** — o buraco viaja com a ilha.
///
/// É a razão de o Repeater não passar pelo `apply_per_contour`: tratar os contornos
/// independentemente copiaria o buraco por conta própria, e um buraco sozinho deixa de ser
/// buraco. O oráculo é a CONTAGEM por cópia, e a `fill_rule` tem de sobreviver.
#[test]
fn a_compound_is_repeated_whole_hole_and_all() {
    let mut p = square();
    p.subpaths.push(Contour {
        verts: [[10.0, 10.0], [30.0, 10.0], [30.0, 30.0], [10.0, 30.0]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
    });
    p.fill_rule = FillRule::EvenOdd;
    let out = repeat_path(&p, &spec(3.0, 150.0, 0.0));
    assert_eq!(
        out.contour_count(),
        6,
        "3 cópias × (ilha + buraco) = 6 contornos"
    );
    assert_eq!(
        out.fill_rule,
        FillRule::EvenOdd,
        "a regra tem de sobreviver"
    );
    // E cada cópia mantém a ilha e o buraco JUNTOS: a caixa de cada par é a mesma, deslocada.
    for c in (0..6).step_by(2) {
        let ilha = out.contour(c).expect("ilha").0;
        let buraco = out.contour(c + 1).expect("buraco").0;
        let dx = buraco[0].anchor[0] - ilha[0].anchor[0];
        assert!(
            (dx - 10.0).abs() < 1e-6,
            "no par {c} o buraco caiu a {dx} da ilha, e no original está a 10"
        );
    }
}

/// A fonte é lida ANTES de a saída crescer — senão as cópias copiavam-se umas às outras e a
/// contagem explodia exponencialmente. O gate mede a contagem exata num número de cópias alto o
/// suficiente para a diferença ser inconfundível (8 vs 128).
#[test]
fn the_copies_are_made_from_the_source_not_from_the_growing_output() {
    let p = square();
    let out = repeat_path(&p, &spec(8.0, 30.0, 0.0));
    assert_eq!(out.contour_count(), 8);
}

/// **Os dois eixos juntos dão uma GRELHA** — `nx × ny` contornos, e é isso que o Enio pediu
/// (2026-07-18) quando falou em copiar nas duas direções.
///
/// No Blender uma grelha faz-se empilhando dois modificadores Array; aqui sai de um só, porque
/// cada eixo tem a sua contagem. A contagem **é** o interruptor do eixo: `1` desliga-o.
#[test]
fn the_two_axes_make_a_grid() {
    let p = square();
    let out = repeat_path(
        &p,
        &RepeatSpec {
            copies_x: 5.0,
            move_x: 100.0,
            copies_y: 3.0,
            move_y: 100.0,
            ..RepeatSpec::default()
        },
    );
    assert_eq!(out.contour_count(), 15, "5 × 3");
    let (lo, hi) = bbox(&out);
    assert!(
        (hi[0] - lo[0] - 200.0).abs() < 1e-6 && (hi[1] - lo[1] - 120.0).abs() < 1e-6,
        "a grelha devia medir 5×40 por 3×40 = 200×120; mede {}×{}",
        hi[0] - lo[0],
        hi[1] - lo[1]
    );
}

/// **Um eixo a 1 é um eixo DESLIGADO** — sem toggle separado, porque duas respostas a
/// *"este eixo está ligado?"* divergem.
#[test]
fn an_axis_at_one_copy_is_off() {
    let p = square();
    let row = repeat_path(
        &p,
        &RepeatSpec {
            copies_x: 4.0,
            move_x: 100.0,
            copies_y: 1.0,
            move_y: 300.0, // um passo enorme que NÃO pode ser usado
            ..RepeatSpec::default()
        },
    );
    assert_eq!(row.contour_count(), 4);
    let (lo, hi) = bbox(&row);
    assert!(
        (hi[1] - lo[1] - 40.0).abs() < 1e-6,
        "o eixo y está desligado, então a fileira mede a altura de UMA forma; mede {}",
        hi[1] - lo[1]
    );
}

/// **Spin e Orbit são ferramentas DIFERENTES, e as duas ficam.**
///
/// ⚠️ A 1ª versão tinha só a órbita; a 2ª substituiu-a pelo spin. **Substituir era o erro**
/// (Enio, 2026-07-18: *"o outro tipo de rot também era útil"*). O oráculo mostra porquê: sobre
/// uma cópia DESLOCADA, girar sobre si mesma deixa-a onde está e girar em torno da origem
/// leva-a para outro sítio. Num par de gates só, isto é a diferença entre um leque e um arranjo
/// radial.
#[test]
fn spin_turns_in_place_and_orbit_moves_the_copy() {
    let p = square();
    let centre_of = |s: &RepeatSpec| -> [f64; 2] {
        let out = repeat_path(&p, s);
        // O centro do 2º contorno — a 1ª cópia.
        let (v, _) = out.contour(1).expect("cópia");
        let (mut lo, mut hi) = ([f64::MAX; 2], [f64::MIN; 2]);
        for w in v {
            for k in 0..2 {
                lo[k] = lo[k].min(w.anchor[k]);
                hi[k] = hi[k].max(w.anchor[k]);
            }
        }
        [(lo[0] + hi[0]) * 0.5, (lo[1] + hi[1]) * 0.5]
    };
    let base = RepeatSpec {
        copies_x: 2.0,
        move_x: 150.0,
        ..RepeatSpec::default()
    };
    let plain = centre_of(&base);
    let spun = centre_of(&RepeatSpec { spin: 90.0, ..base });
    let orbited = centre_of(&RepeatSpec {
        orbit: 90.0,
        ..base
    });

    let d = |a: [f64; 2], b: [f64; 2]| (a[0] - b[0]).hypot(a[1] - b[1]);
    assert!(
        d(plain, spun) < 1e-6,
        "o SPIN roda a cópia sobre si mesma — o centro dela não pode andar ({plain:?} -> {spun:?})"
    );
    assert!(
        d(plain, orbited) > 10.0,
        "a ÓRBITA tem de LEVAR a cópia para outro sítio ({plain:?} -> {orbited:?}); se não leva, \
         as duas rotações são a mesma e uma delas é um botão morto"
    );
}

/// O produto dos dois eixos tem um teto próprio — o teto de UM eixo não é o teto de uma grelha.
#[test]
fn the_grid_has_a_ceiling_of_its_own() {
    let p = square();
    let out = repeat_path(
        &p,
        &RepeatSpec {
            copies_x: 128.0,
            move_x: 100.0,
            copies_y: 128.0,
            move_y: 100.0,
            ..RepeatSpec::default()
        },
    );
    assert!(
        out.contour_count() <= 1024,
        "128 × 128 seriam 16 384 contornos; ficaram {}",
        out.contour_count()
    );
    assert!(out.contour_count() > 900, "e o teto não pode ser mesquinho");
}
