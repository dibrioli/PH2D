//! ⭐⭐⭐ **O FILTRO DE TECIDO — os portões do GESTO**, irmão do [`cloth_tests`].
//!
//! O corte entre os dois é o que cada um afirma: lá *a LEI do pano sob um
//! carimbo*, aqui *o que muda quando o pano deixa de ter pincel* — a peça
//! inteira, o ponto congelado, e a diferença de gesto que separa este filtro do
//! [`super::stroke_filter`].
//!
//! ⚠️ **Nenhum destes gates tem lado APROVADO**: o oráculo nunca correu o filtro
//! (as `86` fixtures são todas do traço). Eles afirmam propriedades da espec §7
//! e da nossa costura, e dizem isso de si mesmos. *Uma barra calibrada sem o lado
//! aprovado mediria os nossos próprios defeitos.*

use super::cloth_tests::plano;
use super::{Brush, ClothFilterStep, SculptStroke, Verb};
use crate::ClothFilterKind;

fn passo(s: f32) -> ClothFilterStep {
    ClothFilterStep {
        s,
        gravity_axis: [0.0, 0.0, -1.0],
        frame: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        axes: [true; 3],
        eye: [0.0, 0.0, 1.0],
    }
}

fn pincel() -> Brush {
    Brush {
        verb: Verb::Cloth,
        radius: 0.35,
        strength: 1.0,
        ..Brush::default()
    }
}

/// Corre `n` passos do filtro e devolve a malha resultante.
fn corre(kind: ClothFilterKind, s: f32, n: usize, ponto: [f32; 3]) -> Vec<[f32; 3]> {
    let mut mesh = plano();
    let b = pincel();
    let mut st = SculptStroke::default();
    st.cloth_filter_begin(&mesh, &b, kind, ponto);
    for _ in 0..n {
        st.cloth_filter_step(&mut mesh, kind, &passo(s));
    }
    mesh.positions().to_vec()
}

fn desvio(a: &[[f32; 3]], b: &[[f32; 3]]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(p, q)| {
            ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2) + (p[2] - q[2]).powi(2)).sqrt()
        })
        .fold(0.0, f32::max)
}

/// ⭐⭐⭐ **UM FILTRO TOCA A PEÇA INTEIRA; UM CARIMBO TOCA UM DISCO.**
///
/// É a afirmação que faz do filtro uma ferramenta diferente e não um pincel
/// grande: a espec §7 diz *todas as células não mascaradas, raio infinito, sem
/// banda*, e o observável disso é a contagem de vértices que se movem.
#[test]
fn um_filtro_toca_a_peca_inteira_e_um_carimbo_toca_um_disco() {
    let mut mesh = plano();
    let n = mesh.vert_count();
    let b = pincel();
    let mut st = SculptStroke::default();
    st.cloth_filter_begin(&mesh, &b, ClothFilterKind::Gravity, [0.0; 3]);
    let movidos = st.cloth_filter_step(&mut mesh, ClothFilterKind::Gravity, &passo(1.0));
    println!("filtro: {movidos} de {n} vertices movidos");
    assert!(movidos > 0, "o filtro nao moveu nada -- a fixtura nao produz o fenomeno");
    assert_eq!(
        movidos, n,
        "o filtro tinha de alcancar a peca INTEIRA (espec §7), e alcancou {movidos} de {n}"
    );
}

/// ⭐⭐⭐ **O FILTRO DE TECIDO ACUMULA; O DE MALHA NÃO.**
///
/// É a diferença de GESTO entre os dois, e ela é load-bearing: o
/// [`super::stroke_filter`] repõe a pose congelada a cada passo (dois passos com
/// a mesma força dão o MESMO resultado — voltar com o dedo desfaz), e a espec §7
/// manda o filtro de tecido correr **um passo de simulação por movimento do
/// rato**. ⛔ Se alguém puser um `restore_frozen_pose` aqui, o pano deixa de cair
/// e este gate reprova.
#[test]
fn o_filtro_de_tecido_acumula_em_vez_de_repor_a_pose() {
    let um = corre(ClothFilterKind::Gravity, 1.0, 1, [0.0; 3]);
    let tres = corre(ClothFilterKind::Gravity, 1.0, 3, [0.0; 3]);
    let base = plano().positions().to_vec();
    let d1 = desvio(&base, &um);
    let d3 = desvio(&base, &tres);
    println!("um passo move {d1:.6}; tres passos movem {d3:.6}");
    assert!(d1 > 0.0, "um passo tinha de mover -- a fixtura nao produz o fenomeno");
    assert!(
        d3 > d1 * 1.5,
        "tres passos moveram {d3:.6} contra {d1:.6} de um -- o filtro esta' a REPOR a pose \
         em vez de simular, e um solver de tecido reposto e' um filtro de malha caro"
    );
}

