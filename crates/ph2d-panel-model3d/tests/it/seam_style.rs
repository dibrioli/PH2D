//! ⭐⭐⭐ **A SECÇÃO DA CENA (o ESTILO) CHEGA AO ECRÃ E AO DEDO** — a costura, sem app.
//!
//! # ⛔⛔⛔ Os dois defeitos que este ficheiro guarda
//!
//! **(a) O painel podia ENGOLIR a secção inteira** (auditoria de 2026-09-19, `docs/Render3d/11`
//! §10.9). Quem publica o retrato apende as fileiras da CENA **no fim** da lista, e o `paint` corta
//! em `MAX_ROWS`. Com o teto a valer exactamente o pior NÓ, um polígono de `27` vértices empurrava
//! as dez fileiras de estilo **todas** para fora — e o rodapé dizia `(+10)`, que o artista lê como
//! *«faltam dez números do meu nó»*.
//!
//! **(b) As cinco cores eram UM controlo** (report do dono, no mesmo dia: *«se modifico qualquer cor
//! em style, todas mudam ao mesmo tempo»*). O gate que curou aquilo mede a PORTA do id; este mede o
//! **dedo**, que é a metade que faltava: `tests/it/seam.rs` tinha **zero** ocorrências de
//! `Param::Style`.
//!
//! # ⚠️ A população é DERIVADA, e do lado de cá da fronteira
//!
//! A crate que publica as fileiras de estilo (`ph2d-app-field3d`) **depende desta**, logo esta não a
//! pode ver — e uma lista de dez chaves escrita aqui envelheceria na hora seguinte (a camada de
//! estilo cresceu de `10` para `12` fileiras no próprio dia desta auditoria). ⇒ o que estes gates
//! percorrem é **tudo o que o painel declara poder hospedar**
//! ([`ph2d_panel_model3d::MAX_SCENE_ROWS`]), e as asserções varrem **todas** as fileiras do retrato
//! publicado — nunca a fileira `0`.
//!
//! ⭐ O gate que prende a outra ponta — *«a cena não apende mais do que a folga»* — vive no
//! PRODUTOR (`ph2d_app_field3d::…::as_fileiras_da_cena_cabem_na_folga_delas`), que é o único sítio
//! onde `estilo::rows()` é contável.

use ph2d_editor_core::interaction::WidgetEvent;
use ph2d_editor_core::panel::{EventOutcome, Panel, PanelHostInternal};
use ph2d_field::{Bound, Param};
use ph2d_panel_model3d::state::{Model3dPanelState, ModelSnapshot, ParamRow};
use ph2d_panel_model3d::{
    MAX_ROWS_DE_UM_NO, MAX_SCENE_ROWS, Model3dPanel, ModelIntent, drain_intents, publish, state,
    swatch_id,
};
use ph2d_ui_testkit::MockPanelHost;

/// A entidade de uma fileira de nó. ⚠️ A da CENA é `0` **por desenho** — o estilo não é de entidade
/// nenhuma, e foi esse `0` que fez as cinco cores partilharem um id.
const UM_NO: u64 = 41;

/// Uma fileira de nó, viva e banal — só para ocupar a família até ao teto do NÓ.
fn fileira_do_no(i: usize) -> ParamRow {
    ParamRow {
        entity: UM_NO,
        param: Param::Dim(u16::try_from(i % 8).expect("cabe")),
        key: "field.dim.round",
        value: 0.25,
        lo: 0.0,
        bound: Bound::Soft(1.0),
        inert: None,
        integral: false,
        choices: &[],
        swatch: None,
        section: None,
        subject: None,
    }
}

/// Quantos slots do estilo uma COR ocupa — ela é o `xyz` de um `vec4` na arrumação dele.
const CANAIS_DE_UMA_COR: usize = 3;

/// Uma fileira de ESTILO.
///
/// ⚠️⚠️ **Quem classifica uma fileira do estilo como COR é o PRODUTOR dela, e ele vive do outro lado
/// da fronteira** (`ph2d_app_field3d::estilo`, que depende desta crate). ⛔ E a porta do documento
/// **não** serve para o adivinhar: medido em 2026-09-19, `Param::Style(k).colour_channels()`
/// responde `Some` para **todo** `k` — ela diz *«uma cor desta família são três slots
/// consecutivos»*, que é outra pergunta. *Uma porta que responde sempre «sim» não é um
/// classificador.*
///
/// ⇒ esta fixtura **escolhe**, e o que ela deve ao gate é ter as DUAS espécies presentes (há um
/// piso a afirmá-lo). O que é derivado aqui é a **POPULAÇÃO** — a folga inteira que o painel declara
/// hospedar —, que é a grandeza que os dois defeitos deste ficheiro tocavam.
fn fileira_da_cena(slot: u8) -> ParamRow {
    let e_cor = usize::from(slot) % CANAIS_DE_UMA_COR == 0;
    ParamRow {
        entity: 0,
        param: Param::Style(slot),
        key: "panel.model3d.section.style",
        value: 0.5,
        lo: 0.0,
        bound: Bound::Soft(1.0),
        inert: None,
        integral: false,
        choices: &[],
        // ⚠️ Uma cor por fileira, e todas DIFERENTES: com a mesma cor em todas, o defeito do id
        // partilhado ficaria invisível — o selector abriria sobre o valor certo por acidente.
        swatch: e_cor.then(|| [slot.wrapping_mul(7), 40, 200_u8.wrapping_sub(slot)]),
        section: (slot == 0).then_some("panel.model3d.section.style"),
        subject: None,
    }
}

