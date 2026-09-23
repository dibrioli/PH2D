//! **AS FILEIRAS PRÓPRIAS DE CADA PINCEL** — os chips que só existem com um
//! verbo na mão: o contorno, a densidade, a pose e o tecido.
//!
//! Irmão (`#[path]`-menos) do [`super::brush`], e o corte é de
//! RESPONSABILIDADE: lá mora *a moldura do pincel* — a linha do nível, a cauda
//! partilhada, os interruptores por verbo —, aqui *o vocabulário PRÓPRIO de
//! cada ferramenta*, que é a lista que cresce um bloco a cada pincel novo. Foi
//! ela que levou o ficheiro ao tecto de 600 LOC do `panel_files_under_loc_cap`
//! quando a fileira da densidade chegou (2026-09-14).
//!
//! ⛔ **Curado por CORTE e nunca por uma entrada no `FILE_OVERAGE_OK`** — a lei
//! do §5.0 do roteador, que esta casa já pagou em cinco painéis.
//!
//! ⚠️ **Nenhum pixel muda de sítio:** as quatro funções são chamadas na mesma
//! ordem, do mesmo sítio, com os mesmos argumentos. O que muda é onde a lista
//! cresce.

use ph2d_editor_core::panel::PaintCtx;
use ph2d_i18n::tr;
use ph2d_sculpt3d::{
    ClothArea, ClothForceFalloff, ClothMode, ProjectMode, SmearMode, TrimForma, Verb,
};
use ph2d_tokens::Spacing;

use super::widgets::{command, labelled_seg, toggle};
use crate::state::Sculpt3dSnapshot;

/// **AS DUAS FILEIRAS DO PINCEL DE CONTORNO.**
///
/// ⚠️⚠️ **Elas são DUAS perguntas diferentes e é fácil lê-las como uma:** a
/// primeira escolhe **o que a borda faz** (dobrar, expandir, inflar, agarrar,
/// torcer, alisar) e a segunda **como isso esmorece AO LONGO da borda**. Uma
/// terceira, a curva do pincel, gradua a profundidade **para dentro** da peça —
/// e essa já vive na secção do pincel, partilhada com todos os verbos.
///
/// ⛔ **Sem a primeira o artista alcança UM dos seis gestos**, que é o defeito
/// que o pincel de tecido pagou: *um motor vivo sem botão nenhum.*
pub(super) fn paint_boundary_rows(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if !snap.ui.brush.offers_boundary_controls() {
        return y;
    }
    let modos = ph2d_sculpt3d::BoundaryModo::ALL;
    let selected = modos
        .iter()
        .position(|&m| m == snap.ui.brush.boundary.modo)
        .unwrap_or(0);
    let labels: Vec<&str> = modos.iter().map(|m| tr(m.label_key())).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.boundary_mode"),
        &crate::ids::SCULPT3D_BOUNDARY_MODE,
        &labels,
        selected,
        x,
        w,
        y,
    );
    let quedas = ph2d_sculpt3d::BoundaryQueda::ALL;
    let selected = quedas
        .iter()
        .position(|&q| q == snap.ui.brush.boundary.queda_no_contorno)
        .unwrap_or(0);
    let labels: Vec<&str> = quedas.iter().map(|q| tr(q.label_key())).collect();
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.boundary_falloff"),
        &crate::ids::SCULPT3D_BOUNDARY_FALLOFF,
        &labels,
        selected,
        x,
        w,
        y,
    ) + Spacing::Sm.px()
}

