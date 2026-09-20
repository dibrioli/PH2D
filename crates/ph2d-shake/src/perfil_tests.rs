//! Os gates dos [`super::Perfil`] — ver o cabeçalho do módulo.

use super::{Numeros, Perfil};

/// A taxa do ecrã a que os quadros da tabela do cabeçalho são contados.
const HZ_DO_ECRA: f32 = 60.0;

/// ⭐⭐⭐ **A ESCADA é a lei.** Os números são de produto e podem ser afinados; a ORDEM não.
///
/// ⚠️ O expoente é **não-decrescente** e os outros dois **estritamente** crescentes, e essa
/// diferença é deliberada: dois perfis podem partilhar a forma da cauda (e partilham), mas dois
/// que tivessem o mesmo tamanho ou a mesma duração seriam o mesmo perfil com dois nomes.
#[test]
fn a_escada_e_a_lei() {
    let escada: Vec<Numeros> = Perfil::ALL.iter().map(|p| p.numeros()).collect();
    for par in escada.windows(2) {
        let (a, b) = (par[0], par[1]);
        assert!(
            b.amplitude > a.amplitude,
            "a amplitude tem de subir na escada: {a:?} -> {b:?}"
        );
        assert!(
            b.decaimento < a.decaimento,
            "o decaimento tem de DESCER (a duração sobe): {a:?} -> {b:?}"
        );
        assert!(
            b.expoente >= a.expoente,
            "o expoente nunca desce na escada: {a:?} -> {b:?}"
        );
    }
    // ⚠️ **O controlo:** sem esta linha a escada podia ser feita de três cópias do mesmo perfil
    // com nomes diferentes, e as três desigualdades acima seriam vácuo sobre uma lista de um.
    assert_eq!(escada.len(), 3, "a escada tem três degraus");
}

/// ⚠️ **A frequência NÃO muda entre perfis**, e o motivo é de recurso (ver o cabeçalho): o tecto é
/// a taxa de amostragem do ecrã, que é do monitor e não do acontecimento.
#[test]
fn a_frequencia_nao_muda_entre_perfis() {
    let hz: Vec<f32> = Perfil::ALL.iter().map(|p| p.numeros().frequencia).collect();
    assert!(
        hz.windows(2).all(|w| w[0].to_bits() == w[1].to_bits()),
        "os três perfis têm de ler a MESMA frequência, ao bit: {hz:?}"
    );
    // E ela é a de fábrica — ⚠️ escrita aqui como literal de propósito: a `ph2d-shake` declara ZERO
    // dependências e não alcança o `CameraShake::default`. O gate que ata as duas pontas vive na
    // `ph2d-app-components` (`o_perfil_do_meio_e_a_fabrica_ao_bit`).
    assert!(
        (hz[0] - 20.0).abs() < 1e-6,
        "a frequência de fábrica é 20 Hz"
    );
}

/// ⭐ **A duração é EXACTA**, e não assintótica: o [`crate::decai`] é linear no trauma.
///
/// ⚠️ **As duas metades:** um instante ANTES ainda há trauma, e no instante ele é ZERO. Sem a
/// primeira, um `decaimento` dez vezes maior passaria — *um gate que só vê o fim não mede a
/// duração, mede que ela acabou alguma vez*.
#[test]
fn a_duracao_de_um_perfil_e_exacta() {
    for p in Perfil::ALL {
        let d = p.numeros().decaimento;
        let dur = p.duracao_s();
        assert!(
            crate::decai(1.0, d, dur * 0.99) > 0.0,
            "{p:?}: a 99 % da duração ainda tem de haver trauma"
        );
        assert_eq!(
            crate::decai(1.0, d, dur),
            0.0,
            "{p:?}: à duração declarada o trauma é EXACTAMENTE zero"
        );
    }
}

