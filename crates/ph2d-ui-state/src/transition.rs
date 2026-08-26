//! **A TRANSIÇÃO** — o casamento pago uma vez, e o passo pago por frame.

use crate::pose::ObjectPose;
use ph2d_vec_blend::Plan;
use ph2d_vec_scene::VecPathId;

/// Meia volta, em radianos — a fronteira do arco mais curto.
const HALF_TURN: f64 = std::f64::consts::PI;
/// Uma volta.
const FULL_TURN: f64 = std::f64::consts::TAU;

/// O que acontece com UM objeto ao longo desta transição.
///
/// ⚠️ `Moving` é muito maior que os outros dois (ele guarda as DUAS poses), e a caixa fica nele em
/// vez de na variante: um `Box<Step>` custaria uma indireção por objeto no laço que roda POR
/// FRAME, para poupar bytes num vetor que tem dezenas de elementos e é construído uma vez.
#[allow(clippy::large_enum_variant)]
enum Step {
    /// Existe nos dois estados e alguma coisa difere.
    Moving {
        from: ObjectPose,
        to: ObjectPose,
        /// O casamento de forma, **só quando as geometrias diferem**. `None` é o caso comum e o
        /// barato — ver o doc da crate.
        shape: Option<Box<Plan>>,
    },
    /// Só no estado de origem: **sai**.
    Leaving(ObjectPose),
    /// Só no estado de destino: **entra**.
    Entering(ObjectPose),
}

/// **Uma forma a meio de uma troca de verbo booleano** — o recado que esta crate manda a quem
/// cozinha a booleana viva.
///
/// ⚠️ **Ela NÃO é serializada e não pode ser:** é *onde a cena está agora*, e o documento guarda
/// *onde as poses são*. É a mesma fronteira que mantém a [`super::Machine`] fora do arquivo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoolMorph {
    /// A forma. É por ela que quem cozinha descobre **qual grupo** está a mudar.
    pub id: VecPathId,
    /// O verbo próprio dela na CHEGADA. `None` = lá ela herda o do grupo. O de PARTIDA não viaja
    /// aqui: ele é o que o componente instalado já diz, e uma segunda cópia dele seria a que
    /// diverge.
    pub op: Option<u8>,
    /// O verbo do GRUPO na chegada. `None` = a pose de chegada não conhece grupo nenhum, e aí a
    /// operação do grupo fica onde está.
    pub group_op: Option<u8>,
    /// Onde no caminho, já clampado a `]0, 1[`.
    pub t: f64,
}

/// ⭐⭐⭐ **Um conjunto de Morph States a meio de uma troca de forma** — o recado que esta crate
/// manda a quem coze o morph (plano 32 W11c).
///
/// ⚠️ **Irmão exacto do [`BoolMorph`]**, e a duplicação é deliberada: são dois motores diferentes
/// do outro lado (a booleana viva e o `morph_live`), e uni-los obrigaria um deles a fingir ser o
/// outro. O que os torna a MESMA ideia é a forma: *esta crate sabe a única coisa que ninguém mais
/// sabe — de que estado para que estado, e a que altura do caminho — e entrega isso a quem coze.*
///
/// ⚠️ **Ela NÃO é serializada e não pode ser:** é *onde a cena está agora*, e o documento guarda
/// *onde as poses são*. É a mesma fronteira que mantém a [`super::Machine`] fora do arquivo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MorphStep {
    /// O conjunto de estados. É por ele que quem coze descobre **qual objecto** está a mudar.
    pub id: VecPathId,
    /// A forma de PARTIDA.
    pub from: VecPathId,
    /// A forma de CHEGADA.
    pub to: VecPathId,
    /// Onde no caminho, já clampado a `]0, 1[`.
    pub t: f64,
}

