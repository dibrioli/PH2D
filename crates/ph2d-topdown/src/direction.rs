//! **A QUANTIZAÇÃO da intenção** — em que direcções este corpo aceita andar.
//!
//! ⚠️⚠️ **Ela corre no espaço da INTENÇÃO, antes do viewpoint** (ver o cabeçalho da
//! crate): num tabuleiro isométrico, *«4 direcções»* tem de encaixar nas diagonais
//! do tabuleiro, e um `snap` feito depois da reprojecção encaixa nos eixos do
//! **ecrã**. As duas ordens compilam e só uma é um jogo isométrico.
//!
//! # ⚠️ E a diagonal NÃO é mais rápida
//!
//! Duas teclas dão `(1, 1)`, que tem comprimento `√2` — um corpo que ande isso
//! anda **41 % mais depressa na diagonal**, que é o defeito de principiante mais
//! antigo do género (e que o `Input.get_vector` do Godot cura normalizando). Aqui
//! a cura é da lei: o comprimento é **cortado a 1**, nunca esticado, para que um
//! manípulo analógico a meio curso continue a andar a meia velocidade.

use crate::{Vec2, len, normalize};

/// **Abaixo disto um eixo não está em baixo.** O mesmo epsilon do corte de comprimento desta porta.
const VIVO: f32 = 1.0e-6;
/// **Acima disto a intenção não está na diagonal EXACTA** — e aí manda a componente maior.
const EMPATE: f32 = 1.0e-6;

/// **Que eixo mandou por ÚLTIMO.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DominantAxis {
    /// Ninguém — nada em baixo, ou um modo que não lê isto.
    #[default]
    None,
    /// O horizontal.
    X,
    /// O vertical.
    Y,
}

/// **A MEMÓRIA da última seta a chegar** (ordem do dono, 2026-09-15: *«no 4 dir, mesmo com duas
/// setas pressionadas, a última a ser pressionada sempre é dominante»*).
///
/// # ⚠️ Por que a lei precisa de MEMÓRIA, e por que ela mora aqui
///
/// A intenção que chega a esta porta é um **vector**: `(1, 1)` não diz qual das duas setas desceu
/// primeiro. A única forma de saber é observar a **TRANSIÇÃO** — que eixo passou de parado a vivo
/// neste tique —, e isso obriga a lembrar o tique anterior.
///
/// ⚠️ Ela viaja dentro do [`crate::TopDownState`], que é o que o anel de checkpoints guarda: um
/// scrub devolve o mundo **e** quem mandava, e o replay reproduz a corrida. ⛔ Uma memória fora
/// dali seria o defeito que o `ControllerMemory` desta mesma wave existe para impedir.
///
/// ⚠️ **E ela não é opcional na assinatura da porta**: o [`quantize`] recebe o eixo dominante, e o
/// [`crate::world_direction`] recebe a memória. *Esquecê-la é erro de compilação* — que é a única
/// forma de uma lei escrita em duas metades não envelhecer numa delas.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Dominance {
    eixo: DominantAxis,
    vivo: [bool; 2],
}

impl Dominance {
    /// **Observa a intenção crua DESTE tique e devolve quem manda.**
    ///
    /// ⚠️ Chamada **uma vez por tique e por corpo** — chamá-la duas vezes com a mesma intenção
    /// come a transição, e a seta nova deixa de roubar o comando.
    pub fn observe(&mut self, raw: Vec2) -> DominantAxis {
        let vivo = [raw[0].abs() > VIVO, raw[1].abs() > VIVO];
        // ⭐ A transição, que é a coisa toda: *acabou de chegar* é `vivo agora && parado antes`.
        let chegou = [vivo[0] && !self.vivo[0], vivo[1] && !self.vivo[1]];
        self.vivo = vivo;
        self.eixo = match (vivo[0], vivo[1]) {
            // ⚠️ Soltar tudo **esquece**: senão a primeira seta do gesto seguinte herdaria um
            // comando velho, e o artista veria o boneco arrancar para o lado errado.
            (false, false) => DominantAxis::None,
            (true, false) => DominantAxis::X,
            (false, true) => DominantAxis::Y,
            (true, true) => match (chegou[0], chegou[1]) {
                (true, false) => DominantAxis::X,
                (false, true) => DominantAxis::Y,
                // ⚠️ **As duas no MESMO tique — ou nenhuma:** fica quem já mandava, e sem ninguém
                // a resposta é **declarada** (o horizontal). ⛔ Não há «última» aqui, e o que
                // importa é ser determinístico e igual nos quatro quadrantes — que é exactamente o
                // que o arredondamento de antes não era.
                _ => match self.eixo {
                    DominantAxis::None => DominantAxis::X,
                    ja_mandava => ja_mandava,
                },
            },
        };
        self.eixo
    }

