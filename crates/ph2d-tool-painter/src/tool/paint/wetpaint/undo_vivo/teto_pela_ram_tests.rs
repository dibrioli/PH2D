//! B42.3: o tecto do undo vivo do Wet deriva da RAM física (o piso são os seis orçamentos do documento).

use super::{orcamento_das_fotos, teto_da_historia_em};

const GIB: u64 = 1 << 30;

#[test]
fn o_teto_do_undo_vivo_deriva_da_ram() {
    let (w, h) = (3840, 2160);
    let piso = 6 * orcamento_das_fotos(w, h);
    assert_eq!(piso, 3_203_137_536, "o piso a 4K");
    assert_eq!(teto_da_historia_em(w, h, None), piso, "sem leitura: o piso");
    assert_eq!(
        teto_da_historia_em(w, h, Some(8 * GIB)),
        piso,
        "8 GiB: o piso"
    );
    assert_eq!(
        teto_da_historia_em(w, h, Some(32 * GIB)),
        piso,
        "32 GiB: o piso"
    );
    assert_eq!(
        teto_da_historia_em(w, h, Some(123 * GIB)),
        (123 * GIB / 32) as usize,
        "123 GiB: RAM/32"
    );
    assert!(
        teto_da_historia_em(w, h, Some(123 * GIB)) > piso,
        "a máquina do dono sobe o tecto"
    );
}
