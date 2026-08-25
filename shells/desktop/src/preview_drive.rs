//! **Estado de PRÉ-VISUALIZAÇÃO contra estado de DOCUMENTO** — o conceito que faltava ao undo.
//!
//! ⚠️ Autorizado pelo Enio em 2026-08-23: *«precisamos corrigir o CtrlZ para ambas»* — **ambas** =
//! a §11 Animation e a física. A auditoria
//! [`21 §4`](../../../docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md) mediu que **não
//! é defeito de nenhum dos dois módulos**: é do modelo do undo, e a cura é este conceito.
//!
//! # O defeito, medido
//!
//! O [`crate::App::post_frame_undo`] regista por **diff**: num quadro com input, se o estado do
//! projeto difere do baseline, o baseline vira um passo. Enquanto **alguma coisa se move sozinha**
//! — uma animação de sprite a tocar, o solver a simular — o diff é não-vazio por razão nenhuma do
//! artista, e o passo registado tem por conteúdo *só o relógio* ou *só a pose do solver*.
//!
//! ⚠️ **A frequência não é «por quadro», é «por CLIQUE»** — e é isso que o torna visível. O
//! `any_input_this_frame` **não** é levantado por mover o cursor (medido: só clique, tecla e roda),
//! então nada acontece enquanto o artista olha. Mas cada clique enquanto algo corre empilha um
//! passo cujo Ctrl+Z **não faz nada visível** — vinte cliques, vinte Ctrl+Z mudos. É exactamente
//! assim que *«o Ctrl+Z não funciona»* se sente do lado de fora.
//!
//! # A lei
//!
//! > **O documento é o valor AUTORADO. O que um motor está a escrever agora é pré-visualização:
//! > vê-se, não se guarda nem se desfaz.**
//!
//! O motor continua a escrever no mundo (um só sink, e o render continua a ler o mesmo campo que
//! sempre leu — ⛔ *nada aqui cria uma segunda fonte de verdade para o que se pinta*). O que muda é
//! a **captura**: [`crate::App::capture_project`] repõe o valor autorado durante a fotografia e
//! devolve o vivo logo a seguir, então o `ProjectState` — que é a unidade do undo **e** do save —
//! descreve o documento, nunca o instante da corrida.
//!
//! ⚠️ **Undo e save partilham a captura de propósito** (é a lei do [`crate::undo`]), e isto vale
//! para os dois: gravar a meio de uma reprodução guarda a célula que o artista escolheu, não onde
//! o ciclo calhou estar. Para a física é a mesma frase com o nome que o
//! [ADR-0131](../../../docs/architecture/decisions/0131-physics-global-runtime-truth-rapier-ecs-bridge.md)
//! já lhe dá: *runtime-truth + **bake opcional*** — uma corrida por assar é pré-visualização.
//!
//! # Porque não as outras duas saídas (as três foram medidas)
//!
//! * **Suprimir o undo enquanto toca** (como o `ui_state_live` faz para uma transição de 150 ms):
//!   ⛔ uma animação em ciclo **nunca** pára — o artista pintaria dez minutos com o baseline preso
//!   antes da reprodução, e um Ctrl+Z levaria tudo. (Medido: o tique da §11 anda no passo fixo do
//!   relógio de parede, **não** no `playhead`, por isso «enquanto toca» não tem fim.)
//! * **Tirar o relógio do componente registado** (a lei da linha de física — *config, nunca estado
//!   vivo de solver*): ⛔ **necessária e NÃO suficiente**, e a conta é esta — o passo não nasce por
//!   quadro, nasce por clique, e no instante do clique o `Sprite::frame` quase de certeza já
//!   avançou desde o último baseline. Tirar o relógio e deixar o índice deixa o defeito inteiro de
//!   pé, e ainda move o `PROJECT_SCHEMA`.
//!
//! ⇒ Fica esta, que é a única das três que trata os dois casos com **um** mecanismo — e eles são
//! **dois casos diferentes**, como a auditoria mandou verificar: o relógio da §11 *nunca foi
//! documento* (não há UI que o escreva), a pose do solver *é documento com um escritor a mais*.
//!
//! # O que o motor tem de dizer
//!
//! Um motor conduz uma entidade quando **ele** escreveu o facto neste quadro. Ele diz-o com
//! [`PreviewDrive::driven`], passando o valor **antes** e **depois** da sua escrita — e é daí que
//! saem as duas metades da lei:
//!
//! 1. o **autorado** é o `before` da PRIMEIRA vez (insere-se só se ausente);
//! 2. ⚠️ se o `before` deste quadro não é o que o motor deixou no anterior, **outra mão escreveu
//!    entre os dois** — e o autorado passa a ser essa mão. Sem isto, editar a pose de um corpo a
//!    meio de uma corrida seria engolido pelo memo do início dela.
//!
//! E [`PreviewDrive::settle`], uma vez por quadro no `post_frame_undo`, esquece quem **não** foi
//! declarado: o motor largou aquela entidade, o valor vivo volta a ser documento, e a corrida
//! inteira colapsa em **UM** passo — que é o passo certo, *«desfaz a corrida»*.

