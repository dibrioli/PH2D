//! **O SUBSTRATO DA UI VIVA** — onde mora o `t` que o chrome deste app nunca teve.
//!
//! # O achado que fez esta wave existir
//!
//! O app corre `ControlFlow::Poll` (redesenha **sempre**), o `render_loop` calcula `wall_dt` todo
//! quadro, e a `ph2d-spring` resolve molas com continuidade de velocidade sob interrupção — e a
//! interface do próprio app era uma **função escada**: nenhum `t` chegava à camada de widgets, e
//! toda mudança de estado era instantânea. *Animação já estava paga; faltava quem a consumisse.*
//!
//! # A regra que o chamador segue
//!
//! ⚠️ **O chamador diz o que a coisa É (`Role`), nunca como ela se move.** Um chamador que passasse
//! uma duração teria **re-implementado o carácter** no sítio dele, e no dia seguinte metade do app
//! estaria em Expressivo e metade não. Quem decide a lei é [`UiMotion::law`], e ela é perguntada
//! pelo pintor E pelo dispatch — duas cópias divergem no primeiro caso especial (a cicatriz do
//! `TimelineInterpScope::menu_table` e a do `stroke_cover_wanted`).
//!
//! # ⚠️ Onde este módulo DIVERGE do plano que o encomendou, e porquê
//!
//! O [`PLANO_UI_viva_2026-08-12.md`] previa **mola em Expressivo e uma CURVA (120 ms) em
//! Discreto**. A construção mostrou que isso custaria um **segundo catálogo de curvas** dentro da
//! `ph2d-editor-core` — exactamente o que o repo recusa em toda parte (*"um segundo catálogo faria
//! ease-out significar coisas diferentes em dois lugares do app"*, `ph2d-ui-state/Cargo.toml`).
//!
//! O que shipa é **UM mecanismo, dois pontos de operação**: uma mola **criticamente amortecida**
//! (`ζ = 1`) é, por construção, uma que **nunca ultrapassa** — que é literalmente o contrato do
//! carácter Discreto. Assim o contrato é **estrutural em vez de prometido**, e a herança de
//! velocidade (que o estudo chamou de *o* diferenciador) vale nos **dois** caracteres em vez de ser
//! luxo de um só.
//!
//! ⚠️ E a rigidez do Discreto **não foi escolhida, foi medida** — e a primeira medição
//! **refutou o número que eu tinha cravado**. Ver a tabela em [`DISCRETE`].
//!
//! # As duas grandezas de custo, que são diferentes
//!
//! - **integrar** é `O(em voo)` — tipicamente 0-3, e é o número que importa por quadro;
//! - **lembrar** é `O(widgets tocados recentemente)` — um `f32` por id, podado quando o widget
//!   deixa de ser pintado.
//!
//! ⚠️ Um app que nunca foi tocado tem o mapa **vazio**, e com o mapa vazio [`UiMotion::animate`]
//! devolve o alvo: **a tela é byte-idêntica à de antes desta wave**. É essa neutralidade que torna
//! a wave segura de landar sozinha.

use std::collections::BTreeMap;

use ph2d_spring::{Spring, SpringState};

use ph2d_a11y::NodeId;

/// **O CARÁCTER** — a escolha do Enio (2026-08-12): as duas, e quem escolhe é o utilizador.
///
/// ⚠️ **Discreto NÃO é Expressivo com os números baixos.** Se fosse, seria um multiplicador global
/// e o resultado seria uma UI expressiva a mexer-se depressa demais, que é a pior das três. São
/// duas respostas à mesma pergunta: *o objecto é físico* contra *a mudança aconteceu, e onde*.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UiCharacter {
    /// Chega e assenta. **Nunca ultrapassa** — e isso é estrutural (`ζ = 1`), não uma promessa.
    #[default]
    Discrete,
    /// O objecto tem peso: ultrapassa e volta.
    Expressive,
}

