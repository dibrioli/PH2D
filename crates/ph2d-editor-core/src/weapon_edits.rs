//! ⭐⭐⭐ **O VOCABULÁRIO DA ARMA** — o instantâneo e a edição, num módulo abaixo do
//! [`crate::action_bus`] e do [`crate::screens`].
//!
//! # ⛔ Por que ele não está no `screens::hero`, com os irmãos
//!
//! Pela mesma razão do [`crate::tags_edits`], do [`crate::factory_edits`], do
//! [`crate::topdown_edits`], do [`crate::projectile_edits`] e do [`crate::ray_edits`], e com o
//! mesmo número atrás: a catraca do DAG tolera a aresta `action_bus → screens` num **tecto**, e
//! escreve a cura ao lado dela. *É mais um degrau da mesma migração, e cada um torna o resto mais
//! barato.*
//!
//! # ⭐⭐⭐ O que o painel DIZ que os oito campos sozinhos não diriam
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `this weapon has no trigger` | `on_signal` vazio ⇒ ela **nunca** dispara | ligar uma acção (o gatilho, #24) |
//! | `the shot goes nowhere` | ela dispara e `on_fire` está vazio ⇒ **nada nasce** | escrever o sinal que a fábrica ouve |
//! | `the magazine is not here` | há um nome de pente e **nenhum contador nesta entidade** | acrescentar o `Counter` com aquele nome |
//! | `it is dry for good` | o pente está a zero e `reload_ms` é `0` | dar-lhe uma recarga, ou munição |
//!
//! ⚠️ **As duas primeiras são de OUTRA espécie que as duas últimas:** ali a arma **não corre**;
//! aqui ela corre e não produz. *Dizer «o pente não está aqui» a quem não tem gatilho é mandá-lo
//! resolver a metade errada* — a lei da recusa dos pincéis, e é por isso que a ORDEM desta tabela é
//! a ordem em que o painel fala.
//!
//! ⭐⭐ **A terceira é a mais valiosa, e é a única que não se vê a olhar para os campos:** um
//! `ammo_counter` que aponta para um contador que vive **noutro objecto** deixa a arma com munição
//! **infinita**, em silêncio — a ponte escolheu a lei permissiva de propósito (*uma configuração a
//! meio não é um erro*), e é este aviso que a torna honesta.

/// ⭐⭐ **O TECTO das duas caixas de tempo, em milissegundos** — o ESPELHO de
/// `ph2d_ecs::WEAPON_MAX_MS`.
///
/// ⛔⛔ **Ele é uma segunda cópia e isso é DECLARADO:** esta crate **não vê o `ph2d-ecs`** (ela é o
/// vocabulário do editor, não do mundo), logo a alternativa seria um literal solto dentro do painel
/// — que é onde os dois números divergem em silêncio.
///
/// ⚠️ **A cópia só é honesta porque há um GATE a atá-la ao original**, e ele vive na
/// `ph2d-app-components`, que é a única crate que vê os dois lados. *Um espelho sem quem o compare
/// é a segunda resposta à mesma pergunta.*
pub const WEAPON_MAX_MS_UI: f64 = 60_000.0;

/// ⭐ **O passo de arrasto das duas caixas de tempo**, em milissegundos.
///
/// ⚠️ **`10` e não `1`**: a cadência de uma arma vive entre `50` e `1 000` ms, e um arrasto de um em
/// um milissegundo atravessaria a faixa útil em cem gestos. ⛔ Ele mora AQUI e não no painel porque
/// tem **dois** leitores — quem regista o widget e quem o pinta —, e duas cópias divergiriam no dia
/// em que uma delas fosse afinada.
pub const WEAPON_STEP_MS: f64 = 10.0;

/// Snapshot da secção WEAPON da entidade selecionada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InspectorWeaponInfo {
    pub entity_bits: u64,
    /// O sinal que puxa o gatilho — vazio = nunca dispara.
    pub on_signal: String,
    /// O intervalo mínimo entre tiros, em milissegundos. `0` = sem cadência.
    pub cooldown_ms: u64,
    /// O nome do contador que é o pente — vazio = munição infinita.
    pub ammo_counter: String,
    /// Quanto demora a recarregar, em milissegundos. `0` = não recarrega.
    pub reload_ms: u64,
    /// O sinal que manda recarregar antes de esvaziar.
    pub reload_on: String,
    /// O sinal publicado a cada tiro — o fio para a fábrica.
    pub on_fire: String,
    /// O clique seco.
    pub on_empty: String,
    /// O pente cheio.
    pub on_reloaded: String,
    /// ⭐ O nome do contador que é o DEPÓSITO. **Vazio = reserva infinita.**
    pub reserve_counter: String,
    /// ⭐ Quantas balas há no depósito, ou `None` se ele não for alcançável.
    ///
    /// ⚠️⚠️ **`None` tem DUAS causas e o painel separa-as** — o nome está vazio (reserva
    /// infinita, o caso normal) ou o nome é AMBÍGUO/ausente (dois objectos com o mesmo
    /// contador, e a arma cai em infinita sem que nada na tela o diga). *As curas são
    /// diferentes, logo os dois silêncios não podem ler-se igual* — a lei que o gatilho do
    /// #24 já paga entre `Desconhecida` e `SemTecla`.
    pub reserva: Option<i64>,
    /// ⭐ **Quantas balas ela tem AGORA**, e `None` quando a munição é infinita. Vem do
    /// `CounterRuntime` desta entidade, nunca de um campo.
    pub municao: Option<i64>,
    /// O pente cheio, do `Counter::start` desta entidade.
    pub pente: i64,
    /// ⭐ Ela está a recarregar agora? Vem do `WeaponRuntime` — a corrida, não a config.
    pub recarregando: bool,
    /// O relógio está a andar?
    pub clock_playing: bool,
    pub selected_count: usize,
}

