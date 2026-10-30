//! ⭐ **O Follow Value sobre uma CAIXA depois da v25** (F41, `PROJECT_SCHEMA` `209 → 210`). As
//! fixtures `tests/fixtures/valor_caixa_v209*` foram gravadas pelo código de `422fec2027`, ANTES da
//! cura, por um teste `#[ignore]` de uma corrida só, apagado a seguir: o alvo `B` (id 1) com uma curva
//! na caixa do 9-slice (`Toggle(Sliced, Simple)`; chaves em 0 s, 1 s e 2 s: desmarcada, marcada,
//! desmarcada) e o dono `A` (id 2) com seis Follow Value — caixa → rotação; altura → caixa
//! (`Replace` e `Add`); caixa → X com mola e extrapolação; e dois degraus (faixa nula em `1` e em
//! `0,5`) — sessenta quadros, com as saídas e os campos escritos de cada um.
//!
//! O documento migra a curva (`doc_v24.rs`) e o registo migra a pilha (`Valor::desvira_caixas`): o
//! MESMO estado de caixa sai em cada quadro. Controlo: sem a lei, o Follow Value lê ao contrário.

use ph2d_app_constraints::relogio::Relogio;
use ph2d_app_constraints::valor::valores;
use ph2d_constraints_ecs::{
    CONSTRAINTS_TYPE_NAME, Constraints, Valores, register_constraint_components,
};
use ph2d_ecs::canal_componente as cc;
use ph2d_ecs::scene::ComponentRegistry;
use ph2d_ecs::{CanaisEscritos, CanaisLidos, FieldKey, SimWorld, StableId, Transform};
use ph2d_timeline::{TimelineDoc, WireId};

const DOC: &[u8] = include_bytes!("../fixtures/valor_caixa_v209_doc.bin");
const PILHA: &[u8] = include_bytes!("../fixtures/valor_caixa_v209_pilha.bin");
const ANTES: &str = include_str!("../fixtures/valor_caixa_v209.json");

fn k(rotulo: &str) -> FieldKey {
    cc::chaves_do_rotulo(rotulo)[0]
}

fn lido(k: FieldKey, estado: &str) -> f32 {
    let (_, f, _) = cc::resolver(k).expect("o campo");
    cc::ler_valor(f, 0, &serde_json::Value::String(estado.to_owned())).expect("lê")
}

struct Quadro {
    fonte: f32,
    saidas: Vec<f64>,
    escritos: [f32; 2],
}

/// Os sessenta quadros do gravador, com a pilha `c`, sobre o documento de agora.
fn corre(c: Constraints) -> Vec<Quadro> {
    let (caixa, caixa2, caixa3) = (
        k("component.field.slice_nine.8"),
        k("component.field.factory_fields.6"),
        k("component.field.damage_fields.6"),
    );
    let mut sim = SimWorld::new();
    let b = sim
        .world_mut()
        .spawn((StableId(1), Transform::IDENTITY))
        .id();
    let a = sim
        .world_mut()
        .spawn((StableId(2), Transform::IDENTITY, c))
        .id();
    let mut doc = TimelineDoc::from_bytes(DOC).expect("o documento v24");
    ph2d_timeline::resolve_entities(&mut doc, |w| (w == WireId(1)).then(|| b.to_bits()));
    let fps = doc.fps_display;
    (0..60_i32)
        .map(|f| {
            let t = f64::from(f) / fps;
            let w = sim.world_mut();
            w.get_mut::<Transform>(b).unwrap().translation.y = (f % 20) as f32 / 20.0;
            let mut esc = CanaisEscritos::default();
            let fonte = ph2d_timeline::field_at(w, &doc, b.to_bits(), caixa, t).expect("a curva");
            esc.escrever(b.to_bits(), caixa, fonte);
            w.insert_resource(esc);
            let mut lidos = CanaisLidos::default();
            lidos.publicar(a.to_bits(), caixa2, lido(caixa2, "Random"));
            lidos.publicar(a.to_bits(), caixa3, lido(caixa3, "Stay"));
            w.insert_resource(lidos);
            let r = Relogio {
                doc: &doc,
                t,
                laco: None,
            };
            valores(&mut sim, Some(r));
            let w = sim.world();
            let s = &w.resource::<Valores>().saidas;
            let e = w.resource::<CanaisEscritos>();
            Quadro {
                fonte,
                saidas: (0..6).map(|id| s[&(a.to_bits(), id)]).collect(),
                escritos: [caixa2, caixa3].map(|k| e.valor(a.to_bits(), k).unwrap()),
            }
        })
        .collect()
}

fn antes() -> Vec<serde_json::Value> {
    let j: serde_json::Value = serde_json::from_str(ANTES).expect("o json");
    j["quadros"].as_array().expect("quadros").clone()
}

fn num(v: &serde_json::Value) -> f64 {
    v.as_f64().expect("número")
}

/// As divergências do quadro de agora contra o gravado: o estado de caixa é o MESMO (o canal virado),
/// o resto igual.
fn divergencias(agora: &[Quadro]) -> Vec<String> {
    let mut d = Vec::new();
    for (f, (q, g)) in agora.iter().zip(antes()).enumerate() {
        if (f64::from(q.fonte) - (1.0 - num(&g["fonte"]))).abs() > 1e-6 {
            d.push(format!("q{f}: a curva da caixa não virou"));
        }
        for id in [0, 3, 4, 5] {
            let (x, y) = (q.saidas[id], num(&g["saidas"][id]));
            if (x - y).abs() > 1e-9 {
                d.push(format!("q{f}: saída {id} = {x}, antes {y}"));
            }
        }
        for (i, x) in q.escritos.iter().enumerate() {
            // O meio exato da caixa (`0,5`) é ambíguo: o `prende` arredonda-o para cima nas duas
            // convenções, e nenhuma faixa o desfaz — o único ponto que muda (item 1, `Replace`).
            if i == 0 && num(&g["saidas"][1]) == 0.5 {
                continue;
            }
            let y = 1.0 - num(&g["escritos"][i]);
            if (f64::from(*x) - y).abs() > 1e-6 {
                d.push(format!("q{f}: caixa escrita {i} = {x}, devia {y}"));
            }
        }
    }
    d
}

#[test]
fn o_follow_value_sobre_caixas_da_o_mesmo_estado_depois_da_v25() {
    let mut reg = ComponentRegistry::new();
    register_constraint_components(&mut reg);
    let nova = reg
        .desvira_caixas(CONSTRAINTS_TYPE_NAME, PILHA)
        .expect("a pilha gravada tem Follow Value sobre caixas");
    let nova: Constraints = postcard::from_bytes(&nova).expect("a pilha nova");
    assert_eq!(antes().len(), 60);
    let d = divergencias(&corre(nova));
    assert!(d.is_empty(), "{d:#?}");
    // Controlo: a pilha crua (o defeito) lê ao contrário em todo item.
    let crua: Constraints = postcard::from_bytes(PILHA).expect("a pilha");
    let d = divergencias(&corre(crua));
    for id in [0, 3, 4, 5] {
        assert!(
            d.iter().any(|x| x.contains(&format!("saída {id} "))),
            "{id}: {d:#?}"
        );
    }
    for i in 0..2 {
        assert!(
            d.iter().any(|x| x.contains(&format!("escrita {i} "))),
            "{i}"
        );
    }
}
