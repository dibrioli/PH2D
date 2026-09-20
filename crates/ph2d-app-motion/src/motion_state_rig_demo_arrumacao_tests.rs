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
