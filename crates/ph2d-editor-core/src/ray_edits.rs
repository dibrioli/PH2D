//! ⭐⭐⭐ **O VOCABULÁRIO DO RAIO** — o instantâneo e a edição do objeto «Ray Sensor» (GEF 07, versão B),
//! num módulo abaixo do [`crate::action_bus`] (a mesma razão do `projectile_edits`: a aresta do DAG).
//!
//! O raio é um OBJETO (dono 15/11): a origem é a posição dele, por isso o cartão não tem Origin.
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `the reach is zero` | o raio nasce e morre no mesmo ponto | subir o alcance |
//! | `nothing in this layer` | nenhum objeto com forma numa camada que esta vê | trocar a camada ou a matriz |
//! | `this ray says nothing` | sem nome nos dois extremos ele vê e cala-se | escrever um sinal |
//! | `sees nothing right now` | ele olha e não há nada no alcance | apontar, ou aproximar |
//!
//! Os dois primeiros dizem que o raio NUNCA verá; o último é normal. A ordem da tabela é a do painel.

/// Snapshot da secção RAY SENSOR da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorRayInfo {
    pub entity_bits: u64,
    /// Para onde aponta, em graus locais (0 = direita, 90 = cima).
    pub angle: f32,
    pub reach: f32,
    pub layer: u32,
    /// Nenhum objeto com forma está numa camada que esta vê (a matriz): o raio nunca verá nada.
    pub layer_empty: bool,
    /// O nome publicado ao passar a ver — vazio = calado.
    pub on_enter: String,
    /// E ao deixar de ver.
    pub on_exit: String,
    /// ⭐ **O que ele vê AGORA** — vazio = nada. Vem do mapa da ponte, nunca de um campo.
    pub sees: String,
    /// A que distância, quando vê.
    pub sees_at: f32,
    /// O relógio está a andar?
    pub clock_playing: bool,
    pub selected_count: usize,
}

impl InspectorRayInfo {
    /// A queixa, da mais ESPECÍFICA para a mais geral — e `None` quando não há nenhuma. Uma porta,
    /// não `if`s no pintor: o gate mede-a sem pintar.
    #[must_use]
    pub fn queixa(&self) -> Option<RayQueixa> {
        if self.reach <= 0.0 {
            return Some(RayQueixa::SemAlcance);
        }
        if self.layer_empty {
            return Some(RayQueixa::CamadaVazia);
        }
        if self.on_enter.trim().is_empty() && self.on_exit.trim().is_empty() {
            return Some(RayQueixa::Mudo);
        }
        if self.sees.is_empty() {
            return Some(RayQueixa::NaoVeNada);
        }
        None
    }
}

/// As razões pelas quais um raio pode não estar a fazer o que o artista espera (enum, nunca chave de
/// i18n: a língua não mora numa lei).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RayQueixa {
    /// O alcance é zero: ele nasce e morre no mesmo ponto.
    SemAlcance,
    /// Nenhum objeto com forma numa camada que esta vê: ele nunca verá nada.
    CamadaVazia,
    /// Ele vê, e não conta a ninguém.
    Mudo,
    /// Ele está a olhar e não há nada no alcance — a única que é **normal**.
    NaoVeNada,
}

/// Uma edição de um campo da secção RAY SENSOR.
#[derive(Clone, Debug, PartialEq)]
pub enum RayFieldEdit {
    /// Graus locais.
    Angle(f32),
    Reach(f32),
    Layer(u32),
    /// ⚠️ Os NOMES, crus — a mesma convenção do alvo do projéctil: quem apara é quem lê.
    OnEnter(String),
    OnExit(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> InspectorRayInfo {
        InspectorRayInfo {
            entity_bits: 1,
            angle: -90.0,
            reach: 1.0,
            layer: 0,
            layer_empty: false,
            on_enter: "vi".into(),
            on_exit: String::new(),
            sees: "Chao".into(),
            sees_at: 0.5,
            clock_playing: true,
            selected_count: 1,
        }
    }

    /// ⚠️ **Todo campo dos dois componentes tem uma edição** — um campo sem variante é um controlo
    /// morto. A lista é a SEGUNDA leitura dos componentes, escrita à mão de propósito.
    #[test]
    fn todo_campo_dos_dois_componentes_tem_uma_edicao() {
        let variantes = [
            RayFieldEdit::Angle(0.0),
            RayFieldEdit::Reach(0.0),
            RayFieldEdit::Layer(0),
            RayFieldEdit::OnEnter(String::new()),
            RayFieldEdit::OnExit(String::new()),
        ];
        assert_eq!(
            variantes.len(),
            5,
            "o `RaySensor` tem TRÊS campos e o `RaySignals` DOIS — se um nasceu, ele precisa de \
             uma variante aqui e de uma row no painel"
        );
    }

    /// ⭐⭐⭐ **A ordem das queixas É a lei** — da mais específica para a mais geral.
    ///
    /// **Mutações que devem sangrar:** trocar a ordem de dois braços · calar a camada vazia.
    #[test]
    fn a_queixa_vai_da_mais_especifica_para_a_mais_geral() {
        assert_eq!(
            base().queixa(),
            None,
            "um raio que ve' e fala nao se queixa"
        );

        let mut sem_alcance = base();
        sem_alcance.reach = 0.0;
        sem_alcance.layer_empty = true;
        sem_alcance.sees = String::new();
        assert_eq!(
            sem_alcance.queixa(),
            Some(RayQueixa::SemAlcance),
            "com tres queixas verdadeiras ao mesmo tempo, sai a mais ESPECIFICA"
        );

        let mut vazia = base();
        vazia.layer_empty = true;
        vazia.on_enter = String::new();
        vazia.sees = String::new();
        assert_eq!(
            vazia.queixa(),
            Some(RayQueixa::CamadaVazia),
            "a camada vazia vem antes do mudo e do cego: ele NUNCA vera'"
        );

        let mut mudo = base();
        mudo.on_enter = "  ".into();
        assert_eq!(
            mudo.queixa(),
            Some(RayQueixa::Mudo),
            "um nome so' de espacos e' um nome vazio — a regra do `SignalOnHit`"
        );

        let mut cego = base();
        cego.sees = String::new();
        assert_eq!(cego.queixa(), Some(RayQueixa::NaoVeNada));
    }
}