impl InspectorWeaponInfo {
    /// ⭐⭐⭐ **A queixa, da mais ESPECÍFICA para a mais geral** — e `None` quando não há nenhuma.
    ///
    /// ⚠️ **Ela é uma PORTA e não quatro `if` no pintor**: o painel pinta a frase e o gate mede-a
    /// sem pintar nada. *Uma decisão que só existe dentro de um pintor não é testável sem um
    /// device, e um gate `#[ignore]` é um gate que o CI nunca corre.*
    #[must_use]
    pub fn queixa(&self) -> Option<WeaponQueixa> {
        if self.on_signal.trim().is_empty() {
            return Some(WeaponQueixa::SemGatilho);
        }
        if self.on_fire.trim().is_empty() {
            return Some(WeaponQueixa::SemSaida);
        }
        if !self.ammo_counter.trim().is_empty() && self.municao.is_none() {
            return Some(WeaponQueixa::PenteAusente);
        }
        // ⚠️ **Irmã do `PenteAusente`, e no mesmo degrau:** o artista nomeou um contador e ele não
        // é alcançável. ⛔ E aqui a causa pode ser AMBIGUIDADE (dois objectos com o mesmo nome),
        // que é a única maneira de o pedido dele ser recusado *por ter sido bem escrito duas vezes*.
        if !self.reserve_counter.trim().is_empty() && self.reserva.is_none() {
            return Some(WeaponQueixa::DepositoAusente);
        }
        if self.municao == Some(0) && self.reload_ms == 0 {
            return Some(WeaponQueixa::SecaParaSempre);
        }
        // ⚠️ **A MAIS GERAL, e a única que descreve uma corrida e não uma configuração:** o pente
        // está vazio E o depósito também. ⛔ Ela exige as DUAS metades — um depósito a zero com o
        // pente cheio é uma arma com a última carga, que é um estado normal e não uma queixa.
        if self.reserva == Some(0) && self.municao == Some(0) {
            return Some(WeaponQueixa::DepositoVazio);
        }
        None
    }
}

