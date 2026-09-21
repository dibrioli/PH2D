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

/// ⭐⭐⭐ **A QUINA QUE O AJUSTE CRAVOU, E O TECTO QUE ELE PRÓPRIO PERMITE — nó a nó.**
///
/// Devolve `(pior quina, o tecto DESSE nó)`, em graus. O tecto é **derivado e não escolhido**: uma
/// alça de comprimento `L` cuja ponta se move `δ` vira, no máximo, `asin(min(δ/L, 1))`.
///
/// ⛔⛔ **Ele é por NÓ, e a 1.ª redacção usava a menor alça do caminho INTEIRO contra a maior
/// correcção do caminho inteiro** — a `120°` isso dava `asin(1) = 90°`, e *uma barra que satura
/// não é uma barra*.
fn quina_e_tecto(referencia: &VecPath, agora: &VecPath) -> (f64, f64) {
    let (r, a) = (viragens(referencia), viragens(agora));
    assert_eq!(r.len(), a.len(), "os dois estados tem de ter os mesmos nos");
    assert!(
        r.iter().filter(|x| x.is_some()).count() >= 8,
        "menos de oito nos com tangente dos dois lados — a barra deixou de conter o fenomeno"
    );
    let (cr, ca) = (referencia.cooked(), agora.cooked());
    let (Some((vr, _)), Some((va, _))) = (cr.contour(0), ca.contour(0)) else {
        return (0.0, 0.0);
    };
    let mut pior = (0.0_f64, 0.0_f64);
    for (k, (x, y)) in r.iter().zip(&a).enumerate() {
        let (Some(x), Some(y)) = (x, y) else { continue };
        let q = (x - y).abs();
        if q <= pior.0 {
            continue;
        }
        // O braço deste nó, e quanto a ponta dele andou.
        let (mut l, mut d) = (f64::INFINITY, 0.0_f64);
        for lado in 0..2 {
            let (hr, ha) = if lado == 0 {
                (vr[k].in_handle, va[k].in_handle)
            } else {
                (vr[k].out_handle, va[k].out_handle)
            };
            let br = (hr[0] - vr[k].anchor[0]).hypot(hr[1] - vr[k].anchor[1]);
            if br > 1e-9 {
                l = l.min(br);
                d = d.max((ha[0] - hr[0]).hypot(ha[1] - hr[1]));
            }
        }
        if l.is_finite() {
            pior = (q, (d / l).min(1.0).asin().to_degrees());
        }
    }
    pior
}

/// ⭐⭐⭐ **A TROCA DO AJUSTE, COM OS DOIS LADOS AFIRMADOS** — ele compra FIDELIDADE e paga
/// TANGENTE, e as duas metades têm de continuar verdadeiras.
///
/// # ⛔⛔⛔ Este gate afirmava o CONTRÁRIO até 2026-09-20, e a premissa dele morreu
///
/// Ele chamava-se `dobrar_a_barra_nao_crava_uma_quina_em_no_nenhum` e exigia `quina < 1e-9` nas
/// quatro dobras, com a tabela `5,86° / 13,00° / 20,92° / 28,62°` a dizer o que a **conciliação**
/// das alças tinha curado. O report seguinte do dono — *«não fica bom. Muito curvado»*, com foto —
/// mediu-se contra essa conciliação: ela custava **`9,3×`** de serpentina e **`1,6×`** de
/// fidelidade, e o passe foi apagado (a tabela inteira está em
/// [`ph2d_vec_skin::curva::aplica_pela_curva`]).
///
/// ⭐⭐ **A quina que fica NÃO é um defeito do ajuste — é o preço mínimo de representar uma curva
/// por cúbicas INDEPENDENTES, e o CHÃO do modelo paga-o igual.** A prova é o excesso de curvatura
/// medido contra o padrão-ouro: o ajuste lê `3,16°` e o chão `3,18°`
/// ([`super::skinned_mesh::rive_tests`]).
///
/// # As quatro metades, e nenhuma sozinha é honesta
///
/// 1. **A barra DOBROU** — sem isto um gate verde diria apenas que a fixtura está parada.
/// 2. **O ajuste MEXEU** — sem isto uma lei que devolvesse `(0, 0)` passaria nas outras duas.
/// 3. **Ele compra FIDELIDADE**: o desenho fica muito mais perto da deformação verdadeira do que
///    a lei ingénua (que é, à letra, a lei do Rive — um afim por vértice nas três metades dele).
/// 4. ⛔ **E paga TANGENTE, afirmado DE PROPÓSITO.** No dia em que alguém voltar a conciliar, esta
///    metade reprova e a morte da premissa fica à vista no diff — *que é exactamente o que esta
///    metade acabou de fazer comigo*.
///
/// ⚠️ **A barra da metade 4 é DERIVADA e não escolhida** — ver [`quina_e_tecto`].
#[test]
fn o_ajuste_compra_fidelidade_e_paga_tangente() {
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

        // (3) A FIDELIDADE, contra o PADRÃO-OURO — a lei da mídia imagem, ponto a ponto.
        let (e_ing, e_aj) = (erro_ao_ouro(&sim, &ingenua), erro_ao_ouro(&sim, &curva));
        // (4) A QUINA, contra o tecto DERIVADO do que o ajuste mexeu.
        let (mudou, tecto) = quina_e_tecto(&ingenua, &curva);
        eprintln!(
            "[alcas] {graus:5.0}° arrasto={arrastou:.4} correccao={corrigiu:.4} \
             erro ingenua={e_ing:.4} -> ajuste={e_aj:.4} · quina={mudou:.3}° (tecto {tecto:.3}°)"
        );
        assert!(
            e_aj * 4.0 < e_ing,
            "a {graus}° o ajuste ({e_aj}) devia ficar bem abaixo da lei ingenua ({e_ing}) — ele \
             deixou de comprar a fidelidade que e' a razao de ele existir"
        );
        assert!(
            mudou > 1e-6,
            "a {graus}° a tangente ficou colinear ao bit ({mudou}°) — alguem voltou a CONCILIAR \
             as alcas, e isso custa 9,3x de serpentina (ver `aplica_pela_curva`)"
        );
        assert!(
            mudou <= tecto,
            "a {graus}° a quina ({mudou}°) passou do tecto que o proprio ajuste permite \
             ({tecto}°) — o desvio de uma alca nao pode virar a tangente mais do que o \
             comprimento dela deixa"
        );
    }
}

