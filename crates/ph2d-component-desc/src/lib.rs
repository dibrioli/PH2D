//! **O descritor de componente** — um componente descreve-se UMA vez, e disso derivam o
//! Inspector, o override por campo, o remap de referências, a política de propagação e a
//! paleta do `+`.
//!
//! Governança: [ADR-0164](../../../docs/architecture/decisions/0164-instances-are-real-entities-linked-by-stableid-with-live-sync-and-incremental-undo.md)
//! (fase F0) e [ADR-0166](../../../docs/architecture/decisions/0166-the-inspector-shows-what-the-object-has-and-components-attach-through-one-palette-filtered-by-object-type.md)
//! (`category` · `attach` · `applies_to`).
//!
//! # Por que a chave é o NOME, e não o tipo
//!
//! O caminho óbvio seria enfiar o descritor no `register::<T>()` do `ComponentRegistry`. Ele
//! foi **medido e recusado**: são **107 sítios de chamada em 5 crates**
//! (`ph2d-ecs` 69 · `ph2d-render` 1 · `ph2d-physics-ecs` 32 · `ph2d-field-ecs` 5 ·
//! `ph2d-script` 1, este último ainda não chamado no boot), e tocar os 107 põe **toda linha
//! que acrescenta um componente** a escrever no mesmo sítio que todas as outras — que é
//! exatamente a superfície de colisão que a DIRETRIZ §1.5.2.1 manda projetar para fora.
//!
//! Em vez disso: **side-metadata chaveada pelo nome canónico**, que já existe nos cinco
//! registros e é o único identificador que atravessa as crates sem as ligar. O precedente da
//! casa é o [`NodeUiManifest`] do `ph2d-node-registry` — *"additive and non-frozen, it lives
//! beside the ops"*, chaveado por `NodeTypeId`. Aqui é o mesmo padrão um nível acima, e o
//! catálogo é cortado por FAMÍLIA ([`catalog`]), de modo que uma linha que acrescente um
//! componente de física apende em `catalog/physics.rs` e não encosta no resto.
//!
//! [`NodeUiManifest`]: https://github.com/dibrioli/PH2D/blob/main/crates/ph2d-node-registry/src/ui.rs
//!
//! # ⚠️ O preço da chave por nome, e o gate que o paga
//!
//! Uma tabela chaveada por string **diverge em silêncio**: renomeie o componente, esqueça o
//! catálogo, e o descritor deixa de ser encontrado sem que nada falhe. É a mesma classe do
//! *"componente não registado é descartado EM SILÊNCIO"* que o próprio `registry.rs` avisa.
//!
//! A cura não é disciplina, é um **censo de dois lados** (a lei da casa: ausência *e*
//! presença), e ele vive onde o registo está COMPLETO — a shell, que é quem chama os cinco
//! registradores: *todo tipo registado tem descritor* **e** *todo descritor nomeia um tipo
//! registado*. Sem os dois lados, metade da deriva passa.

#![forbid(unsafe_code)]

pub mod catalog;

pub use catalog::{all, desc_for};

/// **O descritor de um tipo de componente.**
///
/// `fields` pode ser vazio: a F0 descreve os CAMPOS de um punhado de tipos (o piloto), mas
/// declara `category`/`attach` para **todos** os 107 — porque a paleta da F3 precisa de saber
/// em que gaveta cada tipo vive, e isso é uma linha por tipo, não um esquema por tipo.
/// A tabela de campos cresce **append-only**, por procura.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ComponentDesc {
    /// `ph2d::<crate>::<TypeName>` — a MESMA string do `ComponentRegistry`. É a chave.
    pub canonical_name: &'static str,
    /// O rótulo que o artista lê na paleta e no cabeçalho da seção. Inglês (HR-15), e
    /// **nomeado pelo resultado**, não pelo tipo Rust ("9-Slice", não "SliceNine").
    pub display_name: &'static str,
    /// A gaveta colorida da paleta.
    pub category: ComponentCategory,
    /// Autorável (e onde se aplica) ou máquina.
    pub attach: Attach,
    /// Os campos descritos, em ordem de `field_id` crescente. Vazio = ainda não descrito.
    pub fields: &'static [FieldDesc],
    /// ⭐ **O que este componente NÃO FUNCIONA SEM** — nomes canónicos, anexados em cascata
    /// (ADR-0166 / F3). O equivalente do `[RequireComponent]` do Unity e do `#[require]` do Bevy.
    ///
    /// ⚠️ **A cascata é MOSTRADA antes de ser aplicada**, e essa é a correção da crítica medida ao
    /// Bevy ([discussão #16570](https://github.com/bevyengine/bevy/discussions/16570), doc 02
    /// §1.4): *«não vejo o que vem junto»*. A dependência automática cura o erro de setup e **cria**
    /// um problema de visibilidade — e num editor a UI é o sítio barato de o resolver. Aqui ela
    /// viaja no **rótulo do item da paleta**, pela mesma porta que a razão do inaplicável.
    ///
    /// ⛔ **Só o que é ESTRUTURAL entra**, nunca o que é boa prática: a barra é *o componente é
    /// inerte sem aquele*. A ponte da física consulta `(RigidBody, Collider, Transform)` — um corpo
    /// sem collider nunca é simulado, e isso é uma query, não uma opinião.
    pub requires: &'static [&'static str],
}

