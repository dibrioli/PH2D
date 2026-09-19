//! **PORQUE É QUE UMA FILEIRA DO PAINEL DE MODELAÇÃO ESTÁ APAGADA** — o irmão de assunto do
//! [`super::model3d`].
//!
//! Decisão do dono, 2026-09-18: as fileiras que o modo em mãos não lê ficam *«à vista, apagadas»*,
//! com a razão ao lado. Quem as carrega até aqui é o [`ph2d_field::Span::Locked`].
//!
//! # ⚠️ Porque é um arquivo e não mais sete linhas no irmão
//!
//! O `model3d.rs` estava a `695` de `700` quando estas chegaram. ⛔ *Split por assunto, nunca uma
//! entrada na allowlist* (`CLAUDE.md` §2) — e o assunto separa-se sozinho: as do irmão são **nomes
//! de coisas** (o que uma fileira É), estas são **frases para o artista** (porque é que ele não lhe
//! pode tocar AGORA, e o que fazer a seguir).
//!
//! ⚠️ **O corte compra o mesmo isolamento que o do irmão:** duas linhas paralelas que acrescentem
//! uma razão cada deixam de colidir nas mesmas linhas.

/// A tradução de uma chave `field.inert.*`, ou `None` se ela não é daqui.
///
/// # ⚠️ Cada uma nomeia o GESTO que a destranca, e não a condição interna
///
/// É a lei do `shape_palette::why_not`: *«escolha um contorno fechado»* é acionável, *«profile_pick
/// is none»* não é. ⛔ E nenhuma delas explica a FÍSICA — o artista quer saber o que fazer, não
/// como o modelo mistura os lóbulos.
///
/// # ⚠️ Começam todas por «Inactive:», de propósito
///
/// A fileira já está apagada, mas uma frase solta debaixo de um número lê-se como uma **dica sobre
/// o número** — e o que ela diz é que ele não faz nada agora. *A primeira palavra é a que separa as
/// duas leituras.*
///
/// ⭐ E são **seis** e não uma: o travamento existia desde 14/09 e era **mudo em todas** as
/// famílias — *um controlo travado sem razão à vista lê-se exactamente como um controlo morto* —, e
/// curar só a que o dono perguntou deixaria as outras cinco a mentir do mesmo modo.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "field.inert.metal_has_no_dielectric" => {
            "Inactive: a full metal has no diffuse layer. Lower Metalness to use it."
        }
        "field.inert.coat_is_off" => "Inactive: the coat is off. Raise Coat above zero to use it.",
        "field.inert.emission_is_off" => {
            "Inactive: the object emits no light. Raise Emission above zero to use it."
        }
        "field.inert.subsurface_is_off" => {
            "Inactive: subsurface is off. Raise Subsurface above zero to use it."
        }
        // ⚠️ **«how deep» e não «mean free path»**: o rótulo da fileira já diz *Radius*, e a frase
        // tem de dizer o que ele significa para a peça — uma casca não tem dentro por onde a luz
        // viaje.
        "field.inert.thin_wall_has_no_depth" => {
            "Inactive: a thin wall has no depth, so how deep light travels means nothing to it. \
             Set Thin Walled to Solid to use it."
        }
        // ⚠️ **«Switch Thin Walled on» e não «set Thin Walled to Thin Walled»**: a fileira e a
        // opção têm o mesmo nome, e a frase literal lê-se como um erro de escrita.
        "field.inert.solid_scatters_isotropically" => {
            "Inactive: a solid piece scatters light evenly in every direction, so a direction \
             means nothing to it. Switch Thin Walled on to use it."
        }
        // ⭐ E a trava de CARDAN, que era a primeira de todas e também era muda. ⚠️ Ela nomeia o
        // ângulo do **meio**, que é o único gesto que a destranca — dizer *«gimbal lock»* e parar
        // ali seria o nome do fenómeno em vez da saída dele.
        "field.inert.gimbal_axis" => {
            "Inactive: gimbal lock — this axis and the first one became the same turn. Move the \
             middle angle away from \u{b1}90\u{b0} to split them."
        }
        // ⭐⭐⭐ **AS QUATRO DA CAMADA DE ESTILO** (report do dono, 2026-09-19: *«Zone pivot não sei
        // para que serve mas parece morto»*). Medido: na configuração em que o painel ABRE, quatro
        // das fileiras dela são **inertes por construção** — e o cabeçalho do ficheiro que as
        // publica já citava esta lei sem a cumprir.
        //
        // ⚠️ Cada uma nomeia **o gesto que a destranca**, e o gesto é sempre *outra fileira desta
        // mesma secção* — que é precisamente o que o artista não consegue adivinhar sozinho.
        "field.inert.rim_is_off" => {
            "Inactive: the rim light is off. Raise Rim Strength above zero to use it."
        }
        "field.inert.no_edge_tint" => {
            "Inactive: edges are not tinted. Give Edge Tint a colour to use it."
        }
        "field.inert.no_cavity_tint" => {
            "Inactive: cavities are not tinted. Give Cavity Tint a colour to use it."
        }
        "field.inert.no_curvature_tint" => {
            "Inactive: nothing reads curvature yet. Give Edge Tint or Cavity Tint a colour to use it."
        }
        // ⚠️ **Esta é a resposta LITERAL ao report**: o pivô reparte entre DUAS tintas, e com as duas
        // brancas o parêntesis da lei é exactamente zero — *a mesma lei que faz a omissão ser a
        // identidade ao bit*. Armado, ele é o botão mais forte da camada.
        "field.inert.zones_are_the_same" => {
            "Inactive: both zone tints are the same colour, so there is nothing to split. Give \
             Shadow Tint or Highlight Tint a colour to use it."
        }
        _ => return None,
    })
}
