//! ⭐ **A sonda da cena 27 «O mesmo herói pelo Sprite Inspector»** (`docs/PixelArt/P14_plano.md`
//! §8): a cena MONTADA pelo roteador (`smoke::stage(.., 27, ..)`) e, ao lado, o `.ase` que o Pixel
//! grava do herói (`heroi27::ase_do_documento`, a marca do Add Collider tirada do mundo)
//! importado pelo import do motor: `importa_ase` + `grid_for` + `liga` de `ph2d-sprite-ase-import` —
//! o `nasce` sem o `spawn_rgba` (a subida da textura, que pede a placa) — e vestido como a cena o
//! veste. Mede, nos dois: o quadro mostrado em cada ms, a altura de pouso (a ponte do rapier) e os
//! píxeis (cor e normal) de cada recorte.
//!
//! ```text
//! bash scripts/cargo-test-narrow.sh ph2d-host-pixel-scenes a_cena_27_os_dois_herois_sao_iguais -- --nocapture
//! ```

use bevy_ecs::world::World;
use ph2d_app_pixel::PixelApp;
use ph2d_ecs::{Entity, Name, PixelSprite, SimWorld, Transform};
use ph2d_physics_ecs::PhysicsBridge;
use ph2d_pixel_cena::SpriteImportado;
use ph2d_pixel_cenas::heroi as h;
use ph2d_pixel_core::Sprite;

use crate::cena26::toca_em;
use crate::comum::spawn_root;

struct Cena {
    app: PixelApp,
    sim: SimWorld,
    pixel: Entity,
    motor: Entity,
    imp: SpriteImportado,
    grelha: (u32, u32),
}

fn montada() -> Cena {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    let mut nasceu = None;
    let ok = ph2d_app_pixel::smoke::stage(&mut app, &mut sim, 27, |s, n| {
        let b = spawn_root(s, n);
        nasceu = Some(b);
        b
    });
    assert!(ok, "a cena 27 monta");
    let pixel = Entity::from_bits(nasceu.expect("o herói nasceu"));
    let key = sim
        .world()
        .get::<PixelSprite>(pixel)
        .expect("Pixel Sprite")
        .0;
    let s = app.doc(key).expect("doc").sprite.clone();
    let bytes = ph2d_pixel_cenas::heroi27::ase_do_documento(sim.world_mut(), &s, key)
        .expect("o Pixel grava o .ase");
    let m = 1.0 / h::PPM;
    let imp = ph2d_pixel_cena::importa_ase(&bytes, h::NOME_DO_SPRITE, m).expect("import");
    let (cols, rows) = ph2d_sprite_ase_import::grid_for(imp.cor.len(), imp.largura, imp.altura);
    // O `spawn_rgba` sem a placa: a sprite da folha inteira, à régua da cena.
    #[expect(clippy::cast_precision_loss, reason = "px pequenos")]
    let tamanho = [
        (cols * imp.largura) as f32 / h::PPM as f32,
        (rows * imp.altura) as f32 / h::PPM as f32,
    ];
    let motor = sim
        .world_mut()
        .spawn((
            ph2d_render::Sprite::atlas(0, tamanho, [1.0; 4]),
            Transform::IDENTITY,
            Name::new(h::NOME_DO_SPRITE),
        ))
        .id();
    ph2d_sprite_ase_import::liga(&mut sim, motor, &imp, cols, rows, m);
    let achado = ph2d_pixel_cenas::heroi27::sprite_do_motor(sim.world_mut());
    assert_eq!(
        achado,
        Some(motor),
        "a cena acha o Sprite do motor pelo nome"
    );
    h::veste_o_do_motor(sim.world_mut(), motor, [h::HEROI_27_X, h::HEROI[1]]);
    ph2d_ecs::assign_missing_stable_ids(sim.world_mut());
    Cena {
        app,
        sim,
        pixel,
        motor,
        imp,
        grelha: (cols, rows),
    }
}

