//! Os gates dos tipos que **costuram fios** — `Sketchy` (plano 38 W3) e `Wire` (W4).
//!
//! O que a família promete: a cada dab, fios de opacidade baixa ligam pontos da memória do traço.
//! Os gates perguntam pelos FIOS, não pela fórmula — e o que separa os dois tipos é **qual** par eles
//! escolhem: o Sketchy pergunta *quem está perto no CANVAS*, o Wire *quem está perto no PERCURSO*.

use crate::dynamics::Dynamics;
use crate::falloff::Falloff;
use crate::line_kind::{LineKind, SKETCHY_DENSITY_MAX, WIRE_CURVES_PER_DAB};
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

/// Um GRAMPO: o traço vai para a direita ao longo de `y = 200`, dá a volta, e volta para a esquerda
/// **6 px abaixo** — bem dentro do alcance de 24 px, e a um ARCO enorme de distância. Devolve os
/// fios.
///
/// ⚠️ É a fixture que o Magnetify distingue, e ela existe porque o zigue-zague **não** a distingue:
/// lá as pernas ficam perto no canvas **e** perto no percurso, então os dois modos costuram igual.
fn hairpin(sp: BrushSpec) -> Vec<[f32; 4]> {
    let mut s = Stroke::new(sp, plain_dynamics(), 7);
    let mut out = Vec::new();
    let mut threads = Vec::new();
    let mut all: Vec<[f32; 4]> = Vec::new();
    let mut lay = |s: &mut Stroke, p: [f32; 2], first: bool| {
        let pt = StrokePoint {
            pos: p,
            pressure: 1.0,
        };
        if first {
            s.begin(pt, &mut out);
        } else {
            s.extend(pt, &mut out);
        }
        s.take_threads(&mut threads);
        all.append(&mut threads);
    };
    lay(&mut s, [200.0, 200.0], true);
    for k in 1..=30 {
        #[allow(clippy::cast_precision_loss)]
        lay(&mut s, [200.0 + (k as f32) * 4.0, 200.0], false);
    }
    for k in 0..=30 {
        #[allow(clippy::cast_precision_loss)]
        lay(&mut s, [320.0 - (k as f32) * 4.0, 206.0], false);
    }
    all
}