/// O casamento entre dois estados, computado UMA vez.
///
/// ⚠️ A forma desta API não é gosto: `ph2d_vec_blend::Plan::new` custa **13 079×** um passo, então
/// um `smart_animate(from, to, t)` que casasse a cada frame seria inutilizável. É o mesmo
/// `new`/`at` que o `Plan` já usa, e pelo mesmo motivo.
pub struct Transition {
    steps: Vec<Step>,
    plans_built: usize,
}

impl Transition {
    /// Casa as duas poses **por id** e prepara o que for preciso para andar.
    ///
    /// ⚠️ **Ela recebe LISTAS de pose e não [`UiState`](crate::UiState)s**, e a assinatura é
    /// parte do desenho: o lado de onde se PARTE é quase sempre a pose VIVA da cena — um estado a
    /// meio caminho de outro —, que não tem papel nenhum. Pedir um `UiState` obrigaria a
    /// inventar-lhe um, e o papel inventado seria uma mentira que alguém a jusante leria.
    ///
    /// ⚠️ Objetos **idênticos nos dois lados não entram** na transição. Não é otimização: é a
    /// afirmação de que *não animar* e *animar de x para x* são coisas diferentes — a segunda
    /// custaria um `Plan` e produziria trabalho por frame para não mover nada.
    #[must_use]
    pub fn new(from: &[ObjectPose], to: &[ObjectPose]) -> Self {
        let mut steps = Vec::new();
        let mut plans_built = 0;

        for a in from {
            match to.iter().find(|b| b.id == a.id) {
                Some(b) if a.is_same_as(b) => {}
                Some(b) => {
                    // O `Plan` é construído **só** quando as geometrias de facto diferem. Formas
                    // iguais (ou ausentes) atravessam sem pagar a busca de fase.
                    let shape = match (&a.geometry, &b.geometry) {
                        (Some(ga), Some(gb)) if !same_shape(ga, gb) => {
                            // ⚠️ **O casamento acontece na geometria COZIDA, e quem coze é o
                            // Blend** (`compound::rings` chama `cooked()`): é isso que faz um
                            // Fillet, um Chamfer ou um efeito da pilha VIAJAREM em vez de
                            // aparecerem de uma vez no fim — um raio de quina mora *dentro* do
                            // vértice, e duas fontes com o mesmo desenho de nós são idênticas.
                            //
                            // ⚠️ **E o cozimento NÃO é repetido aqui**, embora a tentação seja
                            // grande: um `cooked()` deste lado seria uma segunda resposta a
                            // *"a interpolação de forma vê a fonte ou o cozido?"*, e a mutação
                            // que a removeu não sangrou — porque ela não decidia nada. A
                            // decisão tem um dono (ADR-0121, a costura fonte≠cozido), e é lá
                            // que ela se muda.
                            let p = Plan::new(ga, gb).map(Box::new);
                            if p.is_some() {
                                plans_built += 1;
                            }
                            p
                        }
                        _ => None,
                    };
                    steps.push(Step::Moving {
                        from: a.clone(),
                        to: b.clone(),
                        shape,
                    });
                }
                None => steps.push(Step::Leaving(a.clone())),
            }
        }
        for b in to {
            if !from.iter().any(|a| a.id == b.id) {
                steps.push(Step::Entering(b.clone()));
            }
        }

        Self { steps, plans_built }
    }

