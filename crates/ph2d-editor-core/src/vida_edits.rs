//! ⭐⭐⭐ **O VOCABULÁRIO DA VIDA E DO DANO** (plano 28, W3) — o instantâneo e a edição, num módulo
//! abaixo do [`crate::action_bus`] e do [`crate::screens`].
//!
//! # ⛔ Por que ele não está no `screens::hero`, com os irmãos
//!
//! Pela mesma razão do [`crate::projectile_edits`] e do [`crate::weapon_edits`]: a catraca do DAG
//! tolera a aresta `action_bus → screens` num **tecto** e escreve a cura ao lado dela.
//!
//! # ⭐ UM vocabulário para DUAS secções, e é o desenho
//!
//! `Health` (quem leva) e `Damage` (quem bate) são secções diferentes no painel — um inimigo que
//! também magoa quando se lhe toca mostra as duas —, mas são **uma família**: o mesmo instantâneo
//! (as duas metades são `Option`), a mesma edição, o mesmo dreno. ⛔ Dois vocabulários pediriam à
//! shell duas listas de edições, duas publicações e dois drenos para a mesma entidade, e a shell é a
//! crate cujo tamanho uma catraca vigia.
//!
//! # ⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `no body` | a ponte só vê quem tem `RigidBody`: sem ele a vida nunca leva um golpe, e o dano nunca o dá | anexar o corpo (a paleta já o pede) |
//! | `dead` | a vida chegou a zero, e um morto é final na casa | rebobinar |
//! | `the clock is stopped` | a ponte corre no passo fixo | carregar no play |
//! | `it hurts nobody` | o dano é `0` | dar-lhe um valor |
//!
//! ⚠️ **A ordem é a em que o painel fala, e ela vive numa PORTA** ([`InspectorVidaInfo::queixa`]):
//! o painel pinta a frase e o gate mede-a sem pintar nada.

/// **A vida da lei, agora** — o readout, vindo do `HealthNow` que a ponte publica.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VidaAgora {
    pub pontos: f64,
    pub escudo: f64,
    pub morta: bool,
}

/// A metade VIDA do instantâneo — os vinte campos do `Health`.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorHealthInfo {
    pub max: f32,
    pub start: f32,
    pub invincible_s: f32,
    pub overheal: bool,
    pub regen: f32,
    pub regen_delay_s: f32,
    pub shield_start: f32,
    pub shield_max: f32,
    pub shield_duration_s: f32,
    pub shield_regen: f32,
    pub shield_regen_delay_s: f32,
    pub shield_blocks_excess: bool,
    pub armor_flat: f32,
    pub armor_percent: f32,
    pub dodge: f32,
    pub team: String,
    pub on_damage: String,
    pub on_heal: String,
    pub on_death: String,
    pub seed: u64,
    /// ⭐ A vida AGORA — `None` antes do 1.º tique (a vida ainda não nasceu).
    pub agora: Option<VidaAgora>,
}

impl InspectorHealthInfo {
    /// ⭐ **O objecto TEM escudo?** — a pergunta que decide se as quatro linhas do escudo aparecem.
    ///
    /// ⚠️ Um máximo **ou** um escudo inicial chega: com só o máximo, o escudo nasce vazio e regenera;
    /// com só o inicial, ele não tem tecto. As duas são configurações legítimas.
    #[must_use]
    pub fn tem_escudo(&self) -> bool {
        self.shield_start > 0.0 || self.shield_max > 0.0
    }
}

/// A metade DANO do instantâneo — os seis campos do `Damage`.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorDamageInfo {
    pub amount: f32,
    pub team: String,
    pub per_second: bool,
    pub ignores_shield: bool,
    pub ignores_armor: bool,
    /// `true` = `OnHit::Vanish` (uma bala); `false` = `OnHit::Stay` (uma espada, um espinho).
    pub vanish: bool,
}

/// **De quem a barra mostra a vida, e o que ela encontrou** (plano 28, W4) — resolvido pela MESMA
/// porta que a ponte da barra usa ao desenhar, para o painel nunca dizer uma coisa e a tela outra.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BarraAlvo {
    /// Nenhum objecto (que não seja um molde) tem o nome escrito em `Target`.
    SemAlvo,
    /// O alvo existe e não tem `Health` (ou tem uma vida sem máximo nem início).
    SemVida,
    /// A vida que a barra mostra agora, e o máximo contra o qual ela mede.
    Mostra { pontos: f32, max: f32 },
}

/// A metade BARRA do instantâneo — os onze campos do `HealthBar` (plano 28, W4).
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorBarInfo {
    pub target: String,
    pub width: f32,
    pub height: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub fill: [f32; 4],
    pub trail: [f32; 4],
    pub back: [f32; 4],
    pub trail_delay_s: f32,
    pub trail_speed: f32,
    pub hide_when_full: bool,
    /// ⭐ O que a barra encontrou — é isto que a secção diz ANTES dos números.
    pub alvo: BarraAlvo,
}

