//! **A cena dos TOKENS** — `PH2D_BUILD_SMOKE=50` (plano UI/UX §4/W4).
//!
//! Módulo irmão do [`crate::build_smoke`] pelo teto de LOC (HR-18), como os `*_smoke` vizinhos.
//!
//! ⚠️ **Ela dá o MATERIAL e não binda nada** — a cicatriz que o `impasto_smoke` prega e que o
//! `bool_smoke` repete: um smoke que arma o estado por baixo do pano pula justamente a costura que
//! existe para provar. Nenhuma forma nasce presa a token nenhum; quem prende é o artista.
//!
//! # A pergunta desta cena é UMA, e é de olho
//!
//! *Prender três propriedades a três tokens e apertar `M`: o card re-veste — **e o app inteiro
//! re-veste junto**, porque é a MESMA tabela.* É a entrega da wave, e a razão de este módulo
//! existir num editor em vez de numa ferramenta de mockup.
//!
//! O que ela monta, e por quê:
//! - **o CARD**: fundo + borda + uma barra de "texto", as três peças de um card de UI — o alvo
//!   natural de `bg-2`, `border` e `text-1`;
//! - **o CONTROLE**, ao lado: um card visualmente idêntico que fica com os literais. Sem ele, o
//!   `M` re-vestiria a tela toda e nada diria o que foi o token e o que foi o tema;
//! - **a forma SEM TRAÇO**: onde a row de token do traço **não** é oferecida — o preço nomeado da
//!   assimetria (um token de cor não inventa largura).

use ph2d_vec_scene::{Paint, Rgba8, StrokeSpec, VecPath, rectangle};

/// Os dois cards, em `x`. O da esquerda é o que se binda; o da direita é o CONTROLE.
const CARD_X: [f64; 2] = [-2.6, 0.9];

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

fn outlined(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.stroke = Some(StrokeSpec::new(
        Rgba8::new(rgb[0], rgb[1], rgb[2], 255),
        0.05,
    ));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        4 => announce(app),
        _ => {}
    }
}

/// Um card: o painel de fundo (com traço) + a barra de "texto" dentro.
fn card(s: &mut ph2d_vec_scene::VecScene, x: f64) {
    s.push_path(outlined(
        tint(rectangle([x - 1.0, 0.4], [x + 1.0, 2.2]), [40, 40, 46]),
        [90, 90, 100],
    ));
    s.push_path(tint(
        rectangle([x - 0.7, 1.6], [x + 0.7, 1.85]),
        [225, 225, 232],
    ));
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let s = &mut gfx.vec_scene;
    card(s, CARD_X[0]);
    card(s, CARD_X[1]);
    // A forma SEM TRAÇO — onde a row do token de traço não é oferecida.
    s.push_path(tint(
        rectangle([CARD_X[1] + 1.6, 0.4], [CARD_X[1] + 2.4, 1.2]),
        [200, 150, 120],
    ));
}

fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    let n = gfx.vec_scene.paths().len();
    let stroked = gfx
        .vec_scene
        .paths()
        .iter()
        .filter(|p| p.stroke.is_some())
        .count();
    eprintln!(
        "[token] cena montada: {n} formas ({stroked} com traco) — o CARD (fundo+borda+texto), o \
         CONTROLE identico ao lado, e a forma SEM TRACO."
    );
    eprintln!("[token] nada nasce bindado — o default do produto. Quem prende e' voce.");
    eprintln!("[token] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Clique no FUNDO do card da esquerda. Na secao Fill, logo abaixo da swatch,");
    eprintln!("     ha' uma row 'Token' com um chip a dizer '—'. ⚠️ Ela fica AO LADO da cor que");
    eprintln!("     substitui, e nao numa secao a' parte: e' onde a pergunta nasce.");
    eprintln!("  2. Abra o chip e escolha 'bg-2'. ⚠️ O fundo muda na hora, e o chip passa a dizer");
    eprintln!("     o nome do token — e' assim que a forma DIZ que nao esta' mais no literal.");
    eprintln!("  3. Com o mesmo fundo selecionado, na secao Stroke escolha o token 'border'.");
    eprintln!("  4. Selecione a BARRA de texto e binde o Fill a 'text-1'.");
    eprintln!("  5. ⚠️ **A PERGUNTA DA WAVE — aperte `M`** (troca o modo: Forge/Workshop/");
    eprintln!("     Sunstone/Blueprint). O card da esquerda re-veste E O APP INTEIRO re-veste");
    eprintln!("     junto: e' a MESMA tabela que veste os 44 widgets do editor.");
    eprintln!("     O card da DIREITA nao se mexe — ele e' o CONTROLE, ainda no literal.");
    eprintln!("  6. Volte ao chip do fundo e escolha 'None (use literal)'. ⚠️ A cor ORIGINAL");
    eprintln!("     volta: bindar nunca apagou o literal, so' o cobriu.");
    eprintln!("  7. Selecione a forma da direita (a SEM TRACO). ⚠️ A row de token do Fill esta'");
    eprintln!("     la', e a do Stroke NAO: um token de cor nao inventa a largura que falta.");
    eprintln!("  8. Ctrl+Z depois de bindar e de soltar — os dois desfazem.");
    eprintln!("  9. Ctrl+S e Ctrl+O: o binding sobrevive ao arquivo.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena contém os fenômenos que o roteiro pede.**
    ///
    /// Um roteiro que pede o que a geometria não tem engana exactamente quem o corre — e os
    /// passos 3 e 7 dependem de haver forma COM traço e forma SEM traço na mesma tela.
    #[test]
    fn the_scene_holds_what_the_script_asks_for() {
        let mut s = ph2d_vec_scene::VecScene::new();
        card(&mut s, CARD_X[0]);
        card(&mut s, CARD_X[1]);
        s.push_path(tint(
            rectangle([CARD_X[1] + 1.6, 0.4], [CARD_X[1] + 2.4, 1.2]),
            [200, 150, 120],
        ));
        let stroked = s.paths().iter().filter(|p| p.stroke.is_some()).count();
        let bare = s.paths().iter().filter(|p| p.stroke.is_none()).count();
        assert_eq!(s.paths().len(), 5, "dois cards de duas pecas + a forma nua");
        assert_eq!(stroked, 2, "o passo 3 precisa de forma COM traco");
        assert!(bare >= 1, "o passo 7 precisa de forma SEM traco");
        assert!(
            s.paths().iter().all(|p| p.fill.is_some()),
            "toda forma tem preenchimento — o passo 1 binda o Fill"
        );
    }

    /// Os dois cards são **visualmente iguais** — é isso que faz do da direita um controle.
    ///
    /// Se nascessem diferentes, o artista atribuiria ao token uma diferença que já estava lá.
    #[test]
    fn the_control_card_is_identical_to_the_bound_one() {
        let mut a = ph2d_vec_scene::VecScene::new();
        card(&mut a, CARD_X[0]);
        let mut b = ph2d_vec_scene::VecScene::new();
        card(&mut b, CARD_X[1]);
        for (x, y) in a.paths().iter().zip(b.paths().iter()) {
            assert_eq!(x.fill, y.fill, "mesma cor de preenchimento");
            assert_eq!(
                x.stroke.as_ref().map(|s| s.color),
                y.stroke.as_ref().map(|s| s.color),
                "mesma cor de traco"
            );
        }
    }
}
