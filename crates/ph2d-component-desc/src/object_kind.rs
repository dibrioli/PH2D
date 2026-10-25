//! **Que TIPO de objeto é este** e o conjunto deles — filho do `lib.rs` pelo tecto de LOC.

/// **Que TIPO de objeto é este** — e a resposta lê-se por **PRESENÇA de um marcador**, nunca
/// por um campo.
///
/// Medido em 2026-08-24 sobre os 107 registados: `Sprite` ⇒ [`ObjectKind::Image`] ·
/// `VecPathRef` ⇒ [`ObjectKind::Vector`] · `FlipObjectRef` ⇒ [`ObjectKind::Flip`] ·
/// `PaintedDoc` ⇒ [`ObjectKind::Painted`]. Nenhum deles ⇒ [`ObjectKind::Empty`], que é
/// exatamente o objeto que a F3 aprende a criar.
///
/// ⚠️ **É derivado, e tem de continuar a ser.** Se algum dia isto virar um campo escrito à
/// mão numa entidade, passa a haver duas respostas para *"que objeto é este?"* — e a que o
/// artista vê é a que envelhece. A derivação vive num sítio só (o consumidor que resolve a
/// seleção), e este enum é só o vocabulário dela.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectKind {
    /// Sem marcador nenhum: `Transform` + `Name` e o que o artista acrescentar.
    Empty,
    /// Sprite de imagem (`ph2d::render::Sprite`).
    Image,
    /// Forma vetorial (`ph2d::ecs::VecPathRef`): cada forma é um objecto (spec/06 F3 ▸ Vector, 2.ª
    /// volta, 05/10 — o contentor `VecObject` de 04/10 saiu).
    Vector,
    /// Objeto Flip (`ph2d::ecs::FlipObjectRef`).
    Flip,
    /// Documento do Painter (`ph2d::ecs::PaintedDoc`).
    Painted,
    /// O esqueleto (`ph2d::skeleton::Skeleton`): os ossos são as partes dele (A14).
    Skeleton,
    /// ⭐ Um gerador do Motion Nodes (`ph2d::ecs::MotionGenerator`): os nós são dele, os moldes são
    /// os filhos (doc ObjectsAndComponents 01 §13, F5).
    Motion,
    /// ⭐ Um sprite de pixel art (`ph2d::ecs::PixelSprite`, ADR-0192): o documento de pixels vive na
    /// família `ph2d-app-pixel`, o objecto só o liga.
    PixelSprite,
}

impl ObjectKind {
    /// Todos, em ordem — a fonte da iteração (⛔ nunca escreva a lista uma segunda vez).
    pub const ALL: [ObjectKind; 8] = [
        ObjectKind::Empty,
        ObjectKind::Image,
        ObjectKind::Vector,
        ObjectKind::Flip,
        ObjectKind::Painted,
        ObjectKind::Skeleton,
        ObjectKind::Motion,
        ObjectKind::PixelSprite,
    ];

    /// O rótulo que o artista lê no filtro da paleta. Inglês (HR-15).
    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            ObjectKind::Empty => "component.object_kind.empty",
            ObjectKind::Image => "component.object_kind.image",
            ObjectKind::Vector => "component.object_kind.vector",
            ObjectKind::Flip => "component.object_kind.flip",
            ObjectKind::Painted => "component.object_kind.painted",
            ObjectKind::Skeleton => "component.object_kind.skeleton",
            ObjectKind::Motion => "component.object_kind.motion",
            ObjectKind::PixelSprite => "component.object_kind.pixel_sprite",
        }
    }

    /// O nome canónico do componente-MARCADOR que faz um objeto ser deste tipo.
    ///
    /// É esta função que impede o `ObjectKind` de virar vocabulário solto: cada variante
    /// (menos `Empty`, que é a ausência de todas) aponta para um tipo que o registo tem de
    /// conhecer, e o censo confere-o.
    #[must_use]
    pub const fn marker(self) -> Option<&'static str> {
        match self {
            ObjectKind::Empty => None,
            ObjectKind::Image => Some("ph2d::render::Sprite"),
            ObjectKind::Vector => Some("ph2d::ecs::VecPathRef"),
            ObjectKind::Flip => Some("ph2d::ecs::FlipObjectRef"),
            ObjectKind::Painted => Some("ph2d::ecs::PaintedDoc"),
            ObjectKind::Skeleton => Some("ph2d::skeleton::Skeleton"),
            ObjectKind::Motion => Some("ph2d::ecs::MotionGenerator"),
            ObjectKind::PixelSprite => Some("ph2d::ecs::PixelSprite"),
        }
    }

    const fn bit(self) -> u16 {
        1u16 << (self as u16)
    }
}

