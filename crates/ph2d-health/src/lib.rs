#![forbid(unsafe_code)]
//! ⭐⭐⭐ **A LEI DA VIDA** — um golpe, uma cura, a invencibilidade, a regeneração, o escudo e a
//! armadura (plano [28](../../../docs/Components/28_plano_vida_e_dano.md), W1).
//!
//! # De onde ela vem
//!
//! É o **porte** da extensão *Health* do GDevelop 5 (versão `0.4.0`, @4ian, **MIT** — a triagem da
//! pesquisa [27](../../../docs/Components/27_pesquisa_vida_e_dano.md) §2 parou na primeira porta
//! aberta), e a régua não é a leitura: é o **ORÁCULO CORRIDO** — o código que o próprio gerador do
//! GDevelop produz, a correr no runtime dele sem interface, gravado quadro a quadro em 19 cenários
//! ([`ferramentas/gdevelop_health`](../../../docs/Components/ferramentas/gdevelop_health/README.md)).
//! O arnês `tests/it/oraculo_do_gdevelop.rs` corre esta lei sobre os mesmos pedidos e exige os
//! mesmos números **ao bit**, em três momentos de cada quadro.
//!
//! # O quadro, na ordem do alvo
//!
//! 1. os dois relógios andam (`desde o golpe` e `duração do escudo`, em **milissegundos**
//!    acumulados, como os `gdjs.Timer` — é isso que faz trinta quadros darem `0,5000000000000002` s);
//! 2. [`Vida::pre_quadro`]: a regeneração da vida, o apagar das três marcas, a regeneração do
//!    escudo (que o **reactiva** se estava a zero), a expiração do escudo, e a marca do escudo;
//! 3. os pedidos do quadro, **pela ordem em que chegam** ([`Vida::golpe`], [`Vida::cura`], …).
//!
//! # O pipeline de um golpe (confirmado pelo oráculo)
//!
//! ```text
//! invencível? → UM sorteio de esquiva → armadura fixa → armadura % → escudo → vida
//! ```
//!
//! ⚠️ Um golpe DENTRO da invencibilidade é **ignorado por inteiro** — nem sorteia. E só um golpe que
//! ENTRA (dano `> 0` na vida ou no escudo) re-arma a invencibilidade.
//!
//! # ⛔ As divergências DECLARADAS, e porque o produto as quer
//!
//! O oráculo mediu quatro comportamentos que a pesquisa lista como **queixas** (§2.2 do 27). A lei
//! reproduz o alvo ao bit sob [`Regras::GDEVELOP`] — é assim que a bancada prova que o resto é a
//! MESMA lei — e o produto corre [`Regras::CASA`]:
//!
//! | o alvo | a casa | a queixa |
//! |---|---|---|
//! | a vida **desce a negativo** (`−30`) | pára em `0` | *vida fora dos limites* (Roblox) |
//! | uma **cura ressuscita** um morto (`−30 + 50 = 20`) e um golpe num morto continua a tirar | morto é FINAL até um [`Vida::reviver`] explícito | uma poção apanhada não pode desfazer uma morte |
//! | `Heal(−20)` **tira** vida; `Hit(−10)` grava um dano de `−10` | um pedido negativo ou não-finito **não faz nada** | *dois caminhos de escrita*, um que respeita o limite e outro que não |
//! | ⭐ **com a sobre-cura ligada, a cura aplica o valor da cura ANTERIOR** (pediu-se `50`, a vida subiu `30`) | cura o que se pede | um **defeito do alvo** que o oráculo apanhou e a documentação não diz |
//!
//! ⚠️ A última é a razão de este porte existir como porte e não como leitura: os eventos da
//! extensão só atribuem a quantidade da cura nos ramos *«sem máximo»* e *«com máximo e SEM
//! sobre-cura»*; no terceiro ramo a variável fica com o valor de antes. **Nenhuma** das 18 leituras
//! a olho o apanharia — a fixtura `c_cura_overheal` passo 5 lê `100 → 130` com `Heal(50)`.

