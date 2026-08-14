//! **OS SENTIDOS** — o que o mundo diz à lei, antes de ela responder.
//!
//! ⚠️ **Corte por ASSUNTO, e a linha já estava nomeada dentro do
//! [`crate::player_motor`]:** o doc dele diz, desde a W10, que *"o pacote certo,
//! no dia em que valer a pena, é os SENTIDOS"* — o que o corpo PERCEBE
//! ([`Buoyed`], [`GroundSample`]) e o que o dedo do jogador PEDE
//! ([`PlayerInput`]), contra o que a lei RESPONDE (o `Motor` e a porta única,
//! que ficam no pai).
//!
//! ⚠️ **O dia barato era este** — o `lib.rs` cruzou o teto de 700 LOC com a
//! `W-Swim` —, e o corte é o mesmo que o `contract.rs` já fez ao lado: o pai
//! fica com *o que o personagem faz*, os filhos com *com que vocabulário se
//! pergunta*.
//!
//! Re-exportados na raiz, então **nenhum caminho de chamador muda**.

use crate::Vec2;

/// **QUANTO DO PESO o fluido está a carregar**, em `[0, 1]` — o sentido que diz
/// à lei que ela **não está num arco balístico**.
///
/// # ⚠️ Por que a lei precisa disto, e por que não é "quanto estou submerso"
///
/// A modelagem do arco de um pulo — leve no ápice, pesada na queda — descreve um
/// corpo em **voo livre**, onde a gravidade é a única força e o arco é o produto
/// dela. Quando é o **empuxo** quem o segura, os mesmos multiplicadores viram
/// **amplificação paramétrica**: pesado ao descer injeta mais energia do que
/// leve ao subir devolve, ciclo após ciclo (medido no produto: largado numa poça
/// ele oscila `−1,05 / +4,71 / +12,08 / −20,31` e sai da cena).
///
/// ⚠️ **A grandeza é a razão empuxo÷peso, e a diferença foi MEDIDA:** à tona, a
/// cápsula de controle desta linha submerge **26%** da área — uma lei que
/// desvanecesse por *"quanto está molhado"* deixaria **74% da bomba ligada
/// exactamente onde o personagem passa a vida**. A razão vale `1` ali por
/// construção, porque *boiar em repouso* **é** o empuxo igualar o peso.
///
/// # ⚠️ A LEI é uma trava, e ela mora no [`crate::JumpState`]
///
/// A lei não desvanece com a fração — ela **cala** enquanto o fluido tiver o
/// corpo, e só re-arma num contato com o CHÃO. O porquê (a energia é ganha no
/// AR, entre dois mergulhos, onde não há fluido nenhum a medir) está escrito em
/// [`crate::JumpState::waterborne`], com os números da ablação.
///
/// ⚠️ **O valor CONTÍNUO fica mesmo assim**, e não é enfeite: ele é o que a
/// sonda imprime para verificar a teoria (à tona a razão vale `1,0000` porque
/// *boiar em repouso* **é** o empuxo igualar o peso, e foi essa leitura que
/// provou que o personagem passou a assentar na linha do controle). Um `bool`
/// vindo da ponte teria escondido isso.
///
/// ⚠️ **E a MAGNITUDE não é load-bearing na lei — só o sinal é.** Está medido:
/// uma mutação que troca o denominador (a razão deixa de ser peso e vira outra
/// coisa) sangra **dois gates da CONSULTA e nenhum do produto**, porque a trava
/// pergunta `> 0`. Escrito aqui para ninguém a ler como um número que a lei
/// pesa; quem o pesa é quem lê a sonda, e a wave que trouxer natação.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Buoyed(pub f32);

impl Buoyed {
    /// Ar seco — o neutro, e o que uma cena sem poça produz.
    pub const DRY: Self = Self(0.0);

    /// **O fluido carrega ALGUMA parte deste peso?** — a única pergunta que a
    /// lei faz a este sentido.
    ///
    /// ⚠️ **O predicado mora AQUI, e não na ponte**, porque é a lei que depende
    /// dele: uma ponte que publicasse um `bool` teria decidido o limiar longe do
    /// único código que sabe o que ele significa.
    ///
    /// ⚠️ **`NaN` conta como SECO**, e é a escolha segura: uma zona degenerada
    /// não pode calar a modelagem que o artista autorou (`NaN > 0.0` é falso, e
    /// esta linha existe para dizer que isso é intencional, não descuido).
    #[must_use]
    pub fn carries_weight(self) -> bool {
        self.0 > 0.0
    }
}

