//! **O vocabulário da secção MOTION INPUT** (ciclo 19 do Motion) — o objecto que publica uma
//! entrada para o grafo (o `value.input` com o mesmo nome a lê).
//!
//! O painel não vê a `ph2d-ecs` (ADR-0029): a fonte viaja como o ÍNDICE de `MotionSource::ALL`, e a
//! tradução é da `ph2d-app-components`, com gate de ida-e-volta.

/// Quantas fontes há — o comprimento de `MotionSource::ALL` (gate na `motion_input_inspector`).
pub const MOTION_INPUT_SOURCES: usize = 10;

/// Índice da fonte `Value` (o número do campo).
pub const SOURCE_VALUE: u8 = 0;
/// Índice da fonte `Color` (a cor do campo).
pub const SOURCE_COLOR: u8 = 1;
/// Índice da fonte `Health` (a vida do objecto).
pub const SOURCE_HEALTH: u8 = 2;
/// Índice da fonte `Damage` (um disparo quando a vida desce).
pub const SOURCE_DAMAGE: u8 = 3;
/// Índice da fonte `Speed` (o módulo da velocidade do corpo do objecto From).
pub const SOURCE_SPEED: u8 = 4;
/// Índice da fonte `Grounded` (`1` no chão, `0` no ar).
pub const SOURCE_GROUNDED: u8 = 5;
/// Índice da fonte `Action` (a força `0..1` de uma acção do Input Map).
pub const SOURCE_ACTION: u8 = 6;
/// Índice da fonte `Property` (um campo numérico de um componente do objecto From).
pub const SOURCE_PROPERTY: u8 = 7;
/// Índice da fonte `Velocity` (o par da velocidade do corpo do objecto From).
pub const SOURCE_VELOCITY: u8 = 8;
/// Índice da fonte `Move` (o par das acções `move_*` do Input Map).
pub const SOURCE_MOVE: u8 = 9;

/// Uma edição da secção.
#[derive(Clone, Debug, PartialEq)]
pub enum MotionInputFieldEdit {
    /// O nome da entrada (o do `value.input`).
    Name(String),
    /// A fonte, pelo índice de `MotionSource::ALL`.
    Source(u8),
    /// O número publicado (fonte `Value`).
    Value(f32),
    /// A cor publicada (fonte `Color`), RGBA linear.
    Color([f32; 4]),
    /// O objecto From pelo NOME (`""` = este); um nome que ninguém tem não escreve.
    From(String),
    /// O nome da acção do Input Map (fonte `Action`).
    Action(String),
    /// O campo lido, `"<canonical>/<serde_key>"` (fonte `Property`).
    Property(String),
    /// A caixa «Remap».
    Remap(bool),
    /// O número que vira `0`.
    RemapFrom(f32),
    /// O número que vira `1`.
    RemapTo(f32),
    /// A curva depois do remap, texto de `ph2d-curve` (`""` = linear).
    Curve(String),
    /// A meia-vida da suavização, em segundos (`0` = desligada).
    Smooth(f32),
    /// Arma (`true`) ou desarma o conta-gotas do From — estado da SESSÃO, sem passo de desfazer.
    PickFrom(bool),
}

/// ⭐ **As predefinições da Curve** do grupo «Shape»: `(chave do rótulo, texto ph2d-curve)`.
///
/// Só segmentos `L`/`S` (HR-5): o Ease In é a primeira metade de um `smoothstep` em `[0, 2]`, o Ease Out a
/// segunda de um em `[-1, 1]` — exactos, com a tangente plana na ponta lenta. A prova vive na
/// `ph2d-app-components` (que lê a `ph2d-curve`).
pub const CURVE_PRESETS: [(&str, &str); 4] = [
    ("panel.inspector.motion_input.curve_linear", ""),
    (
        "panel.inspector.motion_input.curve_ease_in",
        "c1 0:0:S 2:2:L",
    ),
    (
        "panel.inspector.motion_input.curve_ease_out",
        "c1 -1:-1:S 1:1:L",
    ),
    (
        "panel.inspector.motion_input.curve_smooth",
        "c1 0:0:S 1:1:L",
    ),
];

