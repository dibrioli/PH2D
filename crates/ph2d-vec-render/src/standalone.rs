//! **DESENHAR UMA FORMA FORA DO DOCUMENTO** — irmão de [`super`] pelo teto de 700 LOC, e o corte é
//! por RESPONSABILIDADE: ali fica o que compõe a CENA (a pilha de z, os clips, os produtores
//! vivos); aqui, as duas portas que desenham UMA forma isolada num alvo qualquer — o bake de um
//! tile, uma prévia, um carimbo.
//!
//! ⚠️ **O `standalone_tests.rs` já existia** e apontava para cá antes de este ficheiro existir: os
//! gates desta família foram cortados em 2026-08-XX e as funções ficaram para trás. *Um ficheiro de
//! testes que nomeia um módulo que não existe é um corte que parou a meio.*

use super::*;

/// **Desenha UM caminho avulso**, pela mesma [`draw_path`] do `dispatch` — a porta
/// que o bake de tile de uma forma usa, e que garante que o tile é o que a forma
/// PARECE, não uma segunda rasterização.
///
/// ⚠️ **Sem tint de instância, de propósito.** O `tint` é por-CÓPIA e multiplica o
/// tile no shader de sprite; aplicá-lo aqui pintaria a cor duas vezes. É por isso
/// que o BRANCO abaixo é a identidade certa, e não uma escolha de cor.
///
/// ⚠️ **Passa pela porta de INSTÂNCIA e não pela do DOCUMENTO, e a diferença foi um
/// bug** (Enio, 2026-08-20: *"não funcionou"*). Um `source.shape` nu — sem fill nem
/// stroke autorados — é um **PRIMITIVO**, e as duas portas discordam sobre ele: a
/// de instância ([`draw_shape_instance_tessellated`]) tem um ramo que preenche a
/// SILHUETA, e a do documento ([`draw_path`]) não desenha nada. O tile saía
/// **totalmente transparente**, o quad desenhava nada, e o modo de falha é mudo —
/// medido pelo `PH2D_GLOW_DIAG`, que mostrava a camada CERTA (`tile_forma=1`,
/// `camada=120`) e nenhum halo.
///
/// ⚠️ *Escolher a porta pelo que ela É (uma forma de instância) e não pelo que ela
/// PARECE (um caminho) é a regra; o crispo desta mesma forma passa por aqui.*
pub fn draw_path_standalone(path: &VecPath, transform: Affine, target: &mut VectorScene) {
    let tess = instance::tessellate_shape_instance(path);
    instance::draw_shape_instance_tessellated(path, &tess, transform, [1.0, 1.0, 1.0, 1.0], target);
}

/// O transbordo do traço sobre a caixa do fill — extraído junto com [`path_bounds_under`].
pub(crate) fn inflate_for_stroke(path: &VecPath, xf: Affine, r: Rect) -> Rect {
    let mut r = r;
    {
        // O traço transborda o fill por metade da largura; escala com o afim.
        //
        // ⚠️ **E uma junta MITER vai MUITO além disso.** Numa quina de ângulo interno `θ` a ponta do
        // miter fica a `½w / sin(θ/2)` do vértice — numa ponta de estrela (36°), **3,24 × ½w** —, e
        // a kurbo só a corta no `miter_limit`. Inflar por meia largura recortava a ponta contra a
        // borda do scratch, e o efeito visível era **a ponta CEIFADA** (reportado no smoke). O
        // limite é lido do MESMO construtor que o renderer usa, não de uma segunda constante.
        if let Some(s) = path.stroke {
            let [a, b, c, d, _, _] = xf.as_coeffs();
            let sx = (a * a + b * b).sqrt();
            let sy = (c * c + d * d).sqrt();
            // Só a JUNTA e o `miter_limit` são lidos aqui; um tracejado não muda o
            // transbordo, então medir o caminho para o ajustar seria trabalho por nada.
            let k = kurbo_stroke(&s, None);
            let reach = if matches!(k.join, Join::Miter) {
                k.miter_limit.max(1.0)
            } else {
                1.0
            };
            let m = 0.5 * s.width * sx.max(sy) * reach;
            r = r.inflate(m, m);
        }
    }
    r
}

/// **Desenha exatamente UM caminho, como o [`dispatch`] o desenharia, transladado por `offset`
/// (px de tela)** — a rasterização da forma isolada que o produtor de FX (plano 24) lê de volta.
///
/// Honra a geometria DERIVADA (`live`) e a pose, igual ao `dispatch`, então o que o FX borra é
/// exatamente o que a forma É na tela; o `offset` leva o bbox da forma à origem `(0,0)` do scratch.
/// Passa pela MESMA [`draw_path`] do `dispatch` — desenhar por uma 2ª porta faria o FX divergir do
/// que a forma parece de verdade.
///
/// ⛔⛔ **REPORT DO ENIO (2026-08-27): *"filters anula pattern"*.** O `tile` é obrigatório e não
/// opcional por isso: sem ele esta função chamava a `draw_path`, que passa `None`, e uma forma com
/// padrão era rasterizada com a **cor de recurso**. Como no `dispatch` a imagem de FX **toma o
/// lugar** do desenho, ligar um filtro apagava o padrão.
///
/// ⚠️ O doc acima já declarava a lei que isso partia — *"passa pela MESMA `draw_path`"* —, e a
/// segunda porta apareceu **dentro** da primeira quando o ladrilho virou um parâmetro que só o
/// `dispatch` sabia preencher. *Um argumento novo com um default é uma porta nova sem nome.*
#[allow(clippy::too_many_arguments)]
pub fn draw_path_isolated(
    scene: &VecScene,
    xforms: &VecXforms,
    live: &LiveGeometry,
    patterns: &crate::PatternTiles,
    id: VecPathId,
    camera: Affine,
    offset: Affine,
    target: &mut VectorScene,
) {
    let tile = patterns.get(&id);
    if let Some(items) = live.get(&id) {
        for item in items {
            crate::draw_path_tiled(item, offset * camera, target, tile);
        }
    } else if let Some(path) = scene.paths().iter().find(|p| p.id == id) {
        crate::draw_path_tiled(
            path,
            offset * path_to_screen(xforms, id, camera),
            target,
            tile,
        );
    }
}
