//! Theme enum — **duas famílias, um interruptor de distância.**
//!
//! # A família CLÁSSICA (4 temas escritos à mão em `docs/design/tokens.json`)
//!
//! - `forge` — dark + magenta. Default (clássico).
//! - `workshop` — dark + cyan (Procreate-inspired, trademark-free name).
//! - `sunstone` — light + warm orange.
//! - `blueprint` — light + cool blue (sidebar layout).
//!
//! # A família MODERNA (4 presets DERIVADOS — 2026-09-04, decisão do Enio)
//!
//! > *«o que eu pedi do início foi um redesenho completo da UI de modo que se tornasse muito mais
//! > parecida com Blender/Godot […] minimalista, plana, concisa, coerente e simples»* — e, sobre o
//! > modelo: *«1 — aceito [o Godot 4.6 «Modern»] · 2 — [o cinza] do Godot · 3 — o azul do Godot ·
//! > 4 — decida [os presets]»*.
//!
//! Cada tema moderno é **cinco entradas** (cor base · acento · contraste · raio · espaçamento) e
//! **nenhum slot escrito à mão**: os ~83 tokens de cor saem das regras de derivação do tema
//! *Modern* do Godot 4.6 (MIT, [`crate::derive`]). Os quatro presets são os do próprio Godot
//! (`editor/themes/editor_theme_manager.cpp`, tabela `color_preset`), *não* cores inventadas:
//!
//! | preset | base | acento | contraste |
//! |---|---|---|---|
//! | `dark` (o **Default** do Godot 4.6) | `#292929` | `#569eff` | `0,30` |
//! | `gray` | `#3d3d3d` | `#70bafa` | `0,30` |
//! | `light` | `#e6e6e6` | `#2e80ff` | `−0,06` |
//! | `oled` (*Black (OLED)*) | `#000000` | `#73bfff` | `0,00` + bordas extra |
//!
//! ⚠️ **A decisão nº 4 («decida») caiu em QUATRO, um por slot do menu de tema, e os quatro vêm
//! da tabela do Godot** — o critério foi *nenhuma cor nova*: um preset que não estivesse lá seria
//! a primeira cor escolhida à mão do sistema novo, e é exactamente isso que o dono pediu para
//! deixar de existir.
//!
//! # ⛔ A família clássica NÃO se apaga
//!
//! Ela fica atrás de `PH2D_UI_NEW=0` ([`crate::UiLook::Classic`]), byte-idêntica — é a resposta a
//! *«isto já era assim antes?»* que sem ela exige um `git stash`
//! (`pesquisa/07 §22.1`). [`Theme::next`] cicla **dentro** da família, e o menu de tema mostra
//! **uma** família por aparência: misturá-las no mesmo menu poria um tema tingido ao lado de um
//! plano, e o artista não teria como saber que está a escolher entre dois sistemas.

/// Um tema — o **modo** de cor do app.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Theme {
    /// `forge` — dark + magenta. Default theme (clássico).
    #[default]
    Forge,
    /// `workshop` — dark + cyan (Procreate-inspired, trademark-free name).
    Workshop,
    /// `sunstone` — light + warm orange.
    Sunstone,
    /// `blueprint` — light + cool blue (sidebar layout).
    Blueprint,
    /// `dark` — o preset **Default** do Godot 4.6: `#292929` + azul `#569eff`. O default do
    /// redesenho.
    Dark,
    /// `gray` — o preset *Gray* do Godot: `#3d3d3d` + `#70bafa`.
    Gray,
    /// `light` — o preset *Light* do Godot: `#e6e6e6` + `#2e80ff`, contraste negativo (num tema
    /// claro a «elevação» escurece, e o Godot escreve-o assim de propósito).
    Light,
    /// `oled` — o preset *Black (OLED)* do Godot: preto puro + `#73bfff`, contraste `0` (num
    /// fundo preto não há para onde escurecer) e **bordas extra** para separar o que o contraste
    /// já não separa.
    Oled,
}