/// O que a secção diz antes dos campos — a causa de *«o grafo não lê nada»*.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MotionInputQueixa {
    /// O nome está vazio: nenhum `value.input` o pode ler.
    NomeVazio,
    /// O grafo do Motion não tem nenhum nó Input.
    GrafoSemEntradas,
    /// Há nós Input, mas nenhum com este nome.
    NomeNaoLido,
    /// A fonte é a vida (ou o dano) e o objecto não tem `Health`.
    SemVida,
    /// O From nomeia um objecto que não existe.
    SemAlvo,
    /// A fonte é Move e o Input Map não tem nenhuma das acções `move_*`.
    SemMover,
    /// A fonte é Speed (sem corpo) ou Grounded (sem personagem de plataforma).
    SemCorpo,
    /// A fonte é Action e o Input Map não tem acção com esse nome.
    AcaoDesconhecida,
    /// A fonte é Property e nenhum campo foi escolhido.
    CampoVazio,
    /// A fonte é Property e o objecto From não tem o campo escolhido.
    CampoAusente,
}

/// **O que o objecto manda AGORA** — a linha viva do cartão (MI-11).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MotionInputAoVivo {
    /// Um número (a vida lida, ou o valor publicado).
    Numero(f32),
    /// Uma cor, RGBA linear.
    Cor([f32; 4]),
    /// Quantas vezes disparou neste jogo (fonte `Damage`).
    Disparos(u64),
    /// Um par `(x, y)` (Velocity, Move, Property de duas faixas).
    Par([f32; 2]),
}

/// **Quem escreve este nome ANTES deste objecto** — o grafo lê a primeira linha, e o cartão diz
/// quem a tapa (senão o artista mexe num valor que ninguém lê).
#[derive(Clone, Debug, PartialEq)]
pub enum MotionInputOutro {
    /// Um script Luau (`motion.set`).
    Script,
    /// A linha de comando (`PH2D_MOTION_INPUTS`).
    Linha,
    /// Outro objecto, pelo nome.
    Objeto(String),
}

/// **O que o painel mostra** — o instantâneo que a shell publica por quadro.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorMotionInputInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// O nome da entrada.
    pub name: String,
    /// A fonte, pelo índice de `MotionSource::ALL`.
    pub source: u8,
    /// O número (fonte `Value`).
    pub value: f32,
    /// A cor (fonte `Color`).
    pub color: [f32; 4],
    /// Os nomes que o grafo do Motion LÊ (um por `value.input`, sem repetir, em ordem).
    pub nomes_do_grafo: Vec<String>,
    /// O objecto From tem `Health`? (As fontes `Health` e `Damage` leem-na.)
    pub tem_vida: bool,
    /// O objecto From pelo NOME (`""` = o pai, MI-10).
    pub from: String,
    /// O nome do PAI (o dono da entrada, que o From vazio lê); `""` sem nome.
    pub pai: String,
    /// Os bits do objecto que a fonte lê (onde «Add Health» e «Add Physics Body» anexam).
    pub alvo_bits: Option<u64>,
    /// O From acha um objecto (`""` acha sempre: é este).
    pub alvo_achado: bool,
    /// O From tem o que a fonte Speed (corpo) ou Grounded (personagem de plataforma) lê.
    pub tem_corpo: bool,
    /// O nome da acção (fonte `Action`).
    pub action: String,
    /// Os nomes das acções do Input Map do jogo, na ordem dele.
    pub acoes_do_mapa: Vec<String>,
    /// O campo lido, `"<canonical>/<serde_key>"` (fonte `Property`).
    pub property: String,
    /// Os campos numéricos dos componentes do From: `(rótulo «Health · Max», "canonical/serde_key")`.
    pub campos_do_alvo: Vec<(String, String)>,
    /// As chaves dos campos que mandam um PAR `(x, y)` (sufixo na lista).
    pub campos_par: Vec<String>,
    /// A caixa «Remap» e a faixa que vira `0..1`.
    pub remap: bool,
    pub remap_from: f32,
    pub remap_to: f32,
    /// A curva depois do remap (`""` = linear).
    pub curve: String,
    /// A meia-vida da suavização, em segundos (`0` = desligada).
    pub smooth: f32,
    /// O conta-gotas do From está armado para ESTE objecto.
    pub from_pick_armed: bool,
    /// Quantos objectos estão escolhidos (a edição aplica-se ao activo).
    pub selected_count: usize,
    /// O jogo está a correr (as leituras do que chega ao grafo só existem em Play).
    pub a_correr: bool,
    /// O que o objecto manda agora (a vida lida também com o jogo parado).
    pub ao_vivo: Option<MotionInputAoVivo>,
    /// Em Play: quem escreve este nome primeiro, e o que escreve — `None` quando é este objecto.
    pub tapado_por: Option<(MotionInputOutro, MotionInputAoVivo)>,
}

