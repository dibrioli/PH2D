//! **AS PAREDES** (W13) — escorregar por uma, e pular dela.
//!
//! # ⚠️ Uma parede é o que a PERNA já recusou
//!
//! Não há um segundo limiar aqui, e não pode haver: `footing_verdict` já
//! classifica toda superfície ao alcance em *chão* / *íngreme demais* / *nada*,
//! e uma parede é exactamente a do meio. Escrever um `wall_min_angle` seria um
//! segundo número a discordar do `Max Slope` que o artista já autorou — e a
//! discordância teria forma: uma inclinação em que a perna diz *"você
//! escorrega"* e a parede diz *"aqui não dá para se agarrar"*, deixando o
//! personagem sem nenhum dos dois comportamentos.
//!
//! ⚠️ **Por isso [`cling`] recebe o `PlayerConfig` inteiro** e pergunta ao
//! `max_slope_cos()` da caminhada. Um `WallConfig` que carregasse o próprio
//! ângulo compilaria e seria a segunda porta.
//!
//! # ⚠️ Agarrar-se exige EMPURRAR contra a parede
//!
//! Raspar numa parede a caminho de outro lugar não é agarrar-se. Exigir que o
//! `drive` aponte para ela é o que separa as duas — é o que Celeste, Hollow
//! Knight e Ori fazem —, e é também o que torna o sensor barato: a ponte casta
//! **um** raio, na direção em que o jogador está a empurrar, e nenhum quando ele
//! não empurra nada.
//!
//! A alternativa (agarrar por CONTATO, como Super Meat Boy) faz o personagem
//! grudar em cada parede que ele encosta durante um pulo horizontal, e num
//! platformer preciso isso lê como o controle a travar.
//!
//! # ⚠️ E exige estar a DESCER
//!
//! Um escorregamento é um freio de QUEDA. Enquanto o personagem sobe, a parede
//! não tem nada a fazer — freá-lo ali seria cortar o próprio pulo que o levou
//! até ela.

use crate::{Motor, PlayerConfig, Vec2};

/// **O que o sensor lateral viu** — `None` no chamador significa *"não há nada
/// naquela direção"*, e a lei lê isso como não haver parede.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WallSample {
    /// Para que lado ela está: `-1` à esquerda, `+1` à direita.
    ///
    /// ⚠️ É o SINAL do `drive` que a encontrou, não uma derivação da normal: o
    /// sensor casta na direção em que o jogador empurra, então o lado é um fato
    /// sobre o gesto, e derivá-lo da normal daria a resposta errada numa parede
    /// inclinada.
    pub side: f32,
    /// A normal da superfície.
    ///
    /// ⚠️ Ela é o que decide se aquilo é PAREDE (pela mesma régua da perna) e é
    /// também a direção do empurrão do pulo — usá-la em vez de um `[-side, 0]`
    /// horizontal é o que faz uma parede inclinada lançar para onde ela de facto
    /// aponta.
    pub normal: Vec2,
}

