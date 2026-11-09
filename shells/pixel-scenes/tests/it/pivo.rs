//! P14b-2: **o pivô é a origem em todo leitor** — o desenho e o clique (a mesma porta) põem o
//! pivô da slice escolhida na posição do `Transform` (e giram em torno dele).

use ph2d_app_pixel::compose::Mods;
use ph2d_app_pixel::input::{self, Button, Press};
use ph2d_ecs::{Entity, Transform};
use ph2d_pixel_core::{Op, Rect, SliceKey, SliceShape, slice};
use ph2d_vector::Point;

use crate::edit::{PPM, WINDOW, born, canvas, value};

/// O pivô nos pés: célula (16, 31) do sprite 32×32.
const PES: (i32, i32) = (16, 31);

/// A slice `feet` com o pivô em [`PES`], escolhida como pivô do sprite (dois passos do Pixel).
fn com_pivo_nos_pes(app: &mut ph2d_app_pixel::PixelApp, key: u32) {
    let r = Rect::new(8, 16, 16, 16);
    let s = &mut app.foca(key).expect("doc").sprite;
    let (op, id) = slice::create(s, 0, r).expect("slice");
    app.commit(key, op).expect("cria");
    let mut lista = app.doc(key).expect("doc").sprite.slices().to_vec();
    let sl = lista.iter_mut().find(|c| c.id == id).expect("a slice");
    sl.keys = vec![SliceKey {
        frame: 0,
        shape: SliceShape {
            bounds: r,
            center: None,
            pivot: Some((PES.0 - r.x, PES.1 - r.y)),
        },
    }];
    app.commit(key, Op::Slices(lista)).expect("key");
    let op = slice::set_pivot(&app.doc(key).expect("doc").sprite, Some(id)).expect("pivô");
    app.commit(key, op).expect("pivô");
}

#[test]
fn o_pivo_e_a_origem_em_todo_leitor() {
    let (mut app, mut sim, camera, bits) = born();
    let key = input::key_of(sim.world(), bits).expect("marcador");
    com_pivo_nos_pes(&mut app, key);
    let e = Entity::try_from_bits(bits).expect("entidade");
    {
        let mut t = sim.world_mut().get_mut::<Transform>(e).expect("t");
        t.translation.x = 0.3;
        t.translation.y = -0.2;
        t.rotation = 0.5;
    }
    let tr = *sim.world().get::<Transform>(e).expect("t");
    let cam = camera.world_to_screen_affine(WINDOW);
    let alvo = cam * Point::new(f64::from(tr.translation.x), f64::from(tr.translation.y));
    // O desenho: o canto do píxel do pivô cai no Transform (girado em torno dele).
    let m = input::to_screen(
        sim.world(),
        bits,
        app.doc(key).expect("doc"),
        &camera,
        WINDOW,
        PPM,
    )
    .expect("afim");
    let p = m * Point::new(f64::from(PES.0), f64::from(PES.1));
    assert!(
        (p - alvo).hypot() < 1e-3,
        "o pivô desenha-se em {p:?}, o Transform está em {alvo:?}"
    );
    // O clique: um quarto de célula para dentro do píxel do pivô pinta ESSA célula.
    let dentro = m * Point::new(f64::from(PES.0) + 0.25, f64::from(PES.1) + 0.25);
    let antes = value(&app, &sim, bits, PES.0, PES.1);
    let ponto = (dentro.x as f32, dentro.y as f32);
    assert_eq!(
        input::press(
            &mut app,
            &canvas(&sim, &camera, bits),
            ponto,
            Button::Primary,
            Mods::default()
        ),
        Press::Done
    );
    assert!(input::release(&mut app));
    assert_ne!(
        value(&app, &sim, bits, PES.0, PES.1),
        antes,
        "o clique no pivô não pintou a célula do pivô"
    );
}

