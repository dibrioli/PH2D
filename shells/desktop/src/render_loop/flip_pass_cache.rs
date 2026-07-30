//! ADR-0114 W1 T1.8 — o **cache de tesselação por (objeto, desenho)** do passe
//! Flip, extraído de `flip_pass` (HR-18 LOC). Puro CPU (sem GPU) — testável
//! headless.
//!
//! A troca de quadro barata é isto: num *hold*/pan/zoom, `ensure` só recomputa o
//! hash de conteúdo (barato) e reusa a geometria empacotada; a tesselação
//! (ear-clipping + layout) NÃO re-roda.

use ph2d_flip::FlipDrawing;
use ph2d_flip_render::{FlipGpuData, pack_drawing};
use std::collections::BTreeMap;

/// Cache de tesselação por (objeto, desenho) + instrumentação (por frame).
#[derive(Default)]
pub(super) struct TessCache {
    // `BTreeMap` (não `HashMap`): ADR-0022 / HR-5 — determinístico, sem hasher; o
    // cache é pequeno (≤ nº de desenhos ativos), então O(log n) é irrelevante.
    map: BTreeMap<(u64, u32), CachedTess>,
    packs: u32,
    hits: u32,
}

struct CachedTess {
    hash: u64,
    data: FlipGpuData,
}

impl TessCache {
    pub(super) fn reset_stats(&mut self) {
        self.packs = 0;
        self.hits = 0;
    }

    /// Garante que `key` está tesselado e VÁLIDO para o `drawing` atual:
    /// re-empacota só se ausente ou se o conteúdo mudou (hash diverge — pega
    /// edição do W2 e reuso posicional de `DrawingId` após compactação).
    pub(super) fn ensure(&mut self, key: (u64, u32), drawing: &FlipDrawing) {
        let hash = drawing_hash(drawing);
        match self.map.get(&key) {
            Some(c) if c.hash == hash => self.hits += 1,
            _ => {
                self.packs += 1;
                self.map.insert(
                    key,
                    CachedTess {
                        hash,
                        data: pack_drawing(drawing),
                    },
                );
            }
        }
    }

    pub(super) fn get(&self, key: &(u64, u32)) -> Option<&FlipGpuData> {
        self.map.get(key).map(|c| &c.data)
    }

    /// O hash de CONTEÚDO da geometria de `key` — o mesmo que o [`Self::ensure`] já computa por
    /// frame, exposto para a impressão digital da frescura (`flip_pass_stage`).
    ///
    /// ⚠️ **É o hash, não a chave.** A chave é `(objeto, desenho)` e sobrevive a uma EDIÇÃO; o hash
    /// é o que muda quando o artista mexe no desenho, e é ele que o skip do Pass A precisa — usar a
    /// chave congelaria a camada no primeiro traço dela.
    pub(super) fn hash(&self, key: &(u64, u32)) -> Option<u64> {
        self.map.get(key).map(|c| c.hash)
    }

    pub(super) fn log(&self) {
        if std::env::var_os("PH2D_FLIP_STATS").is_some() {
            eprintln!(
                "[ph2d-flip] tess: {} pack(s), {} hit(s) neste frame",
                self.packs, self.hits
            );
        }
    }
}

/// FNV-1a (transcendental-free, HR-5) de um passo.
#[inline]
fn fnv(h: u64, x: u32) -> u64 {
    (h ^ x as u64).wrapping_mul(0x0000_0100_0000_01B3)
}

