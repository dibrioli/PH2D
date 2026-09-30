//! ⭐⭐⭐ **AS ESTRELAS ESTICADAS COM CONTORNO** (cena `=127`) — a cena que MOSTRA a W4 do doc 121,
//! porque **nenhuma cena do catálogo a mostra**.
//!
//! # Porque ela existe
//!
//! A W4 levou à placa o traço de uma cópia **esticada num eixo só** (afim NÃO conforme): antes dela
//! uma cópia assim devolvia o quadro INTEIRO ao Vello. Medido pelo censo de rota
//! (`motion_bridge_gpu_rota_das_formas_probe`), **nenhuma** das cenas com forma do catálogo tem a
//! combinação *traço + escala não-uniforme* — a `=126` tem as estrelas mas sem contorno e com a
//! escala igual nos dois eixos, logo ela desenha exactamente o mesmo antes e depois da W4.
//! *Uma cena que não contém o fenómeno ensina que a cura não faz nada* (`CLAUDE.md` §5.0).
//!
//! # A CADEIA
//!
//! `motion.grid` → `motion.integrate` (a galáxia da `=126`, a MESMA lei) → `motion.duplicator` ←
//! `source.shape` (**Star**, com preenchimento e CONTORNO) → `motion.scale` (**`uniform = 0`**: o
//! X estica e o Y encolhe) → `motion.output`.
//!
//! ⭐ **A simulação é a da `=126`, reutilizada e não copiada** ([`super::carimbo_demo::simulacao`]):
//! o disco em rotação rígida cuja derivação vive lá (o doc da `GALAXIA`). Duas cópias da lei
//! divergiriam no dia em que alguém afinasse uma delas.
//!
//! # O que o dono tem de VER
//!
//! A lei do traço da casa (`ph2d_vec_render::stroke_uniform::pen_for`, bug #27 do vector: *«quando
//! engrossa, engrossa por igual nos dois eixos»*) pede a CANETA REDONDA: o contorno de uma estrela
//! esticada tem a MESMA grossura nos lados compridos e nos curtos. ⛔ Uma caneta elíptica (o que o
//! Vello faria com o traço expandido no espaço local) daria contornos grossos em cima e finos dos
//! lados — é esse o *«deu errado»* do roteiro.
//!
//! ⚠️ **As duas rotas desenham a MESMA lei** (o Vello e a placa, gate
//! `a_rota_da_placa_desenha_o_traco_esticado_como_a_casa`): o que muda entre o comando de sempre e
//! `PH2D_FORMAS_NA_PLACA=0` é a FOLGA do quadro (o `raw`), não a imagem.

use ph2d_motion_doc::MotionDoc;
use ph2d_node_registry::NodeRegistry;
use ph2d_nodegraph::graph::{Edge, NodeId, Pos};

use super::carimbo_demo::PX_POR_UNIDADE;

/// **O esticão**: o X cresce e o Y encolhe. ⚠️ Os dois têm de ser DIFERENTES — é isso que faz a
/// cópia NÃO conforme, e há gate. O produto (`1,08`) fica perto de `1` para a caneta
/// (`w·√|det|`) ter quase a largura autorada: o que a cena mostra é a FORMA dela, não o tamanho.
pub(crate) const ESTICA_X: f32 = 1.8;
pub(crate) const ESTICA_Y: f32 = 0.6;

/// **Os dois arranjos da cena**, e porque são dois.
///
/// ⛔⛔ **As duas leis puxam em sentidos opostos** (a mesma aritmética do cabeçalho da `=126`): o
/// CONTORNO só se julga numa estrela grande, e a FOLGA do quadro só se mexe com milhares de estrelas
/// à vista. Medido na foto (perfil `smoke`, as duas rotas seguidas): com o arranjo LEGÍVEL (`1 024`
/// estrelas de `22 px`, umas `300` à vista) a placa lê `186`/`209 raw` e o Vello `223`/`201` —
/// **ruído**, e o roteiro que prometia *«o `raw` cai»* ensinava uma coisa falsa. ⇒ o passo do relógio
/// corre o arranjo DENSO (`PH2D_TRACO_ESTICADO_DENSO=1`), e o do contorno o legível.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Arranjo {
    /// O lado da grelha.
    pub(crate) lado: u32,
    /// A pegada de uma estrela ANTES de esticar, em píxeis de ecrã na câmara de arranque.
    pub(crate) estrela_px: f32,
    /// O contorno, em píxeis de ecrã (o param é de mundo; sai daqui pela escala da câmara).
    pub(crate) contorno_px: f32,
}

/// O arranjo de omissão: estrelas grandes, o contorno julga-se a olho.
pub(crate) const LEGIVEL: Arranjo = Arranjo {
    lado: 32,
    estrela_px: 22.0,
    contorno_px: 1.5,
};

