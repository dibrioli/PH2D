//! **A cena do REFLUXO** — `PH2D_BUILD_SMOKE=63` (W2a).
//!
//! # A pergunta desta cena é de olho
//!
//! *O texto passa a caber numa caixa que eu autoro — e a caixa é dele, não da vista.*
//!
//! Dois textos com **a MESMA frase**: o de cima com uma caixa armada (ele já abre em várias
//! linhas), o de baixo sem nenhuma (o CONTROLE — ele corre reto e sai do quadro, que é o que
//! todo texto deste editor fazia até agora). O de cima nasce selecionado, então a fileira
//! **Width** está viva no painel no primeiro frame.
//!
//! ⚠️ **E a cena imprime o número que a torna válida:** em quantas linhas cada um coze. Se o
//! de cima disser `1 linha`, PARE — o refluxo não chegou, e o resto do roteiro não diz nada.

use ph2d_ecs::VecShape;
use ph2d_vec_scene::{Paint, Rgba8, VecPathId};

use crate::smoke_script::Step;
use crate::vec_text::VecTextEdit;

/// Tamanho em unidades de MUNDO. A cena vive numa caixa de ~±3,5, então isto deixa a frase
/// legível e larga o bastante para o corte ser óbvio.
const SIZE: f64 = 0.30;

/// A caixa autorada, em unidades de mundo — larga o bastante para caber várias palavras e
/// estreita o bastante para a frase ter de quebrar mais de uma vez.
const BOX: f64 = 3.6;

/// A frase. Longa e de palavras curtas: o quebrador decide em muitos pontos, então uma régua
/// errada aparece como uma linha que passa da caixa em vez de como um caso raro.
const LINE: &str = "the quick brown fox jumps over the lazy dog again and again";

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // Seleciona só no frame seguinte: o `sync` do render loop é que dá entidade ao path, e
        // sem entidade o painel não encontra o `VecShape::Text` (a razão do `text_fx_smoke`).
        4 => arm(app),
        _ => {}
    }
}

/// O texto montado no frame 3, à espera da entidade que o `sync` lhe dá no frame 4.
static PENDING: std::sync::Mutex<Option<Vec<(VecPathId, VecTextEdit)>>> =
    std::sync::Mutex::new(None);

/// A sessão de texto que a ferramenta produziria — montada à mão e cozida UMA vez, então o
/// objeto que nasce aqui é indistinguível de um digitado.
fn edit(origin: [f64; 2], wrap: Option<f64>, rgb: [u8; 3]) -> VecTextEdit {
    VecTextEdit {
        origin,
        size: SIZE,
        weight: 500.0,
        line_height: 1.25,
        tracking: 0.0,
        align: ph2d_tool_vector::TextAlign::Left,
        extra_axes: Vec::new(),
        family: None,
        fill: Some(Paint::solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255))),
        stroke: None,
        text: LINE.to_owned(),
        wrap_width: wrap,
        id: None,
        center: [0.0, 0.0],
    }
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("vector"));
    let mut made = Vec::new();
    // O de CIMA carrega a caixa; o de BAIXO é o controle. A ordem importa para o `arm`, que
    // seleciona o primeiro.
    for (origin, wrap, rgb) in [
        ([-1.8, 1.6], Some(BOX), [90, 150, 220]),
        ([-1.8, -1.8], None, [150, 150, 160]),
    ] {
        let mut e = edit(origin, wrap, rgb);
        crate::vec_text::regen_into(&mut gfx.vec_scene, &mut e);
        if let Some(id) = e.id {
            made.push((id, e));
        }
    }
    PENDING.lock().expect("smoke lock").replace(made);
}

/// Pendura o `VecShape::Text` (o painel só trata como TEXTO quem o tem) e seleciona o de cima.
fn arm(app: &mut crate::App) {
    let Some(made) = PENDING.lock().expect("smoke lock").take() else {
        return;
    };
    let Some(gfx) = app.gfx.as_mut() else { return };
    for (_, e) in &made {
        crate::vec_text_object::upsert_text_shape(&mut gfx.sim, &app.vec_entities, e);
    }
    let Some((boxed_id, boxed_edit)) = made.first() else {
        return;
    };
    let is_text = app
        .vec_entities
        .get(boxed_id)
        .and_then(|&b| {
            gfx.sim
                .world()
                .get::<VecShape>(ph2d_ecs::Entity::from_bits(b))
        })
        .is_some();
    app.vec_pen.select_many(&[*boxed_id]);
    let lines = |e: &VecTextEdit| {
        let font = crate::vec_font::resolve(e.family.as_deref());
        crate::vec_glyph::wrapped_lines(
            &font,
            &e.text,
            &crate::vec_text::layout_of(e),
            &crate::vec_text::axes_of(e),
            &crate::vec_glyph::TextPlacement::At(e.origin),
        )
        .len()
    };
    let boxed_lines = lines(boxed_edit);
    let loose_lines = made.get(1).map_or(0, |(_, e)| lines(e));
    eprintln!(
        "[wrap] montei 2 textos com a MESMA frase: o de cima com caixa de {BOX:.1} \
         ({boxed_lines} linha(s), SELECIONADO) e o de baixo sem caixa ({loose_lines} linha(s)) \
         -- objeto de texto: {is_text}."
    );
    if boxed_lines < 2 || loose_lines != 1 || !is_text {
        eprintln!(
            "[wrap] !! a cena NAO contem o fenomeno (esperado: >=2 linhas em cima, 1 embaixo, \
             objeto de texto). PARE e reporte -- o resto do roteiro nao significa nada."
        );
    }
    crate::smoke_script::script("wrap", "o texto de cima já está selecionado", STEPS);
}

