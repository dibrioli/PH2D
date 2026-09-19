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
        // ⭐⭐⭐ **O TAMANHO é UM número — o nosso modelo.** ⛔ A 1.ª redacção tinha sete
        // `Size 1..7` (o `glow_levels` do Godot), e o dono mandou-os sair: *«nosso bloom original é
        // muito melhor»*. ⚠️ **"Radius" é o nome que o nosso próprio `BloomParams` já lhe dá**, e o
        // artista que lê o halo do Motion lê a mesma palavra.
        "panel.model3d.bloom.radius" => "Radius",
        "panel.model3d.bloom.saturation" => "Saturation",
        "panel.model3d.bloom.tint" => "Tint",
        // ⚠️ **"Clamp" é o nome do Unity URP** para o mesmo antídoto — um tecto no que entra na
        // cadeia, para um pixel absurdo não lavar a tela.
        "panel.model3d.bloom.clamp" => "Clamp",
        // ⚠️ **"Anamorphic" e não "Stretch"**: é o nome do cinema e o do Unity, e o que ele faz é o
        // *streak* horizontal das lentes anamórficas.
        "panel.model3d.bloom.stretch" => "Anamorphic",
        "panel.model3d.bloom.angle" => "Anamorphic Angle",

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
        "panel.model3d.bloom.radius.tip" => {
            "How far the halo spreads. Below 1 it barely moves \u{2014} the chain already blurs \
             that much on its own; from 2 up it is the size knob."
        }
        "panel.model3d.bloom.saturation.tip" => {
            "0 pulls the halo to grey, 1 keeps the colour of the light that made it."
        }
        "panel.model3d.bloom.clamp.tip" => {
            "Caps how bright a single pixel may enter the halo. 0 is off; raise it when one stray \
             highlight washes the screen."
        }
        "panel.model3d.bloom.stretch.tip" => {
            "1 is a round halo. Above 1 it stretches along the angle below \u{2014} the streak an \
             anamorphic lens makes."
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
        "field.inert.bloom_is_round" => {
            "Inactive: the halo is round, so it has no direction to point. Raise Anamorphic above \
             1 to use it."
        }
        "field.inert.bloom_threshold_is_zero" => {
            "Inactive: with Threshold at zero every light already glows, so there is no edge to \
             soften. Raise Threshold to use it."
        }
        _ => return None,
    })
}