impl ComponentDesc {
    /// **Um componente que o artista anexa.**
    ///
    /// Construtor `const` porque o catálogo é `static`, e porque escrever a struct inteira
    /// 108 vezes custaria ~250 linhas a mais por família — o teto de 700 LOC por arquivo é
    /// real, e um catálogo que não cabe num arquivo vira dois com a mesma responsabilidade.
    #[must_use]
    pub const fn authored(
        canonical_name: &'static str,
        display_name: &'static str,
        category: ComponentCategory,
        applies_to: ObjectKinds,
        fields: &'static [FieldDesc],
    ) -> Self {
        Self {
            canonical_name,
            display_name,
            category,
            attach: Attach::Authored { applies_to },
            fields,
            requires: &[],
        }
    }

    /// **Um componente autorado que não funciona sem outros** — ver [`ComponentDesc::requires`].
    ///
    /// ⚠️ **Construtor à parte, e não um argumento a mais no [`Self::authored`]:** aquele é chamado
    /// ~90 vezes, e acrescentar-lhe um `&[]` em todas elas seria ruído em 90 sítios para servir
    /// dois. *A excepção paga o preço dela.*
    #[must_use]
    pub const fn authored_requiring(
        canonical_name: &'static str,
        display_name: &'static str,
        category: ComponentCategory,
        applies_to: ObjectKinds,
        fields: &'static [FieldDesc],
        requires: &'static [&'static str],
    ) -> Self {
        Self {
            canonical_name,
            display_name,
            category,
            attach: Attach::Authored { applies_to },
            fields,
            requires,
        }
    }

    /// **Dado do artista que chega com o gesto** — ver [`Attach::Intrinsic`]. Recebe
    /// `fields` porque ele **pode ter seção**: a `VecShape` não se anexa pela paleta e é
    /// editadíssima.
    #[must_use]
    pub const fn intrinsic(
        canonical_name: &'static str,
        display_name: &'static str,
        category: ComponentCategory,
        fields: &'static [FieldDesc],
    ) -> Self {
        Self {
            canonical_name,
            display_name,
            category,
            attach: Attach::Intrinsic,
            fields,
            requires: &[],
        }
    }

    /// **Um componente que é máquina** — nunca oferecido, nunca uma seção.
    ///
    /// Não recebe `fields` de propósito: descrever os campos de algo que não tem seção seria
    /// construir a tabela que ninguém lê. Se um dia um `Machinery` precisar de campos
    /// descritos (para o remap de referências da F4, por exemplo), este construtor é o sítio
    /// onde essa decisão fica visível.
    #[must_use]
    pub const fn machinery(
        canonical_name: &'static str,
        display_name: &'static str,
        category: ComponentCategory,
    ) -> Self {
        Self {
            canonical_name,
            display_name,
            category,
            attach: Attach::Machinery,
            fields: &[],
            requires: &[],
        }
    }

    /// O campo de `field_id`, se descrito.
    #[must_use]
    pub fn field(&self, field_id: u16) -> Option<&'static FieldDesc> {
        // Linear: a maior tabela de campos do app tem 20 entradas (a `Sprite`), e a busca
        // acontece por override capturado, não por quadro. Um `binary_search` aqui compraria
        // nada e pediria que a ordem fosse invariante de RUNTIME em vez de um gate.
        self.fields.iter().find(|f| f.field_id == field_id)
    }

    /// É oferecido na paleta a um objeto deste tipo? Só o [`Attach::Authored`] é.
    #[must_use]
    pub fn is_offered_to(&self, kind: ObjectKind) -> bool {
        match self.attach {
            Attach::Authored { applies_to } => applies_to.contains(kind),
            Attach::Intrinsic | Attach::Machinery => false,
        }
    }

    /// Aparece na paleta do `+` em ALGUM tipo de objeto?
    #[must_use]
    pub fn is_offered(&self) -> bool {
        matches!(self.attach, Attach::Authored { .. })
    }

    /// **Pode ter uma seção no Inspector?** — `Authored` e `Intrinsic` sim, `Machinery` não.
    ///
    /// ⚠️ É uma pergunta DIFERENTE de [`Self::is_offered`], e confundi-las é o defeito que a
    /// terceira variante existe para impedir: a `VecShape` não se anexa pela paleta e é das
    /// coisas mais editadas do app.
    #[must_use]
    pub fn may_have_section(&self) -> bool {
        !matches!(self.attach, Attach::Machinery)
    }
}

