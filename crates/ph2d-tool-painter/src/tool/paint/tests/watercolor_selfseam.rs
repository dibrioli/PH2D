//! **A costura do retorno sobre o próprio traço** (doc 40, report do Enio 2026-09-20).
//!
//! Com `Charge < 1` o pincel perde carga ao longo do traço. Quando o MESMO traço volta sobre si
//! (sem pen-up), a perna de volta — mais pálida — encosta na perna de ida — mais escura — e a
//! fronteira entre os dois NÍVEIS de reserva fica à vista. Ela era um degrau de poucos pixels lido
//! por vizinho-mais-próximo em coordenadas deformadas pelo Ragged Edge: *«borda plana dura
//! pixelada»*.
//!
//! ## A régua, e de onde sai a barra
//!
//! O oráculo (`docs/Painter/ferramentas/oraculo_costura/`, libmypaint 1.6.1 — ISC — corrido sem
//! interface sobre um traço em U NOSSO) mede que, num motor de dab macio, a costura interna tem a
//! MESMA largura 10–90 % da borda externa do traço (razão `1,00` em quatro durezas). A barra sai
//! desse lado aprovado: **a costura interna não pode ser mais DURA do que a borda externa do mesmo
//! traço** — medido por linha de varredura, com a inclinação normalizada pelo contraste de cada
//! transição (`max |ΔI| / contraste`), porque um degrau de 40 bytes e um de 200 não se comparam em
//! bytes por pixel.
//!
//! ⚠️ A fixtura usa o **preset do produto** (`apply_brush_preset(1)`, Ragged Edge `6 px` incluído):
//! sem o warp a escada de pixel não existe e a régua mediria outro programa. Só a granulação é
//! zerada — ela é ruído multiplicativo por pixel e infla `max |ΔI|` exatamente onde o contraste é
//! menor (a costura), que é o lado que a barra julga.

use super::*;

/// Geometria do U: desce em `xa`, vira, sobe em `xb = xa + 1.2·r` (as pernas sobrepõem-se `0.8·r`).
#[derive(Clone, Copy)]
pub(super) struct UStroke {
    pub size: u32,
    pub r: f32,
    pub xa: f32,
    pub xb: f32,
    pub y0: f32,
    pub y1: f32,
}

impl UStroke {
    pub(super) fn new(r: f32) -> Self {
        let size = 512u32;
        let xa = 200.0;
        Self {
            size,
            r,
            xa,
            xb: xa + 1.2 * r,
            y0: 90.0,
            y1: 420.0,
        }
    }

    fn travel(&self) -> f32 {
        2.0 * (self.y1 - self.y0) + (self.xb - self.xa)
    }
}

/// Os knobs que a sonda varre. `span_factor` = quantas vezes o comprimento do U cabe na reserva
/// (`1.5` ⇒ a perna de ida lê ~0,7 a meia altura e a de volta ~0,25).
#[derive(Clone, Copy)]
pub(super) struct SeamKnobs {
    pub span_factor: f32,
    pub step_px: f32,
    pub rewet: f32,
    pub smudge: f32,
    pub granulation: f32,
    /// `false` tira o papel do preset: o dente dele é textura de ~10 px com inclinação própria, e a
    /// raios grandes (onde a costura curada é SUAVE) era ele que a régua media, não a costura.
    pub paper: bool,
    /// Depois do U, SEM pen-up, esfrega de lado a lado por cima da costura (o gesto do Smudge).
    pub scrub: bool,
}

impl Default for SeamKnobs {
    fn default() -> Self {
        Self {
            span_factor: 1.5,
            step_px: 4.0,
            rewet: 0.0,
            smudge: 0.0,
            granulation: 0.0,
            paper: false,
            scrub: false,
        }
    }
}

/// Pinta o U com o preset de aquarela do PRODUTO e devolve a ferramenta já com o pen-up feito.
pub(super) fn paint_u(u: UStroke, k: SeamKnobs) -> PainterTool {
    let mut t = paint_u_live(u, k);
    let end = if k.scrub { SCRUB_END } else { [u.xb, u.y0] };
    t.on_canvas_pointer(cp(end, PointerPhase::Up));
    for _ in 0..4 {
        frame(&mut t);
    }
    t
}

/// A faixa que o esfregão cobre (linhas) e onde ele acaba — longe da faixa que a régua base lê.
pub(super) const SCRUB_ROWS: (u32, u32) = (120, 170);
const SCRUB_END: [f32; 2] = [200.0, 170.0];

