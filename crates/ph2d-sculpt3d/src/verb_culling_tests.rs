//! Gates do CONJUNTO FRONTAL — o que o pincel usa para decidir a DIREÇÃO.
//!
//! Irmão do `verb_tests.rs`, cortado por ASSUNTO: estes dois medem o ajuste do
//! plano sobre o que o artista VÊ, e carregam fixtures de silhueta que nenhum
//! outro verbo precisa (a esfera grande com um bojo assimétrico do lado
//! invisível, e o dab de olho invertido).
//!
//! Filho do MESMO pai, então `use super::*` alcança `sphere`/`snapshot`/`dab_at`.

use super::*;

/// Mexer na geometria que o artista **não vê** não muda o que o pincel faz.
///
/// ⚠️ **A metade do culling que entra é a INCONDICIONAL**, e ela não é sobre
/// quem se move: o `getFrontVertices` (`SculptBase.js:206-221`) alimenta a
/// normal de área e o ajuste do plano em `Brush.js:32-34` e `Flatten.js:25-27`,
/// sem checkbox nenhum. Filtrar *o que se MOVE* é a outra metade — `_culling`,
/// **`false` por default em dez tools** —, e portá-la ligada seria divergir.
///
/// ⚠️ **E a fixture precisou de três tentativas, todas cegas pelo mesmo motivo.**
/// *"O Draw empurra na direção do olho"* mede a curvatura da esfera (perto da
/// silhueta a superfície É de perfil, e a normal do centro encara o olho a
/// 0,49). *"O Draw segue a normal sob o pincel"* mede **1,0000 nos DOIS mundos**:
/// numa esfera a calota é simétrica em torno do eixo do dab, então os vértices
/// de costas encurtam a média sem inclinar nada — eles cancelam entre si.
///
/// O que contém o fenômeno é uma pegada onde o lado invisível é **assimétrico**:
/// aí, sem o filtro, ele inclina o plano de verdade.
#[test]
fn geometry_behind_the_silhouette_does_not_steer_the_brush() {
    let eye = [0.0, 0.0, -1.0];
    // Na silhueta (82 graus do eixo do olho) e com pegada que alcança bem o
    // outro lado — sem isso não há vértice que seja ao mesmo tempo INVISÍVEL e
    // dentro do pincel, e a premissa do gate não fecha.
    let centre = [0.99, 0.0, 0.141];
    let radius = 0.9;

    // `None` = a esfera limpa · `Behind` = a mancha invisível · `Front` = a MESMA
    // mancha, do lado que se vê. O terceiro é o CONTROLE, e sem ele o gate
    // precisaria de uma barra escolhida à mão entre dois números.
    #[derive(Clone, Copy, PartialEq)]
    enum Bump {
        None,
        Behind,
        Front,
    }

    let push = |bump: Bump| -> [f32; 3] {
        let mut mesh = ph2d_mesh::shapes::uv_sphere(96, 144, 1.0);
        if bump != Bump::None {
            // Só o que está BEM de costas, e só de um lado em `y`: a assimetria
            // é o fenômeno. A faixa perto da silhueta fica intocada para o
            // conjunto FRONTAL não mudar junto — se ele mudasse, o gate mediria
            // duas coisas ao mesmo tempo.
            let mut moved = Vec::new();
            for v in 0..mesh.vert_count() {
                let p = mesh.positions()[v];
                let n = mesh.normals()[v];
                let d = [p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]];
                let inside = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= radius;
                let facing = n[0] * eye[0] + n[1] * eye[1] + n[2] * eye[2];
                let chosen = if bump == Bump::Behind {
                    facing > 0.35
                } else {
                    facing < -0.35
                };
                if inside && chosen && p[1] > 0.0 {
                    moved.push(v as u32);
                }
            }
            assert!(
                moved.len() > 50,
                "a fixture não tem lado invisível dentro da pegada: {} vértices",
                moved.len()
            );
            let out = mesh.positions_mut();
            for &v in &moved {
                let p = out[v as usize];
                out[v as usize] = [p[0] * 1.25, p[1] * 1.25, p[2] * 1.25];
            }
            mesh.rebuild();
        }
        let base = snapshot(&mesh);
        let b = Brush {
            verb: Verb::Draw,
            radius,
            strength: 1.0,
            falloff: Falloff::Smooth,
            ..Brush::default()
        };
        let mut s = SculptStroke::default();
        s.begin(&mesh);
        s.dab(
            &mut mesh,
            &b,
            &Dab::at(centre, radius, eye),
            Symmetry::default(),
        );
        let (dir, len) = mesh
            .positions()
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let d = [p[0] - base[i][0], p[1] - base[i][1], p[2] - base[i][2]];
                (d, (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
            })
            .fold(
                ([0.0; 3], 0.0f32),
                |acc, x| if x.1 > acc.1 { x } else { acc },
            );
        assert!(len > 1e-5, "o dab não moveu nada");
        [dir[0] / len, dir[1] / len, dir[2] / len]
    };

    let tilt_of = |a: [f32; 3], b: [f32; 3]| {
        let agree = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        agree.clamp(-1.0, 1.0).acos().to_degrees()
    };
    let plain = push(Bump::None);
    let behind = tilt_of(plain, push(Bump::Behind));
    let front = tilt_of(plain, push(Bump::Front));
    println!("o bojo INVISÍVEL inclinou o Draw em {behind:.3}° · o VISÍVEL em {front:.3}°");
    assert!(
        front > 1.0,
        "a mancha VISÍVEL não inclinou nada ({front:.3}°) — o controle não \
         contém o fenômeno, e o gate mediria vácuo"
    );
    // ⚠️ **O oráculo é o CONTROLE, e não uma barra.** A mesma mancha, do lado
    // que se vê, é o que o pincel DEVE seguir; do lado que não se vê, o filtro
    // frontal a recusa. Uma barra absoluta aqui envelhece com a ponderação do
    // plano — ela envelheceu: era `< 0,5°` e a troca para o `area_normal` da
    // referência (que pesa por MÁSCARA, uniforme, e não por falloff) levou o
    // resíduo a `0,664°`.
    //
    // ⚠️ **E o resíduo tem MECANISMO:** bumpar a geometria de costas move as
    // faces que ela COMPARTILHA com a faixa da frente, então as normais dos
    // vértices frontais vizinhos giram um pouco — não é o filtro vazando, é a
    // malha sendo uma malha. Com o filtro derrotado o mesmo bump inclina
    // `3,232°`, ou seja ele compra **4,9×**.
    assert!(
        behind < front * 0.25,
        "a geometria de costas girou o pincel em {behind:.3}° contra {front:.3}° \
         da mesma mancha visível — ela está pesando num ajuste que devia ser só \
         do que se vê"
    );
}

