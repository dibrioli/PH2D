//! Testes do `FlipStroke` — módulo-irmão pelo cap de LOC (HR-18).
//!
//! Declarado pelo pai via `#[path]`, então `super` é o módulo pai.

use super::*;

/// 🔴 **`segments()` é a porta única: o fechado tem COSTURA, o aberto não.**
///
/// Um traço fechado de N pontos tem N arestas (a última liga o fim ao começo — é o
/// que o render desenha); o mesmo traço aberto tem N−1. Foi a divergência dessa
/// resposta entre o render e o pick/marquee/hover que produziu *"uma linha do
/// triângulo e uma linha do quadrado não são sensíveis à seleção"* (Enio, §4.A).
///
/// Mutações que sangram: dropar a costura (`seam = false`) → o fechado cai para N−1;
/// emiti-la sempre (ignorar `closed`) → o aberto sobe para N.
#[test]
fn the_seam_exists_only_on_a_closed_stroke() {
    let mut tri = FlipStroke::new();
    for p in [
        Vec2::new(0.0, 0.0),
        Vec2::new(20.0, 0.0),
        Vec2::new(0.0, 10.0),
    ] {
        tri.push_default(p);
    }
    // Aberto: 2 arestas, e nenhuma volta ao ponto 0.
    assert_eq!(tri.segments().count(), 2);
    assert!(
        tri.segments().all(|(_, _, b)| b != tri.positions()[0]),
        "traco aberto nao pode ligar a ponta ao comeco"
    );
    // Fechado: 3 arestas, e a última é EXATAMENTE (último → primeiro), com o índice
    // do ponto de partida (de onde o `hits` tira a espessura).
    tri.closed = true;
    let segs: Vec<_> = tri.segments().collect();
    assert_eq!(segs.len(), 3);
    assert_eq!(segs[2].0, 2, "a costura parte do ULTIMO ponto");
    assert_eq!(segs[2].1, Vec2::new(0.0, 10.0));
    assert_eq!(segs[2].2, Vec2::new(0.0, 0.0));
    // Menos de 2 pontos não tem aresta nenhuma (fechado ou não).
    let mut dot = FlipStroke::new();
    dot.push_default(Vec2::new(1.0, 1.0));
    dot.closed = true;
    assert_eq!(dot.segments().count(), 0);
    assert_eq!(FlipStroke::new().segments().count(), 0);
}

#[test]
fn push_and_insert_keep_soa_consistent() {
    let mut s = FlipStroke::new();
    assert!(s.is_empty());
    s.push_default(Vec2::new(0.0, 0.0));
    s.push_point(Point {
        pos: Vec2::new(1.0, 0.0),
        width: 2.0,
        opacity: 0.5,
        color: Rgba::BLACK,
    });
    assert_eq!(s.len(), 2);
    assert!(s.soa_is_consistent());

    s.insert_point(1, Point::at(Vec2::new(0.5, 0.0)));
    assert_eq!(s.len(), 3);
    assert!(s.soa_is_consistent());
    assert_eq!(s.point(1).unwrap().pos, Vec2::new(0.5, 0.0));

    // Insert além do fim clampa pra len (append).
    s.insert_point(999, Point::at(Vec2::new(9.0, 9.0)));
    assert_eq!(s.point(3).unwrap().pos, Vec2::new(9.0, 9.0));

    let removed = s.remove_point(0).unwrap();
    assert_eq!(removed.pos, Vec2::new(0.0, 0.0));
    assert_eq!(s.len(), 3);
    assert!(s.soa_is_consistent());
}

#[test]
fn defaults_match_reference_table() {
    let s = FlipStroke::new();
    assert_eq!(s.hardness, DEFAULT_HARDNESS);
    assert_eq!(s.cap, (Cap::Round, Cap::Round));
    assert!(!s.closed);
    assert_eq!(s.material, MaterialId(0));
    assert!(s.fill.is_none());

    let p = Point::at(Vec2::new(1.0, 2.0));
    assert_eq!(p.width, DEFAULT_WIDTH);
    assert_eq!(p.opacity, DEFAULT_OPACITY);
    assert_eq!(p.color, Rgba::WHITE);
}

/// 🔴 **O domínio Curve é a projeção `any()` do Point** — selecionar UM ponto acende
/// o traço (os consumidores por-traço continuam certos); desmarcar todos o apaga.
/// Mutação que sangra: `set_point_selected` não re-derivar o `selected`.
#[test]
fn the_curve_domain_is_the_any_projection_of_the_points() {
    let mut s = FlipStroke::new();
    for i in 0..4 {
        s.push_default(Vec2::new(i as f32, 0.0));
    }
    assert!(!s.selected);
    assert!(s.set_point_selected(2, true));
    assert!(s.selected, "um ponto aceso tem de ligar o traco (any)");
    assert!(s.soa_is_consistent());
    assert!(s.set_point_selected(2, false));
    assert!(!s.selected, "nenhum ponto aceso = traco apagado");
}