use ph2d_ecs::{Entity, SimWorld, Transform};
use ph2d_render::Sprite;
use std::collections::BTreeMap;

/// **Qual motor conduz.** Duas entradas para a MESMA entidade não colidem — um corpo rígido com
/// sprite animada é o caso normal, não a excepção.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub(crate) enum Driver {
    /// O relógio da §11 Animation e o índice de célula que ele produz.
    SpriteAnim,
    /// A pose que o solver escreve enquanto o mundo corre (ADR-0131) — e a que as curvas da
    /// timeline escrevem, que é o mesmo facto vindo de outro motor.
    SolverPose,
    /// A **opacidade** que uma curva de `Opacity` escreve (`Sprite::tint[3]`).
    SpriteAlpha,
    /// O `t` de um `VecMorph` que uma curva de `Morph` escreve.
    MorphT,
    /// **O PAR de fontes de um `VecMorph` que a máquina de estados troca** (plano 32 W5).
    ///
    /// ⚠️ **Um segundo motor sobre o MESMO componente, e é seguro porque os campos NÃO se
    /// cruzam:** a curva da timeline escreve o `t`, a máquina escreve o `sources` (e também o
    /// `t`, pelo `MorphT`). É a mesma forma do par `SpriteAnim`/`SpriteAlpha` sobre a `Sprite` —
    /// e ali o doc avisa que *duas granularidades sobre o mesmo componente escreveriam por cima
    /// uma da outra*. ⇒ a fronteira aqui é o CAMPO, e ela é disjunta de propósito.
    MorphPair,
    /// Os parâmetros de um `PhysicsJoint` que as curvas de joint escrevem.
    JointParams,
}

/// **O FACTO que um motor escreve** — o recorte exacto do componente que é dele, e nada mais.
///
/// ⚠️ **A granularidade é o CAMPO, e tinha de ser.** O `SpriteAnimator` guarda o relógio
/// (`elapsed_ticks`/`pingpong_reverse`/`repeat_count`) ao lado do que o artista autora
/// (`playing`/`speed_q16`/`current`/…). Repor o componente INTEIRO engoliria uma mexida na
/// velocidade feita a meio da reprodução; repor os três campos do relógio deixa-a passar.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum Driven {
    /// Os três campos de relógio do `SpriteAnimator` + o índice de célula que ele escreve no
    /// `Sprite::frame`. ⚠️ O índice vem junto **de propósito**: ele é o único sink vivo da §11, e
    /// separá-lo daria uma captura em que o relógio é autorado e a célula não.
    SpriteAnim {
        elapsed_ticks: u64,
        pingpong_reverse: bool,
        repeat_count: u32,
        frame: u32,
    },
    /// A pose de um corpo rígido — ou de um objeto keyado.
    SolverPose(Transform),
    /// ⚠️ **Um `f32`, e não o `Sprite` inteiro.** A §11 já conduz aquele componente pelo `frame`,
    /// e duas granularidades sobre o mesmo componente escreveriam por cima uma da outra — a que
    /// corresse por último ganhava. O que a curva de `Opacity` escreve é **um** número
    /// (`tint[3]`), então é um número que se guarda.
    SpriteAlpha(f32),
    /// O `t` de um `VecMorph` — também **um** número, pela mesma razão.
    MorphT(f32),
    /// **As duas fontes de um `VecMorph`** — o par que a máquina de estados troca a cada
    /// transição. ⚠️ Sem ele, mudar de par durante a pré-visualização entraria no undo: o `MorphT`
    /// cobre o `t` e **só** o `t`.
    MorphPair([u64; 2]),
    /// ⚠️ O joint vai **inteiro**, e aqui isso é correcto: nenhum outro motor o conduz por campo, e
    /// a curva pode keyar qualquer um dos parâmetros dele (o alvo do servo, a taxa, os
    /// comprimentos). Guardar o struct é mais barato que uma variante por campo, e ele é `Copy`.
    JointParams(ph2d_physics_ecs::PhysicsJoint),
}

impl Driven {
    /// Qual motor é dono deste facto — a chave do ledger sai daqui, não de um argumento a mais.
    pub(crate) fn driver(self) -> Driver {
        match self {
            Self::SpriteAnim { .. } => Driver::SpriteAnim,
            Self::SolverPose(_) => Driver::SolverPose,
            Self::SpriteAlpha(_) => Driver::SpriteAlpha,
            Self::MorphT(_) => Driver::MorphT,
            Self::MorphPair(_) => Driver::MorphPair,
            Self::JointParams(_) => Driver::JointParams,
        }
    }