fn doc(c: &Cena) -> Sprite {
    let k = c.sim.world().get::<PixelSprite>(c.pixel).expect("chave").0;
    c.app.doc(k).expect("doc").sprite.clone()
}

/// As duas fitas (o quadro mostrado em cada ms): Pixel e motor.
fn fitas(prime: Option<u32>, solta: u32, ate: u32) -> (Vec<usize>, Vec<usize>) {
    let mut c = montada();
    let s = doc(&c);
    let w = c.sim.world_mut();
    ph2d_pixel_cena::deriva(w, |_| Some((&s, 1)));
    (
        toca_em(w, c.pixel, prime, solta, ate),
        toca_em(w, c.motor, prime, solta, ate),
    )
}

/// A física da cena com os dois corpos; devolve a altura de cada um.
fn pousa(c: &mut Cena) -> (f32, f32) {
    let docs: Vec<(u32, Sprite)> = {
        let w = c.sim.world_mut();
        let mut q = w.query::<&PixelSprite>();
        let ks: Vec<u32> = q.iter(w).map(|p| p.0).collect();
        ks.into_iter()
            .map(|k| (k, c.app.doc(k).expect("doc").sprite.clone()))
            .collect()
    };
    let mut ponte = PhysicsBridge::new();
    for tick in 1..=240_u64 {
        ph2d_pixel_cena::deriva_formas(
            c.sim.world_mut(),
            |k| {
                docs.iter()
                    .find(|(d, _)| *d == k)
                    .map(|(_, s)| (s, s.frames()[0].id))
            },
            1.0 / h::PPM,
        );
        ponte.dispatch(&mut c.sim, true, tick);
    }
    let y = |w: &World, e| w.get::<Transform>(e).expect("t").translation.y;
    (y(c.sim.world(), c.pixel), y(c.sim.world(), c.motor))
}

/// Píxeis diferentes entre o quadro `k` do Pixel e a célula `k` da folha que o import empacota.
fn diferenca(c: &Cena, s: &Sprite, normal: bool, desvio: usize) -> Vec<usize> {
    let (cols, rows) = c.grelha;
    let (fw, fh) = (c.imp.largura, c.imp.altura);
    let fontes = if normal {
        c.imp.normal.as_ref().expect("o import leva a normal")
    } else {
        &c.imp.cor
    };
    let folha = ph2d_sprite_ase_import::pack(fontes, fw, fh, cols, rows);
    let sw = (cols * fw) as usize;
    s.frames()
        .iter()
        .enumerate()
        .map(|(k, f)| {
            let px = if normal {
                ph2d_pixel_core::compose_normal(s, f.id)
            } else {
                let mut v = Vec::new();
                ph2d_pixel_core::compose_rgba(s, f.id, None, &mut v);
                v
            };
            assert_eq!(
                (s.width(), s.height()),
                (fw, fh),
                "o recorte tem o tamanho do quadro"
            );
            let k = ((k + desvio) % s.frames().len()) as u32;
            let (cx, cy) = ((k % cols) * fw, (k / cols) * fh);
            (0..fh)
                .flat_map(|y| (0..fw).map(move |x| (x, y)))
                .filter(|&(x, y)| {
                    let i = ((y * fw + x) * 4) as usize;
                    let j = ((cy + y) as usize * sw + (cx + x) as usize) * 4;
                    let (a, b) = (&px[i..i + 4], &folha[j..j + 4]);
                    // Píxel transparente: a cor por baixo não se vê.
                    a != b && !(a[3] == 0 && b[3] == 0)
                })
                .count()
        })
        .collect()
}

