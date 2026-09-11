//! **A declaração que uma família de módulo faz à shell** (W2, ADR-0075).
//!
//! ⚠️ **Estes tipos vivem na INTERFACE e não no agregador, e a razão é um ciclo:** o
//! `ph2d-app-registry-init` depende de cada `ph2d-app-<fam>` (é o que ele agrega), logo uma família
//! que importasse `AppFamily` de lá fecharia `registry-init → família → registry-init`. Uma crate
//! de interface é precisamente o vértice comum que quebra isso.

/// **Um roteador de cenas de smoke** — a variável de ambiente e a faixa de níveis que ela aceita.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SmokeRouter {
    /// `PH2D_FIELD_SMOKE`, `PH2D_PHYSICS_SMOKE`, …
    pub env: &'static str,
    /// O maior nível que o `match` do roteador de facto responde.
    ///
    /// ⚠️ **Este número CONTA-SE lendo o roteador, nunca se escreve de memória** (CLAUDE.md §5.0) —
    /// e o gate `no_two_*_scenes_claim_the_same_level` da família mede o **piso**, não o tecto: uma
    /// cena acima do tecto é simplesmente **muda**.
    pub max_level: u32,
}

/// **O que uma família declara à shell.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppFamily {
    /// A chave da família (`"field3d"`, `"motion"`, …) — o sufixo da crate `ph2d-app-<chave>`.
    pub key: &'static str,
    /// Os roteadores de smoke que ela possui.
    pub routers: &'static [SmokeRouter],
}

/// O registo das famílias compiladas nesta build.
#[derive(Debug, Default)]
pub struct AppFamilyRegistry {
    families: Vec<AppFamily>,
}

impl AppFamilyRegistry {
    #[must_use]
    pub fn new_empty() -> Self {
        Self::default()
    }

    pub fn push(&mut self, f: AppFamily) {
        self.families.push(f);
    }

    #[must_use]
    pub fn families(&self) -> &[AppFamily] {
        &self.families
    }

    /// Todos os roteadores de todas as famílias, achatados.
    pub fn routers(&self) -> impl Iterator<Item = (&'static str, SmokeRouter)> + '_ {
        self.families
            .iter()
            .flat_map(|f| f.routers.iter().map(move |r| (f.key, *r)))
    }
}
