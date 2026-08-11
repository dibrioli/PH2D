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

/// **O que UM raio lateral viu.**
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WallHit {
    /// Distância da borda do corpo até a superfície.
    pub distance: f32,
    /// A normal dela.
    pub normal: Vec2,
}

/// **O que o sensor lateral viu, altura por altura** (W13, o flanco) — uma
/// entrada por amostra de [`wall_offsets`], na MESMA ordem.
///
/// ⚠️ **O array chega inteiro à lei, e isso é o padrão dos outros dois sensores
/// desta ponte** ([`crate::Headroom`], [`crate::CeilingProbe`]): a ponte
/// AMOSTRA, a lei DECIDE. Reduzir na ponte faria dela a dona de *"qual destas
/// superfícies é a parede?"*, que é exactamente a pergunta que este módulo
/// existe para responder — e a redução divergiria da classificação no dia em que
/// o `max_slope` se movesse.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct WallProbe {
    /// Para que lado o sensor olhou: `-1` à esquerda, `+1` à direita.
    pub side: f32,
    /// Um `Option` por altura do flanco; `None` = aquele raio não achou nada.
    ///
    /// ⚠️ **Dimensionado pelo TETO e não pela contagem autorada** — as caudas
    /// não usadas ficam `None`, e é isso que mantém este tipo sem alocação no
    /// caminho quente com a contagem a variar por personagem.
    pub hits: [Option<WallHit>; MAX_WALL_SAMPLES],
    /// Quantas das `hits` este flanco de facto castou (ver [`odd_samples`]).
    pub samples: usize,
}

impl WallProbe {
    /// **A leitura montada a partir das amostras que de facto existem** — a
    /// cauda até [`MAX_WALL_SAMPLES`] fica `None`.
    ///
    /// ⚠️ Porta única para quem CONSTRÓI uma leitura (a ponte e as fixtures):
    /// sem ela cada chamador soletraria o array do teto, e o dia em que o teto
    /// mudasse seria um dia de churn em vez de uma linha.
    #[must_use]
    pub fn from_hits(side: f32, hits: &[Option<WallHit>]) -> Self {
        let mut out = [None; MAX_WALL_SAMPLES];
        let n = hits.len().min(MAX_WALL_SAMPLES);
        out[..n].copy_from_slice(&hits[..n]);
        Self {
            side,
            hits: out,
            samples: n,
        }
    }
}