/// **Autorável × máquina** — e a ausência é DECLARADA, nunca um esquecimento.
///
/// ⚠️ Quatro tipos registados são **pontes de identidade**, não escolhas: `VecPathRef`,
/// `PaintedDoc`, `BakedForm` e `FlipObjectRef` (cada um é um `u32`/id opaco que liga a
/// entidade ao documento do módulo dela). Uma paleta que listasse *"todo tipo registado"*
/// ofereceria as quatro ao artista. Marcá-las [`Attach::Machinery`] é o que as tira — e por
/// serem declaradas, o censo consegue exigir que **toda** ausência tenha um autor.
/// ⚠️ **Os três estados foram MEDIDOS, não desenhados.** A versão de duas variantes
/// (`Authored`/`Machinery`) sobreviveu até o compilador a refutar: ao converter os
/// registradores para `register_default`, **27 dos 109 tipos não implementam `Default`** —
/// e 17 deles estavam marcados `Authored`. A paleta oferecê-los-ia e não os conseguiria
/// construir. *Um binário de duas casas para uma pergunta de três.*
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Attach {
    /// O artista anexa-o pelo `+`. `applies_to` é o conjunto de tipos de objeto em que ele
    /// tem efeito — ⛔ **nunca vazio** (seria um componente inalcançável em todo objeto, que
    /// é a forma de dizer "morto" sem o dizer). Há gate.
    ///
    /// ⛔ **Exige `insert_default` no registo**, e há censo a prová-lo: a paleta insere o
    /// ponto NEUTRO do tipo, então um tipo sem `Default` não pode estar aqui.
    Authored { applies_to: ObjectKinds },
    /// **Dado do artista que chega com o GESTO** — a forma de um caminho vetorial, os pixels
    /// de uma sprite, o nome de um objeto. Nunca oferecido na paleta; **pode ter seção**.
    ///
    /// São dois motivos distintos, e vale a pena distingui-los ao ler o catálogo:
    ///
    /// 1. **Não há neutro que signifique alguma coisa.** Uma `VecShape` sem geometria não é
    ///    uma forma vazia — não é uma forma. Uma `Sprite` exige uma `source`.
    /// 2. ⚠️ **O neutro existe e anexá-lo seria um NO-OP** — a cerca que o `MassOverride` e o
    ///    `Dominance` documentam: *"absent = the neutral default and the Inspector detaches
    ///    it at 0 (a project file stays free of the no-op)"*. Para estes, a presença é que
    ///    carrega o sentido, e o valor de anexação tem de vir do CONTEXTO (a massa que o
    ///    corpo tem agora), que a paleta genérica não conhece.
    ///
    /// ⇒ **É por isto que nem todas as portas por-seção de hoje são redundantes** (ADR-0166):
    /// as que SEMEIAM um valor do contexto vivo fazem algo que o `+` não pode fazer.
    Intrinsic,
    /// Máquina: nunca oferecido na paleta, nunca uma seção do Inspector. Identidade interna
    /// ou dado derivado (as quatro pontes, o `RootOrder`).
    Machinery,
}

/// **Que TIPO de objeto é este** — e a resposta lê-se por **PRESENÇA de um marcador**, nunca
/// por um campo.
///
/// Medido em 2026-08-24 sobre os 107 registados: `Sprite` ⇒ [`ObjectKind::Image`] ·
/// `VecPathRef` ⇒ [`ObjectKind::Vector`] · `FlipObjectRef` ⇒ [`ObjectKind::Flip`] ·
/// `PaintedDoc` ⇒ [`ObjectKind::Painted`] · `FieldObject` ⇒ [`ObjectKind::Model3D`] ·
/// `BakedForm` ⇒ [`ObjectKind::Sculpt3D`]. Nenhum deles ⇒ [`ObjectKind::Empty`], que é
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
    /// Caminho vetorial (`ph2d::ecs::VecPathRef`).
    Vector,
    /// Objeto Flip (`ph2d::ecs::FlipObjectRef`).
    Flip,
    /// Documento do Painter (`ph2d::ecs::PaintedDoc`).
    Painted,
    /// Peça de modelagem 3D por campo implícito (`ph2d::field::FieldObject`).
    Model3D,
    /// Forma assada do módulo de escultura (`ph2d::ecs::BakedForm`).
    Sculpt3D,
}

