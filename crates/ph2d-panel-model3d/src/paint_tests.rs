//! Os gates da pintura — o que se **lê** numa linha.

use super::{STEPS_ACROSS_THE_RANGE, decimals_for_step};

/// Os **cursos** que este painel serve, de ponta a ponta. ⚠️ É a tabela que justifica a regra
/// existir: eles vão de `2e-4` a `360`, seis ordens de grandeza, e uma constante única não serve as
/// duas pontas ao mesmo tempo.
///
/// ⚠️ **O curso, e não o passo** — porque o passo é uma fração dele e os valores da linha são da
/// ordem do curso. Sondar um passo de `2e-6` a partir de `45,0` não mediria a regra: mediria o ULP
/// do `f32` em 45, que é `3,8e-6` e engoliria o passo antes de a formatação chegar a ver.
const REAL_COURSES: [(&str, f32); 4] = [
    ("ângulo", 360.0),
    ("posição ou largura num quadro de 2,4", 2.4),
    ("filete de uma peça de 0,12", 0.12),
    // ⚠️ Uma peça pequena é real: nada impede digitar `0,0002` num raio, e aí a parede do filete —
    // e portanto o curso da linha dele — é desse tamanho. É esta linha que uma constante reprova.
    ("filete de uma peça de 2e-4", 2.0e-4),
];

/// ⭐ **Dois passos vizinhos do arrasto LEEM diferente.**
///
/// ⚠️ É a lei inteira da regra, e é o que uma constante não consegue cumprir para todos os cursos ao
/// mesmo tempo: com três casas fixas, dois passos do filete de uma peça pequena leem **o mesmo
/// número** — o artista arrasta, a peça muda e a tela não.
///
/// ⚠️ As sondas são **relativas ao curso**, e é o que torna o gate honesto na ponta fina: um valor
/// grande com um passo minúsculo não é um caso deste painel, é um caso que o `f32` já não representa.
#[test]
fn two_neighbouring_drag_steps_never_read_the_same() {
    for (what, course) in REAL_COURSES {
        let step = course / STEPS_ACROSS_THE_RANGE;
        let d = decimals_for_step(step);
        for start in [0.0f32, course * 0.5, -course * 0.5, course, -course] {
            let a = format!("{:.d$}", f64::from(start), d = d);
            let b = format!("{:.d$}", f64::from(start + step), d = d);
            assert_ne!(
                a, b,
                "{what}: em {start} dois passos leem «{a}» com {d} casas — a tela não acompanha o \
                 arrasto"
            );
        }
    }
}

/// ⭐ **O que é digitado ENTRE dois passos aparece** — é a casa extra, e ela tem um motivo.
///
/// ⚠️ Sem ela, escrever `45,5` num ângulo mostraria `46`: o documento guardaria um número e o painel
/// mostraria outro. *Um painel que arredonda o que lhe foi escrito mente sobre o documento* — e a
/// mentira é indistinguível de o campo não ter aceitado o valor.
#[test]
fn a_value_typed_between_two_steps_is_not_rounded_away() {
    for (what, course) in REAL_COURSES {
        let step = course / STEPS_ACROSS_THE_RANGE;
        let d = decimals_for_step(step);
        let base = course * 0.25;
        let between = base + step * 0.5;
        assert_ne!(
            format!("{:.d$}", f64::from(base), d = d),
            format!("{:.d$}", f64::from(between), d = d),
            "{what}: meio passo desaparece na tela com {d} casas"
        );
    }
}

/// **Um passo degenerado não produz um `panic` nem um formato absurdo.**
///
/// ⚠️ Uma faixa de curso zero é possível (um nó cujo teto colapsou), e `log10(0)` é `−inf`: sem a
/// guarda, o `as usize` de um infinito é o tipo de coisa que só aparece na peça de alguém.
#[test]
fn a_degenerate_step_still_gives_a_usable_number_of_decimals() {
    for bad in [0.0f32, -1.0, f32::NAN, f32::INFINITY] {
        let d = decimals_for_step(bad);
        assert!((1..=6).contains(&d), "passo {bad} deu {d} casas");
    }
    // ⚠️ E o teto morde: abaixo de 1e-6 o `f32` de uma coordenada de ordem 1 já não distingue dois
    // valores (ULP = 1,19e-7), então mais casas escreveriam ruído do tipo — ver `decimals_for_step`.
    assert_eq!(decimals_for_step(1e-12), 6);
}

