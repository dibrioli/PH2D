//! ⭐⭐ **O OBSTÁCULO DECLARADO AO MUNDO DE CONTACTO** (doc 121 §9.20, ponto 5) — o aperto de mão em
//! colunas entre o `sim.collide` e o `sim.step` (os systems não se chamam — ADR-0075).
//!
//! Medido no oráculo: com a taça FORA do mundo (projectada depois do passo) o peso do monte não
//! passa pelo solver e a pilha fica `8×` mais agitada. ⇒ o `sim.collide` DECLARA o seu obstáculo e
//! o `sim.step` põe-no no mundo como colisor fixo:
//!
//! 1. o `sim.collide` escreve [`declara`] (a forma, o atrito e o salto POR PEÇA — o `Randomness`);
//! 2. o `sim.step` lê [`declarados`], CONSOME as colunas (nunca as deixa a circular: um `sim.collide`
//!    apagado tira o obstáculo do mundo no passo seguinte) e devolve o [`passa_recibo`];
//! 3. o `sim.collide` que lê [`tem_recibo`] não projecta as peças que o mundo resolve.
//!
//! Sem recibo — um `sim.collide` fora da zona, ou o 1.º passo — ele projecta como sempre.
//!
//! ⚠️ As colunas repetem o MESMO obstáculo em cada linha (o stream só tem colunas por peça): é o
//! salto por peça que o exige, e o resto custa `40` bytes por peça.

use ph2d_nodegraph::attr::{Column, Stream};

/// O prefixo de toda coluna desta porta — quem copia colunas de um stream para outro e possui o
/// contacto (o `sim.step`) salta-as por ele.
pub const PREFIXO: &str = "obstaculo:";

/// A geometria do obstáculo, no MUNDO.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FormaFixa {
    /// As peças ficam do lado da `normal`, acima de `altura` (`p · normal ≥ altura`).
    Plano { normal: [f32; 2], altura: f32 },
    /// As peças ficam FORA do disco.
    Disco { centro: [f32; 2], raio: f32 },
    /// As peças ficam DENTRO do círculo.
    Taca { centro: [f32; 2], raio: f32 },
    /// As peças ficam fora da caixa; `eixo` é a direcção da meia-largura `meia[0]`.
    Caixa {
        centro: [f32; 2],
        meia: [f32; 2],
        eixo: [f32; 2],
    },
}

/// Um obstáculo declarado: a forma e o atrito dele (o salto vem por peça, ver [`declarados`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Obstaculo {
    pub forma: FormaFixa,
    pub atrito: f32,
    /// O salto do obstáculo contra CADA peça (o `Randomness` do cartão varia-o por peça).
    pub salto: Vec<f32>,
}

fn nome(chave: u32) -> String {
    format!("{PREFIXO}{chave}")
}
fn nome_eixo(chave: u32) -> String {
    format!("{PREFIXO}{chave}:eixo")
}
fn nome_material(chave: u32) -> String {
    format!("{PREFIXO}{chave}:material")
}
fn nome_recibo(chave: u32) -> String {
    format!("{PREFIXO}{chave}:recibo")
}

/// O `sim.collide` de chave `chave` declara o seu obstáculo em `out` (`salto.len()` = as peças).
pub fn declara(out: &mut Stream, chave: u32, forma: FormaFixa, atrito: f32, salto: &[f32]) {
    let n = out.count();
    if salto.len() != n {
        return;
    }
    let (a, e) = match forma {
        FormaFixa::Plano { normal, altura } => {
            ([0.0, 0.0, 0.0, altura], [normal[0], normal[1], 0.0, 0.0])
        }
        FormaFixa::Disco { centro, raio } => ([1.0, centro[0], centro[1], raio], [0.0; 4]),
        FormaFixa::Taca { centro, raio } => ([2.0, centro[0], centro[1], raio], [0.0; 4]),
        FormaFixa::Caixa { centro, meia, eixo } => (
            [3.0, centro[0], centro[1], 0.0],
            [eixo[0], eixo[1], meia[0], meia[1]],
        ),
    };
    out.set(nome(chave), Column::Vec4(vec![a; n]));
    out.set(nome_eixo(chave), Column::Vec4(vec![e; n]));
    out.set(
        nome_material(chave),
        Column::Vec2(salto.iter().map(|s| [atrito, *s]).collect()),
    );
}

