//! O enquadramento e a cifra, sem rede.

use std::io::{Cursor, ErrorKind};

use crate::wire::{MAX_SHAKE, Open, Seal, TAG, frame, read_frame};

const K1: [u8; 32] = [7; 32];
const K2: [u8; 32] = [9; 32];

#[test]
fn enquadramento_ida_e_volta() {
    let mut fio = Vec::new();
    for b in [&b""[..], b"a", &[0xAB; 1000][..], &vec![3u8; 300_000][..]] {
        fio.extend(frame(b));
    }
    let mut r = Cursor::new(fio);
    assert_eq!(read_frame(&mut r, 1 << 20).unwrap(), b"");
    assert_eq!(read_frame(&mut r, 1 << 20).unwrap(), b"a");
    assert_eq!(read_frame(&mut r, 1 << 20).unwrap(), vec![0xAB; 1000]);
    let big = read_frame(&mut r, 1 << 20).unwrap();
    assert_eq!(big, vec![3u8; 300_000]);
    assert!(
        big.capacity() <= 300_000,
        "a memória nunca passa do anunciado"
    );
    assert_eq!(
        read_frame(&mut r, 1 << 20).unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );
}

#[test]
fn quadro_acima_do_tecto_recusado_sem_alocar() {
    let mut r = Cursor::new(frame(&[0; MAX_SHAKE + 1]));
    assert_eq!(
        read_frame(&mut r, MAX_SHAKE).unwrap_err().kind(),
        ErrorKind::InvalidData
    );
    // Um cabeçalho mentiroso (4 GiB) sem bytes atrás: recusado pelo tecto, ou fim do fio — nunca
    // 4 GiB alocados.
    let mut r = Cursor::new(u32::MAX.to_le_bytes().to_vec());
    assert_eq!(
        read_frame(&mut r, usize::MAX).unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );
}

#[test]
fn quadro_partido_recusado() {
    let mut fio = frame(b"0123456789");
    fio.truncate(fio.len() - 3);
    let mut r = Cursor::new(fio);
    assert_eq!(
        read_frame(&mut r, 100).unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );
    let mut r = Cursor::new(vec![5, 0]);
    assert_eq!(
        read_frame(&mut r, 100).unwrap_err().kind(),
        ErrorKind::UnexpectedEof
    );
}

#[test]
fn cifra_ida_e_volta() {
    let (mut s, mut o) = (Seal::new(&K1), Open::new(&K1));
    let msgs: Vec<Vec<u8>> = vec![vec![], b"ola".to_vec(), vec![0x5A; 25_000]];
    let mut fio = Vec::new();
    for m in &msgs {
        let w = s.seal(m).unwrap();
        assert_eq!(w.len(), 4 + m.len() + TAG);
        if !m.is_empty() {
            assert_ne!(&w[4..4 + m.len()], &m[..], "em claro no fio");
        }
        fio.extend(w);
    }
    let mut r = Cursor::new(fio);
    for m in &msgs {
        assert_eq!(&o.read(&mut r).unwrap(), m);
    }
}

#[test]
fn um_byte_trocado_no_fio_nunca_entrega_lixo() {
    let plain = vec![0x11u8; 64];
    let clean = Seal::new(&K1).seal(&plain).unwrap();
    // Cada posição do texto cifrado e da etiqueta.
    for i in 4..clean.len() {
        let mut w = clean.clone();
        w[i] ^= 0x01;
        let e = Open::new(&K1).read(&mut Cursor::new(w)).unwrap_err();
        assert_eq!(e.kind(), ErrorKind::InvalidData, "byte {i}");
    }
    // Chave da outra direcção (ou de outro código): não abre.
    assert!(
        Open::new(&K2)
            .read(&mut Cursor::new(clean.clone()))
            .is_err()
    );
    assert_eq!(Open::new(&K1).read(&mut Cursor::new(clean)).unwrap(), plain);
}

#[test]
fn os_nonces_nao_se_repetem() {
    let mut s = Seal::new(&K1);
    let a = s.seal(b"mesmo texto").unwrap();
    let b = s.seal(b"mesmo texto").unwrap();
    assert_ne!(
        a, b,
        "o mesmo texto duas vezes sai diferente: o nonce andou"
    );
    // Repetido ou fora de ordem não abre: o receptor conta como o emissor.
    let mut o = Open::new(&K1);
    assert!(
        o.read(&mut Cursor::new(b.clone())).is_err(),
        "o 2.º no lugar do 1.º"
    );
    let mut o = Open::new(&K1);
    o.read(&mut Cursor::new(a.clone())).unwrap();
    assert!(o.read(&mut Cursor::new(a)).is_err(), "o 1.º repetido");
}

#[test]
fn o_contador_esgota_antes_de_dar_a_volta() {
    let mut s = Seal::new(&K1);
    s.set_counter(u64::MAX - 1);
    assert!(s.seal(b"x").is_ok());
    assert_eq!(s.seal(b"x").unwrap_err().kind(), ErrorKind::InvalidData);
}

#[test]
fn quadro_cifrado_acima_do_tecto_recusado_ao_selar() {
    let mut s = Seal::new(&K1);
    let big = vec![0u8; crate::MAX_FRAME + 1];
    assert!(s.seal(&big).is_err());
}
