//! Os gates do **GIZMO DA CURVATURA** — irmão do [`super`] pelo tecto de LOC do HR-18, com o corte
//! por RESPONSABILIDADE: ali mede-se o que o dedo apanha nas paredes do LIMITE, aqui nas alças que
//! **arqueiam** o corpo.
//!
//! # O defeito que esta wave cura (limite declarado da F8, 2026-09-15)
//!
//! > *«as alças não têm gesto de canvas (hoje só painel)»* — fila do módulo.
//!
//! Um osso arqueia-se por uma FORMA, e uma forma não se escreve em quatro caixas de número. ⇒ duas
//! alças no canvas, desenhadas como as alças de Bézier que todo editor vectorial tem.
//!
//! # ⛔⛔ A armadilha que este ficheiro existe para apanhar
//!
//! No ponto NEUTRO a alça está **em cima do eixo do osso** (ela é o ponto de controlo no terço), e
//! o corpo do osso é um alvo com outro verbo. Sem cuidado ela nasce **inalcançável no único estado
//! em que todo osso nasce** — *uma alça que só se agarra depois de já ter sido movida não se agarra
//! nunca*.

use super::*;
use crate::goal::braco;

/// O zoom de trabalho — o mesmo do irmão: um osso de 10 unidades a ~100 px.
const PX: f64 = 0.1;

/// O que o dedo apanha em `p`, com `foco` no osso.
fn agarra(sim: &SimWorld, p: [f64; 2], foco: Entity) -> Option<ph2d_skeleton_render::BoneHover> {
    crate::bone_pick::hover(
        sim,
        p,
        PX,
        Some(foco.to_bits()),
        ph2d_tool_vector::BoneAction::Transform,
    )
}

/// As duas extremidades de um osso em MUNDO, pela porta do produto.
fn extremos(sim: &SimWorld, bits: u64) -> ([f64; 2], [f64; 2]) {
    let pts = ph2d_skeleton_live::skin_live::bone_polylines(sim)
        .into_iter()
        .find(|(x, _)| *x == bits)
        .map(|(_, p)| p)
        .expect("o osso tem corpo");
    (
        *pts.first().expect("dois nos"),
        *pts.last().expect("dois nos"),
    )
}

/// O braço com o osso `k` a ter `segments` sub-ossos — é o que faz a curvatura deixar de ser inerte.
fn braco_com_segmentos(k: usize, segments: u8) -> (SimWorld, [Entity; 2]) {
    let (mut sim, ossos) = braco();
    if let Some(mut b) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(ossos[k]) {
        b.segments = segments;
    }
    (sim, ossos)
}

/// ⭐⭐⭐⭐ **AS DUAS ALÇAS DE CURVATURA PEGAM ONDE SÃO PINTADAS, E NO PONTO NEUTRO** — o caso que
/// quase as deixou mortas à nascença.
///
/// ⚠️ **A posição vem da porta do DESENHO** ([`ph2d_skeleton_live::skin_live::bend_handles`]), a
/// mesma que o overlay chama. *Escrever aqui onde eu acho que a alça está mediria outra coisa.*
///
/// (Mutação: no `hover`, tirar a excepção `curvatura ||` do filtro do `d_osso` ⇒ RED, porque no
/// neutro a alça e o corpo estão à mesma distância zero.)
#[test]
fn as_alcas_de_curvatura_pegam_onde_sao_pintadas_mesmo_no_ponto_neutro() {
    let (sim, ossos) = braco_com_segmentos(0, 8);
    let (alcas, _) = ph2d_skeleton_live::skin_live::bend_handles(&sim, ossos[0].to_bits())
        .expect("um osso com segmentos tem alcas");
    for (p, esperado) in [
        (alcas[0], ph2d_skeleton_render::BonePart::BendIn),
        (alcas[1], ph2d_skeleton_render::BonePart::BendOut),
    ] {
        let h = agarra(&sim, p, ossos[0]).expect("ha' alvo onde a alca esta' pintada");
        assert_eq!(h.bone, ossos[0].to_bits());
        assert_eq!(h.part, esperado, "em {p:?}");
    }
}

