//! **Os knobs do MUNDO** — o que se AJUSTA nele, ao lado do que ele FAZ.
//!
//! Extraído de `world.rs` quando ele passou do cap de 700 LOC, e o corte é por
//! responsabilidade e não por tamanho: tudo aqui é *"o artista (ou o painel de
//! física) virou este número"* — gravidade, sub-passos, iterações, resposta de
//! contato, defaults de corpo, a matriz de camadas. Nada aqui roda no `step`.
//!
//! ⚠️ Nenhum tipo do rapier atravessa esta fronteira: os knobs entram como
//! números simples e o `IntegrationParameters` fica dentro da crate (a mesma
//! regra que o `lib.rs` enuncia para a superfície pública inteira).

use crate::rmath::Vector;

use super::PhysicsWorld;
use super::defaults::BodyDefaults;
use super::groups_for;
use super::layers::LayerMatrix;

impl PhysicsWorld {
    /// Override gravity. Useful for top-down 2D (set to zero) or
    /// custom worlds.
    pub fn set_gravity(&mut self, x: f32, y: f32) {
        self.gravity = Vector::new(x, y);
    }

    /// Contact response tuning, as plain numbers (rapier's
    /// `IntegrationParameters` stays inside this crate).
    ///
    /// - `damping_ratio` — ⚠️ **`5.0` era o default da rapier ATÉ à 0.34; a 0.35 dobrou-o para
    ///   `10.0` e nós ficámos no `5.0` por MEDIÇÃO** (a tabela está no `world.rs`, ao lado da
    ///   constante: com o nosso `f = 120 Hz`, o `ζ = 10` deles dá `24°` de inclinação em repouso).
    ///   Os docs deles nomeiam este como
    ///   the knob to reach for when the simulation should *look stiffer*,
    ///   in preference to raising the contact natural frequency (which
    ///   overshoots and jitters).
    /// - `max_corrective_velocity` — ⚠️ **era `10.0` na 0.28; a 0.35 afina para `3.0`** e passou
    ///   a grampear **também o viés das juntas**, que na 0.28 era exclusivo de contatos. O ceiling
    ///   on how fast the solver is allowed to push accumulated penetration
    ///   back out.
    ///
    /// ⚠️ These feed the solver, so changing them **changes every
    /// simulation** — including the cross-OS C9 hash. That is a deliberate
    /// product decision, not a free knob.
    pub fn set_contact_response(&mut self, damping_ratio: f32, max_corrective_velocity: f32) {
        // ⚠️ A rapier 0.31 juntou os dois coeficientes do contacto num `SpringCoefficients`
        // (`natural_frequency` + `damping_ratio`), que é o que eles sempre foram: os dois
        // parâmetros de UMA mola. Dois campos soltos deixavam escrever um e esquecer o outro.
        self.write_contact_springs(|s| s.damping_ratio = damping_ratio);
        self.integration_parameters
            .normalized_max_corrective_velocity = max_corrective_velocity;
    }

    /// **A PORTA das duas molas do contacto.** Toda escrita a uma rigidez de contacto passa aqui.
    ///
    /// ⛔⛔ A `rapier2d` 0.35 partiu a rigidez em **duas** — `contact_softness`
    /// (dinâmico↔dinâmico) e `static_contact_softness` (corpo↔cenário **fixo**) — e a subida de
    /// 2026-08-29 honrou-o **onde o mundo nasce** (o construtor) e esqueceu-o **onde ele muda**
    /// (estes setters). Resultado alcançável por um gesto: o artista arrastava *Contact Hz* e o
    /// chão ficava rígido para sempre, com metade do mundo a obedecer.
    ///
    /// ⚠️ **Escrever a linha duas vezes não era a cura** — seria a terceira cópia da mesma lei, e a
    /// próxima metade a nascer (a `rapier` já tem `joint_softness`) divergiria outra vez. *Uma lei
    /// escrita em dois sítios ainda não é uma lei; só uma PORTA é.*
    ///
    /// ⚠️ **As duas recebem o MESMO valor, e isso é escolha nossa:** a `rapier` faz o cenário fixo
    /// mais rígido que o dinâmico (`60` contra `30` Hz), mas o nosso construtor sempre pôs os dois
    /// no mesmo `DEFAULT_CONTACT_HZ`, e um slider que promete uma rigidez tem de a entregar ao
    /// mundo inteiro. Gate: `both_contact_springs_follow_the_frequency_knob`.
    fn write_contact_springs(
        &mut self,
        mut f: impl FnMut(&mut rapier2d::dynamics::SpringCoefficients<crate::rmath::Real>),
    ) {
        f(&mut self.integration_parameters.contact_softness);
        f(&mut self.integration_parameters.static_contact_softness);
    }

