//! ⭐⭐⭐ **Smoke do OLHO** (suplente #21, W6). `PH2D_RAY_SMOKE=1`.
//!
//! # A cena: **um poste com DOIS olhos, e uma caixa que chega**
//!
//! Os dois olhos têm o **mesmo componente** e **uma** diferença — a `Direction`. Uma caixa vem da
//! direita:
//!
//! | olho | `Direction` | o que acontece |
//! |---|---|---|
//! | **o de cima** | `(+1, 0)`, para a direita | vê a caixa, a luz da **direita** acende, e a leitura viva do painel conta a distância |
//! | **o de baixo** (o CONTROLO) | `(−1, 0)`, para trás | fica **calado** o tempo todo |
//!
//! ⭐⭐⭐ **É a coluna que a composição não exprime, desenhada.** A sonda
//! [`mede_o_que_a_composicao_ja_da_ao_raio`] mediu a melhor alternativa que a casa tem — um colisor
//! `is_sensor` fino deitado ao longo da linha — e ela **não distingue frente de trás**, porque *uma
//! FORMA é simétrica por construção*. As outras duas colunas (**ordem** e **métrica**) aparecem no
//! mesmo gesto: o tracinho do acerto marca **onde** ele encontrou e o painel diz **a que distância**.
//!
//! # ⚠️ Os dois olhos gritam nomes DIFERENTES, e não é a diferença que se demonstra
//!
//! Com o mesmo nome as duas luzes acendiam juntas, porque um sinal é um **nome global** — a lição
//! que a cena do golpe já pagou. *O nome isola as duas experiências; o que se mede é a direcção.*
//!
//! # ⚠️ A caixa começa FORA do alcance, e isso é MEDIDO
//!
//! Ela nasce a `+5` e viaja `7` m até parar em `−2`; a linha do olho acaba em `x = 0`. ⇒ o passo (2)
//! mostra o **alcance a ser um limite a sério** — o olho tem a caixa à vista e diz *«Sees nothing
//! right now.»* — e só depois é que ela entra. ⛔ Uma cena em que a caixa já nascesse dentro do
//! alcance perdia metade da lição, que é a espécie de cena que o `CLAUDE.md` §5.0 nomeia.
//!
//! ⭐ **E ela PÁRA dentro do alcance, em vez de atravessar:** um projéctil que o artista pôs na cena
//! à mão é **documento**, logo o fim do alcance dele não o apaga — ele fica. É isso que dá ao dono
//! uma leitura **estável** para ler no painel e um alvo parado para o passo (6) do `Reach`.
//!
//! ⚠️ Se a linha `[ray-smoke]` não aparecer, **PARE**: a cena não montou.

use ph2d_core::Vec2;
use ph2d_ecs::{
    Entity, Name, SignalAction, SignalActions, SignalFrom, SignalTarget, SignalVerb, Transform,
    Visibility, World,
};
use ph2d_physics_ecs::{
    BodyKind, Collider, ColliderShape, ProjectileMotion, RaySensor, RaySignals, RigidBody,
};
use ph2d_projectile::ProjectileLaw;
use ph2d_render::{Sprite, WHITE_TILE_KEY};

/// ⭐⭐ **Quantas cenas este roteador serve** — CONTADO do [`montar`].
pub const CENAS: u32 = 1;

/// **Até onde cada olho enxerga, em metros** — e ele é a régua da cena inteira.
///
/// ⚠️ **A geometria toda é derivada dele** ([`X_POSTE`], [`X_CAIXA`], [`VIAGEM`]), para o passo (2)
/// e o passo (6) do roteiro continuarem verdadeiros se alguém mexer no número. O gate
/// `a_caixa_comeca_fora_do_alcance_e_acaba_dentro` mede as duas pontas.
pub const ALCANCE: f32 = 6.0;

