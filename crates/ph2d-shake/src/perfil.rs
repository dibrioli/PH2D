//! ⭐⭐⭐ **Os PERFIS de um clique** — *Recoil* · *Impact* · *Explosion*.
//!
//! # ⛔ Eles são AÇÚCAR, e o precedente é o `ph2d_tween::Preset`
//!
//! Um perfil **não tem lei nenhuma**: ele escreve os números que o artista escreveria à mão e
//! desaparece. Não há um `CameraShake::perfil`, não há um braço na [`crate::deslocamento`], e um
//! ficheiro gravado depois de um clique é indistinguível de um afinado a dedo.
//!
//! ⚠️ **E é isso que os torna honestos:** depois de carregar em *Explosion* os quatro números ficam
//! à vista e todos se mexem. *Um perfil que guardasse o nome dele seria um MODO, e um modo que o
//! artista não pode desmontar é uma caixa preta.*
//!
//! # ⭐⭐ Ele escreve QUATRO números e NUNCA a semente, e essa ausência é a lei
//!
//! A [`crate::Lei`] tem cinco campos e a semente não é um deles no sentido que interessa aqui: ela
//! é **IDENTIDADE** e não sensação — *duas câmeras com a mesma sensação e sementes diferentes
//! tremem de maneiras diferentes*, que é precisamente o que o doc dela declara. Escrevê-la num
//! perfil faria duas câmeras que receberam o mesmo clique tremer em UNÍSSONO, e o artista leria
//! isso como um defeito do motor. ⇒ [`Numeros`] tem quatro campos, e há gate.
//!
//! # ⚠️ A FREQUÊNCIA é a mesma nos três, e o motivo é de RECURSO
//!
//! O `frequencia: 20` de fábrica declara por escrito de que recurso ele é: **a taxa de amostragem
//! do ecrã** (a `30` já há dois quadros por período; a `60`, um). Esse tecto é uma propriedade do
//! monitor e **não do acontecimento** ⇒ nenhum dos três perfis tem razão para o mudar, e baixá-lo
//! é gosto — que tem um slider ao lado. *Um preset que mexesse num número sem ter argumento para
//! ele estaria a escolher por quem sabe mais do que ele.*
//!
//! # ⚠️⚠️ As duas grandezas que MUDAM são de PRODUTO, e dizê-lo é a única forma honesta (§0.0)
//!
//! Nada na máquina se parte com `amplitude = 1,0` nem com `decaimento = 0,83`. Isto é uma **faixa
//! de produto**, como o [`crate::EXPOENTE_MAX`] ao lado — e, como ele, o que cada degrau COMPRA
//! está medido (`mede_o_que_cada_perfil_compra`):
//!
//! | perfil | pico | % da vista de fábrica (`10,0` m) | dura | quadros a 60 Hz | oscilações |
//! |---|---|---|---|---|---|
//! | *Recoil* | `0,05` m | `0,5 %` | `0,125` s | `7,5` | `2,5` |
//! | *Impact* | `0,25` m | `2,5 %` | `0,50` s | `30` | `10` |
//! | *Explosion* | `1,00` m | `10 %` | `1,25` s | `75` | `25` |
//!
//! ⛔⛔ **A altura da vista é `10,0` m e NÃO os `11,25` que a doc do `CameraShake::default` dizia**
//! — ela é o `GameCamera::default().height_world`, e o número velho não vinha de componente
//! nenhum. *A tabela de fábrica foi corrigida no mesmo commit, e há gate a ler a altura da câmera
//! em vez de um literal* (`a_escada_e_uma_fraccao_da_vista`, na `ph2d-app-components`, que é onde a
//! `ph2d-ecs` é alcançável).
//!
//! ⭐ **A duração é EXACTA e não uma estimativa:** o [`crate::decai`] é linear no trauma, logo um
//! abanão cheio acaba em `1/decaimento` **ao segundo**, e não assintoticamente.
//!
//! ⚠️⚠️ **E os três decaimentos são escolhidos para que ela feche em `f32`** (`d × (1/d) == 1,0`):
//! a 1.ª redacção escreveu `8,3333` para dar `0,12 s`, e o gate da exactidão leu um trauma residual
//! de `5,96e-8` à duração declarada — *a lei é exacta e a aritmética da máquina não, e um perfil
//! que deixasse o abanão a um ULP do fim faria a promessa deste parágrafo mentir*. Com `8,0` e
//! `0,8` a conta fecha ao bit, e o que se paga é `0,005 s` no coice.
//!
//! ⭐⭐ **O `0,125 s` do *Recoil* não é escolhido no ar:** é a mesma cerca de quadros que o `Flash` do
//! `ph2d_tween::Preset` já paga — abaixo de ~4 quadros um acontecimento curto lê-se como artefacto
//! de desenho. Aqui ele vale `2,5` oscilações completas, que é o mínimo que ainda se lê como um
//! TREMOR e não como um solavanco único.
//!
//! # ⭐ A ESCADA é a lei, e é ela que tem gate
//!
//! Os três números de produto podem ser afinados; o que **não** pode mudar é a ordem: um coice é
//! mais pequeno e mais curto que um impacto, e um impacto que uma explosão. *Três números soltos
//! são gosto; três números ordenados são um vocabulário*, e é o vocabulário que o artista aprende.
//!
//! # ⭐⭐ E o *Impact* É o valor de fábrica, ao bit
//!
//! Isso não é decoração: é o que prova que o do meio não foi inventado para encher a escada.
//! Carregar nele numa câmera acabada de nascer é um **no-op byte a byte**, e há gate.

