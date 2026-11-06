//! A ponte com o painel dos cartões: a aba na vaga, as intenções e o instantâneo.

use ph2d_editor_core::screens::hero::HeroScreen;
use ph2d_panel_sfxlab::tabela::{self, forma_nova};
use ph2d_panel_sfxlab::{Instantaneo, Intencao, Material, publica, tomar};
use ph2d_sfx_recipe::{LayerId, Recipe, Source};

use crate::aba::Aba;
use crate::docs::SfxLabDocs;

pub(crate) fn aba_ativa(hero: &mut HeroScreen) -> Option<u64> {
    hero.documents
        .active_surface_mut()
        .and_then(|(f, id)| f.as_any().is::<SfxLabDocs>().then_some(id))
}

/// O painel fica na vaga como ABA enquanto a aba do som está ativa: quem o põe à frente e quem o tira é o `slot_tabs`.
pub(crate) fn vaga(hero: &mut HeroScreen, ativa: bool) {
    hero.panel_visibility
        .insert(ph2d_panel_sfxlab::PANEL_ID, ativa);
}

/// As intenções do painel viram edição, num gesto só enquanto a mão arrasta um campo.
pub(crate) fn drena(hero: &mut HeroScreen, ativa: Option<u64>) {
    let scrub = hero
        .store
        .number_input_drag()
        .is_some_and(|d| tabela::campo(d.id).is_some());
    let pedidos = tomar();
    let Some(a) = ativa.and_then(|id| {
        hero.documents
            .surfaces_mut()
            .find_mut::<SfxLabDocs>()?
            .aba_mut(id)
    }) else {
        return;
    };
    if !pedidos.is_empty() && !a.hist.em_gesto() {
        a.hist.abre_gesto(&a.receita);
        a.gesto_do_cartao = true;
    }
    for p in pedidos {
        aplica(a, p);
    }
    if a.gesto_do_cartao && !scrub {
        a.gesto_do_cartao = false;
        a.fecha_gesto();
    }
}

fn aplica(a: &mut Aba, p: Intencao) {
    let sel = a.sel;
    // O campo mostra o valor arredondado: voltar a ele (Esc, ou arrastar de volta) devolve o valor exato.
    if let Intencao::Valor(id, v) = p
        && let Some(c) = tabela::campo(id)
        && let Some(ini) = a.hist.inicio_do_gesto().cloned()
        && c.le(&ini, sel.and_then(|s| ini.layer(s))) == Some(v)
    {
        a.edita(|r| *r = ini);
        return;
    }
    a.edita(|r| escreve(r, sel, p));
}

fn escreve(r: &mut Recipe, sel: Option<LayerId>, p: Intencao) {
    match p {
        Intencao::Valor(id, v) => {
            if let Some(c) = tabela::campo(id) {
                c.escreve(r, sel, v);
            }
        }
        Intencao::Opcao(id) => {
            if let (Some(o), Some(l)) = (tabela::opcao(id), sel.and_then(|s| r.layer_mut(s))) {
                (o.escolhe)(l);
            }
        }
        Intencao::Material(take) => {
            if let Some(Source::Recording(g)) =
                sel.and_then(|s| r.layer_mut(s)).map(|l| &mut l.source)
            {
                g.take = take;
            }
        }
        Intencao::FormaHit => {
            let segundos = r.seconds;
            if let Some(l) = sel.and_then(|s| r.layer_mut(s)) {
                l.envelope = forma_nova(segundos, l.start);
            }
        }
    }
}

pub(crate) fn publica_o_instantaneo(docs: &SfxLabDocs, ativa: Option<u64>) {
    publica(ativa.and_then(|id| docs.instantaneo(id)));
}

impl SfxLabDocs {
    fn instantaneo(&self, id: u64) -> Option<Instantaneo> {
        let a = self.abas.iter().find(|a| a.id == id)?;
        Some(Instantaneo {
            receita: a.receita.clone(),
            sel: a.sel,
            materiais: a
                .takes
                .iter()
                .map(|t| Material {
                    take: t.id,
                    segundos: t.frames() as f32 / t.rate.max(1) as f32,
                })
                .collect(),
        })
    }
}