/// As quatro razões pelas quais uma arma pode não estar a fazer o que o artista espera.
///
/// ⛔ **Um enum e não uma chave de i18n**, pela lei que o `Brush::curva_inerte` da escultura pagou:
/// devolver a chave daqui poria a língua dentro de uma lei, e o censo do HR-15 não a veria.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WeaponQueixa {
    /// ⛔ Sem gatilho ela nunca dispara — nada acontece, de todo.
    SemGatilho,
    /// Ela dispara e o tiro não vai a lado nenhum: falta o sinal que a fábrica ouve.
    SemSaida,
    /// ⛔ O pente que ela nomeia **não está nesta entidade**, logo a munição é infinita em silêncio.
    PenteAusente,
    /// ⛔ O DEPÓSITO que ela nomeia não tem dono ÚNICO na cena — ou não existe, ou **dois** objectos
    /// carregam o mesmo contador. Nos dois casos a reserva volta a ser infinita, em silêncio.
    DepositoAusente,
    /// O pente está a zero e não há recarga — a única que pode ser o que o artista quer.
    SecaParaSempre,
    /// ⭐ O pente **e** o depósito estão os dois a zero: ela acabou. A mais geral das cinco, e a
    /// única que descreve uma CORRIDA e não uma configuração.
    DepositoVazio,
}

