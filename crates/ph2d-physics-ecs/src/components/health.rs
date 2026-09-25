//! **VIDA e DANO** (plano 28, W2) — os dois componentes de CONFIG, nunca estado vivo de solver
//! (ADR-0131: *«o undo ordena por bytes»*).
//!
//! A lei vive na crate-folha [`ph2d_health`] (porte da extensão *Health* do GDevelop, MIT, ao bit
//! contra o oráculo corrido); a ponte que a corre é `bridge::health`. Aqui só está o que o artista
//! autora e o que o ficheiro guarda.
//!
//! # A divisão: quem LEVA tem [`Health`], quem BATE tem [`Damage`]
//!
//! A pesquisa (doc 27 §3) mediu as duas formas que a indústria usa — o *hitbox/hurtbox* do Godot e o
//! *«o alvo pergunta a quem o tocou»* do Unity — e as duas separam **quem bate** de **quem leva**.
//! ⇒ uma espada, uma bala, um espinho e a lava são o MESMO componente, e um inimigo que também magoa
//! quando se lhe toca carrega os DOIS.
//!
//! # ⚠️ O que eles NÃO guardam
//!
//! O valor vivo, o escudo vivo, os relógios da invencibilidade, o gerador da esquiva e *«quem está a
//! tocar quem»* **mudam por tique** ⇒ vivem no estado da ponte, dentro do `ControllerMemory` que vai
//! para o anel de checkpoints (plano 28 §2 e §8.1). ⛔ Aqui fariam o undo desta casa ver **cada
//! quadro como um passo**.

use bevy_ecs::prelude::Component;
use serde::{Deserialize, Serialize};

use super::signal::signal_name;

/// **Uma VIDA.** Ver o cabeçalho do módulo.
///
/// Os valores de fábrica são os do alvo (lidos no `inicial` de toda fixtura do oráculo), com uma
/// diferença declarada: o [`Self::shield_duration_s`] do alvo é `5` e só significa algo com um
/// escudo — que de fábrica é `0`.
///
/// ⚠️ **Registado desde a W3**, no mesmo commit que a secção do Inspector — ver a nota no
/// `register_physics_components`: sem o registo, a cópia de um molde nascia SEM vida.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // três perguntas independentes, cada uma um campo do alvo
pub struct Health {
    /// O máximo. `0` = **sem máximo** (a lei do alvo; nada limita a vida por cima).
    pub max: f32,
    /// A vida com que o objecto NASCE (e renasce a cada rebobinar).
    pub start: f32,
    /// A invencibilidade depois de um golpe que ENTRA, em segundos. `0` desliga.
    ///
    /// ⚠️ É ela que faz *«dano a dobrar num golpe»* — a queixa nº 1 da pesquisa — morrer: o segundo
    /// golpe do mesmo toque cai dentro dela e é **ignorado por inteiro**.
    pub invincible_s: f32,
    /// Uma cura pode passar do máximo.
    pub overheal: bool,
    /// Regeneração, em pontos por segundo. `0` desliga.
    pub regen: f32,
    /// Quanto tempo depois do último golpe a regeneração arranca, em segundos.
    pub regen_delay_s: f32,
    /// O escudo com que o objecto nasce. `0` = sem escudo.
    pub shield_start: f32,
    /// O máximo do escudo. `0` = sem limite ao activar, e **sem regeneração**.
    pub shield_max: f32,
    /// Quanto dura um escudo depois de activado, em segundos. `≤ 0` = **nunca expira**.
    pub shield_duration_s: f32,
    /// Regeneração do escudo, em pontos por segundo.
    pub shield_regen: f32,
    /// Quanto tempo depois do último golpe o escudo começa a regenerar, em segundos.
    pub shield_regen_delay_s: f32,
    /// O golpe que PARTE o escudo não passa à vida.
    pub shield_blocks_excess: bool,
    /// Armadura plana: subtraída a cada golpe.
    pub armor_flat: f32,
    /// Armadura percentual em `0..1`, aplicada **depois** da plana (a ordem que o oráculo mediu).
    pub armor_percent: f32,
    /// A chance de esquivar em `0..1`.
    pub dodge: f32,
    /// **A equipa.** Um [`Damage`] da MESMA equipa (não vazia) não fere — o *fogo amigo* desligado.
    /// Vazio = sem equipa: fere e é ferido por toda a gente.
    pub team: String,
    /// O sinal que ele grita quando **leva dano** (vazio = calado).
    pub on_damage: String,
    /// O sinal que ele grita quando **é curado** (vazio = calado).
    pub on_heal: String,
    /// O sinal que ele grita quando **morre** (vazio = calado).
    pub on_death: String,
    /// A semente do sorteio da ESQUIVA — dois inimigos com a mesma semente esquivam igual.
    pub seed: u64,
    /// ⭐ **A pausa do golpe FINAL, em segundos** (plano 28, W5) — o jogo congela este tempo quando
    /// esta vida morre (a pesquisa: `~0,15 s`). `0` = sem pausa, o de fábrica.
    ///
    /// ⚠️ **É da vida e não de quem bate**: a morte de um chefe pesa mais do que a de um morcego,
    /// com a MESMA espada. O golpe comum pesa pela [`Damage::hitstop_s`] de quem bate.
    pub death_hitstop_s: f32,
    /// ⭐ **O PISCAR da invencibilidade** — quanto dura cada metade, em segundos (`0` = não pisca,
    /// o de fábrica). Ele dura sozinho a janela da invencibilidade: nenhum segundo número a manter
    /// igual ao `invincible_s`.
    pub blink_s: f32,
    /// ⭐ **Quanto do EMPURRÃO esta vida aceita** (plano 28, W5) — `1` = todo (o de fábrica), `0` =
    /// imóvel, `0,5` = metade. É a pesquisa a mandar: *o alvo decide* — o chefe pesado e o morcego
    /// levam a MESMA espada e voam distâncias diferentes porque a VIDA deles diz, não a massa do
    /// collider, que ninguém autorou como resistência.
    pub knockback_taken: f32,
    /// ⭐ **Os NÚMEROS de dano** (plano 28, W5) — cada golpe que ENTRA nesta vida faz nascer o
    /// número dele por cima, a subir e a desvanecer. Desligado (o de fábrica) = calado.
    ///
    /// ⚠️ **É da vida e não de quem bate** — pela mesma razão do empurrão: o artista quer os números
    /// sobre os inimigos e não sobre o herói, com a MESMA espada a bater nos dois.
    pub numbers: bool,
    /// A cor dos números, RGBA linear.
    pub numbers_color: [f32; 4],
    /// A altura dos números, em metros do mundo (eles crescem e encolhem com a câmera, como tudo o
    /// que vive na cena).
    pub numbers_size: f32,
}