/// O maior afastamento do desenho ao PADRÃO-OURO — a lei da mídia IMAGEM ponto a ponto, que é o
/// que o ajuste persegue e o que o dono compara.
///
/// ⚠️ **Ela chama a porta única do padrão-ouro** ([`crate::skinned_mesh::ouro_reguas_tests::b_ouro_pt`]):
/// reimplementá-la aqui foi a 1.ª redacção deste gate, e ela mediu o **mesmo número dos dois
/// lados** (`0,0370`) — eu tinha fixado o peso no do nó, que é a lei INGÉNUA, logo a régua era o
/// próprio lado que ela devia contradizer.
fn erro_ao_ouro(sim: &SimWorld, p: &VecPath) -> f64 {
    use crate::skinned_mesh::ouro_reguas_tests::b_ouro_pt;
    let Some((e, skin)) = sim
        .world()
        .iter_entities()
        .find_map(|er| Some((er.id(), er.get::<ph2d_skeleton_ecs::SkinBind>()?.clone())))
    else {
        return 0.0;
    };
    let index = crate::skin_live::bone_index(sim);
    let (Some(pele), Some(g)) = (
        crate::skin_live::resolve(sim, &skin, e, &index),
        crate::skinned_mesh::le(&skin.source),
    ) else {
        return 0.0;
    };
    let Some(campo) = g.campo.as_ref() else {
        return 0.0;
    };
    let correcoes = skin.correcoes_resolvidas();
    let (cz, co) = (p.cooked(), g.path.cooked());
    let (Some((va, _)), Some((vf, _))) = (cz.contour(0), co.contour(0)) else {
        return 0.0;
    };
    if va.len() != vf.len() {
        return 0.0;
    }
    let (n, mut pior) = (vf.len(), 0.0_f64);
    for k in 0..n {
        for i in 0..=32 {
            let t = f64::from(i) / 32.0;
            let verdade = b_ouro_pt(
                &pele,
                campo,
                &correcoes,
                cubica_em(&vf[k], &vf[(k + 1) % n], t),
            )
            .0;
            let nosso = cubica_em(&va[k], &va[(k + 1) % n], t);
            pior = pior.max((verdade[0] - nosso[0]).hypot(verdade[1] - nosso[1]));
        }
    }
    pior
}

fn cubica_em(a: &ph2d_vec_scene::VecVertex, b: &ph2d_vec_scene::VecVertex, t: f64) -> [f64; 2] {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    [
        w3.mul_add(
            b.anchor[0],
            w2.mul_add(
                b.in_handle[0],
                w1.mul_add(a.out_handle[0], w0 * a.anchor[0]),
            ),
        ),
        w3.mul_add(
            b.anchor[1],
            w2.mul_add(
                b.in_handle[1],
                w1.mul_add(a.out_handle[1], w0 * a.anchor[1]),
            ),
        ),
    ]
}
