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
//! alguma coisa?** Corrida item a item, **as 14** caíram do outro lado, por quatro mecanismos —
//! e o helper `v` (*«o que o `+` oferece»*) foi **apagado**, porque deixou de ter quem o use:
//!
//! | mecanismo | quem |
//! |---|---|
//! | o gesto **SEMEIA do contexto** que a paleta não tem | `VecContour` (o `d` sai da escala da selecção) |
//! | precisa dos **DOIS lados** e o neutro prende a nada — a forma do `PhysicsJoint` | `VecPatternPath` · `VecWidgetBind` |
//! | é **derivado** de uma lista/knob do painel, e o neutro é o que ele REMOVE | `VecFilter` · `VecLayoutItem` · `VecLayoutSize` · `VecLayoutAbsolute` · `VecPatternRotation` · `VecWidgetValue` |
//! | é adoptado no **NASCIMENTO** do traço, e pendurá-lo depois contradiz o produto | `VecSymmetry` (ordem do dono: *«não deve fazer simetria de formas que já existem previamente»*) · `VecCutPath` (⛔ e armá-lo **mata a lâmina anterior**: não é inerte, é destrutivo) |
//!
//! ⛔⛔ **E TRÊS delas quase escaparam, por um furo na RÉGUA — não no produto.** A 1.ª passagem
//! concluiu que `VecBindings`, `VecLayout` e `VecStrokeProfile` *«não têm um único escritor de
//! produção»* e deixou-as na paleta. A varredura procurava `insert(NomeDoTipo` e **as três são
//! escritas por variável** — `em.insert(b)` · `em.insert(l)` · `em.insert(v.clone())` —, cada uma
//! a duas linhas de um `remove::<…>` que a régua **via**. ⇒ *quando um censo diz «ninguém escreve
//! isto» sobre um componente que alguém REMOVE, o defeito é do censo*: o idioma da presença tem
//! sempre as duas metades, e ver só uma delas é a assinatura do furo.
//!
//! ⚠️ **A intenção declarada mais acima («a paleta é a superfície de descoberta») fica sem objecto
//! aqui, e isso é o achado e não um efeito colateral:** ela supunha que havia componentes do vetor
//! sem porta própria. Medido, não há **nenhum** — quem desenha alcança os catorze pelo painel do
//! vetor, que é onde ele trabalha.
//!
//! ⚠️ **Dois helpers intrínsecos, e a diferença é load-bearing:** [`g`] é *não há neutro*
//! (o tipo não tem `Default`); [`i`] é *há neutro e ele é o que o artista não quer*.
use crate::{ComponentCategory as C, ComponentDesc as D};

/// Um `Vec*` que chega com o GESTO — **não tem `Default`**, e a lista abaixo não foi
/// escolhida: ela é a saída do compilador ao converter os registradores para
/// `register_default` (`the trait bound X: Default is not satisfied`). Para estes não há
/// neutro que signifique alguma coisa — uma `VecShape` sem geometria não é uma forma vazia,
/// não é uma forma.
const fn g(canonical_name: &'static str, display_key: &'static str) -> D {
    D::intrinsic(canonical_name, display_key, C::Vector, &[])
}

/// ⭐ **Uma ROW ou um GESTO da secção, não um item de paleta** (varredura de 2026-09-14 — ver o
/// cabeçalho). Irmão do [`g`] pelo lado OPOSTO: ali **não há neutro** que signifique alguma coisa;
/// aqui há, e ele é exactamente o que o artista **não** quer — porque o que falta à paleta não é o
/// `Default`, é o **CONTEXTO** (a escala da selecção, qual dos dois escolhidos é o guia, que o eixo
/// de simetria se captura no nascimento do traço).
const fn i(canonical_name: &'static str, display_key: &'static str) -> D {
    D::intrinsic(canonical_name, display_key, C::Vector, &[])
}

