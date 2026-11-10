//! ⭐ **O Pixel Sprite na luz** (P14c-pixel 2, `docs/PixelArt/P14_plano.md` §4 L) — pela placa, com
//! as peças do produto: a oclusão do Pixel (`ph2d_pixel_luz::OclusaoDoPixel`) no `gravar_com`, e a
//! banda normal (`banda_normal`) na colagem (`NormaisDasFaixas::colar`) com o momento ligado pela
//! cena (`com_normal` = alguma banda). A fixture é a `heroi_p14`: disco de cor de raio 7,5 px e a
//! camada Normal Map com o hemisfério (y para cima).

use bevy_ecs::world::World;
use ph2d_app_light2d::{DEGRAU_PADRAO, LuzDoQuadro};
use ph2d_ecs::{GlobalTransform, PresentWorld, Transform};
use ph2d_light2d_ecs::{BloqueiaLuz, Luz, NaoRecebeLuz};
use ph2d_pixel_core::Sprite;
use ph2d_pixel_luz::{ObjetoPixel, OclusaoDoPixel, banda_normal, base_da_instancia};
use ph2d_vector::{ImageQuality, StableImage, VectorScene};

use crate::comum_luz::{Cadeia, FUNDO, H, W, afim, gpu, ler, ler_bgra};

/// 16 px/m: a célula do sprite = 1 px da janela de `comum` (16 px/m).
const PPM: f64 = 16.0;
/// A lâmpada, à ESQUERDA do sprite (a «lâmpada lateral»).
const LAMPADA: [f32; 2] = [-1.5, 0.0];

fn pose(x: f32, y: f32) -> (Transform, GlobalTransform) {
    let mut t = Transform::IDENTITY;
    t.translation.x = x;
    t.translation.y = y;
    (t, GlobalTransform::from_transform(t))
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

/// O `Transform` que põe o CENTRO do disco (texel 8,5) em `centro` (a porta imagem → mundo do pivô).
fn translacao_para(s: &Sprite, centro: [f64; 2]) -> [f32; 2] {
    let q = s.frames()[0].id;
    let (px, py) = ph2d_pixel_cena::pivo_px(s, q);
    [
        (centro[0] - (8.5 - px) / PPM) as f32,
        (centro[1] - (py - 8.5) / PPM) as f32,
    ]
}

fn objeto<'a>(
    w: &World,
    e: bevy_ecs::entity::Entity,
    s: &'a Sprite,
    sem_luz: bool,
) -> ObjetoPixel<'a> {
    let tr = ph2d_ecs::world_transform(w, e).unwrap();
    let quadro = s.frames()[0].id;
    let img_mundo = ph2d_pixel_render::image_to_world(
        ph2d_pixel_cena::pivo_px(s, quadro),
        PPM,
        (f64::from(tr.translation.x), f64::from(tr.translation.y)),
        f64::from(tr.rotation),
        (f64::from(tr.scale.x), f64::from(tr.scale.y)),
        (0.0, 0.0),
    );
    ObjetoPixel {
        bits: e.to_bits(),
        sprite: s,
        quadro,
        stamp: e.to_bits(),
        img_mundo,
        lin: base_da_instancia(w, e).unwrap(),
        sem_luz,
    }
}

/// A cor de cada objecto na faixa (o que `draw::art` pinta), com a câmera de `comum`.
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

fn lampada(w: &mut World, intensidade: Option<f32>) {
    let (t, g) = pose(LAMPADA[0], LAMPADA[1]);
    let mut luz = Luz::default();
    if let Some(i) = intensidade {
        luz.intensidade = i;
    }
    w.spawn((luz, t, g));
}