/// **Point → Curve é promoção `any()` + desmaterializa** (half-selected só existe em
/// Point, §11). A volta ao Stroke diz *"as âncoras que toquei são deste traço"*.
///
/// (A ida — Curve→Point — **não** é mais broadcast: entrar no Point começa
/// desselecionado, ver `FlipDrawing::enter_point_domain`. Era o que o
/// `broadcast_selection_to_points` servia, e ele saiu junto.)
#[test]
fn the_point_to_curve_promotion_is_any_and_dematerializes() {
    let mut s = FlipStroke::new();
    for i in 0..3 {
        s.push_default(Vec2::new(i as f32, 0.0));
    }
    // O artista acendeu UMA âncora no domínio Point.
    assert!(s.set_point_selected(2, true));
    assert!(s.has_point_selection());
    s.promote_points_to_stroke();
    assert!(s.selected, "um ponto vivo promove o traco");
    assert!(!s.has_point_selection(), "Point desmaterializado na volta");
    // E nenhum ponto aceso NÃO promove.
    let mut t = FlipStroke::new();
    for i in 0..3 {
        t.push_default(Vec2::new(i as f32, 0.0));
    }
    assert!(t.set_point_selected(1, true));
    assert!(t.set_point_selected(1, false));
    t.promote_points_to_stroke();
    assert!(
        !t.selected,
        "nenhuma ancora acesa nao pode promover o traco"
    );
}

/// **Sem dado de ponto, o ponto herda o TRAÇO** (ausente = broadcast) — é o que faz
/// a máscara do Sculpt e o overlay lerem `point_selected` sem se importar com o
/// domínio corrente.
#[test]
fn absent_point_data_inherits_the_stroke_state() {
    let mut s = FlipStroke::new();
    s.push_default(Vec2::ZERO);
    assert!(!s.point_selected(0));
    s.selected = true;
    assert!(s.point_selected(0));
    assert!(
        !s.point_selected(99),
        "fora do range nunca esta selecionado"
    );
}

/// 🔴 **Dissolver pontos mantém o invariante SoA** (todos os arrays encolhem juntos)
/// e re-deriva o Curve. Mutação que sangra: esquecer um dos `retain`.
#[test]
fn dissolving_selected_points_keeps_the_soa_invariant() {
    let mut s = FlipStroke::new();
    for i in 0..5 {
        s.push_point(Point {
            pos: Vec2::new(i as f32, 0.0),
            width: i as f32,
            opacity: 1.0,
            color: Rgba::WHITE,
        });
    }
    s.set_point_selected(1, true);
    s.set_point_selected(3, true);
    assert_eq!(s.remove_selected_points(), 2);
    assert!(s.soa_is_consistent());
    assert_eq!(s.len(), 3);
    // Os que ficam são os certos (0, 2, 4) — largura carrega a identidade.
    assert_eq!(s.widths(), &[0.0, 2.0, 4.0]);
    assert!(!s.selected, "nada mais selecionado depois do dissolve");
}

/// **Os buracos só andam com o traço INTEIRO selecionado** — eles não têm seleção
/// própria; num move parcial (deformação) o anel fica, e o fill re-tria no render.
#[test]
fn holes_follow_only_a_fully_selected_stroke() {
    let mut s = FlipStroke::new();
    for i in 0..3 {
        s.push_default(Vec2::new(i as f32, 0.0));
    }
    s.holes.push(vec![Vec2::new(5.0, 5.0)]);
    // Parcial: o buraco fica.
    s.set_point_selected(0, true);
    assert!(s.translate_selected_points(Vec2::new(1.0, 0.0)));
    assert_eq!(s.holes[0][0], Vec2::new(5.0, 5.0));
    // Inteiro: o buraco anda.
    for i in 0..3 {
        s.set_point_selected(i, true);
    }
    assert!(s.translate_selected_points(Vec2::new(1.0, 0.0)));
    assert_eq!(s.holes[0][0], Vec2::new(6.0, 5.0));
}

#[test]
fn round_trips_through_postcard() {
    let mut s = FlipStroke::new();
    s.push_default(Vec2::new(0.0, 0.0));
    s.push_default(Vec2::new(1.0, 1.0));
    s.closed = true;
    s.fill = Some(Fill {
        color: Rgba::BLACK,
        opacity: 0.8,
    });
    let bytes = postcard::to_allocvec(&s).unwrap();
    let back: FlipStroke = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(back, s);
}
