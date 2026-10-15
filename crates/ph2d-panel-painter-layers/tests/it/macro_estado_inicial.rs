//! ⭐ **O estado inicial de uma macro leva CADA controlo do painel, nos QUATRO meios** (W6c).
//!
//! A população é a do censo dos controlos ([`super::censo_dos_controlos::gestos`]): cada controlo que a
//! tela pinta em cada meio, conduzido pela porta do ponteiro ANTES de armar a gravação. Depois grava-se
//! os dois traços da bancada e toca-se a macro num tool novo, que só sabe do controlo pelo estado
//! inicial. Verde = a mesma tela ao byte, controlo a controlo. Um controlo que o estado inicial não
//! leve aparece aqui com o nome.
//!
//! Controlo da régua: o mesmo gesto feito DEPOIS de armar (então o tool novo não o vê no inicial, mas
//! recebe-o como passo do painel) dá a mesma tela — e sem ele a tela muda (o censo dos controlos já
//! prova que cada controlo vivo muda o traço).

use super::censo_dos_controlos::{
    Bancada, Gesto, diferenca, em_paralelo, gestos, nome, nomes, risca,
};
use ph2d_editor_core::tool::RasterEditTool;
use ph2d_tool_painter::PaintMedia;
use ph2d_tool_painter::macro_track::tocar::TocadorDoPainter;

/// `(gesto, texels diferentes)` — ou `Err` com o porquê de não se medir (a gravação recusou-se).
/// `durante` = o gesto entra DEPOIS de armar (vira um passo do painel), senão antes (vai no inicial).
fn ensaio_em(meio: PaintMedia, g: &Gesto, durante: bool) -> Result<usize, String> {
    let mut b = Bancada::nova(meio, &[]);
    let mut lixo = super::censo_dos_controlos::Entrega::default();
    if !durante {
        b.aplica(g, &mut lixo);
    }
    let (inicial, _) = b
        .tool
        .macro_grava()
        .map_err(|r| format!("a gravação não armou: {r:?}"))?;
    if durante {
        b.aplica(g, &mut lixo);
    }
    let mut cargas = Vec::new();
    let _ = risca(&mut b.tool);
    cargas.extend(b.tool.macro_para());
    let original = b.tool.run_full().0;
    let mut toc = TocadorDoPainter::novo(&inicial, cargas.iter().map(Vec::as_slice))
        .map_err(|e| format!("o inicial não se aplica: {e:?}"))?;
    for c in &cargas {
        toc.aplica(c)
            .map_err(|e| format!("a reprodução parou: {e:?}"))?;
    }
    let tocado = toc.tool_mut().run_full().0;
    Ok(diferenca(&original, &tocado).0)
}

/// Os quatro meios do seletor.
const MEIOS: [PaintMedia; 4] = [
    PaintMedia::Digital,
    PaintMedia::Watercolor,
    PaintMedia::Impasto,
    PaintMedia::WetPaint,
];

/// Cada controlo de cada meio pelo `ensaio`; as falhas com o meio e o nome do controlo.
fn falhas_em(
    minimo: usize,
    ensaio: impl Fn(PaintMedia, &Gesto) -> Result<usize, String> + Sync,
) -> (usize, Vec<String>) {
    let nomes = nomes();
    let (mut total, mut falhas) = (0, Vec::new());
    for meio in MEIOS {
        let todos = gestos(meio, &[]);
        assert!(
            todos.len() > minimo,
            "o painel de {meio:?} pintou só {} controlos: a bancada partiu-se",
            todos.len()
        );
        total += todos.len();
        let resultados = em_paralelo(&todos, |g| ensaio(meio, g));
        for (g, r) in todos.iter().zip(resultados) {
            let quem = g
                .id()
                .map_or_else(|| format!("{g:?}"), |id| nome(&nomes, id));
            match r {
                Ok(0) => {}
                Ok(n) => falhas.push(format!("{meio:?} · {quem}: {n} texels")),
                Err(e) => falhas.push(format!("{meio:?} · {quem}: {e}")),
            }
        }
    }
    (total, falhas)
}

/// ⭐ Cada controlo do Wet Paint mexido DURANTE a gravação reproduz-se: o passo do painel chega ao tool
/// novo uma vez, e só uma — uma porta gravada que chama outra por dentro não grava as duas (um botão de
/// alternar aplicado duas vezes desfaz-se).
#[test]
fn cada_controlo_mexido_a_gravar_reproduz_se() {
    let (total, falhas) = falhas_em(10, |m, g| ensaio_em(m, g, true));
    assert!(
        falhas.is_empty(),
        "{} de {total} controlos mexidos A GRAVAR não se reproduzem:\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}

#[test]
fn o_estado_inicial_leva_cada_controlo_de_cada_meio() {
    for meio in MEIOS {
        let base = ensaio_em(meio, &Gesto::Nenhum, false).expect("a base mede-se");
        assert_eq!(
            base, 0,
            "{meio:?}: sem gesto nenhum a reprodução já difere ({base} texels)"
        );
    }
    let (total, falhas) = falhas_em(10, |m, g| ensaio_em(m, g, false));
    assert!(
        falhas.is_empty(),
        "{} de {total} controlos NÃO chegam à reprodução pelo estado inicial:\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}

/// O controlo da régua: um estado inicial capturado SEM o gesto, tocado com os passos de uma bancada que
/// o teve, muda a tela em algum controlo do Wet Paint — o gate vê a falta que afirma não haver.
#[test]
fn um_estado_inicial_velho_muda_a_reproducao() {
    let mut velha = Bancada::nova(PaintMedia::WetPaint, &[]);
    let (inicial_velho, _) = velha.tool.macro_grava().expect("arma");
    let _ = velha.tool.macro_para();
    let todos = gestos(PaintMedia::WetPaint, &[]);
    let alguma = todos.iter().take(60).any(|g| {
        let mut b = Bancada::nova(PaintMedia::WetPaint, &[]);
        let mut lixo = super::censo_dos_controlos::Entrega::default();
        b.aplica(g, &mut lixo);
        if b.tool.macro_grava().is_err() {
            return false;
        }
        let _ = risca(&mut b.tool);
        let cargas = b.tool.macro_para();
        let original = b.tool.run_full().0;
        let Ok(mut toc) = TocadorDoPainter::novo(&inicial_velho, cargas.iter().map(Vec::as_slice))
        else {
            return false;
        };
        if cargas.iter().any(|c| toc.aplica(c).is_err()) {
            return false;
        }
        diferenca(&original, &toc.tool_mut().run_full().0).0 > 0
    });
    assert!(
        alguma,
        "nenhum dos 60 primeiros controlos muda a reprodução quando o inicial é o de ANTES do gesto: o gate \
         não distingue um estado inicial que leva o controlo de um que o perde"
    );
}
