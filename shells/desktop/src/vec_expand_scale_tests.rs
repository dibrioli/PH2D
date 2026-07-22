//! Gates da **LEI DA FORMA** do slider de Offset (report de 2026-07-20: *"se selecionar
//! Round, não consegue mudar"*) — arquivo irmão de `vec_expand_tests.rs` pelo teto de LOC.
//!
//! A faixa antiga (±4 unidades de MUNDO, constante) entregava o gesto natural — arrastar
//! até o fim do track — a regimes **join-inertes**: à esquerda a forma ANIQUILA (os três
//! joins produzem o mesmo nada), à direita ela estoura a tela e as quinas saem de vista. A
//! janela de retune funcionava; o dial é que apontava para onde o join não pode aparecer.
//! A lei nova: `d = fração × offset_scale(seleção)`, com `offset_scale = maxdim/2` — o
//! extremo esquerdo é **morte garantida** (inradius ≤ maxdim/2) e o direito é **dobrar a
//! forma** (o eixo maior cresce exatamente 2×, quinas na vizinhança da tela).
//!
//! ⚠️ Mutação canônica: fazer [`offset_scale`] devolver a constante `4.0` (a faixa velha)
//! tem de SANGRAR os dois primeiros gates — foi exatamente o produto que o Enio reportou.

use super::*;
use ph2d_vec_scene::{VecPath, VecVertex};

fn square(s: f64) -> VecPath {
    let h = s * 0.5;
    VecPath {
        verts: [[-h, -h], [h, -h], [h, h], [-h, h]]
            .map(VecVertex::corner)
            .to_vec(),
        closed: true,
        ..VecPath::default()
    }
}

fn scene_with_square(s: f64) -> (VecScene, PenTool, VecXforms) {
    let mut scene = VecScene::new();
    let id = scene.push_path(square(s));
    let mut pen = PenTool::default();
    pen.select_many(&[id]);
    (scene, pen, VecXforms::default())
}

fn maxdim(p: &VecPath) -> f64 {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    for v in &p.verts {
        lo = [lo[0].min(v.anchor[0]), lo[1].min(v.anchor[1])];
        hi = [hi[0].max(v.anchor[0]), hi[1].max(v.anchor[1])];
    }
    (hi[0] - lo[0]).max(hi[1] - lo[1])
}

/// **O curso do slider é quase todo VIVO.** Varre o track inteiro (41 amostras) numa forma
/// sólida: com a lei da forma, só o extremo esquerdo exato (−100% = o inradius de um
/// quadrado) aniquila — ≤ 3% do curso. Com a faixa velha (±4 de mundo), ~35% do curso era
/// forma morta: o polegar do artista passava um terço do track num regime onde NENHUM
/// join pode mudar um pixel (o "não consegue mudar" do report).
#[test]
fn the_slider_track_is_mostly_alive_on_a_solid_shape() {
    let (scene, pen, xf) = scene_with_square(2.4);
    let scale = offset_scale(&scene, &pen, &xf);
    let path = scene.paths()[0].clone();
    let (mut dead, mut total) = (0u32, 0u32);
    for i in 0..=40 {
        let frac = ph2d_tool_vector::params::slider_to_offset_frac(i as f32 / 40.0);
        let d = frac * scale;
        total += 1;
        if d.abs() >= ph2d_vec_boolean::MIN_OFFSET
            && ph2d_vec_boolean::offset_path(
                &path,
                d,
                LineJoin::Miter,
                ph2d_vec_scene::OffsetSide::Both,
            )
            .is_empty()
        {
            dead += 1;
        }
    }
    let frac_dead = f64::from(dead) / f64::from(total);
    assert!(
        frac_dead <= 0.03,
        "{:.0}% do curso do slider é forma MORTA (esperado ≤ 3%) — a faixa não é função \
         da forma (scale={scale})",
        frac_dead * 100.0
    );
}

/// **O extremo direito DOBRA a forma — nunca a estoura.** A +100% o eixo maior cresce
/// exatamente 2× (quadrado + 2·maxdim/2). Com a faixa velha, o mesmo gesto produzia uma
/// forma 4,3× maior que a original — quinas fora da tela, retune de join invisível (a
/// outra metade do report).
#[test]
fn the_full_right_of_the_track_doubles_the_shape_never_blows_past_it() {
    let (scene, pen, xf) = scene_with_square(2.4);
    let scale = offset_scale(&scene, &pen, &xf);
    let d = ph2d_tool_vector::params::slider_to_offset_frac(1.0) * scale;
    let out = ph2d_vec_boolean::offset_path(
        &scene.paths()[0].clone(),
        d,
        LineJoin::Miter,
        ph2d_vec_scene::OffsetSide::Both,
    );
    let grown = out.iter().map(maxdim).fold(0.0f64, f64::max);
    let ratio = grown / 2.4;
    assert!(
        (1.9..=2.1).contains(&ratio),
        "o extremo direito produziu {ratio:.2}× a forma (esperado ~2×) — d={d}"
    );
}

