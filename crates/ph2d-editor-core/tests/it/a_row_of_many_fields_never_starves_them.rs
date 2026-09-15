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
fn cabem_por_linha(control_w: f32, n: usize, lead: f32) -> usize {
    let mut melhor = 1;
    for k in 1..=n {
        let preciso = k as f32 * (lead + NUMBER_INPUT_MIN_W_PX) + (k as f32 - 1.0) * gap();
        if preciso <= control_w {
            melhor = k;
        }
    }
    melhor
}

/// ⭐⭐ **Os dois regimes de decoração que o app de facto pinta.**
///
/// ⛔ Uma varredura só com `0` deixaria o `lead` **inteiramente por medir** — e foi o `lead` que a
/// ordem do dono de 2026-09-15 (*«a mesma formatação … para todo o Transform»*) trouxe: as linhas do
/// Transform levam uma letra de eixo colorida antes de cada caixa, e ela é parte da CÉLULA.
fn leads() -> [f32; 2] {
    [
        0.0,
        ph2d_tokens::Spacing::Lg.px() + ph2d_tokens::Spacing::Xxs.px(),
    ]
}

#[test]
fn a_row_of_many_fields_never_starves_them() {
    let mut mau = Vec::new();
    for &interior in INTERIORES {
        let row = property_row_columns(0.0, interior, 0.0, 22.0);
        for (n, lead) in (1..=4usize).flat_map(|n| leads().map(|l| (n, l))) {
            let (por_linha, linhas, cw) = property_fields_layout(row.control.w, n, gap(), lead);
            let esperado = cabem_por_linha(row.control.w, n, lead);
            // (a) o número por linha é o MÁXIMO que cabe ao piso — nem menos (desperdiça altura),
            //     nem mais (esfomeia a caixa).
            if por_linha != esperado {
                mau.push(format!(
                    "interior {interior}: n={n} lead={lead} deu {por_linha} por linha, o maximo ao piso e' {esperado}"
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
            //     ⚠️ **A régua é a CAIXA (`célula − lead`), nunca a célula** — a letra do eixo não
            //     é sítio para pôr o número, e medir a célula deixaria o `lead` a mascarar o piso.
            let caixa = cw - lead;
            if row.control.w >= lead + NUMBER_INPUT_MIN_W_PX && caixa < NUMBER_INPUT_MIN_W_PX - 0.01
            {
                mau.push(format!(
                    "interior {interior}: n={n} lead={lead} pinta a CAIXA a {caixa:.2}, abaixo do \
                     piso {NUMBER_INPUT_MIN_W_PX} que o dono declarou"
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
            let (por_linha, linhas, _) = property_fields_layout(row.control.w, n, gap(), 0.0);
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
        let (por_linha, linhas, _) = property_fields_layout(largo.control.w, n, gap(), 0.0);
        assert_eq!(
            (por_linha, linhas),
            (n, 1),
            "n={n}: a 700 de interior a linha devia caber inteira"
        );
    }
}

/// ⭐⭐⭐ **A CAIXA SOZINHA DE UMA LINHA COBRE EXACTAMENTE O QUE O PAR COBRE.**
///
/// ⛔⛔ **Feedback do dono, 2026-05-24:** *«a caixa única de Rotation deve se alinhar à caixa de X à
/// esquerda e à direita»*. Até 2026-09-15 isso era uma **fórmula escrita à mão** no pintor do
/// Transform (`2 × two_chip_w + col_gap + axis_col_w + tag_box_gap`) — e o comentário dela registava
/// que ela **já tinha estado errada**: faltava-lhe o `axis_col_w + tag_box_gap`, e a *Rotation*
/// acabava aquém da borda direita do `Y`.
///
/// ⭐ Hoje não há fórmula: com `n = 1` a célula **é** a coluna do controlo. Este gate prova que isso
/// entrega a propriedade em ambos os regimes — com o par lado a lado **e** com ele empilhado — sobre
/// o curso real do dock e com o `lead` real das letras de eixo.
///
/// ⚠️ **É uma propriedade obtida por CONSTRUÇÃO, e mesmo assim tem gate:** *uma propriedade que
/// ninguém mede é uma propriedade que a refactoração seguinte pode perder em silêncio* — foi
/// exactamente o que aconteceu à fórmula que ela substitui.
#[test]
fn the_lone_field_of_a_row_spans_what_the_pair_spans() {
    // O `lead` do Transform: a coluna da letra de eixo mais o vão dela.
    let lead = ph2d_tokens::Spacing::Lg.px() + ph2d_tokens::Spacing::Xxs.px();
    for &interior in INTERIORES {
        let row = property_row_columns(0.0, interior, 0.0, 22.0);
        let (_, _, celula_1) = property_fields_layout(row.control.w, 1, gap(), lead);
        let (por_linha, _, celula_2) = property_fields_layout(row.control.w, 2, gap(), lead);

        // A caixa sozinha e a caixa do `X` começam no MESMO `x`.
        let sozinha_esq = row.control.x + lead;
        let x_esq = row.control.x + lead;
        assert!(
            (sozinha_esq - x_esq).abs() < 0.01,
            "interior {interior}: a caixa sozinha comeca em {sozinha_esq}, o X em {x_esq}"
        );

        // ⛔⛔ **E as duas acabam na BORDA DIREITA DA COLUNA DO CONTROLO — não uma na outra.**
        //
        // ⚠️⚠️ A 1.ª redacção comparava `sozinha_dir` com `par_dir`, os dois derivados da MESMA
        // porta, e a prova de mutação **SOBREVIVEU**: descontar um vão a mais na largura da célula
        // encolhe os dois lados ao mesmo tempo e a igualdade continua a valer. *Um gate que compara
        // duas construções é cego a uma mutação partilhada* (`CLAUDE.md`, e este repo já o pagou na
        // `property_label_col_w`). ⇒ o oráculo é a `property_row_columns`, que é outra porta: a
        // linha tem de **encher** a coluna do controlo, nos dois regimes.
        let borda = row.control.x + row.control.w;
        let sozinha_dir = sozinha_esq + (celula_1 - lead);
        let ultima = (por_linha - 1) as f32;
        let par_dir = row.control.x + (celula_2 + gap()) * ultima + lead + (celula_2 - lead);
        for (quem, dir) in [("sozinha", sozinha_dir), ("o par", par_dir)] {
            assert!(
                (dir - borda).abs() < 0.01,
                "interior {interior} ({por_linha} por linha): {quem} acaba em {dir:.2}, \
                 a coluna do controlo acaba em {borda:.2}"
            );
        }
    }
}

/// ⭐⭐⭐ **UMA LINHA NUNCA QUEBRA ENQUANTO A COLUNA DO NOME TEM FOLGA.**
///
/// ⛔⛔ **Report do dono, 2026-09-15, com foto do Transform:** *«O painel ainda largo com espaço à
/// esquerda e as linhas já se quebram (caixa y passa para baixo. Isso não pode acontecer. Encontre
/// a solução»*. Na foto o nome `Position X / Y` media `~88 px` numa coluna de `~154`, com `~66 px`
/// de vazio à esquerda dele — e o `Y` descia porque ao controlo faltavam **`7 px`**.
///
/// ⇒ a metade deixou de ser um piso e passou a ser um **alvo**: a coluna do nome **cede** ao
/// controlo até ao que o nome de facto precisa.
///
/// # A régua
///
/// Para cada largura do curso do dock e cada nome plausível, se a linha QUEBRA então tem de ser
/// verdade que **nem com o nome no mínimo** o controlo caberia. ⛔ Uma quebra com folga à esquerda é
/// exactamente o que o dono proibiu.
#[test]
fn a_row_never_wraps_while_the_name_column_has_slack() {
    let piso = NUMBER_INPUT_MIN_W_PX;
    let mut mau = Vec::new();
    for &interior in INTERIORES {
        for quer in [40.0_f32, 88.0, 150.0] {
            for n in 2..=4usize {
                let precisa = n as f32 * piso + (n as f32 - 1.0) * gap();
                let row = ph2d_editor_core::widget::property_row_columns_for(
                    0.0,
                    interior,
                    0.0,
                    22.0,
                    Some(quer),
                    Some(precisa),
                );
                let (por_linha, _, _) = property_fields_layout(row.control.w, n, gap(), 0.0);
                if por_linha == n {
                    continue; // não quebrou — nada a provar
                }
                // Quebrou. Só é legítimo se nem com o nome no mínimo o controlo coubesse.
                let usable = interior - ph2d_editor_core::widget::DECORATOR_W;
                let vao = ph2d_tokens::Spacing::Md.px();
                let com_o_nome_no_minimo = usable - vao - quer;
                if com_o_nome_no_minimo >= precisa {
                    mau.push(format!(
                        "interior {interior}, nome {quer}, n={n}: QUEBROU com folga — o nome ocupa \
                         {:.2} e bastavam {quer:.2}, o que deixaria {com_o_nome_no_minimo:.2} \
                         para um controlo que precisa de {precisa:.2}",
                        row.label.w
                    ));
                }
                // ⚠️ E a coluna do nome NUNCA desce abaixo do que o nome precisa — senão a cura
                //    trocaria uma linha quebrada por um nome cortado.
                if row.label.w < quer - 0.01 && row.label.w < interior * 0.5 {
                    mau.push(format!(
                        "interior {interior}, nome {quer}, n={n}: a coluna desceu a {:.2}, abaixo \
                         do que o nome precisa",
                        row.label.w
                    ));
                }
            }
        }
    }
    assert!(
        mau.is_empty(),
        "{} celula(s) quebram uma linha com espaco por usar a' esquerda:\n  {}",
        mau.len(),
        mau.join("\n  ")
    );
}
