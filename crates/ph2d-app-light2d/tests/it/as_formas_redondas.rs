//! ⭐⭐ **As formas REDONDAS não saem em facetas** (smoke do dono, 11/10, a cena 5 «Volume»: o disco
//! saía um cone em fatias com a aresta da sombra serrilhada, a elipse com degraus nas pontas da aresta
//! do meio).
//!
//! A lei do interior (F3) projeta a luz da borda até ao eixo medial: o disco É um cone que aponta a
//! luz e a aresta da elipse É o segmento do eixo maior. Os DEFEITOS eram da precisão junto à borda: o
//! iso-½ da bilinear desvia-se da borda verdadeira até ~0,08 texel conforme a posição, e o raio que sai
//! rente a uma borda curva reentrava no contorno capturado (a passagem luz/sombra em escada, cruzando
//! o meio-tom várias vezes). A cura (W5d): o interior decide «dentro» pela borda EXATA da cobertura
//! (`c3_borda.wgsl`, o AAEDT de Gustavson & Strand).
//!
//! A régua: o disco e a elipse da cena 5 feitos POLÍGONOS de [`LADOS`] lados (o gabarito lê sprites
//! como quads), a luz da placa lida em anéis — dentro a 0,35 · 0,65 · 0,9 do raio e fora a 2,5
//! texels — contra o gabarito: os pontos errados longe das passagens e do eixo, os cruzamentos A MAIS
//! do meio-tom em cada passagem luz/sombra (o serrilhado) e o EXCESSO de variação ao longo do anel (a
//! faceta: um salto que o gabarito não tem). Sem rebote (o gabarito ponto a ponto é o direto + as
//! cascatas).

use bevy_ecs::entity::Entity;
use bevy_ecs::world::World;
use ph2d_app_light2d::{
    DEGRAU_PADRAO, captura_do_quadro, fina_do_quadro, grade_do_quadro, regiao_da_janela, smoke,
};
use ph2d_ecs::Name;
use ph2d_host::WindowSize;
use ph2d_light2d_ecs::coleta::cena_de_luz;
use ph2d_light2d_ecs::{CenaDeLuz, FormaDoSprite};
use ph2d_light2d_gpu::{Config, HOLOGRAFICA_PADRAO, Solver};
use ph2d_render::Camera2d;
use vello::kurbo::{Affine, Point};

use crate::comum::{Fluencia, fluencia_lida, gpu};

const FHD: (u32, u32) = (1920, 1080);
/// Os lados do polígono que faz de disco e de elipse.
pub(crate) const LADOS: usize = 96;
/// Amostras por anel.
pub(crate) const AMOSTRAS: usize = 1024;
/// Um ponto está ERRADO a mais disto do gabarito, relativo ao máximo dele no corpo.
pub(crate) const ERRADO: f64 = 0.10;

/// Um corpo redondo da cena 5 feito polígono: o centro e os semieixos (metros).
#[derive(Clone, Copy)]
pub(crate) struct Redondo {
    pub(crate) nome: &'static str,
    pub(crate) centro: [f64; 2],
    pub(crate) semi: [f64; 2],
}

impl Redondo {
    /// O ponto do contorno no ângulo de parâmetro `t`, escalado por `f` em volta do centro e
    /// empurrado `fora` metros pela normal.
    pub(crate) fn no_anel(&self, t: f64, f: f64, fora: f64) -> [f64; 2] {
        let (s, c) = t.sin_cos();
        let p = [self.semi[0] * c, self.semi[1] * s];
        let n = [self.semi[1] * c, self.semi[0] * s];
        let m = n[0].hypot(n[1]);
        [
            self.centro[0] + f * p[0] + fora * n[0] / m,
            self.centro[1] + f * p[1] + fora * n[1] / m,
        ]
    }

