//! **A cena da PELE POR-WIDGET** — `PH2D_BUILD_SMOKE=60` (plano UI/UX W6.2).
//!
//! # A pergunta desta cena é de olho, e ela é sobre INDISTINGUIBILIDADE
//!
//! *A forma que eu desenhei virou um controle de verdade — o MESMO que o app usa nos painéis
//! dele — e eu escolho qual, e o rótulo é o nome que eu dei ao objeto.*
//!
//! A cena põe **cinco retângulos vestidos** (Button · Toggle · Slider · Checkbox · Card), cada um
//! com o `Name` que vira o rótulo, e um **sexto NU** ao lado — que é o CONTROLE: ele tem de
//! continuar a ser um retângulo desenhado, porque marcar uma forma tem de ser a única coisa que a
//! transforma.
//!
//! ⚠️ O **Checkbox** entrou com uma moldura deliberadamente ALTA (BUGS_vector #26): ele e o
//! Slider eram os dois únicos tipos que não preenchiam a moldura, e como a pele é pintada em px
//! de TELA isso significava que **dar zoom crescia o retângulo e não crescia o widget**.
//!
//! ⚠️ **E ela imprime o número que a torna válida:** quantos tipos o catálogo oferece. Se for
//! zero, PARE — o catálogo não chegou, e o resto do roteiro não diz nada.

use ph2d_vec_scene::{Paint, Rgba8, VecPath, rectangle};

/// Os cinco retângulos: quatro vestidos, um nu.
///
/// ⚠️ Os tamanhos NÃO são iguais, de propósito: um Toggle e um Card querem molduras muito
/// diferentes, e uma cena que desse a mesma caixa a todos mostraria controles esticados e o
/// artista concluiria que a pele deforma.
const ART: [([f64; 4], &str, Option<&str>); 6] = [
    ([-4.6, 1.4, -2.2, 2.0], "Save", Some("Button")),
    ([-4.6, 0.3, -3.4, 0.8], "Snap", Some("Toggle")),
    ([-4.6, -0.9, -2.2, -0.4], "Opacity", Some("Slider")),
    // ⚠️ A moldura ALTA é o assunto do BUGS_vector #26, e a altura é escolhida para ser
    // inequívoca: sob a lei antiga a caixa media o token seja qual fosse a moldura, então uma
    // moldura pouco maior que a linha não distinguiria a correção do defeito.
    ([-4.6, -1.9, -2.2, -1.0], "Grid", Some("Checkbox")),
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
const FILLS: [[u8; 3]; 6] = [
    [92, 108, 148],
    [148, 110, 92],
    [92, 148, 118],
    [148, 140, 92],
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
    eprintln!("  7. Mova e escale uma forma vestida: o controle SEGUE a moldura — inclusive o");
    eprintln!("     CHECKBOX e o SLIDER, que eram os dois unicos que nao seguiam (BUGS #26).");
    eprintln!("     ⚠️ Compare a caixa do 'Grid' (moldura alta) com a de um checkbox do");
    eprintln!("     Inspector: a moldura e' uma CAIXA, e a pele a preenche.");
    eprintln!("  8. ⚠️ **O QUE NAO ESCALA, e e' decisao:** o RAIO DE CANTO, a espessura da borda");
    eprintln!("     e o corpo da letra continuam em px — eles sao TOKENS, e um token nao se mede");
    eprintln!("     em fracoes de moldura. Um widget grande e' um widget grande, nao uma foto");
    eprintln!("     ampliada de um pequeno.");
    eprintln!("  9. **O CONTROLE do #26**: reduza o 'Grid' ate' a altura de uma linha de painel.");
    eprintln!("     Ali a caixa tem de medir EXACTAMENTE o que mede no Inspector — e' o token a");
    eprintln!("     continuar a ser o token.");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A cena tem o que o passo 2 manda olhar** — vestidas, e EXACTAMENTE uma nua.
    ///
    /// ⚠️ A contagem das vestidas não é enumerada (ela cresce quando um tipo novo merece foto);
    /// o que é afirmado é o CONTROLE, que é único por desenho: duas formas nuas não provariam
    /// mais nada e uma zero apagaria a prova.
    #[test]
    fn the_scene_has_exactly_one_bare_control() {
        let dressed = ART.iter().filter(|(_, _, k)| k.is_some()).count();
        assert!(dressed >= 4, "a cena perdeu formas vestidas: {dressed}");
        assert_eq!(ART.len() - dressed, 1, "o CONTROLE sumiu da cena");
        assert_eq!(FILLS.len(), ART.len(), "uma forma ficou sem cor");
    }

    /// **Os dois tipos do BUGS #26 estão na foto, e o Checkbox numa moldura ALTA.**
    ///
    /// ⚠️ Sem esta asserção a cena pode perder exactamente o assunto que ela existe para julgar,
    /// e o roteiro passaria a falar de um controle que não está lá — o modo de falha que o gate
    /// irmão `every_kind_the_scene_names_exists` cobre na outra direção.
    #[test]
    fn the_two_kinds_that_did_not_fill_their_frame_are_on_stage() {
        for want in ["Checkbox", "Slider"] {
            let found = ART.iter().find(|(_, _, k)| *k == Some(want));
            assert!(
                found.is_some(),
                "a cena perdeu o {want} — o assunto do BUGS #26"
            );
        }
        let (r, _, _) = ART
            .iter()
            .find(|(_, _, k)| *k == Some("Checkbox"))
            .expect("o Checkbox esta' na cena");
        let (h, w) = (r[3] - r[1], r[2] - r[0]);
        assert!(
            h > 0.8,
            "a moldura do Checkbox ({h}) nao e' alta o bastante para distinguir a correcao do \
             defeito — sob a lei antiga a caixa media o token em QUALQUER moldura"
        );
        assert!(
            w > h,
            "a moldura do Checkbox ficou sem largura para o rotulo: {w} x {h}"
        );
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
