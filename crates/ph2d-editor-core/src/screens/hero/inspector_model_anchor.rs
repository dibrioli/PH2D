//! **O modelo da §12 Sockets / Named Anchors** ([ADR-0072]) — snapshot e edits.
//!
//! ⚠️ **Irmão de [`super::inspector_model`] por CAP de LOC** — mesmo padrão dos outros quatro.
//!
//! # Este é o primeiro snapshot do Inspector que NÃO é `Copy`
//!
//! Uma âncora tem **nome**, e um nome é uma `String`. Todos os outros snapshots são `Copy` e
//! viajam num `Cell`; este viaja num `RefCell`, como o da §7 Ordering. Não é acidente de
//! implementação: *o nome é a coisa que distingue um socket de outro*, e trocá-lo por um índice
//! seria o anti-padrão nº 4 da spec §7.14 («anchor 5» ≠ «muzzle»).
//!
//! # A seleção é do PAINEL, não do snapshot
//!
//! O snapshot traz a **lista inteira**; qual linha está aberta é estado local do painel
//! (`InspectorState`). ⚠️ Publicar a seleção pelo snapshot obrigaria a shell a saber de um facto
//! que só a UI tem, e faria toda mudança de linha atravessar o barramento e voltar — um quadro de
//! atraso para abrir uma ficha.
//!
//! [ADR-0072]: ../../../../docs/architecture/decisions/0072-named-anchor-unification.md

/// Uma âncora, como o Inspector a lê.
///
/// ⚠️ **`pos` está em pixels da FONTE** e `rot_deg` em graus — as duas unidades que o artista
/// escreve. O componente guarda metros e radianos; a conversão vive na shell, num sítio só.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorAnchorRow {
    pub name: String,
    pub pos: [f32; 2],
    pub rot_deg: f32,
    /// `[x, y, w, h]` em pixels da fonte. `Some` ⇒ pelo menos um Slice.
    pub bounds: Option<[f32; 4]>,
    /// `[x, y, w, h]` dentro de `bounds`.
    pub center: Option<[f32; 4]>,
}

impl InspectorAnchorRow {
    /// O que esta âncora É — **derivado**, como no motor. `0` Socket · `1` Slice · `2` Region.
    ///
    /// ⚠️ Espelha `ph2d_ecs::NamedAnchor::kind()` porque o `editor-core` não depende do motor;
    /// o gate da shell prende os dois.
    pub fn kind_tag(&self) -> u8 {
        match (self.bounds.is_some(), self.center.is_some()) {
            (true, true) => 2,
            (true, false) => 1,
            (false, _) => 0,
        }
    }

    /// A palavra que a linha mostra ao lado do nome.
    pub fn kind_label(&self) -> &'static str {
        match self.kind_tag() {
            2 => "Region",
            1 => "Slice",
            _ => "Socket",
        }
    }
}

/// Snapshot da §12 da entidade selecionada.
#[derive(Clone, Debug, PartialEq)]
pub struct InspectorAnchorInfo {
    pub entity_bits: u64,
    /// A lista inteira, na ordem em que o componente a guarda.
    pub rows: Vec<InspectorAnchorRow>,
    /// O componente está anexado? ⚠️ Distinto de `rows.is_empty()`: um componente anexado e
    /// vazio é um estado que o artista criou, e some se o confundirmos com a ausência.
    pub present: bool,
    pub selected_count: usize,
    /// A seleção múltipla tem listas divergentes.
    pub mixed: bool,
}

/// Uma edição da §12.
///
/// ⚠️ **Todas carregam o ÍNDICE da âncora**, e os campos vetoriais carregam também o do eixo —
/// a mesma lei do `PerCornerTintAt` e do `RegionX/Y/W/H`. Mandar a lista inteira faria um
/// fan-out de seleção múltipla atropelar as âncoras divergentes de todas as outras sprites.
#[derive(Clone, Debug, PartialEq)]
pub enum AnchorFieldEdit {
    /// Cria uma âncora com o próximo nome livre (`anchor_N`).
    Add,
    /// Retira a âncora deste índice.
    Remove(u8),
    /// Renomeia. ⚠️ Um nome inválido ou repetido é **recusado com aviso**, nunca em silêncio.
    Rename(u8, String),
    /// `(âncora, eixo 0..1, valor em pixels da fonte)`.
    Pos(u8, u8, f32),
    /// `(âncora, graus)`.
    Rot(u8, f32),
    /// Liga/desliga a área. Desligar leva o miolo consigo.
    BoundsOn(u8, bool),
    /// `(âncora, campo 0..3 de [x,y,w,h], valor)`.
    Bounds(u8, u8, f32),
    /// Liga/desliga o miolo. Sem área, não faz nada.
    CenterOn(u8, bool),
    /// `(âncora, campo 0..3, valor)`.
    Center(u8, u8, f32),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(bounds: Option<[f32; 4]>, center: Option<[f32; 4]>) -> InspectorAnchorRow {
        InspectorAnchorRow {
            name: "a".into(),
            pos: [0.0, 0.0],
            rot_deg: 0.0,
            bounds,
            center,
        }
    }

    /// A tabela do ADR §2.1, do lado do painel.
    #[test]
    fn the_shape_is_the_kind_here_too() {
        assert_eq!(row(None, None).kind_label(), "Socket");
        assert_eq!(row(Some([0.0; 4]), None).kind_label(), "Slice");
        assert_eq!(row(Some([0.0; 4]), Some([0.0; 4])).kind_label(), "Region");
        // ⚠️ Miolo sem área lê-se como Socket — o mesmo que o motor faz com o estado impossível.
        assert_eq!(row(None, Some([0.0; 4])).kind_label(), "Socket");
    }

    /// Uma lista vazia ANEXADA não é o mesmo que componente ausente.
    #[test]
    fn an_attached_empty_list_is_not_absence() {
        let empty_attached = InspectorAnchorInfo {
            entity_bits: 1,
            rows: Vec::new(),
            present: true,
            selected_count: 1,
            mixed: false,
        };
        let absent = InspectorAnchorInfo {
            present: false,
            ..empty_attached.clone()
        };
        assert_ne!(empty_attached, absent);
    }

    /// As edições indexadas conseguem endereçar cada âncora e cada eixo — senão a última é
    /// inalcançável por gesto nenhum.
    #[test]
    fn the_indexed_edits_address_every_anchor_and_axis() {
        let a = AnchorFieldEdit::Bounds(63, 3, 1.0);
        let b = AnchorFieldEdit::Bounds(63, 2, 1.0);
        assert_ne!(a, b, "o indice do campo tem de distinguir w de h");
        assert_ne!(
            AnchorFieldEdit::Pos(0, 0, 1.0),
            AnchorFieldEdit::Pos(1, 0, 1.0),
            "o indice da ancora tem de distinguir duas ancoras"
        );
    }
}
