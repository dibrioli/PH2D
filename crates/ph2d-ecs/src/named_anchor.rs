//! **Named Anchors** — a §12 do Inspector do sprite ([ADR-0072], spec
//! [`07_named_anchors.md`](../../../docs/Sprite_projeto/07_named_anchors.md)).
//!
//! O ADR está **`Accepted` desde 2026-05-28** e, até 2026-08-21, `NamedAnchor` tinha **cinco
//! ocorrências no repositório — todas doc-comment ou `#[ignore]`**. Uma decisão ratificada sobre
//! código que não existe é a forma mais cara de dívida: ninguém a questiona, porque parece feita.
//!
//! # A unificação, que é a tese
//!
//! Três motores resolvem o mesmo problema com nomes diferentes — o **socket** do Paper 2D
//! (nome + pose), o **slice** do Aseprite (nome + retângulo + centro opcional + dados) e o
//! **image point** do Construct (nome + posição). Nenhum unifica os três. Aqui é **um** tipo, e o
//! que ele É lê-se da forma que tem:
//!
//! | `bounds` | `center` | é um… | serve para |
//! |---|---|---|---|
//! | `None` | `None` | **Socket** | boca de arma, mão, pé |
//! | `Some` | `None` | **Slice** | hitbox, hurtbox, zona de rosto |
//! | `Some` | `Some` | **região 9-slice** | caixa de diálogo nomeada |
//!
//! ⚠️ *A REPRESENTAÇÃO apaga o caso especial* — não há três tipos nem um enum de modo; há dois
//! campos opcionais, e a combinação deles é a semântica. É por isso que [`AnchorKind`] é uma
//! função **derivada** e nunca um campo guardado: um campo podia discordar dos dados.
//!
//! # Caps, e por que estão no TIPO
//!
//! A spec §7.12 congela seis limites. Os que se podem impor por construção estão impostos por
//! construção — [`SortedSmallVec`] não tem `push`, e é isso que torna o invariante de ordenação
//! verdadeiro em vez de declarado. *Um invariante FROZEN sem quem o imponha é uma mentira
//! arquitetural.*
//!
//! [ADR-0072]: ../../../docs/architecture/decisions/0072-named-anchor-unification.md

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::transform::Transform;

/// Comprimento máximo do nome, **em bytes UTF-8** (spec §7.12).
///
/// ⚠️ **Bytes, não caracteres**, e a diferença é determinismo: um emoji são 4 bytes, e contar
/// caracteres deixaria o mesmo nome caber numa máquina e não noutra conforme a normalização.
/// O postcard serializa com prefixo de comprimento em bytes — contar a mesma coisa que o
/// formato conta é o que mantém o hash de cozimento estável.
pub const ANCHOR_NAME_MAX_BYTES: usize = 64;

/// Máximo de âncoras por sprite (spec §7.12). Acima disto, o desenho pede hierarquia ECS.
pub const ANCHORS_MAX: usize = 64;

/// Profundidade máxima de um `Dict` aninhado (spec §7.12) — anti-recursão.
pub const DICT_MAX_DEPTH: u8 = 4;

/// Chaves máximas por nível de `Dict` (spec §7.12) — anti-explosão.
pub const DICT_MAX_KEYS: usize = 32;

/// Por que um nome foi recusado. ⚠️ **Recusar em silêncio seria pior que aceitar**: o artista
/// escreveria um nome, veria a lista não mudar e não saberia porquê.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AnchorNameError {
    /// Vazio — um identificador tem de identificar.
    Empty,
    /// Acima de [`ANCHOR_NAME_MAX_BYTES`].
    TooLong,
    /// Contém um caractere de controlo (inclui `\0` e o `ESC` de sequências ANSI).
    ControlChar,
    /// Já existe uma âncora com este nome no mesmo sprite.
    Duplicate,
    /// A LISTA está cheia (`ANCHORS_MAX`) — nada a ver com o nome.
    ///
    /// ⚠️ **Ela existe porque `TooLong` significava duas coisas** (auditoria 2026-08-22): a
    /// `insert` devolvia `TooLong` quando a lista enchia, e quem lia a mensagem via «over the
    /// limit of 64» tanto para um nome de 65 bytes como para a 65ª âncora. Só não era uma
    /// mentira **por coincidência** de os dois tetos serem 64: mexer num deles tornava uma das
    /// duas mensagens falsa, em silêncio.
    ListFull,
}

