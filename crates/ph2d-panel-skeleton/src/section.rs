//! ⭐⭐⭐ **O CORPO do painel do ESQUELETO** — as linhas que ele desenha.
//!
//! O que ele oferece é o que o gesto do modo Osso **não** pode dar: prender formas ao esqueleto,
//! soltá-las, e os números de um osso.
//!
//! # Por que Keep Pose E Release
//!
//! É o mesmo par do Envelope, e pela mesma razão: prender sem soltar é **porta de mão única**. Os
//! dois soltam a forma e diferem numa pergunta — *qual geometria fica?* **Keep Pose** materializa a
//! deformada (o que o artista está a ver); **Release** devolve a autorada (o que ele desenhou).
//! Adivinhar qual dos dois ele quer é que não.
//!
//! ⚠️ Os dois só são **pintados** com uma forma presa na seleção (`state::skinned`, publicado pela
//! shell) — *um botão que só sabe recusar é pior que um botão ausente*. O **Bind** é pintado sempre:
//! ele age sobre a seleção, e recusar em voz alta é mais honesto que esconder a porta de entrada.
//!
//! # ⚠️ Funções LIVRES sobre o [`RowCtx`], e não métodos
//!
//! O vocabulário de linhas vive na `ph2d-editor-core` e o Rust não deixa uma crate acrescentar
//! métodos inerentes a um tipo de outra. *A regra do órfão é o que escolhe a forma aqui* — e ela
//! não custa nada: o `r` é o mesmo contexto que o painel de vetor passa aos dele.

use crate::state;
use ph2d_a11y::NodeId;
use ph2d_editor_core::ids;
use ph2d_editor_core::panel::RowCtx;
use ph2d_i18n::tr;

/// Seção **SKELETON** — prender ao esqueleto, as duas saídas, e o osso em foco.
/// ⭐⭐⭐ **ESTE SEGMENTO ESTÁ ACESO?** — a lei que o dono descreveu, numa porta.
///
/// ⛔⛔ **Ordem do dono, 2026-09-09:**
///
/// | estado | o que acende |
/// |---|---|
/// | *«se não há ossos no mundo»* (nada armado) | **nenhum**, até ele carregar em *Create* |
/// | *«ao seleccionar o osso»* | **Transform** (a shell arma-o na aresta do foco) |
///
/// ⚠️ **Ela é uma função e não duas comparações no meio da pintura** porque é a única coisa aqui
/// que um gate consegue observar: o estado *aceso* de um segmento **não vive no `WidgetStore`** —
/// ele é passado ao pintor a cada quadro. *Uma lei que só existe dentro de uma chamada de pintura
/// não tem como ser medida.*
///
/// ⚠️ **`None` é «nada armado», e não um terceiro estado do enum**: «armado» já é o modo Osso estar
/// na mão, e um terceiro valor diria a mesma coisa duas vezes.
#[must_use]
pub(crate) fn aceso(armado: Option<usize>, segmento: usize) -> bool {
    armado == Some(segmento)
}

