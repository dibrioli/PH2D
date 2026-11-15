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
    /// Espelha com o lado do personagem dono.
    pub turns_with_facing: bool,
    /// «Ignore my body».
    pub ignore_body: bool,
    /// «Ignore my other parts» — os outros corpos da família do dono.
    pub ignore_parts: bool,
    /// «Ignore up to»: o NOME do topo da família guardado (`""` = automático).
    pub ignore_up_to: String,
    /// O topo guardado já não existe ou não está ACIMA do raio (a caixa mostra o erro).
    pub ignore_up_to_off: bool,
    /// Ninguém se chama `ignore_up_to` — o texto que o artista ESCREVEU, mantido pela sessão; o documento não mudou.
    pub ignore_up_to_unknown: bool,
    /// O nome do topo AUTOMÁTICO (o personagem, senão o corpo mais alto) — o fantasma da caixa.
    pub ignore_up_to_auto: String,
    /// O conta-gotas do «Ignore up to» está armado para ESTE raio.
    pub up_to_pick_armed: bool,
    /// «Only see» — o modo do filtro de tags.
    pub only_see: RaySee,
    /// A tag do filtro (`TagId.0`; `0` = por escolher).
    pub see_tag: u64,
    /// A `see_tag` ainda existe na árvore do projecto (`true` com `0`).
    pub see_tag_exists: bool,
    /// «Walls block» — a primeira matéria tapa quem não passa no filtro.
    pub walls_block: bool,
    /// «Hit triggers».
    pub hit_triggers: bool,
    /// «Starts inside».
    pub starts_inside: bool,
    /// «See all hits».
    pub all_hits: bool,
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
    /// ⭐ Com `all_hits`: TODOS os que ele vê, por distância (nome, metros) — vazio sem a opção.
    pub sees_all: Vec<(String, f32)>,
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
        // A tag vem antes da camada: com a tag apagada ninguém passa, e a culpa não é da camada (C2).
        if self.only_see != RaySee::Everything {
            if self.see_tag == 0 {
                return Some(RayQueixa::EscolhaATag);
            }
            if !self.see_tag_exists {
                return Some(RayQueixa::TagApagada);
            }
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
    /// «Only see»/«Except» sem tag escolhida: o filtro não filtra ninguém.
    EscolhaATag,
    /// A tag do filtro foi apagada do projecto.
    TagApagada,
    /// Nenhum objeto com forma numa camada que esta vê: ele nunca verá nada.
    CamadaVazia,
    /// Ele vê, e não conta a ninguém.
    Mudo,
    /// Ele está a olhar e não há nada no alcance — a única que é **normal**.
    NaoVeNada,
}

/// O modo do «Only see» (o espelho do `SeeFilter` da física, sem a dependência).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum RaySee {
    /// «Everything».
    #[default]
    Everything,
    /// «Only tag».
    OnlyTag,
    /// «All except tag».
    ExceptTag,
}

/// Uma edição de um campo da secção RAY SENSOR.
#[derive(Clone, Debug, PartialEq)]
pub enum RayFieldEdit {
    /// Graus locais.
    Angle(f32),
    Reach(f32),
    TurnsWithFacing(bool),
    Layer(u32),
    IgnoreBody(bool),
    IgnoreParts(bool),
    OnlySee(RaySee),
    /// O `TagId.0` escolhido (`0` = nenhum).
    SeeTag(u64),
    /// «Ignore up to»: o NOME do topo (`""` = automático), resolvido pela cadeia do raio.
    IgnoreUpTo(String),
    /// Arma (`true`) ou desarma o conta-gotas do «Ignore up to» — sessão, sem desfazer.
    PickUpTo(bool),
    WallsBlock(bool),
    HitTriggers(bool),
    StartsInside(bool),
    AllHits(bool),
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
            turns_with_facing: false,
            ignore_body: true,
            ignore_parts: true,
            ignore_up_to: String::new(),
            ignore_up_to_off: false,
            ignore_up_to_unknown: false,
            ignore_up_to_auto: String::new(),
            up_to_pick_armed: false,
            only_see: RaySee::Everything,
            see_tag: 0,
            see_tag_exists: true,
            walls_block: true,
            hit_triggers: false,
            starts_inside: false,
            all_hits: false,
            layer_empty: false,
            on_enter: "vi".into(),
            on_exit: String::new(),
            sees: "Chao".into(),
            sees_at: 0.5,
            sees_all: Vec::new(),
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
            RayFieldEdit::TurnsWithFacing(false),
            RayFieldEdit::Layer(0),
            RayFieldEdit::IgnoreBody(true),
            RayFieldEdit::IgnoreParts(true),
            RayFieldEdit::OnlySee(RaySee::Everything),
            RayFieldEdit::SeeTag(0),
            RayFieldEdit::IgnoreUpTo(String::new()),
            RayFieldEdit::PickUpTo(false),
            RayFieldEdit::WallsBlock(true),
            RayFieldEdit::HitTriggers(false),
            RayFieldEdit::StartsInside(false),
            RayFieldEdit::AllHits(false),
            RayFieldEdit::OnEnter(String::new()),
            RayFieldEdit::OnExit(String::new()),
        ];
        assert_eq!(
            variantes.len(),
            16,
            "o `RaySensor` tem TREZE campos e o `RaySignals` DOIS (mais o conta-gotas) — se um \
             nasceu, ele precisa de uma variante aqui e de uma row no painel"
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

        // C2: a tag por escolher e a tag apagada vêm ANTES da camada (com ela apagada ninguém passa).
        let mut sem_tag = base();
        sem_tag.only_see = RaySee::OnlyTag;
        sem_tag.layer_empty = true;
        assert_eq!(sem_tag.queixa(), Some(RayQueixa::EscolhaATag));
        let mut apagada = sem_tag.clone();
        apagada.see_tag = 7;
        apagada.see_tag_exists = false;
        assert_eq!(apagada.queixa(), Some(RayQueixa::TagApagada));
        let mut tudo = apagada.clone();
        tudo.only_see = RaySee::Everything;
        tudo.layer_empty = false;
        assert_eq!(tudo.queixa(), None, "«Sees everything» não olha para a tag");

        // C8: o topo fora da cadeia queixa-se NA CAIXA (dono 15/11), nunca no topo do cartão.
        let mut fora = base();
        fora.ignore_up_to = "Truck".into();
        fora.ignore_up_to_off = true;
        fora.ignore_up_to_unknown = true;
        assert_eq!(fora.queixa(), None);
    }
}