/// O mesmo U com a caneta AINDA em baixo (o composite vivo, incremental).
pub(super) fn paint_u_live(u: UStroke, k: SeamKnobs) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (u.size * u.size * 4) as usize], u.size, u.size);
    t.paint.brush.radius_px = u.r;
    t.paint.brush.color = [0.85, 0.12, 0.10];
    t.apply_brush_preset(1);
    // span = MIX_DEPLETE_SPAN(120)·r·c/(1−c)  ⇒  c = s/(1+s), s = span/(120·r).
    let s = k.span_factor * u.travel() / (120.0 * u.r);
    t.paint.brush.wet_charge = s / (1.0 + s);
    t.paint.brush.wet_rewet = k.rewet;
    t.paint.brush.wet_smudge = k.smudge;
    t.paint.brush.granulation = k.granulation;
    if !k.paper {
        t.paint.brush.paper = ph2d_painter_brush::TextureSettings::default();
    }
    let seed = t.paint.brush;
    t.paint.brush_by_mode.fill(seed);

    assert!(t.on_canvas_pointer(cp([u.xa, u.y0], PointerPhase::Down)));
    let mut n = 0u32;
    let mut go = |t: &mut PainterTool, x: f32, y: f32| {
        t.on_canvas_pointer(cp([x, y], PointerPhase::Move));
        n += 1;
        if n.is_multiple_of(2) {
            frame(t);
        }
    };
    let mut y = u.y0;
    while y < u.y1 {
        y = (y + k.step_px).min(u.y1);
        go(&mut t, u.xa, y);
    }
    let mut x = u.xa;
    while x < u.xb {
        x = (x + k.step_px).min(u.xb);
        go(&mut t, x, u.y1);
    }
    while y > u.y0 {
        y = (y - k.step_px).max(u.y0);
        go(&mut t, u.xb, y);
    }
    if k.scrub {
        // Vai-e-vem HORIZONTAL por cima da costura, descendo 6 px por passada.
        let (xl, xr) = (u.xa + 0.2 * u.r, u.xa + 1.6 * u.r);
        let mut yy = SCRUB_ROWS.0 as f32;
        go(&mut t, u.xb, yy);
        while yy <= SCRUB_ROWS.1 as f32 {
            let mut x = xr;
            while x > xl {
                x = (x - k.step_px).max(xl);
                go(&mut t, x, yy);
            }
            yy += 6.0;
            while x < xr {
                x = (x + k.step_px).min(xr);
                go(&mut t, x, yy);
            }
            yy += 6.0;
        }
        go(&mut t, SCRUB_END[0], SCRUB_END[1]);
    }
    t
}

/// Corre `f` com a lei de ANTES (o CONTROLO — ver `watercolor_reserve::LEI_ANTIGA`).
pub(super) fn with_the_old_law<T>(f: impl FnOnce() -> T) -> T {
    use crate::tool::paint::watercolor_reserve::LEI_ANTIGA;
    LEI_ANTIGA.with(|c| c.set(true));
    let out = f();
    LEI_ANTIGA.with(|c| c.set(false));
    out
}

/// O que uma linha de varredura diz das duas transições.
///
/// ⚠️ **A grandeza é a LARGURA EFECTIVA** `contraste / maior inclinação`, com a linha alisada por uma
/// caixa de 5 px e a inclinação medida num vão de 4 px. A 1.ª redacção media `max |ΔI|` cru e batia
/// num PISO DE RUÍDO: o dente do papel do preset mexe ~5 bytes por pixel, que sobre um contraste de
/// ~38 bytes lê `0,13` — a costura curada lia `0,15` e a régua não via mais nada. O alisamento põe o
/// chão em 5 px (um degrau duro lê exactamente isso), que é a resolução declarada desta régua.
#[derive(Clone, Copy, Default, Debug)]
pub(super) struct RowSeam {
    /// largura efectiva da costura interna, em px.
    pub seam_w: f32,
    /// idem para a borda externa esquerda (papel → aro da perna de ida).
    pub outer_w: f32,
    /// contraste da costura, em bytes do canal G.
    pub contrast: f32,
}

/// A folga de `a_single_pass_keeps_the_look_it_had`, em bytes do canal G (medida — ver o gate).
const SINGLE_PASS_SLACK: i32 = 16; // medido: 11 (a casca da beira, bilinear contra vizinho-mais-proximo)

