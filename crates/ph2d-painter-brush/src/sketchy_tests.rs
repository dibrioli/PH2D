//! Os gates do **Sketchy** — o `LineKind::Sketchy` do plano 38 W3.
//!
//! O que a feature promete: a cada dab, fios de opacidade baixa ligam os pontos vizinhos **do mesmo
//! traço**, e o acúmulo desenha o hachurado. Os gates perguntam pelos FIOS, não pela fórmula.

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::{LineKind, SKETCHY_DENSITY_MAX};
use crate::spec::BrushSpec;
use crate::stroke::{Stroke, StrokePoint};
use crate::symmetry::{MirrorAxis, SymmetrySettings};

fn spec(kind: LineKind, density: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        spacing: 0.1,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: kind,
        sketchy_reach: 1.0,
        sketchy_density: density,
        ..Default::default()
    }
}

fn plain_dynamics() -> Dynamics {
    Dynamics {
        size_pressure: false,
        strength_pressure: false,
        ..Default::default()
    }
}

/// Um traço em ZIGUE-ZAGUE apertado: ele volta para perto de si mesmo, que é onde o Sketchy tem
/// vizinhos legítimos a costurar. Devolve `(dabs, fios)`.
fn zigzag(sp: BrushSpec, legs: usize) -> (Vec<[f32; 2]>, Vec<[f32; 4]>) {
    let mut s = Stroke::new(sp, plain_dynamics(), 7);
    let mut out = Vec::new();
    let mut threads = Vec::new();
    let mut dabs: Vec<[f32; 2]> = Vec::new();
    let mut all: Vec<[f32; 4]> = Vec::new();
    let start = [200.0f32, 200.0];
    s.begin(
        StrokePoint {
            pos: start,
            pressure: 1.0,
        },
        &mut out,
    );
    dabs.extend(out.iter().map(|d| d.center));
    s.take_threads(&mut threads);
    all.append(&mut threads);
    for leg in 0..legs {
        #[allow(clippy::cast_precision_loss)]
        let x = start[0] + (leg as f32) * 6.0;
        for k in 1..=10 {
            #[allow(clippy::cast_precision_loss)]
            let up = if leg % 2 == 0 { 1.0 } else { -1.0 };
            let y = start[1] + up * (k as f32) * 4.0;
            s.extend(
                StrokePoint {
                    pos: [x, y],
                    pressure: 1.0,
                },
                &mut out,
            );
            dabs.extend(out.iter().map(|d| d.center));
            s.take_threads(&mut threads);
            all.append(&mut threads);
        }
    }
    (dabs, all)
}

/// **O TRAÇO COSTURA-SE A SI MESMO** — a frase da feature, medível: com o tipo armado nascem fios, e
/// cada um liga dois pontos que estão dentro do alcance.
#[test]
fn the_stroke_sews_itself_with_threads_between_neighbours() {
    let sp = spec(LineKind::Sketchy, 1.0);
    let reach = sp.sketchy_reach * 2.0 * sp.clamped_radius();
    let (dabs, threads) = zigzag(sp, 6);
    assert!(dabs.len() > 50, "controle: a fixture tem de emitir tinta");
    assert!(
        threads.len() > 100,
        "o traço não costurou nada: {} fios",
        threads.len()
    );
    for t in &threads {
        let len = (t[2] - t[0]).hypot(t[3] - t[1]);
        assert!(
            len <= reach + 1e-3,
            "um fio passou do alcance: {len:.2} px contra {reach:.2}"
        );
    }
}

/// **`Density = 0` NÃO COSTURA NADA, e o tipo NEUTRO também não** — as duas metades do neutro.
///
/// ⚠️ A segunda é a que importa: um `Vec` de memória que cresce em todo traço do app seria memória
/// paga por quem nunca escolheu o tipo, e o gate a pinaria como bug de custo, não de aparência.
#[test]
fn the_neutral_sews_nothing_and_remembers_nothing() {
    let (_, none) = zigzag(spec(LineKind::None, 1.0), 6);
    assert!(none.is_empty(), "o tipo None costurou {} fios", none.len());
    let (_, zero) = zigzag(spec(LineKind::Sketchy, 0.0), 6);
    assert!(zero.is_empty(), "Density 0 costurou {} fios", zero.len());
    // ⚠️ E o CONTROLE: com o tipo armado a MESMA fixture costura, senão as duas asserções acima são
    // verdadeiras de graça.
    let (_, armed) = zigzag(spec(LineKind::Sketchy, 1.0), 6);
    assert!(!armed.is_empty(), "controle: a fixture tem de costurar");
}

