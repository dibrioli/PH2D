//! ⭐ **O clipe de ENTRADA** (v20, doc ObjectsAndComponents 01 §14, item 1 — escolha 4.1 do dono: o
//! *Autoplay* do Godot). No máximo um clipe por Animação toca sozinho quando a cena corre — o marcado,
//! e por omissão nenhum; os outros tocam quando algo os pede. A Cena toca o Arrange dela (ou, vazio,
//! o seu clipe marcado), e um objecto cujos clipes a Cena ARRANJOU é dirigido por ela.

use std::collections::BTreeSet;

use super::*;
use crate::stack::StripSource;

impl TimelineDoc {
    /// ⭐ O clipe marcado com **Autoplay** na animação de `owner` (`SCENE` = a da Cena), ou nenhum —
    /// como no Godot (dono, 10/10: *«por que o Autoplay vem sempre checado?»* ⇒ por omissão, nenhum).
    #[must_use]
    pub fn entry_clip_of(&self, owner: u64) -> Option<usize> {
        self.clips_of(owner).find(|&i| self.clips[i].autoplay)
    }

    /// Liga (`on`) ou desliga o **Autoplay** do clip `index`. Ligar desliga os irmãos: no máximo um
    /// por animação. Um dono por resolver não muda. Devolve se o documento mudou.
    pub fn set_autoplay(&mut self, index: usize, on: bool) -> bool {
        let Some(c) = self.clips.get(index) else {
            return false;
        };
        let owner = match c.owner {
            None => SCENE,
            Some(o) if !o.missing && o.entity != SCENE => o.entity,
            Some(_) => return false,
        };
        let irmaos: Vec<usize> = self.clips_of(owner).collect();
        let mut mudou = false;
        for i in irmaos {
            let v = on && i == index;
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

    /// ⭐ **Quem PODE tocar com a cena a correr** (o Play da barra superior), com o clipe marcado com
    /// Autoplay de cada um (ou nenhum): a CENA primeiro, quando o Arrange está vazio (com Arrange, é ele
    /// que toca), depois cada objecto vivo com Animação que a Cena não dirige, por ordem do 1.º clipe,
    /// sem repetições. Sem marca, um objecto só toca o que alguém lhe PEDIR (`ClipPlayback`). O objecto
    /// escreve por último: a animação PRÓPRIA dele manda sobre a da Cena.
    #[must_use]
    pub fn runtime_owners(&self) -> Vec<(u64, Option<usize>)> {
        let cena = self
            .stack
            .iter()
            .all(|l| l.strips.is_empty())
            .then(|| (SCENE, self.entry_clip_of(SCENE)));
        let dirigidos = self.arranged_owners();
        let mut vistos = BTreeSet::new();
        cena.into_iter()
            .chain(
                self.animation_owners()
                    .filter(|o| !dirigidos.contains(o) && vistos.insert(*o))
                    .map(|o| (o, self.entry_clip_of(o))),
            )
            .collect()
    }

    /// O que toca SEM pedido com a cena a correr: os de [`Self::runtime_owners`] com Autoplay.
    #[must_use]
    pub fn autoplay_entries(&self) -> Vec<(u64, usize)> {
        self.runtime_owners()
            .into_iter()
            .filter_map(|(o, c)| Some((o, c?)))
            .collect()
    }
}