/// O arranjo do relógio: milhares de estrelas à vista na câmara de arranque.
pub(crate) const DENSO: Arranjo = Arranjo {
    lado: 128,
    estrela_px: 8.0,
    contorno_px: 1.0,
};

impl Arranjo {
    /// O `size` do `source.shape` é o **raio** (metade da pegada).
    pub(crate) fn tamanho(self) -> f32 {
        self.estrela_px / (2.0 * PX_POR_UNIDADE)
    }

    /// O vão entre posições — o MESMO nos dois eixos, e igual à pegada ESTICADA inteira.
    ///
    /// ⛔ A 1.ª redacção dava a cada eixo a pegada dele (`2,4 × tamanho × esticão`), e a foto
    /// desmentiu-a: a galáxia RODA as posições e o esticão fica sempre na horizontal, logo uma
    /// fileira que começa horizontal passa a diagonal e as estrelas de vão curto FUNDEM-SE numa
    /// tira contínua. Com o vão do eixo comprido nos dois sentidos a distância entre vizinhas nunca
    /// fica abaixo da largura esticada, seja qual for o ângulo.
    pub(crate) fn vao(self) -> f32 {
        2.0 * self.tamanho() * ESTICA_X
    }

    pub(crate) fn contorno(self) -> f32 {
        self.contorno_px / PX_POR_UNIDADE
    }

    pub(crate) fn estrelas(self) -> u32 {
        self.lado * self.lado
    }

    /// A distância do centro ao canto do campo.
    pub(crate) fn meia_diagonal(self) -> f32 {
        #[expect(clippy::cast_precision_loss, reason = "um lado de grelha pequeno")]
        let meio = self.vao() * (self.lado as f32 - 1.0) / 2.0;
        meio * std::f32::consts::SQRT_2
    }

    /// **A galáxia deste campo** — a lei da `=126` ([`Galaxia`](super::carimbo_demo::Galaxia)) com o
    /// NÚCLEO a cobrir o canto (fora dele o ímã ganha à órbita e as estrelas caem para o meio) e o
    /// ímã derivado do núcleo (`s_a = s_v² / núcleo`, o equilíbrio de todo raio, corrigido pelo
    /// `1 − d/alcance` a meio raio). ⚠️ Com o campo da `=126` ela devolve os números dela (gate).
    pub(crate) fn galaxia(self) -> super::carimbo_demo::Galaxia {
        let base = super::carimbo_demo::GALAXIA;
        let nucleo = (self.meia_diagonal() * 1.03).max(base.nucleo);
        let iman = base.vortex * base.vortex / nucleo * (1.0 - 0.5 * nucleo / base.alcance);
        super::carimbo_demo::Galaxia {
            nucleo,
            iman,
            ..base
        }
    }
}

/// **Que arranjo a cena monta** — o [`LEGIVEL`], a menos que `PH2D_TRACO_ESTICADO_DENSO=1`.
fn arranjo_semeado() -> Arranjo {
    arranjo_por(std::env::var("PH2D_TRACO_ESTICADO_DENSO").ok().as_deref())
}

/// A LEI da porta acima, **pura** (um gate que lê o ambiente mede a máquina).
pub(crate) fn arranjo_por(valor: Option<&str>) -> Arranjo {
    if valor.map(str::trim) == Some("1") {
        DENSO
    } else {
        LEGIVEL
    }
}

/// O índice da `Star` no enum — pela mesma porta da `=126` (nunca pelo rótulo, que é i18n).
fn indice_da_estrela() -> f32 {
    let i = ph2d_node_motion_shape::ALL_KINDS
        .iter()
        .position(|k| *k == ph2d_node_motion_shape::ShapeKind::Star)
        .expect("a `Star` tem de estar no `ALL_KINDS`");
    #[expect(
        clippy::cast_precision_loss,
        reason = "um indice de enum, sempre pequeno"
    )]
    {
        i as f32
    }
}