impl UiCharacter {
    /// O nome durável que um ficheiro de preferências guarda.
    ///
    /// ⚠️ **O mapeamento nome↔variante mora AQUI, e só aqui.** Escrito duas vezes — uma no leitor,
    /// outra no escritor — passa a ser duas respostas a *"como se chama o Expressivo em disco?"*, e
    /// com duas variantes a divergência é invisível: troque as duas e o ficheiro continua a
    /// desserializar, com o carácter errado. É a cicatriz do `BodyKind::tag`/`from_tag` da física.
    ///
    /// ⚠️ **Uma PALAVRA e não um número, porque o ficheiro é TEXTO** (`shells/desktop/src/prefs.rs`,
    /// irmão do `palette_persist.rs` que já vive em `~/.ph2d/`). Um número num ficheiro que o artista
    /// pode abrir é um número que ele não sabe corrigir.
    #[must_use]
    pub fn wire(self) -> &'static str {
        match self {
            Self::Discrete => "discrete",
            Self::Expressive => "expressive",
        }
    }

    /// A volta. **`None` para um nome que este build não conhece** — quem chama decide, e a decisão
    /// honesta num ficheiro de preferências é *usa o default*: uma preferência que se recusa a
    /// arrancar é pior que uma preferência perdida.
    #[must_use]
    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "discrete" => Some(Self::Discrete),
            "expressive" => Some(Self::Expressive),
            _ => None,
        }
    }
}

/// **O QUE a coisa é** — e é só isto que o chamador declara.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Posição, tamanho, percurso. É o que o *reduced motion* mata.
    Travel,
    /// Opacidade, cor. ⚠️ **Sobrevive ao reduced motion**: o gatilho vestibular é a área grande a
    /// deslocar-se, não a tinta a mudar.
    Fade,
    /// Um número que alguém LÊ.
    ///
    /// ⚠️ **Instantâneo nos três regimes, e é uma cerca.** Uma posição pode balançar; um valor
    /// lido que balança está **errado durante 200 ms**, e alguém vai lê-lo.
    Number,
    /// Enfeite (rasto, partícula, corda). **Ausente em Discreto** — ausente, não atenuado.
    Decoration,
}

/// Rigidez do Expressivo. `ζ < 1` ⇒ ultrapassa e volta: é o carácter inteiro num número.
///
/// ⚠️ **E o número que shipou primeiro NÃO entregava o carácter — a sonda desmentiu-o.** O report
/// do Enio em 2026-08-12 foi *«Discrete pode estar inativado ou não há diferença entre discrete e
/// expressive»*, e o `measure_the_overshoot_each_damping_buys` mostra porquê: a `ζ = 0,72`
/// ultrapassa **3,1%**, que não é uma diferença que um olho veja em canal nenhum. A tabela escolhe
/// sozinha, e o critério é *quanto se VÊ por quanto tempo se espera*:
///
/// | ζ | ultrapassa | assenta (<1%) |
/// |---|---|---|
/// | 0,45 | 19,7% | 0,467 s |
/// | **0,50** | **15,5%** | **0,467 s** |
/// | 0,55 | 11,8% | 0,450 s |
/// | 0,72 | 3,1% | 0,350 s |
///
/// `0,50` porque de 0,55 para 0,50 a ultrapassagem cresce um terço **pelo mesmo tempo de
/// assentamento** — é ganho de graça —, e abaixo disso a cauda começa a cobrar sem o olho ganhar
/// muito mais.
const EXPRESSIVE: Spring = Spring {
    stiffness: 18.0,
    damping: 0.50,
};

/// Rigidez do Discreto. ⚠️ `ζ = 1` é **criticamente amortecido**: a solução não tem termo
/// oscilatório, logo **não pode** ultrapassar. O contrato do carácter é a matemática, não o gosto.
///
/// ⚠️ **O número saiu da sonda `measure_ui_motion`, e a primeira versão desta constante estava
/// ERRADA porque eu media a grandeza errada.** Eu tinha cravado `28.0` afirmando no doc que ele
/// batia os ~120 ms que o plano pedia; a sonda mediu **0,517 s de assentamento** e desmentiu-me.
///
/// A causa não era a rigidez, era a RÉGUA: `SpringState::advance` devolve `true` no critério
/// **assintótico** (`|x-1| < 1e-3`), que é a **CAUDA** — e o olho não julga a cauda, julga o
/// **JOELHO**. Medindo `t95`/`t99`, a tabela decide sozinha:
///
/// | rigidez | t95 | t99 | assenta |
/// |---|---|---|---|
/// | 28,0 | 0,183 s | 0,267 s | 0,517 s |
/// | **40,0** | **0,133 s** | **0,183 s** | 0,383 s |
/// | 60,0 | 0,083 s | 0,133 s | 0,283 s |
///
/// ⚠️ E a sonda **satura**: a 60 Hz ela não resolve abaixo de um quadro, então 60 e 90 imprimem os
/// mesmos números — *porque a régua acabou, não porque as molas sejam iguais*.
const DISCRETE: Spring = Spring {
    stiffness: 40.0,
    damping: 1.0,
};