/// *Mutações vistas VERMELHAS: ver `docs/PixelArt/P14_plano.md`, bloco «Cenas 26–27 finais».*
///
/// ⚠️ **O passo 1 IMITA o Play** (`toca_em`: põe o 1.º clip, liga `playing` e rebobina os dois à mão, e
/// avança pela `advance_frames_marked`): mede que a LEI dos quadros é a mesma nos dois, nunca a fase
/// que o app lhes dá. A fase pelos relógios reais é a de
/// [`os_dois_herois_comecam_no_mesmo_quadro_ao_entrar_em_play`] — esta dizia 0 ms de diferença enquanto
/// o app mostrava o motor no quadro 1 e o Pixel no 0.
#[test]
fn a_cena_27_os_dois_herois_sao_iguais() {
    // 1. O quadro mostrado, ms a ms (Autoplay ⇒ idle; → premida de 1000 a 2600 ⇒ run e volta).
    let (p, m) = fitas(None, 0, 1500);
    let (pr, mr) = fitas(Some(1000), 2600, 2700);
    let em = |f: &[usize]| [500, 580, 1000, 1500].map(|t| f[t]);
    let difere = |a: &[usize], b: &[usize]| a.iter().zip(b).filter(|(x, y)| x != y).count();
    eprintln!(
        "quadros t=500/580/1000/1500: Pixel {:?} · motor {:?} · run 1100/1300/1450/2600/2700: Pixel {:?} · motor {:?} · ms diferentes {} + {}",
        em(&p),
        em(&m),
        [1100, 1300, 1450, 2600, 2700].map(|t| pr[t]),
        [1100, 1300, 1450, 2600, 2700].map(|t| mr[t]),
        difere(&p, &m),
        difere(&pr, &mr)
    );
    assert_eq!(em(&p), [0, 1, 0, 0], "o Pixel toca o idle");
    assert_eq!(p, m, "os dois mostram o mesmo quadro em cada ms (idle)");
    assert_eq!(pr, mr, "os dois mostram o mesmo quadro em cada ms (run)");

    // 2. O pouso: os dois, cada um pela sua forma (slice `body` do Pixel; célula do import).
    let mut c = montada();
    let (yp, ym) = pousa(&mut c);
    let topo = h::topo_do_chao();
    let acima = |y: f32| (f64::from(y) - topo) * h::PPM;
    eprintln!(
        "pouso: Pixel y = {yp:.4} m ({:.2} px acima) · motor y = {ym:.4} m ({:.2} px acima) · diferença {:.4} px",
        acima(yp),
        acima(ym),
        (f64::from(yp) - f64::from(ym)).abs() * h::PPM
    );
    assert!((acima(yp) - 1.0).abs() < 0.5, "o Pixel pousa pela slice");
    assert!(
        (f64::from(yp) - f64::from(ym)).abs() * h::PPM < 0.25,
        "os dois pousam à mesma altura"
    );

    // 3. Os recortes: cor e normal de cada quadro, píxel a píxel.
    let c = montada();
    let s = doc(&c);
    let (cor, normal) = (diferenca(&c, &s, false, 0), diferenca(&c, &s, true, 0));
    // O herói da cena (`heroi_da_cena`) tem quadros desenhados diferentes: o quadro k contra a
    // célula k+1 tem de diferir — a régua vê uma troca de quadros.
    let vizinha = diferenca(&c, &s, false, 1);
    assert!(
        vizinha.iter().filter(|&&n| n > 0).count() >= 4,
        "quadros vizinhos diferem: {vizinha:?}"
    );
    // Controlo: a régua vê píxeis diferentes (a cor do quadro 0 contra a NORMAL da célula 0).
    let mut px = Vec::new();
    ph2d_pixel_core::compose_rgba(&s, s.frames()[0].id, None, &mut px);
    let n0 = &c.imp.normal.as_ref().expect("normal")[0];
    let cruzada = px
        .chunks(4)
        .zip(n0.chunks(4))
        .filter(|(a, b)| a != b && !(a[3] == 0 && b[3] == 0))
        .count();
    eprintln!("célula k+1: cor {vizinha:?} · controlo cor × normal do quadro 0: {cruzada} px");
    assert!(cruzada > 0, "a régua vê píxeis diferentes");
    eprintln!(
        "recortes ({} quadros, {}×{} px, folha {}×{}): píxeis diferentes — cor {cor:?} · normal {normal:?}",
        s.frames().len(),
        c.imp.largura,
        c.imp.altura,
        c.grelha.0,
        c.grelha.1
    );
    assert!(cor.iter().all(|&n| n == 0), "a cor é a mesma: {cor:?}");
    assert!(
        normal.iter().all(|&n| n == 0),
        "a normal é a mesma: {normal:?}"
    );
}