/// **O que o sensor de chão viu.** `None` no chamador significa *"nada ao
/// alcance"*, e a lei lê isso como estar no ar.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GroundSample {
    /// Distância do ponto de origem do raio até a superfície.
    pub distance: f32,
    /// A normal da superfície.
    ///
    /// ⚠️ Pode vir **degenerada** (`[0, 0]`) quando o raio nasce DENTRO da
    /// geometria — é o contrato do `cast_ray` do wrapper, que reporta a
    /// penetração em vez de a esconder. A [`footing`] a trata como chão plano:
    /// não sabemos a orientação, e a suposição menos daninha é a que deixa a
    /// mola empurrar o personagem para fora.
    pub normal: Vec2,
    /// **A velocidade do CHÃO no ponto de contato.**
    ///
    /// ⚠️ É ela que faz a plataforma móvel cair de graça: tudo nesta lei é
    /// medido *relativo ao chão*, então andar sobre um vagão é andar, e o vagão
    /// acelerando não derruba ninguém. Um chão estático manda `[0, 0]`.
    ///
    /// ⚠️ **Uma ESTEIRA chega por aqui** (`W-Surface`), e não por um campo
    /// próprio: a lei já mede tudo relativo ao chão, e uma correia é literalmente
    /// *um chão que anda sem o corpo andar*. Quem soma a velocidade de correia é
    /// a ponte, ao longo da TANGENTE da superfície que o raio acertou — ver
    /// [`Self::grip`] para a outra metade do que uma superfície diz.
    pub ground_velocity: Vec2,
    /// **Este chão é uma plataforma jump-through?** (W12)
    ///
    /// ⚠️ **É o SENSOR quem responde, e é por isso que o campo mora aqui:** a
    /// lei precisa saber *que tipo de chão* achou para decidir o que o botão de
    /// pulo significa neste tique (pular, ou DESCER através dele), e a única
    /// coisa que sabe se um collider é one-way é quem o consultou. Derivá-lo
    /// noutro lugar seria uma segunda resposta para um fato que a amostra já
    /// carrega.
    ///
    /// Chão comum manda `false`, e é isso que mantém a wave inteira inerte em
    /// toda cena que nunca autorou uma plataforma jump-through.
    pub one_way: bool,
    /// **Quanto desta superfície o pé aproveita** — o multiplicador do orçamento
    /// de aceleração e de travagem da [`crate::walk`] (`W-Surface`). Neutro
    /// **`1.0`**; gelo é baixo, borracha é alto.
    ///
    /// ⚠️ **É o SENSOR quem responde**, pela mesma razão do [`Self::one_way`]: a
    /// única coisa que sabe de que superfície é o chão achado é quem o consultou,
    /// e num leque de pés a resposta tem de vir do MESMO raio que ganhou — senão
    /// o personagem anda no gelo e derrapa na madeira no mesmo tique.
    ///
    /// ⚠️ **NÃO é o `friction` do collider, e a razão é estrutural:** a perna
    /// FLUTUA (`crate::ride`), então o atrito de contato do solver nunca se
    /// aplica a este personagem — nem uma vez. Acoplá-los faria *"esta rampa é
    /// escorregadia para o personagem"* significar também *"e todo caixote
    /// desliza nela"*.
    ///
    /// ⚠️ **`1.0` reduz LITERALMENTE ao mundo de antes desta wave** (`x * 1.0` é
    /// `x` em IEEE-754) — é isso que mantém o `physics_ecs_c9` byte-idêntico e
    /// toda cena que nunca autorou uma superfície intocada.
    ///
    /// ⚠️ **`0.0` é legítimo e significa gelo PERFEITO:** o orçamento inteiro
    /// zera, o personagem conserva a velocidade que tem e não consegue nem
    /// arrancar nem parar. Não é um valor inválido a rejeitar — é o limite que a
    /// escala descreve, e ele sai da própria aritmética sem um `if`.
    pub grip: f32,
}

impl GroundSample {
    /// O `grip` de uma superfície que ninguém autorou — e o valor que a lei usa
    /// **no ar**, onde não há superfície nenhuma a que perguntar.
    pub const NEUTRAL_GRIP: f32 = 1.0;
}

/// **A entrada do jogador neste tick.**
///
/// ⚠️ Não é config e não é componente: é o que o dedo do jogador estava fazendo.
/// Hoje a ponte a guarda como estado transiente (set-and-hold); a partir da W7
/// ela vem de uma **fita por tick**, o que torna o player uma função de
/// `(tick, fita)` e devolve o scrub bit-exato que o resto do módulo tem.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct PlayerInput {
    /// O eixo de caminhada em `[-1, 1]`. Positivo é a direita.
    pub drive: f32,
    /// O botão de pulo está PRESSIONADO agora.
    ///
    /// ⚠️ O estado, não a borda. A borda é derivada pela lei
    /// ([`JumpState::was_held`]), e tem de ser: quem a derivasse do lado de fora
    /// precisaria de uma segunda memória do mesmo fato, e as duas divergiriam no
    /// primeiro tick em que um dispatch devesse mais de um passo.
    pub jump: bool,
    /// **O botão de BAIXO está pressionado agora** (W12).
    ///
    /// ⚠️ Ele não anda para lugar nenhum sozinho — hoje serve a uma pergunta
    /// só: *o que o botão de pulo significa em cima de uma plataforma
    /// jump-through?* Segurado, o pulo vira **descida**
    /// ([`PlayerStep::drop_through`]).
    ///
    /// ⚠️ **É `down + jump`, e não `down` sozinho, de propósito:** um jogador
    /// que segura baixo enquanto anda não pode cair da plataforma sem ter
    /// pedido, e o dia em que existir um AGACHAR o botão já estará lá com o
    /// significado certo. É o idioma de Celeste, Hollow Knight, Ori e Dead
    /// Cells.
    pub down: bool,
    /// **O botão de ARRANQUE está pressionado agora** (W14).
    ///
    /// ⚠️ O estado, não a borda — a lei a deriva sozinha
    /// ([`DashState::was_held`]), pela razão exata do pulo: uma segunda memória
    /// do mesmo fato do lado de fora divergiria no primeiro dispatch que deve
    /// mais de um tique.
    pub dash: bool,
    /// **O botão de AGARRAR está pressionado agora** (W23).
    ///
    /// ⚠️ O estado, como os outros três — e aqui nem sequer há borda a derivar:
    /// agarrar-se é um regime que dura enquanto o dedo dura.
    pub grab: bool,
}
