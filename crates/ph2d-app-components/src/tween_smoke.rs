//! ⭐⭐⭐ **Smoke do TWEEN** (suplente #22). `PH2D_TWEEN_SMOKE=1|2`.
//!
//! # `=1` — a GALERIA dos canais: quatro colunas, **um canal de diferença cada**
//!
//! | coluna | o canal | o que se vê |
//! |---|---|---|
//! | **Aparece** | `Opacity` | o quadrado surge do nada e fica |
//! | **Pisca** | `Silhueta` | ele acende numa cor e a ARTE volta |
//! | **Desliza** | `Position X` | ele vai de um lado ao outro |
//! | **Cresce** | `Scale X` **e** `Scale Y` | ⭐ **dois** tweens no MESMO objecto, em **dois** relógios |
//!
//! ⚠️⚠️ **A quarta coluna é a que ensina a lei do módulo:** o tween `i` corre no timer `i`, e os
//! dois relógios dela têm **períodos diferentes de propósito** (`0,8 s` e `1,2 s`) — é por isso que
//! ela espreme e estica fora de fase, em vez de crescer por igual. *Com o mesmo período, a coluna
//! seria indistinguível de um tween só, e a lei do índice ficava por demonstrar.*
//!
//! # ⛔ E o que um LAÇO não mostra é o FIM — por isso o roteiro manda desligá-lo
//!
//! Com `Repeat` ligado o relógio nunca acaba, logo o `On finish` **nunca é lido**: as colunas
//! *Aparece* (`Hold`) e *Pisca* (`Rewind`) correm exactamente iguais. Desligado o `Repeat`, elas
//! separam-se — uma fica no valor final, a outra **deixa de escrever** e a arte volta.
//! *É a mesma diferença que separa um desvanecer de um pisca, e ela não é visível num laço.*
//!
//! # `=2` — o que uma TIMELINE não pode fazer: a CÓPIA que nasce a meio da corrida
//!
//! Duas fábricas iguais, lado a lado, a produzir a MESMA receita ao mesmo ritmo. A da esquerda tem
//! tweens na receita; a da direita é o **CONTROLO** e não tem nenhum.
//!
//! ⚠️ **Esta cena existe porque a §5.0 foi medida ANTES da primeira linha:** uma faixa de timeline
//! com uma curva de opacidade **desvanece exactamente** — logo o componente não existe por causa do
//! desvanecer. Ele existe porque *uma ligação de timeline nomeia UMA entidade*, e estas cópias não
//! existiam quando o projecto foi gravado.
//!
//! ⚠️ Se a linha `[tween-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, Factory, Lifetime, MasterRoot, Name, Timer, Timers, Transform, Tweens, Visibility,
    World,
};
use ph2d_render::{Sprite, WHITE_TILE_KEY};
use ph2d_tween::{AoAcabar, Canal, Preset, Tween};

/// ⭐⭐ **Quantas cenas este roteador serve** — o `max_level` que o [`crate::FAMILY`] declara.
///
/// ⚠️ **CONTADO do `match` do [`montar`]**, nunca escrito de memória (`CLAUDE.md` §5.0).
pub const CENAS: u32 = 2;

/// O sinal que o relógio da fábrica publica na `=2`. ⚠️ Lido nos DOIS sítios pela mesma const.
pub const SINAL: &str = "nascer";

/// **O período do laço da galeria** — o mesmo nas três primeiras colunas, de propósito: com
/// períodos diferentes o olho compara ritmos em vez de comparar CANAIS.
const PERIODO_US: u64 = 1_200_000;
/// O segundo período da quarta coluna — ver o cabeçalho.
const PERIODO_CURTO_US: u64 = 800_000;

/// Quanto tempo uma cópia da `=2` vive. ⚠️ **Menor que o período da fábrica** (`NASCER_US`), senão
/// há sempre duas na tela e o que se lê é uma pilha em vez de um objecto.
const VIDA_US: u64 = 1_200_000;
/// De quanto em quanto tempo nasce uma cópia na `=2`.
const NASCER_US: u64 = 1_600_000;

