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
    /// Tamanho 0: o ponto exato ainda não é desenhado (o caminho analítico é da W5).
    PontoSemTamanho,
    /// A lâmpada é menor que meio texel da grade a esta distância da câmera.
    PequenaDemaisParaOZoom,
    /// A lâmpada está fora da área que a luz calcula (longe demais da vista).
    ForaDaVista,
    /// `com_quad` de `total` sprites que emitem ou bloqueiam entram pelo QUAD (malha, ladrilho ou
    /// textura cozida), não pelo alfa de cada texel.
    SpritesPeloQuad { com_quad: usize, total: usize },
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
