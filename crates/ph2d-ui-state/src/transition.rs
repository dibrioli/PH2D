//! **A TRANSIÇÃO** — o casamento pago uma vez, e o passo pago por frame.

use crate::pose::ObjectPose;
use ph2d_vec_blend::Plan;

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
                Step::Leaving(p) => ObjectPose {
                    opacity: lerp_f32(p.opacity, 0.0, tc),
                    ..p.clone()
                },
                Step::Entering(p) => ObjectPose {
                    opacity: lerp_f32(0.0, p.opacity, tc),
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
