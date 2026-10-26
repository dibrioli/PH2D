//! **O que as regras de CAD dizem, por cima dos nós** (W4.4, `docs/BlenderConstraints/04_W4_cad.md` §0): cada regra é
//! um traço entre os pontos que liga, na cor do seu estado; cada ponto citado é um losango — livre ou definido; um
//! conflito escreve «apague uma destas» com o nome de cada regra envolvida. Figuras novas (o losango, o traço fino):
//! os quadrados são as âncoras e o anel é o nó soldado.

use ph2d_text::TextSystem;
use ph2d_tokens::{ColorToken, Theme};
use ph2d_vec_constraints::{Estado, EstadoRegra, Liberdade};
use ph2d_vec_scene::cad::{Lei, PontoCad, circulo_de};
use ph2d_vec_scene::{VecScene, VecViewState, VecXforms, Xform};
use ph2d_vector::{Affine, BezPath, Brush, Color as VelloColor, Point, Stroke, VectorScene};

/// Meia-diagonal do losango de um ponto citado, em px de TELA: dentro do anel do nó soldado (`7,5`), fora do quadrado
/// de uma âncora (meia-aresta `4,5`) só nas pontas — lê-se como marca dela, não como outra âncora.
const LOSANGO_PX: f64 = 6.0; // LITERAL-PX-OK: figura do overlay
/// Espessura do traço de uma regra e do losango, em px de TELA.
const TRACO_PX: f64 = 1.2; // LITERAL-PX-OK: figura do overlay
/// A ficha do conflito pousa acima do traço da regra, para não o tapar.
const FICHA_ACIMA_PX: f64 = 18.0; // LITERAL-PX-OK: figura do overlay
const FICHA_MIN_W_PX: f32 = 120.0; // LITERAL-PX-OK: ficha do conflito (chrome)

/// O que se desenha, já em px de TELA.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct Plano {
    pub tracos: Vec<([f64; 2], [f64; 2], ColorToken)>,
    pub pontos: Vec<([f64; 2], ColorToken)>,
    pub conflito: Option<(String, [f64; 2])>,
}

fn cor_da_regra(e: EstadoRegra) -> Option<ColorToken> {
    match e {
        EstadoRegra::Ok => Some(ColorToken::Info),
        EstadoRegra::Ref => Some(ColorToken::Text3),
        EstadoRegra::Redundante => Some(ColorToken::Warn),
        EstadoRegra::Conflito => Some(ColorToken::Danger),
        EstadoRegra::SemAlvo => None,
    }
}

fn nome(lei: &Lei) -> String {
    ph2d_i18n::tr(match lei {
        Lei::Coincidente(..) => "app.vec.cad.lei.coincidente",
        Lei::Horizontal(..) => "app.vec.cad.lei.horizontal",
        Lei::Vertical(..) => "app.vec.cad.lei.vertical",
        Lei::Paralelo(..) => "app.vec.cad.lei.paralelo",
        Lei::Perpendicular(..) => "app.vec.cad.lei.perpendicular",
        Lei::Tangente(..) => "app.vec.cad.lei.tangente",
        Lei::Igual(..) => "app.vec.cad.lei.igual",
        Lei::Simetrico(..) => "app.vec.cad.lei.simetrico",
        Lei::SobreRecta(..) => "app.vec.cad.lei.sobre_recta",
        Lei::Distancia(..) => "app.vec.cad.lei.distancia",
        Lei::Angulo(..) => "app.vec.cad.lei.angulo",
        Lei::Raio(..) => "app.vec.cad.lei.raio",
        Lei::Fixo(..) => "app.vec.cad.lei.fixo",
    })
    .to_string()
}

/// O plano das marcas: a geometria de AGORA (o arrasto mexe-a a cada quadro), as cores da última análise.
pub(crate) fn plano(
    scene: &VecScene,
    view: &VecViewState,
    xf: &VecXforms,
    cam: Affine,
    e: &Estado,
) -> Plano {
    let tela = |p: PontoCad| -> Option<[f64; 2]> {
        let path = scene.path(p.path()).filter(|_| !view.is_hidden(p.path()))?;
        let local = match p {
            PontoCad::Vert(_, v) => path.vert_by_id(v)?.anchor,
            PontoCad::Centro(_) => circulo_de(path, Xform::IDENTITY)?.0,
        };
        let q = ph2d_vec_render::overlay_transform(view, xf, p.path(), cam)
            * Point::new(local[0], local[1]);
        Some([q.x, q.y])
    };
    let mut plano = Plano::default();
    let mut em_conflito = Vec::new();
    for (r, &est) in scene.regras().iter().zip(&e.regras) {
        let Some(cor) = cor_da_regra(est) else {
            continue;
        };
        let ps: Vec<[f64; 2]> = r.lei.pontos().into_iter().filter_map(tela).collect();
        if let [a, b, ..] = ps[..] {
            plano.tracos.push((a, b, cor));
        }
        if est == EstadoRegra::Conflito {
            em_conflito.push((nome(&r.lei), ps.first().copied()));
        }
    }
    for (&p, &l) in &e.pontos {
        if let Some(q) = tela(p) {
            let cor = match l {
                Liberdade::Livre => ColorToken::Accent,
                Liberdade::Definido => ColorToken::Success,
            };
            plano.pontos.push((q, cor));
        }
    }
    if let Some(onde) = em_conflito.iter().find_map(|(_, p)| *p) {
        let nomes: Vec<String> = em_conflito.into_iter().map(|(n, _)| n).collect();
        let texto = ph2d_i18n::tr_with("app.vec.cad.conflito", &[("regras", &nomes.join(" · "))]);
        plano.conflito = Some((texto, [onde[0], onde[1] - FICHA_ACIMA_PX]));
    }
    plano
}

fn cor(t: ColorToken, theme: Theme) -> Brush {
    let c = t.resolve(theme);
    Brush::Solid(VelloColor::from_rgba8(c.r, c.g, c.b, c.a))
}

/// Pinta o plano: traços, losangos e, por cima, a ficha do conflito (a cena tem de estar livre para o texto).
pub(crate) fn pinta(
    p: &Plano,
    theme: Theme,
    text_system: &mut TextSystem,
    target: &mut VectorScene,
) {
    let pena = Stroke::new(TRACO_PX);
    for &(a, b, t) in &p.tracos {
        let mut l = BezPath::new();
        l.move_to(Point::new(a[0], a[1]));
        l.line_to(Point::new(b[0], b[1]));
        target
            .inner_mut()
            .stroke(&pena, Affine::IDENTITY, &cor(t, theme), None, &l);
    }
    for &(q, t) in &p.pontos {
        let mut d = BezPath::new();
        d.move_to(Point::new(q[0], q[1] - LOSANGO_PX));
        d.line_to(Point::new(q[0] + LOSANGO_PX, q[1]));
        d.line_to(Point::new(q[0], q[1] + LOSANGO_PX));
        d.line_to(Point::new(q[0] - LOSANGO_PX, q[1]));
        d.close_path();
        target
            .inner_mut()
            .stroke(&pena, Affine::IDENTITY, &cor(t, theme), None, &d);
    }
    if let Some((texto, onde)) = &p.conflito {
        ph2d_editor_core::readout::paint_chip(
            text_system,
            target,
            texto,
            [onde[0] as f32, onde[1] as f32],
            FICHA_MIN_W_PX,
            theme,
        );
    }
}

#[cfg(test)]
#[path = "cad_marks_tests.rs"]
mod tests;