    /// A folga até o eixo medial (a elipse: o segmento do eixo maior entre os centros de curvatura
    /// das pontas; o disco: o centro), em metros.
    pub(crate) fn ao_eixo(&self, p: [f64; 2]) -> f64 {
        let (a, b) = (self.semi[0], self.semi[1]);
        let x = p[0] - self.centro[0];
        let y = p[1] - self.centro[1];
        let meio = (a - b * b / a).max(0.0);
        (x.abs() - meio).max(0.0).hypot(y)
    }

    pub(crate) fn poligono(&self) -> Vec<[f32; 2]> {
        (0..LADOS)
            .map(|k| {
                let p = self.no_anel(std::f64::consts::TAU * k as f64 / LADOS as f64, 1.0, 0.0);
                [p[0] as f32, p[1] as f32]
            })
            .collect()
    }
}

/// A cena 5 com o disco e a elipse feitos polígonos, deslocados `d` metros.
pub(crate) fn cena_redonda(d: [f64; 2]) -> (CenaDeLuz, Vec<Redondo>) {
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let mut c = cena_de_luz(&w);
    let mut redondos = Vec::new();
    for corpo in smoke::CORPOS_DO_VOLUME.iter().take(2) {
        let entidade: Entity = w
            .query::<(Entity, &Name)>()
            .iter(&w)
            .find(|(_, n)| n.as_str() == corpo.nome)
            .expect("o corpo")
            .0;
        let r = Redondo {
            nome: corpo.nome,
            centro: [
                f64::from(corpo.centro[0]) + d[0],
                f64::from(corpo.centro[1]) + d[1],
            ],
            semi: [
                0.5 * f64::from(corpo.lado[0]),
                0.5 * f64::from(corpo.lado[1]),
            ],
        };
        let i = c
            .sprites
            .iter()
            .find_map(|s| match s.forma {
                FormaDoSprite::Oclusor(i) if s.entidade == entidade => Some(i),
                _ => None,
            })
            .expect("o oclusor do corpo");
        c.oclusores[i].vertices = r.poligono();
        redondos.push(r);
    }
    (c, redondos)
}

/// Os anéis: `(nome, fator de escala, deslocamento pela normal em texels finos)`.
pub(crate) const ANEIS: [(&str, f64, f64); 4] = [
    ("dentro 0,35", 0.35, 0.0),
    ("dentro 0,65", 0.65, 0.0),
    ("dentro 0,9", 0.9, 0.0),
    ("fora +2,5 tx", 1.0, 2.5),
];

/// O gabarito de um anel: `(pontos, valores)`.
pub(crate) fn anel(c: &CenaDeLuz, r: &Redondo, f: f64, fora_m: f64) -> (Vec<[f64; 2]>, Vec<f64>) {
    let pts: Vec<[f64; 2]> = (0..AMOSTRAS)
        .map(|k| {
            r.no_anel(
                std::f64::consts::TAU * k as f64 / AMOSTRAS as f64,
                f,
                fora_m,
            )
        })
        .collect();
    let v = pts
        .iter()
        .map(|p| ph2d_light2d_ref::fluencia(c, *p)[1])
        .collect();
    (pts, v)
}

/// Meia janela (amostras do anel) em volta de cada passagem luz/sombra do gabarito.
pub(crate) const JANELA: isize = 24;

/// Um anel lido na placa contra o gabarito.
#[derive(Clone, Copy, Default)]
pub(crate) struct Leitura {
    /// Pontos a mais de [`ERRADO`] do gabarito, longe das passagens e do eixo medial.
    pub(crate) errados: usize,
    pub(crate) pior: f64,
    /// O excesso de variação total sobre o laço, relativo à amplitude do gabarito (uma faceta é um
    /// salto que o gabarito não tem).
    pub(crate) tv: f64,
    /// Cruzamentos A MAIS do meio-tom junto a uma passagem do gabarito (0 = limpa; serrilhada > 0).
    pub(crate) cruzes: usize,
    /// O maior desvio de um cruzamento da placa ao do gabarito, em texels finos ao longo do anel.
    pub(crate) desvio: f64,
}

