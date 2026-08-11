//! **O LEITOR DO ORÁCULO** — o parser dos bits, a comparação e os acessores.
//!
//! ⚠️ **Ele mora num SUBDIRETÓRIO, e isso não é gosto:** um `.rs` solto em
//! `tests/` vira **OUTRO binário de teste** — a mesma armadilha que o
//! `physics_ecs_c9` pagou em `src/bin/`. Dentro de `tests/sculptgl_parity/` o
//! cargo não o compila sozinho, e o pai o alcança por `mod reader;`.
//!
//! O corte é por RESPONSABILIDADE: aqui mora *como o arquivo do oráculo é LIDO
//! e comparado*; no pai, *o que se AFIRMA sobre ele*. Toda adição de kernel
//! cresce o pai; esta metade só cresce quando o FORMATO muda.

use super::rk;

// ---------------------------------------------------------------------------
// O PARSER — bits, nunca decimais.
// ---------------------------------------------------------------------------

/// Um caso do oráculo: as entradas e a saída que o JS produziu.
pub struct Case {
    pub name: String,
    pub params: std::collections::BTreeMap<String, Vec<f64>>,
    pub verts: usize,
    pub in_pos: Vec<f32>,
    pub in_nrm: Vec<f32>,
    /// A máscara **na polaridade da REFERÊNCIA**: `1` é livre, `0` é travado.
    pub free: Vec<f32>,
    pub sel: Vec<u32>,
    pub out_pos: Vec<f32>,
    /// A máscara DEPOIS do kernel, na mesma polaridade. Igual à entrada em todo
    /// caso menos o `mask` — e é por isso que ela é despejada para TODOS: um
    /// campo que só existisse no caso que o move não poderia provar que os
    /// outros onze **não** o tocam.
    pub out_free: Vec<f32>,
    /// O ANEL, só na fixture de grade (o caso `smooth`) — a forma do CSR do
    /// [`ph2d_mesh::Csr::parts`], vinda do ARQUIVO e não re-derivada aqui.
    pub ring_start: Vec<u32>,
    pub ring_len: Vec<u32>,
    pub ring_values: Vec<u32>,
    pub on_edge: Vec<u8>,
}

pub struct Oracle {
    pub center: [f64; 3],
    pub radius2: f64,
    pub eye: [f64; 3],
    pub cases: Vec<Case>,
}

fn f32s(rest: &str) -> Vec<f32> {
    rest.split_whitespace()
        .map(|t| f32::from_bits(u32::from_str_radix(t, 16).expect("hex f32")))
        .collect()
}

fn u32s(rest: &str) -> Vec<u32> {
    rest.split_whitespace()
        .map(|t| t.parse().expect("u32 decimal"))
        .collect()
}

fn f64s(rest: &str) -> Vec<f64> {
    rest.split_whitespace()
        .map(|t| f64::from_bits(u64::from_str_radix(t, 16).expect("hex f64")))
        .collect()
}

