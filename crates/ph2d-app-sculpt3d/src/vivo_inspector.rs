//! **O instantâneo do CATAVENTO e o dreno das edições dele.**
//!
//! Irmão do `ph2d_app_components::weapon_inspector`: a shell publica o que o painel mostra, e o
//! painel devolve edições que voltam por aqui.
//!
//! ⭐ **Ele mora na família e não na `ph2d-app-components`**, que é onde as pontes genéricas do
//! Inspector vivem: o `Mesh3D` é o vocabulário desta rota, e o *«está assado?»* que a queixa lê é
//! um facto do mapa de formas — o assunto desta crate. *A shell é composição; ela chama a casa que
//! sabe da pergunta.*
//!
//! # ⚠️ O `assado` chega por ARGUMENTO, e é deliberado
//!
//! O mapa vive no `AppGfx`, que nenhuma crate vê. Passá-lo como `bool` mantém a ponte pura e
//! testável **sem um device**, que é a diferença entre um gate que corre no CI e um `#[ignore]`.

use ph2d_ecs::{Entity, Mesh3D, SimWorld};
use ph2d_editor_core::mesh3d_edits::{InspectorMesh3dInfo, Mesh3dFieldEdit as E};

/// **O que o painel mostra do catavento deste objecto**, ou `None` se ele não tiver o componente.
#[must_use]
pub fn build_info(sim: &SimWorld, bits: u64, assado: bool) -> Option<InspectorMesh3dInfo> {
    let m = sim.world().get::<Mesh3D>(Entity::from_bits(bits))?;
    Some(InspectorMesh3dInfo {
        entity_bits: bits,
        yaw: m.yaw,
        pitch: m.pitch,
        spin: m.spin,
        assado,
    })
}

/// Aplica as edições. Devolve `true` se alguma coisa mudou (o que suja a fila do undo).
pub fn apply_all(sim: &mut SimWorld, edits: &[(u64, E)]) -> bool {
    let mut mudou = false;
    for (bits, e) in edits {
        let Some(mut m) = sim.world_mut().get_mut::<Mesh3D>(Entity::from_bits(*bits)) else {
            continue;
        };
        // ⚠️ **A conversão de GRAUS para RADIANOS vive AQUI**, numa porta só — o painel oferece
        // graus porque é o que o artista lê, e a lei consome radianos. *Convertê-la no pintor seria
        // a segunda resposta à pergunta «que unidade é esta».*
        #[allow(clippy::cast_possible_truncation)]
        let novo = match *e {
            E::YawGraus(g) => Mesh3D {
                yaw: (g as f32).to_radians(),
                ..*m
            },
            E::PitchGraus(g) => Mesh3D {
                pitch: (g as f32).to_radians(),
                ..*m
            },
            E::Spin(v) => Mesh3D {
                spin: v as f32,
                ..*m
            },
        };
        // ⚠️ **Só escreve se MUDOU**, e não é higiene: um `get_mut` marca o componente como sujo em
        // toda passagem, e o undo desta casa regista por DIFF — escrever o mesmo valor a cada
        // quadro de arrasto poria um passo por quadro.
        if novo != *m {
            *m = novo;
            mudou = true;
        }
    }
    mudou
}

#[cfg(test)]
mod tests {
    use super::{apply_all, build_info};
    use ph2d_ecs::{Mesh3D, SimWorld};
    use ph2d_editor_core::mesh3d_edits::{Mesh3dFieldEdit as E, Mesh3dQueixa};

    /// ⭐⭐ **O `assado` chega de FORA e é ele que decide a queixa** — as duas metades.
    ///
    /// **Mutação que deve sangrar:** `assado,` → `assado: true,` no `build_info`.
    #[test]
    fn o_assado_chega_por_argumento_e_decide_a_queixa() {
        let mut sim = SimWorld::default();
        let e = sim.world_mut().spawn(Mesh3D::default()).id();
        let bits = e.to_bits();

        let por_assar = build_info(&sim, bits, false).expect("tem o componente");
        assert_eq!(por_assar.queixa(), Some(Mesh3dQueixa::SemForma));

        let assado = build_info(&sim, bits, true).expect("tem o componente");
        assert_eq!(
            assado.queixa(),
            Some(Mesh3dQueixa::Parado),
            "assado e parado: a queixa passa a ser a mais geral"
        );

        // ⭐ O CONTROLO: quem não tem o componente não tem secção nenhuma (ADR-0166).
        let outro = sim.world_mut().spawn(()).id();
        assert!(build_info(&sim, outro.to_bits(), true).is_none());
    }

    /// ⭐⭐⭐ **Os GRAUS do painel entram como RADIANOS no componente**, e escrever o mesmo valor
    /// NÃO suja a fila.
    ///
    /// ⚠️ A segunda metade não é higiene: o undo desta casa regista por DIFF, e um arrasto que
    /// escreve o mesmo número a cada quadro poria **um passo por quadro**.
    ///
    /// **Mutações que devem sangrar:** `to_radians()` → nada · `if novo != *m` → `if true`.
    #[test]
    fn os_graus_entram_em_radianos_e_o_igual_nao_suja() {
        let mut sim = SimWorld::default();
        let e = sim.world_mut().spawn(Mesh3D::default()).id();
        let bits = e.to_bits();

        assert!(apply_all(&mut sim, &[(bits, E::YawGraus(180.0))]));
        let m = *sim.world().get::<Mesh3D>(e).expect("o componente fica");
        assert!(
            (m.yaw - std::f32::consts::PI).abs() < 1.0e-5,
            "180 graus tem de entrar como PI radianos, entrou {}",
            m.yaw
        );

        assert!(
            !apply_all(&mut sim, &[(bits, E::YawGraus(180.0))]),
            "escrever o MESMO valor nao pode sujar a fila do undo"
        );

        assert!(apply_all(&mut sim, &[(bits, E::Spin(0.25))]));
        let m = *sim.world().get::<Mesh3D>(e).expect("o componente fica");
        assert!((m.spin - 0.25).abs() < f32::EPSILON);
        assert!(
            (m.yaw - std::f32::consts::PI).abs() < 1.0e-5,
            "uma edicao de um campo nao pode reescrever os outros"
        );
    }
}
