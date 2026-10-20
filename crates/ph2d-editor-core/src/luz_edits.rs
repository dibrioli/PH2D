//! **O vocabulário da secção LUZ do Inspector** (Luz 2D, W4 — `docs/Luz2D/01_plano.md` §4).
//!
//! ⚠️ Irmão do [`crate::shake_edits`], pela mesma razão: o painel não conhece a `ph2d-ecs`
//! (ADR-0029) nem a `ph2d-light2d-ecs`, logo os tipos que atravessam a fronteira vivem aqui.

/// Uma edição de um campo da `Luz`.
#[derive(Clone, Debug, PartialEq)]
pub enum LuzFieldEdit {
    /// A cor como o seletor a dá — sRGB de 8 bits. A família converte-a para LINEAR.
    Cor([u8; 3]),
    /// A fluência a 1 m (plano §2.3: `2π` = branco a 1 m).
    Intensidade(f32),
    /// O raio da fonte, em metros — decide a penumbra.
    Tamanho(f32),
    /// O alcance da queda autoral, em metros; `f32::INFINITY` = física pura (sem alcance).
    Alcance(f32),
    /// A posição em [`TIPOS`]: `0` = Ponto, `1` = Ambiente. ⚠️ Viaja como `u8` pela cerca de
    /// dependência; a tradução é da família, com gate de ida-e-volta.
    Tipo(u8),
    /// A `BloqueiaLuz.opacidade` do corpo — a fração bloqueada por entrada (`0..=1`).
    Opacidade(f32),
    /// Retira a `BloqueiaLuz` do corpo (o cartão «Light Occluder»).
    RetirarOclusor,
    /// Retira a `NaoRecebeLuz` do corpo (o cartão «Unlit»).
    RetirarSemLuz,
}

/// ⭐ O que os cartões do CORPO na luz — «Light Occluder» e «Unlit» — precisam para se pintar
/// (componente = ícone + cartão). `None` no painel = o objeto não tem nenhum dos dois.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorCorpoDaLuzInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// A `BloqueiaLuz.opacidade`, se o objeto bloqueia a luz.
    pub oclusor: Option<f32>,
    /// O objeto tem a `NaoRecebeLuz`?
    pub sem_luz: bool,
}

/// As chaves dos chips do TIPO, na ordem da tag.
pub const TIPOS: [&str; 2] = ["panel.inspector.luz.point", "panel.inspector.luz.ambient"];

/// O que a luz do quadro diz sobre ESTA lâmpada (ou a cena) — o painel transforma em frase.
/// ⭐ Recusa que só o terminal vê é botão mudo: cada caso aqui era um `eprintln!` ou nada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AvisoDaLuz {
    /// O solver recusou a grade: a luz da cena INTEIRA está apagada neste quadro.
    GradeGrandeDemais { lado: u32, limite: u32 },
    /// O dispositivo não tem `SHADER_F16` (ou a configuração é inválida): a luz está apagada.
    SolverIndisponivel,
    /// ⭐ W5e: o dispositivo recusou a memória da luz nesta janela, nem com a sombra mais grossa: a
    /// luz está apagada.
    SemMemoria { mb: u64 },
    /// ⭐ W5e: a janela é grande para o dispositivo (o lado de textura ou a memória) — a sombra
    /// direta é desenhada `vezes` mais grossa que a do nível.
    SombraEngrossada { vezes: u32 },
    /// Tamanho 0: o ponto exato ainda não é desenhado (o caminho analítico é da W5).
    PontoSemTamanho,
    /// A lâmpada é menor que meio texel da grade a esta distância da câmera.
    PequenaDemaisParaOZoom,
    /// A lâmpada está fora da área que a luz calcula (longe demais da vista).
    ForaDaVista,
    /// `com_quad` de `total` sprites que emitem ou bloqueiam entram pelo QUAD (malha, ladrilho ou
    /// textura cozida), não pelo alfa de cada texel.
    SpritesPeloQuad { com_quad: usize, total: usize },
    /// ⭐ W6g r9: a cena tem mais cantos vivos do que a placa guarda (o teto dos cantos da borda): em
    /// `blocos` blocos 8×8 os cantos das formas saem arredondados como antes.
    CantosArredondados { blocos: u32 },
}

