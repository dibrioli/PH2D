//! **As bandas são cortadas por TRABALHO** — os gates do lote ESPARSO.
//!
//! Irmão do `stamp_banded_tests` por ASSUNTO: lá mora *"dividir as linhas não move um byte"*, aqui
//! *"as linhas são divididas de modo que as bandas paguem o mesmo"*. As duas perguntas têm oráculos
//! diferentes — a primeira compara pixels, a segunda compara relógios — e é a segunda que a fixture
//! densa daquele arquivo **não consegue fazer**: num arco toda linha da união recebe trabalho, então
//! cortar por altura e cortar por trabalho dão bandas parecidas e a diferença é invisível.

use super::stamp_banded::{
    BATCH_MIN_AREA, batch_bounds, row_work, stamp_plain_dabs_banded_with, work_bands,
};
use super::stamp_banded_tests::{H, W, arc, brush, identity_of};
use ph2d_painter_brush::Dab;

/// **O lote ESPARSO** — a figura no centro mais um punhado de dabs num canto distante.
///
/// ⚠️ **Não é uma cena exótica montada para o teste passar:** é o que Tiling produz (as cópias
/// embrulhadas vão para a borda oposta), o que Symmetry produz (o espelho cai do outro lado da tela)
/// e o que uma segunda forma na tela produz. A união fica quase do tamanho do canvas e o miolo dela
/// está VAZIO.
fn sparse(n: usize, radius: f32) -> Vec<Dab> {
    let mut v = arc(n, radius);
    for (i, d) in arc(6, 18.0).into_iter().enumerate() {
        v.push(Dab {
            #[allow(clippy::cast_precision_loss)]
            center: [30.0 + (i as f32) * 9.0, 26.0],
            ..d
        });
    }
    v
}

/// A identidade do gate irmão, agora sobre o lote que a wave endereça.
///
/// ⚠️ Ele reusa a MESMA `identity_of` — um segundo comparador aqui seria uma segunda resposta a *"as
/// duas rotas concordam?"*, e a que esquecesse o retângulo ficaria verde sobre metade da pergunta.
#[test]
fn the_banded_batch_is_identical_on_a_sparse_batch_too() {
    for n in [2usize, 17, 200] {
        for radius in [40.0f32, 160.0] {
            identity_of(&sparse(n, radius), &format!("esparso n={n} r={radius}"));
        }
    }
}

/// **O corte reparte o trabalho, não a altura** — a propriedade, sem relógio.
///
/// Um perfil com todo o trabalho concentrado num quinto das linhas: cortado por ALTURA, quatro de
/// cinco bandas ficam vazias; cortado por TRABALHO, as cinco pagam o mesmo.
#[test]
fn the_bands_split_the_work_not_the_height() {
    // 500 linhas, e só as 100 do meio têm trabalho.
    let mut rows = vec![0u32; 500];
    for r in &mut rows[200..300] {
        *r = 64;
    }
    let bands = work_bands(&rows, 5);

    // Nenhuma linha se perde nem é contada duas vezes.
    assert_eq!(
        bands.iter().sum::<usize>(),
        rows.len(),
        "as alturas têm de somar a união inteira: {bands:?}"
    );
    assert!(
        bands.iter().all(|&b| b > 0),
        "banda de altura ZERO é uma thread aberta para não fazer nada: {bands:?}"
    );

    // E o trabalho de cada banda fica perto da média.
    let mut y = 0usize;
    let per: Vec<u64> = bands
        .iter()
        .map(|&b| {
            let s: u64 = rows[y..y + b].iter().map(|&x| u64::from(x)).sum();
            y += b;
            s
        })
        .collect();
    let total: u64 = per.iter().sum();
    #[allow(clippy::cast_precision_loss)]
    let mean = total as f64 / per.len() as f64;
    #[allow(clippy::cast_precision_loss)]
    let worst = per.iter().map(|&x| x as f64).fold(0.0f64, f64::max);
    assert!(
        worst <= mean * 1.35,
        "a banda mais pesada carrega {worst} contra uma média de {mean:.0} — o corte não equilibra \
         ({bands:?} → {per:?})"
    );
}

/// **O trabalho por linha é o que a BANDA de fato executa** — a mesma `dab_write_bounds` do lote.
///
/// ⚠️ Uma segunda estimativa (a área do círculo, o raio) equilibraria uma grandeza que ninguém paga:
/// a banda percorre o retângulo DECLARADO de cada dab, então é ele que tem de ser somado.
#[test]
fn the_row_profile_is_the_declared_width_of_each_dab() {
    let dabs = arc(40, 90.0);
    let bounds = batch_bounds(&dabs, W, H).expect("a fixture escreve");
    let rows = row_work(&dabs, W, H, bounds);
    assert_eq!(
        rows.len(),
        bounds.h as usize,
        "uma entrada por linha da união"
    );
    let declared: u64 = dabs
        .iter()
        .filter_map(|d| ph2d_painter_brush::dab_write_bounds(d.center, d.radius_px, W, H))
        .map(|b| u64::from(b.w) * u64::from(b.h))
        .sum();
    let profiled: u64 = rows.iter().map(|&x| u64::from(x)).sum();
    assert_eq!(
        profiled, declared,
        "o perfil por linha tem de somar exatamente a área declarada do lote"
    );
}

