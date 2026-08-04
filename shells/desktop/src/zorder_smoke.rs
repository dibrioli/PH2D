//! **A cena da ORDEM DE Z** — `PH2D_BUILD_SMOKE=57` (Enio, 2026-08-04).
//!
//! Módulo irmão do [`crate::component_smoke`] pelo teto de LOC. Duas perguntas, e as duas são de
//! olho:
//!
//! 1. *O filho aparece?* — a lei do Godot: o filho desenha **sobre** o pai. Antes desta wave um
//!    pai comum (qualquer forma com uma forma filha) pintava por cima do próprio conteúdo, e o
//!    filho ficava invisível a menos que se baixasse a opacidade.
//! 2. *Os botões de z-order fazem alguma coisa?* — eles escreviam na ordem do VETOR da cena, que a
//!    projeção da árvore reescreve a cada frame. Acendiam, mexiam, e o frame seguinte desfazia.
//!
//! ⚠️ **A cena não arma nada** — nem parentesco de conveniência, nem seleção. O que ela dá é o
//! material: um pai grande com um filho pequeno DENTRO dele (onde a sobreposição é total, que é o
//! caso em que o defeito era fatal) e três irmãos que se sobrepõem em cascata, para o Forward /
//! Backward terem o que trocar.

use ph2d_ecs::Entity;
use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// O PAI: um retângulo grande e opaco.
const PARENT: [f64; 4] = [-5.0, 0.4, -1.0, 2.6];
/// O FILHO: pequeno, **inteiramente dentro** do pai — se o pai pintar por cima, ele some.
const CHILD: [f64; 4] = [-4.2, 1.0, -1.8, 2.0];
/// Os três irmãos da cascata, para os botões de z-order terem o que trocar.
const SIBS: [[f64; 4]; 3] = [
    [0.6, 0.6, 3.0, 2.0],
    [1.6, 1.0, 4.0, 2.4],
    [2.6, 1.4, 5.0, 2.8],
];
/// O CONTROLE: um quadrado solto, longe, que nunca participa de nada.
const CONTROL: [f64; 4] = [-5.0, -2.2, -4.2, -1.4];

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // O parentesco só depois do `sync` — é ele que dá entidade a cada caminho.
        6 => adopt(app),
        7 => announce(app),
        _ => {}
    }
}

/// **Os seis caminhos da cena** — porta única, e é o que torna os gates abaixo um ORÁCULO em vez
/// de um espelho: eles medem a geometria que a cena de facto empurra.
fn paths() -> Vec<VecPath> {
    let mut out = vec![
        tint(
            rectangle([PARENT[0], PARENT[1]], [PARENT[2], PARENT[3]]),
            [58, 96, 168],
        ),
        tint(
            rectangle([CHILD[0], CHILD[1]], [CHILD[2], CHILD[3]]),
            [240, 196, 64],
        ),
    ];
    // Os três irmãos, cada um de uma cor, para o artista ver QUEM subiu.
    for (i, r) in SIBS.iter().enumerate() {
        let rgb = [[214, 82, 82], [82, 184, 120], [148, 110, 214]][i];
        out.push(tint(rectangle([r[0], r[1]], [r[2], r[3]]), rgb));
    }
    out.push(tint(
        rectangle([CONTROL[0], CONTROL[1]], [CONTROL[2], CONTROL[3]]),
        [80, 82, 92],
    ));
    out
}

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for p in paths() {
        gfx.vec_scene.push_path(p);
    }
}