/// Hash de conteúdo de um desenho — cobre TUDO que `pack_drawing` lê (posições,
/// larguras, opacidades, cores, fechamento, cap, hardness, material, fill). Se
/// o desenho muda (edição do W2) ou um `DrawingId` é reusado após compactação, o
/// hash diverge e o cache re-empacota. Muito mais barato que a tesselação.
fn drawing_hash(d: &FlipDrawing) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for s in &d.strokes {
        for p in s.positions() {
            h = fnv(h, p.x.to_bits());
            h = fnv(h, p.y.to_bits());
        }
        for w in s.widths() {
            h = fnv(h, w.to_bits());
        }
        for o in s.opacities() {
            h = fnv(h, o.to_bits());
        }
        for c in s.colors() {
            for k in c.0 {
                h = fnv(h, k.to_bits());
            }
        }
        h = fnv(h, u32::from(s.closed));
        h = fnv(h, s.cap.0 as u32);
        h = fnv(h, s.cap.1 as u32);
        h = fnv(h, s.hardness.to_bits());
        h = fnv(h, s.material.0);
        match &s.fill {
            Some(f) => {
                h = fnv(h, 1);
                for k in f.color.0 {
                    h = fnv(h, k.to_bits());
                }
                h = fnv(h, f.opacity.to_bits());
            }
            None => h = fnv(h, 0),
        }
        // Os buracos e o `hide_stroke` MUDAM o que a GPU desenha (a tesselação e a
        // existência do contorno), então entram no hash. Sem eles, uma edição que só
        // mexesse nesses campos renderizaria a tesselação velha — hoje inalcançável (todo
        // fill/unpaint mexe também nas posições), mas a W4 os tornou carga estrutural, e
        // um cache que ignora um campo carregado é uma armadilha esperando o próximo.
        h = fnv(h, u32::from(s.hide_stroke));
        for ring in &s.holes {
            for p in ring {
                h = fnv(h, p.x.to_bits());
                h = fnv(h, p.y.to_bits());
            }
            h = fnv(h, 0xFFFE_FFFF); // separa anéis
        }
        // separa strokes adjacentes (evita colisão por concatenação).
        h = fnv(h, 0xFFFF_FFFF);
    }
    fnv(h, d.strokes.len() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ph2d_core::Vec2;
    use ph2d_flip::{FlipStroke, Point, Rgba};

    fn line(a: Vec2, b: Vec2) -> FlipDrawing {
        let mut s = FlipStroke::new();
        for p in [a, b] {
            s.push_point(Point {
                pos: p,
                width: 0.1,
                opacity: 1.0,
                color: Rgba::new(1.0, 0.0, 0.0, 1.0),
            });
        }
        let mut d = FlipDrawing::new();
        d.strokes.push(s);
        d
    }

    /// T1.8: um *hold* (o MESMO desenho segurando N quadros) NÃO re-tessela — a
    /// segunda amostra do mesmo `(key, drawing)` é cache-hit (`packs` não sobe).
    #[test]
    fn hold_reuses_tessellation_zero_repacks() {
        let mut c = TessCache::default();
        let d = line(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
        let key = (7, 3);

        c.reset_stats();
        c.ensure(key, &d); // 1º quadro do hold: miss → pack
        assert_eq!((c.packs, c.hits), (1, 0));

        // Quadros seguintes do MESMO hold: cache-hit, zero re-tesselação.
        for _ in 0..12 {
            c.reset_stats();
            c.ensure(key, &d);
            assert_eq!((c.packs, c.hits), (0, 1), "hold não pode re-tesselar");
        }
        assert!(c.get(&key).is_some(), "a geometria fica residente");
    }

    /// Trocar para um desenho de CONTEÚDO diferente (ainda que sob a mesma
    /// `cache_key`, ex.: reuso posicional de `DrawingId` / edição do W2)
    /// re-tessela — o hash diverge.
    #[test]
    fn content_change_forces_repack() {
        let mut c = TessCache::default();
        let key = (1, 0);
        let a = line(Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0));
        let b = line(Vec2::new(0.0, 0.0), Vec2::new(2.0, 0.0)); // ponto diferente

        c.ensure(key, &a);
        let after_a = c.packs;
        c.ensure(key, &b); // mesmo slot, conteúdo diferente → re-pack
        assert_eq!(c.packs, after_a + 1, "conteúdo mudou deve re-tesselar");
    }

    /// O hash é estável para conteúdo idêntico e sensível a uma mudança mínima.
    #[test]
    fn hash_is_stable_and_sensitive() {
        let a = line(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
        let a2 = line(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0));
        let moved = line(Vec2::new(0.0, 0.0), Vec2::new(1.0, 1.0001));
        assert_eq!(
            drawing_hash(&a),
            drawing_hash(&a2),
            "idêntico -> mesmo hash"
        );
        assert_ne!(
            drawing_hash(&a),
            drawing_hash(&moved),
            "mudança mínima -> hash diferente"
        );
    }
}
