//! **As duas rotas do depósito de um dab pousam a MESMA tinta** — o gate de identidade do
//! ADR-0175.
//!
//! O depósito de um dab do produto (`Trail::accumulate_paint_shaped`) caminha as linhas do dab em
//! série ou em paralelo conforme o piso medido. Aqui o MESMO traço é pousado pelas duas rotas
//! forçadas, sobre o MESMO estado molhado, e tudo o que o depósito escreve é comparado **byte a
//! byte**: o carimbo de umidade no grid, os planos de pigmento e de água da janela, a extensão da
//! janela e o retângulo que cada dab declara ter escrito. Não há tolerância — a lei da célula é UMA
//! função, e a única diferença entre as rotas é *que thread avaliou que linha*.
//!
//! ⚠️ **O verde-sobre-nada que este ficheiro recusa:** com a fixture abaixo do piso as duas rotas
//! seriam a série contra ela própria. O [`a_fixtura_contem_o_fenomeno`] afirma as premissas dos
//! outros — o dab cruza o piso, pousa tinta de verdade, e parte dele cai FORA da janela (senão o
//! ramo que só carimba a umidade nunca correria) e parte dele é recortada pela borda do canvas.

use crate::util;

use ph2d_wet_paint::brush::{BrushShape, CellFn};
use ph2d_wet_paint::painter::Engine;
use ph2d_wet_paint::par::{MIN_CELLS_DEPOSIT, Rows};
use ph2d_wet_paint::trail::{Dab, Trail, TrailMode};
use ph2d_wet_paint::tuning::Knob;
use util::drive_stroke;

const W: usize = 900;
const H: usize = 700;
/// O raio do dab: `2·230 + 1 = 461` células de lado, `~212 000` na caixa — bem acima do piso.
const R: f64 = 230.0;
/// Os centros do traço. O PRIMEIRO é recortado pelas bordas esquerda e de cima do canvas, o
/// ÚLTIMO pelas da direita e de baixo — os quatro lados da caixa do dab (uma mutação que recuava a
/// caixa só na esquerda SOBREVIVEU com todos os dabs longe das bordas) —, e o passo de `140` leva
/// os do meio para fora da janela, que se ancora no primeiro.
const CENTROS: [(f64, f64); 5] = [
    (150.0, 200.0),
    (290.0, 330.0),
    (430.0, 330.0),
    (570.0, 330.0),
    (870.0, 560.0),
];
/// A cor base do traço — a que a limpeza do bico persegue.
const BASE: [f64; 3] = [40.0, 90.0, 200.0];

/// Um grid com água viva e pigmento assente — o estado em que a lei da célula decide coisas
/// (o dente do papel contra o `GateSaturation`, o amolecimento pela película).
fn molhado() -> Engine {
    let mut e = Engine::new(W, H);
    e.sliders.water = 1.0;
    e.sliders.size = 1.0;
    drive_stroke(&mut e, 120.0, 300.0, 820.0, 380.0, 24.0, 2);
    // E tinta SUSPENSA **e** ASSENTE em toda a área, com cores que variam de célula para célula.
    // ⚠️ Load-bearing: a recolha do bico só age onde o canvas tem tinta, e o peso da tinta assente
    // (`w_s`) só onde ela está assente — sobre o traço de cima (uma faixa estreita, nada assente)
    // TRÊS mutações do bico sobreviveram, porque ali o passo novo e o antigo coincidem por vácuo.
    let g = e.active_grid_mut();
    for i in 0..g.susp.len() {
        g.susp[i] = 20.0 + (i % 97) as f32 * 3.0;
        g.sett[i] = 10.0 + (i % 89) as f32 * 2.0;
        g.susp_rgb[i] = [(i % 251) as f32, (i % 241) as f32, (i % 239) as f32];
        g.sett_rgb[i] = [(i % 233) as f32, (i % 229) as f32, (i % 227) as f32];
    }
    e
}

fn dab((x, y): (f64, f64)) -> Dab {
    Dab {
        x,
        y,
        r: R,
        hardness: 0.5,
        // Forte o bastante para o carimbo SATURAR na maior parte do disco: com `1,0` a cerda
        // (média `0,01`) e o dente do papel rejeitavam quase tudo, e o traço pousava `2 395`
        // texels — o gate de premissa abaixo apanhou-o à primeira corrida.
        intensity: 40.0,
        water_amount: 0.5,
        dry_gate: 0.2,
        shape: BrushShape::Round,
        dir_x: 1.0,
        dir_y: 0.0,
    }
}

