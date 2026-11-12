//! B18.1/B18.2: trocar, duplicar e colher o documento NÃO copiam plano de camada — o `rgba8` é um
//! `Arc` partilhado (o mesmo endereço) — e a duplicata continua independente da origem (cópia na escrita).

use super::super::*;

// 1600×1280 RGBA = 8,19 MB: acima do limiar da cópia em blocos (7,5 MiB), para a rota paralela correr.
const W: u32 = 1600;
const H: u32 = 1280;

fn ruido(semente: u32) -> Vec<u8> {
    let mut s = semente | 1;
    (0..(W * H * 4) as usize)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            (s >> 24) as u8
        })
        .collect()
}

fn documento() -> (PainterTool, RtLayerId, RtLayerId) {
    let mut t = PainterTool::default();
    t.set_source(vec![255u8; (W * H * 4) as usize], W, H);
    let a = t.add_raster_layer_with_pixels("A", ruido(7)).expect("A");
    let b = t.add_raster_layer_with_pixels("B", ruido(11)).expect("B");
    (t, a, b)
}

/// Os ficheiros que copiaram um plano em blocos desde a última pergunta (nesta thread).
fn copiaram_em_blocos() -> Vec<&'static str> {
    crate::plane_copy::par_clone_sites_desde_a_ultima()
        .into_iter()
        .map(|(f, _)| f)
        .collect()
}

/// Os píxeis de `id` onde quer que vivam (a tela, se é a activa).
fn pixeis(t: &PainterTool, id: RtLayerId) -> Vec<u8> {
    if t.layers.active() == Some(id) {
        t.canvas_rgba.as_ref().clone()
    } else {
        t.images[&id].rgba8.as_ref().clone()
    }
}

/// Trocar de camada não copia plano nenhum: a camada que sai guarda o endereço da tela, e a tela nova é
/// o endereço do plano da que entra. Cada plano continua ao bit.
#[test]
fn trocar_de_camada_nao_copia_plano_e_guarda_os_pixeis_ao_bit() {
    let (mut t, a, b) = documento();
    let tela_b = Arc::as_ptr(&t.canvas_rgba);
    let plano_a = Arc::as_ptr(&t.images[&a].rgba8);
    let _ = copiaram_em_blocos();
    t.select_layer(a);
    let sitios = copiaram_em_blocos();
    assert!(sitios.is_empty(), "a troca copiou um plano: {sitios:?}");
    assert_eq!(Arc::as_ptr(&t.images[&b].rgba8), tela_b, "o flush copiou");
    assert_eq!(Arc::as_ptr(&t.canvas_rgba), plano_a, "a posse copiou");
    t.select_layer(b);
    t.select_layer(a);
    assert_eq!(pixeis(&t, a), ruido(7));
    assert_eq!(pixeis(&t, b), ruido(11));
}

/// A duplicata nasce com os bytes da origem e é independente dela: pintar a cópia não toca a origem.
#[test]
fn a_camada_duplicada_e_independente_da_origem() {
    let (mut t, a, _) = documento();
    let _ = copiaram_em_blocos();
    let nova = t.duplicate_layer(a).expect("duplicata");
    let sitios = copiaram_em_blocos();
    assert!(sitios.is_empty(), "duplicar copiou um plano: {sitios:?}");
    assert_eq!(
        Arc::as_ptr(&t.canvas_rgba),
        Arc::as_ptr(&t.images[&a].rgba8),
        "a duplicata não partilha o plano da origem"
    );
    assert_eq!(t.layers.active(), Some(nova));
    assert_eq!(pixeis(&t, nova), ruido(7));
    // Escreve na tela da duplicata pela porta de posse do plano (a mesma que um carimbo usa).
    Arc::make_mut(&mut t.canvas_rgba)[0] ^= 0xFF;
    t.select_layer(a);
    assert_eq!(
        pixeis(&t, a),
        ruido(7),
        "pintar a duplicata mexeu na origem"
    );
    assert_ne!(pixeis(&t, nova)[0], ruido(7)[0]);
}