/// **UM FIO NASCE ENTRE PONTOS DO MESMO TRAÇO, NUNCA DO ANTERIOR** — a memória morre no `begin`.
///
/// ⚠️ Sem isto o Sketchy costuraria o desenho INTEIRO à medida que o artista trabalha: o segundo
/// traço ligaria fios ao primeiro, e apagar um deixaria a teia do outro pendurada no vazio.
///
/// **Mutação que sangra:** a memória não ser limpa em [`Stroke::begin`].
#[test]
fn a_thread_never_reaches_back_into_the_previous_stroke() {
    let sp = spec(LineKind::Sketchy, 1.0);
    let mut s = Stroke::new(sp, plain_dynamics(), 7);
    let mut out = Vec::new();
    let mut threads = Vec::new();
    // Traço 1, à esquerda.
    let lay = |s: &mut Stroke, x0: f32, out: &mut Vec<_>, th: &mut Vec<[f32; 4]>| {
        let mut got: Vec<[f32; 4]> = Vec::new();
        s.begin(
            StrokePoint {
                pos: [x0, 200.0],
                pressure: 1.0,
            },
            out,
        );
        s.take_threads(th);
        got.append(th);
        for k in 1..=20 {
            #[allow(clippy::cast_precision_loss)]
            let y = 200.0 + (k as f32) * 2.0;
            s.extend(
                StrokePoint {
                    pos: [x0, y],
                    pressure: 1.0,
                },
                out,
            );
            s.take_threads(th);
            got.append(th);
        }
        s.finish(out);
        got
    };
    let first = lay(&mut s, 200.0, &mut out, &mut threads);
    assert!(!first.is_empty(), "controle: o 1º traço tem de costurar");
    // Traço 2 a 5 px do primeiro — bem DENTRO do alcance de 24 px, então um fio entre traços seria
    // geometricamente possível. É isso que torna o gate não-vácuo.
    let second = lay(&mut s, 205.0, &mut out, &mut threads);
    assert!(!second.is_empty(), "controle: o 2º traço tem de costurar");
    for t in &second {
        assert!(
            t[0] >= 204.0 && t[2] >= 204.0,
            "um fio do 2º traço alcançou o 1º: [{:.1} {:.1}] → [{:.1} {:.1}]",
            t[0],
            t[1],
            t[2],
            t[3]
        );
    }
}

/// **SOB SIMETRIA, O FIO ESPELHADO LIGA OS DABS ESPELHADOS** — e é este gate que impede a lei de
/// simetria de existir em duas cópias que divergem.
///
/// ⚠️ Um fio é publicado por um canal PRÓPRIO (ele é um segmento, não um dab), então nada no
/// compilador força as duas a concordarem. O oráculo é geométrico: toda ponta de fio tem de cair
/// **sobre um centro de dab**, do mesmo lado do eixo.
///
/// **Mutação que sangra:** o `push_symmetric_segment` refletir sem transladar pelo centro do eixo.
#[test]
fn under_symmetry_a_thread_ends_on_mirrored_dab_centres() {
    let mut sp = spec(LineKind::Sketchy, 1.0);
    sp.symmetry = SymmetrySettings {
        enabled: true,
        circular: false,
        axis: MirrorAxis::Y,
        center: [200.0, 200.0],
        ..Default::default()
    };
    let (dabs, threads) = zigzag(sp, 4);
    assert!(!threads.is_empty(), "controle: a fixture tem de costurar");
    let on_a_dab = |p: [f32; 2]| dabs.iter().any(|d| (d[0] - p[0]).hypot(d[1] - p[1]) < 0.01);
    // ⚠️ **O controle é a CONTAGEM, não uma coordenada:** afirmar *"algum fio tem x < 199"* amarra o
    // gate a qual eixo o `MirrorAxis` espelha, e ele reprovaria um produto correto no dia em que a
    // fixture escolhesse o outro. A simetria de espelho dobra os fios — isso é verdade em todo eixo.
    let mut plain = spec(LineKind::Sketchy, 1.0);
    plain.symmetry = SymmetrySettings::default();
    let (_, base) = zigzag(plain, 4);
    assert_eq!(
        threads.len(),
        2 * base.len(),
        "a simetria de espelho tem de dobrar os fios: {} contra {}",
        threads.len(),
        base.len()
    );
    for t in &threads {
        assert!(
            on_a_dab([t[0], t[1]]) && on_a_dab([t[2], t[3]]),
            "a ponta de um fio não caiu sobre um centro de dab: [{:.2} {:.2}] → [{:.2} {:.2}]",
            t[0],
            t[1],
            t[2],
            t[3]
        );
    }
}

/// **O SKETCHY NÃO MEXE NO JITTER** — ele sorteia de um fluxo de RNG PRÓPRIO.
///
/// ⚠️ O `rng` do traço é o mesmo que serve posição, escala, rotação e cor por dab. Se o Sketchy o
/// consumisse, **ligar o tipo mudaria o jitter do pincel** — em silêncio, e sem nada na tela dizendo
/// por quê. Dois consumidores, dois fluxos.
///
/// **Mutação que sangra:** o sorteio do fio ler `self.rng`.
#[test]
fn sewing_threads_does_not_disturb_the_brushs_jitter() {
    let mut sp = spec(LineKind::None, 1.0);
    sp.jitter = 0.5; // scatter de posição vivo: é ele que denuncia um fluxo compartilhado
    let (plain, _) = zigzag(sp, 4);
    sp.line_kind = LineKind::Sketchy;
    let (sewn, threads) = zigzag(sp, 4);
    assert!(!threads.is_empty(), "controle: a fixture tem de costurar");
    assert_eq!(
        plain, sewn,
        "ligar o Sketchy moveu a tinta: o sorteio dos fios está consumindo o RNG do jitter"
    );
}

/// **O TETO DA DENSIDADE É O QUE O SLIDER ENTREGA** — e ele deposita mais fio que o alcance cheio
/// não depositaria, senão o controle não existe.
#[test]
fn the_density_ceiling_is_the_budget_the_slider_hands_out() {
    let (_, low) = zigzag(spec(LineKind::Sketchy, SKETCHY_DENSITY_MAX), 8);
    let (_, full) = zigzag(spec(LineKind::Sketchy, 1.0), 8);
    assert!(!low.is_empty(), "o teto orçado não costura nada");
    assert!(
        full.len() > 4 * low.len(),
        "a densidade não governa o gasto: {} fios no teto contra {} na cheia",
        low.len(),
        full.len()
    );
}
