//! **A família do vetor** — os 30 `Vec*` registados.
//!
//! ⚠️ **O OSSO e a PELE saíram daqui em 2026-09-06** (-2) para a família [`super::skeleton`],
//! quando o esqueleto virou módulo próprio: ele serve raster, 3D e Flip além do vetor, e
//! declará-lo aqui prometeria ao artista que ele só serve caminhos.
//!
//! ⚠️ **Nenhum destes é hoje alcançável pelo Inspector**: a auditoria de 2026-08-21 mediu que
//! 31 dos 36 tipos ausentes do Inspector são desta família, editados por **outro painel
//! artesanal** (`ph2d-panel-vector`). Declará-los aqui não os põe no Inspector — põe-nos na
//! paleta do `+`, que é a pergunta *"que componentes existem para este objeto?"*, e essa
//! pergunta tinha 31 respostas invisíveis.
//!
//! ⚠️ **`applies_to` é `VECTOR` para todos, e isto é uma afirmação a MEDIR no smoke**, não uma
//! medição: o critério do ADR-0166 é *"o tipo cujo marcador o componente lê"*, e o marcador
//! aqui é o `VecPathRef`. O que não está provado é o inverso — se algum destes tem efeito
//! sobre um objeto que não é um caminho vetorial. Onde isso aparecer, corrija a linha e
//! escreva a razão.
//!
//! ⚠️ **`VecComponentMain`/`VecInstance` são a instância VETORIAL de hoje, e a F4 subsume-os**
//! (ADR-0164 §4): quando o mecanismo geral existir, estas duas linhas saem daqui e o que fica
//! é `ObjectInstance` na família `Instancing`. Ficam declaradas `Machinery` — o artista já as
//! opera por verbos (*Create/Place/Detach*), nunca anexando o componente.

//!
//! # ⭐⭐⭐ A RÉGUA DO DONO, corrida sobre esta família (2026-09-14) — e o que ela achou aqui
//!
//! ⚠️⚠️ **Quem classificava esta família era o COMPILADOR, e estava escrito aqui.** O doc do helper
//! [`g`] di-lo com todas as letras: *«a lista abaixo não foi escolhida: ela é a saída do compilador
//! ao converter os registadores para `register_default`»*. Ou seja, a fronteira entre *oferecido* e
//! *não oferecido* era **implementa `Default`?** — uma propriedade do tipo, não uma pergunta sobre
//! o artista. É a forma pura do defeito que a [`super::physics`] pagou em 13/09 (*o caminho de
//! menor esforço escolheu a resposta*), e aqui a causa estava confessada no próprio ficheiro.
//!
//! A pergunta a sério é: **o neutro deste tipo, pendurado numa forma que JÁ EXISTE, quer dizer
//! alguma coisa?** Corrida item a item, **11 de 14** caíram do outro lado, e por quatro mecanismos:
//!
//! | mecanismo | quem |
//! |---|---|
//! | o gesto **SEMEIA do contexto** que a paleta não tem | `VecContour` (o `d` sai da escala da selecção) |
//! | precisa dos **DOIS lados** e o neutro prende a nada — a forma do `PhysicsJoint` | `VecPatternPath` · `VecWidgetBind` |
//! | é **derivado** de uma lista/knob do painel, e o neutro é o que ele REMOVE | `VecFilter` · `VecLayoutItem` · `VecLayoutSize` · `VecLayoutAbsolute` · `VecPatternRotation` · `VecWidgetValue` |
//! | é adoptado no **NASCIMENTO** do traço, e pendurá-lo depois contradiz o produto | `VecSymmetry` (ordem do dono: *«não deve fazer simetria de formas que já existem previamente»*) · `VecCutPath` (⛔ e armá-lo **mata a lâmina anterior**: não é inerte, é destrutivo) |
//!
//! ⭐ **E as TRÊS que ficam, ficam por medição, não por inércia:** `VecBindings`, `VecLayout` e
//! `VecStrokeProfile` não têm **um único** escritor de produção (só `remove`) ⇒ a paleta é
//! literalmente a única porta delas. *O mesmo instrumento que manda podar onze manda NÃO podar
//! três* — e é isso que responde à intenção declarada mais acima («a paleta é a superfície de
//! descoberta»): ela continua a sê-lo, exactamente para as que não têm outra.
//!
//! ⚠️ **Dois helpers intrínsecos, e a diferença é load-bearing:** [`g`] é *não há neutro*
//! (o tipo não tem `Default`); [`i`] é *há neutro e ele é o que o artista não quer*.
use crate::{ComponentCategory as C, ComponentDesc as D, ObjectKinds as O};

