//! **A INTEGRAÇÃO CINEMÁTICA** — o motor de intenção vira um deslocamento
//! (W-KinMove, K5/K7 do plano 07).
//!
//! # ⚠️ Por que isto é LEI e não ponte
//!
//! Um corpo cinemático não tem velocidade que o solver possua, então alguém tem
//! de a possuir — e o aviso já estava escrito no próprio [`crate::PlayerState`]:
//! *"um estado de player que vivesse num segundo mapa da ponte teria de ser
//! acrescentado àquele ring à mão, e esquecê-lo é um scrub que devolve o mundo
//! de um tique e a memória de outro"*. Estando aqui, ela entra no ring de
//! checkpoints e sobrevive a um scrub de graça.
//!
//! E o que esta função faz é **aritmética sobre a intenção**, que é exactamente
//! o que esta crate é. O que fica na ponte é a única coisa que precisa do mundo:
//! perguntar ao `move_shape` **quanto do deslocamento coube**.
//!
//! # ⚠️ A gravidade é aplicada AQUI, e é a assimetria central dos dois modos
//!
//! No modo dinâmico o solver integra a gravidade e a mola a cancela; num corpo
//! cinemático **ninguém a aplica**, então esta lei a soma ao motor. É por isso
//! que a [`crate::PlayerStep::gravity_hold`] não é consultada neste caminho: ela
//! existe para a ponte dinâmica dividir o impulso entre sub-passos, e aqui não
//! há sub-passo nenhum a dividir.
//!
//! # ⚠️ E o CHÃO absorve a componente que aponta para ele — mas a RÉGUA importa
//!
//! Sem a absorção um personagem parado numa rampa **DESLIZA**: a gravidade entra
//! no deslocamento pedido, o controlador a projeta ao longo da superfície
//! (`slide`, que é o que faz uma parede não travar o movimento) e o resultado é
//! deriva morro abaixo. Medido, numa rampa de 30°: **−0,0279 m em 10 s**, e
//! insensível ao limite de rampa — não é o *auto-slide* do rapier, é o slide
//! genérico. É o `floor_stop_on_slope` do Godot, que shipa **ligado**.
//!
//! ⚠️ **A primeira versão desta lei absorvia com a régua ERRADA, e a medição a
//! pegou:** ela perguntava à [`crate::footing`], cujo alcance é o da PERNA
//! (`float_height + cling_distance`) — calibrado para uma cápsula que **paira**.
//! Sob Snap não há perna, e absorver a esse alcance congelava o personagem a
//! **0,4 m no ar**, exatamente onde nasceu, com todos os outros gates verdes (a
//! caminhada andava, a rampa não derivava).
//!
//! A cura não foi tirar a absorção — foi **corrigir a régua**: sob Snap o
//! `float_height` que a lei recebe é o
//! [`PhysicsWorld::body_foot_distance`](../../ph2d_physics/struct.PhysicsWorld.html#method.body_foot_distance),
//! *onde os pés deste corpo de facto ficam*. Aí *"estou no chão"* volta a
//! significar o que a palavra diz, e a [`crate::footing`] continua a porta ÚNICA
//! nos dois modos (K4).
//!
//! ⚠️ **Só a componente que aponta PARA o chão é absorvida**: zerar o eixo
//! inteiro mataria o pulo no tique da decolagem, em que o raio ainda vê o chão e
//! a subida já começou.

use crate::{Buoyed, GroundSample, Motor, Vec2, ground_carry};