impl Default for Health {
    fn default() -> Self {
        let lei = ph2d_health::Config::default();
        // ⚠️ Os números saem da lei, e não de uma segunda cópia: um valor de fábrica escrito aqui à
        // mão divergiria do que a bancada do oráculo mede no dia em que um dos dois mudasse.
        #[allow(clippy::cast_possible_truncation)]
        let f = |x: f64| x as f32;
        Self {
            max: f(lei.maximo),
            start: f(lei.maximo),
            invincible_s: f(lei.invencivel_s),
            overheal: lei.sobre_cura,
            regen: f(lei.regen),
            regen_delay_s: f(lei.regen_atraso_s),
            shield_start: 0.0,
            shield_max: f(lei.escudo_max),
            shield_duration_s: f(lei.escudo_duracao_s),
            shield_regen: f(lei.escudo_regen),
            shield_regen_delay_s: f(lei.escudo_regen_atraso_s),
            shield_blocks_excess: lei.escudo_bloqueia_excesso,
            armor_flat: f(lei.armadura_fixa),
            armor_percent: f(lei.armadura_pct),
            dodge: f(lei.esquiva),
            team: String::new(),
            on_damage: String::new(),
            on_heal: String::new(),
            on_death: String::new(),
            seed: 0,
            death_hitstop_s: 0.0,
            blink_s: 0.0,
            knockback_taken: 1.0,
            numbers: false,
            numbers_color: [1.0, 0.86, 0.3, 1.0],
            numbers_size: 0.45,
        }
    }
}

impl Health {
    /// **A porta ÚNICA** componente ⇒ lei. A ponte, o painel e os gates leem daqui.
    #[must_use]
    pub fn config(&self) -> ph2d_health::Config {
        let d = |x: f32| f64::from(x);
        ph2d_health::Config {
            maximo: d(self.max),
            invencivel_s: d(self.invincible_s),
            sobre_cura: self.overheal,
            esquiva: d(self.dodge),
            regen: d(self.regen),
            regen_atraso_s: d(self.regen_delay_s),
            escudo_max: d(self.shield_max),
            escudo_duracao_s: d(self.shield_duration_s),
            escudo_regen: d(self.shield_regen),
            escudo_regen_atraso_s: d(self.shield_regen_delay_s),
            escudo_bloqueia_excesso: self.shield_blocks_excess,
            armadura_fixa: d(self.armor_flat),
            armadura_pct: d(self.armor_percent),
        }
    }