/// **MAGNETIFY: LIGADO, O TRAÇO COSTURA-SE A DOIS TRECHOS SEPARADOS; DESLIGADO, SÓ À PORÇÃO ATIVA.**
///
/// ⚠️ A lei é a do manual do Krita, verbatim: *"It's what causes curve lines to form between two
/// close line sections … With Magnetify off, the curve line just forms on either side of the current
/// active portion of your connection line."* Ele escolhe QUE PARES viram fio — **não** com que força
/// um fio desenha, que foi como esta wave o construiu primeiro (a rampa de opacidade por distância
/// morreu em `thread_raster`, e o que ela fazia é o *Distance Opacity* do Krita, outro controle).
///
/// **O oráculo é GEOMÉTRICO:** um fio que atravessa as duas pernas do grampo liga `y = 200` a
/// `y = 206`; um fio dentro de uma perna tem as duas pontas no mesmo `y`. E o CONTROLE é a segunda
/// metade — desligado ele **continua costurando** (a porção ativa), senão o gate confundiria *"o
/// Magnetify recusa o par distante"* com *"o Sketchy parou de funcionar"*.
///
/// ⚠️ **A DOBRA é excluída, e o gate nasceu VERMELHO por não a excluir** (51 fios cross-perna com o
/// Magnetify desligado). Na volta do grampo as duas pernas estão a ~6 px de percurso uma da outra:
/// ali elas **são** a porção ativa, e costurá-las é o comportamento CERTO nos dois modos. O que
/// distingue os modos é o outro extremo do grampo, a 240 px de arco — daí o corte em `x < 280`.
///
/// **Mutação que sangra:** a janela de arco ignorar o flag (⇒ o OFF costura entre as pernas).
#[test]
fn magnetify_sews_across_sections_and_without_it_only_the_active_portion() {
    // Cross-perna e LONGE da dobra (que fica em x = 320): é ali que os dois modos discordam.
    let crosses = |ts: &[[f32; 4]]| {
        ts.iter()
            .filter(|t| (t[3] - t[1]).abs() > 3.0 && t[0] < 280.0 && t[2] < 280.0)
            .count()
    };

    let mut on = spec(LineKind::Sketchy, 1.0);
    on.sketchy_magnetify = true;
    let a = hairpin(on);
    assert!(
        crosses(&a) > 0,
        "com Magnetify o traço TEM de ligar as duas pernas"
    );

    let mut off = spec(LineKind::Sketchy, 1.0);
    off.sketchy_magnetify = false;
    let b = hairpin(off);
    // ⚠️ **O CONTROLE é a CAUDA, não `!is_empty()`** — e a diferença foi MEDIDA por uma mutação que
    // sobreviveu: com a memória guardando arco `0.0` em vez do `Dab::arc_len`, o modo OFF costura os
    // primeiros 24 px do traço e depois **emudece para sempre**, o que `!is_empty()` aceita. A porção
    // ativa VIAJA com o dedo, então tem de haver fio no FIM do grampo.
    //
    // ⚠️ E o fim é `x` pequeno **na perna de VOLTA** (`y = 206`): o grampo começa e termina no mesmo
    // `x`, então um corte só em `x` mede o COMEÇO do traço — que é exatamente o pedaço que a mutação
    // preserva, e foi assim que ela sobreviveu à primeira versão deste controle.
    let tail = |ts: &[[f32; 4]]| {
        ts.iter()
            .filter(|t| t[1] > 203.0 && t[3] > 203.0 && t[0] < 220.0 && t[2] < 220.0)
            .count()
    };
    assert!(
        tail(&b) > 0,
        "controle: sem Magnetify a porção ativa tem de VIAJAR — nada costurado no fim do traço"
    );
    assert_eq!(
        crosses(&b),
        0,
        "sem Magnetify um fio alcançou a outra perna do grampo: {} de {}",
        crosses(&b),
        b.len()
    );
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

/// **A DENSIDADE É O ORÇAMENTO: o gasto é PROPORCIONAL a ela.**
///
/// ⚠️ **A barra é a PROPORÇÃO, não um fator escolhido.** A versão anterior pedia `cheia > 4 × teto`,
/// um número calibrado contra um teto de `0,04` — e quando a medição do produto subiu o teto para
/// `0,40` (dez vezes; ver o doc da constante) o gate reprovou um produto correto. A lei do motor é
/// *cada par candidato vira fio com probabilidade `density`*, então o que se afirma é isso: dobrar a
/// densidade dobra os fios, seja qual for o teto do dia.
#[test]
fn the_density_is_the_budget_and_the_spend_is_proportional_to_it() {
    let count = |d: f32| zigzag(spec(LineKind::Sketchy, d), 8).1.len();
    let base = count(0.1);
    assert!(
        base > 50,
        "controle: a fixture tem de costurar ({base} fios)"
    );
    for mult in [2.0f32, 5.0, 10.0] {
        let got = count(0.1 * mult);
        #[allow(clippy::cast_precision_loss)]
        let ratio = got as f32 / base as f32;
        assert!(
            (ratio - mult).abs() < 0.25 * mult,
            "densidade {mult}× deu {ratio:.2}× os fios ({got} contra {base})"
        );
    }
    // E o teto que o produto SUSTENTA costura — senão o slider entrega uma faixa que não pinta.
    assert!(
        !zigzag(spec(LineKind::Sketchy, SKETCHY_DENSITY_MAX), 8)
            .1
            .is_empty()
    );
}

// ---------------------------------------------------------------------------------------------
// WIRE (plano 38 W4) — o mesmo produtor com uma janela de ARCO.
// ---------------------------------------------------------------------------------------------

fn wire_spec(history: f32) -> BrushSpec {
    BrushSpec {
        radius_px: 12.0,
        spacing: 0.1,
        falloff: Falloff::Constant,
        space_attenuation: false,
        stabilizer: 0.0,
        line_kind: LineKind::Wire,
        wire_history: history,
        ..Default::default()
    }
}

/// Um traço RETO de `len` px, com o espaçamento pedido. Devolve `(dabs, fios)`.
///
/// ⚠️ Reto de propósito: é a fixture em que a janela de arco e a janela de contagem de pontos
/// **divergem visivelmente** quando o espaçamento muda, e onde o comprimento de uma corda é
/// exatamente o arco que ela cobre.
fn straight(sp: BrushSpec, len: f32, step: f32) -> (Vec<[f32; 2]>, Vec<[f32; 4]>) {
    let mut s = Stroke::new(sp, plain_dynamics(), 7);
    let mut out = Vec::new();
    let mut threads = Vec::new();
    let mut dabs: Vec<[f32; 2]> = Vec::new();
    let mut all: Vec<[f32; 4]> = Vec::new();
    s.begin(
        StrokePoint {
            pos: [100.0, 300.0],
            pressure: 1.0,
        },
        &mut out,
    );
    dabs.extend(out.iter().map(|d| d.center));
    s.take_threads(&mut threads);
    all.append(&mut threads);
    let mut x = 100.0f32;
    while x < 100.0 + len {
        x += step;
        s.extend(
            StrokePoint {
                pos: [x, 300.0],
                pressure: 1.0,
            },
            &mut out,
        );
        dabs.extend(out.iter().map(|d| d.center));
        s.take_threads(&mut threads);
        all.append(&mut threads);
    }
    (dabs, all)
}

/// **O WIRE OLHA PARA O PERCURSO, NÃO PARA O CANVAS** — o discriminante contra o irmão.
///
/// ⚠️ **É a MESMA fixture do Magnetify**, e não por economia: o grampo é precisamente a cena em que
/// as duas perguntas divergem. As duas pernas estão a **6 px no canvas** e a **240 px de arco** uma
/// da outra na ponta medida.
///
/// ⚠️ **A comparação é feita com o MESMO número — um diâmetro para os dois** (o Sketchy alcança
/// 24 px de canvas; o Wire, 24 px de arco). Sem isso o gate provaria só que uma janela é menor que a
/// outra; com isso ele prova que a PERGUNTA é outra.
///
/// ⚠️ **E o gate nasceu VERMELHO sobre um produto CORRETO** (31 fios cross-perna), porque a 1ª versão
/// deu ao Wire uma janela de 6 diâmetros = 144 px de arco contra um grampo de 240 px no total: **a
/// dobra estava DENTRO da janela**, e alcançar a outra perna ali é a lei do Wire a funcionar, não a
/// falhar. O fato fica escrito para ninguém "consertar" o motor: *um arame CRUZA uma dobra que caiba
/// na janela dele* — o que ele nunca faz é cruzar por proximidade no canvas.
///
/// **Mutação que sangra:** o Wire testar a distância no canvas em vez do arco.
#[test]
fn the_wire_looks_along_the_path_where_the_sketchy_looks_across_the_canvas() {
    let crosses = |ts: &[[f32; 4]]| {
        ts.iter()
            .filter(|t| (t[3] - t[1]).abs() > 3.0 && t[0] < 280.0 && t[2] < 280.0)
            .count()
    };
    let sketchy = hairpin(spec(LineKind::Sketchy, 1.0));
    assert!(
        crosses(&sketchy) > 0,
        "controle: o Sketchy costura entre as pernas (é o que o canvas vê)"
    );
    let wire = hairpin(wire_spec(1.0));
    assert!(
        wire.len() > 100,
        "controle: o Wire tem de costurar ao longo do percurso ({} fios)",
        wire.len()
    );
    assert_eq!(
        crosses(&wire),
        0,
        "o Wire ligou as duas pernas do grampo: {} de {} fios",
        crosses(&wire),
        wire.len()
    );
}

/// **A JANELA É DE ARCO, NÃO DE CONTAGEM DE PONTOS** — o mesmo gesto com o DOBRO dos dabs desenha o
/// mesmo arame.
///
/// ⚠️ **É a lei que esta casa já pagou quatro vezes no relevo:** *a lei é função do CAMINHO, nunca de
/// quão fino o motor amostrou o caminho*. Uma janela em pontos (que é como o Krita a mede) faria
/// apertar o Spacing encurtar o arame **sem ninguém tocar no slider**.
///
/// O oráculo é o COMPRIMENTO DA MAIOR CORDA — num traço reto ele é o arco que a janela cobre, e não
/// depende de quantos dabs couberam nela.
///
/// **Mutação que sangra:** a janela contar pontos (`thread_pts.len() - k`) em vez de arco.
#[test]
fn the_wire_window_is_measured_in_arc_not_in_dabs() {
    let longest = |ts: &[[f32; 4]]| {
        ts.iter()
            .map(|t| (t[2] - t[0]).hypot(t[3] - t[1]))
            .fold(0.0f32, f32::max)
    };
    let mut fine = wire_spec(2.0);
    fine.spacing = 0.05; // dabs a ~1,2 px
    let mut coarse = wire_spec(2.0);
    coarse.spacing = 0.2; // dabs a ~4,8 px — QUATRO vezes menos pontos no mesmo arco
    let (df, tf) = straight(fine, 400.0, 3.0);
    let (dc, tc) = straight(coarse, 400.0, 3.0);
    assert!(
        df.len() > 2 * dc.len(),
        "controle: os dois espaçamentos têm de dar contagens de dab bem diferentes ({} contra {})",
        df.len(),
        dc.len()
    );
    let (lf, lc) = (longest(&tf), longest(&tc));
    // A janela é 2 diâmetros = 48 px; a maior corda cobre a janela inteira.
    assert!(
        (lf - lc).abs() < 6.0,
        "o arame mudou de tamanho com o espaçamento: {lf:.1} px contra {lc:.1} px"
    );
    assert!(
        (lf - 48.0).abs() < 6.0,
        "a maior corda não cobre a janela de 48 px: {lf:.1} px"
    );
}

/// **A CONTAGEM DE CORDAS POR DAB É FIXA** — e é por isso que o Wire não tem slider de densidade.
///
/// ⚠️ *"Ligue aos últimos N"* lido ao pé da letra são `N` fios por dab, com `N` = quantos dabs
/// couberam na janela ⇒ o CUSTO passaria a ser função do Spacing. Amostrando posições igualmente
/// espaçadas em arco, a contagem é a constante — e o orçamento é conhecido antes de o artista tocar
/// num slider.
///
/// **Mutação que sangra:** ligar a todos os pontos da janela.
#[test]
fn the_wire_draws_a_fixed_number_of_cords_per_dab() {
    let sp = wire_spec(2.0);
    let (dabs, threads) = straight(sp, 400.0, 3.0);
    assert!(dabs.len() > 100, "controle: a fixture tem de emitir tinta");
    // Os primeiros dabs não têm janela cheia (as cordas são PULADAS), então a razão é um teto.
    #[allow(clippy::cast_precision_loss)]
    let per_dab = threads.len() as f32 / dabs.len() as f32;
    #[allow(clippy::cast_precision_loss)]
    let cap = WIRE_CURVES_PER_DAB as f32;
    assert!(
        per_dab <= cap + 0.01,
        "o Wire desenhou {per_dab:.2} cordas por dab, acima da constante de {cap}"
    );
    assert!(
        per_dab > cap * 0.8,
        "o Wire desenhou só {per_dab:.2} cordas por dab: a janela nunca encheu?"
    );
}

/// **NO COMEÇO DO TRAÇO AS CORDAS SÃO PULADAS, NÃO PRESAS À ORIGEM.**
///
/// ⚠️ Prendê-las no 1º ponto desenharia um LEQUE saindo da origem do traço — muitas cordas
/// compartilhando exatamente o mesmo ponto —, e é o que o manual do Krita descreve como o contrário
/// do certo: *"if you set this value too high, the curve lines will only start forming relatively
/// late"*. Formar tarde É o comportamento.
///
/// **Mutação que sangra:** trocar o `break` por um clamp no ponto mais antigo.
#[test]
fn the_wire_skips_the_cords_the_window_cannot_reach_yet() {
    let sp = wire_spec(4.0); // janela de 96 px
    let (_, threads) = straight(sp, 400.0, 3.0);
    assert!(!threads.is_empty(), "controle: a fixture tem de costurar");
    let at_origin = threads
        .iter()
        .filter(|t| (t[2] - 100.0).abs() < 0.5 && (t[3] - 300.0).abs() < 0.5)
        .count();
    assert!(
        at_origin <= WIRE_CURVES_PER_DAB,
        "{at_origin} cordas ancoraram na ORIGEM do traço: a janela vazia está a prender em vez de pular"
    );
}

/// **O NEUTRO DO WIRE** — `History = 0` não costura, e o CONTROLE prova que a fixture costuraria.
#[test]
fn the_wire_neutral_sews_nothing() {
    let (_, zero) = straight(wire_spec(0.0), 400.0, 3.0);
    assert!(zero.is_empty(), "History 0 costurou {} fios", zero.len());
    let (_, armed) = straight(wire_spec(2.0), 400.0, 3.0);
    assert!(!armed.is_empty(), "controle: a fixture tem de costurar");
}