/// O chão da régua: um degrau perfeito alisado por 5 e derivado num vão de 4 lê isto.
pub(super) const RULER_FLOOR_PX: f32 = 5.0;

fn g_at(t: &PainterTool, size: u32, x: i32, y: u32) -> f32 {
    let x = x.clamp(0, size as i32 - 1) as u32;
    f32::from(px(t, size, x, y)[1])
}

fn median(mut v: Vec<f32>) -> f32 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

pub(super) fn measure_row(t: &PainterTool, u: UStroke, y: u32) -> RowSeam {
    let span = |a: f32, b: f32| -> Vec<f32> {
        let (a, b) = (a.round() as i32, b.round() as i32);
        (a..=b).map(|x| g_at(t, u.size, x, y)).collect()
    };
    let i1 = median(span(u.xa - 0.5 * u.r, u.xa + 0.05 * u.r));
    let i2 = median(span(u.xa + 1.35 * u.r, u.xa + 1.9 * u.r));
    let contrast = (i2 - i1).abs().max(1.0);
    // Caixa de 5 px e inclinação num vão de 4: o ruído do papel sai, um degrau duro lê 5 px.
    let slope = |v: &[f32]| -> f32 {
        let s: Vec<f32> = v.windows(5).map(|w| w.iter().sum::<f32>() / 5.0).collect();
        s.windows(5)
            .map(|w| (w[4] - w[0]).abs() / 4.0)
            .fold(1e-3, f32::max)
    };
    let seam = span(u.xa + 0.2 * u.r - 4.0, u.xa + 1.3 * u.r + 4.0);
    let outer = span(u.xa - u.r - 16.0, u.xa - 0.55 * u.r);
    let outer_contrast = (255.0 - outer.iter().copied().fold(255.0, f32::min)).max(1.0);
    RowSeam {
        seam_w: contrast / slope(&seam),
        outer_w: outer_contrast / slope(&outer),
        contrast,
    }
}

/// A régua agregada: medianas sobre as linhas da faixa do meio (longe da virada e das pontas).
#[derive(Clone, Copy, Default, Debug)]
pub(super) struct SeamReport {
    /// largura efectiva da costura (px) e da borda externa (px), medianas sobre as linhas.
    pub seam_w: f32,
    pub outer_w: f32,
    /// `costura / externa` — `≥ 1` = a costura não é mais dura do que a borda do próprio traço.
    pub ratio: f32,
    pub contrast: f32,
}

pub(super) fn measure(t: &PainterTool, u: UStroke) -> SeamReport {
    measure_rows(t, u, (190..=320).step_by(3).collect())
}

pub(super) fn measure_rows(t: &PainterTool, u: UStroke, ys: Vec<u32>) -> SeamReport {
    let rows: Vec<RowSeam> = ys.iter().map(|&y| measure_row(t, u, y)).collect();
    let seam_w = median(rows.iter().map(|r| r.seam_w).collect());
    let outer_w = median(rows.iter().map(|r| r.outer_w).collect());
    SeamReport {
        seam_w,
        outer_w,
        ratio: seam_w / outer_w.max(1e-6),
        contrast: median(rows.iter().map(|r| r.contrast).collect()),
    }
}

/// `PH2D_SEAM_DUMP=<dir>` grava o recorte em PPM (P6) — a régua não decide sem o desenho ao lado.
fn dump(t: &PainterTool, u: UStroke, label: &str) {
    let Ok(dir) = std::env::var("PH2D_SEAM_DUMP") else {
        return;
    };
    let (x0, x1) = ((u.xa - 2.0 * u.r) as u32, (u.xb + 2.0 * u.r) as u32);
    let (y0, y1) = (150u32, 360u32);
    let (w, h) = (x1 - x0, y1 - y0);
    let mut out = format!("P6\n{w} {h}\n255\n").into_bytes();
    for y in y0..y1 {
        for x in x0..x1 {
            let p = px(t, u.size, x.min(u.size - 1), y);
            out.extend_from_slice(&p[..3]);
        }
    }
    let _ = std::fs::create_dir_all(&dir);
    let _ = std::fs::write(format!("{dir}/{label}.ppm"), out);
}