/// Valida um nome de âncora contra os caps da spec §7.12.
///
/// ⚠️ **A duplicação NÃO é vista aqui** — ela é uma propriedade da lista, não do nome. Quem a
/// vê é [`NamedAnchorList::insert`].
pub fn validate_anchor_name(name: &str) -> Result<(), AnchorNameError> {
    if name.is_empty() {
        return Err(AnchorNameError::Empty);
    }
    if name.len() > ANCHOR_NAME_MAX_BYTES {
        return Err(AnchorNameError::TooLong);
    }
    if name.chars().any(char::is_control) {
        return Err(AnchorNameError::ControlChar);
    }
    Ok(())
}

/// O payload livre de uma âncora (ADR-0072 §2.1).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum AnchorData {
    #[default]
    None,
    Str(String),
    Int(i64),
    Float(f64),
    Color([f32; 4]),
    /// Dicionário recursivo, com as chaves **sempre** ordenadas — ver [`SortedSmallVec`].
    ///
    /// ⚠️ **O `Box` não é estilo: sem ele isto não compila.** O ADR-0072 §2.1 escreve
    /// `Dict(SortedSmallVec)` com `SortedSmallVec(SmallVec<[(String, AnchorData); 8]>)` — e um
    /// `SmallVec` guarda o array **inline**, por isso um `AnchorData` que contenha um
    /// `SmallVec` de `AnchorData` tem tamanho infinito (`E0072`/`E0391`). A recursão precisa de
    /// uma indireção, e o `Box` é a mais barata: mantém o buffer inline de 8 pares que o ADR
    /// queria e paga um ponteiro só no nível que de facto aninha.
    ///
    /// *Um ADR `Accepted` cujo código de exemplo não compila é o preço de ratificar sobre
    /// código que não existe* — a mesma classe que a auditoria de 2026-08-21 mediu neste ADR.
    Dict(Box<SortedSmallVec>),
}

impl AnchorData {
    /// Profundidade de aninhamento: `None`/escalares são `1`, um `Dict` é `1 + máx(filhos)`.
    pub fn depth(&self) -> u8 {
        match self {
            Self::Dict(d) => {
                1u8.saturating_add(d.iter().map(|(_, v)| v.depth()).max().unwrap_or(0))
            }
            _ => 1,
        }
    }

    /// `true` quando a árvore respeita os caps de profundidade e de chaves por nível.
    pub fn within_caps(&self) -> bool {
        if self.depth() > DICT_MAX_DEPTH {
            return false;
        }
        match self {
            Self::Dict(d) => d.len() <= DICT_MAX_KEYS && d.iter().all(|(_, v)| v.within_caps()),
            _ => true,
        }
    }
}

/// Pares `(chave, valor)` **lexicograficamente ordenados por construção**.
///
/// ⚠️ **A API não tem `push`, `insert(idx)` nem `extend`, e isso é o ponto.** Um `SmallVec` cru
/// expõe três formas de violar o invariante em silêncio; um invariante que só existe num
/// comentário não é um invariante. A ordenação é o que torna o save/load byte-idêntico entre
/// sistemas operativos — e o `HashMap` está banido na simulação (ADR-0022).
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct SortedSmallVec(SmallVec<[(String, AnchorData); 8]>);