/// Os obstáculos declarados em `s`, pela ordem da chave — vazio sem peças.
#[must_use]
pub fn declarados(s: &Stream) -> Vec<(u32, Obstaculo)> {
    let n = s.count();
    if n == 0 {
        return Vec::new();
    }
    let mut chaves: Vec<u32> = s
        .columns()
        .filter_map(|(k, _)| k.strip_prefix(PREFIXO)?.parse::<u32>().ok())
        .collect();
    chaves.sort_unstable();
    chaves
        .into_iter()
        .filter_map(|chave| {
            let (Some(Column::Vec4(a)), Some(Column::Vec4(e)), Some(Column::Vec2(m))) = (
                s.get(&nome(chave)),
                s.get(&nome_eixo(chave)),
                s.get(&nome_material(chave)),
            ) else {
                return None;
            };
            let (a, e) = (*a.first()?, *e.first()?);
            if m.len() != n {
                return None;
            }
            #[expect(clippy::cast_possible_truncation, reason = "o código da forma é 0..=3")]
            let forma = match a[0].round() as i32 {
                0 => FormaFixa::Plano {
                    normal: [e[0], e[1]],
                    altura: a[3],
                },
                1 => FormaFixa::Disco {
                    centro: [a[1], a[2]],
                    raio: a[3],
                },
                2 => FormaFixa::Taca {
                    centro: [a[1], a[2]],
                    raio: a[3],
                },
                3 => FormaFixa::Caixa {
                    centro: [a[1], a[2]],
                    meia: [e[2], e[3]],
                    eixo: [e[0], e[1]],
                },
                _ => return None,
            };
            let valido = a.iter().chain(&e).all(|x| x.is_finite());
            valido.then(|| {
                (
                    chave,
                    Obstaculo {
                        forma,
                        atrito: m[0][0],
                        salto: m.iter().map(|x| x[1]).collect(),
                    },
                )
            })
        })
        .collect()
}

/// O `sim.step` diz ao `sim.collide` de chave `chave` que o obstáculo dele está no mundo.
pub fn passa_recibo(out: &mut Stream, chave: u32) {
    let n = out.count();
    out.set(nome_recibo(chave), Column::Scalar(vec![1.0; n]));
}

/// O mundo de contacto resolve as peças contra o obstáculo `chave`?
#[must_use]
pub fn tem_recibo(s: &Stream, chave: u32) -> bool {
    matches!(s.get(&nome_recibo(chave)), Some(Column::Scalar(v)) if v.len() == s.count())
}

/// Uma coluna da declaração ou do recibo do obstáculo `chave` — o `sim.collide` dessa chave
/// consome-as (re-declara a cada passagem e lê o recibo uma vez).
#[must_use]
pub fn e_desta_chave(nome_da_coluna: &str, chave: u32) -> bool {
    nome_da_coluna
        .strip_prefix(PREFIXO)
        .and_then(|r| r.split(':').next())
        .is_some_and(|k| k.parse::<u32>().ok() == Some(chave))
}

/// Uma coluna desta porta (declaração ou recibo) — o `sim.step` e o `sim.collide` CONSOMEM-nas.
#[must_use]
pub fn e_da_porta(nome_da_coluna: &str) -> bool {
    nome_da_coluna.starts_with(PREFIXO)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_declaracao_volta_inteira_pela_ordem_da_chave() {
        let mut s = Stream::new(3);
        let taca = FormaFixa::Taca {
            centro: [0.5, -1.2],
            raio: 1.8,
        };
        let caixa = FormaFixa::Caixa {
            centro: [1.0, 2.0],
            meia: [0.3, 0.4],
            eixo: [0.6, 0.8],
        };
        declara(&mut s, 9, caixa, 0.2, &[0.1, 0.2, 0.3]);
        declara(&mut s, 4, taca, 0.6, &[0.05; 3]);
        let d = declarados(&s);
        assert_eq!(d.iter().map(|x| x.0).collect::<Vec<_>>(), vec![4, 9]);
        assert_eq!(d[0].1.forma, taca);
        assert_eq!(d[1].1.forma, caixa);
        assert_eq!(d[1].1.salto, vec![0.1, 0.2, 0.3]);
        assert!((d[0].1.atrito - 0.6).abs() < 1e-9);
        let plano = FormaFixa::Plano {
            normal: [0.0, 1.0],
            altura: -2.0,
        };
        let mut p = Stream::new(1);
        declara(&mut p, 1, plano, 0.0, &[0.0]);
        assert_eq!(declarados(&p)[0].1.forma, plano);
        assert!(!tem_recibo(&p, 1));
        passa_recibo(&mut p, 1);
        assert!(tem_recibo(&p, 1) && !tem_recibo(&p, 2));
        assert!(p.columns().all(|(k, _)| e_da_porta(k)));
        assert!(p.columns().all(|(k, _)| e_desta_chave(k, 1)));
        assert!(!e_desta_chave("obstaculo:11", 1) && !e_desta_chave("friction", 1));
    }
}