/// **Sonda** — a tabela que o doc 40 e o plano citam. `cargo test … -- --ignored --nocapture`.
#[test]
#[ignore = "sonda de medicao: imprime a regua da costura"]
fn measure_the_self_seam() {
    eprintln!(
        "\n=== A COSTURA DO RETORNO (U, pernas a 1,2 r; preset do produto, Ragged Edge 6) ==="
    );
    eprintln!("    r  knobs                    costura px  (em r)  externa px   razao  contraste");
    for r in [8.0f32, 20.0, 32.0, 45.0, 96.0] {
        let u = UStroke::new(r);
        for (label, k) in [
            ("base", SeamKnobs::default()),
            (
                "preset (papel+gran)",
                SeamKnobs {
                    granulation: 0.30,
                    paper: true,
                    ..Default::default()
                },
            ),
            (
                "rewet1",
                SeamKnobs {
                    rewet: 1.0,
                    ..Default::default()
                },
            ),
            (
                "smudge1",
                SeamKnobs {
                    smudge: 1.0,
                    ..Default::default()
                },
            ),
        ] {
            let t = paint_u(u, k);
            let m = measure(&t, u);
            let old = measure(&with_the_old_law(|| paint_u(u, k)), u);
            eprintln!(
                "  {r:4.0}  {label:<22} {:9.2} {:8.3} {:10.2} {:8.2} {:9.1}   | lei antiga: {:6.2} px",
                m.seam_w,
                m.seam_w / r,
                m.outer_w,
                m.ratio,
                m.contrast,
                old.seam_w
            );
            dump(&t, u, &format!("r{r:.0}_{label}"));
        }
    }
}

// ── OS GATES ────────────────────────────────────────────────────────────────────────────────────

/// **A costura do retorno não é mais dura do que a borda do próprio traço — e tem a escala do
/// pincel.** A disputa cede ao longo da cauda do feather (`0,38·r`), logo a largura efectiva é
/// `(1 − v₂/v₁)·0,38·r ≈ 0,25·r` nesta fixtura; a barra de `0,20·r` deixa a folga da óptica (o byte
/// não é linear na reserva). CONTROLO dentro do gate: a lei de antes lê o CHÃO da régua.
#[test]
fn the_return_seam_is_as_soft_as_the_brush_not_a_pixel_step() {
    for r in [32.0f32, 45.0, 96.0] {
        let u = UStroke::new(r);
        let new = measure(&paint_u(u, SeamKnobs::default()), u);
        let old = measure(&with_the_old_law(|| paint_u(u, SeamKnobs::default())), u);
        assert!(
            new.contrast > 20.0,
            "r={r}: a fixtura perdeu o contraste ({})",
            new.contrast
        );
        assert!(
            new.seam_w >= 0.20 * r && new.ratio >= 1.25,
            "r={r}: costura {:.2} px ({:.3} r), razão {:.2} contra a borda externa",
            new.seam_w,
            new.seam_w / r,
            new.ratio
        );
        // A `r = 32` o degrau antigo (`~3 px`) lê o CHÃO da régua; a raios maiores é a rampa de 15 %.
        assert!(
            r > 32.0 || old.seam_w <= 1.4 * RULER_FLOOR_PX,
            "chão: {}",
            old.seam_w
        );
        assert!(
            old.seam_w < 0.20 * r && old.seam_w < 0.75 * new.seam_w,
            "r={r}: o CONTROLO deixou de conter o fenómeno — lei antiga {:.2} px, nova {:.2} px",
            old.seam_w,
            new.seam_w
        );
    }
}

/// **O Rewet alcança a costura do próprio traço**, e à escala do pincel (a cauda da disputa
/// alarga, não só o raio em pixels do alisamento — que a `r = 96` não se via: `22,0 → 22,1 px`).
#[test]
fn rewet_widens_the_return_seam_at_every_brush_size() {
    for r in [32.0f32, 96.0] {
        let u = UStroke::new(r);
        let dry = measure(&paint_u(u, SeamKnobs::default()), u);
        let wet = SeamKnobs {
            rewet: 1.0,
            ..Default::default()
        };
        let wet = measure(&paint_u(u, wet), u);
        assert!(
            wet.seam_w >= 1.25 * dry.seam_w,
            "r={r}: Rewet 1 deu {:.2} px contra {:.2} a seco",
            wet.seam_w,
            dry.seam_w
        );
    }
}