/// Como o personagem se comporta contra uma parede.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WallConfig {
    /// **A velocidade com que se desce uma parede agarrada**, em m/s.
    ///
    /// ⚠️ **A velocidade, não um teto** — ver o aviso de [`wall_slide`]: a
    /// medição derrubou a versão-teto porque com o atrito default o personagem
    /// não cai, e um teto nunca dispararia.
    ///
    /// `0.0` **desliga** o escorregamento (a queda é a de sempre), e é o idioma
    /// dos irmãos `coyote_time`/`corner_reach`: um zero que significa *"esta
    /// assistência não existe"*.
    ///
    /// ⚠️ **Não confundir com AGARRAR-SE de vez:** um personagem que fica parado
    /// numa parede é outra mecânica (o botão de *grab* do Celeste), com botão
    /// próprio, e não se alcança escrevendo `0` aqui — `0` é a ausência da
    /// assistência, não o extremo dela.
    pub slide_speed: f32,

    /// **A altura de um pulo de parede**, metros. `0.0` desliga.
    ///
    /// Medida na mesma unidade e pela mesma conversão do pulo do chão
    /// (`v₀ = √(2·|g|·h)`), porque é a mesma frase do artista: *"este pulo
    /// alcança aquela plataforma"*.
    pub jump_height: f32,

    /// **O empurrão para LONGE da parede**, em m/s, no instante do pulo.
    ///
    /// ⚠️ É uma velocidade e não uma altura porque o que ela decide é *quão
    /// longe da parede ele chega*, e isso é horizontal — a métrica que o artista
    /// julga é a distância entre duas paredes que ele consegue subir em
    /// ziguezague.
    pub jump_push: f32,

    /// **Quanto ALÉM da própria largura o sensor procura**, metros.
    ///
    /// ⚠️ Não é zero de propósito: o personagem PAIRA, mas contra uma parede ele
    /// ENCOSTA, e o contato deixa-o a uma folga que depende do solver. Um alcance
    /// de exatamente meia-largura tornaria o agarrar-se intermitente — ligado num
    /// tique, desligado no seguinte —, e o sintoma seria um escorregamento que
    /// pisca.
    pub reach: f32,

    /// **Por quantos segundos o CONTROLE AÉREO fica calado depois de um pulo de
    /// parede.** `0.0` desliga.
    ///
    /// # ⚠️ Sem ele o pulo de parede entrega METADE do que promete, e está medido
    ///
    /// O jogador que acabou de pular de uma parede ainda está a segurar a
    /// direção DELA — é o gesto que o levou até lá. O controle aéreo obedece,
    /// puxa-o de volta, e o resto do voo é gasto a raspar: medido nesta fixture,
    /// o personagem afasta-se **0,44 m** e volta, e o pulo entrega **1,53 m de
    /// 2,0 autorados (76%)**, porque o atrito da raspagem come o topo.
    ///
    /// ⚠️ **É a MESMA doença que o `lift_momentum` da W10 nomeou** — *"a doença
    /// não era do solver: quem apagava era a ASSISTÊNCIA"* —, e a cura é da
    /// mesma família: calar o controle aéreo enquanto o gesto que ele
    /// contradiria ainda está a acontecer.
    ///
    /// ⚠️ E **calar** é diferente de **zerar o `drive`**: zerar faria a caminhada
    /// mirar velocidade nula e FREAR o empurrão que o pulo acabou de dar — o
    /// mesmo erro, com outra roupa.
    pub jump_lockout: f32,
}

impl WallConfig {
    /// O ponto de partida — ⚠️ **não são defaults de produto** (a nota dos
    /// irmãos [`crate::RideConfig::STARTING_POINT`] e
    /// [`crate::JumpConfig::STARTING_POINT`]).
    ///
    /// ⚠️ **E ele nasce DESLIGADO**, ao contrário dos irmãos: parede é uma
    /// CAPACIDADE do personagem, não uma correção de física. Um platformer sem
    /// paredes é um gênero inteiro (o Mario original), e ligá-la por default
    /// mudaria o comportamento de todo player já autorado — a mesma razão pela
    /// qual o toggle `Physics` do transporte nasce desmarcado.
    pub const STARTING_POINT: Self = Self {
        slide_speed: 0.0,
        jump_height: 0.0,
        // ⚠️ Os dois números abaixo NÃO são inertes com a capacidade desligada —
        // eles são o que ela vale quando alguém a liga, e nascer em zero faria
        // o primeiro clique no `Wall Slide` entregar um pulo de parede que não
        // afasta ninguém da parede. É o mesmo desenho do `takeoff_speed`, que
        // acompanha um multiplicador neutro.
        jump_push: 6.0,
        reach: 0.08,
        // ⚠️ **0,2 s, e o número saiu da varredura** (`measure_wall`, tabela no
        // doc do `measure_the_wall_jump_lockout`): é onde a ALTURA para de ser
        // perdida — 81% em zero, 96% em 0,10, 97% em 0,20 e nunca mais.
        // ⚠️ **Não é onde o AFASTAMENTO satura**, e a distinção é a medição a
        // corrigir uma frase minha: o afastamento cresce LINEAR (0,46 → 1,74 →
        // 3,44 m), porque com o controle calado nada freia a horizontal. Cada
        // décimo além de 0,2 s é controle tirado do jogador comprando alcance —
        // uma escolha de PRODUTO, e o slider está lá para quem a quiser fazer.
        jump_lockout: 0.2,
    };