impl InspectorMotionInputInfo {
    /// A fonte lê o campo `value`? (Senão a linha some — controlo morto.)
    #[must_use]
    pub fn le_valor(&self) -> bool {
        self.source == SOURCE_VALUE
    }

    /// A fonte lê o campo `color`?
    #[must_use]
    pub fn le_cor(&self) -> bool {
        self.source == SOURCE_COLOR
    }

    /// A primeira causa de o grafo não ler o que este objecto publica, por ordem de cura.
    #[must_use]
    pub fn queixa(&self) -> Option<MotionInputQueixa> {
        let nome = self.name.trim();
        if nome.is_empty() {
            Some(MotionInputQueixa::NomeVazio)
        } else if let Some(q) = self.queixa_da_fonte() {
            Some(q)
        } else if self.nomes_do_grafo.is_empty() {
            Some(MotionInputQueixa::GrafoSemEntradas)
        } else if !self.nomes_do_grafo.iter().any(|n| n == nome) {
            Some(MotionInputQueixa::NomeNaoLido)
        } else {
            None
        }
    }

    /// A queixa da FONTE (o objecto From e o que ela lê dele), por ordem de cura.
    fn queixa_da_fonte(&self) -> Option<MotionInputQueixa> {
        use MotionInputQueixa as Q;
        if self.le_from() && !self.alvo_achado {
            return Some(Q::SemAlvo);
        }
        match self.source {
            SOURCE_HEALTH | SOURCE_DAMAGE if !self.tem_vida => Some(Q::SemVida),
            SOURCE_SPEED | SOURCE_GROUNDED | SOURCE_VELOCITY if !self.tem_corpo => {
                Some(Q::SemCorpo)
            }
            SOURCE_MOVE if self.acoes_de_mover().is_empty() => Some(Q::SemMover),
            SOURCE_ACTION if !self.acoes_do_mapa.iter().any(|a| *a == self.action) => {
                Some(Q::AcaoDesconhecida)
            }
            SOURCE_PROPERTY if self.property.trim().is_empty() => Some(Q::CampoVazio),
            SOURCE_PROPERTY if !self.campos_do_alvo.iter().any(|(_, k)| *k == self.property) => {
                Some(Q::CampoAusente)
            }
            _ => None,
        }
    }

