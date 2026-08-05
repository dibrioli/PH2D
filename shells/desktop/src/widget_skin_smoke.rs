//! **A cena da PELE POR-WIDGET** — `PH2D_BUILD_SMOKE=60` (plano UI/UX W6.2).
//!
//! # A pergunta desta cena é de olho, e ela é sobre INDISTINGUIBILIDADE
//!
//! *A forma que eu desenhei virou um controle de verdade — o MESMO que o app usa nos painéis
//! dele — e eu escolho qual, e o rótulo é o nome que eu dei ao objeto.*
//!
//! A cena põe **quatro retângulos vestidos** (Button · Toggle · Slider · Card), cada um com o
//! `Name` que vira o rótulo, e um **quinto NU** ao lado — que é o CONTROLE: ele tem de continuar
//! a ser um retângulo desenhado, porque marcar uma forma tem de ser a única coisa que a
//! transforma.
//!
//! ⚠️ **E ela imprime o número que a torna válida:** quantos tipos o catálogo oferece. Se for
//! zero, PARE — o catálogo não chegou, e o resto do roteiro não diz nada.

use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// Os cinco retângulos: quatro vestidos, um nu.
///
/// ⚠️ Os tamanhos NÃO são iguais, de propósito: um Toggle e um Card querem molduras muito
/// diferentes, e uma cena que desse a mesma caixa a todos mostraria controles esticados e o
/// artista concluiria que a pele deforma.
const ART: [([f64; 4], &str, Option<&str>); 5] = [
    ([-4.6, 1.4, -2.2, 2.0], "Save", Some("Button")),
    ([-4.6, 0.3, -3.4, 0.8], "Snap", Some("Toggle")),
    ([-4.6, -0.9, -2.2, -0.4], "Opacity", Some("Slider")),
    ([-1.4, -0.9, 1.6, 2.0], "Inspector", Some("Card")),
    ([2.4, -0.9, 4.4, 2.0], "Just a rectangle", None),
];

pub(crate) fn frame(app: &mut crate::App, f: u32) {
    match f {
        3 => build(app),
        5 => dress(app),
        7 => announce(app),
        _ => {}
    }
}

/// As cores, na ordem do `ART`. A última é a NUA, e é por isso que ela é a mais saturada: o
/// controle tem de ser óbvio na foto.
const FILLS: [[u8; 3]; 5] = [
    [92, 108, 148],
    [148, 110, 92],
    [92, 148, 118],
    [120, 96, 148],
    [188, 96, 96],
];

fn build(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    for (i, (r, _, _)) in ART.iter().enumerate() {
        let mut p: VecPath = rectangle([r[0], r[1]], [r[2], r[3]]);
        // Uma cor visível: quem NÃO for vestido tem de continuar a mostrá-la, e é essa a prova.
        let c = FILLS[i];
        p.fill = Some(Paint::Solid(Rgba8::new(c[0], c[1], c[2], 255)));
        gfx.vec_scene.push_path(p);
    }
}

/// Dá o NOME a cada forma e veste as quatro primeiras.
///
/// ⚠️ Num frame POSTERIOR ao `build`, e é obrigatório: a entidade de uma forma nasce no
/// `vec_entities::sync`, que corre no frame do desenho. Marcar antes seria escrever num objeto
/// que ainda não existe — a mesma ordem que o `variant_smoke` já documenta.
fn dress(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_mut() else {
        return;
    };
    let ids: Vec<u64> = gfx.vec_scene.paths().iter().map(|p| p.id).collect();
    if ids.len() < ART.len() {
        return;
    }
    for (i, (_, name, kind)) in ART.iter().enumerate() {
        let Some(&bits) = app.vec_entities.get(&ids[i]) else {
            continue;
        };
        let Ok(mut ent) = gfx
            .sim
            .world_mut()
            .get_entity_mut(ph2d_ecs::Entity::from_bits(bits))
        else {
            continue;
        };
        ent.insert(ph2d_ecs::Name::new(*name));
        if let Some(k) = kind
            && let Some(kind) = ph2d_editor::widget::WidgetKind::ALL
                .iter()
                .find(|w| ph2d_i18n::tr(w.i18n_key()) == *k)
        {
            ent.insert(ph2d_ecs::VecWidget { kind: kind.code() });
        }
    }
}

fn announce(app: &mut crate::App) {
    let Some(gfx) = app.gfx.as_ref() else {
        return;
    };
    eprintln!(
        "[widget-skin] {} formas na cena, {} tipos no catalogo. As quatro primeiras VESTEM um \
         widget; a ultima e' o CONTROLE.",
        gfx.vec_scene.paths().len(),
        ph2d_editor::widget::WidgetKind::ALL.len()
    );
    eprintln!("[widget-skin] o roteiro:");
    eprintln!("  1. ⚠️ **A PROVA DA WAVE**: as quatro primeiras formas nao sao retangulos — sao");
    eprintln!("     um Button, um Toggle, um Slider e um Card, pintados pelo MESMO codigo que");
    eprintln!("     desenha os controles dos paineis a' volta. Compare com os do Inspector.");
    eprintln!("  2. ⚠️ **O CONTROLE**: a ultima forma (vermelha, a' direita) continua um");
    eprintln!("     retangulo desenhado. Marcar uma forma e' a UNICA coisa que a transforma.");
    eprintln!("  3. O ROTULO e' o NOME do objeto: renomeie 'Save' na Hierarquia e o botao muda");
    eprintln!("     de texto. Nao ha campo de rotulo, e a seção diz isso numa linha.");
    eprintln!("  4. Selecione a forma vermelha -> secao **Widget Skin** -> **Wear a Widget**:");
    eprintln!("     ela vira um Button. Os chips de tipo aparecem; escolha outro e ele troca.");
    eprintln!("  5. **Back to Drawing** devolve o retangulo — com a cor original intacta.");
    eprintln!("  6. ⚠️ **O TEMA**: aperte **M** para ciclar o tema. Os quatro controles do canvas");
    eprintln!("     mudam junto com os paineis — e' a ponte token->widget atravessando o canvas.");
    eprintln!("  7. Mova e escale uma forma vestida: o controle SEGUE a moldura. ⚠️ O zoom");
    eprintln!("     amplia a MOLDURA, nao os cantos — um widget e' autorado em px porque um");
    eprintln!("     token e' px, e isso e' decisao, nao defeito.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena tem o que o passo 2 manda olhar** — quatro vestidas e uma nua.
    #[test]
    fn the_scene_has_four_dressed_and_one_bare() {
        let dressed = ART.iter().filter(|(_, _, k)| k.is_some()).count();
        assert_eq!(dressed, 4);
        assert_eq!(ART.len() - dressed, 1, "o CONTROLE sumiu da cena");
    }

    /// **Todo tipo que a cena nomeia EXISTE no catálogo** — um nome errado deixaria a forma nua
    /// sem que nada dissesse por quê, e o roteiro falaria de um controle que não está lá.
    #[test]
    fn every_kind_the_scene_names_exists() {
        for (_, name, kind) in ART {
            let Some(k) = kind else { continue };
            assert!(
                ph2d_editor::widget::WidgetKind::ALL
                    .iter()
                    .any(|w| ph2d_i18n::tr(w.i18n_key()) == k),
                "a cena veste '{name}' com um tipo '{k}' que o catalogo nao tem"
            );
        }
    }
}