/// ⭐⭐⭐ **E a cerca entre as duas é ERRO DE COMPILAÇÃO, não um teste.**
///
/// ⚠️ Um `assert!` de teste sobre duas constantes é **dobrado pelo compilador** antes de
/// correr — o clippy di-lo em voz alta (`assertions_on_constants`), e a forma que ele aponta é
/// esta. ⭐ Ao nível do MÓDULO ela é avaliada pelo `cargo check`, que é onde qualquer um a
/// encontra; ⛔ **dentro de uma função um `const { assert!(…) }` é CEGO ao `check`** (só é
/// avaliado quando a função é construída) — armadilha que a `line/sculpt3d` mediu em 14/09.
///
/// *Sem isto há sempre duas cópias na tela de cada lado, e o que o dono lê é uma pilha em vez
/// de um objecto.*
const _: () = assert!(VIDA_US < NASCER_US);

const CHAO_RGBA: [f32; 4] = [0.14, 0.16, 0.20, 1.0];
const PECA_RGBA: [f32; 4] = [0.45, 0.68, 0.92, 1.0];
const COPIA_RGBA: [f32; 4] = [0.95, 0.76, 0.30, 1.0];
const COPIA_CTRL_RGBA: [f32; 4] = [0.62, 0.64, 0.68, 1.0];

/// O lado de um quadrado da galeria, em metros.
const LADO: f32 = 1.1;
/// A distância entre duas colunas da galeria.
const PASSO_X: f32 = 2.6;
/// Quanto a coluna *Desliza* percorre, para cada lado do sítio dela.
const CURSO: f32 = 0.9;

/// **A receita que esta fábrica ainda vai apontar** — o marcador de MONTAGEM da irmã do `#11`.
///
/// ⚠️ Ele existe porque a identidade só é atribuída depois: o `Factory::master` é um `StableId`, e
/// no instante em que a cena monta o mestre ainda não tem um.
#[derive(bevy_ecs::component::Component, Clone, Copy)]
struct Pendente(Entity);

/// Um relógio que **repete para sempre** e não fala — o motor de uma coluna da galeria.
///
/// ⚠️ **`signal` vazio é CALADO** (a lei do `#2`), e é o que se quer: aqui o timer é o relógio de um
/// tween, não um produtor de eventos. *Um nome aqui punha avisos na tela a cada segundo.*
///
/// ⚠️ **A `=2` é a excepção declarada, e a FOTO mostrou-a:** ali o relógio TEM de falar (é o
/// sinal que a fábrica ouve — o relógio próprio dela é recusa medida do `#11`), logo a tela
/// enche-se de avisos `Signal: nascer`. ⭐ O roteiro **nomeia-os** em vez de os esconder: um
/// aviso que a cena não explica lê-se como parte da lição.
fn laco(nome: &str, periodo_us: u64) -> Timer {
    Timer {
        name: nome.to_owned(),
        duration_us: periodo_us,
        repeat: true,
        autostart: true,
        signal: String::new(),
    }
}

/// Um quadrado da galeria: nome, sítio, os tweens e os relógios deles.
///
/// ⚠️ **As duas listas andam sempre juntas e pela mesma ordem** — é essa a lei do módulo (o tween
/// `i` corre no timer `i`), e montá-las em dois sítios diferentes é como ela se parte.
fn coluna(world: &mut World, nome: &str, x: f32, tweens: Vec<Tween>, timers: Vec<Timer>) -> Entity {
    world
        .spawn((
            Name::new(nome),
            Transform::from_translation(Vec2::new(x, 0.0)),
            Visibility::visible(),
            Sprite::atlas(WHITE_TILE_KEY, [LADO, LADO], PECA_RGBA),
            Tweens(tweens),
            Timers(timers),
        ))
        .id()
}

