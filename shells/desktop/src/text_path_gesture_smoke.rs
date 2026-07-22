//! **A cena do GESTO do Text on Path** — `PH2D_BUILD_SMOKE=23`.
//!
//! Irmã da cena 22 (que mostra o MOTOR, montando a geometria programaticamente) e módulo irmão
//! pelo teto de LOC. A diferença é o sujeito: **aqui quem faz é o artista.**
//!
//! A 22 prova que o texto sabe cavalgar uma curva. Esta prova que ele **chega lá pelo painel** —
//! e essas duas coisas falham por motivos completamente diferentes. Um vínculo perfeito atrás de
//! um botão que não despacha é indistinguível de uma feature quebrada.
//!
//! # O roteiro (impresso no terminal)
//!
//! A cena deixa a mesa posta: um texto e uma curva, **os dois selecionados**, com a ferramenta
//! Vector ativa. O painel Vector já deve mostrar a seção **Text on Path** com um botão.
//!
//! 1. **Text on Path** — o texto salta para cima da curva.
//! 2. **Offset** — arrastar o slider faz o texto correr ao longo dela.
//! 2b. **A alça** (W5) — no modo Node, uma bolinha marca onde o texto começa; arrastá-la corre
//!     o texto pela curva (a manipulação direta do Offset).
//! 3. **Side: Other side** — o texto passa para o outro lado, a ler ao contrário.
//! 4. Mova a **CURVA** com o gizmo — o texto vai junto.
//! 5. Tente mover o **TEXTO** — ele não sai do lugar, de propósito: mover um texto preso não
//!    quer dizer nada; o que se move é o caminho (é o que o conector já faz, e o Illustrator).
//! 6. **Detach from Path** — o texto volta a ser texto reto e **a curva fica**.

use ph2d_ecs::VecShape;
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecPathId, VecVertex, VertexKind};

use crate::vec_text::VecTextEdit;

/// Tamanho do texto em unidades de MUNDO.
const SIZE: f64 = 0.55;

/// A palavra. Curta o bastante para caber com folga na curva — um texto que já nasce truncado
/// leria como bug antes de o artista clicar em coisa nenhuma.
const WORD: &str = "RIDE THE CURVE";

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // Só no frame seguinte: é o `sync` do render loop que dá entidade ao path, e sem
        // entidade não há `VecShape::Text` a pendurar nem seleção a fazer (a mesma razão da
        // cena 21).
        4 => arm(app),
        _ => {}
    }
}

/// O texto (pelo caminho REAL da ferramenta) + a curva que ele vai cavalgar.
fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));

    let mut edit = VecTextEdit {
        origin: [-2.4, -2.2],
        size: SIZE,
        weight: 700.0,
        line_height: 1.2,
        tracking: 0.0,
        align: ph2d_tool_vector::TextAlign::Left,
        extra_axes: Vec::new(),
        family: None,
        fill: Some(Paint::solid(Rgba8::new(90, 150, 220, 255))),
        stroke: None,
        text: WORD.to_owned(),
        id: None,
        center: [0.0, 0.0],
    };
    crate::vec_text::regen_into(&mut gfx.vec_scene, &mut edit);

    // A curva: um arco largo e raso, onde a rotação por-glyph se vê sem a palavra se dobrar
    // sobre si mesma.
    let guide = gfx.vec_scene.push_path(VecPath {
        verts: arc(),
        closed: false,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(120, 120, 130, 255), 0.02)),
        ..Default::default()
    });
    PENDING.lock().expect("smoke lock").replace((edit, guide));
}

/// O texto montado no frame 3 + o guia, à espera das entidades que o `sync` lhes dá.
static PENDING: std::sync::Mutex<Option<(VecTextEdit, VecPathId)>> = std::sync::Mutex::new(None);

/// Pendura o `VecShape::Text` (sem ele o painel não trata isto como texto) e seleciona OS DOIS.
fn arm(app: &mut crate::App) {
    let Some((edit, guide)) = PENDING.lock().expect("smoke lock").take() else {
        return;
    };
    let Some(id) = edit.id else { return };
    let Some(gfx) = app.gfx.as_mut() else { return };
    crate::vec_text_object::upsert_text_shape(&mut gfx.sim, &app.vec_entities, &edit);
    let is_text = app
        .vec_entities
        .get(&id)
        .and_then(|&b| {
            gfx.sim
                .world()
                .get::<VecShape>(ph2d_ecs::Entity::from_bits(b))
        })
        .is_some();
    // Os DOIS: é a seleção que o gesto exige, e a cena existe para a pôr pronta.
    app.vec_pen.select_many(&[id, guide]);
    let can = crate::vec_text_ride::link_candidate(
        &gfx.sim,
        &app.vec_entities,
        app.vec_pen.selected_paths(),
    )
    .is_some();
    eprintln!(
        "[smoke] W4 text on path -- a mesa esta' posta: o texto \"{WORD}\" e uma curva, os DOIS \
         selecionados (objeto de texto: {is_text} · gesto oferecido: {can}).\n\
         [smoke]   1. No painel Vector, secao TEXT ON PATH -> clique \"Text on Path\".\n\
         [smoke]      O texto tem de SALTAR para cima da curva.\n\
         [smoke]   2. Arraste o Offset -- o texto corre ao longo dela.\n\
         [smoke]   3. Side: \"Other side\" -- passa para o outro lado, a ler ao contrario.\n\
         [smoke]   4. (W5) Entre no modo NODE (a seta branca). Uma BOLINHA aparece onde o texto\n\
         [smoke]      COMECA na curva -- arraste-a: o texto corre pela curva (o mesmo que o\n\
         [smoke]      Offset, direto na tela). Fora do Node ela nao aparece.\n\
         [smoke]   5. Mova a CURVA com o gizmo -- o texto vai junto.\n\
         [smoke]   6. Tente mover o TEXTO -- ele NAO sai do lugar. E' de proposito: mover um\n\
         [smoke]      texto preso nao quer dizer nada, o que se move e' o caminho.\n\
         [smoke]   7. \"Detach from Path\" -- volta a ser texto reto, e a CURVA fica."
    );
    if !is_text || !can {
        eprintln!(
            "[smoke] !! a mesa NAO esta' posta (texto: {is_text}, gesto: {can}) -- o painel nao \
             vai oferecer o botao, e o resto do smoke nao significa nada. PARE e reporte."
        );
    }
}

/// Um arco largo e raso, aberto, da esquerda para a direita.
fn arc() -> Vec<VecVertex> {
    let pt = |x: f64, y: f64, hx: f64| VecVertex {
        anchor: [x, y],
        in_handle: [x - hx, y],
        out_handle: [x + hx, y],
        kind: VertexKind::Smooth,
        corner_radius: 0.0,
    };
    vec![pt(-3.4, 0.0, 1.1), pt(0.0, 1.7, 1.4), pt(3.4, 0.0, 1.1)]
}
