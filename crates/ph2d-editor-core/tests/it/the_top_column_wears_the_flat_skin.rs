//! ⭐⭐⭐ **A COLUNA DO TOPO — o balão de aviso e a barra de trabalho — nunca passou por porta
//! nenhuma, e dois buracos do censo explicam porquê.**
//!
//! As waves 2–5 levaram a porta da moldura (`stroke_frame`) a 44 pintores e puseram a catraca a
//! zero: num tema moderno a pele é **plana** e o contorno de repouso desaparece. Os dois inquilinos
//! da coluna do topo continuaram a traçar um `stroke_rounded_rect` cru a 1 px — logo, num tema
//! moderno, eles desenhavam a moldura que toda a casa tinha perdido.
//!
//! # ⛔⛔ Porque é que o censo não os viu — são DOIS buracos, não um
//!
//! **(a) Uma isenção escrita para uma FUNÇÃO protegia o FICHEIRO.** O `paint.rs` estava na lista
//! com o motivo *«é a PORTA: `stroke_frame` chama `stroke_rounded_rect` por definição»* — verdade
//! para o corpo daquela função, e o pintor do balão vive **290 linhas abaixo**, no mesmo ficheiro.
//! *Uma isenção nomeia uma coisa e cobre tudo o que partilhe o ficheiro com ela.*
//!
//! **(b) O censo enumerava directórios À MÃO.** Ele varria `widget/` e `screens/hero/` mais **um
//! ficheiro escrito à mão** (`paint.rs`) — e o `progress.rs`, que pinta o outro inquilino da mesma
//! coluna, nunca foi olhado. *Um censo que enumera à mão afirma sobre o que alguém se lembrou de
//! escrever.*
//!
//! # Porque este gate mede PIXEL e não texto
//!
//! O censo textual da moldura pergunta *«este ficheiro conhece a porta?»*, e as duas curas desta
//! wave deixam o ficheiro a conhecê-la — logo ele fica verde **mesmo que alguém reponha o traço
//! cru ao lado**. ⇒ a régua que fecha a família tem de ser a CENA: pinta-se a coluna nos dois
//! temas e conta-se a geometria. *Uma faixa reservada não é uma faixa pintada, e um censo que lê
//! nomes não distingue as duas.*

use ph2d_editor_core::paint::{Paint, PaintCtx};
use ph2d_editor_core::progress::{JobQueue, Progress};
use ph2d_editor_core::toast::{Toast, ToastQueue, ToastSeverity};
use ph2d_editor_core::zones::Rect;
use ph2d_text::TextSystem;
use ph2d_tokens::Theme;
use ph2d_vector::VectorScene;

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 1200.0, 800.0)
}

/// Quanta geometria a coluna emite ao pintar um balão de aviso neste tema.
fn toast_segments(theme: Theme) -> u32 {
    let mut q = ToastQueue::default();
    q.push(Toast::new("mensagem", ToastSeverity::Info));
    let mut text = TextSystem::new();
    let mut scene = VectorScene::new();
    let mut ctx = PaintCtx {
        theme,
        viewport: viewport(),
        text: &mut text,
    };
    q.paint(&mut scene, &mut ctx);
    scene.inner().encoding().n_path_segments
}

/// O mesmo para a barra de trabalho, o outro inquilino da coluna.
fn job_segments(theme: Theme) -> u32 {
    // ⚠️ `new()`, e não `default()`: até esta wave o `derive(Default)` dava `cap = 0` e a fila
    //    descartava toda barra **em silêncio** — a 1.ª corrida deste gate pintou uma coluna vazia
    //    e acusou o pintor. A armadilha está curada; o gate dela vive em `progress/tests.rs`.
    let mut q = JobQueue::new();
    q.push(Progress::new("A trabalhar"));
    let mut text = TextSystem::new();
    let mut scene = VectorScene::new();
    let mut ctx = PaintCtx {
        theme,
        viewport: viewport(),
        text: &mut text,
    };
    q.paint_below(0, &mut scene, &mut ctx);
    scene.inner().encoding().n_path_segments
}

/// ⭐⭐⭐ **Num tema moderno a coluna do topo não traça moldura de repouso.**
///
/// ⚠️ A régua é a DIFERENÇA entre os dois temas, não um número absoluto: o clássico traça e o
/// moderno não, logo o clássico tem de emitir **mais** caminhos. Um número absoluto envelheceria
/// à primeira mudança de conteúdo do balão.
#[test]
fn the_toast_drops_its_frame_in_a_modern_theme() {
    let classico = toast_segments(Theme::Forge);
    let moderno = toast_segments(Theme::Dark);
    assert!(classico > 0, "o balao nao pintou nada no tema classico");
    assert!(moderno > 0, "o balao nao pintou nada no tema moderno");
    assert!(
        moderno < classico,
        "o balao emite a mesma geometria nos dois temas ({moderno} contra {classico}): ele traca \
         no moderno a moldura que a pele plana apagou em toda a casa"
    );
}

/// ⭐ **E a barra de trabalho, que partilha a coluna, tem de dizer o mesmo.**
///
/// ⚠️ Sem este, curar só o balão deixaria os dois inquilinos da MESMA coluna com peles diferentes
/// — que é pior que os dois errados por igual.
#[test]
fn the_job_bar_drops_its_frame_in_a_modern_theme() {
    let classico = job_segments(Theme::Forge);
    let moderno = job_segments(Theme::Dark);
    assert!(classico > 0, "a barra nao pintou nada no tema classico");
    assert!(moderno > 0, "a barra nao pintou nada no tema moderno");
    assert!(
        moderno < classico,
        "a barra de trabalho emite a mesma geometria nos dois temas ({moderno} contra \
         {classico}): ela traca a moldura que a pele plana apagou"
    );
}

/// ⛔ **Os dois inquilinos concordam.** A coluna é uma; duas peles nela seria o defeito seguinte.
#[test]
fn both_tenants_of_the_column_answer_the_theme_the_same_way() {
    let toast_perde = toast_segments(Theme::Forge) - toast_segments(Theme::Dark);
    let job_perde = job_segments(Theme::Forge) - job_segments(Theme::Dark);
    assert_eq!(
        toast_perde, job_perde,
        "o balao perde {toast_perde} caminhos ao mudar de tema e a barra perde {job_perde}: os \
         dois inquilinos da mesma coluna deixaram de responder ao tema da mesma maneira"
    );
}
