//! **O ESTADO VIVO SERVE DE BASE PARA O DELTA?** — a pergunta que decide o S3 (doc 28 §7), medida.
//!
//! O `undo_delta` afirma que **não**: *"`restore_shape_overlay` RE-CARIMBA a figura, então o vivo depois
//! de um undo não é byte-a-byte o snapshot instalado"*. A frase decide se o `cursor` — hoje um segundo
//! dono PERMANENTE dos quatro planos canvas-shaped — pode largá-los, e com ele o fork do pen-down e os
//! três forks do fold.
//!
//! ⚠️ **Uma afirmação sobre o produto não se cita: mede-se.** A rede
//! ([`crate::undo_planes::PlaneDeltas::divergences`]) foi chamada no instante de TODO undo/redo da suíte
//! desta crate, em debug, com controle positivo (a 1ª rodada não imprimiu nada — não porque concordasse,
//! mas porque o `cargo test` **captura stderr** de teste que passa: o `--nocapture` é parte da medição).
//!
//! ```text
//!   81 chamadas de undo/redo   ·   79 concordam   ·   2 divergem, as duas no canvas_rgba
//! ```
//!
//! **A frase é verdadeira em 2 de 81, e as duas exceções têm NOME** — é isso que os três gates abaixo
//! pinam, para que o S3 saiba exatamente o que tem de fechar antes de largar o cursor:
//!
//! 1. **o re-stamp do shape** (`apply_and_keep_…`, no REDO) — a frase do doc, real: o
//!    `restore_shape_overlay` re-carimba o editor sobre a base pristina, e o vivo passa a ser
//!    *instalado + re-stamp*.
//! 2. **o escorrido do Wet Paint** (no UNDO) — a sim continua compositando depois do pen-up, sem gravar
//!    entrada. É a escrita estrangeira que o `absorb_foreign_writes` cura **no caminho de record**, e que
//!    no caminho de undo segue viva.
//!
//! ⚠️ **Nenhuma das duas é bug HOJE**, e o gate 1 diz por quê: a materialização constrói um snapshot
//! COMPLETO e o `restore_model` o instala por atacado (`self.canvas_rgba = m.canvas_rgba`), então o
//! resíduo é substituído em vez de sobreviver. Elas só viram corrupção **silenciosa** sob o S3, que
//! escreveria a janela DENTRO do plano vivo e deixaria o resto como está.

use crate::tool::PainterTool;
use crate::tool::paint::media::PaintMedia;
use crate::undo_planes::PlaneDeltas;
use ph2d_editor_core::tool::{CanvasPaintTool, CanvasPointer, PointerPhase, RasterEditTool};
use ph2d_painter_brush::{BrushSpec, Falloff};

fn cp(pos: [f32; 2], phase: PointerPhase) -> CanvasPointer {
    CanvasPointer {
        pos,
        pressure: 1.0,
        tilt: [0.0, 0.0],
        phase,
    }
}

/// Tela branca com um pincel opaco — o caso comum, sem meio nenhum armado.
fn tool(w: u32, h: u32) -> PainterTool {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (w * h * 4) as usize], w, h);
    let b = BrushSpec {
        radius_px: 10.0,
        hardness: 1.0,
        falloff: Falloff::Constant,
        color: [0.1, 0.2, 0.9],
        space_attenuation: false,
        ..Default::default()
    };
    t.paint.brush = b;
    for slot in &mut t.paint.brush_by_mode {
        *slot = b;
    }
    t
}

fn stroke(t: &mut PainterTool, y: f32) {
    t.on_canvas_pointer(cp([20.0, y], PointerPhase::Down));
    for k in 1..=5u8 {
        t.on_canvas_pointer(cp([20.0 + f32::from(k) * 8.0, y], PointerPhase::Move));
    }
    t.on_canvas_pointer(cp([60.0, y], PointerPhase::Up));
}

/// Onde o VIVO do tool difere do cursor, agora.
fn divergences(t: &PainterTool) -> Vec<&'static str> {
    let cursor = t
        .undo
        .cursor_for_audit()
        .expect("a historia tem de ter um cursor");
    PlaneDeltas::divergences(&t.snapshot_model(), cursor)
}