/// Ordenado por `canonical_name` (gate `the_catalog_is_sorted_and_unique`).
pub const DESCS: &[D] = &[
    g("ph2d::ecs::VecAnchors", "component.vec_anchors.name"),
    // ⇒ as rows de TOKEN do painel escrevem-no (`bindings::set_selected_binding`), com o idioma da presença: vazio ⇒ `remove`.
    i("ph2d::ecs::VecBindings", "component.vec_bindings.name"),
    g("ph2d::ecs::VecBlend", "component.vec_blend.name"),
    g("ph2d::ecs::VecBoolGroup", "component.vec_bool_group.name"),
    g("ph2d::ecs::VecBoolOp", "component.vec_bool_op.name"),
    g("ph2d::ecs::VecBucketFill", "component.vec_bucket_fill.name"),
    g(
        "ph2d::ecs::VecClipContent",
        "component.vec_clip_content.name",
    ),
    g("ph2d::ecs::VecConnector", "component.vec_connector.name"),
    // ⇒ o botão *Add* do painel SEMEIA o `d` da escala da selecção (`contour_live::arm`: `DEFAULT_D_FRAC * scale`) — o neutro é `d = 0`, anéis invisíveis. É a razão do `Collider` da física, sem o seed que lá existe.
    i("ph2d::ecs::VecContour", "component.vec_contour.name"),
    // ⇒ é o marcador da LÂMINA, posto no NASCIMENTO do traço (`cut_line::upkeep`), e armá-lo MATA a lâmina anterior — pendurá-lo à mão não é inerte, é destrutivo.
    i("ph2d::ecs::VecCutPath", "component.vec_cut_path.name"),
    g("ph2d::ecs::VecEnvelope", "component.vec_envelope.name"),
    // ⇒ é DERIVADO da lista de filtros do painel (`ui_state_edit`: lista vazia ⇒ `remove`) — um `default()` vazio é removido na primeira edição.
    i("ph2d::ecs::VecFilter", "component.vec_filter.name"),
    g("ph2d::ecs::VecFrame", "component.vec_frame.name"),
    // ⚠️ `VecLabel.host` é um `VecPathId` cru (correção de 2026-08-21 ao doc 01 §1.3: NÃO é
    // um hash de nome) ⇒ `RefKind::VecPath` quando o campo for descrito, e entra no remap da
    // F4 como as juntas da física.
    g("ph2d::ecs::VecLabel", "component.vec_label.name"),
    // ⇒ o controlo de Auto Layout do painel escreve-o (`vec_layout_edit::write_layout`: `Some` ⇒ `insert`, `None` ⇒ `remove`).
    i("ph2d::ecs::VecLayout", "component.vec_layout.name"),
    // ⇒ a caixa *Absolute* do painel de layout: presença = bandeira, e ela alterna (`vec_layout_edit`).
    i(
        "ph2d::ecs::VecLayoutAbsolute",
        "component.vec_layout_absolute.name",
    ),
    // ⇒ row do painel de layout, com o idioma da presença escrito ao lado dela: *«o neutro DESTACA: um componente que não faz nada não viaja no arquivo»* — a paleta escreveria exactamente o neutro que o painel remove.
    i("ph2d::ecs::VecLayoutItem", "component.vec_layout_item.name"),
    // ⇒ idem, o mesmo `if next == default() { remove }` do irmão.
    i("ph2d::ecs::VecLayoutSize", "component.vec_layout_size.name"),
    g("ph2d::ecs::VecMorph", "component.vec_morph.name"),
    g(
        "ph2d::ecs::VecMorphMachine",
        "component.vec_morph_machine.name",
    ),
    g("ph2d::ecs::VecOffset", "component.vec_offset.name"),
    // ⇒ precisa dos DOIS lados (motivo + guia) e a porta é `pattern_live::link`, que os recebe resolvidos; o `default()` é um padrão preso a caminho nenhum — a forma do `PhysicsJoint`.
    i(
        "ph2d::ecs::VecPatternPath",
        "component.vec_pattern_path.name",
    ),
    // ⇒ é um knob do padrão acima; sem ele não tem sujeito.
    i(
        "ph2d::ecs::VecPatternRotation",
        "component.vec_pattern_rotation.name",
    ),
    g("ph2d::ecs::VecResizeBox", "component.vec_resize_box.name"),
    g("ph2d::ecs::VecShape", "component.vec_shape.name"),
    // ⇒ os QUATRO sliders de largura e as alças de canvas passam os dois pela MESMA porta (`profile_live::arm`), que o anexa com os stops e o **destaca** no neutro — anexá-lo pela paleta dá um perfil que o painel apaga no toque seguinte.
    i(
        "ph2d::ecs::VecStrokeProfile",
        "component.vec_stroke_profile.name",
    ),
    // ⇒ é adoptado no NASCIMENTO do traço (o eixo de sessão é capturado em LOCAL), e a exigência do dono é explícita: *«não deve fazer simetria de formas que já existem previamente»*.
    i("ph2d::ecs::VecSymmetry", "component.vec_symmetry.name"),
    g("ph2d::ecs::VecTextPath", "component.vec_text_path.name"),
    g("ph2d::ecs::VecWidget", "component.vec_widget.name"),
    // ⇒ guarda o ALVO, e a porta (`widget_edit`) confere que este widget conduz e que o alvo não é widget; o `default()` prende ao alvo `0` — a forma do `PhysicsJoint`.
    i("ph2d::ecs::VecWidgetBind", "component.vec_widget_bind.name"),
    g("ph2d::ecs::VecWidgetIcon", "component.vec_widget_icon.name"),
    // ⇒ é escrito pelo próprio CONTROLO autorado quando o artista o move (`widget_value`), e a AUSÊNCIA dele significa *«onde quer que o controlo esteja»* — anexá-lo pela paleta grava uma posição que ninguém escolheu, que é pior do que o neutro.
    i(
        "ph2d::ecs::VecWidgetValue",
        "component.vec_widget_value.name",
    ),
];

