//! ⭐⭐ **AS PALAVRAS DOS MOTORES DAS FERRAMENTAS** — a 8.ª fatia da fronteira dos motores.
//!
//! Quatro vocabulários que um painel pinta e uma crate `ph2d-tool-*` publica: a queda do pincel de
//! recorte de fundo, os dezasseis **looks** do equalizador de cor, os oito verbos de remodelar do
//! Flip e os quatro **meios** do Painter. Os quatro foram enumerados pela régua nova
//! (`ph2d_label_census::fronteira`), nunca por uma fotografia.
//!
//! # ⛔ O que NÃO entrou, e cada um com a medição
//!
//! - **Os dez nomes de ferramenta** (`Tool::label()`): o nome que o artista lê ao ESCOLHER uma
//!   ferramenta está no rail, com chave própria (`chrome.rail.*`) e uma segunda palavra abreviada
//!   para o chip. A `label()` chega a pixel em dois sítios, e nenhum é chrome do artista — a barra
//!   de **título** (uma linha de diagnóstico: `sprites=… | atlas=… | theme=… | tool=…`) e a paleta
//!   do caminho **legado sem-herói**. ⇒ traduzi-la poria uma SEGUNDA palavra por ferramenta na
//!   tabela, e *duas respostas à mesma pergunta divergem no dia em que uma mudar.*
//! - **`LutPreset::group()`** (5 palavras): **órfã provada** — só um `assert!` a lia. Apagada.
//! - **`ShapeGroup::label()`** do vector (7): órfã provada; quem pinta é o `group_i18n_key` do
//!   painel, chaveado pela VARIANTE.
//!
//! # ⚠️ A chave deriva do ID, nunca da palavra
//!
//! `tool.<id da ferramenta>.<família>.<variante>` — e o `<id>` é o `ToolId` que o produto já
//! persiste. É a lei que o `CLAUDE.md` §5 escreve para esta fronteira: *«ali a chave tem de ser
//! derivada do id»*.

/// A tradução de uma chave `tool.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "hud.fit.expand" => "Expand",
        "hud.fit.keep" => "Keep",
        "hud.fit.stretch" => "Stretch",
        "tool.bgremoval.falloff.constant" => "Hard",
        "tool.bgremoval.falloff.sharp" => "Sharp",
        "tool.bgremoval.falloff.smooth" => "Smooth",
        "tool.bgremoval.falloff.sphere" => "Sphere",
        "tool.color_equalization.preset.bleach_bypass" => "Bleach Bypass",
        "tool.color_equalization.preset.blockbuster" => "Blockbuster",
        "tool.color_equalization.preset.cinematic" => "Cinematic",
        "tool.color_equalization.preset.cool" => "Cool",
        "tool.color_equalization.preset.cross_process" => "Cross Process",
        "tool.color_equalization.preset.faded_film" => "Faded Film",
        "tool.color_equalization.preset.film_noir" => "Film Noir",
        "tool.color_equalization.preset.golden_hour" => "Golden Hour",
        "tool.color_equalization.preset.matte" => "Matte",
        "tool.color_equalization.preset.moonlight" => "Moonlight",
        // ⚠️ **`None` aqui é *«nenhum look»*, e não a ausência de uma opção** — ele é a primeira
        // entrada do selector e o valor de fábrica.
        "tool.color_equalization.preset.none" => "None",
        "tool.color_equalization.preset.polaroid" => "Polaroid",
        "tool.color_equalization.preset.sepia" => "Sepia",
        "tool.color_equalization.preset.vibrant" => "Vibrant",
        "tool.color_equalization.preset.vintage" => "Vintage",
        "tool.color_equalization.preset.warm" => "Warm",
        "tool.flip.reshape.grab" => "Grab",
        "tool.flip.reshape.pinch" => "Pinch",
        "tool.flip.reshape.push" => "Push",
        // ⚠️ **`Jitter` e não `Randomize`**: o rótulo diz o EFEITO que o artista vê no traço, e a
        // variante do enum (`Randomize`) diz o mecanismo. *A chave deriva da variante; o texto não.*
        "tool.flip.reshape.randomize" => "Jitter",
        "tool.flip.reshape.smooth" => "Smooth",
        "tool.flip.reshape.strength" => "Strength",
        // ⚠️ **`Thicken` e não `Thickness`** — é um verbo, como os sete irmãos dele na fileira.
        "tool.flip.reshape.thickness" => "Thicken",
        "tool.flip.reshape.twist" => "Twist",
        // Os quatro MEIOS do Painter — o dropdown que substituiu três caixas de marcar em
        // 2026-07-22, por ordem do dono.
        "tool.painter.media.digital" => "Digital",
        "tool.painter.media.impasto" => "Impasto",
        "tool.painter.media.watercolor" => "Watercolor",
        "tool.painter.media.wet_paint" => "Wet Paint",
        "tween.ao_acabar.hold" => "Hold",
        "tween.ao_acabar.rewind" => "Rewind",
        "tween.canal.opacity" => "Opacity",
        "tween.canal.position_x" => "Position X",
        "tween.canal.position_y" => "Position Y",
        "tween.canal.rotation" => "Rotation",
        "tween.canal.scale_x" => "Scale X",
        "tween.canal.scale_y" => "Scale Y",
        "tween.canal.silhueta" => "Silhouette",
        "tween.canal.tint" => "Tint",
        "tween.ciclo.ping_pong" => "Ping-Pong",
        "tween.ciclo.reinicia" => "Restart",
        "shake.perfil.explosion" => "Explosion",
        "shake.perfil.impact" => "Impact",
        "shake.perfil.recoil" => "Recoil",
        "tween.preset.fade_in" => "Fade In",
        "tween.preset.fade_out" => "Fade Out",
        "tween.preset.flash" => "Flash",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