/// **O MEIO em que este corpo está** — o sentido que o integrador cinemático lê
/// para a água existir (W-KinFluid).
///
/// # ⚠️ Por que a lei, e não o solver
///
/// No modo dinâmico o empuxo e o arrasto de uma zona chegam por `apply_impulse` no
/// corpo — e um corpo cinemático tem **massa infinita**, então o solver os ignora.
/// É a MESMA frase que explica por que o `move_shape` não empurrava um caixote
/// antes da `W-KinPush`, agora do outro lado: ele não *recebe*. Medido antes da
/// cura, numa poça funda de 4 s: o controle boiava a `1,1072 m` do ponto de
/// largada, o player dinâmico a `1,0893`, e o **cinemático afundava `139,67 m`** —
/// queda livre com o multiplicador de queda por cima.
///
/// A ponte já lia a [`Buoyed`] para a trava do fluido; o que faltava era a lei
/// **integrar** aquilo onde ela já integra a gravidade.
///
/// # ⚠️ Duas grandezas, e o par é o que impede uma mola sem amortecimento
///
/// Só o empuxo faz um oscilador conservativo: o personagem mergulha, é expulso,
/// volta a cair, para sempre. É a mesma frase que a fixture da poça já carregava
/// (*"empuxo sem resistência é uma mola sem amortecimento, e o repouso que este
/// oráculo lê nunca chegaria"*), e o `drag` é a metade que a fecha.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Fluid {
    /// Quantos pesos deste corpo o fluido carrega — a mesma grandeza da [`Buoyed`].
    pub buoyed: Buoyed,
    /// A resistência do meio, por segundo (o `AreaDrag` da zona).
    pub drag: f32,
    /// **A ACELERAÇÃO que os empurrões das zonas dão a este corpo**, m/s², em eixos
    /// de MUNDO (W-ZoneForce) — uma correnteza, uma esteira, uma coluna de vento.
    ///
    /// # ⚠️ Por que uma aceleração, e não a força
    ///
    /// Esta lei **não tem massa nenhuma**: ela possui uma velocidade e um `dt`. Quem
    /// tem a massa é o solver, e é ele quem divide — a consulta devolve `F/m` já
    /// pronto. É essa divisão, feita do outro lado, que preserva a assimetria que É a
    /// feature de uma zona de força: *a folha voa, o caixote não*.
    ///
    /// # ⚠️ E ela NÃO é escalada pelo empuxo
    ///
    /// O [`gravity_share`](Fluid::gravity_share) pesa a GRAVIDADE, porque o empuxo é
    /// uma força para cima proporcional ao peso. O empurrão de uma zona não tem
    /// relação nenhuma com o peso do corpo — escalá-lo junto faria a correnteza
    /// afrouxar exatamente onde a água carrega mais, que é o oposto do que a água faz.
    pub push: Vec2,
}

impl Default for Fluid {
    /// Ar seco — ver [`Fluid::DRY`], de que este é o espelho exigido pelo lint.
    fn default() -> Self {
        Self::DRY
    }
}

impl Fluid {
    /// Ar seco — o neutro, e o único valor com que a lei é **byte-idêntica** ao
    /// mundo de antes desta wave.
    pub const DRY: Self = Self {
        buoyed: Buoyed::DRY,
        drag: 0.0,
        push: [0.0, 0.0],
    };

    /// **A fração da gravidade que ainda pesa.** `1` seco · `0` boiando em
    /// equilíbrio · **negativo** para o que sobe.
    ///
    /// ⚠️ **Ela pode ficar negativa, e é isso que faz um corpo SUBIR** — a
    /// consulta deixou de capar a razão em `1` exactamente por isto (o teto
    /// binava já com metade do corpo fora d'água, e uma gravidade efetiva zero
    /// deixaria o personagem pendurado no meio da poça para sempre).
    ///
    /// ⚠️ **Não é clampada por baixo, de propósito:** uma rolha num fluido vinte
    /// vezes mais denso SOBE vinte vezes mais depressa do que cairia, e é isso
    /// que o dinâmico faz com a mesma zona. Capar aqui seria a lei a discordar do
    /// solver sobre a mesma água.
    #[must_use]
    pub fn gravity_share(self) -> f32 {
        1.0 - self.buoyed.0
    }
}

