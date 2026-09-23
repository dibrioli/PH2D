//! **O registo da família da CÂMERA** — irmão do [`super::registry`] por CAP de LOC.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e não por tamanho:** o irmão bateu `702` de `700` ao ganhar
//! os dois componentes do ABANÃO (suplente #25), e esta é **exactamente** a fronteira que o catálogo
//! de descritores já usa — a categoria `ComponentCategory::Camera`, com os mesmos cinco tipos.
//! ⛔ **Curado por CORTE, nunca por uma entrada no `FILE_OVERAGE_OK`** — aquela lista está VAZIA.
//!
//! ⚠️ **A W1 da paralaxe acrescentou um SEXTO** (`ScrollFactor`): ele mora no objecto e não na
//! câmera, e é desta família porque o número dele é sobre o movimento DELA — a mesma fronteira que
//! o catálogo de descritores usa.
//!
//! ⚠️ **A ORDEM do registo não muda nada** (o registo é um mapa por nome, não uma escada), mas o
//! bloco saiu **verbatim e na mesma posição** de propósito: um corte que reordene o que ele move
//! obriga a próxima pessoa a provar que a ordem não importava.

use super::ComponentRegistry;

/// Os SEIS tipos da família — ver o cabeçalho.
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
    // ⭐⭐⭐ **A PARALAXE** (plano 24, W1) — ela mora no OBJECTO e não na câmera, e ainda assim é
    // desta família: o número inteiro dela quer dizer *«quanto do movimento da CÂMERA este objecto
    // guarda»*, e é ali que o artista o vai procurar.
    //
    // ⛔ **Não há `ScrollFactorRuntime`**, e a ausência é a lei: a pose deslocada é função PURA da
    // vista (`autorada + centro·(1 − k)`), logo não há um bit para guardar — um scrub e um
    // rebobinar reconstroem-na sozinhos. *O que não tem estado não pode sobreviver errado.*
    reg.register_default::<crate::ScrollFactor>("ph2d::ecs::ScrollFactor");
    // ⭐⭐ **A REPETIÇÃO** (plano 24, W2) — irmã do de cima e da mesma família pela mesma razão: o
    // número dela corrige o deslocamento que a CÂMERA impõe. ⛔ Ela é um componente SEPARADO e não
    // um campo do `ScrollFactor`, e a razão é a populacao: quase todo objecto com paralaxe **não**
    // repete (um primeiro plano, uma nuvem solta), e um campo a mais ali seria um knob morto em
    // todos eles — a mesma lei que separa o `CameraFollow` do `GameCamera`, três linhas acima.
    reg.register_default::<crate::ScrollRepeat>("ph2d::ecs::ScrollRepeat");
    // ⭐⭐ **O CONFINAMENTO** (plano 24, W3) — a terceira da família, e separada pela mesma razão
    // das outras duas: um fundo que repete não tem borda para esconder, e um primeiro plano não é
    // confinado por nada. *Três componentes porque são três populações.*
    reg.register_default::<crate::ScrollLimits>("ph2d::ecs::ScrollLimits");
    // ⭐⭐ **O MOVIMENTO PRÓPRIO** (plano 24, W4) — nuvens que andam sozinhas. ⛔ Sem runtime pela
    // razão do `ScrollFactor`: ele é `velocidade × playhead`, uma função PURA do relógio, logo um
    // scrub e um rebobinar reconstroem-na — e um acumulador daria uma nuvem a andar ao contrário
    // quando o artista puxasse a régua para trás.
    reg.register_default::<crate::ScrollMotion>("ph2d::ecs::ScrollMotion");
}
