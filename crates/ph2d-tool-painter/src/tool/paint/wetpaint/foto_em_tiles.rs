//! Um plano da grade numa fotografia do undo vivo, em tiles: um tile igual ao da fotografia anterior é o mesmo `Arc`.
//! Medido (dono 06/11, `sonda_o_preco_de_um_passo`): a 4096×2560, com óleo, 97 % dos tiles repetem-se de um passo ao outro.

use ph2d_wet_paint::grid::TILE;
use rayon::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Clone, Default)]
pub(crate) struct PlanoEmTiles {
    s: usize,
    rows: usize,
    tiles: Vec<Arc<[f32]>>,
    /// As células dos tiles que esta fotografia não partilha com a anterior.
    novas: usize,
}

/// Por plano: início e comprimento de cada tile nas células distintas.
pub(super) type MapaDosTiles = Vec<Vec<(usize, usize)>>;

impl PlanoEmTiles {
    fn tiles_por_linha(s: usize) -> usize {
        s.div_ceil(TILE)
    }

    /// As linhas da grelha do tile `i`: `(x0, x1, y0, y1)`.
    fn caixa(s: usize, rows: usize, i: usize) -> (usize, usize, usize, usize) {
        let n = Self::tiles_por_linha(s);
        let (tx, ty) = (i % n, i / n);
        (
            tx * TILE,
            ((tx + 1) * TILE).min(s),
            ty * TILE,
            ((ty + 1) * TILE).min(rows),
        )
    }

    /// Fotografa `v` (`s` células por linha), partilhando com `anterior` os tiles iguais ao bit.
    ///
    /// Um comprimento que não é múltiplo de `s` é outra grade: truncá-lo devolvia um plano mais curto ao motor.
    pub(crate) fn de(v: &[f32], s: usize, anterior: Option<&Self>) -> Self {
        if v.is_empty() || s == 0 {
            return Self::default();
        }
        assert!(
            v.len().is_multiple_of(s),
            "plano de {} células não tem linhas de {s}: é de outra grade",
            v.len()
        );
        let rows = v.len() / s;
        let anterior = anterior.filter(|a| a.s == s && a.rows == rows && !a.tiles.is_empty());
        let total = Self::tiles_por_linha(s) * rows.div_ceil(TILE);
        let feitos: Vec<(Arc<[f32]>, usize)> = (0..total)
            .into_par_iter()
            .map_init(Vec::new, |junta, i| {
                let (x0, x1, y0, y1) = Self::caixa(s, rows, i);
                let w = x1 - x0;
                if let Some(a) = anterior {
                    let velho = &a.tiles[i];
                    let igual = (y0..y1).zip(velho.chunks_exact(w)).all(|(y, l)| {
                        let o = y * s + x0;
                        v[o..o + w]
                            .iter()
                            .zip(l)
                            .all(|(p, q)| p.to_bits() == q.to_bits())
                    });
                    if igual {
                        return (Arc::clone(velho), 0);
                    }
                }
                let novo = Self::copia_tile(v, s, (x0, x1, y0, y1), junta);
                let n = novo.len();
                (novo, n)
            })
            .collect();
        let novas = feitos.iter().map(|f| f.1).sum();
        Self {
            s,
            rows,
            tiles: feitos.into_iter().map(|f| f.0).collect(),
            novas,
        }
    }

    /// O tile `(x0, x1, y0, y1)` de `v` num `Arc` novo, juntado em `junta` (reusada: quente na cache, sem `Vec` por tile).
    fn copia_tile(
        v: &[f32],
        s: usize,
        (x0, x1, y0, y1): (usize, usize, usize, usize),
        junta: &mut Vec<f32>,
    ) -> Arc<[f32]> {
        junta.clear();
        for y in y0..y1 {
            let o = y * s + x0;
            junta.extend_from_slice(&v[o..o + x1 - x0]);
        }
        Arc::from(&junta[..])
    }

    /// Devolve o plano ao bit.
    #[cfg(test)]
    pub(crate) fn repoe(&self, v: &mut Vec<f32>) {
        self.repoe_sobre(v, None);
    }

