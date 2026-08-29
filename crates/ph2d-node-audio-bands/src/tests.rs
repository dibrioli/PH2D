//! Gates de `audio.bands`.
//!
//! ⚠️ Este crate é uma FOLHA e **não alcança um arquivo de som** — de propósito
//! (doc 63 §6: *"FFT NUNCA entra no cook"*). Então não há aqui um gate de *"a
//! música fez a barra subir"*: o que ele PODE afirmar é a **LEI** (o corte do
//! eixo, o pico dentro da banda, a normalização) e o contrato que a divergência do
//! shell atacaria. O caminho até a tela é gateado do lado do shell, onde a
//! transformada existe.

use super::*;
use ph2d_node_registry::NodeRegistry;

fn defaults(name: &str) -> f32 {
    MANIFEST
        .params
        .iter()
        .find(|p| p.name == name)
        .map(|p| p.default)
        .unwrap_or_else(|| panic!("param {name} existe no manifesto"))
}

fn spec() -> BandSpec {
    BandSpec::from_params(defaults)
}

/// **Todo param do manifesto entra na chave da análise, e vice-versa.** Um param
/// que o manifesto declara e a chave não vê fica **inerte depois da primeira vez**
/// — o artista mexe o controle, o cache do shell devolve a análise velha, e nada
/// acusa. É a mesma lei que o `source.text` enuncia sobre a chave de conteúdo.
#[test]
fn every_manifest_param_moves_the_analysis_key() {
    let base = spec().key("a.wav");
    for name in param::ALL {
        // Um degrau grande o bastante para atravessar o `round()` dos enums.
        let bumped = BandSpec::from_params(|n| {
            if n == *name {
                defaults(n) + 1.0
            } else {
                defaults(n)
            }
        })
        .key("a.wav");
        assert_ne!(bumped, base, "o param `{name}` tem de mexer a chave");
    }
    assert_ne!(spec().key("b.wav"), base, "o arquivo");
    // ...e a lista `param::ALL` não pode ter esquecido nenhum.
    let mut a: Vec<&str> = MANIFEST.params.iter().map(|p| p.name).collect();
    let mut b: Vec<&str> = param::ALL.to_vec();
    a.sort_unstable();
    b.sort_unstable();
    assert_eq!(a, b, "a chave e o manifesto listam os mesmos params");
}

/// **A chave é INJETIVA sobre (params, caminho), inclusive com dois-pontos no
/// caminho.**
///
/// ⚠️ **A 1ª versão deste gate afirmava o CONTRÁRIO da propriedade** — pedia que
/// `key("a:b")` diferisse de `key("a") + ":b"`, e as duas SÃO a mesma string,
/// porque o caminho vai por ÚLTIMO e nada o segue. *Isso não é a falha: é
/// exatamente o que "por último" significa*, e a forma certa de o afirmar é que
/// dois pedidos DIFERENTES nunca colidem.
///
/// ⚠️ **A cerca que isto guarda é para o dia em que um SEGUNDO campo de texto
/// entrar** (o `source.text` já pagou essa conta com a fonte antes do texto): a
/// partir daí o campo que deixa de ser o último precisa de **prefixo de
/// comprimento**, senão `a:b` + `c` e `a` + `b:c` mintem a mesma chave.
#[test]
fn a_colon_in_the_path_cannot_forge_another_key() {
    let bumped = BandSpec {
        count: spec().count + 1,
        ..spec()
    };
    let keys = [
        spec().key("a:b"),
        spec().key("a"),
        spec().key("a:b:c"),
        spec().key("/tmp/x.wav"),
        spec().key("/tmp/y.wav"),
        bumped.key("a:b"),
    ];
    for (i, a) in keys.iter().enumerate() {
        for b in &keys[i + 1..] {
            assert_ne!(a, b, "duas analises distintas partilham uma chave");
        }
    }
}

/// Os índices que um grafo salvo guarda. ⚠️ **APPEND ONLY** — mover um renomeia a
/// escolha de todo documento já autorado, em silêncio.
#[test]
fn the_stored_indices_do_not_move() {
    assert_eq!(Scale::from_index(0.0), Scale::Linear);
    assert_eq!(Scale::from_index(1.0), Scale::Log);
    assert_eq!(Scale::from_index(2.0), Scale::Mel);
    assert_eq!(Weighting::from_index(0.0), Weighting::None);
    assert_eq!(Weighting::from_index(1.0), Weighting::A);
    // E os defaults do manifesto são os que a doc do módulo afirma.
    assert_eq!(Scale::from_index(defaults(param::SCALE)), Scale::Log);
    assert_eq!(
        Weighting::from_index(defaults(param::WEIGHTING)),
        Weighting::None
    );
}