    /// **A capacidade está armada?** — nenhuma das duas metades ligada significa
    /// que a parede não existe para este personagem, e o sensor não é castado.
    #[must_use]
    pub fn armed(&self) -> bool {
        self.slide_speed > 0.0 || self.jump_height > 0.0
    }
}

/// **Vale a pena castar o raio lateral?** — a PORTA ÚNICA da pergunta.
///
/// ⚠️ O molde é o [`crate::corner_probe_wanted`] da W10, e a razão é a mesma: a
/// ponte pergunta isto para decidir se gasta um raio, e a lei pergunta o mesmo
/// para decidir se pode agir. Duas cópias dariam uma assistência que existe de
/// um lado da fronteira e não do outro — e o sintoma seria um custo pago sem
/// efeito, ou um efeito que depende de o sensor ter sido castado por acaso.
///
/// - No CHÃO não há parede que interesse: o que se faz contra uma superfície
///   íngreme estando em pé já é assunto do [`crate::no_uphill`].
/// - Sem `drive` não há direção para onde olhar, e agarrar-se exige empurrar.
#[must_use]
pub fn wall_probe_wanted(cfg: &WallConfig, grounded: bool, drive: f32) -> bool {
    cfg.armed() && cfg.reach >= 0.0 && !grounded && drive != 0.0
}

/// **AGARRADO?** — a pergunta feita **UMA vez**, e as duas leis desta wave são
/// vistas dela.
///
/// Devolve a amostra quando as três metades valem: aquilo é uma **parede** (pela
/// régua da perna, nunca por uma segunda), o jogador **empurra** contra ela, e o
/// personagem está a **descer**.
///
/// # ⚠️ *Empurrar contra ela* é uma DEFESA EM CAMADAS, e está medido
///
/// A ponte já casta o raio **só** na direção do `drive`, então na prática ela
/// nunca entrega uma parede do lado errado — e a mutação que apaga o
/// `drive * side` daqui **sobrevive a todos os gates de comportamento**. Quem a
/// mata é o gate de unidade `brushing_a_wall_is_not_clinging`, que passa a
/// amostra à mão: cada camada com o seu gate
/// ([[feedback_layered_defenses_need_per_layer_gates]]).
///
/// A camada de dentro **fica**, e não é higiene: esta função é `pub`, e no dia
/// em que alguém castar os dois lados (um *wall grab* sem direção, por exemplo)
/// ela é a única coisa entre o produto e um personagem que se agarra à parede de
/// que está a fugir.
///
/// ⚠️ Uma normal degenerada (raio nascido dentro da geometria — ver
/// [`crate::GroundSample::normal`]) é recusada aqui, e não tratada como plano
/// como o chão a trata: no chão a suposição menos daninha é deixar a mola
/// empurrar o personagem para fora; numa parede seria agarrá-lo ao que quer que
/// ele esteja atravessado, que é o oposto de menos daninho.
#[must_use]
pub fn cling<'a>(
    cfg: &PlayerConfig,
    wall: Option<&'a WallSample>,
    drive: f32,
    rel_up: f32,
    up: Vec2,
) -> Option<&'a WallSample> {
    if !cfg.wall.armed() {
        return None;
    }
    let w = wall?;
    if drive * w.side <= 0.0 || rel_up >= 0.0 {
        return None;
    }
    let n2 = w.normal[0] * w.normal[0] + w.normal[1] * w.normal[1];
    if n2 < 1.0e-6 {
        return None;
    }
    // A MESMA régua da perna: parede é o que ela recusou por inclinação.
    let cos = w.normal[0] * up[0] + w.normal[1] * up[1];
    if cos >= cfg.walk.max_slope_cos() {
        return None;
    }
    Some(w)
}

