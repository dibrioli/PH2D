//! Os gates da malha que leva os pesos dentro.

use super::SkinnedMesh;
use ph2d_poly2d::Mesh2d;

fn malha(verts: usize) -> Mesh2d {
    Mesh2d {
        #[expect(clippy::cast_precision_loss, reason = "fixtura de meia dúzia de vértices")]
        rest: (0..verts).map(|i| [i as f64, 0.0]).collect(),
        tris: vec![[0, 1, 2]],
        size: [10, 10],
    }
}

/// ⭐⭐⭐ **A CONTAGEM DE OSSOS É DERIVADA, e a fatia de cada vértice sai certa.**
#[test]
fn a_contagem_de_ossos_deriva_se_e_cada_vertice_le_a_fatia_dele() {
    let s = SkinnedMesh {
        mesh: malha(4),
        pesos: vec![
            1.0, 0.0, 0.0, // v0
            0.5, 0.5, 0.0, // v1
            0.0, 1.0, 0.0, // v2
            0.0, 0.0, 1.0, // v3
        ],
    };
    assert_eq!(s.ossos(), 3, "4 vertices x 3 ossos = 12 pesos");
    assert!(s.valida());
    assert_eq!(s.pesos_de(1), &[0.5, 0.5, 0.0]);
    assert_eq!(s.pesos_de(3), &[0.0, 0.0, 1.0]);
    assert!(s.pesos_de(4).is_empty(), "fora da malha nao inventa fatia");
}

/// ⛔⛔ **UMA TABELA QUE NÃO FECHA É RECUSADA, e não lida deslocada.**
///
/// ⚠️ **É o modo de falha que importa:** uma tabela a que falte um vértice ainda entrega pesos
/// *plausíveis* a todo vértice — só que os do vizinho. A arte sai deformada de um jeito que passa
/// por todo gate de geometria (a soma é 1, nada é negativo, nada é órfão).
///
/// (Mutação: trocar `% n == 0` por `>= n` ⇒ RED.)
#[test]
fn uma_tabela_que_nao_fecha_e_recusada() {
    let s = SkinnedMesh {
        mesh: malha(4),
        // 11 pesos para 4 vértices — não é múltiplo de nada.
        pesos: vec![0.25; 11],
    };
    assert!(!s.valida(), "11 nao e' multiplo de 4 e a porta tem de o dizer");
}

/// ⭐ **SEM PESOS É UM ESTADO LEGAL** — a leitura de *«resolve pela lei derivada»*.
#[test]
fn sem_pesos_e_um_estado_legal_e_diz_zero_ossos() {
    let s = SkinnedMesh::sem_pesos(malha(4));
    assert_eq!(s.ossos(), 0);
    assert!(s.valida(), "uma malha sem tabela e' coerente");
    assert!(s.pesos_de(0).is_empty());
}

/// ⭐⭐⭐ **A MALHA E OS PESOS ATRAVESSAM O ARQUIVO JUNTOS** — não há como gravar uma sem a outra.
///
/// ⚠️ Este é o gate que substitui a objecção do *vector paralelo*: a ida-e-volta é de **um**
/// objecto, logo uma edição não tem por onde dessincronizar as duas listas.
#[test]
fn a_malha_e_os_pesos_atravessam_o_arquivo_juntos() {
    let s = SkinnedMesh {
        mesh: malha(3),
        pesos: vec![1.0, 0.0, 0.25, 0.75, 0.5, 0.5],
    };
    let bytes = postcard::to_allocvec(&s).expect("serializa");
    let volta: SkinnedMesh = postcard::from_bytes(&bytes).expect("desserializa");
    assert_eq!(volta, s, "a ida-e-volta perdeu alguma coisa");
    assert_eq!(volta.ossos(), 2);
}