/// **O Smudge alcança o traço VIVO.** Medido antes da cura: Smudge `1,0` deixava o traço
/// byte-idêntico ao de Smudge `0` (a esfregada só arrastava a tinta ASSADA por baixo, e num papel
/// em branco não há nenhuma). Esfregando de lado a lado por cima da costura, sem pen-up, ela tem de
/// abrir — medido `10,4 → 33,1 px`.
#[test]
fn smudge_reaches_the_seam_of_the_live_stroke() {
    let u = UStroke::new(32.0);
    let scrubbed = |smudge: f32| {
        paint_u(
            u,
            SeamKnobs {
                smudge,
                scrub: true,
                ..Default::default()
            },
        )
    };
    let (off, on) = (scrubbed(0.0), scrubbed(1.0));
    let band: Vec<u32> = (SCRUB_ROWS.0 + 8..SCRUB_ROWS.1 - 8).step_by(3).collect();
    let (w_off, w_on) = (
        measure_rows(&off, u, band.clone()).seam_w,
        measure_rows(&on, u, band).seam_w,
    );
    assert!(
        w_on >= 1.3 * w_off,
        "esfregar não abriu a costura: {w_on:.2} px com Smudge contra {w_off:.2} sem"
    );
    // CONTROLO: fora da faixa esfregada as pernas correm PARALELAS à costura, e arrastar ao longo
    // dela não a abre — a régua lê o mesmo com e sem Smudge. (Não é «nada muda»: o Smudge arrasta o
    // gradiente da depleção ao longo do traço inteiro; o que ele não faz é abrir o que não cruzou.)
    let (p_off, p_on) = (measure(&off, u).seam_w, measure(&on, u).seam_w);
    assert!(
        (p_on / p_off - 1.0).abs() < 0.25,
        "fora do esfregão a costura devia ficar como estava: {p_on:.2} contra {p_off:.2}"
    );
}

/// **O traço é facto do CAMINHO, não de como os dabs chegam em LOTES.** A esfregada dos níveis
/// corre POR DAB dentro do depósito (nunca por lote, como a da base): a mesma lista de dabs entregue
/// de uma vez ou em três lotes deixa os DOIS planos iguais ao byte. É a forma testável da
/// independência do polling — ⚠️ a nível de EVENTOS a fixtura não serve: o preset tem dinâmica de
/// velocidade, e 4 px contra 11 px por evento já diferem `Δ226` a Charge 1, sem mixer nenhum
/// (sonda `measure_polling_and_smudge_reach`).
#[test]
fn the_level_planes_do_not_depend_on_how_the_dabs_are_batched() {
    let u = UStroke::new(24.0);
    let mut path: Vec<[f32; 2]> = Vec::new();
    let mut y = 90.0f32;
    while y < 300.0 {
        path.push([u.xa, y]);
        y += 3.0;
    }
    let mut x = u.xa;
    while x < u.xb {
        path.push([x, 300.0]);
        x += 3.0;
    }
    while y > 90.0 {
        path.push([u.xb, y]);
        y -= 3.0;
    }
    // ...e de volta por cima da costura, para o Smudge ter o que arrastar.
    for i in 0..30 {
        path.push([u.xb - 2.0 * i as f32, 150.0 + i as f32]);
    }
    let dabs: Vec<Dab> = path
        .iter()
        .enumerate()
        .map(|(i, &c)| Dab {
            center: c,
            radius_px: u.r,
            coverage: 1.0,
            color: [0.85, 0.12, 0.10],
            rotation: [1.0, 0.0],
            dir: [0.0, 1.0],
            arc_len: 3.0 * i as f32,
            stroke_radius_px: u.r,
        })
        .collect();
    let planes = |cuts: &[usize]| {
        let mut t = PainterTool::default();
        t.set_source(vec![255u8; (u.size * u.size * 4) as usize], u.size, u.size);
        t.paint.brush.radius_px = u.r;
        t.apply_brush_preset(1);
        t.paint.brush.wet_charge = 0.4;
        t.paint.brush.wet_smudge = 0.7;
        let seed = t.paint.brush;
        t.paint.brush_by_mode.fill(seed);
        assert!(t.on_canvas_pointer(cp(path[0], PointerPhase::Down)));
        let mut from = 0;
        for &to in cuts.iter().chain([dabs.len()].iter()) {
            t.stamp_dabs(&dabs[from..to]);
            from = to;
        }
        (
            t.paint.stroke_deplete.clone(),
            t.paint.stroke_deplete_prox.clone(),
        )
    };
    let (one, three) = (planes(&[]), planes(&[37, 38, 151]));
    assert!(
        one.0.iter().filter(|&&v| v > 0).count() > 5_000,
        "piso de população"
    );
    assert!(
        one == three,
        "os planos da reserva mudaram com o corte dos lotes"
    );
}

