//! ⭐⭐⭐ **UMA LINHA DE VÁRIAS COMPONENTES NUNCA PINTA UM CAMPO ABAIXO DO PISO — ELA DESCE.**
//!
//! ⛔⛔ **Ela cobra DUAS ordens do dono que se contrariam numa row de N campos:**
//!
//! - *«Label acima do campo numérico! Muito ruim!»* (2026-09-14) põe o nome ao lado, e isso entrega
//!   ao controlo **metade** da linha;
//! - *«não permita que a caixa seja redimencionada para menor que isso»* (2026-05-24) põe um piso
//!   de [`ph2d_editor_core::widget::NUMBER_INPUT_MIN_W_PX`] em cada caixa.
//!
//! À largura de omissão do Inspector a coluna do controlo mede `128 px` e dois campos ao piso pedem
//! `148`. ⇒ o que não cabe **desce**, dentro da coluna do controlo — a mesma lei que a
//! `sections/rows::seg_row` já praticava para um segmentado (*«o controlo REFLUI e a coluna do
//! rótulo não»*).
//!
//! # ⚠️ Porque a régua mede a LEI e não o painel
//!
//! As rows de N campos do Inspector (âncoras · 9-slice · amostragem · visibilidade) só são pintadas
//! com uma sprite seleccionada que TENHA aquele componente, e o arnês de painel não a produz —
//! medido em 2026-09-15: das oito ids de várias componentes, **zero** são registadas pelo
//! `MockPanelHost`. *Uma régua que pedisse o rect registado mediria a ausência delas.*
//!
//! ⇒ Mede-se a porta pura, com o oráculo escrito de OUTRA forma (o `k` máximo por procura linear,
//! não pela fórmula), e o caminho até ela é o do produto: a largura da coluna do controlo sai da
//! [`ph2d_editor_core::widget::property_row_columns`], sobre a escada REAL do dock.

use ph2d_editor_core::widget::{
    NUMBER_INPUT_MIN_W_PX, property_fields_layout, property_row_columns,
};

/// ⭐ **O curso que o artista alcança** — `DOCK_W_MIN`..`DOCK_W_MAX`, menos o recuo do corpo do
/// Inspector (`BODY_PAD = 10` de cada lado). ⛔ Não é uma escada escolhida: a do meio (`273,3`) é a
/// largura a que o dono tem o dock dele.
const INTERIORES: &[f32] = &[200.0, 225.0, 253.3, 284.0, 380.0, 700.0];

/// O vão entre duas caixas da mesma linha — a porta do ritmo, não um literal.
fn gap() -> f32 {
    ph2d_tokens::control_gap_px()
}

/// **O oráculo, escrito de outra maneira:** o maior `k` que cabe ao piso, por PROCURA, sem a
/// fórmula que a porta usa.
fn cabem_por_linha(control_w: f32, n: usize) -> usize {
    let mut melhor = 1;
    for k in 1..=n {
        let preciso = k as f32 * NUMBER_INPUT_MIN_W_PX + (k as f32 - 1.0) * gap();
        if preciso <= control_w {
            melhor = k;
        }
    }
    melhor
}

#[test]
fn a_row_of_many_fields_never_starves_them() {
    let mut mau = Vec::new();
    for &interior in INTERIORES {
        let row = property_row_columns(0.0, interior, 0.0, 22.0);
        for n in 1..=4usize {
            let (por_linha, linhas, cw) = property_fields_layout(row.control.w, n, gap());
            let esperado = cabem_por_linha(row.control.w, n);
            // (a) o número por linha é o MÁXIMO que cabe ao piso — nem menos (desperdiça altura),
            //     nem mais (esfomeia a caixa).
            if por_linha != esperado {
                mau.push(format!(
                    "interior {interior}: n={n} deu {por_linha} por linha, o maximo ao piso e' {esperado}"
                ));
            }
            // (b) as linhas cobrem TODOS os campos.
            if por_linha * linhas < n {
                mau.push(format!(
                    "interior {interior}: n={n} deu {por_linha}x{linhas} — nao cobre {n} campos"
                ));
            }
            // (c) as caixas cabem na coluna do controlo.
            let ocupado = por_linha as f32 * cw + (por_linha as f32 - 1.0) * gap();
            if ocupado > row.control.w + 0.01 {
                mau.push(format!(
                    "interior {interior}: n={n} ocupa {ocupado:.2} numa coluna de {:.2}",
                    row.control.w
                ));
            }
            // (d) ⭐⭐ **A ORDEM DO DONO:** sempre que a coluna comporta uma caixa ao piso, a caixa
            //     pintada NÃO fica abaixo dele. ⛔ Quando nem uma cabe, o recurso que falta é o
            //     PAINEL, e aí a caixa recebe a coluna inteira (é o que a porta declara).
            if row.control.w >= NUMBER_INPUT_MIN_W_PX && cw < NUMBER_INPUT_MIN_W_PX - 0.01 {
                mau.push(format!(
                    "interior {interior}: n={n} pinta a caixa a {cw:.2}, abaixo do piso \
                     {NUMBER_INPUT_MIN_W_PX} que o dono declarou"
                ));
            }
        }
    }
    assert!(
        mau.is_empty(),
        "a linha de varias componentes desobedece em {} celula(s):\n  {}",
        mau.len(),
        mau.join("\n  ")
    );
}

/// ⚠️ **A metade MONOTÓNICA** — alargar o painel nunca pode fazer caber MENOS campos por linha nem
/// produzir MAIS linhas.
///
/// ⛔ Ela existe porque a metade de cima compara a porta com um oráculo da mesma família: as duas
/// poderiam estar erradas do mesmo modo. Esta não pergunta *quanto*, pergunta *em que sentido* — e
/// uma inversão de sinal na fórmula mata-a sem tocar na primeira.
#[test]
fn a_wider_panel_never_fits_fewer_fields() {
    for n in 2..=4usize {
        let mut anterior = (0usize, usize::MAX);
        for &interior in INTERIORES {
            let row = property_row_columns(0.0, interior, 0.0, 22.0);
            let (por_linha, linhas, _) = property_fields_layout(row.control.w, n, gap());
            assert!(
                por_linha >= anterior.0 && linhas <= anterior.1,
                "n={n}: a {interior} da' {por_linha}x{linhas}, mais estreito dava {}x{}",
                anterior.0,
                anterior.1
            );
            anterior = (por_linha, linhas);
        }
        // ⭐ E no topo do curso do dock as quatro componentes cabem TODAS numa linha — senão a lei
        //   estaria a empilhar onde há espaço de sobra.
        let largo = property_row_columns(0.0, 700.0, 0.0, 22.0);
        let (por_linha, linhas, _) = property_fields_layout(largo.control.w, n, gap());
        assert_eq!(
            (por_linha, linhas),
            (n, 1),
            "n={n}: a 700 de interior a linha devia caber inteira"
        );
    }
}