/// O que um traço inteiro deixou, pela rota `mode`.
struct Saida {
    wet: Vec<u8>,
    /// Todo plano de massa, água e cor do grid, em bits — o transfer escreve-os todos.
    grid: Vec<u32>,
    pig: Vec<u32>,
    water: Vec<u32>,
    bico: Vec<u32>,
    extensao: Option<(i32, i32, i32, i32)>,
    escritos: Vec<Option<(i32, i32, i32, i32)>>,
    pousos: Vec<Option<(i32, i32, i32, i32)>>,
    /// Texels do bico em que o transfer discordou da [`bico_de_referencia`], somados sobre todos
    /// os transfers do traço.
    bico_fora_da_referencia: usize,
}

/// **Os passos 1–2 do transfer COMO ERAM antes do ADR-0175** — dois laços sobre a janela, a
/// limpeza inteira e depois a recolha, copiados do `transfer.rs` de então e CONGELADOS aqui.
///
/// ⚠️ É o único oráculo independente do bico: o código novo é partilhado pelas duas rotas E pelo
/// caminho do próprio motor, logo um defeito nele (uma linha saltada, um texel a menos) ficava
/// igual dos dois lados de todo outro gate — as duas mutações disso SOBREVIVERAM à 1.ª ronda.
#[allow(clippy::too_many_arguments)]
fn bico_de_referencia(
    tip: [&mut [f32]; 3],
    clean: f64,
    tip_retain: f64,
    ext: Option<(i32, i32, i32, i32)>,
    (ax, ay): (i32, i32),
    half: i32,
    g: &ph2d_wet_paint::grid::Grid,
) {
    let [tr, tg, tb] = tip;
    if clean > 0.0 {
        for l in 0..tr.len() {
            tr[l] = (tr[l] as f64 + (BASE[0] - tr[l] as f64) * clean) as f32;
            tg[l] = (tg[l] as f64 + (BASE[1] - tg[l] as f64) * clean) as f32;
            tb[l] = (tb[l] as f64 + (BASE[2] - tb[l] as f64) * clean) as f32;
        }
    }
    let Some((lx0, ly0, lx1, ly1)) = ext else {
        return;
    };
    let size = half * 2 + 1;
    let (s, w, h) = (g.s, g.w as i32, g.h as i32);
    for ly in ly0..=ly1 {
        let cy = ay + (ly - half);
        if cy < 2 || cy > h - 1 {
            continue;
        }
        for lx in lx0..=lx1 {
            let cx = ax + (lx - half);
            if cx < 2 || cx > w - 1 {
                continue;
            }
            let i = cx as usize + cy as usize * s;
            let w_s = ph2d_wet_paint::opacity::alpha_of_mass(g.sett[i] as f64) * 0.5;
            let w_f = ph2d_wet_paint::opacity::alpha_of_mass(g.susp[i] as f64);
            let fe = w_s + w_f;
            if fe <= 0.0 {
                continue;
            }
            let inv = 1.0 / fe;
            let sc = g.sett_rgb[i];
            let uc = g.susp_rgb[i];
            let cr = (sc[0] as f64 * w_s + uc[0] as f64 * w_f) * inv;
            let cg = (sc[1] as f64 * w_s + uc[1] as f64 * w_f) * inv;
            let cb = (sc[2] as f64 * w_s + uc[2] as f64 * w_f) * inv;
            let k = tip_retain + (1.0 - tip_retain) * (1.0 - fe.min(1.0));
            let l = (lx + ly * size) as usize;
            tr[l] = (tr[l] as f64 * k + cr * (1.0 - k)) as f32;
            tg[l] = (tg[l] as f64 * k + cg * (1.0 - k)) as f32;
            tb[l] = (tb[l] as f64 * k + cb * (1.0 - k)) as f32;
        }
    }
}

fn bits(v: &[f32]) -> impl Iterator<Item = u32> + '_ {
    v.iter().map(|x| x.to_bits())
}