/// Os quatro números que um perfil escreve. ⚠️ **A semente NÃO está aqui** — ver o cabeçalho.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Numeros {
    /// Metros de deslocamento no pico. Ver [`crate::Lei::amplitude`].
    pub amplitude: f32,
    /// Hz. Ver [`crate::Lei::frequencia`].
    pub frequencia: f32,
    /// Trauma por segundo. Ver [`crate::Lei::decaimento`].
    pub decaimento: f32,
    /// A potência do trauma. Ver [`crate::Lei::expoente`].
    pub expoente: u8,
}

/// Ver o cabeçalho do módulo.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum Perfil {
    /// ⭐ O **coice de uma arma**: pequeno e curto, e acaba antes do tiro seguinte.
    Recuo,
    /// O **impacto** — bater numa parede, aterrar de um salto. O valor de FÁBRICA.
    #[default]
    Impacto,
    /// ⭐ A **explosão**: grande, longa, e com o murro todo no princípio.
    Explosao,
}

/// A frequência dos três. ⚠️ Ver o cabeçalho: ela é a mesma porque o tecto é do ECRÃ.
const HZ: f32 = 20.0;

impl Perfil {
    /// Todos, em ordem — **a fonte da iteração**. ⚠️ **APPEND-ONLY**: a posição é a tag, e ela
    /// viaja num clique (o painel manda a posição do chip).
    pub const ALL: [Perfil; 3] = [Perfil::Recuo, Perfil::Impacto, Perfil::Explosao];

    /// O rótulo que o artista lê. ⚠️ **O motor publica a CHAVE e quem pinta é que a resolve** — a
    /// lei da fronteira dos motores (a jornada do HR-15 de 2026-09-20).
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Perfil::Recuo => "shake.perfil.recoil",
            Perfil::Impacto => "shake.perfil.impact",
            Perfil::Explosao => "shake.perfil.explosion",
        }
    }

    /// A posição em [`Self::ALL`].
    #[must_use]
    pub const fn tag(self) -> u8 {
        self as u8
    }

    /// O perfil desta posição, ou o de fábrica.
    #[must_use]
    pub fn from_tag(tag: u8) -> Self {
        Self::ALL.get(tag as usize).copied().unwrap_or_default()
    }

    /// ⭐⭐⭐ **Os quatro números que este perfil escreve** — e nada mais.
    ///
    /// ⚠️ O expoente só sobe na explosão, e a razão é a que a [`crate::EXPOENTE_MAX`] já mediu: ele
    /// governa **que fracção do movimento cabe no primeiro quarto** (`44 %` · `58 %` · `68 %`). Num
    /// coice de `0,12 s` a forma da cauda não é observável — *um knob que não muda nada no regime
    /// em que corre não deve ser mexido por um preset*.
    #[must_use]
    pub const fn numeros(self) -> Numeros {
        match self {
            Perfil::Recuo => Numeros {
                amplitude: 0.05,
                frequencia: HZ,
                decaimento: 8.0,
                expoente: 2,
            },
            Perfil::Impacto => Numeros {
                amplitude: 0.25,
                frequencia: HZ,
                decaimento: 2.0,
                expoente: 2,
            },
            Perfil::Explosao => Numeros {
                amplitude: 1.0,
                frequencia: HZ,
                decaimento: 0.8,
                expoente: 3,
            },
        }
    }

    /// **Quanto tempo um abanão cheio deste perfil dura, em segundos.** ⭐ Exacto: o decaimento é
    /// linear (ver [`crate::decai`]).
    #[must_use]
    pub fn duracao_s(self) -> f32 {
        1.0 / self.numeros().decaimento
    }
}

#[cfg(test)]
#[path = "perfil_tests.rs"]
mod tests;
