//! W10, `docs/Macros/08_plano_W10_cobertura.md` — **a marca é COBERTURA**: uma pincelada nova a cruzar as
//! gravadas não deixa resquício no Play (smoke do dono, 19/10: pincel macio com grão e Digital duro, manchas
//! e «x» pretos onde a pincelada nova cruza uma gravada ainda por nascer). Filho de `palcos_fim_tests`.
//!
//! O oráculo é à parte da mistura: a MESMA pincelada pintada num tool novo cuja tela é a reprodução
//! daquele instante (antes de as gravadas nascerem, o branco) — a pincelada por cima do que a gravação tem.

use super::*;

const L: u32 = 96;
const N: usize = (L * L) as usize;

/// O pincel de cada foto do dono e os vizinhos que passam pelas três rotas que medem a força (por
/// píxel, o carimbo em cache, o grão fixo à tela). Sem o grão ALEATÓRIO: o sorteio depende do que o
/// pincel já carimbou, e a régua (um tool novo) sorteia outro grão — medido, 52 níveis entre as duas.
/// Um pincel com nome: o acerto por cima do pincel preto da fixtura.
type Caso = (&'static str, fn(&mut PainterTool));

const CASOS: [Caso; 7] = [
    ("macio", |_| {}),
    ("duro (esfera)", |t| t.set_brush_falloff(2)),
    ("duro (disco)", |t| t.set_brush_falloff(8)),
    (
        "macio com grão (segue o pincel: o carimbo em cache)",
        |t| {
            t.new_brush_texture();
            t.set_brush_texture_mapping(0);
        },
    ),
    ("macio com grão fixo à tela", |t| {
        t.new_brush_texture();
        t.set_brush_texture_mapping(1);
    }),
    ("força ½ (o tecto do traço)", |t| {
        t.set_brush_strength(0.5)
    }),
    ("contínuo esparso", |t| {
        t.toggle_brush_continuous_spacing();
        t.set_brush_spacing(1.0);
    }),
];

/// O pincel preto de raio 14 em Digital, acertado por `pincel`.
fn acerta(t: &mut PainterTool, pincel: fn(&mut PainterTool)) {
    t.set_paint_media(ph2d_tool_painter::PaintMedia::Digital);
    t.set_brush_size_px(14.0);
    t.set_brush_color_srgb8([0, 0, 0]);
    pincel(t);
}

/// A vertical em `x` de cima a baixo (cruza as duas gravadas), levantada.
fn vertical(t: &mut PainterTool, x: f32) {
    #[expect(clippy::cast_precision_loss, reason = "o lado cabe num f32")]
    let l = L as f32;
    traco(t, [x, 0.12 * l], [x, 0.88 * l], 14, true);
    tique(t);
}

/// A régua: a pincelada sozinha sobre `fundo` (opaco, straight = premultiplicado).
fn sobre(fundo: &[u8], pincel: fn(&mut PainterTool), x: f32) -> Vec<u8> {
    assert!(
        fundo.len() == N * 4 && fundo.as_chunks::<4>().0.iter().all(|p| p[3] == 255),
        "fundo opaco"
    );
    let mut r = PainterTool::default();
    r.set_source(fundo.to_vec(), L, L);
    acerta(&mut r, pincel);
    vertical(&mut r, x);
    r.imagem_aplicada().expect("documento").0
}

/// O desvio máximo, em níveis, do que o carimbo pôs sobre `fundo` (`x`) para o que a mesma pincelada
/// preta pôs sobre o branco (`xw`) escalado ao fundo — o piso do arredondamento de cada dab.
fn piso(xw: &[u8], fundo: &[u8], x: &[u8]) -> u8 {
    xw.iter()
        .zip(fundo)
        .zip(x)
        .enumerate()
        .filter(|(k, _)| k % 4 != 3)
        .map(|(_, ((&w, &f), &x))| {
            let linear = (u32::from(w) * u32::from(f) + 127) / 255;
            u8::try_from(linear).expect("≤ 255").abs_diff(x)
        })
        .max()
        .unwrap_or(0)
}

/// O desvio que nenhum piso justifica (2,4 % da escala; medido 19/10: o pior caso, o grão fixo à tela,
/// 5 níveis com o piso dele em 3 + 0).
const TECTO: u8 = 6;

/// O maior desvio (níveis) entre `r` e `e` e onde.
fn desvio(r: &[u8], e: &[u8]) -> (u8, usize) {
    r.iter()
        .zip(e)
        .enumerate()
        .map(|(k, (&a, &b))| (a.abs_diff(b), k / 4))
        .max()
        .unwrap_or((0, 0))
}

/// ⭐⭐ **A pincelada nova a cruzar as gravadas, no Play: só ela sobre o que a gravação tem naquele
/// instante** — antes de as gravadas nascerem (`0,1 s`: a vertical sobre o branco, sem resquício do
/// fim), com a 1.ª nascida e a 2.ª por nascer (`0,6 s`) e no fim. A régua: o mesmo traço pintado sobre a
/// reprodução daquele instante, até ao piso do próprio carimbo (o arredondamento de cada dab ao byte
/// sobre os dois fundos, medido à parte, + 2) e nunca acima de [`TECTO`]. ⛔ VERMELHO com a marca
/// binária (199 a 231 níveis em todos os pincéis menos o disco duro): o texel que a borda da vertical
/// mudou mostrava a imagem atual, que JÁ tem a gravada por baixo (as manchas e os «x» das fotos do dono).
#[test]
fn a_pincelada_nova_a_cruzar_as_gravadas_nao_deixa_resquicio() {
    let mut falhas = Vec::new();
    for (nome, pincel) in CASOS {
        let mut world = World::new();
        let e = world.spawn(MacroTempo { segundos: 0.0 }).id();
        let (mut t, doc, d) =
            ph2d_tool_painter::macro_track::roteiro::grava_duas_horizontais_ligado(L, e.to_bits())
                .expect("grava");
        let mut c = Clipe::com(world, e, &doc);
        let r = c.quadro_ligado(0.1, Some(&mut t), &mut nunca);
        assert!(
            r == c.palco,
            "controlo {nome}: sem pintar, a ranhura é o palco"
        );
        acerta(&mut t, pincel);
        let gravado = t.imagem_aplicada().expect("doc").0;
        let x = 48.3;
        vertical(&mut t, x);
        let atual = t.imagem_aplicada().expect("doc").0;
        let regua = sobre(&vec![255; N * 4], pincel, x);
        let pintou = regua
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[0] < 255)
            .count();
        assert!(pintou > 100, "controlo {nome}: a régua pintou ({pintou})");
        for instante in [0.1, 0.6, d] {
            let r = c.quadro_ligado(instante, Some(&mut t), &mut nunca);
            let gravada = c
                .palco
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[0] < 128)
                .count();
            if instante < d {
                assert!(
                    (instante < 0.2) == (gravada == 0),
                    "controlo {nome} {instante:.1} s: o palco tem {gravada} texels escuros"
                );
            }
            let oraculo = sobre(&c.palco, pincel, x);
            let (dv, k) = desvio(&r, &oraculo);
            // O piso do carimbo: a pincelada preta é linear no fundo (`X = (1 − a)·fundo`) a menos do
            // arredondamento de cada dab ao byte, que não é o mesmo sobre fundos diferentes.
            let tecto = piso(&regua, &gravado, &atual) + piso(&regua, &c.palco, &oraculo) + 2;
            if dv > tecto.min(TECTO) {
                falhas.push(format!(
                    "{nome} a {instante:.2} s: {dv} níveis no texel ({}, {}) — o carimbo explica {tecto}",
                    k % L as usize,
                    k / L as usize
                ));
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "resquícios no Play:\n{}",
        falhas.join("\n")
    );
}