/// A cena `=1`. Devolve **quem nasce ESCOLHIDO** — a coluna *Aparece*, que é a do passo (1).
fn cena_um(world: &mut World) -> Entity {
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [20.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));

    let x0 = -1.5 * PASSO_X;

    // ⭐ **APARECE** — o preset *Fade In* tal e qual, para o roteiro poder dizer *«é este botão»*.
    let aparece = coluna(
        world,
        "Aparece (Opacity)",
        x0,
        vec![Preset::FadeIn.tween()],
        vec![laco("aparecer", PERIODO_US)],
    );

    // ⭐ **PISCA** — o preset *Flash*: `Silhueta` com `Rewind`, que é o que faz a arte voltar.
    coluna(
        world,
        "Pisca (Silhueta)",
        x0 + PASSO_X,
        vec![Preset::Flash.tween()],
        vec![laco("piscar", PERIODO_US)],
    );

    // ⭐ **VAI-E-VOLTA** — o canal de POSE, e é ele que prova que a pose é CONDUZIDA e não autorada:
    // o objecto anda o tempo todo e **não enche a fila do desfazer**.
    //
    // ⭐⭐⭐ **E é a coluna do PING-PONG** (pedido do dono, 19/09): ela corre com
    // [`Ciclo::PingPong`], logo vai e volta **suavemente** dentro do mesmo período. ⚠️ **A
    // comparação é um CLIQUE e não uma quinta coluna**, de propósito: com `Restart` na fileira
    // *Cycle* ela passa a SALTAR de volta, e é o artista a carregar no chip que vê a diferença —
    // o que prova, de graça, que o chip está vivo sob o dedo. *Uma quinta coluna também não caberia:
    // a meia-largura visível é `5,2 m` (medida pelo #18) e cinco passos de `2,6` põem as pontas
    // fora do ecrã.*
    let x_desliza = x0 + 2.0 * PASSO_X;
    coluna(
        world,
        "Vai-e-volta (Position X)",
        x_desliza,
        vec![Tween {
            easing: ph2d_anim::Easing::new(
                ph2d_anim::EasingFamily::Quad,
                ph2d_anim::EasingMode::InOut,
            ),
            ciclo: ph2d_tween::Ciclo::PingPong,
            ..Tween::linear(Canal::PositionX, x_desliza - CURSO, x_desliza + CURSO)
        }],
        vec![laco("deslizar", PERIODO_US)],
    );

    // ⭐⭐ **CRESCE** — DOIS tweens no MESMO objecto, em DOIS relógios de períodos diferentes.
    //
    // ⚠️ É a coluna que torna a lei do índice **visível**: se os dois relógios tivessem o mesmo
    // período, o quadrado cresceria por igual e isto leria-se como um tween só.
    coluna(
        world,
        "Cresce (Scale X e Y)",
        x0 + 3.0 * PASSO_X,
        vec![
            Tween::linear(Canal::ScaleX, 0.45, 1.25),
            Tween::linear(Canal::ScaleY, 0.45, 1.25),
        ],
        vec![
            laco("largura", PERIODO_CURTO_US),
            laco("altura", PERIODO_US),
        ],
    );

    aparece
}

/// A RECEITA de uma cópia da `=2`: um mestre escondido, com vida curta e (talvez) tweens.
///
/// ⚠️ **O `Lifetime` não é enfeite** — sem ele cada cópia fica na cena para sempre e ao fim de meio
/// minuto a `=2` é uma pilha de quadrados parados (a lei do `#12`: sem ela o `#11` VAZA).
fn receita(world: &mut World, nome: &str, cor: [f32; 4], com_tweens: bool) -> Entity {
    let mut e = world.spawn((
        Name::new(nome),
        MasterRoot,
        Transform::from_translation(Vec2::ZERO),
        Visibility::visible(),
        Sprite::atlas(WHITE_TILE_KEY, [LADO, LADO], cor),
        Lifetime {
            duration_us: VIDA_US,
            ..Lifetime::default()
        },
    ));
    if com_tweens {
        // ⭐ Os dois tweens que fazem uma cópia **ENTRAR** em vez de aparecer: ela cresce e ganha
        // corpo ao mesmo tempo. ⚠️ **`Hold` nos dois** — uma cópia que desaparecesse no fim do
        // relógio ficaria invisível durante o resto da vida dela, e o que se lia era outro defeito.
        e.insert((
            Tweens(vec![
                Preset::FadeIn.tween(),
                Tween {
                    ao_acabar: AoAcabar::Hold,
                    ..Tween::linear(Canal::ScaleX, 0.3, 1.0)
                },
                Tween {
                    ao_acabar: AoAcabar::Hold,
                    ..Tween::linear(Canal::ScaleY, 0.3, 1.0)
                },
            ]),
            Timers(vec![
                nascimento("aparecer"),
                nascimento("largura"),
                nascimento("altura"),
            ]),
        ));
    }
    e.id()
}

