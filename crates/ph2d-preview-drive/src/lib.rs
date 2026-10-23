//! **Estado de PRÉ-VISUALIZAÇÃO contra estado de DOCUMENTO** — o conceito que faltava ao undo.
//!
//! ⚠️ Autorizado pelo Enio em 2026-08-23: *«precisamos corrigir o CtrlZ para ambas»* — **ambas** =
//! a §11 Animation e a física. A auditoria
//! [`21 §4`](../../../docs/Sprite_projeto/21_auditoria_da_animacao_2026-08-23.md) mediu que **não
//! é defeito de nenhum dos dois módulos**: é do modelo do undo, e a cura é este conceito.
//!
//! # O defeito, medido
//!
//! O o `App::post_frame_undo` da shell regista por **diff**: num quadro com input, se o estado do
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
//! a **captura**: o `App::capture_project` da shell repõe o valor autorado durante a fotografia e
//! devolve o vivo logo a seguir, então o `ProjectState` — que é a unidade do undo **e** do save —
//! descreve o documento, nunca o instante da corrida.
//!
//! ⚠️ **Undo e save partilham a captura de propósito** (é a lei do `undo` da shell), e isto vale
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
pub enum Driver {
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
    /// ⭐⭐⭐ **A pose que um SCRIPT do artista escreve durante a corrida** (TOP-20 #16).
    ///
    /// ⚠️ **Não é o [`Self::SolverPose`], e a razão é a do [`Self::PrefabStage`]:** a chave do ledger
    /// é `(entidade, driver)`, e um objecto que seja também um corpo teria duas mãos na mesma
    /// entrada — o *«outra mão escreveu»* de uma engoliria o autorado da outra.
    ///
    /// ⚠️ **E ele é um condutor PERSISTENTE durante a corrida inteira** (a ponte declara-o também nos
    /// quadros em que o script não escreveu, e com a corrida pausada): um script que PAROU de mexer
    /// não ACABOU — sem isso a `settle` promovia a pose da corrida a documento, e o rebobinar já não
    /// tinha para onde voltar.
    ScriptPose,
    /// ⭐⭐⭐ **A pose da RAIZ de um HUD** (TOP-20 #20) — ela é reescrita a cada quadro para colar a
    /// caixa de referência à vista da câmera do jogo, e por isso **não é documento**: sem esta
    /// entrada, *«o HUD acompanhou a câmera»* seria um passo de `Ctrl+Z` por quadro.
    CanvasPose,
    /// ⭐⭐⭐ **A pose que a PARALAXE desloca** (plano 24, W1) — um objecto com `ScrollFactor` é
    /// re-colocado a cada quadro contra o centro da vista, e por isso **não é documento**: sem esta
    /// entrada, panhar a câmera num fundo seria um passo de `Ctrl+Z` por quadro (a mesma razão que
    /// o [`Self::CanvasPose`] escreve, e ela vale aqui letra por letra — um HUD é a paralaxe com
    /// fracção `0`).
    ParallaxPose,
    /// ⭐⭐⭐ **A pose que um TWEEN escreve** (suplente #22).
    ///
    /// ⚠️ **Não é o [`Self::SolverPose`], e a razão é a do [`Self::ScriptPose`] e a do
    /// [`Self::PrefabStage`]:** a chave do ledger é `(entidade, driver)`, e um objecto que seja
    /// também um corpo — ou que tenha uma curva de timeline — teria duas mãos na mesma entrada, e o
    /// *«outra mão escreveu»* de uma engoliria o autorado da outra.
    TweenPose,
    /// ⭐⭐ **A COR que um tween escreve** — o `self_tint` do `Sprite` e, com ele, o `tint_fill`.
    ///
    /// ⚠️ **Os DOIS campos numa entrada só, e é obrigatório:** eles são um facto — *«a silhueta
    /// está acesa nesta cor»* — e separá-los deixaria um `Rewind` a devolver a cor e a esquecer o
    /// interruptor, o que pinta o objecto como uma silhueta BRANCA em vez de o devolver à arte.
    ///
    /// ⛔ **E NÃO há uma entrada para a OPACIDADE**, que é o achado desta wave: o `tint[3]` já tem
    /// dono — o [`Self::SpriteAlpha`], que a curva de `Opacity` da timeline escreve. Um driver novo
    /// sobre o **mesmo campo** seria duas entradas para um valor, e o `authored` de uma apagaria o
    /// da outra; usar o que existe é a leitura certa, porque é o **mesmo facto vindo de outro
    /// motor** — exactamente o que o doc daquele driver já declara para o solver e a timeline.
    TweenTint,
    /// ⭐⭐ **A cor da sprite que as curvas da família OBJETO escrevem** (`ph2d_ecs::canal_objeto`):
    /// o RGB do `tint`, a opacidade final, o degradê dos cantos e a emissão. ⚠️ Os campos são
    /// DISJUNTOS dos outros drivers da sprite: o `tint[3]` é do [`Self::SpriteAlpha`] e o
    /// `self_tint`/`tint_fill` do [`Self::TweenTint`] (a curva usa-o — o mesmo facto, outro motor).
    SpriteCor,
    /// ⭐ **O lado B de uma junta preso ao MUNDO** que um canal JUNTA escreve — a presença do
    /// marcador `JointWorldAnchor`, ao lado do [`Self::JointParams`] (outro componente).
    PontaNoMundo,
    /// ⭐ **A roldana de uma corda** que os canais JUNTA da §13 escrevem: a `PulleyWheel` inteira
    /// e o marcador `WestonAxle` (a talha é a presença dele) — um facto só, como o
    /// [`Self::JointParams`].
    Roldana,
}