/// ⭐⭐⭐ **A JANELA ÚTIL do canvas, em metros `(x0, x1, y0, y1)` — MEDIDA na foto de 19/09.**
///
/// A câmera de omissão (`height_world = 10`, centro na origem) sobre a superfície `1930×1012` dá um
/// mundo visível de `x ∈ [−9,54; +9,54]` e `y ∈ [−5; +5]`, **e os painéis TAPAM-NO**: a Hierarquia
/// até `x ≈ −7,3`, o Inspector a partir de `x ≈ +6,8`, e a régua da timeline a partir de
/// `y ≈ −1,3`.
///
/// ⚠️⚠️ **É a régua que a cena `=45` da escultura pagou com um report do dono** (*«resultado bem
/// bizarro»*): ali `d/R` lia `5,70`–`7,85` em todo o ecrã e **nenhuma** posição produzia o efeito.
/// Aqui a pergunta é a mesma — *o dono VÊ o que o roteiro manda ver?* —, e a 1.ª redacção desta
/// cena reprovava: a luz do CONTROLO ficava atrás da Hierarquia.
///
/// ⛔ **Ela é de UMA superfície e diz-se assim.** Um número derivado da câmera em runtime seria mais
/// geral e não é alcançável de um gate sem device; o que a torna honesta é a medição estar escrita
/// ao lado dela, e o gate `a_cena_inteira_cabe_na_janela_util` nomear cada peça que ela abriga.
///
/// ⚠️ **`pub` como o [`ALCANCE`], e pela mesma razão:** ela é um FACTO medido sobre esta cena, e o
/// único consumidor dela é um gate — ⛔ escondê-la atrás de `#[cfg(test)]` poria uma medição num
/// sítio onde nenhum leitor da família a encontra.
pub const JANELA_UTIL: (f32, f32, f32, f32) = (-7.3, 6.8, -1.3, 5.0);

/// Onde o poste está.
///
/// ⚠️ **MEDIDO na foto e não escolhido:** a 1.ª redacção pô-lo em `−6`, e a luz do CONTROLO (que
/// vive `1,6` m à esquerda dele) caía **fora do canvas útil**, tapada pela Hierarquia — *um controlo
/// que não se vê não é um controlo*, e o passo (5) do roteiro afirma que ele está na tela. Ver
/// [`JANELA_UTIL`] e o gate `a_cena_inteira_cabe_na_janela_util`.
const X_POSTE: f32 = -4.0;
/// Onde a caixa nasce — **além** da ponta da linha (`X_POSTE + ALCANCE = 2`).
const X_CAIXA: f32 = 6.0;
/// Quantos metros ela anda antes de parar. `6 → −0,5`, logo a `3,1` m do olho.
const VIAGEM: f32 = 6.5;
/// A rapidez dela, m/s — devagar de propósito: o tracinho do acerto tem de **deslizar** à vista.
const RAPIDEZ: f32 = 2.0;

/// Meia-largura e meia-altura da caixa.
const CAIXA_MEIA: [f32; 2] = [0.4, 0.6];
/// A altura das duas luzes, e o lado de cada uma.
const Y_LUZ: f32 = 2.4;
const LUZ_LADO: f32 = 0.8;
/// O `y` de cada olho — ⚠️ os dois dentro da altura da caixa, senão a diferença deixava de ser
/// **só** a direcção (ver o gate `os_dois_olhos_diferem_so_na_direccao`).
const Y_OLHO: f32 = 0.35;

/// O nome do sinal de quem VÊ.
const VIU: &str = "vi";
/// E o de quem deixa de ver.
const PERDI: &str = "perdi";
/// Os do CONTROLO — ⚠️ outros nomes, senão as duas luzes acendem juntas (ver o cabeçalho).
const VIU_ATRAS: &str = "vi-atras";
const PERDI_ATRAS: &str = "perdi-atras";

const CHAO_RGBA: [f32; 4] = [0.16, 0.18, 0.22, 1.0];
const POSTE_RGBA: [f32; 4] = [0.45, 0.47, 0.52, 1.0];
const LUZ_RGBA: [f32; 4] = [0.98, 0.82, 0.25, 1.0];
/// O SUPORTE — ver [`luz`]. Escuro, e sempre visível.
const SUPORTE_RGBA: [f32; 4] = [0.26, 0.27, 0.30, 1.0];
const CAIXA_RGBA: [f32; 4] = [0.80, 0.80, 0.84, 1.0];

/// Uma linha de tabela que age sobre **este** objecto.
fn linha(on: &str, verb: SignalVerb) -> SignalAction {
    SignalAction {
        on: on.to_owned(),
        target: String::new(),
        verb,
        arg: String::new(),
        target_by: SignalTarget::Named,
        from: SignalFrom::Anyone,
    }
}