/// Os quadros da luz com a banda de `objetos` (o que a faixa encheria) e, se `bloqueia`, a oclusão.
fn assentar(
    gpu: &ph2d_gpu::GpuContext,
    c: &Cadeia,
    l: &mut LuzDoQuadro,
    w: &World,
    objetos: &[ObjetoPixel<'_>],
    bloqueia: bool,
) {
    let mut present = PresentWorld::new();
    for _ in 0..30 {
        l.normais_mut().cenas = vec![banda_normal(objetos, afim())];
        assert!(l.preparar(w, afim(), (W, H)));
        if bloqueia {
            let mut o = OclusaoDoPixel {
                objetos: objetos.to_vec(),
            };
            l.gravar_com(gpu, &mut present, &c.renderer, &mut [&mut o]);
        } else {
            l.gravar(gpu, &mut present, &c.renderer);
        }
    }
}

/// A faixa Pixel colada sobre o fundo com a luz `l` (acesa) ou sem ela; devolve o BGRA do mundo.
fn colar(
    gpu: &ph2d_gpu::GpuContext,
    c: &mut Cadeia,
    l: &mut LuzDoQuadro,
    cor: &VectorScene,
    acesa: bool,
) -> Vec<u8> {
    if acesa {
        l.ligar(gpu, &mut c.renderer, &mut c.colagem, &c.tonemap);
    } else {
        c.colagem.receber_luz(gpu, None);
    }
    c.mundo.clear_linear(gpu, FUNDO);
    l.normais_mut().colar(
        gpu,
        &mut c.vello,
        &c.colagem,
        c.mundo.blend_view(),
        (0, cor),
        (W, H),
    );
    ler_bgra(gpu, c.mundo.texture())
}

/// O píxel da janela sob o ponto do mundo `p` (`comum`: 16 px/m, centro no meio).
fn px_de(p: [f64; 2]) -> (u32, u32) {
    (
        (f64::from(W) / 2.0 + p[0] * PPM) as u32,
        (f64::from(H) / 2.0 - p[1] * PPM) as u32,
    )
}

fn luma(bgra: &[u8], (x, y): (u32, u32)) -> f64 {
    let i = ((y * W + x) * 4) as usize;
    (f64::from(bgra[i]) + f64::from(bgra[i + 1]) + f64::from(bgra[i + 2])) / 3.0
}

/// ⭐ **`BloqueiaLuz` ⇒ o Pixel Sprite faz sombra**: a fluência atrás do disco (do lado oposto à
/// lâmpada) cai contra o MESMO ponto sem a família na luz; à frente fica.
#[test]
fn o_pixel_sprite_faz_sombra() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let s = ph2d_pixel_cena::fixture::heroi_p14();
    let mut w = World::new();
    lampada(&mut w, None);
    let t = translacao_para(&s, [0.0, 0.0]);
    let (tr, g) = pose(t[0], t[1]);
    let e = w
        .spawn((
            tr,
            g,
            ph2d_ecs::PixelSprite(1),
            BloqueiaLuz { opacidade: 1.0 },
        ))
        .id();
    let cena = ph2d_light2d_ecs::coleta::cena_de_luz(&w);
    assert_eq!(cena.outros.len(), 1, "o Pixel Sprite bloqueia como «outro»");
    let objetos = [objeto(&w, e, &s, false)];
    let c = Cadeia::nova(&gpu);
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    assentar(&gpu, &c, &mut l, &w, &objetos, false);
    let sem = crate::comum_luz::fluencia_lida(&gpu, l.solver().unwrap());
    assentar(&gpu, &c, &mut l, &w, &objetos, true);
    let com = crate::comum_luz::fluencia_lida(&gpu, l.solver().unwrap());
    let (atras, frente) = ([0.85, 0.0], [-0.85, 0.0]);
    let (a0, a1) = (sem.amostra_canal(atras, 0), com.amostra_canal(atras, 0));
    let (f0, f1) = (sem.amostra_canal(frente, 0), com.amostra_canal(frente, 0));
    eprintln!(
        "sombra do Pixel: atrás {a1:.4} (sem a família {a0:.4}, × {:.3}) · à frente {f1:.4} (sem {f0:.4})",
        a1 / a0
    );
    assert!(
        a0 > 0.0 && a1 < 0.25 * a0,
        "atrás do disco tem de haver sombra: {a1:.4} contra {a0:.4}"
    );
    assert!(f1 > 0.8 * f0, "à frente a luz fica: {f1:.4} contra {f0:.4}");
}