/// **A velocidade que o modo cinemático possui** — o que o solver possuiria se
/// o corpo fosse dinâmico.
///
/// ⚠️ Um tipo e não um `Vec2` solto: ele mora no [`crate::PlayerState`], que é
/// o valor que a fita guarda, e um par de `f32` anônimo ali seria indistinguível
/// da memória do chão que já vive ao lado.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct KinematicState {
    /// Metros por segundo, em MUNDO.
    pub velocity: Vec2,
    /// **O mundo segurou-me no tique anterior?**
    ///
    /// ⚠️ **Esta NÃO é a resposta da lei sobre chão** (K4) — essa é a
    /// [`crate::footing`], e é ela que decide pulo, perdão do coyote, caminhada
    /// e agachar, nos DOIS modos. Esta é a pergunta do INTEGRADOR, que é outra:
    /// *"há alguma coisa a segurar-me AGORA?"*.
    ///
    /// As duas medem coisas diferentes de propósito, e colapsá-las foi medido:
    /// a `footing` tem uma faixa de tolerância (`cling_distance`) para o gesto
    /// não morrer num degrau, e absorver a gravidade dentro dessa faixa deixa o
    /// personagem **pendurado no ar** na borda dela — 1,237 m onde o chão está a
    /// 1,000, para sempre, com todos os outros gates verdes.
    ///
    /// Vem do controlador, que é quem tocou no mundo; e mora aqui — e não num
    /// mapa da ponte — pelo mesmo motivo que a velocidade: é o ring de tiques
    /// âncora que guarda este tipo.
    pub grounded: bool,
}