    /// A equipa, ou `None` se estiver em branco (um espaço não é uma equipa).
    #[must_use]
    pub fn team(&self) -> Option<&str> {
        signal_name(&self.team)
    }
}

/// **O que acontece a quem bate, depois de bater.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OnHit {
    /// Fica (uma espada, um espinho, a lava).
    #[default]
    Stay,
    /// Sai da cena no tique em que acerta (uma bala). ⚠️ Só sai quem **nasceu numa corrida**
    /// (`ph2d_ecs::is_transient`) — uma bala posta à mão é documento, e pára.
    Vanish,
}

/// **Um DANO** — o que fere quem tem [`Health`] ao tocar-lhe. Ver o cabeçalho do módulo.
#[derive(Component, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // três perguntas independentes
pub struct Damage {
    /// Quanto tira por golpe — ou por SEGUNDO, com [`Self::per_second`].
    pub amount: f32,
    /// A equipa: não fere uma [`Health`] da MESMA equipa (não vazia).
    pub team: String,
    /// ⭐ **Contínuo:** fere `amount` por segundo enquanto toca (a lava, o gás). Desligado (o de
    /// fábrica) fere **uma vez por contacto** — tem de deixar de tocar e voltar para ferir outra vez.
    pub per_second: bool,
    /// Atravessa o escudo (fere a vida directamente).
    pub ignores_shield: bool,
    /// Atravessa a armadura.
    pub ignores_armor: bool,
    /// O que acontece a quem bate, depois de bater.
    pub on_hit: OnHit,
    /// ⭐ **A PAUSA NO GOLPE, em segundos** (plano 28, W5; *hitstop*) — o jogo inteiro congela este
    /// tempo quando o golpe ENTRA (na vida ou no escudo; uma esquiva não pesa). A pesquisa mede
    /// `~0,05 s` para um golpe comum. `0` = sem pausa, o de fábrica.
    pub hitstop_s: f32,
    /// ⭐ **O EMPURRÃO, em m/s** (plano 28, W5) — a mudança de velocidade que o golpe dá a quem o
    /// leva, na DIRECÇÃO REAL do contacto (a normal que o solver achou; num sensor, do centro de
    /// quem bate ao de quem leva). `0` = não empurra, o de fábrica.
    pub knockback: f32,
    /// ⭐ **E para CIMA, em m/s** — somado ao empurrão, a direito. É o que faz um golpe num chão
    /// plano levantar o herói: ali a normal do contacto é horizontal e o empurrão sozinho arrasta-o.
    pub knockback_lift: f32,
}

impl Default for Damage {
    fn default() -> Self {
        Self {
            amount: 10.0,
            team: String::new(),
            per_second: false,
            ignores_shield: false,
            ignores_armor: false,
            on_hit: OnHit::Stay,
            hitstop_s: 0.0,
            knockback: 0.0,
            knockback_lift: 0.0,
        }
    }
}

impl Damage {
    /// A equipa, ou `None` se estiver em branco.
    #[must_use]
    pub fn team(&self) -> Option<&str> {
        signal_name(&self.team)
    }

    /// **Este dano fere esta vida?** — a regra do fogo amigo, numa porta só.
    ///
    /// ⚠️ Duas equipas VAZIAS ferem-se (o de fábrica: sem equipa ninguém é amigo de ninguém).
    #[must_use]
    pub fn fere(&self, alvo: &Health) -> bool {
        match (self.team(), alvo.team()) {
            (Some(a), Some(b)) => a != b,
            _ => true,
        }
    }
}

/// ⭐⭐ **A vida AGORA** — o que a ponte publica no mundo no fim de cada `dispatch`, para quem não
/// alcança a ponte: o Inspector (o readout *«Now: 70 / 100»*) e a barra de vida (W4).
///
/// ⛔⛔ **Ela é DERIVADA e NÃO é registada, e a ausência é a lei:** a fonte é o estado da ponte, que
/// vai no anel de checkpoints; registada, ela entraria no `.ph2dproj` e no `Ctrl+Z` como uma
/// segunda resposta a *«quanta vida ele tem?»*, e um undo devolveria o número de um instante com a
/// ponte noutro — o precedente é o `CounterRuntime`, que também não deriva `Serialize`.
///
/// ⚠️ Ausente antes do 1.º tique (a vida ainda não nasceu) — quem lê cai no `start` da config.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct HealthNow {
    /// Os pontos de vida agora.
    pub pontos: f64,
    /// O escudo agora.
    pub escudo: f64,
    /// Morreu (um morto é final na casa).
    pub morta: bool,
}
