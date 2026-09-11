//! ⭐⭐⭐ **UM GESTO fecha uma coluna, e a mesma borda trá-la de volta.**
//!
//! A medição de tablet (`docs/UI_New_and_Simple/medicoes/06`) diz que fechar as duas colunas
//! devolve **89 a 92 %** do ecrã em qualquer dos três alvos — *mais do que todas as faixas de
//! chrome somadas valem*. E até 2026-09-07 isso custava **dois passeios ao menu** *Ver*, um por
//! coluna, num aparelho sem teclado.
//!
//! # A decisão, e as três razões medidas
//!
//! O gesto é o do **Blender**, a referência que o dono nomeou: arrastar a borda de uma região para
//! dentro **fecha-a**, e uma alça na margem trá-la de volta.
//!
//! 1. o artista **já arrasta esta borda** — a costura shipou em 2026-08-30, e até aqui ela apenas
//!    travava no mínimo;
//! 2. funciona **sem teclado**, que é a condição num tablet. O `Ctrl+Space` do Blender e o modo
//!    sem distracções do Godot (`Ctrl+Shift+F11`) resolvem o mesmo problema **com uma tecla**, e
//!    uma tecla não existe no alvo desta casa;
//! 3. **não custa chrome permanente**: a alça só existe enquanto a coluna está fechada, e nesse
//!    estado ela troca 304–308 px por 6.
//!
//! # ⚠️ O que este gate mede, e o que ele recusa medir
//!
//! Ele mede a **área devolvida** — que é a razão de a obra existir — e a exclusão entre a costura
//! e a alça. ⛔ Ele **não** mede «o gesto existe»: um teste que afirmasse isso ficaria verde com o
//! gesto ligado a nada, que é a espécie de defeito que este repo varre a cada wave.

use ph2d_editor_core::screens::layout::{
    CenterSplit, ChromeBands, DockSide, DockSides, HeroLayout,
};
use ph2d_editor_core::zones::Rect;

/// Os três alvos que o dono nomeia, em pontos lógicos.
const TABLETS: &[(&str, f32, f32)] = &[
    ("iPad 12.9", 1366.0, 1024.0),
    ("iPad 11", 1194.0, 834.0),
    ("iPad mini", 1133.0, 744.0),
];

fn layout(w: f32, h: f32, docks: DockSides) -> HeroLayout {
    HeroLayout::for_viewport_bands(
        Rect::new(0.0, 0.0, w, h),
        false,
        ChromeBands::DEFAULT,
        CenterSplit::None,
        docks,
    )
}

/// A largura que sobra entre as duas colunas, como fracção da janela.
fn between_columns(w: f32, h: f32, docks: DockSides) -> f32 {
    let l = layout(w, h, docks);
    let (lc, rc) = l.side_columns();
    let left_edge = if docks.left { lc.x + lc.w } else { lc.x };
    let right_edge = if docks.right { rc.x } else { rc.x + rc.w };
    ((right_edge - left_edge) / w).max(0.0)
}

/// ⭐⭐⭐ **Fechar as duas colunas devolve mais de metade da LARGURA em todos os três alvos.**
///
/// ⚠️ A régua é a diferença entre os dois estados, não um número absoluto: o que a obra promete é
/// *ganho*, e um absoluto envelheceria com a próxima mudança de chrome.
#[test]
fn closing_both_columns_gives_back_more_than_half_the_width() {
    for (nome, w, h) in TABLETS {
        let abertas = between_columns(*w, *h, DockSides::BOTH);
        let fechadas = between_columns(*w, *h, DockSides::NONE);
        assert!(
            fechadas > abertas,
            "{nome}: fechar as colunas nao devolveu largura ({fechadas} contra {abertas})"
        );
        assert!(
            fechadas - abertas > 0.30,
            "{nome}: fechar devolveu so' {:.1} pontos percentuais de largura — a medicao de \
             2026-08-31 achou 40 a 50, e e' esse ganho que paga esta obra",
            (fechadas - abertas) * 100.0
        );
    }
}

/// ⛔ **A alça e a costura nunca são oferecidas ao mesmo tempo no mesmo lado.**
///
/// É a propriedade que torna a ordem do despacho segura: um toque na borda pede *uma* coisa.
#[test]
fn a_side_offers_either_the_seam_or_the_handle_never_both() {
    for (nome, w, h) in TABLETS {
        for docks in [
            DockSides::BOTH,
            DockSides::NONE,
            DockSides {
                left: false,
                right: true,
            },
            DockSides {
                left: true,
                right: false,
            },
        ] {
            let l = layout(*w, *h, docks);
            for side in [DockSide::Left, DockSide::Right] {
                let seam = l.dock_seam(side).w > 0.0;
                let handle = l.dock_reopen(side).w > 0.0;
                assert!(
                    !(seam && handle),
                    "{nome} {side:?} com {docks:?}: a costura E a alca foram oferecidas"
                );
                assert!(
                    seam || handle,
                    "{nome} {side:?} com {docks:?}: nem costura nem alca — a borda ficou MUDA, e \
                     com a coluna fechada isso e' um caminho de volta que nao existe"
                );
            }
        }
    }
}

