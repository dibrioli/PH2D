//! ⭐⭐⭐ **A ARRUMAÇÃO AUTOMÁTICA, medida na cena que o dono fotografou** — os gates dos três
//! reports de 2026-09-20 (*«a linha do Shape cruza os nós»* · *«nós estão se interpenetrando»* ·
//! *«se há 2 níveis, os níveis se alinham no centro»*).
//!
//! ⚠️ **Irmãos dos gates da cena por RESPONSABILIDADE** (e o tecto de LOC obrigou ao corte, que a
//! pergunta já pedia): ali mede-se o que a cena ENSINA — a corda cai, o campo ondula, a pele segue
//! os ossos —, aqui mede-se onde os cartões ficam POUSADOS. As duas mudam por razões diferentes, e
//! só uma delas tem a ver com a lei em camadas do [`ph2d_nodegraph::layout`].
//!
//! ⛔ **A régua é sempre a caixa DESENHADA e nunca o passo da grelha** — é exactamente essa a
//! distinção que os defeitos tinham: com um passo constante de `220` duas pastilhas de `232,5`
//! tocavam-se sem que régua nenhuma desta casa o visse.

use super::build;
use crate::motion_state::MotionState;
use ph2d_motion_doc::layout::Medida as _;
use ph2d_nodegraph::graph::NodeId;

// ---------------------------------------------------------------------------
// A ARRUMAÇÃO AUTOMÁTICA sobre esta cena — os reports do dono de 2026-09-20.
// ---------------------------------------------------------------------------

/// A caixa que um cartão DESENHA, em unidades de grafo.
fn caixa(m: &MotionState, medidas: &impl ph2d_motion_doc::layout::Medida, n: NodeId) -> [f32; 4] {
    let p = m.doc.graph.pos(n).expect("posicao");
    let e = medidas.extensao(ph2d_motion_doc::layout::Carta::No(n));
    [p.x + e.left, p.y + e.top, p.x + e.right, p.y + e.bottom]
}

/// **NENHUM cartão desta cena se sobrepõe a outro depois de ARRUMAR** — o report
/// *«nós estão se interpenetrando»* (2026-09-20), medido pela porta do PRODUTO.
///
/// ⚠️ **A régua é a caixa DESENHADA e não o passo da grelha**, que é exactamente a distinção que
/// o defeito tinha: com um passo constante de `220` duas pastilhas de `232,5` tocavam-se sem que
/// nenhuma régua desta casa o visse.
///
/// ⚠️⚠️ **Aqui a medida do nome cai na ESTIMATIVA** (a tabela do painel é preenchida no quadro, e
/// um teste não pinta), e ela é generosa de propósito — *o gate mede o lado LARGO, logo aprova
/// menos do que o produto*. FALSIFICADO por devolver o passo a `DX`/`DY` constantes.
#[test]
fn a_arrumacao_nao_sobrepoe_dois_cartoes() {
    let mut m = MotionState::new();
    let _ = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_arrumar::arrumar(&mut m);

    let medidas = crate::motion_bridge::medida::medir(&m);
    let ids: Vec<NodeId> = m.doc.graph.nodes().iter().map(|n| n.id).collect();
    assert!(
        ids.len() > 30,
        "a cena tem cartoes que cheguem ({})",
        ids.len()
    );

    for (i, &a) in ids.iter().enumerate() {
        for &b in &ids[i + 1..] {
            let ca = caixa(&m, &medidas, a);
            let cb = caixa(&m, &medidas, b);
            let cruza = ca[0] < cb[2] && cb[0] < ca[2] && ca[1] < cb[3] && cb[1] < ca[3];
            assert!(
                !cruza,
                "os cartoes {a:?} e {b:?} sobrepoem-se: {ca:?} contra {cb:?}"
            );
        }
    }
}

