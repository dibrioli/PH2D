//! ⭐⭐⭐ **O vocabulário do BRILHO da cena 3D** (`docs/Render3d/12`, a `W7`).
//!
//! # Porque é um ficheiro irmão, e não mais entradas no [`super::model3d_render`]
//!
//! ⛔ **Corte por responsabilidade, antecipando o tecto de `700` linhas** — aquele ficheiro estava a
//! `542` e esta secção traz **onze** fileiras mais as razões e as dicas. *A hora de cortar é antes
//! de a catraca reprovar, não depois* (`CLAUDE.md` §5.0: a cura é o corte, **nunca** uma entrada
//! nova no `FILE_OVERAGE_OK`).
//!
//! ⚠️ E o corte é honesto como assunto: o irmão nomeia a gestão de cor e a direcção de arte **por
//! pixel**; isto nomeia um **passe** que lê a vizinhança.

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ⚠️ **"Bloom" e não "Glow"**: é o nome que o Blender, o Unreal e o Godot dão a este passe.
        // ⛔ *Glow* já tem dono neste app (o `fx.glow` dos nós de movimento), e duas coisas com o
        // mesmo nome a fazer coisas diferentes é a forma mais barata de um artista pedir a errada.
        "panel.model3d.section.bloom" => "Bloom",
        "panel.model3d.bloom.enabled" => "Bloom",
        "panel.model3d.bloom.off" => "Off",
        "panel.model3d.bloom.on" => "On",
        // ⚠️ **"Threshold" é a luminância de PICO**, e a dica di-lo: um vermelho saturado estoura
        // tanto quanto um branco da mesma força.
        "panel.model3d.bloom.threshold" => "Threshold",
        // ⚠️ **"Knee" é o vocabulário de COMPRESSÃO** (um compressor de áudio, um tonemapper), e é
        // exactamente o que ele faz: arredonda o canto entre «não passa» e «passa».
        "panel.model3d.bloom.knee" => "Knee",
        "panel.model3d.bloom.intensity" => "Intensity",
        // ⚠️ **Os níveis contam-se de `1`, como no alvo** — a arrumação indexa de `0` e o artista
        // lê `1..7`. *Um painel que mostrasse `0..6` obrigaria a traduzir de cabeça todo tutorial.*
        "panel.model3d.bloom.level_1" => "Size 1 (finest)",
        "panel.model3d.bloom.level_2" => "Size 2",
        "panel.model3d.bloom.level_3" => "Size 3",
        "panel.model3d.bloom.level_4" => "Size 4",
        "panel.model3d.bloom.level_5" => "Size 5",
        "panel.model3d.bloom.level_6" => "Size 6",
        "panel.model3d.bloom.level_7" => "Size 7 (widest)",

        // ⭐⭐ **AS DICAS** — e cada uma diz o que MEDIÇÃO deu, não o que a lei é.
        "panel.model3d.bloom.threshold.tip" => {
            "Light brighter than this glows. It reads the brightest channel, so a saturated red \
             blooms as much as a white of the same strength."
        }
        "panel.model3d.bloom.knee.tip" => {
            "Softens the edge between not glowing and glowing. The transition is twice this wide, \
             so at Knee = Threshold it reaches all the way down to black."
        }
        "panel.model3d.bloom.intensity.tip" => {
            "How much of the halo comes back into the image. It changes the strength, not the \
             size \u{2014} the sizes below do that."
        }
        "panel.model3d.bloom.level_1.tip" => {
            "How much of each halo size to mix in. Each size is twice as wide as the one above it."
        }

        // ⭐⭐⭐ **AS TRÊS RAZÕES de uma fileira apagada**, da mais geral para a mais específica.
        //
        // ⚠️ **A do MOTOR vem primeiro**, e é a que existe hoje na configuração de fábrica: o passe
        // corre na cauda do sombreamento de CPU e o caminho de omissão deste módulo é o
        // dispositivo. *Dizer «o brilho está desligado» a quem também está no caminho errado é
        // mandá-lo resolver a metade errada.*
        "field.inert.bloom_runs_on_the_reference_path" => {
            "Inactive: bloom is drawn by the reference renderer, which this view is not using. \
             Start the app with PH2D_FIELD_GPU=0 to see it."
        }
        "field.inert.bloom_is_off" => "Inactive: bloom is off. Switch Bloom to On to use it.",
        "field.inert.bloom_threshold_is_zero" => {
            "Inactive: with Threshold at zero every light already glows, so there is no edge to \
             soften. Raise Threshold to use it."
        }
        _ => return None,
    })
}