pub fn load() -> Oracle {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/3D/ferramentas/sculptgl_oracle.txt"
    );
    let text = std::fs::read_to_string(path).expect("o oráculo do SculptGL tem de estar commitado");
    let mut o = Oracle {
        center: [0.0; 3],
        radius2: 0.0,
        eye: [0.0; 3],
        cases: Vec::new(),
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, rest) = line.split_once(' ').unwrap_or((line, ""));
        match key {
            "sphere" => {}
            "center" => o.center = f64s(rest).try_into().expect("center 3"),
            "radius2" => o.radius2 = f64s(rest)[0],
            "eye" => o.eye = f64s(rest).try_into().expect("eye 3"),
            "case" => o.cases.push(Case {
                name: rest.to_string(),
                params: std::collections::BTreeMap::new(),
                verts: 0,
                in_pos: Vec::new(),
                in_nrm: Vec::new(),
                free: Vec::new(),
                sel: Vec::new(),
                out_pos: Vec::new(),
                out_free: Vec::new(),
                ring_start: Vec::new(),
                ring_len: Vec::new(),
                ring_values: Vec::new(),
                on_edge: Vec::new(),
            }),
            _ => {
                let c = o.cases.last_mut().expect("um campo antes de `case`");
                match key {
                    // ⚠️ **Um `param` BOOLEANO chega como `1`/`0` decimal**, e
                    // atravessa o mesmo `f64s` — `"1"` vira `f64::from_bits(1)`,
                    // um subnormal minúsculo, e `"0"` vira `0.0` exato. Os
                    // consumidores perguntam `!= 0.0`, então a distinção é
                    // exata; o que NÃO se pode fazer com um desses é
                    // aritmética, e é por isso que a nota está aqui em vez de
                    // no sítio de leitura.
                    "param" => {
                        let (k, v) = rest.split_once(' ').expect("param <k> <v>");
                        c.params.insert(k.to_string(), f64s(v));
                    }
                    "verts" => c.verts = rest.parse().expect("verts"),
                    "in.pos" => c.in_pos = f32s(rest),
                    "in.nrm" => c.in_nrm = f32s(rest),
                    "in.mask" => c.free = f32s(rest),
                    "sel" => {
                        c.sel = rest
                            .split_whitespace()
                            .map(|t| t.parse().expect("índice"))
                            .collect();
                    }
                    "out.pos" => c.out_pos = f32s(rest),
                    "out.mask" => c.out_free = f32s(rest),
                    "ring.start" => c.ring_start = u32s(rest),
                    "ring.len" => c.ring_len = u32s(rest),
                    "ring.values" => c.ring_values = u32s(rest),
                    "ring.onedge" => {
                        c.on_edge = rest
                            .split_whitespace()
                            .map(|t| t.parse().expect("flag de borda"))
                            .collect();
                    }
                    other => panic!("campo desconhecido no oráculo: {other}"),
                }
            }
        }
    }
    assert!(!o.cases.is_empty(), "o oráculo veio vazio");
    o
}

// ---------------------------------------------------------------------------
// A COMPARAÇÃO
// ---------------------------------------------------------------------------

/// Compara bit a bit e devolve uma frase útil quando diverge.
///
/// ⚠️ **Ele conta os DIVERGENTES e mede o PIOR, e as duas perguntas são
/// diferentes** — a lição que o gate de paridade da luz do impasto pagou: um
/// limite só de magnitude deixou passar 2375 bytes errados por um nível.
pub fn assert_bit_identical(name: &str, got: &[f32], want: &[f32]) {
    assert_eq!(got.len(), want.len(), "[{name}] comprimentos diferentes");
    let mut diff = 0usize;
    let mut worst = 0.0f64;
    let mut first = None;
    for (i, (&g, &w)) in got.iter().zip(want).enumerate() {
        if g.to_bits() != w.to_bits() {
            diff += 1;
            let d = (f64::from(g) - f64::from(w)).abs();
            if d > worst {
                worst = d;
            }
            if first.is_none() {
                first = Some((i, g, w));
            }
        }
    }
    assert!(
        diff == 0,
        "[{name}] {diff} de {} componentes divergem dos bits do SculptGL \
         (pior delta absoluto {worst:.3e}); o primeiro é o índice {:?}",
        got.len(),
        first
    );
}

/// A fixture do caso, pronta para ser mexida.
pub fn scratch(c: &Case) -> Vec<f32> {
    c.in_pos.clone()
}

pub fn front_with(c: &Case, eye: [f64; 3]) -> Vec<u32> {
    let mut out = Vec::new();
    rk::front_vertices(&c.in_nrm, &c.sel, eye, &mut out);
    out
}

pub fn case<'a>(o: &'a Oracle, name: &str) -> &'a Case {
    o.cases
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("o oráculo não tem o caso `{name}`"))
}

/// O centro deste caso — o do arquivo, ou o que o caso sobrescreveu.
pub fn center_of(o: &Oracle, c: &Case) -> [f64; 3] {
    c.params
        .get("center")
        .map_or(o.center, |v| [v[0], v[1], v[2]])
}

/// O olho deste caso — idem. Só o caso da terminadora o sobrescreve.
pub fn eye_of(o: &Oracle, c: &Case) -> [f64; 3] {
    c.params.get("eye").map_or(o.eye, |v| [v[0], v[1], v[2]])
}