/// O pior retrato possível: o NÓ no teto dele **mais** a cena inteira.
fn o_pior_retrato() {
    let mut rows: Vec<ParamRow> = (0..MAX_ROWS_DE_UM_NO).map(fileira_do_no).collect();
    rows.extend(
        (0..MAX_SCENE_ROWS).map(|s| fileira_da_cena(u8::try_from(s).expect("a folga cabe num u8"))),
    );
    publish(ModelSnapshot {
        rows,
        node_count: 1,
        ..ModelSnapshot::default()
    });
}

fn pinta() -> (MockPanelHost, Model3dPanelState) {
    let mut host = MockPanelHost::with_panel::<Model3dPanel>();
    host.set_panel_visible(Model3dPanel::ID, true);
    let mut estado = Model3dPanelState;
    // ⚠️ **Alto de propósito:** o corpo do painel RECORTA, e a pergunta deste ficheiro é *«a fileira
    // existe?»* e não *«ela cabe sem rolar?»*. Num ecrã curto a última fileira ficaria fora da banda
    // e o gate leria o RECORTE como o corte da família — duas causas, o mesmo sintoma.
    let viewport = ph2d_editor_core::zones::Rect::new(0.0, 0.0, 1280.0, 6000.0);
    let _ = host.paint::<Model3dPanel>(&mut estado, viewport);
    println!(
        "  retrato: {} fileiras · conteúdo {:.0} px",
        MAX_ROWS_DE_UM_NO + MAX_SCENE_ROWS,
        ph2d_panel_model3d::last_content_h()
    );
    (host, estado)
}

/// ⭐⭐⭐ **A ÚLTIMA FILEIRA DA CENA CHEGA AO ÍNDICE DE ACERTO** — com o nó no teto dele.
///
/// ⛔⛔ **Era esta a fileira que desaparecia.** O corte é `.take(MAX_ROWS)` e a cena vem **por
/// último**, logo o que cai fora é sempre a secção da cena — nunca a do nó. Medido antes da cura,
/// com `MAX_ROWS = 85`: `85` linhas de nó + `10` de estilo ⇒ **`0` de `10`** visíveis.
///
/// ⚠️ **O sujeito é a ÚLTIMA e não a primeira**: um teto com um degrau de folga deixaria a primeira
/// passar e continuaria a comer a última. *Um gate que mede a fileira `0` de uma lista cortada pelo
/// fim não mede o corte.*
///
/// **Mutação que deve sangrar:** `MAX_SCENE_ROWS = 0` (o teto volta a ser o de 2026-09-19).
#[test]
fn a_ultima_fileira_da_cena_chega_ao_ecra() {
    let _ = drain_intents();
    o_pior_retrato();
    let (mut host, _) = pinta();
    let ultima = u32::try_from(MAX_ROWS_DE_UM_NO + MAX_SCENE_ROWS - 1).expect("cabe");
    let alvo = fileira_da_cena(u8::try_from(MAX_SCENE_ROWS - 1).expect("cabe"));
    // ⚠️⚠️ **A condição é `row.swatch`, e NÃO «a porta devolve um id»** — a `swatch_id` responde
    // `Some` a *toda* fileira de estilo (ela diz *«esta família tem espaço de nomes próprio»*), e o
    // que decide se a fileira é pintada como amostra é o `swatch` dela. *Perguntar à porta errada
    // dá um id válido para um widget que não foi pintado* — e o gate lê-o como um corte.
    let id = if alvo.swatch.is_some() {
        swatch_id(&alvo).expect("uma fileira de cor tem id")
    } else {
        ph2d_panel_model3d::ids::model3d_radius_slider(ultima)
    };
    assert!(
        host.hit_index_mut().rect_for(id).is_some(),
        "a última fileira da CENA não chegou ao índice de acerto — ela foi cortada pelo \
         `take(MAX_ROWS)`, e o rodapé anuncia isso como «(+N)» linhas do NÓ em falta"
    );
}

