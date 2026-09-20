//! ⭐⭐⭐ **ESCOLHER UM PINCEL NA PALETA FECHA-A, E O PICK É O DO PAINEL.**
//!
//! # ⛔ O report que isto fecha (Enio, 2026-09-20)
//!
//! *«AO selecionar o pincel, o modal deveria se fechar automaticamente.»*
//!
//! ⚠️ **A causa medida foi OUTRA** — o teclado do modal vazava para os atalhos da escultura
//! (`shells/desktop/tests/it/um_modal_aberto_tem_o_teclado_antes_da_cena_3d.rs`), logo escrever
//! `clay` trocava o pincel POR BAIXO da paleta sem a fechar: do lado do artista, *«escolhi o
//! pincel e o modal ficou aberto»*. O caminho do CLIQUE já fechava.
//!
//! ⇒ este gate existe para isso deixar de ser uma dedução minha. *Uma causa descartada por
//! raciocínio e não por medição volta na wave seguinte.*
//!
//! # ⚠️ Porque ele vive AQUI
//!
//! Ele precisa do `HeroScreen` (que é da `ph2d-editor-core`) **e** do catálogo de pincéis (que é
//! da `ph2d-panel-sculpt3d`), e esta é a crate mais barata que enxerga as duas — o mesmo argumento
//! do `global_palette_catalog` ao lado.

use ph2d_editor_core::HeroScreen;
use ph2d_editor_core::NodeId;
use ph2d_editor_core::interaction::WidgetEvent;

/// O id do primeiro pincel da paleta — o que um clique escolheria.
fn primeiro_pincel() -> NodeId {
    let m = ph2d_panel_sculpt3d::brush_palette::build();
    m.groups
        .iter()
        .flat_map(|g| &g.subs)
        .flat_map(|s| &s.items)
        .map(|i| i.id)
        .next()
        .expect("a paleta de pincéis não pode estar vazia")
}

/// ⭐⭐⭐ **Um clique num pincel: a paleta FECHA e o pick fica lá para a ponte drenar.**
///
/// *Mutação que sangra:* o `apply` da paleta deixar de chamar `close_command_palette` no ramo do
/// item.
#[test]
fn escolher_um_pincel_fecha_a_paleta_e_deixa_o_pick() {
    // ⛔⛔⛔ **A ESCULTURA TEM DE ESTAR ARMADA, e sem isso o gate não contém o fenómeno.**
    //
    // ⚠️ A 2.ª redacção deste gate já entrava pela porta do quadro e **continuava verde** sobre o
    //    defeito: sem retrato publicado o painel da escultura ignora o evento, logo a rota nunca
    //    chega a passar por quem o consome. *Uma fixtura que não arma o sujeito mede o caminho
    //    onde o defeito não existe.*
    let _ = ph2d_panel_registry_init::register_all_panels();
    super::o_sculpt3d_armado::arma();

    let mut hero = HeroScreen::new(NodeId(1));
    let pincel = primeiro_pincel();
    hero.store
        .open_command_palette(ph2d_panel_sculpt3d::brush_palette::build());
    assert!(
        hero.store.command_palette_open(),
        "a paleta tinha de abrir — sem isso o resto deste gate não mede nada",
    );

    // ⛔⛔⛔ **PELA PORTA DO QUADRO — `HeroScreen::apply_event` — E NÃO PELO CHROME.**
    //
    // ⚠️⚠️ A 1.ª redacção deste gate chamava o `chrome::dispatch_all`, com um comentário a
    //    gabar-se de *«pela porta REAL do chrome»*. **Ele passava, e o app não fechava o modal**
    //    (report do dono, 2026-09-20, o mesmo defeito uma segunda vez). A rota do quadro é:
    //
    // ```text
    //    pre_dispatch  →  os PAINÉIS (Consumed ⇒ return)  →  showcase  →  chrome::dispatch_all
    // ```
    //
    //    ⇒ o painel da escultura reconhece o `SCULPT3D_VERB[i]` (é a lei que o *pick* reusa),
    //    **consome** o clique, troca o pincel, e o chrome nunca corre. *Eu tinha entrado ABAIXO
    //    da rotura e chamado-lhe a porta real.*
    //
    // ⭐ O `apply_event` é a porta que o quadro usa (`fase_chrome_clock` drena por ela), e é a
    //    única que percorre a rota inteira.
    let consumiu = hero.apply_event(WidgetEvent::Click(pincel));
    // ⛔ O estado que uma fixtura deixa para trás é o estado que a régua seguinte mede.
    super::o_sculpt3d_armado::desarma();

    assert!(
        consumiu,
        "o clique num item tem de ser consumido pela paleta"
    );
    assert!(
        !hero.store.command_palette_open(),
        "a paleta ficou ABERTA depois de escolher — é o report do dono de 2026-09-20",
    );
    assert_eq!(
        hero.store.take_command_pick(),
        Some(pincel),
        "o pick não ficou registado, logo a ponte não teria o que aplicar",
    );
}

/// ⭐⭐ **E o pick resolve para o pincel que a ficha resolvia** — a ponta que fecha a corrente.
///
/// ⚠️ Sem ela, a paleta podia fechar e entregar um id que a ponte não reconhece: *o modal
/// comporta-se bem e nada acontece*, que é o report seguinte.
///
/// *Mutação que sangra:* a paleta cunhar ids próprios em vez dos do painel.
#[test]
fn o_pick_resolve_para_um_pincel() {
    // ⚠️ **A ponte pede o retrato publicado** (sem cena não há pincel para trocar), e é por isso
    //    que a armação entra aqui: é a mesma porta `thread_local` que o painel lê.
    super::o_sculpt3d_armado::arma();
    let resolvido = ph2d_panel_sculpt3d::intent_for_palette_pick(primeiro_pincel());
    super::o_sculpt3d_armado::desarma();
    assert!(
        matches!(
            resolvido,
            Some(ph2d_panel_sculpt3d::Sculpt3dIntent::SetUi(_))
        ),
        "o id que a paleta entrega tem de resolver para uma troca de pincel, e resolveu {resolvido:?}",
    );
}