/// Panel layout flag — declared per-theme in tokens.json.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PanelLayout {
    /// Floating panels (Forge / Workshop / Sunstone).
    Floating,
    /// Panels docked in the sidebar (Blueprint, CAD style).
    Sidebar,
}

impl Theme {
    /// Todos os temas, na ordem do menu — a família clássica primeiro.
    pub const ALL: [Self; 8] = [
        Self::Forge,
        Self::Workshop,
        Self::Sunstone,
        Self::Blueprint,
        Self::Dark,
        Self::Gray,
        Self::Light,
        Self::Oled,
    ];

    /// A família clássica — os quatro escritos à mão no `tokens.json`.
    pub const CLASSIC: [Self; 4] = [Self::Forge, Self::Workshop, Self::Sunstone, Self::Blueprint];

    /// A família moderna — os quatro presets DERIVADOS pelas regras do Godot 4.6.
    pub const MODERN: [Self; 4] = [Self::Dark, Self::Gray, Self::Light, Self::Oled];

    /// **É um tema derivado** (moderno), e não uma tabela do `tokens.json`?
    ///
    /// É a única pergunta que separa as duas famílias no código: quem pinta não precisa de saber,
    /// e é por isso que ela está aqui e não em cada pintor.
    #[must_use]
    pub const fn is_modern(self) -> bool {
        matches!(self, Self::Dark | Self::Gray | Self::Light | Self::Oled)
    }

