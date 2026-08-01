//! **A LINHA DE CORTE desenhada** — hachurada, com uma tesoura na ponta (plano 25 §7).
//!
//! Ela é a única geometria da cena que o **render de arte não desenha** e o overlay desenha: o
//! caminho perde `fill` e `stroke` ao ser adotado como lâmina, e é aqui que ele reaparece.
//!
//! # Por que a lâmina não pode ser desenhada como arte
//!
//! Três consequências, todas de produto: uma lâmina com estilo herdaria a cor e a espessura do
//! traço corrente e ficaria **indistinguível de um desenho**; sairia no export; e mudaria de
//! aparência quando o artista mexesse no traço, que é a última coisa que uma ferramenta deve
//! fazer. Desenhá-la aqui dá-lhe um vocabulário próprio, constante em pixels de tela, imune ao
//! zoom e ao estilo.
//!
//! # O desenho
//!
//! **Casing escuro + hachura clara por cima.** O casing existe para a lâmina se ler sobre arte
//! clara E sobre arte escura sem piscar; a hachura é o que a nomeia à distância (nenhum outro
//! overlay deste editor é tracejado a dois tons). A **tesoura** senta na ponta FINAL e aponta
//! para onde a linha ia — é ela que diz *"isto corta"* sem uma palavra.

use ph2d_vector::{Affine, BezPath, Brush, Color, PathEl, Point, Shape, Stroke, VectorScene};

/// O casing: quase preto, translúcido. Ele é 2 px mais largo que a hachura de propósito — é o
/// contorno que a faz existir sobre qualquer fundo.
const CASING: Color = Color::from_rgba8(20, 22, 26, 190);
/// A hachura: âmbar. A cor de "ferramenta armada" deste editor (as fichas do Pattern, os anéis
/// de âncora da física), e não o ciano dos overlays de seleção — a lâmina não é uma seleção.
const BLADE: Color = Color::from_rgba8(255, 186, 80, 235);

/// Largura da hachura, em px de tela.
const BLADE_W: f64 = 1.6;
/// O tracejado, em px de tela: traço curto, vão curto. Curto o bastante para uma curva apertada
/// continuar a ler como tracejada em vez de como pontilhada.
const DASH: [f64; 2] = [7.0, 4.0];

/// Meia-abertura da tesoura (o comprimento de cada lâmina), em px de tela.
const SCISSOR_BLADE_PX: f64 = 9.0;
/// Raio dos dois anéis do cabo, em px de tela.
const SCISSOR_RING_PX: f64 = 2.6;

/// Desenha a linha de corte `path` (já em coordenadas de MUNDO) sob o afim `camera`.
///
/// `path` é a geometria **cozida** — o que o artista vê é o que a lâmina corta, e re-derivar a
/// curva aqui seria uma segunda resposta a *"por onde passa este corte?"*.
pub fn draw_cut_line(path: &BezPath, camera: Affine, target: &mut VectorScene) {
    let screen = camera * path.clone();
    if screen.elements().is_empty() {
        return;
    }
    target.inner_mut().stroke(
        &Stroke::new(BLADE_W + 2.0),
        Affine::IDENTITY,
        &Brush::Solid(CASING),
        None,
        &screen,
    );
    target.inner_mut().stroke(
        &Stroke::new(BLADE_W).with_dashes(0.0, DASH),
        Affine::IDENTITY,
        &Brush::Solid(BLADE),
        None,
        &screen,
    );
    if let Some((tip, dir)) = tip_and_direction(&screen) {
        draw_scissors(tip, dir, target);
    }
}

