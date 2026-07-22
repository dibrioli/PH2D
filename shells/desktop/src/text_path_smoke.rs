//! **A cena da W3 — o texto cavalga o caminho** (`PH2D_BUILD_SMOKE=22`).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `text_fx_smoke`.
//!
//! # O que esta cena mostra
//!
//! A W3 é o motor, não a UI (ligar um texto a um caminho pelo painel é a W4) — então sem
//! esta cena a wave inteira seria invisível, e "invisível" e "quebrado" têm a mesma cara.
//! Aqui o texto é montado programaticamente sobre três caminhos, ao lado dos caminhos que
//! ele cavalga, para o desenho poder ser JULGADO em vez de deduzido dos gates.
//!
//! 1. **A onda** — as letras giram com a curva, ficam de pé, e o espaçamento entre elas não
//!    aperta nem abre ao passar pelas dobras (é o comprimento de ARCO que conta, não o `t`).
//! 2. **O círculo de cima** — duas linhas cobrindo ~70% da volta; a de baixo corre paralela
//!    à de cima, à distância exata da entrelinha, curvando junto.
//! 3. **O círculo virado** (`flip`) — o mesmo caminho, texto do outro lado, lendo ao
//!    contrário. É a metade que um erro de sinal só na tangente (e não na normal) deixaria
//!    do lado certo mas a ler para trás.
//!
//! # O que julgar
//!
//! - Nenhuma letra deitada, esticada ou cisalhada — o referencial é uma ROTAÇÃO.
//! - Nenhuma letra empilhada na ponta de um caminho.
//! - Na onda, as letras seguem a inclinação da curva ponto a ponto (não o ângulo médio).

use ph2d_tool_vector::TextAlign;
use ph2d_vec_scene::arc_path::ArcPath;
use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, VecScene, VecVertex, VertexKind};

use crate::vec_glyph::{TextLayout, TextPlacement, text_to_compound_path};

/// Tamanho do texto em unidades de MUNDO — grande o bastante para a rotação por-glyph ser
/// legível a olho, que é o ponto inteiro do smoke.
const SIZE: f64 = 0.42;

/// O eixo variável da fonte — o mesmo nas duas metades (desenho e gate).
const AXES: [(ph2d_vector_font::AxisTag, f32); 1] =
    [(ph2d_vector_font::AxisTag::new(*b"wght"), 700.0)];

/// Uma cena: o caminho, o que se escreve nele, e de que lado.
struct Ride {
    name: &'static str,
    verts: Vec<VecVertex>,
    closed: bool,
    text: &'static str,
    start_offset: f64,
    flip: bool,
}

/// As três cenas, numa tabela só — o `frame` DESENHA e o gate MEDE a partir dela, senão o
/// gate garante uma cena que não é a que o artista vê.
fn scenes() -> Vec<Ride> {
    vec![
        Ride {
            name: "onda",
            verts: wave_contour(),
            closed: false,
            text: "TEXT ON A PATH",
            start_offset: 0.15,
            flip: false,
        },
        Ride {
            name: "círculo de cima",
            verts: circle_contour([0.0, 1.9], 1.35),
            closed: true,
            text: "AROUND AND AROUND IT GO\nTHE SECOND LINE FOLLOWS",
            start_offset: 0.0,
            flip: false,
        },
        Ride {
            name: "círculo de baixo (virado)",
            verts: circle_contour([0.0, -1.9], 1.35),
            closed: true,
            text: "READ ME FROM THE OTHER SIDE",
            start_offset: 0.0,
            flip: true,
        },
    ]
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    if f != 3 {
        return;
    }
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let scene = &mut gfx.vec_scene;

    let mut names = Vec::new();
    for r in scenes() {
        ride(scene, &r);
        names.push(r.name);
    }

    eprintln!(
        "[smoke] W3 texto em caminho -- {}.\n\
         [smoke]   1. ONDA -- as letras giram com a curva (medido: de +37 a -54 graus ao longo\n\
         [smoke]      dela) e ficam de pe'; o espacamento nao aperta nem abre nas dobras, porque\n\
         [smoke]      quem conta e' o ARCO e nao o parametro.\n\
         [smoke]   2. CIRCULO de cima -- 2 linhas cobrindo ~70% da volta: a de baixo corre\n\
         [smoke]      PARALELA a' de cima, do lado de fora (num percurso anti-horario a subida\n\
         [smoke]      da letra aponta para o centro, entao a 2a linha empilha para fora).\n\
         [smoke]   3. CIRCULO de baixo -- VIRADO: mesmo caminho, texto do outro lado e a ler ao\n\
         [smoke]      contrario.\n\
         [smoke]   Medido: 11 + 39 + 22 glyphs, TODOS desenhados -- nenhum truncado.\n\
         [smoke]   REPROVE se: alguma letra deitada/esticada/cisalhada, letras empilhadas numa\n\
         [smoke]   ponta, ou a onda com as letras todas no mesmo angulo.",
        names.join(" | ")
    );
}