impl ObjectKind {
    /// Todos, em ordem — a fonte da iteração (⛔ nunca escreva a lista uma segunda vez).
    pub const ALL: [ObjectKind; 7] = [
        ObjectKind::Empty,
        ObjectKind::Image,
        ObjectKind::Vector,
        ObjectKind::Flip,
        ObjectKind::Painted,
        ObjectKind::Model3D,
        ObjectKind::Sculpt3D,
    ];

    /// O rótulo que o artista lê no filtro da paleta. Inglês (HR-15).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            ObjectKind::Empty => "Empty",
            ObjectKind::Image => "Image",
            ObjectKind::Vector => "Vector",
            ObjectKind::Flip => "Flip",
            ObjectKind::Painted => "Painted",
            ObjectKind::Model3D => "3D Model",
            ObjectKind::Sculpt3D => "Sculpt",
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
            ObjectKind::Model3D => Some("ph2d::field::FieldObject"),
            ObjectKind::Sculpt3D => Some("ph2d::ecs::BakedForm"),
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
    pub const ANY: ObjectKinds = ObjectKinds(0b111_1111);
    /// Só imagem — o caso do 9-Slice, da folha, da animação de sprite.
    pub const IMAGE: ObjectKinds = ObjectKinds(ObjectKind::Image.bit());
    /// Só vetor.
    pub const VECTOR: ObjectKinds = ObjectKinds(ObjectKind::Vector.bit());
    /// Só modelagem 3D por campo.
    pub const MODEL3D: ObjectKinds = ObjectKinds(ObjectKind::Model3D.bit());

    /// Qualquer objeto que tenha uma forma visível — tudo menos [`ObjectKind::Empty`].
    /// É a resposta certa para o que precisa de algo para desenhar (blend, máscara, camada
    /// de visibilidade): um objeto vazio não tem o que misturar.
    pub const DRAWABLE: ObjectKinds = ObjectKinds(ObjectKinds::ANY.0 & !ObjectKind::Empty.bit());

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

    /// Nenhum tipo — ⛔ estado inválido para um [`Attach::Authored`], e o censo mata-o.
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

/// **A gaveta colorida da paleta** — o [`NodeUiCategory`] dos componentes.
///
/// ⚠️ As categorias foram **contadas sobre os 107 registados**, não inventadas: cada uma
/// existe porque há tipos reais nela hoje. A contagem por categoria vive no censo, não aqui
/// (um número escrito num comentário é um número que envelhece).
///
/// [`NodeUiCategory`]: https://github.com/dibrioli/PH2D/blob/main/crates/ph2d-node-registry/src/ui.rs
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComponentCategory {
    /// Nome, visibilidade, trava, agrupamento — o que todo objeto é.
    Identity,
    /// Pose no mundo.
    Transform,
    /// Camada, ordem, Z, Y-sort, recorte de filhos.
    Ordering,
    /// Como o pixel sai: blend, filtro, repetição, máscara, emissivo, UV.
    Rendering,
    /// Sprite de imagem: pixels, folha, região, 9-slice.
    Image,
    /// Animação de sprite (a §11 do Inspector).
    Animation,
    /// Âncoras e encaixes nomeados (a §12).
    Anchors,
    /// Geometria vetorial e os efeitos vivos dela.
    Vector,
    /// Corpos, colisores, juntas, zonas, player.
    Physics,
    /// Campo implícito e escultura.
    Model3D,
    /// Script do utilizador.
    Scripting,
    /// Instância, mestre, override (nascem na F4).
    Instancing,
}

impl ComponentCategory {
    /// Todas, na ordem em que a paleta as mostra. ⛔ Fonte única da iteração.
    pub const ALL: [ComponentCategory; 12] = [
        ComponentCategory::Identity,
        ComponentCategory::Transform,
        ComponentCategory::Ordering,
        ComponentCategory::Rendering,
        ComponentCategory::Image,
        ComponentCategory::Animation,
        ComponentCategory::Anchors,
        ComponentCategory::Vector,
        ComponentCategory::Physics,
        ComponentCategory::Model3D,
        ComponentCategory::Scripting,
        ComponentCategory::Instancing,
    ];

    /// O cabeçalho do grupo na paleta. Inglês (HR-15).
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            ComponentCategory::Identity => "Identity",
            ComponentCategory::Transform => "Transform",
            ComponentCategory::Ordering => "Ordering",
            ComponentCategory::Rendering => "Rendering",
            ComponentCategory::Image => "Image",
            ComponentCategory::Animation => "Animation",
            ComponentCategory::Anchors => "Anchors",
            ComponentCategory::Vector => "Vector",
            ComponentCategory::Physics => "Physics",
            ComponentCategory::Model3D => "3D",
            ComponentCategory::Scripting => "Scripting",
            ComponentCategory::Instancing => "Instancing",
        }
    }
}

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
    pub name: &'static str,
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
}