    /// O [`Self::repoe`] sobre um plano que é `ja` ao bit (`ja` fotografado dele, sem escrita no meio): os tiles que as
    /// duas fotografias partilham (o mesmo `Arc`) já lá estão e não se reescrevem.
    pub(crate) fn repoe_sobre(&self, v: &mut Vec<f32>, ja: Option<&Self>) {
        if self.tiles.is_empty() {
            v.clear();
            return;
        }
        let ja = ja
            .filter(|j| j.s == self.s && j.rows == self.rows && j.tiles.len() == self.tiles.len());
        let ja = ja.filter(|_| v.len() == self.s * self.rows);
        v.resize(self.s * self.rows, 0.0);
        let n = Self::tiles_por_linha(self.s);
        v.par_chunks_mut(self.s).enumerate().for_each(|(y, linha)| {
            let ty = y / TILE;
            for tx in 0..n {
                let i = ty * n + tx;
                if ja.is_some_and(|j| Arc::ptr_eq(&j.tiles[i], &self.tiles[i])) {
                    continue;
                }
                let (x0, x1, y0, _) = Self::caixa(self.s, self.rows, i);
                let w = x1 - x0;
                let o = (y - y0) * w;
                linha[x0..x1].copy_from_slice(&self.tiles[i][o..o + w]);
            }
        });
    }

    /// Os bytes da fotografia inteira, sozinha.
    pub(crate) fn bytes(&self) -> usize {
        self.tiles.iter().map(|t| t.len() * 4).sum()
    }

    /// Os bytes que ela acrescenta a quem está antes dela na mesma pilha.
    pub(crate) fn bytes_novos(&self) -> usize {
        if self.tiles.is_empty() {
            0
        } else {
            self.novas * 4
        }
    }
}

/// Tira os tiles dos `planos`, cada `Arc` uma vez: a partilha sobrevive ao vai e volta.
pub(super) fn tira_os_tiles(planos: Vec<&mut PlanoEmTiles>) -> (Vec<f32>, MapaDosTiles) {
    let tiles: Vec<Vec<Arc<[f32]>>> = planos
        .into_iter()
        .map(|p| std::mem::take(&mut p.tiles))
        .collect();
    let mut vistos: BTreeMap<*const f32, (usize, usize)> = BTreeMap::new();
    let mut total = 0;
    let mapa: MapaDosTiles = tiles
        .iter()
        .map(|p| {
            p.iter()
                .map(|t| {
                    *vistos.entry(t.as_ptr()).or_insert_with(|| {
                        let inicio = total;
                        total += t.len();
                        (inicio, t.len())
                    })
                })
                .collect()
        })
        .collect();
    let mut celulas = Vec::with_capacity(total);
    for (t, &(inicio, _)) in tiles.iter().flatten().zip(mapa.iter().flatten()) {
        if inicio == celulas.len() {
            celulas.extend_from_slice(t);
        }
    }
    (celulas, mapa)
}

