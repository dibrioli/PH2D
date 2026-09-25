//! ⭐⭐⭐⭐ **Os gates da GRELHA DA LUZ DEVOLVIDA AO CHÃO** — o report do dono de 2026-09-24 (foto da
//! cena `=28` com a lâmpada encostada ao nó: *«qualidade do render melhor mas ainda com áreas
//! retangulares ruins»*).
//!
//! Medido antes da cura (a sonda `diag_o_campo_do_chao_visto_de_cima` da `ph2d-app-field3d`): com a
//! lâmpada encostada, a mancha acesa da peça age como uma SEGUNDA lâmpada e projecta no chão riscas
//! de sombra muito mais finas que uma célula. A grelha tinha DOIS defeitos, e cada um tem o seu
//! gate:
//!
//! 1. **o nó lia um PONTO** — cada nó apanhava uma risca ou um vão ao acaso (a grelha DOBRAVA a
//!    feição fina em manchas do tamanho de uma célula), e subir a grelha de `32²` para `128²` quase
//!    não mexia no erro (`61 %` → `57 %` do pico, no pior ponto);
//! 2. **a leitura era BILINEAR** — contínua, com a derivada a saltar em cada linha da grelha: com
//!    contraste, o chão desenha os losangos das células.

use crate::ground_bounce::{
    GROUND_BOUNCE_DIRS, GROUND_BOUNCE_GRID, GroundBounce, bake_ground_bounce,
};
use crate::{Ground, Orbit, PointLamp, Surfaces};
use ph2d_field::{FieldDoc, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;
use ph2d_material::OpenPbr;

/// Um campo sintético com contraste de uma célula para a outra — um xadrez de valores.
fn xadrez(n: usize) -> GroundBounce {
    GroundBounce {
        origin: [0.0, 0.0],
        step: 1.0,
        n,
        height: 0.0,
        value: (0..n * n)
            .map(|k| {
                let v = if (k % n + k / n).is_multiple_of(2) {
                    1.0
                } else {
                    0.2
                };
                [v, v, v]
            })
            .collect(),
    }
}

/// A bilinear da lei antiga — só para o CONTROLO (a fixtura tem de conter o vinco que ela desenha).
fn bilinear(c: &GroundBounce, x: f32, z: f32) -> f32 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let (i, j) = (x.floor() as usize, z.floor() as usize);
    #[allow(clippy::cast_precision_loss)]
    let (fx, fz) = (x - i as f32, z - j as f32);
    let v = |a: usize, b: usize| c.value[b * c.n + a][0];
    (1.0 - fz) * ((1.0 - fx) * v(i, j) + fx * v(i + 1, j))
        + fz * ((1.0 - fx) * v(i, j + 1) + fx * v(i + 1, j + 1))
}

/// O maior SALTO DE INCLINAÇÃO ao atravessar uma linha da grelha, em unidades da maior inclinação
/// ao longo da linha INTEIRA — `0` para uma curva `C¹`, da ordem de `1` para uma que vira a direito
/// na linha.
///
/// ⚠️ A normalização é pela linha inteira e não pelas inclinações junto às linhas: num xadrez a
/// B-spline tem os extremos EXACTAMENTE nos nós, logo ali as duas inclinações são quase zero e o
/// quociente delas lia `2,0` — a mesma leitura que a bilinear, sobre curvas opostas.
fn vinco(f: &dyn Fn(f32) -> f32, de: usize, ate: usize) -> f32 {
    const H: f32 = 1.0 / 1024.0;
    let mut salto = 0.0f32;
    for linha in de..=ate {
        #[allow(clippy::cast_precision_loss)]
        let x = linha as f32;
        let (esq, dir) = ((f(x) - f(x - H)) / H, (f(x + H) - f(x)) / H);
        salto = salto.max((dir - esq).abs());
    }
    let mut maior = 0.0f32;
    #[allow(clippy::cast_precision_loss)]
    let (a, b) = (de as f32, ate as f32);
    let passos = 4096;
    for k in 0..passos {
        #[allow(clippy::cast_precision_loss)]
        let x = a + (b - a) * k as f32 / passos as f32;
        maior = maior.max(((f(x + H) - f(x)) / H).abs());
    }
    salto / maior.max(1e-9)
}

/// ⭐⭐⭐⭐ **A LEITURA DA GRELHA NÃO TEM VINCOS nas linhas das células** — a metade da RECONSTRUÇÃO.
///
/// Uma linha que atravessa o miolo de um xadrez, com a inclinação medida dos dois lados de cada
/// linha da grelha. ⭐ **O CONTROLO vive dentro:** a bilinear da lei antiga sobre o MESMO campo —
/// sem ele, a barra podia estar a medir uma fixtura onde nada vira.
#[test]
fn a_luz_do_chao_nao_desenha_as_celulas_da_grelha() {
    let c = xadrez(12);
    let z = 5.37f32;
    // ⚠️ Linhas `4..=7` a `z = 5,37`: a orla só começa a `75 %` do campo, logo ali ela vale `1`.
    let produto = vinco(&|x| c.sample([x, 0.0, z])[0], 4, 7);
    let antiga = vinco(&|x| bilinear(&c, x, z), 4, 7);
    println!("vinco: B-spline {produto:.4} · bilinear {antiga:.4}");
    assert!(
        antiga > 0.5,
        "CONTROLO: a bilinear tem de virar a direito nas linhas desta fixtura (vinco {antiga:.4})"
    );
    assert!(
        produto < 0.02,
        "a leitura da grelha desenha as células: a inclinação salta {produto:.4} numa linha"
    );
}

