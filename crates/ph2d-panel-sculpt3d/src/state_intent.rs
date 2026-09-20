//! ⭐⭐⭐ **O QUE O PAINEL PEDE** — o [`Sculpt3dIntent`], irmão do [`super::state`] pelo tecto de
//! `600` LOC.
//!
//! ⚠️ **O corte é por RESPONSABILIDADE e não por tamanho:** lá fica o que o painel **MOSTRA** (o
//! `Sculpt3dUi` autorado e o `Sculpt3dSnapshot` que a cena publica), aqui o que ele **PEDE**. São
//! as duas pontas do mesmo canal e mudam por motivos diferentes — um knob novo engorda o retrato,
//! um gesto novo engorda esta lista.
//!
//! ⛔ **Quem o obrigou foi a wave do selector de pincéis** (2026-09-20): o
//! [`Sculpt3dIntent::OpenBrushPalette`] pôs o ficheiro em `608/600`, e a lei desta casa é **cortar,
//! nunca escrever uma entrada no `FILE_OVERAGE_OK`**.

use crate::state::Sculpt3dUi;

/// Um gesto do artista, para o shell aplicar.
///
/// ⚠️ **O `SetUi` é MUITO maior que os irmãos, e a caixa não entra — o
/// precedente é o `Step` do `ph2d-ui-state`.** Ele carrega o estado autorado
/// inteiro de propósito (é *"substitua o pincel por este"*, não *"mude este
/// campo"*), e a fila tem **um punhado de elementos por gesto**, drenada no mesmo
/// frame: um `Box` compraria bytes numa fila efémera ao preço de uma indireção e
/// de uma alocação por clique. ⚠️ E o aviso **nasceu de crescer o `Brush`** —
/// a lâmina em V lhe acrescentou dois campos —, o que quer dizer que ele mede a
/// LARGURA do estado autorado e não um defeito desta fila.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum Sculpt3dIntent {
    /// Substitui o estado autorado inteiro — ver [`Sculpt3dUi`].
    SetUi(Sculpt3dUi),
    /// **Arma (ou desarma) o transform.** Ver `ph2d_sculpt3d::TransformKind`.
    ///
    /// ⚠️ Ele carrega o TIPO e não um `Option`: a cena é quem sabe o que já
    /// está armado, e mandar *"desligue"* de dentro do painel exigiria que ele
    /// guardasse uma segunda cópia do arm para decidir. Clicar o aceso desarma —
    /// e quem faz essa conta é `Sculpt3dScene::arm_transform`, uma vez.
    ArmTransform(ph2d_sculpt3d::TransformKind),
    /// ⭐⭐⭐ **ABRE A PALETA DE PINCÉIS** — o selector que saiu do painel (ordem do dono,
    /// 2026-09-20).
    ///
    /// ⚠️ **Sem operando, e a razão é que ele não escolhe nada**: escolher é o que o *pick* da
    /// paleta faz, noutro quadro. Este intent só diz *«mostra-me o catálogo»* — e é por isso que
    /// quem o serve é o [`crate::…::panel_bridge::dispatch`], que tem o `HeroScreen` na mão, e não
    /// o `apply_panel_intent`, que só tem a cena.
    OpenBrushPalette,
    /// **Arma (ou desarma) o FILTRO** — o verbo corrente na malha inteira.
    ///
    /// ⚠️ **Sem operando, e a assimetria com o irmão acima é REAL, não
    /// descuido:** o transform tem três espécies e por isso manda qual; o filtro
    /// tem uma só, porque *qual lei ele roda* já está respondido pelo verbo na
    /// mão. Um operando aqui seria uma segunda porta para escolher a ferramenta.
    ArmFilter,
    /// **Re-arma a IMAGEM que a cena lembra** — o chip do slot de imagem.
    ///
    /// ⚠️ **Um intent e não um `SetUi`**, porque o painel **não tem** a imagem:
    /// quando o artista escolhe `Grain`, o `Arc<AlphaImage>` sai do
    /// [`Sculpt3dUi`] e só a cena continua a segurá-lo. Sem esta porta o chip da
    /// imagem seria um controle que só sabe deixar de estar aceso.
    ArmStoredImage,
    /// Liga/desliga a topologia dinâmica (e triangula, se ligar).
    ToggleDyntopo,
    /// Desce (`false`) ou sobe (`true`) um nível de multiresolução.
    ChangeLevel(bool),
    Subdivide,
    ReverseLevel,
    /// Achata a pilha de multiresolução numa malha só.
    Flatten,
    Remesh,
    /// **RETOPOLOGIA por campo cruzado** (ADR-0160). Irmã do [`Self::Remesh`] e
    /// não substituta: aquele re-amostra um campo de voxels (a arrumação
    /// destrutiva), esta preserva a topologia e alinha a grade à FORMA.
    QuadRemesh,
    CloseHoles,
    /// ⭐⭐⭐ **Congela a BASE PERSISTENTE do pincel de tecido** nas posições de
    /// agora (espec §6.4, o operador *Set Persistent Base*).
    ///
    /// ⚠️⚠️ **A ORDEM é a lei:** para a opção morder, a base tem de ser gravada
    /// **ANTES** do traço que a há-de contradizer. Gravá-la DEPOIS de um traço é
    /// um **no-op exacto** — ali ela É o repouso do traço seguinte. *É a
    /// experiência que ocorre primeiro a quem desenha o botão, e ela não mede
    /// nada.*
    ///
    /// ⚠️ Um comando e não um knob, pela mesma razão do [`Self::BakeAo`]: é um
    /// gesto que o artista PEDE sobre a forma que está a ver.
    SetClothPersistentBase,
    /// Mede quanto do céu cada vértice enxerga e instala o canal.
    ///
    /// ⚠️ Um comando e não um knob: o bake custa ~338 ms na malha da cena `=16`,
    /// então ele é um gesto que o artista PEDE, nunca um passe que roda sozinho.
    BakeAo,
    /// **Assa a FORMA no sprite selecionado** — o objetivo 2 (`docs/3D/02.2`).
    ///
    /// ⚠️ **É o único intent que a cena 3D não sabe executar**, e isso é o
    /// desenho: o bake precisa do mundo, do renderizador e do mapa de atlas, e os
    /// três só existem dentro do laço de frame. Ele ARMA um pedido e sai — o
    /// mesmo caminho que o `Shift+B` já usava, e é por passarem pela MESMA porta
    /// que o botão e o atalho não podem divergir.
    BakeToSprite,
    /// **Usar o sprite selecionado como padrão** — o alpha por IMAGEM.
    ///
    /// ⚠️ **Ele ARMA e sai, pelo mesmo motivo do [`Self::BakeToSprite`] logo
    /// acima:** ler os pixels de um sprite precisa do mundo, do renderizador e
    /// do mapa de atlas, e os três só existem dentro do laço de frame. O painel
    /// não sabe o que é um atlas — e não deve saber.
    AlphaFromSprite,
    /// As quatro primitivas, na ordem em que o painel as lista.
    ///
    /// ⚠️ **Um comando por forma, e não um enum espelho do `Primitive` do
    /// shell.** Um enum duplicado aqui concordaria com o de lá exatamente até
    /// alguém acrescentar a quinta forma num só dos dois; um comando novo não
    /// compila sem que o painel também ganhe o botão dela, que é a ordem certa.
    AddSphere,
    AddCube,
    AddCylinder,
    AddTorus,
    Duplicate,
    Delete,
    ToggleIsolate,
    Merge,
    MaskClear,
    MaskInvert,
    MaskBlur,
    MaskSharpen,
    /// **Recorta a região mascarada numa peça nova** — ver
    /// [`ph2d_mesh::extract_masked`].
    ///
    /// ⚠️ Um comando, e não um `SetUi`: extrair não é ajustar um número, é uma
    /// peça que passa a EXISTIR. Os dois knobs que ele lê viajam no
    /// [`Sculpt3dUi::extract`], que é estado autorado — o comando só diz
    /// *agora*.
    Extract,
}