/// ⭐⭐⭐ **O ID DE UMA AMOSTRA É DERIVADO NUM SÍTIO SÓ** (report do Enio, 2026-09-19).
///
/// # ⛔⛔ Porque este censo existe
///
/// O defeito que ele guarda **não** foi um braço em falta: foi a mesma pergunta — *«qual é o id
/// desta amostra?»* — respondida em **dois** sítios. O [`crate::paint_rows::paint_swatch`] derivava
/// o id com um `match` e a lista que fecha um selector órfão ([`crate::paint::close_a_stranded_picker`])
/// derivava-o com **outro**, que só conhecia `Param::Material`. As duas respostas já divergiam para
/// a **luz** desde a wave dela, e a camada de estilo foi o dia em que a divergência chegou ao dedo
/// do artista: cinco cores com um id só.
///
/// ⚠️ **É um censo DERIVADO, não uma lista escrita à mão:** ele lê os ficheiros de produção e conta
/// as chamadas fora de comentário. Um terceiro leitor a cunhar o próprio id **reprova aqui**, com o
/// endereço — e a cura dele é chamar a porta.
///
/// **Mutação que deve sangrar:** repor a derivação dentro do `paint_swatch` ou da lista.
#[test]
fn o_id_de_uma_amostra_e_derivado_num_sitio_so() {
    // ⚠️ **Os ficheiros de PRODUÇÃO deste assunto.** Os `tests/it/` e os `_tests.rs` ficam de fora
    // de propósito: um gate que constrói o id esperado para o comparar é um leitor legítimo — ele
    // afirma a porta em vez de a contornar.
    const FONTES: [(&str, &str); 3] = [
        ("paint_rows.rs", include_str!("paint_rows.rs")),
        ("paint.rs", include_str!("paint.rs")),
        ("populate.rs", include_str!("populate.rs")),
    ];
    let mut chamadas: Vec<(&str, usize)> = Vec::new();
    for (nome, src) in FONTES {
        for (n, linha) in src.lines().enumerate() {
            let corte = linha.trim_start();
            // ⚠️ Prosa não é chamada — e este ficheiro tem os dois nomes escritos por extenso na
            // explicação acima. Sem este corte o censo mediria a própria documentação.
            if corte.starts_with("//") {
                continue;
            }
            if corte.contains("model3d_color_swatch(") || corte.contains("model3d_style_swatch(") {
                chamadas.push((nome, n + 1));
            }
        }
    }
    // ⚠️ **Piso de população**: zero chamadas quer dizer que o censo deixou de achar o que mede
    // (um `rename`, um ficheiro que mudou de sítio) — e lê-se exactamente como «está tudo bem».
    assert!(
        !chamadas.is_empty(),
        "o censo não achou chamada nenhuma — ele deixou de medir o que diz medir"
    );
    let fora: Vec<_> = chamadas
        .iter()
        .filter(|(f, _)| *f != "paint_rows.rs")
        .collect();
    assert!(
        fora.is_empty(),
        "o id de uma amostra é cunhado fora da porta `swatch_id`: {fora:?}"
    );
    // ⚠️ **DUAS e não uma**: a porta tem um braço por família de nome (o das entidades e o da cena).
    // Uma terceira aqui é um `match` a nascer noutro sítio do mesmo ficheiro.
    assert_eq!(
        chamadas.len(),
        2,
        "esperava as duas cunhagens da porta e achei {chamadas:?}"
    );
}

/// ⭐⭐ **UMA FAMÍLIA SEM ID NÃO RECEBE UM ID INVENTADO** — a outra metade da porta.
///
/// ⛔ O braço final do [`crate::paint_rows::swatch_id`] responde `None`, e **era ele o defeito**:
/// escrito como `Some(…, 0)` ele dava a toda família nova o mesmo controlo, em silêncio. Sem este
/// gate ele é uma linha que a mutação não consegue matar — *um comentário com sintaxe de código*.
///
/// **Mutação que deve sangrar:** `_ => Some(crate::ids::model3d_color_swatch(row.entity, 0))`.
#[test]
fn uma_familia_sem_id_devolve_none() {
    let linha = |param| crate::state::ParamRow {
        entity: 7,
        param,
        key: "x",
        value: 0.0,
        lo: 0.0,
        bound: ph2d_field::Bound::Soft(1.0),
        inert: None,
        integral: false,
        choices: &[],
        section: None,
        swatch: Some([1, 2, 3]),
        subject: None,
    };
    // ⚠️ **CONTROLO POSITIVO primeiro**: sem ele, uma porta que devolvesse `None` a TUDO passaria
    // este gate — e apagaria as amostras que funcionam.
    for vivo in [
        ph2d_field::Param::Material(0),
        ph2d_field::Param::Light(0),
        ph2d_field::Param::Style(0),
    ] {
        assert!(
            crate::paint_rows::swatch_id(&linha(vivo)).is_some(),
            "{vivo:?}: uma família que TEM amostra ficou sem id"
        );
    }
    // ⚠️ Uma dimensão não é uma cor, e nenhuma fileira de dimensão publica `swatch` — o que este
    // gate prende é a RESPOSTA da porta ao que ela não conhece, não o que o painel publica.
    for alheio in [
        ph2d_field::Param::Scale,
        ph2d_field::Param::Dim(0),
        ph2d_field::Param::Pos(0),
    ] {
        assert_eq!(
            crate::paint_rows::swatch_id(&linha(alheio)),
            None,
            "{alheio:?}: recebeu um id de amostra inventado"
        );
    }
}