pub(crate) fn body(r: &mut RowCtx, y: f32) -> f32 {
    // ⭐⭐⭐ **REVELAR-AO-FOCAR** (report do dono, 2026-09-08: *«selecionar o bone nem sempre
    // abre a secção de skeleton»*). ⚠️ **O pedido consome-se AQUI, antes da porta**: se o corpo
    // não é pintado neste quadro não há nada a revelar, e um pedido que sobrevive rolaria o
    // painel muito depois do gesto que o armou.
    //
    // ⚠️ **A porta que decidia se a SECÇÃO existia saiu daqui** (2026-09-09): quem decide se este
    // painel é pintado é a shell (`panel_visible`), pela mesma lei que a secção seguia — *um painel
    // que fala de algo que não existe é ruído*.
    // ⛔ **Sem cabeçalho de secção**, e a ausência é a decisão (2026-09-09): num painel PRÓPRIO o
    // título dele **é** o cabeçalho, e uma secção única lá dentro escreveria o nome duas vezes — e
    // daria uma dobra que colapsa um painel que já se fecha.
    let mut y = y;
    // ⭐⭐⭐ **CRIAR × TRANSFORMAR, no TOPO — e ele é a PORTA, não um refinamento.**
    //
    // ⛔⛔ **Ordem do dono, 2026-09-09:** o pill `Bone` saiu da fileira de modos do painel de vector
    // (*«melhor tirar de lá»*), e com ele foi-se a única porta para o modo. Estes dois segmentos
    // passaram a **trocar de modo E de verbo**.
    //
    // ⚠️ **Ele é pintado SEMPRE, e é aí que mora a regra que o dono descreveu:**
    //
    // | estado | o que acende |
    // |---|---|
    // | nenhum osso no mundo, painel aberto pelo menu | **nada**, até ele carregar em *Create* |
    // | um osso seleccionado | **Transform** (a shell arma-o na aresta do foco) |
    //
    // ⇒ o que acende sai de `state::bone_tool()`, que é `None` fora do modo Osso. *Um terceiro
    // estado do enum diria a mesma coisa duas vezes* — «armado» já é o modo estar na mão.
    //
    // ⚠️ Ele vem antes dos verbos porque **decide o que os outros controlos significam**: com
    // *Criar* o arrasto faz osso, com *Transformar* ele posa — e ler isso depois de já ter carregado
    // é tarde.
    let armado = state::bone_tool();
    {
        let rotulos = [
            tr("panel.vector.bone.create"),
            tr("panel.vector.bone.transform"),
        ];
        let acoes: [(NodeId, &str, bool); 2] = [
            (
                ph2d_tool_vector::ids::VECTOR_BONE_ACT_CREATE,
                rotulos[0],
                aceso(armado, 0),
            ),
            (
                ph2d_tool_vector::ids::VECTOR_BONE_ACT_TRANSFORM,
                rotulos[1],
                aceso(armado, 1),
            ),
        ];
        y = r.segmented(tr("panel.vector.bone.action"), &acoes, y);
    }
    // Tabela tipada como a do Blend e a do Envelope (HR-12): o `action_button` delega ao
    // `paint_button` canónico, que é quem costura o AccessKit — e nomear o [`ph2d_a11y::NodeId`]
    // aqui é o idioma que o gate `every_widget_file_wires_a11y` lê.
    let verbos: [(ph2d_a11y::NodeId, &str); 1] =
        [(ids::VECTOR_BONE_BIND, tr("panel.vector.bone.bind"))];
    for (id, label) in verbos {
        y = r.action_button(id, label, y);
    }
    // ⭐⭐⭐ **AS SAÍDAS APARECEM PARA AS DUAS MÍDIAS — o *Expand* só para UMA.**
    //
    // ⛔⛔ **O `Expand` não alcança uma imagem, e a razão é da MÍDIA e não da fiação:** ele troca o
    // desenho autorado pela geometria deformada de agora, e uma imagem **não tem geometria
    // autorada** — a malha é derivada da própria tinta, por quadro. Assar a deformação nos pixels é
    // **outra** operação, e ela não existe. ⇒ ele é **escondido**, que é o que a casa faz quando
    // pode (a alternativa — pintá-lo e recusar — deixa um controlo morto sob o dedo).
    let presa = state::skinned();
    if presa.alguma() {
        if presa.vector {
            y = r.action_button(ids::VECTOR_BONE_EXPAND, tr("panel.vector.bone.expand"), y);
        }
        y = r.action_button(ids::VECTOR_BONE_RELEASE, tr("panel.vector.bone.release"), y);
    }
    // ⛔⛔⛔ **A FILEIRA `Deform` SAIU** (ordem do dono, 2026-09-17: *«Pode apagar a secção
    // deform»*), e o que a matou foi uma MEDIÇÃO.
    //
    // Ela nasceu do report de 2026-09-10 (*«ao dobrar a articulação temos arestas retas na
    // imagem»*), quando a malha do bind era uma grelha uniforme e os pesos eram euclidianos. As
    // duas waves seguintes — a grelha **graduada pelas articulações** e os pesos do
    // **padrão-ouro** com a lei de Hermite — curaram a faceta na própria malha do bind, e ninguém
    // reconferiu a nota. Medido em 2026-09-17, em pixels de ECRÃ (a unidade do olho, e não o desvio
    // ao campo em pixels da ARTE, que é o que todas as réguas desta linha mediam): as duas leis
    // punham a tinta a **`0,04 px` na mediana e `0,34 px` no pior ponto**, e o dono reportou
    // (repetidamente) que eram *«sempre idênticas»*. Ele tinha razão.
    //
    // ⇒ o knob inteiro morreu — o enum, o campo do `VectorDrawConfig`, os dois ids, o espelho da
    // shell e esta fileira. ⛔ **Apagado e não escondido:** um controlo vivo e inalcançável é a
    // classe que o `CLAUDE.md` §5.0 nomeia, e o precedente é o `Ctrl` do *Scene Project* (15/09),
    // onde retirar o gesto retirou a capacidade.
    // Os dois números do OSSO em foco. Sem osso não há sujeito — e um campo sem sujeito é a
    // classe de controlo morto que o `CLAUDE.md` §5.0 nomeia.
    if let Some(campos) = campos_do_osso() {
        for (id, label, step, _valor) in campos {
            y = r.labeled_number_field(label, id, step, y);
        }
        y = handles_row(r, y);
        y = crate::section_tip::tip_row(r, y);
        y = limit_rows(r, y);
        y = crate::section_smart::smart_rows(r, y);
        y = ik_rows(r, y);
    }
    y
}

