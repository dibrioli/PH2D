//! ⭐ **A sonda da cena 26 «O herói na cena»** (`docs/PixelArt/P14_plano.md` §8): a cena MONTADA
//! pelo roteador (`smoke::stage(.., 26, ..)`), medida — o quadro mostrado em t = 500/1000/1500 ms, a
//! altura de pouso pela slice `body` (ponte do rapier real), a razão de luz esquerda/direita do herói
//! com e sem a camada Normal Map e a área da sombra do herói na parede (com e sem Blocks Light), pela
//! placa.
//!
//! ```text
//! bash scripts/ph2d-run.sh bash scripts/cargo-test-narrow.sh ph2d-host-pixel-scenes \
//!     a_cena_26_mede_o_heroi -- --ignored --nocapture
//! ```

use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro};
use ph2d_app_pixel::PixelApp;
use ph2d_ecs::{
    Entity, FrameRef, GlobalTransform, Name, PixelSprite, PresentWorld, SimWorld, SpriteAnimations,
    SpriteAnimator, SpriteGrid, Transform,
};
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::BloqueiaLuz;
use ph2d_physics_ecs::{AnimRules, AnimWhen, PhysicsBridge};
use ph2d_pixel_cenas::heroi as h;
use ph2d_pixel_core::Sprite;
use ph2d_pixel_luz::{ObjetoPixel, OclusaoDoPixel, banda_normal, base_da_instancia};
use ph2d_render::Camera2d;
use ph2d_vector::{Affine, ImageQuality, Point, StableImage, VectorScene};

use crate::comum::spawn_root;
use crate::comum_luz::{Cadeia, FUNDO, gpu, ler_bgra};

/// A janela da sonda: 256 × 128 px a 100 px/m (a régua da cena), centrada sobre o chão.
const W: u32 = 256;
const H: u32 = 128;

fn afim() -> Affine {
    Camera2d::new(
        [0.0, -0.1],
        f32::from(u16::try_from(H).unwrap()) / h::PPM as f32,
    )
    .world_to_screen_affine(WindowSize::new(W, H))
}

struct Cena {
    app: PixelApp,
    sim: SimWorld,
    heroi: Entity,
}

fn por_nome(w: &mut World, nome: &str) -> Entity {
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w).find(|(_, n)| n.0 == nome).expect(nome).0
}

fn chave(w: &World, e: Entity) -> u32 {
    w.get::<PixelSprite>(e).expect("Pixel Sprite").0
}

/// A cena 26 pelo roteador da família, como o `PH2D_PIXEL_SMOKE=26` a monta.
fn montada() -> Cena {
    let mut app = PixelApp::default();
    let mut sim = SimWorld::new();
    let mut nasceu = None;
    let ok = ph2d_app_pixel::smoke::stage(&mut app, &mut sim, 26, |s, n| {
        let b = spawn_root(s, n);
        nasceu = Some(b);
        b
    });
    assert!(ok, "a cena 26 monta");
    let heroi = Entity::from_bits(nasceu.expect("o herói nasceu"));
    Cena { app, sim, heroi }
}

fn sprite_de(app: &PixelApp, w: &World, e: Entity) -> Sprite {
    app.doc(chave(w, e)).expect("doc").sprite.clone()
}

/// O tique de `ph2d_app_sprite_anim::tick` (a lei `advance_frames_marked`), como em `derivado.rs`.
fn tique(w: &mut World, e: Entity, dt_us: u64) {
    let tags = w.get::<SpriteAnimations>(e).expect("clips").clone();
    let mut grid = *w.get::<SpriteGrid>(e).expect("grelha");
    let mut anim = w.get::<SpriteAnimator>(e).expect("animador").clone();
    if !anim.playing {
        return;
    }
    let tag = tags.get(&anim.current).expect("tag").clone();
    let cells = ph2d_ecs::cells_per_sheet(Some(&grid), None);
    let mut frame = grid.shown();
    let mut step = 0;
    ph2d_ecs::advance_frames_marked(
        &mut anim,
        &tag,
        None,
        &mut frame,
        &mut step,
        &cells,
        dt_us,
        None,
        &mut Vec::new(),
    );
    grid.show(frame);
    w.entity_mut(e).insert((grid, anim));
}