/// Censo: o centro fixo (`translate(-w/2, -h/2)`) e a meia-caixa simétrica não existem fora da porta
/// `ph2d_pixel_render::image_to_world`.
#[test]
fn o_centro_fixo_so_mora_na_porta() {
    let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../../crates");
    let mut achados = Vec::new();
    for dir in ["ph2d-app-pixel/src", "ph2d-pixel-render/src"] {
        let mut pilha = vec![std::path::PathBuf::from(raiz).join(dir)];
        while let Some(d) = pilha.pop() {
            for f in std::fs::read_dir(&d).expect("dir").flatten() {
                let p = f.path();
                if p.is_dir() {
                    pilha.push(p);
                    continue;
                }
                if p.extension().is_none_or(|x| x != "rs")
                    || p.ends_with("ph2d-pixel-render/src/view.rs")
                {
                    continue;
                }
                let txt = std::fs::read_to_string(&p).expect("lê");
                for (i, l) in txt.lines().enumerate() {
                    let fixo = l.contains("Affine::translate((-")
                        || l.contains("fn half_extents(")
                        || (l.contains("width()") && l.contains("* 0.5"));
                    if fixo {
                        achados.push(format!("{}:{}: {}", p.display(), i + 1, l.trim()));
                    }
                }
            }
        }
    }
    assert!(
        achados.is_empty(),
        "centro fixo fora da porta:\n{}",
        achados.join("\n")
    );
}

/// ⭐ **Pivot do cartão** (P14 §6), a SEQUÊNCIA: o cartão publica as slices com pivô; o pedido
/// `PivotSlice` grava no documento (UM passo do Pixel) e a origem passa a ser o pivô; *Center*
/// e o desfazer devolvem o centro.
#[test]
fn o_pivot_do_cartao_muda_a_origem_num_passo() {
    use ph2d_editor_core::pixel_panel::PixelRequest;
    let (mut app, mut sim, camera, bits) = born();
    let key = input::key_of(sim.world(), bits).expect("marcador");
    com_pivo_nos_pes(&mut app, key);
    let centro = |app: &ph2d_app_pixel::PixelApp, sim: &ph2d_ecs::SimWorld| {
        let d = app.doc(key).expect("doc");
        let e = Entity::try_from_bits(bits).expect("e");
        input::origem_px(sim.world(), e, d)
    };
    let feet = app
        .doc(key)
        .expect("doc")
        .sprite
        .pivot_slice()
        .expect("pés")
        .0;
    app.aplica(PixelRequest::PivotSlice {
        doc: key,
        slice: None,
    });
    assert_eq!(centro(&app, &sim), (16.0, 16.0), "Center");
    let n = app.doc(key).expect("doc").history.undo_len();
    app.aplica(PixelRequest::PivotSlice {
        doc: key,
        slice: Some(feet),
    });
    assert_eq!(centro(&app, &sim), (16.0, 31.0), "a origem nos pés");
    assert_eq!(
        app.doc(key).expect("doc").history.undo_len(),
        n + 1,
        "UM passo"
    );
    assert!(app.undo(key));
    assert_eq!(
        centro(&app, &sim),
        (16.0, 16.0),
        "o desfazer devolve o centro"
    );
    // O cartão lista a slice com pivô, para a lista do Inspector.
    let mut hero = ph2d_editor_core::screens::hero::HeroScreen::new(ph2d_editor_core::NodeId(1));
    hero.gizmo.replace_selection(Some(bits));
    let _ = ph2d_app_pixel::panel::frame(
        &mut app,
        sim.world(),
        &mut hero,
        &mut ph2d_editor_core::ToastQueue::default(),
    );
    let c = hero.store.pixel_sprite_card(bits).expect("o cartão");
    assert_eq!(c.doc, key);
    assert_eq!(c.pivots.iter().map(|(s, _)| *s).collect::<Vec<_>>(), [feet]);
    let _ = (&mut sim, camera);
}
