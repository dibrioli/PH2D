//! Os gestos da exploração na aba: `R`, `M`, `N`, `K`, `L`, `Tab`, `1`–`8`, `Enter`, a faixa Heard e a grade.
//! A lei mora em `ph2d-sfx-explore`; aqui só se liga a mão à aba.

use ph2d_editor_core::doc_surface::{Key, KeyInput, KeyOutcome};
use ph2d_panel_sfxlab::campo_sob_o_mouse;
use ph2d_sfx_explore::{FILHOS, TAMANHO_MUTATE, TAMANHO_ROLL};
use ph2d_sfx_explore_tela::{AlvoE, tamanho_em};
use ph2d_sfx_recipe::{LayerId, Recipe};

use crate::aba::{Aba, Alvo, Gesto};

/// Dois cliques na mesma linha da Heard: o mesmo tempo do resto do app.
const DUPLO_NS: u128 = 400_000_000;
/// Menos do que isto é um clique, não um arrasto.
const ARRASTO_MIN_PX: f32 = 6.0;
/// O arrasto vai até o dobro do Roll: além disto, o som já não tem parentesco com a base.
const TAMANHO_MAX: f64 = 2.0 * TAMANHO_ROLL;

impl Aba {
    /// O som novo vira o atual, um passo de desfazer, sem tocar no som aprovado.
    fn troca_por(&mut self, nova: Recipe) {
        self.edita(|r| {
            let aprovado = r.approved.take();
            *r = nova;
            r.approved = aprovado;
        });
        self.sobreviventes();
    }

    fn sorteado(&mut self, nova: Recipe) {
        self.troca_por(nova);
        self.cursor = 0.0;
        self.quer_tocar = true;
        self.quer_nivelado = true;
    }

    fn roll(&mut self) {
        if self.ex.aberta() {
            let (r, t) = (self.receita.clone(), self.takes.clone());
            return self.ex.semeia(&r, TAMANHO_ROLL, &t);
        }
        let nova = self.ex.sessao.roll(&self.receita);
        self.sorteado(nova);
    }

    fn mutate(&mut self) {
        if self.ex.aberta() {
            let (r, t) = (self.receita.clone(), self.takes.clone());
            return self.ex.semeia(&r, TAMANHO_MUTATE, &t);
        }
        let nova = self.ex.sessao.mutate(&self.receita);
        self.sorteado(nova);
    }

    fn next(&mut self) {
        if self.ex.aberta() {
            let t = self.takes.clone();
            return self.ex.proximo_filho(&t);
        }
        let nova = self.ex.sessao.next(&self.receita);
        self.sorteado(nova);
    }

    /// No Explore, é a variação ouvida por último que ganha a estrela; ela não vira a atual.
    fn keep(&mut self) {
        let do_filho = self.ex.grade.as_ref().and_then(|g| {
            let i = g.ultimo?;
            Some((g.filhos[i].as_ref()?.receita.clone(), g.como(i)))
        });
        match do_filho {
            Some((r, como)) => {
                self.ex.sessao.ouviu(&r, como);
                self.ex.sessao.keep(&r);
            }
            None => self.ex.sessao.keep(&self.receita),
        }
    }

    /// `Enter`: a variação ouvida por último vira a atual, e nascem oito novas em volta dela.
    fn escolhe_o_filho(&mut self) {
        let nova = self
            .ex
            .grade
            .as_ref()
            .and_then(|g| g.filhos[g.ultimo?].as_ref())
            .map(|f| f.receita.clone());
        if let Some(r) = nova {
            self.troca_por(r);
        }
    }

    fn alterna_explore(&mut self) {
        self.gesto = None;
        let (r, t) = (self.receita.clone(), self.takes.clone());
        self.ex.alterna(&r, &t);
    }

    fn trava_camada(&mut self, id: LayerId) {
        self.sem_tocar(|r| {
            if let Some(l) = r.layer_mut(id) {
                l.travada = !l.travada;
            }
        });
    }

    fn trava_campo(&mut self, campo: u64) {
        let Some(id) = self.sel else {
            return;
        };
        self.sem_tocar(|r| {
            if let Some(l) = r.layer_mut(id) {
                if let Some(i) = l.travas.iter().position(|c| *c == campo) {
                    l.travas.remove(i);
                } else {
                    l.travas.push(campo);
                }
            }
        });
    }

    /// Travar não muda o som: o Play on change fica quieto.
    fn sem_tocar(&mut self, f: impl FnOnce(&mut Recipe)) {
        let tocava = self.quer_tocar;
        self.edita(f);
        self.quer_tocar = tocava;
    }

    /// `L`: o campo sob o mouse; senão a camada sob o mouse; senão a camada escolhida.
    fn lock(&mut self) {
        if let Some(c) = campo_sob_o_mouse() {
            return self.trava_campo(c.0);
        }
        let sob = match self.hover {
            Some(Alvo::Linha(i) | Alvo::Alca(i) | Alvo::Onda(i) | Alvo::Mudo(i)) => Some(i),
            Some(Alvo::Solo(i) | Alvo::Trava(i)) => Some(i),
            _ => None,
        };
        if let Some(id) = sob.or(self.sel) {
            self.trava_camada(id);
        }
    }

    fn da_linha(&mut self, i: u32) {
        let duplo = self
            .clique_anterior
            .is_some_and(|(j, t)| j == i && self.agora_ns.saturating_sub(t) <= DUPLO_NS);
        self.clique_anterior = Some((i, self.agora_ns));
        if duplo {
            if let Some(r) = self
                .ex
                .sessao
                .ouvidos
                .get(i as usize)
                .map(|o| o.receita.clone())
            {
                self.troca_por(r);
            }
            return;
        }
        let t = self.takes.clone();
        self.ex.ouve_a_linha(i as usize, &t);
    }
}