    /// As acções `move_*` que o Input Map tem (o que a fonte Move lê), na ordem direita, esquerda, cima, baixo.
    #[must_use]
    pub fn acoes_de_mover(&self) -> Vec<&'static str> {
        use ph2d_input::{PLAYER_MOVE_DOWN, PLAYER_MOVE_LEFT, PLAYER_MOVE_RIGHT, PLAYER_MOVE_UP};
        [PLAYER_MOVE_RIGHT, PLAYER_MOVE_LEFT, PLAYER_MOVE_UP, PLAYER_MOVE_DOWN]
            .into_iter()
            .filter(|n| self.acoes_do_mapa.iter().any(|a| a == n))
            .collect()
    }

    /// A fonte lê um OBJECTO (o From)? Value, Color e Action não leem ninguém.
    #[must_use]
    pub fn le_from(&self) -> bool {
        matches!(
            self.source,
            SOURCE_HEALTH
                | SOURCE_DAMAGE
                | SOURCE_SPEED
                | SOURCE_GROUNDED
                | SOURCE_PROPERTY
                | SOURCE_VELOCITY
        )
    }

    /// A fonte publica um NÚMERO (o Remap e o Smooth só valem aí).
    #[must_use]
    pub fn le_numero(&self) -> bool {
        !matches!(self.source, SOURCE_COLOR | SOURCE_DAMAGE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(name: &str, source: u8, nomes: &[&str], tem_vida: bool) -> InspectorMotionInputInfo {
        InspectorMotionInputInfo {
            entity_bits: 1,
            name: name.into(),
            source,
            value: 0.0,
            color: [1.0; 4],
            nomes_do_grafo: nomes.iter().map(|s| (*s).to_string()).collect(),
            tem_vida,
            from: String::new(),
            pai: String::new(),
            alvo_bits: None,
            alvo_achado: true,
            tem_corpo: false,
            action: String::new(),
            acoes_do_mapa: vec!["jump".into()],
            property: String::new(),
            campos_do_alvo: vec![("Health · Max".into(), "Health/max".into())],
            campos_par: Vec::new(),
            remap: false,
            remap_from: 0.0,
            remap_to: 1.0,
            curve: String::new(),
            smooth: 0.0,
            from_pick_armed: false,
            selected_count: 1,
            a_correr: false,
            ao_vivo: None,
            tapado_por: None,
        }
    }

    /// Cada queixa aparece só com a causa dela, e um objecto bem ligado não se queixa.
    #[test]
    fn a_queixa_nomeia_a_causa() {
        use MotionInputQueixa as Q;
        assert_eq!(info("hp", SOURCE_VALUE, &["hp"], false).queixa(), None);
        assert_eq!(
            info("  ", SOURCE_VALUE, &["hp"], false).queixa(),
            Some(Q::NomeVazio)
        );
        assert_eq!(
            info("hp", SOURCE_HEALTH, &["hp"], false).queixa(),
            Some(Q::SemVida)
        );
        assert_eq!(info("hp", SOURCE_DAMAGE, &["hp"], true).queixa(), None);
        assert_eq!(
            info("hp", SOURCE_VALUE, &[], false).queixa(),
            Some(Q::GrafoSemEntradas)
        );
        assert_eq!(
            info("hq", SOURCE_VALUE, &["hp"], false).queixa(),
            Some(Q::NomeNaoLido)
        );
    }

    /// As queixas das fontes novas nomeiam a causa, e a do From vem primeiro.
    #[test]
    fn as_queixas_das_fontes_novas_nomeiam_a_causa() {
        use MotionInputQueixa as Q;
        let mut i = info("hp", SOURCE_SPEED, &["hp"], true);
        assert_eq!(i.queixa(), Some(Q::SemCorpo));
        i.tem_corpo = true;
        assert_eq!(i.queixa(), None);
        i.alvo_achado = false;
        assert_eq!(i.queixa(), Some(Q::SemAlvo));
        let mut a = info("hp", SOURCE_ACTION, &["hp"], false);
        a.alvo_achado = false; // a acção não lê o From
        assert_eq!(a.queixa(), Some(Q::AcaoDesconhecida));
        a.action = "jump".into();
        assert_eq!(a.queixa(), None);
        let mut p = info("hp", SOURCE_PROPERTY, &["hp"], false);
        assert_eq!(p.queixa(), Some(Q::CampoVazio));
        p.property = "Health/current".into();
        assert_eq!(p.queixa(), Some(Q::CampoAusente));
        p.property = "Health/max".into();
        assert_eq!(p.queixa(), None);
        assert!(p.le_numero() && !info("a", SOURCE_COLOR, &[], false).le_numero());
        let mut v = info("hp", SOURCE_VELOCITY, &["hp"], false);
        assert!(v.le_from() && v.queixa() == Some(Q::SemCorpo));
        v.tem_corpo = true;
        assert_eq!(v.queixa(), None);
        let mut m = info("hp", SOURCE_MOVE, &["hp"], false);
        assert!(!m.le_from() && m.queixa() == Some(Q::SemMover));
        m.acoes_do_mapa = vec!["move_left".into()];
        assert_eq!(m.queixa(), None);
    }

    /// Só a fonte que lê um campo o mostra.
    #[test]
    fn a_fonte_decide_que_campo_vive() {
        let v = info("a", SOURCE_VALUE, &[], false);
        assert!(v.le_valor() && !v.le_cor());
        let c = info("a", SOURCE_COLOR, &[], false);
        assert!(!c.le_valor() && c.le_cor());
        let h = info("a", SOURCE_HEALTH, &[], false);
        assert!(!h.le_valor() && !h.le_cor());
    }
}