    /// Contact spring frequency, Hz. ⚠️ **O default da rapier é `30` (dinâmico) e `60` (contra
    /// cenário fixo) na 0.35** — eram `30` para tudo na 0.28. O nosso é `DEFAULT_CONTACT_HZ` nos
    /// dois, e o slider escreve os dois (ver [`PhysicsWorld::write_contact_springs`]). rapier's docs:
    /// *"increasing this value will make it so that penetrations get fixed
    /// more quickly at the expense of potential jitter due to overshooting"*.
    pub fn set_contact_frequency(&mut self, hz: f32) {
        // ⚠️ Pela PORTA — ver [`PhysicsWorld::write_contact_springs`].
        self.write_contact_springs(|s| s.natural_frequency = hz);
    }

    /// How many integration sub-steps one [`PhysicsWorld::step`] runs.
    ///
    /// The **only** lever on how deep a fast body is already overlapping the
    /// frame it first touches: that depth is `velocity × dt` and no solver
    /// can undo it after the fact. Halving the sub-step halves the overlap,
    /// at a proportional cost.
    pub fn set_substeps(&mut self, n: u32) {
        self.substeps = n.max(1);
        self.integration_parameters.dt = self.base_dt / self.substeps as f32;
    }

    /// Number of solver iterations per step (rapier default `4`). More
    /// iterations resolve a stack's contacts more completely, at linear cost.
    ///
    /// ⚠️ **A rapier 0.31 tirou o `NonZeroUsize` daqui** — o campo é `usize` puro. A guarda
    /// contra o zero deixou de ser do TIPO e passa a ser nossa: um `0` faria o solver não
    /// resolver contacto nenhum, e é isso que o `if n > 0` impede. *Quando a rede muda de
    /// dono, ela tem de continuar a existir.*
    pub fn set_solver_iterations(&mut self, n: usize) {
        if n > 0 {
            self.integration_parameters.num_solver_iterations = n;
        }
    }

    /// The world-level values new bodies are born with (damping, sleep).
    pub fn body_defaults(&self) -> BodyDefaults {
        self.body_defaults
    }

    /// Replace the world-level body defaults.
    ///
    /// **Applies to the bodies that already exist, not only to future ones.**
    /// The artist is describing the world in front of them; a drag value that
    /// only reached the next body spawned would be a number that appears to do
    /// nothing. See [`BodyDefaults`] for why these are world settings at all.
    pub fn set_body_defaults(&mut self, d: BodyDefaults) {
        self.body_defaults = d;
        d.apply_to_all(&mut self.bodies);
    }

    /// Which layers collide with which.
    pub fn layer_matrix(&self) -> LayerMatrix {
        self.layer_matrix
    }

