//! A tabela das cenas despacha como as listas literais que ela substituiu (antes do corte do tecto
//! de LOC): cada escrita da variável → as mesmas chaves e o mesmo 1.º estágio.

use super::{handles_first, of};

/// As listas literais de antes do corte, escritas à mão: `(want, bars, handles, quick)`.
fn antes(s: &str) -> (u8, bool, u8, u8) {
    let want = if matches!(
        s,
        "6" | "7"
            | "9"
            | "10"
            | "11"
            | "12"
            | "13"
            | "14"
            | "15"
            | "16"
            | "17"
            | "18"
            | "19"
            | "20"
            | "21"
            | "22"
            | "23"
            | "24"
            | "25"
            | "26"
            | "27"
            | "28"
    ) {
        1
    } else {
        9
    };
    let bars = matches!(
        s,
        "9" | "10"
            | "11"
            | "12"
            | "13"
            | "14"
            | "15"
            | "16"
            | "17"
            | "18"
            | "19"
            | "20"
            | "21"
            | "22"
            | "23"
            | "24"
            | "25"
            | "26"
            | "27"
            | "28"
    );
    let handles = match s {
        "15" => 1,
        "16" => 2,
        "17" => 3,
        "18" => 4,
        "19" => 5,
        "20" => 6,
        "21" => 7,
        "22" => 8,
        "23" => 9,
        "24" => 10,
        "25" => 11,
        "26" => 12,
        "27" => 13,
        "28" => 14,
        _ => 0,
    };
    let quick = match s {
        "12" => 1,
        "13" => 2,
        _ => 0,
    };
    (want, bars, handles, quick)
}

#[test]
fn a_tabela_das_cenas_despacha_como_as_listas_de_antes() {
    let mut escritas: Vec<String> = (0..=40).map(|n| n.to_string()).collect();
    escritas.extend(["07", "+7", " 7", "", "abc", "256"].map(String::from));
    for s in &escritas {
        let c = of(Some(s));
        assert_eq!((c.want, c.bars, c.handles, c.quick), antes(s), "cena {s:?}");
        assert_eq!(c.weld, s == "7", "cena {s:?}");
        assert_eq!(c.reach, s == "10", "cena {s:?}");
        assert_eq!(c.nav, s == "11", "cena {s:?}");
        assert_eq!(c.frame, s == "14", "cena {s:?}");
    }
    let nada = of(None);
    assert_eq!(
        (nada.want, nada.bars, nada.handles, nada.quick),
        (9, false, 0, 0)
    );
}

#[test]
fn a_cena_24_comeca_no_tracejado_e_a_15_nas_alcas() {
    assert_eq!(of(Some("24")).handles, 10);
    assert_eq!(handles_first(10), Some(crate::vector_mode_tracejado::FIRST));
    assert_eq!(handles_first(9), Some(crate::vector_mode_controle::FIRST));
    assert_eq!(handles_first(8), Some(crate::vector_mode_alinhar::FIRST));
    assert_eq!(handles_first(7), Some(crate::vector_mode_quinas::FIRST));
    let fixos = [(1, 70), (2, 80), (3, 90), (4, 100), (5, 110), (6, 130)];
    for (h, first) in fixos {
        assert_eq!(handles_first(h), Some(first), "família {h}");
    }
    assert_eq!(handles_first(0), None);
}

/// ⭐ **As cenas 25 e 26 começam no mesmo 1.º estágio** (o das abas, F17) e distinguem-se pelo
/// `handles` — a 26 é a 25 com o clique no interruptor da ①.
#[test]
fn as_cenas_25_e_26_comecam_nas_abas() {
    assert_eq!((of(Some("25")).handles, of(Some("26")).handles), (11, 12));
    for h in [11, 12] {
        assert_eq!(
            handles_first(h),
            Some(crate::vector_mode_interface::FIRST),
            "família {h}"
        );
    }
}

/// ⭐ **As faixas de estágios do `smoke_step` não se cruzam.** Um braço de faixa que apanha o
/// estágio de outra cena corre-a com o passo ERRADO, e a cena pára muda (medido 16/10: a 25
/// nascia em `246`, que é o `PICK` da 24 — a 24 tem quatro formas e acaba em `247`). As faixas
/// literais são as do `match` do `smoke_step`; as dos módulos, os `FIRST`/`LAST` deles.
#[test]
fn as_faixas_de_estagios_nao_se_cruzam() {
    let mut faixas: Vec<(u8, u8, &str)> = vec![
        (1, 6, "base"),
        (9, 10, "feito/seletor"),
        (13, 16, "7"),
        (20, 21, "9"),
        (30, 32, "10"),
        (40, 41, "11"),
        (50, 56, "12/13"),
        (60, 63, "14"),
        (70, 76, "15"),
        (80, 84, "16"),
        (90, 93, "17"),
        (100, 103, "18"),
        (110, 120, "19"),
        (130, 136, "20"),
    ];
    faixas.extend([
        (
            crate::vector_mode_quinas::FIRST,
            crate::vector_mode_quinas::LAST,
            "21",
        ),
        (
            crate::vector_mode_alinhar::FIRST,
            crate::vector_mode_alinhar::LAST,
            "22",
        ),
        (
            crate::vector_mode_controle::FIRST,
            crate::vector_mode_controle::LAST,
            "23",
        ),
        (
            crate::vector_mode_tracejado::FIRST,
            crate::vector_mode_tracejado::LAST,
            "24",
        ),
        (
            crate::vector_mode_interface::FIRST,
            crate::vector_mode_interface::LAST,
            "25/26",
        ),
        (
            crate::vector_mode_vestidos::FIRST,
            crate::vector_mode_vestidos::LAST,
            "27/28",
        ),
    ]);
    faixas.sort_unstable();
    for w in faixas.windows(2) {
        let ((_, fim, a), (inicio, _, b)) = (w[0], w[1]);
        assert!(
            inicio > fim,
            "a faixa da cena {b} cruza a da cena {a}: {faixas:?}"
        );
    }
}
