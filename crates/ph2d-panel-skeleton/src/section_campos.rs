//! ⭐⭐⭐ **A TABELA DOS CAMPOS NUMÉRICOS do painel do esqueleto** — o que ele oferece como
//! NÚMERO, e em que passo.
//!
//! # Por que ela é um ficheiro e não parte do pintor
//!
//! O [`crate::section`] responde *«que linhas eu desenho»*; isto responde *«que números existem,
//! com que rótulo, que passo e que valor»* — e a diferença tem consequência: esta tabela tem
//! **DOIS** consumidores que não se conhecem (quem pinta a fileira e o [`crate::paint`], que
//! semeia o valor no `WidgetStore`), enquanto o pintor tem um só. *Um campo novo traz o valor no
//! tuplo ou **não compila**, em vez de nascer a mostrar `0` e ninguém dar por isso.*
//!
//! ⚠️ **Os passos vivem AQUI com as tabelas que os usam**, e não no pintor: eles são o domínio do
//! DOCUMENTO (unidades de mundo, contagens, graus) e nunca medida de design — é isso que a marca
//! `LITERAL-PX-OK` de cada um declara.

use crate::state;
use ph2d_editor_core::ids;
use ph2d_i18n::tr;

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
            ids::VECTOR_BONE_SEGMENTS,
            tr("panel.vector.bone.segments"),
            SEGMENTS_STEP,
            f64::from(osso.segments),
        ),
    ];
    // ⭐⭐⭐ **O ENVELOPE SÓ É PINTADO ONDE AINDA MANDA** (ordem do dono, 2026-09-18, a seguir à
    // pergunta dele: *«Por que o envelope já não influencia na deformação?»*).
    //
    // ⛔⛔ Com os pesos do **padrão-ouro** uma imagem deforma **igual** a `1` e a `2` — está medido,
    // coluna a coluna. Num rig só de imagens este campo aceita teclas, grava no documento e **não
    // muda um pixel**: é a mesma mentira que as quatro alças de curvatura pagaram logo abaixo.
    // ⚠️ **E ele VOLTA assim que houver uma forma vectorial presa**, onde a lei euclidiana manda
    // como sempre — o envelope não morreu, mudou de dono.
    if state::envelope_manda() {
        campos.insert(
            1,
            (
                ids::VECTOR_BONE_STRENGTH,
                tr("panel.vector.bone.strength"),
                STRENGTH_STEP,
                osso.strength,
            ),
        );
    }
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
pub(crate) fn campos_da_ancora() -> Option<Vec<(ph2d_a11y::NodeId, &'static str, f64, f64)>> {
    let (mix, softness, chain, _) = state::current_bone_ik()?;
    // ⭐⭐⭐ **A LENTE DO APONTAR, e ela é MEDIDA e não escolhida**
    // (`ph2d_app_skeleton::goal::sonda_do_apontar_tests`): com a corrente resolvida em UM, a
    // *Softness* move o osso **zero** e o desvio é o único número novo que ele lê. ⛔ Pintar a
    // *Softness* ali seria a classe de controlo morto que o `CLAUDE.md` §5.0 nomeia.
    let aponta = state::current_bone_aim();
    let mut campos = vec![(
        ids::VECTOR_BONE_IK_MIX,
        tr("panel.vector.bone.ik.mix"),
        MIX_STEP,
        mix,
    )];
    if aponta.is_none() {
        campos.push((
            ids::VECTOR_BONE_IK_SOFTNESS,
            tr("panel.vector.bone.ik.softness"),
            MIX_STEP,
            softness,
        ));
    }
    campos.push((
        ids::VECTOR_BONE_IK_CHAIN,
        tr("panel.vector.bone.ik.chain"),
        CHAIN_STEP,
        chain,
    ));
    // ⚠️ **Depois da corrente**, e a ordem diz o que ele é: o desvio só existe porque a corrente
    // resolve a UM — ler *«de quanto está rodado»* antes de saber *«de que corrente»* é ler a
    // resposta antes da pergunta (a mesma lei que já põe o lado da dobra depois do `Chain`).
    if let Some(graus) = aponta {
        campos.push((
            ids::VECTOR_BONE_IK_OFFSET,
            tr("panel.vector.bone.ik.offset"),
            OFFSET_STEP,
            graus,
        ));
    }
    Some(campos)
}

/// Passo do campo de comprimento, no domínio do DOCUMENTO (unidades de mundo).
pub(crate) const LENGTH_STEP: f64 = 1.0; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

/// Passo do campo de força — ela é um **múltiplo do comprimento do osso**, então a escala útil é
/// a unidade, e o passo é o décimo dela.
pub(crate) const STRENGTH_STEP: f64 = 0.1; // LITERAL-PX-OK: passo no domínio do documento, não medida de design

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

/// Passo do DESVIO do apontar, em **graus** — o mesmo do limite da junta, pela mesma razão: os dois
/// são arcos que o artista lê em graus, e o documento guarda-os em radianos.
const OFFSET_STEP: f64 = ANGLE_STEP;
