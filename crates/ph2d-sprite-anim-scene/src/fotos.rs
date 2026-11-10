//! Os passos das fotos da cena 5 que pedem o rato real no editor de animações (Align, By feet, Pivot, Follow) e as
//! do desfazer: irmão de [`super`] pelo tecto de linhas.

use super::*;

/// ⭐ A prévia Align (GEF 28 B) com o rato real: a `run` parada no 4.º quadro, o botão Align, e (nas fotos do gesto)
/// a pega do canto puxada para o Pivot até `2/3`; depois um Ctrl+Z. Cada etapa relata as folhas.
pub(super) fn previa_align(q: u32, foto: Option<Foto>) -> Vec<Passo> {
    use Passo as P;
    if let Some(f) = foto.filter(|f| matches!(f, Foto::AlinharPes | Foto::AlinharPesDesfaz)) {
        return pes(q, f);
    }
    if let Some(f) = foto.filter(|f| {
        matches!(
            f,
            Foto::PivoAntes
                | Foto::PivoPes
                | Foto::PivoDesfaz
                | Foto::PivoMao
                | Foto::PivoSegue
                | Foto::PivoSegueDesfaz
                | Foto::PivoSegueEscalaDesfaz
        )
    }) {
        return pivo_no_editor(q, f);
    }
    let Some(f) =
        foto.filter(|f| matches!(f, Foto::Alinhar | Foto::AlinharEscala | Foto::AlinharDesfaz))
    else {
        return Vec::new();
    };
    let gesto = f != Foto::Alinhar;
    match q {
        20 => vec![
            P::Anim(DUAS, Ed::SetCurrent("run".into())),
            P::Anim(DUAS, Ed::Playing(false)),
            P::Anim(DUAS, Ed::SetStep(3)),
        ],
        24 | 25 => vec![P::Alinha(DUAS, (q - 24) as u8)],
        30 => vec![P::Relata("align antes".into(), DUAS)],
        32..=37 if gesto => vec![P::Alinha(DUAS, (q - 30) as u8)],
        40 if gesto => vec![P::Relata("align escalada".into(), DUAS)],
        50 if f == Foto::AlinharDesfaz => vec![P::Desfaz],
        60 if f == Foto::AlinharDesfaz => vec![P::Relata("align 1 Ctrl+Z".into(), DUAS)],
        _ => Vec::new(),
    }
}

/// ⭐ «Misaligned Sheets» (GEF 28 D): a `run` (visível desde o quadro 9) anda e dois quadros entortam; nas fotos, o
/// Align abre na «attack» e (na acertada) a `run` reduz a 2/3 e o «By feet» é clicado pelo rato real.
pub(super) fn desalinhadas(q: u32, foto: Option<Foto>) -> Vec<Passo> {
    use Passo as P;
    let acerta = foto == Some(Foto::DesalinhadasAcertadas);
    match q {
        10 => vec![P::Verbo(DESALINHADAS, Verbo::MoveSheet(DESLOCA_A_RUN))],
        11 => TORTOS
            .iter()
            .map(|&(cell, by)| P::Verbo(DESALINHADAS, Verbo::NudgeCell { sheet: 1, cell, by }))
            .collect(),
        _ if !matches!(foto, Some(Foto::Desalinhadas | Foto::DesalinhadasAcertadas)) => Vec::new(),
        20 => vec![
            P::Anim(DESALINHADAS, Ed::SetCurrent("attack".into())),
            P::Anim(DESALINHADAS, Ed::Playing(false)),
            P::Anim(DESALINHADAS, Ed::SetStep(1)),
        ],
        24 | 25 => vec![P::Alinha(DESALINHADAS, (q - 24) as u8)],
        30 => vec![P::Relata("desalinhadas".into(), DESALINHADAS)],
        32 if acerta => vec![P::Verbo(DESALINHADAS, Verbo::ScaleSheet(ESCALA_DA_RUN))],
        34 | 35 if acerta => vec![P::Alinha(DESALINHADAS, (q - 26) as u8)],
        40 if acerta => vec![P::Relata("acertadas".into(), DESALINHADAS)],
        _ => Vec::new(),
    }
}