/// Uma pegada INTEIRAMENTE de costas ainda ajusta um plano são.
///
/// ⚠️ Pelo pick esta situação é inalcançável — o `raycast` devolve o acerto mais
/// PRÓXIMO, cuja face encara o olho por construção, então o centro do dab é
/// sempre frontal. Mas o `Dab::at` é público e o eixo do olho é um argumento:
/// sem o recuo, o filtro esvaziaria a soma, o `sum <= 0` devolveria um plano
/// degenerado, e o verbo empurraria ao longo de um `+Y` que ninguém escolheu.
#[test]
fn a_footprint_entirely_facing_away_still_fits_a_sane_plane() {
    let centre = [0.0, 0.0, 1.0];
    let radius = 0.4;
    // O olho ao contrário: todo vértice sob o pincel está "de costas".
    let backwards = [0.0, 0.0, 1.0];
    let mut mesh = sphere();
    let base = snapshot(&mesh);
    let b = Brush {
        verb: Verb::Draw,
        radius,
        strength: 1.0,
        ..Brush::default()
    };
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    s.dab(
        &mut mesh,
        &b,
        &Dab::at(centre, radius, backwards),
        Symmetry::default(),
    );

    let (dir, len) = mesh
        .positions()
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let d = [p[0] - base[i][0], p[1] - base[i][1], p[2] - base[i][2]];
            (d, (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt())
        })
        .fold(
            ([0.0; 3], 0.0f32),
            |acc, x| if x.1 > acc.1 { x } else { acc },
        );
    assert!(len > 1e-5, "o dab não moveu nada");
    // A resposta certa continua sendo a superfície sob o pincel — o `+Y` do
    // plano degenerado seria perpendicular a ela.
    assert!(
        dir[2] / len > 0.99,
        "o recuo devolveu um plano que não descreve a superfície: {dir:?}"
    );
}