/// O documento que a gravação colhe PARTILHA os planos do tool (B18.2): a tela, o relevo, a cobertura, o
/// material e o vidro são o mesmo endereço, sem cópia nenhuma — e no fio (postcard e JSON) é o documento
/// de `Vec`s de antes, byte a byte.
#[test]
fn o_documento_colhido_partilha_os_planos_e_grava_os_bytes_do_vec() {
    let (mut t, a, b) = documento();
    let n = (W * H) as usize;
    t.heights
        .insert(a, Arc::new((0..n).map(|i| i as f32 * 0.25).collect()));
    t.covers.insert(b, Arc::new(ruido(13)[..n].to_vec()));
    t.mats.insert(a, Arc::new(vec![[3u8, 1, 4, 1, 5, 9, 2]; n]));
    t.vidros
        .insert(b, Arc::new(vec![[2u8, 7, 1, 8, 2, 8, 1]; n]));
    let _ = copiaram_em_blocos();
    let novo = t.documento_vivo(0);
    let guardado = crate::tool::documents::StashedDoc::from_painted(novo.clone()).to_painted(0);
    let sitios = copiaram_em_blocos();
    assert!(sitios.is_empty(), "a colheita copiou um plano: {sitios:?}");
    fn mesmos<T>(
        x: &BTreeMap<RtLayerId, Arc<Vec<T>>>,
        y: &BTreeMap<RtLayerId, Arc<Vec<T>>>,
    ) -> bool {
        x.len() == y.len() && x.iter().all(|(k, v)| Arc::ptr_eq(v, &y[k]))
    }
    for d in [&novo, &guardado] {
        assert!(Arc::ptr_eq(&d.canvas_rgba, &t.canvas_rgba), "a tela copiou");
        assert!(mesmos(&d.heights, &t.heights), "o relevo copiou");
        assert!(mesmos(&d.covers, &t.covers), "a cobertura copiou");
        assert!(mesmos(&d.mats, &t.mats), "o material copiou");
        assert!(mesmos(&d.vidros, &t.vidros), "o vidro copiou");
        for (id, img) in &d.images {
            assert!(
                Arc::ptr_eq(&img.rgba8, &t.images[id].rgba8),
                "a imagem copiou"
            );
        }
    }
    // ORÁCULO — o documento de `Vec`s de antes do B18.2, campo a campo na mesma ordem.
    #[derive(serde::Serialize)]
    struct DeVecs {
        id: u32,
        layers: crate::layers::LayerStack,
        canvas_rgba: Vec<u8>,
        images: BTreeMap<RtLayerId, crate::compositor::LayerImage>,
        heights: BTreeMap<RtLayerId, Vec<f32>>,
        covers: BTreeMap<RtLayerId, Vec<u8>>,
        mats: BTreeMap<RtLayerId, Vec<ph2d_painter_brush::material::MaterialBytes>>,
        size: (u32, u32),
        papel: Option<[u8; 3]>,
        vidros: BTreeMap<RtLayerId, Vec<crate::compositor::vidro::Vidro>>,
        tintas_molhadas: std::collections::BTreeSet<RtLayerId>,
        fator_aparencia: f32,
    }
    fn vecs<T: Clone>(m: &BTreeMap<RtLayerId, Arc<Vec<T>>>) -> BTreeMap<RtLayerId, Vec<T>> {
        m.iter().map(|(k, v)| (*k, v.as_ref().clone())).collect()
    }
    let velho = DeVecs {
        id: 0,
        layers: t.layers.clone(),
        canvas_rgba: t.canvas_rgba.as_ref().clone(),
        images: t
            .images
            .iter()
            .map(|(k, v)| (*k, v.as_ref().clone()))
            .collect(),
        heights: vecs(&t.heights),
        covers: vecs(&t.covers),
        mats: vecs(&t.mats),
        size: t.source_size,
        papel: t.papel,
        vidros: vecs(&t.vidros),
        tintas_molhadas: t.tintas_molhadas.clone(),
        fator_aparencia: t.fator_aparencia,
    };
    let fio = postcard::to_allocvec(&velho).expect("postcard");
    assert!(
        postcard::to_allocvec(&novo).expect("postcard") == fio,
        "o postcard mudou"
    );
    assert!(
        serde_json::to_string(&novo).expect("json") == serde_json::to_string(&velho).expect("json"),
        "o JSON mudou"
    );
    let volta: crate::tool::persist::PaintedDocument = postcard::from_bytes(&fio).expect("lê");
    assert!(volta == novo, "o fio de `Vec`s não volta ao documento");
}

/// O formato gravado de uma imagem de camada é o do `Vec<u8>` de antes do B18.1, byte a byte
/// (postcard e JSON): o `Arc` é só partilha em memória.
#[test]
fn a_imagem_partilhada_grava_os_mesmos_bytes_que_o_vec() {
    #[derive(serde::Serialize)]
    struct Antiga {
        width: u32,
        height: u32,
        rgba8: Vec<u8>,
    }
    let px = ruido(5)[..4 * 7 * 3].to_vec();
    let nova = crate::compositor::LayerImage {
        width: 7,
        height: 3,
        rgba8: px.clone().into(),
    };
    let antiga = Antiga {
        width: 7,
        height: 3,
        rgba8: px,
    };
    assert_eq!(
        postcard::to_allocvec(&nova).unwrap(),
        postcard::to_allocvec(&antiga).unwrap()
    );
    assert_eq!(
        serde_json::to_string(&nova).unwrap(),
        serde_json::to_string(&antiga).unwrap()
    );
    let volta: crate::compositor::LayerImage =
        postcard::from_bytes(&postcard::to_allocvec(&antiga).unwrap()).unwrap();
    assert!(volta == nova);
}