/// Devolve aos `planos` (os mesmos, na mesma ordem) o que [`tira_os_tiles`] tirou.
pub(super) fn poe_os_tiles(planos: Vec<&mut PlanoEmTiles>, celulas: &[f32], mapa: &MapaDosTiles) {
    let mut feitos: BTreeMap<usize, Arc<[f32]>> = BTreeMap::new();
    for (p, m) in planos.into_iter().zip(mapa) {
        p.tiles = m
            .iter()
            .map(|&(inicio, n)| {
                Arc::clone(
                    feitos
                        .entry(inicio)
                        .or_insert_with(|| Arc::from(&celulas[inicio..inicio + n])),
                )
            })
            .collect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plano(s: usize, rows: usize) -> Vec<f32> {
        (0..s * rows).map(|i| (i as f32).sin()).collect()
    }

    #[test]
    fn a_foto_volta_ao_bit_e_partilha_o_que_nao_mudou() {
        let (s, rows) = (150, 70);
        let mut v = plano(s, rows);
        let a = PlanoEmTiles::de(&v, s, None);
        assert_eq!(a.bytes_novos(), a.bytes());
        let velho = v[0];
        v[0] = -velho;
        let b = PlanoEmTiles::de(&v, s, Some(&a));
        assert_eq!(b.bytes(), a.bytes());
        assert_eq!(b.bytes_novos(), 64 * 64 * 4, "só o tile do canto mudou");
        let mut volta = Vec::new();
        b.repoe(&mut volta);
        assert!(
            volta
                .iter()
                .zip(&v)
                .all(|(p, q)| p.to_bits() == q.to_bits())
        );
        let mut volta = vec![9.0; 3];
        a.repoe(&mut volta);
        assert!(
            volta
                .iter()
                .zip(&plano(s, rows))
                .all(|(p, q)| p.to_bits() == q.to_bits())
        );
    }

    /// O undo vivo deixa de zerar o fluxo antes do `repoe` (ver `FotoDoMotor::repoe_o_fluxo_de`): o `repoe` não pode ler
    /// o que o plano tinha — lixo, zeros ou de outro comprimento dão o mesmo bit.
    #[test]
    fn o_repoe_nao_le_o_que_o_plano_tinha() {
        let (s, rows) = (150, 70);
        let a = PlanoEmTiles::de(&plano(s, rows), s, None);
        let volta = |mut v: Vec<f32>| {
            a.repoe(&mut v);
            v.iter().map(|x| x.to_bits()).collect::<Vec<_>>()
        };
        let zeros = volta(vec![0.0; s * rows]);
        assert_eq!(volta(vec![7.0; s * rows]), zeros);
        assert_eq!(volta(vec![f32::NAN; s * rows + 9]), zeros);
        assert_eq!(volta(Vec::new()), zeros);
        let mut v = vec![3.0; 5];
        PlanoEmTiles::de(&[], s, None).repoe(&mut v);
        assert!(v.is_empty(), "a foto vazia esvazia o plano");
    }

    #[test]
    #[should_panic(expected = "é de outra grade")]
    fn um_plano_de_outra_grade_e_recusado() {
        // 1026² células lidas com as linhas de 2050: o erro do smoke do dono (07/11).
        let _ = PlanoEmTiles::de(&vec![0.0; 1026 * 1026], 2050, None);
    }

    #[test]
    fn menos_zero_nao_passa_por_zero() {
        let (s, rows) = (64, 64);
        let v = vec![0.0f32; s * rows];
        let a = PlanoEmTiles::de(&v, s, None);
        let mut w = v.clone();
        w[3] = -0.0;
        let b = PlanoEmTiles::de(&w, s, Some(&a));
        assert_eq!(b.bytes_novos(), b.bytes());
    }

    #[test]
    fn o_plano_vazio_continua_vazio() {
        let a = PlanoEmTiles::de(&[], 100, None);
        let mut v = vec![1.0];
        a.repoe(&mut v);
        assert!(v.is_empty());
        assert_eq!(a.bytes(), 0);
    }

    /// Sobre o plano de que `b` foi tirada, o `repoe_sobre` de `a` dá o bit do `repoe` (o tile do canto mudou, o resto é
    /// partilhado); CONTROLO: sobre outro plano, os tiles partilhados ficavam o que lá estava.
    #[test]
    fn o_repoe_sobre_a_foto_da_grade_da_o_bit_do_repoe() {
        let (s, rows) = (150, 70);
        let v = plano(s, rows);
        let a = PlanoEmTiles::de(&v, s, None);
        let mut w = v.clone();
        w[0] = 5.0;
        w[s * rows - 1] = -0.0;
        let b = PlanoEmTiles::de(&w, s, Some(&a));
        assert!(b.bytes_novos() < b.bytes(), "partilham tiles");
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
        let (mut cheio, mut sobre) = (w.clone(), w.clone());
        a.repoe(&mut cheio);
        a.repoe_sobre(&mut sobre, Some(&b));
        assert_eq!(bits(&sobre), bits(&cheio));
        assert_eq!(bits(&cheio), bits(&v));
        let mut outro = vec![9.0; s * rows];
        a.repoe_sobre(&mut outro, Some(&b));
        assert_ne!(bits(&outro), bits(&v), "CONTROLO: o salto é real");
        let mut curto = vec![9.0; 3];
        a.repoe_sobre(&mut curto, Some(&b));
        assert_eq!(
            bits(&curto),
            bits(&v),
            "de outro comprimento, reescreve tudo"
        );
    }

    #[test]
    fn a_partilha_sobrevive_ao_vai_e_volta() {
        let (s, rows) = (150, 70);
        let v = plano(s, rows);
        let mut a = PlanoEmTiles::de(&v, s, None);
        let mut b = PlanoEmTiles::de(&v, s, Some(&a));
        let (celulas, mapa) = tira_os_tiles(vec![&mut a, &mut b]);
        assert_eq!(celulas.len(), v.len(), "cada tile uma vez");
        assert_eq!(celulas.capacity(), celulas.len(), "sem reservar a mais");
        assert_eq!(a.bytes(), 0);
        assert_eq!(a.bytes_novos(), 0, "sem tiles nada retém");
        poe_os_tiles(vec![&mut a, &mut b], &celulas, &mapa);
        assert!(Arc::ptr_eq(&a.tiles[1], &b.tiles[1]));
        let mut volta = Vec::new();
        b.repoe(&mut volta);
        assert!(
            volta
                .iter()
                .zip(&v)
                .all(|(p, q)| p.to_bits() == q.to_bits())
        );
    }
}
