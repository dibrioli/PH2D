//! Auditoria por foto 11/11 (janela 1280×720): (A) o caminho «script › line N» da barra da aba não
//! passa por cima da engrenagem — o que não cabe é UMA linha com `…` (lei do dono); (B) as listas que
//! rolam («In this script» e SCRIPTS) mostram a barra de rolagem, como as outras da aba, e só quando
//! o conteúdo transborda.

use ph2d_code_buffer::Buffer;
use ph2d_code_viewmodel::{Entry, Who};
use ph2d_editor_core::zones::Rect;
use ph2d_panel_code::bancada::BancadaView;
use ph2d_panel_code::tab_bar::{Crumb, TabBar, height};
use ph2d_panel_code::{ListHit, ListRow, ScriptList, ScriptListView};
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

const ELIPSE: char = '\u{2026}';

fn barra(w: f32, func: bool, ts: &mut TextSystem) -> TabBar<u8> {
    let mut bar = TabBar::<u8>::default();
    let crumb = Crumb {
        script: "a_very_long_script_name_for_the_crumb.luau".into(),
        func: func.then(|| ("update_everything_now".into(), 19, 0)),
        line: 20,
    };
    let titles = [(0u8, "File"), (1u8, "Edit"), (2u8, "View")];
    bar.paint(
        Rect::new(0.0, 0.0, w, height()),
        &titles,
        &crumb,
        &mut VectorScene::new(),
        ts,
        &Theme::default(),
    );
    bar
}

/// Larga: o caminho inteiro, sem `…`, à esquerda da engrenagem.
#[test]
fn o_caminho_largo_sai_inteiro() {
    let mut ts = TextSystem::without_system_fonts();
    let bar = barra(2000.0, true, &mut ts);
    let gear = bar.painted.gear.expect("engrenagem");
    let (r, partes) = bar.painted.crumb.clone().expect("caminho pintado");
    assert_eq!(partes.len(), 3, "{partes:?}");
    assert!(partes.iter().all(|p| !p.contains(ELIPSE)), "{partes:?}");
    assert!(r.x + r.w <= gear.x, "caminho {r:?} × engrenagem {gear:?}");
}

/// Estreita: o caminho acaba antes da engrenagem, encurta o script com `…`, guarda «line 20», e a
/// caixa clicável da função é a que se pintou (dentro do caminho).
#[test]
fn o_caminho_estreito_corta_em_reticencias_antes_da_engrenagem() {
    let mut ts = TextSystem::without_system_fonts();
    for w in [380.0, 420.0, 480.0, 560.0] {
        let bar = barra(w, true, &mut ts);
        let gear = bar.painted.gear.expect("engrenagem");
        let (r, partes) = bar.painted.crumb.clone().expect("caminho pintado");
        let titulos = bar.painted.titles.last().map_or(0.0, |t| t.1.x + t.1.w);
        assert!(
            r.x + r.w <= gear.x,
            "w={w}: caminho {r:?} × engrenagem {gear:?}"
        );
        assert!(r.x >= titulos, "w={w}: caminho {r:?} por cima dos menus");
        assert!(
            partes.iter().any(|p| p.contains(ELIPSE)),
            "w={w}: {partes:?}"
        );
        if partes.len() == 3 {
            assert!(partes[2].contains("20"), "w={w}: {partes:?}");
            let (f, l, _) = bar.painted.crumb_fn.expect("função clicável");
            assert_eq!(l, 19);
            assert!(
                f.x >= r.x && f.x + f.w <= r.x + r.w,
                "w={w}: {f:?} fora de {r:?}"
            );
        }
    }
    // Tão estreita que só cabe uma parte: o caminho inteiro numa linha que acaba em `…`.
    // Varre-se de largo a estreito: nunca por cima da engrenagem, e o caso de uma parte aparece.
    let mut uma = 0;
    for w in (120..=600).step_by(2) {
        let bar = barra(w as f32, w % 4 == 0, &mut ts);
        let gear = bar.painted.gear.expect("engrenagem");
        if let Some((r, partes)) = bar.painted.crumb.clone() {
            assert!(r.x + r.w <= gear.x, "w={w}: {r:?} × engrenagem {gear:?}");
            if partes.len() == 1 {
                uma += 1;
                assert!(partes[0].ends_with(ELIPSE), "w={w}: {partes:?}");
                assert!(bar.painted.crumb_fn.is_none(), "w={w}: função sem caixa");
            }
        }
    }
    assert!(
        uma > 0,
        "a varredura nunca chegou ao caminho de uma só parte"
    );
}

fn quem(n: usize) -> Who {
    Who {
        functions: (0..n).map(|i| Entry(format!("fn_{i}"), i * 3)).collect(),
        ..Who::default()
    }
}

/// O «In this script» com mais entradas do que cabem pinta a barra (e o `ln N` acaba antes dela);
/// com poucas, nenhuma.
#[test]
fn o_in_this_script_mostra_a_barra_quando_transborda() {
    let mut ts = TextSystem::without_system_fonts();
    let theme = Theme::default();
    let area = Rect::new(0.0, 0.0, 260.0, 160.0);
    let mut v = BancadaView::default();
    v.paint_outline(&quem(2), area, &mut VectorScene::new(), &mut ts, &theme);
    assert_eq!(v.outline_bar(), None, "duas entradas cabem");
    v.paint_outline(&quem(30), area, &mut VectorScene::new(), &mut ts, &theme);
    let bar = v.outline_bar().expect("30 entradas transbordam: a barra");
    let r = v
        .target(ph2d_panel_code::bancada::Aim::Entry(0))
        .expect("a 1.ª entrada é alvo");
    assert!(
        r.x + r.w <= bar.x,
        "a entrada {r:?} por baixo da barra {bar:?}"
    );
}

/// A lista de scripts com mais linhas do que cabem pinta a barra (a linha acaba antes dela); com
/// poucas, nenhuma.
#[test]
fn a_lista_de_scripts_mostra_a_barra_quando_transborda() {
    let mut ts = TextSystem::without_system_fonts();
    let filtro = Buffer::new("");
    let nomes: Vec<String> = (0..30).map(|i| format!("s{i}.luau")).collect();
    let linhas: Vec<ListRow> = nomes
        .iter()
        .enumerate()
        .map(|(i, n)| ListRow {
            id: i as u64,
            name: n,
            dirty: false,
            current: i == 0,
            users: 1,
        })
        .collect();
    let pinta = |l: &mut ScriptListView, rows: &[ListRow], ts: &mut TextSystem| {
        let lista = ScriptList {
            open: true,
            filter: &filtro,
            focus: None,
            rows,
            renaming: None,
            notice: None,
            hover_count: None,
        };
        let area = Rect::new(0.0, 0.0, 240.0, ScriptListView::height_for(4));
        l.paint(&lista, area, &mut VectorScene::new(), ts, &Theme::default());
    };
    let mut l = ScriptListView::default();
    pinta(&mut l, &linhas[..3], &mut ts);
    assert_eq!(l.painted_bar(), None, "três linhas cabem");
    pinta(&mut l, &linhas, &mut ts);
    let bar = l.painted_bar().expect("30 linhas transbordam: a barra");
    let lixo = l.target_rect(ListHit::Delete(0)).expect("o lixo da actual");
    assert!(
        lixo.x + lixo.w <= bar.x,
        "o lixo {lixo:?} por baixo da barra {bar:?}"
    );
}
