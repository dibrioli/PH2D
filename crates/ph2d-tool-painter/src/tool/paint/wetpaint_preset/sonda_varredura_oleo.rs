//! **F0 — a régua-mãe da refatoração do Wet Paint** (`docs/Painter/Arquitetura/F0_varredura.md`; dono 31/10,
//! `smoke_31_10_oleo/dono_6.png`). Para cada pincel de óleo de fábrica: Pigment × Water (as faixas do painel) ×
//! pressão × N passadas; azul assenta (pausa longa), amarelo por cima meio deslocado (sobre o azul e sobre o papel),
//! papel creme. Por célula: distância à curva K-M azul–amarelo (máx e p99), manchas de concentração (mais escuras
//! que o azul sozinho), ferrugem (matiz fora do segmento azul–verde–amarelo), furos e rugosidade. Grava TSV e PNG.
//! `PH2D_SONDA_OUT=<dir> bash scripts/ph2d-run.sh cargo test -p ph2d-tool-painter --profile smoke --lib sonda_varredura_oleo -- --ignored --nocapture`
#[path = "portao_mae_f0.rs"]
mod portao_mae_f0;
#[path = "sonda_varredura_regua.rs"]
mod regua;

use super::sonda_oleo_grumos::{alturas, mede};
use super::*;
use crate::PaintMedia;
use ph2d_brush_presets::Biblioteca;
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool,
};
use ph2d_wet_paint::colorops::{ks_of_srgb255, reflectance_of_ks, srgb255_of_linear};
use portao_mae_f0::{fora_do_segmento, segmento_oklab};
use regua::*;

const W: u32 = 360;
const H: u32 = 240;
const AZ: [u8; 3] = [40, 90, 220];
const AM: [u8; 3] = [245, 205, 25];
const CREME: [f64; 3] = [236.0, 226.0, 206.0];
/// O recorte gravado por célula (px): x 30..330, y 70..200.
const RX: (usize, usize) = (30, 330);
const RY: (usize, usize) = (70, 200);

/// O Pigment (0..2000, `PIGMENT_MAX` do painel) e o Water (0..1, `volume_row` do painel).
const PIGMENTOS: [f64; 3] = [40.0, 600.0, 2000.0];
const AGUAS: [f64; 3] = [0.0, 0.5, 1.0];
const PRESSOES: [f32; 3] = [0.2, 0.6, 1.0];
const NS: [usize; 2] = [1, 3];

pub(super) const PINCEIS: [&str; 15] = [
    "Oil/Flat/Thick Impasto",
    "Oil/Flat/Filbert",
    "Oil/Flat/Filbert Dry",
    "Oil/Flat/Flat Bristle",
    "Oil/Flat/Tilted Shade",
    "Oil/Dabber/Dabber",
    "Oil/Dabber/Dabber 4",
    "Oil/Knife/Knife Smooth 3",
    "Oil/Knife/Knife Oily",
    "Oil/Knife/Knife Grainy",
    "Oil/Knife/Knife Dry",
    "Oil/Knife/Knife Textured",
    "Oil/Knife/Knife Stain",
    "Oil/Metallic/Metallic Flat",
    "ExpressOil/Round/Express Oil",
];

fn preset(b: &Biblioteca, rel: &str) -> ph2d_brush_presets::Preset {
    let rel = format!("WetPainter/{rel}.ph2d-brush.png");
    let mut p = b
        .entradas
        .iter()
        .find(|e| e.relativo == rel)
        .unwrap_or_else(|| panic!("preset {rel}"))
        .preset
        .clone();
    // A régua mede a lei de MISTURA: a cor própria de cada cerda é desenho do pincel (E9b); `PH2D_SONDA_COR_DO_FIO=1`
    // repõe-na.
    if std::env::var("PH2D_SONDA_COR_DO_FIO").as_deref() != Ok("1") {
        for k in ["bristleNoiseHue", "bristleNoiseSat", "bristleNoiseLight"] {
            p.parametros.engine.remove(k);
        }
    }
    p
}

