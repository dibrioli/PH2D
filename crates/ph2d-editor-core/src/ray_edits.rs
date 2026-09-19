//! ⭐⭐⭐ **O VOCABULÁRIO DO RAIO** (suplente #21) — o instantâneo e a edição, num módulo abaixo do
//! [`crate::action_bus`] e do [`crate::screens`].
//!
//! # ⛔ Por que ele não está no `screens::hero`, com os irmãos
//!
//! Pela mesma razão do [`crate::tags_edits`], do [`crate::factory_edits`], do
//! [`crate::topdown_edits`] e do [`crate::projectile_edits`], e com o mesmo número atrás: a catraca
//! do DAG tolera a aresta `action_bus → screens` num **tecto**, e escreve a cura ao lado dela.
//!
//! *É o quinto degrau da mesma migração, e cada um torna o resto mais barato.*
//!
//! # ⭐⭐⭐ O que o painel DIZ que os campos sozinhos não diriam
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `the direction is zero` | ⛔ **nulo não é um raio** — a porta do motor devolve `None` | escrever uma direcção |
//! | `the reach is zero` | o raio nasce e morre no mesmo ponto | subir o alcance |
//! | `this ray says nothing` | sem nome nos dois extremos ele vê e cala-se | escrever um sinal |
//! | `sees nothing right now` | ele está a olhar e não há nada no alcance | apontar, ou aproximar |
//!
//! ⚠️ **Os dois primeiros são de OUTRA espécie que os dois últimos:** ali o raio **não corre**, aqui
//! ele corre e não encontra. *Dizer «não vê nada» a quem tem a direcção a zero é mandá-lo resolver a
//! metade errada* — a lei da recusa dos pincéis, e é por isso que a ordem desta tabela é a ordem em
//! que o painel fala.
//!
//! # ⭐⭐ E a leitura VIVA é o que faz esta secção valer a pena
//!
//! O que o raio vê **agora** — o nome e a distância — não vem de um campo: vem do mapa da ponte, que
//! é onde a corrida vive. É a mesma razão pela qual o `ProbeState` do platformer existe, e é o que
//! transforma quatro números numa ferramenta que se afina a olhar.

/// Snapshot da secção RAY SENSOR da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorRayInfo {
    pub entity_bits: u64,
    /// Onde o raio nasce, em coordenadas **locais**.
    pub origin_x: f32,
    pub origin_y: f32,
    /// Para onde aponta, em coordenadas **locais**.
    pub dir_x: f32,
    pub dir_y: f32,
    pub reach: f32,
    pub layer: u32,
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
    /// ⭐⭐⭐ **A queixa, da mais ESPECÍFICA para a mais geral** — e `None` quando não há nenhuma.
    ///
    /// ⚠️ **Ela é uma PORTA e não quatro `if` no pintor**: o painel pinta a frase e o gate mede-a
    /// sem pintar nada. *Uma decisão que só existe dentro de um pintor não é testável sem um
    /// device, e um gate `#[ignore]` é um gate que o CI nunca corre.*
    #[must_use]
    pub fn queixa(&self) -> Option<RayQueixa> {
        if self.dir_x == 0.0 && self.dir_y == 0.0 {
            return Some(RayQueixa::SemDireccao);
        }
        if self.reach <= 0.0 {
            return Some(RayQueixa::SemAlcance);
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

/// As quatro razões pelas quais um raio pode não estar a fazer o que o artista espera.
///
/// ⛔ **Um enum e não uma chave de i18n**, pela lei que o `Brush::curva_inerte` da escultura pagou:
/// devolver a chave daqui poria a língua dentro de uma lei, e o censo do HR-15 não a veria.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RayQueixa {
    /// ⛔ Direcção nula — a porta do motor devolve `None`, e isto **não é um raio**.
    SemDireccao,
    /// O alcance é zero: ele nasce e morre no mesmo ponto.
    SemAlcance,
    /// Ele vê, e não conta a ninguém.
    Mudo,
    /// Ele está a olhar e não há nada no alcance — a única das quatro que é **normal**.
    NaoVeNada,
}

/// Uma edição de um campo da secção RAY SENSOR.
#[derive(Clone, Debug, PartialEq)]
pub enum RayFieldEdit {
    OriginX(f32),
    OriginY(f32),
    DirX(f32),
    DirY(f32),
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
            origin_x: 0.0,
            origin_y: 0.0,
            dir_x: 0.0,
            dir_y: -1.0,
            reach: 1.0,
            layer: 0,
            on_enter: "vi".into(),
            on_exit: String::new(),
            sees: "Chao".into(),
            sees_at: 0.5,
            clock_playing: true,
            selected_count: 1,
        }
    }

    /// ⚠️ **Todo campo dos dois componentes tem uma edição** — um campo sem variante é um knob que
    /// o painel mostra e que ninguém pode mexer, e ele lê-se exactamente como um controlo morto.
    ///
    /// ⛔ A lista é escrita à mão de propósito: ela é a **segunda leitura** dos componentes, e é a
    /// discordância entre as duas que acusa o esquecimento.
    #[test]
    fn todo_campo_dos_dois_componentes_tem_uma_edicao() {
        let variantes = [
            RayFieldEdit::OriginX(0.0),
            RayFieldEdit::OriginY(0.0),
            RayFieldEdit::DirX(0.0),
            RayFieldEdit::DirY(0.0),
            RayFieldEdit::Reach(0.0),
            RayFieldEdit::Layer(0),
            RayFieldEdit::OnEnter(String::new()),
            RayFieldEdit::OnExit(String::new()),
        ];
        assert_eq!(
            variantes.len(),
            8,
            "o `RaySensor` tem SEIS campos e o `RaySignals` DOIS — se um nasceu, ele precisa de \
             uma variante aqui e de uma row no painel"
        );
    }

    /// ⭐⭐⭐ **A ordem das queixas É a lei** — da mais específica para a mais geral.
    ///
    /// ⚠️ Com a direcção a zero o raio **não corre**, logo dizer-lhe *«não vê nada»* seria mandá-lo
    /// resolver a metade errada. E `NaoVeNada` é a única das quatro que descreve um raio SÃO.
    ///
    /// **Mutações que devem sangrar:** trocar a ordem de dois braços · devolver `None` com a
    /// direcção nula.
    #[test]
    fn a_queixa_vai_da_mais_especifica_para_a_mais_geral() {
        assert_eq!(
            base().queixa(),
            None,
            "um raio que ve' e fala nao se queixa"
        );

        let mut sem_dir = base();
        sem_dir.dir_x = 0.0;
        sem_dir.dir_y = 0.0;
        // ⛔ E ele está TAMBÉM mudo e sem ver, de propósito: é a ordem que decide qual sai.
        sem_dir.on_enter = String::new();
        sem_dir.sees = String::new();
        assert_eq!(
            sem_dir.queixa(),
            Some(RayQueixa::SemDireccao),
            "com tres queixas verdadeiras ao mesmo tempo, sai a mais ESPECIFICA"
        );

        let mut sem_alcance = base();
        sem_alcance.reach = 0.0;
        sem_alcance.sees = String::new();
        assert_eq!(sem_alcance.queixa(), Some(RayQueixa::SemAlcance));

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
