//! Os gates de [`super::snap_tips`].
//!
//! ⚠️ **A fixtura é a do irmão** ([`crate::tip_rows::tests::cone`], o fuso de dois espinhos):
//! *o remate e a régua que o mede têm de falar da mesma peça*, senão o gate mede uma
//! geometria que a tabela nunca viu.

use super::{TIP_SNAP_TRAVEL, snap_tips};
use crate::tip_rows::tests::{cone, cone_com};
use crate::{TIP_GAP_MAX, tip_rows};

/// ⭐ **A ponta que o PRODUTO entrega** — leque de `6` a `43°`, que é onde a grade sustenta um
/// bico. ⛔ A agulha (`cone`) tem o leque a `15°` e serve à pergunta OPOSTA — ver
/// [`cone_com`].
fn ponta() -> ph2d_mesh::Mesh {
    cone_com(false, 6, 0.75)
}

/// Puxa o bico (vértice `0`) para DENTRO da peça, `f` células — é o embotamento que o
/// acabamento produz, encenado sem correr o acabamento.
fn embotado(f: f32, unit: f32) -> ph2d_mesh::Mesh {
    let mut m = ponta();
    let p = m.positions()[0];
    m.positions_mut()[0] = [p[0], p[1], p[2] - f * unit];
    m.rebuild();
    m
}

/// ⭐⭐⭐ **GATE — o bico encosta, e a ponta que já encostava não se mexe.**
///
/// ⛔ **É o gate do report de 2026-09-04.** O acabamento embota a ponta em meia célula porque
/// a projecção ao ponto mais próximo de uma superfície **nunca escolhe o ápice**; o remate
/// devolve-a, e a régua tem de o ver.
#[test]
fn o_bico_encosta_no_apice_e_a_ponta_ja_certa_fica_ao_bit() {
    let escultura = ponta();
    let unit = 0.2;
    let mut m = embotado(0.4, unit);
    let antes = tip_rows(&escultura, &m, unit);
    let gap_antes = antes
        .iter()
        .find(|r| r.apex == 0)
        .expect("o bico esta' na tabela")
        .gap;
    assert!(
        gap_antes > 0.3,
        "a fixtura tem de estar embotada: {gap_antes}"
    );

    let n = snap_tips(&mut m, &escultura, unit);
    assert_eq!(n, 1, "⛔ um bico encostou, e um so'");
    let depois = tip_rows(&escultura, &m, unit);
    let gap_depois = depois
        .iter()
        .find(|r| r.apex == 0)
        .expect("o bico esta' na tabela")
        .gap;
    assert!(
        gap_depois < 1.0e-3,
        "o bico tem de ficar EM CIMA do apice: {gap_depois}"
    );
    // ⭐ E a outra ponta — que já estava certa — não foi tocada.
    for r in depois.iter().filter(|r| r.apex != 0) {
        assert!(r.gap < 0.05, "a ponta {} mexeu-se: {r:?}", r.apex);
    }

    // ⚠️ **A malha JÁ certa fica ao bit** — `viagem == 0` não é um reparo.
    let mut igual = ponta();
    let copia = igual.positions().to_vec();
    assert_eq!(snap_tips(&mut igual, &escultura, unit), 0);
    assert_eq!(igual.positions(), copia.as_slice());
}

/// ⭐⭐⭐ **GATE — a ponta AMPUTADA não se remata.**
///
/// ⛔ Acima de [`TIP_GAP_MAX`] o que falta é **célula**, não fase: mudar um vértice ali
/// esconderia do selector do botão um defeito que ele tem de ver — e a malha continuaria
/// errada, agora com um espeto de uma célula a sair de um bico chato.
#[test]
fn a_ponta_amputada_fica_acusada_e_a_malha_ao_bit() {
    let escultura = ponta();
    let unit = 0.2;
    // ⚠️ Mais fundo que a barra, e ainda dentro da viagem — é a célula que separa as duas
    // cercas: se a barra do `gap` não existisse, esta ponta seria «reparada».
    let mut m = embotado(0.9, unit);
    let copia = m.positions().to_vec();
    let antes = tip_rows(&escultura, &m, unit);
    let gap = antes.iter().find(|r| r.apex == 0).expect("o bico").gap;
    assert!(
        gap > TIP_GAP_MAX && gap < TIP_SNAP_TRAVEL,
        "a fixtura tem de estar entre as duas cercas: {gap}"
    );
    assert_eq!(snap_tips(&mut m, &escultura, unit), 0, "⛔ nao se remata");
    assert_eq!(m.positions(), copia.as_slice());
}