/// O relógio de uma cópia: corre **uma vez**, e arranca no instante em que ela nasce.
///
/// ⚠️ **`autostart` é o que faz a cópia animar-se sozinha** — o `reconcile` cria o estado dela com
/// `born(t)` no quadro em que ela aparece, e é por isso que isto não precisa de sinal nenhum.
fn nascimento(nome: &str) -> Timer {
    Timer {
        name: nome.to_owned(),
        duration_us: 450_000,
        repeat: false,
        autostart: true,
        signal: String::new(),
    }
}

/// Uma fábrica com o relógio dela ao lado, no sítio `x`.
fn fabrica(world: &mut World, nome: &str, x: f32, receita_id: Entity) {
    world.spawn((
        Name::new(nome),
        Transform::from_translation(Vec2::new(x, 0.0)),
        Timers(vec![Timer {
            name: "nascer".to_owned(),
            duration_us: NASCER_US,
            repeat: true,
            autostart: true,
            signal: SINAL.to_owned(),
        }]),
        Factory {
            master: 0, // resolvido em `resolver_receitas`
            on_signal: SINAL.to_owned(),
            burst: 1,
            ..Factory::default()
        },
        Pendente(receita_id),
    ));
}

/// A cena `=2`. Devolve **quem nasce ESCOLHIDO** — a fábrica da esquerda, a que tem os tweens.
fn cena_dois(world: &mut World) -> Entity {
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [20.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));

    let com = receita(world, "Copia (com tween)", COPIA_RGBA, true);
    let sem = receita(world, "Copia (controlo)", COPIA_CTRL_RGBA, false);

    fabrica(world, "Fabrica (com tween)", -2.2, com);
    fabrica(world, "Fabrica (controlo)", 2.2, sem);

    // ⚠️ **A fábrica da esquerda é a escolhida** porque é dela que o roteiro fala; e a `=2` inteira
    // só é legível com as duas lado a lado — *uma cópia que entra suave, sozinha, não se distingue
    // de uma cópia que aparece*.
    let mut q = world.query::<(Entity, &Name)>();
    q.iter(world)
        .find(|(_, n)| n.0 == "Fabrica (com tween)")
        .map(|(e, _)| e)
        .expect("a fabrica acabou de ser montada")
}

/// Troca cada [`Pendente`] pelo `StableId` do mestre. ⚠️ Corre DEPOIS de a identidade existir.
fn resolver_receitas(world: &mut World) {
    ph2d_ecs::assign_missing_stable_ids(world);
    let pares: Vec<(Entity, Entity)> = {
        let mut q = world.query::<(Entity, &Pendente)>();
        q.iter(world).map(|(e, p)| (e, p.0)).collect()
    };
    for (fab, master) in pares {
        let Some(id) = world.get::<ph2d_ecs::StableId>(master).map(|s| s.0) else {
            continue;
        };
        if let Some(mut f) = world.get_mut::<Factory>(fab) {
            f.master = id;
        }
        world.entity_mut(fab).remove::<Pendente>();
    }
}

