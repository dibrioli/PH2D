//! ⭐⭐ **O NOME do que um pincel percorre e do que uma estampa revela** — e ele **não é "Shape"**.
//!
//! # O defeito que este módulo existe para matar
//!
//! Os três botões que escolhem a arte diziam *"Pick Shape…"*, *"Change Shape…"* e *"Use Shape…"* —
//! e desde 2026-08-30 os dois modelos aceitam **um grupo** (`20881b0b0` na estampa, `59a80bd6e` no
//! pincel). O rótulo passou a prometer **menos** do que a porta aceita, e a forma de falhar é a
//! cara: o artista lê *"Shape"*, agrupa duas formas para as usar juntas, e conclui que não dá.
//!
//! ⚠️ **Um rótulo que promete menos que a porta não dá erro** — ele apaga a feature para quem o lê.
//! É a irmã da lei que a casa já paga do outro lado (*"um rótulo tem de prometer o que o modelo
//! entrega"*), com o sinal invertido.
//!
//! # Porque é um módulo e não três literais
//!
//! Eram **três** literais em **dois** ficheiros, e a palavra tem de ser a mesma nos dois: o pincel e
//! a estampa escolhem a arte pelo **mesmo gesto** (clicar numa forma do documento), e dois nomes
//! para um gesto são dois conceitos aos olhos de quem aprende. *Uma lei escrita em dois sítios ainda
//! não é uma lei — só uma PORTA é* (`stroke_uniform`, a mesma lição, na mesma crate-família).
//!
//! ⛔ O gate `the_art_pickers_speak_one_word` varre os dois painéis e recusa um literal novo.

/// **Escolher a arte pela primeira vez.** O pincel diz isto quando ainda não tem arte, e o rótulo
/// *é* o estado: um pincel sem arte pinta a cor de recurso, e sem esta diferença o artista não tem
/// como saber porquê.
pub(crate) const PICK: &str = "Pick Art...";

/// **Trocar a arte que já existe** — o outro braço do mesmo botão.
pub(crate) const CHANGE: &str = "Change Art...";

/// **A porta do canvas na estampa** (a irmã de *Source…*, que abre o diálogo de ficheiro). ⚠️ O
/// verbo é outro de propósito: ali existem DUAS portas de arte lado a lado, e *"Use"* separa
/// *«aponta uma forma que já está na tela»* de *«vai buscar um ficheiro»*.
pub(crate) const USE: &str = "Use Art...";