/// ⭐⭐⭐ **CADA FILEIRA DE COR DA CENA É UM CONTROLO PRÓPRIO** — sobre a folga INTEIRA.
///
/// ⛔⛔ **O report do dono era este defeito com cinco fileiras**; aqui ele é medido sobre as
/// [`MAX_SCENE_ROWS`] que o painel declara poder hospedar, que é a população que a camada de estilo
/// pode vir a ter sem ninguém tocar neste ficheiro.
///
/// **Mutação que deve sangrar:** devolver um id fixo no braço `Param::Style` da porta `swatch_id`.
#[test]
fn cada_fileira_de_cor_da_cena_e_um_controlo_proprio() {
    let mut vistos: Vec<ph2d_a11y::NodeId> = Vec::new();
    for s in 0..MAX_SCENE_ROWS {
        let row = fileira_da_cena(u8::try_from(s).expect("cabe"));
        if row.swatch.is_none() {
            continue;
        }
        let id = swatch_id(&row).expect("uma fileira de cor da cena tem de ter id");
        assert!(
            !vistos.contains(&id),
            "a fileira de estilo {s} partilha o id de uma anterior — mexer numa cor mexe em todas"
        );
        vistos.push(id);
    }
    // ⚠️ **Piso de população**: se a porta do documento deixar de classificar nenhuma fileira como
    // cor, o laço acima não compara nada e lê-se como «está tudo bem».
    assert!(
        vistos.len() >= 3,
        "o censo achou {} fileiras de cor na folga da cena — ele deixou de medir o que diz medir",
        vistos.len()
    );
}

/// ⭐⭐⭐ **ARRASTAR UMA FILEIRA DA CENA CHEGA AO DOCUMENTO COM O SLOT DELA** — gesto REAL.
///
/// ⚠️ **A fileira escolhida é a ÚLTIMA da cena**, que é a que o corte comia e a que um id partilhado
/// mandaria para o slot errado. *Medir a primeira mede o caso que nenhum dos dois defeitos tocava.*
///
/// **Mutações que devem sangrar:** `MAX_SCENE_ROWS = 0`; cravar o `slot` do intent.
#[test]
fn arrastar_uma_fileira_da_cena_chega_ao_intent_com_o_slot_dela() {
    let _ = drain_intents();
    o_pior_retrato();
    let (mut host, mut estado) = pinta();

    // A posição da fileira na lista publicada, e o SLOT dela no estilo — dois números diferentes, e
    // é exactamente essa distinção que o defeito do id partilhado apagava.
    let (posicao, slot) = (0..MAX_SCENE_ROWS)
        .rev()
        .map(|s| (MAX_ROWS_DE_UM_NO + s, u8::try_from(s).expect("cabe")))
        .find(|(_, s)| fileira_da_cena(*s).swatch.is_none())
        .expect("a folga da cena tem pelo menos uma fileira de NÚMERO");
    let slider = ph2d_panel_model3d::ids::model3d_radius_slider(
        u32::try_from(posicao).expect("cabe num u32"),
    );

    host.set_slider_value(slider, 0.5);
    let outcome =
        host.apply_panel_event::<Model3dPanel>(&mut estado, WidgetEvent::ValueChanged(slider));
    assert_eq!(
        outcome,
        EventOutcome::Consumed,
        "a fileira {posicao} da cena não respondeu a uma edição REAL — ou ela ficou fora da família \
         registada, ou o braço do `event.rs` não a alcança"
    );
    assert_eq!(
        drain_intents(),
        vec![ModelIntent::SetParam {
            entity: 0,
            param: Param::Style(slot),
            value: 0.5,
        }],
        "a edição da fileira {posicao} saiu com outro sujeito — o slot do estilo é a POSIÇÃO na \
         arrumação dele, e não a da linha no painel"
    );
    let _ = state::current();
}

/// ⭐⭐⭐ **CLICAR NA ÚLTIMA COR DA CENA ABRE O SELECTOR SOBRE ELA** — e não sobre outra.
///
/// ⛔ **É o report do dono medido pelo DEDO.** O gate da porta prova que os ids são distintos; este
/// prova que o clique num deles abre o selector **naquele**, o que exige as três metades do
/// `paint_swatch` (registar como amostra, registar no índice de acerto, e a cor semeada).
///
/// **Mutação que deve sangrar:** apagar o `register_picker_swatch`; devolver um id fixo na porta.
#[test]
fn clicar_na_ultima_cor_da_cena_abre_o_selector_sobre_ela() {
    let _ = drain_intents();
    o_pior_retrato();
    let (mut host, _) = pinta();
    let slot = (0..MAX_SCENE_ROWS)
        .rev()
        .map(|s| u8::try_from(s).expect("cabe"))
        .find(|s| fileira_da_cena(*s).swatch.is_some())
        .expect("a folga da cena tem pelo menos uma fileira de COR");
    let id = swatch_id(&fileira_da_cena(slot)).expect("a cor tem id");
    let r = host
        .hit_index_mut()
        .rect_for(id)
        .expect("a última cor da cena não chegou ao índice de acerto");
    let _ = host.click_at(r.x + r.w * 0.5, r.y + r.h * 0.5);
    assert_eq!(
        host.store().picker_target(),
        Some(id),
        "clicar na última cor da cena abriu o selector sobre OUTRA amostra"
    );
}