/// **O deslocamento que este tique PEDE** — e o estado que ele deixa.
///
/// O chão entra aqui e não na ponte (K7): ele já viaja no
/// [`crate::GroundSample`] desde a W3, e somá-lo fora seria uma segunda resposta
/// para *"quanto o chão me leva?"*.
///
/// ⚠️ **E ele entra pelo [`ground_carry`], nunca pela `ground_velocity` crua** —
/// ler o campo aqui contava a plataforma DUAS vezes (1,98× medido; a tabela e a
/// ablação estão no doc daquela porta). O parâmetro é a AMOSTRA e não um `Vec2`
/// justamente para que passar o número cru volte a ser inexprimível.
///
/// ⚠️ **A velocidade do chão não é ESCRITA na velocidade guardada por esta
/// função**, e a distinção continua a valer para o que ela ainda paga: o que a
/// plataforma acrescenta pelo CONTATO é deslocamento **deste** tique. O que o
/// personagem de facto *possui* da plataforma vem da [`crate::walk`], que o
/// acelera até o referencial do chão e o freia ao sair dela — e é por isso que
/// ele sai de um vagão com o impulso dele em vez de colado a ele.
/// **A velocidade sem a parte que o chão já segura** — a porta ÚNICA da
/// absorção, e ela tem DOIS consumidores.
///
/// # ⚠️ O segundo consumidor é a LEI, e é por isso que isto é uma função
///
/// O integrador a chama para a gravidade não se acumular contra o chão. Mas a
/// **lei da caminhada** lê a mesma velocidade um passo ANTES, e enquanto ela
/// via a componente que o integrador ia apagar, o freio dela — que existe para
/// parar quem escorrega — lia a QUEDA como escorregão.
///
/// ⚠️ **O sintoma disso não é "ele freia demais", é uma DIREÇÃO:** o freio age
/// ao longo da tangente do chão, então cancelar a componente tangencial de uma
/// queda vertical deixa **só a componente normal** — o personagem deixa de cair
/// para baixo e passa a entrar na rampa **ao longo da normal dela**. Medido numa
/// rampa de 30°: **7,3 cm morro acima** em sete tiques, e o Enio desenhou a seta
/// exatamente na normal.
///
/// Medido pela ablação do knob `acceleration` (que desliga o freio pela porta do
/// artista): com freio `−0,0711 m`, sem freio `+0,0001 m` — o freio é a causa
/// inteira.
///
/// ⚠️ **A pergunta continua sendo a do INTEGRADOR** (ver
/// [`KinematicState::grounded`]) — o que mudou foi QUANDO ela é feita, não quem
/// a responde: a `footing` segue a porta única da lei sobre chão (K4), e este
/// `grounded` segue vindo de quem tocou no mundo.
/// # ⚠️ E ela absorve até um PISO, que é a descida que a superfície exige (§8.1)
///
/// A versão sem piso absorvia **toda** componente para baixo, e isso é certo
/// parado e errado a descer: numa rampa, a parte vertical da caminhada é o que
/// faz o personagem SEGUIR a superfície, e cancelá-la o deixa a andar na
/// horizontal enquanto o chão foge por baixo. Medido, o salto chegava a
/// **1,3258 m** a 40°/6 m/s, com o modo dinâmico em `0,0000` exato.
///
/// ⛔ **Trocar o EIXO pela normal foi tentado, medido e REVERTIDO** — o
/// personagem parado passava a derivar 0,7297 m e nunca assentava; o `up` está
/// aqui de propósito (é o `floor_stop_on_slope` do Godot). As duas metades
/// pedem coisas opostas do mesmo eixo, e a saída não é escolher uma.
///
/// O piso separa-as **sem trocar o eixo**, e a régua não é o alvo da caminhada
/// — é GEOMETRIA: *dada a minha velocidade TANGENCIAL, que descida seguir esta
/// superfície exige?* (`along × (tangente · up)`, nunca positiva).
///
/// | situação | `along` | piso | efeito |
/// |---|---|---|---|
/// | parado numa rampa | 0 | 0 | absorve tudo — **não escorrega** |
/// | a descer a 6 m/s | 6 | −2,54 | mantém exatamente o que a rampa pede |
/// | a subir | −6 | 0 (o `min`) | inalterado |
/// | a pousar parado | 0 | 0 | absorve tudo |
///
/// ⚠️ **O ponto fixo é estável, e é o que impede a rampa de virar um tobogã:** a
/// gravidade empurra `into` abaixo do piso, a absorção o traz de volta AO piso
/// (nunca acima), e o freio da caminhada devolve a tangencial ao alvo.
///
/// ⚠️ **`normal == up` reduz LITERALMENTE à lei anterior** — `perp_cw(up)` é
/// perpendicular a `up`, logo `tangente · up` é zero, logo o piso é zero e a
/// expressão é a de antes, termo a termo. É por isso que **chão plano e subida
/// são byte-idênticos**, e é a rede de segurança desta mudança: quem não sabe a
/// superfície passa `up` e recebe o mundo de ontem.
#[must_use]
pub fn supported_velocity(v: Vec2, grounded: bool, up: Vec2, floor: f32) -> Vec2 {
    if !grounded {
        return v;
    }
    let into = v[0] * up[0] + v[1] * up[1];
    // ⚠️ **`into < floor`, e não `!(into >= floor)`** — as duas divergem em
    // `NaN`: a primeira não absorve (o `NaN` passa intacto, como sempre passou),
    // a segunda o propagaria para os dois eixos.
    if into < floor {
        let take = into - floor;
        [v[0] - up[0] * take, v[1] - up[1] * take]
    } else {
        v
    }
}