/// O nó de toro `(2, 3)` da cena `=28` do report, com a lâmpada ENCOSTADA a ele (`0,069` da peça,
/// o regime da foto: `0,094` na cena do dono).
fn no_e_luz() -> (FieldDoc, Ground, PointLamp) {
    let (radius, tube, winds, loops) = (0.20f32, 0.085f32, 2u32, 3u32);
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::TorusKnot {
                radius,
                tube,
                cord: ph2d_field::knot_cord_ceiling(radius, tube, winds, loops) * 0.85,
                winds,
                loops,
            },
            Xform::IDENTITY,
        )],
        ph2d_field::NodeId(0),
    )
    .expect("o nó");
    let chao = crate::lowest_point(&doc, &Registry::new())
        .map(|height| Ground { height })
        .expect("o chão");
    let luz = PointLamp {
        world: [0.0, 0.0, -0.15],
        radiance_at_one: [7.0, 7.0, 7.0],
    };
    // ⚠️ A fixtura só contém o fenómeno com a lâmpada ENCOSTADA: é isso que faz a mancha acesa da
    // peça ser pequena e as sombras dela no chão mais finas que uma célula.
    let d = ph2d_field_eval::Field::new(&doc).at(0.0, 0.0, -0.15);
    assert!(
        (0.03..0.12).contains(&d),
        "FIXTURA: a lâmpada tem de estar encostada à peça (a {d:.4})"
    );
    (doc, chao, luz)
}

/// ⭐⭐⭐⭐ **CADA NÓ VALE A MÉDIA DA SUA CÉLULA** — a metade do PRÉ-FILTRO.
///
/// A régua é uma assadura `4×` mais fina (`125²`, cujos nós caem em quartos da célula grossa),
/// dobrada à média de cada célula grossa com os pesos do trapézio. Um nó pontual lê a feição fina
/// que calhar em cima dele; um nó pré-filtrado lê a média — o que a régua também lê.
#[test]
fn cada_no_da_grelha_do_chao_vale_a_media_da_celula() {
    let (doc, chao, luz) = no_e_luz();
    let reg = Registry::new();
    let cam = Orbit::default();
    let mats = [OpenPbr::default().prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let n = GROUND_BOUNCE_GRID;
    let fino_n = 4 * (n - 1) + 1;
    let assa = |m: usize| {
        bake_ground_bounce(
            &doc,
            &reg,
            &cam,
            chao,
            &surfaces,
            &[luz],
            m,
            GROUND_BOUNCE_DIRS,
            720,
        )
    };
    let (grossa, fina) = (assa(n), assa(fino_n));
    let pontual = crate::ground_bounce::bake_ground_bounce_pontual(
        &doc,
        &reg,
        &cam,
        chao,
        &surfaces,
        &[luz],
        n,
        GROUND_BOUNCE_DIRS,
        720,
    );
    let lum = |v: [f32; 3]| (v[0] + v[1] + v[2]) / 3.0;
    let pico = fina.value.iter().map(|v| lum(*v)).fold(0.0f32, f32::max);
    let peso = |d: i64| if d.abs() == 2 { 0.5f32 } else { 1.0 };
    let mede = |campo: &GroundBounce| {
        let mut erros = Vec::new();
        // O miolo: a orla só começa a `75 %` do campo.
        let (de, ate) = (n / 8 + 1, n - n / 8 - 2);
        for iz in de..=ate {
            for ix in de..=ate {
                let (mut soma, mut w) = (0.0f32, 0.0f32);
                for dz in -2i64..=2 {
                    for dx in -2i64..=2 {
                        #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
                        let k = ((4 * iz) as i64 + dz) as usize * fino_n
                            + ((4 * ix) as i64 + dx) as usize;
                        let p = peso(dx) * peso(dz);
                        soma += p * lum(fina.value[k]);
                        w += p;
                    }
                }
                erros.push((lum(campo.value[iz * n + ix]) - soma / w).abs() / pico);
            }
        }
        erros.sort_by(f32::total_cmp);
        (erros[erros.len() - 1], erros[erros.len() * 95 / 100])
    };
    let (pior, p95) = mede(&grossa);
    let (pior_c, p95_c) = mede(&pontual);
    println!(
        "nó contra a média da célula (pico {pico:.4}): pré-filtrado pior {pior:.4} · p95 {p95:.4} \
         · pontual pior {pior_c:.4} · p95 {p95_c:.4}"
    );
    // Medido (2026-09-24): pré-filtrado pior `0,157` · p95 `0,043`; pontual pior `0,612` · p95
    // `0,030`. ⚠️ **O p95 NÃO separa as duas leis** (é o ruído das `128` direcções por nó, igual
    // nas duas) — o que as separa é o PIOR nó, o que caiu em cima de uma feição fina: a barra
    // mora no vale entre os dois.
    assert!(
        pior_c > 0.4,
        "CONTROLO: a leitura pontual tem de dobrar a feição fina desta fixtura (pior {pior_c:.4})"
    );
    assert!(
        pior < 0.3,
        "um nó da grelha do chão não vale a média da célula: desvia {pior:.4} do pico \
         (p95 {p95:.4}) — a assadura voltou a ler um ponto?"
    );
}
