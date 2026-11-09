//! Gates do Pixel Sprite na luz (P14c-pixel 2) sem placa: a normal roda com a instância, a banda só
//! nasce com quem precisa dela, o `NaoRecebeLuz` tira só esse objecto e a oclusão é o alfa da cor
//! no sítio do desenho.

use super::*;
use ph2d_ecs::{Transform, World};

fn com(rot: f32, sx: f32) -> Transform {
    let mut t = Transform {
        rotation: rot,
        ..Transform::default()
    };
    t.scale.x = sx;
    t
}

/// O objecto de `sprite` na entidade `e`, posto pela porta imagem → mundo (pivô da fixture, 16 px/m).
fn objeto<'a>(w: &World, e: Entity, sprite: &'a Sprite, sem_luz: bool) -> ObjetoPixel<'a> {
    let tr = ph2d_ecs::world_transform(w, e).unwrap();
    let quadro = sprite.frames()[0].id;
    let img_mundo = image_to_world(
        ph2d_pixel_cena_pivo(sprite, quadro),
        16.0,
        (f64::from(tr.translation.x), f64::from(tr.translation.y)),
        f64::from(tr.rotation),
        (f64::from(tr.scale.x), f64::from(tr.scale.y)),
        (0.0, 0.0),
    );
    ObjetoPixel {
        bits: e.to_bits(),
        sprite,
        quadro,
        stamp: 1,
        img_mundo,
        lin: base_da_instancia(w, e).unwrap(),
        sem_luz,
    }
}

fn ph2d_pixel_cena_pivo(s: &Sprite, q: FrameId) -> (f64, f64) {
    ph2d_pixel_cena::pivo_px(s, q)
}

fn sem_normal(mut s: Sprite) -> Sprite {
    let ids: Vec<_> = s
        .layers()
        .iter()
        .filter(|l| l.normal)
        .map(|l| l.id)
        .collect();
    assert!(!ids.is_empty(), "controlo: a fixture tem Normal Map");
    for id in ids {
        // Escondida (a bandeira fica): sem normal À VISTA, e a cor não ganha a camada.
        let op = ph2d_pixel_core::layers::edit(&s, id, |l| l.visible = false).unwrap();
        ph2d_pixel_core::apply(&mut s, op).unwrap();
    }
    s
}

/// A normal `+x` pintada (`[255, 128, 128]`) depois da base da entidade: `(nx, ny)` decodificados.
fn normal_x_no_mundo(tr: Transform) -> (f64, f64) {
    let mut w = World::new();
    let e = w.spawn(tr).id();
    let px = normal_no_mundo(&[255, 128, 128, 255], base_da_instancia(&w, e).unwrap());
    let dec = |v: u8| f64::from(v) / 255.0 * 2.0 - 1.0;
    (dec(px[0]), dec(px[1]))
}

#[test]
fn a_normal_roda_com_a_instancia() {
    let (nx, ny) = normal_x_no_mundo(com(0.0, 1.0));
    assert!(
        nx > 0.98 && ny.abs() < 0.02,
        "controlo sem rotação: ({nx}, {ny})"
    );
    let (nx, ny) = normal_x_no_mundo(com(std::f32::consts::FRAC_PI_2, 1.0));
    assert!(
        ny > 0.98 && nx.abs() < 0.02,
        "girado 90°: +y do mundo: ({nx}, {ny})"
    );
    let (nx, ny) = normal_x_no_mundo(com(std::f32::consts::FRAC_PI_2, -1.0));
    assert!(
        ny < -0.98 && nx.abs() < 0.02,
        "girado 90° e espelhado: −y: ({nx}, {ny})"
    );
    let (nx, ny) = normal_x_no_mundo(com(0.0, -3.0));
    assert!(
        nx < -0.98 && ny.abs() < 0.02,
        "espelho com escala 3 não inclina: ({nx}, {ny})"
    );
}