/// Empurra o caminho (cinzento, só traço) e o texto que o cavalga (azul, preenchido).
fn ride(scene: &mut VecScene, r: &Ride) {
    scene.push_path(VecPath {
        verts: r.verts.clone(),
        closed: r.closed,
        fill: None,
        stroke: Some(StrokeSpec::new(Rgba8::new(120, 120, 130, 255), 0.012)),
        ..Default::default()
    });
    let Some(path) = ArcPath::from_contour(&r.verts, r.closed) else {
        return;
    };
    let fill = Some(Paint::solid(Rgba8::new(90, 150, 220, 255)));
    if let Some(p) = text_to_compound_path(
        &crate::vec_font::resolve(None),
        r.text,
        &layout(),
        &AXES,
        &TextPlacement::OnPath {
            path: &path,
            start_offset: r.start_offset,
            flip: r.flip,
        },
        &fill,
        &None,
    ) {
        scene.push_path(p);
    }
}

/// O texto da cena: pesado, para a rotação por-glyph ser legível a olho.
fn layout() -> TextLayout {
    TextLayout {
        size: SIZE,
        line_height: 1.25,
        tracking: 0.0,
        align: TextAlign::Left,
    }
}

/// Uma onda em S larga — duas dobras em sentidos opostos, que é onde um referencial preso
/// ao ângulo médio (em vez de à tangente local) se denuncia.
fn wave_contour() -> Vec<VecVertex> {
    let pt = |x: f64, y: f64, hx: f64| VecVertex {
        anchor: [x, y],
        in_handle: [x - hx, y],
        out_handle: [x + hx, y],
        kind: VertexKind::Smooth,
        corner_radius: 0.0,
    };
    vec![
        pt(-3.2, 0.0, 0.9),
        pt(-1.1, 0.9, 0.9),
        pt(1.1, -0.9, 0.9),
        pt(3.2, 0.0, 0.9),
    ]
}

/// Um círculo em quatro cúbicas, ANTI-HORÁRIO — o percurso decide de que lado o texto
/// assenta, e é por isso que o flip existe.
fn circle_contour(c: [f64; 2], r: f64) -> Vec<VecVertex> {
    const K: f64 = 0.552_284_749_830_793_4;
    let p = [
        [c[0] + r, c[1]],
        [c[0], c[1] + r],
        [c[0] - r, c[1]],
        [c[0], c[1] - r],
    ];
    let t = [[0.0, K * r], [-K * r, 0.0], [0.0, -K * r], [K * r, 0.0]];
    (0..4)
        .map(|i| VecVertex {
            anchor: p[i],
            in_handle: [p[i][0] - t[i][0], p[i][1] - t[i][1]],
            out_handle: [p[i][0] + t[i][0], p[i][1] + t[i][1]],
            kind: VertexKind::Smooth,
            corner_radius: 0.0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena mostra o que a mensagem diz que ela mostra.**
    ///
    /// A mensagem do smoke faz afirmações — *nenhum glyph truncado*, *as letras giram* — e
    /// uma cena que as contradiga é pior do que nenhuma cena: o artista reprova o produto
    /// por um defeito do palco. Duas cenas da linha de física afirmaram coisas que a medição
    /// desmentiu, e a política que nasceu daquilo é esta: **a sonda roda ANTES de a mensagem
    /// ser escrita**, e fica.
    #[test]
    fn the_scene_draws_every_glyph_and_the_letters_turn() {
        let font = crate::vec_font::resolve(None);
        let layout = layout();
        for r in scenes() {
            let (name, flip) = (r.name, r.flip);
            let ap = ArcPath::from_contour(&r.verts, r.closed).expect("caminho");
            let on = TextPlacement::OnPath {
                path: &ap,
                start_offset: r.start_offset,
                flip,
            };
            let plain = crate::vec_glyph::text_to_vec_paths(
                &font,
                r.text,
                &layout,
                &AXES,
                &TextPlacement::At([0.0, 0.0]),
                &None,
                &None,
            );
            let ridden = crate::vec_glyph::text_to_vec_paths(
                &font, r.text, &layout, &AXES, &on, &None, &None,
            );
            assert_eq!(
                plain.len(),
                ridden.len(),
                "{name}: a cena trunca — o caminho é curto demais para o texto que ela põe nele"
            );
            // E as letras GIRAM: o ângulo do referencial varia ao longo do caminho. Uma cena
            // em que ele não varia demonstra a feature sem a mostrar.
            // ⚠️ A sonda zera o `start_offset`: aqui a pergunta é sobre a direção do CAMINHO,
            // e somar o offset do texto empurraria a última amostra para fora dele.
            let sweep = TextPlacement::OnPath {
                path: &ap,
                start_offset: 0.0,
                flip,
            };
            let mut lo = f64::INFINITY;
            let mut hi = f64::NEG_INFINITY;
            for i in 0..=6 {
                let s = ap.total() * f64::from(i) / 6.0 * 0.999;
                let f = crate::vec_glyph::caret_frame(&sweep, [s, 0.0]).expect("referencial");
                let a = f.x_axis[1].atan2(f.x_axis[0]).to_degrees();
                lo = lo.min(a);
                hi = hi.max(a);
            }
            assert!(
                hi - lo > 60.0,
                "{name}: as letras mal giram ({lo:.0} a {hi:.0} graus)"
            );
        }
    }
}