#[cfg(test)]
mod tests {
    use super::DESCS;

    /// ⭐⭐ **O vetor não oferece NADA na paleta, e a lista vazia é a afirmação.**
    ///
    /// ⚠️⚠️ **Esta família é o caso mais puro do defeito**, e a causa estava escrita no cabeçalho
    /// deste ficheiro antes de alguém a ler como tal: a divisão entre oferecido e não-oferecido era
    /// *«a saída do compilador ao converter os registadores para `register_default`»* — ou seja,
    /// **quem classificou foi o `Default`**, não uma pergunta sobre o artista. A varredura de
    /// 2026-09-14 correu a pergunta a sério e **as catorze** caíram do outro lado: cada uma tem
    /// uma porta no painel do vetor ou no gesto que a cria.
    ///
    /// ⛔ **Três delas quase ficaram, por um furo na RÉGUA** (ver o cabeçalho): a varredura
    /// procurava `insert(NomeDoTipo` e elas são escritas por variável, a duas linhas de um
    /// `remove::<…>` que ela via. *Um censo que diz «ninguém escreve isto» sobre um componente que
    /// alguém REMOVE está a medir mal.*
    ///
    /// ⚠️ **A lista é a afirmação.** Uma entrada nova aqui responde: *o neutro deste tipo, pendurado
    /// numa forma que já existe, quer dizer alguma coisa?* Se o que falta à paleta é o CONTEXTO (a
    /// escala, o outro lado do par, o instante do nascimento), é `i` e esta lista não cresce.
    ///
    /// (Mutação: trocar o `i` do `VecSymmetry` por `D::authored(..., O::VECTOR, &[])` ⇒ RED.)
    #[test]
    fn the_vector_family_offers_only_what_has_no_other_door() {
        const PORTAS: [&str; 0] = [];
        let offered: Vec<&str> = DESCS
            .iter()
            .filter(|d| d.is_offered())
            .map(|d| d.canonical_name)
            .collect();
        assert_eq!(
            offered,
            PORTAS.to_vec(),
            "a paleta do VETOR tem de oferecer exatamente estas tres.\n\
             Antes de acrescentar a primeira: o NEUTRO dela, pendurado numa forma que ja' existe, \
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