    /// A família a que este tema pertence, na ordem do menu.
    #[must_use]
    pub const fn family(self) -> &'static [Self] {
        if self.is_modern() {
            &Self::MODERN
        } else {
            &Self::CLASSIC
        }
    }

    /// **O tema com que o app ABRE, por aparência.** O clássico abre em `forge` (o `Default` do
    /// enum, que os gates antigos afirmam); o redesenho abre no `dark` — o *Default* do Godot.
    ///
    /// ⚠️ Existe para que `Theme::default()` continue a valer `Forge`: mudar o `Default` do enum
    /// mudaria o tema de todo fixture de teste do repo, e o que se quer é só o arranque do produto.
    #[must_use]
    pub const fn default_for(look: crate::UiLook) -> Self {
        match look {
            crate::UiLook::Classic => Self::Forge,
            crate::UiLook::Redesign => Self::Dark,
        }
    }

    /// Cycle through the themes of **this family**, in menu order.
    ///
    /// ⚠️ Dentro da família de propósito: a tecla `M` da shell cicla temas, e saltar de um tema
    /// plano para um tingido a meio do ciclo seria trocar de *sistema* sem o artista pedir.
    #[must_use]
    pub fn next(self) -> Self {
        let fam = self.family();
        let i = fam.iter().position(|t| *t == self).unwrap_or(0);
        fam[(i + 1) % fam.len()]
    }

    /// True when the theme is dark (low background luminance).
    #[must_use]
    pub const fn is_dark(self) -> bool {
        matches!(
            self,
            Self::Forge | Self::Workshop | Self::Dark | Self::Gray | Self::Oled
        )
    }

    /// Panel layout declared by the theme.
    ///
    /// ⚠️ Os modernos são `Floating` por omissão — a docagem no redesenho é do **modelo de áreas**
    /// (`screens/slot.rs`), não do tema; e `PanelLayout` não tem leitor de produção
    /// (`medicoes/03 §5`).
    #[must_use]
    pub const fn panel_layout(self) -> PanelLayout {
        match self {
            Self::Blueprint => PanelLayout::Sidebar,
            _ => PanelLayout::Floating,
        }
    }

    /// Stable identifier (matches `tokens.json` keys for the classic family; the modern family
    /// has no JSON entry — the id is the preset name, and it is what the DTCG export/import and
    /// the project file carry).
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Forge => "forge",
            Self::Workshop => "workshop",
            Self::Sunstone => "sunstone",
            Self::Blueprint => "blueprint",
            Self::Dark => "dark",
            Self::Gray => "gray",
            Self::Light => "light",
            Self::Oled => "oled",
        }
    }

    /// The theme with this [`Theme::id`], if any.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|t| t.id() == id)
    }

    /// Human-readable display name (used by the topbar theme
    /// cluster + the theme menu items).
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Forge => "Forge",
            Self::Workshop => "Workshop",
            Self::Sunstone => "Sunstone",
            Self::Blueprint => "Blueprint",
            Self::Dark => "Dark",
            Self::Gray => "Gray",
            Self::Light => "Light",
            Self::Oled => "Black (OLED)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_forge() {
        assert_eq!(Theme::default(), Theme::Forge);
    }

    /// `next` cicla a família inteira e volta ao princípio — nas DUAS famílias, e sem saltar de
    /// uma para a outra.
    #[test]
    fn next_cycles_each_family_and_never_crosses() {
        for fam in [&Theme::CLASSIC[..], &Theme::MODERN[..]] {
            let mut t = fam[0];
            let mut visited = vec![t];
            for _ in 0..fam.len() {
                t = t.next();
                assert!(fam.contains(&t), "{t:?} saiu da familia de {:?}", fam[0]);
                visited.push(t);
            }
            assert_eq!(visited[0], visited[fam.len()], "nao voltou ao principio");
            for want in fam {
                assert!(
                    visited[..fam.len()].contains(want),
                    "{want:?} nunca visitado"
                );
            }
        }
    }

    #[test]
    fn dark_themes_are_dark() {
        assert!(Theme::Forge.is_dark());
        assert!(Theme::Workshop.is_dark());
        assert!(!Theme::Sunstone.is_dark());
        assert!(!Theme::Blueprint.is_dark());
        assert!(Theme::Dark.is_dark());
        assert!(Theme::Gray.is_dark());
        assert!(!Theme::Light.is_dark());
        assert!(Theme::Oled.is_dark());
    }

    #[test]
    fn blueprint_uses_sidebar_layout() {
        assert_eq!(Theme::Blueprint.panel_layout(), PanelLayout::Sidebar);
        assert_eq!(Theme::Forge.panel_layout(), PanelLayout::Floating);
        assert_eq!(Theme::Dark.panel_layout(), PanelLayout::Floating);
    }

    #[test]
    fn ids_match_tokens_json() {
        assert_eq!(Theme::Forge.id(), "forge");
        assert_eq!(Theme::Workshop.id(), "workshop");
        assert_eq!(Theme::Sunstone.id(), "sunstone");
        assert_eq!(Theme::Blueprint.id(), "blueprint");
    }

    /// Os ids são únicos e `from_id` é o inverso exacto de `id` — é por eles que o ficheiro de
    /// projeto e o DTCG nomeiam um modo.
    #[test]
    fn ids_are_unique_and_round_trip() {
        for t in Theme::ALL {
            assert_eq!(Theme::from_id(t.id()), Some(t));
        }
        let mut ids: Vec<&str> = Theme::ALL.iter().map(|t| t.id()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), Theme::ALL.len(), "dois temas com o mesmo id");
        assert_eq!(Theme::from_id("nope"), None);
    }

    /// As duas famílias cobrem `ALL` sem sobreposição — e `is_modern` é o que as separa.
    #[test]
    fn the_two_families_partition_all() {
        let mut n = 0;
        for t in Theme::ALL {
            assert_eq!(t.is_modern(), Theme::MODERN.contains(&t));
            assert_eq!(!t.is_modern(), Theme::CLASSIC.contains(&t));
            n += 1;
        }
        assert_eq!(n, Theme::CLASSIC.len() + Theme::MODERN.len());
    }

    /// O redesenho abre no *Default* do Godot; o clássico continua a abrir onde sempre abriu.
    #[test]
    fn each_look_opens_in_its_own_family() {
        assert_eq!(Theme::default_for(crate::UiLook::Classic), Theme::Forge);
        assert_eq!(Theme::default_for(crate::UiLook::Redesign), Theme::Dark);
        assert!(Theme::default_for(crate::UiLook::Redesign).is_modern());
    }
}