/// ⭐⭐ **SÓ O APERTO LÊ O PONTO CONGELADO** — os outros quatro tipos dão o mesmo
/// bloco de vértices, ao bit, com o cursor noutro sítio.
///
/// ⚠️ **A régua é o produto, não o código:** o [`ClothFilterKind::le_o_ponto`]
/// afirma-o, e este gate mede-o correndo cada tipo com dois pontos distintos.
/// *Um censo que só lê a tabela que ele próprio afirma não é um censo.*
#[test]
fn so_o_aperto_le_o_ponto_congelado() {
    for kind in ClothFilterKind::ALL {
        let a = corre(kind, 1.0, 2, [0.0, 0.0, 0.0]);
        let b = corre(kind, 1.0, 2, [0.7, 0.4, 0.0]);
        let d = desvio(&a, &b);
        println!("{:<8} com o ponto noutro sitio: desvio {d:.6}", kind.label());
        if kind.le_o_ponto() {
            assert!(
                d > 0.0,
                "{:?} diz LER o ponto e nao mudou com ele",
                kind
            );
        } else {
            assert_eq!(
                d, 0.0,
                "{:?} diz NAO ler o ponto e mudou com ele (desvio {d:.6})",
                kind
            );
        }
    }
}

/// ⭐⭐ **CADA UM DOS CINCO MOVE A PEÇA** — o gate que impede um tipo de nascer
/// mudo, que é o defeito que esta casa varre a cada wave.
#[test]
fn os_cinco_tipos_movem_a_peca() {
    let base = plano().positions().to_vec();
    for kind in ClothFilterKind::ALL {
        let out = corre(kind, 1.0, 2, [0.0; 3]);
        let d = desvio(&base, &out);
        println!("{:<8} move {d:.6}", kind.label());
        assert!(
            d > 0.0,
            "{:?} nao moveu um vertice -- um tipo mudo e' pior que um tipo ausente",
            kind
        );
    }
}

/// ⭐ **SEM PEN-DOWN O PASSO É UM NO-OP EXACTO** — e a guarda é DERIVADA (há
/// sessão? a captura cobre a malha?), nunca um flag: dois campos a dizerem
/// *«estou em modo filtro»* podem discordar, e nesse dia o filtro correria sobre
/// o `pre` de outro gesto.
#[test]
fn sem_pen_down_o_passo_nao_toca_a_peca() {
    let mut mesh = plano();
    let antes = mesh.positions().to_vec();
    let mut st = SculptStroke::default();
    assert!(!st.cloth_filter_running());
    let movidos = st.cloth_filter_step(&mut mesh, ClothFilterKind::Gravity, &passo(1.0));
    assert_eq!(movidos, 0, "o passo correu sem pen-down");
    assert_eq!(desvio(&antes, mesh.positions()), 0.0, "a peca mexeu-se sem pen-down");
}

/// ⭐⭐ **O UNDO COBRE A PEÇA INTEIRA, e ele é o do TRAÇO.**
///
/// A espec §7 pede *um passo por uso do filtro*, e é exactamente o que o
/// `close_stroke` dá: ele grava `touched` + as posições congeladas. ⛔ Uma porta
/// de undo própria seria a segunda resposta a *«como se desfaz um punhado de
/// vértices deslocados»* — o mesmo argumento que fez o transform reusar a do
/// traço.
#[test]
fn o_pen_down_do_filtro_congela_a_peca_inteira_para_o_undo() {
    let mesh = plano();
    let n = mesh.vert_count();
    let mut st = SculptStroke::default();
    st.cloth_filter_begin(&mesh, &pincel(), ClothFilterKind::Gravity, [0.0; 3]);
    assert_eq!(
        st.touched().len(),
        n,
        "o pen-down do filtro tem de congelar a malha INTEIRA, senao o Ctrl+Z devolve metade"
    );
    assert!(st.cloth_filter_running());
}

/// ⭐ **O `s = 0` NÃO MEXE A PEÇA** — o controlo que todo gate de força precisa,
/// e a promessa que faz do arrasto uma recta: no ponto em que a mão não andou,
/// nada acontece.
///
/// ⚠️ **Ele NÃO é vácuo:** o solver corre à mesma (as restrições relaxam), e o
/// que se afirma é que sem força externa a malha de repouso é ponto fixo dele.
#[test]
fn com_arrasto_zero_a_peca_fica_parada() {
    let base = plano().positions().to_vec();
    for kind in ClothFilterKind::ALL {
        let out = corre(kind, 0.0, 3, [0.0; 3]);
        let d = desvio(&base, &out);
        println!("{:<8} com s = 0 move {d:.3e}", kind.label());
        assert_eq!(d, 0.0, "{:?} mexeu a peca com arrasto zero", kind);
    }
}

/// ⭐⭐ **O CENSO dos cinco contra os arms da lei** — quem escreve `σ` é quem a
/// tabela diz que é.
#[test]
fn so_a_escala_e_de_ancora_entre_os_cinco() {
    let de_ancora: Vec<_> = ClothFilterKind::ALL
        .into_iter()
        .filter(|k| super::stroke_cloth_filter::e_de_ancora(*k))
        .collect();
    assert_eq!(
        de_ancora,
        vec![ClothFilterKind::Scale],
        "a particao dos arms mudou -- releia a espec §7 antes de mexer no selector"
    );
}