/// Uma edição de um campo da secção WEAPON.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WeaponFieldEdit {
    /// ⚠️ Os NOMES, crus — a mesma convenção do alvo do projéctil: quem apara é quem lê.
    OnSignal(String),
    CooldownMs(u64),
    AmmoCounter(String),
    ReloadMs(u64),
    ReloadOn(String),
    OnFire(String),
    OnEmpty(String),
    OnReloaded(String),
    /// ⭐ O nome do contador que é o DEPÓSITO.
    ReserveCounter(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> InspectorWeaponInfo {
        InspectorWeaponInfo {
            entity_bits: 1,
            on_signal: "fire".into(),
            cooldown_ms: 250,
            ammo_counter: "ammo".into(),
            reload_ms: 800,
            reload_on: "reload".into(),
            on_fire: "shot".into(),
            on_empty: "click".into(),
            on_reloaded: "ready".into(),
            // ⚠️ Reserva INFINITA na fixtura base, que é o valor de toda cena já gravada.
            reserve_counter: String::new(),
            reserva: None,
            municao: Some(6),
            pente: 6,
            recarregando: false,
            clock_playing: true,
            selected_count: 1,
        }
    }

    /// ⚠️ **Todo campo do componente tem uma edição** — um campo sem variante é um knob que o painel
    /// mostra e que ninguém pode mexer, e ele lê-se exactamente como um controlo morto.
    ///
    /// ⛔ A lista é escrita à mão de propósito: ela é a **segunda leitura** do componente, e é a
    /// discordância entre as duas que acusa o esquecimento.
    #[test]
    fn todo_campo_do_componente_tem_uma_edicao() {
        let variantes = [
            WeaponFieldEdit::OnSignal(String::new()),
            WeaponFieldEdit::CooldownMs(0),
            WeaponFieldEdit::AmmoCounter(String::new()),
            WeaponFieldEdit::ReloadMs(0),
            WeaponFieldEdit::ReloadOn(String::new()),
            WeaponFieldEdit::OnFire(String::new()),
            WeaponFieldEdit::OnEmpty(String::new()),
            WeaponFieldEdit::OnReloaded(String::new()),
            WeaponFieldEdit::ReserveCounter(String::new()),
        ];
        assert_eq!(
            variantes.len(),
            9,
            "o `WeaponFire` tem NOVE campos — se um nasceu, ele precisa de uma variante aqui e de \
             uma row no painel"
        );
    }

    /// ⭐⭐⭐ **A ordem das queixas É a lei** — da mais específica para a mais geral.
    ///
    /// ⚠️ Sem gatilho a arma **não corre**, logo dizer-lhe *«o pente não está aqui»* seria mandá-la
    /// resolver a metade errada. E `SecaParaSempre` é a única das quatro que pode descrever uma
    /// arma que o artista desenhou de propósito.
    ///
    /// **Mutações que devem sangrar:** trocar a ordem de dois braços · devolver `None` sem gatilho ·
    /// ler `municao == Some(0)` como pente ausente.
    #[test]
    fn a_queixa_vai_da_mais_especifica_para_a_mais_geral() {
        assert_eq!(
            base().queixa(),
            None,
            "uma arma armada e ligada nao se queixa"
        );

        let mut sem_gatilho = base();
        sem_gatilho.on_signal = "  ".into();
        // ⛔ E ela está TAMBÉM sem saída e sem pente, de propósito: é a ordem que decide qual sai.
        sem_gatilho.on_fire = String::new();
        sem_gatilho.municao = None;
        assert_eq!(
            sem_gatilho.queixa(),
            Some(WeaponQueixa::SemGatilho),
            "com tres queixas verdadeiras ao mesmo tempo, sai a mais ESPECIFICA — e um nome so' de \
             espacos e' um nome vazio"
        );

        let mut sem_saida = base();
        sem_saida.on_fire = String::new();
        sem_saida.municao = None;
        assert_eq!(sem_saida.queixa(), Some(WeaponQueixa::SemSaida));

        let mut ausente = base();
        ausente.municao = None;
        assert_eq!(
            ausente.queixa(),
            Some(WeaponQueixa::PenteAusente),
            "um nome de pente sem contador NESTA entidade da' municao infinita em silencio"
        );

        // ⭐ O CONTROLO da terceira: munição infinita AUTORADA (o nome vazio) não é queixa nenhuma.
        let mut infinita = base();
        infinita.ammo_counter = String::new();
        infinita.municao = None;
        assert_eq!(
            infinita.queixa(),
            None,
            "quem nao nomeia pente quer municao infinita, e isso nao e' um defeito"
        );

        // ⭐⭐ **O DEPÓSITO ausente** — o nome está lá e ninguém o carrega (ou dois carregam-no).
        let mut sem_deposito = base();
        sem_deposito.reserve_counter = "caixa".into();
        sem_deposito.reserva = None;
        assert_eq!(
            sem_deposito.queixa(),
            Some(WeaponQueixa::DepositoAusente),
            "um nome de deposito sem dono UNICO da' reserva infinita em silencio"
        );

        // ⭐ **O CONTROLO dele, e é o mesmo da terceira:** não nomear depósito é reserva infinita
        // AUTORADA — o caso de toda cena gravada antes de 2026-09-20.
        let mut sem_nome = base();
        sem_nome.reserve_counter = String::new();
        sem_nome.reserva = None;
        assert_eq!(sem_nome.queixa(), None);

        // ⭐⭐ **O DEPÓSITO vazio** — e ⚠️ ele exige as DUAS metades.
        let mut acabou = base();
        acabou.reserve_counter = "caixa".into();
        acabou.reserva = Some(0);
        acabou.municao = Some(0);
        assert_eq!(acabou.queixa(), Some(WeaponQueixa::DepositoVazio));

        // ⚠️ **O CONTROLO da quinta:** depósito a zero com o pente CHEIO é a última carga, que é um
        // estado normal e não uma queixa. *Sem esta linha, uma arma acabada de recarregar gritaria.*
        let mut ultima_carga = base();
        ultima_carga.reserve_counter = "caixa".into();
        ultima_carga.reserva = Some(0);
        assert_eq!(ultima_carga.queixa(), None);

        let mut seca = base();
        seca.municao = Some(0);
        seca.reload_ms = 0;
        assert_eq!(seca.queixa(), Some(WeaponQueixa::SecaParaSempre));

        // ⭐ E o CONTROLO da quarta: com recarga, um pente a zero é um estado NORMAL da corrida.
        let mut vazia_mas_recarrega = base();
        vazia_mas_recarrega.municao = Some(0);
        assert_eq!(vazia_mas_recarrega.queixa(), None);
    }
}
