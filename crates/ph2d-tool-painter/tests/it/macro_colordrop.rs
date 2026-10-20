//! W11 (`docs/Macros/handoffs/PROMPT_W11_trocar_a_cor_nao_quebra_a_macro_2026-10-19.md`): o botão **C&F**
//! a gravar. As quatro portas do ColorDrop (`begin_colordrop_fill`, `set_fill_threshold`, `fill_commit`,
//! `restore_after_colordrop`) chegam da shell, e recusá-las parava a macro na 1.ª — as pinceladas
//! seguintes não tocavam (smoke do dono, 19/10).
//!
//! - O gesto que só abre o seletor de cor (a shell de antes entrava em Fill ao primeiro movimento, fora
//!   da tela, e saía ao soltar) deixa a trilha só com a `Cor`, e a macro toca até ao fim na cor nova.
//! - O ColorDrop de verdade (soltar a cor na tela = preenchimento; o limiar no modal ou no gesto parado)
//!   grava-se e toca ao byte — o preenchimento é determinístico dado o estado (`tool/paint/fill.rs`).

use super::macro_comum::{Gravador, cp, texels_diferentes, toca};
use ph2d_editor_core::tool::{CanvasPaintTool, PointerPhase, RasterEditTool, Tool};
use ph2d_macro::MacroDoc;
use ph2d_tool_painter::macro_track::Passo;
use ph2d_tool_painter::{PaintMedia, PainterTool};

const N: u32 = 64;
const PRETO: [u8; 3] = [20, 20, 20];
const NOVA: [u8; 3] = [200, 30, 30];

fn ferramenta() -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (N * N * 4) as usize], N, N);
    t.set_paint_media(PaintMedia::Digital);
    t.set_brush_size_px(4.0);
    t.set_brush_color_srgb8(PRETO);
    t
}

fn quadro(t: &mut PainterTool, g: &mut Gravador) {
    let _ = t.take_preview_arc();
    t.on_tick(1000.0 / 60.0);
    g.quadro(t);
}

/// Uma pincelada horizontal na linha `v` (fracção da tela), um evento por quadro.
fn pincelada(t: &mut PainterTool, g: &mut Gravador, v: f32) {
    let s = N as f32;
    const AMOSTRAS: usize = 12;
    for i in 0..=AMOSTRAS {
        let fase = match i {
            0 => PointerPhase::Down,
            AMOSTRAS => PointerPhase::Up,
            _ => PointerPhase::Move,
        };
        let u = 0.15 + 0.7 * i as f32 / AMOSTRAS as f32;
        t.on_canvas_pointer(cp([s * u, s * v], fase));
        quadro(t, g);
    }
}

fn passos(doc: &MacroDoc) -> Vec<Passo> {
    doc.tracks[0]
        .events
        .iter()
        .map(|e| Passo::descodifica(&e.payload).expect("passo legível"))
        .collect()
}

/// A trilha sem recusas, tocada até ao fim, igual ao byte à tela de quem gravou.
fn toca_igual(original: &mut PainterTool, doc: &MacroDoc) -> Vec<u8> {
    let recusas: Vec<Passo> = passos(doc)
        .into_iter()
        .filter(|p| matches!(p, Passo::Recusada(_)))
        .collect();
    assert!(
        recusas.is_empty(),
        "o C&F a gravar deixou recusas na trilha (a macro pára na 1.ª): {recusas:?}"
    );
    let mut toc = toca(doc).expect("a macro toca até ao fim");
    let (a, b) = (original.run_full().0, toc.tool_mut().run_full().0);
    assert_eq!(
        texels_diferentes(&a, &b),
        0,
        "a reprodução não é a gravação"
    );
    a
}

fn texel(tela: &[u8], u: f32, v: f32) -> [u8; 3] {
    let (x, y) = ((u * N as f32) as usize, (v * N as f32) as usize);
    let i = (y * N as usize + x) * 4;
    [tela[i], tela[i + 1], tela[i + 2]]
}

/// O caminho do dono: Gravar, uma pincelada, o C&F premido e arrastado um pouco SEM entrar na tela
/// (a shell de antes entrava em Fill a cada movimento e saía ao soltar), a cor escolhida no seletor,
/// outra pincelada, Parar — e ▷ toca as duas, a segunda na cor nova.
#[test]
fn trocar_a_cor_no_cef_a_gravar_toca_ate_ao_fim_na_cor_nova() {
    let mut t = ferramenta();
    let mut g = Gravador::arma(&mut t);
    pincelada(&mut t, &mut g, 0.3);
    let volta = t.active_paint_mode_id();
    for _ in 0..3 {
        t.begin_colordrop_fill(volta);
        quadro(&mut t, &mut g);
    }
    t.restore_after_colordrop();
    t.set_brush_color_srgb8(NOVA);
    quadro(&mut t, &mut g);
    pincelada(&mut t, &mut g, 0.6);
    let doc = g.para(&mut t);
    assert!(
        passos(&doc).contains(&Passo::Cor(NOVA)),
        "a cor do seletor não ficou na trilha"
    );
    let tela = toca_igual(&mut t, &doc);
    assert_eq!(
        texel(&tela, 0.5, 0.6),
        NOVA,
        "a 2.ª pincelada não é da cor nova"
    );
    assert_eq!(
        texel(&tela, 0.5, 0.3),
        PRETO,
        "a 1.ª pincelada mudou de cor"
    );
}

