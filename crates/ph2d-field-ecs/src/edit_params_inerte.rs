//! ⭐⭐⭐ **PORQUE É QUE UM NÚMERO DO MATERIAL NÃO CHEGA AO PIXEL** — a lei que decide quais fileiras
//! o painel pinta APAGADAS, e com que razão ao lado.
//!
//! # ⛔⛔ Porque ela é um arquivo e não um fecho dentro do [`super::params_of`]
//!
//! Ela nasceu lá dentro como um `|k| -> bool` de cinco linhas (14/09) e cresceu duas vezes: ganhou
//! a **razão** e ganhou a **partição** (18/09). Duas coisas mudaram com isso — ela passou a ser uma
//! resposta que outros querem fazer **directamente** (o censo, sem montar um painel), e o
//! `edit_params.rs` passou dos `700`. ⛔ *Split por responsabilidade, nunca uma entrada na
//! allowlist* (`CLAUDE.md` §2).
//!
//! ⭐ E o corte é melhor do que o ficheiro era: a lei tem agora **endereço**, e um gate que a
//! afirme não tem de a alcançar através da pintura.
//!
//! # ⚠️ A fronteira: esta lei é de APRESENTAÇÃO, e a porta de ESCRITA não se estreita
//!
//! O [`super::super::set_param`] continua a aceitar as 33 posições. *Travar é da apresentação* — um
//! pedido guardado de um quadro atrás tem de poder aterrar, e um número que o artista escreveu com
//! o verniz ligado não pode evaporar por ele o ter desligado a seguir.

use crate::FieldMaterial;

/// ⭐⭐⭐ **A RAZÃO de a posição `k` deste material ser inerte** — `None` quando ela chega ao pixel.
///
/// O `&'static str` é a **chave i18n** que o painel pinta ao lado da fileira apagada (ver
/// [`ph2d_field::Span::Locked`], que a carrega até lá).
///
/// # ⚠️ O que cada condição significa
///
/// Todas medidas (`docs/Render3d/05` §20–§22 e o censo de 18/09), e as quatro primeiras com o mesmo
/// mecanismo: *o número é multiplicado por algo que é zero*.
///
/// | posições | inertes quando | porquê |
/// |---|---|---|
/// | `4`, `11` | `metalness == 1` | alimentam o lóbulo **dieléctrico**, que o metal mistura para fora |
/// | `13`–`18` | `coat == 0` | o `prepare` mistura os quatro do verniz pelo peso dele |
/// | `20`–`22` | `emission == 0` | a cor **multiplica** a luminância |
/// | `24`–`32` | `subsurface_weight == 0` | o `mix` do grafo deita fora o ramo inteiro |
/// | `27`–`30` | `thin_walled == 1` | uma parede fina **não tem profundidade** ⇒ um caminho livre médio não lhe diz nada |
/// | `31` | `thin_walled == 0` | o caminho maciço integra um perfil **isotrópico** ⇒ uma fase não lhe diz nada |
///
/// ⚠️ **E as nove da subsuperfície incluem a PAREDE FINA** (`32`): com o peso a zero, o caminho que
/// ela escolhe não é avaliado, logo o interruptor não move um pixel. *Um controlo que só faz sentido
/// depois de outro estar ligado é a mesma lei do verniz.*
///
/// # ⭐⭐⭐ As duas últimas são a PARTIÇÃO, e ela foi MEDIDA AO BIT
///
/// Pergunta do dono, 2026-09-18: *«SS Anisotropy está morto?»*. **Não está** — ele é do caminho da
/// parede fina, e no maciço não tem consumidor. ⭐ E o censo que a pergunta obrigou
/// (`ph2d_app_field3d::censo_dos_knobs_do_material_tests`) achou **cinco** e não um: cada modo lê
/// metade da família, ao contrário um do outro.
///
/// ⛔ **As duas metades são o porte fiel** do modelo publicado, logo isto não se cura no motor —
/// cura-se aqui, porque o painel mostrava a **UNIÃO** sobre uma lei que lê uma **PARTIÇÃO**.
///
/// ⚠️ **A lista não se escreve outra vez em lado nenhum:** o gate
/// `o_painel_apaga_exactamente_o_que_a_medicao_diz_estar_morto` compara o que esta função tranca
/// com o que a medição ao bit acha, **nos dois sentidos**. *Uma segunda lista seria a que
/// envelhece.*
///
/// # ⚠️ A ORDEM DOS BRAÇOS É A LEI, e não arrumação
///
/// Com o peso a zero **e** a parede fina ligada, as posições `27`–`30` têm duas razões verdadeiras.
/// Dizer *«uma parede fina não tem profundidade»* a quem também tem a subsuperfície desligada é
/// mandá-lo resolver a metade que **não** o destranca. ⇒ **o bloqueio mais externo fala primeiro**,
/// que é a mesma lei da `recusa::Entradas::recusa` na família do esculpir.
///
/// ⛔ **E nenhum braço responde «sim» sem uma razão**: a resposta **é** a chave, logo *travar em
/// silêncio deixou de ser exprimível*.
#[must_use]
pub(super) fn razao_inerte(m: &FieldMaterial, k: u8) -> Option<&'static str> {
    match k {
        4 | 11 if m.metalness >= 1.0 => Some("field.inert.metal_has_no_dielectric"),
        13..=18 if m.coat <= 0.0 => Some("field.inert.coat_is_off"),
        20..=22 if m.emission <= 0.0 => Some("field.inert.emission_is_off"),
        24..=32 if m.subsurface_weight <= 0.0 => Some("field.inert.subsurface_is_off"),
        27..=30 if m.thin_walled >= 0.5 => Some("field.inert.thin_wall_has_no_depth"),
        31 if m.thin_walled < 0.5 => Some("field.inert.solid_scatters_isotropically"),
        _ => None,
    }
}

#[cfg(test)]
#[path = "edit_params_inerte_tests.rs"]
mod tests;