/// **A parede escolhida** — o que a lei devolve depois de olhar o flanco todo.
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

    /// **QUANTOS raios o flanco casta.** Ímpar (ver [`odd_samples`]).
    ///
    /// ⚠️ O que ele compra é COBERTURA DE FRESTA, não precisão: os raios cobrem
    /// o flanco inteiro, então uma fresta que cegue todos tem de ser mais alta
    /// que o espaçamento. Com 3 num corpo de 1 m a fresta cega mede 0,5 m; com
    /// 9, 12,5 cm. O teto e o preço estão em [`MAX_WALL_SAMPLES`] — **18 ns por
    /// raio, plano em N**, então o custo não é o que decide.
    pub samples: usize,

    /// **Onde as amostras de FORA se sentam**, como fração da meia-altura do
    /// corpo. `1.0` põe-nas na borda exata da caixa (o comportamento de sempre).
    ///
    /// ⚠️ Baixá-lo afasta o sensor das PONTAS, que é onde uma cápsula é um ponto
    /// e um raio rasante vê parede onde o corpo mal encosta — o trade está
    /// nomeado no doc de [`wall_offsets`], e agora ele tem um número.
    pub spread: f32,

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

    /// **Por quantos SEGUNDOS ele segura a parede de vez**, com o botão de
    /// agarrar apertado. `0.0` **desliga** — a capacidade não existe.
    ///
    /// # ⚠️ O zero não é um caso especial, ele é exato
    ///
    /// Segurar por zero segundos **é** não ter agarrar-se, então o idioma dos
    /// irmãos (`coyote_time`, `corner_reach`, `slide_speed`) cai aqui sem
    /// nenhuma cerimônia — não há um `bool` ao lado a discordar do número.
    ///
    /// # ⚠️ Por que uma RESERVA, e não um interruptor
    ///
    /// Um agarrar-se sem custo transforma toda parede numa beirada permanente, e
    /// a pesquisa é unânime sobre isso: o Celeste **começou sem reserva** e o
    /// jogo ficava resolvível pendurando-se; um TEMPORIZADOR simples também foi
    /// tentado e foi abandonado por não distinguir *escalar* de *pendurar*. A
    /// reserva é o que ficou, e é o mesmo desenho de Hollow Knight/Ori pelo lado
    /// oposto (lá o que limita é a habilidade, não o recurso).
    ///
    /// ⚠️ **UM número, e a assimetria do Celeste NÃO foi construída** (lá subir
    /// custa mais que pendurar): o segundo knob teria o valor certo em função do
    /// primeiro, que é a ergonomia que este repositório trata como bug de
    /// desenho. Aqui a leitura é direta — *quanto tempo ele fica pendurado*.
    pub grab_stamina: f32,
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
        // ⚠️ Os defaults SÃO o mundo de sempre — 3 amostras nas bordas exatas —,
        // e é isso que mantém todo player já autorado byte-idêntico.
        samples: WALL_SAMPLES,
        spread: 1.0,
        // ⚠️ **0,2 s, e o número saiu da varredura** (`measure_wall`, tabela no
        // doc do `measure_the_wall_jump_lockout`): é onde a ALTURA para de ser
        // perdida — 81% em zero, 96% em 0,10, 97% em 0,20 e nunca mais.
        // ⚠️ **Não é onde o AFASTAMENTO satura**, e a distinção é a medição a
        // corrigir uma frase minha: o afastamento cresce LINEAR (0,46 → 1,74 →
        // 3,44 m), porque com o controle calado nada freia a horizontal. Cada
        // décimo além de 0,2 s é controle tirado do jogador comprando alcance —
        // uma escolha de PRODUTO, e o slider está lá para quem a quiser fazer.
        jump_lockout: 0.2,
        // ⚠️ Desligado, como as duas metades acima: agarrar-se de vez é uma
        // CAPACIDADE do personagem, e ligá-la por default mudaria o
        // comportamento de todo player já autorado.
        grab_stamina: 0.0,
    };

    /// **A capacidade está armada?** — nenhuma das duas metades ligada significa
    /// que a parede não existe para este personagem, e o sensor não é castado.
    #[must_use]
    pub fn armed(&self) -> bool {
        self.slide_speed > 0.0 || self.jump_height > 0.0 || self.grab_stamina > 0.0
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

/// **Quantos raios o sensor lateral casta.**
pub const WALL_SAMPLES: usize = 3;

/// **O TETO da contagem de amostras** — de que recurso ele é, com a medição.
///
/// ⚠️ **Não é de TEMPO.** Medido (`measure_player_probes::measure_what_a_sample_costs`,
/// mundo com chão, parede e teto): **18 ns por raio, PLANO em N** — o BVH não
/// encarece por amostra —, então 257 raios custam **4,55 µs, 0,027% de um quadro
/// de 60 fps**, e só nos tiques em que a lei pergunta.
///
/// O que se esgota é a **precisão de representação**: o passo entre amostras cai
/// para **2,5 mm em 257**, e o solver assenta com um
/// `normalized_allowed_linear_error` de ~**1,3 mm**. Abaixo disso as amostras
/// descrevem geometria que a própria física não resolve — é aí que o número
/// deixa de comprar coisa alguma, e é esse o teto.
pub const MAX_WALL_SAMPLES: usize = 257;

/// **Onde os raios da parede nascem**, medidos do centro do corpo — o FLANCO
/// inteiro, não só a cintura.
///
/// # ⚠️ O meio vem PRIMEIRO, e a ordem é load-bearing
///
/// A ponte fica com o hit mais PRÓXIMO e desempata pela ordem desta lista. Numa
/// parede plana os três raios medem a mesma distância, então o meio vence — e é
/// isso que torna toda parede que já funcionava **byte-idêntica**: o que muda é
/// só a geometria que o meio sozinho não via.
///
/// # ⚠️ O preço deste sensor estava MEDIDO antes de ele existir
///
/// Com um raio só, uma parede com uma **fresta** de 0,75 m (num corpo de 1,0 m,
/// ou seja com 12,5 cm de pé E de ombro ainda encostados) **recusa o pulo de
/// parede por inteiro** — 0,000 m de subida contra 2,162 m na parede sólida. E o
/// escorregamento quase não denuncia (0,0500 → 0,0632 m/tique), porque a *cola*
/// do módulo segura o personagem de qualquer jeito: quem paga o defeito é o
/// PULO, que não tem cola. Abaixo de ~0,70 m de fresta o **buffer do pulo**
/// mascarava tudo — ele guarda o aperto até o bloco de baixo reaparecer.
///
/// ⚠️ **As bordas são as da CAIXA envolvente**, e a direção do erro é a certa:
/// numa cápsula a ponta é um ponto, então um raio de pé rasante pode ver parede
/// onde o corpo mal encosta — agarrar-se um triz cedo demais é o que Celeste faz
/// por sobreposição de hitbox, e a lei ainda exige empurrar contra ela e estar a
/// descer.
///
/// # ⚠️ A varredura de forma EXISTE agora, e este sensor continua de raios
///
/// A `W-ShapeCast` deu ao wrapper o `sweep_body`, e o sensor do agachar trocou
/// os três raios dele por uma varredura. Este **não** trocou, e a razão é
/// MEDIDA, não inércia:
///
/// - o vão cego aqui exigiria uma parede de **RIPAS** — os três raios cobrem o
///   flanco inteiro, então uma fresta única teria de ser mais alta que o corpo
///   para os apagar aos três, e nesse caso não há parede à frente do corpo;
/// - e o [`cling`] **reduz sobre as amostras** com a régua da perna (`max_slope`)
///   para decidir *qual* superfície é parede. Uma varredura devolve **um**
///   contacto, o mais próximo — que pode ser a rampa aos pés, que a perna aceita
///   e que este sensor tem de descartar. Trocar seria perder a escolha, não
///   ganhar precisão.
///
/// A fresta que sobra invisível é a **benigna** (um buraco reportado como
/// parede: ele agarra-se a uma parede que tem um furo). Fica nomeada, com o
/// motivo de não ter sido curada junto.
#[must_use]
pub fn wall_offsets(half_height: f32, samples: usize, spread: f32) -> [f32; MAX_WALL_SAMPLES] {
    let n = odd_samples(samples, MAX_WALL_SAMPLES);
    let reach = half_height * spread.clamp(0.0, 1.0);
    let mut out = [0.0; MAX_WALL_SAMPLES];
    // ⚠️ **O MEIO primeiro, e depois os pares para fora** — a ordem é a lei do
    // desempate do [`cling`] (numa parede plana os raios medem o mesmo, e quem
    // vence é o primeiro da lista). Preencher de baixo para cima faria a resposta
    // saltar para a ponta do pé no dia em que a contagem mudasse.
    let pairs = (n - 1) / 2;
    for k in 1..=pairs {
        let d = reach * (k as f32) / (pairs as f32);
        out[2 * k - 1] = -d;
        out[2 * k] = d;
    }
    out
}

/// **Quantas amostras este flanco de facto casta** — a porta única do clamp.
///
/// ⚠️ **ÍMPAR, e não é cerimónia:** a amostra do meio é a âncora do desempate do
/// [`cling`] e o flanco é simétrico em torno da cintura, então uma contagem par
/// ou deixaria o meio de fora (perdendo o desempate) ou enviesaria um lado.
/// Arredondar para cima mantém *"pedi mais, recebi mais"*.
#[must_use]
pub const fn odd_samples(samples: usize, max: usize) -> usize {
    let n = if samples < 1 { 1 } else { samples };
    let n = if n > max { max } else { n };
    if n % 2 == 0 { n + 1 } else { n }
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
///
/// # ⚠️ O FLANCO tem várias alturas, e a escolha é feita AQUI
///
/// De todas as amostras, fica a **mais próxima que é PAREDE** — e cada motivo de
/// descarte já era uma regra desta função (nada visto · normal degenerada ·
/// inclinação que a perna aceita). É por isso que a redução mora aqui e não na
/// ponte: escolher *qual superfície* e decidir *se é parede* são a mesma
/// pergunta, e separá-las daria duas respostas que divergem no dia em que o
/// `max_slope` autorado se mover.
///
/// ⚠️ **Numa parede plana as amostras empatam e a PRIMEIRA vence** — e a
/// primeira é a cintura ([`wall_offsets`]), então toda parede que já funcionava
/// responde exactamente o que respondia. O que muda é só a geometria que a
/// cintura sozinha não via.
///
/// ⚠️ **E isto resolve um caso que um raio só não tinha como resolver:** uma
/// rampa aos pés (que a perna ACEITA, logo não é parede) deixava de cegar o
/// tronco encostado à parede — a rampa é descartada por inclinação e a parede
/// continua lá.
#[must_use]
pub fn cling(
    cfg: &PlayerConfig,
    wall: Option<&WallProbe>,
    drive: f32,
    rel_up: f32,
    up: Vec2,
) -> Option<WallSample> {
    if !cfg.wall.armed() {
        return None;
    }
    let w = wall?;
    if drive * w.side <= 0.0 || rel_up >= 0.0 {
        return None;
    }
    let mut best: Option<WallHit> = None;
    for hit in w.hits.iter().flatten() {
        let n2 = hit.normal[0] * hit.normal[0] + hit.normal[1] * hit.normal[1];
        if n2 < 1.0e-6 {
            continue;
        }
        // A MESMA régua da perna: parede é o que ela recusou por inclinação.
        let cos = hit.normal[0] * up[0] + hit.normal[1] * up[1];
        if cos >= cfg.walk.max_slope_cos() {
            continue;
        }
        if best.is_none_or(|b| hit.distance < b.distance) {
            best = Some(*hit);
        }
    }
    Some(WallSample {
        side: w.side,
        normal: best?.normal,
    })
}

/// **O que o personagem carrega entre tiques sobre agarrar-se** (W23).
///
/// ⚠️ Ele mora no [`crate::PlayerState`] e não num mapa à parte da ponte, pela
/// razão que aquele tipo já documenta: é o `PlayerState` que a **fita** guarda no
/// ring de tiques âncora, e um estado que vivesse noutro lugar teria de ser
/// acrescentado àquele ring **à mão** — esquecê-lo é um scrub que devolve o
/// mundo de um tique e a memória do controlador de outro.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct GrabState {
    /// **Quanto já se gastou da reserva**, em segundos.
    ///
    /// ⚠️ O GASTO, e não o que sobra, e a diferença não é cosmética: o artista
    /// pode mover o `grab_stamina` com o personagem pendurado, e um "quanto
    /// sobra" guardado ficaria acima da reserva nova — um número que descreve um
    /// mundo que deixou de existir. O gasto é sempre legível contra qualquer
    /// reserva.
    pub spent: f32,
}

/// **A RESERVA de agarrar-se neste tique** — quanto se gastou depois dele, e se
/// o personagem está de facto AGARRADO.
///
/// # ⚠️ O agarrar-se cavalga o [`cling`], não o substitui
///
/// A pergunta *estou numa parede?* é feita **uma vez** e já tem dono. Agarrar-se
/// acrescenta duas condições ao que ela respondeu (o botão está apertado, e
/// ainda há reserva) — e é por isso que ele herda de graça as três metades do
/// `cling`: é parede pela régua da perna, o jogador empurra contra ela, e ele
/// está a descer.
///
/// ⚠️ **A reserva volta ao cheio no CHÃO, de uma vez.** Qualquer outra regra
/// (recarga por segundo, recarga ao soltar) faria o jogador esperar parado, e um
/// jogo que ensina a esperar é o que a reserva existe para não ser.
#[must_use]
pub fn grab_step(
    cfg: &WallConfig,
    state: GrabState,
    clinging: bool,
    held: bool,
    grounded: bool,
    dt: f32,
) -> (GrabState, bool) {
    if grounded {
        return (GrabState::default(), false);
    }
    if cfg.grab_stamina <= 0.0 || !clinging || !held {
        return (state, false);
    }
    if state.spent >= cfg.grab_stamina {
        return (state, false);
    }
    let spent = (state.spent + dt.max(0.0)).min(cfg.grab_stamina);
    (GrabState { spent }, true)
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
pub fn wall_slide(
    cfg: &WallConfig,
    clinging: bool,
    gripping: bool,
    rel_up: f32,
    up: Vec2,
) -> Motor {
    if !clinging || (cfg.slide_speed <= 0.0 && !gripping) {
        return Motor::default();
    }
    // ⚠️ **UMA expressão, dois regimes:** agarrado, a velocidade alvo e' ZERO;
    // solto, e' a descida autorada. Um segundo termo somado ao escorregamento
    // daria dois donos do mesmo numero, e o sintoma seria um personagem que
    // "quase" para.
    let target = if gripping { 0.0 } else { -cfg.slide_speed };
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
#[path = "wall_tests.rs"]
mod tests;