/// O quadro que cada herói MOSTRA (índice do quadro do Pixel; célula da grelha do motor).
fn mostrados(c: &Cena, s: &Sprite) -> (usize, usize) {
    let w = c.sim.world();
    let id = ph2d_pixel_cena::quadro_mostrado(w, c.pixel, s, s.frames()[0].id);
    let p = s.frames().iter().position(|f| f.id == id).expect("quadro");
    let m = match w
        .get::<ph2d_ecs::SpriteGrid>(c.motor)
        .expect("grelha")
        .shown()
    {
        ph2d_ecs::FrameRef::Cell { cell, .. } => cell as usize,
        ph2d_ecs::FrameRef::Empty => usize::MAX,
    };
    (p, m)
}

/// ⭐ **Os relógios REAIS** (smoke do dono 1, fase da cena 27): o import liga `playing` (`liga`), o passo
/// fixo corre no editor e no Play pela `tick_sprite_animations` (a da shell, `fase_fixed_step_clocks`,
/// 1 ms por tique), e a aresta do Play é a `start_autoplay_animations` (a do `undo_jogo::jogo_entra`).
/// Lei do dono (09/11): ao entrar em Play todo Autoplay começa do quadro 0 no MESMO tique, seja quanto
/// for o tempo que o editor correu antes. *Visto VERMELHO com o código de 09/11: o motor, que tocava
/// no editor desde o import e tem Rules, não rebobinava.*
#[test]
fn os_dois_herois_comecam_no_mesmo_quadro_ao_entrar_em_play() {
    use ph2d_app_sprite_anim::tick as t;
    const DT: f64 = 0.001;
    for editor_ms in [0_u32, 450, 530, 1234] {
        let mut c = montada();
        let s = doc(&c);
        ph2d_pixel_cena::deriva(c.sim.world_mut(), |_| Some((&s, 1)));
        let mut drive = ph2d_preview_drive::PreviewDrive::default();
        let mut no_editor = Vec::new();
        for _ in 0..editor_ms {
            t::tick_sprite_animations(&mut c.sim, 1, DT, &[], &mut drive);
            no_editor.push(mostrados(&c, &s));
        }
        t::start_autoplay_animations(&mut c.sim);
        let mut fita = vec![mostrados(&c, &s)];
        for _ in 0..1500 {
            t::tick_sprite_animations(&mut c.sim, 1, DT, &[], &mut drive);
            fita.push(mostrados(&c, &s));
        }
        let mudou_no_editor = |k: fn(&(usize, usize)) -> usize| {
            no_editor
                .windows(2)
                .filter(|w| k(&w[0]) != k(&w[1]))
                .count()
        };
        let difere: Vec<usize> = (0..fita.len())
            .filter(|&i| fita[i].0 != fita[i].1)
            .collect();
        eprintln!(
            "editor {editor_ms} ms: trocas de quadro no editor Pixel {} · motor {} · no Play t=0/500/580/1000: {:?} · ms diferentes {} (1.º {:?})",
            mudou_no_editor(|q| q.0),
            mudou_no_editor(|q| q.1),
            [0, 500, 580, 1000].map(|i| fita[i]),
            difere.len(),
            difere.first()
        );
        assert_eq!(
            fita[0],
            (0, 0),
            "o Play começa no quadro 0 (editor {editor_ms} ms)"
        );
        assert!(
            difere.is_empty(),
            "os dois no mesmo quadro a cada ms do Play (editor {editor_ms} ms)"
        );
    }
}
