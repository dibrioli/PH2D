//! ⭐⭐ **AS PALAVRAS DOS MOTORES DE ÁUDIO** (`ph2d-audio-edit` · `ph2d-audio-encode`) — a fatia que
//! a população LARGA do `a_fronteira_dos_motores` trouxe.
//!
//! # Porque nenhuma régua as via
//!
//! As três crates são MOTORES: nenhuma das 30 réguas lexicais as varre, e a régua de porta
//! (`censo-texto-pintado.py`) segue o texto até um pintor **dentro da mesma crate** — e aqui quem
//! pinta é a `ph2d-app-audio`. Elas só apareceram quando o filtro de população do gate da fronteira
//! caiu de «depende da `ph2d-editor-core` ou da `ph2d-i18n`» (44 crates) para «toda crate que não é
//! painel nem família» (325).
//!
//! # ⛔⛔ E DUAS das três não eram um rótulo: eram um IDENTIFICADOR a fazer de rótulo
//!
//! Esta é a armadilha que esta fatia existe para nomear, e ela é **pior** do que o texto cru:
//!
//! - **`PickStrategy::name()`** era lido pelo selector `◀ nome ▶` do painel **e** pelo
//!   `from_name()`, que faz o caminho de volta a partir de um MANIFESTO. Traduzi-lo teria partido a
//!   leitura de todo ficheiro já gravado — em silêncio, porque `from_name` devolve `None` e o
//!   chamador cai no valor de omissão.
//! - **`Platform.name`** era o que a aba diz **e** o infixo do nome do ficheiro exportado
//!   (`{stem}.mobile.ogg`, por `p.name.to_lowercase()`). Traduzi-lo mudaria o nome dos ficheiros
//!   que o artista exporta conforme a língua da interface.
//!
//! ⇒ *quando um `&str` tem dois papéis, a cura não é traduzi-lo: é PARTI-LO.* O identificador fica
//! onde estava (com a razão escrita), e a palavra que o artista lê passa a ser uma chave.
//!
//! # ⚠️ A chave deriva da VARIANTE, nunca da palavra inglesa
//!
//! A forma forte, a mesma do `ecs_scene`: uma ponte que casasse por palavra daria ao `Console` da
//! plataforma a palavra de qualquer outro *Console* do app.
//!
//! # ⭐ E o motor NÃO ganha a `ph2d-i18n` como dependência
//!
//! Nenhuma destas três precisa do INGLÊS: quem precisa da palavra é o pintor, e ele já fala a
//! tabela. ⛔ Publicar um `label()` que fosse `tr_em(Ingles, label_key())` seria construir
//! exactamente o acessório que custou o report *«prefab e Image ainda errados»* — um pintor que o
//! chamasse ficava preso à língua de omissão, **e nenhum teste de igualdade o separaria** do
//! caminho certo num processo em inglês. *Aqui o motor publica só a CHAVE, e quem não tem a palavra
//! não pode pintá-la errada.*

/// A tradução de uma chave `audio.codec.*` / `audio.platform.*` / `audio.pick.*`, ou `None` se ela
/// não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        // ⚠️ **Os quatro nomes de CODEC são nomes de formato** e ficam assim em toda língua, como
        // `RGB` ou `UV`. A chave não existe para os traduzir: existe para que a única lista de
        // palavras da interface seja esta tabela — e para que a frase que os rodeia (o selector, o
        // aviso de perda) possa mudar de língua sem que ninguém tenha de procurar o literal dentro
        // de um motor de codificação.
        "audio.codec.wav16" => "WAV 16-bit",
        "audio.codec.wav24" => "WAV 24-bit",
        "audio.codec.ogg_vorbis" => "Ogg Vorbis",
        "audio.codec.opus" => "Opus",
        // As três plataformas de entrega. ⚠️ O nome do FICHEIRO exportado continua a sair do
        // `Platform::id`, que não é isto — ver o cabeçalho.
        "audio.platform.mobile" => "Mobile",
        "audio.platform.desktop" => "Desktop",
        "audio.platform.console" => "Console",
        // As três leis de escolha de uma variação. ⚠️ O manifesto continua a gravar o
        // `PickStrategy::name()`, que não é isto — ver o cabeçalho.
        "audio.pick.random" => "Random",
        "audio.pick.sequence" => "Sequence",
        "audio.pick.shuffle" => "Shuffle",
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
