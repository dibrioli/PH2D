//! ⭐⭐⭐ **A ARMA DO JOGADOR** — o `WeaponFire` do [levantamento §6](../../../docs/Components/00_levantamento_componentes.md),
//! e o item que o `#14 ProjectileMotion` deixou **ABERTO por escrito**.
//!
//! Ela responde a **três** perguntas que a composição de hoje não responde, e a **nenhuma** que ela
//! já responda — o plano
//! [22](../../../docs/Components/22_plano_weapon_fire.md) tem a tabela da medição:
//!
//! | pergunta | a porta | o que a composição dava |
//! |---|---|---|
//! | **quando posso disparar outra vez** | [`WeaponFire::cooldown_ms`] | `60` tiros por segundo, ou o 1.º atrasado um período |
//! | **quantas balas tenho** | um [`crate::Counter`] NOMEADO | `10` balas de um pente de `6`, com o contador a `−4` |
//! | **como se recarrega** | [`WeaponFire::reload_ms`] / [`WeaponFire::reload_on`] | inexprimível — o único verbo que escreve um contador SOMA |
//!
//! ⛔ **E a MIRA não está aqui, porque já estava paga:** o `aim_from_spawner` da [`crate::Factory`]
//! nasceu com o gatilho (#18), e a sonda mede-a a passar **ao bit** (`1,5707964` rad do corpo para
//! `Birth::aim`). *A quarta peça do item era uma ausência que já tinha sido preenchida.*
//!
//! # ⚠️ Porque o `cooldown` NÃO é um [`crate::Timer`]
//!
//! A [`crate::Factory`] **recusou** ter um `rate` próprio, e a recusa dela continua de pé — ela
//! responde *«quem dá o ritmo a quem nasce sozinho»*. Esta pergunta é outra: *«quanto tempo depois
//! de eu CARREGAR»*. Um relógio é uma **emissão periódica** e dispara no **FIM** do período; uma
//! cadência é um **piso no intervalo entre pedidos honrados** e o primeiro é **imediato**.
//!
//! Medido pela composição (o desvio que um artista escreveria — `Timer{repeat}` arrancado no
//! `Press` e parado no `Release`): a 1.ª bala sai `0,250 s` depois de carregar, e saem `3` num
//! segundo onde se pediam `4`. *Carregar e não acontecer nada durante um quarto de segundo lê-se
//! como um botão que não funciona.*
//!
//! # ⚠️ O PENTE é um contador, e não um campo daqui
//!
//! O [`crate::LabelSource::Counter`] do HUD lê a soma dos contadores com um nome. Com a munição a
//! viver num [`crate::Counter`], o placar mostra-a **sem uma linha nova**, o Inspector já a edita,
//! *«apanhei um pacote de balas»* é o `Add to Counter` que já existe, e o rebobinar já sabe o que
//! fazer com ela. ⛔ E ela **não** é duplicada no [`WeaponRuntime`]: *dois sítios com o número de
//! balas divergem no dia em que um deles ganhar uma cerca*.
//!
//! # ⚠️ Esta lei não toca no mundo
//!
//! Ela recebe a munição que a ponte leu e devolve [`Tiro`], como o [`crate::tick_factories`]
//! devolve `Birth`es: quem escreve o contador e quem publica os sinais é a ponte.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

use crate::SimComponent;

/// A maior cadência e a maior recarga autoráveis, em milissegundos (**um minuto**).
///
/// ⚠️ **De que recurso ele é: o PAINEL.** O acumulador é `u64` de microssegundos e só satura depois
/// de ~584 mil anos — o teto **não** é de representação. É a lei que o [`crate::TIMER_MAX_US`] já
/// escreve: *uma caixa que aceita um valor que ninguém consegue esperar produz estado inalcançável*
/// — e aqui o intervalo é mais curto que o de um relógio de cena por natureza, porque quem espera
/// por ele é a mão de quem joga.
pub const WEAPON_MAX_MS: u64 = 60_000;