/// **A PREMISSA DO S3, no caso comum: depois de um traço commitado o plano vivo É o do cursor.**
///
/// Não *igual em conteúdo* — o **mesmo buffer**: o commit toma o cursor como `after.clone()`, que é um
/// clone de `Arc`. É essa identidade que torna a comparação barata e **imune à armadilha do ADR-0124**
/// (endereço igual com conteúdo diferente), porque escrever no lugar exige dono ÚNICO e a referência
/// forte do cursor o impede.
///
/// É o número que a rede mediu em 79 de 81 undos da suíte, e é sobre ele que o S3 se apoiaria.
#[test]
fn after_a_committed_stroke_the_live_planes_are_the_cursor() {
    let mut t = tool(140, 140);
    stroke(&mut t, 40.0);
    assert!(
        divergences(&t).is_empty(),
        "o vivo devia SER o cursor: {:?}",
        divergences(&t)
    );
    let cursor = t.undo.cursor_for_audit().expect("cursor");
    assert!(
        std::sync::Arc::ptr_eq(&t.canvas_rgba, &cursor.canvas_rgba),
        "e pelo MESMO buffer, nao so pelo mesmo conteudo"
    );

    // …e segue valendo depois de um segundo traço, que é onde a CADEIA passa a ser observável (um delta
    // sozinho está sempre certo; o que pode estar errado é a base, e ela só aparece do 2º passo em
    // diante — a lição de fixture da U1).
    stroke(&mut t, 80.0);
    assert!(
        divergences(&t).is_empty(),
        "2o traco: {:?}",
        divergences(&t)
    );
}

/// **EXCEÇÃO 1 — O ESCORRIDO: a água move o canvas vivo para além do cursor.**
///
/// A entrada é gravada no pen-up e a sim **continua correndo**, compositando pigmento sem gravar entrada
/// nenhuma. O cursor congelou no pen-up; o vivo não.
///
/// ⚠️ É por isso que o S3 **não pode** simplesmente escrever a janela dentro do plano vivo: fora da
/// janela o vivo carrega a gota, e o passo desfeito a deixaria lá — exatamente o bug que o
/// `absorb_foreign_writes` fechou no caminho de record, reaberto pelo outro lado.
#[test]
fn the_wet_drip_moves_the_live_canvas_past_the_cursor() {
    let mut t = tool(160, 220);
    t.set_paint_media(PaintMedia::WetPaint);
    use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};
    t.paint
        .wetpaint
        .knobs
        .set(Knob::Gravity, KNOB_DEFS[Knob::Gravity as usize].max);
    t.paint.wetpaint.knobs.water = 1.0;

    stroke(&mut t, 30.0);
    assert!(
        divergences(&t).is_empty(),
        "no pen-up o cursor ainda descreve o vivo"
    );

    // …e a água segue correndo. Nenhum destes ticks grava entrada de undo.
    t.wet_step_sync(240);
    assert_eq!(
        divergences(&t),
        vec!["canvas_rgba"],
        "o escorrido tem de ter movido o vivo para longe do cursor"
    );
}

/// **A CONSEQUÊNCIA, e por que hoje isto NÃO é bug:** a materialização não olha o vivo — ela constrói um
/// snapshot completo a partir do cursor, e o `restore_model` o instala **por atacado**. O escorrido é
/// substituído, não preservado.
///
/// O gate existe para separar as duas coisas que o S3 confunde com facilidade: *o vivo divergiu* (mede-se
/// acima) e *o undo errou* (não erra — hoje). Quem trocar a instalação por um patch no plano vivo tem de
/// vir por aqui.
#[test]
fn the_wholesale_install_is_what_makes_the_divergence_harmless_today() {
    let mut t = tool(160, 220);
    let pristine = t.canvas_rgba.as_ref().clone();
    t.set_paint_media(PaintMedia::WetPaint);
    use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};
    t.paint
        .wetpaint
        .knobs
        .set(Knob::Gravity, KNOB_DEFS[Knob::Gravity as usize].max);
    t.paint.wetpaint.knobs.water = 1.0;

    stroke(&mut t, 30.0);
    t.wet_step_sync(240);
    assert!(t.undo_last(), "havia um traco a desfazer");
    assert_eq!(
        *t.canvas_rgba, pristine,
        "desfazer o traco devolve a tela de antes dele, AO BYTE — escorrido incluso"
    );
}