impl SortedSmallVec {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    /// Insere ou substitui, mantendo a ordem. Devolve o valor anterior, se havia.
    pub fn insert_sorted(&mut self, key: String, value: AnchorData) -> Option<AnchorData> {
        match self.0.binary_search_by(|(k, _)| k.cmp(&key)) {
            Ok(idx) => Some(std::mem::replace(&mut self.0[idx].1, value)),
            Err(idx) => {
                self.0.insert(idx, (key, value));
                None
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&AnchorData> {
        self.0
            .binary_search_by(|(k, _)| k.as_str().cmp(key))
            .ok()
            .map(|i| &self.0[i].1)
    }

    pub fn remove(&mut self, key: &str) -> Option<AnchorData> {
        let idx = self.0.binary_search_by(|(k, _)| k.as_str().cmp(key)).ok()?;
        Some(self.0.remove(idx).1)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(String, AnchorData)> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// ⚠️ **A desserialização VALIDA o invariante.** Sem isto, um ficheiro adulterado (ou gravado
/// por uma versão com um bug) entraria com as chaves fora de ordem e o `binary_search` passaria
/// a devolver `None` para chaves que existem — um "não encontrado" indistinguível de "não está lá".
impl<'de> Deserialize<'de> for SortedSmallVec {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = SmallVec::<[(String, AnchorData); 8]>::deserialize(d)?;
        if v.windows(2).all(|w| w[0].0 < w[1].0) {
            Ok(Self(v))
        } else {
            Err(serde::de::Error::custom(
                "AnchorData::Dict keys must be lexicographically sorted and unique",
            ))
        }
    }
}

/// O que uma âncora É — **derivado** de `bounds`/`center`, nunca guardado.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AnchorKind {
    /// Só uma pose: boca de arma, mão, ponto de spawn.
    Socket,
    /// Uma área: hitbox, hurtbox, zona.
    Slice,
    /// Área com miolo: uma região de 9-slice nomeada.
    NineSliceRegion,
}

/// Uma âncora nomeada (ADR-0072 §2.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NamedAnchor {
    /// Identificador único dentro do sprite, em `snake_case` por convenção.
    pub name: String,
    /// Pose relativa ao sprite. Reusa o [`Transform`] do ECS — o ADR corrigiu aqui um
    /// `Transform2D` que era vapor.
    pub transform: Transform,
    /// `[x, y, w, h]` em pixels da fonte. `Some` ⇒ é pelo menos um Slice.
    pub bounds: Option<[f32; 4]>,
    /// `[x, y, w, h]` dentro de `bounds`. Só tem sentido com `bounds` presente.
    pub center: Option<[f32; 4]>,
    #[serde(default)]
    pub user_data: AnchorData,
}

impl NamedAnchor {
    /// Uma âncora nova, no ponto de origem do sprite.
    pub fn socket(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transform: Transform::default(),
            bounds: None,
            center: None,
            user_data: AnchorData::None,
        }
    }

    /// O que esta âncora é, lido da forma que ela tem.
    ///
    /// ⚠️ **`center` sem `bounds` é um estado impossível** e lê-se como Socket: um miolo sem
    /// área que o contenha não descreve nada. A porta que edita ([`Self::set_bounds`]) impede-o
    /// de nascer; isto é a rede para o que vier de um ficheiro.
    pub fn kind(&self) -> AnchorKind {
        match (self.bounds, self.center) {
            (Some(_), Some(_)) => AnchorKind::NineSliceRegion,
            (Some(_), None) => AnchorKind::Slice,
            (None, _) => AnchorKind::Socket,
        }
    }

    /// Liga/desliga a área. ⚠️ Desligar `bounds` **leva o `center` consigo** — deixá-lo para trás
    /// criaria o estado impossível que o `kind()` tem de interpretar.
    pub fn set_bounds(&mut self, bounds: Option<[f32; 4]>) {
        self.bounds = bounds;
        if bounds.is_none() {
            self.center = None;
        }
    }

    /// Liga/desliga o miolo. Sem `bounds`, não faz nada — não há onde o pôr.
    pub fn set_center(&mut self, center: Option<[f32; 4]>) {
        if self.bounds.is_some() || center.is_none() {
            self.center = center;
        }
    }
}

/// **A lista de âncoras de um sprite.** Componente opcional: ausente = sprite sem âncoras.
///
/// ⚠️ **Não são entidades ECS filhas**, e isso é uma recusa medida (spec §7.14 anti-padrão 2):
/// cada socket viraria uma entidade com o seu custo, e a hierarquia — que é o que o artista lê —
/// encheria-se de linhas que não são objetos.
#[derive(Component, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NamedAnchorList(pub SmallVec<[NamedAnchor; 4]>);

impl NamedAnchorList {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &NamedAnchor> {
        self.0.iter()
    }

