//! ⭐ **OS NÚMEROS CALIBRADOS da lei VBD do tecido** — a região, o passo, o
//! orçamento e a rigidez da mão.
//!
//! ⚠️ **Cada um traz a CALIBRAÇÃO ao lado**, que é o que os torna auditáveis: um
//! número sem a medição que o produziu é um palpite à espera de um smoke
//! (CLAUDE.md §0.0). Eles são consumidos por [`super::stroke_cloth`], e a lei
//! que os lê é alcançável por `PH2D_CLOTH_LAW=vbd` desde 2026-09-06 — o caminho
//! de omissão é a lei da referência.
//!
//! Irmão do [`super::stroke_cloth`], e o corte é *os NÚMEROS* (aqui) contra *a
//! tradução malha ⇄ solver* (lá).

/// **Quantos raios de pincel a região que simula tem.**
///
/// ⚠️ **Ela é MAIOR que a pegada de propósito**, e é isso que dá ao pano onde
/// responder: uma prega nasce porque o tecido em volta do dedo é puxado junto.
/// Com região = pegada, o que está fora da pegada estaria pregado, e o gesto
/// viraria um Grab com bordas duras.
pub const CLOTH_SIM_LIMIT: f32 = 2.0;

/// **Quanto o pincel pode andar antes de a região o seguir**, em fração do raio
/// dela.
///
/// ⚠️ Ele não é um teto: é a distância a partir da qual reconstruir sai mais
/// barato que empurrar de longe. Perto de `0` a região é refeita a cada dab
/// (caro, e o repouso re-medido a cada passo apaga a memória do gesto); perto de
/// `1` o pincel chega ao anel pregado antes de a região o seguir, que é o arco
/// escuro do report.
pub const CLOTH_FOLLOW: f32 = 0.25;

/// **A RIGIDEZ DA MÃO**, na mesma unidade do módulo do pano.
///
/// ⚠️ **Ela é o que o `Strength` do pincel multiplica**, e a calibração tem
/// critério: com o material de fábrica, um traço a arrastar o pincel por três
/// raios move a superfície `~16 %` do raio do pincel — visível, e com a malha a
/// esticar menos de `10 %`, que é a propriedade publicada de um tecido.
pub const CLOTH_GRIP: f64 = 600.0;

/// Sub-passos por evento de ponteiro.
///
/// ⚠️ **O orçamento é gasto em SUB-PASSOS e não em iterações**, que é o achado do
/// *Small Steps* (Macklin et al. 2019): `n` sub-passos de uma iteração batem um
/// passo de `n` iterações. O VBD é estável nos dois.
pub const CLOTH_SUBSTEPS: u32 = 4;

/// Iterações de VBD por sub-passo.
pub const CLOTH_ITERATIONS: u32 = 1;

/// O relógio de um evento de ponteiro.
///
/// ⚠️ **FIXO, e não o relógio de parede.** Um passo derivado do tempo real
/// tornaria o resultado função da taxa de quadros — a mesma pincelada daria
/// pregas diferentes num dia de máquina carregada, e o replay desta casa não o
/// reproduziria. *O tecido responde ao GESTO, não ao relógio.*
pub const CLOTH_DT: f64 = 1.0 / 60.0;