/// **A FILEIRA E AS DUAS CAIXAS DO PINCEL DE POSE.**
///
/// ⚠️ **A fileira de modos NÃO é um luxo: sem ela o artista alcança UM dos três
/// gestos.** Girar/torcer, escalar/transladar e espremer/esticar são
/// deformações diferentes — e cada barra tem duas metades, porque o modificador
/// de inversão **troca de deformação** em vez de trocar o sinal da força.
///
/// ⚠️ **A trava aparece SÓ no modo de escala** — ver
/// [`ph2d_sculpt3d::Brush::offers_pose_rotation_lock`]: nos outros dois ela não
/// tem o que travar, e um interruptor inerte é pior que um ausente.
pub(super) fn paint_pose_rows(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if !snap.ui.brush.offers_pose_controls() {
        return y;
    }
    let modos = ph2d_sculpt3d::PoseDeformacao::ALL;
    let selected = modos
        .iter()
        .position(|&m| m == snap.ui.brush.pose.deformacao)
        .unwrap_or(0);
    let labels: Vec<&str> = modos.iter().map(|m| tr(m.label_key())).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.pose_mode"),
        &crate::ids::SCULPT3D_POSE_MODE,
        &labels,
        selected,
        x,
        w,
        y,
    );
    // ⛔⛔⛔ **O `Drag Reads` SAIU por veredito do dono (2026-09-17):** *«Full
    // drag parece ser o único necessário»*. Ele nasceu no dia anterior como um
    // par de chips (*«cada modo com opção, com um botão para mudar o modo»*),
    // o dono testou-o, e a escolha ficou sendo uma só — ⇒ o pincel lê o arrasto
    // INTEIRO sempre, e a lei mora em [`ph2d_sculpt3d::PoseControlos::lei`],
    // onde a divergência declarada contra a espec está registada com gate.
    //
    // ⚠️ *Um selector de uma opção é um controlo morto com cara de escolha* —
    // e o par de fileiras acima e abaixo já responde ao que ele perguntava.
    let y = toggle(
        ctx,
        crate::ids::SCULPT3D_POSE_ANCHORED,
        tr("panel.sculpt3d.pose_anchored"),
        snap.ui.brush.pose.ancorado,
        x,
        w,
        y,
    ) + Spacing::Sm.px();
    if snap.ui.brush.offers_pose_rotation_lock() {
        toggle(
            ctx,
            crate::ids::SCULPT3D_POSE_ROT_LOCK,
            tr("panel.sculpt3d.pose_rot_lock"),
            snap.ui.brush.pose.trava_rotacao,
            x,
            w,
            y,
        ) + Spacing::Sm.px()
    } else {
        y
    }
}

/// **A FILEIRA DO PINCEL DE ESFREGAR DESLOCAMENTO** — *Deformation*, as três
/// direcções da espec §5.3.
///
/// ⚠️⚠️ **Elas NÃO são três leis: são três direcções de UMA lei.** O que o chip
/// escolhe é contra que vector o peso de cada vizinho é medido; a média
/// ponderada, a normalização com peso próprio `1` e o tecto são os mesmos nos
/// três. *Ler isto como «três modos» faz parecer que falta partilhar código
/// entre eles, e não falta — já é um.*
///
/// ⛔ **Sem ela o artista alcança UM dos três gestos**, que é o defeito que o
/// pincel de tecido pagou por escrito: *um motor vivo sem botão nenhum.*
pub(super) fn paint_smear_rows(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if !snap.ui.brush.offers_smear_controls() {
        return y;
    }
    let selected = SmearMode::ALL
        .iter()
        .position(|&m| m == snap.ui.brush.smear_mode)
        .unwrap_or(0);
    let labels: Vec<&str> = SmearMode::ALL.iter().map(|m| tr(m.label_key())).collect();
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.smear_mode"),
        &crate::ids::SCULPT3D_SMEAR_MODE,
        &labels,
        selected,
        x,
        w,
        y,
    ) + Spacing::Sm.px()
}

/// **A FORMA QUE O BOX TRIM CORTA** — ordem do dono (2026-09-15): *«Nos
/// parâmetros botões box e circle (novo) e laço»*.
///
/// ⚠️ **A segunda superfície dele — a SUAVIZAÇÃO do traço — é um SLIDER e vive
/// na tabela** (`rows.rs`), como todo knob numérico deste painel; e ela só é
/// pintada com o LAÇO na mão, porque a caixa e o círculo saem de dois pontos e
/// não têm traço a suavizar. *Pintá-la aqui à mão seria a segunda resposta a
/// «como se desenha um número», e o painel guiado por tabela é o único desta
/// casa sem knob morto* (CLAUDE.md §5.0).
pub(super) fn paint_trim_rows(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if snap.ui.brush.verb != ph2d_sculpt3d::Verb::BoxTrim {
        return y;
    }
    let selected = TrimForma::ALL
        .iter()
        .position(|&f| f == snap.ui.brush.trim_forma)
        .unwrap_or(0);
    let labels: Vec<&str> = TrimForma::ALL.iter().map(|f| tr(f.label_key())).collect();
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.trim_forma"),
        &crate::ids::SCULPT3D_TRIM_FORMA,
        &labels,
        selected,
        x,
        w,
        y,
    ) + Spacing::Sm.px()
}

