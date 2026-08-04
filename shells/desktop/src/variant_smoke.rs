//! **A cena dos VARIANTS** — `PH2D_BUILD_SMOKE=58` (plano UI/UX W5c).
//!
//! Módulo irmão do [`crate::component_pieces_smoke`] pelo assunto: ali prova-se *"e a cópia pode
//! discordar"*, aqui ***"e a cópia pode ser OUTRA VERSÃO"***.
//!
//! # A pergunta desta cena, e ela é de olho
//!
//! *Esta cópia é o botão pequeno em repouso — e escolher "Large" no painel a troca pelo botão
//! grande, sem apagar o estado "Idle" que eu tinha escolhido.*
//!
//! # O que a cena DÁ, e o que ela deliberadamente NÃO faz
//!
//! Ela monta o **conjunto**: quatro formas irmãs sob um mesmo pai, **nomeadas** na convenção do
//! Figma (`Size=Small, State=Idle`). O parentesco e os nomes são material — parentear é gesto de
//! Hierarquia e renomear é digitar quatro strings, e nenhum dos dois é a costura que esta wave
//! constrói.
//!
//! ⚠️ **Ela NÃO cria componente nenhum.** *Create Component* e *Place Instance* são os gestos que
//! a `=53` já prova, e repeti-los aqui é barato — um smoke que arma o estado por baixo do pano
//! pula justamente a costura que existe para provar. E aqui há uma razão a mais: **é o gesto de
//! marcar os QUATRO como mestres que faz deles um conjunto de variants**, então armá-lo escondia a
//! metade mais importante do desenho.
//!
//! # Os dois eixos têm consequências VISUAIS diferentes, de propósito
//!
//! `Size` muda a GEOMETRIA (pequeno × grande) e `State` muda a COR (azul × laranja) — assim cada
//! fileira do painel tem um efeito que o olho separa. Com os dois eixos mudando a mesma coisa, um
//! chip errado seria indistinguível do certo.

use ph2d_ecs::{Entity, Name};
use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// A MOLDURA do conjunto: o retângulo claro que segura as quatro versões (o *component set* do
/// Figma). Ela é uma forma comum — **não** um mestre.
const FRAME: [f64; 4] = [-5.4, -0.4, -1.2, 2.8];
/// As quatro versões: `(caixa, nome)`. ⚠️ Pequeno e grande **medem diferente**; Idle e Hover
/// **pintam diferente**.
const VARIANTS: [([f64; 4], &str); 4] = [
    ([-5.0, 1.6, -4.0, 2.2], "Size=Small, State=Idle"),
    ([-3.6, 1.4, -1.6, 2.4], "Size=Large, State=Idle"),
    ([-5.0, 0.0, -4.0, 0.6], "Size=Small, State=Hover"),
    ([-3.6, -0.2, -1.6, 0.8], "Size=Large, State=Hover"),
];
/// O AZUL de `Idle` e o LARANJA de `Hover`.
const IDLE_RGB: [u8; 3] = [58, 96, 168];
const HOVER_RGB: [u8; 3] = [214, 128, 58];
/// **O SOLITÁRIO**: um candidato a mestre SEM irmãos — a metade da AUSÊNCIA, na tela.
const LONE: [f64; 4] = [0.8, 1.0, 2.2, 2.0];
/// O CONTROLE: um quadrado solto, longe, que nunca participa de nada.
const CONTROL: [f64; 4] = [0.8, -2.4, 1.6, -1.6];

fn tint(mut p: VecPath, rgb: [u8; 3]) -> VecPath {
    p.fill = Some(Paint::Solid(Rgba8::new(rgb[0], rgb[1], rgb[2], 255)));
    p
}

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        // O parentesco e os nomes só depois do `sync` — é ele que dá entidade a cada caminho.
        6 => adopt(app),
        7 => announce(app),
        _ => {}
    }
}

