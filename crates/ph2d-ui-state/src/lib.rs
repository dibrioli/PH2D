//! **O SMART ANIMATE** — dados dois estados de uma cena, que pose ela tem no meio
//! (plano UI/UX §4 W7).
//!
//! # O que esta crate é, e o que ela deliberadamente não é
//!
//! Um botão tem *idle / hover / press / disabled*; um menu abre; um card expande. O que faz isso
//! parecer produto e não uma troca de imagem é o **tween automático entre os estados** — o artista
//! autora as duas pontas e o motor descobre o meio.
//!
//! ⚠️ **Não há relógio aqui.** [`Transition::at`] recebe um `t` e a [`Machine`] recebe um `dt`;
//! nenhuma das duas o conta. O relógio é o
//! `Playhead`, e a razão está medida noutro módulo deste repo: o `MotionTransport` do Motion
//! **morreu** na W4.T7 porque dois relógios divergem, e o modo de falha é a UI a andar noutra
//! velocidade que a cena.
//!
//! ⚠️ **Não há ECS e não há UI.** A pose é feita de números simples; quem a lê do mundo e quem a
//! escreve de volta é a shell. Foi isso que manteve esta crate testável de cabeça para baixo, sem
//! janela e sem GPU.
//!
//! ⚠️ **E não há motor próprio de forma nem de cor.** A geometria interpola pelo `ph2d-vec-blend`
//! — o mesmo Hungarian + espiral logarítmica que o Blend e o Morph do artista já usam — e a tinta
//! pela porta OKLab da mesma crate. Um segundo motor divergiria daquele, e a divergência só
//! apareceria numa screenshot.
//!
//! # O estado tem um PAPEL, não um nome — e é isso que dispensa a tabela de gatilhos
//!
//! Um estado é *Default / Hover / Pressed / Disabled* ([`StateRole`]). Com um nome livre alguém
//! teria de autorar **uma segunda tabela** — *"quando o rato entra, vá para o estado chamado
//! assim"* — e mantê-la em dia com a lista; com o papel, o gatilho é **derivado**, e a shell
//! chama [`Machine::go_to_role`] com o que aconteceu.
//!
//! ⚠️ **A lista de papéis é OPCIONAL:** um papel que ninguém gravou recua para o `Default`, então
//! autorar só o Hover não deixa o botão preso ao ser apertado. O que isto deliberadamente **não**
//! é: um grafo de estados com transições autoradas (a *state machine* do Rive) — aquilo tem
//! condições, entradas nomeadas e um editor próprio.
//!
//! # A lei do casamento: por ID, nunca por nome
//!
//! Dois estados são duas listas de objetos, e a pergunta *"quem é quem"* tem uma resposta só: o
//! [`VecPathId`](ph2d_vec_scene::VecPathId). **Nunca o nome, nunca a posição na lista.** Um nome
//! muda quando o artista renomeia; uma posição muda quando ele reordena — e as duas coisas são
//! gestos que ninguém espera que quebrem uma animação. É o gate
//! `matching_survives_renaming_and_reordering` que pina isso.
//!
//! O resto cai do casamento:
//!
//! | no estado A | no estado B | o que acontece |
//! |---|---|---|
//! | sim | sim | **interpola** o que difere |
//! | sim | não | **sai** (fade-out, parado onde estava) |
//! | não | sim | **entra** (fade-in, já no lugar de destino) |
//! | idêntico | idêntico | **não anima** — e não custa nada |
//!
//! # ⛔ A MOLA foi MEDIDA e o solver NÃO se justifica — não o reconstrua
//!
//! O plano deixou em aberto *"se a curva `Elastic` bastar, o solver não se constrói"*. Medido
//! (`tests/measure_spring.rs`), em duas frentes:
//!
//! **A FORMA já está no catálogo.** `Elastic Out` mede pico **1,373** / assenta em **0,631** /
//! **4** travessias do alvo, contra **1,309 / 0,600 / 3** de um oscilador massa-mola-amortecedor
//! macio (`ω=12, ζ=0,35`) — a mesma animação. `Back Out` (1,100 / 1 travessia) cobre o *overshoot*
//! único que a maioria das molas de UI de facto usa.
//!
//! **A INTERRUPÇÃO é a pergunta de verdade, e o default passa.** O que uma mola dá e uma curva
//! não dá é *continuidade de velocidade*: revertendo a 30% do caminho, a volta arranca a
//! **1,34×** a velocidade com que a ida chegava sob o [`DEFAULT_EASING`] — o olho não separa isso
//! de 1,00×. Os dois regimes onde ela morde são `InOut` (**0,00×**: a cena PARA e recomeça, o
//! *stutter* que faz alguém pedir um solver) e `Elastic` (**7,02×**: estalo).
//!
//! ⚠️ **E os dois são INALCANÇÁVEIS hoje**, porque o seletor de curva não existe — a mesma
//! decisão, por outro lado: o catálogo não dá nome às 33 combinações, então o knob não nasceu. O
//! dia em que ele nascer, **esta medição volta à mesa** (§0: quem move o número reconfere a
//! nota). O gate `the_default_curve_reverses_without_stopping_dead` pina a banda, com o `InOut`
//! como controle.
//!
//! # A correspondência é do PAR, nunca do `t` — e o número é grande
//!
//! `ph2d_vec_blend::Plan::new` custa **0,64 ms mesmo quando as duas formas são IGUAIS** (a busca
//! de fase 256×256 roda de qualquer maneira) contra **0,0001 ms** de um passo — uma razão de
//! **13 079×**. Daí a forma desta API, que é a mesma do `Plan` que ela usa: [`Transition::new`]
//! paga o casamento **uma vez** e [`Transition::at`] é barato o bastante para rodar por frame.
//!
//! ⚠️ **E daí também a regra de ouro do custo:** um par cuja geometria é **idêntica** não constrói
//! `Plan` nenhum. Vinte objetos numa troca de estado só-de-cor pagariam **12,79 ms** — 77% de um
//! quadro de 60 fps — para não mover um vértice.

#![forbid(unsafe_code)]

mod machine;
mod pose;
mod role;
mod sets;
mod transition;

pub use machine::Machine;
pub use pose::{ObjectPose, UiState};
pub use role::StateRole;
pub use sets::{DEFAULT_DURATION_S, DEFAULT_EASING, HostStates, MAX_DURATION_S, StateSets};
pub use transition::Transition;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "machine_tests.rs"]
mod machine_tests;