/// ⭐ **A 4.ª condição da UI do Normal Map**: pintar com a roda na camada Normal Map ⇒ a cor do sprite
/// não muda e a luz SOMBREIA. Lâmpada à esquerda: a razão claro/escuro (luma a ±0,3 m do centro do
/// disco) com o hemisfério é maior que a do MESMO sprite sem a normal (só a distância à lâmpada).
#[test]
fn a_normal_map_sombreia_e_nao_muda_a_cor() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let com_n = ph2d_pixel_cena::fixture::heroi_p14();
    let plano = sem_normal(com_n.clone());
    let q = com_n.frames()[0].id;
    let (mut a, mut b) = (Vec::new(), Vec::new());
    ph2d_pixel_core::compose_rgba(&com_n, q, None, &mut a);
    ph2d_pixel_core::compose_rgba(&plano, q, None, &mut b);
    assert_eq!(a, b, "a Normal Map não muda a cor composta");
    let razao = |s: &Sprite| {
        let mut w = World::new();
        lampada(&mut w, None);
        let t = translacao_para(s, [0.0, 0.0]);
        let (tr, g) = pose(t[0], t[1]);
        let e = w.spawn((tr, g)).id();
        let objetos = [objeto(&w, e, s, false)];
        let mut c = Cadeia::nova(&gpu);
        let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
        assentar(&gpu, &c, &mut l, &w, &objetos, false);
        let bgra = colar(&gpu, &mut c, &mut l, &cena_de_cor(&objetos), true);
        let (claro, escuro) = (
            luma(&bgra, px_de([-0.3, 0.0])),
            luma(&bgra, px_de([0.3, 0.0])),
        );
        (
            claro,
            escuro,
            l.solver().map_or(0, |s| s.bytes_do_momento()),
        )
    };
    let (c0, e0, m0) = razao(&plano);
    let (c1, e1, m1) = razao(&com_n);
    eprintln!(
        "4.ª condição: sem normal claro {c0:.1} / escuro {e0:.1} = {:.3} (momento {m0} B) · com a roda {c1:.1} / {e1:.1} = {:.3} (momento {m1} B)",
        c0 / e0.max(1.0),
        c1 / e1.max(1.0)
    );
    assert_eq!(m0, 0, "sem Normal Map à vista o momento não é alocado (G6)");
    assert!(m1 > 0, "com a banda a cena liga o momento");
    assert!(
        c1 / e1.max(1.0) > 1.5 * (c0 / e0.max(1.0)),
        "a normal pintada tem de sombrear: {:.3} contra {:.3}",
        c1 / e1.max(1.0),
        c0 / e0.max(1.0)
    );
}

/// ⭐ **`NaoRecebeLuz` tira SÓ esse objecto** (a sentinela da banda): numa sala escura (lâmpada a
/// zero) os dois sprites na MESMA faixa — o marcado sai na cor de sempre (a colagem apagada), o outro
/// apaga.
#[test]
fn o_no_light_tira_so_esse_objeto_na_colagem() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let s = sem_normal(ph2d_pixel_cena::fixture::heroi_p14());
    let mut w = World::new();
    lampada(&mut w, Some(0.0));
    let (ta, tb) = (
        translacao_para(&s, [-1.0, 0.0]),
        translacao_para(&s, [1.0, 0.0]),
    );
    let a = w.spawn(pose(ta[0], ta[1])).id();
    let b = w.spawn((pose(tb[0], tb[1]), NaoRecebeLuz)).id();
    let objetos = [objeto(&w, a, &s, false), objeto(&w, b, &s, true)];
    let mut c = Cadeia::nova(&gpu);
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    assentar(&gpu, &c, &mut l, &w, &objetos, false);
    let cor = cena_de_cor(&objetos);
    let acesa = colar(&gpu, &mut c, &mut l, &cor, true);
    let apagada = colar(&gpu, &mut c, &mut l, &cor, false);
    let (pa, pb) = (px_de([-1.0, 0.0]), px_de([1.0, 0.0]));
    eprintln!(
        "No Light: o vizinho {:.1} (apagada {:.1}) · o marcado {:.1} (apagada {:.1})",
        luma(&acesa, pa),
        luma(&apagada, pa),
        luma(&acesa, pb),
        luma(&apagada, pb)
    );
    assert!(luma(&apagada, pa) > 40.0, "controlo: o disco tem cor");
    assert!(
        luma(&acesa, pa) < 8.0,
        "o vizinho recebe a sala escura e apaga"
    );
    assert!(
        (luma(&acesa, pb) - luma(&apagada, pb)).abs() <= 1.0,
        "o marcado sai intacto"
    );
}

