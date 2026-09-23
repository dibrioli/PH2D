//! Os gates do [`super::export_assado`] — ver o cabeçalho dele para o mecanismo.
//!
//! ⚠️ **Todos correm SEM adaptador**, e isso é a decisão: a metade que decide
//! *o que sai no ficheiro* não tem um pixel dentro. *Quando um gate precisa de
//! um device para medir uma decisão que não tem pixel nenhum, a lei está no
//! sítio errado* — a frase que esta família já pagou duas vezes.

use super::{Assados, material, nomes, perdeu_tinta_fina, uvs};
use ph2d_mesh::MeshFormat;
use ph2d_mesh_colors::{Assado, Relatorio};

/// Um assado de brincar — o que interessa aqui é a PRESENÇA, não os píxeis.
fn assado() -> Assado {
    Assado {
        lado_px: 8,
        rgba: vec![0; 8 * 8 * 4],
        uv: vec![[0.1, 0.2], [0.3, 0.4], [0.5, 0.6]],
        off_uv: vec![0, 3],
        relatorio: Relatorio::default(),
    }
}

/// ⭐⭐⭐ **A tabela-verdade do aviso, e ela tem QUATRO células porque «ter» e
/// «PERDER» são perguntas diferentes.**
///
/// ⛔ A célula que custa é a última: o formato **carrega** a tinta fina e a peça
/// **não coube na textura** ⇒ ela fica para trás e o aviso **tem** de a nomear.
/// *Um aviso que se calasse ali seria a resposta errada com a confiança da
/// certa* — e é o único sítio onde o `keeps_fine_paint` sozinho mente.
#[test]
fn ter_tinta_fina_e_perde_la_sao_perguntas_diferentes() {
    let com = Assados {
        por_peca: vec![Some(assado())],
        recusas: Vec::new(),
    };
    let recusada = Assados {
        por_peca: vec![None],
        recusas: vec![(0, ph2d_mesh_colors::Recusa::SemFaces)],
    };
    let vazio = Assados::default();

    for fmt in MeshFormat::ALL {
        assert!(
            !perdeu_tinta_fina(fmt, false, &vazio),
            "{fmt:?}: uma cena SEM tinta fina não pode perder nenhuma"
        );
        assert!(
            !perdeu_tinta_fina(fmt, false, &com),
            "{fmt:?}: o assado não inventa tinta fina onde não havia"
        );
        if fmt.keeps_fine_paint() {
            assert!(
                !perdeu_tinta_fina(fmt, true, &com),
                "{fmt:?} carrega a tinta e ela assou: o aviso tem de se calar"
            );
            assert!(
                perdeu_tinta_fina(fmt, true, &recusada),
                "{fmt:?} carrega a tinta e ela NÃO assou: ficou para trás e \
                 o aviso tem de o dizer"
            );
        } else {
            assert!(
                perdeu_tinta_fina(fmt, true, &com),
                "{fmt:?} não carrega a tinta: o aviso tem de soar sempre"
            );
        }
    }
    // O CONTROLO da população: as duas metades da tabela têm de existir.
    assert!(MeshFormat::ALL.iter().any(|f| f.keeps_fine_paint()));
    assert!(MeshFormat::ALL.iter().any(|f| !f.keeps_fine_paint()));
}

/// ⚠️ **Os três nomes saem do que o artista ESCREVEU**, e o `.png` de uma peça
/// sem plano fica **vazio** — nunca um nome que aponta para um ficheiro que
/// ninguém gravou.
#[test]
fn os_nomes_saem_do_que_o_artista_escreveu() {
    let pecas = vec![Some(assado()), None, Some(assado())];
    let (mtl, pngs) = nomes(std::path::Path::new("/tmp/arte/retrato-v3.obj"), &pecas);
    assert_eq!(mtl, "retrato-v3.mtl");
    assert_eq!(pngs, vec!["retrato-v3_0.png", "", "retrato-v3_2.png"]);
    for n in pngs.iter().filter(|n| !n.is_empty()) {
        assert!(!n.contains('/'), "o nome do png leva um caminho: {n}");
    }
}

/// ⭐ **Uma peça sem plano NÃO entra na lista de coordenadas**, e a que entra
/// leva o material DELA.
///
/// ⚠️ É a metade que impede o `.obj` de apontar uma peça ao material de outra —
/// um erro que **abre sem queixa** no destino e pinta a peça errada.
#[test]
fn so_quem_assou_entra_nas_coordenadas() {
    let assados = Assados {
        por_peca: vec![None, Some(assado())],
        recusas: Vec::new(),
    };
    let mats: Vec<String> = (0..2).map(material).collect();
    let uvs = uvs(&assados, &mats);
    assert!(uvs[0].is_none(), "uma peça sem plano recebeu coordenadas");
    let u = uvs[1].as_ref().expect("a peça com plano tem coordenadas");
    assert_eq!(u.material, "ph2d_1", "a peça 1 aponta ao material de outra");
    assert_eq!(u.uv.len(), 3);
    assert_eq!(u.off, &[0, 3]);
}
