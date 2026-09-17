//! ⭐⭐⭐ **O que o Inspector mostra e edita do EMISSOR DE PARTÍCULAS** (TOP-20 #18, W3).
//!
//! O painel não conhece o `ph2d-ecs` nem a simulação: ele recebe este instantâneo e devolve estas
//! edições. É o molde das secções irmãs (o projéctil, o cérebro, o script).
//!
//! ⚠️ **Os números viajam numa variante SÓ** ([`ParticlesFieldEdit::Number`]) — vinte e sete
//! variantes, uma por campo, davam vinte e sete braços iguais no dreno e vinte e sete sítios onde
//! esquecer um.

/// Qual NÚMERO do emissor uma edição mexe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticlesNumber {
    /// Partículas por ciclo (e o tecto de vivas).
    Amount,
    /// Segundos de vida.
    Life,
    /// Quanto a vida encurta, em fracção.
    LifeRandom,
    /// Quanto o ciclo aperta a emissão.
    Explosiveness,
    /// Segundos simulados antes do primeiro quadro.
    Prewarm,
    /// A velocidade do relógio.
    TimeScale,
    /// A semente.
    Seed,
    /// A largura (ou o raio) da forma de nascimento.
    ShapeW,
    /// A altura da forma de nascimento.
    ShapeH,
    /// A velocidade de saída.
    Speed,
    /// Quanto a velocidade varia.
    SpeedRandom,
    /// A direcção, em graus.
    Angle,
    /// A abertura do cone, em graus.
    Spread,
    /// A gravidade em `x`.
    GravityX,
    /// A gravidade em `y`.
    GravityY,
    /// O amortecimento.
    Damping,
    /// O lado de uma partícula.
    Size,
    /// Quanto o tamanho varia.
    SizeRandom,
    /// O tamanho ao morrer, em fracção.
    SizeEnd,
}

/// ⭐⭐ **A ORDEM dos números da secção, e ela vive AQUI, uma vez.**
///
/// O painel tem uma tabela de ids e o dreno um braço por campo; as três leem esta ordem, então uma
/// linha nova entra num sítio só. ⛔ Uma segunda lista escrita à mão é o defeito que o Inspector já
/// pagou (o id errado numa lista só é visível depois de ler quem a consome).
pub const PARTICLES_NUMBERS: [ParticlesNumber; 19] = [
    ParticlesNumber::Amount,
    ParticlesNumber::Life,
    ParticlesNumber::LifeRandom,
    ParticlesNumber::Explosiveness,
    ParticlesNumber::Prewarm,
    ParticlesNumber::TimeScale,
    ParticlesNumber::Seed,
    ParticlesNumber::ShapeW,
    ParticlesNumber::ShapeH,
    ParticlesNumber::Speed,
    ParticlesNumber::SpeedRandom,
    ParticlesNumber::Angle,
    ParticlesNumber::Spread,
    ParticlesNumber::GravityX,
    ParticlesNumber::GravityY,
    ParticlesNumber::Damping,
    ParticlesNumber::Size,
    ParticlesNumber::SizeRandom,
    ParticlesNumber::SizeEnd,
];

/// Qual TEXTO do emissor uma edição mexe — os quatro sinais.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticlesText {
    /// O sinal que liga.
    StartOn,
    /// O sinal que desliga.
    StopOn,
    /// O sinal que recomeça.
    RestartOn,
    /// O sinal que ele grita ao acabar.
    FinishedSignal,
}

/// A ORDEM dos textos — a mesma lei da [`PARTICLES_NUMBERS`].
pub const PARTICLES_TEXTS: [ParticlesText; 4] = [
    ParticlesText::StartOn,
    ParticlesText::StopOn,
    ParticlesText::RestartOn,
    ParticlesText::FinishedSignal,
];

/// Uma edição da secção **Particles**.
#[derive(Clone, Debug, PartialEq)]
pub enum ParticlesFieldEdit {
    /// Um número.
    Number(ParticlesNumber, f32),
    /// Começa a emitir com a corrida?
    Emitting(bool),
    /// Uma rajada só?
    OneShot(bool),
    /// A forma de nascimento (o índice do `EmissionShape`).
    Shape(u8),
    /// O espaço (`0` = World · `1` = Local).
    Space(u8),
    /// Uma cor — `true` = a do fim da vida.
    Color(bool, [f32; 4]),
    /// Um dos nomes de sinal.
    Text(ParticlesText, String),
}

