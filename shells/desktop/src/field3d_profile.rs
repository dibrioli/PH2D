//! ⭐⭐ **O PERFIL DESENHADO VIRA PEÇA** (W53) — o fluxo do MoI, e a ponte que faltava.
//!
//! # O buraco, e o tamanho dele
//!
//! `Primitive::Extrude` e `Primitive::Revolve` existem no motor **desde a W3**, medidos contra
//! oráculos independentes (um `n`-gono extrudado *é* o `Cylinder` analítico; um revolvido *é* o
//! `Torus`), com o arredondamento das quinas verticais a vir do *corner widget* do editor vetorial.
//! O plano do módulo chama-lhes a razão de existir: *"é aqui que o fluxo do MoI renasce, com a
//! caneta que a casa já tem"*.
//!
//! ⛔ **E nenhum botão os alcançava.** Só as cenas de smoke os construíam. É a lei da W34 — *o painel
//! oferece exatamente o que o gesto faz* — na maior escala em que este módulo a pagou: não um
//! controle mudo, uma **família de features** inteira, completa e invisível.
//!
//! ⚠️ **E o gate da alcançabilidade não a apanhava**, por uma razão escrita: a tabela `ROWS` cobre só
//! as fileiras que **dependem da seleção**, e as formas (`adds`) foram deixadas de fora como *"ações
//! sempre disponíveis"*. A pergunta que faltava é outra — *toda forma que o MOTOR sabe fazer tem
//! botão?* —, e ela tem gate agora.
//!
//! # A ponte já existia inteira
//!
//! `ph2d_field_profile::cook_path_auto(&VecPath) -> Profile` faz a travessia toda, incluindo as
//! quinas vivas. Esta wave não escreveu geometria nenhuma: escreveu o **gesto**.

use ph2d_field::{Primitive, Profile};
use ph2d_vec_scene::{VecPathId, VecScene};

use crate::field3d_smoke::ProfileShape;

/// A altura de uma extrusão, em **frações da extensão do próprio contorno**.
///
/// ⚠️ **Do contorno, e não do enquadramento**: a espessura de uma peça extrudada é uma proporção da
/// forma dela (uma cantoneira de 10 cm não tem 3 m de espessura), e é isso que faz o resultado ser
/// utilizável sem tocar num número. O tamanho de **convivência** — o quanto ela ocupa na cena — sai
/// da pose, como na escultura.
const HEIGHT_OF_EXTENT: f32 = 0.35;

