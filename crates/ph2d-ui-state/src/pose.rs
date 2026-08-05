//! **A POSE** — tudo o que um objeto pode ter de diferente entre dois estados, e nada mais.

use ph2d_vec_scene::{Paint, StrokeSpec, VecPath, VecPathId};
use serde::{Deserialize, Serialize};

/// Onde um objeto está, como ele se parece, e que forma ele tem — num estado.
///
/// ⚠️ **O transform é DECOMPOSTO em T/R/S, e não é uma matriz.** Interpolar matrizes é o erro
/// clássico: o caminho entre duas rotações passa por dentro (a forma **encolhe** no meio) e uma
/// escala negativa vira cisalhamento. Aqui a representação **apaga o caso especial** — não existe
/// matriz para alguém lerpar por engano.
///
/// ⚠️ **A TINTA é campo de primeira classe, e não vive dentro da geometria.** Foi a autoria que
/// expôs isto: um botão que só muda de cor no hover tem a MESMA forma nos dois estados, então
/// `geometry` é `None` — e uma tinta que morasse lá dentro não teria por onde viajar. São dois
/// fatos independentes sobre um objeto, e um estado precisa de poder autorar um sem o outro.
///
/// ⚠️ **A geometria é opcional, e o `None` é o caso comum.** É isso que lhe poupa **0,64 ms por
/// objeto** (ver o doc da crate). `None` significa *este objeto não muda de forma neste estado* —
/// a forma vem da cena.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectPose {
    /// **A identidade, e a única chave de casamento.** Nunca o nome, nunca o índice.
    pub id: VecPathId,
    pub translation: [f64; 2],
    /// Radianos. Interpolada pelo **arco mais curto** — ver [`super::Transition::at`].
    pub rotation: f64,
    pub scale: [f64; 2],
    /// `0` = invisível, `1` = opaco. É o canal que carrega o fade de quem entra e de quem sai.
    pub opacity: f32,
    /// O preenchimento neste estado. `None` = sem preenchimento (não *"herda"*: um estado que
    /// autora um objeto autora a tinta dele).
    pub fill: Option<Paint>,
    /// O traço neste estado.
    pub stroke: Option<StrokeSpec>,
    /// A forma, quando este estado a autora. `None` ⇒ a cena manda, e nenhum `Plan` é construído.
    pub geometry: Option<VecPath>,
}

impl ObjectPose {
    /// Uma pose neutra: na origem, sem giro, escala 1, opaca, sem forma própria.
    #[must_use]
    pub fn new(id: VecPathId) -> Self {
        Self {
            id,
            translation: [0.0, 0.0],
            rotation: 0.0,
            scale: [1.0, 1.0],
            opacity: 1.0,
            fill: None,
            stroke: None,
            geometry: None,
        }
    }

    /// **Esta pose e a outra descrevem a mesma coisa?**
    ///
    /// ⚠️ É esta pergunta — e não um limiar — que decide se o par ENTRA na transição. Um objeto
    /// que não muda entre dois estados não é interpolado, não constrói `Plan` e não aparece na
    /// conta: *não animar* é mais barato e mais correto que *animar de x para x*.
    #[must_use]
    pub fn is_same_as(&self, other: &Self) -> bool {
        self == other
    }
}

/// Um ESTADO: o nome que o artista lhe deu, e a pose de cada objeto que ele autora.
///
/// ⚠️ **O nome é rótulo, nunca chave.** Ele existe para o artista escolher *"hover"* num menu; o
/// casamento entre estados é sempre pelo [`ObjectPose::id`]. Um estado renomeado continua a casar
/// com o mesmo objeto, e é isso que o gate de rename prova.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiState {
    pub name: String,
    pub objects: Vec<ObjectPose>,
}

impl UiState {
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            objects: Vec::new(),
        }
    }

    /// A pose de `id` neste estado, se ele a autora.
    ///
    /// ⚠️ Busca LINEAR de propósito: um estado de UI tem dezenas de objetos, não milhares, e um
    /// mapa aqui custaria uma ordem de iteração que o `serde` teria de preservar para o arquivo
    /// ser determinista. Se um dia a conta crescer, o índice nasce em [`super::Transition::new`],
    /// que roda **uma vez por par** — nunca por frame.
    #[must_use]
    pub fn pose(&self, id: VecPathId) -> Option<&ObjectPose> {
        self.objects.iter().find(|p| p.id == id)
    }
}
