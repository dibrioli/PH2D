//! **A cena pronta para o smoke do AIRBRUSH** (`PH2D_FLIP_AIRBRUSH_SMOKE=1`, 03 §8).
//!
//! O pincel airbrush analítico (Ciallo): o falloff da borda deixa de ser o `pow`+smoothstep e vira
//! a **transmitância física** da tinta por um dab esférico (Beer-Lambert): `A = 1 − exp(−k·√(1−dn²))`,
//! `k = mix(1, 8, hardness)`. É um **domo largo** de núcleo chato e borda SEMPRE macia — o oposto do
//! pico estreito do `pow`. O slider Hardness vira a densidade da névoa.
//!
//! A cena põe o MESMO traço grosso lado a lado, na MESMA hardness (0.5):
//!
//! - **ESQUERDA (padrão)** — `airbrush = false`, o `pow`+smoothstep de sempre: um **pico** — o
//!   núcleo fino é opaco e a tinta some rápido do eixo. É o default, byte-idêntico ao Flip de sempre.
//! - **DIREITA (airbrush)** — `airbrush = true`: um **domo** — a tinta cobre quase toda a largura
//!   antes de rolar suave a zero na borda.
//!
//! **Números MEDIDOS** (a sonda headless é o gate GPU `an_airbrush_has_a_flatter_core_than_the_
//! standard_brush`, banda de raio 10 a hardness 0.5): no EIXO os dois ficam ~255; no MEIO-RAIO
//! (dn≈0.5) o padrão DESABA para **~1** (pico estreito) e o airbrush fica CHEIO em **~250** (domo);
//! perto da borda (dn≈0.8) o airbrush ainda pinta **~238** e rola a zero. Casa com o Self Overlap:
//! a acumulação `over` de airbrush é a multiplicação de transmitâncias (o build-up físico).

use ph2d_core::Vec2;
use ph2d_flip::{FlipStroke, Hold, KeyKind, Point, Rgba};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU32, Ordering};

static FRAME: AtomicU32 = AtomicU32::new(0);

fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("PH2D_FLIP_AIRBRUSH_SMOKE").is_some())
}

/// UM traço reto e GROSSO (vertical), na hardness 0.5: onde o padrão vira pico e o airbrush vira
/// domo. `dx` desloca a cópia em x; `airbrush` troca o falloff. Opacity 1.0 (o falloff É o ponto).
fn thick_stroke(dx: f32, airbrush: bool) -> FlipStroke {
    let ink = Rgba::new(0.20, 0.55, 0.85, 1.0); // um azul de tinta, o mesmo nos dois
    let mut s = FlipStroke::new();
    for &y in &[-0.9_f32, 0.9] {
        s.push_point(Point {
            pos: Vec2::new(dx, y),
            width: 0.7, // grosso: várias linhas de queda, o perfil aparece
            opacity: 1.0,
            color: ink,
        });
    }
    s.hardness = 0.5;
    s.airbrush = airbrush;
    s
}

/// **Monta a cena** — porta única (o gate/mensagem encenam por aqui). Uma camada, um quadro: o
/// traço padrão à esquerda, o airbrush à direita. Devolve `(x_std, x_air)` das duas cópias.
pub(crate) fn stage(obj: &mut ph2d_flip::FlipObject) -> (f32, f32) {
    obj.fps = 12.0;
    obj.onion.enabled = false; // um quadro só; o onion sujaria a leitura do perfil.

    let (x_std, x_air) = (-1.2, 1.2);
    let layer = obj.add_layer("Airbrush");
    if let Some(d) = obj.insert_frame(layer, 0, Hold::Implicit, KeyKind::Keyframe) {
        let strokes = &mut obj.drawing_mut(d).expect("desenho").strokes;
        strokes.push(thick_stroke(x_std, false)); // ESQUERDA: padrão (pico)
        strokes.push(thick_stroke(x_air, true)); // DIREITA: airbrush (domo)
    }
    (x_std, x_air)
}

impl crate::App {
    /// Roda no prólogo do frame (ao lado dos outros smokes). No-op sem a env.
    pub(crate) fn flip_airbrush_smoke(&mut self) {
        if !enabled() || self.gfx.is_none() {
            return;
        }
        if FRAME.fetch_add(1, Ordering::Relaxed) != 3 {
            return;
        }
        let gfx = self.gfx.as_mut().expect("gfx");
        let tool_ok = gfx.tools.set_active(&ph2d_editor::ToolId::new("flip"));

        let oid = gfx.flip.push_object("Airbrush Smoke");
        let obj = gfx.flip.object_mut(oid).expect("objeto recem-criado");
        let (x_std, x_air) = stage(obj);

        self.playhead.seek(0.0);
        self.playhead.pause();

        eprintln!(
            "\n[airbrush-smoke] cena montada: 2 tracos grossos a hardness 0.5 -- padrao em x={x_std}, \
             airbrush em x={x_air}. Ferramenta flip ativa: {}.",
            if tool_ok {
                "sim"
            } else {
                "NAO (PARE: sem ela o traco nao e dirigido pela tool Flip)"
            }
        );
        eprintln!(
            "\n\
             ============================================================\n\
             ANTES DE TUDO: este terminal imprimiu, logo acima, a linha\n\
             comecando com '[airbrush-smoke] cena montada'? Se NAO, PARE:\n\
             o smoke nao rodou (arvore ou variavel de ambiente errada).\n\
             ============================================================\n\
             \n\
             O que esta na tela: o MESMO traco grosso, a MESMA hardness\n\
             (0.5), desenhado DUAS vezes:\n\
               - ESQUERDA : Airbrush OFF -- o pincel padrao (pow): um\n\
                            PICO, nucleo fino opaco, some rapido do eixo.\n\
               - DIREITA  : Airbrush ON  -- um DOMO largo: a tinta cobre\n\
                            quase toda a largura antes de rolar suave a\n\
                            zero na borda (borda SEMPRE macia).\n\
             \n\
             ------------------------------------------------------------\n\
             Medido (sonda headless, banda raio 10 a hardness 0.5):\n\
                 eixo (centro)     ~255  nos dois\n\
                 meio-raio (dn.5)  ~1 padrao   vs  ~250 airbrush\n\
                 borda (dn.8)      ~0 padrao   vs  ~238 airbrush\n\
             ------------------------------------------------------------\n\
             \n\
             O AJUSTE: no painel Flip (modo DRAW), abaixo do toggle Self\n\
             Overlap, ha o toggle 'Airbrush'. Ligado, o slider Hardness\n\
             vira a DENSIDADE da nevoa (0 = tenue, 1 = domo quase solido\n\
             de borda macia). Casa com o Self Overlap: a acumulacao de\n\
             airbrush e o build-up fisico da tinta. Se algo destoar, diga.\n"
        );
    }
}
