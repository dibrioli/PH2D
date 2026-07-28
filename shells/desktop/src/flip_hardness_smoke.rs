//! **A cena pronta para o smoke da DUREZA** (`PH2D_FLIP_HARDNESS_SMOKE=1`, 03 §8.6).
//!
//! É a foto do Enio encenada: *"Tudo que quero é que tenha o aspecto do traço do nosso próprio
//! módulo painter digital"* (2026-07-28, 4ª rodada, com setas vermelhas sobre cunhas ESCURAS nas
//! quinas de um rabisco que cruza a si mesmo).
//!
//! ⚠️ **A cura foi UMA frase:** o Flip desenha um **TRAÇO**, então o perfil dele tem de ser o
//! perfil de **TRAÇO** do Painter (a fileira de dabs a `spacing × diâmetro` de arco, composta por
//! `over`), nunca o de um **DAB** dele. As duas rodadas anteriores igualaram a lei do dab — que é
//! muito mais RALA — e é isso que abria as cunhas.
//!
//! **Números MEDIDOS** (`ph2d-flip-render/tests/painter_look.rs`, contra o depósito de verdade,
//! numa estrela de um traço só):
//!
//! | | falta de tinta | px fora de 16 |
//! |---|---|---|
//! | lei do DAB (o que a foto mostra) | **−112 de 255** | 613 |
//! | lei do TRAÇO (agora) | **−4** | 166, TODOS de SOBRA |
//!
//! E num traço RETO o Flip virou o depósito do Painter ao **±1 de 255**.
//!
//! ⚠️ **`hardness = 1.0` é byte-idêntico nas duas leis** (disco duro), e é o default do Flip ⇒
//! o X da ESQUERDA é o CONTROLE: se ele mudou, quebrei outra coisa.

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_HARDNESS_SMOKE").is_some())
}

/// As durezas encenadas. A 1ª é o CONTROLE (byte-idêntica nas duas leis).
const HARDNESS: [f32; 3] = [1.0, 0.7, 0.4];
/// O `dn` de meia-tinta sob a lei de TRAÇO (a de hoje), medido — a metade VISÍVEL da largura.
const HALF_INK_NOW: [f32; 3] = [1.000, 0.899, 0.824];
/// O mesmo número sob a lei de DAB (a rodada anterior, que o Enio reprovou).
const HALF_INK_WAS: [f32; 3] = [1.000, 0.850, 0.700];

/// **UM cruzamento** — duas retas que se cortam no centro `(cx, 0)`, exatamente a figura da foto.
/// O X é a fixture certa porque é onde o defeito lia pior: a cauda macia de uma passagem sobre o
/// NÚCLEO da outra. Traço GROSSO de propósito: o perfil precisa de várias linhas para se ver.
fn crossing(cx: f32, hardness: f32) -> [FlipStroke; 2] {
    let ink = Rgba::new(0.92, 0.92, 0.95, 1.0);
    let arm = 0.62_f32;
    let mut out = Vec::with_capacity(2);
    for (dx, dy) in [(arm, arm), (arm, -arm)] {
        let mut s = FlipStroke::new();
        for k in [-1.0_f32, 1.0] {
            s.push_point(Point {
                pos: Vec2::new(cx + k * dx, k * dy),
                width: 0.42,
                opacity: 1.0,
                color: ink,
            });
        }
        s.hardness = hardness;
        out.push(s);
    }
    let mut it = out.into_iter();
    [it.next().expect("perna 1"), it.next().expect("perna 2")]
}

/// **A ESTRELA DE UM TRAÇO** — a figura da foto: quinas de 36° em cada ponta e cinco
/// auto-cruzamentos no miolo, desenhada **sem levantar a caneta**.
///
/// ⚠️ **É esta a cena que faltava.** As rodadas anteriores encenavam cada cruzamento como DOIS
/// traços, e dois traços cruzados nunca tiveram o defeito (o depth deles difere e o mais novo
/// pinta por cima, ou seja **já compõe**). O caso do Enio — um traço só, com quina — exigia
/// desenhar à mão para aparecer.
fn one_stroke_star(cx: f32, hardness: f32) -> FlipStroke {
    let ink = Rgba::new(0.92, 0.92, 0.95, 1.0);
    let outer = 0.80_f32;
    let mut corners = Vec::new();
    for k in 0..5 {
        // Passo de 2/5 de volta = a estrela de um traço só.
        let a = -std::f32::consts::FRAC_PI_2 + (k as f32) * 4.0 * std::f32::consts::PI / 5.0;
        corners.push((cx + outer * a.cos(), outer * a.sin()));
    }
    corners.push(corners[0]);
    let mut s = FlipStroke::new();
    for &(x, y) in &corners {
        s.push_point(Point {
            pos: Vec2::new(x, y),
            width: 0.42,
            opacity: 1.0,
            color: ink,
        });
    }
    s.hardness = hardness;
    s
}

