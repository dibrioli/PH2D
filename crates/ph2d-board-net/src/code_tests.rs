//! O código curto (sorteio, escrita da pessoa) e o travão da força bruta, com o relógio injectado.

use std::time::{Duration, Instant};

use crate::code::{Code, Gate};
use crate::{ALPHABET, CODE_LEN, LOCKOUT, MAX_FAILS};

#[test]
fn o_alfabeto_nao_tem_ambiguos() {
    for c in b"0O1IL" {
        assert!(!ALPHABET.contains(c), "{}", char::from(*c));
    }
    let mut v = ALPHABET.to_vec();
    v.sort_unstable();
    v.dedup();
    assert_eq!(v.len(), 31);
}

#[test]
fn o_sorteio_da_seis_simbolos_do_alfabeto_e_varia() {
    let codes: Vec<Code> = (0..200).map(|_| Code::random()).collect();
    for c in &codes {
        assert_eq!(c.bytes().len(), CODE_LEN);
        assert!(c.bytes().iter().all(|b| ALPHABET.contains(b)));
        let d = c.display();
        assert_eq!(d.len(), 7);
        assert_eq!(&d[3..4], "-");
    }
    let mut distinct: Vec<&[u8]> = codes.iter().map(Code::bytes).collect();
    distinct.sort_unstable();
    distinct.dedup();
    assert!(distinct.len() > 190, "{} de 200", distinct.len());
    // Todos os símbolos aparecem (1 200 sorteios, 31 símbolos): o resto da divisão não exclui nenhum.
    for s in ALPHABET {
        assert!(
            codes.iter().any(|c| c.bytes().contains(s)),
            "{}",
            char::from(*s)
        );
    }
}

#[test]
fn a_escrita_da_pessoa_normaliza() {
    let want = Code::parse("ABC-DEF").unwrap();
    for typed in [
        "abc-def",
        "ABCDEF",
        " abc def ",
        "a b c - d e f",
        "AbC--dEf",
        "abc\tdef",
    ] {
        assert_eq!(Code::parse(typed).as_ref(), Some(&want), "{typed:?}");
    }
    assert_eq!(want.display(), "ABC-DEF");
    for bad in [
        "", "ABCDE", "ABCDEFG", "ABC-DE0", "ABC-DEI", "ABC_DEF", "ABCDÉF", "ABC-DEL",
    ] {
        assert_eq!(Code::parse(bad), None, "{bad:?}");
    }
}

#[test]
fn cinco_errados_seguidos_bloqueiam_trinta_segundos() {
    let t0 = Instant::now();
    let mut g = Gate::default();
    for i in 0..MAX_FAILS {
        let now = t0 + Duration::from_secs(u64::from(i));
        assert_eq!(g.begin(now), Ok(()), "tentativa {i}");
        g.end(false, now);
    }
    let last = t0 + Duration::from_secs(u64::from(MAX_FAILS - 1));
    assert_eq!(g.begin(last), Err(LOCKOUT));
    assert_eq!(
        g.begin(last + LOCKOUT - Duration::from_millis(1)),
        Err(Duration::from_millis(1))
    );
    assert_eq!(g.begin(last + LOCKOUT), Ok(()), "o bloqueio acaba");
    // E recomeça a contar do zero.
    g.end(false, last + LOCKOUT);
    assert_eq!(g.begin(last + LOCKOUT), Ok(()));
}

#[test]
fn um_certo_zera_a_contagem() {
    let t = Instant::now();
    let mut g = Gate::default();
    for _ in 0..MAX_FAILS - 1 {
        g.begin(t).unwrap();
        g.end(false, t);
    }
    g.begin(t).unwrap();
    g.end(true, t);
    for _ in 0..MAX_FAILS - 1 {
        g.begin(t).unwrap();
        g.end(false, t);
    }
    assert_eq!(g.begin(t), Ok(()), "4 + certo + 4 não são 5 seguidos");
}

#[test]
fn apertos_em_curso_contam_como_palpites() {
    let t = Instant::now();
    let mut g = Gate::default();
    for _ in 0..MAX_FAILS {
        g.begin(t).unwrap();
    }
    assert!(g.begin(t).is_err(), "o 6.º em paralelo espera");
    g.abandon();
    assert_eq!(
        g.begin(t),
        Ok(()),
        "um que não testou código liberta o lugar"
    );
}
