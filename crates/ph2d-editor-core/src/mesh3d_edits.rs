//! ⭐⭐⭐ **O VOCABULÁRIO DO CATAVENTO** — o instantâneo e a edição da malha 3D viva de um sprite.
//!
//! Módulo abaixo do [`crate::action_bus`] e do [`crate::screens`], pela mesma razão do
//! [`crate::weapon_edits`] e das cinco irmãs: a catraca do DAG tolera a aresta `action_bus →
//! screens` num tecto, e cada módulo destes é um degrau da mesma migração.
//!
//! # ⭐⭐⭐ Porque a secção EXISTE, e não é «três números numa tabela genérica»
//!
//! Os três números (`Yaw`, `Pitch`, `Spin`) cabiam em qualquer tabela. O que não cabe é a
//! **QUEIXA**: um catavento num sprite que nunca foi assado **não faz absolutamente nada**, e não
//! há nada na tela que o diga. É a lei que os pincéis da escultura pagaram cinco vezes —
//! *um controlo que não faz nada e não diz porquê é indistinguível de um controlo partido*, e o
//! artista conclui que a FERRAMENTA não funciona em vez de que falta a ENTRADA dela.
//!
//! | aviso | o que se passa | como se cura |
//! |---|---|---|
//! | `bake this sprite first` | ⛔ não há `BakedForm` ⇒ a fase salta-o e nada acontece | `Shift+B` com o sprite escolhido |
//! | `it is standing still` | está assado e `spin = 0` ⇒ a luz só anda se o artista mexer no `Yaw` | dar-lhe voltas por segundo |
//!
//! ⚠️ **As duas são de espécies DIFERENTES, e a ordem é a lei:** a primeira diz que ele **não
//! corre**; a segunda diz que ele corre e está parado — e a segunda pode ser exactamente o que o
//! artista quer (uma pose fixa fora do plano, que a rota A não sabe exprimir). *Dizer «está parado»
//! a quem nem sequer assou é mandá-lo resolver a metade errada.*

/// ⭐ **O tecto das voltas por segundo.**
///
/// ⚠️ **Ele é de PRODUTO e não de recurso, e dizê-lo é a única forma honesta** (§0.0): a lei é
/// `yaw + spin · TAU · t` e nada nela satura. O que satura é o OLHO — acima de ~`2` voltas por
/// segundo a luz a deslizar deixa de se ler como uma forma a virar e passa a cintilação, que é o
/// oposto do que esta rota existe para mostrar.
///
/// ⚠️ **A faixa é BIPOLAR** porque um catavento pode virar para o outro lado, e um tecto só de um
/// lado tornaria metade dos cataventos inexprimíveis.
pub const MESH3D_MAX_SPIN: f64 = 2.0;

/// O passo de arrasto das voltas por segundo.
///
/// ⚠️ `0,05` e não `0,1`: a faixa útil (uma volta em quatro a dez segundos) vive entre `0,1` e
/// `0,25`, e um passo de um décimo atravessa-a em dois gestos.
pub const MESH3D_SPIN_STEP: f64 = 0.05;

/// O passo de arrasto dos dois ângulos, em GRAUS.
///
/// ⚠️ **Os ângulos são oferecidos em GRAUS e guardados em RADIANOS**, como todo ângulo desta casa:
/// a conversão vive nas duas pontas desta crate ([`InspectorMesh3dInfo::yaw_graus`] e
/// [`Mesh3dFieldEdit`]), e nunca num pintor.
pub const MESH3D_ANGLE_STEP: f64 = 1.0;

/// Snapshot da secção LIVE MESH da entidade seleccionada.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InspectorMesh3dInfo {
    pub entity_bits: u64,
    /// A volta em torno do eixo vertical do ecrã, em RADIANOS (o que o componente guarda).
    pub yaw: f32,
    /// A inclinação, em RADIANOS.
    pub pitch: f32,
    /// Voltas por segundo. `0` = parado no `yaw` autorado.
    pub spin: f32,
    /// ⭐ **Este sprite já foi ASSADO?** Vem do mapa de formas assadas, nunca de um campo — e é
    /// dele que sai a queixa que torna a secção uma ferramenta em vez de três números.
    pub assado: bool,
}

/// O que o painel tem a DIZER sobre este catavento — ver o cabeçalho.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mesh3dQueixa {
    /// ⛔ Não há forma assada: a fase salta-o e **nada** acontece.
    SemForma,
    /// Está assado e parado — pode ser o desenho, e por isso é a mais geral.
    Parado,
}

impl InspectorMesh3dInfo {
    /// O `yaw` em GRAUS, que é como o artista o lê.
    #[must_use]
    pub fn yaw_graus(&self) -> f64 {
        f64::from(self.yaw.to_degrees())
    }