/// Como o limiar chega ao ColorDrop: pelo modal depois de soltar, ou pelo gesto parado (o «dwell» da
/// shell inunda no sítio, afina o limiar com o rato e o soltar só confirma — sem `Up` na tela).
#[derive(Clone, Copy, Debug)]
enum Limiar {
    Modal,
    Parado,
}

/// Grava: uma pincelada preta, o ColorDrop vermelho (o limiar por `limiar`), uma pincelada preta.
fn grava_o_colordrop(limiar: Limiar) -> (PainterTool, MacroDoc) {
    let mut t = ferramenta();
    let mut g = Gravador::arma(&mut t);
    pincelada(&mut t, &mut g, 0.3);
    t.set_brush_color_srgb8(NOVA);
    quadro(&mut t, &mut g);
    let volta = t.active_paint_mode_id();
    t.begin_colordrop_fill(volta);
    let s = N as f32;
    t.on_canvas_pointer(cp([s * 0.7, s * 0.7], PointerPhase::Down));
    quadro(&mut t, &mut g);
    t.on_canvas_pointer(cp([s * 0.8, s * 0.8], PointerPhase::Move));
    quadro(&mut t, &mut g);
    match limiar {
        Limiar::Modal => {
            t.on_canvas_pointer(cp([s * 0.8, s * 0.8], PointerPhase::Up));
            quadro(&mut t, &mut g);
            t.set_fill_threshold(0.4);
        }
        Limiar::Parado => t.set_fill_threshold(0.2),
    }
    quadro(&mut t, &mut g);
    t.fill_commit();
    quadro(&mut t, &mut g);
    t.set_brush_color_srgb8(PRETO);
    pincelada(&mut t, &mut g, 0.85);
    let doc = g.para(&mut t);
    (t, doc)
}

/// O painel MACROS: o ColorDrop é uma etapa «Fill» com a cor dele, entre as pinceladas — também o do
/// gesto parado, que não tem `Up` (sem ela as cores das pinceladas seguintes desalinhavam). A cor
/// editada chega ao preenchimento inteiro, o limiar afinado depois de soltar incluído; o tamanho não.
#[test]
fn o_colordrop_e_uma_etapa_fill_com_a_cor_dele() {
    use ph2d_tool_painter::macro_track::etapas::{TipoDaEtapa, edita_cor, edita_tamanho, etapas};
    const VERDE: [u8; 3] = [30, 160, 60];
    for limiar in [Limiar::Modal, Limiar::Parado] {
        let (_, mut doc) = grava_o_colordrop(limiar);
        let es = etapas(&doc.tracks[0]).expect("etapas");
        let vistas: Vec<(TipoDaEtapa, [u8; 3])> =
            es.iter().map(|e| (e.tipo.clone(), e.cor)).collect();
        assert_eq!(
            vistas,
            vec![
                (TipoDaEtapa::Pincelada(1), PRETO),
                (TipoDaEtapa::Preenchimento(1), NOVA),
                (TipoDaEtapa::Pincelada(2), PRETO),
            ],
            "{limiar:?}: as etapas do painel"
        );
        let k = es[1].k;
        assert!(
            edita_tamanho(&mut doc.tracks[0], k, 9.0).is_err(),
            "{limiar:?}: o tamanho do pincel não é do ColorDrop"
        );
        edita_cor(&mut doc.tracks[0], k, VERDE).expect("a cor do Fill edita-se");
        let tocavel = doc.tracks[0]
            .tocavel(&ph2d_tool_painter::macro_track::etapas::LeitorDoPainter)
            .expect("tocável");
        let editada = MacroDoc {
            tracks: vec![tocavel],
            ..doc.clone()
        };
        let mut toc = toca(&editada).expect("toca a editada");
        let tela = toc.tool_mut().run_full().0;
        assert_eq!(
            texel(&tela, 0.6, 0.6),
            VERDE,
            "{limiar:?}: o Fill não ficou verde"
        );
        assert_eq!(
            texel(&tela, 0.5, 0.85),
            PRETO,
            "{limiar:?}: a pincelada a seguir mudou"
        );
    }
}

/// O ColorDrop de verdade a gravar: a cor arrastada do C&F para a tela e solta inunda a área branca;
/// a macro toca-o igual ao byte e a pincelada seguinte também.
#[test]
fn o_colordrop_a_gravar_toca_igual() {
    for limiar in [Limiar::Modal, Limiar::Parado] {
        let (mut t, doc) = grava_o_colordrop(limiar);
        let tela = toca_igual(&mut t, &doc);
        assert_eq!(
            texel(&tela, 0.6, 0.6),
            NOVA,
            "{limiar:?}: o preenchimento não aconteceu"
        );
        assert_eq!(
            t.active_paint_mode_id(),
            "brush",
            "{limiar:?}: o ColorDrop não voltou ao pincel"
        );
    }
}
