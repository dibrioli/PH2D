//! **O que um CAMPO de um componente é** — o irmão de [`super`] pelo tecto de 700 LOC do HR-18.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE:** lá *o que um componente é* (categoria, tipo de objecto,
//! como se anexa); aqui *o que uma propriedade dele é* (o id declarado, o controlo que a edita, o
//! que a propagação mestre→instância lhe faz, e o que uma referência aponta).

/// **Um campo descrito.**
///
/// ⚠️ `field_id` é **DECLARADO e append-only**, nunca posicional — e isto não é gosto, é uma
/// refutação medida ([refutação 3 §1-b](../../../docs/Components/pesquisa/instancias_2026-08-21/refutacao_3_override_aninhado.md)):
/// o postcard é posicional, então trocar `Collider::Ball{radius}` por `Cuboid{hx,hy}` faria
/// um override de `radius` re-alvejar `hx` **em silêncio**. Com id declarado, o override
/// vira "sem alvo" (detetável) e os outros continuam certos. É o `FormerlySerializedAs` do
/// Unity, de graça.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FieldDesc {
    /// Estilo tag de protobuf: **nunca reusar, nunca reordenar, só apender**. Há gate de
    /// snapshot com prova de mutação.
    pub field_id: u16,
    /// O nome que o Inspector mostra. Inglês (HR-15).
    pub label_key: &'static str,
    /// Que controlo o edita.
    pub kind: FieldKind,
    /// O que a propagação mestre→instância faz com ele (F4).
    pub policy: Propagation,
    /// Se o campo é uma REFERÊNCIA a outra coisa, o que ele referencia — para o remap em
    /// toda propagação (F4). `None` = valor puro.
    pub is_ref: Option<RefKind>,
}

/// **Que controlo edita este campo** — espelha o vocabulário do `ParamRow` do Motion (12
/// variantes), que é o inspector derivado que já funciona no repo para ~180 tipos.
///
/// ⚠️ Duas variantes que o `ParamRow` **não** tem, e a razão: o Motion nasceu de um manifesto
/// em que *todo param é `f32`* (`ParamSpec { name, default: f32 }`), então ele não precisa de
/// distinguir inteiro de real nem de exprimir um par. Um componente do ECS precisa: o
/// `OrderInLayer` é `i32` e o eixo do `YSort` é um `Vec2`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FieldKind {
    /// Real contínuo — slider + chip numérico.
    Scalar,
    /// Inteiro — o chip arredonda (camada, ordem, índice).
    Int,
    /// Par de reais (eixo, tamanho, deslocamento).
    Vec2,
    /// Quatro reais que se editam como UM valor — um retângulo `[x, y, w, h]`.
    ///
    /// ⚠️ **Não é «um `Vec2` de posição mais um de tamanho»**, e a distinção é o override
    /// por-campo da F4: o `SpriteRegion.rect` é **um** campo do componente, então descrevê-lo
    /// como dois daria dois `field_id` a um só `[f32; 4]` — e um override gravado sobre metade
    /// dele não teria onde ser aplicado. O descritor espelha a ESTRUTURA, não o layout do painel.
    Vec4,
    /// Cor RGBA.
    Color,
    /// Liga/desliga.
    Toggle,
    /// Escolha entre variantes nomeadas.
    Enum,
    /// Ângulo — a caixa com o chip `deg`. **Guarda GRAUS** (a unidade autorada do app).
    Angle,
    /// Semente aleatória — caixa inteira + botão de re-rolar.
    Seed,
    /// Texto livre.
    Text,
    /// Curva autorada.
    Curve,
    /// Rampa de cor.
    Gradient,
    /// Paleta indexada.
    Palette,
    /// Marcador de tamanho zero: não tem valor, a **presença** é o valor.
    /// (`ShowBehindParent`, `TopLevel`, `Locked` — um componente sem campos.)
    Marker,
    /// **Uma REFERÊNCIA a outra coisa** — o controlo é um *picker*, nunca um número.
    ///
    /// ⚠️ **Não é redundante com o [`FieldDesc::is_ref`], e a divergência entre os dois é que
    /// seria o defeito:** `is_ref` diz *o que* o campo aponta (para o remap da F4), e `kind`
    /// diz *que controlo o edita*. Descrever o `PhysicsJoint.body_a` como [`Self::Int`]
    /// — que é o que ele é nos bytes, um `u64` — poria um chip numérico na frente do artista
    /// a pedir a identidade de um corpo. *O rótulo tem de prometer o que o modelo entrega.*
    ///
    /// Os dois andam juntos por gate (`a_ref_field_declares_both_halves`).
    Ref,
}

/// **O que a propagação mestre→instância faz com este campo** (ADR-0164 §2.4).
///
/// A política é por `(tipo, campo)` e não por tipo, porque a raiz de uma instância tem
/// `Transform` *local* e as peças dela têm `Transform` que *propaga* — o mesmo tipo, duas
/// respostas, decididas pelo sítio.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Propagation {
    /// Segue o mestre; overridável por campo. O default de quase tudo.
    Propagate,
    /// Nunca propaga, é sempre da instância, e **não conta como override** — os
    /// *"default overrides"* do Unity (`Transform`/`Name` da RAIZ da instância).
    InstanceLocal,
    /// Nem propaga nem é capturado como override: o dono é um sistema (a pose de uma peça
    /// cujo `pose_owner` é o solver ou o player). ⚠️ Sem isto o sync escreveria na célula
    /// que o solver possui e o readback marcaria um override por tique.
    RuntimeOwned,
}

/// **O que uma referência aponta** — para o remap em toda propagação (F4).
///
/// ⚠️ *Toda* propagação, não só a instanciação: o sync reescreve o componente sempre que o
/// mestre muda, então uma referência não remapeada faz a junta da instância prender os
/// corpos do MESTRE.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RefKind {
    /// Outro objeto, por `StableId` (F1). Hoje ainda é `stable_name_id`.
    Object,
    /// Um caminho vetorial, por `VecPathId`.
    VecPath,
    /// Um asset, por `LogicalId` (F6).
    Asset,
    /// ⭐ **Uma tag da árvore do projecto**, por `TagId` (TOP-20 #9).
    ///
    /// ⚠️ **Não se remapeia ao copiar, e é a decisão:** o `TagId` é do DOCUMENTO, não da subárvore
    /// copiada — a cópia de um inimigo continua a ser inimiga (decisão do dono D4). O remapeador da
    /// F4 só olha `Object`, e é esta variante própria que o impede de a confundir com um objecto.
    Tag,
}
