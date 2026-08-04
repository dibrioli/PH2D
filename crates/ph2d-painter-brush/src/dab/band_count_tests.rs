//! Os gates da **porta única da divisão** ([`super::bands::band_count_with`]).
//!
//! ⚠️ **Eles interrogam a face PURA de propósito.** A irmã que shipa pergunta à máquina quantos núcleos
//! existem, e uma máquina não é uma fixture: um gate sobre ela afirmaria *"aqui deu 32"*, que é sobre o
//! hardware do dia. Injetar o `cores` é o que torna a regra observável — e é a mesma razão pela qual o
//! `band_count` delega em vez de duplicar a aritmética.

use super::bands::{PARALLEL_MIN_AREA, SPAWN_EQUIV_VISITS, band_count_with};

/// **O piso é DERIVADO, não escolhido** — e este gate é o que impede alguém de re-escolhê-lo.
///
/// `PARALLEL_MIN_AREA` tem de ser *exatamente* a menor área em que a contagem alcança 2: um texel
/// abaixo dela a resposta é serial, e nela a divisão começa. Se o piso virar um literal outra vez, as
/// duas metades divergem e este gate sangra — que é o defeito que a wave de 2026-08-04 encontrou vivo
/// (um piso escolhido para UM ponto de operação, aplicado como se fosse propriedade do trabalho).
#[test]
fn the_floor_is_exactly_where_the_count_reaches_two() {
    const CORES: usize = 32;
    const ROWS: usize = 4096;
    assert_eq!(
        band_count_with(PARALLEL_MIN_AREA - 1, ROWS, 0, CORES),
        1,
        "um texel abaixo do piso a divisão ainda não se paga"
    );
    assert!(
        band_count_with(PARALLEL_MIN_AREA, ROWS, 0, CORES) >= 2,
        "no piso a divisão TEM de começar — senão o piso não é o piso"
    );
    // E a derivação em si: o piso é o ponto em que a raiz alcança 2, ou seja `4 × SPAWN_EQUIV_VISITS`.
    assert_eq!(
        PARALLEL_MIN_AREA,
        SPAWN_EQUIV_VISITS * 4,
        "o piso deixou de ser derivado do custo de abrir uma thread"
    );
}

/// **A contagem cresce com o TRABALHO** — a propriedade inteira da wave, num gate sem relógio.
///
/// ⚠️ O defeito que ela corrige é que a contagem era **constante**: `available_parallelism()` devolvia
/// o mesmo número para um dab de 7 k visitas e para um de 500 k, então o pequeno pagava 32 spawns por
/// 3 µs de trabalho cada. Uma mutação que volte a devolver `cores` sempre sangra aqui na primeira
/// linha da tabela.
#[test]
fn the_band_count_grows_with_the_work() {
    const CORES: usize = 32;
    const ROWS: usize = 4096;
    let mut prev = 0usize;
    for area in [
        0usize, 1_000, 3_232, 8_000, 17_161, 33_489, 67_081, 133_225, 528_529, 4_000_000,
    ] {
        let n = band_count_with(area, ROWS, 0, CORES);
        assert!(
            n >= prev,
            "a contagem tem de ser monotônica no trabalho: {area} deu {n} depois de {prev}"
        );
        assert!(n <= CORES, "a contagem passou dos núcleos: {area} deu {n}");
        prev = n;
    }
    // …e a ponta de baixo NÃO divide, a de cima satura nos núcleos: sem estas duas a monotonia
    // sozinha ficaria verde para uma constante.
    assert_eq!(band_count_with(1_000, ROWS, 0, CORES), 1);
    assert_eq!(band_count_with(4_000_000, ROWS, 0, CORES), CORES);
}

/// **O trabalho não é a única régua: as LINHAS e os NÚCLEOS capam.**
///
/// Uma banda é um bloco de linhas, então não pode haver mais bandas que linhas — e mais threads que
/// núcleos só troca spawn por troca de contexto.
#[test]
fn the_count_is_capped_by_the_rows_and_by_the_cores() {
    assert_eq!(
        band_count_with(4_000_000, 3, 0, 32),
        3,
        "três linhas não viram trinta e duas bandas"
    );
    assert_eq!(
        band_count_with(4_000_000, 4096, 0, 4),
        4,
        "quatro núcleos não viram trinta e duas bandas"
    );
    assert_eq!(
        band_count_with(4_000_000, 1, 0, 32),
        1,
        "uma linha só nunca se divide"
    );
    assert_eq!(
        band_count_with(4_000_000, 4096, 0, 0),
        1,
        "uma máquina que reporta zero núcleos ainda tem de pintar"
    );
}

/// **O syscall é perguntado UMA vez por processo** — o gate de um defeito que só um relógio vê.
///
/// ⚠️ **Ele existe porque eu o introduzi e a sonda o pegou:** ao criar a porta única eu pus o
/// `available_parallelism()` no topo dela, e a rota em banda de um lote de 128 dabs foi de **0,99 para
/// 9,79 ms** — 10×, com a MESMA contagem de bandas. O `band_split` roda uma vez por dab POR BANDA
/// (128 × 32 = 4096 chamadas), e ali um syscall por chamada é o custo inteiro.
///
/// ⚠️ **Nenhum gate de comportamento podia ver isso**, porque a saída é byte-idêntica: a contagem de
/// bandas não move um pixel. Por isso a propriedade é afirmada sobre a FONTE — a única forma de o
/// próximo `cores()` escrito à mão nascer vermelho em vez de custar outro smoke.
#[test]
fn the_core_count_is_asked_once_per_process() {
    let src = include_str!("bands.rs");
    let calls = src
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//") && !t.starts_with("///") && t.contains("available_parallelism")
        })
        .count();
    assert_eq!(
        calls, 1,
        "o `available_parallelism()` tem de aparecer UMA vez no `bands.rs`, dentro do `cores()` — \
         ele é um syscall e este caminho roda uma vez por dab POR BANDA ({calls} chamadas)"
    );
    assert!(
        src.contains("static CORES: std::sync::OnceLock<usize>"),
        "a resposta tem de ser memoizada por processo, não re-perguntada ao kernel do SO"
    );
}

/// **A porta de ABLAÇÃO continua fechando**, e é dela que todo gate de identidade depende.
///
/// `min_area = usize::MAX` é como os gates pedem *"rode o laço `for d in dabs` de antes desta wave"*.
/// Se ela deixasse de forçar serial, a comparação byte-a-byte viraria uma rota contra ela mesma —
/// verde sobre nada, para a suíte inteira de uma vez.
#[test]
fn the_ablation_door_still_forces_serial() {
    for area in [0usize, 3_232, 528_529, usize::MAX - 1] {
        assert_eq!(
            band_count_with(area, 4096, usize::MAX, 32),
            1,
            "min_area = usize::MAX tem de devolver serial (area={area})"
        );
    }
}
