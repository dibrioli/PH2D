//! ⭐⭐⭐ **OS GATES DO PASSE** — e o que eles defendem não é o desemaranhamento (a crate
//! [`ph2d_untangle`] tem os gates dele); é a **cerca**: *a costura não se mexe, e um mapa bom
//! não se toca.*

use super::{UntangleReport, untangle_patches};
use crate::cut::CutMesh;
use crate::solve::GridMap;
use ph2d_mesh::{Face, Mesh};

/// Uma malha plana `3 × 3` num retalho só, com o mapa **igual** à geometria — logo a identidade.
fn fixtura() -> (Mesh, CutMesh, GridMap) {
    let mut pos: Vec<[f32; 3]> = Vec::new();
    for j in 0..3 {
        for i in 0..3 {
            #[expect(
                clippy::cast_precision_loss,
                reason = "indices 0..3 convertidos para posicao"
            )]
            pos.push([i as f32 * 0.5, j as f32 * 0.5, 0.0]);
        }
    }
    let idx = |i: usize, j: usize| u32::try_from(j * 3 + i).expect("grelha pequena");
    let mut tris: Vec<[u32; 3]> = Vec::new();
    let mut faces: Vec<Face> = Vec::new();
    for j in 0..2 {
        for i in 0..2 {
            for t in [
                [idx(i, j), idx(i + 1, j), idx(i + 1, j + 1)],
                [idx(i, j), idx(i + 1, j + 1), idx(i, j + 1)],
            ] {
                tris.push(t);
                faces.push(Face::tri(t[0], t[1], t[2]));
            }
        }
    }
    let n = tris.len();
    let mesh = Mesh::from_parts(pos.clone(), faces).expect("a fixtura e' construida aqui");
    let cut = CutMesh {
        origin: vec![(0..9u32).collect()],
        tris: vec![tris],
        tri_face: vec![(0..u32::try_from(n).expect("poucos")).collect()],
        seams: Vec::new(),
    };
    let map = GridMap {
        uv: vec![pos.iter().map(|p| [p[0], p[1]]).collect()],
        shift: Vec::new(),
    };
    (mesh, cut, map)
}

/// ⭐⭐⭐ **GATE — desfaz a dobra E NÃO MEXE NA FRONTEIRA.**
///
/// ⛔ **A segunda metade é a que protege a propriedade `GP`**: as transições de carta vivem nas
/// fronteiras dos retalhos, e um passe que as mexesse desfaria a obra de 24/08 em silêncio — com
/// todas as réguas de dobra a melhorar.
#[test]
fn desfaz_a_dobra_e_a_fronteira_do_retalho_nao_se_mexe() {
    let (mesh, cut, mut map) = fixtura();
    map.uv[0][4] = [1.4, 1.4]; // o centro para fora ⇒ dobras
    let antes = map.uv[0].clone();

    let rep = untangle_patches(&mesh, &cut, &mut map);

    // ⛔ CONTROLE: a fixtura tinha MESMO dobras.
    assert!(
        rep.before > 0,
        "⛔ a fixtura tem de conter dobras, senao este gate nao prova nada"
    );
    assert_eq!(rep.patches, 1, "⛔ um retalho com dobra");
    assert_eq!(rep.after, 0, "⛔ sobraram {} dobras", rep.after);
    assert_eq!(rep.gave_up, 0);

    // ⭐⭐ E a fronteira — os oito de fora — está BYTE a byte onde estava.
    for v in [0usize, 1, 2, 3, 5, 6, 7, 8] {
        assert!(
            map.uv[0][v] == antes[v],
            "⛔ o vertice de FRONTEIRA {v} mexeu-se: {:?} -> {:?}",
            antes[v],
            map.uv[0][v]
        );
    }
    // ⛔ E o CONTROLE do controlo: o interior TEM de se ter mexido.
    assert!(
        map.uv[0][4] != antes[4],
        "⛔ o interior nao se mexeu -- o gate da fronteira ficaria vacuo"
    );
}

/// ⭐⭐⭐ **GATE — um mapa SEM dobras sai BYTE A BYTE igual.**
///
/// ⛔ *Este passe não existe para melhorar um mapa bom; existe para desfazer uma dobra.* Sem esta
/// cerca ele mexeria toda peça limpa do corpus para baixar a energia — e todo *golden* desta
/// cadeia mudaria de valor por uma razão que ninguém pediu.
#[test]
fn um_mapa_sem_dobras_sai_byte_a_byte_igual() {
    let (mesh, cut, mut map) = fixtura();
    let antes = map.uv.clone();
    let rep = untangle_patches(&mesh, &cut, &mut map);
    assert_eq!(
        rep,
        UntangleReport::default(),
        "⛔ nada a fazer tem de sair como relatorio VAZIO"
    );
    assert!(map.uv == antes, "⛔ o mapa limpo tem de sair intacto");
}

/// ⭐⭐ **GATE — `before == 0` é «não havia dobra», e não «não medido».**
///
/// ⚠️ A diferença importa no log: um `0` que significasse «não corri» leria como aprovação. O
/// passe corre sempre e **conta antes de decidir se mexe**.
#[test]
fn zero_e_sem_dobra_e_nao_sem_medir() {
    let (mesh, cut, mut map) = fixtura();
    let rep = untangle_patches(&mesh, &cut, &mut map);
    assert_eq!(rep.before, 0);
    assert_eq!(
        rep.patches, 0,
        "⛔ nenhum retalho ENTROU, porque nenhum tinha dobra"
    );

    // ⭐ E com dobra o mesmo campo conta — logo `0` não pode ser confundido com «não olhou».
    map.uv[0][4] = [1.4, 1.4];
    let rep = untangle_patches(&mesh, &cut, &mut map);
    assert!(rep.before > 0 && rep.patches == 1);
}

/// ⭐⭐⭐ **GATE — o passe nasce DESLIGADO, e a tabela da recusa mora ao lado dele.**
///
/// ⛔ **Ele melhora TODAS as colunas de forma e mesmo assim não shipa ligado** — ver a tabela
/// em [`super::enabled`]: custa `+56 %` do relógio do artista, não move o defeito que o dono
/// fotografou, e piora duas colunas. *Uma melhoria que o dono não vê, paga com metade de um
/// relógio que ele vê, é uma troca — e a troca é dele.*
///
/// ⚠️ **Este gate existe para que ligá-lo seja uma DECISÃO e não uma deriva:** quem inverter o
/// default tem de vir aqui, e a tabela está à distância de um salto.
#[test]
fn nasce_desligado_e_a_tabela_da_recusa_esta_ao_lado() {
    let posta = std::env::var("PH2D_GRIDMAP_UNTANGLE").ok();
    assert!(
        posta.is_none(),
        "⛔ este gate mede o DEFAULT; corre-o sem a env posta"
    );
    assert!(!super::enabled(), "⛔ o passe nasce DESLIGADO");

    // ⭐ E a razão tem de continuar escrita: um default invertido em silêncio é o que este
    // gate impede, e um doc sem os números seria um default sem razão.
    let src = include_str!("untangle_pass.rs");
    for numero in ["21,4 s", "33,4 s", "`27`", "`21`"] {
        assert!(
            src.contains(numero),
            "⛔ a tabela da recusa perdeu {numero} -- o default ficaria sem razao escrita"
        );
    }
}
