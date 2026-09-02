//! Os gates da LEI DO ÁPICE — o suporte por ponta, o censo, e a forma que separa um espinho
//! de uma bossa. Irmão de `local_tests.rs`, cortado com o módulo em 2026-09-02.

use super::super::local::dist;

/// ⭐⭐⭐ **UMA PONTA CORTADA E UMA REAMOSTRADA TÊM DE SER DISTINGUÍVEIS.**
///
/// ⛔⛔ **É a barra [`super::TIP_CUT_PCT`] que faz a distinção, e sem este gate ela não
/// é exercitada por nada** — a mutação que a punha em `0` (⇒ *toda* ponta conta como
/// cortada) **sobreviveu** ao gate da linha do relatório, que constrói a contagem à mão.
///
/// ⚠️ **As duas populações estão separadas por uma ordem de grandeza**, e é aí que a
/// barra vive: medido na peça do artista, as pontas intactas dão `−0,0 %` a `−0,4 %` e
/// as cortadas `−5 %` a `−22 %`.
#[test]
fn uma_ponta_cortada_distingue_se_de_uma_reamostrada() {
    use ph2d_mesh::{Face, Mesh};
    // Uma tenda: anel de quatro na base, um ápice em cima.
    let tent = |apex_z: f32| -> Mesh {
        Mesh::from_parts(
            vec![
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, 0.0, 0.0],
                [0.0, -1.0, 0.0],
                [0.0, 0.0, apex_z],
            ],
            vec![
                Face::tri(0, 1, 4),
                Face::tri(1, 2, 4),
                Face::tri(2, 3, 4),
                Face::tri(3, 0, 4),
                Face::quad(3, 2, 1, 0),
            ],
        )
        .expect("a fixtura e' construida aqui")
    };
    let entrada = tent(3.0);

    let cortada = super::tip_survival(&entrada, &tent(2.4));
    assert_eq!(cortada.total, 1, "a tenda tem UM apice");
    assert_eq!(
        cortada.cut, 1,
        "uma ponta 20 % mais curta tem de contar como CORTADA (mediu {:.1} %)",
        cortada.worst_pct
    );

    // ⚠️ **O CONTROLO, e é ele que a mutação da barra mata:** uma perda de reamostragem
    // (a saída é poliédrica e os vértices não se correspondem) **não** é amputação.
    let quase = super::tip_survival(&entrada, &tent(2.985));
    assert_eq!(
        quase.total, 1,
        "o controlo tem de medir a mesma ponta, senao ele nao e' controlo"
    );
    assert_eq!(
        quase.cut, 0,
        "uma perda de {:.2} % e' reamostragem, nao amputacao -- se ela contar, a coluna \
         acusa toda peca e o artista deixa de a ler",
        quase.worst_pct
    );
}

/// **UMA ESFERA COM TRÊS FEIÇÕES** — um espinho LONGO em `+x`, um espinho CURTO em `+y` e uma
/// BOSSA larga em `−x`, os dois últimos com o **mesmo raio** ao centro. Cada feição é uma
/// gaussiana radial `H · exp(−θ² / 2σ²)` sobre a esfera unitária.
fn esfera_com_tres_feicoes() -> ph2d_mesh::Mesh {
    let base = ph2d_mesh::shapes::uv_sphere(64, 96, 1.0);
    let feicoes: [([f32; 3], f32, f32); 3] = [
        ([1.0, 0.0, 0.0], 2.0, 0.15),  // longo:  ápice a raio 3,0 = `far`
        ([0.0, 1.0, 0.0], 0.5, 0.15),  // curto:  ápice a raio 1,5 = 0,50 · far
        ([-1.0, 0.0, 0.0], 0.5, 0.50), // bossa:  ápice a raio 1,5 = 0,50 · far, mas LARGA
    ];
    let verts: Vec<[f32; 3]> = base
        .positions()
        .iter()
        .map(|p| {
            let n = {
                let l = p[0].mul_add(p[0], p[1].mul_add(p[1], p[2] * p[2])).sqrt();
                [p[0] / l, p[1] / l, p[2] / l]
            };
            let mut r = 1.0f32;
            for (d, h, sigma) in feicoes {
                let c = n[0]
                    .mul_add(d[0], n[1].mul_add(d[1], n[2] * d[2]))
                    .clamp(-1.0, 1.0);
                let theta = c.acos();
                r += h * (-theta * theta / (2.0 * sigma * sigma)).exp();
            }
            [n[0] * r, n[1] * r, n[2] * r]
        })
        .collect();
    ph2d_mesh::Mesh::from_parts(verts, base.faces().to_vec()).expect("a fixtura e' construida aqui")
}