/// **O FACTO que um motor escreve** — o recorte exacto do componente que é dele, e nada mais.
///
/// ⚠️ **A granularidade é o CAMPO, e tinha de ser.** O `SpriteAnimator` guarda o relógio
/// (`elapsed_ticks`/`pingpong_reverse`/`repeat_count`) ao lado do que o artista autora
/// (`playing`/`speed_q16`/`current`/…). Repor o componente INTEIRO engoliria uma mexida na
/// velocidade feita a meio da reprodução; repor os três campos do relógio deixa-a passar.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Driven {
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
    /// ⭐ **A pose de um objecto movido por script** — ver [`Driver::ScriptPose`].
    ScriptPose(Transform),
    /// ⭐ A pose conduzida da raiz de um HUD — ver [`Driver::CanvasPose`].
    CanvasPose(Transform),
    /// ⭐ A pose deslocada pela paralaxe — ver [`Driver::ParallaxPose`].
    ParallaxPose(Transform),
    /// ⭐ A pose que um tween escreve — ver [`Driver::TweenPose`].
    TweenPose(Transform),
    /// ⭐⭐ A cor que um tween escreve — o `self_tint` **e** o `tint_fill`, que são um facto só.
    /// Ver [`Driver::TweenTint`].
    TweenTint {
        /// O `Sprite::self_tint`.
        self_tint: [f32; 4],
        /// O `Sprite::tint_fill` — o interruptor da silhueta.
        fill: bool,
    },
    /// ⭐⭐ A cor da sprite que um canal OBJETO escreve — ver [`Driver::SpriteCor`]. Os dois
    /// componentes opcionais vão como `Option`: repor um autorado AUSENTE é remover o componente
    /// (ausência = identidade, e o quadro volta byte-idêntico).
    SpriteCor {
        /// `Sprite::tint` r, g, b (o alfa é o [`Driven::SpriteAlpha`]).
        tint: [f32; 3],
        /// `Sprite::opacity`.
        opacidade: f32,
        /// `SpriteCornerTint`.
        cantos: Option<[[f32; 4]; 4]>,
        /// `SpriteEmissive`.
        emissao: Option<f32>,
    },
    /// ⭐ O lado B no mundo — ver [`Driver::PontaNoMundo`].
    PontaNoMundo(bool),
    /// ⭐ A roldana e a talha de Weston — ver [`Driver::Roldana`].
    Roldana {
        /// A `PulleyWheel`.
        roda: ph2d_physics_ecs::PulleyWheel,
        /// O marcador `WestonAxle` está lá.
        weston: bool,
    },
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
/// Vive no a `App` da shell, é declarado pelos motores e consumido por um sítio só (a captura).
/// ⚠️ **`BTreeMap`, nunca `HashMap`** — a ordem da substituição atravessa a captura, que é a
/// unidade do undo *e* do save; a espinha do determinismo desta casa não se quebra por conveniência.
#[derive(Default)]
pub struct PreviewDrive {
    memo: BTreeMap<(u64, Driver), Entry>,
}