    /// Lê do mundo o facto que `driver` conduz em `entity`. `None` = a entidade não o tem (foi
    /// despawnada por um restore, ou nunca teve o componente).
    #[must_use]
    pub(crate) fn read(driver: Driver, sim: &SimWorld, entity: Entity) -> Option<Self> {
        match driver {
            Driver::SpriteAnim => {
                let a = sim.world().get::<ph2d_ecs::SpriteAnimator>(entity)?;
                let s = sim.world().get::<Sprite>(entity)?;
                Some(Self::SpriteAnim {
                    elapsed_ticks: a.elapsed_ticks,
                    pingpong_reverse: a.pingpong_reverse,
                    repeat_count: a.repeat_count,
                    frame: s.frame,
                })
            }
            Driver::SolverPose => Some(Self::SolverPose(*sim.world().get::<Transform>(entity)?)),
            Driver::SpriteAlpha => Some(Self::SpriteAlpha(
                sim.world().get::<Sprite>(entity)?.tint[3],
            )),
            Driver::MorphT => Some(Self::MorphT(
                sim.world().get::<ph2d_ecs::VecMorph>(entity)?.t,
            )),
            Driver::MorphPair => Some(Self::MorphPair(
                sim.world().get::<ph2d_ecs::VecMorph>(entity)?.sources,
            )),
            Driver::JointParams => Some(Self::JointParams(
                *sim.world().get::<ph2d_physics_ecs::PhysicsJoint>(entity)?,
            )),
        }
    }

    /// Escreve este facto no mundo. Entidade sem o componente = no-op silencioso (o restore do
    /// undo respawna tudo com bits novos, e um memo pode sobreviver-lhe um quadro).
    ///
    /// ⚠️ **Escreve só quando MUDA**, pela razão de sempre nesta casa: o `bevy` marca a alteração
    /// no `deref_mut`, e um componente tocado todo o quadro é ruído para quem lê `Changed<…>`.
    fn write(self, sim: &mut SimWorld, entity: Entity) {
        match self {
            Self::SpriteAnim {
                elapsed_ticks,
                pingpong_reverse,
                repeat_count,
                frame,
            } => {
                if let Some(mut a) = sim.world_mut().get_mut::<ph2d_ecs::SpriteAnimator>(entity) {
                    if a.elapsed_ticks != elapsed_ticks {
                        a.elapsed_ticks = elapsed_ticks;
                    }
                    if a.pingpong_reverse != pingpong_reverse {
                        a.pingpong_reverse = pingpong_reverse;
                    }
                    if a.repeat_count != repeat_count {
                        a.repeat_count = repeat_count;
                    }
                }
                if let Some(mut s) = sim.world_mut().get_mut::<Sprite>(entity)
                    && s.frame != frame
                {
                    s.frame = frame;
                }
            }
            Self::SolverPose(pose) => {
                if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity)
                    && *t != pose
                {
                    *t = pose;
                }
            }
            Self::SpriteAlpha(a) => {
                if let Some(mut s) = sim.world_mut().get_mut::<Sprite>(entity)
                    && s.tint[3] != a
                {
                    s.tint[3] = a;
                }
            }
            Self::MorphT(v) => {
                if let Some(mut m) = sim.world_mut().get_mut::<ph2d_ecs::VecMorph>(entity)
                    && m.t != v
                {
                    m.t = v;
                }
            }
            Self::MorphPair(v) => {
                if let Some(mut m) = sim.world_mut().get_mut::<ph2d_ecs::VecMorph>(entity)
                    && m.sources != v
                {
                    m.sources = v;
                }
            }
            Self::JointParams(j) => {
                if let Some(mut cur) = sim
                    .world_mut()
                    .get_mut::<ph2d_physics_ecs::PhysicsJoint>(entity)
                    && *cur != j
                {
                    *cur = j;
                }
            }
        }
    }
}

/// Uma entidade sob condução: o que o artista autorou, e o que o motor deixou no último quadro.
#[derive(Clone, Copy, Debug)]
struct Entry {
    /// O valor que o documento tem enquanto o motor conduz.
    authored: Driven,
    /// O que o motor escreveu da última vez — a referência que denuncia **outra mão**.
    last_written: Driven,
    /// Declarado neste quadro? O [`PreviewDrive::settle`] esquece quem não foi.
    seen: bool,
}

