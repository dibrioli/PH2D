//! ⭐ **A sonda intercalada DENTRO do app** (`PH2D_SONDA=<n>`, [MEDIR_VELOCIDADE.md](../../../docs/DevOps/MEDIR_VELOCIDADE.md)):
//! `n` variantes do MESMO processo, escolhidas em execução por quem decide ([`variante`] é um índice; cada
//! consumidor lê o SEU bit), em blocos de [`BLOCO`] quadros por ordem RODADA, depois de [`AQUECE`] quadros.
//! Por bloco: o relógio da placa ([`crate::pass_profiler::drain`], com `PH2D_FLUID_PROFILE=1`), as notas de
//! CPU ([`nota_cpu`]) e a parede do quadro. No fim, o MÍNIMO e a mediana por variante, e o processo sai.
//!
//! `PH2D_SONDA_RODADAS` (7 por omissão) · `PH2D_SONDA_NOMES=a,b,…` (o rótulo de cada variante no relatório).

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// Quadros de cada bloco — abaixo da janela de 120 do `pass_profiler`, que de outro modo zeraria o acumulado.
pub const BLOCO: u32 = 30;
/// Quadros antes da 1.ª medida: a cena assenta e os shaders compilam fora da régua.
pub const AQUECE: u32 = 180;

/// A ordem das variantes na rodada `r`: a inicial roda a cada rodada.
#[must_use]
pub fn ordem(r: u32, n: u32) -> Vec<u32> {
    (0..n).map(|i| (r + i) % n).collect()
}

/// `(mínimo, mediana)` de amostras não vazias.
#[must_use]
pub fn resumo(amostras: &[f64]) -> Option<(f64, f64)> {
    let mut v = amostras.to_vec();
    v.sort_by(f64::total_cmp);
    Some((*v.first()?, v[v.len() / 2]))
}

/// O índice da variante a correr neste quadro; `None` com a sonda desligada.
#[must_use]
pub fn variante() -> Option<u32> {
    estado().map(|e| e.lock().map_or(0, |e| e.atual()))
}

/// Soma `ns` de CPU sob `rotulo` no bloco em curso (fora do aquecimento).
pub fn nota_cpu(rotulo: &'static str, ns: u64) {
    if let Some(Ok(mut e)) = estado().map(Mutex::lock)
        && e.quadro >= AQUECE
    {
        let c = e.cpu.entry(rotulo).or_default();
        c.0 += ns;
        c.1 += 1;
    }
}

struct Estado {
    n: u32,
    rodadas: u32,
    nomes: Vec<String>,
    quadro: u32,
    inicio_do_bloco: Option<Instant>,
    cpu: BTreeMap<&'static str, (u64, u32)>,
    /// Por variante, por rótulo: ms por quadro de cada bloco.
    amostras: Vec<BTreeMap<String, Vec<f64>>>,
    carga_inicial: String,
}

impl Estado {
    fn bloco(&self) -> Option<u32> {
        self.quadro.checked_sub(AQUECE).map(|k| k / BLOCO)
    }

    fn atual(&self) -> u32 {
        self.bloco()
            .map_or(0, |b| ordem(b / self.n, self.n)[(b % self.n) as usize])
    }
}

fn estado() -> Option<&'static Mutex<Estado>> {
    static E: OnceLock<Option<Mutex<Estado>>> = OnceLock::new();
    E.get_or_init(|| {
        let n: u32 = std::env::var("PH2D_SONDA")
            .ok()?
            .trim()
            .parse()
            .ok()
            .filter(|&n| n > 0)?;
        let rodadas = std::env::var("PH2D_SONDA_RODADAS")
            .ok()
            .and_then(|r| r.trim().parse().ok())
            .unwrap_or(7);
        let nomes = std::env::var("PH2D_SONDA_NOMES").map_or_else(
            |_| (0..n).map(|v| format!("v{v}")).collect(),
            |s| s.split(',').map(str::to_owned).collect(),
        );
        Some(Mutex::new(Estado {
            n,
            rodadas,
            nomes,
            quadro: 0,
            inicio_do_bloco: None,
            cpu: BTreeMap::new(),
            amostras: (0..n).map(|_| BTreeMap::new()).collect(),
            carga_inicial: carga(),
        }))
    })
    .as_ref()
}