/// **O ESCORREGAMENTO** — a velocidade com que se desce uma parede.
///
/// ⚠️ **Ele DEFINE a velocidade, não a limita — e a medição é que decidiu.**
///
/// A primeira versão desta lei era um TETO (*"não caia mais rápido do que
/// isto"*), escrita por raciocínio, com o argumento de que acelerar o
/// personagem para baixo faria dele alguém que **cai mais depressa por estar
/// encostado a uma parede**. O argumento é bonito e o knob que ele produz é
/// **INERTE**: medido na fixture do `platform_wall`, um personagem que empurra
/// contra uma parede **NÃO CAI** — ele desce 9 cm em um segundo inteiro.
///
/// O mecanismo tem duas metades e as duas são do produto que já shipa:
///
/// 1. o **atrito** (`Collider::DEFAULT_FRICTION = 0,5`) contra a normal que o
///    controle aéreo sustenta enquanto o jogador segura a direção da parede;
/// 2. e a **gravidade do ÁPICE** — com `|rel_up| ≤ peak_speed` a lei do pulo
///    aplica `peak_gravity = 0,5`, ou seja METADE do peso. Isso é
///    auto-reforçante: parado, o personagem cai na janela do ápice, a gravidade
///    é cortada ao meio, e o atrito passa a ganhar.
///
/// Um teto nunca dispara nesse estado (não há queda a frear), e o resultado é um
/// personagem **COLADO** à parede sob um knob chamado *Wall Slide Speed*. Um
/// número que não faz nada é o defeito que este repositório trata como tal.
///
/// ⚠️ **Definir a velocidade subsume as duas direções e é o que o nome promete:**
/// quem cai depressa é freado ATÉ ela, quem está colado é solto ATÉ ela. É
/// também o que o Celeste faz, e é por isso que lá um escorregamento se vê.
///
/// ⚠️ **Um `boost`, e não um `accel`:** o que se quer é uma velocidade EXATA, e
/// uma força chegaria lá com atraso e passaria do ponto — a mesma escolha que o
/// amortecimento da mola faz, pelo mesmo motivo.
#[must_use]
pub fn wall_slide(cfg: &WallConfig, clinging: bool, rel_up: f32, up: Vec2) -> Motor {
    if !clinging || cfg.slide_speed <= 0.0 {
        return Motor::default();
    }
    let target = -cfg.slide_speed;
    let delta = target - rel_up;
    Motor {
        accel: [0.0, 0.0],
        boost: [up[0] * delta, up[1] * delta],
    }
}

/// **O que a parede OFERECE a um pulo** — `None` quando não há nada a oferecer.
///
/// ⚠️ **A parede oferece; a lei do pulo é que aceita.** Manter a decisão dentro
/// do [`crate::jump_step`] é o que impede a segunda porta: quem decide *"este
/// aperto vira um pulo"* já é ele — ele possui a borda do botão, o buffer, o
/// coyote e o `airborne` —, e um segundo dono do mesmo aperto daria um tique em
/// que o personagem pula do chão E da parede.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WallLaunch {
    /// A direção para LONGE da parede, já com a velocidade dentro (m/s).
    pub away: Vec2,
    /// A altura deste pulo, metros — convertida pela mesma lei do pulo do chão.
    pub height: f32,
    /// Por quantos segundos o controle aéreo fica calado depois deste pulo.
    ///
    /// ⚠️ **Viaja AQUI, com a oferta**, e não é lido do `WallConfig` pela lei do
    /// pulo: é a parede que sabe o que oferece, e o `jump_step` não conhece — nem
    /// tem por que conhecer — a config dela.
    pub lockout: f32,
}

