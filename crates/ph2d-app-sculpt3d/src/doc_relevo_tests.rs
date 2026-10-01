//! ⭐⭐ **Os gates do RELEVO no documento** (`v4` a altura, `v5` o par
//! altura+corpo — o impasto do Painter na peça, `docs/3D/29`). Irmão (`#[path]`) do [`super`]; correm sem janela, pelo
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
        let a = t.relevo_mut();
        let n = a.len();
        for (i, r) in a.iter_mut().enumerate().take(n / 2).skip(n / 5) {
            *r = [0.01 + i as f32 * 1e-4, 0.25 + i as f32 * 1e-3];
        }
        a[1] = [-0.0, 0.0];
        a[2] = [-0.004, 1.0];
    }
    (stack, Pose::new([0.0, 0.0, 0.0], 1.0), t)
}

fn bits(a: &[[f32; 2]]) -> Vec<[u32; 2]> {
    a.iter().map(|r| [r[0].to_bits(), r[1].to_bits()]).collect()
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
        bits(volta.relevo().expect("o relevo voltou")),
        bits(t.relevo().expect("a fixtura tem relevo")),
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

/// O documento da peça escrito à mão numa versão `version`, com o relevo na
/// forma dada — a régua das duas versões que o têm.
fn forja<R: serde::Serialize>(version: u32, t: &Tinta, relevo: Option<R>) -> Vec<u8> {
    #[derive(serde::Serialize)]
    struct TintaR<R> {
        nivel: u8,
        amostras: super::doc_tinta::AmostrasDoc,
        niveis: Vec<u8>,
        relevo: Option<R>,
    }
    #[derive(serde::Serialize)]
    struct ObjectR<R> {
        stack: StackData,
        pose: PoseData,
        tinta: Option<TintaR<R>>,
    }
    #[derive(serde::Serialize)]
    struct DocR<R> {
        version: u32,
        objects: Vec<ObjectR<R>>,
        active: u32,
    }
    let (stack, pose, _) = peca(false);
    postcard::to_allocvec(&DocR {
        version,
        objects: vec![ObjectR {
            stack: stack.to_data(),
            pose: pose.to_data(),
            tinta: Some(TintaR {
                nivel: t.nivel(),
                amostras: super::doc_tinta::a_menor_forma(t.amostras()),
                niveis: Vec::new(),
                relevo,
            }),
        }],
        active: 0,
    })
    .expect("os bytes")
}

/// ⛔⛔ **GATE — Um relevo que não soma as amostras da malha RECUSA o load**,
/// pela mesma razão da cor: *espessura no sítio errado*.
#[test]
fn um_relevo_que_nao_descreve_o_plano_recusa_o_load() {
    let (_, _, t) = peca(false);
    let n = t.amostras().len();
    let com = |n: usize| {
        forja(
            SCULPT_DOC_VERSION,
            &t,
            Some(super::doc_tinta::a_menor_forma(&vec![[0.01f32, 1.0]; n])),
        )
    };
    assert!(
        decode(&com(n)).is_ok(),
        "o CONTROLO: com a contagem certa abre"
    );
    assert!(
        matches!(decode(&com(n + 1)), Err(SculptDocError::Tinta { .. })),
        "um relevo com uma amostra a mais tem de ser recusado"
    );
}

/// ⭐⭐⭐ **GATE — UM DOCUMENTO v4 ABRE, com a ALTURA ao bit e o CORPO
/// derivado** (`docs/3D/29` §6): antes do corpo uma altura não nula só nascia
/// debaixo de tinta, logo o v4 lê-se com corpo `1` onde há altura e `0` onde
/// não há. ⚠️ O CONTROLO é o `0.0` e o `-0.0` — os dois não têm espessura e
/// saem sem corpo, e o `-0.0` volta com o sinal dele.
#[test]
fn um_documento_v4_abre_com_o_corpo_derivado_da_altura() {
    let (_, _, t) = peca(false);
    let n = t.amostras().len();
    let mut alturas = vec![0.0f32; n];
    alturas[1] = -0.0;
    alturas[2] = -0.004;
    alturas[3] = 0.0125;
    let bytes = forja(4, &t, Some(super::doc_tinta::a_menor_forma(&alturas)));
    let (lidas, _) = decode(&bytes).expect("um v4 tem de abrir");
    let r = lidas[0]
        .tinta
        .as_ref()
        .expect("o plano")
        .relevo()
        .expect("o v4 tinha relevo")
        .to_vec();
    for (i, (&h, got)) in alturas.iter().zip(&r).enumerate() {
        assert_eq!(got[0].to_bits(), h.to_bits(), "amostra {i}: a altura");
        let corpo = if h != 0.0 { 1.0 } else { 0.0 };
        assert_eq!(got[1], corpo, "amostra {i}: o corpo derivado");
    }
}