/// Snapshot das secções HEALTH, DAMAGE e HEALTH BAR da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorVidaInfo {
    pub entity_bits: u64,
    pub health: Option<InspectorHealthInfo>,
    pub damage: Option<InspectorDamageInfo>,
    /// ⭐ A barra de vida (plano 28, W4) — a terceira metade da família.
    pub bar: Option<InspectorBarInfo>,
    /// O objecto tem `RigidBody`?
    pub has_body: bool,
    /// O relógio está a andar?
    pub clock_playing: bool,
    pub selected_count: usize,
}

/// **O que o painel diz ANTES dos números** — ver o cabeçalho.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VidaQueixa {
    /// Sem `RigidBody`: a ponte não vê o objecto.
    SemCorpo,
    /// A vida chegou a zero.
    Morto,
    /// O dano é `0`.
    NaoFere,
}

impl InspectorVidaInfo {
    /// ⭐⭐⭐ **A queixa, da mais ESPECÍFICA para a mais geral** — e `None` quando não há nenhuma.
    ///
    /// ⚠️ **Sem corpo vem primeiro**: com ele a faltar, nenhuma das outras chega a acontecer, e
    /// dizer «ele morreu» a quem não tem corpo é mandá-lo resolver a metade errada.
    ///
    /// ⚠️ **Só a VIDA e o DANO precisam de corpo**: uma barra sozinha (a do placar) não tem de onde
    /// o tirar, e acusá-la de «sem corpo» mandaria o artista pôr um corpo num placar.
    #[must_use]
    pub fn queixa(&self) -> Option<VidaQueixa> {
        if !self.has_body && (self.health.is_some() || self.damage.is_some()) {
            return Some(VidaQueixa::SemCorpo);
        }
        if self
            .health
            .as_ref()
            .and_then(|h| h.agora)
            .is_some_and(|a| a.morta)
        {
            return Some(VidaQueixa::Morto);
        }
        if self.damage.as_ref().is_some_and(|d| d.amount <= 0.0) {
            return Some(VidaQueixa::NaoFere);
        }
        None
    }
}

/// Uma edição de um campo das secções HEALTH e DAMAGE.
#[derive(Clone, Debug, PartialEq)]
pub enum VidaFieldEdit {
    Max(f32),
    Start(f32),
    InvincibleS(f32),
    Overheal(bool),
    Regen(f32),
    RegenDelayS(f32),
    ShieldStart(f32),
    ShieldMax(f32),
    ShieldDurationS(f32),
    ShieldRegen(f32),
    ShieldRegenDelayS(f32),
    ShieldBlocksExcess(bool),
    ArmorFlat(f32),
    ArmorPercent(f32),
    Dodge(f32),
    Team(String),
    OnDamage(String),
    OnHeal(String),
    OnDeath(String),
    Seed(u64),
    DamageAmount(f32),
    DamageTeam(String),
    PerSecond(bool),
    IgnoresShield(bool),
    IgnoresArmor(bool),
    Vanish(bool),
    BarTarget(String),
    BarWidth(f32),
    BarHeight(f32),
    BarOffsetX(f32),
    BarOffsetY(f32),
    BarFill([f32; 4]),
    BarTrail([f32; 4]),
    BarBack([f32; 4]),
    BarTrailDelayS(f32),
    BarTrailSpeed(f32),
    BarHideWhenFull(bool),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vida() -> InspectorHealthInfo {
        InspectorHealthInfo {
            max: 100.0,
            start: 100.0,
            invincible_s: 0.0,
            overheal: false,
            regen: 0.0,
            regen_delay_s: 0.0,
            shield_start: 0.0,
            shield_max: 0.0,
            shield_duration_s: 5.0,
            shield_regen: 0.0,
            shield_regen_delay_s: 0.0,
            shield_blocks_excess: false,
            armor_flat: 0.0,
            armor_percent: 0.0,
            dodge: 0.0,
            team: String::new(),
            on_damage: String::new(),
            on_heal: String::new(),
            on_death: String::new(),
            seed: 0,
            agora: None,
        }
    }

    fn dano(amount: f32) -> InspectorDamageInfo {
        InspectorDamageInfo {
            amount,
            team: String::new(),
            per_second: false,
            ignores_shield: false,
            ignores_armor: false,
            vanish: false,
        }
    }

    fn info() -> InspectorVidaInfo {
        InspectorVidaInfo {
            entity_bits: 1,
            health: Some(vida()),
            damage: None,
            bar: None,
            has_body: true,
            clock_playing: true,
            selected_count: 1,
        }
    }