/// Por que caminho o traço é pousado.
#[derive(Clone, Copy)]
enum Caminho {
    /// A porta do produto por linhas, na rota forçada, com a silhueta de disco suave e (ou não)
    /// o grão às riscas.
    Linhas(Rows, bool),
    /// A porta do produto por linhas com a silhueta que REPRODUZ a queda do próprio motor.
    LinhasComAQuedaDoMotor(Rows),
    /// O caminho do PRÓPRIO motor (`accumulate_paint`), em série e com a caminhada dele — o
    /// oráculo independente da caminhada por linhas.
    Motor,
}

/// A queda redonda do motor (`for_each_stamp_pixel`, forma `Round`, sem direcção), escrita com as
/// MESMAS operações e na MESMA ordem — é isso que faz dela um oráculo ao bit, e não uma aproximação.
fn queda_do_motor(x: i32, y: i32, cx: f64, cy: f64, r: f64, dureza: f64) -> f64 {
    let inv_r = 1.0 / r;
    let nx = (f64::from(x) - cx) * inv_r;
    let ny = (f64::from(y) - cy) * inv_r;
    let d2 = nx * nx + ny * ny;
    if d2 < 1.0 {
        ph2d_wet_paint::brush::radial_falloff(d2.sqrt(), dureza)
    } else {
        0.0
    }
}

fn pousa(mode: Rows, com_grao: bool) -> Saida {
    pousa_por(Caminho::Linhas(mode, com_grao))
}