/// ⭐⭐⭐ **GATE — a lei do ápice vê o espinho CURTO e não chama «ponta» à BOSSA do mesmo
/// raio.**
///
/// ⛔⛔⛔ **É o gate do piso de `0,55` que escondia as pontas da foto** (2026-09-02): o
/// espinho curto está a `0,50` do raio máximo — abaixo do piso antigo, logo invisível a toda
/// régua desta linha — e a bossa está ao MESMO raio, logo um piso mais baixo sozinho
/// contá-la-ia. *O que separa os dois é a FORMA, e a forma mede-se a `4 h` do bico.*
#[test]
fn a_lei_do_apice_ve_o_espinho_curto_e_nao_ve_a_bossa_do_mesmo_raio() {
    let mesh = esfera_com_tres_feicoes();
    let unit = super::median_edge(&mesh);
    assert!(
        unit > 0.0 && unit < 0.1,
        "a esfera 64x96 tem aresta mediana ~0,05: {unit}"
    );
    let (mid, apex) = super::apices(&mesh, unit);
    let pos = mesh.positions();
    let far = pos.iter().map(|p| dist(*p, mid)).fold(0.0f32, f32::max);
    let dir = |i: usize| {
        let p = pos[i];
        let l = dist(p, [0.0; 3]);
        [p[0] / l, p[1] / l, p[2] / l]
    };
    assert_eq!(
        apex.len(),
        2,
        "dois espinhos e nenhuma bossa; a lei devolveu {:?}",
        apex.iter()
            .map(|&i| (dir(i), dist(pos[i], mid) / far))
            .collect::<Vec<_>>()
    );
    let longo = apex[0];
    let curto = apex[1];
    assert!(
        dir(longo)[0] > 0.99,
        "o mais longo e' o de +x: {:?}",
        dir(longo)
    );
    assert!(dir(curto)[1] > 0.99, "o curto e' o de +y: {:?}", dir(curto));
    let raio_curto = dist(pos[curto], mid) / far;
    assert!(
        raio_curto < 0.55,
        "⛔ o espinho curto esta' a {raio_curto:.2} do raio -- ABAIXO do piso antigo de 0,55, \
         que e' exactamente por que a foto e a regua discordavam"
    );

    // ⭐ **E a razão pela qual a bossa fica de fora é o CONE**, não o raio: o ápice dela é um
    // máximo local acima do piso, com o mesmo raio do espinho curto.
    let nbr = super::adjacency(&mesh);
    let bossa = (0..pos.len())
        .filter(|&i| dir(i)[0] < -0.99)
        .max_by(|&a, &b| dist(pos[a], mid).total_cmp(&dist(pos[b], mid)))
        .expect("a bossa tem um apice");
    assert!(
        (dist(pos[bossa], mid) / far - raio_curto).abs() < 0.02,
        "a bossa e o espinho curto tem o MESMO raio por construcao"
    );
    let cone_bossa = super::cone_of(pos, &nbr, bossa, unit).expect("a bossa tem anel");
    let cone_curto = super::cone_of(pos, &nbr, curto, unit).expect("o espinho tem anel");
    assert!(
        cone_bossa > super::CONE_MAX && cone_curto < super::CONE_MAX,
        "bossa {cone_bossa:.2} contra espinho {cone_curto:.2}, barra {}",
        super::CONE_MAX
    );

    // ⛔ Sem unidade não há cone, e sem cone a lista seria a de todos os máximos locais.
    assert!(super::apices(&mesh, 0.0).1.is_empty());
    assert!(super::apices(&mesh, f32::NAN).1.is_empty());
}

/// ⭐⭐ **Uma grade mais grossa nunca PERDE um espinho verdadeiro** — e o limite da lei está
/// nomeado, não escondido.
///
/// ⛔ A 1.ª redacção deste gate afirmava o contrário do que a geometria faz: *«a uma grade
/// `4×` mais grossa o espinho curto vira bossa»*. Medido, é o INVERSO — visto do ápice, um
/// corpo esférico de raio `R` lê `r/t = cot(θ/2)`, que a `unit ≳ 0,15 R` desce abaixo de `1`
/// e faz o **corpo** parecer cónico. A `4 × unit = 0,26 R` a esfera inteira lê espinho, e o
/// que o gate pode afirmar honestamente é: o espinho longo sobrevive, e o censo só CRESCE.
/// O limite vive no doc de [`super::cone_of`], com o número.
#[test]
fn uma_grade_mais_grossa_nunca_perde_o_espinho_longo() {
    let mesh = esfera_com_tres_feicoes();
    let unit = super::median_edge(&mesh);
    let (_, fina) = super::apices(&mesh, unit);
    let (_, grossa) = super::apices(&mesh, 2.0 * unit);
    assert_eq!(fina.len(), 2, "{fina:?}");
    assert!(
        grossa.contains(&fina[0]) && grossa.contains(&fina[1]),
        "a 2 h os dois espinhos continuam a ser espinhos: {grossa:?}"
    );
    assert!(
        grossa.len() >= fina.len(),
        "o censo so' cresce com a grade mais grossa: {} contra {}",
        grossa.len(),
        fina.len()
    );
}
