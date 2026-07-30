//! **A FRESCURA de uma camada de Flip** — a pergunta *"o que este frame produziria já está na
//! tela?"*, e ela é o passo 2 do padrão-ouro (doc 12 §22.3).
//!
//! Irmão do `flip_pass.rs`: lá mora *o que é composto*, aqui mora *se precisa ser composto de novo*.
//!
//! ## Por que isto existe
//!
//! O Pass A rasterizava **por camada, por frame, SEMPRE** — panhando, parado, em playback. Nenhum
//! app de pintura faz isso (o nosso Painter não faz: canvas + preview + dirty-rect), e o Flip
//! multiplica o preço por conta própria, porque **cada fantasma do onion é uma camada** com o seu
//! `stage_layer`. Medido no §21.2: 4,33 ms por camada a 200 traços em 1080p ⇒ com onion de 3
//! vizinhos, um quadro inteiro de 60 fps gasto todo frame para redesenhar pixels idênticos.
//!
//! ## A lei do skip, e as duas metades dela
//!
//! Pular é correto quando **duas** coisas valem, e cada uma responde uma pergunta que a outra não
//! pode:
//!
//! 1. **a impressão digital bate** — o que este frame produziria é byte a byte o que foi produzido
//!    (a nossa metade, o [`StageMemo`]);
//! 2. **o compositor AINDA tem a fatia** — `LayerCompositor::has_slice` (a metade DELE: a fatia pode
//!    ter sido despejada pelo LRU ou limpa por um rebuild do array, e nenhum memo nosso sabe disso).
//!
//! ⚠️ **Sem a (2) o modo de falha é arte VELHA na tela** e nada parece quebrado. Com ela, o pior
//! caso é *fazer o trabalho* — que é exatamente o que o produto fazia antes desta wave.

use ph2d_flip_render::{CameraRaw, FlipGpuData};
use std::collections::BTreeMap;

/// O que já foi rasterizado em cada fatia do compositor: `chave → impressão digital`.
#[derive(Default)]
pub(super) struct StageMemo {
    // `BTreeMap` (não `HashMap`): ADR-0022 / HR-5 — determinístico, e o mapa é do tamanho do número
    // de camadas visíveis (unidades), então O(log n) é irrelevante.
    map: BTreeMap<u64, u64>,
    /// Instrumentação por frame (`PH2D_FLIP_STATS=1`): camadas rasterizadas vs puladas.
    staged: u32,
    skipped: u32,
}

impl StageMemo {
    pub(super) fn reset_stats(&mut self) {
        self.staged = 0;
        self.skipped = 0;
    }

    pub(super) fn stats(&self) -> (u32, u32) {
        (self.staged, self.skipped)
    }

    /// **`true` quando a camada TEM de ser rasterizada.**
    ///
    /// ⚠️ A ordem dos termos não é estética: o `has_slice` é a palavra do DONO dos pixels, e ela
    /// vence o nosso memo sempre — um memo que batesse sobre uma fatia despejada mandaria o
    /// compositor compor lixo.
    pub(super) fn needs_stage(&mut self, key: u64, fp: u64, compositor_has_slice: bool) -> bool {
        let fresh = compositor_has_slice && self.map.get(&key) == Some(&fp);
        if fresh {
            self.skipped += 1;
        } else {
            self.staged += 1;
        }
        !fresh
    }

    /// Registra que `key` foi rasterizada com a impressão `fp`.
    pub(super) fn record(&mut self, key: u64, fp: u64) {
        self.map.insert(key, fp);
    }
}

/// **A impressão digital de TUDO que a rasterização de uma camada consome.**
///
/// ⚠️ **Esquecer uma entrada aqui é o único bug grave desta wave**, e ele é silencioso: a camada
/// congela na tela mostrando o estado anterior. Por isso a câmera entra como o **RESULTADO**
/// (`CameraRaw` inteiro, POD de 96 bytes) e não como os ingredientes dela — a paralaxe multiplano, o
/// fold do `model` do objeto e o tint de fantasma já estão dobrados lá dentro, então uma porta nova
/// que mexa na câmera é coberta sem ninguém se lembrar desta função.
///
/// As entradas, e por que cada uma:
///
/// - **`geom`** — o hash de conteúdo do desenho (o mesmo do `TessCache`, que já o computa por frame);
/// - **`preview`** — o traço VIVO, que muda a cada movimento do mouse ⇒ a camada-alvo re-renderiza
///   todo frame, e é isso que a medição chama de *1 traço = 0,166 ms*. Parar de mexer volta a ser
///   grátis;
/// - **`cam`** — vide acima (câmera + zoom + paralaxe + tint);
/// - **`size`** — o alvo; um resize muda todo pixel;
/// - **`walk`** — QUAL motor produziu. Hoje é constante no processo (um `OnceLock`), e entra de
///   propósito: se algum dia o interruptor virar dinâmico, sem ele o frame seguinte reusaria pixels
///   do outro motor.
pub(super) fn fingerprint(
    geom: u64,
    preview: Option<&FlipGpuData>,
    cam: &CameraRaw,
    size: (u32, u32),
    walk: bool,
) -> u64 {
    let mut h = mix(0xcbf2_9ce4_8422_2325, geom);
    for b in bytemuck::bytes_of(cam) {
        h = mix(h, u64::from(*b));
    }
    h = mix(h, u64::from(size.0));
    h = mix(h, u64::from(size.1));
    h = mix(h, u64::from(walk));
    if let Some(pv) = preview {
        h = mix(h, preview_hash(pv));
    }
    h
}

/// O hash do traço em curso — os pontos (posição, largura, opacidade, cor) e a contagem de traços.
///
/// ⚠️ **`bytemuck` sobre os pontos, não campo a campo:** o `GpuPoint` é POD (é o que sobe para a
/// GPU), então varrer os bytes cobre um campo NOVO sem que alguém precise se lembrar daqui — a mesma
/// razão de a câmera entrar como resultado.
fn preview_hash(pv: &FlipGpuData) -> u64 {
    let mut h = mix(0x9e37_79b9_7f4a_7c15, pv.strokes.len() as u64);
    for b in bytemuck::cast_slice::<_, u8>(&pv.points) {
        h = mix(h, u64::from(*b));
    }
    h
}

/// FNV-1a de 64 bits — determinístico e transcendental-free (HR-5); é impressão digital, não
/// criptografia.
fn mix(h: u64, v: u64) -> u64 {
    (h ^ v).wrapping_mul(0x0000_0100_0000_01b3)
}

#[cfg(test)]
#[path = "flip_pass_stage_tests.rs"]
mod tests;
