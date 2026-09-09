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
    /// ⭐ O **PALCO** do *Edit Prefab*: enquanto a receita está aberta, a pose de mundo dela é da
    /// VISTA. Ver [`Driven::StagePose`].
    PrefabStage,
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
    /// ⭐⭐⭐ **A VISIBILIDADE que uma acção de sinal escreve** (TOP-20 #5).
    ///
    /// ⚠️ **Sem isto, cada porta que abre é um passo de `Ctrl+Z`.** A `Visibility` é um componente
    /// REGISTADO — o que o artista autora — e um sinal a escondê-la é o motor, não a mão. É a
    /// mesma fronteira que o `Timer` não precisou de declarar (o relógio dele não é registado) e
    /// que esta metade da família precisa.
    SignalVisibility,
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
    /// ⭐⭐⭐ **A pose de uma receita enquanto ela está no PALCO** (o *Edit Prefab*, 2026-09-07).
    ///
    /// ⚠️ **Mesmo componente do [`Self::SolverPose`], driver DIFERENTE, e é isso que importa:** a
    /// chave do ledger é `(entidade, driver)`, e uma receita cuja raiz seja também um corpo
    /// dinâmico é conduzida pelos dois. Reusar o driver do solver faria o palco e a corrida
    /// escreverem na mesma entrada, e o `authored` de um apagaria o do outro.
    StagePose(Transform),
    /// ⭐ **Escondido ou visível** — o `bool` da [`ph2d_ecs::Visibility`], e nada mais.
    ///
    /// ⚠️ **Um `bool` e não o componente**, pela lei do recorte: a `Visibility` tem UM campo hoje,
    /// e guardar o struct faria um campo novo dela entrar na pré-visualização sem ninguém decidir.
    Visible(bool),
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
            Self::StagePose(_) => Driver::PrefabStage,
            Self::Visible(_) => Driver::SignalVisibility,
        }
    }

    /// Lê do mundo o facto que `driver` conduz em `entity`. `None` = a entidade não o tem (foi
    /// despawnada por um restore, ou nunca teve o componente).
    #[must_use]
    pub(crate) fn read(driver: Driver, sim: &SimWorld, entity: Entity) -> Option<Self> {
        match driver {
            Driver::SpriteAnim => {
                let a = sim.world().get::<ph2d_ecs::SpriteAnimator>(entity)?;
                // ⚠️ **A célula viva mudou de casa** (ADR-0164 F1 passo 6): era `Sprite::frame`,
                // hoje é `SpriteGrid::frame`. Uma sprite SEM grelha não tem célula que o motor
                // possa mexer, então não há facto de pré-visualização a guardar — e o `?` aqui é
                // a leitura certa disso, não uma falha.
                let g = sim.world().get::<ph2d_ecs::SpriteGrid>(entity)?;
                Some(Self::SpriteAnim {
                    elapsed_ticks: a.elapsed_ticks,
                    pingpong_reverse: a.pingpong_reverse,
                    repeat_count: a.repeat_count,
                    frame: g.frame,
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
            Driver::PrefabStage => Some(Self::StagePose(*sim.world().get::<Transform>(entity)?)),
            Driver::SignalVisibility => Some(Self::Visible(
                sim.world().get::<ph2d_ecs::Visibility>(entity)?.hidden,
            )),
        }
    }

    /// Escreve este facto no mundo. Entidade sem o componente = no-op silencioso (o restore do
    /// undo respawna tudo com bits novos, e um memo pode sobreviver-lhe um quadro).
    ///
    /// ⚠️ **Escreve só quando MUDA**, pela razão de sempre nesta casa: o `bevy` marca a alteração
    /// no `deref_mut`, e um componente tocado todo o quadro é ruído para quem lê `Changed<…>`.
    ///
    /// ⚠️ **`pub(crate)` desde 2026-09-07, e com uma obrigação colada:** quem escreve um facto de
    /// pré-visualização por aqui tem de o **declarar** ([`PreviewDrive::driven`]) no mesmo quadro,
    /// senão a `settle` esquece-o e o valor de pré-visualização vira documento. O primeiro
    /// consumidor de fora é o palco do *Edit Prefab* ([`crate::prefab_stage`]), que escreve a pose
    /// de palco e a repõe ao fechar — e usar esta porta é o que o impede de ter a sua própria
    /// versão da regra *«só quando muda»*.
    pub(crate) fn write(self, sim: &mut SimWorld, entity: Entity) {
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
                if let Some(mut g) = sim.world_mut().get_mut::<ph2d_ecs::SpriteGrid>(entity)
                    && g.frame != frame
                {
                    g.frame = frame;
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
            Self::StagePose(pose) => {
                if let Some(mut t) = sim.world_mut().get_mut::<Transform>(entity)
                    && *t != pose
                {
                    *t = pose;
                }
            }
            Self::Visible(hidden) => {
                if let Some(mut v) = sim.world_mut().get_mut::<ph2d_ecs::Visibility>(entity)
                    && v.hidden != hidden
                {
                    v.hidden = hidden;
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

    /// ⭐⭐⭐ **AINDA ESTOU A CONDUZIR, e o valor não mudou.** `true` se havia condução a manter.
    ///
    /// # ⛔⛔ O defeito que ela fecha (report do dono, 2026-09-09)
    ///
    /// *«Remove Smart Bone não devolve o objeto animado à posição inicial»* — e a medição foi
    /// inequívoca: com o osso parado, o ledger largava o objecto **no quadro seguinte**.
    ///
    /// | quadro | `x` | o ledger conduz? |
    /// |---|---|---|
    /// | 0 | 10,000 | **sim** |
    /// | 1..4 | 10,000 | **não** |
    ///
    /// ⚠️ **A causa é uma lei certa aplicada a um motor de outra espécie.** Quem declara só o que
    /// MUDOU está certo para a **timeline**: uma reprodução que pára tem de deixar o valor virar
    /// documento, e é isso que faz *«desfazer a corrida»* ser **um** passo. Mas um **condutor
    /// PERSISTENTE** — uma âncora de IK, um osso inteligente — nunca «pára»: ele escreve todo
    /// quadro, e o output dele é **constante na maior parte do tempo**. A [`Self::settle`] lê essa
    /// constância como *«o motor largou»* e promove a pré-visualização a documento.
    ///
    /// ⇒ *para um condutor persistente, «não mudou» e «acabou» são factos diferentes com a mesma
    /// forma* — e é esta porta que os separa.
    ///
    /// ⛔ **Ela NUNCA cria uma entrada.** Sem entrada não há condução a manter: o motor está a
    /// escrever exactamente o que o documento já diz, e não há nada para devolver.
    pub(crate) fn still_driving(&mut self, entity: Entity, driver: Driver) -> bool {
        match self.memo.get_mut(&(entity.to_bits(), driver)) {
            Some(e) => {
                e.seen = true;
                true
            }
            None => false,
        }
    }

    /// ⭐⭐⭐ **DEVOLVE O AUTORADO E LARGA** — para um motor que é DESLIGADO, não que apenas parou.
    ///
    /// # ⚠️ Por que a [`Self::settle`] não serve aqui
    ///
    /// Ela trata de um motor que **largou**: o valor vivo passa a ser documento, e é isso que faz
    /// uma corrida colapsar num passo (*«desfaz a corrida»*). Mas quando o artista **apaga** a
    /// restrição, o valor vivo é o que ela escreveu — e promovê-lo a documento é exactamente o que
    /// este módulo existe para impedir: *o que um motor escreveu vê-se, não se guarda*.
    ///
    /// ⛔ **Sem isto, apagar a restrição ASSA a pose dela no documento, em silêncio.** É o report
    /// de 2026-09-07 (*«Remove IK … não funciona plenamente»*): o verbo tirava a âncora e deixava a
    /// corrente dobrada onde a âncora a tinha posto, sem forma de voltar. O Blender faz o contrário
    /// — remover a *constraint* devolve o osso à pose que o artista autorou.
    ///
    /// Devolve `true` se havia condução a devolver.
    pub(crate) fn release_to_authored(
        &mut self,
        sim: &mut SimWorld,
        entity: Entity,
        driver: Driver,
    ) -> bool {
        let Some(e) = self.memo.remove(&(entity.to_bits(), driver)) else {
            return false;
        };
        e.authored.write(sim, entity);
        true
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

    /// ⭐⭐⭐ **UM MOTOR ESTÁ A CONDUZIR ESTA ENTIDADE NESTE QUADRO?**
    ///
    /// ⛔⛔ **Ela existe por um laço fechado que a auditoria de 2026-09-08 mediu.** O cabeçalho do
    /// `autokey_pass` declara a invariante que o protege — *«the apply pass has already written the
    /// document's value to the world … world == curve and keys nothing — no feedback loop»* — e ela
    /// exige que **ninguém escreva pose entre o apply e o autokey**. Os passes do esqueleto (o osso
    /// inteligente, a âncora de IK) escrevem exactamente aí. Com o objecto conduzido **seleccionado**
    /// e o AutoKey armado, o autokey lê a saída do motor, ela difere da curva do clip activo, e ele
    /// cunha uma chave **a partir do que o motor está a mostrar**.
    ///
    /// ⇒ *o que um motor conduz é pré-visualização, e pré-visualização não é autoria.* Este ledger
    /// já sabe exactamente quem está sob condução — faltava alguém perguntar-lhe.
    ///
    /// ⚠️ Ela responde por ENTIDADE e não por `(entidade, driver)`: a pergunta do autokey é *«esta
    /// pose é do artista?»*, e basta um motor a conduzir para a resposta ser não.
    ///
    /// ⚠️⚠️ **Ela nasceu no sítio errado, e o defeito foi o que a auditoria acabara de nomear:** a
    /// 1.ª inserção caiu ENTRE o `#[cfg(test)]` da vizinha e a vizinha, e o método herdou-o — só
    /// existia em `cfg(test)`, e o produto não compilava. *Um item novo colado a um atributo rouba-o
    /// ao dono.*
    #[must_use]
    pub(crate) fn drives(&self, entity: u64) -> bool {
        self.memo.keys().any(|(bits, _)| *bits == entity)
    }

    /// ⭐ **QUE MOTORES conduzem esta entidade agora** — a lista, para quem precisa de a **largar**
    /// e não só de saber que ela existe.
    ///
    /// ⚠️ Ela é o oráculo do censo que ata [`crate::timeline_preview::DRIVERS`] ao que o
    /// `declare_timeline_writes` de facto escreve: *uma lista escrita à mão ao lado de um produtor
    /// é a segunda resposta à mesma pergunta, e a que envelhece é a escrita à mão*.
    ///
    /// ⚠️ `cfg(test)` pela razão do [`Self::is_empty`]: no produto quem percorre os motores é a
    /// própria lista nomeada, e um método que só os gates usam é exactamente o que o clippy nomeia.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn drivers_of(&self, entity: u64) -> Vec<Driver> {
        self.memo
            .keys()
            .filter(|(bits, _)| *bits == entity)
            .map(|(_, d)| *d)
            .collect()
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
