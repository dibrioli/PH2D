//! **A cena pronta para o smoke da DUREZA** (`PH2D_FLIP_HARDNESS_SMOKE=1`, 03 §8.6).
//!
//! É a foto do Enio (2026-07-28) encenada: *"o cruzamento de cima é o FLIP, o de baixo é do
//! Painter. O correto é o aspecto do cruzamento de baixo e o flip deveria ser idêntico"*.
//!
//! O `hardness_mask` do `flip.wgsl` era o `gpencil_stroke_round_cap_mask` ao pé da letra —
//! `smoothstep(0,1, pow(1−dn, mix(0,10,1−h)))` — fiel ao Blender e **incompatível com o resto do
//! app**: sem platô, o traço **ENCOLHE ao amaciar**. Agora a lei é a do Painter
//! (`BrushSpec::falloff_weight` + `Falloff::Smooth`): núcleo CHEIO até `hardness`, queda na faixa
//! restante.
//!
//! **Números MEDIDOS** (`ph2d-flip-render/tests/hardness_law.rs`, o `dn` onde a tinta cruza
//! meia-tinta — ou seja a metade VISÍVEL da largura pedida):
//!
//! | hardness | era (GP) | é (Painter) |
//! |---|---|---|
//! | 0,9 | 0,500 | 0,951 |
//! | 0,7 | 0,207 | 0,850 |
//! | 0,5 | **0,130** | 0,751 |
//!
//! Em hardness 0,5 a largura visível era **13% da pedida** — o resto era névoa, e é isso que
//! aparecia como um filete brilhante dentro de um borrão nos cruzamentos.
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

/// As três durezas encenadas. A 1ª é o CONTROLE (byte-idêntica nas duas leis).
const HARDNESS: [f32; 3] = [1.0, 0.7, 0.4];
/// O `dn` de meia-tinta sob a lei NOVA, medido em `hardness_law.rs` — o que a mensagem promete.
const HALF_INK_NOW: [f32; 3] = [1.000, 0.850, 0.701];
/// O mesmo número sob a lei ANTIGA (o que o Enio fotografou).
const HALF_INK_WAS: [f32; 3] = [1.000, 0.207, 0.110];

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

/// **Monta a cena** — porta única (a mensagem encena por aqui). Devolve os x dos três centros.
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> [f32; 3] {
    obj.fps = 12.0;
    obj.onion.enabled = false; // um quadro só; o onion sujaria a leitura do perfil.

    let xs = [-1.7_f32, 0.0, 1.7];
    let layer = obj.add_layer("Hardness");
    if let Some(d) = obj.insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe) {
        let strokes = &mut obj.drawing_mut(d).expect("desenho").strokes;
        for (x, h) in xs.iter().zip(HARDNESS) {
            strokes.extend(crossing(*x, h));
        }
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
            "\n[hardness-smoke] cena montada: 3 cruzamentos em x={:?}, hardness {:?}. \
             Ferramenta flip ativa: {}.",
            xs,
            HARDNESS,
            if tool_ok {
                "sim"
            } else {
                "NAO (PARE: sem ela o traco nao e dirigido pela tool Flip)"
            }
        );

        let mut tabela = String::new();
        for i in 0..3 {
            tabela.push_str(&format!(
                "                 {:>4.1}        {:>5.3}         {:>5.3}\n",
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
             A FOTO, ENCENADA. Tres cruzamentos identicos, so a hardness\n\
             muda -- da mais dura (esquerda) para a mais macia (direita):\n\
               - ESQUERDA (1.0) : o CONTROLE. As duas leis sao byte-\n\
                                  identicas aqui, e este e o default do\n\
                                  Flip. Se ele mudou, algo mais quebrou.\n\
               - MEIO     (0.7) : nucleo CHEIO ate 70% do raio, depois cai.\n\
               - DIREITA  (0.4) : nucleo CHEIO ate 40%, queda mais longa.\n\
             \n\
             O QUE OLHAR -- e e so isso:\n\
               1. Cada traco tem um MIOLO SOLIDO com uma borda macia.\n\
                  NAO pode ser um filete brilhante dentro de um borrao.\n\
               2. No CRUZAMENTO, o miolo de uma passagem nao pode ficar\n\
                  mais claro nem mais escuro: as duas se fundem lisas.\n\
               3. Abra o PAINTER, pincel normal, mesma hardness, e cruze\n\
                  dois tracos. O aspecto tem de ser o MESMO -- e a razao\n\
                  desta wave existir.\n\
             \n\
             ------------------------------------------------------------\n\
             Medido (sonda `hardness_law.rs`): o dn onde a tinta cruza\n\
             meia-tinta = a metade VISIVEL da largura pedida.\n\
             \n\
             hardness      era (GP)      e (Painter)\n\
             {tabela}\
             Em 0.4 a largura visivel era 11% da pedida. O resto,\n\
             que voce fotografou, era nevoa.\n\
             ------------------------------------------------------------\n\
             \n\
             ⚠️ O QUE ESTA WAVE **NAO** IGUALA, e o smoke decide se\n\
             importa: o Painter carimba dabs e os compoe por `over`, e\n\
             o que ele DEPOSITA e mais cheio que o falloff de um dab\n\
             (delta maximo medido 0.47 em hardness 0 / 0.41 em 0.5 /\n\
             0.20 em 0.9, so no aro). O Flip compoe por UNIAO, entao a\n\
             seccao dele E o perfil. Casar o deposito foi medido e\n\
             REJEITADO: ele depende do SPACING do Painter e nao tem\n\
             limite livre dele. Se o ombro ainda parecer fino ao lado do\n\
             Painter, ISSO e o residual -- reporte e a curva muda.\n"
        );
    }
}