/// ⭐ **Coze o contorno escolhido e pendura a forma** — a mensagem é o que o artista lê.
///
/// ⚠️ Ela **não** toca o mundo: o terceiro salto é o da ponte com a cena, que é quem o tem. Mesma
/// forma (e mesma razão) da escultura importada.
pub(crate) fn from_selection(
    scene: &VecScene,
    closed: &[VecPathId],
    which: ProfileShape,
) -> String {
    if closed.is_empty() {
        // ⚠️ Não devia acontecer — o botão só é oferecido com contorno escolhido —, mas dizê-lo é
        // mais barato do que um `expect` que derruba o app se a costura se soltar.
        return "Draw and select a closed shape first".to_string();
    }
    // ⭐⭐⭐ **TODA forma escolhida vira peça** (W74) — e não só a primeira.
    //
    // ⛔ **O que isto fecha era MUDO:** com duas formas escolhidas, esta função cozia a primeira e
    // **ignorava** o resto sem uma palavra; e mesmo que cozesse as duas, a caixa de correio era um
    // slot e a segunda apagava a primeira. *Duas perdas silenciosas em série, e o artista via uma
    // peça e nenhuma explicação.*
    //
    // ⚠️ **Uma peça por forma, e não uma peça com todas** — a diferença é o **vínculo vivo**: o
    // `FieldProfileSource` aponta para **um** desenho, então uma peça de N contornos ou perdia o
    // vínculo de N−1 deles ou obrigava o componente (que viaja no arquivo) a mudar de forma. Com
    // uma peça por forma, **todas** continuam a seguir o desenho delas, e juntá-las numa só é a
    // booleana que o módulo já tem. *A composição já exprime «uma peça»; ela não exprimia «o resto
    // existe».*
    let mut made = 0usize;
    let mut edges = 0usize;
    let mut gone = 0usize;
    let mut refused: Option<String> = None;
    for id in closed {
        let Some(path) = scene.paths().iter().find(|p| p.id == *id) else {
            gone += 1;
            continue;
        };
        let profile = match ph2d_field_profile::cook_path_auto(path) {
            Ok(p) => p,
            // ⚠️ **O erro do documento é dito, não engolido** — e **traduzido**: o artista tem de
            // saber o que fazer, e `Rejected(SelfIntersecting)` não lhe diz isso. *As frases são
            // para o artista, nunca o nome da variante* — a mesma lei do `field3d_notice`.
            Err(e) => {
                refused.get_or_insert_with(|| explain(&e));
                continue;
            }
        };
        let extent = extent_of(&profile);
        let prim = match which {
            ProfileShape::Extrude => Primitive::Extrude {
                profile,
                half_height: HEIGHT_OF_EXTENT * extent * 0.5,
                // ⚠️ **Aro vivo por omissão.** O filete do aro é uma linha do painel; o das quinas
                // **verticais** já veio do editor vetorial. *Uma quina, um dono* — e inventar um
                // raio aqui seria dar ao módulo uma opinião sobre um desenho que não é dele.
                round: 0.0,
            },
            ProfileShape::Revolve => Primitive::Revolve { profile },
        };
        edges += segments_of(&prim);
        // ⭐⭐ **O ID DO DESENHO VIAJA COM A FORMA** (W55) — é ele que vira o vínculo vivo, e sem
        // ele a peça nasceria já a ser uma fotografia do contorno.
        crate::field3d_smoke::ask_spawn_profile(prim, extent, *id);
        made += 1;
    }
    if made == 0 {
        // ⚠️ **A recusa do documento ganha à ausência**: ela diz ao artista o que corrigir, e
        // «já não está na cena» é a frase de quem apagou o desenho entre o clique e o quadro.
        return refused
            .unwrap_or_else(|| "The selected shape is no longer in the scene".to_string());
    }
    let verb = match which {
        ProfileShape::Extrude => "Extruded",
        ProfileShape::Revolve => "Revolved",
    };
    // ⚠️ **O singular é o texto de sempre** — ele é o caso normal, e mudá-lo para «1 shape» tornaria
    // a mensagem de toda a gente mais fria para servir a excepção.
    let axis = match which {
        ProfileShape::Extrude => "",
        ProfileShape::Revolve => " around Y",
    };
    let head = if made == 1 {
        format!("{verb} the shape{axis} ({edges} edges)")
    } else {
        format!("{verb} {made} shapes{axis} ({edges} edges)")
    };
    // ⭐ **E o que ficou de fora é DITO** — foi a ausência desta frase que fez o defeito ser mudo.
    match (gone, refused) {
        (0, None) => head,
        (0, Some(why)) => format!("{head}. One was skipped: {why}"),
        (n, None) => format!("{head}. {n} were no longer in the scene"),
        (n, Some(why)) => format!("{head}. {n} were gone, and one was skipped: {why}"),
    }
}

/// ⭐ **O erro, na língua do artista.**
///
/// ⚠️ Ela existe pela lei que o `field3d_notice` já carrega: *"as frases dizem o que está errado na
/// peça — nunca o nome da variante, do campo ou do nó. Um `Rejected(SelfIntersecting)` no ecrã é a
/// mesma coisa que silêncio para quem está a modelar."*
pub(crate) fn explain(e: &ph2d_field_profile::CookError) -> String {
    use ph2d_field_profile::CookError as C;
    match e {
        C::OpenContour { .. } => "This shape is open — close it before making it solid".to_string(),
        C::Empty => "This shape has no points".to_string(),
        C::Rejected(_) => {
            "This outline cannot become a solid — it may cross itself or be too small".to_string()
        }
    }
}

/// A maior dimensão da caixa do perfil — a régua da escala, como na escultura.
fn extent_of(p: &Profile) -> f32 {
    let (lo, hi) = p.bounds();
    (hi[0] - lo[0]).max(hi[1] - lo[1]).max(f32::EPSILON)
}

fn segments_of(p: &Primitive) -> usize {
    match p {
        Primitive::Extrude { profile, .. } | Primitive::Revolve { profile } => {
            profile.segment_count()
        }
        _ => 0,
    }
}

#[cfg(test)]
#[path = "field3d_profile_tests.rs"]
mod tests;
