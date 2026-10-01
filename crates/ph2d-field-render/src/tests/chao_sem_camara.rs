//! ⭐⭐⭐⭐ **O campo do chão NÃO depende do ZOOM** — a precisão da assadura é a do mundo
//! ([`crate::Sharpness::do_mundo`]), e é isso que deixa a chave do campo no app sem a câmara.
//!
//! ⛔ Report do dono (2026-10-01): *«se aproximar do objeto ainda fica lento e perde resolução»* —
//! de perto a precisão da câmara mudava a cada quadro de zoom, a chave mudava com ela, e o campo era
//! re-assado na CPU em todo quadro (`~88 ms` no nó).
//!
//! ⚠️⚠️ **O PREÇO, medido (2026-09-21, pelo gate que este substitui — a premissa dele morreu aqui):**
//! o campo contra o da precisão de fábrica, `lado_px = 1080` fixo.
//!
//! | `half_extent` | `hit` da câmara | em bytes |
//! |---:|---:|---:|
//! | `1,6` · `0,8` | `2,0e-4` (preso no tecto) | `0,000` |
//! | `0,4` | `1,85e-4` | `0,021` |
//! | `0,2` | `9,26e-5` | `0,169` |
//! | `0,05` | `2,31e-5` | `0,527` |
//! | `0,005` | `2,31e-6` | **`0,910`** |
//!
//! ⇒ a precisão do mundo é a que todo enquadramento de fábrica já usava, e fica a **menos de um byte**
//! da resposta convergida (o desvio SATURA quando o `hit` desce). ⛔ A alternativa — deixar a câmara
//! na chave — compra esse `< 1` byte só no zoom apertado e paga um re-assar por QUADRO de zoom; e
//! guardar sem a câmara na chave mas assar com ela faria a mesma vista depender do zoom em que o
//! armazém nasceu. ⭐ **As SONDAS seguem a mesma lei** (o 2.º gate), dos dois lados da placa.

use crate::{Ground, Orbit, PointLamp, Surfaces};
use ph2d_field::{FieldDoc, NodeId, Primitive, Xform};
use ph2d_field_eval::hybrid::Registry;

const RAIO: f32 = 0.5;

#[test]
fn o_campo_do_chao_e_o_mesmo_a_qualquer_zoom() {
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: RAIO },
            Xform::at(0.0, RAIO, 0.0),
        )],
        NodeId(0),
    )
    .expect("a bola");
    let reg = Registry::new();
    let mats = [ph2d_material::OpenPbr {
        base_color: [0.75, 0.06, 0.06],
        ..ph2d_material::OpenPbr::default()
    }
    .prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let luz = [PointLamp {
        world: [1.2, 1.8, 0.8],
        radiance_at_one: [6.0, 6.0, 6.0],
    }];
    let lado = 1080usize;
    let longe = Orbit {
        target: [0.0, RAIO, 0.0],
        half_extent: 1.6,
        ..Orbit::default()
    };
    let perto = Orbit {
        half_extent: 0.05,
        ..longe
    };
    // CONTROLO: a precisão da câmara DIFERE entre os dois — senão o gate não conteria o fenómeno.
    assert_ne!(
        crate::Sharpness::for_frame(longe.half_extent, lado),
        crate::Sharpness::for_frame(perto.half_extent, lado),
        "os dois enquadramentos têm a mesma precisão de câmara — a fixtura não mede nada"
    );
    let assa = |cam: &Orbit| {
        crate::ground_bounce::bake_ground_bounce(
            &doc,
            &reg,
            cam,
            Ground { height: 0.0 },
            &surfaces,
            &luz,
            8,
            16,
            lado,
        )
    };
    let (a, b) = (assa(&longe), assa(&perto));
    assert!(
        a.value.iter().any(|v| v[0] > 0.0),
        "CONTROLO: o campo saiu vazio — a fixtura não acende o chão"
    );
    assert_eq!(
        a.value, b.value,
        "o campo do chão mudou com o ZOOM — a chave do app (`ChaveDoChao`) teria de levar a câmara"
    );
}

/// ⭐⭐⭐⭐ **E as SONDAS também** — o dispositivo guarda-as entre quadros sem a câmara na chave
/// (`ph2d_field_gpu::sondas_na_placa`), logo a assadura não pode ler a precisão do quadro: senão a
/// mesma vista sairia diferente conforme o zoom em que o armazém nasceu.
#[test]
fn as_sondas_sao_as_mesmas_a_qualquer_zoom() {
    let doc = FieldDoc::new(
        vec![ph2d_field_eval::leaf(
            Primitive::Sphere { radius: RAIO },
            Xform::at(0.0, RAIO, 0.0),
        )],
        NodeId(0),
    )
    .expect("a bola");
    let reg = Registry::new();
    let mats = [ph2d_material::OpenPbr {
        base_color: [0.75, 0.06, 0.06],
        ..ph2d_material::OpenPbr::default()
    }
    .prepare()];
    let surfaces = Surfaces {
        all: &mats,
        owners: None,
    };
    let luz = [PointLamp {
        world: [1.2, 1.8, 0.8],
        radiance_at_one: [6.0, 6.0, 6.0],
    }];
    let lado = 1080usize;
    let longe = Orbit {
        target: [0.0, RAIO, 0.0],
        half_extent: 1.6,
        ..Orbit::default()
    };
    let perto = Orbit {
        half_extent: 0.05,
        ..longe
    };
    assert_ne!(
        crate::Sharpness::for_frame(longe.half_extent, lado),
        crate::Sharpness::for_frame(perto.half_extent, lado),
        "CONTROLO: os dois enquadramentos têm a mesma precisão de câmara — a fixtura não mede nada"
    );
    let assa =
        |cam: &Orbit| crate::probes::bake_probes(&doc, &reg, cam, &surfaces, &luz, 6, 16, lado);
    let (a, b) = (assa(&longe), assa(&perto));
    assert!(
        a.sh.iter().flatten().any(|c| c[0] > 0.0),
        "CONTROLO: as sondas saíram vazias — a fixtura não acende nada"
    );
    assert_eq!(a.inside, b.inside, "quem está DENTRO mudou com o ZOOM");
    assert_eq!(
        a.sh, b.sh,
        "as sondas mudaram com o ZOOM — a chave da placa teria de levar a câmara"
    );
}