    /// ⭐ **OS OPERANDOS A MEIO DE UMA TROCA DE VERBO**, no ponto `t` do caminho — vazio no caso
    /// comum, que é o de nada mudar.
    ///
    /// # Por que ela é uma pergunta SEPARADA de [`Self::at`]
    ///
    /// Uma pose descreve *um objeto*. O que muda quando o verbo troca não é um objeto: é **o que o
    /// GRUPO desenha**, e o grupo não tem pose (ele não tem `VecPathId`). Enfiar as duas pontas
    /// dentro da pose obrigaria o campo serializado a carregar um transitório — um valor que só
    /// tem sentido enquanto uma transição está no ar, dentro de um tipo que vai para o arquivo.
    ///
    /// ⚠️ **E esta crate não pode cozinhar a booleana:** ela não vê ECS, não sabe o que é um grupo
    /// e não conhece o motor. Ela sabe a única coisa que ninguém mais sabe — *de que verbo para
    /// que verbo, e a que altura do caminho* — e entrega isso a quem cozinha.
    ///
    /// ⚠️ **As pontas `t = 0` e `t = 1` devolvem VAZIO**, e não é economia: nelas o desenho é
    /// exatamente uma das duas pontas, que é o que o cozimento normal já produz a partir do
    /// componente instalado. Publicar um morph ali faria o quadro de chegada pagar dois
    /// cozimentos e um casamento para desenhar o que já estava na tela.
    #[must_use]
    pub fn bool_morphs(&self, t: f64) -> Vec<BoolMorph> {
        let tc = t.clamp(0.0, 1.0);
        if tc <= 0.0 || tc >= 1.0 {
            return Vec::new();
        }
        self.steps
            .iter()
            .filter_map(|s| match s {
                Step::Moving { from, to, .. }
                    if from.bool_op != to.bool_op || from.bool_group_op != to.bool_group_op =>
                {
                    Some(BoolMorph {
                        id: from.id,
                        op: to.bool_op,
                        group_op: to.bool_group_op,
                        t: tc,
                    })
                }
                _ => None,
            })
            .collect()
    }

    /// ⭐⭐⭐ **Os conjuntos de Morph States que TROCAM DE FORMA neste par**, com o `t` (W11c).
    ///
    /// ⚠️ **Irmã exacta do [`Self::bool_morphs`]**, e o doc dela vale palavra por palavra: uma pose
    /// descreve *um objecto*, e esta crate **não pode cozer o morph** — ela não vê ECS, não sabe o
    /// que é um conjunto e não conhece o motor. Ela sabe *de que forma para que forma, e a que
    /// altura*, e entrega isso a quem coze.
    ///
    /// ⚠️ **As pontas `t = 0` e `t = 1` devolvem VAZIO**, e não é economia: nelas o desenho é
    /// exactamente uma das duas formas, que é o que o cozimento normal já produz a partir do
    /// componente. Publicar um passo ali faria o quadro de chegada pagar um casamento para desenhar
    /// o que já estava na tela.
    ///
    /// ⛔ **Um lado sem forma (`None`) não entra.** `None` é *«não me pronuncio»*, e interpolar a
    /// partir dele obrigaria a inventar uma ponta — o objecto ficaria a saltar para a primeira
    /// forma da lista no dia em que alguém gravasse uma pose antes de ele ser um conjunto.
    #[must_use]
    pub fn morph_steps(&self, t: f64) -> Vec<MorphStep> {
        let tc = t.clamp(0.0, 1.0);
        if tc <= 0.0 || tc >= 1.0 {
            return Vec::new();
        }
        self.steps
            .iter()
            .filter_map(|s| match s {
                Step::Moving { from, to, .. } if from.morph_shape != to.morph_shape => {
                    Some(MorphStep {
                        id: from.id,
                        from: from.morph_shape?,
                        to: to.morph_shape?,
                        t: tc,
                    })
                }
                _ => None,
            })
            .collect()
    }

    /// **Quantos casamentos de forma este par custou.** Existe para o gate de custo: um par
    /// só-de-cor tem de responder `0`, e é esse zero que vale 12,79 ms numa cena de vinte objetos.
    #[must_use]
    pub fn plans_built(&self) -> usize {
        self.plans_built
    }

