//! **O RECORTE** — *"esta forma esconde o que sai dela?"*, e nada mais.
//!
//! Módulo irmão do [`crate::vec_frame`], e a separação é a decisão inteira. O bit vivia lá dentro
//! (`VecFrame { clip: bool }`), o que amarrava recortar a **ser uma moldura** — e uma moldura não
//! é só um contêiner que recorta: ela carrega o rótulo flutuante com o nome, as alças de
//! redimensionar, e a elegibilidade a auto layout e a âncoras (sete leitores de `VecFrame` em
//! produção, medidos em 2026-08-21).
//!
//! Enio: *"coloque a feature Clip Content para qualquer forma vetorial fechada"*. Pendurar o
//! `VecFrame` numa estrela para ela recortar teria dado à estrela um nome flutuante em cima e
//! alças de contêiner — o artista pediu um recorte e receberia uma moldura. Então as duas
//! perguntas passam a ter **componentes** diferentes, exactamente como o doc-comment do
//! `VecFrame` já dizia que elas eram: *"a pergunta «isto é um contêiner?» e a pergunta «ele
//! esconde o que sai?» são independentes"*. Elas eram independentes na prosa e acopladas no tipo.
//!
//! # Por que um MARCADOR, e não um `bool`
//!
//! Presença = recorta. O estado "não recorta" é a **ausência**, e é o que dá a propriedade que
//! esta crate persegue em todo componente novo: sem ele, o mundo é byte-idêntico ao de antes
//! desta feature — nenhuma entidade ganha um campo, nenhum documento ganha um byte, e o
//! `clip_spans` não produz intervalo nenhum. Um `bool` teria dois jeitos de dizer "não" (ausente,
//! ou presente-e-falso) e o undo os distinguiria sem que nada na tela os distinguisse.
//!
//! # O recorte é do desenhista que ENTENDE a forma
//!
//! ⚠️ Isto **não** é o [`crate::ClipChildren`]: aquele é o passe de stencil de **sprite** em
//! `ph2d-render`, e não alcança um caminho vetorial. Este é a camada de clip do Vello
//! (`VectorScene::push_clip_with_rule`), aberta pelo renderer sobre o intervalo contíguo de z que
//! a sub-árvore ocupa. Os dois recortes coexistem, cada um do renderer que o desenha.
//!
//! E é **por isso que a generalização é barata**: o recorte nunca soube que a moldura era um
//! retângulo. Ele já recortava pela silhueta do caminho, com a regra de preenchimento do próprio
//! caminho (`ph2d_vec_render::frame_clip`) — um card vazado já recortava pelo furo. Quem exigia
//! uma moldura era a **elegibilidade**, não o desenho.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// **Recorta os descendentes à silhueta desta forma.**
///
/// Vale para qualquer forma vetorial **fechada** — uma moldura, uma elipse, uma estrela. A
/// silhueta que recorta é a que a forma DESENHA (a derivada viva, se houver), então um caminho
/// com efeitos vivos recorta pelo que se vê, e não pela fonte.
///
/// ⚠️ **Fechada é requisito de PRODUTO, e ele mora em quem OFERECE o controlo**, não aqui: um
/// caminho aberto não tem interior, então "o que está dentro" não quer dizer nada e o Vello
/// recortaria por uma região que o artista não desenhou. A regra é aplicada na fronteira de
/// autoria (`vec_clip_edit`), pelo mesmo predicado `VecPath::closed` que a booleana viva e o
/// blend já usam para recusar caminhos abertos.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VecClipContent;

impl SimComponent for VecClipContent {}