/// **A ARMA** — CONFIG, como o [`crate::Timer`] e a [`crate::Factory`]. O que é vivo mora no
/// [`WeaponRuntime`].
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponFire {
    /// O sinal que puxa o gatilho. **Vazio = nunca dispara** — a lei da casa, e o painel di-lo.
    ///
    /// ⭐ Ele vem normalmente do [`crate::SignalOnAction`] (#18), mas a arma não sabe disso: uma
    /// armadilha, um relógio ou outra arma servem igual.
    pub on_signal: String,
    /// O intervalo MÍNIMO entre dois tiros, em milissegundos. **`0` = sem cadência** (todo pedido
    /// dispara), e é o neutro observável.
    ///
    /// ⚠️ **O primeiro tiro é IMEDIATO.** A cadência conta do ÚLTIMO tiro, e no princípio não há
    /// nenhum — é exactamente isto que a separa de um relógio.
    pub cooldown_ms: u64,
    /// O nome do [`crate::Counter`] que É o pente. **Vazio = munição infinita**, e então esta arma
    /// nunca escreve num contador.
    ///
    /// ⚠️ **O contador vive na MESMA entidade** (`requires` no catálogo): a leitura soma todos os
    /// do nome e a escrita precisa de um dono.
    pub ammo_counter: String,
    /// Quanto demora a recarregar, em milissegundos. **`0` = não recarrega** — o pente acaba e a
    /// arma fica seca.
    pub reload_ms: u64,
    /// Um sinal que manda recarregar ANTES de esvaziar. Vazio = só a automática.
    pub reload_on: String,
    /// ⭐⭐⭐ **O nome do [`crate::Counter`] que é o DEPÓSITO.** **Vazio = reserva INFINITA**, que é o
    /// comportamento de sempre e o de toda cena já gravada.
    ///
    /// ⚠️⚠️ **Ele NÃO vive nesta entidade, ao contrário do pente** — e não por gosto: uma entidade
    /// tem **um** `Counter`, e o pente já o ocupa. ⇒ o depósito é um contador NOMEADO em qualquer
    /// sítio da cena, e é isso que o põe no HUD, no Inspector e no `Add to Counter` de graça.
    ///
    /// ⛔ **Um nome que DOIS objectos carregam é recusado** (ver [`crate::counter::dono_unico`]):
    /// somar dez depósitos é exacto, *tirar cinco a dez não é*, e escolher um por ordem de
    /// varredura faria a bala sair de um sítio que o artista não escolheu.
    pub reserve_counter: String,
    /// O sinal publicado quando ela DISPARA — **o fio para a [`crate::Factory`]**. Vazio = calada.
    ///
    /// ⛔ **A arma não tem `master`**, e a ausência é a lei: instanciar já tem um motor, e um
    /// segundo seria a segunda resposta a *«como nasce uma cópia»*.
    pub on_fire: String,
    /// O clique seco — publicado quando o gatilho encontra o pente vazio. Vazio = calada.
    pub on_empty: String,
    /// Publicado quando o pente fica cheio. Vazio = calada.
    pub on_reloaded: String,
}

impl SimComponent for WeaponFire {}

/// **O ESTADO VIVO da arma** — o que o motor escreve, e o undo não fotografa.
///
/// ⚠️ **`Default` é «pronta a disparar»**, e é o estado certo para uma arma acabada de nascer: o
/// primeiro tiro é imediato.
///
/// ⛔ **A munição NÃO está aqui** — ela é o [`crate::Counter`], pela razão do cabeçalho.
///
/// ⛔ **Ele NÃO é componente registado** — como o `FactoryRuntime` e o `TimerRuntime`: o que o
/// motor escreve não entra no ficheiro nem no `Ctrl+Z`.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WeaponRuntime {
    /// Quanto falta da cadência, em microssegundos.
    pub cooldown_left_us: u64,
    /// Quanto falta da recarga, em microssegundos.
    pub reload_left_us: u64,
    /// ⭐ **Está a recarregar AGORA** — o facto que `reload_left_us == 0` não sabe dizer.
    ///
    /// É a lei que o [`crate::TimerState`] já pagou (*«um one-shot terminado e um que nunca começou
    /// eram o mesmo estado, bit a bit»*): sem este campo, o tique em que a recarga acaba e o estado
    /// «nunca recarreguei» leem-se iguais.
    pub reloading: bool,
}

