//! ⭐⭐⭐ **AS ALÇAS, NA BARRA DA CENA DO DONO** — o report de 2026-09-19, medido no produto.
//!
//! *«Ainda temos muitas irregularidades na deformação de vetores. Certamente um mau tratamento das
//! alças dos handles.»*
//!
//! A causa e o mecanismo vivem em [`ph2d_vec_skin::curva`]; o que este ficheiro guarda é o que o
//! artista vê: **dobrar a barra não pode cravar uma quina num nó que ele desenhou liso**.
//!
//! ⚠️ O outro report daquele dia — *«num vector linkado aos ossos não consigo mudar a espessura do
//! stroke»* — é outro mecanismo e mora com o que exercita
//! (o `skin_live_traco_tests.rs`, ao lado do `recook` que o escreve).

use crate::barra_da_cena_tests_support::barra_da_cena;
use ph2d_ecs::{SimWorld, Transform};
use ph2d_vec_scene::{VecPath, VecPathId, VecScene};

/// A barra com as duas juntas dobradas `graus` para o mesmo lado — o cotovelo do report.
fn dobrada(graus: f64) -> (SimWorld, VecScene, VecPathId) {
    let (mut sim, scene, _m, id, ossos) = barra_da_cena();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "o angulo do gate, um punhado"
    )]
    let r = (graus.to_radians() * 0.5) as f32;
    for &o in &ossos[1..] {
        sim.world_mut()
            .get_mut::<Transform>(o)
            .expect("pose")
            .rotation = r;
    }
    (sim, scene, id)
}

fn le(scene: &VecScene, id: VecPathId) -> VecPath {
    scene
        .paths()
        .iter()
        .find(|p| p.id == id)
        .expect("a barra")
        .clone()
}

/// O ângulo com que a tangente VIRA ao atravessar cada nó, em graus — `None` onde uma das alças é
/// degenerada e não tem direcção.
///
/// ⚠️ **`None` e não um salto:** saltar um nó desalinharia o emparelhamento com o outro estado do
/// mesmo caminho, e a régua passaria a comparar nós diferentes.
fn viragens(p: &VecPath) -> Vec<Option<f64>> {
    let c = p.cooked();
    let mut out = Vec::new();
    for k in 0..c.contour_count() {
        let Some((v, fechado)) = c.contour(k) else {
            continue;
        };
        if !fechado {
            continue;
        }
        for no in v {
            let ent = [
                no.anchor[0] - no.in_handle[0],
                no.anchor[1] - no.in_handle[1],
            ];
            let sai = [
                no.out_handle[0] - no.anchor[0],
                no.out_handle[1] - no.anchor[1],
            ];
            if ent[0].hypot(ent[1]) <= 0.0 || sai[0].hypot(sai[1]) <= 0.0 {
                out.push(None);
                continue;
            }
            let mut d = sai[1].atan2(sai[0]) - ent[1].atan2(ent[0]);
            while d > std::f64::consts::PI {
                d -= std::f64::consts::TAU;
            }
            while d < -std::f64::consts::PI {
                d += std::f64::consts::TAU;
            }
            out.push(Some(d.abs().to_degrees()));
        }
    }
    out
}

/// O maior afastamento entre as viragens de dois estados do mesmo caminho, em graus.
fn mudanca(referencia: &VecPath, agora: &VecPath) -> f64 {
    let (r, a) = (viragens(referencia), viragens(agora));
    assert_eq!(r.len(), a.len(), "os dois estados tem de ter os mesmos nos");
    assert!(
        r.iter().filter(|x| x.is_some()).count() >= 8,
        "menos de oito nos com tangente dos dois lados — a barra deixou de conter o fenomeno"
    );
    r.into_iter()
        .zip(a)
        .filter_map(|(x, y)| Some((x?, y?)))
        .fold(0.0_f64, |m, (x, y)| m.max((x - y).abs()))
}

/// ⭐⭐⭐ **DOBRAR NÃO CRAVA UMA QUINA EM NÓ NENHUM** — o report do dono, na barra dele.
///
/// ⚠️ **A REFERÊNCIA é a lei INGÉNUA corrida na mesma árvore** (`recook_com(.., false)`): ela
/// aplica **um** afim às três metades de cada vértice, e um afim preserva colinearidade — logo ela
/// é, por construção, o desenho com a continuidade que a fonte tinha. *O que se mede é quanto a
/// correcção das alças se afasta dela.*
///
/// | dobra | mudança da tangente, antes de 2026-09-19 | hoje |
/// |---|---|---|
/// | `30°` | `5,86°` | **`0,000°`** |
/// | `60°` | `13,00°` | **`0,000°`** |
/// | `90°` | `20,92°` | **`0,000°`** |
/// | `120°` | `28,62°` | **`0,000°`** |
///
/// ⚠️⚠️ **As duas primeiras asserções são CONTROLOS e vêm primeiro.** Sem a do arrasto, um gate
/// verde diria apenas que a barra não dobrou; sem a da correcção, uma lei que devolvesse `(0, 0)`
/// passaria — ela seria trivialmente colinear, e o desvio ao desenho verdadeiro não é medido aqui.
///
/// (Mutações: apagar a chamada a `reconcilia` ⇒ RED na 3.ª · conciliar só uma das duas metades ⇒
/// RED na 3.ª · devolver `(0,0)` da `correccao_das_alcas` ⇒ RED na 2.ª.)
#[test]
fn dobrar_a_barra_nao_crava_uma_quina_em_no_nenhum() {
    for graus in [30.0, 60.0, 90.0, 120.0] {
        let (sim, mut a, id) = dobrada(graus);
        crate::skin_live::recook_com(&sim, &mut a, false);
        let ingenua = le(&a, id);

        let (sim2, mut b, id2) = dobrada(graus);
        crate::skin_live::recook_com(&sim2, &mut b, true);
        let curva = le(&b, id2);

        let (_s3, repouso, id3) = dobrada(0.0);
        let parada = le(&repouso, id3);

        let arrastou = crate::test_support::pior_desvio_do_desenho(&parada, &ingenua);
        assert!(
            arrastou > 0.5,
            "a {graus}° a barra mexeu-se so' {arrastou} — a fixtura deixou de dobrar, e as \
             asserções abaixo passam a ser triviais"
        );
        let corrigiu = crate::test_support::pior_desvio(&ingenua, &curva);
        assert!(
            corrigiu > 0.01,
            "a {graus}° a correccao das alcas mexeu so' {corrigiu} — ela parou, e um desenho sem \
             correccao nenhuma e' trivialmente colinear"
        );
        let mudou = mudanca(&ingenua, &curva);
        eprintln!(
            "[alcas] {graus:5.0}° arrasto={arrastou:.4} correccao={corrigiu:.4} quina={mudou:.3e}°"
        );
        assert!(
            mudou.is_finite() && mudou < 1e-9,
            "a {graus}° a correccao virou a tangente {mudou}° num no' — as duas alcas de um no' \
             tem de rodar JUNTAS, senao o no' que o artista desenhou liso vira uma QUINA"
        );
    }
}
