//! **A cena pronta para o smoke da W0** — `PH2D_BUILD_SMOKE=21`.
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC, como o `fx_smoke` e o
//! `envelope_smoke`.
//!
//! # O que esta cena prova, e por que ela precisa de existir
//!
//! A W0 é uma **correção**, e uma correção só se vê reproduzindo o defeito. O defeito era:
//!
//! > aplique um efeito a um TEXTO, depois **mexa no texto** — e a pilha de efeitos desaparece.
//!
//! E ele era silencioso de um jeito particularmente mau: o re-cook do texto é *event-driven*,
//! então só dispara quando o artista volta a escrever ou mexe num knob — que é exatamente o
//! momento em que ele não está a olhar para o efeito. Quem monta a cena à mão gasta um minuto a
//! chegar aqui e pode nem carregar na tecla certa.
//!
//! A cena entrega o estado imediatamente ANTES do gesto que revelava o bug: um texto com Zig Zag
//! ativo, **já selecionado**, com a seção Text do painel a apontar para ele.
//!
//! # O roteiro (impresso no terminal)
//!
//! 1. A palavra tem de abrir **rugosa** — o Zig Zag está armado nos glifos.
//! 2. Mexa em **QUALQUER** knob da seção Text (Size, Weight, Tracking, Line Height…), ou
//!    dê duplo-clique na palavra e escreva mais uma letra.
//! 3. **A rugosidade tem de continuar lá.** Antes da W0 ela sumia neste gesto, e o texto voltava
//!    a ser liso — sem erro, sem aviso, sem nada na tela a dizer o que se perdeu.

use ph2d_ecs::VecShape;
use ph2d_vec_scene::effect::{FxEntry, PathEffect};
use ph2d_vec_scene::fx_zigzag::ZigZagSpec;
use ph2d_vec_scene::{Paint, Rgba8, VecPathId};

use crate::vec_text::VecTextEdit;

/// Tamanho do texto em unidades de MUNDO. A cena vive numa caixa de ~±3.5, então isto enche o
/// quadro sem sair dele — e um glifo grande é o que torna a rugosidade legível.
const SIZE: f64 = 1.4;

/// A palavra. Curta (o re-cook é por tecla) e com uma letra de **furo** (o `A`) — a pilha corre
/// por contorno, então o furo é a prova de que o efeito não trata o compound como um contorno só.
const WORD: &str = "PATH";

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // Seleciona só no frame seguinte: o `sync` do render loop é que dá entidade ao path, e
        // sem entidade o painel não encontra o `VecShape::Text` (a mesma razão do `fx_smoke`).
        4 => arm(app),
        _ => {}
    }
}

/// Monta o texto e arma o Zig Zag nele.
fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));

    // Uma sessão de texto montada à mão e cozida UMA vez — o mesmo caminho que a ferramenta de
    // texto usa, então o objeto que nasce aqui é indistinguível de um digitado.
    let mut edit = VecTextEdit {
        origin: [-2.6, -0.5],
        size: SIZE,
        weight: 700.0,
        line_height: 1.2,
        tracking: 0.0,
        align: ph2d_tool_vector::TextAlign::Left,
        extra_axes: Vec::new(),
        family: None,
        fill: Some(Paint::solid(Rgba8::new(90, 150, 220, 255))),
        stroke: None,
        text: WORD.to_owned(),
        id: None,
        center: [0.0, 0.0],
    };
    crate::vec_text::regen_into(&mut gfx.vec_scene, &mut edit);

    let Some(id) = edit.id else { return };
    // Zig Zag ATIVO. `amplitude` é PERCENTAGEM da forma, então 6 desenha o mesmo em qualquer
    // escala; 40 cristas sobre a palavra inteira dá uma borda que ninguém confunde com lisa.
    if let Some(p) = gfx.vec_scene.path_mut(id) {
        p.effects = vec![FxEntry::new(PathEffect::ZigZag(ZigZagSpec {
            amplitude: 6.0,
            ridges: 40.0,
            smooth: false,
            rough_seed: None,
        }))];
    }
    PENDING.lock().expect("smoke lock").replace((id, edit));
}

/// O texto montado no frame 3, à espera da entidade que o `sync` lhe dá no frame 4.
static PENDING: std::sync::Mutex<Option<(VecPathId, VecTextEdit)>> = std::sync::Mutex::new(None);

/// Pendura o `VecShape::Text` (o painel só trata como TEXTO quem o tem) e seleciona.
fn arm(app: &mut crate::App) {
    let Some((id, edit)) = PENDING.lock().expect("smoke lock").take() else {
        return;
    };
    let Some(gfx) = app.gfx.as_mut() else { return };
    crate::vec_text_object::upsert_text_shape(&mut gfx.sim, &app.vec_entities, &edit);
    let is_text = app
        .vec_entities
        .get(&id)
        .and_then(|&b| {
            gfx.sim
                .world()
                .get::<VecShape>(ph2d_ecs::Entity::from_bits(b))
        })
        .is_some();
    app.vec_pen.select_many(&[id]);
    eprintln!(
        "[smoke] W0 texto+efeitos: a palavra \"{WORD}\" com Zig Zag ATIVO, selecionada \
         (objeto de texto: {is_text}).\n\
         [smoke]   1. A palavra tem de abrir RUGOSA.\n\
         [smoke]   2. Mexa em QUALQUER knob da secao Text (Size / Weight / Tracking / Line \
         Height) -- ou de duplo-clique nela e escreva mais uma letra.\n\
         [smoke]   3. A RUGOSIDADE TEM DE CONTINUAR LA. Antes da W0 ela sumia neste gesto, \
         em silencio."
    );
    if !is_text {
        eprintln!(
            "[smoke] !! o VecShape::Text NAO foi pendurado -- o painel nao vai tratar isto \
             como texto, e o smoke nao significa nada. PARE e reporte."
        );
    }
}