/// Constrói o documento. `None` se algum tipo de nó não estiver registado.
pub(super) fn build(doc: &mut MotionDoc, reg: &NodeRegistry) -> Option<Vec<NodeId>> {
    for tipo in [
        "motion.grid",
        "motion.duplicator",
        "source.shape",
        "motion.scale",
        "motion.output",
        "motion.integrate",
        "force.vortex",
        "force.attractor",
        "force.curl",
        "motion.falloff",
    ] {
        reg.manifests()
            .find(|m| m.id == ph2d_nodegraph::node::NodeTypeId::of(tipo))?;
    }
    let g = &mut doc.graph;
    let no = |g: &mut ph2d_nodegraph::graph::Graph, tipo: &str, x: f32, y: f32| {
        let n = g.add_node(tipo.to_string());
        g.set_pos(n, Pos { x, y });
        n
    };

    let arranjo = arranjo_semeado();
    let grade = no(g, "motion.grid", 0.0, 0.0);
    #[expect(clippy::cast_precision_loss, reason = "um lado de grelha pequeno")]
    let lado = arranjo.lado as f32;
    g.set_param(grade, "rows", lado);
    g.set_param(grade, "cols", lado);
    g.set_param(grade, "gap_x", arranjo.vao());
    g.set_param(grade, "gap_y", arranjo.vao());

    // ── A FORMA: uma estrela AMARELA com CONTORNO azul-escuro. ⚠️ Sem tracejado: o tracejado sob
    // escala não-uniforme continua no Vello (doc 121 §9) e a cena mostraria a rota errada.
    use ph2d_node_motion_shape::param as p;
    let forma = no(g, "source.shape", 0.0, 220.0);
    for (chave, valor) in [
        (p::KIND, indice_da_estrela()),
        (p::SIZE, arranjo.tamanho()),
        (p::FILL, 1.0),
        (p::FILL_R, 1.0),
        (p::FILL_G, 0.82),
        (p::FILL_B, 0.25),
        (p::FILL_A, 1.0),
        (p::STROKE_WIDTH, arranjo.contorno()),
        (p::STROKE_R, 0.06),
        (p::STROKE_G, 0.10),
        (p::STROKE_B, 0.35),
        (p::STROKE_A, 1.0),
    ] {
        g.set_param(forma, chave, valor);
    }

    // ── A SIMULAÇÃO COM CAMPOS: a galáxia da `=126` (regra do dono, doc 103 §1).
    let ig = super::carimbo_demo::simulacao_com(g, grade, &arranjo.galaxia())?;

    // ── O CARIMBO. A forma na porta `0`, os pontos na `1` (o manifesto do duplicador).
    let dup = no(g, "motion.duplicator", 240.0, 110.0);
    for (de, porta) in [(forma, 0u16), (ig, 1)] {
        g.connect(Edge {
            from: (de, 0),
            to: (dup, porta),
            delayed: false,
        })
        .ok()?;
    }

    // ── O ESTICÃO: um eixo cresce, o outro encolhe. É ele que faz cada cópia NÃO conforme.
    let estica = no(g, "motion.scale", 460.0, 110.0);
    g.set_param(estica, "uniform", 0.0);
    g.set_param(estica, "amount", ESTICA_X);
    g.set_param(estica, "amount_y", ESTICA_Y);
    g.connect(Edge {
        from: (dup, 0),
        to: (estica, 0),
        delayed: false,
    })
    .ok()?;

    let saida = no(g, "motion.output", 680.0, 110.0);
    g.connect(Edge {
        from: (estica, 0),
        to: (saida, 0),
        delayed: false,
    })
    .ok()?;
    Some(vec![saida])
}

/// **O roteiro que o dono segue.** Cada passo nomeia o que aparece NA TELA (`CLAUDE.md` §0.8).
pub(super) fn announce() {
    let n = LEGIVEL.estrelas();
    let d = DENSO.estrelas();
    eprintln!(
        "\n[estrelas esticadas] {n} ESTRELAS AMARELAS COM CONTORNO AZUL, esticadas para os lados:\n\
         cada uma fica mais larga do que alta. Elas GIRAM devagar, como uma galaxia: e' uma\n\
         SIMULACAO com campos de forca (redemoinho, iman, ruido), a mesma da cena =126.\n\
         \n\
         (1) Aproxime com a roda do rato ate' uma estrela ocupar um bom pedaco do ecra.\n    \
         Olhe o CONTORNO AZUL: ele tem a MESMA grossura nas pontas compridas (dos lados) e\n    \
         nas curtas (em cima e em baixo), como um risco feito com uma caneta redonda, e nao\n    \
         ha' nenhuma LINHA a atravessar a estrela.\n\
         (2) Arraste o fundo com o botao do meio: tem de passear LISO, e as estrelas continuam\n    \
         a girar.\n\
         (3) Feche o app e corra o MESMO comando com `PH2D_TRACO_ESTICADO_DENSO=1` a' frente:\n    \
         agora sao {d} estrelas pequenas. ANOTE o terceiro numero da barra de baixo, o `raw`\n    \
         (a folga do quadro: quanto MAIOR, melhor).\n\
         (4) Feche e corra com `PH2D_TRACO_ESTICADO_DENSO=1 PH2D_FORMAS_NA_PLACA=0` a' frente:\n    \
         e' o caminho antigo, em que o processador desenhava as estrelas. A imagem tem de ser\n    \
         a MESMA; o `raw` CAI.\n\
         \n\
         DEU ERRADO se: as estrelas nao aparecerem; se ficarem PARADAS; se o contorno for\n\
         GROSSO em cima e em baixo e FINO dos lados (ou ao contrario); se houver uma LINHA\n\
         escura a atravessar as estrelas; se o contorno tiver BURACOS ou DENTES nas pontas;\n\
         se a imagem for DIFERENTE entre as corridas (3) e (4); ou se o `raw` nao cair.\n"
    );
}

#[cfg(test)]
#[path = "motion_state_traco_esticado_demo_tests.rs"]
mod tests;