/// Os passos que o artista executa. ⚠️ Eles vivem numa CONST para o gate de largura os poder
/// medir sem abrir uma janela — um roteiro que quebra no terminal é o defeito que a porta
/// [`crate::smoke_script`] existe para impedir.
const STEPS: &[Step] = &[
    Step {
        verb: "O QUE ABRE",
        lines: &[
            "Duas frases IGUAIS. A de cima quebra em várias linhas; a de baixo",
            "corre reta e sai do quadro — é o que todo texto fazia até agora.",
        ],
    },
    Step {
        verb: "A FILEIRA WIDTH",
        lines: &[
            "No painel, na seção Text: 'Width: Auto | Fixed'.",
            "O texto de cima abre em Fixed, com o slider 'Wrap width' logo abaixo.",
            "Clique Auto: o slider SOME e a frase volta a correr numa linha só.",
            "Um slider em Auto seria um controle que não faz nada.",
        ],
    },
    Step {
        verb: "A LARGURA É AUTORADA, NÃO DA VISTA",
        lines: &[
            "Volte a Fixed e arraste 'Wrap width'. O bloco re-quebra ao vivo.",
            "Agora dê zoom e pan: a quebra NÃO muda. A caixa é do texto.",
        ],
    },
    Step {
        verb: "O CURSOR SEGUE A LINHA DESENHADA",
        lines: &[
            "Duplo-clique no texto de cima para entrar em edição.",
            "O cursor tem de piscar no fim da ÚLTIMA linha desenhada —",
            "não no fim da linha que você digitou. Escreva mais palavras:",
            "quando a linha enche, o cursor desce junto com elas.",
        ],
    },
    Step {
        verb: "UMA PALAVRA MAIOR QUE A CAIXA",
        lines: &[
            "Aperte o slider até bem estreito. Uma palavra que não caiba",
            "TRANSBORDA inteira — ela não é partida ao meio (não há hífen).",
        ],
    },
    Step {
        verb: "SALVE E ABRA",
        lines: &[
            "Ctrl+S e Ctrl+O. A caixa tem de voltar com o texto:",
            "ela viaja no arquivo, não é estado de sessão.",
        ],
    },
    Step {
        verb: "O CONTROLE",
        lines: &[
            "Selecione o texto de BAIXO: ele abre em Auto, sem slider.",
            "Ele tem de continuar exatamente como estava — nada nesta wave",
            "muda um texto que nunca recebeu uma caixa.",
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena contém o fenômeno** — a caixa quebra, e o controle não.
    ///
    /// ⚠️ É a metade que o `arm` imprime, medida aqui sem janela: uma cena que monta dois
    /// textos e os deixa iguais é indistinguível da feature quebrada.
    #[test]
    fn the_boxed_text_wraps_and_the_control_does_not() {
        let font = crate::vec_font::resolve(None);
        let lines = |wrap: Option<f64>| {
            let e = edit([-1.8, 0.0], wrap, [0, 0, 0]);
            crate::vec_glyph::wrapped_lines(
                &font,
                &e.text,
                &crate::vec_text::layout_of(&e),
                &crate::vec_text::axes_of(&e),
                &crate::vec_glyph::TextPlacement::At(e.origin),
            )
            .len()
        };
        assert_eq!(lines(None), 1, "o controle nao pode quebrar");
        assert!(
            lines(Some(BOX)) >= 2,
            "a caixa de {BOX} tem de quebrar a frase — ela deu {} linha(s)",
            lines(Some(BOX))
        );
    }

    /// **O roteiro cabe no terminal** — a mesma régua dos irmãos.
    #[test]
    fn the_script_fits_the_terminal() {
        crate::smoke_script::assert_fits("wrap", STEPS);
    }
}