/// **O que a ponte leu do pente antes de chamar a lei.**
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Municao {
    /// Quantas balas há agora (o `CounterRuntime`).
    pub tem: i64,
    /// O pente CHEIO (o `Counter::start`) — é para aqui que a recarga repõe.
    pub cheio: i64,
    /// Existe um contador com aquele nome? ⚠️ **`false` é munição INFINITA**, e é o que um
    /// `ammo_counter` vazio produz; ⛔ não é «zero balas».
    pub existe: bool,
    /// ⭐ Quantas balas há no DEPÓSITO. Só faz sentido com [`Self::reserva_existe`].
    pub reserva: i64,
    /// Existe um depósito com dono ÚNICO? ⚠️ **`false` é reserva INFINITA** — o de sempre —, e é o
    /// que um `reserve_counter` vazio **e também** um nome ambíguo produzem. ⛔ Não é «zero
    /// balas», e é o painel que separa os dois silêncios.
    pub reserva_existe: bool,
}

/// **O que um tique da arma produziu.**
///
/// ⚠️ **Factos, nunca ordens:** quem escreve o contador e publica os sinais é a ponte, e é isso que
/// faz esta lei ser testável sem um mundo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tiro {
    /// Ela disparou neste tique.
    pub disparou: bool,
    /// O gatilho encontrou o pente VAZIO — o clique seco.
    pub seca: bool,
    /// Uma recarga começou neste tique.
    pub comecou_a_recarregar: bool,
    /// Uma recarga ACABOU neste tique: o pente está cheio.
    pub recarregou: bool,
    /// Quantas balas o contador deve ter DEPOIS deste tique.
    ///
    /// ⚠️ Só faz sentido com [`Municao::existe`]; com munição infinita a ponte não escreve nada.
    pub municao: i64,
    /// ⭐ Quantas balas o DEPÓSITO deve ter depois deste tique.
    ///
    /// ⚠️ Só faz sentido com [`Municao::reserva_existe`]. ⛔ E ele desce **exactamente** o que o
    /// pente subiu: *a transferência conserva*, e há gate.
    pub reserva: i64,
}

/// Converte milissegundos autorados em microssegundos, com o teto do painel.
#[must_use]
fn us(ms: u64) -> u64 {
    ms.min(WEAPON_MAX_MS).saturating_mul(1_000)
}