impl PreviewDrive {
    /// **O motor conduziu esta entidade neste quadro.** `before` é o que lá estava quando ele
    /// pegou nela; `after` o que ele deixou.
    ///
    /// ⚠️ Chame **só quando o motor de facto escreveu** (`before != after`): declarar uma entidade
    /// que ninguém mexeu faz a substituição repor um valor idêntico — inofensivo — mas mantém viva
    /// uma condução que já acabou, e é a `settle` que precisa de a ver morrer.
    pub fn driven(&mut self, entity: Entity, before: Driven, after: Driven) {
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
    pub fn still_driving(&mut self, entity: Entity, driver: Driver) -> bool {
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
    pub fn release_to_authored(
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

    // ⛔⛔⛔ **RECUSA MEDIDA (2026-09-19): `release_all_to_authored` foi CONSTRUÍDA E RETIRADA.**
    //
    // Ela devolvia ao autorado **tudo** o que estivesse no memo, para o `SignalVerb::RestartRun`
    // curar *«as vidas voltaram a três e as luzes ficaram às escuras»*. Duas medições derrubaram-na,
    // e as duas valem para quem a quiser reconstruir:
    //
    // 1. ⛔ **Ela não cura o caso que a motivou.** O [`PreviewDrive::settle`] abaixo esquece quem
    //    não foi declarado neste quadro ⇒ uma escrita de UM TIRO (um `Hide`, um `Show`) sobrevive
    //    **dois quadros** e depois é DOCUMENTO — que é precisamente a lei que faz uma corrida
    //    colapsar em um passo de `Ctrl+Z`. *Um recomeço cinco segundos depois já não tem o que
    //    devolver.*
    // 2. ⛔⛔ **E ela LUTA contra o artista.** As linhas que a tabela corre no MESMO sinal já
    //    escreveram, e o «autorado» que o memo guarda por baixo delas é o valor de ANTES ⇒ a
    //    devolução desfaz o que o artista acabou de mandar fazer. *Uma porta que devolve «o que
    //    estava antes» não sabe distinguir o que a corrida escreveu do que o artista acabou de
    //    mandar escrever.*
    //
    // ⭐ O que ela ia comprar — a POSE de um corpo voltar ao princípio — já estava pago: a ponte da
    // física rebobina quando o tique recua. E a cura do caso real é a que o modelo prescreve:
    // **o que a corrida escreve, a corrida desfaz** (o verbo oposto, no mesmo sinal).

    /// **Esquece quem deixou de ser conduzido.** Uma vez por quadro, no topo do
    /// `post_frame_undo` — antes da captura, para que a fotografia deste quadro já veja o vivo de
    /// quem parou.
    ///
    /// É isto que faz a corrida colapsar em **UM** passo: enquanto o motor conduz não há passo
    /// nenhum; quando ele larga, a captura seguinte vê o valor vivo, difere do baseline (que é o
    /// pré-corrida) e regista um — *«desfaz a corrida»*.
    pub fn settle(&mut self) {
        self.memo.retain(|_, e| e.seen);
        for e in self.memo.values_mut() {
            e.seen = false;
        }
    }

    /// Nada sob condução? Então a captura não paga nada — nem uma varredura.
    ///
    /// ⛔⛔ **O `cfg(any(test, feature = "test-support"))` que ela tinha SAIU, e a razão é um
    /// vermelho que o portão do `ship` NÃO PODE VER:** com ele, a build **de omissão** não tem
    /// `is_empty` e a irmã [`Self::len`] — que é de produto — acorda o `clippy::len_without_is_empty`.
    /// O `ship.sh` corre o clippy **com features**, onde `is_empty` existe ⇒ *a configuração que
    /// reprova é exactamente a que o portão nunca compila*, e só o `-D warnings` do portão de fecho
    /// a alcança.
    ///
    /// ⚠️ **E a nota antiga justificava o `cfg` com *«sem chamador daria um aviso do clippy»*:**
    /// isso vale para um `pub(crate)` e **não** vale para um `pub` de biblioteca, que é superfície
    /// pública e nunca conta como morto. *Uma isenção escrita contra um aviso passa a produzir
    /// OUTRO no dia em que o vizinho muda de visibilidade.*
    ///
    /// ⭐ **DUAS linhas acharam isto no mesmo dia, uma sem saber da outra** (`line/Vector` e
    /// `line/components`, 2026-09-18) e escreveram-no com palavras diferentes — a integração de
    /// 20/09 fundiu as duas redacções numa. *Um defeito que duas linhas independentes tropeçam no
    /// mesmo dia não é azar: é uma cerca que o portão de cada uma não alcança.*
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.memo.is_empty()
    }

    /// Quantas entidades estão sob condução (diagnóstico do `PH2D_UNDO_LOG`).
    #[must_use]
    pub fn len(&self) -> usize {
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
    pub fn substitute_authored(&self, sim: &mut SimWorld) -> Vec<((u64, Driver), Driven)> {
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
    pub fn restore_live(sim: &mut SimWorld, live: &[((u64, Driver), Driven)]) {
        for &((bits, _), value) in live {
            value.write(sim, Entity::from_bits(bits));
        }
    }
}

/// ⭐ **As PERGUNTAS ao ledger** — o que ele responde sem mudar nada (quem conduz quem, e com que
/// valores). Irmão por tecto de LOC e por responsabilidade: as portas de cima MUDAM o ledger, estas
/// só o lêem (auditoria 26 da paralaxe acrescentou duas e o ficheiro passou os 700).
mod consultas;

/// ⭐ **O FACTO de cada motor** — `Driven::driver`/`read`/`write`, a ponte entre o ledger e os
/// componentes do mundo. Irmão por tecto de LOC e por responsabilidade: cresce uma variante por
/// motor novo (F25–F26 da UIUX passaram os 700), e o ledger de cima não muda com ele.
mod facto;

// ⚠️ **Os gates desta lei NÃO vivem aqui, e a ausência é a decisão** (HOWTO §1.2): cada um deles
// passa pela `ProjectState::capture` e pelo `tick_sprite_animations` da shell — eles medem *a
// captura repõe o autorado*, que é um facto da shell, não do memo. Ficaram em
// `shells/desktop/src/preview_drive_tests.rs`, declarados pelo `main.rs`.
