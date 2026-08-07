//! **A cena de PRENDER ao token** — `PH2D_BUILD_SMOKE=51` (plano UI/UX §4/W4).
//!
//! ⚠️ **Ela NÃO abre o painel de Tokens** — o sujeito aqui é a ARTE que segue a tabela, e o
//! painel (autorar a cor, o elo, o contraste) é a cena irmã **`=59`** (`tokens_smoke`). Os dois
//! nomes de arquivo diferem por uma letra e as duas cenas dizem "TOKENS": se a pergunta é *"o
//! artista consegue MUDAR a tabela?"*, a cena é a `=59`.
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
        4 => adopt(app),
        5 => announce(app),
        _ => {}
    }
}

/// A BARRA da W4c.4: uma moldura com três filhos, para o token de VÃO ter onde espaçar.
///
/// ⚠️ **Ela não recebe `VecLayout`**, pela mesma lei do resto desta cena e da cena do auto layout:
/// o fluxo é o que o artista arma. Uma cena que já o trouxesse ligado provaria o passe e não o
/// produto.
const BAR_KIDS: usize = 3;
const BAR_Y: [f64; 2] = [-1.4, -0.4];

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

/// **A geometria da cena, numa porta pura** — a mesma que o produto empurra e que o gate mede.
///
/// ⚠️ O gate re-implementava esta lista, e por isso ela DERIVAVA: acrescentar uma forma ao produto
/// deixava o gate a medir a cena de ontem, verde, sobre um roteiro que já falava de outra coisa.
#[cfg(test)]
fn scene_paths() -> ph2d_vec_scene::VecScene {
    let mut s = ph2d_vec_scene::VecScene::new();
    fill_scene(&mut s);
    s
}

fn fill_scene(s: &mut ph2d_vec_scene::VecScene) {
    card(s, CARD_X[0]);
    card(s, CARD_X[1]);
    // A forma SEM TRAÇO — onde a row do token de traço não é oferecida.
    s.push_path(tint(
        rectangle([CARD_X[1] + 1.6, 0.4], [CARD_X[1] + 2.4, 1.2]),
        [200, 150, 120],
    ));
    // A BARRA (W4c.4): os três filhos primeiro, a moldura por último — a mesma ordem de pilha da
    // cena do auto layout, e pela mesma razão (a moldura é o fundo do card).
    for i in 0..BAR_KIDS {
        let x = CARD_X[0] + 0.15 + i as f64 * 0.5;
        s.push_path(tint(
            rectangle([x, BAR_Y[0] + 0.15], [x + 0.4, BAR_Y[1] - 0.15]),
            [120, 170, 220],
        ));
    }
    s.push_path(tint(
        rectangle([CARD_X[0], BAR_Y[0]], [CARD_X[0] + 3.4, BAR_Y[1]]),
        [48, 48, 56],
    ));
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    fill_scene(&mut gfx.vec_scene);
}

