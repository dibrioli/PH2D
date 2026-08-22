//! **A POSE** — tudo o que um objeto pode ter de diferente entre dois estados, e nada mais.

use crate::role::StateRole;
use ph2d_fx_op::FxOp;
use ph2d_stroke_width::WidthStops;
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
/// expôs isto: um botão que só muda de cor no hover tem a MESMA forma nos dois estados, e uma
/// tinta que morasse lá dentro **não teria por onde viajar** — o par de formas idênticas nem
/// entra no casamento. São dois fatos independentes sobre um objeto, e um estado precisa de
/// poder autorar um sem o outro.
///
/// ⚠️ **A geometria é a FONTE autorada, e a autoria grava-a sempre.** Ela chegou a ser opcional
/// *"porque a forma raramente muda"* — e o preço dessa frase foi que uma edição de nó, um Fillet
/// ou um Chamfer **não animavam**: o campo existia, o motor sabia casá-lo, e ninguém o
/// preenchia. `None` sobrou para o caso honesto de não haver forma nenhuma a que se agarrar (o
/// objeto sumiu do documento).
///
/// ⚠️ **E o que poupa os 0,64 ms por objeto (ver o doc da crate) é a IGUALDADE, não a ausência:**
/// duas formas idênticas não constroem `Plan` nenhum. O par só-de-cor continua a custar zero.
///
/// ⚠️ **Fonte, nunca cozido.** É a fonte que o modo Node edita e é ela que a chegada devolve ao
/// documento; guardar a cozida assaria o raio de quina e a pilha de efeitos, e o artista
/// perderia as alças no primeiro Show. Quem coze é o Blend, para o CAMINHO — a costura
/// fonte≠cozido do ADR-0121, no nível do estado.
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
    /// **O perfil de largura viva** (ADR-0148), quando o traço tem um. `None` = largura uniforme.
    ///
    /// ⚠️ **É o único canal de forma que não mora no `VecPath`** — ele é um componente ECS —, e
    /// é por isso que tem campo próprio aqui em vez de viajar dentro da `geometry`. Sem ele o
    /// Width Tool seria a única ferramenta de desenho cuja edição não animaria entre dois
    /// estados, e a ausência não teria nome nenhum.
    pub width: Option<WidthStops>,
    /// **A pilha de FX raster** (blur, glow, sombra, ajuste de cor) neste estado. Vazia = sem
    /// filtro.
    ///
    /// ⚠️ **Pela mesma razão do [`Self::width`]**: é um canal de APARÊNCIA que não vive no
    /// `VecPath` — ele é um componente ECS (`ph2d_ecs::VecFilter`) —, então a pose tem de o
    /// carregar por si. Sem ele, um blur seria o único efeito do editor incapaz de diferir entre
    /// *Default* e *Hover*, e a seção Filters do painel ficaria muda dentro de um botão animado.
    ///
    /// ⚠️ **E o vazio NÃO é "herda"**, é *sem filtro* — a mesma lei do `fill`. Quando um dos
    /// lados não conhece um degrau que o outro tem, quem resolve é o alinhamento
    /// (`ph2d_fx_op::mix_stacks`), que o faz crescer do neutro em vez de o omitir.
    pub filters: Vec<FxOp>,
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
            width: None,
            filters: Vec::new(),
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

/// Um ESTADO: o papel que ele desempenha, e a pose de cada objeto que ele autora.
///
/// ⚠️ **O papel é a CHAVE do estado; o id é a chave do OBJETO.** São duas perguntas diferentes e
/// cada uma tem uma resposta só — *que estado é este?* responde-se com o
/// [`StateRole`], *quem é quem entre dois estados?* com o [`ObjectPose::id`]. Nenhuma das duas é
/// um nome livre, e é isso que faz uma animação sobreviver a renomear e a reordenar.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiState {
    pub role: StateRole,
    pub objects: Vec<ObjectPose>,
}

impl UiState {
    #[must_use]
    pub fn new(role: StateRole) -> Self {
        Self {
            role,
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