/// O Play: o Autoplay põe o 1.º clip (o `idle`); a regra do cartão, com `move_right` premida a
/// partir de `prime` ms, troca para `run` (a `troca` do `personagem.rs`). Devolve a célula em cada ms.
fn toca(c: &mut Cena, prime: Option<u32>, solta: u32, ate: u32) -> Vec<usize> {
    let s = sprite_de(&c.app, c.sim.world(), c.heroi);
    let w = c.sim.world_mut();
    ph2d_pixel_cena::deriva(w, |_| Some((&s, 1)));
    let e = c.heroi;
    let clips = w.get::<SpriteAnimations>(e).expect("clips").clone();
    let regra = AnimRules::of_entity(w, e).expect("regras");
    let run = match &regra.0[0].when {
        AnimWhen::Action(a) if a == h::ACAO => regra.0[0].play.clone(),
        o => panic!("a regra da cena é «Key held move_right»: {o:?}"),
    };
    {
        let mut a = w.get_mut::<SpriteAnimator>(e).expect("animador");
        assert!(a.autoplay && a.current.is_empty(), "nasce com Autoplay");
        a.current = clips.0.first().expect("1.º clip").name.clone();
        a.playing = true;
    }
    let primeiro = clips.0.first().map(|t| t.name.clone()).unwrap();
    let r = clips.get(&primeiro).cloned();
    w.get_mut::<SpriteAnimator>(e).unwrap().rewind(r.as_ref());
    tique(w, e, 0);
    let celula = |w: &World| match w.get::<SpriteGrid>(e).expect("grelha").shown() {
        FrameRef::Cell { cell, .. } => cell as usize,
        FrameRef::Empty => usize::MAX,
    };
    let mut fita = vec![celula(w)];
    let entrada = ph2d_physics_ecs::clip_de_entrada(w, e);
    let mut antes = None;
    for t in 1..=ate {
        // A lei real do «Key held» (`segura_regra`, a que o `personagem.rs` chama): laço e soltar.
        let manda = prime.filter(|p| (*p..solta).contains(&t)).map(|_| 0);
        if prime.is_some_and(|p| t > p) {
            let mut a = w.get_mut::<SpriteAnimator>(e).unwrap();
            antes =
                ph2d_physics_ecs::segura_regra(&regra.0, manda, antes, &mut a, entrada.as_deref());
        }
        if prime == Some(t) {
            let mut a = w.get_mut::<SpriteAnimator>(e).unwrap();
            a.current.clone_from(&run);
            a.elapsed_ticks = 0;
            a.pingpong_reverse = false;
            a.repeat_count = 0;
            a.playing = true;
        }
        tique(w, e, 1000);
        fita.push(celula(w));
    }
    fita
}

/// A física da cena (a ponte do rapier, a forma derivada a cada passo); devolve a altura do herói.
fn pousa(c: &mut Cena) -> f32 {
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
    c.sim
        .world()
        .get::<Transform>(c.heroi)
        .expect("t")
        .translation
        .y
}

fn objeto<'a>(w: &World, e: Entity, s: &'a Sprite) -> ObjetoPixel<'a> {
    let tr = ph2d_ecs::world_transform(w, e).unwrap();
    let quadro = s.frames()[0].id;
    ObjetoPixel {
        bits: e.to_bits(),
        sprite: s,
        quadro,
        stamp: e.to_bits(),
        img_mundo: ph2d_pixel_render::image_to_world(
            ph2d_pixel_cena::pivo_px(s, quadro),
            h::PPM,
            (f64::from(tr.translation.x), f64::from(tr.translation.y)),
            f64::from(tr.rotation),
            (f64::from(tr.scale.x), f64::from(tr.scale.y)),
            (0.0, 0.0),
        ),
        lin: base_da_instancia(w, e).unwrap(),
        sem_luz: false,
    }
}

