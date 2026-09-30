//! ⭐⭐ **Os gates do RELEVO no documento** (`v4`, o impasto do Painter na
//! peça — `docs/3D/29`). Irmão (`#[path]`) do [`super`]; correm sem janela, pelo
//! par [`encode`]/[`decode`] como os do plano.

use super::*;
use ph2d_mesh::{Multires, shapes};
use ph2d_mesh_colors::Tinta;

/// Uma peça com plano de tinta fina e, se `relevo`, uma lomba nele — com um
/// `-0.0` e um valor que nenhuma corrida junta a outro, para a régua dos BITS.
fn peca(relevo: bool) -> (Multires, Pose, Tinta) {
    let stack = Multires::new(shapes::octahedron(1.0));
    let m = stack.mesh();
    let faces = || m.faces().iter().map(ph2d_mesh::Face::verts);
    let mut t = Tinta::nova(m.vert_count(), faces(), 2);
    if relevo {
        let a = t.alturas_mut();
        let n = a.len();
        for (i, h) in a.iter_mut().enumerate().take(n / 2).skip(n / 5) {
            *h = 0.01 + i as f32 * 1e-4;
        }
        a[1] = -0.0;
        a[2] = -0.004;
    }
    (stack, Pose::new([0.0, 0.0, 0.0], 1.0), t)
}

fn bits(a: &[f32]) -> Vec<u32> {
    a.iter().map(|h| h.to_bits()).collect()
}

/// ⭐⭐⭐ **GATE — O RELEVO ATRAVESSA O FICHEIRO, AO BIT** (o `-0.0` e a altura
/// negativa incluídos), e um plano SEM relevo volta sem ele — não com um vector
/// de zeros, que a `256x` seriam dezenas de MB por nada.
#[test]
fn o_relevo_atravessa_o_ficheiro_ao_bit() {
    let (stack, pose, t) = peca(true);
    let bytes = encode(&[(stack.to_data(), pose.to_data(), Some(&t))], 0);
    let (lidas, _) = decode(&bytes).expect("ida e volta");
    let volta = lidas[0].tinta.as_ref().expect("o plano");
    assert_eq!(
        bits(volta.alturas().expect("o relevo voltou")),
        bits(t.alturas().expect("a fixtura tem relevo")),
        "o relevo não voltou AO BIT"
    );

    let (stack, pose, t) = peca(false);
    let bytes = encode(&[(stack.to_data(), pose.to_data(), Some(&t))], 0);
    let (lidas, _) = decode(&bytes).expect("ida e volta");
    assert!(
        !lidas[0].tinta.as_ref().expect("o plano").tem_relevo(),
        "um plano sem relevo não pode voltar com um"
    );
}

/// ⭐⭐ **GATE — Um plano sem relevo custa UM byte a mais do que custava.** O
/// CONTROLO é o mesmo plano com relevo, que tem de custar mais.
#[test]
fn um_plano_sem_relevo_custa_um_byte() {
    let (stack, pose, t) = peca(false);
    let com_none = encode(&[(stack.to_data(), pose.to_data(), Some(&t))], 0).len();
    let (_, _, com) = peca(true);
    let com_relevo = encode(&[(stack.to_data(), pose.to_data(), Some(&com))], 0).len();
    assert!(
        com_relevo > com_none + 8,
        "o CONTROLO: o relevo ocupa bytes"
    );
    // O v3 escrevia o mesmo documento sem o campo: um byte a menos.
    let v3 = v3_bytes(&stack, &pose, &t);
    assert_eq!(com_none, v3.len() + 1, "o `None` é um byte, e mais nada");
}

#[derive(serde::Serialize)]
struct TintaV3 {
    nivel: u8,
    amostras: super::doc_tinta::AmostrasDoc,
    niveis: Vec<u8>,
}
#[derive(serde::Serialize)]
struct ObjectV3 {
    stack: StackData,
    pose: PoseData,
    tinta: Option<TintaV3>,
}
#[derive(serde::Serialize)]
struct DocV3 {
    version: u32,
    objects: Vec<ObjectV3>,
    active: u32,
}

/// Os bytes de um documento **v3**, escritos pela forma congelada (e não por
/// um ficheiro guardado: ver o gate do v1 no irmão).
fn v3_bytes(stack: &Multires, pose: &Pose, t: &Tinta) -> Vec<u8> {
    postcard::to_allocvec(&DocV3 {
        version: 3,
        objects: vec![ObjectV3 {
            stack: stack.to_data(),
            pose: pose.to_data(),
            tinta: Some(TintaV3 {
                nivel: t.nivel(),
                amostras: super::doc_tinta::a_menor_forma(t.amostras()),
                niveis: Vec::new(),
            }),
        }],
        active: 0,
    })
    .expect("os bytes v3")
}

/// ⭐⭐⭐ **GATE — UM DOCUMENTO v3 ABRE, com o plano AO BIT e sem relevo.**
/// *Subir a versão de um formato sem degrau é apagar o trabalho de quem já o
/// usou* — a lei das duas migrações de antes.
#[test]
fn um_documento_v3_abre_sem_relevo_e_com_a_tinta_ao_bit() {
    let (stack, pose, mut t) = peca(false);
    for (i, c) in t.amostras_mut().iter_mut().enumerate() {
        *c = [i as f32 * 1e-3, 0.5, 0.25];
    }
    let (lidas, _) = decode(&v3_bytes(&stack, &pose, &t)).expect("um v3 tem de abrir");
    let volta = lidas[0].tinta.as_ref().expect("o plano do v3");
    assert!(!volta.tem_relevo(), "um v3 é anterior ao relevo");
    assert_eq!(volta.amostras(), t.amostras(), "a tinta do v3 voltou");
}

/// ⛔⛔ **GATE — Um relevo que não soma as amostras da malha RECUSA o load**,
/// pela mesma razão da cor: *espessura no sítio errado*.
#[test]
fn um_relevo_que_nao_descreve_o_plano_recusa_o_load() {
    #[derive(serde::Serialize)]
    struct TintaV4 {
        nivel: u8,
        amostras: super::doc_tinta::AmostrasDoc,
        niveis: Vec<u8>,
        alturas: Option<super::doc_tinta::AlturasDoc>,
    }
    #[derive(serde::Serialize)]
    struct ObjectV4 {
        stack: StackData,
        pose: PoseData,
        tinta: Option<TintaV4>,
    }
    #[derive(serde::Serialize)]
    struct DocV4 {
        version: u32,
        objects: Vec<ObjectV4>,
        active: u32,
    }
    let (stack, pose, t) = peca(false);
    let forja = |n: usize| {
        postcard::to_allocvec(&DocV4 {
            version: SCULPT_DOC_VERSION,
            objects: vec![ObjectV4 {
                stack: stack.to_data(),
                pose: pose.to_data(),
                tinta: Some(TintaV4 {
                    nivel: t.nivel(),
                    amostras: super::doc_tinta::a_menor_forma(t.amostras()),
                    niveis: Vec::new(),
                    alturas: Some(super::doc_tinta::a_menor_forma(&vec![0.01f32; n])),
                }),
            }],
            active: 0,
        })
        .expect("os bytes")
    };
    let n = t.amostras().len();
    assert!(
        decode(&forja(n)).is_ok(),
        "o CONTROLO: com a contagem certa abre"
    );
    assert!(
        matches!(decode(&forja(n + 1)), Err(SculptDocError::Tinta { .. })),
        "um relevo com uma amostra a mais tem de ser recusado"
    );
}