/// A ponta FINAL da linha em tela e a direção em que ela chega — o eixo da tesoura.
///
/// Sai dos ELEMENTOS do caminho, e não de um achatamento: o penúltimo ponto de controle de um
/// segmento **é** a tangente de chegada dele, exatamente, e achatar para a redescobrir seria
/// trabalho e aproximação por nada. Os fallbacks descem para o ponto anterior quando um handle é
/// coincidente com a âncora (uma ponta reta) — e uma curva que é um ponto só não tem direção
/// nenhuma, que é o `None`.
fn tip_and_direction(screen: &BezPath) -> Option<([f64; 2], [f64; 2])> {
    let mut cur = Point::ZERO;
    let mut tip: Option<Point> = None;
    let mut from: Option<Point> = None;
    for el in screen.elements() {
        match *el {
            PathEl::MoveTo(p) => {
                cur = p;
                tip = Some(p);
                from = None;
            }
            PathEl::LineTo(p) => {
                from = Some(cur);
                cur = p;
                tip = Some(p);
            }
            PathEl::QuadTo(c, p) => {
                from = Some(c);
                cur = p;
                tip = Some(p);
            }
            PathEl::CurveTo(_, c2, p) => {
                from = Some(c2);
                cur = p;
                tip = Some(p);
            }
            PathEl::ClosePath => {}
        }
        // Um handle coincidente com a âncora não diz direção — desce para o ponto anterior.
        if let (Some(t), Some(f)) = (tip, from)
            && (t.x - f.x).hypot(t.y - f.y) <= 1e-9
        {
            from = None;
        }
        if from.is_none()
            && let Some(t) = tip
            && (cur.x - t.x).hypot(cur.y - t.y) > 1e-9
        {
            from = Some(cur);
        }
    }
    let (tip, from) = (tip?, from?);
    let d = [tip.x - from.x, tip.y - from.y];
    let len = d[0].hypot(d[1]);
    if len <= 1e-9 {
        return None;
    }
    Some(([tip.x, tip.y], [d[0] / len, d[1] / len]))
}

/// A TESOURA: duas lâminas cruzadas no pivô, com os dois anéis do cabo atrás.
///
/// Ela é desenhada em px de TELA, com o eixo em `dir` — girada para a direção em que a linha
/// chega. Um glifo fixo apontaria sempre para o mesmo lado e leria como um ícone colado, não
/// como a ponta da ferramenta.
fn draw_scissors(tip: [f64; 2], dir: [f64; 2], target: &mut VectorScene) {
    let perp = [-dir[1], dir[0]];
    // O pivô fica ATRÁS da ponta: a ponta da linha é onde o corte acaba, e as lâminas abrem para
    // lá dela (é assim que uma tesoura de verdade se apoia no traço).
    let at = |along: f64, across: f64| {
        Point::new(
            tip[0] + dir[0] * along + perp[0] * across,
            tip[1] + dir[1] * along + perp[1] * across,
        )
    };
    let pivot = at(-SCISSOR_BLADE_PX * 0.55, 0.0);

    let mut blades = BezPath::new();
    // As duas lâminas cruzam no pivô e abrem PARA A FRENTE (na direção da linha).
    for s in [1.0, -1.0] {
        blades.move_to(at(-SCISSOR_BLADE_PX * 1.5, s * SCISSOR_RING_PX * 1.4));
        blades.line_to(pivot);
        blades.line_to(at(SCISSOR_BLADE_PX * 0.5, -s * SCISSOR_BLADE_PX * 0.42));
    }
    target.inner_mut().stroke(
        &Stroke::new(BLADE_W + 2.0),
        Affine::IDENTITY,
        &Brush::Solid(CASING),
        None,
        &blades,
    );
    target.inner_mut().stroke(
        &Stroke::new(BLADE_W),
        Affine::IDENTITY,
        &Brush::Solid(BLADE),
        None,
        &blades,
    );
    // Os anéis do cabo, atrás do pivô — é o que faz o glifo ler como TESOURA e não como um V.
    for s in [1.0, -1.0] {
        let c = at(-SCISSOR_BLADE_PX * 1.85, s * SCISSOR_RING_PX * 1.55);
        let ring = ph2d_vector::Circle::new(c, SCISSOR_RING_PX).to_path(0.1);
        target.inner_mut().stroke(
            &Stroke::new(BLADE_W),
            Affine::IDENTITY,
            &Brush::Solid(BLADE),
            None,
            &ring,
        );
    }
}