/// ⭐⭐⭐ **O LIMITE DE ÂNGULO** da junta em foco — a porta de entrada, ou os dois extremos.
///
/// ⚠️ **Vem ANTES da âncora**, e a ordem diz o que ele é: o limite é uma propriedade da JUNTA
/// (vale com IK e sem ela), a âncora é uma restrição que se põe e se tira. Pô-lo dentro do
/// bloco da IK ensinaria que ele é parte dela — que é exactamente o desenho do Godot, e o que
/// esta casa decidiu não fazer.
fn limit_rows(r: &mut RowCtx, y: f32) -> f32 {
    let Some(_) = state::current_bone_limit() else {
        return r.action_button(
            ids::VECTOR_BONE_LIMIT_ADD,
            tr("panel.vector.bone.limit.add"),
            y,
        );
    };
    let mut y = r.action_button(
        ids::VECTOR_BONE_LIMIT_REMOVE,
        tr("panel.vector.bone.limit.remove"),
        y,
    );
    let campos: [(ph2d_a11y::NodeId, &str); 2] = [
        (
            ids::VECTOR_BONE_LIMIT_MIN,
            tr("panel.vector.bone.limit.min"),
        ),
        (
            ids::VECTOR_BONE_LIMIT_MAX,
            tr("panel.vector.bone.limit.max"),
        ),
    ];
    for (id, label) in campos {
        y = r.labeled_number_field(label, id, ANGLE_STEP, y);
    }
    y
}