/// ⭐⭐ **O QUE O `Ctrl` FAZ AO PINCEL DE PLANO** — as duas leis da espec §5.
///
/// ⚠️ **É um SELECTOR e não um segundo modificador:** o artista escolhe o que a
/// tecla vai fazer e depois carrega. Com *trocar os tectos* ele tem **aparar e
/// encher na mesma mão** — que é a razão de existir do controlo.
pub(super) fn paint_plano_rows(
    ctx: &mut PaintCtx<'_>,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if snap.ui.brush.verb != ph2d_sculpt3d::Verb::Plane {
        return y;
    }
    let selected = ph2d_sculpt3d::PlanoInversao::ALL
        .iter()
        .position(|&m| m == snap.ui.brush.plano_inversao)
        .unwrap_or(0);
    let labels: Vec<&str> = ph2d_sculpt3d::PlanoInversao::ALL
        .iter()
        .map(|m| tr(m.label_key()))
        .collect();
    labelled_seg(
        ctx,
        tr("panel.sculpt3d.plano_inversao"),
        &crate::ids::SCULPT3D_PLANO_INVERSAO,
        &labels,
        selected,
        x,
        w,
        y,
    ) + Spacing::Sm.px()
}

/// **AS DUAS SUPERFÍCIES PINTADAS DO PINCEL DE PROJECTAR** — a direcção do raio
/// (espec §6.2) e procurar também para trás (§6.3.2).
///
/// ⚠️ **A terceira — a FOLGA — é um SLIDER e vive na tabela** (`rows.rs`), como
/// todo knob numérico deste painel. *Pintá-la aqui à mão seria a segunda
/// resposta a «como se desenha um número», e o painel guiado por tabela é o
/// único desta casa sem knob morto* (CLAUDE.md §5.0).
///
/// ⛔ **Sem elas o artista alcança UM dos dois modos e nunca o alvo do lado
/// errado** — o defeito que o pincel de tecido pagou por escrito: *um motor
/// vivo sem botão nenhum.*
pub(super) fn paint_project_rows(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if !snap.ui.brush.offers_project_controls() {
        return y;
    }
    let selected = ProjectMode::ALL
        .iter()
        .position(|&m| m == snap.ui.brush.project_mode)
        .unwrap_or(0);
    let labels: Vec<&str> = ProjectMode::ALL.iter().map(|m| tr(m.label_key())).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.project_mode"),
        &crate::ids::SCULPT3D_PROJECT_MODE,
        &labels,
        selected,
        x,
        w,
        y,
    ) + Spacing::Sm.px();
    // ⚠️ **A caixa responde *«a lei existe»*, nunca *«o flag está ligado»*** — a
    // mesma cerca do `Connected Only`: uma caixa que se escondesse quando
    // desmarcada seria uma caixa que ninguém consegue marcar.
    toggle(
        ctx,
        crate::ids::SCULPT3D_PROJECT_BIDIR,
        tr("panel.sculpt3d.project_bidir"),
        snap.ui.brush.project_bidirectional,
        x,
        w,
        y,
    ) + Spacing::Sm.px()
}