/// O que a secção **LUZ** precisa de saber para se pintar.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorLuzInfo {
    /// Os bits da entidade — o endereço que toda edição carrega.
    pub entity_bits: u64,
    /// A cor em sRGB de 8 bits (a amostra e o seletor falam sRGB; o componente guarda linear).
    pub cor_srgb: [u8; 3],
    pub intensidade: f32,
    pub tamanho_m: f32,
    /// `f32::INFINITY` = sem alcance.
    pub alcance_m: f32,
    /// A posição em [`TIPOS`].
    pub tipo: u8,
    /// ⭐ A luz que uma superfície BRANCA recebe a 1 m, no canal mais forte (`I·max(cor)/2π`, o
    /// ponto — plano §2.3). O painel di-lo em palavras: «branco a `d` m» (`luz_em(r) = este/r`) ou,
    /// na Ambiente, a fração do forno branco (ela não cai com a distância).
    pub luz_a_1m: f32,
    /// Da mais grave para a mais branda.
    pub avisos: Vec<AvisoDaLuz>,
    /// ⭐ A nitidez em uso (Settings ▸ Light), se é a do «Automatic», e o custo de placa que ele
    /// leu (ms por quadro). `None` = a luz do quadro ainda não recebeu a preferência.
    pub qualidade: Option<(NivelDaLuz, bool, Option<f32>)>,
    /// Quantos objetos estão escolhidos — a secção edita o primário, e di-lo.
    pub selected_count: usize,
}

impl InspectorLuzInfo {
    /// É a luz AMBIENTE? (Tamanho e alcance não se aplicam a ela.)
    #[must_use]
    pub fn ambiente(&self) -> bool {
        self.tipo == 1
    }
}

/// ⭐ **A nitidez da luz no editor** (pedido do dono, 2026-10-09): o tamanho da célula da grade da
/// luz, em px de TELA. ⚠️ Os níveis estão a uma OITAVA um do outro: a grade arredonda a célula para a
/// potência de 2 (em metros) acima (G11 — a luz não treme com o zoom), e com 4 · 3 · 2 px a Média e
/// a Alta caíam na MESMA célula (medido no smoke do dono, 09/10). Por oitava, cada nível tem metade
/// da célula e 4× o custo, em qualquer zoom.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum NivelDaLuz {
    /// Células de 4–8 px.
    Baixa,
    /// Células de 2–4 px.
    Media,
    /// Células de 1–2 px.
    #[default]
    Alta,
}

impl NivelDaLuz {
    /// Do mais barato ao mais nítido.
    pub const ALL: [Self; 3] = [Self::Baixa, Self::Media, Self::Alta];

    /// O lado da célula, em px de tela.
    #[must_use]
    pub fn celula_px(self) -> f32 {
        match self {
            Self::Baixa => 4.0, // LITERAL-PX-OK: o degrau da W3 (K7 a 1080p)
            Self::Media => 2.0, // LITERAL-PX-OK: uma oitava abaixo
            Self::Alta => 1.0,  // LITERAL-PX-OK: duas oitavas abaixo
        }
    }

    /// A chave do nome.
    #[must_use]
    pub fn chave(self) -> &'static str {
        match self {
            Self::Baixa => "chrome.menu.light_low",
            Self::Media => "chrome.menu.light_medium",
            Self::Alta => "chrome.menu.light_high",
        }
    }

    /// O nome da preferência gravada (`~/.ph2d/prefs.txt`).
    #[must_use]
    pub fn token(self) -> &'static str {
        match self {
            Self::Baixa => "low",
            Self::Media => "medium",
            Self::Alta => "high",
        }
    }

    /// A inversa de [`Self::token`].
    #[must_use]
    pub fn de_token(t: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|n| n.token() == t)
    }
}

/// ⭐ **A preferência da luz do editor**: o nível escolhido e o «Automático». Com o automático, o
/// nível escolhido é o TETO e o editor desce quando a placa não aguenta (o governador da família).
/// Preferência do EDITOR (o `prefs.txt`), nunca dado do projeto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreferenciaDaLuz {
    pub nivel: NivelDaLuz,
    pub automatica: bool,
}

impl Default for PreferenciaDaLuz {
    fn default() -> Self {
        Self {
            nivel: NivelDaLuz::Alta,
            automatica: true,
        }
    }
}