/// ⭐ **O coice ainda é um TREMOR e não um solavanco.** A cerca é a mesma do `Flash` do
/// `ph2d_tween::Preset`: abaixo de ~4 quadros um acontecimento curto lê-se como artefacto de
/// desenho — e aqui há ainda a segunda metade, que é ele completar mais do que uma oscilação.
#[test]
fn o_recuo_ainda_e_um_tremor() {
    let p = Perfil::Recuo;
    let dur = p.duracao_s();
    let quadros = dur * HZ_DO_ECRA;
    let oscilacoes = dur * p.numeros().frequencia;
    assert!(quadros >= 4.0, "o coice dura {quadros:.1} quadros a 60 Hz");
    assert!(
        oscilacoes >= 2.0,
        "o coice completa {oscilacoes:.1} oscilações — abaixo de duas é um solavanco"
    );
}

/// A ida e a volta da tag, **nos dois sentidos** — sem a volta, uma lista com um duplicado fecharia
/// a primeira metade e um chip mandaria o valor do irmão.
#[test]
fn a_ida_e_a_volta_da_tag_fecham() {
    for (i, p) in Perfil::ALL.iter().enumerate() {
        assert_eq!(usize::from(p.tag()), i, "{p:?} declara a posição dele");
        assert_eq!(Perfil::from_tag(p.tag()), *p, "e a volta devolve-o");
    }
    let mut rotulos: Vec<&str> = Perfil::ALL.iter().map(|p| p.label_key()).collect();
    rotulos.sort_unstable();
    let antes = rotulos.len();
    rotulos.dedup();
    assert_eq!(antes, rotulos.len(), "duas chaves iguais são um chip morto");
    // Fora da faixa cai no de fábrica, e o de fábrica é o do MEIO.
    assert_eq!(Perfil::from_tag(99), Perfil::Impacto);
}

/// ⭐⭐ **A tabela do cabeçalho, medida** — ver o §0.0: uma faixa de produto declara o que COMPRA.
///
/// ⚠️ Ela é impressa **e afirmada**: uma tabela só impressa é a impressora que a `W8` do render
/// pagou (ninguém lê uma tabela que passa).
#[test]
fn mede_o_que_cada_perfil_compra() {
    let esperado = [
        // (perfil, pico m, dura s, quadros, oscilações)
        (Perfil::Recuo, 0.05, 0.125, 7.5, 2.5),
        (Perfil::Impacto, 0.25, 0.500, 30.0, 10.0),
        (Perfil::Explosao, 1.00, 1.250, 75.0, 25.0),
    ];
    println!("perfil       pico(m)  dura(s)  quadros  oscilações");
    for (p, pico, dura, quadros, osc) in esperado {
        let n = p.numeros();
        let d = p.duracao_s();
        println!(
            "{:<12} {:>7.3} {:>8.3} {:>8.1} {:>11.1}",
            format!("{p:?}"),
            n.amplitude,
            d,
            d * HZ_DO_ECRA,
            d * n.frequencia
        );
        assert!((n.amplitude - pico).abs() < 1e-6, "{p:?}: pico");
        assert!((d - dura).abs() < 1e-3, "{p:?}: duração (leu {d})");
        assert!((d * HZ_DO_ECRA - quadros).abs() < 0.1, "{p:?}: quadros");
        assert!((d * n.frequencia - osc).abs() < 0.05, "{p:?}: oscilações");
    }
}

/// ⚠️ **Os quatro números são TODOS honrados pela lei** — um perfil que escrevesse um campo que a
/// [`crate::deslocamento`] ignora seria um knob morto com cara de preset.
///
/// A régua é o PRODUTO: dois perfis consecutivos têm de dar deslocamentos diferentes no MESMO
/// instante, com a mesma semente.
#[test]
fn dois_perfis_dao_abanoes_diferentes() {
    let lei = |p: Perfil| {
        let n = p.numeros();
        crate::Lei {
            amplitude: n.amplitude,
            frequencia: n.frequencia,
            decaimento: n.decaimento,
            expoente: n.expoente,
            semente: 0x5EED,
        }
    };
    for par in Perfil::ALL.windows(2) {
        let a = crate::deslocamento(&lei(par[0]), 0.5, 0.37);
        let b = crate::deslocamento(&lei(par[1]), 0.5, 0.37);
        assert!(
            (a[0] - b[0]).abs() > 1e-4 || (a[1] - b[1]).abs() > 1e-4,
            "{:?} e {:?} entregam o mesmo abanão: {a:?} / {b:?}",
            par[0],
            par[1]
        );
    }
}