/// Pendura o AMARELO no AZUL — e mais nada.
fn adopt(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    if ids.len() < 2 {
        return;
    }
    let (Some(&pb), Some(&cb)) = (app.vec_entities.get(&ids[0]), app.vec_entities.get(&ids[1]))
    else {
        return;
    };
    // ⚠️ Pela PORTA (`reparent_keeping_world`): o `settle_origins` já pôs cada forma-raiz a
    // carregar a própria translação, e um `ChildOf` cru SOMA as duas — o filho saltaria para fora
    // do pai e a cena deixaria de conter o fenômeno.
    crate::vec_transform::reparent_keeping_world(
        &mut gfx.sim,
        Entity::from_bits(cb),
        Entity::from_bits(pb),
    );
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    eprintln!(
        "[zorder] cena montada: {} formas — um PAI azul com um filho AMARELO dentro dele, tres \
         irmaos sobrepostos e um controle. Nenhuma selecao armada.",
        gfx.vec_scene.paths().len()
    );
    eprintln!("[zorder] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. ⚠️ **A PROVA DA LEI**: o retangulo AMARELO tem de estar VISIVEL, por cima do");
    eprintln!(
        "     azul. Ele e' FILHO do azul — e em Godot (e no Figma, e no Illustrator) o filho"
    );
    eprintln!("     desenha SOBRE o pai. Se ele sumiu, a lei quebrou.");
    eprintln!("  2. ⚠️ **A HIERARQUIA VIROU**: o amarelo aparece ABAIXO do azul na lista, e mais");
    eprintln!("     abaixo = mais A' FRENTE (a convencao das game engines). Confira nos tres");
    eprintln!("     irmaos da direita: o que esta' por CIMA no canvas e' o ULTIMO da lista.");
    eprintln!("  3. **Desenhe uma forma nova** (Pen, ou a ferramenta de retangulo) por cima dos");
    eprintln!("     tres irmaos. Ela tem de nascer no FIM da Hierarquia e POR CIMA do desenho.");
    eprintln!("  4. ⚠️ **A PROVA DA INSTANCIA** (o report que abriu esta wave): selecione o AZUL,");
    eprintln!("     **Create Component**, depois **Place Instance**. Na copia o amarelo tem de");
    eprintln!("     estar POR CIMA tambem — era exactamente aqui que ele ia para tras do pai.");
    eprintln!("  5. ⚠️ **O Z INDEX** (o que estava MUDO ate' agora): clique no retangulo VERMELHO");
    eprintln!("     (o de tras dos tres da direita). A secao **Arrange** mostra um campo **Z**");
    eprintln!("     com 0. Escreva **5** e prima Enter: ele salta para a frente dos outros dois");
    eprintln!("     SEM mudar de lugar na Hierarquia. ⚠️ E' isso que 'global' significa — o Z");
    eprintln!("     manda, e a lista so' conta entre objetos que EMPATAM nele. Volte a 0 e ele");
    eprintln!("     regressa ao fundo.");
    eprintln!("  6. ⚠️ **OS BOTOES SO' MEXEM NO Z** (a lei nova): com o vermelho selecionado,");
    eprintln!("     carregue em **To Front**, **Backward**, **Forward**, **To Back**. Em TODOS:");
    eprintln!("     (a) a forma muda de lugar no canvas · (b) o numero do campo **Z** muda junto");
    eprintln!("     · (c) a **Hierarquia NAO se mexe** — a lista fica na mesma ordem, sempre.");
    eprintln!("     ⚠️ E' a metade (c) que esta wave entrega: antes os botoes re-arrumavam a");
    eprintln!("     lista do artista por baixo dele. Ctrl+Z desfaz cada passo.");
    eprintln!("     ⚠️ **Limite conhecido e honesto**: com os tres irmaos ainda empatados em 0,");
    eprintln!("     **Forward** passa os DOIS de uma vez — nao existe inteiro entre eles. Depois");
    eprintln!("     de haver Z distinto, ele anda um lugar de cada vez.");
    eprintln!("  7. ⚠️ **O CONTROLE**: o quadrado CINZA nunca virou nada e tem de estar");
    eprintln!("     exactamente onde nasceu, em todos os passos.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A caixa de mundo de um dos caminhos que a cena empurra.
    fn bbox(i: usize) -> ([f64; 2], [f64; 2]) {
        let p = &paths()[i];
        let mut lo = [f64::MAX; 2];
        let mut hi = [f64::MIN; 2];
        for v in p.verts_all() {
            for a in 0..2 {
                lo[a] = lo[a].min(v.anchor[a]);
                hi[a] = hi[a].max(v.anchor[a]);
            }
        }
        (lo, hi)
    }

    /// **O filho está INTEIRAMENTE dentro do pai** — é a única fixture que contém o fenômeno.
    ///
    /// ⚠️ Com o filho a transbordar, a metade dele que sobra ficaria visível mesmo com o pai a
    /// pintar por cima, e o smoke passaria com o defeito de pé.
    #[test]
    fn the_child_is_entirely_inside_the_parent() {
        let (plo, phi) = bbox(0);
        let (clo, chi) = bbox(1);
        for a in 0..2 {
            assert!(
                clo[a] > plo[a] && chi[a] < phi[a],
                "o filho sai do pai no eixo {a}: {clo:?}..{chi:?} contra {plo:?}..{phi:?}"
            );
        }
    }

    /// **Os três irmãos SE SOBREPÕEM** — sem sobreposição, trocar a ordem deles não se vê.
    #[test]
    fn the_three_siblings_overlap_each_other() {
        for i in 0..2 {
            let (a_lo, a_hi) = bbox(2 + i);
            let (b_lo, b_hi) = bbox(3 + i);
            assert!(
                a_hi[0] > b_lo[0] && b_hi[0] > a_lo[0] && a_hi[1] > b_lo[1] && b_hi[1] > a_lo[1],
                "os irmaos {i} e {} nao se sobrepoem: trocar o z deles seria invisivel",
                i + 1
            );
        }
    }

    /// **O controle está longe de tudo** — e não toca a cascata nem o par pai/filho.
    #[test]
    fn the_control_is_far_from_the_action() {
        let (plo, _) = bbox(0);
        let (_, chi) = bbox(5);
        assert!(
            chi[1] < plo[1],
            "o controle ({:.2}) encosta na faixa do pai ({:.2})",
            chi[1],
            plo[1]
        );
    }

    /// **Cada forma tem uma cor própria** — duas iguais fariam o roteiro nomear uma coisa que o
    /// artista não consegue apontar.
    #[test]
    fn every_shape_has_its_own_colour() {
        let fills: Vec<_> = paths().iter().map(|p| p.fill.clone()).collect();
        assert!(fills.iter().all(Option::is_some), "uma forma sem tinta");
        for i in 0..fills.len() {
            for j in i + 1..fills.len() {
                assert_ne!(fills[i], fills[j], "as formas {i} e {j} tem a mesma cor");
            }
        }
    }
}