impl Leitura {
    pub(crate) fn somar(&mut self, o: Self) {
        self.errados += o.errados;
        self.pior = self.pior.max(o.pior);
        self.tv = self.tv.max(o.tv);
        self.cruzes = self.cruzes.max(o.cruzes);
        self.desvio = self.desvio.max(o.desvio);
    }
}

/// Lê um anel: `passo_tx` = texels finos entre amostras.
pub(crate) fn ler_anel(
    fl: &Fluencia,
    pts: &[[f64; 2]],
    gab: &[f64],
    max: f64,
    passo_tx: f64,
    eixo: impl Fn([f64; 2]) -> bool,
) -> Leitura {
    let n = pts.len();
    let em = |v: &[f64], k: isize| v[k.rem_euclid(n as isize) as usize];
    let g: Vec<f64> = pts.iter().map(|p| fl.amostra(*p)).collect();
    let (lo, hi) = gab
        .iter()
        .fold((f64::INFINITY, 0.0_f64), |(a, b), &x| (a.min(x), b.max(x)));
    let mut l = Leitura::default();
    let (mut tv_g, mut tv_r) = (0.0, 0.0);
    let mut perto = vec![false; n];
    for k in 0..n as isize {
        tv_g += (em(&g, k + 1) - em(&g, k)).abs();
        tv_r += (em(gab, k + 1) - em(gab, k)).abs();
        let (lo_l, hi_l) = (-JANELA..=JANELA)
            .map(|j| em(gab, k + j))
            .fold((f64::INFINITY, 0.0_f64), |(a, b), x| (a.min(x), b.max(x)));
        if hi_l - lo_l <= 0.2 * max {
            continue;
        }
        let meio = 0.5 * (lo_l + hi_l);
        let (r0, r1) = (em(gab, k) - meio, em(gab, k + 1) - meio);
        if r0 * r1 >= 0.0 {
            continue;
        }
        let no_gab = k as f64 + r0 / (r0 - r1);
        for j in -JANELA..=JANELA {
            perto[(k + j).rem_euclid(n as isize) as usize] = true;
        }
        let mut cruzamentos = 0;
        for j in -JANELA..JANELA {
            let (g0, g1) = (em(&g, k + j) - meio, em(&g, k + j + 1) - meio);
            if g0 * g1 < 0.0 || (g0 == 0.0 && g1 != 0.0) {
                cruzamentos += 1;
                let na_placa = (k + j) as f64 + g0 / (g0 - g1);
                l.desvio = l.desvio.max((na_placa - no_gab).abs() * passo_tx);
            }
        }
        if cruzamentos == 0 {
            l.desvio = l.desvio.max(JANELA as f64 * passo_tx);
        }
        l.cruzes = l.cruzes.max(cruzamentos.max(1) - 1);
    }
    for k in 0..n {
        if perto[k] || eixo(pts[k]) {
            continue;
        }
        let e = (g[k] - gab[k]).abs() / max;
        l.pior = l.pior.max(e);
        l.errados += usize::from(e > ERRADO);
    }
    l.tv = (tv_g - tv_r) / (hi - lo).max(1e-9);
    l
}

fn fabrica() -> Affine {
    Camera2d::default().world_to_screen_affine(WindowSize::new(FHD.0, FHD.1))
}

/// Texels finos entre duas amostras de um anel.
pub(crate) fn passo_tx(pts: &[[f64; 2]], tx: f64) -> f64 {
    pts.windows(2)
        .map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1]))
        .sum::<f64>()
        / (pts.len() - 1) as f64
        / tx
}