/// **Monta a cena** — porta única (a mensagem encena por aqui). Devolve os x dos três grupos.
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> [f32; 3] {
    obj.fps = 12.0;
    obj.onion.enabled = false; // um quadro só; o onion sujaria a leitura do perfil.

    let xs = [-2.2_f32, -0.3, 1.9];
    let layer = obj.add_layer("Hardness");
    if let Some(d) = obj.insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe) {
        let strokes = &mut obj.drawing_mut(d).expect("desenho").strokes;
        // Dois X de DOIS traços: o controle duro e o macio.
        strokes.extend(crossing(xs[0], HARDNESS[0]));
        strokes.extend(crossing(xs[1], HARDNESS[2]));
        // E a ESTRELA de UM traço — a foto.
        strokes.push(one_stroke_star(xs[2], HARDNESS[2]));
    }
    xs
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_hardness_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Hardness Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recem-criado");
        let xs = stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[hardness-smoke] cena montada: 2 cruzamentos + 1 ESTRELA DE UM TRACO em \
             x={:?}, hardness {:?}. Ferramenta flip ativa: {}.",
            xs,
            [HARDNESS[0], HARDNESS[2], HARDNESS[2]],
            if tool_ok {
                "sim"
            } else {
                "NAO (PARE: sem ela o traco nao e dirigido pela tool Flip)"
            }
        );

        let mut tabela = String::new();
        for i in [0usize, 1, 2] {
            tabela.push_str(&format!(
                "                 {:>4.1}         {:>5.3}          {:>5.3}\n",
                HARDNESS[i], HALF_INK_WAS[i], HALF_INK_NOW[i]
            ));
        }

        eprintln!(
            "\n\
             ============================================================\n\
             ANTES DE TUDO: este terminal imprimiu, logo acima, a linha\n\
             comecando com '[hardness-smoke] cena montada'? Se NAO, PARE:\n\
             o smoke nao rodou (arvore ou variavel de ambiente errada).\n\
             ============================================================\n\
             \n\
             A CENA, da esquerda para a direita:\n\
               1. X duro (hardness 1.0) -- o CONTROLE. As duas leis sao\n\
                  byte-identicas aqui, e este e o default do Flip. Se ele\n\
                  mudou, algo mais quebrou.\n\
               2. X macio (hardness 0.4), DOIS tracos cruzados.\n\
               3. ESTRELA de UM traco (hardness 0.4) -- **A SUA FOTO**:\n\
                  quina afiada em cada ponta e cinco auto-cruzamentos.\n\
                  As rodadas anteriores so encenavam o item 2, e dois\n\
                  tracos cruzados NUNCA tiveram o defeito -- por isso ele\n\
                  so aparecia quando voce desenhava a mao.\n\
             \n\
             O QUE OLHAR -- e e so isso:\n\
               1. Na ESTRELA, nenhuma cunha ESCURA mordendo a tinta nas\n\
                  quinas nem nos cruzamentos. Era isso que as setas\n\
                  vermelhas apontavam.\n\
               2. Abra o PAINTER, pincel digital normal, MESMA hardness, e\n\
                  rabisque uma estrela sem levantar a caneta. O aspecto\n\
                  tem de ser o MESMO -- e a razao desta wave existir.\n\
               3. O X da esquerda nao pode ter mudado.\n\
             \n\
             ------------------------------------------------------------\n\
             A CURA, numa frase: o Flip desenha um TRACO, entao o perfil\n\
             dele e o perfil de TRACO do Painter (a fileira de dabs\n\
             composta por `over`), nunca o de um DAB dele. As duas\n\
             rodadas anteriores igualaram a lei do DAB, que e muito mais\n\
             rala -- em hardness 0.4 e dn 0.70 um dab pesa 0.500 e o\n\
             traco pesa 0.916.\n\
             \n\
             Medido: o dn onde a tinta cruza meia-tinta (= a metade\n\
             VISIVEL da largura pedida).\n\
             \n\
             hardness      lei do DAB      lei do TRACO\n\
             {tabela}\
             ------------------------------------------------------------\n\
             \n\
             Contra o deposito REAL do Painter (sonda `painter_look.rs`,\n\
             a mesma estrela): ZERO pixel com MENOS tinta que o Painter,\n\
             em toda a faixa de hardness, e num traco RETO o Flip virou\n\
             o deposito dele ao +-1 de 255.\n\
             \n\
             ⚠️ RESIDUO NOMEADO: na PONTA de uma quina muito afiada os\n\
             dabs do Painter RECUAM em vez de correr paralelos, e o Flip\n\
             pinta ali um pouco mais cheio (+122 de 255 no vertice de\n\
             36 graus; some conforme a hardness sobe). E a direcao\n\
             OPOSTA a queixa -- a ponta fica mais redonda, nao mordida.\n\
             Se ISSO incomodar, reporte: e outra wave.\n"
        );
    }
}