/// **Incremental ≡ full com o mixer LIGADO** — a irmã de
/// `watercolor_incremental_composite_matches_full_recompose`, que corre a Charge 1 e por isso nunca
/// constrói o campo da reserva. O campo lê vizinhança (`R`), logo é ele que pode deixar pixel stale —
/// e com Rewet o raio passa do Bleed, que é quando a janela tem de o saber (`reserve_reach`).
///
/// ⚠️ **As barras saem da sonda `measure_who_owns_the_stale_pixel`, com o CONTROLO ao lado** (o pior
/// byte, nova lei · lei antiga): fixtura nua `Δ1 · Δ1` — é a barra dura; papel + granulação `Δ2 · Δ2`,
/// e **a Charge 1 também `Δ2`** (é o `f32` dos campos da casa, somado desde a origem da janela, e não
/// é desta wave); Rewet 1 `Δ2 · Δ1` em QUATRO bytes da imagem. O campo em si é exacto e independente
/// da janela AO BIT (gate `the_field_is_a_function_of_the_map_not_of_the_window`), logo o que ele
/// muda é ONDE o ruído de `±1` dos vizinhos aterra. ⛔ A `r = 96` esta fixtura NÃO serve de régua:
/// lê `Δ90`–`Δ160` em ~12 mil bytes nas duas leis **e a Charge 1** — achado pré-existente, nomeado
/// no handoff, fora desta wave.
/// **SONDA — a escada do raio, e o defeito PRÉ-EXISTENTE que ela achou.**
///
/// Varre o `stale` (incremental contra cheio) por raio de pincel em TRÊS colunas: molhado · seco ·
/// **seco com a LEI ANTIGA**. A terceira é o CONTROLO que DATA o defeito — sem ela um pico desta
/// escada lê-se como dívida desta wave, e não é.
///
/// Medido em 2026-09-20, máquina calma:
///
/// | `r` | molhado | seco | LEI ANTIGA, seco |
/// |---|---|---|---|
/// | 88  | 161 | 94 | **94** |
/// | 96  | 139 | 93 | **90** |
/// | 120 | 2   | 2  | **2** |
///
/// ⛔ Os picos a `88` e `96` **não são desta wave**: aparecem a SECO — onde o campo da reserva mal
/// participa, porque o `reach` é o `core_any` e o raio do campo é `~9` — e a lei antiga lê o mesmo
/// número. É o **raio de invalidação** que os dois `watercolor_app_params_incremental_*` já
/// declaram `#[ignore]`, e cuja nota diz por escrito que `pad += 2·raio` **não** é a cura.
/// ⚠️ É por isso que o gate irmão mede a `120`: *uma barra posta num raio onde outro defeito já
/// vive não afirma nada sobre este*.
#[test]
#[ignore = "sonda: a escada do raio — escolhe onde a janela deixa de cobrir o campo, e data o pico pré-existente"]
fn diag_a_escada_do_raio_da_janela() {
    let stale = |r: f32, k: SeamKnobs| -> i32 {
        let mut t = paint_u_live(UStroke::new(r), k);
        let incremental: Vec<u8> = t.canvas_rgba.to_vec();
        t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
        t.apply_watercolor(false);
        incremental
            .iter()
            .zip(t.canvas_rgba.iter())
            .map(|(a, b)| (i32::from(*a) - i32::from(*b)).abs())
            .max()
            .unwrap()
    };
    let wet = SeamKnobs {
        rewet: 1.0,
        ..SeamKnobs::default()
    };
    let seco = SeamKnobs::default();
    for r in [88.0f32, 96.0, 120.0] {
        eprintln!(
            "[escada] r={r} molhado={} seco={} LEI-ANTIGA-seco={}",
            stale(r, wet),
            stale(r, seco),
            with_the_old_law(|| stale(r, seco))
        );
    }
}