fn traco(t: &mut PainterTool, a: [f32; 2], b: [f32; 2], pressure: f32, pausa: u32) {
    let p = |pos, phase| CanvasPointer {
        pos,
        pressure,
        tilt: [0.0, 0.0],
        phase,
    };
    let n = ((b[0] - a[0]).hypot(b[1] - a[1]) / 3.0).ceil() as u32;
    t.on_canvas_pointer(p(a, PointerPhase::Down));
    for k in 1..=n {
        let s = k as f32 / n as f32;
        t.on_canvas_pointer(p(
            [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s],
            PointerPhase::Move,
        ));
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(p(b, PointerPhase::Up));
    for _ in 0..pausa {
        t.on_tick(1000.0 / 60.0);
    }
}

/// O cenário da célula para lá da grelha: papel, ordem das cores.
#[derive(Clone, Copy, Default)]
pub(super) struct Cena {
    /// Depth do substrato (`0` = o do app por omissão: nenhum papel escolhido; `> 0` arma o papel de fábrica,
    /// `PaperCold`, como o slider Depth do app).
    pub papel: f32,
    /// Azul → amarelo → azul (a 3.ª mão volta pela orla).
    pub aya: bool,
    /// A lei da tinta (`PH2D_SONDA_LEI`), pela porta do produto `set_wet_lei_da_tinta`.
    pub lei: ph2d_wet_paint::sim::LeiDaTinta,
    /// O papel do documento (`PH2D_SONDA_PAPEIS`).
    pub fundo: Papel,
}

/// O papel do documento da célula (E9; dono 31/10: o portão-mãe decide-se sobre o PADRÃO).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Papel {
    /// O «Textured Paper» do New Image (`PAPEL_TEXTURIZADO`, Cold Press `#E6D9CC`), pelo caminho do produto:
    /// a sprite nascida (`pixeis_do_papel_texturizado`) aberta como documento (`abre_a_sprite_nova`).
    #[default]
    Padrao,
    /// Sem papel (tela transparente, o caminho da F0).
    Liso,
}

impl Papel {
    pub(super) const fn nome(self) -> &'static str {
        match self {
            Self::Padrao => "padrao",
            Self::Liso => "liso",
        }
    }
}

impl std::str::FromStr for Papel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "padrao" => Ok(Self::Padrao),
            "liso" => Ok(Self::Liso),
            o => Err(format!("papel desconhecido: {o:?} (padrao|liso)")),
        }
    }
}

/// Uma célula da grelha. Grava o recorte com luz em `png` (e o plano ao lado, `…_plano.png`) se pedido.
#[allow(clippy::too_many_arguments)]
pub(super) fn celula(
    b: &Biblioteca,
    pincel: &str,
    pig: f64,
    agua: f64,
    pressao: f32,
    n: usize,
    pausa_assenta: u32,
    cena: Cena,
    png: Option<&std::path::Path>,
) -> Celula {
    let mut refs = (0.0, 0.0, 0.0, 0.0);
    let mut px_azul = Vec::new();
    let mut t = pinta(
        b,
        pincel,
        pig,
        agua,
        pressao,
        n,
        pausa_assenta,
        cena,
        &mut |t| {
            t.wet_recomposite_full();
            let (ref_l, man_l) = le_azul(t, None);
            let cob = render_plano(t);
            let (ref_p, man_p) = le_azul(t, cob.as_deref());
            refs = (ref_l, man_l, ref_p, man_p);
            if png.is_some() {
                px_azul = recorte(t);
            }
        },
    );
    let (ref_l, man_l, ref_p, man_p) = refs;
    t.wet_recomposite_full();
    let mut px_l = Vec::with_capacity((RX.1 - RX.0) * (RY.1 - RY.0));
    let luz = le_tudo(&t, ref_l, man_l, Some(&mut px_l), None);
    let (mut miolo, mut papel) = (0usize, 0usize);
    for y in 118..142 {
        for x in 100..260 {
            miolo += 1;
            if t.canvas_rgba[(x + y * W as usize) * 4 + 3] < 230 {
                papel += 1;
            }
        }
    }
    let (rug, p99_altura) = if t.wet_oleo_real().is_some() {
        let (h, m, gw, _gh, r) = alturas(&mut t);
        let md = mede(&h, &m, gw, r);
        (md.rug, md.p99)
    } else {
        (f64::NAN, f64::NAN)
    };
    let cob = render_plano(&mut t);
    let mut px_p = Vec::with_capacity(px_l.len());
    let plano = le_tudo(&t, ref_p, man_p, Some(&mut px_p), cob.as_deref());
    if let Some(f) = png {
        use crate::tool::paint::tests::look_watercolor_arc::{Canvas, write_png};
        let (w, h) = (RX.1 - RX.0, RY.1 - RY.0);
        write_png(f, &Canvas { w, h, px: px_l }).expect("png");
        let fp = f.with_file_name(format!(
            "{}_plano.png",
            f.file_stem().and_then(|s| s.to_str()).unwrap_or("c")
        ));
        write_png(&fp, &Canvas { w, h, px: px_p }).expect("png");
        let fa = f.with_file_name(format!(
            "{}_azul_plano.png",
            f.file_stem().and_then(|s| s.to_str()).unwrap_or("c")
        ));
        write_png(&fa, &Canvas { w, h, px: px_azul }).expect("png");
    }
    Celula {
        luz,
        plano,
        furos: if miolo == 0 {
            0.0
        } else {
            100.0 * papel as f64 / miolo as f64
        },
        rug,
        p99_altura,
    }
}