fn cena_de_cor(objetos: &[ObjetoPixel<'_>]) -> VectorScene {
    let mut cena = VectorScene::new();
    for o in objetos {
        let mut px = Vec::new();
        ph2d_pixel_core::compose_rgba(o.sprite, o.quadro, None, &mut px);
        let img =
            StableImage::from_rgba(std::sync::Arc::new(px), o.sprite.width(), o.sprite.height())
                .unwrap();
        cena.draw_stable_image_transformed(&img, afim() * o.img_mundo, ImageQuality::Low);
    }
    cena
}

/// O BGRA do mundo com a luz acesa, a oclusão das peças e a banda normal (o que o quadro faz).
fn foto(gpu: &ph2d_gpu::GpuContext, w: &World, objetos: &[ObjetoPixel<'_>]) -> Vec<u8> {
    let mut c = Cadeia::nova_com(gpu, (W, H));
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    let mut present = PresentWorld::new();
    for _ in 0..30 {
        l.normais_mut().cenas = vec![banda_normal(objetos, afim())];
        assert!(l.preparar(w, afim(), (W, H)));
        let mut o = OclusaoDoPixel {
            objetos: objetos.to_vec(),
        };
        l.gravar_com(gpu, &mut present, &c.renderer, &mut [&mut o]);
    }
    l.ligar(gpu, &mut c.renderer, &mut c.colagem, &c.tonemap);
    c.mundo.clear_linear(gpu, FUNDO);
    let cor = cena_de_cor(objetos);
    l.normais_mut().colar(
        gpu,
        &mut c.vello,
        &c.colagem,
        c.mundo.blend_view(),
        (0, &cor),
        (W, H),
    );
    ler_bgra(gpu, c.mundo.texture())
}

fn px_de(p: [f64; 2]) -> (u32, u32) {
    let q = afim() * Point::new(p[0], p[1]);
    (q.x as u32, q.y as u32)
}

fn luma(bgra: &[u8], (x, y): (u32, u32)) -> f64 {
    let i = ((y * W + x) * 4) as usize;
    (f64::from(bgra[i]) + f64::from(bgra[i + 1]) + f64::from(bgra[i + 2])) / 3.0
}

/// Variantes da mesma cena pousada: com/sem a Normal Map à vista, com/sem o Blocks Light do herói.
struct Medida {
    razao: f64,
    claro: f64,
    escuro: f64,
    /// A luma das 3 colunas da face esquerda da parede, linha a linha.
    face: Vec<f64>,
}

fn mede(gpu: &ph2d_gpu::GpuContext, normal: bool, bloqueia: bool, presente: bool) -> (Medida, f32) {
    let mut c = montada();
    let y = pousa(&mut c);
    let w = c.sim.world_mut();
    if !presente {
        // A referência da parede: o herói fora da janela.
        w.get_mut::<Transform>(c.heroi).unwrap().translation.x = 5.0;
    }
    let (chao, parede) = (por_nome(w, "Floor"), por_nome(w, "Wall"));
    let es = [c.heroi, chao, parede];
    for e in es.iter().copied().chain([por_nome(w, "Light")]) {
        let t = *w.get::<Transform>(e).unwrap();
        w.entity_mut(e).insert(GlobalTransform::from_transform(t));
    }
    if !bloqueia {
        w.entity_mut(c.heroi).remove::<BloqueiaLuz>();
    }
    let mut sprites: Vec<Sprite> = es.iter().map(|&e| sprite_de(&c.app, w, e)).collect();
    if !normal {
        let s = &mut sprites[0];
        let ids: Vec<_> = s
            .layers()
            .iter()
            .filter(|l| l.normal)
            .map(|l| l.id)
            .collect();
        assert!(!ids.is_empty(), "controlo: o herói tem Normal Map");
        for id in ids {
            let op = ph2d_pixel_core::layers::edit(s, id, |l| l.visible = false).unwrap();
            ph2d_pixel_core::apply(s, op).unwrap();
        }
    }
    let w = c.sim.world();
    let objetos: Vec<_> = es
        .iter()
        .zip(&sprites)
        .map(|(&e, s)| objeto(w, e, s))
        .collect();
    let bgra = foto(gpu, w, &objetos);
    // O centro do disco (texel 8,5) no mundo, pela mesma porta imagem → mundo.
    let ctr = objetos[0].img_mundo * Point::new(8.5, 8.5);
    let d = 4.0 / h::PPM;
    let (claro, escuro) = (
        luma(&bgra, px_de([ctr.x - d, ctr.y])),
        luma(&bgra, px_de([ctr.x + d, ctr.y])),
    );
    // A face da parede que olha para o herói: as 3 colunas de píxeis da esquerda, de alto a baixo.
    let pw = &objetos[2];
    let mut face = Vec::new();
    for yy in 0..h::PAREDE.1 {
        for xx in 0..3 {
            let p = pw.img_mundo * Point::new(f64::from(xx) + 0.5, f64::from(yy) + 0.5);
            face.push(luma(&bgra, px_de([p.x, p.y])));
        }
    }
    let m = Medida {
        razao: claro / escuro.max(1.0),
        claro,
        escuro,
        face,
    };
    (m, y)
}

/// *Mutações vistas VERMELHAS (09/11): sem o `BloqueiaLuz` do herói em `heroi::monta` ⇒ sombra 0 ·
/// `banda_normal` sempre `None` ⇒ razão 1,645 = 1,645 · o documento com o pivô no centro ⇒ pousa 8 px
/// acima · a regra a tocar `hit` ⇒ run [5, 5, 5]. Sobreviveu (e devia): o pivô no centro SÓ na forma
/// inicial — o `deriva_formas` refaz a forma pelo documento a cada passo.*
#[test]
#[ignore = "needs a real GPU device; run with --ignored on the GPU lane"]
fn a_cena_26_mede_o_heroi() {
    // 1. O quadro mostrado (idle por Autoplay; → premida a partir de 1000 ms ⇒ run).
    let mut c = montada();
    let so_idle = toca(&mut c, None, 0, 1500);
    let mut c = montada();
    let com_run = toca(&mut c, Some(1000), 2600, 2700);
    let em = |f: &[usize]| [500, 580, 1000, 1500].map(|t| f[t]);
    eprintln!(
        "quadros t=500/580/1000/1500: só idle {:?} · → desde 1000 {:?} (run em 1100/1300/1450: {:?})",
        em(&so_idle),
        em(&com_run),
        [1100, 1300, 1450].map(|t| com_run[t])
    );
    // idle 80 80: 500 ⇒ 20 ms no ciclo (q0), 580 ⇒ 100 (q1), 1000 ⇒ 40 (q0), 1500 ⇒ 60 (q0).
    assert_eq!(em(&so_idle), [0, 1, 0, 0], "idle 0–1 (80 80)");
    // run PingPong 2 3 4 3 2 (120 120 60 120 120): +100 ⇒ 2, +300 ⇒ 3 (2.º), +450 ⇒ 2 (o último).
    assert_eq!([1100, 1300, 1450].map(|t| com_run[t]), [2, 3, 2], "run");
    assert!((2..=4).contains(&com_run[1500]), "run a tocar em 1500");
    // Segurado 1,6 s: o run (540 ms) volta em laço — sem laço ficaria parado no 2 depois de 1540.
    let segurado = &com_run[1000..2600];
    let depois: Vec<usize> = [1700, 2000, 2300].map(|t| com_run[t]).to_vec();
    eprintln!(
        "segurado: run em 1700/2000/2300 = {depois:?} · soltar em 2600 ⇒ {} em 2600, {} em 2700",
        com_run[2600], com_run[2700]
    );
    assert!(
        segurado.iter().all(|c| (2..=4).contains(c)),
        "segurado, só quadros do run"
    );
    assert!(
        com_run[1600..2600].iter().any(|&c| c != 2),
        "o run segue em laço"
    );
    assert!(
        com_run[2600] <= 1 && com_run[2700] <= 1,
        "soltar volta ao idle"
    );

    // 2. O pouso pela slice `body` (fundo 16, pés 15 ⇒ a origem 1 px acima do topo do chão).
    let mut c = montada();
    let y = pousa(&mut c);
    let topo = h::topo_do_chao();
    let acima_px = (f64::from(y) - topo) * h::PPM;
    eprintln!("pouso: y = {y:.4} m · topo do chão {topo:.4} m · a origem {acima_px:.2} px acima");
    assert!(
        (acima_px - 1.0).abs() < 0.5,
        "pousa pelo fundo da slice: {acima_px:.2} px (devia 1)"
    );

    // 3–4. A luz, pela placa.
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — luz saltada");
        return;
    };
    let (com, _) = mede(&gpu, true, true, true);
    let (plano, _) = mede(&gpu, false, true, true);
    let (sem_bloq, _) = mede(&gpu, true, false, true);
    let (sem_heroi, _) = mede(&gpu, true, false, false);
    // A sombra: os píxeis da face com menos de 85 % da luz que a MESMA face recebe sem o herói.
    let sombra = |m: &Medida| {
        m.face
            .iter()
            .zip(&sem_heroi.face)
            .filter(|(a, b)| **a < 0.85 * **b)
            .count()
    };
    eprintln!(
        "luz: com Normal Map {:.1}/{:.1} = {:.3} · sem {:.1}/{:.1} = {:.3} · sombra na parede {} px² (sem Blocks Light {} px²)",
        com.claro,
        com.escuro,
        com.razao,
        plano.claro,
        plano.escuro,
        plano.razao,
        sombra(&com),
        sombra(&sem_bloq)
    );
    assert!(com.razao > 1.0, "a luz vem da esquerda: {:.3}", com.razao);
    assert!(
        com.razao > 1.2 * plano.razao,
        "o relevo aumenta a razão: {:.3} contra {:.3}",
        com.razao,
        plano.razao
    );
    assert!(sombra(&com) > 0, "o herói faz sombra na parede");
    assert_eq!(sombra(&sem_bloq), 0, "sem Blocks Light, nenhuma sombra");
}
