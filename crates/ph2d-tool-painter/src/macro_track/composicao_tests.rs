//! ⭐ **W5e — a cena compõe UMA vez por quadro** (`docs/Macros/02_plano_W5.md` §7): os `Preview` da
//! reprodução só drenam o estado ([`PainterTool::drena_sem_compor`]) e o retângulo sujo acumula até a
//! cena ler; a leitura compõe só esse retângulo sobre a composição anterior. Verdade: a composição
//! COMPLETA do mesmo estado, em cada quadro, na tela com PAPEL (a sprite branca do produto).

use super::passo::Passo;
use super::roteiro::grava_no_papel;
use super::tocar::TocadorDoPainter;
use ph2d_macro::MacroTime;

#[test]
fn a_composicao_de_uma_vez_por_quadro_e_a_completa_ao_bit() {
    let (bytes, d) = grava_no_papel(96, 0, 3).expect("arma no Wet Paint");
    let doc = ph2d_macro::codec::read(&bytes).expect("lê-se");
    let tr = &doc.tracks[0];
    let novo = || {
        TocadorDoPainter::novo(&tr.initial, tr.events.iter().map(|e| e.payload.as_slice()))
            .expect("toca")
    };
    let (mut cena, mut verdade) = (novo(), novo());
    let (mut i, mut q, mut lidas, mut parciais) = (0, 0u32, 0, 0);
    while f64::from(q) / 60.0 <= d + 0.05 {
        let to = MacroTime::from_secs_f64(f64::from(q) / 60.0);
        while i < tr.events.len() && tr.events[i].t <= to {
            let c = &tr.events[i].payload;
            cena.aplica(c).expect("a cena toca");
            verdade.aplica(c).expect("a verdade toca");
            i += 1;
        }
        if let Some((a, _, _)) = cena.toma_a_tela() {
            parciais += usize::from(cena.caixa_da_tela().is_some());
            let t = verdade.tool_mut();
            t.invalidate_composite();
            let (v, _, _) = t.composto_da_cena().expect("compõe");
            let diferentes = a.chunks(4).zip(v.chunks(4)).filter(|(x, y)| x != y).count();
            assert_eq!(diferentes, 0, "quadro {q}");
            lidas += 1;
        }
        q += 1;
    }
    assert!(lidas > 60, "a cena leu {lidas} telas");
    assert!(
        parciais > lidas / 2,
        "controlo: a cena compôs parcial em {parciais} de {lidas} leituras"
    );
    assert!(
        tr.events
            .iter()
            .any(|e| Passo::descodifica(&e.payload).is_ok_and(|p| p == Passo::Preview)),
        "a gravação drenou pela CPU"
    );
}

/// O arrasto da Timeline (W9, 18/10): a cena ainda não leu uma CÓPIA de um tocador — a caixa suja dela
/// é a do original, não o que difere do que a cena tem —, então a 1.ª caixa dela é a tela inteira (a da
/// volta atrás e a da etapa editada); a leitura seguinte volta à caixa.
#[test]
fn a_copia_de_um_tocador_sobe_inteira_na_primeira_leitura() {
    use super::roteiro::grava_no_papel_nos_meios;
    use super::tocar::respostas_de;
    use crate::PaintMedia;
    let (bytes, _) = grava_no_papel_nos_meios(96, 0, &[PaintMedia::Digital; 2]).expect("arma");
    let doc = ph2d_macro::codec::read(&bytes).expect("lê-se");
    let ev = &doc.tracks[0].events;
    let cargas = || ev.iter().map(|e| e.payload.as_slice());
    let mut original = TocadorDoPainter::novo(&doc.tracks[0].initial, cargas()).expect("toca");
    let le = |t: &mut TocadorDoPainter| t.toma_a_tela().map(|_| t.caixa_da_tela());
    assert_eq!(
        le(&mut original),
        Some(None),
        "um tocador novo sobe inteiro"
    );
    let respostas = respostas_de(cargas()).expect("lê-se").into();
    // As cópias do instante em que o original, lido, sobe só uma caixa (com a caixa suja dele por ler).
    let mut achado = None;
    for (k, e) in ev.iter().enumerate() {
        original.aplica(&e.payload).expect("toca");
        let copias = [
            original.instantaneo().expect("copia-se"),
            original.recomeca(&respostas).expect("recomeça"),
        ];
        if matches!(le(&mut original), Some(Some(_))) {
            achado = Some((k + 1, copias));
            break;
        }
    }
    let (fim, mut copias) = achado.expect("controlo: o original sobe só uma caixa");
    for c in &mut copias {
        assert_eq!(
            le(c),
            Some(None),
            "a 1.ª leitura de uma cópia é a tela inteira"
        );
        let mut caixa = None;
        for e in &ev[fim..] {
            c.aplica(&e.payload).expect("toca");
            if let Some(cx) = le(c) {
                caixa = Some(cx);
                break;
            }
        }
        assert!(
            matches!(caixa, Some(Some(_))),
            "depois volta à caixa: {caixa:?}"
        );
    }
}