fn pousa_por(caminho: Caminho) -> Saida {
    let mut e = molhado();
    // O bico só RECOLHE com `Pickup > 0` (com `0` a retenção é `1` e a recolha é inerte — a
    // premissa abaixo apanhou-o à primeira corrida), e só se LIMPA com `TipClean > 0`: os dois
    // ramos do passo do bico têm de trabalhar para as rotas serem comparadas a fazer alguma coisa.
    e.tuning.set(Knob::Pickup, 0.2);
    e.tuning.set(Knob::TipClean, 0.02);
    let p = e.sim.gather_params(&e.tuning);
    let tex = e.bristle_texture_for_measure();
    let mut t = Trail::default();
    t.start_stroke(CENTROS[0].0, CENTROS[0].1, BASE, TrailMode::Paint);
    // Janela de UM dab (`floor(12/12) = 1`): ela enche a cada dois dabs, e o traço faz DUAS
    // transferências. ⚠️ Com uma só, o bico ainda estava LIMPO quando a limpeza corria (ela
    // persegue a cor base, e a base limpa é ponto fixo) — a mutação que limpava meia linha
    // sobreviveu à referência congelada e só um gate antigo a apanhou.
    t.on_segment(12.0, 12.0);
    let g = e.active_grid_mut();
    let mut escritos = Vec::new();
    let mut pousos = Vec::new();
    let mut bico_fora_da_referencia = 0;
    for &(cx, cy) in &CENTROS {
        // Uma silhueta com borda suave e zero fora do disco — e um grão com riscas, para o ramo
        // `grain` ser exercitado com textura que ANULA células.
        let sil = move |x: i32, y: i32| -> f64 {
            let (dx, dy) = (f64::from(x) - cx, f64::from(y) - cy);
            let d = (dx * dx + dy * dy).sqrt() / R;
            if d >= 1.0 { 0.0 } else { 1.0 - d * d }
        };
        let grao = |x: i32, y: i32| -> f64 { if (x + 2 * y) % 5 == 0 { 0.0 } else { 0.7 } };
        let d = dab((cx, cy));
        let motor = move |x: i32, y: i32| queda_do_motor(x, y, cx, cy, R, d.hardness);
        let (a, mode) = match caminho {
            Caminho::Linhas(mode, com_grao) => (
                t.accumulate_paint_shaped_rows(
                    g,
                    &p,
                    &tex,
                    &d,
                    false,
                    &sil,
                    com_grao.then_some(&grao as CellFn<'_>),
                    mode,
                ),
                mode,
            ),
            Caminho::LinhasComAQuedaDoMotor(mode) => (
                t.accumulate_paint_shaped_rows(g, &p, &tex, &d, false, &motor, None, mode),
                mode,
            ),
            Caminho::Motor => (t.accumulate_paint(g, &p, &tex, &d, false), Rows::Serial),
        };
        escritos.push(a.wrote.map(|r| (r.x0, r.y0, r.x1, r.y1)));
        // Como o produto: a janela cheia é pousada no canvas pela MESMA rota.
        if a.window_full {
            let (r0, g0, b0) = t.tip_planes_for_measure();
            let (mut rr, mut rg, mut rb) = (r0.to_vec(), g0.to_vec(), b0.to_vec());
            bico_de_referencia(
                [&mut rr, &mut rg, &mut rb],
                p.k(Knob::TipClean),
                1.0 - p.k(Knob::Pickup),
                t.touched_extent_for_measure(),
                t.anchor_for_measure(),
                t.window_half_for_measure(),
                g,
            );
            let r = t.transfer_paint_rows(g, &p, mode);
            pousos.push(r.map(|r| (r.x0, r.y0, r.x1, r.y1)));
            let (r1, g1, b1) = t.tip_planes_for_measure();
            bico_fora_da_referencia += [(&rr, r1), (&rg, g1), (&rb, b1)]
                .iter()
                .map(|(a, b)| {
                    a.iter()
                        .zip(b.iter())
                        .filter(|(p, q)| p.to_bits() != q.to_bits())
                        .count()
                })
                .sum::<usize>();
        }
    }
    let (pig, water) = t.planes_for_measure();
    let (tr, tg, tb) = t.tip_planes_for_measure();
    let rgb = |v: &[[f32; 3]]| {
        v.iter()
            .flatten()
            .map(|x| x.to_bits())
            .collect::<Vec<u32>>()
    };
    let mut grid: Vec<u32> = bits(&g.susp)
        .chain(bits(&g.sett))
        .chain(bits(&g.film))
        .collect();
    grid.extend(rgb(&g.susp_rgb));
    grid.extend(rgb(&g.sett_rgb));
    Saida {
        wet: g.wet.clone(),
        grid,
        pig: bits(pig).collect(),
        water: bits(water).collect(),
        bico: bits(tr).chain(bits(tg)).chain(bits(tb)).collect(),
        extensao: t.touched_extent_for_measure(),
        escritos,
        pousos,
        bico_fora_da_referencia,
    }
}

fn afirma_iguais(a: &Saida, b: &Saida, rotulo: &str) {
    for (lado, s) in [("1.º", a), ("2.º", b)] {
        assert_eq!(
            s.bico_fora_da_referencia, 0,
            "{rotulo} ({lado} lado): o passo do bico discordou dos laços de ANTES em {} texels",
            s.bico_fora_da_referencia
        );
    }
    let diferentes = |x: &[u32], y: &[u32]| x.iter().zip(y).filter(|(p, q)| p != q).count();
    assert_eq!(
        a.escritos, b.escritos,
        "{rotulo}: o retângulo que cada dab declara divergiu"
    );
    assert_eq!(
        a.extensao, b.extensao,
        "{rotulo}: a extensão da janela divergiu"
    );
    let wet = a.wet.iter().zip(&b.wet).filter(|(p, q)| p != q).count();
    assert_eq!(wet, 0, "{rotulo}: {wet} células de umidade diferentes");
    assert_eq!(
        diferentes(&a.pig, &b.pig),
        0,
        "{rotulo}: texels de pigmento da janela diferentes"
    );
    assert_eq!(
        diferentes(&a.water, &b.water),
        0,
        "{rotulo}: texels de água da janela diferentes"
    );
    assert_eq!(
        a.pousos, b.pousos,
        "{rotulo}: o retângulo que cada transfer pousou divergiu"
    );
    assert_eq!(
        diferentes(&a.grid, &b.grid),
        0,
        "{rotulo}: massas, água ou cores do grid diferentes depois dos transfers"
    );
    assert_eq!(
        diferentes(&a.bico, &b.bico),
        0,
        "{rotulo}: texels do bico diferentes"
    );
}

/// **A rota paralela É a rota serial** — com a textura do motor (as cerdas).
#[test]
fn o_deposito_em_paralelo_e_o_deposito_em_serie() {
    afirma_iguais(
        &pousa(Rows::Serial, false),
        &pousa(Rows::Parallel, false),
        "cerdas",
    );
}

/// E com o GRÃO do hospedeiro no lugar das cerdas — o outro ramo da amostragem.
#[test]
fn o_deposito_com_grao_em_paralelo_e_o_deposito_em_serie() {
    afirma_iguais(
        &pousa(Rows::Serial, true),
        &pousa(Rows::Parallel, true),
        "grão",
    );
}

/// **A caminhada por LINHAS é a caminhada do próprio MOTOR** — o oráculo independente.
///
/// Os dois gates acima comparam as duas rotas do mesmo código: um defeito na parte PARTILHADA da
/// caminhada (a caixa do dab, a linha da janela que cabe a cada `y`, o teste de janela, a união das
/// extensões) apareceria igual nas duas e ficaria verde. Aqui o outro lado é o `accumulate_paint`
/// do motor, que caminha a caixa em série com o `touch_ext` dele e NUNCA passou pelo código novo;
/// a silhueta do produto é a queda redonda do motor, escrita com as mesmas operações, logo cada
/// célula recebe a mesma `(queda, textura)` e a única diferença possível é a caminhada.
#[test]
fn a_caminhada_por_linhas_e_a_caminhada_do_proprio_motor() {
    let motor = pousa_por(Caminho::Motor);
    afirma_iguais(
        &motor,
        &pousa_por(Caminho::LinhasComAQuedaDoMotor(Rows::Parallel)),
        "motor × linhas em paralelo",
    );
    afirma_iguais(
        &motor,
        &pousa_por(Caminho::LinhasComAQuedaDoMotor(Rows::Serial)),
        "motor × linhas em série",
    );
}

/// **As premissas dos dois gates acima**, afirmadas em vez de assumidas.
#[test]
fn a_fixtura_contem_o_fenomeno() {
    let lado = (2.0 * R) as usize + 1;
    assert!(
        lado * lado >= MIN_CELLS_DEPOSIT,
        "a caixa do dab ({} células) está abaixo do piso {MIN_CELLS_DEPOSIT}: o produto \
         escolheria a série e os gates comparariam a série consigo própria",
        lado * lado
    );
    let s = pousa(Rows::Serial, false);
    let tinta: usize = s.pig.iter().filter(|&&b| f32::from_bits(b) > 0.0).count();
    assert!(
        tinta > 10_000,
        "o traço pousou {tinta} texels de pigmento: não há o que comparar"
    );
    let (_, _, lx1, _) = s.extensao.expect("a janela foi tocada");
    let largura_janela = lx1 + 1;
    let (wx0, _, wx1, _) = s
        .escritos
        .iter()
        .flatten()
        .copied()
        .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)))
        .expect("os dabs escreveram umidade");
    assert!(
        wx1 - wx0 + 1 > largura_janela,
        "a umidade cobre {} colunas e a janela {largura_janela}: nenhum dab caiu fora dela, e o \
         ramo que só carimba a umidade nunca correu",
        wx1 - wx0 + 1
    );
    let (_, wy0, _, wy1) = s
        .escritos
        .iter()
        .flatten()
        .copied()
        .reduce(|a, b| (a.0.min(b.0), a.1.min(b.1), a.2.max(b.2), a.3.max(b.3)))
        .expect("os dabs escreveram umidade");
    assert_eq!(
        (wx0, wy0, wx1, wy1),
        (2, 2, W as i32 - 1, H as i32 - 1),
        "a fixtura tinha de tocar os QUATRO limites da área pincelável do canvas"
    );
    // O transfer tem de CORRER, e a recolha do bico tem de apanhar cor do canvas — senão as duas
    // rotas do passo do bico seriam comparadas a limpar uma janela que nada sujou.
    assert!(
        s.pousos.iter().flatten().count() >= 2,
        "{} transfers pousaram tinta: com menos de dois a limpeza do bico corre sobre um bico \
         LIMPO e é ponto fixo — a referência não a veria",
        s.pousos.iter().flatten().count()
    );
    let base = [40.0f32, 90.0, 200.0];
    let n = s.bico.len() / 3;
    let sujos = (0..n)
        .filter(|&l| (0..3).any(|c| (f32::from_bits(s.bico[c * n + l]) - base[c]).abs() > 1.0))
        .count();
    // Medido: `840` (a faixa pintada pelo `molhado` é estreita, e a limpeza puxa o resto de volta
    // à cor base). A barra é «a recolha não é inerte», com margem — `0` foi o que a fixtura dava
    // com o `Pickup` de fábrica do motor.
    assert!(
        sujos > 200,
        "só {sujos} texels do bico se afastaram da cor base: a recolha não apanhou o canvas"
    );
}
