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
    /// **Quantos filhos montam nesta âncora** (ADR-0072 §2.6).
    ///
    /// ⚠️ Fecha o laço do outro lado: o objeto que MONTA vê em que âncora anda, mas o dono da
    /// âncora não via ninguém. Sem isto, saber se `hand_r` está em uso obriga a selecionar cada
    /// filho um a um — e apagar uma âncora em uso não avisa de nada.
    pub riders: usize,
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
    /// **As âncoras do PAI** — o que este objeto pode montar (ADR-0072 §2.6).
    ///
    /// ⚠️ Dono diferente do de [`Self::rows`], e é por isso que é um campo à parte em vez de uma
    /// flag: sem pai, ou com um pai sem âncoras, isto é vazio e o seletor **não se pinta**.
    pub parent_anchors: Vec<String>,
    /// Em que âncora do pai este objeto anda hoje. `None` = não monta.
    pub mount: Option<String>,
    /// **O quanto este objeto está DESLOCADO da âncora** que monta, em pixels da fonte.
    ///
    /// ⚠️ `[0, 0]` **exacto** = está em cima dela. A comparação é exacta de propósito e não por
    /// epsilon: o único caminho que escreve zero é o snap, e ele escreve zero *exacto*; qualquer
    /// arrasto deixa um resíduo. Um epsilon esconderia um deslocamento real de meio pixel.
    pub mount_offset: [f32; 2],
    /// A caixa «manter as âncoras visíveis» — do DONO das âncoras, não de quem monta.
    pub vis_in_editor: bool,
    /// A caixa «manter as âncoras visíveis em runtime».
    pub vis_at_runtime: bool,
}

impl InspectorAnchorInfo {
    /// O índice, na lista do pai, da âncora que este objeto monta.
    ///
    /// ⚠️ **Derivado, nunca guardado** — a mesma lei do `kind()` de uma âncora. Um índice
    /// guardado podia discordar do nome depois de o pai reordenar ou apagar a lista, e discordar
    /// em silêncio é o modo de falha que nada denuncia.
    pub fn mount_index(&self) -> Option<usize> {
        let name = self.mount.as_deref()?;
        self.parent_anchors.iter().position(|a| a == name)
    }

    /// **O vínculo aponta para um nome que o pai não tem.** Renomearam a âncora, apagaram-na, ou
    /// o objeto mudou de pai.
    ///
    /// Geometricamente comporta-se como não montar (`ph2d_ecs::MountState::Dangling`); o que isto
    /// existe para permitir é **mostrá-lo** — e, sobretudo, oferecer a linha que o desfaz.
    pub fn mount_dangling(&self) -> bool {
        self.mount.is_some() && self.mount_index().is_none()
    }

    /// O seletor de montagem tem o que oferecer?
    ///
    /// ⛔ `false` ⇒ **não pinte a linha**. Um controlo com uma opção só («—») é a mesma
    /// afordância a mentir que o botão `Simple` do 9-slice era (Enio, 2026-08-22).
    pub fn mount_pick_is_useful(&self) -> bool {
        !self.parent_anchors.is_empty() || self.mount.is_some()
    }

    /// **Este objeto está fora da âncora que monta?**
    ///
    /// ⛔ É a condição EXACTA para pintar o botão «Reset to Anchor». Um botão que não tem nada que
    /// fazer é a terceira ação sem efeito visível desta família — as duas primeiras (o slider
    /// morto e o `× Remove 9-Slice`) foram apanhadas pelo Enio, não por um gate.
    ///
    /// `false` quando não monta, quando o vínculo está pendurado (não há âncora a que voltar), e
    /// quando já está lá.
    pub fn is_off_anchor(&self) -> bool {
        self.mount_index().is_some() && self.mount_offset != [0.0, 0.0]
    }
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
    /// **Montar numa âncora do PAI**, ou em nenhuma (`None`).
    ///
    /// ⚠️ **A única variante que NÃO carrega o índice de uma âncora deste objeto**, porque não
    /// fala de uma: ela fala da relação com o pai. Carrega o NOME e não o índice pela mesma razão
    /// que o componente o faz — apagar a âncora `0` do pai faria toda a gente descer uma casa em
    /// silêncio.
    Mount(Option<String>),
    /// **Voltar a pousar na âncora** — zera o deslocamento local de quem monta.
    ///
    /// ⚠️ Zera a POSIÇÃO e mais nada. A rotação e a escala do filho continuam a ser dele: uma
    /// espada tem um ângulo próprio dentro da mão, e repô-lo seria decidir no lugar do artista.
    SnapToAnchor,
    /// A caixa «manter as âncoras visíveis» (sem seleção) — do DONO das âncoras.
    VisibilityInEditor(bool),
    /// A caixa «manter as âncoras visíveis em runtime».
    VisibilityAtRuntime(bool),
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
            riders: 0,
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
            parent_anchors: Vec::new(),
            mount: None,
            mount_offset: [0.0, 0.0],
            vis_in_editor: false,
            vis_at_runtime: false,
        };
        let absent = InspectorAnchorInfo {
            present: false,
            ..empty_attached.clone()
        };
        assert_ne!(empty_attached, absent);
    }

    /// **A montagem lê-se por DERIVAÇÃO** — o índice e o «pendurado» saem do nome contra a lista
    /// do pai, e nunca de um campo guardado que pudesse discordar dela.
    #[test]
    fn the_mount_index_and_the_dangling_state_are_derived_from_the_parents_list() {
        let base = InspectorAnchorInfo {
            entity_bits: 1,
            rows: Vec::new(),
            present: false,
            selected_count: 1,
            mixed: false,
            parent_anchors: vec!["muzzle".into(), "hand_r".into()],
            mount: None,
            mount_offset: [0.0, 0.0],
            vis_in_editor: false,
            vis_at_runtime: false,
        };
        assert_eq!(base.mount_index(), None);
        assert!(!base.mount_dangling());
        assert!(base.mount_pick_is_useful(), "o pai tem ancoras");

        let bound = InspectorAnchorInfo {
            mount: Some("hand_r".into()),
            ..base.clone()
        };
        assert_eq!(bound.mount_index(), Some(1));
        assert!(!bound.mount_dangling());

        let lost = InspectorAnchorInfo {
            mount: Some("gone".into()),
            ..base.clone()
        };
        assert_eq!(lost.mount_index(), None);
        assert!(lost.mount_dangling(), "o nome nao esta' na lista do pai");
    }

    /// ⛔ Sem pai e sem vínculo **não se pinta o seletor** — um controlo com uma opção só é a
    /// afordância a mentir que o `Simple` do 9-slice era. Mas um vínculo PENDURADO sobre um pai
    /// sem âncoras **tem** de aparecer, senão fica preso.
    #[test]
    fn the_picker_hides_when_it_has_nothing_to_offer_but_never_traps_a_dangling_mount() {
        let nothing = InspectorAnchorInfo {
            entity_bits: 1,
            rows: Vec::new(),
            present: false,
            selected_count: 1,
            mixed: false,
            parent_anchors: Vec::new(),
            mount: None,
            mount_offset: [0.0, 0.0],
            vis_in_editor: false,
            vis_at_runtime: false,
        };
        assert!(!nothing.mount_pick_is_useful());
        let trapped = InspectorAnchorInfo {
            mount: Some("gone".into()),
            ..nothing.clone()
        };
        assert!(
            trapped.mount_pick_is_useful(),
            "sem a linha, o artista nao tem como desfazer um vinculo pendurado"
        );
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