/// Pinta a célula (azul, pausa, `apos_azul`, amarelo, [azul], 120 ticks) e devolve a ferramenta.
#[allow(clippy::too_many_arguments)]
pub(super) fn pinta(
    b: &Biblioteca,
    pincel: &str,
    pig: f64,
    agua: f64,
    pressao: f32,
    n: usize,
    pausa_assenta: u32,
    cena: Cena,
    apos_azul: &mut dyn FnMut(&mut PainterTool),
) -> PainterTool {
    let mut t = PainterTool::default();
    match cena.fundo {
        Papel::Liso => t.set_source(vec![0u8; (W * H * 4) as usize], W, H),
        Papel::Padrao => {
            const SPRITE: u64 = 1;
            let px = PainterTool::pixeis_do_papel_texturizado(W, H);
            t.agenda_papel_texturizado(SPRITE);
            t.abre_a_sprite_nova(SPRITE, px, W, H);
            assert!(
                t.papel().is_some(),
                "o papel padrão não chegou ao documento"
            );
        }
    }
    t.set_paint_media(PaintMedia::WetPaint);
    t.set_wet_relogio_fixo(true);
    t.set_semente_dos_tracos(0);
    if cena.papel > 0.0 {
        t.set_substrate_depth(cena.papel);
    }
    t.set_wet_lei_da_tinta(cena.lei);
    let p = preset(b, pincel);
    let passadas = |t: &mut PainterTool, cor: [u8; 3], y0: f32| {
        for k in 0..n {
            let erros = t.wet_preset_aplicar(&p);
            assert!(erros.is_empty(), "{erros:?}");
            assert_eq!(
                t.wet_lei_da_tinta(),
                Some(cena.lei),
                "a lei não sobreviveu ao preset"
            );
            // E10c2: a carga de tinta `pig` (o Pigment de antes) é a Dilution com a MESMA `φ` sobre esta água.
            let wpd = t
                .paint
                .wetpaint
                .knobs
                .get(ph2d_wet_paint::tuning::Knob::WaterPerDab);
            let (diluicao, _) =
                ph2d_wet_paint::painter::receita::migra_para_controlos(pig, wpd, agua, 1.0, 1.0);
            for (id, v) in [
                (crate::ids::PAINTER_WETPAINT_DILUTION, diluicao),
                (crate::ids::PAINTER_WETPAINT_WATER, agua),
            ] {
                assert!(t.route_brush_wetpaint_event(&PanelEvent::SetValue(id, v)));
            }
            // `PH2D_SONDA_KNOBS=chave=v;…` (a chave do registo): isola um mecanismo numa célula.
            for (k, v) in std::env::var("PH2D_SONDA_KNOBS")
                .unwrap_or_default()
                .split(';')
                .filter_map(|kv| kv.split_once('='))
            {
                let d = ph2d_wet_paint::tuning::KNOB_DEFS
                    .iter()
                    .find(|d| d.key == k);
                let d = d.unwrap_or_else(|| panic!("knob {k}"));
                t.paint
                    .wetpaint
                    .knobs
                    .set(d.knob, v.parse().expect("valor"));
            }
            t.sonda_raio(32.0);
            t.set_brush_color_srgb8(cor);
            let y = y0 + [0.0, 6.0, -6.0][k % 3];
            let (a, z) = ([50.0, y], [310.0, y + 4.0]);
            if k % 2 == 0 {
                traco(t, a, z, pressao, 60);
            } else {
                traco(t, z, a, pressao, 60);
            }
        }
    };
    passadas(&mut t, AZ, 120.0);
    for _ in 0..pausa_assenta {
        t.on_tick(1000.0 / 60.0);
    }
    apos_azul(&mut t);
    passadas(&mut t, AM, 150.0);
    if cena.aya {
        for _ in 0..pausa_assenta {
            t.on_tick(1000.0 / 60.0);
        }
        passadas(&mut t, AZ, 132.0);
    }
    for _ in 0..120 {
        t.on_tick(1000.0 / 60.0);
    }
    t
}

fn lista<T>(var: &str, omissao: &[T]) -> Vec<T>
where
    T: std::str::FromStr + Copy,
{
    std::env::var(var).ok().map_or_else(
        || omissao.to_vec(),
        |s| s.split(',').filter_map(|v| v.trim().parse().ok()).collect(),
    )
}

