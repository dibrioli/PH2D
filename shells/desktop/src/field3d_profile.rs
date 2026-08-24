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
    let Some(id) = closed.first() else {
        // ⚠️ Não devia acontecer — o botão só é oferecido com contorno escolhido —, mas dizê-lo é
        // mais barato do que um `expect` que derruba o app se a costura se soltar.
        return "Draw and select a closed shape first".to_string();
    };
    let Some(path) = scene.paths().iter().find(|p| p.id == *id) else {
        return "The selected shape is no longer in the scene".to_string();
    };
    let profile = match ph2d_field_profile::cook_path_auto(path) {
        Ok(p) => p,
        // ⚠️ **O erro do documento é dito, não engolido** — e **traduzido**: o artista tem de saber
        // o que fazer, e `Rejected(SelfIntersecting)` não lhe diz isso. *As frases são para o
        // artista, nunca o nome da variante* — a mesma lei do `field3d_notice`.
        Err(e) => return explain(&e),
    };
    let extent = extent_of(&profile);
    let prim = match which {
        ProfileShape::Extrude => Primitive::Extrude {
            profile,
            half_height: HEIGHT_OF_EXTENT * extent * 0.5,
            // ⚠️ **Aro vivo por omissão.** O filete do aro é uma linha do painel; o das quinas
            // **verticais** já veio do editor vetorial. *Uma quina, um dono* — e inventar um raio
            // aqui seria dar ao módulo uma opinião sobre um desenho que não é dele.
            round: 0.0,
        },
        ProfileShape::Revolve => Primitive::Revolve { profile },
    };
    let n = segments_of(&prim);
    // ⭐⭐ **O ID DO DESENHO VIAJA COM A FORMA** (W55) — é ele que vira o vínculo vivo, e sem ele a
    // peça nasceria já a ser uma fotografia do contorno.
    crate::field3d_smoke::ask_spawn_profile(prim, extent, *id);
    match which {
        ProfileShape::Extrude => format!("Extruded the shape ({n} edges)"),
        ProfileShape::Revolve => format!("Revolved the shape around Y ({n} edges)"),
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
