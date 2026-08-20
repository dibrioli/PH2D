//! Os gates da navegação da janela 3D de modelagem.
//!
//! ⚠️ **Nenhum destes afirma um sinal.** A pergunta que o artista faz é *"o modelo segue a minha
//! mão?"*, e é essa que se mede — traçando a peça e olhando **onde ela ficou na tela**. Argumentar
//! sobre `yaw += dx` é literalmente como o erro entrou na linha irmã, nos dois eixos de uma vez.

use super::law;
use ph2d_field::{FieldDoc, Node, NodeId, NodeKind, Primitive, Xform};
use ph2d_field_render::{Orbit, trace_with};

const W: u32 = 120;
const H: u32 = 120;

/// Uma esfera pequena **do lado de cá** da origem.
///
/// ⚠️ A posição não é decorativa: com a câmera de frente (`yaw = 0`), um ponto em `+Z` projeta-se no
/// **centro** do quadro, e é justamente por isso que ele serve — qualquer giro tira-o do centro, e o
/// lado para onde ele sai responde à pergunta sem ambiguidade. Uma esfera em `+X` já começaria
/// deslocada e giraria *para dentro*, que é a armadilha desta medição.
fn nose() -> FieldDoc {
    FieldDoc::new(
        vec![Node {
            xform: Xform::at(0.0, 0.0, 0.45),
            kind: NodeKind::Leaf(Primitive::Sphere { radius: 0.12 }),
        }],
        NodeId(0),
    )
    .expect("esfera")
}

fn front() -> Orbit {
    Orbit::from_yaw_pitch(0.0, 0.0)
}

/// O centro de massa da peça na tela, em pixels — `None` se ela não aparece.
///
/// ⚠️ **Sem anti-serrilhado de propósito**: o que se mede aqui é para onde a forma foi, e a máscara
/// crua responde a isso sem uma segunda variável no meio.
fn centroid(doc: &FieldDoc, cam: &Orbit) -> Option<(f32, f32)> {
    let g = trace_with(doc, cam, W, H, true, false);
    let (mut sx, mut sy, mut n) = (0.0f64, 0.0f64, 0usize);
    for i in 0..(W as usize) * (H as usize) {
        if g.hit[i] {
            sx += (i % W as usize) as f64;
            sy += (i / W as usize) as f64;
            n += 1;
        }
    }
    (n > 0).then(|| ((sx / n as f64) as f32, (sy / n as f64) as f32))
}

/// ⭐ **O modelo segue a mão** — nos dois eixos, medido na tela.
#[test]
fn dragging_right_turns_the_model_right_and_dragging_down_shows_its_top() {
    let doc = nose();
    let base = centroid(&doc, &front()).expect("a peça aparece de frente");
    // De frente, a esfera em +Z cai no centro do quadro.
    assert!(
        (base.0 - W as f32 * 0.5).abs() < 2.0,
        "de frente a peça devia estar centrada em x, e está em {}",
        base.0
    );

    let mut right = front();
    law::orbit(&mut right, 40.0, 0.0);
    let moved = centroid(&doc, &right).expect("a peça continua no quadro");
    assert!(
        moved.0 > base.0 + 5.0,
        "arrastar para a DIREITA tem de levar o modelo para a direita: {} -> {}",
        base.0,
        moved.0
    );

    let mut down = front();
    law::orbit(&mut down, 0.0, 40.0);
    let tipped = centroid(&doc, &down).expect("a peça continua no quadro");
    assert!(
        tipped.1 > base.1 + 5.0,
        "arrastar para BAIXO tem de mostrar o TOPO — a face de cá desce na tela: {} -> {}",
        base.1,
        tipped.1
    );
}

/// **O pan leva a peça para onde a mão vai**, e o passo é em fração de tela: o mesmo arrasto move o
/// mesmo tanto de modelo em qualquer zoom.
#[test]
fn panning_carries_the_model_with_the_hand_at_any_zoom() {
    let doc = nose();
    let base = centroid(&doc, &front()).expect("a peça aparece");

    let mut moved = front();
    law::pan(&mut moved, 20.0, 12.0, H as f32 * 0.5);
    let after = centroid(&doc, &moved).expect("a peça continua no quadro");
    assert!(
        after.0 > base.0 + 5.0 && after.1 > base.1 + 3.0,
        "a peça tem de andar com a mão: ({}, {}) -> ({}, {})",
        base.0,
        base.1,
        after.0,
        after.1
    );

    // ⭐ E o deslocamento em PIXELS é o mesmo com outro zoom — é isso que "fração de tela"
    // significa, e o que impede o pan de ficar inútil de perto e insano de longe.
    let mut near = front();
    near.half_extent = 0.2;
    let base_near = centroid(&doc, &near).expect("a peça aparece de perto");
    let mut near_moved = near;
    law::pan(&mut near_moved, 20.0, 12.0, H as f32 * 0.5);
    let after_near = centroid(&doc, &near_moved).expect("a peça continua no quadro");
    let (a, b) = (after.0 - base.0, after_near.0 - base_near.0);
    assert!(
        (a - b).abs() < 2.0,
        "o mesmo arrasto tem de mover os mesmos pixels em qualquer zoom: {a:.1} contra {b:.1}"
    );
}