/// **As sete formas da cena** — porta única, e é o que torna os gates abaixo um ORÁCULO em vez de
/// um espelho: eles medem a geometria que a cena de facto empurra, e não as constantes.
fn paths() -> Vec<VecPath> {
    let mut out = vec![tint(
        rectangle([FRAME[0], FRAME[1]], [FRAME[2], FRAME[3]]),
        [46, 48, 56],
    )];
    for (r, name) in VARIANTS {
        let rgb = if name.contains("Hover") {
            HOVER_RGB
        } else {
            IDLE_RGB
        };
        out.push(tint(rectangle([r[0], r[1]], [r[2], r[3]]), rgb));
    }
    out.push(tint(
        rectangle([LONE[0], LONE[1]], [LONE[2], LONE[3]]),
        [82, 184, 120],
    ));
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

/// Pendura as quatro versões na moldura e dá o NOME de cada uma.
///
/// ⚠️ O nome é o endereço do eixo: sem ele o painel cai no modo de nomes crus e mostra UMA fileira
/// de `#id`, que é honesto e não demonstra nada.
fn adopt(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    if ids.len() < VARIANTS.len() + 1 {
        return;
    }
    let Some(&fb) = app.vec_entities.get(&ids[0]) else {
        return;
    };
    for (i, (_, name)) in VARIANTS.iter().enumerate() {
        let Some(&cb) = app.vec_entities.get(&ids[i + 1]) else {
            continue;
        };
        let (ce, fe) = (Entity::from_bits(cb), Entity::from_bits(fb));
        // ⚠️ Pela PORTA (`reparent_keeping_world`): o `settle_origins` já pôs cada forma-raiz a
        // carregar a própria translação, e um `ChildOf` cru SOMA as duas — as versões saltariam
        // para fora da moldura. O defeito está medido no `component_smoke`.
        crate::vec_transform::reparent_keeping_world(&mut gfx.sim, ce, fe);
        if let Ok(mut ent) = gfx.sim.world_mut().get_entity_mut(ce) {
            ent.insert(Name((*name).to_string()));
        }
    }
}

/// A mensagem — com os números MEDIDOS da própria cena, nunca de memória.
fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    eprintln!(
        "[variant] cena montada: {} formas — uma MOLDURA com {} versoes irmas ({}), um mestre \
         SOLITARIO e um controle. NENHUM componente criado.",
        gfx.vec_scene.paths().len(),
        VARIANTS.len(),
        VARIANTS.map(|(_, n)| n).join(" | ")
    );
    eprintln!("[variant] o roteiro (pegue a ferramenta VECTOR primeiro):");
    eprintln!("  1. Na Hierarquia, selecione CADA uma das quatro versoes dentro da moldura e");
    eprintln!("     carregue em **Create Component**. Quatro vezes. ⚠️ E' este gesto que faz");
    eprintln!("     delas um CONJUNTO: um conjunto de variants e' *os mestres irmaos*.");
    eprintln!("  2. Selecione a versao **Size=Small, State=Idle** e **Place Instance**. Arraste a");
    eprintln!("     copia para longe da moldura.");
    eprintln!("  3. ⚠️ **A PROVA DA WAVE**: com a copia selecionada, a secao Component mostra");
    eprintln!("     agora DUAS fileiras de chips — **Size** (Small|Large) e **State**");
    eprintln!("     (Idle|Hover) —, com o chip vigente aceso em cada uma.");
    eprintln!("  4. Carregue em **Large**: a copia vira o botao GRANDE e continua AZUL. O chip de");
    eprintln!("     State nao se mexeu — escolher um eixo nao apaga o outro.");
    eprintln!(
        "  5. Carregue em **Hover**: ela fica grande e LARANJA. Volte a **Small**: pequena e"
    );
    eprintln!("     laranja. As quatro combinacoes sao alcancaveis, e a moldura nao se mexe.");
    eprintln!("  6. ⚠️ **A metade da AUSENCIA**: selecione o retangulo VERDE solto a' direita e");
    eprintln!("     **Create Component**, depois **Place Instance**. Essa copia **nao mostra");
    eprintln!(
        "     fileira nenhuma** — um mestre sem irmaos nao tem versoes, e uma fileira com um"
    );
    eprintln!("     chip so' seria uma escolha que nao escolhe.");
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

    /// **Os quatro nomes declaram as MESMAS duas propriedades** — sem isso o painel cai no modo de
    /// nomes crus e a cena demonstra a fileira errada.
    #[test]
    fn the_four_names_declare_the_same_two_axes() {
        for (_, name) in VARIANTS {
            let combo = crate::vec_variants::parse_combo(name)
                .unwrap_or_else(|| panic!("`{name}` nao e' uma combinacao"));
            let keys: Vec<&str> = combo.iter().map(|(k, _)| k.as_str()).collect();
            assert_eq!(keys, ["Size", "State"], "os eixos de `{name}` divergem");
        }
    }

    /// **A matriz é COMPLETA** — com um buraco, um chip legítimo não seria oferecido e o passo 5
    /// do roteiro falharia sobre um produto correto.
    #[test]
    fn every_combination_exists() {
        for size in ["Small", "Large"] {
            for state in ["Idle", "Hover"] {
                assert!(
                    VARIANTS
                        .iter()
                        .any(|(_, n)| n.contains(size) && n.contains(state)),
                    "falta a versao {size}/{state}"
                );
            }
        }
    }

    /// **`Size` muda a GEOMETRIA** — se as quatro medissem igual, trocar de tamanho seria
    /// invisível e o smoke não poderia julgar nada.
    #[test]
    fn the_size_axis_is_visible_as_geometry() {
        let w = |i: usize| bbox(i + 1).1[0] - bbox(i + 1).0[0];
        assert!(
            w(1) > w(0) * 1.5 && w(3) > w(2) * 1.5,
            "as versoes Large nao sao visivelmente maiores: {:?}",
            [w(0), w(1), w(2), w(3)]
        );
    }

    /// **`State` muda a COR** — o segundo eixo precisa de uma consequência PRÓPRIA.
    #[test]
    fn the_state_axis_is_visible_as_colour() {
        let fill = |i: usize| paths()[i + 1].fill.clone();
        assert_eq!(fill(0), fill(1), "as duas Idle tem de partilhar a cor");
        assert_eq!(fill(2), fill(3), "as duas Hover tem de partilhar a cor");
        assert_ne!(fill(0), fill(2), "Idle e Hover pintam igual");
    }

    /// **As quatro cabem DENTRO da moldura** — uma versão que transborda lê como forma solta, e o
    /// artista não veria o conjunto.
    #[test]
    fn every_variant_sits_inside_the_frame() {
        let (flo, fhi) = bbox(0);
        for i in 0..VARIANTS.len() {
            let (lo, hi) = bbox(i + 1);
            for a in 0..2 {
                assert!(
                    lo[a] >= flo[a] && hi[a] <= fhi[a],
                    "a versao {i} sai da moldura no eixo {a}: {lo:?}..{hi:?}"
                );
            }
        }
    }

    /// **O solitário e o controle estão FORA da moldura** — o primeiro é a metade da ausência (um
    /// mestre sem irmãos), e ele só a demonstra se não for irmão de ninguém.
    #[test]
    fn the_lone_master_and_the_control_are_outside_the_frame() {
        let (_, fhi) = bbox(0);
        for i in [VARIANTS.len() + 1, VARIANTS.len() + 2] {
            let (lo, _) = bbox(i);
            assert!(lo[0] > fhi[0], "a forma {i} encosta na moldura");
        }
    }
}