    /// Quem manda agora, sem observar nada.
    #[must_use]
    pub const fn axis(self) -> DominantAxis {
        self.eixo
    }
}

/// **O ÍNDICE do encaixe que a dominância impõe** — `None` quando ela não tem nada a dizer.
///
/// ⚠️⚠️ **Ela só fala na diagonal EXACTA, e essa cerca é o que a torna correcta fora do teclado.**
/// Com componentes diferentes ganha a maior — que é o que o encaixe já fazia —, senão rodar um
/// manípulo faria o corpo andar para o lado errado a meio da volta: o eixo que cruzasse o limiar
/// por último mandaria mesmo com o manípulo a apontar o outro.
///
/// ⚠️ **E o índice é o que o ARREDONDAMENTO produz para cada seta SOZINHA** (`→` `0` · `↑` `1` ·
/// `←` `2` · `↓` `−1`), para que a saída seja byte-idêntica à da seta sem companhia. ⛔ Devolver um
/// `[0, 1]` exacto criaria duas aritméticas para o mesmo rumo: `libm::cosf(π/2)` é `−4,4e-8`, não
/// zero.
fn indice_dominante(bruto: Vec2, mode: DirectionMode, dominante: DominantAxis) -> Option<f32> {
    if !matches!(mode, DirectionMode::FourWay) {
        return None;
    }
    let (ax, ay) = (bruto[0].abs(), bruto[1].abs());
    if ax <= VIVO || ay <= VIVO || (ax - ay).abs() > EMPATE {
        return None;
    }
    match dominante {
        DominantAxis::None => None,
        DominantAxis::X => Some(if bruto[0] >= 0.0 { 0.0 } else { 2.0 }),
        DominantAxis::Y => Some(if bruto[1] >= 0.0 { 1.0 } else { -1.0 }),
    }
}

/// **Em que direcções o corpo aceita andar.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DirectionMode {
    /// Qualquer ângulo — o manípulo analógico e o rato.
    Free,
    /// Os oito rumos, de 45 em 45 graus. É o default, e é o do *8 Direction* do
    /// Construct e do *Top-down movement* do GDevelop com diagonais ligadas.
    #[default]
    EightWay,
    /// Só os quatro rumos cardeais. O sokoban, o roguelike, o Pokémon.
    FourWay,
    /// Só o eixo horizontal da intenção.
    AxisX,
    /// Só o eixo vertical da intenção.
    AxisY,
}

impl DirectionMode {
    /// Todos, na ordem em que o painel os mostra. ⛔ **É a FONTE da contagem** —
    /// um modo novo aparece no painel sem ninguém somar nada à mão.
    pub const ALL: [Self; 5] = [
        Self::Free,
        Self::EightWay,
        Self::FourWay,
        Self::AxisX,
        Self::AxisY,
    ];

    // ⛔⛔ **Aqui viveu um `label()` e o doc dele dizia *«o rótulo que o painel pinta»* — FALSO**
    // (apagado em 2026-09-19). Quem o painel pinta é o ESPELHO desta lei em
    // `ph2d_editor_core::topdown_edits`, cujo `label()` já resolve chaves (`panel.topdown.*`), e
    // esse espelho é DELIBERADO: o doc dele escreve que a tag do segmentado tem de ser separada da
    // variante da lei, senão reordenar uma reordena a outra.
    //
    // ⚠️ A varredura da workspace inteira não achou um chamador de produto: os únicos eram as
    // mensagens de `assert!` dos testes desta crate, que passaram a usar `{:?}`. ⇒ ÓRFÃO, e a cura
    // é apagar — traduzi-lo poria uma SEGUNDA palavra por variante na tabela, ao lado das que o
    // Inspector já lá tem.