/// Os NÚMEROS de uma vida — o que o artista escreve no painel (e que o alvo deixa mudar por acção).
///
/// ⚠️ Os nomes seguem a pergunta e não o alvo; a correspondência com as propriedades da extensão
/// está no doc de cada campo, porque é ela que a bancada usa para ler as fixturas.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Config {
    /// `MaxHealth` — `0` quer dizer **sem máximo** (o alvo não limita nada).
    pub maximo: f64,
    /// `DamageCooldown` (s) — a invencibilidade depois de um golpe que ENTRA. `0` desliga.
    pub invencivel_s: f64,
    /// `AllowOverHealing` — uma cura pode passar do máximo.
    pub sobre_cura: bool,
    /// `ChanceToDodge` em `0..1` — esquiva se o sorteio `< chance`.
    pub esquiva: f64,
    /// `HealthRegenRate` (pontos/s).
    pub regen: f64,
    /// `HealthRegenDelay` (s) — quanto tempo depois do último golpe a regeneração arranca.
    pub regen_atraso_s: f64,
    /// `MaxShieldPoints` — `0` quer dizer **sem limite** ao activar (e **sem** regeneração).
    pub escudo_max: f64,
    /// `ShieldDuration` (s) — `≤ 0` quer dizer que o escudo **nunca expira**.
    pub escudo_duracao_s: f64,
    /// `ShieldRegenRate` (pontos/s).
    pub escudo_regen: f64,
    /// `ShieldRegenDelay` (s).
    pub escudo_regen_atraso_s: f64,
    /// `BlockExcessDamage` — o golpe que parte o escudo não passa à vida.
    pub escudo_bloqueia_excesso: bool,
    /// `FlatDamageReduction` — subtraída a cada golpe com armadura.
    pub armadura_fixa: f64,
    /// `PercentDamageReduction` em `0..1` — aplicada DEPOIS da fixa.
    pub armadura_pct: f64,
}

impl Default for Config {
    /// Os valores de fábrica do alvo (lidos no `inicial` de toda fixtura).
    fn default() -> Self {
        Self {
            maximo: 100.0,
            invencivel_s: 0.0,
            sobre_cura: false,
            esquiva: 0.0,
            regen: 0.0,
            regen_atraso_s: 0.0,
            escudo_max: 0.0,
            escudo_duracao_s: 5.0,
            escudo_regen: 0.0,
            escudo_regen_atraso_s: 0.0,
            escudo_bloqueia_excesso: false,
            armadura_fixa: 0.0,
            armadura_pct: 0.0,
        }
    }
}

/// ⛔ As quatro DIVERGÊNCIAS declaradas — ver o cabeçalho. Cada campo liga o comportamento do ALVO.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)] // são quatro perguntas independentes, cada uma com o seu gate
pub struct Regras {
    /// A vida pode descer abaixo de `0`.
    pub vida_negativa: bool,
    /// Um morto ainda leva golpes e uma cura ressuscita-o.
    pub morto_nao_e_final: bool,
    /// Um pedido negativo é aplicado (uma cura negativa TIRA vida).
    pub aceita_negativos: bool,
    /// Com a sobre-cura ligada, a cura aplica a quantidade da cura ANTERIOR (o defeito do alvo).
    pub sobre_cura_usa_a_anterior: bool,
}

impl Regras {
    /// O GDevelop 5.6.282 / extensão `0.4.0`, tal como o oráculo o mediu.
    pub const GDEVELOP: Self = Self {
        vida_negativa: true,
        morto_nao_e_final: true,
        aceita_negativos: true,
        sobre_cura_usa_a_anterior: true,
    };
    /// O produto.
    pub const CASA: Self = Self {
        vida_negativa: false,
        morto_nao_e_final: false,
        aceita_negativos: false,
        sobre_cura_usa_a_anterior: false,
    };
}

/// Um relógio do alvo: **inexistente** até ser reposto, e em MILISSEGUNDOS acumulados.
///
/// ⚠️ A inexistência é load-bearing: *«o escudo está activo?»* compara o relógio da duração, e um
/// relógio que nunca foi reposto responde **falso** à comparação — um escudo com pontos e sem
/// `RenewShieldDuration` está INACTIVO (fixtura `e3`).
#[derive(Copy, Clone, Debug, PartialEq)]
struct Relogio(Option<f64>);