/// O que a cena montou — o que a shell precisa de saber para acabar o prólogo.
pub struct Montada {
    /// Qual cena foi montada.
    pub nivel: u32,
    /// Quem nasce escolhido — ver [`cena_um`] e [`cena_dois`].
    pub escolhido: u64,
}

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
///
/// ⚠️ Separada do prólogo pela razão das irmãs: é a metade que um gate consegue correr — o resto
/// pede o relógio e os painéis, que não são o mundo.
pub fn montar(world: &mut World, nivel: u32) -> Montada {
    match nivel {
        2 => {
            let escolhido = cena_dois(world);
            resolver_receitas(world);
            println!(
                "[tween-smoke] cena=2  a copia que nasce a MEIO da corrida\n\
                 (1) espere: de {:.1} em {:.1} segundos nasce um quadrado de cada lado\n\
                 (2) o da ESQUERDA (amarelo) ENTRA — cresce e ganha corpo; o da DIREITA (cinzento) \
                 APARECE de uma vez: e' o CONTROLO, e a unica diferenca e' a receita dele nao ter \
                 tweens\n\
                 (3) as duas receitas sao iguais em tudo o resto, e as duas fabricas correm no \
                 mesmo ritmo\n\
                 (4) os avisos «Signal: nascer» no topo sao o RELOGIO a falar com as fabricas — e' \
                 assim que elas sabem quando produzir, e por isso aparecem a cada nascimento\n\
                 (5) na barra de CIMA carregue em `Reset`: a cena volta ao principio e param de \
                 nascer — as copias DESAPARECEM, porque nasceram na corrida e nao estao no \
                 ficheiro. `Play`, ao lado, traz-nas de volta\n\
                 (6) deu errado se: os dois lados entram iguais · nada nasce · ou os quadrados se \
                 acumulam em vez de desaparecerem",
                NASCER_US as f64 / 1e6,
                NASCER_US as f64 / 1e6,
            );
            Montada {
                nivel: 2,
                escolhido: escolhido.to_bits(),
            }
        }
        _ => {
            let escolhido = cena_um(world);
            println!(
                "[tween-smoke] cena=1  a galeria dos canais (periodo {:.1}s)\n\
                 (1) quatro quadrados, UM canal de diferenca cada: Aparece · Pisca · Desliza · \
                 Cresce\n\
                 (2) o quarto usa DOIS tweens em DOIS relogios de periodos diferentes — e' por isso \
                 que ele espreme e estica fora de fase\n\
                 (3) role o painel da direita ate' ao FIM (o «Aparece» ja' esta' escolhido): a \
                 seccao TWEEN e' a ultima, e tem a lista; escolha a linha e veja o canal, o de/para \
                 e a curva — e a linha «Duration … set in Timer N, above» diz QUANTO tempo ele leva \
                 e ONDE se muda, porque o tempo e' do RELOGIO e nao do tween\n\
                 (4) na seccao TIMER do mesmo objecto DESLIGUE o `Repeat`: o desvanecer acontece \
                 UMA vez e FICA. Faca o mesmo no «Pisca» e ele acende uma vez e a ARTE VOLTA — e' a \
                 diferenca entre `Hold` e `Rewind`, e ela nao se ve^ num laco\n\
                 (5) na seccao TWEEN carregue em `Flash`: um clique escreve os cinco campos E a \
                 duracao do relogio\n\
                 (5-bis) escolha o «Vai-e-volta» e na fileira `Cycle` carregue em `Restart`: ele \
                 passa a SALTAR de volta ao principio. `Ping-Pong` devolve o ir-e-voltar suave\n\
                 (6) carregue em `Pause` na barra de CIMA: tudo congela onde esta'. `Play` devolve\n\
                 (7) deu errado se: algum quadrado nao se mexe · o quarto cresce por igual nos dois \
                 eixos · desligar o `Repeat` nao separa o «Aparece» do «Pisca» · ou o `Restart` nao \
                 faz o «Vai-e-volta» saltar",
                PERIODO_US as f64 / 1e6,
            );
            Montada {
                nivel: 1,
                escolhido: escolhido.to_bits(),
            }
        }
    }
}

#[cfg(test)]
#[path = "tween_smoke_tests.rs"]
mod tests;