/// A vista no `zoom` com `centro_m` no meio da janela.
pub(crate) fn vista_em(zoom: f64, centro_m: [f64; 2]) -> Affine {
    let f = fabrica();
    let alvo = Point::new(f64::from(FHD.0) * 0.5, f64::from(FHD.1) * 0.5);
    let foco = f * Point::new(centro_m[0], centro_m[1]);
    Affine::translate(alvo.to_vec2()) * Affine::scale(zoom) * Affine::translate(-foco.to_vec2()) * f
}

/// A configuração do produto sem rebote na vista `a`; `controlo` = a W5c (o iso-½, sem a tenda).
fn config(gpu: &ph2d_gpu::GpuContext, a: Affine, controlo: bool) -> Config {
    let dg = DEGRAU_PADRAO;
    let cfg = Config {
        fina: fina_do_quadro(a, dg),
        captura: captura_do_quadro(a, dg),
        holografica: dg.holografica.then_some(HOLOGRAFICA_PADRAO),
        rebote: false,
        ..Config::padrao_para(&gpu.device, grade_do_quadro(a, FHD, dg).expect("grade"))
    };
    if controlo {
        Config {
            borda_exata: false,
            c0_ao_longo_da_face: 0.0,
            ..cfg
        }
    } else {
        cfg
    }
}

/// O teto do excesso de variação num anel do produto — MEDIDO (RTX 5060 Ti, `DEGRAU_PADRAO`): o
/// pior +0,31 (o disco no zoom 1, deslocado 3/8, a 0,9 do raio); o controlo (a W5c) +1,89, com 4
/// cruzamentos a mais numa passagem e o terminador a 5,3 texels do gabarito.
const TETO_TV: f64 = 0.40;

/// ⭐⭐ **GATE — o disco e a elipse saem lisos**: em dois zooms, duas posições na grade e quatro
/// anéis, o produto não tem ponto errado longe das passagens, nem cruzamento a mais numa passagem
/// luz/sombra, nem excesso de variação acima de [`TETO_TV`]; o controlo (a W5c) tem de mostrar as
/// facetas na mesma corrida.
#[test]
fn as_formas_redondas_nao_saem_em_facetas() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let mut falhas = Vec::new();
    let (mut tv_controlo, mut cruzes_controlo) = (0.0_f64, 0usize);
    for zoom in [1.0, 2.5] {
        for ri in 0..2 {
            for k in [0u32, 3] {
                let a = vista_em(zoom, cena_redonda([0.0; 2]).1[ri].centro);
                let tx = {
                    let cfg = config(&gpu, a, false);
                    cfg.grade.texel_m / f64::from(cfg.fina)
                };
                let d = [f64::from(k) * tx / 8.0, f64::from(k) * tx * 3.0 / 8.0];
                let (c, redondos) = cena_redonda(d);
                let r = redondos[ri];
                let ler = |controlo: bool| {
                    let mut s = Solver::novo(&gpu, config(&gpu, a, controlo)).expect("solver");
                    let (i, f) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
                    s.regiao_fina(i, f);
                    s.quadro(&gpu, &c);
                    fluencia_lida(&gpu, &s)
                };
                let (produto, controlo) = (ler(false), ler(true));
                let aneis: Vec<_> = ANEIS
                    .iter()
                    .map(|(nome, f, fora)| (*nome, anel(&c, &r, *f, fora * tx)))
                    .collect();
                let max = aneis
                    .iter()
                    .flat_map(|(_, (_, v))| v.iter().copied())
                    .fold(0.0, f64::max);
                for (nome, (pts, gab)) in &aneis {
                    let passo = passo_tx(pts, tx);
                    let eixo = |p: [f64; 2]| r.ao_eixo(p) < 2.0 * tx;
                    let l = ler_anel(&produto, pts, gab, max, passo, eixo);
                    let l0 = ler_anel(&controlo, pts, gab, max, passo, eixo);
                    tv_controlo = tv_controlo.max(l0.tv);
                    cruzes_controlo = cruzes_controlo.max(l0.cruzes);
                    eprintln!(
                        "redondas · zoom {zoom} · {k}/8 · {} · {nome}: {} errados ({:.3}) · TV {:+.2} · ×{} · ↔{:.1} tx (W5c: TV {:+.2} · ×{})",
                        r.nome, l.errados, l.pior, l.tv, l.cruzes, l.desvio, l0.tv, l0.cruzes
                    );
                    if l.errados > 0 || l.cruzes > 0 || l.tv > TETO_TV {
                        falhas.push(format!(
                            "zoom {zoom} · {k}/8 · {} · {nome}: {} errados · ×{} · TV {:+.2}",
                            r.nome, l.errados, l.cruzes, l.tv
                        ));
                    }
                }
            }
        }
    }
    assert!(
        tv_controlo > 2.5 * TETO_TV && cruzes_controlo > 0,
        "o controlo (a W5c) não mostra as facetas (TV {tv_controlo:+.2}, ×{cruzes_controlo}) — a régua não as vê"
    );
    assert!(
        falhas.is_empty(),
        "as formas redondas saem em facetas: {}",
        falhas.join(" · ")
    );
}

