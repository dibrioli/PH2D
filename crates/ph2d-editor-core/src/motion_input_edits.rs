//! **O vocabulário da secção MOTION INPUT** (ciclo 19 do Motion) — o objecto que publica uma
//! entrada para o grafo (o `value.input` com o mesmo nome a lê).
//!
//! O painel não vê a `ph2d-ecs` (ADR-0029): a fonte viaja como o ÍNDICE de `MotionSource::ALL`, e a
//! tradução é da `ph2d-app-components`, com gate de ida-e-volta.

/// Quantas fontes há — o comprimento de `MotionSource::ALL` (gate na `motion_input_inspector`).
pub const MOTION_INPUT_SOURCES: usize = 4;

/// Índice da fonte `Value` (o número do campo).
pub const SOURCE_VALUE: u8 = 0;
/// Índice da fonte `Color` (a cor do campo).
pub const SOURCE_COLOR: u8 = 1;
/// Índice da fonte `Health` (a vida do objecto).
pub const SOURCE_HEALTH: u8 = 2;
/// Índice da fonte `Damage` (um disparo quando a vida desce).
pub const SOURCE_DAMAGE: u8 = 3;

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
}

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
    /// O objecto tem `Health`? (As fontes `Health` e `Damage` leem-na.)
    pub tem_vida: bool,
    /// Quantos objectos estão escolhidos (a edição aplica-se ao activo).
    pub selected_count: usize,
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
        } else if matches!(self.source, SOURCE_HEALTH | SOURCE_DAMAGE) && !self.tem_vida {
            Some(MotionInputQueixa::SemVida)
        } else if self.nomes_do_grafo.is_empty() {
            Some(MotionInputQueixa::GrafoSemEntradas)
        } else if !self.nomes_do_grafo.iter().any(|n| n == nome) {
            Some(MotionInputQueixa::NomeNaoLido)
        } else {
            None
        }
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
            selected_count: 1,
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
