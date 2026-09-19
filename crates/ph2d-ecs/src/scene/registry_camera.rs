//! **O registo da família da CÂMERA** — irmão do [`super::registry`] por CAP de LOC.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e não por tamanho:** o irmão bateu `702` de `700` ao ganhar
//! os dois componentes do ABANÃO (suplente #25), e esta é **exactamente** a fronteira que o catálogo
//! de descritores já usa — a categoria `ComponentCategory::Camera`, com os mesmos cinco tipos.
//! ⛔ **Curado por CORTE, nunca por uma entrada no `FILE_OVERAGE_OK`** — aquela lista está VAZIA.
//!
//! ⚠️ **A ORDEM do registo não muda nada** (o registo é um mapa por nome, não uma escada), mas o
//! bloco saiu **verbatim e na mesma posição** de propósito: um corte que reordene o que ele move
//! obriga a próxima pessoa a provar que a ordem não importava.

use super::ComponentRegistry;

/// Os cinco tipos da família — ver o cabeçalho.
pub(super) fn register_camera(reg: &mut ComponentRegistry) {
    // ⭐⭐⭐ **A CÂMERA DE JOGO** (TOP-20 #7). Os TRÊS são CONFIG e gravam-se; o centro que ela
    // ocupa agora é o `CameraRuntime`, que **não deriva `Serialize` e por isso não cabe aqui** —
    // a cerca é do TIPO, como no `TimerRuntime`. ⛔ Registá-lo faria cada quadro com clique, numa
    // cena a seguir o jogador, virar um passo de undo.
    reg.register_default::<crate::GameCamera>("ph2d::ecs::GameCamera");
    // ⭐⭐⭐ **O ABANÃO DA VISTA** (suplente #25, 2026-09-19) — os dois são CONFIG inteira: *como*
    // esta câmera treme, e *ao ouvir o quê* aquela bomba a abana. Sem o registo, o artista afina o
    // estrondo, grava, reabre, e a cena volta PARADA — nada some da tela e nada dá erro.
    //
    // ⛔⛔ **O `CameraShakeRuntime` NÃO está aqui**, e a porta fecha-se pelo **TIPO** (ele não
    // deriva `Serialize`, logo a linha nem compila — o precedente do `TimerRuntime`). Registá-lo
    // poria **cada quadro de um abanão** dentro do ficheiro e um passo na pilha de `Ctrl+Z`. Ele
    // entra é no `rewind_runtime`, que é a outra metade da mesma lei.
    reg.register_default::<crate::CameraShake>("ph2d::ecs::CameraShake");
    reg.register_default::<crate::ShakeEmitter>("ph2d::ecs::ShakeEmitter");
    // ⚠️ **Separado da câmera de propósito**: um objecto pode ter câmera sem seguir ninguém (a
    // fixa de uma sala), e é a ausência do componente que o diz — não um campo `enabled` a mais.
    reg.register_default::<crate::CameraFollow>("ph2d::ecs::CameraFollow");
    reg.register_default::<crate::CameraLimits>("ph2d::ecs::CameraLimits");
}