/// O disco da cena 5 (o sprite do disco do atlas) pela captura do PRODUTO, na vista `a`, com `cfg`:
/// `(a fluência, o disco como polígono do raio da textura, a cena do gabarito)` — a borda da textura
/// do disco fica a 127,5/128 do meio-lado.
pub(crate) fn disco_de_imagem(
    gpu: &ph2d_gpu::GpuContext,
    a: Affine,
    cfg: Config,
) -> (Fluencia, Redondo, CenaDeLuz) {
    use ph2d_ecs::{GlobalTransform, PresentWorld, SimRef};
    use ph2d_render::{DISC_TILE_KEY, DISC_TILE_PX, Sprite, disc_tile_pixels};
    let mut w = World::new();
    assert!(smoke::encenar(&mut w, 5));
    let cena = cena_de_luz(&w);
    let corpo = &smoke::CORPOS_DO_VOLUME[0];
    let r = Redondo {
        nome: corpo.nome,
        centro: [f64::from(corpo.centro[0]), f64::from(corpo.centro[1])],
        semi: [0.5 * f64::from(corpo.lado[0]) * 127.5 / 128.0; 2],
    };
    let mut gab = cena_redonda([0.0; 2]).0;
    let i = gab
        .oclusores
        .iter()
        .position(|o| {
            let n = o.vertices.len() as f64;
            let c = o.vertices.iter().fold([0.0_f64; 2], |a, v| {
                [a[0] + f64::from(v[0]) / n, a[1] + f64::from(v[1]) / n]
            });
            (c[0] - r.centro[0]).abs() < 1e-3 && (c[1] - r.centro[1]).abs() < 1e-3
        })
        .expect("o disco no gabarito");
    gab.oclusores[i].vertices = r.poligono();
    let mut rend = crate::comum::renderer(gpu);
    let branca = rend.acquire_individual(1, 1, &[255; 4]).expect("textura");
    let disco: Vec<u8> = disc_tile_pixels(DISC_TILE_PX)
        .into_iter()
        .flat_map(|p| p.0)
        .collect();
    let disco = rend
        .acquire_individual(DISC_TILE_PX, DISC_TILE_PX, &disco)
        .expect("textura do disco");
    let mut present = PresentWorld::new();
    let mut q = w.query_filtered::<(Entity, &Sprite, &GlobalTransform), bevy_ecs::query::With<ph2d_light2d_ecs::BloqueiaLuz>>();
    for (e, s, g) in q.iter(&w) {
        let [ax, ay, bx, by, tx, ty] = g.affine();
        let e_disco =
            matches!(s.source, ph2d_render::SpriteSource::Atlas { key } if key == DISC_TILE_KEY);
        let mut i = crate::comum::quad(if e_disco { disco } else { branca }, [tx, ty]);
        i.size = s.size;
        i.anchor = s.anchor;
        i.basis = [ax, ay, bx, by];
        present.world_mut().spawn((i, SimRef(e)));
    }
    let mut s = Solver::novo(gpu, cfg).expect("solver");
    let (i0, f0) = regiao_da_janela(s.grade_da_fluencia(), a, FHD);
    s.regiao_fina(i0, f0);
    let capt = ph2d_app_light2d::sprites_da_cena(&cena, &mut present, &rend);
    let mut enc = gpu.device.create_command_encoder(&Default::default());
    s.gravar_com_sprites(gpu, &mut enc, &cena, &capt);
    gpu.queue.submit([enc.finish()]);
    (fluencia_lida(gpu, &s), r, gab)
}