    /// Quantos objetos esta transição de facto move.
    #[must_use]
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// A pose de cada objeto em movimento, no ponto `t` do caminho.
    ///
    /// ⭐ **O clamp é POR CANAL, e a linha é *o que passar do alvo significa alguma coisa?*.**
    ///
    /// - **Posição e rotação** recebem o `t` CRU. Passar do alvo ali **é** o movimento: é o que um
    ///   `Back`/`Elastic` desenha, e é o que uma MOLA faz ao reverter (ela continua um instante
    ///   para onde ia, e só depois volta — isso é `t < 0`).
    /// - **Escala, opacidade, tinta e largura** são clampadas. Passar do alvo numa escala que vai
    ///   a zero **espelha o objeto**; numa opacidade, pede um alfa negativo. Não é overshoot, é
    ///   lixo.
    /// - **A geometria** é clampada, e é a razão original desta linha: um morph casado por
    ///   Hungarian não tem significado além do destino (a mesma escolha do `VecMorph`).
    ///
    /// ⚠️ **O clamp era GLOBAL, e isso custava duas coisas.** Medido: `Back Out` pica em **1,100**
    /// e `Elastic Out` em **1,3731** — os dois eram postos em 1,000, então o artista escolhia
    /// *Elastic* e via um botão que apenas chegava; metade do seletor de curva da W7c era um
    /// controle morto. E a MOLA não era entregável de todo: o primeiro quadro de uma reversão
    /// media **0,000000** de deslocamento — o objeto **congelava** em vez de carregar o momento,
    /// que é a única coisa que ela compra sobre uma curva.
    ///
    /// ⚠️ **Toda curva contida em `[0, 1]` é BYTE-IDÊNTICA** ao que já shipava — para ela o clamp
    /// nunca mordia. As que mudam são exatamente as duas cujo nome promete o que elas não
    /// entregavam.
    ///
    /// ⚠️ **A ROTAÇÃO vai pelo ARCO MAIS CURTO.** De 350° para 10° ela anda +20°, não −340°, que é
    /// o que qualquer ferramenta de UI faz e o que o artista espera de um par de estados. **A
    /// consequência é nomeada em vez de descoberta:** uma volta inteira autorada (0 → 360°) é o
    /// mesmo ângulo, logo ela **não gira** — girar N voltas é animação com percurso, e percurso é
    /// keyframe, não estado.
    #[must_use]
    pub fn at(&self, t: f64) -> Vec<ObjectPose> {
        // `tc` é o `t` dos canais onde passar do alvo não significa nada — ver o doc acima.
        let tc = t.clamp(0.0, 1.0);
        self.steps
            .iter()
            .map(|s| match s {
                Step::Moving { from, to, shape } => {
                    let mut p = ObjectPose {
                        id: from.id,
                        translation: [
                            lerp(from.translation[0], to.translation[0], t),
                            lerp(from.translation[1], to.translation[1], t),
                        ],
                        rotation: lerp_angle(from.rotation, to.rotation, t),
                        scale: [
                            lerp(from.scale[0], to.scale[0], tc),
                            lerp(from.scale[1], to.scale[1], tc),
                        ],
                        opacity: lerp_f32(from.opacity, to.opacity, tc),
                        // A TINTA vai SEMPRE pela porta do Blend, com forma ou sem ela — uma
                        // resposta só para *"como duas tintas interpolam neste app"*.
                        fill: ph2d_vec_blend::mix_paint(from.fill.as_ref(), to.fill.as_ref(), tc),
                        stroke: ph2d_vec_blend::mix_stroke(from.stroke, to.stroke, tc),
                        geometry: None,
                        // A LARGURA VIVA pela porta da crate que a define — e o `None` é o
                        // perfil uniforme, então um lado sem perfil é um lado com o perfil que
                        // não faz nada. Não há caso especial a escrever.
                        width: mix_width(from.width.as_ref(), to.width.as_ref(), tc),
                        // **Os FILTROS pela porta da folha que os define** — e o alinhamento é
                        // que carrega a lei: um degrau que só existe de um lado cresce do NEUTRO
                        // em vez de saltar, então acrescentar um blur depois de o Default já ter
                        // sido gravado anima na mesma (Enio, 2026-08-21). ⚠️ `tc`, e não `t`:
                        // um overshoot daria raio e intensidade NEGATIVOS, que é lixo e não
                        // exagero — a mesma razão da opacidade e da escala aqui em cima.
                        filters: ph2d_fx_op::mix_stacks(&from.filters, &to.filters, tc),
                        // ⚠️ **O VERBO é DISCRETO e ele SEGURA na ponta de PARTIDA.** Não há meio
                        // caminho entre `Union` e `Subtract`, e um número interpolado entre dois
                        // códigos daria a operação ERRADA — o `2` entre `Union` (0) e `Exclude`
                        // (3) é `Intersect`, que não está em nenhuma das duas pontas.
                        //
                        // ⭐ Quem desenha o meio é o COZIMENTO, e não este lerp: ele recebe as
                        // duas pontas por [`Transition::bool_morphs`], cozinha **as duas** com as
                        // formas onde elas estão AGORA, e morfa os dois RESULTADOS. É por isso
                        // que segurar aqui não é um degrau: é a metade honesta de uma resposta
                        // cuja outra metade mora onde a booleana de facto acontece.
                        bool_op: if tc >= 1.0 { to.bool_op } else { from.bool_op },
                        bool_group_op: if tc >= 1.0 {
                            to.bool_group_op
                        } else {
                            from.bool_group_op
                        },
                        // ⚠️ **A FORMA de um conjunto de estados é DISCRETA, e segura na ponta de
                        // PARTIDA** — exactamente a lei do `bool_op` acima, e pela mesma razão:
                        // não há meio caminho entre duas formas *nesta lista*, e um `VecPathId`
                        // interpolado entre dois ids seria o id de uma **terceira forma**, ou de
                        // nenhuma.
                        //
                        // ⭐ Quem desenha o meio é o motor do Morph, e não este lerp: ele recebe
                        // as duas pontas por [`Transition::morph_steps`] e interpola a GEOMETRIA
                        // das duas. Segurar aqui não é um degrau — é a metade honesta de uma
                        // resposta cuja outra metade mora onde o morph de facto acontece.
                        morph_shape: if tc >= 1.0 {
                            to.morph_shape
                        } else {
                            from.morph_shape
                        },
                    };
                    // ⚠️ E a forma que sai do `Plan` recebe a tinta da POSE, não a que o `Plan`
                    // interpolou por conta: se o objeto sai auto-consistente daqui, ninguém a
                    // jusante tem de decidir qual das duas vale.
                    //
                    // ⚠️ **Sem `Plan`, a forma é a de PARTIDA** — uma regra, dois casos. Formas
                    // iguais: `from` e `to` dão o mesmo desenho, e a escolha não é observável.
                    // Par degenerado (o `Plan` recusou): a forma **fica onde está** até a chegada
                    // a trocar, que é o que quem SAI e quem ENTRA já fazem — inventar um caminho
                    // que o motor não sabe traçar seria um salto no primeiro quadro.
                    p.geometry = match shape.as_ref() {
                        Some(plan) => {
                            let mut g = plan.at(tc);
                            g.fill.clone_from(&p.fill);
                            g.stroke = p.stroke;
                            Some(g)
                        }
                        None => from.geometry.clone(),
                    };
                    p
                }
                // Quem SAI fica onde estava e desvanece; quem ENTRA já está no lugar de destino e
                // aparece. ⚠️ Nenhum dos dois se move: mover algo que só existe de um lado seria
                // inventar a outra ponta do caminho.
                //
                // ⚠️ **`tc`, e não `t`** — os dois têm um canal só, e ele é a OPACIDADE. Um
                // `Back Out` (pico 1,100) daria alfa **−0,4** a quem sai; uma mola a carregar o
                // momento (`t < 0`) daria alfa negativo a quem entra. É exatamente o caso que o
                // doc acima nomeia: *não é overshoot, é lixo*.
                //
                // ⭐ **E NENHUM DOS DOIS FALA PELO GRUPO** (`bool_group_op: None`, auditoria de
                // 2026-08-23). A operação de uma booleana é um fato do GRUPO, e a pose carrega-o
                // por REDUNDÂNCIA — cada operando repete o mesmo número. Quem só existe de um dos
                // lados não tem com que concordar: deixá-lo falar faria o `install`, que escreve
                // pose a pose no MESMO quadro, decidir a receita pela **ordem de iteração** de um
                // `Vec` — o de fora escreve `Union`, o que entra escreve `Subtract`, e o último
                // ganha. `None` aqui é *"não sei de grupo nenhum"* e o `install` não escreve, que
                // é exatamente o que se quer.
                //
                // ⚠️ O verbo PRÓPRIO (`bool_op`) fica verbatim, e a assimetria é a lei destes dois
                // passos: ele é fato de UMA forma, e quem entra já chega na pose de destino — como
                // já chega na posição, na forma e na tinta de destino.
                Step::Leaving(p) => ObjectPose {
                    opacity: lerp_f32(p.opacity, 0.0, tc),
                    bool_group_op: None,
                    ..p.clone()
                },
                Step::Entering(p) => ObjectPose {
                    opacity: lerp_f32(0.0, p.opacity, tc),
                    bool_group_op: None,
                    ..p.clone()
                },
            })
            .collect()
    }
}