#[test]
fn the_reserve_field_keeps_incremental_equal_to_full() {
    let stale = |r: f32, k: SeamKnobs| -> i32 {
        let mut t = paint_u_live(UStroke::new(r), k);
        let incremental: Vec<u8> = t.canvas_rgba.to_vec();
        t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
        t.apply_watercolor(false);
        incremental
            .iter()
            .zip(t.canvas_rgba.iter())
            .map(|(a, b)| (i32::from(*a) - i32::from(*b)).abs())
            .max()
            .unwrap()
    };
    let bare = SeamKnobs::default();
    let (dry, wet) = (stale(32.0, bare), SeamKnobs { rewet: 1.0, ..bare });
    assert!(dry <= 1, "pixel stale na fixtura nua: Δ{dry}");
    let wet32 = stale(32.0, wet);
    assert!(wet32 <= 2, "Rewet 1 deixou pixel stale: Δ{wet32}");
    // ⛔⛔ **ESTE GATE NÃO TESTEMUNHA O `reserve_reach` DA JANELA, e a 1.ª redacção dizia que sim.**
    // Ela trazia escrito *«sem o `reserve_reach` no alcance dela, é aqui que o pixel fica velho»* e
    // uma prova de mutação refutou-a: apagar aquele termo deixa ESTE gate **verde**, a `r = 80` e a
    // `r = 120`. Medido: a janela reserva `pad = reach + ceil(warp) + 2` e o campo é construído na
    // janela de LEITURA e amostrado na região de SAÍDA, logo a margem real é o `pad`. Sem o termo o
    // `pad` vale `14 + 6 + 2 = 22` contra um campo que pede `R = 20` (`r = 80`) e `R = 30`
    // (`r = 120`) — ou seja, **no segundo a margem É deficiente e a imagem final ainda concorda a
    // dois níveis**: o erro do campo mora a `22 px` de qualquer dab daquele quadro, onde a lavagem
    // já não pesa, e o quadro seguinte reescreve por cima.
    // ⇒ quem afirma aquela metade são [`super::super::watercolor_reserve::tests::
    // a_caixa_truncada_le_outro_campo`] (a premissa: margem abaixo de `R` muda o campo) e
    // [`a_janela_do_composite_reserva_o_raio_do_campo`] (a fiação). O que ESTE gate afirma é o que
    // o nome diz: incremental ≡ cheio, e o `r = 120` está aqui como o Rewet mais largo que a
    // fixtura suporta, não como testemunha da janela.
    // ⛔ E a escada do raio NÃO é monótona por uma razão que não é desta wave — a `r = 88` e `96`
    // esta mesma medição lê `93`–`94` **a seco e com a LEI ANTIGA**: a sonda irmã
    // [`diag_a_escada_do_raio_da_janela`] tem a tabela e o controlo que o datam.
    let wet120 = stale(120.0, wet);
    assert!(
        wet120 <= 2,
        "r=120: Rewet largo deixou pixel stale: Δ{wet120}"
    );
    eprintln!("[incremental≡full] nu Δ{dry} · Rewet r=32 Δ{wet32} · Rewet r=120 Δ{wet120}");
}

/// **Uma passada só fica com a cara que tinha.** A disputa cancela (o mesmo dab ganha em cima e em
/// baixo) e o afilamento da beira é o de sempre, logo a nova lei e a antiga desenham o mesmo traço
/// recto — é a anatomia da borda externa que o dono aprovou. A folga é a da leitura bilinear contra
/// a de vizinho-mais-próximo na casca de 15 % do raio, medida.
#[test]
fn a_single_pass_keeps_the_look_it_had() {
    let straight = || {
        let u = UStroke::new(32.0);
        let line = UStroke {
            y1: 91.0,
            xb: u.xa + 260.0,
            ..u
        };
        let mut t = paint_u_live(line, SeamKnobs::default());
        t.on_canvas_pointer(cp([line.xb, 91.0], PointerPhase::Up));
        t
    };
    let (new, old) = (straight(), with_the_old_law(straight));
    let (mut worst, mut painted) = (0, 0usize);
    for (a, b) in new.canvas_rgba.chunks(4).zip(old.canvas_rgba.chunks(4)) {
        painted += usize::from(a[1] < 250);
        worst = worst.max((i32::from(a[1]) - i32::from(b[1])).abs());
    }
    assert!(painted > 10_000, "piso de população: {painted} px pintados");
    assert!(
        worst <= SINGLE_PASS_SLACK,
        "uma passada só mudou de cara: Δ{worst}"
    );
    eprintln!("[uma passada] pior byte nova x antiga: Δ{worst} em {painted} px");
}

