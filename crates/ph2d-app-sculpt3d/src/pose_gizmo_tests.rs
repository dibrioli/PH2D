//! Os gates da FIGURA do osso — ver [`super::pose_gizmo`].
//!
//! ⚠️ **A cena exige um `wgpu::Device`**, então o que se pode afirmar aqui é o
//! que vive em funções livres: a **forma** que o artista lê. O que a cena
//! decide (de que peça é o osso, que a cache é a do traço) está gateado um
//! nível abaixo, em `ph2d-sculpt3d`, onde não é preciso janela.

use super::{anel, silhueta_do_osso};

/// Os pontos do caminho, em pares.
fn pontos(caminho: &ph2d_vector::BezPath) -> Vec<(f64, f64)> {
    use ph2d_vector::PathEl;
    caminho
        .elements()
        .iter()
        .filter_map(|e| match e {
            PathEl::MoveTo(p) | PathEl::LineTo(p) => Some((p.x, p.y)),
            _ => None,
        })
        .collect()
}

/// ⭐⭐ **A figura diz DE QUE LADO está a dobradiça, e é a única coisa que ela
/// tem de dizer antes de o artista arrastar.**
///
/// Uma linha entre os dois pontos seria simétrica e não diria nada; a silhueta
/// de uma armadura é **larga junto da raiz e afila para a mão**. ⚠️ Este gate
/// mede a assimetria, não a grafia: ele reprova tanto um losango centrado como
/// um virado ao contrário.
#[test]
fn a_silhueta_e_mais_larga_junto_da_dobradica() {
    let caminho = silhueta_do_osso(0.0, 0.0, 100.0, 0.0).expect("um osso de 100 px");
    let p = pontos(&caminho);
    assert_eq!(p.len(), 4, "a silhueta tem quatro cantos: {p:?}");
    // Os dois cantos largos são os que saem do eixo.
    let largos: Vec<(f64, f64)> = p.iter().copied().filter(|(_, y)| y.abs() > 1e-9).collect();
    assert_eq!(largos.len(), 2, "dois cantos fora do eixo: {p:?}");
    let x_do_ombro = largos[0].0;
    assert!(
        (largos[1].0 - x_do_ombro).abs() < 1e-9,
        "os dois cantos largos tem de estar a mesma altura do osso: {largos:?}"
    );
    assert!(
        x_do_ombro < 50.0,
        "o ombro ficou em x={x_do_ombro} de 100 — a figura e' simetrica ou esta' \
         virada, e deixa de dizer onde esta' a articulacao"
    );
    // E ele não pode colar-se à dobradiça, senão a figura vira um triângulo.
    assert!(
        x_do_ombro > 1.0,
        "o ombro colou na dobradica (x={x_do_ombro}) — a figura vira um triangulo"
    );
}

/// ⚠️ **Um osso curto demais não desenha figura nenhuma.**
///
/// Abaixo do piso os dois flancos pousam na mesma coluna de pixels (o traço do
/// overlay mede `1,5 px`) e o losango degenera na linha que este módulo existe
/// para não desenhar. ⛔ *Desenhar mesmo assim uma figura com uma normal
/// arbitrária inventa uma articulação que a cadeia não tem.*
#[test]
fn um_osso_curto_demais_nao_desenha_figura() {
    assert!(silhueta_do_osso(10.0, 10.0, 10.0, 10.0).is_none(), "zero");
    assert!(
        silhueta_do_osso(10.0, 10.0, 12.0, 10.0).is_none(),
        "2 px ainda e' uma linha"
    );
    // E o controlo positivo: um pouco acima do piso ele desenha.
    assert!(
        silhueta_do_osso(10.0, 10.0, 40.0, 10.0).is_some(),
        "30 px tem de desenhar — sem isto o gate acima passaria com a funcao a \
         devolver `None` sempre"
    );
}

/// ⚠️ **A cintura tem um PISO em pixels**, e é ele que impede um osso curto
/// (mas desenhável) de se ler como um traço.
#[test]
fn a_cintura_tem_piso_em_pixels() {
    let curto = silhueta_do_osso(0.0, 0.0, 6.0, 0.0).expect("6 px desenha");
    let meia_largura = pontos(&curto)
        .iter()
        .map(|(_, y)| y.abs())
        .fold(0.0f64, f64::max);
    assert!(
        meia_largura >= 2.0,
        "a cintura de um osso de 6 px ficou em {meia_largura} px — abaixo de 2 \
         os dois flancos pousam na mesma coluna de pixels"
    );
}

/// O anel fecha e tem o raio pedido.
#[test]
fn o_anel_fecha_no_raio_pedido() {
    let a = anel(100.0, 50.0, 6.0);
    for (x, y) in pontos(&a) {
        let r = (x - 100.0).hypot(y - 50.0);
        assert!((r - 6.0).abs() < 1e-6, "ponto a {r} do centro");
    }
}