    /// Replace the collision-layer matrix.
    ///
    /// **Applies to the colliders that already exist**, for the same reason
    /// [`Self::set_body_defaults`] does: the artist is describing the scene in
    /// front of them, and a rule that only reached the next body spawned would
    /// look like a dead checkbox.
    ///
    /// A collider already carries its own layer — it is `memberships`, a single
    /// bit — so the layer never has to be stored twice or looked up elsewhere.
    pub fn set_layer_matrix(&mut self, matrix: LayerMatrix) {
        self.layer_matrix = matrix;
        for (_, collider) in self.colliders.iter_mut() {
            let membership_bits = collider.collision_groups().memberships.bits();
            let layer = membership_bits.trailing_zeros() as usize;
            collider.set_collision_groups(groups_for(layer, matrix));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::PhysicsWorld;

    /// ⚠️ Valores escolhidos **longe** dos de construção: se um calhasse igual ao default, o
    /// `assert` passaria sem provar nada. Os `assert_ne!` de abertura são essa metade.
    const HZ: f32 = 37.0;
    const ZETA: f32 = 3.25;
    const MAX_CORRECTIVE: f32 = 7.5;

    /// **O contacto tem DUAS molas desde a `rapier2d` 0.35, e os knobs têm de escrever as duas.**
    ///
    /// ⛔⛔ A 0.35 partiu a rigidez do contacto em `contact_softness` (dinâmico↔dinâmico) e
    /// `static_contact_softness` (corpo↔cenário **fixo**). A subida de 2026-08-29 fixou as duas no
    /// construtor e deixou **estes dois setters** a escrever só a metade dinâmica.
    ///
    /// ⇒ com o slider *Contact Hz* em qualquer valor ≠ o de construção, o artista via
    /// caixa-contra-caixa amolecer enquanto **o chão continuava rígido**, e um `.ph2dproj` gravado
    /// carregava um mundo internamente incoerente sem nada na tela a explicá-lo.
    ///
    /// ⚠️ **A lei já estava escrita, no construtor:** *«uma lei escrita num sítio quando o modelo
    /// tem dois não é uma lei»*. Foi honrada onde o mundo NASCE e esquecida onde ele MUDA — e
    /// escrevê-la sem a gatear é o defeito que este repo já registou três vezes.
    ///
    /// ⭐ De leitura pura, sem simular um passo: o defeito não precisa de física para existir.
    #[test]
    fn both_contact_springs_follow_the_frequency_knob() {
        let mut w = PhysicsWorld::new();
        let antes = w
            .integration_parameters
            .static_contact_softness
            .natural_frequency;
        assert_ne!(antes, HZ, "a fixtura tem de CONTER o fenomeno");
        w.set_contact_frequency(HZ);
        let p = &w.integration_parameters;
        assert_eq!(p.contact_softness.natural_frequency, HZ);
        assert_eq!(
            p.static_contact_softness.natural_frequency, HZ,
            "⛔ o CENARIO FIXO ficou em {} Hz enquanto corpo-contra-corpo foi para {HZ}: o artista \
             arrasta um slider e METADE do mundo obedece",
            p.static_contact_softness.natural_frequency
        );
    }

    /// ⛔ **Ninguém escreve UMA das duas molas fora da porta.**
    ///
    /// Os dois gates acima provam os dois setters que existem **hoje**. Este apanha o terceiro —
    /// o que ainda não foi escrito. ⚠️ *Foi exactamente assim que o defeito nasceu: a lei estava
    /// escrita no construtor, e o setter seguinte não a leu.*
    ///
    /// Os dois sítios legítimos são o **construtor** (`world.rs`, onde as duas nascem lado a lado,
    /// visíveis uma à outra) e a **porta** (`write_contact_springs`). Qualquer outro é a divergência
    /// a começar de novo.
    #[test]
    fn nobody_writes_one_contact_spring_without_the_other() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut vistos = 0usize;
        let mut infractores = Vec::new();
        let mut pilha = vec![dir];
        while let Some(d) = pilha.pop() {
            for e in std::fs::read_dir(&d).expect("ler src") {
                let path = e.expect("entrada").path();
                if path.is_dir() {
                    pilha.push(path);
                    continue;
                }
                if path.extension().is_none_or(|x| x != "rs") {
                    continue;
                }
                let nome = path
                    .file_name()
                    .expect("nome")
                    .to_string_lossy()
                    .to_string();
                let texto = std::fs::read_to_string(&path).expect("ler");
                for (i, linha) in texto.lines().enumerate() {
                    // ⚠️ **Qualquer MENÇÃO em código, não só `= …`.** A 1.ª versão exigia um `=` e
                    // o controle apanhou-a de imediato: o construtor escreve por CAMPO de struct
                    // (`contact_softness: …`) e a porta por `&mut` — nenhum dos quatro sítios reais
                    // tem `=`. *Um censo cuja regra não casa com a forma do código conta zero e
                    // passa sempre.*
                    let corpo = linha.trim_start();
                    if corpo.starts_with("//") || !corpo.contains("contact_softness") {
                        continue;
                    }
                    vistos += 1;
                    // Dois sítios legítimos, e a razão é a mesma nos dois: **as duas molas têm de
                    // ser visíveis uma à outra na linha em que são escritas.**
                    //   · `solver_params.rs` — o CONSTRUTOR, onde nascem lado a lado;
                    //   · `tuning.rs`        — a PORTA, que escreve as duas por construção.
                    // ⚠️ Este gate apanhou a própria mudança que pôs o construtor aqui (ele vivia
                    // no `world.rs` até ao corte de 700 LOC do mesmo dia) — que é o comportamento
                    // certo: mover a construção é exactamente quando a lei se pode perder.
                    if nome != "solver_params.rs" && nome != "tuning.rs" {
                        infractores.push(format!("{nome}:{}", i + 1));
                    }
                }
            }
        }
        assert!(
            vistos >= 4,
            "a sonda tem de VER as escritas que ja' existem (construtor x2 + porta x2); viu {vistos}.              Um censo que casa zero linhas passa sempre e nao diz nada."
        );
        assert!(
            infractores.is_empty(),
            "estes sitios escrevem uma rigidez de contacto fora da PORTA `write_contact_springs`:              {infractores:?}. A rapier 0.35 tem DUAS molas (dinamica e contra cenario fixo) e uma              lei escrita num sitio quando o modelo tem dois nao e' uma lei."
        );
    }

    /// A metade da resposta — ver [`both_contact_springs_follow_the_frequency_knob`].
    #[test]
    fn both_contact_springs_follow_the_response_knob() {
        let mut w = PhysicsWorld::new();
        let antes = w
            .integration_parameters
            .static_contact_softness
            .damping_ratio;
        assert_ne!(antes, ZETA, "a fixtura tem de CONTER o fenomeno");
        w.set_contact_response(ZETA, MAX_CORRECTIVE);
        let p = &w.integration_parameters;
        assert_eq!(p.contact_softness.damping_ratio, ZETA);
        assert_eq!(
            p.static_contact_softness.damping_ratio, ZETA,
            "⛔ o CENARIO FIXO ficou em {} enquanto corpo-contra-corpo foi para {ZETA}",
            p.static_contact_softness.damping_ratio
        );
    }
}
