//! **A continuação noutro documento** (W19: `Track::continua`): o artista voltou a um objecto Motion que
//! uma trilha anterior já gravou. Ao aplicar, os nós dessa cadeia JÁ estão no vivo — a continuação não os
//! junta de novo: os ids dos passos dela reescrevem-se para os que eles têm lá.
//!
//! Os nós de uma cadeia levam-se da gravação ao destino por uma função crescente (o `import_owner` planta
//! por ordem, os criados andam com a base), por isso os do dono no inicial da continuação casam, por
//! ordem, com os da cadeia no vivo — e o tipo de cada par confere-se.

use super::inicial::{self, Inicial};
use super::mapa::Mapa;
use super::{
    E, ErroAoTranspor, em_grupo_desligado, fim, fios_so_do_dono, intactos, le, proxima_moldura,
};
use ph2d_macro::Track;
use ph2d_motion_doc::MotionDoc;
use ph2d_motion_doc::subgraph;
use ph2d_nodegraph::graph::NodeId;
use std::collections::BTreeSet;

/// Os nós da cadeia no fim de cada trilha dela: `anteriores` = (como veio, a aplicada), pela ordem.
fn da_cadeia(anteriores: &[(&Track, &Track)], novo: u64) -> Result<BTreeSet<NodeId>, E> {
    let mut s = BTreeSet::new();
    for (j, (original, aplicada)) in anteriores.iter().enumerate() {
        let ini = inicial::le(&aplicada.initial).map_err(E::Inicial)?;
        if j == 0 {
            // A primeira: os que o `import_owner` plantou, os `n` ids abaixo da base nova.
            let o = le(original)?;
            let n =
                u32::try_from(o.doc.nodes_of(original.target).len()).map_err(|_| E::Continuacao)?;
            let d = ini.base.checked_sub(n).ok_or(E::Continuacao)?;
            s.extend(
                ini.doc
                    .nodes_of(novo)
                    .into_iter()
                    .filter(|x| x.0 >= d && x.0 < ini.base),
            );
        }
        let f = fim(aplicada).map_err(E::Tocar)?;
        let vivos: BTreeSet<NodeId> = f.nodes_of(novo).into_iter().collect();
        s.retain(|x| vivos.contains(x));
        s.extend(vivos.into_iter().filter(|x| x.0 >= ini.base));
    }
    Ok(s)
}

/// **Aplicar a continuação** `track` a `novo` em `vivo`, onde a cadeia `anteriores` (a trilha que ela
/// continua e as continuações já aplicadas) já pôs os nós. Devolve a trilha a instalar (inicial = `vivo`)
/// e o documento do fim dela.
///
/// # Errors
/// [`ErroAoTranspor`] — [`ErroAoTranspor::Continuacao`] se ela não começa onde a cadeia acabou.
pub fn continua(
    track: &Track,
    anteriores: &[(&Track, &Track)],
    vivo: &MotionDoc,
    novo: u64,
) -> Result<(Track, MotionDoc), ErroAoTranspor> {
    if novo == 0 {
        return Err(E::SemDono);
    }
    let ini = le(track)?;
    let dono = track.target;
    let i = &ini.doc;
    fios_so_do_dono(i, |n| i.owner_of(n) == Some(dono))?;
    if i.nodes_of(dono)
        .iter()
        .any(|n| em_grupo_desligado(i, *n, u32::MAX))
    {
        return Err(E::GrupoDesligado);
    }
    let cadeia = da_cadeia(anteriores, novo)?;
    let de = i.nodes_of(dono);
    let para: Vec<NodeId> = cadeia.iter().copied().collect();
    let tipo = |d: &MotionDoc, n: NodeId| d.graph.node(n).map(|x| x.type_name.clone());
    if de.len() != para.len()
        || de
            .iter()
            .zip(&para)
            .any(|(a, b)| tipo(i, *a).is_none() || tipo(i, *a) != tipo(vivo, *b))
    {
        return Err(E::Continuacao);
    }
    let mut sem = vivo.clone();
    for n in &para {
        sem.graph.remove_node(*n);
    }
    sem.forget_nodes(&para);
    let mapa = Mapa {
        ini: i,
        destino: &sem,
        dono,
        novo,
        nos: de.into_iter().zip(para).collect(),
        base: ini.base,
        base_nova: vivo.graph.next_id(),
        grupos: (
            subgraph::next_id(&i.subgraphs),
            subgraph::next_id(&vivo.subgraphs),
        ),
        fundos: (proxima_moldura(i), proxima_moldura(vivo)),
    };
    let mut t = track.clone();
    t.target = novo;
    t.initial = inicial::escreve(&Inicial {
        doc: vivo.clone(),
        base: mapa.base_nova,
        lei: ini.lei,
        uv: ini.uv,
        tamanho: ini.tamanho,
    });
    for e in &mut t.events {
        e.payload = mapa.carga(&e.payload)?;
    }
    for ed in &mut t.edicoes {
        for (_, c) in ed.trocas.iter_mut().chain(ed.insercoes.iter_mut()) {
            *c = mapa.carga(c)?;
        }
    }
    let doc = fim(&t).map_err(E::Tocar)?;
    if !intactos(&sem, &doc) {
        return Err(E::TocouOsNosDeAntes);
    }
    Ok((t, doc))
}
