//! **A assinatura da secção CAMERA** — o que decide se os números dela se re-semeiam.
//!
//! ⚠️ **Irmão do [`crate::sync_sections`] por CAP de FICHEIRO**, e pela mesma lei das irmãs
//! (`sync_ray`, `sync_parallax`): *só o que SEMEIA um widget entra*. As caixas espelham o mundo
//! todo o quadro e ficam fora, e as leituras derivadas (quantas câmeras há, se esta manda) também —
//! elas não semeiam número nenhum.

use std::hash::{Hash, Hasher};

use ph2d_editor_core::screens::hero::InspectorCameraInfo;

pub(crate) fn assinatura(cam: &InspectorCameraInfo) -> u64 {
    let mut h = std::hash::DefaultHasher::new();
    cam.entity_bits.hash(&mut h);
    cam.camera.priority.hash(&mut h);
    let mut numeros = vec![
        cam.camera.height_world,
        cam.camera.offset[0],
        cam.camera.offset[1],
        cam.camera.dolly,
    ];
    // ⚠️ **A PRESENÇA de cada bloco entra**, e não só o valor: anexar o seguidor ou a cerca com os
    // valores de fábrica tem de re-semear o quadro em que as fileiras nascem.
    cam.follow.is_some().hash(&mut h);
    cam.limits.is_some().hash(&mut h);
    if let Some(f) = cam.follow.as_ref() {
        numeros.extend(f.damping);
        numeros.extend(f.dead_zone);
        numeros.extend(f.lookahead);
        numeros.extend(f.offset);
        f.target.hash(&mut h);
    }
    if let Some(l) = cam.limits.as_ref() {
        numeros.extend(l.min);
        numeros.extend(l.max);
    }
    for v in numeros {
        v.to_bits().hash(&mut h);
    }
    h.finish()
}
