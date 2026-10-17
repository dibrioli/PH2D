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
    Bancada, Gesto, diferenca, em_paralelo, gestos, gestos_da, nome, nomes, risca,
};
use ph2d_editor_core::tool::{
    CanvasPaintTool, CanvasPointer, PanelEvent, PointerPhase, RasterEditTool, Tool,
};
use ph2d_tool_painter::PaintMedia;
use ph2d_tool_painter::macro_track::tocar::TocadorDoPainter;

/// `(gesto, texels diferentes)` — ou `Err` com o porquê de não se medir (a gravação recusou-se).
/// `durante` = o gesto entra DEPOIS de armar (vira um passo do painel), senão antes (vai no inicial).
fn ensaio_em(meio: PaintMedia, g: &Gesto, durante: bool) -> Result<usize, String> {
    let mut b = Bancada::nova(meio, &[]);
    let mut lixo = super::censo_dos_controlos::Entrega::default();
    if durante {
        return grava_e_compara_com(b, |b| b.aplica(g, &mut lixo));
    }
    b.aplica(g, &mut lixo);
    grava_e_compara(b)
}

/// Arma JÁ, risca os dois traços, pára e toca num tool novo: os texels que diferem.
fn grava_e_compara(b: Bancada) -> Result<usize, String> {
    grava_e_compara_com(b, |_| {})
}

/// [`grava_e_compara`] com `depois_de_armar` feito a gravar, antes dos traços.
fn grava_e_compara_com(
    mut b: Bancada,
    depois_de_armar: impl FnOnce(&mut Bancada),
) -> Result<usize, String> {
    let (inicial, _) = b
        .tool
        .macro_grava()
        .map_err(|r| format!("a gravação não armou: {r:?}"))?;
    depois_de_armar(&mut b);
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

/// As ferramentas do TRILHO (W7): a preparação na tela é o que elas sabem antes de armar.
const TRILHO: [&str; 3] = ["clone", "mask", "selection"];

/// Um gesto na tela de `de` a `ate`, pela porta do ponteiro (um clique quando são o mesmo ponto).
fn na_tela(t: &mut ph2d_tool_painter::PainterTool, de: [f32; 2], ate: [f32; 2]) {
    let p = |pos, phase| CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    };
    t.on_canvas_pointer(p(de, PointerPhase::Down));
    for k in 1..=12 {
        let f = k as f32 / 12.0;
        let pos = [de[0] + (ate[0] - de[0]) * f, de[1] + (ate[1] - de[1]) * f];
        t.on_canvas_pointer(p(pos, PointerPhase::Move));
        let _ = t.take_preview_arc();
        t.on_tick(1000.0 / 60.0);
    }
    t.on_canvas_pointer(p(ate, PointerPhase::Up));
    let _ = t.take_preview_arc();
    t.on_tick(1000.0 / 60.0);
}

/// A bancada na ferramenta `modo`, com o que ela sabe preparado na tela: a fonte do Clone, uma
/// proteção da Máscara, um rectângulo da Seleção.
fn no_trilho(meio: PaintMedia, modo: &str) -> Bancada {
    use ph2d_tool_painter::ids;
    let mut b = Bancada::nova(meio, &[]);
    b.no_modo(modo);
    let t = &mut b.tool;
    match modo {
        "clone" => {
            t.handle_panel_event(PanelEvent::Click(ids::PAINTER_BRUSH_CLONE_SET_SOURCE));
            na_tela(t, [100.0, 100.0], [100.0, 100.0]);
            assert!(t.clone_has_source(), "{meio:?}: a fonte escolheu-se");
        }
        "mask" => {
            t.handle_panel_event(PanelEvent::Click(ids::PAINTER_MASK_BRUSH[0]));
            na_tela(t, [0.0, 70.0], [128.0, 80.0]);
            assert!(t.mask_scratch_active(), "{meio:?}: a máscara pintou-se");
        }
        _ => {
            t.handle_panel_event(PanelEvent::Click(ids::PAINTER_SEL_MODE_IDS[2]));
            na_tela(t, [10.0, 10.0], [70.0, 120.0]);
            assert!(t.selection_active(), "{meio:?}: a seleção existe");
        }
    }
    b.aplica(
        &Gesto::Nenhum,
        &mut super::censo_dos_controlos::Entrega::default(),
    );
    b
}

/// O controlo `g` da ferramenta `modo` mexido ANTES de armar, depois da preparação; os traços com a
/// ferramenta (o Clone) ou com o pincel (a Máscara e a Seleção restringem-no).
fn ensaio_no_trilho(meio: PaintMedia, modo: &str, g: &Gesto) -> Result<usize, String> {
    let mut b = no_trilho(meio, modo);
    b.aplica(g, &mut super::censo_dos_controlos::Entrega::default());
    if modo != "clone" {
        b.no_modo("brush");
    }
    grava_e_compara(b)
}

/// ⭐ **O estado inicial leva o que as ferramentas do trilho sabem, e cada controlo delas** (W7): em
/// cada meio, para Clone, Máscara e Seleção, a preparação na tela e cada controlo que o painel pinta
/// SÓ nessa ferramenta (os do pincel já estão no censo de cima), mexidos antes de armar.
#[test]
fn o_estado_inicial_leva_as_ferramentas_do_trilho() {
    let nomes = nomes();
    let (mut total, mut falhas) = (0, Vec::new());
    for meio in MEIOS {
        let do_pincel: Vec<_> = gestos(meio, &[]).iter().filter_map(Gesto::id).collect();
        for modo in TRILHO {
            let mut todos: Vec<Gesto> = gestos_da(no_trilho(meio, modo))
                .into_iter()
                .filter(|g| g.id().is_some_and(|id| !do_pincel.contains(&id)))
                .collect();
            assert!(
                !todos.is_empty(),
                "{meio:?} · {modo}: o painel não pintou controlo próprio nenhum — a bancada partiu-se"
            );
            todos.insert(0, Gesto::Nenhum);
            total += todos.len();
            let resultados = em_paralelo(&todos, |g| ensaio_no_trilho(meio, modo, g));
            for (g, r) in todos.iter().zip(resultados) {
                let quem = g
                    .id()
                    .map_or_else(|| "a preparação".to_owned(), |id| nome(&nomes, id));
                match r {
                    Ok(0) => {}
                    Ok(n) => falhas.push(format!("{meio:?} · {modo} · {quem}: {n} texels")),
                    Err(e) => falhas.push(format!("{meio:?} · {modo} · {quem}: {e}")),
                }
            }
        }
    }
    assert!(
        falhas.is_empty(),
        "{} de {total} ensaios das ferramentas do trilho NÃO chegam à reprodução pelo estado inicial:\n{}",
        falhas.len(),
        falhas.join("\n")
    );
}