/// **AS DUAS FILEIRAS DO PINCEL DE TECIDO** — *Deformation* e *Simulation Area*,
/// na ordem em que o painel da referência as põe (espec §8.4).
///
/// ⚠️ **Elas só existem com o verbo Cloth na mão**, e a pergunta é ao VERBO — a
/// mesma cerca da lâmina do `MultiplaneScrape` acima: uma lista paralela aqui
/// seria uma fileira que aparece noutra ferramenta e não move um vértice.
///
/// ⚠️ **Antes de 2026-09-06 os dois selectores eram VARIÁVEIS DE AMBIENTE.** O
/// motor respondia aos oito modos e às três áreas desde que a lei da referência
/// nasceu, e o artista chegava a UM. *Não era um botão morto: era um motor vivo
/// sem botão nenhum.*
pub(super) fn paint_cloth_rows(
    ctx: &mut PaintCtx,
    snap: &Sculpt3dSnapshot,
    x: f32,
    w: f32,
    y: f32,
) -> f32 {
    if snap.ui.brush.verb != Verb::Cloth {
        return y;
    }
    let selected = ClothMode::ALL
        .iter()
        .position(|&m| m == snap.ui.brush.cloth_mode)
        .unwrap_or(0);
    let labels: Vec<&str> = ClothMode::ALL.iter().map(|m| tr(m.label_key())).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.cloth_mode"),
        &crate::ids::SCULPT3D_CLOTH_MODE,
        &labels,
        selected,
        x,
        w,
        y,
    );
    let selected = ClothArea::ALL
        .iter()
        .position(|&a| a == snap.ui.brush.cloth_area)
        .unwrap_or(0);
    let labels: Vec<&str> = ClothArea::ALL.iter().map(|a| tr(a.label_key())).collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.cloth_area"),
        &crate::ids::SCULPT3D_CLOTH_AREA,
        &labels,
        selected,
        x,
        w,
        y,
    );
    let selected = ClothForceFalloff::ALL
        .iter()
        .position(|&f| f == snap.ui.brush.cloth_force_falloff)
        .unwrap_or(0);
    let labels: Vec<&str> = ClothForceFalloff::ALL
        .iter()
        .map(|f| tr(f.label_key()))
        .collect();
    let y = labelled_seg(
        ctx,
        tr("panel.sculpt3d.cloth_force_falloff"),
        &crate::ids::SCULPT3D_CLOTH_FORCE_FALLOFF,
        &labels,
        selected,
        x,
        w,
        y,
    );
    // **A BASE PERSISTENTE** — o interruptor e o botão que o torna observável.
    //
    // ⚠️⚠️ **Os DOIS, e nenhum basta sozinho:** ligar a opção sem base gravada é
    // um **no-op exacto** (a construção cai no repouso do traço), e gravar a base
    // com a opção desligada não muda nada. *Um controlo sozinho aqui seria um
    // botão que o artista carrega e não vê acontecer nada.*
    let y = toggle(
        ctx,
        crate::ids::SCULPT3D_CLOTH_PERSISTENT,
        tr("panel.sculpt3d.cloth_persistent"),
        snap.ui.brush.cloth_persistent,
        x,
        w,
        y,
    );
    let y = command(
        ctx,
        crate::ids::SCULPT3D_CLOTH_SET_BASE,
        tr("panel.sculpt3d.cloth_set_base"),
        x,
        w,
        y,
    );
    // **A COLISÃO** — o pano pára nas outras peças da cena.
    //
    // ⚠️ **Ela nasce desligada e o painel não a esconde:** o custo é um raio por
    // vértice activo, por colisor e por passo, e é o artista que decide se o
    // paga. *Um controlo caro escondido é um controlo que ninguém sabe que tem.*
    let y = toggle(
        ctx,
        crate::ids::SCULPT3D_CLOTH_COLLISIONS,
        tr("panel.sculpt3d.cloth_collisions"),
        snap.ui.brush.cloth_collisions,
        x,
        w,
        y,
    );
    // **O PINO DA FRONTEIRA**, e só onde a lei existe. ⚠️ A pergunta é a mesma
    // que [`ph2d_sculpt3d`] faz ao construir o pincel — a lei recusa o pino fora
    // da área *Local* (espec §2.3), e a caixa nos outros dois terços do selector
    // seria um interruptor de coisa nenhuma. ⛔ Duas cópias da pergunta
    // divergiriam; esta pergunta ao MOTOR, pelo mesmo `area()`.
    if !snap.ui.brush.cloth_area.offers_pin() {
        return y;
    }
    toggle(
        ctx,
        crate::ids::SCULPT3D_CLOTH_PIN,
        tr("panel.sculpt3d.cloth_pin"),
        snap.ui.brush.cloth_pin,
        x,
        w,
        y,
    )
}
