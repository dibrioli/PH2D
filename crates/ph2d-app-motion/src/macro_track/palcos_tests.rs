//! ⭐ **As macros em cena** (W4 item 5.5 A + B): o palco nasce do `Macro`, segue o `MacroTempo` que a
//! Timeline escreve, só renasce quando o `Macro` muda, sai com a entidade — e enquanto toca, o editor
//! não coze os sinks do dono.

use super::gate_tests::{QUADROS, pelo_nucleo, retrato, sessao_em};
use super::palcos::PalcosDasMacros;
use crate::motion_mundo::MundoDaCena;
use crate::motion_objeto::{sinks_visiveis, tocados_por_macro};
use crate::motion_state::MotionState;
use ph2d_ecs::{Macro, MacroTempo, ModoDaMacro, SimWorld};

fn em(q: u32) -> f64 {
    f64::from(q) * 0.016
}

/// A sessão dos gates, gravada com a trilha do objecto `alvo` (`0` = todos).
fn gravada(alvo: u64) -> (Vec<u8>, Vec<Vec<u32>>) {
    let g = sessao_em(None);
    let mut doc = pelo_nucleo(&g);
    doc.tracks[0].target = alvo;
    (ph2d_macro::codec::write(&doc), g.retratos)
}

fn macro_de(doc: Vec<u8>) -> Macro {
    Macro {
        doc,
        modo: ModoDaMacro::Fiel,
    }
}

fn o_palco(p: &PalcosDasMacros) -> &MotionState {
    p.fontes().next().expect("um palco toca").0
}

/// ⭐ Red-first: com a macro conduzida, o editor larga os sinks do dono (e só os dele) e recoze; sem o
/// `MacroTempo` no quadro seguinte, devolve-os.
#[test]
fn com_a_macro_a_tocar_o_editor_nao_coze_o_dono_e_sem_relogio_volta_a_coze_lo() {
    const DONO: u64 = 7;
    let (bytes, _) = gravada(DONO);
    let mut sim = SimWorld::new();
    let e = sim
        .world_mut()
        .spawn((macro_de(bytes), MacroTempo { segundos: em(10) }))
        .id();
    let mut editor = MotionState::new();
    let sa = editor.doc.graph.add_node("motion.output");
    let sb = editor.doc.graph.add_node("motion.output");
    editor.doc.set_owner(sa, DONO);
    editor.doc.set_owner(sb, DONO + 1);
    let (mundo, mut palcos) = (MundoDaCena::default(), PalcosDasMacros::default());

    let marcas = editor.pump.sujidades();
    let tocados = palcos.segue(sim.world_mut(), &mundo, None);
    tocados_por_macro(&mut editor, tocados);
    assert!(palcos.tocam(), "a Timeline conduz a macro: o palco toca");
    assert_eq!(sinks_visiveis(&editor, vec![sa, sb]), vec![sb]);
    assert_eq!(
        editor.pump.sujidades(),
        marcas + 1,
        "o editor deixou de cozer um dono: recoze"
    );

    sim.world_mut().entity_mut(e).remove::<MacroTempo>();
    let tocados = palcos.segue(sim.world_mut(), &mundo, None);
    tocados_por_macro(&mut editor, tocados);
    assert!(
        !palcos.tocam(),
        "sem relógio neste quadro, o palco não toca"
    );
    assert_eq!(sinks_visiveis(&editor, vec![sa, sb]), vec![sa, sb]);
}

/// ⭐ O palco segue o relógio da Timeline (para a frente e para trás, o que o artista viu), fica em
/// cache num quadro sem relógio, renasce quando o `Macro` muda e sai com a entidade.
#[test]
fn o_palco_segue_o_relogio_e_so_renasce_quando_o_macro_muda() {
    let (bytes, retratos) = gravada(0);
    let mut sim = SimWorld::new();
    let fim = QUADROS - 1;
    let e = sim
        .world_mut()
        .spawn((macro_de(bytes), MacroTempo { segundos: em(fim) }))
        .id();
    let (mundo, mut palcos) = (MundoDaCena::default(), PalcosDasMacros::default());
    let quadro = |sim: &mut SimWorld, palcos: &mut PalcosDasMacros, q: Option<u32>| {
        let mut ent = sim.world_mut().entity_mut(e);
        match q {
            Some(q) => {
                ent.insert(MacroTempo { segundos: em(q) });
            }
            None => {
                ent.remove::<MacroTempo>();
            }
        }
        palcos.segue(sim.world_mut(), &mundo, None)
    };

    assert_eq!(quadro(&mut sim, &mut palcos, Some(fim)), [0].into());
    assert_eq!(retrato(o_palco(&palcos)), retratos[fim as usize]);
    let meio = QUADROS / 2;
    quadro(&mut sim, &mut palcos, Some(meio));
    assert_eq!(
        retrato(o_palco(&palcos)),
        retratos[meio as usize],
        "recuar na Timeline dá o que o artista viu nesse quadro"
    );

    // Sem relógio: não toca, mas o palco fica onde estava (em cache).
    assert!(quadro(&mut sim, &mut palcos, None).is_empty());
    let em_meio = super::em_cena::instante(em(meio));
    assert_eq!(palcos.tempo(e.to_bits()), Some(em_meio));
    // Um quadro sem mudança não o recria (o controlo do de baixo).
    quadro(&mut sim, &mut palcos, None);
    assert_eq!(palcos.tempo(e.to_bits()), Some(em_meio));
    // O `Macro` muda (os mesmos bytes: a pergunta é ao tique, nunca aos bytes) ⇒ o palco renasce.
    sim.world_mut().get_mut::<Macro>(e).expect("o Macro").modo = ModoDaMacro::Fiel;
    quadro(&mut sim, &mut palcos, None);
    assert_eq!(
        palcos.tempo(e.to_bits()),
        Some(ph2d_macro::MacroTime(0)),
        "o Macro mudou: o palco renasce no estado inicial"
    );

    sim.world_mut().despawn(e);
    palcos.segue(sim.world_mut(), &mundo, None);
    assert_eq!(palcos.quantos(), 0, "a entidade saiu: o palco sai");
}