/// **Um lote sem trabalho nenhum devolve UMA banda** — o degenerado que não se divide.
#[test]
fn a_batch_with_no_work_is_one_band() {
    assert_eq!(work_bands(&[0, 0, 0, 0], 8), vec![4]);
    assert_eq!(work_bands(&[], 8), vec![0]);
    assert_eq!(work_bands(&[3, 3, 3], 1), vec![3]);
}

/// **Nenhuma banda nasce com altura ZERO** — e a fixture tem o trabalho no FIM do perfil, que é o
/// único lugar onde a guarda pode morder.
///
/// ⚠️ **A primeira versão deste gate punha o trabalho no MEIO e a mutação que remove a guarda PASSAVA:**
/// com o trabalho no meio o quantil sempre cai numa linha com linhas de sobra, e a guarda nunca é
/// consultada. Ela só é alcançada quando o trabalho está tão perto do fim que o corte pediria mais
/// bandas do que restam linhas — a fixture TEM de conter o fenômeno.
#[test]
fn no_band_is_born_with_zero_height() {
    for (rows, bands) in [
        (vec![0u32, 0, 0, 100], 4usize),
        (vec![0, 0, 0, 0, 0, 7, 9], 5),
        (vec![5], 8),
        (vec![0, 1], 4),
    ] {
        let out = work_bands(&rows, bands);
        assert_eq!(
            out.iter().sum::<usize>(),
            rows.len(),
            "as alturas têm de somar a união ({rows:?}, {bands} bandas) → {out:?}"
        );
        assert!(
            out.iter().all(|&b| b > 0),
            "banda de altura ZERO é uma thread aberta para não fazer nada ({rows:?}, {bands} \
             bandas) → {out:?}"
        );
        assert!(
            out.len() <= bands,
            "mais bandas do que threads pedidas ({rows:?}, {bands}) → {out:?}"
        );
    }
}

/// **A CONSEQUÊNCIA, que só um relógio pode ver:** espalhar o MESMO lote não pode custar mais caro.
///
/// Os dois lotes têm a mesma contagem de dabs, o mesmo trabalho declarado e pintam a mesma
/// quantidade de tinta — só a esparsidade da união muda. Com bandas de altura uniforme a razão medida
/// era **2,07×** (o carimbo do lote espalhado contra o do colado, 4096², medido pela porta do produto
/// em 2026-08-06); com o corte por trabalho ela é ~1,0.
///
/// ⚠️ **É uma RAZÃO, não um wall-clock:** a barra tem de sobreviver a uma máquina carregada e a um
/// perfil de compilação diferente, e as duas metades pagam a mesma deriva.
/// ⚠️ **A fixture é GRANDE de propósito, e a primeira versão deste gate não era.** Com a tela de 512²
/// e o pincel de raio 12 dos gates de identidade a união rala tem poucas centenas de linhas, a fatia
/// vazia de cada banda é curta, e **a mutação que reinstala o corte por altura PASSA** — medido. O
/// desequilíbrio só é observável quando a união é grande o bastante para que a maioria das bandas
/// caia inteira no vão: é por isso que aqui a tela é 2048², o pincel tem raio 40 e o aglomerado
/// distante fica no canto oposto.
#[test]
#[ignore = "clock — run explicitly with --test-threads=1"]
fn spreading_the_same_batch_does_not_make_it_cost_more() {
    const S: u32 = 2048;
    let big = |c: [f32; 2], n: usize, radius: f32| -> Vec<Dab> {
        (0..n)
            .map(|i| {
                #[allow(clippy::cast_precision_loss)]
                let t = (i as f32) / (n as f32) * std::f32::consts::TAU;
                let (s, co) = (t.sin(), t.cos());
                Dab {
                    center: [c[0] + co * radius, c[1] + s * radius],
                    radius_px: 40.0,
                    coverage: 0.6,
                    #[allow(clippy::cast_precision_loss)]
                    color: [(i % 7) as f32 / 7.0, 0.3, 0.9],
                    rotation: [1.0, 0.0],
                    dir: [co, s],
                    arc_len: 0.0,
                    stroke_radius_px: 40.0,
                }
            })
            .collect()
    };
    let mut spec = brush();
    spec.radius_px = 40.0;
    let ms = |dabs: &[Dab]| -> f64 {
        let mut best = f64::MAX;
        for _ in 0..7 {
            let mut buf = vec![255u8; (S as usize) * (S as usize) * 4];
            let t0 = std::time::Instant::now();
            let _ = stamp_plain_dabs_banded_with(
                &mut buf,
                S,
                S,
                dabs,
                &spec,
                false,
                None,
                BATCH_MIN_AREA,
            );
            best = best.min(t0.elapsed().as_secs_f64() * 1e3);
        }
        best
    };
    // A MESMA figura COMPACTA e o MESMO aglomerado extra; só o lugar do aglomerado muda.
    // ⚠️ A figura é compacta de propósito: é a razão *linhas com trabalho / linhas da união* que
    // decide quantas bandas caem no vão, e uma figura que já cobre a união inteira não tem vão.
    let figure = big([1024.0, 1720.0], 300, 180.0);
    let mut tight = figure.clone();
    tight.extend(big([1024.0, 1450.0], 30, 60.0));
    let mut far = figure;
    far.extend(big([140.0, 140.0], 30, 60.0));

    let (a, b) = (ms(&tight), ms(&far));
    assert!(
        b <= a * 1.45,
        "espalhar o MESMO lote custou {:.2}x ({b:.3} ms contra {a:.3}) — as bandas voltaram a ser \
         cortadas por altura e a maioria delas está vazia",
        b / a.max(1e-9)
    );
}