/// Quantos **SEGUNDOS** uma entrada sobrevive sem ser pintada antes de ser podada.
///
/// ⚠️ **Segundos, não quadros — e isto foi um defeito MEU, apanhado pelo gate do relógio de
/// parede.** A primeira versão contava quadros (`PRUNE_FRAMES: u32 = 8`), que é *exactamente* a
/// doença que o estudo desta wave diagnosticou no `ToastQueue`: a 120 fps a memória durava 66 ms e
/// a 30 fps durava 266 ms, e o gate `the_motion_is_a_fact_of_the_wall_clock_not_of_the_frame_rate`
/// nasceu VERMELHO por causa disso (0,847 contra 1,0). *Escrever a lição num documento não impede
/// ninguém de a repetir no arquivo seguinte; o gate impede.*
///
/// ⚠️ E não é ~zero: um widget que pisca fora da tela por um quadro (uma secção a re-medir, um
/// painel a re-pintar noutra ordem) perderia a memória e re-animaria do zero ao voltar.
const PRUNE_AFTER_S: f32 = 0.25;

/// Uma coisa que se move — ou que **já se moveu** e é lembrada só pelo valor onde parou.
#[derive(Clone, Copy, Debug)]
struct Track {
    /// Onde o percurso actual começou.
    from: f32,
    /// Onde ele termina — e, quando assentado, **é** o valor.
    to: f32,
    /// `None` = assentado. Só quem tem `Some` custa integração.
    flight: Option<SpringState>,
    role: Role,
    /// **Segundos** desde a última vez que alguém a pintou.
    idle_s: f32,
}

impl Track {
    fn value(&self) -> f32 {
        match self.flight {
            None => self.to,
            #[allow(clippy::cast_possible_truncation)]
            Some(s) => self.from + (self.to - self.from) * s.x as f32,
        }
    }

    /// A velocidade em unidades de **VALOR** por segundo (a `SpringState` fala em fracções do
    /// percurso, e o percurso muda a cada interrupção).
    #[allow(clippy::cast_possible_truncation)]
    fn velocity(&self) -> f32 {
        match self.flight {
            None => 0.0,
            Some(s) => s.v as f32 * (self.to - self.from),
        }
    }
}

/// **O substrato.** Um por tela; vive ao lado do store de interacção, nunca dentro dele.
///
/// ⚠️ **Fora do `InteractiveState` de propósito:** aquele é o estado **semântico**, e dezenas de
/// gates o comparam — misturar animação faria cada um passar a ver ruído, e um `assert_eq!` de
/// estado passaria a depender de *quando* foi lido. Mapa paralelo é o idioma que este repo já usa
/// três vezes para estender sem colidir (`bypassed_subgraphs`, `node_text_params`).
/// **Quanto um chip CRESCE quando o rato pousa nele**, no pico do hover.
///
/// ⚠️ **Número de APARÊNCIA: sai do smoke, não de um teste.** Três px sobre um chip de 36 lê como
/// *«ele reagiu»* sem mover o vizinho (o retângulo de HIT não cresce — ver [`hover_lift`]).
pub const HOVER_LIFT_PX: f32 = 3.0;

/// O retângulo que um chip DESENHA quando o hover está `t` presente.
///
/// ⚠️ **Este canal existe porque o outro não podia ter o carácter.** Uma fracção de fade é
/// clampada em `[0, 1]` — pela `blend_token_color` e pela própria noção de mistura —, e a
/// ultrapassagem do Expressivo vive **acima de 1**. Medido: com a `ζ` que shipava primeiro os dois
/// carácteres diferiam por 3,1% de ultrapassagem (clampada, logo invisível) e 50 ms de duração, e
/// o report do Enio foi exactamente *«não há diferença entre discrete e expressive»*. Numa
/// GEOMETRIA a ultrapassagem tem onde pousar: o chip passa do tamanho e volta.
///
/// ⚠️ **O hit NÃO cresce com o desenho, e isto é lei.** Um alvo que se move debaixo do dedo
/// enquanto reage ao dedo é um alvo que foge — e a fronteira de saída passaria a estar noutro
/// sítio da fronteira de entrada, que é como nasce o hover a piscar em cima da borda.
///
/// `t = 0` devolve o retângulo **intacto**: um chip em repouso desenha hoje o que sempre desenhou.
#[must_use]
pub fn hover_lift(rect: crate::zones::Rect, t: f32) -> crate::zones::Rect {
    let d = HOVER_LIFT_PX * t.max(0.0);
    if d <= 0.0 {
        return rect;
    }
    crate::zones::Rect::new(rect.x - d, rect.y - d, rect.w + d * 2.0, rect.h + d * 2.0)
}