/// ⭐⭐⭐ **A LEI.** Um tique da arma.
///
/// # A ORDEM dentro do tique, que É a lei
///
/// 1. o relógio anda — a cadência e a recarga descem;
/// 2. a recarga que **acaba** enche o pente e fala;
/// 3. um pedido de recarga **explícito** arranca uma, se houver o que encher;
/// 4. o gatilho dispara se **não** estiver a recarregar, **e** a cadência estiver a zero, **e**
///    houver munição.
///
/// ⚠️ **O gatilho que encontra o pente vazio arranca a recarga automática** (com `reload_ms > 0`),
/// e publica o clique seco no mesmo tique. ⭐ Quem quiser recarregar **no instante** em que a
/// última bala sai já o exprime sem uma linha nova: uma [`crate::CounterWatch`] com `AtMost 0`
/// sobre o pente, a falar para o `reload_on`.
///
/// ⚠️ **Recarregar durante uma recarga não a reinicia** — senão carregar na tecla em pânico faria a
/// arma nunca ficar pronta.
#[must_use]
pub fn avanca(
    cfg: &WeaponFire,
    st: &mut WeaponRuntime,
    mun: Municao,
    dt_us: u64,
    pediu_tiro: bool,
    pediu_recarga: bool,
) -> Tiro {
    let mut out = Tiro {
        municao: mun.tem,
        reserva: mun.reserva,
        ..Tiro::default()
    };
    // ⭐⭐⭐ **A lei INTEIRA da reserva é esta função**, e ela é o MÍNIMO entre o que falta e o que
    // há — a conta que nenhum verbo da tabela de acções sabe fazer (medido antes da 1.ª linha:
    // o `AddToCounter` soma um delta FIXO, e um `-5` com `2` no depósito deixava-o negativo).
    //
    // ⚠️ **Com reserva infinita ela devolve o que falta, e a saída fica byte-idêntica à de antes
    // desta wave** — que é a razão de o campo novo nascer vazio.
    let tirar = |falta: i64, ha: i64| {
        if mun.reserva_existe {
            falta.min(ha.max(0))
        } else {
            falta
        }
    };
    // ⚠️ **«Há de onde tirar?» é uma pergunta SÓ do depósito** — com ele infinito a resposta é
    // sempre sim, e ⛔ ela **não** olha o pente: quem decide se falta é o `precisa`, abaixo.
    let ha_de_onde = !mun.reserva_existe || out.reserva > 0;

    // (1) o relógio anda.
    st.cooldown_left_us = st.cooldown_left_us.saturating_sub(dt_us);
    if st.reloading {
        st.reload_left_us = st.reload_left_us.saturating_sub(dt_us);
        // (2) a recarga que acaba enche o pente.
        if st.reload_left_us == 0 {
            st.reloading = false;
            out.recarregou = true;
            // ⚠️ **PARCIAL, e é essa a feature:** com `2` no depósito e `5` de pente o artista
            // recebe `2` e o depósito fica a `0`. ⛔ Repor ao `cheio` aqui era o depósito infinito.
            let leva = tirar(mun.cheio - out.municao, out.reserva);
            out.municao += leva;
            out.reserva -= if mun.reserva_existe { leva } else { 0 };
        }
    }

    // (3) o pedido explícito. ⚠️ `out.municao` e não `mun.tem`: uma recarga que acabou AGORA já
    // encheu, e pedir outra em cima dela seria um no-op ruidoso.
    let precisa = mun.existe && out.municao < mun.cheio;
    if pediu_recarga && !st.reloading && cfg.reload_ms > 0 && precisa && ha_de_onde {
        st.reloading = true;
        st.reload_left_us = us(cfg.reload_ms);
        out.comecou_a_recarregar = true;
    }

    // (4) o gatilho.
    if pediu_tiro && !st.reloading && st.cooldown_left_us == 0 {
        if !mun.existe || out.municao > 0 {
            out.disparou = true;
            st.cooldown_left_us = us(cfg.cooldown_ms);
            if mun.existe {
                out.municao -= 1;
            }
        } else {
            out.seca = true;
            // ⚠️ **Com o depósito vazio a recarga automática NÃO arranca** — senão a arma ficava
            // presa num prazo que não dá bala nenhuma, e o artista via *«ela recarrega e continua
            // seca»*. O clique seco continua a soar, que é o report certo: *clique, clique,
            // clique* **é** ficar sem munição. O painel diz qual dos dois silêncios é.
            if cfg.reload_ms > 0 && mun.cheio > 0 && ha_de_onde {
                st.reloading = true;
                st.reload_left_us = us(cfg.reload_ms);
                out.comecou_a_recarregar = true;
            }
        }
    }

    out
}

/// **Nascer** — a entrada desta família no `rewind_runtime`: *rebobinar é RENASCER*.
///
/// ⚠️ **Não é `Default::default()` por acaso, é por LEI**: a arma pronta é a que tem a cadência a
/// zero e nenhuma recarga a meio. O pente não é reposto aqui — ele é um [`crate::Counter`], e o
/// `CounterRuntime` já tem a entrada dele naquela porta.
#[must_use]
pub const fn born() -> WeaponRuntime {
    WeaponRuntime {
        cooldown_left_us: 0,
        reload_left_us: 0,
        reloading: false,
    }
}

#[cfg(test)]
#[path = "weapon_tests.rs"]
mod tests;