/// **A forma de cada pano é arrumada ao LADO do duplicador dela e ACIMA do fio das posições** —
/// as duas primeiras ordens do dono (2026-09-20), medidas na cena que ele fotografou.
///
/// ⛔ Antes: a `source.shape` não tem produtor, logo a disposição por caminho mais longo
/// arquivava-a na coluna `0` — medido, `x = 60` contra `x = 720` do `motion.duplicator` —, e o
/// fio dela atravessava o `Scale` e o `Move` para aterrar na porta de CIMA. FALSIFICADO por
/// apagar o passe de puxar à direita (a forma volta à coluna `0`) ou o desempate por PORTA (ela
/// e o `motion.move` trocam de lugar metade das vezes).
#[test]
fn a_forma_e_arrumada_ao_lado_do_duplicador_e_acima_do_fio() {
    let mut m = MotionState::new();
    let _ = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_arrumar::arrumar(&mut m);

    let por_tipo = |t: &str| -> Vec<NodeId> {
        m.doc
            .graph
            .nodes()
            .iter()
            .filter(|n| n.type_name == t)
            .map(|n| n.id)
            .collect()
    };
    let dups = por_tipo("motion.duplicator");
    assert_eq!(dups.len(), 6, "os seis panos da cena");

    for dup in dups {
        // A porta 0 do duplicador é a FORMA e a 1 é a corrente de posições (ADR-0155).
        let de = |porta: u16| -> NodeId {
            m.doc
                .graph
                .edges()
                .iter()
                .find(|e| e.to == (dup, porta))
                .unwrap_or_else(|| panic!("o duplicador {dup:?} tem a porta {porta} ligada"))
                .from
                .0
        };
        let forma = de(0);
        let fio = de(1);
        let (pf, pd, pm) = (
            m.doc.graph.pos(forma).expect("forma"),
            m.doc.graph.pos(dup).expect("dup"),
            m.doc.graph.pos(fio).expect("fio"),
        );
        assert!(
            pf.x < pd.x && (pf.x - pm.x).abs() < 0.5,
            "a forma partilha a coluna de quem alimenta a outra porta do mesmo duplicador \
             (forma {:.1}, fio {:.1}, dup {:.1})",
            pf.x,
            pm.x,
            pd.x
        );
        assert!(
            pf.y < pm.y,
            "a porta 0 desenha ACIMA da porta 1 (forma {:.1}, fio {:.1})",
            pf.y,
            pm.y
        );
    }
}

/// ⭐⭐⭐ **O RETRATO DE QUEM ESTÁ NO TOPO DE UMA COLUNA SAI POR CIMA** — o report do dono de
/// 2026-09-20 (*«neste caso o preview deveria ser colocado para cima»*), medido na cena que ele
/// fotografou e pela porta do PRODUTO.
///
/// Em cada um dos seis painéis a `source.shape` alimenta a porta `0` do duplicador dela e tem o
/// `motion.move` do painel logo por baixo, na MESMA coluna: ali o lado de baixo é o corredor
/// entre duas fileiras — que é o que a foto mostra — e o de cima é espaço aberto.
///
/// ⚠️⚠️ **A segunda metade é o CONTROLO e vale metade do gate:** o cartão de baixo **mantém** a
/// moldura em baixo. Sem ela, uma lei que respondesse *«em cima»* a toda gente passava a primeira
/// asserção — e punha a moldura do `motion.move` exactamente no corredor de que ela tirou a da
/// forma. FALSIFICADO por devolver `Retrato em baixo` a toda gente (a 1.ª cai) ou a toda gente
/// (a 2.ª cai).
#[test]
fn o_retrato_de_quem_tem_alguem_por_baixo_sai_por_cima() {
    let mut m = MotionState::new();
    let _ = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_arrumar::arrumar(&mut m);
    let medidas = crate::motion_bridge::medida::medir(&m);
    let topo = |n: NodeId| medidas.extensao(ph2d_motion_doc::layout::Carta::No(n)).top;

    let dups: Vec<NodeId> = m
        .doc
        .graph
        .nodes()
        .iter()
        .filter(|n| n.type_name == "motion.duplicator")
        .map(|n| n.id)
        .collect();
    assert_eq!(dups.len(), 6, "os seis panos da cena");

    for dup in dups {
        let de = |porta: u16| -> NodeId {
            m.doc
                .graph
                .edges()
                .iter()
                .find(|e| e.to == (dup, porta))
                .unwrap_or_else(|| panic!("o duplicador {dup:?} tem a porta {porta} ligada"))
                .from
                .0
        };
        let (forma, fio) = (de(0), de(1));
        let (pf, pm) = (
            m.doc.graph.pos(forma).expect("forma"),
            m.doc.graph.pos(fio).expect("fio"),
        );
        assert!(
            (pf.x - pm.x).abs() < 0.5 && pf.y < pm.y,
            "a premissa deste gate: a forma esta na coluna do fio e por cima dele"
        );
        assert!(
            topo(forma) < 0.0,
            "a forma tem alguem por baixo na coluna dela, logo o retrato sobe (topo {})",
            topo(forma)
        );
        assert!(
            topo(fio).abs() < f32::EPSILON,
            "o cartao de baixo nao tem para onde subir e mantem a moldura em baixo (topo {})",
            topo(fio)
        );
    }
}