#[test]
#[ignore = "sonda — corre-se à mão no perfil smoke"]
fn sonda_varredura_oleo() {
    let b = Biblioteca::carregar(
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/brushes"),
        None,
    );
    let dir = std::path::PathBuf::from(
        std::env::var("PH2D_SONDA_OUT").unwrap_or_else(|_| "/tmp/f0_varredura".into()),
    );
    std::fs::create_dir_all(dir.join("celulas")).expect("pasta");
    let pinceis: Vec<String> = std::env::var("PH2D_SONDA_PINCEIS").map_or_else(
        |_| PINCEIS.iter().map(|s| (*s).to_string()).collect(),
        |s| s.split(',').map(str::to_string).collect(),
    );
    let pigs = lista("PH2D_SONDA_PIGMENTOS", &PIGMENTOS);
    let aguas = lista("PH2D_SONDA_AGUAS", &AGUAS);
    let pressoes = lista("PH2D_SONDA_PRESSOES", &PRESSOES);
    let ns = lista("PH2D_SONDA_NS", &NS);
    let leis = lista("PH2D_SONDA_LEI", &[ph2d_wet_paint::sim::LeiDaTinta::Velha]);
    let papeis = lista("PH2D_SONDA_PAPEIS", &[Papel::Padrao, Papel::Liso]);
    let assenta: u32 = std::env::var("PH2D_SONDA_ASSENTA")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(300);
    let cena0 = Cena {
        papel: std::env::var("PH2D_SONDA_PAPEL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0.0),
        aya: std::env::var("PH2D_SONDA_ORDEM").is_ok_and(|v| v == "aya"),
        lei: ph2d_wet_paint::sim::LeiDaTinta::Velha,
        fundo: Papel::Padrao,
    };
    let tsv_path =
        dir.join(std::env::var("PH2D_SONDA_TSV").unwrap_or_else(|_| "varredura.tsv".into()));
    let mut tsv = String::from(
        "pincel\tpigment\twater\tpressao\tn\tkm_max\tkm_p99\tmanchas_pct\tmanchas_azul_pct\tferrugem_pct\tfuros_pct\trug\tp99_altura\tref_azul\tpng\
         \tkm_p99_plano\tkm_max_plano\tmanchas_plano\tmanchas_azul_plano\tferrugem_plano\tref_azul_plano\tdegraus_y\tdegraus_x\tdegraus_y_plano\tdegraus_x_plano\tpausa\tpapel_depth\tordem\tlei\tpapel\tmix_p99\tmix_max\tmix_p99_plano\tmix_max_plano\torla_p99_plano\n",
    );
    for pincel in &pinceis {
        let slug = pincel
            .rsplit('/')
            .next()
            .unwrap_or(pincel)
            .replace(' ', "_")
            .to_lowercase();
        for &pig in &pigs {
            for &agua in &aguas {
                for &pr in &pressoes {
                    for &n in &ns {
                        for (&fundo, &lei) in
                            papeis.iter().flat_map(|f| leis.iter().map(move |l| (f, l)))
                        {
                            let cena = Cena {
                                lei,
                                fundo,
                                ..cena0
                            };
                            let ordem = if cena.aya { "aya" } else { "ay" };
                            let nome = format!(
                                "{slug}_p{pig}_w{agua}_pr{pr}_n{n}_s{assenta}_pa{}_{ordem}_{}_{}.png",
                                cena.papel,
                                lei.nome(),
                                fundo.nome()
                            );
                            let t0 = std::time::Instant::now();
                            let c = celula(
                                &b,
                                pincel,
                                pig,
                                agua,
                                pr,
                                n,
                                assenta,
                                cena,
                                Some(&dir.join("celulas").join(&nome)),
                            );
                            let (l, q) = (c.luz, c.plano);
                            let linha = format!(
                                "{pincel}\t{pig}\t{agua}\t{pr}\t{n}\t{:.1}\t{:.1}\t{:.2}\t{:.2}\t{:.2}\t{:.2}\t{:.3}\t{:.2}\t{:.1}\t{nome}\
                             \t{:.1}\t{:.1}\t{:.2}\t{:.2}\t{:.2}\t{:.1}\t{}\t{}\t{}\t{}\t{assenta}\t{}\t{ordem}\t{}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\n",
                                l.km_max,
                                l.km_p99,
                                l.manchas,
                                l.manchas_azul,
                                l.ferrugem,
                                c.furos,
                                c.rug,
                                c.p99_altura,
                                l.ref_azul,
                                q.km_p99,
                                q.km_max,
                                q.manchas,
                                q.manchas_azul,
                                q.ferrugem,
                                q.ref_azul,
                                l.degraus_y,
                                l.degraus_x,
                                q.degraus_y,
                                q.degraus_x,
                                cena.papel,
                                lei.nome(),
                                fundo.nome(),
                                l.mix_p99,
                                l.mix_max,
                                q.mix_p99,
                                q.mix_max,
                                q.orla_p99
                            );
                            print!("[f0 {:>5.1}s] {linha}", t0.elapsed().as_secs_f64());
                            tsv.push_str(&linha);
                            std::fs::write(&tsv_path, &tsv).expect("tsv");
                        }
                    }
                }
            }
        }
    }
}
