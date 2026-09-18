//! **PARA ONDE O RAIO APONTA** — os dois modos de direcção do
//! [`crate::Verb::SceneProject`] (`SPEC_unblocked_brushes.md` §6.2).
//!
//! ⭐ **É UMA direcção para o dab INTEIRO, nunca uma por vértice**, e isso é uma
//! recusa medida por terceiros: a versão por normal do vértice foi construída e
//! rejeitada pelo autor do alvo como inutilizável (espec §9.2). ⛔ Não a
//! reconstrua sem um motivo novo.
//!
//! ⚠️ **Os dois modos são o OPOSTO de uma normal**, e a palavra é load-bearing:
//! o raio vai *para dentro* da peça, afastando-se de quem olha — é isso que faz
//! o barro andar até encostar no alvo em vez de fugir dele.

/// Contra que direcção o dab mede a distância — ver o cabeçalho.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ProjectMode {
    /// **O oposto da normal da VISTA** — para dentro do ecrã. É o de fábrica, e
    /// é o que o corpus do oráculo usa em `21` das `24` fixturas.
    #[default]
    View,
    /// **O oposto da normal do PLANO do pincel** — a normal da área sob o dab.
    ///
    /// ⚠️ Numa peça virada para quem olha os dois modos coincidem; eles separam-se
    /// numa parede lateral, onde a vista aponta para um lado e a superfície para
    /// outro.
    Plane,
}

impl ProjectMode {
    /// Todos, na ordem em que a UI os lista.
    pub const ALL: [Self; 2] = [Self::View, Self::Plane];

    /// O nome que a UI mostra.
    #[must_use]
    pub fn label(self) -> &'static str {
        ph2d_i18n::tr_em(ph2d_i18n::Idioma::Ingles, self.label_key())
    }

    /// ⭐⭐ **A CHAVE do rótulo** — `sculpt3d.project_mode.<variante>`, e é ela que a interface passa ao
    /// [`ph2d_i18n::tr`]. O texto vive na tabela (`ph2d-i18n/src/sculpt_engine.rs`), com a
    /// [`label`](Self::label) acima a lê-lo em inglês: *uma lei só, com um acessório derivado.*
    #[must_use]
    pub fn label_key(self) -> &'static str {
        match self {
            Self::View => "sculpt3d.project_mode.view",
            Self::Plane => "sculpt3d.project_mode.plane",
        }
    }

    /// **A DIREÇÃO DO DAB**, já com o sinal de «para dentro» aplicado.
    ///
    /// ⚠️ **Porta ÚNICA, com dois chamadores** (a lei e a bancada), e é essa a
    /// razão de ela existir separada do `match`: escrita duas vezes, o dia em
    /// que um modo novo entrasse deixaria a bancada a medir o modo antigo.
    ///
    /// Os dois argumentos chegam **já no espaço do objecto activo**, que é onde
    /// o dab escreve.
    #[must_use]
    pub fn direccao(self, olho: [f32; 3], normal_da_area: [f32; 3]) -> [f32; 3] {
        match self {
            // ⚠️ O `olho` é a direcção em que o raio de picagem VIAJA — ou seja
            // já aponta para dentro do ecrã. ⛔ Negá-lo aqui mandaria o barro
            // para fora da peça, que é o pincel ao contrário.
            Self::View => olho,
            Self::Plane => [-normal_da_area[0], -normal_da_area[1], -normal_da_area[2]],
        }
    }
}