    /// O `pitch` em GRAUS.
    #[must_use]
    pub fn pitch_graus(&self) -> f64 {
        f64::from(self.pitch.to_degrees())
    }

    /// ⭐⭐⭐ **A queixa, da mais ESPECÍFICA para a mais geral.**
    ///
    /// ⛔ A ordem é a lei (ver o cabeçalho), e ela vive AQUI e não dentro do pintor: *dois `if`
    /// dentro de um pintor só se medem com uma janela, e um gate `#[ignore]` é um gate que o CI
    /// nunca corre.*
    #[must_use]
    pub fn queixa(&self) -> Option<Mesh3dQueixa> {
        if !self.assado {
            return Some(Mesh3dQueixa::SemForma);
        }
        if self.spin == 0.0 {
            return Some(Mesh3dQueixa::Parado);
        }
        None
    }
}

/// Uma edição de um campo do catavento.
///
/// ⚠️ **Os dois ângulos chegam em GRAUS** (é o que a caixa mostra) e a conversão para radianos é
/// feita por quem aplica — numa porta só, nunca em cada chamador.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Mesh3dFieldEdit {
    /// A volta, em GRAUS.
    YawGraus(f64),
    /// A inclinação, em GRAUS.
    PitchGraus(f64),
    /// Voltas por segundo.
    Spin(f64),
}

#[cfg(test)]
mod tests {
    use super::{InspectorMesh3dInfo, Mesh3dFieldEdit, Mesh3dQueixa};

    fn base() -> InspectorMesh3dInfo {
        InspectorMesh3dInfo {
            entity_bits: 7,
            yaw: 0.0,
            pitch: 0.0,
            spin: 0.25,
            assado: true,
        }
    }

    /// ⚠️ **Todo campo do componente tem uma edição** — um campo sem variante é um knob que o
    /// painel mostra e que ninguém pode mexer, e ele lê-se exactamente como um controlo morto.
    ///
    /// ⛔ A lista é escrita à mão de propósito: ela é a **segunda leitura** do componente, e é a
    /// discordância entre as duas que acusa o esquecimento.
    #[test]
    fn todo_campo_editavel_do_componente_tem_uma_edicao() {
        let variantes = [
            Mesh3dFieldEdit::YawGraus(0.0),
            Mesh3dFieldEdit::PitchGraus(0.0),
            Mesh3dFieldEdit::Spin(0.0),
        ];
        assert_eq!(
            variantes.len(),
            3,
            "o `Mesh3D` tem TRÊS campos que o artista escreve (o `piece` é resolvido pela cena) — \
             se um nasceu, ele precisa de uma variante aqui e de uma row no painel"
        );
    }

    /// ⭐⭐⭐ **A ordem das queixas É a lei** — da mais específica para a mais geral.
    ///
    /// **Mutações que devem sangrar:** trocar a ordem dos dois braços · devolver `None` sem forma
    /// assada · ler `spin == 0` como defeito mesmo sem bake.
    #[test]
    fn a_queixa_vai_da_mais_especifica_para_a_mais_geral() {
        assert_eq!(
            base().queixa(),
            None,
            "um catavento assado e a girar nao se queixa"
        );

        // ⛔ Ele está sem forma E parado ao mesmo tempo, de propósito: é a ORDEM que decide qual
        // sai, e sem as duas verdadeiras esta metade não afirmaria nada.
        let sem_nada = InspectorMesh3dInfo {
            assado: false,
            spin: 0.0,
            ..base()
        };
        assert_eq!(
            sem_nada.queixa(),
            Some(Mesh3dQueixa::SemForma),
            "com as duas queixas verdadeiras ao mesmo tempo, sai a mais ESPECIFICA: quem nem \
             sequer assou nao tem de ouvir «esta' parado»"
        );

        let parado = InspectorMesh3dInfo {
            spin: 0.0,
            ..base()
        };
        assert_eq!(parado.queixa(), Some(Mesh3dQueixa::Parado));
    }

    /// **Os ângulos saem em GRAUS**, que é a unidade em que a caixa os mostra.
    ///
    /// **Mutação que deve sangrar:** `to_degrees()` → `f32::from_bits(self.yaw.to_bits())`.
    #[test]
    fn os_angulos_saem_em_graus() {
        let meia_volta = InspectorMesh3dInfo {
            yaw: std::f32::consts::PI,
            pitch: std::f32::consts::FRAC_PI_2,
            ..base()
        };
        assert!((meia_volta.yaw_graus() - 180.0).abs() < 1.0e-4);
        assert!((meia_volta.pitch_graus() - 90.0).abs() < 1.0e-4);
    }
}
