//! Os gates da composição do brilho ([`crate::soma_halo`]).

use crate::Presentation;

/// ⭐ Um brilho LIGADO com os params que o chamador pedir — a porta das fixturas deste ficheiro.
fn ligado(p: ph2d_bloom::BloomParams) -> ph2d_bloom::Bloom {
    ph2d_bloom::Bloom {
        enabled: true,
        params: p,
    }
}

fn com(bloom: ph2d_bloom::Bloom) -> Presentation {
    Presentation {
        bloom,
        ..Presentation::of(ph2d_view_transform::Look::default())
    }
}

/// ⭐⭐⭐ **O OLHAR MANDA O PRETO EM PRETO** — a propriedade de que o [`super::soma_halo`] depende,
/// gateada **onde ela vive** e não onde é consumida.
///
/// ⛔ Ela nasceu de uma prova de mutação: aquele passe subtraía `look([0,0,0])` do halo *«porque o
/// olhar tem um desvio para o preto»*, e a mutação que apagava a subtracção **sobreviveu**. Medido,
/// `look([0,0,0])` é `[0,0,0]` ao bit — a subtracção era morta e saiu.
///
/// ⚠️ **A régua varre o espaço INTEIRO do olhar, não o de omissão:** as duas transformações × uma
/// escada de exposições que inclui os extremos e o que não é número. *Um gate escrito só sobre o
/// `Look::default()` afirmaria sobre uma célula de uma tabela e leria-se como afirmando sobre ela
/// toda* — e é a tabela toda que este passe assume.
#[test]
fn o_olhar_manda_o_preto_em_preto() {
    use ph2d_view_transform::{Look, ViewTransform};
    let mut celulas = 0usize;
    for view in ViewTransform::ALL {
        for stops in [
            -32.0,
            -8.0,
            -1.0,
            0.0,
            1.0,
            8.0,
            32.0,
            f32::NAN,
            f32::INFINITY,
        ] {
            let z = Look {
                exposure_stops: stops,
                view,
            }
            .apply([0.0; 3]);
            assert_eq!(
                z.map(f32::to_bits),
                [0u32; 3],
                "look({view:?}, {stops} stops) levou o preto a {z:?} — o passe do brilho \
                 ([`super::soma_halo`]) soma `look(halo)` CRU e conta com isto"
            );
            celulas += 1;
        }
    }
    assert!(
        celulas >= 18,
        "piso de população: a varredura leu só {celulas} células do olhar"
    );
}

/// ⭐⭐ **O INTERRUPTOR DO QUADRO É A LEI DA CRATE, e não uma segunda resposta** — ida-e-volta sobre
/// um corpus que tem os dois lados.
///
/// ⛔ Ele nasceu de uma mutação SOBREVIVENTE: cravar `Presentation::blooms()` em `true` não muda um
/// byte, porque o [`ph2d_bloom::halo`] tem a **própria** guarda e devolve um halo mudo ⇒ *as duas
/// guardas são redundantes na IMAGEM e não no RELÓGIO*, e o que a de fora compra — o buffer de cena
/// não nascer — é um custo, que um gate de bytes não vê.
///
/// ⚠️ *A cura não é medir o relógio* (seria mais um membro da família de flakes sob fan-out): é
/// medir que a porta **delega**, que é a afirmação que o doc dela faz.
#[test]
fn o_interruptor_do_quadro_e_a_lei_da_crate() {
    let corpus = [
        ("omissão", ph2d_bloom::Bloom::default()),
        ("ligado", ligado(ph2d_bloom::BloomParams::default())),
        (
            "ligado e mudo",
            ligado(ph2d_bloom::BloomParams {
                intensity: 0.0,
                ..ph2d_bloom::BloomParams::default()
            }),
        ),
        (
            "ligado e sem raio",
            ligado(ph2d_bloom::BloomParams {
                radius: 0.0,
                ..ph2d_bloom::BloomParams::default()
            }),
        ),
    ];
    let (mut sim, mut nao) = (0usize, 0usize);
    for (rot, b) in corpus {
        let esperado = b.contributes();
        assert_eq!(
            com(b).blooms(),
            esperado,
            "{rot}: a porta do quadro discordou da lei da crate"
        );
        if esperado { sim += 1 } else { nao += 1 }
    }
    // ⚠️ Sem os DOIS lados, cravar a porta numa constante passava: um corpus só de «não contribui»
    // aprova um `false` cravado, e um só de «contribui» aprova um `true`.
    assert!(
        sim >= 1 && nao >= 2,
        "o corpus tem de conter os dois lados — leu {sim} a contribuir e {nao} a não contribuir"
    );
}

/// ⭐⭐ **O ALFA ATRAVESSA A LEI SEM PERDER UM BYTE** — a propriedade que substituiu uma guarda.
///
/// ⛔ A 1.ª redacção da cobertura tinha um `if c > 0.0` à volta dela, com o doc a dizer que sem ele
/// *«a ida e volta pelo `f32` moveria bytes no caminho de omissão»*. Medido: ela **não move** — os
/// `256` valores voltam ao mesmo byte. ⇒ a guarda era provadamente morta, e o que ela alegava
/// proteger mora aqui, onde uma mudança de codificação (outro arredondamento, meia precisão)
/// reprova em voz alta em vez de aparecer como deriva de um byte numa imagem.
#[test]
fn o_alfa_atravessa_a_lei_sem_perder_um_byte() {
    for x in 0u8..=255 {
        let a = f32::from(x) / 255.0;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let volta = (0.0f32.mul_add(-a, 0.0 + a) * 255.0).ceil() as u8;
        assert_eq!(
            volta, x,
            "o alfa {x} voltou como {volta} com cobertura zero"
        );
    }
}