impl Relogio {
    fn anda(&mut self, dt_ms: f64) {
        if let Some(ms) = &mut self.0 {
            *ms += dt_ms;
        }
    }
    /// O que a expressão `ObjectTimerElapsedTime` devolve: `0` se o relógio não existe.
    fn segundos(self) -> f64 {
        self.0.map_or(0.0, |ms| ms / 1000.0)
    }
    /// A condição `CompareObjectTimer(<)`: falsa se o relógio não existe.
    fn menor_que(self, s: f64) -> bool {
        self.0.is_some_and(|ms| ms / 1000.0 < s)
    }
    /// A condição `CompareObjectTimer(>)`: falsa se o relógio não existe.
    fn maior_que(self, s: f64) -> bool {
        self.0.is_some_and(|ms| ms / 1000.0 > s)
    }
}

/// O ESTADO VIVO de uma vida — o que a corrida muda e o anel de checkpoints guarda.
#[derive(Copy, Clone, Debug, PartialEq)]
#[allow(clippy::struct_excessive_bools)] // as quatro marcas de um quadro são o que o alvo expõe
pub struct Vida {
    /// `CurrentHealth`.
    pub pontos: f64,
    /// `CurrentShieldPoints`.
    pub escudo: f64,
    desde_golpe: Relogio,
    escudo_relogio: Relogio,
    /// `HitAtLeastOnce` — a invencibilidade só existe depois do primeiro golpe que entra.
    pub golpeada_alguma_vez: bool,
    /// `IsHealthJustDamaged` — dura o quadro do golpe.
    pub acabou_de_levar_dano: bool,
    /// `IsJustHealed`.
    pub acabou_de_ser_curada: bool,
    /// `IsJustDodged`.
    pub acabou_de_esquivar: bool,
    /// `IsShieldJustDamaged`.
    pub escudo_acabou_de_levar_dano: bool,
    /// `DamageToBeApplied` — o que o último golpe levou à VIDA (`PreviousDamageTaken`).
    pub dano_anterior: f64,
    /// `ShieldDamageTaken` — o que o último golpe levou ao escudo.
    pub dano_ao_escudo_anterior: f64,
    /// `HealToBeApplied` — o que a última cura aplicou.
    pub cura_anterior: f64,
    /// O estado do `Once()` da expiração do escudo: `true` enquanto o escudo está inactivo e a
    /// expiração já disparou.
    expiracao_disparada: bool,
}

impl Vida {
    /// A criação (`onCreated`): o relógio do golpe nasce a `0`, o da duração do escudo não existe.
    #[must_use]
    pub fn nasce(inicial: f64, cfg: &Config) -> Self {
        let mut v = Self {
            pontos: 0.0,
            escudo: 0.0,
            desde_golpe: Relogio(Some(0.0)),
            escudo_relogio: Relogio(None),
            golpeada_alguma_vez: false,
            acabou_de_levar_dano: false,
            acabou_de_ser_curada: false,
            acabou_de_esquivar: false,
            escudo_acabou_de_levar_dano: false,
            dano_anterior: 0.0,
            dano_ao_escudo_anterior: 0.0,
            cura_anterior: 0.0,
            expiracao_disparada: false,
        };
        v.define(cfg, Regras::GDEVELOP, inicial);
        v
    }

    /// Os relógios andam — o início do quadro, antes do [`Self::pre_quadro`].
    pub fn anda(&mut self, dt_ms: f64) {
        self.desde_golpe.anda(dt_ms);
        self.escudo_relogio.anda(dt_ms);
    }