/// O teto do excesso de variação num anel do disco de IMAGEM no zoom 1 — MEDIDO (RTX 5060 Ti,
/// `DEGRAU_PADRAO`): o pior +0,66 (a 0,9 do raio); o controlo (a W5c) +1,30 com 2 cruzamentos a mais.
/// ⚠️ No zoom 2,5 (a textura ampliada) o produto ainda tem 2 cruzamentos a mais (W5c: 4) — ABERTO.
const TETO_TV_IMAGEM: f64 = 0.80;

/// ⭐⭐ **GATE — o disco de IMAGEM sai liso** na vista do dono (zoom 1), pela captura do produto (o
/// sprite do atlas): a cobertura do sprite pelo meio (a área da forma alfa ≥ ½) é o que a borda exata
/// lê; com a média do alfa (a W5c sem ela) a borda saía em zigue-zague.
#[test]
fn o_disco_de_imagem_nao_sai_em_facetas() {
    let Some(gpu) = gpu() else {
        eprintln!("sem GPU — gate saltado");
        return;
    };
    let a = vista_em(1.0, cena_redonda([0.0; 2]).1[0].centro);
    let produto = config(&gpu, a, false);
    let tx = produto.grade.texel_m / f64::from(produto.fina);
    let (fl, r, gab_cena) = disco_de_imagem(&gpu, a, produto);
    let (fl0, ..) = disco_de_imagem(&gpu, a, config(&gpu, a, true));
    let aneis: Vec<_> = ANEIS
        .iter()
        .map(|(nome, f, fora)| (*nome, anel(&gab_cena, &r, *f, fora * tx)))
        .collect();
    let max = aneis
        .iter()
        .flat_map(|(_, (_, v))| v.iter().copied())
        .fold(0.0, f64::max);
    let (mut falhas, mut cruzes_controlo) = (Vec::new(), 0);
    for (nome, (pts, gab)) in &aneis {
        let passo = passo_tx(pts, tx);
        let eixo = |p: [f64; 2]| r.ao_eixo(p) < 2.0 * tx;
        let l = ler_anel(&fl, pts, gab, max, passo, eixo);
        let l0 = ler_anel(&fl0, pts, gab, max, passo, eixo);
        cruzes_controlo = cruzes_controlo.max(l0.cruzes);
        eprintln!(
            "disco de imagem · {nome}: {} errados ({:.3}) · TV {:+.2} · ×{} (W5c: {} · TV {:+.2} · ×{})",
            l.errados, l.pior, l.tv, l.cruzes, l0.errados, l0.tv, l0.cruzes
        );
        if l.errados > 0 || l.cruzes > 0 || l.tv > TETO_TV_IMAGEM {
            falhas.push(format!(
                "{nome}: {} errados · ×{} · TV {:+.2}",
                l.errados, l.cruzes, l.tv
            ));
        }
    }
    assert!(
        cruzes_controlo > 0,
        "o controlo (a W5c) não mostra a passagem serrilhada — a régua não a vê"
    );
    assert!(
        falhas.is_empty(),
        "o disco de imagem sai em facetas: {}",
        falhas.join(" · ")
    );
}