#[inline]
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

#[inline]
#[allow(clippy::cast_possible_truncation)]
fn lerp_f32(a: f32, b: f32, t: f64) -> f32 {
    a + (b - a) * t as f32
}

/// O arco mais curto entre dois ângulos.
///
/// ⚠️ O `rem_euclid` traz a diferença para `[0, 2π)` **antes** de escolher o lado, e é isso que
/// torna a função correta para ângulos fora de uma volta (um objeto girado 3 voltas pelo artista).
#[inline]
fn lerp_angle(a: f64, b: f64, t: f64) -> f64 {
    let mut d = (b - a).rem_euclid(FULL_TURN);
    if d > HALF_TURN {
        d -= FULL_TURN;
    }
    a + d * t
}

/// **Estas duas geometrias desenham a mesma coisa?**
///
/// ⚠️ Ela compara **tudo o que o `install` escreve**, e a coincidência é a lei: se um campo
/// entrasse aqui sem entrar lá, um estado escreveria metade da forma; se entrasse lá sem entrar
/// aqui, duas formas diferentes passariam por iguais e a transição nunca as casaria. `id`, `fill`
/// e `stroke` ficam de fora porque **não são forma** — a identidade e a tinta são campos da POSE,
/// e cada fato tem uma casa só.
fn same_shape(a: &ph2d_vec_scene::VecPath, b: &ph2d_vec_scene::VecPath) -> bool {
    a.verts == b.verts
        && a.closed == b.closed
        && a.subpaths == b.subpaths
        && a.fill_rule == b.fill_rule
        && a.effects == b.effects
}

/// **Dois perfis de largura, e o que está entre eles** — a fronteira entre o `Option` da pose e
/// a mistura da [`ph2d_stroke_width::WidthStops`].
///
/// ⚠️ **Ausente é UNIFORME, e é isso que dispensa os casos especiais:** um estado sem perfil e
/// um com perfil misturam-se como *uniforme → perfil*, que é exatamente o que o artista vê. E o
/// resultado uniforme volta a `None`, senão o documento acumularia relações que não desenham
/// nada — a mesma lei que a shell aplica ao componente.
fn mix_width(
    a: Option<&ph2d_stroke_width::WidthStops>,
    b: Option<&ph2d_stroke_width::WidthStops>,
    t: f64,
) -> Option<ph2d_stroke_width::WidthStops> {
    if a.is_none() && b.is_none() {
        return None;
    }
    let empty = ph2d_stroke_width::WidthStops::default();
    let m = a.unwrap_or(&empty).mix(b.unwrap_or(&empty), t);
    (!m.is_uniform()).then_some(m)
}