/// 📏 **Medida do limiar ¼ da sentinela** numa borda com objectos SOBREPOSTOS: o marcado (No Light)
/// por cima do vizinho, deslocado meio píxel (a borda cai a meio de um píxel). Lê-se o `b` da banda
/// (renderizada sozinha pelo Vello) ao longo da linha do centro e conta-se quantos píxeis da borda
/// ficam entre «claramente fora» e «claramente dentro» — e de que lado do ¼ caem.
#[test]
fn medida_o_limiar_da_sentinela_na_borda() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — medida saltada");
        return;
    };
    let s = sem_normal(ph2d_pixel_cena::fixture::heroi_p14());
    let mut w = World::new();
    let (ta, tb) = (
        translacao_para(&s, [-0.2, 0.0]),
        translacao_para(&s, [0.2 + 0.5 / PPM, 0.0]),
    );
    let a = w.spawn(pose(ta[0], ta[1])).id();
    let b = w.spawn((pose(tb[0], tb[1]), NaoRecebeLuz)).id();
    let objetos = [objeto(&w, a, &s, false), objeto(&w, b, &s, true)];
    let banda = banda_normal(&objetos, afim()).unwrap();
    let mut c = Cadeia::nova(&gpu);
    c.vello
        .render_to_intermediate(&gpu, banda.inner(), (W, H), ph2d_vector::Color::TRANSPARENT)
        .unwrap();
    let px = ler(&gpu, c.vello.intermediate_texture(), 4);
    let y = H / 2;
    let mut linha = Vec::new();
    for x in 0..W {
        let i = ((y * W + x) * 4) as usize;
        linha.push((x, px[i + 2], px[i + 3]));
    }
    let meio: Vec<_> = linha
        .iter()
        .filter(|(_, b, a)| *a > 0 && *b > 8 && *b < 247 && *b != 255)
        .collect();
    let abaixo = meio
        .iter()
        .filter(|(_, b, _)| f64::from(*b) / 255.0 < 0.25)
        .count();
    eprintln!(
        "limiar ¼: linha y={y} (x, b, a) com a > 0: {:?} · intermédios {} (abaixo de ¼: {abaixo})",
        linha.iter().filter(|(_, _, a)| *a > 0).collect::<Vec<_>>(),
        meio.len()
    );
    assert!(
        linha.iter().any(|(_, b, a)| *a > 0 && *b == 0),
        "controlo: o marcado tem a sentinela"
    );
    assert!(
        linha.iter().any(|(_, b, _)| *b > 247),
        "controlo: o vizinho tem a normal de frente"
    );
}

/// ⭐ **G6 do Pixel**: com a luz acesa e um Pixel Sprite SEM Normal Map à vista (nem No Light), a
/// faixa não tem banda, a cena não liga o momento (0 bytes) e a colagem não aloca textura de banda.
#[test]
fn g6_o_pixel_sem_normal_a_vista_nao_aloca_nada() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let s = sem_normal(ph2d_pixel_cena::fixture::heroi_p14());
    let mut w = World::new();
    lampada(&mut w, None);
    let t = translacao_para(&s, [0.0, 0.0]);
    let e = w.spawn(pose(t[0], t[1])).id();
    let objetos = [objeto(&w, e, &s, false)];
    assert!(
        banda_normal(&objetos, afim()).is_none(),
        "sem normal à vista, nenhuma banda"
    );
    let mut c = Cadeia::nova(&gpu);
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    assentar(&gpu, &c, &mut l, &w, &objetos, false);
    let momento = l.solver().map_or(u64::MAX, |s| s.bytes_do_momento());
    let _ = colar(&gpu, &mut c, &mut l, &cena_de_cor(&objetos), true);
    let banda = l.normais_mut().bytes();
    eprintln!("G6 do Pixel: momento {momento} B · banda {banda} B");
    assert_eq!(momento, 0, "o momento não nasce sem normal à vista");
    assert_eq!(banda, 0, "a textura da banda não nasce sem normal à vista");
}