    /// `doStepPreEvents`, na ordem do alvo. `dt_s` é o `TimeDelta()` do quadro.
    pub fn pre_quadro(&mut self, cfg: &Config, regras: Regras, dt_s: f64) {
        // A vida regenera (e nunca passa do máximo, nem com sobre-cura).
        let viva = regras.morto_nao_e_final || !self.morta();
        if viva
            && cfg.regen != 0.0
            && self.pontos < cfg.maximo
            && self.desde_golpe.maior_que(cfg.regen_atraso_s)
        {
            self.pontos += cfg.regen * dt_s;
            if self.pontos > cfg.maximo {
                self.pontos = cfg.maximo;
            }
        }
        self.acabou_de_levar_dano = false;
        self.acabou_de_ser_curada = false;
        self.acabou_de_esquivar = false;
        // O escudo regenera — e a ZERO, regenerar é REACTIVÁ-LO com duração nova (o ciclo do `e4`).
        if cfg.escudo_regen != 0.0
            && self.escudo < cfg.escudo_max
            && self.desde_golpe.maior_que(cfg.escudo_regen_atraso_s)
        {
            if self.escudo == 0.0 {
                self.renova_escudo();
            }
            self.escudo += cfg.escudo_regen * dt_s;
            if self.escudo > cfg.escudo_max {
                self.escudo = cfg.escudo_max;
            }
        }
        // A expiração: UMA vez, no primeiro quadro em que o escudo deixa de estar activo.
        if self.escudo_activo(cfg) {
            self.expiracao_disparada = false;
        } else {
            if !self.expiracao_disparada {
                self.escudo = 0.0;
            }
            self.expiracao_disparada = true;
        }
        self.escudo_acabou_de_levar_dano = false;
    }

    /// `IsDead` — `pontos ≤ 0`.
    #[must_use]
    pub fn morta(&self) -> bool {
        self.pontos <= 0.0
    }

    /// `IsDamageCooldownActive`.
    #[must_use]
    pub fn invencivel(&self, cfg: &Config) -> bool {
        self.golpeada_alguma_vez
            && cfg.invencivel_s > 0.0
            && self.desde_golpe.menor_que(cfg.invencivel_s)
    }

    /// `DamageCooldownRemaining`.
    #[must_use]
    pub fn invencivel_resta_s(&self, cfg: &Config) -> f64 {
        if self.invencivel(cfg) {
            (cfg.invencivel_s - self.desde_golpe.segundos()).max(0.0)
        } else {
            0.0
        }
    }

    /// `TimeSinceLastHit` (s).
    #[must_use]
    pub fn desde_golpe_s(&self) -> f64 {
        self.desde_golpe.segundos()
    }

    /// `IsShieldActive` — pontos `> 0` **e** (duração `≤ 0` **ou** relógio `<` duração).
    #[must_use]
    pub fn escudo_activo(&self, cfg: &Config) -> bool {
        self.escudo > 0.0
            && (cfg.escudo_duracao_s <= 0.0 || self.escudo_relogio.menor_que(cfg.escudo_duracao_s))
    }

    /// `ShieldTimeRemaining` — ⚠️ lê a duração inteira com o escudo por activar (o relógio que
    /// não existe vale `0`), que é a leitura enganadora que o oráculo registou.
    #[must_use]
    pub fn escudo_resta_s(&self, cfg: &Config) -> f64 {
        if cfg.escudo_duracao_s > 0.0 {
            (cfg.escudo_duracao_s - self.escudo_relogio.segundos()).max(0.0)
        } else {
            0.0
        }
    }

    /// `ShieldDuration` do relógio, em s (`None` se nunca foi reposto) — para a bancada.
    #[must_use]
    pub fn escudo_relogio_s(&self) -> Option<f64> {
        self.escudo_relogio.0.map(|ms| ms / 1000.0)
    }

    /// `TriggerDamageCooldown` — arma a invencibilidade sem dano.
    pub fn arma_invencibilidade(&mut self) {
        self.golpeada_alguma_vez = true;
        self.desde_golpe = Relogio(Some(0.0));
    }

    /// `RenewShieldDuration`.
    pub fn renova_escudo(&mut self) {
        self.escudo_relogio = Relogio(Some(0.0));
    }

    /// `ActivateShield` — ⚠️ SUBSTITUI os pontos (não soma), limitados pelo máximo se houver.
    pub fn activa_escudo(&mut self, cfg: &Config, pontos: f64, renova: bool) {
        self.escudo = pontos;
        if cfg.escudo_max > 0.0 {
            self.escudo = pontos.min(cfg.escudo_max);
        }
        if renova {
            self.renova_escudo();
        }
    }