/// **A CHAVE DO S3: absorver na entrada do UNDO leva as duas divergências a ZERO.**
///
/// Os dois gates acima medem a premissa e mostram que ela falha em dois casos; este mostra a cura, e
/// ela **não é código novo** — é o `absorb_foreign_writes` que o caminho de *record* já usava, chamado
/// no **segundo** consumidor do mesmo invariante. Depois dele o topo da história termina onde o vivo
/// está, por construção, e é isso que autoriza o S3 a escrever a janela DENTRO do plano vivo.
///
/// Medido sobre a suíte inteira (`PH2D_UNDO_AUDIT=1`): **92 undos/redos, 0 divergências** (era 2).
///
/// ⚠️ **Este gate prova a CURA, não a FIAÇÃO — e a diferença foi medida.** Ele chama a porta direto,
/// então tirar a chamada do `undo_last` o deixa **VERDE**: ele testa que a função funciona, não que o
/// produto a usa. Quem pina a fiação é o irmão logo abaixo
/// (`redo_brings_the_drip_back_…`), que dirige `undo_last`/`redo_last` de verdade e sangra com a mesma
/// mutação. Os dois não são redundantes — são as duas metades de *"a cura existe"* e *"ela está
/// ligada"*.
#[test]
fn absorbing_on_the_undo_path_makes_the_live_planes_the_cursor_even_after_a_drip() {
    let mut t = tool(160, 220);
    t.set_paint_media(PaintMedia::WetPaint);
    use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};
    t.paint
        .wetpaint
        .knobs
        .set(Knob::Gravity, KNOB_DEFS[Knob::Gravity as usize].max);
    t.paint.wetpaint.knobs.water = 1.0;

    stroke(&mut t, 30.0);
    t.wet_step_sync(240);
    // Controle: a água DE FATO escreveu depois do pen-up — sem isso o gate mede um caso que não contém
    // o fenômeno e fica verde por nada (é o irmão de `the_wet_drip_moves_the_live_canvas_past_the_cursor`).
    assert_eq!(
        divergences(&t),
        vec!["canvas_rgba"],
        "controle: o escorrido tinha de ter movido o vivo para alem do cursor"
    );

    t.absorb_foreign_writes_now();
    assert!(
        divergences(&t).is_empty(),
        "a absorcao no caminho de undo nao levou o vivo de volta ao cursor: {:?}",
        divergences(&t)
    );
}

/// **A consequência de PRODUTO, nomeada em vez de contrabandeada:** o refazer devolve o escorrido.
///
/// Antes, a gota nascida depois do pen-up era história de ninguém: o undo a apagava junto com o traço e
/// o **redo não a trazia de volta** — o artista perdia tinta que o produto tinha desenhado. Com a
/// absorção ela passa a pertencer ao passo que a causou, então desfazer a remove e refazer a devolve.
#[test]
fn redo_brings_the_drip_back_now_that_it_belongs_to_the_stroke_that_caused_it() {
    let mut t = tool(160, 220);
    t.set_paint_media(PaintMedia::WetPaint);
    use ph2d_wet_paint::tuning::{KNOB_DEFS, Knob};
    t.paint
        .wetpaint
        .knobs
        .set(Knob::Gravity, KNOB_DEFS[Knob::Gravity as usize].max);
    t.paint.wetpaint.knobs.water = 1.0;

    stroke(&mut t, 30.0);
    t.wet_step_sync(240);
    let with_drip = t.canvas_rgba.as_ref().clone();

    assert!(t.undo_last(), "havia um traco a desfazer");
    assert!(t.redo_last(), "e um a refazer");
    assert_eq!(
        *t.canvas_rgba, with_drip,
        "o refazer devolveu o traco SEM o escorrido — a gota continua sendo historia de ninguem"
    );
}