/// **O piso da absorção: a descida que SEGUIR esta superfície exige** (§8.1) —
/// a definição, com dois chamadores.
///
/// ⚠️ **A referência é a velocidade que o personagem JÁ TINHA, nunca a que está
/// a ser corrigida**, e a diferença é a wave inteira. A 1ª versão media o
/// `along` sobre o `v` do tique — que já leva a gravidade somada —, e **a
/// gravidade tem componente TANGENCIAL numa rampa**: o piso lia o começo de um
/// escorregão como *"ele está a caminhar para baixo"* e o autorizava. Medido, o
/// personagem parado voltava a derivar **0,0299 m** (contra 0,0000 com a
/// referência certa). *Uma régua derivada do número que ela vai corrigir mede a
/// própria correção.*
///
/// A régua honesta é o estado de ENTRADA: o que ele possuía antes de este tique
/// lhe somar nada.
///
/// ⚠️ **Nunca positiva** (o `min`) — e a razão é CONSERVADORISMO, não correção.
///
/// Eu tinha escrito aqui que sem o `min` *"o personagem seria lançado ladeira
/// acima"*, e a **mutação que o remove SOBREVIVEU à suíte**. Medido então em
/// vez de argumentado (`measure_what_the_floors_min_does_to_a_climb`), a subida
/// caminhada em 4 s:
///
/// ```text
///   rampa      com o min      sem o min
///     10°         4.1118         4.1629   (+1,2%)
///     25°         9.9453        10.0732   (+1,3%)
///     40°        15.0329        15.2959   (+1,7%)
/// ```
///
/// Sem o `min` o piso fica POSITIVO a subir e a absorção **assiste a subida**
/// em vez de a lançar. É pequeno, é plausível, e **ninguém o mediu antes de
/// pedir** — o `min` existe para a metade *subida* ficar **byte-idêntica** ao
/// mundo de antes do piso, que é a promessa desta wave. Ligá-lo é uma decisão
/// própria, com cena própria. Gate: `the_floor_never_lifts_and_a_climb_is_untouched`.
///
/// ⚠️ **Superfície plana (ou `normal == up`) devolve ZERO exatamente** —
/// `perp_cw` é perpendicular a `up`, então o produto é zero — e é isso que faz
/// o chão plano e a subida serem **byte-idênticos** ao mundo de antes do piso.
///
/// # ⚠️ Só vale para movimento CONTINUADO no chão — num POUSO o piso é zero
///
/// A referência de quem acaba de aterrar é uma QUEDA, e uma queda vertical sobre
/// uma rampa **tem componente tangencial** (`v · perp_cw(n) = v·sen θ`): esta
/// função não a distingue de uma caminhada, e autorizá-la faz o personagem
/// pousar deslizando. Medido nesta fixture, largado na vertical sobre uma rampa:
/// **+0,01432 m** ao lado, contra `0,00000` com o piso gateado na continuidade.
///
/// ⇒ **quem chama gateia em *"ele já estava no chão no tique anterior?"***. É a
/// mesma pergunta do integrador (`KinematicState::grounded`, *"eu TOQUEI no
/// mundo?"*, respondida pelo tique passado), e não a do `footing` — as duas
/// parecem a mesma e é justamente no tique do CONTATO que elas discordam.
#[must_use]
pub fn surface_descent(reference: Vec2, normal: Vec2, up: Vec2) -> f32 {
    let t = crate::perp_cw(normal);
    let along = reference[0] * t[0] + reference[1] * t[1];
    (along * (t[0] * up[0] + t[1] * up[1])).min(0.0)
}