    /// ⚠️ **Todo campo dos dois componentes tem uma edição** — um campo sem variante é um knob que o
    /// painel mostra e que ninguém pode mexer. ⛔ A lista é escrita à mão de propósito: ela é a
    /// SEGUNDA leitura dos componentes, e é a discordância entre as duas que acusa o esquecimento.
    #[test]
    fn todo_campo_dos_dois_componentes_tem_uma_edicao() {
        let variantes = [
            VidaFieldEdit::Max(0.0),
            VidaFieldEdit::Start(0.0),
            VidaFieldEdit::InvincibleS(0.0),
            VidaFieldEdit::Overheal(false),
            VidaFieldEdit::Regen(0.0),
            VidaFieldEdit::RegenDelayS(0.0),
            VidaFieldEdit::ShieldStart(0.0),
            VidaFieldEdit::ShieldMax(0.0),
            VidaFieldEdit::ShieldDurationS(0.0),
            VidaFieldEdit::ShieldRegen(0.0),
            VidaFieldEdit::ShieldRegenDelayS(0.0),
            VidaFieldEdit::ShieldBlocksExcess(false),
            VidaFieldEdit::ArmorFlat(0.0),
            VidaFieldEdit::ArmorPercent(0.0),
            VidaFieldEdit::Dodge(0.0),
            VidaFieldEdit::Team(String::new()),
            VidaFieldEdit::OnDamage(String::new()),
            VidaFieldEdit::OnHeal(String::new()),
            VidaFieldEdit::OnDeath(String::new()),
            VidaFieldEdit::Seed(0),
            VidaFieldEdit::DamageAmount(0.0),
            VidaFieldEdit::DamageTeam(String::new()),
            VidaFieldEdit::PerSecond(false),
            VidaFieldEdit::IgnoresShield(false),
            VidaFieldEdit::IgnoresArmor(false),
            VidaFieldEdit::Vanish(false),
            VidaFieldEdit::BarTarget(String::new()),
            VidaFieldEdit::BarWidth(0.0),
            VidaFieldEdit::BarHeight(0.0),
            VidaFieldEdit::BarOffsetX(0.0),
            VidaFieldEdit::BarOffsetY(0.0),
            VidaFieldEdit::BarFill([0.0; 4]),
            VidaFieldEdit::BarTrail([0.0; 4]),
            VidaFieldEdit::BarBack([0.0; 4]),
            VidaFieldEdit::BarTrailDelayS(0.0),
            VidaFieldEdit::BarTrailSpeed(0.0),
            VidaFieldEdit::BarHideWhenFull(false),
        ];
        assert_eq!(
            variantes.len(),
            20 + 6 + 11,
            "o `Health` tem VINTE campos, o `Damage` SEIS e o `HealthBar` ONZE — se um nasceu, ele \
             precisa de uma variante aqui e de uma row no painel"
        );
    }

    /// ⭐⭐ **A ordem da queixa** — sem corpo ganha a tudo, e cada degrau só fala quando os de cima
    /// estão calados.
    #[test]
    fn a_queixa_fala_da_mais_especifica_para_a_mais_geral() {
        assert_eq!(info().queixa(), None, "uma vida sã não se queixa");

        let mut morto = info();
        morto.health.as_mut().unwrap().agora = Some(VidaAgora {
            pontos: 0.0,
            escudo: 0.0,
            morta: true,
        });
        assert_eq!(morto.queixa(), Some(VidaQueixa::Morto));

        let mut sem_corpo = morto.clone();
        sem_corpo.has_body = false;
        assert_eq!(
            sem_corpo.queixa(),
            Some(VidaQueixa::SemCorpo),
            "sem corpo vem ANTES de morto"
        );

        let mut inofensivo = info();
        inofensivo.damage = Some(dano(0.0));
        assert_eq!(inofensivo.queixa(), Some(VidaQueixa::NaoFere));
        inofensivo.damage = Some(dano(10.0));
        assert_eq!(inofensivo.queixa(), None, "o CONTROLO: um dano de 10 fere");
    }

    /// ⭐ **Uma barra SOZINHA não precisa de corpo** — a do placar mostra a vida de outro objecto, e
    /// acusá-la de «sem corpo» mandaria o artista pôr um corpo num placar. ⚠️ Com o CONTROLO: a
    /// mesma falta de corpo com uma vida ao lado continua a queixar-se.
    #[test]
    fn uma_barra_sozinha_nao_se_queixa_de_corpo() {
        let mut so_barra = info();
        so_barra.health = None;
        so_barra.has_body = false;
        assert_eq!(so_barra.queixa(), None);
        so_barra.health = Some(vida());
        assert_eq!(so_barra.queixa(), Some(VidaQueixa::SemCorpo));
    }

    /// ⚠️ **O escudo aparece por UMA de duas portas** — o máximo ou o inicial.
    #[test]
    fn o_escudo_aparece_pelo_maximo_ou_pelo_inicial() {
        let mut v = vida();
        assert!(!v.tem_escudo());
        v.shield_max = 50.0;
        assert!(v.tem_escudo());
        v.shield_max = 0.0;
        v.shield_start = 20.0;
        assert!(v.tem_escudo());
    }
}