/// ⭐⭐⭐ **A ÂNCORA DE IK** do osso em foco — a porta de entrada, ou os três números dela.
///
/// ⚠️ **É um OU exclusivo, e é a lei do controlo morto:** *Add IK* só aparece em quem não tem
/// âncora, e *Remove IK* mais os três números só em quem tem. Oferecer as duas portas ao mesmo
/// tempo daria um botão que só sabe recusar — e o gesto recusa-o também, então o painel estaria
/// a prometer o que o app não faz.
fn ik_rows(r: &mut RowCtx, y: f32) -> f32 {
    let Some((_, _, _, lado)) = state::current_bone_ik() else {
        return r.action_button(ids::VECTOR_BONE_IK_ADD, tr("panel.vector.bone.ik.add"), y);
    };
    let mut y = r.action_button(
        ids::VECTOR_BONE_IK_REMOVE,
        tr("panel.vector.bone.ik.remove"),
        y,
    );
    for (id, label, step, _valor) in campos_da_ancora().unwrap_or_else(|| {
        unreachable!("a âncora existe: o `let Some(..) = current_bone_ik()` acima devolveu cedo")
    }) {
        y = r.labeled_number_field(label, id, step, y);
    }
    // ⭐⭐⭐ **PARA QUE LADO O JOELHO DOBRA** — e ele vem DEPOIS de `Chain` porque é o `Chain`
    // que decide quantos ossos têm lado: ler *«de que lado»* antes de saber *«de que corrente»*
    // é ler a resposta antes da pergunta.
    //
    // ⚠️ A fileira é construída a partir de [`ph2d_skeleton::BendSide::ALL`] e da tabela de ids
    // **ao mesmo tempo**, por índice: uma variante nova na lei sem um id ao lado é erro de
    // compilação (os dois arrays têm de ter o mesmo comprimento), em vez de um segmento que
    // desaparece em silêncio.
    // ⚠️⚠️ **A promessa de «erro de compilação» era FALSA até 2026-09-14**: as duas listas eram
    // indexadas lado a lado e um comprimento diferente dava **pânico em runtime**, apanhado só pelo
    // gate de costura. Este `assert!` de `const` torna-a verdadeira — e foi preciso acrescentar uma
    // variante (`Mixed`) para o descobrir.
    const _: () = assert!(
        ids::VECTOR_BONE_BEND_IDS.len() == ph2d_skeleton::BendSide::ALL.len(),
        "um lado novo na LEI precisa de um id ao lado dele"
    );
    let rotulos = [
        tr("panel.vector.bone.ik.bend.auto"),
        tr("panel.vector.bone.ik.bend.ccw"),
        tr("panel.vector.bone.ik.bend.cw"),
        tr("panel.vector.bone.ik.bend.mixed"),
    ];
    let mut lados: [(ph2d_a11y::NodeId, &str, bool); ph2d_skeleton::BendSide::ALL.len()] =
        [(ids::VECTOR_BONE_BEND_IDS[0], "", false); ph2d_skeleton::BendSide::ALL.len()];
    for (i, slot) in lados.iter_mut().enumerate() {
        *slot = (ids::VECTOR_BONE_BEND_IDS[i], rotulos[i], i == lado);
    }
    r.segmented(tr("panel.vector.bone.ik.bend"), &lados, y)
}

/// ⭐⭐⭐⭐ **DE ONDE VÊM AS DUAS ALÇAS DE CURVATURA** — o *Handle Type* do *Bendy Bone*.
///
/// ⚠️ **Só é pintada num osso que a sabe LER**, pela mesma régua e pela mesma razão que os quatro
/// números da curvatura: com um segmento só a curvatura é **provadamente inerte**, e um segmentado
/// que grava no documento sem mudar um pixel é o painel a mentir.
///
/// ⚠️ **Vem DEPOIS dos quatro números e ANTES do limite**, e a ordem diz o que ela é: ela decide
/// **quem escreve** aqueles quatro — ler *«quanto»* antes de *«quem»* é ler a resposta antes da
/// pergunta, que é a mesma ordem que o `Chain`/`Bend` da âncora já segue.
fn handles_row(r: &mut RowCtx, y: f32) -> f32 {
    let Some(osso) = state::current_bone() else {
        return y;
    };
    if osso.is_rigid() && ph2d_skeleton::bend::segments_of(osso.segments) <= 1 {
        return y;
    }
    // ⚠️ A MESMA lei de alinhamento por índice da fileira do lado da dobra, e o mesmo `assert!` de
    // `const`: um modo novo na lei sem um id ao lado **não compila**.
    const _: () = assert!(
        ids::VECTOR_BONE_HANDLES_IDS.len() == 2,
        "um modo novo de alca precisa de um id ao lado dele"
    );
    let auto = state::current_bone_handles() == Some(1);
    let modos: [(NodeId, &str, bool); 2] = [
        (
            ids::VECTOR_BONE_HANDLES_AUTHORED,
            tr("panel.vector.bone.handles.authored"),
            !auto,
        ),
        (
            ids::VECTOR_BONE_HANDLES_AUTO,
            tr("panel.vector.bone.handles.auto"),
            auto,
        ),
    ];
    r.segmented(tr("panel.vector.bone.handles"), &modos, y)
}