/// ⛔⛔ **UM OSSO SEM SEGMENTOS NÃO TEM ALÇA DE CURVATURA** — nem pintada, nem agarrável, nem
/// escrevível.
///
/// ⚠️ **Com `segments = 1` a curvatura é inerte POR CONSTRUÇÃO** (o `BoneSpec::is_rigid` colapsa o
/// osso num só, seja qual for a `Bend`). Oferecer a alça ali seria prometer um verbo que o arrasto
/// não executa — a espécie de controlo morto que este módulo acabou de pagar no botão `Smooth`.
///
/// (Mutação: tirar a guarda `segments_of(..) <= 1` do `bend_handles` ⇒ RED nas três metades.)
#[test]
fn um_osso_sem_segmentos_nao_tem_alca_de_curvatura() {
    let (mut sim, ossos) = braco();
    let bits = ossos[0].to_bits();
    // (a) o desenho não tem o que pintar…
    assert!(
        ph2d_skeleton_live::skin_live::bend_handles(&sim, bits).is_none(),
        "um osso rigido nao devia oferecer alcas"
    );
    // (b) …e o dedo, no sítio onde ELAS ESTARIAM, apanha o corpo do osso.
    //
    // ⚠️ **Os terços saem da POLILINHA do produto**, e não de uma pose reconstruída aqui: num osso
    // rígido ela é o eixo, e o terço dela É onde a alça estaria.
    let (raiz, ponta) = extremos(&sim, bits);
    for f in [1.0 / 3.0, 2.0 / 3.0] {
        let p = [
            raiz[0] + (ponta[0] - raiz[0]) * f,
            raiz[1] + (ponta[1] - raiz[1]) * f,
        ];
        let h = agarra(&sim, p, ossos[0]).expect("o corpo do osso esta' la'");
        assert_ne!(h.part, ph2d_skeleton_render::BonePart::BendIn);
        assert_ne!(h.part, ph2d_skeleton_render::BonePart::BendOut);
    }
    // (c) …e o verbo recusa escrever.
    assert!(
        !ph2d_skeleton_live::skin_live::set_bend_handle(&mut sim, bits, [1.0, 1.0], false),
        "escrever curvatura num osso rigido devia ser recusado"
    );
}

