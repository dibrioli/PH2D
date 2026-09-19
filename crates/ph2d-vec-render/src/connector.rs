//! **As alças de ponta do conector** — os dois círculos que dizem onde a linha encosta.
//!
//! # O raio mora AQUI, e o hit-test importa daqui
//!
//! O círculo é desenhado num raio e agarrado noutro: se os dois números divergirem, o usuário
//! clica exatamente no meio da bolinha e não pega nada — e não há sintoma mais enlouquecedor,
//! porque a tela está certa. É o mesmo tropeço do tempo remapeado
//! (`feedback_derived_coordinate_seed_must_match_sample`), na versão de dois pixels: **quem
//! desenha e quem agarra leem a MESMA constante.**
//!
//! # Verde e azul não são decoração
//!
//! É a convenção do draw.io e do Visio, e ela carrega informação que não cabe em outro lugar:
//! **verde** = a ponta escolhe o lado sozinha (ela se re-decide quando a outra forma se move);
//! **azul** = o usuário a pregou naquele ponto. Sem a distinção não há como saber, olhando, se
//! aquela linha ainda vai se reorganizar sozinha ou não.

use ph2d_vector::{Affine, BezPath, Brush, Circle, Color, Fill, Point, Stroke, VectorScene};

/// O raio do círculo da alça, em PIXELS de tela. Em pixels e não em mundo: uma alça que
/// encolhesse com o zoom-out ficaria impegável exatamente quando o diagrama fica grande.
pub const HANDLE_R_PX: f64 = 7.0;

/// A ponta **automática** (o lado é re-escolhido a cada frame conforme a outra forma se move).
const FLOATING: Color = Color::from_rgba8(90, 210, 130, 255);
/// A ponta **fixada** pelo usuário num ponto da forma.
const PINNED: Color = Color::from_rgba8(90, 160, 240, 255);
const RING: Color = Color::from_rgba8(255, 255, 255, 255);

/// Desenha as alças. `points` = `(ponto de MUNDO, está fixada?)`; `transform` leva mundo→tela.
///
/// O círculo é desenhado em espaço de TELA (raio constante em pixels), então o ponto sobe pelo
/// afim e o raio não.
pub fn draw_connector_handles(
    points: &[([f64; 2], bool)],
    transform: Affine,
    target: &mut VectorScene,
) {
    for &(w, pinned) in points {
        let p = transform * Point::new(w[0], w[1]);
        let c = Circle::new(p, HANDLE_R_PX);
        target.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(if pinned { PINNED } else { FLOATING }),
            None,
            &c,
        );
        // O anel branco destaca a alça de qualquer fundo — a linha do conector passa por baixo
        // dela, e sem o anel a bolinha some sobre um traço claro.
        target.inner_mut().stroke(
            &Stroke::new(1.5),
            Affine::IDENTITY,
            &Brush::Solid(RING),
            None,
            &c,
        );
    }
}

/// O ponto de passagem — um QUADRADO, e não um círculo.
///
/// A forma carrega o significado: um círculo é uma ponta (ela se prende a alguma coisa), um
/// quadrado é um ponto fincado no espaço. Distinguir só pela cor obrigaria o usuário a lembrar
/// qual é qual; pela forma, ele não precisa lembrar de nada.
const WAYPOINT: Color = Color::from_rgba8(245, 190, 90, 255);

/// Desenha os pontos de passagem, em MUNDO.
pub fn draw_connector_waypoints(points: &[[f64; 2]], transform: Affine, target: &mut VectorScene) {
    let h = HANDLE_R_PX * 0.85; // meia-aresta: um quadrado de mesma área "pesa" mais que o círculo
    for &w in points {
        let p = transform * Point::new(w[0], w[1]);
        let mut sq = BezPath::new();
        sq.move_to(Point::new(p.x - h, p.y - h));
        sq.line_to(Point::new(p.x + h, p.y - h));
        sq.line_to(Point::new(p.x + h, p.y + h));
        sq.line_to(Point::new(p.x - h, p.y + h));
        sq.close_path();
        target.inner_mut().fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(WAYPOINT),
            None,
            &sq,
        );
        target.inner_mut().stroke(
            &Stroke::new(1.5),
            Affine::IDENTITY,
            &Brush::Solid(RING),
            None,
            &sq,
        );
    }
}

/// A **prévia de inserção** — onde um clique da caneta poria um ponto.
///
/// ⚠️ **VERDE e OCA, e as duas coisas carregam significado:** a cor é a das pontas flutuantes (algo
/// que ainda não existe), e o miolo vazio diz que ali **não há** nó — um disco cheio leria-se como
/// uma âncora já posta, que é a coisa que o artista está a tentar distinguir.
const PREVIA: Color = Color::from_rgba8(90, 210, 130, 255);

/// ⭐⭐⭐ **Desenha a prévia de inserção da caneta**, num ponto de MUNDO.
///
/// ⛔ Report do dono, 2026-09-19: *«não tem indicação visual que você está em cima da linha para
/// criar um ponto»*. ⚠️ O raio é constante em PÍXEIS (o ponto sobe pelo afim e o raio não), como nas
/// alças ao lado — um realce que encolhe com o zoom desaparece exactamente quando se aproxima para
/// acertar.
pub fn draw_insert_preview(world: [f64; 2], transform: Affine, target: &mut VectorScene) {
    let p = transform * Point::new(world[0], world[1]);
    let c = Circle::new(p, HANDLE_R_PX);
    // O anel branco por baixo destaca de qualquer fundo — a própria linha da forma passa por ali.
    target.inner_mut().stroke(
        &Stroke::new(3.0),
        Affine::IDENTITY,
        &Brush::Solid(RING),
        None,
        &c,
    );
    target.inner_mut().stroke(
        &Stroke::new(1.5),
        Affine::IDENTITY,
        &Brush::Solid(PREVIA),
        None,
        &c,
    );
    // A CRUZ no meio: ela diz *acrescentar*, e é o que separa este realce de uma âncora existente.
    let b = HANDLE_R_PX * 0.55;
    for (a, z) in [
        ((p.x - b, p.y), (p.x + b, p.y)),
        ((p.x, p.y - b), (p.x, p.y + b)),
    ] {
        let mut l = BezPath::new();
        l.move_to(Point::new(a.0, a.1));
        l.line_to(Point::new(z.0, z.1));
        target.inner_mut().stroke(
            &Stroke::new(1.5),
            Affine::IDENTITY,
            &Brush::Solid(PREVIA),
            None,
            &l,
        );
    }
}