/// ⭐⭐ **ARRUMAR OUTRA VEZ NÃO MEXE UM BIT** — a lei do lado do retrato lê onde os cartões estão
/// e move-os, logo ela realimenta-se; [`crate::motion_bridge::medida::arrumar`] pára quando a
/// disposição que saiu pede exactamente os lados com que foi construída.
///
/// ⛔ **Este gate é o que impede o tecto de passes de virar um palpite:** ele afirma que o ponto
/// fixo é ALCANÇADO nesta cena, e não que ele existe sempre. FALSIFICADO por baixar o tecto a `2`
/// (a cena do dono precisa de `3`) — que foi exactamente a redacção que esta wave teve primeiro.
#[test]
fn arrumar_uma_segunda_vez_nao_mexe_um_bit() {
    let mut m = MotionState::new();
    let _ = build(&mut m.doc, &m.registry).expect("a cena monta");
    crate::motion_arrumar::arrumar(&mut m);

    let antes: Vec<(u32, f32, f32)> = m
        .doc
        .graph
        .nodes()
        .iter()
        .map(|n| {
            let p = m.doc.graph.pos(n.id).expect("pos");
            (n.id.0, p.x, p.y)
        })
        .collect();
    assert!(antes.len() > 30, "a cena tem cartoes que cheguem");

    crate::motion_arrumar::arrumar(&mut m);
    let depois: Vec<(u32, f32, f32)> = m
        .doc
        .graph
        .nodes()
        .iter()
        .map(|n| {
            let p = m.doc.graph.pos(n.id).expect("pos");
            (n.id.0, p.x, p.y)
        })
        .collect();
    assert_eq!(antes, depois, "a arrumacao chegou a um ponto fixo");
}

/// ⭐ **A MEDIDA cobre o cartão que se PINTA com o número por baixo** (auditoria do fecho,
/// 2026-09-24). O gate irmão mede sobreposição com a MESMA medida que arruma, logo não podia ver
/// uma medida curta: os dois lados encolhiam juntos. ⇒ esta régua mede contra o retrato do
/// PAINEL com um readout carimbado, que é o que o quadro desenha depois de cozinhar.
/// FALSIFICADO por apagar a reserva no `medir` (cada cartão sai uma fileira mais baixo).
#[test]
fn a_medida_reserva_a_fileira_do_numero() {
    let mut m = MotionState::new();
    let _ = build(&mut m.doc, &m.registry).expect("a cena monta");
    let medidas = crate::motion_bridge::medida::medir(&m);
    let mut snap = ph2d_panel_motion_graph::snapshot_from(&m.doc.graph, &m.registry);
    crate::motion_bridge::params::card::stamp_card_params(
        &m,
        ph2d_editor_core::ProjectSettings::default(),
        &mut snap,
    );
    let mut vistos = 0;
    for n in &mut snap.nodes {
        n.readout = Some("0".to_string());
        let pintado = ph2d_panel_motion_graph::extensao_desenhada(n, false);
        let medido = medidas.extensao(ph2d_motion_doc::layout::Carta::No(NodeId(n.id)));
        // A ALTURA e nunca a borda: o retrato do cartão sai em cima OU em baixo conforme o
        // corredor (`lados_dos_retratos`), e a altura total é a mesma dos dois lados.
        let (h_medido, h_pintado) = (medido.bottom - medido.top, pintado.bottom - pintado.top);
        assert!(
            h_medido >= h_pintado - 1e-3,
            "no' {}: a medida tem {h_medido} de altura e o cartao com o numero pinta {h_pintado}",
            n.id,
        );
        vistos += 1;
    }
    assert!(vistos > 30, "a cena tem cartoes que cheguem ({vistos})");
}