/// Pendura os três filhos na barra e a marca como MOLDURA — sem armar o fluxo (ver [`BAR_KIDS`]).
fn adopt(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    // A barra é o ÚLTIMO caminho; os três filhos são os que a precedem.
    let Some((&bar, kids)) = ids.split_last().map(|(b, r)| (b, &r[r.len() - BAR_KIDS..])) else {
        return;
    };
    let Some(&fb) = app.vec_entities.get(&bar) else {
        return;
    };
    let frame = ph2d_ecs::Entity::from_bits(fb);
    if let Ok(mut e) = gfx.sim.world_mut().get_entity_mut(frame) {
        e.insert(ph2d_ecs::VecFrame { clip: false });
    }
    for k in kids {
        let Some(&kb) = app.vec_entities.get(k) else {
            continue;
        };
        if let Ok(mut e) = gfx
            .sim
            .world_mut()
            .get_entity_mut(ph2d_ecs::Entity::from_bits(kb))
        {
            e.insert(ph2d_ecs::ChildOf(frame));
        }
    }
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
    eprintln!("  6. ⚠️ Repare na swatch de Fill: ela tem uma RACHURA vermelha. A cor que ela");
    eprintln!("     mostra e' o literal, e o token o cobre — a rachura diz que ele nao esta'");
    eprintln!("     em uso. Reabra o chip: a linha DESTACADA e' a do token, nao 'None'.");
    eprintln!("  7. Volte ao chip do fundo e escolha 'None (use literal)'. ⚠️ A cor ORIGINAL");
    eprintln!("     volta (bindar nunca apagou o literal) e a rachura some.");
    eprintln!("  8. ⚠️ Binde outra vez e agora CLIQUE NA SWATCH e escolha uma cor no picker: o");
    eprintln!("     token volta sozinho para None. Escolher uma cor SOLTA o token — senao a");
    eprintln!("     swatch mostraria um valor que a arte nao usa.");
    eprintln!("  9. Selecione a forma da direita (a SEM TRACO). ⚠️ A row de token do Fill esta'");
    eprintln!("     la', e a do Stroke NAO: um token de cor nao inventa a largura que falta.");
    eprintln!(" 10. Ctrl+Z depois de bindar e de soltar — os dois desfazem.");
    eprintln!(" 11. Ctrl+S e Ctrl+O: o binding sobrevive ao arquivo.");
    eprintln!("[token] ⚠️ **W4c.4 — OS TOKENS DE ESCALA (e' ESTA a wave nova):**");
    eprintln!(" 12. Selecione o FUNDO do card da esquerda (ele tem traco). Na secao Stroke, logo");
    eprintln!("     abaixo do slider 'Width', ha' uma row 'Token' NOVA. Escolha 'stroke.heavy'.");
    eprintln!("     ⚠️ O contorno ENGROSSA, e o CHIP do Width ganha a rachura: o numero que ele");
    eprintln!("     mostra e' a largura autorada, e o token a cobre.");
    eprintln!(" 13. ⚠️ **A pergunta:** va' ao painel de TOKENS (tecla T), secao 'Scale (px)', e");
    eprintln!("     mexa em 'stroke.heavy'. O contorno da ARTE segue junto — e o app tambem.");
    eprintln!(" 14. Digite um numero no campo Width. ⚠️ O token volta para None sozinho: autorar");
    eprintln!("     um valor SOLTA o token, a mesma lei da cor.");
    eprintln!(" 15. Selecione a BARRA de baixo (a moldura com tres quadrados). Na secao Layout");
    eprintln!("     escolha a direcao 'Row' — os tres entram em fila. Agora, na row 'Token' logo");
    eprintln!("     abaixo do campo 'Gap', escolha 'spacing.4xl'. ⚠️ A FILA SE ABRE.");
    eprintln!("     Mexa em 'spacing.4xl' no painel de tokens: o espacamento segue.");
    eprintln!(" 16. ⚠️ CONTROLE da regua: um token de escala vale PIXELS e o documento mede");
    eprintln!("     MUNDO. Sem a regua do projeto (Settings > pixels per meter, 100 por padrao)");
    eprintln!("     'stroke.default' seria 1,5 UNIDADE — um traco com 19% da altura de uma");
    eprintln!("     moldura de telefone. Se algum traco sair grosso assim, a regua se perdeu.");
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
        let s = scene_paths();
        let stroked = s.paths().iter().filter(|p| p.stroke.is_some()).count();
        let bare = s.paths().iter().filter(|p| p.stroke.is_none()).count();
        assert_eq!(
            s.paths().len(),
            5 + BAR_KIDS + 1,
            "dois cards de duas pecas + a forma nua + a barra com os filhos dela"
        );
        assert_eq!(stroked, 2, "os passos 3 e 12 precisam de forma COM traco");
        assert!(bare >= 1, "o passo 9 precisa de forma SEM traco");
        assert!(
            s.paths().iter().all(|p| p.fill.is_some()),
            "toda forma tem preenchimento — o passo 1 binda o Fill"
        );
    }

    /// **A BARRA do passo 15 existe, e os filhos dela CABEM dentro dela.**
    ///
    /// ⚠️ O oráculo é a GEOMETRIA, não a contagem: uma barra estreita demais faria o `Row` empurrar
    /// os filhos para fora no primeiro frame, e o artista atribuiria ao token um transbordo que a
    /// cena já tinha. E ela mede a FOLGA — sem folga, aumentar o vão não abre nada e o passo não
    /// mostra o que promete.
    #[test]
    fn the_bar_has_room_for_the_gap_to_open() {
        let s = scene_paths();
        let paths = s.paths();
        let bar = paths.last().expect("a barra e' o ultimo caminho");
        let (blo, bhi) =
            ph2d_vec_scene::curve_bbox_in_frame(bar, 1.0, 0.0).expect("a barra tem caixa");
        let kids = &paths[paths.len() - 1 - BAR_KIDS..paths.len() - 1];
        let content: f64 = kids
            .iter()
            .filter_map(|k| ph2d_vec_scene::curve_bbox_in_frame(k, 1.0, 0.0))
            .map(|(lo, hi)| hi[0] - lo[0])
            .sum();
        let slack = (bhi[0] - blo[0]) - content;
        assert_eq!(kids.len(), BAR_KIDS);
        assert!(
            slack > 1.0,
            "a barra tem de sobrar espaco para o vao ABRIR a fila, e sobra {slack}"
        );
        for k in kids {
            let (lo, hi) = ph2d_vec_scene::curve_bbox_in_frame(k, 1.0, 0.0).expect("caixa");
            assert!(
                lo[0] >= blo[0] && hi[0] <= bhi[0] && lo[1] >= blo[1] && hi[1] <= bhi[1],
                "um filho nasce FORA da barra: {lo:?}..{hi:?} contra {blo:?}..{bhi:?}"
            );
        }
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