/// **Uma luz que acende ao ouvir e apaga ao deixar de ouvir**, com o SUPORTE por baixo dela.
///
/// ⚠️ **Ela fica do LADO para onde o olho dela olha**, e isso é o que torna a cena legível sem ler
/// um nome: a luz que acende está na direcção da linha que acendeu.
///
/// ⛔⛔⛔ **O SUPORTE não é enfeite, e a FOTO é que o exigiu (19/09).** Os verbos que existem são
/// `Show` e `Hide`, logo uma luz apagada **não está lá** — e o passo (5) do roteiro afirma *«a luz
/// da ESQUERDA nunca acende»*. Sem o suporte, *«não acendeu»* e *«não há luz nenhuma deste lado»*
/// dão ao dono exactamente o mesmo ecrã, e o CONTROLO deixa de provar o que quer que seja.
/// ⇒ um quadrado ESCURO, sempre visível, no mesmo sítio: a luz acende **por cima** dele.
fn luz(world: &mut World, nome: &str, x: f32, acende: &str, apaga: &str) -> Entity {
    world.spawn((
        Name::new(format!("Suporte: {nome}")),
        Transform::from_translation(Vec2::new(x, Y_LUZ)),
        Visibility::visible(),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [LUZ_LADO * 1.4, LUZ_LADO * 1.4],
            SUPORTE_RGBA,
        ),
    ));
    world
        .spawn((
            Name::new(nome),
            Transform::from_translation(Vec2::new(x, Y_LUZ)),
            Visibility::hidden(),
            Sprite::atlas(WHITE_TILE_KEY, [LUZ_LADO, LUZ_LADO], LUZ_RGBA),
            SignalActions(vec![
                linha(acende, SignalVerb::Show),
                linha(apaga, SignalVerb::Hide),
            ]),
        ))
        .id()
}

/// **Um olho** — o componente e a voz, e **NADA MAIS**.
///
/// ⛔⛔⛔ **Ele NÃO tem `Sprite`, e a ausência foi MEDIDA pela foto (19/09).** A 1.ª redacção deu-lhe
/// um, pelo argumento certo do `#15` (*«o que tem cérebro tem CORPO»*, senão o dedo do dono não lhe
/// chega) — e a foto mostrou o preço: um `Sprite` traz ao Inspector as secções **RENDER SOURCE**,
/// **COLOR & TINT** e **SPRITE SHEET**, e a secção `Ray Sensor` que o roteiro manda ler caía
/// **três ecrãs abaixo** da dobra. *Um passo que manda procurar uma linha AFIRMA que ela está na
/// tela, e o dono aprova o smoke com a afirmação dentro.*
///
/// ⭐ **A premissa do `#15` não se aplica aqui, e a diferença é o ROTEIRO:** lá o passo era *carregue
/// na porta*, e aqui **nenhum passo pede um clique de canvas** — o olho da frente nasce ESCOLHIDO e
/// o de baixo tem linha própria na Hierarquia. O que o dono vê no canvas são as **duas linhas** e as
/// **duas luzes**, que é exactamente a lição.
///
/// ⛔ **E ele não tem colisor**, também de propósito: com um, o raio do olho de baixo acertaria no de
/// cima e o CONTROLO deixava de estar calado — por uma razão que não é a direcção.
fn olho(world: &mut World, nome: &str, dir: Vec2, y: f32, voz: RaySignals) -> Entity {
    world
        .spawn((
            Name::new(nome),
            Transform::from_translation(Vec2::new(X_POSTE, y)),
            Visibility::visible(),
            RaySensor {
                origin: Vec2::ZERO,
                dir,
                reach: ALCANCE,
                layer: 0,
            },
            voz,
        ))
        .id()
}