/// Um `Vec*` que o `+` OFERECE: tem `Default`, logo a paleta consegue construí-lo no ponto
/// neutro. Sempre `Vector`, sempre sobre um caminho, ainda sem campos descritos.
const fn v(canonical_name: &'static str, display_name: &'static str) -> D {
    D::authored(canonical_name, display_name, C::Vector, O::VECTOR, &[])
}

/// Um `Vec*` que chega com o GESTO — **não tem `Default`**, e a lista abaixo não foi
/// escolhida: ela é a saída do compilador ao converter os registradores para
/// `register_default` (`the trait bound X: Default is not satisfied`). Para estes não há
/// neutro que signifique alguma coisa — uma `VecShape` sem geometria não é uma forma vazia,
/// não é uma forma.
const fn g(canonical_name: &'static str, display_name: &'static str) -> D {
    D::intrinsic(canonical_name, display_name, C::Vector, &[])
}

/// ⭐ **Uma ROW ou um GESTO da secção, não um item de paleta** (varredura de 2026-09-14 — ver o
/// cabeçalho). Irmão do [`g`] pelo lado OPOSTO: ali **não há neutro** que signifique alguma coisa;
/// aqui há, e ele é exactamente o que o artista **não** quer — porque o que falta à paleta não é o
/// `Default`, é o **CONTEXTO** (a escala da selecção, qual dos dois escolhidos é o guia, que o eixo
/// de simetria se captura no nascimento do traço).
const fn i(canonical_name: &'static str, display_name: &'static str) -> D {
    D::intrinsic(canonical_name, display_name, C::Vector, &[])
}

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    g("ph2d::ecs::VecAnchors", "Anchors"),
    v("ph2d::ecs::VecBindings", "Bindings"),
    g("ph2d::ecs::VecBlend", "Blend"),
    g("ph2d::ecs::VecBoolGroup", "Boolean Group"),
    g("ph2d::ecs::VecBoolOp", "Boolean Op"),
    g("ph2d::ecs::VecBucketFill", "Bucket Fill"),
    g("ph2d::ecs::VecClipContent", "Clip Content"),
    g("ph2d::ecs::VecConnector", "Connector"),
    // ⇒ o botão *Add* do painel SEMEIA o `d` da escala da selecção (`contour_live::arm`: `DEFAULT_D_FRAC * scale`) — o neutro é `d = 0`, anéis invisíveis. É a razão do `Collider` da física, sem o seed que lá existe.
    i("ph2d::ecs::VecContour", "Contour"),
    // ⇒ é o marcador da LÂMINA, posto no NASCIMENTO do traço (`cut_line::upkeep`), e armá-lo MATA a lâmina anterior — pendurá-lo à mão não é inerte, é destrutivo.
    i("ph2d::ecs::VecCutPath", "Cut Path"),
    g("ph2d::ecs::VecEnvelope", "Envelope"),
    // ⇒ é DERIVADO da lista de filtros do painel (`ui_state_edit`: lista vazia ⇒ `remove`) — um `default()` vazio é removido na primeira edição.
    i("ph2d::ecs::VecFilter", "Filter"),
    g("ph2d::ecs::VecFrame", "Frame"),
    // ⚠️ `VecLabel.host` é um `VecPathId` cru (correção de 2026-08-21 ao doc 01 §1.3: NÃO é
    // um hash de nome) ⇒ `RefKind::VecPath` quando o campo for descrito, e entra no remap da
    // F4 como as juntas da física.
    g("ph2d::ecs::VecLabel", "Label"),
    v("ph2d::ecs::VecLayout", "Auto Layout"),
    // ⇒ a caixa *Absolute* do painel de layout: presença = bandeira, e ela alterna (`vec_layout_edit`).
    i("ph2d::ecs::VecLayoutAbsolute", "Layout Absolute"),
    // ⇒ row do painel de layout, com o idioma da presença escrito ao lado dela: *«o neutro DESTACA: um componente que não faz nada não viaja no arquivo»* — a paleta escreveria exactamente o neutro que o painel remove.
    i("ph2d::ecs::VecLayoutItem", "Layout Item"),
    // ⇒ idem, o mesmo `if next == default() { remove }` do irmão.
    i("ph2d::ecs::VecLayoutSize", "Layout Size"),
    g("ph2d::ecs::VecMorph", "Morph"),
    g("ph2d::ecs::VecMorphMachine", "Morph States"),
    g("ph2d::ecs::VecOffset", "Offset"),
    // ⇒ precisa dos DOIS lados (motivo + guia) e a porta é `pattern_live::link`, que os recebe resolvidos; o `default()` é um padrão preso a caminho nenhum — a forma do `PhysicsJoint`.
    i("ph2d::ecs::VecPatternPath", "Pattern Path"),
    // ⇒ é um knob do padrão acima; sem ele não tem sujeito.
    i("ph2d::ecs::VecPatternRotation", "Pattern Rotation"),
    g("ph2d::ecs::VecResizeBox", "Resize Box"),
    g("ph2d::ecs::VecShape", "Shape"),
    v("ph2d::ecs::VecStrokeProfile", "Stroke Profile"),
    // ⇒ é adoptado no NASCIMENTO do traço (o eixo de sessão é capturado em LOCAL), e a exigência do dono é explícita: *«não deve fazer simetria de formas que já existem previamente»*.
    i("ph2d::ecs::VecSymmetry", "Symmetry"),
    g("ph2d::ecs::VecTextPath", "Text on Path"),
    g("ph2d::ecs::VecWidget", "Widget"),
    // ⇒ guarda o ALVO, e a porta (`widget_edit`) confere que este widget conduz e que o alvo não é widget; o `default()` prende ao alvo `0` — a forma do `PhysicsJoint`.
    i("ph2d::ecs::VecWidgetBind", "Widget Bind"),
    g("ph2d::ecs::VecWidgetIcon", "Widget Icon"),
    // ⇒ é escrito pelo próprio CONTROLO autorado quando o artista o move (`widget_value`), e a AUSÊNCIA dele significa *«onde quer que o controlo esteja»* — anexá-lo pela paleta grava uma posição que ninguém escolheu, que é pior do que o neutro.
    i("ph2d::ecs::VecWidgetValue", "Widget Value"),
];