/// ⭐⭐⭐ **ARRASTAR A ALÇA ARQUEIA O OSSO, e a alça fica DEBAIXO DO DEDO** — a ida e a volta.
///
/// ⚠️ **A régua é a IDA E VOLTA pela porta do desenho**: depois de escrever, a alça pintada tem de
/// estar no ponto para onde a mão foi. *Um arrasto em que o controlo não segue o dedo lê-se como
/// «o gizmo escorrega», e foi assim que três reports deste módulo nasceram.*
#[test]
fn arrastar_a_alca_arqueia_o_osso_e_ela_segue_o_dedo() {
    for (ponta, qual) in [(false, 0usize), (true, 1)] {
        let (mut sim, ossos) = braco_com_segmentos(0, 8);
        let bits = ossos[0].to_bits();
        let antes = ph2d_skeleton_live::skin_live::bend_handles(&sim, bits)
            .expect("alcas")
            .0[qual];
        // Puxar a alça para FORA do eixo — é `y` que arqueia.
        let alvo = [antes[0] + 1.0, antes[1] + 3.0];
        assert!(
            crate::bone_pose::pose(
                &mut sim,
                ossos[0],
                alvo,
                if ponta {
                    ph2d_skeleton_render::BonePart::BendOut
                } else {
                    ph2d_skeleton_render::BonePart::BendIn
                },
            ),
            "o verbo tinha de executar"
        );
        let depois = ph2d_skeleton_live::skin_live::bend_handles(&sim, bits)
            .expect("alcas")
            .0[qual];
        assert!(
            (depois[0] - alvo[0]).abs() < 1e-9 && (depois[1] - alvo[1]).abs() < 1e-9,
            "a alca {qual} foi para {depois:?} e o dedo estava em {alvo:?}"
        );
        // ⭐ E o osso deixou de ser recto — sem isto o gate acima passaria com uma curvatura
        // gravada que o produtor de sub-ossos ignora.
        let spec = sim
            .world()
            .get::<ph2d_skeleton_ecs::Bone>(ossos[0])
            .expect("osso")
            .spec();
        assert!(
            !spec.is_rigid(),
            "o osso continuou rigido depois do arrasto"
        );
        assert!(
            ph2d_skeleton::bend::polyline(spec).len() > 2,
            "a polilinha do corpo continuou a ser um segmento"
        );
        // ⚠️ E a OUTRA alça não se mexeu: são dois controlos, não um.
        let outra = ph2d_skeleton_live::skin_live::bend_handles(&sim, bits)
            .expect("alcas")
            .0[1 - qual];
        let original =
            ph2d_skeleton_live::skin_live::bend_handles(&braco_com_segmentos(0, 8).0, bits);
        if let Some((o, _)) = original {
            assert!(
                (outra[0] - o[1 - qual][0]).abs() < 1e-9,
                "mexer numa alca mexeu a outra"
            );
        }
    }
}