/// **O corte logarítmico dá larguras iguais em OITAVAS** — a razão entre fronteiras
/// vizinhas é constante. É a propriedade inteira da escala, e afirmá-la é o que
/// impede alguém de "simplificar" o `powf` para uma interpolação linear.
#[test]
fn the_log_axis_is_constant_in_octaves() {
    let s = BandSpec {
        count: 8,
        min_hz: 40.0,
        max_hz: 16_000.0,
        scale: Scale::Log,
        ..spec()
    };
    let e = s.edges();
    assert_eq!(e.len(), 9);
    assert!((e[0] - 40.0).abs() < 1e-3, "começa no mínimo");
    assert!((e[8] - 16_000.0).abs() < 1.0, "acaba no máximo");
    let r0 = e[1] / e[0];
    for k in 1..8 {
        let r = e[k + 1] / e[k];
        assert!(
            (r - r0).abs() < 1e-3,
            "razão constante: {r} contra {r0} na banda {k}"
        );
    }
    // O CONTROLE: a escala linear NÃO tem essa propriedade — sem ele, um `edges`
    // que devolvesse sempre linear passaria no teste acima se as barras fossem
    // poucas o bastante.
    let lin = BandSpec {
        scale: Scale::Linear,
        ..s
    }
    .edges();
    assert!(
        (lin[1] / lin[0] - lin[8] / lin[7]).abs() > 1.0,
        "a linear não é constante em oitavas"
    );
}

/// A mel é a inversa exata de si mesma, e fica ENTRE a linear e a log — mais
/// resolução nos graves que a log pura não dá.
#[test]
fn the_mel_axis_inverts_and_sits_between_the_other_two() {
    for hz in [40.0f32, 440.0, 1000.0, 8000.0] {
        let round = mel_to_hz(hz_to_mel(hz));
        assert!((round - hz).abs() < hz * 1e-4, "{hz} → {round}");
    }
    let base = BandSpec {
        count: 8,
        min_hz: 40.0,
        max_hz: 16_000.0,
        ..spec()
    };
    let mid = |s: Scale| BandSpec { scale: s, ..base }.edges()[4];
    let (lin, mel, log) = (mid(Scale::Linear), mid(Scale::Mel), mid(Scale::Log));
    assert!(
        log < mel && mel < lin,
        "log {log} < mel {mel} < linear {lin}"
    );
}

/// **Dentro de uma banda o nível é o PICO, não a média.** Uma banda larga com um
/// harmônico forte e silêncio em volta tem a média de um silêncio — a barra ficaria
/// parada durante a única coisa que havia para ver.
#[test]
fn a_band_takes_its_peak_not_its_average() {
    let s = BandSpec {
        count: 1,
        min_hz: 0.0,
        max_hz: 1000.0,
        scale: Scale::Linear,
        floor_db: -60.0,
        gain: 1.0,
        weighting: Weighting::None,
        smoothing: 0.0,
    };
    // Dez compartimentos: um alto, nove no chão.
    let mut db = vec![-60.0f32; 10];
    db[3] = 0.0;
    let mut out = Vec::new();
    fold(&db, 100.0, &s, &mut out);
    assert_eq!(out.len(), 1);
    assert!(out[0] > 0.99, "o pico manda: {}", out[0]);
    // A média daria ~0,1 — o número que este gate existe para recusar.
    let mean: f32 = db.iter().map(|d| (d + 60.0) / 60.0).sum::<f32>() / 10.0;
    assert!(mean < 0.2, "a média seria {mean}, e não é isto que sai");
}

/// **Uma banda estreita demais para caber num compartimento ainda lê UM.**
///
/// ⚠️ **E a MUTAÇÃO corrigiu o raciocínio que escreveu este gate:** eu supunha que o
/// `clamp(b0 + 1, ..)` era o que o segurava, e tirá-lo **não sangra** — enquanto
/// `lo < hi`, `floor(lo/h) < ceil(hi/h)` por aritmética, então nenhuma banda de uma
/// faixa REAL colapsa. O gate segue certo sobre o produto (64 bandas em 20–200 Hz
/// leem todas), só não é o piso que o faz passar; quem o exercita é o irmão
/// [`the_degenerate_range_still_reads_a_bin`].
#[test]
fn a_band_narrower_than_a_bin_still_reads_one() {
    let s = BandSpec {
        count: 64,
        min_hz: 20.0,
        max_hz: 200.0,
        scale: Scale::Log,
        floor_db: -60.0,
        gain: 1.0,
        weighting: Weighting::None,
        smoothing: 0.0,
    };
    // Compartimentos LARGOS (46,9 Hz é o que uma janela de 1024 a 48 kHz dá), então
    // 64 bandas entre 20 e 200 Hz são todas mais estreitas que um deles.
    let db = vec![0.0f32; 16];
    let mut out = Vec::new();
    fold(&db, 46.875, &s, &mut out);
    assert_eq!(out.len(), 64);
    assert!(
        out.iter().all(|v| *v > 0.99),
        "nenhuma banda pode devolver zero por ser estreita: {out:?}"
    );
}