/// A cena `=1`. Devolve **quem nasce ESCOLHIDO** — o olho que vê, porque o roteiro fala da secção
/// dele.
fn cena_um(world: &mut World) -> Entity {
    // ⚠️⚠️ **O CHÃO PRIMEIRO**, e isto não é estilo: a ordem das raízes é a de CRIAÇÃO, logo quem
    // nasce primeiro desenha por baixo (a cura de 15/09).
    world.spawn((
        Name::new("Ground"),
        Sprite::atlas(WHITE_TILE_KEY, [24.0, 12.0], CHAO_RGBA),
        Transform::from_translation(Vec2::ZERO),
    ));
    // ⚠️ **A altura e o centro do poste também saem da foto:** a régua da timeline tapa o canvas a
    // partir de `y = −1,3`, e um poste mais comprido aparecia CORTADO pela borda de baixo.
    world.spawn((
        Name::new("Poste"),
        Sprite::atlas(WHITE_TILE_KEY, [0.5, 1.8], POSTE_RGBA),
        Transform::from_translation(Vec2::new(X_POSTE, -0.25)),
    ));

    // As duas luzes, cada uma do lado para onde o olho dela olha.
    luz(world, "Luz da frente", X_POSTE + 1.6, VIU, PERDI);
    luz(world, "Luz de tras", X_POSTE - 1.6, VIU_ATRAS, PERDI_ATRAS);

    let ve = olho(
        world,
        "Olho que ve",
        Vec2::new(1.0, 0.0),
        Y_OLHO,
        RaySignals {
            on_enter: VIU.to_owned(),
            on_exit: PERDI.to_owned(),
        },
    );
    olho(
        world,
        "Olho que olha para tras",
        Vec2::new(-1.0, 0.0),
        -Y_OLHO,
        RaySignals {
            on_enter: VIU_ATRAS.to_owned(),
            on_exit: PERDI_ATRAS.to_owned(),
        },
    );

    // ⭐ **A CAIXA é matéria**, e tem de o ser: a porta do motor põe `EXCLUDE_SENSORS` dentro dela
    // (*«um sensor é um marcador, não matéria»*), logo um alvo `is_sensor` seria invisível ao raio.
    //
    // ⚠️ **`face_velocity: false`**: a rotação é o RUMO do lançamento (`π` = para a esquerda) e tem
    // de ficar quieta — uma caixa a rodar não muda nada na lei e distrai de onde está a lição.
    world.spawn((
        Name::new("Caixa"),
        Transform {
            translation: Vec2::new(X_CAIXA, 0.0),
            rotation: std::f32::consts::PI,
            ..Transform::default()
        },
        Visibility::visible(),
        Sprite::atlas(
            WHITE_TILE_KEY,
            [CAIXA_MEIA[0] * 2.0, CAIXA_MEIA[1] * 2.0],
            CAIXA_RGBA,
        ),
        RigidBody {
            kind: BodyKind::Kinematic,
        },
        Collider {
            shape: ColliderShape::Cuboid {
                half_x: CAIXA_MEIA[0],
                half_y: CAIXA_MEIA[1],
            },
            ..Collider::default()
        },
        ProjectileMotion::from_law(
            ProjectileLaw {
                initial_speed: RAPIDEZ,
                range: VIAGEM,
                face_velocity: false,
                ..ProjectileLaw::default()
            },
            0,
        ),
    ));
    ve
}

/// O que o prólogo precisa de saber da cena montada.
pub struct Montada {
    /// Qual cena foi montada.
    pub nivel: u32,
    /// O OLHO QUE VÊ, que nasce escolhido.
    pub escolhido: u64,
}

/// **Monta a cena que `nivel` pede, e devolve QUAL montou.**
pub fn montar(world: &mut World, _nivel: u32) -> Montada {
    let escolhido = cena_um(world);
    println!(
        "[ray-smoke] cena=1  alcance={ALCANCE} m\n\
         (1) o poste da esquerda tem DOIS olhos, e de cada um sai uma linha fina — uma para a \
         DIREITA e outra para a ESQUERDA. Cada linha e' o alcance de um olho, e o tracinho na PONTA \
         marca onde ele acaba. Por cima do poste estao as DUAS luzes deles, as duas apagadas\n\
         (2) a caixa clara vem da direita. Enquanto ela estiver ALEM da ponta da linha nao acontece \
         nada, e no painel da direita a seccao `Ray Sensor` diz `Sees nothing right now.` — o \
         alcance e' um limite a serio\n\
         (3) quando ela ENTRA na linha: aparece um segundo tracinho SOBRE a linha, no ponto em que \
         o olho a encontrou, e a luz da DIREITA acende. A caixa continua a vir e o tracinho desliza \
         com ela ate' a caixa parar\n\
         (4) o olho da frente ja' nasce escolhido: no painel, a linha por baixo do titulo passa a \
         dizer `Sees Caixa` com a distancia em metros, e o numero MUDA enquanto ela se aproxima\n\
         (5) a luz da ESQUERDA fica APAGADA o tempo todo, e esse e' o CONTROLO: o olho de baixo \
         tem o MESMO componente com UMA diferenca — o `Direction` esta' ao contrario. Uma forma nao \
         distingue a frente de tras; um raio distingue\n\
         (6) agora mude o `Reach` do olho escolhido para 2: a linha ENCOLHE no canvas na hora e a \
         luz da direita APAGA, porque o olho deixou de alcancar a caixa. Ponha 6 outra vez e ela \
         volta a acender\n\
         (7) deu errado se: as linhas nao aparecem · o tracinho do encontro nao desliza com a caixa \
         · a luz da esquerda acende · a distancia do painel nao muda · ou mudar o `Reach` nao muda \
         a linha"
    );
    Montada {
        nivel: 1,
        escolhido: escolhido.to_bits(),
    }
}

#[cfg(test)]
#[path = "ray_smoke_tests.rs"]
mod tests;