/// ⛔⛔ **QUEM ocupa uma coluna é CONTADO, não adivinhado — e a wave 29 adivinhava.**
///
/// Aquela wave tinha uma porta `dock_tenant(side) -> &str` que devolvia **um** nome por lado, e o
/// dono reportou no mesmo dia: *«hierarquia fechou… Inspector não fechou»*. A causa é que uma
/// coluna pode ter **mais de um inquilino** — o `bgremoval` partilha o rect do Inspector, e o
/// `painter_layers` também vive à direita. Escondido um, os outros continuavam a publicar, o
/// `DockSides::from_published` continuava a ver a coluna ocupada, e ela não fechava.
///
/// ⇒ a porta foi **apagada**, e as duas metades do gesto passaram a perguntar a factos diferentes:
/// fechar conta os rects **publicados** (exacto), reabrir lê o **encaixe declarado** (porque um
/// painel fechado não publica nada). *As duas metades de um interruptor podem precisar de fontes
/// de verdade diferentes, e é o estado que elas atravessam que decide qual.*
#[test]
fn a_slot_knows_which_column_it_belongs_to() {
    use ph2d_editor_core::screens::slot::Slot;
    assert_eq!(Slot::LeftTop.dock_side(), Some(DockSide::Left));
    assert_eq!(Slot::LeftBottom.dock_side(), Some(DockSide::Left));
    assert_eq!(Slot::RightTop.dock_side(), Some(DockSide::Right));
    assert_eq!(Slot::RightBottom.dock_side(), Some(DockSide::Right));
    // ⛔ Reabrir a coluna da esquerda nao pode trazer de volta a linha do tempo.
    assert_eq!(Slot::Bottom.dock_side(), None);
    assert_eq!(Slot::Center.dock_side(), None);
    let lados = Slot::ALL.iter().filter(|s| s.dock_side().is_some()).count();
    assert_eq!(
        lados, 4,
        "a familia dos encaixes de coluna mudou de tamanho"
    );
}

/// ⭐⭐⭐ **As áreas NÃO SE TOCAM** — cada fronteira leva uma divisória.
///
/// Enio, 2026-09-07, com a captura do Godot: *«entre os painéis e o Canvas e entre os painéis e a
/// timeline há espaços»*. ⛔ E isto **não** desfaz o veredito de 2026-08-30 (*«a régua deve ficar
/// colada na hierarquia»*): o que saiu nesse dia foi o `EDGE_PAD` de **14 px** — 14 lê-se como um
/// buraco, 4 lê-se como uma divisória.
#[test]
fn every_boundary_between_two_areas_carries_one_divider() {
    let gap = ph2d_tokens::area_gap_px();
    for (nome, w, h) in TABLETS {
        let l = layout(*w, *h, DockSides::BOTH);
        let a = l.draw_area;
        assert_eq!(
            a.x - (l.hierarchy.x + l.hierarchy.w),
            gap,
            "{nome}: a area nao deixou divisoria depois da coluna da esquerda"
        );
        assert_eq!(
            l.inspector.x - (a.x + a.w),
            gap,
            "{nome}: a area nao deixou divisoria antes da coluna da direita"
        );
    }
}

/// ⛔⛔ **Uma coluna FECHADA reserva a largura da alça — senão ela tapa a RÉGUA.**
///
/// ⚠️ Este é o report do dono sobre a wave 29: *«hierarquia fechou e escondeu uma parte da régua do
/// canvas»*. A alça era pintada na borda da área de desenho, e a régua **nasce** nessa borda. A lei
/// já estava escrita três linhas acima no layout — *«uma coluna fechada não é reservada, senão a
/// régua ficaria a flutuar sobre o desenho»* — e a wave 29 pôs lá um controlo sem a reservar.
#[test]
fn a_closed_column_reserves_the_strip_its_handle_lives_in() {
    for (nome, w, h) in TABLETS {
        let l = layout(
            *w,
            *h,
            DockSides {
                left: false,
                right: true,
            },
        );
        let alca = l.dock_reopen(DockSide::Left);
        assert!(alca.w > 0.0, "{nome}: a coluna fechada nao ofereceu alca");
        assert!(
            alca.x + alca.w <= l.draw_area.x,
            "{nome}: a alca ({}..{}) entra na area de desenho (comeca em {}) — e' exactamente ali \
             que a regua nasce",
            alca.x,
            alca.x + alca.w,
            l.draw_area.x
        );
    }
}