    pub fn get(&self, name: &str) -> Option<&NamedAnchor> {
        self.0.iter().find(|a| a.name == name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut NamedAnchor> {
        self.0.iter_mut().find(|a| a.name == name)
    }

    /// Insere, impondo os caps de nome, de unicidade e de contagem.
    ///
    /// ⚠️ **A porta ÚNICA de crescimento.** O campo é `pub` (o `postcard` precisa dele), mas todo
    /// gesto passa por aqui: é a diferença entre um cap escrito na spec e um cap que existe.
    pub fn insert(&mut self, anchor: NamedAnchor) -> Result<(), AnchorNameError> {
        validate_anchor_name(&anchor.name)?;
        if self.0.iter().any(|a| a.name == anchor.name) {
            return Err(AnchorNameError::Duplicate);
        }
        if self.0.len() >= ANCHORS_MAX {
            return Err(AnchorNameError::ListFull);
        }
        self.0.push(anchor);
        Ok(())
    }

    /// Remove por nome. `true` se removeu.
    pub fn remove(&mut self, name: &str) -> bool {
        let before = self.0.len();
        self.0.retain(|a| a.name != name);
        self.0.len() != before
    }

    /// Um nome livre da forma `anchor_N` — o default que a UI oferece.
    pub fn next_free_name(&self) -> String {
        for n in 0..=ANCHORS_MAX {
            let candidate = format!("anchor_{n}");
            if !self.0.iter().any(|a| a.name == candidate) {
                return candidate;
            }
        }
        // Inalcançável enquanto o cap for respeitado; devolver algo válido é melhor que entrar
        // em pânico numa porta de UI.
        String::from("anchor")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A tabela do ADR §2.1, verificada.** O que a âncora É lê-se da forma que tem.
    #[test]
    fn the_shape_is_the_kind() {
        let mut a = NamedAnchor::socket("muzzle");
        assert_eq!(a.kind(), AnchorKind::Socket);
        a.set_bounds(Some([0.0, 0.0, 8.0, 8.0]));
        assert_eq!(a.kind(), AnchorKind::Slice);
        a.set_center(Some([2.0, 2.0, 4.0, 4.0]));
        assert_eq!(a.kind(), AnchorKind::NineSliceRegion);
    }

    /// ⚠️ Desligar a área leva o miolo consigo — o estado impossível não pode nascer.
    #[test]
    fn dropping_the_bounds_takes_the_centre_with_it() {
        let mut a = NamedAnchor::socket("face");
        a.set_bounds(Some([0.0, 0.0, 8.0, 8.0]));
        a.set_center(Some([1.0, 1.0, 2.0, 2.0]));
        a.set_bounds(None);
        assert_eq!(a.center, None, "o miolo sobreviveu a` area que o continha");
        assert_eq!(a.kind(), AnchorKind::Socket);
    }

    /// E um miolo sem área nem chega a entrar.
    #[test]
    fn a_centre_without_bounds_is_refused_at_the_door() {
        let mut a = NamedAnchor::socket("x");
        a.set_center(Some([1.0, 1.0, 2.0, 2.0]));
        assert_eq!(a.center, None);
    }

    /// Os caps de nome da spec §7.12 — **em bytes**.
    #[test]
    fn the_name_caps_are_measured_in_bytes_not_chars() {
        assert_eq!(validate_anchor_name(""), Err(AnchorNameError::Empty));
        assert!(validate_anchor_name("muzzle").is_ok());
        // 16 emojis = 64 bytes: cabe exatamente.
        let emoji = "🔥".repeat(16);
        assert_eq!(emoji.len(), 64);
        assert!(validate_anchor_name(&emoji).is_ok());
        // 17 = 68 bytes: não cabe, apesar de serem só 17 "caracteres".
        assert_eq!(
            validate_anchor_name(&"🔥".repeat(17)),
            Err(AnchorNameError::TooLong)
        );
        assert_eq!(
            validate_anchor_name("bad\u{1b}[0m"),
            Err(AnchorNameError::ControlChar),
            "uma sequencia ANSI num nome vaza para todo log que o imprima"
        );
        assert_eq!(
            validate_anchor_name("nul\0"),
            Err(AnchorNameError::ControlChar)
        );
    }

    /// A lista impõe unicidade e contagem — os caps existem, não só se declaram.
    #[test]
    fn the_list_enforces_uniqueness_and_the_count_cap() {
        let mut l = NamedAnchorList::new();
        assert!(l.insert(NamedAnchor::socket("a")).is_ok());
        assert_eq!(
            l.insert(NamedAnchor::socket("a")),
            Err(AnchorNameError::Duplicate),
            "dois nomes iguais tornam `get` ambiguo, e ele responde sempre o primeiro"
        );
        while l.len() < ANCHORS_MAX {
            let n = l.next_free_name();
            l.insert(NamedAnchor::socket(n)).expect("dentro do cap");
        }
        assert_eq!(l.len(), ANCHORS_MAX);
        assert!(
            l.insert(NamedAnchor::socket("overflow")).is_err(),
            "o cap de {ANCHORS_MAX} nao foi imposto"
        );
    }

    /// `next_free_name` não colide, mesmo com buracos na sequência.
    #[test]
    fn the_offered_name_is_always_free() {
        let mut l = NamedAnchorList::new();
        for n in [0, 1, 3] {
            l.insert(NamedAnchor::socket(format!("anchor_{n}")))
                .unwrap();
        }
        assert_eq!(l.next_free_name(), "anchor_2", "saltou o buraco");
    }

    /// **O dicionário fica ordenado por construção**, insira-se por que ordem se inserir.
    #[test]
    fn the_dict_is_sorted_no_matter_the_insertion_order() {
        let mut d = SortedSmallVec::new();
        for k in ["zeta", "alpha", "mid", "beta"] {
            d.insert_sorted(k.to_string(), AnchorData::Int(1));
        }
        let keys: Vec<&str> = d.iter().map(|(k, _)| k.as_str()).collect();
        assert_eq!(keys, ["alpha", "beta", "mid", "zeta"]);
        assert_eq!(d.get("mid"), Some(&AnchorData::Int(1)));
        assert_eq!(d.get("nope"), None);
        // Reinserir substitui, não duplica.
        assert_eq!(
            d.insert_sorted("mid".into(), AnchorData::Int(2)),
            Some(AnchorData::Int(1))
        );
        assert_eq!(d.len(), 4);
        assert_eq!(d.remove("mid"), Some(AnchorData::Int(2)));
        assert_eq!(d.len(), 3);
    }

    /// ⚠️ Um `Dict` com chaves fora de ordem é **recusado na desserialização**, não aceite em
    /// silêncio — senão o `binary_search` diria «não existe» sobre uma chave que existe.
    #[test]
    fn out_of_order_keys_are_refused_on_load() {
        let good = SortedSmallVec({
            let mut v: SmallVec<[(String, AnchorData); 8]> = SmallVec::new();
            v.push(("a".into(), AnchorData::Int(1)));
            v.push(("b".into(), AnchorData::Int(2)));
            v
        });
        let bytes = postcard::to_allocvec(&good).unwrap();
        assert!(postcard::from_bytes::<SortedSmallVec>(&bytes).is_ok());

        let bad = SortedSmallVec({
            let mut v: SmallVec<[(String, AnchorData); 8]> = SmallVec::new();
            v.push(("b".into(), AnchorData::Int(2)));
            v.push(("a".into(), AnchorData::Int(1)));
            v
        });
        let bytes = postcard::to_allocvec(&bad).unwrap();
        assert!(
            postcard::from_bytes::<SortedSmallVec>(&bytes).is_err(),
            "as chaves desordenadas passaram — o binary_search passa a mentir"
        );
    }

    /// A profundidade e a largura do `Dict` são medidas, e o cap da spec §7.12 é o limite.
    #[test]
    fn the_dict_depth_and_width_caps_are_measured() {
        let leaf = AnchorData::Int(1);
        assert_eq!(leaf.depth(), 1);
        let mut d1 = SortedSmallVec::new();
        d1.insert_sorted("k".into(), leaf.clone());
        let lvl2 = AnchorData::Dict(Box::new(d1));
        assert_eq!(lvl2.depth(), 2);
        // Empilha até 4 (o cap) e depois passa dele.
        let mut cur = lvl2;
        while cur.depth() < DICT_MAX_DEPTH {
            let mut d = SortedSmallVec::new();
            d.insert_sorted("k".into(), cur);
            cur = AnchorData::Dict(Box::new(d));
        }
        assert_eq!(cur.depth(), DICT_MAX_DEPTH);
        assert!(cur.within_caps(), "no cap exacto tem de caber");
        let mut over = SortedSmallVec::new();
        over.insert_sorted("k".into(), cur);
        assert!(
            !AnchorData::Dict(Box::new(over)).within_caps(),
            "um nivel acima do cap tem de reprovar"
        );
        // Largura.
        let mut wide = SortedSmallVec::new();
        for i in 0..=DICT_MAX_KEYS {
            wide.insert_sorted(format!("k{i:03}"), AnchorData::Int(i as i64));
        }
        assert!(
            !AnchorData::Dict(Box::new(wide)).within_caps(),
            "o cap de {DICT_MAX_KEYS} chaves por nivel nao foi imposto"
        );
    }

    /// Round-trip da lista inteira pelo postcard — é como ela viaja no save e no undo.
    #[test]
    fn the_list_round_trips_through_postcard() {
        let mut l = NamedAnchorList::new();
        let mut a = NamedAnchor::socket("muzzle");
        a.transform.translation = ph2d_core::Vec2::new(28.0, -4.0);
        a.set_bounds(Some([8.0, 4.0, 24.0, 24.0]));
        a.set_center(Some([2.0, 2.0, 4.0, 4.0]));
        let mut d = SortedSmallVec::new();
        d.insert_sorted("dmg".into(), AnchorData::Int(10));
        d.insert_sorted("fires".into(), AnchorData::Str("projectile_basic".into()));
        a.user_data = AnchorData::Dict(Box::new(d));
        l.insert(a).unwrap();

        let bytes = postcard::to_allocvec(&l).unwrap();
        let back: NamedAnchorList = postcard::from_bytes(&bytes).unwrap();
        assert_eq!(back, l);
        assert_eq!(
            back.get("muzzle").unwrap().kind(),
            AnchorKind::NineSliceRegion
        );
    }
}