/// O que a parede oferece, dado que o personagem está agarrado a ela.
#[must_use]
pub fn wall_launch(cfg: &WallConfig, clinging: Option<&WallSample>) -> Option<WallLaunch> {
    let w = clinging?;
    if cfg.jump_height <= 0.0 {
        return None;
    }
    let len = (w.normal[0] * w.normal[0] + w.normal[1] * w.normal[1]).sqrt();
    // ⚠️ `is_finite` E `> 0`: a normal degenerada já foi recusada pelo `cling`,
    // mas esta função é `pub` e uma divisão por zero aqui viraria um `NaN` na
    // velocidade — que o `readback` escreveria no `Transform` e no hash.
    if !len.is_finite() || len <= 0.0 {
        return None;
    }
    let push = cfg.jump_push.max(0.0);
    Some(WallLaunch {
        away: [w.normal[0] / len * push, w.normal[1] / len * push],
        height: cfg.jump_height,
        lockout: cfg.jump_lockout.max(0.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WalkConfig;

    const UP: Vec2 = [0.0, 1.0];

    fn cfg(slide: f32, height: f32) -> PlayerConfig {
        PlayerConfig {
            wall: WallConfig {
                slide_speed: slide,
                jump_height: height,
                ..WallConfig::STARTING_POINT
            },
            ..PlayerConfig::STARTING_POINT
        }
    }

    /// Uma parede à direita, normal apontando para a esquerda.
    fn right_wall() -> WallSample {
        WallSample {
            side: 1.0,
            normal: [-1.0, 0.0],
        }
    }

    /// **A régua é a da PERNA**, e este gate é o que impede alguém de escrever
    /// um segundo ângulo: uma rampa que a caminhada ACEITA nunca é parede.
    #[test]
    fn a_surface_the_leg_accepts_is_never_a_wall() {
        let c = cfg(4.0, 2.0);
        // 30°: dentro do `max_slope` default (45°), logo é chão, não parede.
        let ramp = WallSample {
            side: 1.0,
            normal: [-0.5, 0.866],
        };
        assert!(cling(&c, Some(&ramp), 1.0, -5.0, UP).is_none());
        // E a vertical é parede.
        assert!(cling(&c, Some(&right_wall()), 1.0, -5.0, UP).is_some());
    }

    /// **Agarrar-se exige EMPURRAR contra ela** — raspar não conta.
    #[test]
    fn brushing_a_wall_is_not_clinging() {
        let c = cfg(4.0, 2.0);
        let w = right_wall();
        assert!(
            cling(&c, Some(&w), -1.0, -5.0, UP).is_none(),
            "empurrando para LONGE da parede"
        );
        assert!(
            cling(&c, Some(&w), 0.0, -5.0, UP).is_none(),
            "sem empurrar nada"
        );
        assert!(cling(&c, Some(&w), 1.0, -5.0, UP).is_some());
    }

    /// **E exige DESCER** — subindo, a parede não tem nada a fazer.
    #[test]
    fn rising_past_a_wall_is_not_clinging() {
        let c = cfg(4.0, 2.0);
        let w = right_wall();
        assert!(cling(&c, Some(&w), 1.0, 5.0, UP).is_none(), "subindo");
        assert!(cling(&c, Some(&w), 1.0, 0.0, UP).is_none(), "no apice");
        assert!(cling(&c, Some(&w), 1.0, -0.1, UP).is_some());
    }

    /// **O escorregamento DEFINE a velocidade, nas DUAS direções.**
    ///
    /// ⚠️ A metade de baixo (`-1 → -3`) é a que a medição obrigou: com um teto
    /// só, um personagem colado à parede por atrito nunca era solto, e o knob
    /// era um número que não fazia nada. Ver o doc da função.
    #[test]
    fn the_slide_sets_the_speed_in_both_directions() {
        let c = WallConfig {
            slide_speed: 3.0,
            ..WallConfig::STARTING_POINT
        };
        let braked = wall_slide(&c, true, -9.0, UP);
        assert!(
            (braked.boost[1] - 6.0).abs() < 1.0e-5,
            "de -9 para -3 sao +6: {:?}",
            braked.boost
        );
        let released = wall_slide(&c, true, -1.0, UP);
        assert!(
            (released.boost[1] + 2.0).abs() < 1.0e-5,
            "de -1 para -3 sao -2 -- o COLADO tem de ser solto: {:?}",
            released.boost
        );
        assert_eq!(braked.accel, [0.0, 0.0], "e' um boost, nunca uma forca");
    }

    /// **Desligado é desligado, AO BIT** — o zero de `slide_speed` não é um teto
    /// muito alto, é a ausência da assistência.
    #[test]
    fn a_zero_slide_speed_is_the_world_untouched() {
        let c = WallConfig {
            slide_speed: 0.0,
            ..WallConfig::STARTING_POINT
        };
        assert_eq!(wall_slide(&c, true, -40.0, UP), Motor::default());
    }

    /// **O empurrão sai pela NORMAL**, e é isso que faz uma parede inclinada
    /// lançar para onde ela aponta.
    #[test]
    fn the_launch_leaves_along_the_normal() {
        let c = WallConfig {
            jump_height: 2.0,
            jump_push: 6.0,
            ..WallConfig::STARTING_POINT
        };
        let w = right_wall();
        let l = wall_launch(&c, Some(&w)).expect("ha' pulo a oferecer");
        assert!(
            (l.away[0] + 6.0).abs() < 1.0e-5,
            "para a ESQUERDA: {:?}",
            l.away
        );
        assert!(l.away[1].abs() < 1.0e-5);
        assert_eq!(l.height, 2.0);
        assert_eq!(
            l.lockout,
            WallConfig::STARTING_POINT.jump_lockout,
            "a oferta carrega o silencio do controle aereo"
        );
    }

    /// **Sem altura autorada a parede não oferece pulo nenhum** — e o
    /// escorregamento continua a valer sozinho.
    #[test]
    fn a_wall_with_no_jump_height_offers_nothing() {
        let c = WallConfig {
            jump_height: 0.0,
            ..WallConfig::STARTING_POINT
        };
        assert!(wall_launch(&c, Some(&right_wall())).is_none());
    }

    /// **O sensor não é castado onde não pode agir** — a porta única.
    #[test]
    fn the_probe_is_only_wanted_where_it_can_act() {
        let off = WallConfig::STARTING_POINT;
        assert!(!off.armed(), "o ponto de partida nasce DESLIGADO");
        assert!(!wall_probe_wanted(&off, false, 1.0));

        let on = WallConfig {
            slide_speed: 4.0,
            ..off
        };
        assert!(wall_probe_wanted(&on, false, 1.0));
        assert!(!wall_probe_wanted(&on, true, 1.0), "no chao, nao");
        assert!(!wall_probe_wanted(&on, false, 0.0), "sem empurrar, nao");
    }

    /// ⚠️ **Uma normal degenerada é RECUSADA**, ao contrário do chão que a trata
    /// como plano: ali a suposição menos daninha empurra o personagem para fora,
    /// aqui ela o agarraria ao que quer que ele esteja atravessado.
    #[test]
    fn a_degenerate_normal_is_refused() {
        let c = cfg(4.0, 2.0);
        let w = WallSample {
            side: 1.0,
            normal: [0.0, 0.0],
        };
        assert!(cling(&c, Some(&w), 1.0, -5.0, UP).is_none());
    }

    /// O `max_slope` autorado MOVE a fronteira da parede — a prova de que a
    /// régua é mesmo a da perna, e não uma cópia.
    #[test]
    fn the_authored_max_slope_moves_the_wall_boundary() {
        // 60° de inclinação: normal a 60° do `up`.
        let steep = WallSample {
            side: 1.0,
            normal: [-0.866, 0.5],
        };
        let mut c = cfg(4.0, 2.0);
        c.walk = WalkConfig {
            max_slope_deg: 45.0,
            ..c.walk
        };
        assert!(
            cling(&c, Some(&steep), 1.0, -5.0, UP).is_some(),
            "com o limite em 45, uma rampa de 60 e' parede"
        );
        c.walk = WalkConfig {
            max_slope_deg: 70.0,
            ..c.walk
        };
        assert!(
            cling(&c, Some(&steep), 1.0, -5.0, UP).is_none(),
            "com o limite em 70 ela volta a ser CHAO, e chao nao se agarra"
        );
    }
}