/// ⭐ P13 L — **«Light: Palette»: o sprite aceso fica na rampa**, pelo caminho do artista: o
/// `set_luz_na_paleta` que o cartão grava, e as bandas como a família as dá à shell
/// (`banda_rampa_da_faixa` ⇒ `RampaDaFaixa::from` ⇒ `NormaisDasFaixas::empurra`). A fixture com uma
/// rampa de 4 degraus (a cor dela é o 2.º) e a lâmpada à esquerda: todo píxel do miolo do sprite sai
/// numa cor da RAMPA (nunca o `rgb × luz`), e o lado aceso fica num degrau ≥ o do lado escuro. Com
/// *Smooth* o quadro é o de antes da L byte a byte. Mede o custo do passe (mínimo de 20, intercalado).
#[test]
fn a_luz_na_rampa_no_quadro_fica_na_rampa() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let rampa = [
        [20, 40, 90, 255],
        [70, 120, 200, 255],
        [150, 190, 240, 255],
        [230, 240, 255, 255],
    ];
    let mut s = ph2d_pixel_cena::fixture::heroi_p14();
    let op = ph2d_pixel_rampa::op_rampa_nova(&s, "r", &rampa).expect("rampa nova");
    ph2d_pixel_core::apply(&mut s, op).unwrap();
    let suave = s.clone();
    let op = ph2d_pixel_core::ramp::set_luz_na_paleta(&s, true).expect("o passo do cartão");
    ph2d_pixel_core::apply(&mut s, op).unwrap();
    let mut w = World::new();
    lampada(&mut w, None);
    let t = translacao_para(&s, [0.0, 0.0]);
    let (tr, g) = pose(t[0], t[1]);
    let e = w.spawn((tr, g)).id();
    let objetos = [objeto(&w, e, &s, false)];
    let objetos_suaves = [objeto(&w, e, &suave, false)];
    let mut c = Cadeia::nova(&gpu);
    let mut l = LuzDoQuadro::com_degrau(DEGRAU_PADRAO);
    assentar(&gpu, &c, &mut l, &w, &objetos, false);
    let cor = cena_de_cor(&objetos);
    let quadro = |l: &mut LuzDoQuadro, c: &mut Cadeia, o: &[ObjetoPixel<'_>], rampa: bool| {
        l.normais_mut().limpar();
        let banda = ph2d_pixel_luz::banda_rampa_da_faixa(o, afim()).map(Into::into);
        assert_eq!(
            banda.is_some(),
            rampa,
            "a banda de rampa nasce só com Palette"
        );
        l.normais_mut().empurra((banda_normal(o, afim()), banda));
        let t0 = std::time::Instant::now();
        let bgra = colar(&gpu, c, l, &cor, true);
        (bgra, t0.elapsed())
    };
    let (hoje, _) = {
        l.normais_mut().limpar();
        l.normais_mut().cenas = vec![banda_normal(&objetos_suaves, afim())];
        let t0 = std::time::Instant::now();
        (colar(&gpu, &mut c, &mut l, &cor, true), t0.elapsed())
    };
    let (smooth, _) = quadro(&mut l, &mut c, &objetos_suaves, false);
    assert!(smooth == hoje, "Smooth: o quadro de hoje byte a byte");
    let (mut com, mut sem) = (std::time::Duration::MAX, std::time::Duration::MAX);
    for _ in 0..20 {
        com = com.min(quadro(&mut l, &mut c, &objetos, true).1);
        sem = sem.min(quadro(&mut l, &mut c, &objetos_suaves, false).1);
    }
    eprintln!("custo do passe (colar + leitura, mínimo de 20): Palette {com:?} · Smooth {sem:?}");
    let (bgra, _) = quadro(&mut l, &mut c, &objetos, true);
    let degrau = |p: (u32, u32)| {
        let i = ((p.1 * W + p.0) * 4) as usize;
        let rgb = [bgra[i + 2], bgra[i + 1], bgra[i]];
        rampa.iter().position(|r| r[..3] == rgb)
    };
    let (mut na_rampa, mut fora) = (0, 0);
    for y in -4..=4 {
        for x in -4..=4 {
            let p = px_de([f64::from(x) * 0.05, f64::from(y) * 0.05]);
            match degrau(p) {
                Some(_) => na_rampa += 1,
                None => fora += 1,
            }
        }
    }
    let (claro, escuro) = (degrau(px_de([-0.3, 0.0])), degrau(px_de([0.3, 0.0])));
    eprintln!(
        "luz na rampa no quadro: miolo {na_rampa} na rampa / {fora} fora · degrau aceso {claro:?} · escuro {escuro:?} · banda {} B",
        l.normais_mut().bytes()
    );
    assert_eq!(fora, 0, "o miolo do sprite aceso tem de ficar na rampa");
    let (Some(a), Some(b)) = (claro, escuro) else {
        panic!("os dois lados têm de estar na rampa: {claro:?} {escuro:?}");
    };
    assert!(a >= b, "mais luz nunca escurece o degrau: {a} contra {b}");
}