    /// `SetHealth` — a ÚNICA porta de escrita da vida: limita em cima pelo máximo (se houver) e,
    /// na casa, em baixo por `0`.
    pub fn define(&mut self, cfg: &Config, regras: Regras, valor: f64) {
        let mut v = valor;
        if cfg.maximo > 0.0 {
            v = v.min(cfg.maximo);
        }
        if !regras.vida_negativa {
            v = v.max(0.0);
        }
        self.pontos = v;
    }

    /// `SetMaxHealthOp(=)` — o máximo muda e a vida é cortada a ele (mesmo com sobre-cura).
    pub fn muda_maximo(&mut self, cfg: &mut Config, maximo: f64) {
        cfg.maximo = maximo;
        if self.pontos > cfg.maximo {
            self.pontos = cfg.maximo;
        }
    }

    /// `Heal`.
    pub fn cura(&mut self, cfg: &Config, regras: Regras, pedido: f64) {
        if !regras.aceita_negativos && (!pedido.is_finite() || pedido < 0.0) {
            return;
        }
        if !regras.morto_nao_e_final && self.morta() {
            return;
        }
        if cfg.maximo == 0.0 {
            self.cura_anterior = pedido;
        } else if !cfg.sobre_cura {
            self.cura_anterior = pedido.min(cfg.maximo - self.pontos);
        } else if !regras.sobre_cura_usa_a_anterior {
            self.cura_anterior = pedido;
        }
        self.pontos += self.cura_anterior;
        self.acabou_de_ser_curada = true;
    }

    /// `Hit(dano, usa_escudo, usa_armadura)` — o pipeline do cabeçalho. `sorteio` é chamado
    /// **exactamente uma vez** por golpe que passa a invencibilidade (mesmo com chance `0`).
    pub fn golpe(
        &mut self,
        cfg: &Config,
        regras: Regras,
        dano: f64,
        usa_escudo: bool,
        usa_armadura: bool,
        sorteio: &mut impl FnMut() -> f64,
    ) {
        if !regras.aceita_negativos && (!dano.is_finite() || dano < 0.0) {
            return;
        }
        if !regras.morto_nao_e_final && self.morta() {
            return;
        }
        if self.invencivel(cfg) {
            return;
        }
        let mut d = dano;
        if sorteio() < cfg.esquiva {
            self.acabou_de_esquivar = true;
            d = 0.0;
        }
        if usa_armadura && d > 0.0 {
            d = (d - cfg.armadura_fixa).max(0.0);
            if cfg.armadura_pct > 0.0 && d > 0.0 {
                d *= 1.0 - cfg.armadura_pct.min(1.0);
            }
        }
        if usa_escudo && self.escudo_activo(cfg) && d > 0.0 {
            self.escudo_acabou_de_levar_dano = true;
            self.arma_invencibilidade();
            if d <= self.escudo {
                self.escudo -= d;
                self.dano_ao_escudo_anterior = d;
                d = 0.0;
            }
            if d > self.escudo {
                self.dano_ao_escudo_anterior = self.escudo;
                if cfg.escudo_bloqueia_excesso {
                    d = 0.0;
                } else {
                    d -= self.escudo;
                }
                self.escudo = 0.0;
            }
        }
        self.dano_anterior = d;
        if d > 0.0 {
            self.acabou_de_levar_dano = true;
            self.arma_invencibilidade();
            let novo = self.pontos - d;
            self.define(cfg, regras, novo);
        }
    }

    /// ⭐ **Reviver** — a porta EXPLÍCITA que tira uma vida da morte na casa (no alvo uma cura
    /// fazia-o por acidente). Não arma nada e não acende nenhuma marca: é um recomeço.
    pub fn reviver(&mut self, cfg: &Config, regras: Regras, pontos: f64) {
        if !pontos.is_finite() || pontos <= 0.0 {
            return;
        }
        self.define(cfg, regras, pontos);
    }
}

pub mod impacto;
pub use impacto::{Pausa, pisca_visivel};

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
