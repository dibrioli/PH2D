//! ⭐⭐⭐ **O PINCEL DE TECIDO** — e estes gates existem porque ele SAIU de dois
//! censos.
//!
//! ⚠️ **Tirar um verbo de um censo é uma dívida até ele ter os próprios gates.**
//! O tecido não escreve `accum` nem `target` (ele desvia antes do laço), então
//! os censos do `stroke_apply` afirmariam vácuo sobre ele — a exclusão está feita
//! por LEI ([`Verb::writes_through_applicator`]) e o que se cobra dele está aqui.

use super::*;
use ph2d_mesh::Mesh;

fn plano() -> Mesh {
    // Uma grade PLANA, e não uma esfera: um pano tem de poder fazer prega, e numa
    // esfera de raio 1 a curvatura do repouso domina o que a régua vê.
    const N: usize = 24;
    let s = 2.0 / N as f32;
    let mut pos = Vec::new();
    for j in 0..=N {
        for i in 0..=N {
            pos.push([i as f32 * s - 1.0, j as f32 * s - 1.0, 0.0]);
        }
    }
    let id = |i: usize, j: usize| u32::try_from(j * (N + 1) + i).unwrap_or(u32::MAX);
    let mut faces = Vec::new();
    for j in 0..N {
        for i in 0..N {
            faces.push(ph2d_mesh::Face::quad(
                id(i, j),
                id(i + 1, j),
                id(i + 1, j + 1),
                id(i, j + 1),
            ));
        }
    }
    Mesh::from_parts(pos, faces).expect("grade plana")
}

fn pincel() -> Brush {
    Brush {
        verb: Verb::Cloth,
        radius: 0.30,
        strength: 1.0,
        ..Brush::default()
    }
}

/// Um dab olhando de `+z` para o plano.
fn dab_em(c: [f32; 3], r: f32) -> Dab {
    Dab::at(c, r, [0.0, 0.0, -1.0])
}

/// Arrasta a mão por `passos` eventos, devolvendo a malha e o traço.
fn arrastar(passos: usize, brush: &Brush) -> (Mesh, SculptStroke) {
    let mut mesh = plano();
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..passos {
        let c = [0.02 * k as f32, 0.0, 0.0];
        s.dab(
            &mut mesh,
            brush,
            &dab_em(c, brush.radius),
            Symmetry::default(),
        );
    }
    (mesh, s)
}

fn desloc(a: &Mesh, b: &Mesh, v: usize) -> f32 {
    let (p, q) = (a.positions()[v], b.positions()[v]);
    ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
}

/// ⭐⭐⭐ **GATE — encostar sem mover NÃO deforma.**
///
/// ⚠️ **É a razão de o tecido ter saído dos dois censos do aplicador**, e é
/// também produto correto: o pano responde à VELOCIDADE da mão, então pousar o
/// dedo e não andar não tem por que fazer prega. Os outros 23 verbos carimbam no
/// primeiro dab; este não, e a diferença é a lei, não um buraco.
#[test]
fn encostar_sem_mover_nao_deforma() {
    let antes = plano();
    let (depois, _) = arrastar(1, &pincel());
    let pior = (0..antes.vert_count())
        .map(|v| desloc(&antes, &depois, v))
        .fold(0.0f32, f32::max);
    assert_eq!(pior, 0.0, "um dab parado deformou o pano");
}

/// ⭐⭐⭐ **GATE — arrastar move o MIOLO, e o anel pregado NÃO se mexe.**
///
/// ⛔ O anel é a feature: é ele que faz a transição para o resto da escultura
/// não estourar. Um pregado que escorrega escorrega visivelmente num traço de
/// mil eventos.
#[test]
fn arrastar_move_o_miolo_e_o_anel_fica() {
    let antes = plano();
    let (depois, _) = arrastar(12, &pincel());
    let mut miolo = 0.0f32;
    let mut longe = 0.0f32;
    for v in 0..antes.vert_count() {
        let p = antes.positions()[v];
        let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
        let d = desloc(&antes, &depois, v);
        if r < 0.15 {
            miolo = miolo.max(d);
        } else if r > 0.80 {
            // Bem fora da região de simulação (`0,30 × 2 = 0,60`).
            longe = longe.max(d);
        }
    }
    assert!(miolo > 1e-3, "o pano nao respondeu ao arrasto: {miolo:.3e}");
    assert_eq!(longe, 0.0, "o pano moveu fora da regiao: {longe:.3e}");
}