/// **Sonda** — de quem é a dependência do polling, e quanto o Smudge mexe fora da faixa esfregada.
#[test]
#[ignore = "sonda de medicao"]
fn measure_polling_and_smudge_reach() {
    let u = UStroke::new(32.0);
    let worst = |a: &PainterTool, b: &PainterTool, y0: u32, y1: u32| -> i32 {
        let mut w = 0;
        for y in y0..y1 {
            for x in 0..u.size {
                let (p, q) = (px(a, u.size, x, y), px(b, u.size, x, y));
                for c in 0..3 {
                    w = w.max((i32::from(p[c]) - i32::from(q[c])).abs());
                }
            }
        }
        w
    };
    eprintln!("\n=== POLLING: eventos de 4 px contra 11 px (pior byte, imagem inteira) ===");
    for (label, smudge, rewet, scrub, charge1) in [
        ("U, mixer, sem knobs", 0.0, 0.0, false, false),
        ("U + esfregao", 0.0, 0.0, true, false),
        ("U + esfregao, CHARGE 1", 0.0, 0.0, true, true),
        ("U + esfregao + smudge .6", 0.6, 0.0, true, false),
        ("U + esfregao + smudge .6, CHARGE 1", 0.6, 0.0, true, true),
        ("U + esfregao + rewet .4", 0.0, 0.4, true, false),
    ] {
        let at = |step_px: f32| {
            let k = SeamKnobs {
                step_px,
                smudge,
                rewet,
                scrub,
                span_factor: if charge1 { 1.0e9 } else { 1.5 },
                ..Default::default()
            };
            paint_u(u, k)
        };
        let (a, b) = (at(4.0), at(11.0));
        eprintln!("  {label:<38} Δ{}", worst(&a, &b, 0, u.size));
    }
    eprintln!("\n=== SMUDGE 0 contra 1 (mesmo caminho com esfregao) ===");
    let s = |smudge: f32| {
        paint_u(
            u,
            SeamKnobs {
                smudge,
                scrub: true,
                ..Default::default()
            },
        )
    };
    let (off, on) = (s(0.0), s(1.0));
    eprintln!(
        "  pior byte longe do esfregao (y 230..400): Δ{}",
        worst(&off, &on, 230, 400)
    );
    eprintln!(
        "  pior byte na faixa esfregada:            Δ{}",
        worst(&off, &on, SCRUB_ROWS.0, SCRUB_ROWS.1)
    );
    let band: Vec<u32> = (SCRUB_ROWS.0 + 8..SCRUB_ROWS.1 - 8).step_by(3).collect();
    eprintln!(
        "  costura na faixa: sem {:.2} px · com {:.2} px",
        measure_rows(&off, u, band.clone()).seam_w,
        measure_rows(&on, u, band).seam_w
    );
}

/// **Sonda** — de quem é o pixel stale do incremental: do campo da reserva, ou já lá estava?
#[test]
#[ignore = "sonda de medicao"]
fn measure_who_owns_the_stale_pixel() {
    let stale = |u: UStroke, k: SeamKnobs| -> (i32, usize) {
        let mut t = paint_u_live(u, k);
        let inc: Vec<u8> = t.canvas_rgba.to_vec();
        t.paint.wet_frame_dirty = t.paint.wet_cum_dirty;
        t.apply_watercolor(false);
        let d: Vec<i32> = inc
            .iter()
            .zip(t.canvas_rgba.iter())
            .map(|(a, b)| (i32::from(*a) - i32::from(*b)).abs())
            .collect();
        (
            *d.iter().max().unwrap(),
            d.iter().filter(|&&v| v > 1).count(),
        )
    };
    eprintln!("\n=== INCREMENTAL x FULL no U (pior byte, n.o de bytes > 1) ===");
    for r in [32.0f32, 48.0, 64.0, 80.0, 96.0] {
        let u = UStroke::new(r);
        for (label, k) in [
            ("nu (sem papel, sem gran)", SeamKnobs::default()),
            (
                "papel + gran",
                SeamKnobs {
                    paper: true,
                    granulation: 0.3,
                    ..Default::default()
                },
            ),
            (
                "CHARGE 1, papel + gran",
                SeamKnobs {
                    paper: true,
                    granulation: 0.3,
                    span_factor: 1.0e9,
                    ..Default::default()
                },
            ),
            (
                "rewet 1",
                SeamKnobs {
                    rewet: 1.0,
                    ..Default::default()
                },
            ),
        ] {
            let new = stale(u, k);
            let old = with_the_old_law(|| stale(u, k));
            eprintln!(
                "  r={r:3.0} {label:<28} nova lei Δ{} ({} bytes) · lei antiga Δ{} ({} bytes)",
                new.0, new.1, old.0, old.1
            );
        }
    }
}