/// ⚠️ **O caso em que o piso MORDE, e ele é alcançável digitando:** com
/// `min_hz == max_hz` numa fronteira exata de compartimento, `floor == ceil` e a
/// fatia seria VAZIA — a banda devolveria silêncio com o número certo na tela.
/// A mutação que troca o `clamp(b0 + 1, ..)` por `min(..)` sangra **só aqui**.
#[test]
fn the_degenerate_range_still_reads_a_bin() {
    let hz = 46.875f32; // uma janela de 1024 a 48 kHz
    let s = BandSpec {
        count: 1,
        // 93,75 Hz = 2 compartimentos EXATOS.
        min_hz: hz * 2.0,
        max_hz: hz * 2.0,
        scale: Scale::Linear,
        weighting: Weighting::None,
        floor_db: -60.0,
        gain: 1.0,
        smoothing: 0.0,
    };
    let db = vec![0.0f32; 16];
    let mut out = Vec::new();
    fold(&db, hz, &s, &mut out);
    assert_eq!(out.len(), 1);
    assert!(
        out[0] > 0.99,
        "uma faixa degenerada ainda le UM compartimento: {out:?}"
    );
}

/// **A ponderação é aplicada no CENTRO GEOMÉTRICO, e isto passa pelo `fold`.**
///
/// ⚠️ O irmão acima mede a `a_weight_db` DIRETO, então a mutação que troca
/// `(lo*hi).sqrt()` por `(lo+hi)/2` **dentro do fold** sobrevivia a ele — a função
/// estava certa e ninguém afirmava por onde o `fold` a chamava.
#[test]
fn the_fold_weights_at_the_geometric_centre_of_the_band() {
    let base = BandSpec {
        count: 1,
        min_hz: 100.0,
        max_hz: 1000.0,
        scale: Scale::Linear,
        weighting: Weighting::A,
        floor_db: -60.0,
        gain: 1.0,
        smoothing: 0.0,
    };
    // Um campo CHATO: o pico da banda é conhecido, então o que sobra na saída é
    // exactamente a ponderação.
    let db = vec![-20.0f32; 32];
    let mut out = Vec::new();
    fold(&db, 100.0, &base, &mut out);

    let expect = |centre: f32| {
        ((-20.0 + a_weight_db(centre) - base.floor_db) / -base.floor_db).clamp(0.0, 1.0)
    };
    let geo = expect((100.0f32 * 1000.0).sqrt());
    let arith = expect((100.0 + 1000.0) * 0.5);
    assert!(
        (out[0] - geo).abs() < 1e-5,
        "pesa no centro geometrico: {} vs {geo}",
        out[0]
    );
    assert!(
        (geo - arith).abs() > 0.05,
        "e os dois centros sao distinguiveis: {geo} vs {arith}"
    );
}

/// A normalização: `floor_db` vira `0`, escala cheia vira `gain`, e nada sai fora.
#[test]
fn the_floor_maps_to_zero_and_full_scale_to_the_gain() {
    let s = BandSpec {
        count: 1,
        min_hz: 0.0,
        max_hz: 1000.0,
        scale: Scale::Linear,
        floor_db: -60.0,
        gain: 2.0,
        weighting: Weighting::None,
        smoothing: 0.0,
    };
    let mut out = Vec::new();
    for (db, want) in [(-60.0f32, 0.0f32), (-30.0, 1.0), (0.0, 2.0), (-90.0, 0.0)] {
        fold(&[db], 1000.0, &s, &mut out);
        assert!(
            (out[0] - want).abs() < 1e-3,
            "{db} dB → {} (esperado {want})",
            out[0]
        );
    }
}