/// **O que o painel mostra do emissor** — o instantâneo que a shell publica por quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorParticlesInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// Começa a emitir com a corrida?
    pub emitting: bool,
    /// Uma rajada só?
    pub one_shot: bool,
    /// Os números, na ordem da secção: ver [`ParticlesNumber`].
    pub amount: f32,
    /// Segundos de vida.
    pub life: f32,
    /// Quanto a vida encurta.
    pub life_random: f32,
    /// Quanto o ciclo aperta a emissão.
    pub explosiveness: f32,
    /// Segundos de pré-aquecimento.
    pub prewarm: f32,
    /// A velocidade do relógio.
    pub time_scale: f32,
    /// A semente.
    pub seed: f32,
    /// A forma de nascimento.
    pub shape: u8,
    /// O tamanho da forma.
    pub shape_size: [f32; 2],
    /// A velocidade de saída.
    pub speed: f32,
    /// Quanto ela varia.
    pub speed_random: f32,
    /// A direcção, em graus.
    pub angle: f32,
    /// A abertura do cone.
    pub spread: f32,
    /// A gravidade.
    pub gravity: [f32; 2],
    /// O amortecimento.
    pub damping: f32,
    /// O lado de uma partícula.
    pub size: f32,
    /// Quanto ele varia.
    pub size_random: f32,
    /// O tamanho ao morrer.
    pub size_end: f32,
    /// A cor ao nascer.
    pub color: [f32; 4],
    /// A cor ao morrer.
    pub color_end: [f32; 4],
    /// O espaço (`0` = World · `1` = Local).
    pub space: u8,
    /// O sinal que liga.
    pub start_on: String,
    /// O sinal que desliga.
    pub stop_on: String,
    /// O sinal que recomeça.
    pub restart_on: String,
    /// O sinal que ele grita ao acabar.
    pub finished_signal: String,
    /// O relógio está a andar? (Parado, nada nasce — e o painel di-lo.)
    pub clock_playing: bool,
    /// Quantas partículas vivem agora — o número que separa *«não configurei»* de *«não corre»*.
    pub alive: usize,
    /// Quantos objectos estão escolhidos (a edição aplica-se ao activo).
    pub selected_count: usize,
}

impl InspectorParticlesInfo {
    /// O valor de um número — a leitura que o painel faz pela [`PARTICLES_NUMBERS`].
    #[must_use]
    pub fn number(&self, which: ParticlesNumber) -> f32 {
        use ParticlesNumber as N;
        match which {
            N::Amount => self.amount,
            N::Life => self.life,
            N::LifeRandom => self.life_random,
            N::Explosiveness => self.explosiveness,
            N::Prewarm => self.prewarm,
            N::TimeScale => self.time_scale,
            N::Seed => self.seed,
            N::ShapeW => self.shape_size[0],
            N::ShapeH => self.shape_size[1],
            N::Speed => self.speed,
            N::SpeedRandom => self.speed_random,
            N::Angle => self.angle,
            N::Spread => self.spread,
            N::GravityX => self.gravity[0],
            N::GravityY => self.gravity[1],
            N::Damping => self.damping,
            N::Size => self.size,
            N::SizeRandom => self.size_random,
            N::SizeEnd => self.size_end,
        }
    }

    /// O texto de um sinal — a irmã da [`Self::number`].
    #[must_use]
    pub fn text(&self, which: ParticlesText) -> &str {
        match which {
            ParticlesText::StartOn => &self.start_on,
            ParticlesText::StopOn => &self.stop_on,
            ParticlesText::RestartOn => &self.restart_on,
            ParticlesText::FinishedSignal => &self.finished_signal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **O valor de um número, lido do instantâneo pela MESMA ordem da tabela** — é isto que faz a
    /// linha `i` da secção mostrar o campo `i` e editar o campo `i`.
    #[test]
    fn a_ordem_da_tabela_e_a_do_instantaneo() {
        let info = InspectorParticlesInfo {
            entity_bits: 1,
            emitting: true,
            one_shot: false,
            amount: 16.0,
            life: 1.0,
            life_random: 0.1,
            explosiveness: 0.2,
            prewarm: 0.3,
            time_scale: 1.5,
            seed: 7.0,
            shape: 1,
            shape_size: [2.0, 3.0],
            speed: 4.0,
            speed_random: 0.4,
            angle: 90.0,
            spread: 30.0,
            gravity: [0.0, -9.8],
            damping: 0.5,
            size: 0.15,
            size_random: 0.6,
            size_end: 0.25,
            color: [1.0; 4],
            color_end: [0.0, 0.0, 0.0, 1.0],
            space: 0,
            start_on: String::new(),
            stop_on: String::new(),
            restart_on: String::new(),
            finished_signal: String::new(),
            clock_playing: true,
            alive: 0,
            selected_count: 1,
        };
        let lidos: Vec<f32> = PARTICLES_NUMBERS.iter().map(|&n| info.number(n)).collect();
        assert_eq!(
            lidos,
            vec![
                16.0, 1.0, 0.1, 0.2, 0.3, 1.5, 7.0, 2.0, 3.0, 4.0, 0.4, 90.0, 30.0, 0.0, -9.8, 0.5,
                0.15, 0.6, 0.25
            ]
        );
        assert_eq!(PARTICLES_NUMBERS.len(), 19);
        assert_eq!(PARTICLES_TEXTS.len(), 4);
    }

    /// **Uma edição carrega o CAMPO e o valor** — e o número não sabe qual campo é, o que faz o
    /// dreno ter um braço por PERGUNTA e não por controlo.
    #[test]
    fn uma_edicao_diz_que_campo_e() {
        let a = ParticlesFieldEdit::Number(ParticlesNumber::Speed, 2.0);
        let b = ParticlesFieldEdit::Number(ParticlesNumber::Spread, 2.0);
        assert_ne!(
            a, b,
            "o mesmo valor em campos diferentes não é a mesma edição"
        );
        assert_eq!(
            ParticlesFieldEdit::Color(true, [1.0; 4]),
            ParticlesFieldEdit::Color(true, [1.0; 4])
        );
        assert_ne!(
            ParticlesFieldEdit::Color(true, [1.0; 4]),
            ParticlesFieldEdit::Color(false, [1.0; 4]),
            "a cor do fim não é a do princípio"
        );
    }
}
