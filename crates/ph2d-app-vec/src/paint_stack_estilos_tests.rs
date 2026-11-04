//! Gates dos estilos de cor do lado da cena (D28/D30): os pedidos da lista chegam à `VecScene`, a
//! vista volta à lista, e a cor escolhida à mão solta a camada.

use super::drena;
use ph2d_ecs::ItemId;
use ph2d_editor_core::panel::cards::estilos::{self as lista, Pedido};
use ph2d_vec_scene::{
    LINHA_BASE_FILL, LINHA_BASE_STROKE, Paint, PaintEntry, Rgba8, StrokeSpec, VecPath, VecScene,
    VecVertex,
};

const AZUL: Rgba8 = Rgba8::new(0x4C, 0x8D, 0xF0, 0xFF);
const LARANJA: Rgba8 = Rgba8::new(0xE8, 0x74, 0x3B, 0xFF);

/// Duas formas escolhidas, cada uma com um preenchimento azul de id 5.
fn cena() -> (VecScene, Vec<u64>) {
    let mut scene = VecScene::default();
    let ids = (0..2)
        .map(|_| {
            let mut f = PaintEntry::fill(Paint::solid(AZUL));
            f.id = ItemId(5);
            scene.push_path(VecPath {
                verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
                    .map(VecVertex::corner)
                    .to_vec(),
                closed: true,
                paints: vec![f],
                ..VecPath::default()
            })
        })
        .collect();
    (scene, ids)
}

fn cor(s: &VecScene, id: u64) -> Rgba8 {
    s.path(id).unwrap().paints[0].swatch_color()
}

#[test]
fn a_new_style_links_the_selection_and_editing_it_recolours_both() {
    let (mut s, sel) = cena();
    let mut store = ph2d_editor_core::WidgetStore::default();
    let _ = lista::take_pedidos();
    lista::pede(Pedido::Novo {
        camada: 0,
        nome: "Brand blue".into(),
    });
    assert!(drena(&mut store, &mut s, &sel), "o «+» não gravou nada");
    let e = lista::estilos();
    assert_eq!(e.len(), 1);
    assert_eq!(
        (e[0].name.as_str(), e[0].rgba, e[0].usos),
        ("Brand blue", [0x4C, 0x8D, 0xF0, 0xFF], 2)
    );
    assert_eq!(
        lista::ligado(0).map(|x| x.id),
        Some(e[0].id),
        "a camada não mostra o estilo"
    );

    lista::pede(Pedido::Cor {
        estilo: e[0].id,
        rgba: [0xE8, 0x74, 0x3B, 0xFF],
    });
    assert!(drena(&mut store, &mut s, &sel));
    assert_eq!((cor(&s, sel[0]), cor(&s, sel[1])), (LARANJA, LARANJA));

    // A cor escolhida à mão numa forma solta-a; a outra continua ligada.
    assert!(super::super::set_color(&mut s, &sel[..1], 0, AZUL));
    drena(&mut store, &mut s, &sel[1..]);
    assert_eq!(lista::ligado(0).map(|x| x.id), Some(e[0].id));
    assert_eq!(lista::estilos()[0].usos, 1);
    drena(&mut store, &mut s, &sel[..1]);
    assert_eq!(lista::ligado(0), None, "a cor à mão não soltou a camada");
}

#[test]
fn applying_detaching_and_deleting_keep_the_colour_law() {
    let (mut s, sel) = cena();
    let mut store = ph2d_editor_core::WidgetStore::default();
    let id = s.add_estilo("Grass", LARANJA);
    let _ = lista::take_pedidos();
    lista::pede(Pedido::Aplicar {
        estilo: id,
        camada: 0,
    });
    assert!(drena(&mut store, &mut s, &sel));
    assert_eq!(cor(&s, sel[1]), LARANJA, "aplicar não pintou");
    lista::pede(Pedido::Soltar { camada: 0 });
    drena(&mut store, &mut s, &sel[..1]);
    assert_eq!(lista::ligado(0), None);
    lista::pede(Pedido::Apagar { estilo: id });
    drena(&mut store, &mut s, &sel);
    assert!(lista::estilos().is_empty());
    assert_eq!(
        cor(&s, sel[1]),
        LARANJA,
        "apagar o estilo mudou a cor da camada"
    );
}