/// A ponderação A é ~0 dB em 1 kHz (é como ela é definida), tira dos graves e é
/// aplicada no CENTRO GEOMÉTRICO da banda — o meio de uma banda logarítmica.
#[test]
fn the_a_weighting_is_flat_at_a_kilohertz_and_cuts_the_bass() {
    assert!(a_weight_db(1000.0).abs() < 0.1, "{}", a_weight_db(1000.0));
    assert!(a_weight_db(50.0) < -25.0, "corta os graves");
    assert!(
        a_weight_db(100.0) > a_weight_db(50.0),
        "monotônica nos graves"
    );
    // Uma banda de 100..1000 pesa pelo centro GEOMÉTRICO (316 Hz), não pelo
    // aritmético (550 Hz) — os dois diferem por mais de 3 dB, então a mutação
    // que troca `(lo*hi).sqrt()` por `(lo+hi)*0.5` é visível.
    let geo = a_weight_db((100.0f32 * 1000.0).sqrt());
    let arith = a_weight_db((100.0 + 1000.0) * 0.5);
    assert!((geo - arith).abs() > 3.0, "geo {geo} contra arit {arith}");
}

/// **O suavizado é ataque imediato e queda amortecida**, e `0` devolve a matriz AO
/// BIT — o mundo sem o controle, que é o que torna o default seguro de mover.
#[test]
fn the_smoothing_holds_the_fall_and_zero_is_byte_identical() {
    let base = vec![1.0f32, 0.0, 0.0, 0.0];
    let mut off = base.clone();
    smooth_over_columns(&mut off, 1, 0.0);
    assert_eq!(off, base, "smoothing 0 é o mundo de antes, ao bit");

    let mut on = base.clone();
    smooth_over_columns(&mut on, 1, 0.8);
    assert_eq!(on[0], 1.0, "a primeira coluna não tem passado");
    assert!(
        on[1] > 0.7 && on[1] < 0.9,
        "a queda é amortecida: {}",
        on[1]
    );
    assert!(on[2] < on[1], "e continua caindo");

    // O ATAQUE não é amortecido: uma batida chega inteira no quadro em que chega.
    let mut attack = vec![0.0f32, 1.0];
    smooth_over_columns(&mut attack, 1, 0.9);
    assert_eq!(attack[1], 1.0, "o transiente não é suavizado");
}

/// O teto de bandas **diz de que recurso ele é** (§0): acima de `bins` bandas duas
/// vizinhas leem o mesmo compartimento e o controle deixa de controlar.
#[test]
fn the_band_ceiling_is_the_bin_count_of_the_analysis() {
    assert_eq!(MAX_BANDS, 1024 / 2 + 1, "a janela do ph2d-audio-spectral");
    let s = BandSpec::from_params(|n| {
        if n == param::COUNT {
            100_000.0
        } else {
            defaults(n)
        }
    });
    assert_eq!(s.count, MAX_BANDS, "a contagem é capada no teto medido");
}

/// O nó registra, é desenhado inteiro (a 4ª lei do doc 88), e o **arquivo** tem
/// row — sem ela o caminho seria autorável só editando um arquivo salvo à mão.
#[test]
fn every_param_is_painted_including_the_file() {
    let mut reg = NodeRegistry::new();
    register(&mut reg).expect("registra");
    let hints = reg.param_ui(MANIFEST.id).expect("tem hints");
    for p in MANIFEST.params {
        assert!(
            hints.iter().any(|h| h.param == p.name),
            "o param `{}` tem de ser desenhado",
            p.name
        );
    }
    // ⚠️⚠️ **Era `ParamWidget::Text` e ficou vermelho quando o BOTÃO nasceu** (2026-08-29):
    // um caminho de ficheiro autorado por digitação é um campo em que o artista tem de saber
    // escrever um caminho absoluto de cor. O widget é hoje o `File`, que abre o diálogo.
    //
    // ⭐ E a régua afirma o **KIND**, não a lista de extensões: a cerca deste nó é ESTRUTURAL
    // (ele não depende de crate de áudio nenhuma), então ele **não pode saber** o que este
    // build decodifica. Quem sabe é a shell, que resolve o kind no diálogo.
    assert!(
        hints.iter().any(|h| h.param == FILE_KEY
            && matches!(
                h.widget,
                ParamWidget::File {
                    kind: ph2d_node_registry::FileKind::Audio
                }
            )),
        "`{FILE_KEY}` tem de ser um BOTAO de ficheiro que declara o kind `Audio`"
    );
}

/// ⚠️ **`Temporal`, e não `Pure`.** O nó não lê o relógio com as próprias mãos, mas
/// o canal externo que ele lê é reescrito a cada quadro; declarado `Pure`, o cook
/// estaria autorizado a memoizá-lo e **as barras congelariam com tudo verde**.
#[test]
fn the_node_tells_the_cook_that_its_value_moves() {
    assert_eq!(MANIFEST.effect, Effect::Temporal);
}