/// ⭐⭐⭐ **OS CAMPOS NUMÉRICOS DO OSSO — id, rótulo, passo e O VALOR QUE ELES MOSTRAM.**
///
/// ⛔⛔ **Os cinco campos desta secção nasciam com `0` e nada lá escrevia** (report do dono,
/// 2026-09-14: *«IK chain mostra 0 ao inserir IK»*). O `populate` regista-os com `value: 0.0`, o
/// publicador (`state::set_current_bone_ik`) existe e a shell chama-o todo quadro — e os valores
/// morriam no `state`, porque quem pinta a fileira ([`RowCtx::labeled_number_field`]) tira o valor
/// do **WidgetStore**. *Um publicador sem quem o leia e uma lei ausente produzem o mesmo painel.*
///
/// ⚠️ **`Chain = 0` não é um zero inócuo:** ele significa *«até à raiz»*, que é o default do Blender
/// e a queixa nº 1 documentada da feature. O painel dizia ao artista exactamente a coisa que o
/// modelo existe para não fazer — e um `Mix = 0` lido como verdade é a restrição **desligada**.
///
/// ⭐ **UMA tabela, DOIS consumidores:** quem pinta a fileira e quem semeia o valor no store
/// ([`crate::paint`]) percorrem esta MESMA lista. Um campo novo traz o valor no tuplo ou **não
/// compila** — em vez de nascer a mostrar `0` e ninguém dar por isso.
///
/// `None` = não há osso em foco, e então não se pinta nem se semeia nada.
pub(crate) fn campos_do_osso() -> Option<Vec<(ph2d_a11y::NodeId, &'static str, f64, f64)>> {
    let osso = state::current_bone()?;
    let mut campos = vec![
        (
            ids::VECTOR_BONE_LENGTH,
            tr("panel.vector.bone.length"),
            LENGTH_STEP,
            osso.length,
        ),
        (
            ids::VECTOR_BONE_STRENGTH,
            tr("panel.vector.bone.strength"),
            STRENGTH_STEP,
            osso.strength,
        ),
        (
            ids::VECTOR_BONE_SEGMENTS,
            tr("panel.vector.bone.segments"),
            SEGMENTS_STEP,
            f64::from(osso.segments),
        ),
    ];
    // ⭐⭐⭐ **A CURVATURA SÓ É PINTADA NUM OSSO QUE A SABE LER** (F8) — com um segmento só ela é
    // **provadamente inerte**, e há gate a dizê-lo pelo nome
    // (`one_segment_never_bends_whatever_the_handles_say`, em `ph2d-skeleton`).
    //
    // ⛔ **Mostrá-la sempre seria o painel a MENTIR:** quatro campos que aceitam teclas, gravam no
    // documento e não mudam um pixel. É a mesma lei que o L-System pagou — *nenhum molde mostra um
    // knob que a gramática dele não sabe ler* —, aqui com a régua a ser a lei e não uma varredura.
    // ⛔⛔ **E em `Auto` eles SOMEM**, porque ali eles não são autorados — são derivados da
    // corrente a cada quadro. *Quatro caixas que aceitam teclas e cujo valor o quadro seguinte
    // recalcula são a mesma mentira que este painel já pagou nos quatro números da âncora.*
    if (!osso.is_rigid() || ph2d_skeleton::bend::segments_of(osso.segments) > 1)
        && state::current_bone_handles() != Some(1)
    {
        for (id, chave, valor) in [
            (
                ids::VECTOR_BONE_CURVE_IN_Y,
                "panel.vector.bone.curve.in.y",
                osso.curve.inn[1],
            ),
            (
                ids::VECTOR_BONE_CURVE_OUT_Y,
                "panel.vector.bone.curve.out.y",
                osso.curve.out[1],
            ),
            (
                ids::VECTOR_BONE_CURVE_IN_X,
                "panel.vector.bone.curve.in.x",
                osso.curve.inn[0],
            ),
            (
                ids::VECTOR_BONE_CURVE_OUT_X,
                "panel.vector.bone.curve.out.x",
                osso.curve.out[0],
            ),
        ] {
            campos.push((id, tr(chave), CURVE_STEP, valor));
        }
    }
    Some(campos)
}