#[derive(Clone, Debug, Default)]
pub struct UiMotion {
    tracks: BTreeMap<NodeId, Track>,
    character: UiCharacter,
    reduced: bool,
}

impl UiMotion {
    /// O carácter escolhido pelo utilizador (pill Settings).
    #[must_use]
    pub fn character(&self) -> UiCharacter {
        self.character
    }

    pub fn set_character(&mut self, c: UiCharacter) {
        self.character = c;
    }

    /// **Reduced motion — um eixo INDEPENDENTE do carácter**, não uma terceira posição dele.
    ///
    /// ⚠️ *Expressivo + reduced* é uma combinação legítima e tem de funcionar: alguém que gosta do
    /// material e do som, mas a quem a paralaxe faz mal. Um seletor de três posições tornaria essa
    /// pessoa incapaz de pedir o que precisa sem desistir do que gosta.
    #[must_use]
    pub fn reduced_motion(&self) -> bool {
        self.reduced
    }

    pub fn set_reduced_motion(&mut self, on: bool) {
        self.reduced = on;
    }

    /// **A PORTA.** A única função que sabe o que cada carácter faz.
    ///
    /// `None` = instantâneo, e o chamador não ganha entrada nenhuma — é o que faz do reduced motion
    /// também o modo mais **barato**.
    #[must_use]
    pub fn law(&self, role: Role) -> Option<Spring> {
        match role {
            Role::Number => None,
            Role::Travel if self.reduced => None,
            Role::Decoration if self.reduced || self.character == UiCharacter::Discrete => None,
            Role::Travel | Role::Fade | Role::Decoration => Some(match self.character {
                UiCharacter::Discrete => DISCRETE,
                UiCharacter::Expressive => EXPRESSIVE,
            }),
        }
    }

    /// **Um enfeite deve sequer ser desenhado?** O par da [`Self::law`] para quem emite partículas
    /// ou simula uma corda: em Discreto a decoração é **ausente**, e ausente significa *não gastar
    /// o trabalho*, não *desenhar com opacidade zero*.
    #[must_use]
    pub fn decorates(&self) -> bool {
        !self.reduced && self.character == UiCharacter::Expressive
    }

    /// **A chamada única do pintor:** diz o alvo, recebe o valor de agora.
    ///
    /// ⚠️ **A primeira vez que um id é visto NÃO anima** — ele chega ao alvo. Um widget que acaba
    /// de aparecer não tem de onde vir, e animá-lo do zero seria inventar uma história que não
    /// aconteceu.
    pub fn animate(&mut self, id: NodeId, target: f32, role: Role) -> f32 {
        let Some(spring) = self.law(role) else {
            // Instantâneo: nem sequer lembra. Um `Role::Number` nunca ocupa memória.
            self.tracks.remove(&id);
            return target;
        };
        let _ = spring;
        match self.tracks.get_mut(&id) {
            None => {
                self.tracks.insert(
                    id,
                    Track {
                        from: target,
                        to: target,
                        flight: None,
                        role,
                        idle_s: 0.0,
                    },
                );
                target
            }
            Some(t) => {
                t.idle_s = 0.0;
                t.role = role;
                if (t.to - target).abs() > f32::EPSILON {
                    // ⚠️ INTERRUPÇÃO — a lei do `Machine::go_to`: o caminho começa no valor VIVO,
                    // nunca no autorado. Partir do alvo antigo faria a UI SALTAR antes de voltar.
                    let (from, v) = (t.value(), t.velocity());
                    let span = target - from;
                    t.from = from;
                    t.to = target;
                    // ⚠️ A re-normalização `v / span` é a linha que faz a interrupção funcionar: a
                    // `SpringState` mede o caminho em [0,1], então uma velocidade em unidades de
                    // VALOR tem de ser dividida pelo comprimento NOVO. Sem isto um alvo próximo
                    // herda uma velocidade enorme e estala.
                    t.flight = Some(if span.abs() > f32::EPSILON {
                        SpringState::resuming(f64::from(v / span))
                    } else {
                        SpringState::at_rest()
                    });
                }
                t.value()
            }
        }
    }

