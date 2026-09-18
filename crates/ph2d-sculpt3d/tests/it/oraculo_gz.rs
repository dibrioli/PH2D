//! **DESCOMPRIMIR um `.gz` do corpus, sem sair da árvore.**
//!
//! ⚠️ **Ele é um módulo PRÓPRIO porque havia DUAS cópias desta função nesta
//! crate** (a bancada do pincel afiado e a do pente) e mais oito noutras. *Uma
//! lei escrita em dois sítios ainda não é uma lei — só uma porta é.* ⛔ As outras
//! oito vivem noutras crates e não se alcançam daqui: é dívida NOMEADA, não
//! esquecida.
//!
//! ⚠️ **`inflate` à mão e não uma dependência:** um corpus de oráculo é lido por
//! testes, e acrescentar uma crate de compressão à árvore para os ler poria uma
//! dependência nova no caminho de toda build.

/// Descomprime um `.gz` inteiro em memória e devolve o texto.
///
/// ⚠️ **Reconhece o formato pelos DOIS BYTES MÁGICOS e nunca pela extensão** —
/// a mesma lei que a vassoura da parede pagou em 14/09.
pub fn inflar(caminho: &std::path::Path) -> String {
    let raw = std::fs::read(caminho).unwrap_or_else(|e| panic!("{}: {e}", caminho.display()));
    assert!(
        raw.len() > 18 && raw[0] == 0x1f && raw[1] == 0x8b,
        "{}: nao e' gzip",
        caminho.display()
    );
    let flg = raw[3];
    let mut off = 10usize;
    if flg & 0x04 != 0 {
        off += 2 + (usize::from(raw[off]) | (usize::from(raw[off + 1]) << 8));
    }
    for bit in [0x08u8, 0x10] {
        if flg & bit != 0 {
            while raw[off] != 0 {
                off += 1;
            }
            off += 1;
        }
    }
    if flg & 0x02 != 0 {
        off += 2;
    }
    String::from_utf8(inflate(&raw[off..])).expect("o corpus e' texto")
}

/// DEFLATE cru (RFC 1951) — blocos armazenados, fixos e dinâmicos.
fn inflate(dados: &[u8]) -> Vec<u8> {
    let mut bits = Bits {
        dados,
        pos: 0,
        bit: 0,
    };
    let mut saida = Vec::new();
    loop {
        let fim = bits.ler(1) == 1;
        match bits.ler(2) {
            0 => {
                bits.alinhar();
                let n = bits.ler(16) as usize;
                let _ = bits.ler(16);
                for _ in 0..n {
                    saida.push(bits.byte());
                }
            }
            1 => {
                let (lit, dist) = arvores_fixas();
                bloco(&mut bits, &lit, &dist, &mut saida);
            }
            2 => {
                let (lit, dist) = arvores_dinamicas(&mut bits);
                bloco(&mut bits, &lit, &dist, &mut saida);
            }
            _ => panic!("bloco DEFLATE reservado"),
        }
        if fim {
            break;
        }
    }
    saida
}

struct Bits<'a> {
    dados: &'a [u8],
    pos: usize,
    bit: u32,
}

impl Bits<'_> {
    fn ler(&mut self, n: u32) -> u32 {
        let mut v = 0u32;
        for k in 0..n {
            let b = (u32::from(self.dados[self.pos]) >> self.bit) & 1;
            v |= b << k;
            self.bit += 1;
            if self.bit == 8 {
                self.bit = 0;
                self.pos += 1;
            }
        }
        v
    }
    fn alinhar(&mut self) {
        if self.bit != 0 {
            self.bit = 0;
            self.pos += 1;
        }
    }
    fn byte(&mut self) -> u8 {
        let b = self.dados[self.pos];
        self.pos += 1;
        b
    }
}

/// Uma árvore de Huffman canónica: `(comprimentos, símbolos)` por código.
struct Arvore {
    contagem: Vec<u16>,
    simbolos: Vec<u16>,
}