pub(crate) fn clique(a: &mut Aba, alvo: AlvoE) {
    match alvo {
        AlvoE::Roll => a.roll(),
        AlvoE::Mutate => a.mutate(),
        AlvoE::Next => a.next(),
        AlvoE::Keep => a.keep(),
        AlvoE::Lock => {
            if let Some(id) = a.sel {
                a.trava_camada(id);
            }
        }
        AlvoE::Explore => a.alterna_explore(),
        AlvoE::Atual => {
            a.cursor = 0.0;
            a.quer_tocar = true;
            a.quer_nivelado = true;
        }
        AlvoE::SoKept => a.so_kept = !a.so_kept,
        AlvoE::Ouvido(i) => a.da_linha(i),
        AlvoE::Estrela(i) => a.ex.sessao.alterna_estrela(i as usize),
        AlvoE::Filho(_) | AlvoE::Mudanca => {}
    }
}

pub(crate) fn trava(a: &mut Aba, id: LayerId) {
    a.trava_camada(id);
}

/// O carregar nos alvos que têm arrasto.
pub(crate) fn desce(a: &mut Aba, alvo: AlvoE, x: f32, y: f32) -> bool {
    match alvo {
        AlvoE::Filho(i) => {
            let i = usize::from(i);
            let t0 = a.ex.grade.as_ref().map_or(0.0, |g| g.tamanhos[i]);
            a.gesto = Some(Gesto::Filho {
                i,
                x0: x,
                y0: y,
                t0,
                andou: false,
            });
            true
        }
        AlvoE::Mudanca => {
            a.gesto = Some(Gesto::Mudanca);
            muda_o_tamanho(a, x);
            true
        }
        _ => false,
    }
}

fn muda_o_tamanho(a: &mut Aba, x: f32) {
    let Some(r) = a.rect_de_ex(AlvoE::Mudanca) else {
        return;
    };
    if let Some(g) = a.ex.grade.as_mut() {
        g.tamanho = tamanho_em(x, r);
    }
}

pub(crate) fn move_(a: &mut Aba, gesto: Gesto, x: f32, y: f32) {
    match gesto {
        Gesto::Mudanca => muda_o_tamanho(a, x),
        Gesto::Filho {
            i,
            x0,
            y0,
            t0,
            andou,
        } => {
            let longe = ((x - x0).powi(2) + (y - y0).powi(2)).sqrt();
            if !andou && longe < ARRASTO_MIN_PX {
                return;
            }
            a.gesto = Some(Gesto::Filho {
                i,
                x0,
                y0,
                t0,
                andou: true,
            });
            a.mao = (x, y);
            let t = a.takes.clone();
            let tamanho = tamanho_do_arrasto(a, i, x, y, x0, y0, t0);
            a.ex.redimensiona(i, tamanho, &t);
        }
        _ => {}
    }
}

/// A distância ao som atual manda: encostar nele é a base, a célula é o tamanho de agora e longe é mais. Voltar ao ponto de onde
/// saiu devolve o valor exato.
fn tamanho_do_arrasto(a: &Aba, _i: usize, x: f32, y: f32, x0: f32, y0: f32, t0: f64) -> f64 {
    if ((x - x0).powi(2) + (y - y0).powi(2)).sqrt() < ARRASTO_MIN_PX {
        return t0;
    }
    let Some(c) = a.rect_de_ex(AlvoE::Atual) else {
        return t0;
    };
    let (cx, cy) = (c.x + c.w / 2.0, c.y + c.h / 2.0);
    let d0 = ((x0 - cx).powi(2) + (y0 - cy).powi(2)).sqrt().max(1.0);
    let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();
    if c.contains(x, y) {
        return 0.0;
    }
    let base = if t0 > 0.0 { t0 } else { TAMANHO_MUTATE };
    (base * f64::from(d / d0)).min(TAMANHO_MAX)
}

pub(crate) fn sobe(a: &mut Aba, gesto: Gesto) {
    a.gesto = None;
    match gesto {
        Gesto::Mudanca => {
            if let Some(tamanho) = a.ex.grade.as_ref().map(|g| g.tamanho) {
                let (r, t) = (a.receita.clone(), a.takes.clone());
                a.ex.semeia(&r, tamanho, &t);
            }
        }
        Gesto::Filho { i, .. } => a.ex.toca(i),
        _ => {}
    }
}

/// `None` = a tecla não é da exploração.
pub(crate) fn tecla(a: &mut Aba, k: &KeyInput<'_>) -> Option<KeyOutcome> {
    let toma = Some(KeyOutcome {
        consumed: true,
        copy: None,
    });
    let desceu = k.pressed;
    match k.key {
        Key::Tab => {
            if desceu {
                a.alterna_explore();
            }
            toma
        }
        Key::Enter if a.ex.aberta() => {
            if desceu {
                a.escolhe_o_filho();
            }
            toma
        }
        Key::Escape if a.ex.aberta() => {
            if desceu {
                a.alterna_explore();
            }
            toma
        }
        Key::Char(c) => {
            let c = c.to_ascii_lowercase();
            let digito = c.to_digit(10).map(|d| d as usize);
            match (c, digito) {
                ('r', _) if desceu => a.roll(),
                ('m', _) if desceu => a.mutate(),
                ('n', _) if desceu => a.next(),
                ('k', _) if desceu => a.keep(),
                ('l', _) if desceu => a.lock(),
                ('r' | 'm' | 'n' | 'k' | 'l', _) => {}
                (_, Some(d)) if a.ex.aberta() && (1..=FILHOS).contains(&d) => {
                    if desceu {
                        a.ex.toca(d - 1);
                    }
                }
                _ => return None,
            }
            toma
        }
        _ => None,
    }
}