/// Os três números da ÂNCORA — ver [`campos_do_osso`]. `None` = o osso não tem restrição.
pub(crate) fn campos_da_ancora() -> Option<[(ph2d_a11y::NodeId, &'static str, f64, f64); 3]> {
    let (mix, softness, chain, _) = state::current_bone_ik()?;
    Some([
        (
            ids::VECTOR_BONE_IK_MIX,
            tr("panel.vector.bone.ik.mix"),
            MIX_STEP,
            mix,
        ),
        (
            ids::VECTOR_BONE_IK_SOFTNESS,
            tr("panel.vector.bone.ik.softness"),
            MIX_STEP,
            softness,
        ),
        (
            ids::VECTOR_BONE_IK_CHAIN,
            tr("panel.vector.bone.ik.chain"),
            CHAIN_STEP,
            chain,
        ),
    ])
}

/// Passo do campo de comprimento, no domínio do DOCUMENTO (unidades de mundo).
const LENGTH_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo do campo de força — ela é um **múltiplo do comprimento do osso**, então a escala útil é
/// a unidade, e o passo é o décimo dela.
const STRENGTH_STEP: f64 = 0.1; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo do campo dos SEGMENTOS — eles são uma CONTAGEM, então o passo é a unidade.
const SEGMENTS_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo das quatro alças de curvatura — elas são **múltiplos do comprimento do osso**, como a
/// força, então a escala útil é a unidade e o passo é o décimo dela.
const CURVE_STEP: f64 = 0.1; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo dos dois números adimensionais da âncora (`Mix` e `Softness`), que vivem em `0..1`: o
/// décimo da unidade, como o da força do osso.
const MIX_STEP: f64 = 0.1; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo dos dois extremos do limite, em **graus**: cinco de cada vez.
///
/// ⚠️ O campo fala GRAUS e o documento guarda radianos — a conversão vive na shell. Um passo de
/// `0,0873` (um grau em radianos) neste campo seria o número certo na unidade errada.
pub(crate) const ANGLE_STEP: f64 = 5.0; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo da CORRENTE — ela conta ossos, então o passo é **um osso**.
const CHAIN_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

#[cfg(test)]
mod tests {
    use super::aceso;

    /// ⭐⭐⭐ **NADA ACESO ATÉ O ARTISTA CARREGAR** — a ordem do dono, palavra por palavra.
    ///
    /// ⛔ *«Se não há ossos no mundo nenhum botão fica selecionado até o usuário apertar Create»*
    /// (2026-09-09). ⚠️ **A metade que importa é a PRIMEIRA:** um default aceso faria o painel
    /// afirmar um verbo que a ferramenta não tem armado — e o arrasto no canvas faria outra coisa
    /// que a fileira diz.
    #[test]
    fn nothing_is_lit_until_the_artist_arms_it() {
        assert!(
            !aceso(None, 0) && !aceso(None, 1),
            "um segmento nasceu aceso sem nada armado"
        );
        assert!(aceso(Some(0), 0) && !aceso(Some(0), 1), "*Create* armado");
        assert!(
            !aceso(Some(1), 0) && aceso(Some(1), 1),
            "*Transform* armado"
        );
    }
}