#[cfg(test)]
mod tests {
    use super::DESCS;

    /// ⭐⭐ **O vetor oferece as três que NÃO têm outra porta — e mais nenhuma.**
    ///
    /// ⚠️⚠️ **Esta família é o caso mais puro do defeito**, e a causa estava escrita no cabeçalho
    /// deste ficheiro antes de alguém a ler como tal: a divisão entre oferecido e não-oferecido era
    /// *«a saída do compilador ao converter os registadores para `register_default`»* — ou seja,
    /// **quem classificou foi o `Default`**, não uma pergunta sobre o artista. A varredura de
    /// 2026-09-14 correu a pergunta a sério e **11 das 14** caíram do outro lado.
    ///
    /// ⭐ **E as três que ficam ficam por MEDIÇÃO:** `VecBindings`, `VecLayout` e
    /// `VecStrokeProfile` não têm **um único** escritor de produção (só `remove`) — a paleta é
    /// literalmente a única porta delas, e podá-las tornaria as features inalcançáveis. *É o mesmo
    /// instrumento a dizer «tira» a onze e «não tires» a três.*
    ///
    /// ⚠️ **A lista é a afirmação.** Uma entrada nova aqui responde: *o neutro deste tipo, pendurado
    /// numa forma que já existe, quer dizer alguma coisa?* Se o que falta à paleta é o CONTEXTO (a
    /// escala, o outro lado do par, o instante do nascimento), é `i` e esta lista não cresce.
    ///
    /// (Mutação: devolver o `VecSymmetry` a `v` ⇒ RED, nomeando-o.)
    #[test]
    fn the_vector_family_offers_only_what_has_no_other_door() {
        const PORTAS: [&str; 3] = [
            "ph2d::ecs::VecBindings",
            "ph2d::ecs::VecLayout",
            "ph2d::ecs::VecStrokeProfile",
        ];
        let offered: Vec<&str> = DESCS
            .iter()
            .filter(|d| d.is_offered())
            .map(|d| d.canonical_name)
            .collect();
        assert_eq!(
            offered,
            PORTAS.to_vec(),
            "a paleta do VETOR tem de oferecer exatamente estas tres.\n\
             Antes de acrescentar uma quarta: o NEUTRO dela, pendurado numa forma que ja' existe, \
             quer dizer alguma coisa? Se o que falta a' paleta e' o CONTEXTO — a escala da \
             seleccao, qual dos dois e' o guia, o instante do nascimento do traco —, entao a porta \
             e' o gesto e o helper e' o `i`."
        );
        assert!(
            DESCS.len() >= 28,
            "a familia encolheu para {} — o gate acima deixaria de afirmar",
            DESCS.len()
        );
    }
}
