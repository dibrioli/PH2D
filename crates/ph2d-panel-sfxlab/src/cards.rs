//! Os cartões: derivados da tabela. Camada escolhida: Layer · Source · Shape. Nenhuma: Sound.

use std::rc::Rc;

use ph2d_editor_core::interaction::WidgetStore;
use ph2d_editor_core::panel::cards::dropdown;
use ph2d_editor_core::panel::cards::ferramenta::{Numero, campo, cartao, opcoes, prepara_numeros};
use ph2d_editor_core::panel::cards::{CardCtx, CardDef, Cell, Seg, Widget};
use ph2d_i18n::{tr, tr_with};
use ph2d_sfx_recipe::{Envelope, Layer, Source};

use crate::estado::Instantaneo;
use crate::ids;
use crate::tabela::{CAMPOS, Cartao, Grupo, OPCOES};

/// Com o store mutável, todo quadro: marca e semeia os campos que se aplicam, e registra as opções de material.
pub(crate) fn prepare(store: &mut WidgetStore, snap: Option<&Instantaneo>) {
    dropdown::declare(ids::MATERIAL);
    let Some(s) = snap else {
        return;
    };
    let l = s.camada();
    let numeros: Vec<Numero> = CAMPOS
        .iter()
        .filter_map(|c| {
            c.le(&s.receita, l).map(|valor| Numero {
                chip: c.id,
                step: c.passo,
                scrub: c.scrub,
                fabrica: c.fabrica,
                valor,
            })
        })
        .collect();
    prepara_numeros(store, &numeros);
    for i in 0..s.materiais.len() {
        let id = ids::opcao_de_material(i);
        if store.get(id).is_none() {
            crate::registra_botao(store, id);
        }
    }
}

pub(crate) fn cards(snap: Option<Rc<Instantaneo>>) -> Vec<CardDef<'static>> {
    let Some(s) = snap else {
        return Vec::new();
    };
    if s.camada().is_none() {
        let dele = s.clone();
        return vec![cartao(
            ids::SEC_SOUND,
            tr("sfxlab.card.sound"),
            None,
            move |c, y| campos_do_cartao(c, &dele, Cartao::Som, y),
        )];
    }
    let (a, b, d) = (s.clone(), s.clone(), s);
    vec![
        cartao(
            ids::SEC_LAYER,
            tr("sfxlab.card.layer"),
            None,
            move |c, y| campos_do_cartao(c, &a, Cartao::Camada, y),
        ),
        cartao(
            ids::SEC_SOURCE,
            tr("sfxlab.card.source"),
            None,
            move |c, y| fonte(c, &b, y),
        ),
        cartao(
            ids::SEC_SHAPE,
            tr("sfxlab.card.shape"),
            None,
            move |c, y| forma(c, &d, y),
        ),
    ]
}

/// Lei da compactação: os números curtos dividem a linha.
fn campos_do_cartao(c: &mut CardCtx<'_>, s: &Instantaneo, cartao: Cartao, y: f32) -> f32 {
    let l = s.camada();
    let dele: Vec<_> = CAMPOS
        .iter()
        .filter(|f| f.cartao == cartao && f.le(&s.receita, l).is_some())
        .map(|f| (f.id, rotulo(l, f.id.0, tr(f.chave))))
        .collect();
    let cells: Vec<Cell<'_>> = dele.iter().map(|(id, t)| campo(*id, t)).collect();
    c.compact(&cells, y)
}

/// Um campo travado diz que está: é o único sinal de que o sorteio não passa por ele.
fn rotulo(l: Option<&Layer>, id: u64, nome: &str) -> String {
    if l.is_some_and(|l| l.travas.contains(&id)) {
        tr_with("sfxlab.card.locked", &[("name", &nome)])
    } else {
        nome.to_owned()
    }
}

fn escolha(c: &mut CardCtx<'_>, l: &Layer, grupo: Grupo, nome: &str, y: f32) -> f32 {
    let dela: Vec<_> = OPCOES.iter().filter(|o| o.grupo == grupo).collect();
    let travada = dela.iter().find(|o| l.travas.contains(&o.id.0));
    let dito = rotulo(Some(l), travada.map_or(0, |o| o.id.0), nome);
    let pares: Vec<_> = dela.iter().map(|o| (o.id, tr(o.chave))).collect();
    let aceso = dela
        .iter()
        .position(|o| (o.aceso)(l) == Some(true))
        .unwrap_or(0);
    let segs: Vec<Seg<'_>> = opcoes(&pares, aceso);
    c.full(Some(&dito), Widget::Choice(&segs), y)
}

fn fonte(c: &mut CardCtx<'_>, s: &Instantaneo, y: f32) -> f32 {
    let Some(l) = s.camada() else {
        return y;
    };
    let y = match &l.source {
        Source::Noise(_) => escolha(c, l, Grupo::Filter, tr("sfxlab.card.filter"), y),
        Source::Tone(_) => escolha(c, l, Grupo::Wave, tr("sfxlab.card.wave"), y),
        Source::Body(_) => escolha(c, l, Grupo::Law, tr("sfxlab.card.law"), y),
        Source::Recording(g) => material(c, s, &g.take, y),
    };
    campos_do_cartao(c, s, Cartao::Fonte, y)
}

fn nome_do_material(i: usize, segundos: f32) -> String {
    let s = format!("{segundos:.2}");
    tr_with("sfxlab.card.take_name", &[("n", &(i + 1)), ("s", &s)])
}

fn material(c: &mut CardCtx<'_>, s: &Instantaneo, take: &[u8; 32], y: f32) -> f32 {
    let nomes: Vec<String> = s
        .materiais
        .iter()
        .enumerate()
        .map(|(i, m)| nome_do_material(i, m.segundos))
        .collect();
    let segs: Vec<Seg<'_>> = s
        .materiais
        .iter()
        .enumerate()
        .map(|(i, m)| Seg {
            id: ids::opcao_de_material(i),
            icon: None,
            text: nomes[i].as_str(),
            on: m.take == *take,
        })
        .collect();
    let atual = segs
        .iter()
        .find(|g| g.on)
        .map_or(tr("sfxlab.missing"), |g| g.text);
    c.full(
        Some(tr("sfxlab.card.material")),
        Widget::Dropdown {
            id: ids::MATERIAL,
            text: atual,
            options: &segs,
        },
        y,
    )
}

fn forma(c: &mut CardCtx<'_>, s: &Instantaneo, y: f32) -> f32 {
    let Some(l) = s.camada() else {
        return y;
    };
    if crate::tabela::hit_de(&l.envelope).is_some() {
        return campos_do_cartao(c, s, Cartao::Forma, y);
    }
    let frase = if l.envelope == Envelope::default() {
        tr("sfxlab.card.shape.flat")
    } else {
        tr("sfxlab.card.shape.custom")
    };
    let y = c.hint(frase, y);
    c.row(
        &[Cell::new(Widget::Button {
            id: ids::FORMA_HIT,
            text: tr("sfxlab.card.shape.use_hit"),
            primary: false,
        })],
        y,
    )
}