/// **A escala é a bbox de MUNDO da seleção — e a UNIÃO numa multi-seleção.** A pose
/// (escala 2× no Transform) entra na conta: um quadrado local de 2 com scale 2 mede 4 no
/// mundo → `offset_scale = 2`. Seleção vazia cai no fallback inerte `1.0`.
#[test]
fn the_scale_is_half_the_selections_world_bbox() {
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let mut scene = VecScene::new();
    let id = scene.push_path(square(2.0));
    let e = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform {
                translation: ph2d_core::Vec2::new(4.0, 0.0),
                scale: ph2d_core::Vec2::new(2.0, 1.0),
                ..ph2d_ecs::Transform::IDENTITY
            },
            ph2d_ecs::Name::new("S"),
            ph2d_ecs::VecPathRef(id),
        ))
        .id();
    map.insert(id, e.to_bits());
    let mut pen = PenTool::default();
    pen.select_many(&[id]);
    let xf = crate::vec_transform::build(&sim, &map);
    assert!(
        (offset_scale(&scene, &pen, &xf) - 2.0).abs() < 1e-9,
        "a pose (scale 2×) tem de entrar na bbox de mundo"
    );

    // União: um segundo quadrado longe alarga a bbox conjunta (de x∈[2,6] para x∈[2,11]).
    let id2 = scene.push_path(square(2.0));
    let e2 = sim
        .world_mut()
        .spawn((
            ph2d_ecs::Transform {
                translation: ph2d_core::Vec2::new(10.0, 0.0),
                ..ph2d_ecs::Transform::IDENTITY
            },
            ph2d_ecs::Name::new("T"),
            ph2d_ecs::VecPathRef(id2),
        ))
        .id();
    map.insert(id2, e2.to_bits());
    pen.select_many(&[id, id2]);
    let xf = crate::vec_transform::build(&sim, &map);
    assert!(
        (offset_scale(&scene, &pen, &xf) - 4.5).abs() < 1e-9,
        "multi-seleção usa a bbox da UNIÃO (x de 2 a 11 -> maxdim 9 -> 4.5)"
    );

    let empty = PenTool::default();
    assert!(
        (offset_scale(&scene, &empty, &xf) - 1.0).abs() < 1e-9,
        "seleção vazia cai no fallback inerte 1.0"
    );
}

/// **A escala é função das FONTES, e o preview já não as move.** Enquanto o preview churnava
/// a cena (a fonte virava o resultado, cuja bbox cresce com o próprio `d`), esta escala tinha
/// de ser CONGELADA no grab, senão o mesmo polegar valia distâncias diferentes a cada frame.
/// Hoje o documento não é tocado durante o arrasto ⇒ a bbox não se move ⇒ não há o que
/// congelar. Este gate pina a premissa: se alguém voltar a escrever o resultado na cena
/// durante o preview, ele fica VERMELHO e diz porquê.
#[test]
fn the_scale_is_a_fact_of_the_sources_which_the_preview_never_moves() {
    let (scene, pen, xf) = scene_with_square(2.0);
    let before = offset_scale(&scene, &pen, &xf);
    assert!((before - 1.0).abs() < 1e-9, "quadrado 2 -> maxdim/2 = 1");
    // O "frame de preview" do modelo novo: cozer e desenhar. A cena entra por `&VecScene` —
    // é o COMPILADOR que garante que ela não muda —, e a escala tem de sair idêntica.
    let mut sim = ph2d_ecs::SimWorld::default();
    let mut map = crate::vec_entities::VecEntityMap::new();
    let mut scene = scene;
    crate::vec_entities::sync(&mut sim, &mut scene, &mut map);
    let ids: Vec<VecPathId> = scene.paths().iter().map(|p| p.id).collect();
    crate::offset_live::arm(&mut sim, &map, &ids, 0.8, 1, 2);
    let mut live = crate::offset_live::OffsetLive::default();
    live.recook(&scene, &sim, &map, &xf);
    assert!(
        !live.live().is_empty(),
        "pré-condição: o offset tem de estar VIVO, senão o teste não observa nada"
    );
    assert!(
        (offset_scale(&scene, &pen, &xf) - before).abs() < 1e-9,
        "cozer o preview não pode mover a bbox das fontes — se moveu, o preview voltou a \
         escrever no documento e o mapa do slider deixou de ser função do grab"
    );
}

/// **O percentual do chip e a fração do slider são o MESMO número.** O mapa do store é
/// estático (`OFFSET_SLIDER_SCALE`/`OFFSET_SLIDER_OFFSET`, em pontos percentuais) e o
/// motor lê `slider_to_offset_frac` — se os dois divergirem, o chip mostra um número e o
/// offset aplica outro (o rótulo que mente).
#[test]
fn the_chip_percent_and_the_slider_fraction_agree() {
    use ph2d_tool_vector::params::{
        OFFSET_SLIDER_OFFSET, OFFSET_SLIDER_SCALE, slider_to_offset_frac,
    };
    for i in 0..=10 {
        let t = i as f32 / 10.0;
        let pct_motor = slider_to_offset_frac(t) * 100.0;
        let pct_chip =
            f64::from(OFFSET_SLIDER_OFFSET) + f64::from(t) * f64::from(OFFSET_SLIDER_SCALE);
        assert!(
            (pct_motor - pct_chip).abs() < 1e-4,
            "track {t}: motor diz {pct_motor}%, chip diz {pct_chip}%"
        );
    }
}
