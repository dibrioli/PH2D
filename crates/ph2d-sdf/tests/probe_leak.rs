//! Quantas resolucoes vazam, antes e depois.
use ph2d_mesh::{Mesh, shapes, shapes_open};
use ph2d_sdf::VoxelField;

fn leaks(closed: &Mesh, res: u32) -> bool {
    let mut f = VoxelField::for_bounds(closed.bounds(), res);
    f.voxelize(closed);
    f.flood_fill() == 0
}

fn closed_of(m: &Mesh) -> Mesh {
    let mut c = Mesh::from_parts(m.positions().to_vec(), m.faces().to_vec()).unwrap();
    let _ = ph2d_mesh::fill_holes(&mut c);
    c
}

#[test]
#[ignore = "sonda"]
fn how_many_resolutions_leak() {
    for (name, m) in [
        ("esfera uv(96,144)", shapes::uv_sphere(96, 144, 1.0)),
        ("esfera uv(24,32)", shapes::uv_sphere(24, 32, 1.0)),
        ("cubo", shapes::cube(1.0)),
        ("tubo aberto", shapes_open::open_tube3()),
    ] {
        let c = closed_of(&m);
        let bad: Vec<u32> = (40u32..=400).filter(|r| leaks(&c, *r)).collect();
        eprintln!(
            "{name:20} vazam {:3} de 361  {}",
            bad.len(),
            if bad.len() <= 14 {
                format!("{bad:?}")
            } else {
                format!("[{:?} ...]", &bad[..14])
            }
        );
    }
}

/// **A FRAÇÃO do volume que o campo de fato encontra** — a régua candidata para
/// uma recusa que pegue o vazamento PARCIAL.
///
/// O guard que shipa recusa `inside == 0`, e o log do produto mostrou um caso
/// que ele NÃO pega: `567828 -> 40 vertices`. Um campo que vaza *quase* todo
/// deixa poucas células dentro, passa pelo guard e devolve um caco com `Ok`.
///
/// Uma malha fechada tem volume conhecido (teorema da divergência, `O(faces)`),
/// e o campo devia encontrá-lo a menos do erro de discretização. Esta sonda diz
/// qual é a banda saudável e qual é a de um vazamento — e é ela que escolhe o
/// número, não eu.
#[test]
#[ignore = "sonda"]
fn what_fraction_of_the_volume_the_field_finds() {
    for (name, m) in [
        ("esfera uv(96,144)", shapes::uv_sphere(96, 144, 1.0)),
        ("cubo", shapes::cube(1.0)),
        ("tubo aberto", shapes_open::open_tube3()),
    ] {
        let c = closed_of(&m);
        let want = ph2d_mesh::signed_volume(&c).abs();
        let mut lo = f32::MAX;
        let mut hi = 0.0f32;
        let mut leaked: Vec<(u32, f32)> = Vec::new();
        for res in 40u32..=400 {
            let mut f = VoxelField::for_bounds(c.bounds(), res);
            f.voxelize(&c);
            let inside = f.flood_fill();
            let step = f.step();
            let got = inside as f32 * step * step * step;
            let frac = got / want;
            if frac < 0.5 {
                leaked.push((res, frac));
            } else {
                lo = lo.min(frac);
                hi = hi.max(frac);
            }
        }
        eprintln!(
            "{name:20} volume {want:8.4}  banda SÃ [{lo:.4} .. {hi:.4}]  \
             fora-da-banda {} {:?}",
            leaked.len(),
            &leaked[..leaked.len().min(6)]
        );
    }
}

/// **PERTURBAR A GRADE cura o vazamento?** — a saída padrão para
/// degenerescência em geometria computacional, testada nos dois casos que
/// reproduzem.
///
/// O mecanismo do vazamento é uma amostra da grade caindo EXATAMENTE sobre a
/// superfície: a travessia pousa na fronteira compartilhada entre duas janelas
/// de aresta consecutivas e o arredondamento a expulsa das duas. Deslocar a
/// origem da grade por uma fração do passo torna a coincidência exata
/// medida-zero — sem mudar o modelo, só a FASE em que ele é amostrado.
///
/// ⚠️ A sonda desloca a CAIXA, não a malha: a origem sai de `bounds.min`, então
/// baixar o mínimo move a grade e deixa a peça onde está (e só ALARGA a folga,
/// nunca a encurta).
#[test]
#[ignore = "sonda"]
fn does_nudging_the_grid_cure_the_leak() {
    let c = closed_of(&shapes_open::open_tube3());
    let want = ph2d_mesh::signed_volume(&c).abs();
    let b = c.bounds();
    let ext = [
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2],
    ];
    let longest = ext[0].max(ext[1]).max(ext[2]);
    for res in [280u32, 377] {
        eprint!("res {res}:");
        let step = longest / res as f32;
        for frac in [0.0f32, 0.1, 0.25, 0.381_966, 0.5, 0.618_034] {
            let d = step * frac;
            let moved = ph2d_mesh::Aabb {
                min: [b.min[0] - d, b.min[1] - d, b.min[2] - d],
                max: b.max,
            };
            let mut f = VoxelField::for_bounds(moved, res);
            f.voxelize(&c);
            let inside = f.flood_fill();
            let s = f.step();
            let got = inside as f32 * s * s * s / want;
            eprint!("  {frac:.3}->{got:.3}");
        }
        eprintln!();
    }
}