/// ⭐⭐⭐ **GATE — o pano responde FORA da pegada, e é isso que o separa de um Grab.**
///
/// ⚠️⚠️ **É a propriedade que justifica a feature inteira.** Um Grab move
/// exatamente quem está sob o dedo e mais nada; um tecido é arrastado pela
/// MEMBRANA e pela DOBRA, então a vizinhança anda junto. Sem isto não há prega —
/// há um Grab com outro nome.
#[test]
fn o_pano_responde_fora_da_pegada() {
    let antes = plano();
    let b = pincel();
    let (depois, _) = arrastar(12, &b);
    let mut fora = 0.0f32;
    for v in 0..antes.vert_count() {
        let p = antes.positions()[v];
        let r = (p[0] * p[0] + p[1] * p[1]).sqrt();
        // Fora da pegada (`radius`), dentro da região (`radius × 2`) e antes do
        // anel pregado (`× 0,7`).
        if r > b.radius * 1.05 && r < b.radius * 2.0 * 0.65 {
            fora = fora.max(desloc(&antes, &depois, v));
        }
    }
    assert!(
        fora > 1e-4,
        "so' a pegada se moveu -- isto e' um Grab, nao um tecido: {fora:.3e}"
    );
}

/// ⭐⭐⭐ **GATE — a MÁSCARA protege o que o artista protegeu.**
///
/// ⛔ Um pincel de tecido que ignorasse a máscara destruiria a região que o
/// artista mascarou — e ele a lê pela MESMA porta dos outros verbos
/// (`mask_ops::free_weight`, no `pre` congelado).
#[test]
fn a_mascara_protege() {
    let mut mesh = plano();
    // Mascara TUDO: nenhum vértice pode sentir força.
    let n = mesh.vert_count();
    mesh.masks_mut()[..n].fill(1.0);
    let antes = mesh.clone();
    let b = pincel();
    let mut s = SculptStroke::default();
    s.begin(&mesh);
    for k in 0..12 {
        let c = [0.02 * k as f32, 0.0, 0.0];
        s.dab(&mut mesh, &b, &dab_em(c, b.radius), Symmetry::default());
    }
    let pior = (0..n)
        .map(|v| desloc(&antes, &mesh, v))
        .fold(0.0f32, f32::max);
    assert_eq!(pior, 0.0, "a mascara nao segurou o tecido: {pior:.3e}");
    // Controle: SEM máscara o mesmo gesto move.
    let (livre, _) = arrastar(12, &b);
    let base = plano();
    let move_ = (0..n)
        .map(|v| desloc(&base, &livre, v))
        .fold(0.0f32, f32::max);
    assert!(
        move_ > 1e-3,
        "o controle nao se mexeu: o gate mediria vacuo"
    );
}

/// ⭐⭐⭐ **GATE — todo vértice que o pano moveu foi CAPTURADO.**
///
/// ⛔⛔ É o contrato do undo. O `Ctrl+Z` repõe o `base_pos` dos vértices
/// capturados; um vértice que a simulação move sem ter sido capturado é um
/// vértice que o desfazer **não sabe repor** — e o defeito só apareceria depois,
/// como uma mossa que não sai.
#[test]
fn todo_vertice_movido_foi_capturado() {
    let antes = plano();
    let (depois, s) = arrastar(12, &pincel());
    let mut moveu = 0usize;
    for v in 0..antes.vert_count() {
        if desloc(&antes, &depois, v) > 0.0 {
            moveu += 1;
            assert!(
                s.touched().contains(&u32::try_from(v).unwrap_or(u32::MAX)),
                "o vertice {v} moveu e nao foi capturado -- o undo nao o repoe"
            );
        }
    }
    assert!(moveu > 10, "o gate mediria vacuo: so' {moveu} vertices");
}

/// ⭐⭐⭐ **GATE — um traço novo não herda a região do anterior.**
///
/// ⚠️ A região é medida no pen-down e morre no pen-up. Herdá-la faria o traço
/// seguinte simular onde o artista já não está — e, pior, sobre um repouso que
/// descreve a peça de antes do primeiro traço.
#[test]
fn um_traco_novo_nao_herda_a_regiao() {
    let mut mesh = plano();
    let b = pincel();
    let mut s = SculptStroke::default();
    // Primeiro traço, num canto.
    s.begin(&mesh);
    for k in 0..8 {
        let c = [-0.6 + 0.02 * k as f32, -0.6, 0.0];
        s.dab(&mut mesh, &b, &dab_em(c, b.radius), Symmetry::default());
    }
    let entre = mesh.clone();
    // Segundo traço, no canto OPOSTO.
    s.begin(&mesh);
    for k in 0..8 {
        let c = [0.6 + 0.02 * k as f32, 0.6, 0.0];
        s.dab(&mut mesh, &b, &dab_em(c, b.radius), Symmetry::default());
    }
    // O canto do PRIMEIRO traço não pode ter andado no segundo.
    let mut velho = 0.0f32;
    for v in 0..mesh.vert_count() {
        let p = entre.positions()[v];
        if p[0] < -0.3 && p[1] < -0.3 {
            velho = velho.max(desloc(&entre, &mesh, v));
        }
    }
    assert_eq!(velho, 0.0, "o 2o traco mexeu na regiao do 1o: {velho:.3e}");
}