/// A fixture (hemisfério) girada 90°: o texel da borda DIREITA da arte (normal `+x`) chega à banda
/// como `+y` do mundo, pela imagem que a banda desenha.
#[test]
fn a_banda_da_fixture_girada_aponta_para_o_mundo() {
    let s = ph2d_pixel_cena::fixture::heroi_p14();
    let mut w = World::new();
    let e = w.spawn(com(std::f32::consts::FRAC_PI_2, 1.0)).id();
    let o = objeto(&w, e, &s, false);
    assert!(banda_normal(&[o], Affine::IDENTITY).is_some());
    let px = ph2d_pixel_core::compose_normal(&s, o.quadro);
    let girada = normal_no_mundo(&px, o.lin);
    let t = (8 * 16 + 14) * 4;
    assert!(
        px[t] > 192,
        "controlo: a borda direita é +x ({:?})",
        &px[t..t + 4]
    );
    assert!(
        girada[t + 1] > 192 && girada[t].abs_diff(128) < 24,
        "girada: {:?}",
        &girada[t..t + 4]
    );
    assert_eq!(girada[t + 3], px[t + 3], "o alfa (o da cor) não roda");
}

#[test]
fn sem_normal_a_vista_nada_e_alocado() {
    let s = ph2d_pixel_cena::fixture::heroi_p14();
    let mut w = World::new();
    let e = w.spawn(com(0.0, 1.0)).id();
    assert!(banda_normal(&[objeto(&w, e, &s, false)], Affine::IDENTITY).is_some());
    let s = sem_normal(s);
    assert!(banda_normal(&[objeto(&w, e, &s, false)], Affine::IDENTITY).is_none());
}

#[test]
fn o_no_light_tira_so_esse_objeto() {
    let s = sem_normal(ph2d_pixel_cena::fixture::heroi_p14());
    let mut w = World::new();
    let (a, b) = (w.spawn(com(0.0, 1.0)).id(), w.spawn(com(0.0, 1.0)).id());
    let (oa, ob) = (objeto(&w, a, &s, false), objeto(&w, b, &s, true));
    assert!(
        !precisa_da_banda(&[oa]),
        "controlo: sem normal nem No Light"
    );
    assert!(banda_normal(&[oa, ob], Affine::IDENTITY).is_some());
    let fora = |bits: u64| NORMAIS.with(|n| n.borrow().get(&bits).map(|(sig, _)| sig.3));
    assert_eq!(fora(a.to_bits()), Some(false), "o vizinho continua na luz");
    assert_eq!(fora(b.to_bits()), Some(true), "o No Light sai da luz");
}

/// A oclusão é o alfa da cor no lugar do desenho: o disco da fixture (raio 7,5 px, 16 px/m) na
/// janela `[−1, 1]²` a 64 px ⇒ cobre o centro, não 0,6 m ao lado; deslocado, segue.
#[test]
fn a_oclusao_do_pixel_e_o_disco_no_lugar_dele() {
    let s = ph2d_pixel_cena::fixture::heroi_p14();
    let mut w = World::new();
    let e = w.spawn(com(0.0, 1.0)).id();
    let ler = |w: &World, p: Point| {
        let o = OclusaoDoPixel {
            objetos: vec![objeto(w, e, &s, false)],
        };
        let px = o
            .rasterizar((64, 64), [-1.0, -1.0, 1.0, 1.0], &[e.to_bits()])
            .expect("desenhou");
        let (i, j) = (((p.x + 1.0) * 32.0) as usize, ((1.0 - p.y) * 32.0) as usize);
        px[(j * 64 + i) * 4 + 3]
    };
    let centro = |w: &World| objeto(w, e, &s, false).img_mundo * Point::new(8.5, 8.5);
    let c = centro(&w);
    assert_eq!(ler(&w, c), 255, "o centro do disco bloqueia ({c:?})");
    assert_eq!(ler(&w, Point::new(c.x + 0.6, c.y)), 0, "fora do disco não");
    w.get_mut::<Transform>(e).unwrap().translation.x = 0.3;
    let d = centro(&w);
    assert!((d.x - c.x - 0.3).abs() < 1e-6);
    assert_eq!(ler(&w, d), 255, "segue o objecto");
    assert_eq!(
        ler(&w, Point::new(c.x - 0.25, c.y)),
        0,
        "e larga o sítio antigo"
    );
    let o = OclusaoDoPixel {
        objetos: vec![objeto(&w, e, &s, false)],
    };
    assert!(
        o.rasterizar((64, 64), [-1.0, -1.0, 1.0, 1.0], &[])
            .is_none(),
        "quem vazio: nada"
    );
}