fn arvore(comprimentos: &[u8]) -> Arvore {
    let mut contagem = vec![0u16; 16];
    for &c in comprimentos {
        contagem[c as usize] += 1;
    }
    contagem[0] = 0;
    let mut offs = [0u16; 16];
    for i in 1..15 {
        offs[i + 1] = offs[i] + contagem[i];
    }
    let mut simbolos = vec![0u16; comprimentos.len()];
    for (s, &c) in comprimentos.iter().enumerate() {
        if c != 0 {
            simbolos[offs[c as usize] as usize] = u16::try_from(s).expect("simbolo");
            offs[c as usize] += 1;
        }
    }
    Arvore { contagem, simbolos }
}

fn descodificar(bits: &mut Bits, a: &Arvore) -> u16 {
    let (mut codigo, mut primeiro, mut indice) = (0i32, 0i32, 0i32);
    for c in 1..16usize {
        codigo |= i32::try_from(bits.ler(1)).expect("bit");
        let conta = i32::from(a.contagem[c]);
        if codigo - conta < primeiro {
            return a.simbolos[usize::try_from(indice + (codigo - primeiro)).expect("indice")];
        }
        indice += conta;
        primeiro = (primeiro + conta) << 1;
        codigo <<= 1;
    }
    panic!("codigo de Huffman invalido");
}

fn arvores_fixas() -> (Arvore, Arvore) {
    let mut lit = vec![8u8; 288];
    lit[144..256].fill(9);
    lit[256..280].fill(7);
    (arvore(&lit), arvore(&[5u8; 30]))
}

fn arvores_dinamicas(bits: &mut Bits) -> (Arvore, Arvore) {
    let hlit = bits.ler(5) as usize + 257;
    let hdist = bits.ler(5) as usize + 1;
    let hclen = bits.ler(4) as usize + 4;
    const ORDEM: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let mut cl = [0u8; 19];
    for &o in ORDEM.iter().take(hclen) {
        cl[o] = u8::try_from(bits.ler(3)).expect("comprimento");
    }
    let arv_cl = arvore(&cl);
    let mut comprimentos = Vec::with_capacity(hlit + hdist);
    while comprimentos.len() < hlit + hdist {
        let s = descodificar(bits, &arv_cl);
        match s {
            0..=15 => comprimentos.push(u8::try_from(s).expect("comprimento")),
            16 => {
                let anterior = *comprimentos.last().expect("repeticao sem anterior");
                for _ in 0..3 + bits.ler(2) {
                    comprimentos.push(anterior);
                }
            }
            17 => {
                let n = 3 + bits.ler(3) as usize;
                comprimentos.resize(comprimentos.len() + n, 0);
            }
            _ => {
                let n = 11 + bits.ler(7) as usize;
                comprimentos.resize(comprimentos.len() + n, 0);
            }
        }
    }
    (arvore(&comprimentos[..hlit]), arvore(&comprimentos[hlit..]))
}

fn bloco(bits: &mut Bits, lit: &Arvore, dist: &Arvore, saida: &mut Vec<u8>) {
    const COMP: [u16; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const COMP_EXTRA: [u32; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DIST: [u16; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    const DIST_EXTRA: [u32; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];
    loop {
        let s = descodificar(bits, lit);
        match s {
            0..=255 => saida.push(u8::try_from(s).expect("literal")),
            256 => return,
            _ => {
                let i = usize::from(s) - 257;
                let comprimento = usize::from(COMP[i]) + bits.ler(COMP_EXTRA[i]) as usize;
                let d = usize::from(descodificar(bits, dist));
                let distancia = usize::from(DIST[d]) + bits.ler(DIST_EXTRA[d]) as usize;
                let inicio = saida.len() - distancia;
                for k in 0..comprimento {
                    saida.push(saida[inicio + k]);
                }
            }
        }
    }
}
