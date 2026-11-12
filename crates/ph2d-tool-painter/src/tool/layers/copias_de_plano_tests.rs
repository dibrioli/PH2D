//! B18.1/B18.2: trocar, duplicar e colher o documento copiam cada plano inteiro em BLOCOS paralelos
//! ([`crate::plane_copy::par_clone`]) — os mesmos bytes que a cópia serial, e a duplicata continua
//! independente da origem (cópia na escrita).

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

fn copiou(sitios: &[&str], ficheiro: &str) -> bool {
    sitios.iter().any(|f| f.ends_with(ficheiro))
}

/// Os píxeis de `id` onde quer que vivam (a tela, se é a activa).
fn pixeis(t: &PainterTool, id: RtLayerId) -> Vec<u8> {
    if t.layers.active() == Some(id) {
        t.canvas_rgba.as_ref().clone()
    } else {
        t.images[&id].rgba8.clone()
    }
}

/// Trocar de camada leva cada plano ao bit, e a troca copia pela rota em blocos (a do `flush` e a da
/// posse do plano que o undo partilha): o contador sobe — uma cópia serial de volta reprova aqui.
#[test]
fn trocar_de_camada_copia_em_blocos_e_guarda_os_pixeis_ao_bit() {
    let (mut t, a, b) = documento();
    let _ = copiaram_em_blocos();
    t.select_layer(a);
    let sitios = copiaram_em_blocos();
    assert!(
        copiou(&sitios, "layers/mutate.rs"),
        "o flush voltou à cópia serial: {sitios:?}"
    );
    assert!(
        copiou(&sitios, "tool/internal.rs"),
        "a posse voltou à cópia serial: {sitios:?}"
    );
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
    assert!(
        copiou(&sitios, "layers/estrutura.rs"),
        "a duplicata voltou à cópia serial: {sitios:?}"
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

/// O documento que a gravação colhe é o da cópia serial antiga, campo a campo e byte a byte no fio
/// (postcard) — com relevo, cobertura, material e vidro presentes.
#[test]
fn o_documento_colhido_e_o_da_copia_serial_ao_byte() {
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
    let sitios = copiaram_em_blocos();
    let em_persist = sitios
        .iter()
        .filter(|f| f.ends_with("tool/persist.rs"))
        .count();
    // tela + relevo + material + vidro; a cobertura (`u8`, 2 MB) fica abaixo do limiar e copia-se em
    // série pela mesma porta; as imagens copiam-se em `internal.rs`.
    assert_eq!(
        em_persist, 4,
        "a colheita voltou à cópia serial: {sitios:?}"
    );
    assert!(copiou(&sitios, "tool/internal.rs"));
    // ORÁCULO — o corpo de antes do B18.2: `as_ref().clone()` de cada plano.
    let serial = |t: &PainterTool| crate::tool::persist::PaintedDocument {
        id: 0,
        layers: t.layers.clone(),
        canvas_rgba: t.canvas_rgba.as_ref().clone(),
        images: t
            .images
            .iter()
            .map(|(k, v)| (*k, v.as_ref().clone()))
            .collect(),
        heights: t
            .heights
            .iter()
            .map(|(k, v)| (*k, v.as_ref().clone()))
            .collect(),
        covers: t
            .covers
            .iter()
            .map(|(k, v)| (*k, v.as_ref().clone()))
            .collect(),
        mats: t
            .mats
            .iter()
            .map(|(k, v)| (*k, v.as_ref().clone()))
            .collect(),
        size: t.source_size,
        papel: t.papel,
        vidros: t
            .vidros
            .iter()
            .map(|(k, v)| (*k, v.as_ref().clone()))
            .collect(),
        tintas_molhadas: t.tintas_molhadas.clone(),
        fator_aparencia: t.fator_aparencia,
    };
    let velho = serial(&t);
    assert!(novo == velho, "o documento colhido mudou");
    assert_eq!(
        postcard::to_allocvec(&novo).expect("postcard"),
        postcard::to_allocvec(&velho).expect("postcard"),
    );
}
