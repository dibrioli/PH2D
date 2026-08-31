//! **`PH2D_MOTION_OBJ_SMOKE=12` — A FOLHA À FRENTE DOS GALHOS.**
//!
//! ⛔⛔ **Esta cena existe porque a `=108` NÃO PODE mostrar isto** — report do Enio
//! (2026-08-30): *"Leaves in front não funciona, nada muda"*. E ele estava certo sobre o que
//! via: a ordem de composição de um quadro desta casa é
//!
//! | passe | alvo |
//! |---|---|
//! | 1 — sprites | `game_rt`, `Rgba16Float` **HDR** |
//! | 3 — Vello (chrome + o vector do documento) | intermediário `Rgba8Unorm` |
//!
//! ⇒ **todo vector fica por cima de todo sprite, por construção** — e a `=108` só tem folhas
//! que são IMAGENS. ⚠️ A nota que dizia isto citava o `vello` 0.8; reconferido no fonte do
//! **0.10** (o stack subiu em 29/08): o `render_to_texture` continua a exigir `Rgba8Unorm`, e o
//! alvo HDR está fora do alcance da biblioteca. *Não é uma escolha nossa.*
//!
//! ⇒ o `Leaves In Front` só tem sujeito quando a folha é uma **FORMA DESENHADA** — que é o
//! padrão-ouro da indústria para folhagem (uma folha é um *card* de geometria, não um sprite à
//! parte, e é por isso que ela se intercala com os ramos). Esta cena põe uma na mão dele.
//!
//! ⚠️ **A dança de dois frames é a do `=2`**: a forma entra no frame 3, e a ENTIDADE dela — que
//! é onde o nome mora — só existe depois do `vec_entities::sync`, no frame 6.

use ph2d_ecs::{Name, Transform};
use ph2d_node_source_lsystem as ls;
use ph2d_render::Sprite;

/// O nome que o campo *Leaf (J)* vai oferecer.
pub(super) const LEAF: &str = "Leaf";

/// ⭐⭐⭐ **A folha é uma IMAGEM, de propósito** — é o caso que o report nomeia (*"Leaves in
/// front ainda não funciona quando a folha é IMG"*), e o que a terceira média destrava. Uma
/// folha DESENHADA sempre pôde ir à frente; ela não prova nada aqui.
pub(super) fn spawn_leaf_sprite(sim: &mut ph2d_ecs::SimWorld) {
    sim.world_mut().spawn((
        Transform::from_translation(ph2d_core::Vec2::new(0.0, 0.0)),
        Sprite::atlas(super::DEMO_TILE_KEY, [0.5, 0.5], [1.0, 1.0, 1.0, 1.0]),
        Name::new(LEAF),
    ));
}

/// Monta a planta com a folha nomeada e METADE das folhas à frente.
pub(super) fn run(gfx: &mut crate::AppGfx) {
    let g = &mut gfx.motion.doc.graph;
    let l = g.add_node(ls::MANIFEST.name);
    g.set_param(l, ls::param::MODE, ls::MODE_GRAMMAR as f32);
    g.set_text_param(l, ls::AXIOM_PARAM, ls::PRESETS[0].axiom);
    g.set_text_param(l, ls::RULES_PARAM, ls::PRESETS[0].rules);
    g.set_param(l, ls::param::GENERATIONS, ls::PRESETS[0].generations);
    g.set_param(l, ls::param::ANGLE, ls::PRESETS[0].angle);
    g.set_param(l, ls::param::STEP, ls::PRESETS[0].step);
    g.set_param(l, ls::param::WIDTH, ls::PRESETS[0].width);
    g.set_param(
        l,
        ls::param::LEAF_FIRST_LEVEL,
        ls::PRESETS[0].leaf_first_level,
    );
    g.set_text_param(l, ls::LEAF_PARAMS[0], LEAF);
    // ⭐ **METADE à frente** — o número que torna a feature VISÍVEL num quadro parado. Com `0`
    // ou `1` a cena mostraria uma ordem só, e uma cena que só mostra um estado não ensina o
    // que o knob faz.
    g.set_param(l, ls::param::LEAF_FRONT, 0.5);
    // As folhas grandes, para a sobreposição com o galho se ver de longe.
    g.set_param(l, ls::param::LEAF_SIZE, 1.6);
    g.set_param(l, ls::param::LEAF_SIZE_JITTER, 0.5);
    g.set_param(l, ls::param::LEAF_POS_JITTER, 0.4);
    g.set_param(l, ls::param::LEAF_SPREAD, 90.0);
    let out = g.add_node("motion.output");
    if g.connect(ph2d_nodegraph::graph::Edge {
        from: (l, 0),
        to: (out, 0),
        delayed: false,
    })
    .is_err()
    {
        return;
    }
    gfx.motion.sinks.push(out);
    let _ = gfx.tools.set_active(&ph2d_editor::ToolId::new("motion"));
    eprintln!(
        "[motion.obj smoke =12] UMA ARVORE COM FOLHAS DESENHADAS.

  Clique na planta e abra a seccao «Leaves» no painel.

  · «Leaves In Front» esta' em 0,50: METADE das folhas esta' A' FRENTE dos galhos e
    metade ATRAS. Suba para 1 e todas passam a' frente; baixe para 0 e todas vao para
    tras. Aproxime o zoom num galho grosso para ver a diferenca.
  · «Leaf Size» / «Size Jitter» / «Position Jitter» / «Leaf Spread» mudam o tamanho, a
    variacao entre folhas, o quanto elas se desencostam do ramo e o quanto viram.

  ⭐ A FOLHA AQUI E' UMA IMAGEM, e e' esse o ponto: ate' 2026-08-30 uma imagem NUNCA podia
  ficar a' frente de um galho, porque o programa desenha todas as imagens ANTES de todo o
  desenho vectorial. Com o knob acima de 0 a copa passa a desenhar-se na mesma camada da
  arvore, e ai' a ordem manda -- sem perder nitidez e sem mudar uma cor."
    );
}
