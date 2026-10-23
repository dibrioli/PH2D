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
    /// ⭐ **O nome com que um SCRIPT o lê e escreve** (`ph2d.get(e, "<code>.<name>")`) — o `snake`
    /// do rótulo inglês, a lei do Godot (`capitalize(name)` = rótulo; doc CodeEditor 08 §3.1, gate
    /// `o_nome_de_codigo_e_o_rotulo`). Vazio = ainda sem nome: o script não o alcança.
    pub name: &'static str,
    /// A chave serde do campo na struct, quando difere do [`Self::name`] (`height_world` ×
    /// `height`). Vazio = a mesma; [`WHOLE`] = o componente inteiro (um newtype).
    pub key: &'static str,
    /// Como o script vê o valor guardado: o que o CARTÃO mostra (doc CodeEditor 08 §3.12).
    pub view: View,
}

/// ⭐ **Como o script vê um valor guardado** — o que o cartão mostra, com uma excepção: um ângulo no
/// script é sempre em RADIANOS (a pose `rotation`, o `math.sin`, o Godot). Lei única em
/// `ph2d_ecs::scene::property`; gate de ida e volta em `ph2d-app-components`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum View {
    /// O valor guardado, tal e qual.
    Same,
    /// Guardado em graus; o script lê e escreve radianos (num par, cada elemento).
    Degrees,
    /// Guardado = script × k (`1_000_000`: µs no componente, segundos no cartão; `65_536`: q16).
    Scale(u32),
    /// Guardado = script − n: o cartão conta a partir de `n` e o componente de 0.
    Offset(i32),
    /// Booleano invertido: o cartão «Visible», o componente `hidden`.
    Not,
    /// Uma caixa no cartão sobre um enum: `(valor de true, valor de false)`, como em [`Self::Options`].
    Toggle(&'static str, &'static str),
    /// Uma escolha: `(nome no script, valor guardado)` — o nome é o `snake` do rótulo da opção no
    /// cartão; o valor é json (`0`, `null`, `true`) ou, se não o for, o NOME serde de uma variante
    /// unitária (`Full`). Escreve-se com [`crate::options!`].
    Options(&'static [(&'static str, &'static str)]),
    /// Pode ser infinito (sem limite): `math.huge` no script, `null` no valor serde.
    Unbounded,
    /// Guardado em fracção (0–1); o cartão e o script em percentagem (0–100).
    Percent,
    /// Cor guardada LINEAR; o cartão e o script em sRGB (cada canal de cor).
    Srgb,
    /// Uma referência guardada pela IDENTIDADE do alvo; o script lê e escreve o NOME dele (`nil` =
    /// ninguém). Lei em `ph2d_ecs::scene::property_ref`.
    Named(Target),
    /// Um par guardado como `{a, b}` (`SpawnAt::Area {w, h}`); o script vê `vec2(a, b)`.
    Pair(&'static str, &'static str),
}

/// **A que identidade um [`View::Named`] dá nome.**
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// Um objecto pelo próprio NOME (`String`, vazio = ninguém): o valor já é o nome, e escrever um
    /// que nenhum objecto tem é a gralha que se diz.
    Name,
    /// Um objecto pelo `stable_name_id` do seu `Name` (`u64`, `0` = ninguém).
    NameHash,
    /// Um objecto pelo seu `StableId` (`u64`, `0` = nenhum).
    Stable,
    /// Uma tag da árvore do projecto, pelo caminho (`"Enemy/Flying"`); `u64`, `0` = nenhuma.
    Tag,
    /// Um conjunto de tags (os `Tags` do objecto): uma lista de caminhos.
    TagSet,
    /// Uma camada de ordenação do projecto (`SortingLayers`), pelo nome; guardada pelo índice.
    Layer,
}

/// ⭐ **Uma [`View::Options`]**: `options!["full" => Full, "none" => Nothing, "inherit" => null]`. O
/// valor guardado escreve-se como IDENTIFICADOR (a variante serde, o `0`, o `null`): é código, nunca
/// texto pintado — o censo do HR-15 não o confunde com língua. [`View::Toggle`] usa `stringify!`.
#[macro_export]
macro_rules! options {
    ($($nome:literal => $v:tt),* $(,)?) => {
        $crate::View::Options(&[$(($nome, stringify!($v))),*])
    };
}

/// A chave de um campo que é o valor inteiro do componente (`OrderInLayer(i32)`).
pub const WHOLE: &str = ".";

/// **Porque um componente, ou uma chave dele, não tem nome de código** — a ausência declarada (doc
/// CodeEditor 08 §3.11; gate `todo_campo_de_um_cartao_tem_nome_ou_motivo`).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Hidden {
    /// Sem valor: a presença É o valor (`LockRotation`). Pôr e tirar é outro verbo.
    Presence,
    /// Uma lista: os itens não têm nome estável, e o cartão edita-os linha a linha.
    List,
    /// Editado fora do Inspector (o painel do Vetor, o modo do Esqueleto, a cena).
    OtherEditor,
    /// Estado da corrida, não autorado (`elapsed_ticks`): o cartão não o mostra.
    Runtime,
    /// Tem secção própria no cartão, com a sua lista (as resistências da Vida).
    OwnSection,
    /// Máquina do editor: identidade, instância ou documento — não é do jogo.
    Editor,
    /// Nenhum cartão do Inspector o mostra: o script alcança o que o artista vê.
    NoCard,
    /// ⏳ Uma referência a outro objecto ou a uma imagem: ganha nome pelo NOME do alvo (fase d4, doc
    /// CodeEditor 08 §5), nunca pelo id cru.
    Reference,
}

impl FieldDesc {
    /// O nome de código; a chave serde é a mesma.
    #[must_use]
    pub const fn named(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// O nome de código e a chave serde, quando diferem.
    #[must_use]
    pub const fn named_as(mut self, name: &'static str, key: &'static str) -> Self {
        self.name = name;
        self.key = key;
        self
    }

    /// ⏳ Sem nome de código, só a chave serde: o campo tem rótulo no cartão mas a chave está em
    /// `hiding` (uma [`Hidden::Reference`] por nome, que ganha nome no d4). Gate
    /// `o_nome_de_codigo_e_o_rotulo`: um campo sem nome tem de ter a chave oculta.
    #[must_use]
    pub const fn keyed(mut self, key: &'static str) -> Self {
        self.key = key;
        self
    }

    /// Como o script vê o valor ([`View`]).
    #[must_use]
    pub const fn viewed(mut self, view: View) -> Self {
        self.view = view;
        self
    }

    /// A chave do campo no valor serde do componente. Num enum com dados, `"chave:MOLDE/resto"`: o
    /// campo é o conteúdo da variante do molde json (`"target:{\"Named\":0}"`, `"at:{…}/tag"`), e
    /// escrevê-lo noutra variante passa o enum ao molde primeiro.
    #[must_use]
    pub const fn serde_key(&self) -> &'static str {
        if self.key.is_empty() {
            self.name
        } else {
            self.key
        }
    }

    /// A chave que o ledger da corrida grava e repõe: o enum inteiro, num campo de variante.
    #[must_use]
    pub fn stored_key(&self) -> &'static str {
        let k = self.serde_key();
        k.split_once(':').map_or(k, |(top, _)| top)
    }
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