    /// De quantos em quantos graus este modo encaixa — `None` é livre.
    ///
    /// ⚠️ Os dois modos de eixo **não** são um encaixe de 180°: eles apagam uma
    /// componente, e apagar não é o mesmo que rodar para o rumo mais perto (uma
    /// intenção a 80° daria `→` num encaixe e `↑` num apagamento de `x`).
    #[must_use]
    pub const fn snap_deg(self) -> Option<f32> {
        match self {
            Self::Free | Self::AxisX | Self::AxisY => None,
            Self::EightWay => Some(45.0),
            Self::FourWay => Some(90.0),
        }
    }
}

/// **A porta**: entrada crua ⇒ intenção quantizada, de comprimento `<= 1`.
///
/// O comprimento sobrevive (cortado a `1`) para que um manípulo a meio curso ande
/// a meia velocidade; só a **direcção** é encaixada.
#[must_use]
pub fn quantize(raw: Vec2, mode: DirectionMode, dominante: DominantAxis) -> Vec2 {
    let bruto = match mode {
        DirectionMode::AxisX => [raw[0], 0.0],
        DirectionMode::AxisY => [0.0, raw[1]],
        _ => raw,
    };
    let comprimento = len(bruto).min(1.0);
    if comprimento < 1.0e-6 {
        return [0.0, 0.0];
    }
    let Some(dir) = normalize(bruto) else {
        return [0.0, 0.0];
    };
    let Some(passo) = mode.snap_deg() else {
        // ⚠️ **Sem encaixe e dentro do corte, a saída é a ENTRADA, ao bit.** A 1.ª
        // redacção devolvia `dir * comprimento` — uma ida e volta por `normalize`
        // que muda o último bit e faz `Free + TopDown` deixar de ser identidade.
        // O gate `o_neutro_atravessa_a_porta_sem_tocar_no_vector` apanhou-o.
        return if len(bruto) <= 1.0 {
            bruto
        } else {
            [dir[0] * comprimento, dir[1] * comprimento]
        };
    };
    // ⚠️ `atan2`/`cos`/`sin` do `libm`, nunca do `std`: esta direcção entra na
    // velocidade de um corpo que o `physics_ecs_c9` compara entre TRÊS sistemas
    // operacionais, e ali 1 ulp é um bug.
    let ang = libm::atan2f(dir[1], dir[0]);
    let passo_rad = passo.to_radians();
    // ⭐⭐⭐ **A ÚLTIMA SETA MANDA** (ordem do dono, 2026-09-15) — e ela entra REESCREVENDO o índice
    // do encaixe, não devolvendo um vector à parte: ver [`indice_dominante`].
    let indice =
        indice_dominante(bruto, mode, dominante).unwrap_or_else(|| libm::roundf(ang / passo_rad));
    let encaixado = indice * passo_rad;
    [
        libm::cosf(encaixado) * comprimento,
        libm::sinf(encaixado) * comprimento,
    ]
}

/// **O valor de FIO** deste modo — o byte que o ficheiro guarda.
///
/// ⚠️⚠️ **Os números são EXPLÍCITOS e não a ordem da declaração**: o postcard é
/// posicional, e um dia em que alguém reordene as variantes por gosto trocaria o
/// modo de toda cena já gravada, **em silêncio**. Quem acrescenta um modo dá-lhe
/// o número seguinte e nunca mexe nos que já existem.
#[must_use]
pub const fn to_wire(m: DirectionMode) -> u8 {
    match m {
        DirectionMode::Free => 0,
        DirectionMode::EightWay => 1,
        DirectionMode::FourWay => 2,
        DirectionMode::AxisX => 3,
        DirectionMode::AxisY => 4,
    }
}

/// O inverso. ⚠️ Um byte desconhecido cai no **default**, nunca em pânico: um
/// ficheiro de uma versão mais nova tem de abrir com o modo mais parecido, e o
/// degrau de schema é quem recusa em voz alta quando é caso disso.
#[must_use]
pub const fn from_wire(v: u8) -> DirectionMode {
    match v {
        0 => DirectionMode::Free,
        2 => DirectionMode::FourWay,
        3 => DirectionMode::AxisX,
        4 => DirectionMode::AxisY,
        _ => DirectionMode::EightWay,
    }
}