#[must_use]
pub fn kinematic_advance(
    state: KinematicState,
    motor: Motor,
    footing: Option<&GroundSample>,
    gravity: Vec2,
    up: Vec2,
    dt: f32,
    fluid: Fluid,
) -> (KinematicState, Vec2) {
    let carry = ground_carry(footing);

    // ⚠️ **A ÁGUA entra AQUI, e é a única linha que ela precisa** (W-KinFluid): o
    // empuxo é uma força para cima proporcional ao peso, então a soma dele com o
    // peso é uma gravidade ESCALADA — `g·(1 − buoyed)`, que é exactamente a
    // resultante que o solver dá ao corpo dinâmico na mesma poça. Seco, o fator é
    // `1.0` e a expressão é a de antes **ao bit** (`x * 1.0` é `x` em IEEE-754), o
    // que é a rede de segurança desta mudança inteira.
    //
    // ⚠️ **A gravidade escalada e o `motor.accel` NÃO são a mesma coisa, e a ordem
    // importa:** o motor traz o cancelamento de gravidade que a lei do pulo
    // autorou (a `gravity_hold`), e ele foi calculado contra a gravidade CHEIA —
    // escalar os dois juntos pagaria o empuxo duas vezes num pulo dentro d'água.
    // ⚠️ **E o EMPURRÃO da zona entra AQUI, ao lado do motor** (W-ZoneForce): ele já
    // chega como aceleração (a consulta dividiu pela massa REAL do solver), e é uma
    // aceleração que se SOMA — nunca um fator sobre a gravidade, como o empuxo é. Do
    // outro lado da cerca o solver faz exatamente isto: `apply_impulse(F·dt)` junto do
    // peso, no mesmo sub-passo. Fora de zona o termo é `0` e a expressão é a de antes.
    let g = fluid.gravity_share();
    let v = [
        state.velocity[0] + (gravity[0] * g + motor.accel[0] + fluid.push[0]) * dt + motor.boost[0],
        state.velocity[1] + (gravity[1] * g + motor.accel[1] + fluid.push[1]) * dt + motor.boost[1],
    ];

    // ⚠️ **E o MEIO resiste**, com a mesma lei que o `effector::apply` usa —
    // `v /= 1 + d·dt`, que é literalmente o integrador de `linear_damping` do
    // rapier. Escrevê-la como impulso seria uma segunda lei para a mesma palavra,
    // e as duas discordariam em `d` grande (um impulso atravessa o zero e empurra
    // o corpo para TRÁS; isto não pode).
    //
    // ⚠️ **A paridade com o dinâmico é aproximada, e o número é DESTA paridade:**
    // o solver amortece por SUB-PASSO e esta lei uma vez por TIQUE, então o que
    // ele aplica é `(1 + d·h)⁻⁴` contra o `(1 + d·4h)⁻¹` daqui. Um corpo
    // cinemático não tem sub-passo para dividir.
    //
    // ⚠️ **Isto foi precificado por ANALOGIA (*"a mesma classe que a W-AreaDrag
    // mediu em 1,25%"*) até 2026-08-10, e uma analogia com outra medição não é a
    // medição desta.** Medido em arrasto puro: **`1,149%` no pico (1 s)**, e a
    // decair — `0,257%` · `0,056%` · `0,018%`. A forma não é acidente: a
    // velocidade terminal é `g/d` nos DOIS por álgebra, então a divergência vive
    // só no transiente e não pode acumular. Gate:
    // `the_drag_parity_between_modes_stays_within_its_measured_price`.
    let v = if fluid.drag > 0.0 {
        let k = 1.0 / (1.0 + fluid.drag * dt);
        [v[0] * k, v[1] * k]
    } else {
        v
    };

    // ⚠️ **A absorção pede as DUAS respostas** (§8.3), e é a correção que a
    // varredura de rampa de 2026-08-09 obrigou.
    //
    // As duas perguntas parecem uma e são duas:
    //
    // - `state.grounded` — *"eu TOQUEI no mundo?"*, do INTEGRADOR, respondida
    //   pelo tique passado. Ela é o que impede a lei de receber uma queda que o
    //   passo vai apagar, e é ela que gateia o piso da §8.1 (a continuidade).
    // - `footing.is_some()` — *"isto é CHÃO?"*, a **K4**, que conhece o limite
    //   de rampa autorado.
    //
    // Só com a primeira, um personagem encostado numa parede de 60° tem a
    // gravidade absorvida e **fica imóvel numa rampa que a própria lei recusou**
    // (medido: `x = 0,0000` exato em 300 tiques a 60° e a 80°, contra o corpo
    // dinâmico a sair de quadro). O `max_slope` deixava de significar o que diz
    // — a metade *descer* do defeito que a W9 curou no *subir*.
    //
    // ⚠️ **Chão plano e rampa caminhável ficam BYTE-IDÊNTICOS**: ali as duas são
    // verdadeiras. Só muda o caso *toco-mas-não-é-chão*, e nele o deslizamento
    // do controlador já sabe o que fazer — ele só precisava de deslocamento para
    // redirecionar.
    //
    // ⚠️ **A superfície entra por aqui** (§8.1): sem ela a absorção cancela a
    // descida que a caminhada comandou e o personagem desce a rampa aos pulos.
    // Sem chão o piso é zero — `up` é o neutro exprimível.
    //
    // ⚠️ **A referência é `state.velocity`, o que ele tinha ANTES deste tique**,
    // e não o `v` de duas linhas acima: ver o aviso do [`surface_descent`].
    let floor = surface_descent(state.velocity, footing.map_or(up, |s| s.normal), up);
    let v = supported_velocity(v, state.grounded && footing.is_some(), up, floor);

    let wanted = [(v[0] + carry[0]) * dt, (v[1] + carry[1]) * dt];
    (
        KinematicState {
            velocity: v,
            ..state
        },
        wanted,
    )
}