/// **O ledger da condução** — quem está a ser escrito por um motor, e qual era o valor autorado.
///
/// Vive no [`crate::App`], é declarado pelos motores e consumido por um sítio só (a captura).
/// ⚠️ **`BTreeMap`, nunca `HashMap`** — a ordem da substituição atravessa a captura, que é a
/// unidade do undo *e* do save; a espinha do determinismo desta casa não se quebra por conveniência.
#[derive(Default)]
pub(crate) struct PreviewDrive {
    memo: BTreeMap<(u64, Driver), Entry>,
}

impl PreviewDrive {
    /// **O motor conduziu esta entidade neste quadro.** `before` é o que lá estava quando ele
    /// pegou nela; `after` o que ele deixou.
    ///
    /// ⚠️ Chame **só quando o motor de facto escreveu** (`before != after`): declarar uma entidade
    /// que ninguém mexeu faz a substituição repor um valor idêntico — inofensivo — mas mantém viva
    /// uma condução que já acabou, e é a `settle` que precisa de a ver morrer.
    pub(crate) fn driven(&mut self, entity: Entity, before: Driven, after: Driven) {
        let key = (entity.to_bits(), after.driver());
        match self.memo.get_mut(&key) {
            None => {
                self.memo.insert(
                    key,
                    Entry {
                        authored: before,
                        last_written: after,
                        seen: true,
                    },
                );
            }
            Some(e) => {
                // ⚠️ **Outra mão escreveu entre os dois quadros.** O que o motor encontrou não é o
                // que ele deixou ⇒ alguém autorou por cima, e o documento passa a ser essa mão.
                // Sem esta linha, editar a pose de um corpo a meio de uma corrida (ou o `frame`
                // por um caminho que não pause) ficaria para sempre por baixo do memo do início.
                if e.last_written != before {
                    e.authored = before;
                }
                e.last_written = after;
                e.seen = true;
            }
        }
    }

    /// **Esquece quem deixou de ser conduzido.** Uma vez por quadro, no topo do
    /// `post_frame_undo` — antes da captura, para que a fotografia deste quadro já veja o vivo de
    /// quem parou.
    ///
    /// É isto que faz a corrida colapsar em **UM** passo: enquanto o motor conduz não há passo
    /// nenhum; quando ele larga, a captura seguinte vê o valor vivo, difere do baseline (que é o
    /// pré-corrida) e regista um — *«desfaz a corrida»*.
    pub(crate) fn settle(&mut self) {
        self.memo.retain(|_, e| e.seen);
        for e in self.memo.values_mut() {
            e.seen = false;
        }
    }

    /// Nada sob condução? Então a captura não paga nada — nem uma varredura.
    ///
    /// ⚠️ `cfg(test)`: no produto quem responde a esta pergunta é a própria
    /// [`Self::substitute_authored`], que sai cedo. Deixá-la `pub(crate)` sem chamador daria um
    /// aviso do clippy — e um método que só os gates usam é exactamente o que o aviso nomeia.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.memo.is_empty()
    }

    /// Quantas entidades estão sob condução (diagnóstico do `PH2D_UNDO_LOG`).
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.memo.len()
    }

    /// **Põe o mundo no estado AUTORADO** e devolve o vivo que deslocou, para o
    /// [`Self::restore_live`] repor.
    ///
    /// ⚠️ A substituição é no MUNDO, e não no snapshot já serializado, por uma razão de exactidão:
    /// as linhas do `WorldSnapshot` são ordenadas por CONTEÚDO e não carregam a entidade, então
    /// mexer nelas seria adivinhar qual linha é de quem. Aqui a chave é a entidade, que é o que
    /// temos.
    #[must_use]
    pub(crate) fn substitute_authored(&self, sim: &mut SimWorld) -> Vec<((u64, Driver), Driven)> {
        if self.memo.is_empty() {
            return Vec::new(); // o caso normal: nem uma varredura
        }
        let mut live = Vec::with_capacity(self.memo.len());
        for (&(bits, driver), entry) in &self.memo {
            let entity = Entity::from_bits(bits);
            let Some(now) = Driven::read(driver, sim, entity) else {
                continue; // a entidade morreu debaixo do memo; a `settle` limpa-o a seguir
            };
            live.push(((bits, driver), now));
            entry.authored.write(sim, entity);
        }
        live
    }

    /// Devolve ao mundo o que a [`Self::substitute_authored`] deslocou. Ordem inversa não importa
    /// (uma entrada por entidade-e-motor), mas o par tem de correr **sempre**: sair a meio deixaria
    /// a cena a mostrar o autorado em vez do vivo, e o artista veria a animação saltar para trás.
    pub(crate) fn restore_live(sim: &mut SimWorld, live: &[((u64, Driver), Driven)]) {
        for &((bits, _), value) in live {
            value.write(sim, Entity::from_bits(bits));
        }
    }
}

#[cfg(test)]
#[path = "preview_drive_tests.rs"]
mod tests;