fn carga() -> String {
    std::fs::read_to_string("/proc/loadavg")
        .map(|s| s.split_whitespace().take(3).collect::<Vec<_>>().join(" "))
        .unwrap_or_default()
}

/// O fim de um quadro (chamado por [`crate::pass_profiler::end_frame`], depois de resolver as consultas).
pub(crate) fn depois_do_quadro(device: &wgpu::Device) {
    let Some(Ok(mut e)) = estado().map(Mutex::lock) else {
        return;
    };
    e.quadro += 1;
    let Some(k) = e.quadro.checked_sub(AQUECE) else {
        return;
    };
    if k % BLOCO != 0 {
        return;
    }
    // A fronteira de um bloco: o que se juntou desde a anterior é da variante que acabou de correr.
    let gpu = crate::pass_profiler::drain(device);
    let agora = Instant::now();
    if let Some(t0) = e.inicio_do_bloco.replace(agora) {
        e.quadro -= 1;
        let v = e.atual() as usize;
        e.quadro += 1;
        let q = f64::from(BLOCO);
        let mut linha: Vec<(String, f64)> = vec![(
            "quadro.parede".into(),
            agora.duration_since(t0).as_secs_f64() * 1e3 / q,
        )];
        if let Some(g) = gpu {
            linha.push(("gpu.total".into(), g.span_ns as f64 / 1e6 / q));
            linha.extend(
                g.per_label
                    .iter()
                    .map(|(l, ns, _)| ((*l).to_owned(), *ns as f64 / 1e6 / q)),
            );
        }
        let cpu = std::mem::take(&mut e.cpu);
        for (l, (ns, vezes)) in cpu {
            linha.push((format!("cpu.{l}"), ns as f64 / 1e6 / q));
            linha.push((format!("cpu.{l}.vezes"), f64::from(vezes) / q));
        }
        for (l, ms) in linha {
            e.amostras[v].entry(l).or_default().push(ms);
        }
    } else {
        e.cpu.clear();
    }
    if k / BLOCO >= e.n * e.rodadas {
        relata(&e);
        std::process::exit(0);
    }
}

fn relata(e: &Estado) {
    eprintln!(
        "[sonda] {} variantes × {} rodadas × {BLOCO} quadros · load {} → {} · ms/quadro: mínimo (mediana)",
        e.n,
        e.rodadas,
        e.carga_inicial,
        carga()
    );
    let mut rotulos: Vec<&String> = e.amostras.iter().flat_map(BTreeMap::keys).collect();
    rotulos.sort();
    rotulos.dedup();
    for l in rotulos {
        let mut s = format!("[sonda] {l:<28}");
        for (v, a) in e.amostras.iter().enumerate() {
            let nome = e.nomes.get(v).map_or("?", String::as_str);
            match a.get(l).and_then(|x| resumo(x)) {
                Some((min, med)) => s.push_str(&format!(" {nome}={min:.3} ({med:.3})")),
                None => s.push_str(&format!(" {nome}=—")),
            }
        }
        eprintln!("{s}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cada_rodada_comeca_pela_variante_seguinte() {
        assert_eq!(ordem(0, 4), [0, 1, 2, 3]);
        assert_eq!(ordem(1, 4), [1, 2, 3, 0]);
        assert_eq!(ordem(5, 4), [1, 2, 3, 0]);
    }

    #[test]
    fn o_resumo_e_o_minimo_com_a_mediana() {
        assert_eq!(resumo(&[3.0, 1.0, 2.0]), Some((1.0, 2.0)));
        assert_eq!(resumo(&[]), None);
    }

    #[test]
    fn o_aquecimento_corre_a_variante_zero_e_os_blocos_rodam() {
        let mut e = Estado {
            n: 2,
            rodadas: 1,
            nomes: Vec::new(),
            quadro: 0,
            inicio_do_bloco: None,
            cpu: BTreeMap::new(),
            amostras: Vec::new(),
            carga_inicial: String::new(),
        };
        assert_eq!(e.atual(), 0);
        let em = |e: &mut Estado, bloco: u32| {
            e.quadro = AQUECE + bloco * BLOCO;
            e.atual()
        };
        assert_eq!(
            [em(&mut e, 0), em(&mut e, 1), em(&mut e, 2), em(&mut e, 3)],
            [0, 1, 1, 0]
        );
    }
}
