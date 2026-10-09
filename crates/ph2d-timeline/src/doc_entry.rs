//! ⭐ **O clipe de ENTRADA** (v20, doc ObjectsAndComponents 01 §14, item 1 — escolha 4.1 do dono: o
//! *Autoplay* do Godot). Cada Animação de objecto tem um clipe que começa sozinho no Play — o
//! marcado, ou o 1.º; os outros tocam quando algo os pede. A Cena não tem entrada: ela toca o
//! Arrange dela, como sempre, e um objecto cujos clipes a Cena ARRANJOU é dirigido por ela (não toca
//! a entrada por cima do que a Cena compôs).

use std::collections::BTreeSet;

use super::*;
use crate::stack::StripSource;

impl TimelineDoc {
    /// ⭐ O clipe de entrada da animação de `owner`: o marcado ([`NamedClip::autoplay`]), ou o 1.º.
    #[must_use]
    pub fn entry_clip_of(&self, owner: u64) -> Option<usize> {
        self.clips_of(owner)
            .find(|&i| self.clips[i].autoplay)
            .or_else(|| self.clips_of(owner).next())
    }

    /// Marca `index` como a entrada da animação dele e desmarca os irmãos. A Cena e um dono por
    /// resolver não têm entrada: `false`, nada muda. Devolve se o documento mudou.
    pub fn set_entry_clip(&mut self, index: usize) -> bool {
        let Some(owner) = self
            .clips
            .get(index)
            .and_then(|c| c.owner)
            .filter(|o| !o.missing && o.entity != SCENE)
            .map(|o| o.entity)
        else {
            return false;
        };
        let irmaos: Vec<usize> = self.clips_of(owner).collect();
        let mut mudou = false;
        for i in irmaos {
            let v = i == index;
            if self.clips[i].autoplay != v {
                self.clips[i].autoplay = v;
                mudou = true;
            }
        }
        mudou
    }

    /// Os objectos que a Cena DIRIGE: os donos de um clipe que o Arrange toca, directamente ou
    /// dentro de um container arranjado.
    #[must_use]
    pub fn arranged_owners(&self) -> BTreeSet<u64> {
        let mut donos = BTreeSet::new();
        let mut vistos = BTreeSet::new();
        let mut pilhas: Vec<&[ClipLane]> = vec![&self.stack];
        while let Some(pilha) = pilhas.pop() {
            for s in pilha.iter().flat_map(|l| &l.strips) {
                match s.source {
                    StripSource::Clip(i) => {
                        if let Some(o) = self.clips.get(usize::from(i)).and_then(|c| c.owner)
                            && !o.missing
                        {
                            donos.insert(o.entity);
                        }
                    }
                    StripSource::Container(c) => {
                        if vistos.insert(c)
                            && let Some(n) = self.containers.get(usize::from(c))
                        {
                            pilhas.push(&n.stack);
                        }
                    }
                }
            }
        }
        donos
    }

    /// ⭐ **O tempo do clipe `index` no relógio do OBJECTO**, `t` segundos depois de começar: o loop
    /// do próprio clipe (o do Keys, com ida-e-volta) dobrado pela MESMA lei de uma faixa do Arrange
    /// ([`crate::ClipStrip`]); sem loop, toca uma vez e fica na última pose; o corte de duração vale
    /// nos dois.
    #[must_use]
    pub fn clip_local_time(&self, index: usize, t: f64) -> f64 {
        let Some(c) = self.clips.get(index) else {
            return t;
        };
        let t = t.max(0.0);
        let local = match c.keys_loop_range {
            Some((a, b)) if b > a && t > a => {
                let mut faixa = crate::ClipStrip::new(StripSource::Clip(0), 0.0, f64::MAX, b);
                faixa.src_in = a;
                faixa.loop_mode = if c.keys_loop_ping_pong {
                    crate::StripLoop::PingPong
                } else {
                    crate::StripLoop::Loop
                };
                faixa.fold(t - a)
            }
            _ => t,
        };
        self.clip_cut(index, local)
    }

    /// ⭐ **Quem toca a entrada no Play**: cada objecto vivo com Animação que a Cena não dirige, com
    /// o clipe de entrada dele — por ordem do 1.º clipe, sem repetições.
    #[must_use]
    pub fn autoplay_entries(&self) -> Vec<(u64, usize)> {
        let dirigidos = self.arranged_owners();
        let mut vistos = BTreeSet::new();
        self.animation_owners()
            .filter(|o| !dirigidos.contains(o) && vistos.insert(*o))
            .filter_map(|o| Some((o, self.entry_clip_of(o)?)))
            .collect()
    }
}