/// ⛔ **LARGAR A ALÇA ONDE ELA ESTAVA NÃO MUDA O OSSO** — a ida e a volta fecham.
///
/// ⚠️ Sem isto um clique sem arrasto sobre a alça mexeria o osso, e um esqueleto autorado mudaria
/// só por alguém lhe ter tocado.
///
/// ⚠️⚠️ **E a barra é `1e-12` e não `assert_eq!`, porque a 1.ª redacção afirmou EXACTIDÃO e
/// reprovou por 1 ULP.** O par [`ph2d_skeleton::bend::handles`] / `bend_from_handle` **é** exacto —
/// é a mesma expressão nos dois sentidos, e há gate na folha a dizê-lo. O que não é exacto é a
/// travessia que o GESTO faz por fora dele: a alça sai para MUNDO pela pose do osso e volta pelo
/// **inverso** dela, e inverter um afim custa arredondamento. *Uma afirmação de exactidão tem de
/// nomear o caminho em que ela vale.*
#[test]
fn largar_a_alca_onde_ela_estava_nao_muda_o_osso() {
    let (mut sim, ossos) = braco_com_segmentos(0, 8);
    // Um osso já arqueado: no neutro o gate passaria por os dois lados serem zero.
    if let Some(mut b) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(ossos[0]) {
        b.curve = ph2d_skeleton::bend::Bend {
            inn: [0.13, -0.37],
            out: [-0.21, 0.44],
        };
    }
    let bits = ossos[0].to_bits();
    let antes = sim
        .world()
        .get::<ph2d_skeleton_ecs::Bone>(ossos[0])
        .expect("osso")
        .curve;
    let (alcas, _) = ph2d_skeleton_live::skin_live::bend_handles(&sim, bits).expect("alcas");
    for (ponta, p) in [(false, alcas[0]), (true, alcas[1])] {
        assert!(ph2d_skeleton_live::skin_live::set_bend_handle(
            &mut sim, bits, p, ponta
        ));
    }
    let depois = sim
        .world()
        .get::<ph2d_skeleton_ecs::Bone>(ossos[0])
        .expect("osso")
        .curve;
    for (a, b, nome) in [
        (antes.inn, depois.inn, "inn"),
        (antes.out, depois.out, "out"),
    ] {
        assert!(
            (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12,
            "largar a alca {nome} no sitio mudou a curvatura: {a:?} -> {b:?}"
        );
    }
}

/// ⛔⛔ **O MEIO DE UM OSSO CURVO AINDA GIRA** — o recurso que o [`crate::bone_pick::BEND_HIT_PX`]
/// gasta, medido.
///
/// As duas alças ficam nos terços, então elas comem `2 × 2 × BEND_HIT_PX` do osso e o ponto do meio
/// fica a `L/6` de cada uma. ⇒ o verbo de girar sobrevive no meio a partir de `48 px` de osso na
/// tela — e a fixtura está a `100 px`, que é a ordem em que ele de facto aparece.
///
/// ⚠️ **É a régua que impede alguém de «engordar» aquele raio**: a alça é apertada de propósito,
/// porque é a única que ganha ao corpo do osso.
#[test]
fn o_meio_de_um_osso_curvo_ainda_gira() {
    let (sim, ossos) = braco_com_segmentos(0, 8);
    let (raiz, ponta) = extremos(&sim, ossos[0].to_bits());
    let meio = [
        f64::midpoint(raiz[0], ponta[0]),
        f64::midpoint(raiz[1], ponta[1]),
    ];
    let h = agarra(&sim, meio, ossos[0]).expect("o corpo esta' la'");
    assert_eq!(
        h.part,
        ph2d_skeleton_render::BonePart::Body,
        "o meio do osso deixou de girar — a alca de curvatura comeu-o"
    );
    // ⭐ E a conta que o doc do `BEND_HIT_PX` faz: o meio dista `L/6` de cada alça, em px de tela.
    let l_px = (ponta[0] - raiz[0]).hypot(ponta[1] - raiz[1]) / PX;
    assert!(
        l_px / 6.0 > crate::bone_pick::BEND_HIT_PX,
        "a fixtura nao reproduz a condicao do doc: L/6 = {:.1} px contra {:.1}",
        l_px / 6.0,
        crate::bone_pick::BEND_HIT_PX
    );
}

// ─────────────────────────────────────────────────────────────────────────────────────────────
// O `From Chain` — as alças derivadas dos ossos vizinhos
// ─────────────────────────────────────────────────────────────────────────────────────────────

/// Põe o osso `k` do braço em `Auto`.
fn braco_auto(k: usize, segments: u8) -> (SimWorld, [Entity; 2]) {
    let (mut sim, ossos) = braco_com_segmentos(k, segments);
    if let Some(mut b) = sim.world_mut().get_mut::<ph2d_skeleton_ecs::Bone>(ossos[k]) {
        b.handles = ph2d_skeleton::bend::Handles::Auto;
    }
    (sim, ossos)
}

/// ⭐⭐⭐⭐ **NUMA CORRENTE RECTA, LIGAR O `From Chain` NÃO MOVE UM PIXEL** — e a régua é a
/// polilinha do PRODUTO, ao bit.
///
/// ⚠️⚠️ **É esta propriedade que faz o modo ser seguro de experimentar**, e ela só vale porque as
/// tangentes saem da transformação RELATIVA e não de uma volta pelo mundo (ver o
/// [`ph2d_skeleton_live::skin_live::effective_spec`]). *Uma volta pelo mundo teria deixado
/// `y ≈ 1e-17`, e o rig inteiro arquearia um bocadinho ao ligar o modo.*
///
/// (Mutação: no `effective_spec`, tirar o `local_de` e usar o `world_of` ⇒ RED.)
#[test]
fn numa_corrente_recta_o_from_chain_nao_move_um_pixel() {
    let (reto, ossos) = braco_com_segmentos(0, 8);
    let antes = ph2d_skeleton_live::skin_live::bone_polylines(&reto);
    let (auto, _) = braco_auto(0, 8);
    let depois = ph2d_skeleton_live::skin_live::bone_polylines(&auto);
    assert_eq!(
        antes, depois,
        "ligar o From Chain numa corrente recta mexeu"
    );
    // ⭐ E o osso continua a colapsar num só sub-osso — a fábrica não paga por uma recta.
    let spec = ph2d_skeleton_live::skin_live::effective_spec(&auto, ossos[0]).expect("osso");
    assert!(spec.is_rigid(), "a corrente recta deixou de colapsar");
}

/// ⭐⭐⭐ **COM A CORRENTE DOBRADA, O `From Chain` ARQUEIA O OSSO** — e o *Manual* deixa-o recto.
///
/// ⚠️ **O controlo é o MESMO mundo com o modo em `Authored`**: sem ele, este gate mediria a dobra
/// da corrente em vez do modo.
#[test]
fn com_a_corrente_dobrada_o_from_chain_arqueia_o_osso() {
    let dobrar = |sim: &mut SimWorld, ossos: [Entity; 2]| {
        if let Some(mut t) = sim.world_mut().get_mut::<Transform>(ossos[1]) {
            t.rotation = 0.9;
        }
    };
    // (a) o CONTROLO: modo autorado, curvatura zero ⇒ o osso fica recto.
    let (mut manual, ossos) = braco_com_segmentos(0, 8);
    dobrar(&mut manual, ossos);
    let spec_m = ph2d_skeleton_live::skin_live::effective_spec(&manual, ossos[0]).expect("osso");
    assert!(
        spec_m.is_rigid(),
        "em Manual e sem curvatura escrita o osso tinha de ficar recto"
    );
    // (b) e o modo novo: a tangente da ponta vira com o filho, e o corpo arqueia.
    let (mut auto, _) = braco_auto(0, 8);
    dobrar(&mut auto, ossos);
    let spec_a = ph2d_skeleton_live::skin_live::effective_spec(&auto, ossos[0]).expect("osso");
    assert!(
        !spec_a.is_rigid(),
        "em From Chain, com a corrente dobrada, o osso tinha de arquear"
    );
    let pts = ph2d_skeleton::bend::polyline(spec_a);
    assert!(pts.len() > 2, "a polilinha continuou a ser um segmento");
    // ⛔ E a PONTA não se mexe — a curvatura arqueia o CORPO, e quem manda na ponta é o `length`.
    let ponta = *pts.last().expect("dois nos");
    assert!(
        (ponta[0] - spec_a.length).abs() < 1e-12 && ponta[1].abs() < 1e-12,
        "a ponta do osso mexeu-se: {ponta:?}"
    );
}

/// ⛔⛔ **EM `From Chain` NÃO HÁ ALÇA PARA AGARRAR** — nem pintada, nem agarrável, nem escrevível.
///
/// ⚠️ **As duas são DERIVADAS da corrente**, então arrastar uma seria escrever num valor que o
/// quadro seguinte recalcula — *o artista veria o gizmo voltar debaixo do dedo*, que é o defeito
/// que a âncora de IK deste módulo já pagou por escrito.
///
/// (Mutação: tirar a guarda `handles == Auto` do `bend_handles` ⇒ RED.)
#[test]
fn em_from_chain_nao_ha_alca_para_agarrar() {
    let (mut sim, ossos) = braco_auto(0, 8);
    let bits = ossos[0].to_bits();
    assert!(
        ph2d_skeleton_live::skin_live::bend_handles(&sim, bits).is_none(),
        "em From Chain nao devia haver alca pintada"
    );
    assert!(
        !ph2d_skeleton_live::skin_live::set_bend_handle(&mut sim, bits, [1.0, 1.0], false),
        "em From Chain o arrasto devia ser recusado"
    );
    // ⭐ E o valor AUTORADO fica intocado: voltar a Manual devolve o que lá estava.
    let antes = sim
        .world()
        .get::<ph2d_skeleton_ecs::Bone>(ossos[0])
        .expect("osso")
        .curve;
    assert_eq!(antes, ph2d_skeleton::bend::Bend::STRAIGHT);
}