/// **Conjunto de tipos de objeto** — bitset sobre [`ObjectKind`], construível em `const`.
///
/// Bitset e não `&'static [ObjectKind]` porque o caso comum é *"vale para todos"* e
/// *"vale para imagem"*, e um `contains` de bit é o que a paleta faz por item por quadro
/// enquanto o artista digita na busca.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ObjectKinds(u16);

impl ObjectKinds {
    /// Vale para qualquer objeto — o caso de `Transform`, `Name`, `Visibility`, ordenação.
    /// ⚠️ Derivado do [`ObjectKind::ALL`]: o literal `0b1_1111` deixou de fora a 6.ª variante.
    pub const ANY: ObjectKinds = ObjectKinds::of(&ObjectKind::ALL);
    /// Só imagem — o caso do 9-Slice, da folha, da animação de sprite.
    pub const IMAGE: ObjectKinds = ObjectKinds(ObjectKind::Image.bit());
    /// Só vetor.
    pub const VECTOR: ObjectKinds = ObjectKinds(ObjectKind::Vector.bit());

    /// Qualquer objeto que tenha uma forma visível — tudo menos [`ObjectKind::Empty`],
    /// [`ObjectKind::Skeleton`] e [`ObjectKind::Motion`]. É a resposta certa para o que precisa de
    /// algo para desenhar (blend, máscara, camada de visibilidade): um vazio ou um esqueleto não têm
    /// pixels a misturar (os ossos são overlay do editor), e o que um Motion desenha é o fluxo dos
    /// sinks dele, não pixels da entidade.
    pub const DRAWABLE: ObjectKinds = ObjectKinds(
        ObjectKinds::ANY.0
            & !ObjectKind::Empty.bit()
            & !ObjectKind::Skeleton.bit()
            & !ObjectKind::Motion.bit()
            & !ObjectKind::PixelSprite.bit(),
    );

    /// Constrói a partir de uma lista.
    #[must_use]
    pub const fn of(kinds: &[ObjectKind]) -> ObjectKinds {
        let mut bits = 0u16;
        let mut i = 0;
        while i < kinds.len() {
            bits |= kinds[i].bit();
            i += 1;
        }
        ObjectKinds(bits)
    }

    /// União — para um componente que serve duas famílias.
    #[must_use]
    pub const fn or(self, other: ObjectKinds) -> ObjectKinds {
        ObjectKinds(self.0 | other.0)
    }

    #[must_use]
    pub const fn contains(self, kind: ObjectKind) -> bool {
        self.0 & kind.bit() != 0
    }

    /// Nenhum tipo — ⛔ estado inválido para um [`crate::Attach::Authored`], e o censo mata-o.
    /// Existe para o censo o poder PERGUNTAR (uma condição que não se pode exprimir não se
    /// pode gatear).
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Os tipos, em ordem — para a UI listar a razão de um item esmaecido.
    ///
    /// ⚠️ **Sem `#[must_use]` de propósito:** o `Iterator` que isto devolve **já** o é, e
    /// repeti-lo aqui sem mensagem é o que o clippy `--all-features` chama de redundante.
    pub fn iter(self) -> impl Iterator<Item = ObjectKind> {
        ObjectKind::ALL
            .into_iter()
            .filter(move |k| self.contains(*k))
    }
}