/// Duas formas escolhidas com tintas de BASE (azuis, sem camada) e uma terceira sem base nenhuma.
fn cena_de_base() -> (VecScene, Vec<u64>, u64) {
    let mut scene = VecScene::default();
    let mut forma = |base: bool| {
        scene.push_path(VecPath {
            verts: [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0]]
                .map(VecVertex::corner)
                .to_vec(),
            closed: true,
            fill: base.then_some(Paint::Solid(AZUL)),
            stroke: base.then(|| StrokeSpec::new(AZUL, 1.0)),
            ..VecPath::default()
        })
    };
    let sel = vec![forma(true), forma(true)];
    let sem = forma(false);
    (scene, sel, sem)
}

fn cor_base(s: &VecScene, id: u64, linha: usize) -> Option<Rgba8> {
    s.path(id).unwrap().cor_da_linha(linha)
}

/// ⭐ A lista fala com as tintas de base pelo MESMO índice do Inspector: aplicar, criar e soltar.
/// *Mutação: `drena` a tratar a linha só como camada ⇒ nada liga.*
#[test]
fn the_list_applies_creates_and_detaches_on_the_base_paints() {
    let (mut s, sel, sem) = cena_de_base();
    let mut store = ph2d_editor_core::WidgetStore::default();
    let _ = lista::take_pedidos();

    // Aplicar no Fill: as duas escolhidas; a sem base fica de fora e o Stroke não muda.
    let id = s.add_estilo("Grass", LARANJA);
    lista::pede(Pedido::Aplicar {
        estilo: id,
        camada: LINHA_BASE_FILL,
    });
    let mut todas = sel.clone();
    todas.push(sem);
    assert!(drena(&mut store, &mut s, &todas), "aplicar não gravou nada");
    for f in &sel {
        assert_eq!(cor_base(&s, *f, LINHA_BASE_FILL), Some(LARANJA));
        assert_eq!(cor_base(&s, *f, LINHA_BASE_STROKE), Some(AZUL));
    }
    assert_eq!(
        s.estilo_de((sem, ItemId::BASE_FILL)),
        None,
        "ligou uma base que não existe"
    );
    assert_eq!(lista::estilos()[0].usos, 2);
    assert_eq!(lista::ligado(LINHA_BASE_FILL).map(|e| e.id), Some(id));
    assert_eq!(lista::ligado(LINHA_BASE_STROKE), None);

    // Criar no Stroke: o estilo nasce com a cor dele e liga as duas.
    lista::pede(Pedido::Novo {
        camada: LINHA_BASE_STROKE,
        nome: "Line".into(),
    });
    assert!(drena(&mut store, &mut s, &sel));
    let line = lista::estilos()[1].clone();
    assert_eq!(line.rgba, [AZUL.r, AZUL.g, AZUL.b, AZUL.a]);
    assert_eq!(
        (line.usos, lista::ligado(LINHA_BASE_STROKE).map(|e| e.id)),
        (2, Some(line.id))
    );

    // Soltar deixa a cor; soltar um não solta o outro.
    lista::pede(Pedido::Soltar {
        camada: LINHA_BASE_FILL,
    });
    assert!(drena(&mut store, &mut s, &sel));
    assert_eq!(lista::ligado(LINHA_BASE_FILL), None);
    assert_eq!(cor_base(&s, sel[0], LINHA_BASE_FILL), Some(LARANJA));
    assert_eq!(
        lista::ligado(LINHA_BASE_STROKE).map(|e| e.id),
        Some(line.id)
    );
}