/// ⭐⭐⭐ **GATE — o remate nunca piora a malha, e é o CENSO GLOBAL que o diz.**
///
/// ⚠️ Ele varre a fixtura com o ápice deslocado de LADO — o caso em que encostar torce as
/// faces do bico —, e exige a promessa que a função faz: *faces péssimas e faces do avesso
/// não sobem*. ⛔ Sem a guarda, um reparo que endireita uma ponta pode deixar um leque
/// dobrado atrás dele, que é o defeito que o dono fotografa.
#[test]
fn o_remate_nunca_sobe_as_faces_pessimas_nem_as_do_avesso() {
    let unit = 0.2;
    for k in 0..8 {
        #[expect(clippy::cast_precision_loss, reason = "k < 8")]
        let t = k as f32 / 8.0;
        // A escultura tem o bico inclinado; a saída é o cone recto.
        let mut escultura = ponta();
        let p = escultura.positions()[0];
        escultura.positions_mut()[0] = [
            (t * TIP_SNAP_TRAVEL).mul_add(unit, p[0]),
            p[1],
            p[2] - 0.2 * t * unit,
        ];
        escultura.rebuild();
        let mut m = ponta();
        let base = crate::quad_shape(&m);
        let base_avesso =
            crate::local_shape(&m).0.bowties + crate::quality::folded_faces_by_neighbours(&m).len();
        snap_tips(&mut m, &escultura, unit);
        let agora = crate::quad_shape(&m);
        let avesso =
            crate::local_shape(&m).0.bowties + crate::quality::folded_faces_by_neighbours(&m).len();
        assert!(
            agora.skew_over_60 <= base.skew_over_60,
            "k={k}: faces pessimas {} -> {}",
            base.skew_over_60,
            agora.skew_over_60
        );
        assert!(
            avesso <= base_avesso,
            "k={k}: faces do avesso {base_avesso} -> {avesso}"
        );
    }
}

/// ⭐⭐ **GATE — sem ápice, sem unidade ou sem malha, o remate é um no-op ao bit.**
#[test]
fn sem_o_que_rematar_a_malha_fica_ao_bit() {
    let escultura = ponta();
    let mut m = ponta();
    let copia = m.positions().to_vec();
    assert_eq!(
        snap_tips(&mut m, &escultura, 0.0),
        0,
        "unidade nao positiva"
    );
    assert_eq!(snap_tips(&mut m, &ph2d_mesh::Mesh::default(), 0.2), 0);
    assert_eq!(m.positions(), copia.as_slice());
}

/// ⭐⭐⭐ **GATE — o bico que é uma LASCA não se endireita, e a guarda é quem o diz.**
///
/// ⛔⛔ **É o controlo do gate de cima, e ele nasceu de uma medição:** na agulha (leque de `8`
/// a `15°`) devolver o ápice ao sítio leva as faces péssimas de `8` para **`16`** — *um bico
/// sem células para segurar um ponto não fica afiado, fica com um espeto*. Na malha que o
/// produto entrega a mesma operação deixa o censo **intacto** (`>60` `8 → 8`, avesso `2 → 2`
/// nas cinco pontas da peça do dono), e é por isso que ela é aceite lá e recusada aqui.
///
/// ⚠️ *A guarda não é uma precaução: é a diferença entre as duas fixturas.*
#[test]
fn o_bico_que_e_uma_lasca_nao_se_endireita() {
    let escultura = cone(false);
    let unit = 0.2;
    let mut m = cone(false);
    let p = m.positions()[0];
    m.positions_mut()[0] = [p[0], p[1], p[2] - 0.4 * unit];
    m.rebuild();
    let copia = m.positions().to_vec();
    assert_eq!(
        snap_tips(&mut m, &escultura, unit),
        0,
        "⛔ a lasca nao se endireita"
    );
    assert_eq!(m.positions(), copia.as_slice(), "e a malha fica ao bit");
}