/// ⭐ «By feet» (GEF 28 C) com o rato real na «attack», parada num quadro da `run`: o botão Align, o «By feet», e
/// (na foto do desfazer) um Ctrl+Z. Cada etapa relata os acertos.
pub(super) fn pes(q: u32, f: Foto) -> Vec<Passo> {
    use Passo as P;
    match q {
        20 => vec![
            P::Anim(DUAS, Ed::SetCurrent("attack".into())),
            P::Anim(DUAS, Ed::Playing(false)),
            P::Anim(DUAS, Ed::SetStep(1)),
        ],
        24 | 25 => vec![P::Alinha(DUAS, (q - 24) as u8)],
        30 => vec![P::Relata("pés antes".into(), DUAS)],
        32 | 33 => vec![P::Alinha(DUAS, (q - 24) as u8)],
        40 => vec![P::Relata("pés alinhados".into(), DUAS)],
        50 if f == Foto::AlinharPesDesfaz => vec![P::Desfaz],
        60 if f == Foto::AlinharPesDesfaz => vec![P::Relata("pés 1 Ctrl+Z".into(), DUAS)],
        _ => Vec::new(),
    }
}

/// ⭐ O Pivot no editor (GEF 29) com o rato real: o editor abre com o Align ligado (o campo Pivot do Inspector), na
/// «attack»; o ponto «Bottom» da grade, ou o «By hand» e a cruz arrastada; e (na foto do desfazer) um Ctrl+Z.
pub(super) fn pivo_no_editor(q: u32, f: Foto) -> Vec<Passo> {
    use Passo as P;
    let so_escala = f == Foto::PivoSegueEscalaDesfaz;
    let segue = so_escala || matches!(f, Foto::PivoSegue | Foto::PivoSegueDesfaz);
    let pes = segue || matches!(f, Foto::PivoPes | Foto::PivoDesfaz);
    let desfaz = so_escala || matches!(f, Foto::PivoDesfaz | Foto::PivoSegueDesfaz);
    match q {
        20 => vec![
            P::Anim(DUAS, Ed::SetCurrent("attack".into())),
            P::Anim(DUAS, Ed::Playing(false)),
            P::Anim(DUAS, Ed::SetStep(1)),
        ],
        30 => vec![P::Relata("pivô antes".into(), DUAS)],
        32 | 33 if pes => vec![P::Alinha(DUAS, (q - 22) as u8)],
        32..=37 if f == Foto::PivoMao => vec![P::Alinha(DUAS, (q - 20) as u8)],
        // O Follow: ligado (18/19), a quina (2..=7), relato, a folha arrastada (20..=22) em quadros distintos.
        34 if segue => vec![P::Relata("pivô no ponto".into(), DUAS)],
        35 | 36 if segue => vec![P::Alinha(DUAS, (q - 17) as u8)],
        38..=43 if segue => vec![P::Alinha(DUAS, (q - 36) as u8)],
        44 if segue => vec![P::Relata("pivô depois da escala".into(), DUAS)],
        45..=47 if segue && !so_escala => vec![P::Alinha(DUAS, (q - 25) as u8)],
        49 if segue && !so_escala => vec![P::Relata("pivô seguiu a folha".into(), DUAS)],
        40 if !segue => vec![P::Relata("pivô depois".into(), DUAS)],
        50 if desfaz => vec![P::Desfaz],
        60 if desfaz => vec![P::Relata("pivô 1 Ctrl+Z".into(), DUAS)],
        _ => Vec::new(),
    }
}

/// A folha à vista depois do Ctrl+Z (`FolhasDesfaz`) e a tira rolada ao fim (`TiraRolada`), com o editor aberto.
pub(super) fn folhas_desfaz(q: u32, foto: Option<Foto>) -> Vec<Passo> {
    use Passo as P;
    match (foto, q) {
        (Some(Foto::FolhasDesfaz), 20) => vec![
            P::Relata("folha antes".into(), DUAS),
            P::Entrada,
            P::Verbo(DUAS, Verbo::ScaleSheet(ESCALA_DA_RUN)),
        ],
        (Some(Foto::FolhasDesfaz), 21..=23) => vec![P::Entrada],
        (Some(Foto::FolhasDesfaz), 30) => vec![P::Relata("folha escalada".into(), DUAS), P::Desfaz],
        (Some(Foto::FolhasDesfaz), 40) => vec![P::Relata("folha 1 Ctrl+Z".into(), DUAS)],
        (Some(Foto::TiraRolada), 20) => vec![
            P::Anim(DUAS, Ed::SetCurrent("run".into())),
            P::Anim(DUAS, Ed::Playing(false)),
            P::Anim(DUAS, Ed::SetStep(11)),
        ],
        _ => Vec::new(),
    }
}