/// **A roda aproxima**, e a peça cresce na tela.
#[test]
fn the_wheel_zooms_in_and_the_part_grows() {
    let doc = nose();
    let cam = front();
    let before = trace_with(&doc, &cam, W, H, true, false).hits();
    let mut closer = cam;
    law::zoom(&mut closer, 3.0);
    let after = trace_with(&doc, &closer, W, H, true, false).hits();
    assert!(
        after > before,
        "três passos de roda para a frente têm de APROXIMAR: {before} -> {after} pixels"
    );
    assert!(
        closer.half_extent < cam.half_extent,
        "aproximar é reduzir o enquadramento"
    );
}

/// ⚠️ **Os limites de zoom existem e saturam** — e são os únicos dois números de faixa deste
/// arquivo, cada um com o recurso de que é (ver as constantes).
#[test]
fn the_zoom_saturates_instead_of_running_away() {
    let mut cam = front();
    for _ in 0..500 {
        law::zoom(&mut cam, 10.0);
    }
    assert!(
        cam.half_extent > 0.0 && cam.half_extent.is_finite(),
        "aproximar sem parar não pode chegar a zero nem a NaN: {}",
        cam.half_extent
    );
    let floor = cam.half_extent;
    let mut out = front();
    for _ in 0..500 {
        law::zoom(&mut out, -10.0);
    }
    assert!(
        out.half_extent.is_finite() && out.half_extent > floor,
        "afastar sem parar tem de saturar acima do piso: {}",
        out.half_extent
    );
}

/// ⭐ **A rotação é LIVRE: não há polo onde o gesto morra.**
///
/// Este gate substitui o que prendia a elevação em ±90°, e a troca é o registo de um veredito de
/// produto: *"só rotaciona em uma direção. não tem rot livre"* (Enio, 2026-08-19). A câmera de dois
/// ângulos batia na parede ao fim de ~105 px de arrasto para baixo — e a cura não foi um limite
/// maior, foi trocar a representação (`ph2d_field_render::Orbit`).
///
/// # ⚠️ A primeira versão deste gate mediu a coisa errada, e vale registar
///
/// Ela seguia o **centro de massa da peça na tela** e chamava "polo" a qualquer passo em que ele
/// não se mexia. Reprovou com **1 parada em 200** — e a parada era real e não era um polo: o
/// centroide de um ponto que gira num plano descreve uma senóide, e uma senóide tem **pontos de
/// retorno**, onde a velocidade é zero por geometria. *O sintoma que se procura e o sintoma que se
/// mede podem ter a mesma forma por motivos diferentes.*
///
/// O que se afirma agora é **exato**: cada chamada é uma rotação de `|dy|·k` em torno de um eixo
/// local fixo, logo a direção da vista tem de virar **exatamente** isso — nos 400 passos, que são
/// mais de uma volta inteira.
#[test]
fn vertical_dragging_never_hits_a_wall() {
    const DY: f32 = 2.0;
    // A lei da casa: 0,01 rad por pixel (ver `ORBIT_RAD_PER_PX`).
    const EXPECTED: f32 = DY * 0.01;
    let mut cam = front();
    let (_, _, mut prev) = cam.basis();
    let mut worst = f32::INFINITY;
    for _ in 0..400 {
        law::orbit(&mut cam, 0.0, DY);
        let (_, _, fwd) = cam.basis();
        let dot = (0..3)
            .map(|k| prev[k] * fwd[k])
            .sum::<f32>()
            .clamp(-1.0, 1.0);
        worst = worst.min(dot.acos());
        prev = fwd;
    }
    assert!(
        worst > EXPECTED * 0.99,
        "algum passo vertical virou só {worst} rad em vez de {EXPECTED} — isso é o polo da câmera \
         de dois ângulos, e é exatamente o que se removeu"
    );
}

/// **A tecla de repor devolve o enquadramento inicial** — a volta que a rotação livre torna
/// necessária, porque ela inclina o horizonte de propósito.
#[test]
fn home_restores_the_opening_view() {
    let home = Orbit::default();
    let mut cam = home;
    law::orbit(&mut cam, 137.0, -89.0);
    law::zoom(&mut cam, 6.0);
    law::pan(&mut cam, 50.0, -30.0, 60.0);
    assert!(cam != home, "o teste tem de partir de uma vista MEXIDA");

    law::home(&mut cam);
    assert_eq!(
        cam, home,
        "repor tem de devolver exatamente o enquadramento de abertura"
    );
}