/// **O que o mundo NÃO deixou acontecer** — a diferença entre o pedido e o
/// efetivo, devolvida como velocidade.
///
/// # ⚠️ Sem isto o personagem acelera contra uma parede para sempre
///
/// A velocidade que esta lei possui é uma ficção: nada no mundo a corrige. Um
/// personagem encostado num teto continuaria a somar `+v` para cima tique após
/// tique, e no instante em que o teto acabasse ele sairia disparado — o defeito
/// clássico de todo controlador cinemático escrito sem este passo.
///
/// # ⚠️ Só o que FREIA a velocidade PRÓPRIA é absorvido
///
/// A regra ingênua (`v −= (pedido − efetivo)/dt`) tem um artefato que o caso
/// mais comum de plataforma expõe: um personagem **parado** sobre um vagão,
/// prensado contra uma parede, tem o deslocamento do VAGÃO bloqueado — e a
/// subtração cega deixa-o com `v = −velocidade_do_vagão`. Ele fica quieto
/// enquanto o vagão o empurra e **dispara para trás** no instante em que sai
/// dele.
///
/// A lei é por componente e tem duas metades, ambas load-bearing:
///
/// 1. **o sinal** — só se absorve o que aponta no mesmo sentido da velocidade
///    própria (um bloqueio que impede a plataforma de me levar não é uma
///    velocidade minha a corrigir);
/// 2. **o teto** — nunca se remove mais do que existe, senão parar contra uma
///    parede vira um empurrão para o outro lado.
///
/// ⚠️ **E o que sobra é deslizar:** numa rampa, parte do pedido vira movimento
/// noutra direção sem nada ter sido *bloqueado* ali — a componente onde a
/// velocidade própria é zero passa intacta, que é o que mantém o deslize vivo.
#[must_use]
pub fn kinematic_settle(
    state: KinematicState,
    wanted: Vec2,
    effective: Vec2,
    grounded: bool,
    dt: f32,
) -> KinematicState {
    if dt <= 0.0 {
        return KinematicState { grounded, ..state };
    }
    let mut v = state.velocity;
    for i in 0..2 {
        let lost = (wanted[i] - effective[i]) / dt;
        // `clamp` entre 0 e a própria velocidade cobre as duas metades de uma
        // vez: fora do sentido de `v` o intervalo colapsa em zero, e dentro dele
        // o teto é `v` — o resultado nunca troca de sinal.
        v[i] -= lost.clamp(v[i].min(0.0), v[i].max(0.0));
    }
    KinematicState {
        velocity: v,
        grounded,
    }
}

#[cfg(test)]
#[path = "kinematic_tests.rs"]
mod tests;