    /// Anda o relógio. **Uma vez por quadro, com o `dt` de PAREDE** — nunca uma contagem de
    /// quadros (a lição que o `wall_dt` do `render_loop` já traz escrita, e que o `ToastQueue`
    /// passou anos sem aprender).
    pub fn advance(&mut self, dt: f64) {
        let laws: Vec<(NodeId, Option<Spring>)> = self
            .tracks
            .iter()
            .map(|(id, t)| (*id, self.law(t.role)))
            .collect();
        for (id, law) in laws {
            let Some(t) = self.tracks.get_mut(&id) else {
                continue;
            };
            #[allow(clippy::cast_possible_truncation)]
            {
                t.idle_s += dt as f32;
            }
            if let (Some(s), Some(spring)) = (t.flight.as_mut(), law)
                && s.advance(dt, spring)
            {
                // ⚠️ Assentar põe o valor EXACTO e larga o voo — a lei do `arrive`. Sem ela a mola
                // converge assintoticamente e o app integra para sempre.
                t.flight = None;
            }
        }
        // ⚠️ A PODA é o que torna verdadeira a alegação de custo: sem ela o mapa cresce
        // monotonamente com ids transientes e o `O(...)` vira falso em silêncio.
        self.tracks.retain(|_, t| t.idle_s < PRUNE_AFTER_S);
    }

    /// **Leitura pura** — o pintor pergunta sem alvejar. Ausente = nunca animou.
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<f32> {
        self.tracks.get(&id).map(Track::value)
    }

    /// Quantas coisas estão **em voo** — o número que custa integração por quadro.
    #[must_use]
    pub fn in_flight(&self) -> usize {
        self.tracks.values().filter(|t| t.flight.is_some()).count()
    }

    /// Quantos ids são **lembrados** (em voo ou assentados). Grandeza diferente da de cima.
    #[must_use]
    pub fn remembered(&self) -> usize {
        self.tracks.len()
    }

    /// Esquece tudo — troca de documento, de tela, ou o fim de um smoke.
    pub fn forget(&mut self) {
        self.tracks.clear();
    }
}

#[cfg(test)]
#[path = "motion_tests.rs"]
mod motion_tests;

/// **A PORTA ÚNICA da mistura de cor de token.** Um `t` fora de `[0,1]` é clampado aqui, e não em
/// cada chamador — a segunda cópia de um clamp é a que alguém esquece.
///
/// ⚠️ Mistura em **sRGB directo**, de propósito: estas são duas tintas de UI vizinhas na mesma
/// família de token (repouso → hover), e uma travessia OKLab entre elas custaria a dependência do
/// espaço de cor num caminho que corre por widget, por quadro, para uma diferença que ninguém
/// distingue em dois tons adjacentes. *Se um dia a mistura for entre tons distantes, esta é a
/// linha que muda — e é uma linha só.*
#[must_use]
pub fn blend_token_color(
    rest: Option<ph2d_tokens::Color>,
    hot: Option<ph2d_tokens::Color>,
    t: f32,
) -> Option<ph2d_tokens::Color> {
    let t = t.clamp(0.0, 1.0);
    match (rest, hot) {
        (None, None) => None,
        // ⚠️ Um lado ausente é **transparente**, não "a outra cor": um botão Default em repouso
        // não tem fundo, e o hover dele tem de EMERGIR do nada em vez de aparecer de repente.
        (Some(a), None) => Some(fade(a, 1.0 - t)),
        (None, Some(b)) => Some(fade(b, t)),
        (Some(a), Some(b)) => Some(ph2d_tokens::Color {
            r: mix(a.r, b.r, t),
            g: mix(a.g, b.g, t),
            b: mix(a.b, b.b, t),
            a: mix(a.a, b.a, t),
        }),
    }
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn mix(a: u8, b: u8, t: f32) -> u8 {
    (f32::from(a) + (f32::from(b) - f32::from(a)) * t)
        .round()
        .clamp(0.0, 255.0) as u8
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn fade(c: ph2d_tokens::Color, t: f32) -> ph2d_tokens::Color {
    ph2d_tokens::Color {
        a: (f32::from(c.a) * t).round().clamp(0.0, 255.0) as u8,
        ..c
    }
}
