//! Gates do [`super::BoneRest`] — a ida, a volta, e os **seis** números.

use super::*;

/// ⚠️ **Os seis números escritos CAMPO A CAMPO, e não por um literal de `Transform`**: o tipo do
/// vector vive na `ph2d-core`, que esta crate não declara — e acrescentar uma dependência só para
/// escrever um literal de teste é o que o `cargo-machete` cobra na integração.
fn pose(tx: f32, ty: f32, rot: f32, sx: f32, sy: f32, kx: f32, ky: f32) -> Transform {
    let mut t = Transform::IDENTITY;
    t.translation.x = tx;
    t.translation.y = ty;
    t.rotation = rot;
    t.scale.x = sx;
    t.scale.y = sy;
    t.skew_x = kx;
    t.skew_y = ky;
    t
}

/// Uma pose em que os **seis** números são diferentes uns dos outros e nenhum é o neutro.
///
/// ⚠️ **É a fixtura que faz os gates valerem:** com `scale = (1,1)` ou `skew = 0` esquecer aquele
/// campo na volta passaria despercebido — *um corpus no ponto neutro de um campo não testa esse
/// campo*, que é a lei que esta casa já pagou no `Accumulate` do apagador.
fn pose_com_os_seis_distintos() -> Transform {
    pose(3.25, -7.5, 0.875, 2.5, 0.375, 0.125, -0.0625)
}

/// A ida e a volta são a **identidade ao bit** — e é isso que faz repor a pose dez vezes dar a
/// mesma pose dez vezes.
#[test]
fn guardar_e_repor_devolvem_a_pose_ao_bit() {
    let original = pose_com_os_seis_distintos();
    let repouso = BoneRest::de(&original);
    // Uma pose bem longe da guardada: se a volta esquecer um campo, é este valor que fica.
    let mut t = pose(-100.0, 42.0, -2.5, 9.0, 9.0, 0.5, 0.5);
    repouso.aplica(&mut t);
    assert_eq!(
        t, original,
        "repor tem de devolver a pose guardada ao bit, e nao uma aproximacao dela"
    );
}

/// ⭐ **Cada um dos seis, sozinho.** O gate acima passa com cinco campos certos e um esquecido se o
/// `assert_eq` de `Transform` alguma vez deixar de comparar tudo; este nomeia-os um a um, e a
/// mensagem diz **qual** ficou para trás.
#[test]
fn a_volta_repoe_cada_um_dos_seis_numeros() {
    let original = pose_com_os_seis_distintos();
    let repouso = BoneRest::de(&original);
    let mut t = Transform::IDENTITY;
    repouso.aplica(&mut t);
    let campos: [(&str, f32, f32); 6] = [
        ("translation.x", t.translation.x, original.translation.x),
        ("translation.y", t.translation.y, original.translation.y),
        ("rotation", t.rotation, original.rotation),
        ("scale.x", t.scale.x, original.scale.x),
        ("scale.y", t.scale.y, original.scale.y),
        ("skew_x", t.skew_x, original.skew_x),
    ];
    for (nome, teve, queria) in campos {
        assert!(
            (teve - queria).abs() < f32::EPSILON,
            "`{nome}` nao voltou: {teve} contra {queria}"
        );
    }
    assert!(
        (t.skew_y - original.skew_y).abs() < f32::EPSILON,
        "`skew_y` nao voltou: {} contra {}",
        t.skew_y,
        original.skew_y
    );
}

/// ⚠️ **A DIRECÇÃO é o que a identidade apagava.** Este gate é a forma mínima do defeito do dono:
/// um osso virado não pode voltar a `rotation = 0`, porque a rotação dele **é** para onde ele
/// aponta.
#[test]
fn o_repouso_de_um_osso_virado_nao_e_a_identidade() {
    let virado = pose(5.0, 2.0, 1.25, 1.0, 1.0, 0.0, 0.0);
    let repouso = BoneRest::de(&virado);
    let identidade = BoneRest::de(&Transform::IDENTITY);
    assert_ne!(
        repouso, identidade,
        "um osso que aponta para algum lado nao repousa na identidade"
    );
}
