//! **9-Slice** — a seção 5 do Inspector do sprite (spec
//! [`03_inspector_secoes.md`](../../../docs/Sprite_projeto/03_inspector_secoes.md) §3.5),
//! declarada em 2026-05 e nunca construída: até 2026-08-21 `git grep -c SliceNine` dava **0**
//! em todo o repositório.
//!
//! Um sprite com 9-slice desenha-se como **até nove quads** em vez de um: os quatro cantos
//! mantêm o tamanho intrínseco, as quatro bordas esticam (ou repetem) num eixo só, e o centro
//! nos dois. É o que faz uma caixa de diálogo, um botão ou uma moldura de HUD crescerem sem
//! deformar o canto arredondado — o caso que todo motor 2D resolve e que este não resolvia.
//!
//! # Onde este componente NÃO manda
//!
//! ⚠️ Ele descreve **autoria**, nunca geometria cozida. Quem transforma isto em quads é
//! [`ph2d_render::nine_slice`], uma função **pura**, e quem os emite é o extract. Guardar aqui
//! o resultado seria guardar estado vivo de renderização num componente que o undo ordena por
//! bytes — a lei que o `ph2d-physics-ecs` já pagou.
//!
//! # Isolamento (ADR-0107)
//!
//! Módulo irmão, sem tocar em nada existente: um `SliceNine` **ausente** é o sprite de sempre,
//! byte-idêntico. É por isso que ele é um componente opcional e não campos novos em `Sprite` —
//! a esmagadora maioria dos sprites nunca terá 9-slice, e `Sprite` é `Copy` e viaja em toda
//! cena.

use bevy_ecs::component::Component;
use serde::{Deserialize, Serialize};

/// Como o sprite se desenha.
///
/// `Simple` é o default e é **exatamente o sprite de hoje** — a presença do componente em modo
/// `Simple` não muda um pixel. Isso é deliberado: permite anexar o componente, mexer nas bordas
/// e ver o efeito ligando o modo, sem que anexá-lo já altere a cena.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SliceDrawMode {
    /// Um quad, como sempre.
    #[default]
    Simple,
    /// Nove quads; bordas e centro **esticam**.
    Sliced,
    /// Nove quads; bordas e centro **repetem** em vez de esticar.
    Tiled,
}

impl SliceDrawMode {
    /// A ordem canônica dos segmentos na UI — e a tag que viaja no barramento é a **posição**
    /// nesta lista. Ver a lei em `inspector_regression_sections.rs`.
    pub const ALL: [SliceDrawMode; 3] = [Self::Simple, Self::Sliced, Self::Tiled];

    /// Tag estável para o barramento / persistência.
    pub const fn tag(self) -> u8 {
        match self {
            Self::Simple => 0,
            Self::Sliced => 1,
            Self::Tiled => 2,
        }
    }

    /// Inverso de [`Self::tag`]. Uma tag que o motor não produz cai no default — nunca entra em
    /// pânico, porque o valor pode vir de um ficheiro de projeto mais novo.
    pub const fn from_tag(tag: u8) -> Self {
        match tag {
            1 => Self::Sliced,
            2 => Self::Tiled,
            _ => Self::Simple,
        }
    }

    /// `true` quando o desenho é de nove quads. O extract usa isto como porta única.
    pub const fn is_nine(self) -> bool {
        matches!(self, Self::Sliced | Self::Tiled)
    }
}

/// O que uma das oito regiões à volta do centro faz quando o alvo é maior que a fonte.
///
/// ⚠️ **Por-região é o que o GameMaker tem de único** (Unity e Godot só têm global): dá para ter
/// cantos fixos, a lateral a repetir, o topo a esticar e o fundo escondido — o vocabulário de
/// uma barra de botões ou de uma caixa de diálogo de verdade.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileRegionMode {
    /// Estica a região para preencher (o default, e o que Unity/Godot fazem sempre).
    #[default]
    Stretch,
    /// Repete o pedaço da fonte tantas vezes quantas couberem.
    Repeat,
    /// Repete espelhando a cada repetição — costura invisível em texturas não-tileáveis.
    ///
    /// ⚠️ **Espelhar impõe duas coisas ao resto da grelha, e ambas são consequência, não gosto:**
    /// (1) o número de ladrilhos é sempre **inteiro** (um espelho cortado a meio não encosta em
    /// borda nenhuma — ver [`SliceTileMode`]); (2) com um número **PAR** de ladrilhos a faixa
    /// acaba numa cópia invertida, e a borda que combina com ela é a do **lado oposto,
    /// espelhada**. Quem executa as duas é `ph2d_render::nine_slice`.
    Mirror,
    /// Não desenha nada nesta região (o «hide» do GameMaker).
    Blank,
}

impl TileRegionMode {
    pub const ALL: [TileRegionMode; 4] = [Self::Stretch, Self::Repeat, Self::Mirror, Self::Blank];

    pub const fn tag(self) -> u8 {
        match self {
            Self::Stretch => 0,
            Self::Repeat => 1,
            Self::Mirror => 2,
            Self::Blank => 3,
        }
    }

    pub const fn from_tag(tag: u8) -> Self {
        match tag {
            1 => Self::Repeat,
            2 => Self::Mirror,
            3 => Self::Blank,
            _ => Self::Stretch,
        }
    }
}

/// As oito regiões à volta do centro, em ordem canônica de leitura (cima → baixo,
/// esquerda → direita), **saltando o centro** — que tem a sua própria lei (`fill_center`).
///
/// A ordem é a fonte: o índice numa `[T; 8]` É a região, e a UI pinta nesta ordem.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SliceRegion {
    TopLeft = 0,
    Top = 1,
    TopRight = 2,
    Left = 3,
    Right = 4,
    BottomLeft = 5,
    Bottom = 6,
    BottomRight = 7,
}

impl SliceRegion {
    pub const ALL: [SliceRegion; 8] = [
        Self::TopLeft,
        Self::Top,
        Self::TopRight,
        Self::Left,
        Self::Right,
        Self::BottomLeft,
        Self::Bottom,
        Self::BottomRight,
    ];

    /// Rótulo curto para a UI (inglês — HR-15 / UI em inglês).
    pub const fn label(self) -> &'static str {
        match self {
            Self::TopLeft => "TL",
            Self::Top => "T",
            Self::TopRight => "TR",
            Self::Left => "L",
            Self::Right => "R",
            Self::BottomLeft => "BL",
            Self::Bottom => "B",
            Self::BottomRight => "BR",
        }
    }

    /// A célula `(coluna, linha)` da grelha 3×3 que esta região ocupa.
    pub const fn cell(self) -> (usize, usize) {
        match self {
            Self::TopLeft => (0, 0),
            Self::Top => (1, 0),
            Self::TopRight => (2, 0),
            Self::Left => (0, 1),
            Self::Right => (2, 1),
            Self::BottomLeft => (0, 2),
            Self::Bottom => (1, 2),
            Self::BottomRight => (2, 2),
        }
    }
}

/// Como QUALQUER região que repita distribui as suas repetições.
///
/// ⚠️ Não é «a opção do `Tiled`»: uma célula posta em `Repeat`/`Mirror` repete também em
/// `Sliced` — `Tiled` só acrescenta que o `Stretch` passa a repetir. Foi por o ler como opção do
/// `Tiled` que o painel o escondia em `Sliced`.
///
/// # ⚠️ A lei que estes três modos existem para exprimir (smoke do Enio, 2026-08-22)
///
/// **Uma faixa ladrilhada só encontra a sua borda quando contém um número INTEIRO de
/// ladrilhos.** Um ladrilho cortado a meio acaba numa coluna qualquer da fonte, e a borda começa
/// na coluna que o autor recortou: as duas nunca coincidem, e o resultado é uma emenda vertical
/// (ou horizontal) rente ao canto — foi isso que as duas setas amarelas do smoke marcaram.
///
/// [`Self::Continuous`] aceita o corte de propósito (é o `TILE` do Godot); [`Self::Whole`] é a
/// cura (o `TILE_FIT`). São os dois modos que o Godot tem, e não por imitação: são os dois
/// resultados que existem.
///
/// # ⛔ O `Adaptive` do Unity foi RETIRADO — o mecanismo dele não pode funcionar aqui
///
/// Ele existiu por um dia (2026-08-21 → 22) com um slider `stretch_value`: um **limiar** que
/// decidia, pelo tamanho do resto, entre cortar e arredondar. O Enio reportou que «não funciona
/// perfeitamente», e a medição diz que ele tem razão e que não era afinação:
///
/// - o resultado é **binário** (há emenda ou não há), e para um dado tamanho de caixa o resto é
///   **fixo** — logo todo o curso do slider é morto menos **um ponto**, o do cruzamento;
/// - esse ponto é invisível: o painel não sabe quantos ladrilhos o sprite tem;
/// - misturar não salva o controlo — meio caminho entre cortado e inteiro continua **cortado**.
///
/// *Um botão de duas posições disfarçado de slider é a afordância a mentir* — e um controlo
/// contínuo sobre um resultado binário nunca vai «funcionar perfeitamente».
///
/// ⚠️ **A capacidade que se perdeu tem um dono futuro:** escolher automaticamente ao
/// **redimensionar** (o resto varre 0..1 e o limiar diz onde saltar) é real, e é invisível num
/// editor de sprites de tamanho autorado. Quando houver UI redimensionável em runtime, ela volta
/// como **caixa de seleção** — «arredonda quando sobra pouco» — que é o controlo honesto para um
/// resultado de duas posições. Não voltar como slider é o ponto.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SliceTileMode {
    /// Repete no tamanho intrínseco e deixa a última repetição cortada.
    #[default]
    Continuous,
    /// **Sempre um número inteiro de ladrilhos**, arredondando ao mais próximo: os ladrilhos
    /// esticam um pouco e a faixa encosta na borda sem emenda (o `TILE_FIT` do Godot).
    ///
    /// ⚠️ A distorção é no máximo de um meio-ladrilho repartido pelos que ficam — 33% num alvo
    /// que só cabia 1,5 ladrilho, e tanto menos quanto mais ladrilhos couberem.
    Whole,
}

impl SliceTileMode {
    pub const ALL: [SliceTileMode; 2] = [Self::Continuous, Self::Whole];

    pub const fn tag(self) -> u8 {
        match self {
            Self::Continuous => 0,
            Self::Whole => 1,
        }
    }

    pub const fn from_tag(tag: u8) -> Self {
        match tag {
            1 => Self::Whole,
            _ => Self::Continuous,
        }
    }
}

/// **A autoria de 9-slice de um sprite.** Componente opcional: ausente = sprite normal.
///
/// ⚠️ `borders` está em **pixels da FONTE**, não em metros — é a mesma unidade de
/// `Sprite::region_rect`, e pela mesma razão: o artista mede a borda na imagem, e ela não deve
/// mudar quando o sprite é escalado. A conversão para metros acontece no cozimento, via
/// `ProjectSettings::pixels_per_meter`.
#[derive(Component, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SliceNine {
    pub draw_mode: SliceDrawMode,
    /// `[left, top, right, bottom]` em pixels da fonte.
    pub borders: [f32; 4],
    /// Tamanho alvo em metros. `[0, 0]` = usa o `Sprite::size` (o caso comum de quem só quer
    /// as bordas fixas sem mudar o tamanho).
    pub size: [f32; 2],
    /// Modo por-região, indexado por [`SliceRegion`].
    pub tile_modes: [TileRegionMode; 8],
    /// **O modo do MIOLO** — a nona célula, que não é uma das oito.
    ///
    /// ⚠️ **Nasceu de um smoke (2026-08-22).** O miolo tinha o modo fixado em código, e ele é a
    /// maior área ladrilhada — exatamente onde a emenda entre dois ladrilhos aparece. Espelhar
    /// era inalcançável justamente onde mais serve. *Um modo que a maior área não alcança é um
    /// modo que não existe.*
    ///
    /// `Blank` **não se usa aqui**: quem apaga o miolo é [`Self::fill_center`], e duas portas
    /// para o mesmo estado divergem. A UI cicla só Stretch → Repeat → Mirror.
    #[serde(default)]
    pub centre_tile_mode: TileRegionMode,
    pub tile_mode: SliceTileMode,
    /// `false` deixa o miolo vazio — uma moldura oca.
    pub fill_center: bool,
}

impl SliceNine {
    /// O componente acabado de anexar: **visualmente inerte**.
    ///
    /// ⚠️ `Simple` + bordas a zero é o sprite de sempre, byte-idêntico. Anexar não pode mudar a
    /// cena: senão o botão «+ Add 9-Slice» é uma edição destrutiva disfarçada de acesso a uma
    /// seção.
    pub const INERT: Self = Self {
        draw_mode: SliceDrawMode::Simple,
        borders: [0.0; 4],
        size: [0.0, 0.0],
        tile_modes: [TileRegionMode::Stretch; 8],
        centre_tile_mode: TileRegionMode::Stretch,
        tile_mode: SliceTileMode::Continuous,
        fill_center: true,
    };

    /// O modo por-região desta região.
    pub fn region_mode(&self, region: SliceRegion) -> TileRegionMode {
        self.tile_modes[region as usize]
    }

    /// Saneia a autoria para o que o cozimento pode consumir: bordas e tamanho não-negativos e
    /// finitos.
    ///
    /// ⚠️ **Sanear na LEITURA e não em cada gesto**: a lei é «imponha o invariante na derivação,
    /// não em cada porta» — há mais portas de escrita (painel, undo, carregamento de ficheiro,
    /// animação futura) do que leitores, e uma delas esquece sempre.
    pub fn sanitized(&self) -> Self {
        let f = |v: f32| if v.is_finite() { v.max(0.0) } else { 0.0 };
        // ⚠️ **Um CANTO só tem dois estados: desenhar, ou não desenhar.** Ele nunca ladrilha, por
        // construção — o 9-slice repete no eixo que CRESCE, e num canto nenhum dos dois cresce
        // (ficar no tamanho intrínseco é a razão de existir dele). `Repeat` e `Mirror` num canto
        // produzem geometria byte-idêntica a `Stretch`: guardá-los seria guardar autoria que não
        // significa nada, e foi assim que a grelha do painel passou a ter três posições inertes
        // (auditoria 2026-08-22). Normalizar aqui — na derivação — cura também o que já estiver
        // gravado num ficheiro de projeto.
        let mut tile_modes = self.tile_modes;
        for r in SliceRegion::ALL {
            let (col, row) = r.cell();
            if col != 1 && row != 1 && tile_modes[r as usize] != TileRegionMode::Blank {
                tile_modes[r as usize] = TileRegionMode::Stretch;
            }
        }
        Self {
            draw_mode: self.draw_mode,
            borders: [
                f(self.borders[0]),
                f(self.borders[1]),
                f(self.borders[2]),
                f(self.borders[3]),
            ],
            size: [f(self.size[0]), f(self.size[1])],
            tile_modes,
            // `Blank` no miolo é o estado que `fill_center` já exprime; saneia para Stretch em
            // vez de deixar duas portas discordarem sobre a mesma coisa.
            centre_tile_mode: if self.centre_tile_mode == TileRegionMode::Blank {
                TileRegionMode::Stretch
            } else {
                self.centre_tile_mode
            },
            tile_mode: self.tile_mode,
            fill_center: self.fill_center,
        }
    }

    /// O tamanho alvo efetivo em metros: o autorado, ou o do sprite quando ele é `0`.
    pub fn effective_size(&self, sprite_size: [f32; 2]) -> [f32; 2] {
        [
            if self.size[0] > 0.0 {
                self.size[0]
            } else {
                sprite_size[0]
            },
            if self.size[1] > 0.0 {
                self.size[1]
            } else {
                sprite_size[1]
            },
        ]
    }
}

impl Default for SliceNine {
    fn default() -> Self {
        Self::INERT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A posição na lista É a tag — nos três enums com segmented na UI.
    #[test]
    fn position_in_all_is_the_tag() {
        for (i, m) in SliceDrawMode::ALL.iter().enumerate() {
            assert_eq!(m.tag() as usize, i, "SliceDrawMode {m:?}");
            assert_eq!(SliceDrawMode::from_tag(i as u8), *m);
        }
        for (i, m) in TileRegionMode::ALL.iter().enumerate() {
            assert_eq!(m.tag() as usize, i, "TileRegionMode {m:?}");
            assert_eq!(TileRegionMode::from_tag(i as u8), *m);
        }
        for (i, m) in SliceTileMode::ALL.iter().enumerate() {
            assert_eq!(m.tag() as usize, i, "SliceTileMode {m:?}");
            assert_eq!(SliceTileMode::from_tag(i as u8), *m);
        }
    }

    /// Uma tag que o motor não produz cai no default em vez de entrar em pânico: ela pode vir
    /// de um ficheiro de projeto gravado por uma versão mais nova.
    #[test]
    fn an_unknown_tag_falls_back_instead_of_panicking() {
        assert_eq!(SliceDrawMode::from_tag(200), SliceDrawMode::Simple);
        assert_eq!(TileRegionMode::from_tag(200), TileRegionMode::Stretch);
        assert_eq!(SliceTileMode::from_tag(200), SliceTileMode::Continuous);
    }

    /// ⚠️ O componente recém-anexado é INERTE — anexar não é editar.
    #[test]
    fn a_freshly_attached_component_draws_the_same_sprite() {
        let s = SliceNine::INERT;
        assert_eq!(s.draw_mode, SliceDrawMode::Simple);
        assert!(
            !s.draw_mode.is_nine(),
            "Simple nao pode entrar no caminho de nove quads"
        );
        assert_eq!(s.borders, [0.0; 4]);
        assert!(
            s.fill_center,
            "um miolo vazio por default seria uma mudanca visual"
        );
        assert_eq!(SliceNine::default(), SliceNine::INERT);
    }

    /// As oito regiões cobrem a grelha 3×3 menos o centro — sem repetir nenhuma célula.
    #[test]
    fn the_eight_regions_are_the_ring_around_the_centre() {
        let mut cells: Vec<(usize, usize)> = SliceRegion::ALL.iter().map(|r| r.cell()).collect();
        cells.sort_unstable();
        cells.dedup();
        assert_eq!(cells.len(), 8, "duas regioes reclamam a mesma celula");
        assert!(
            !cells.contains(&(1, 1)),
            "o centro nao e uma das oito: ele tem a sua propria lei (fill_center)"
        );
        for r in SliceRegion::ALL {
            let (c, l) = r.cell();
            assert!(c < 3 && l < 3, "{r:?} caiu fora da grelha 3x3");
        }
        // O índice na lista É o discriminante — o que permite indexar `tile_modes`.
        for (i, r) in SliceRegion::ALL.iter().enumerate() {
            assert_eq!(*r as usize, i, "{r:?} nao esta na sua propria posicao");
        }
    }

    /// Saneamento: NaN, infinito e negativo não podem chegar à geometria.
    #[test]
    fn sanitize_rejects_what_the_geometry_cannot_consume() {
        let dirty = SliceNine {
            borders: [-4.0, f32::NAN, f32::INFINITY, 8.0],
            size: [-1.0, f32::NAN],
            ..SliceNine::INERT
        };
        let c = dirty.sanitized();
        assert_eq!(c.borders, [0.0, 0.0, 0.0, 8.0]);
        assert_eq!(c.size, [0.0, 0.0]);
    }

    /// ⚠️ **Um CANTO só tem duas posições, e o saneamento é quem o impõe.** `Repeat`/`Mirror`
    /// num canto produzem geometria byte-idêntica a `Stretch` (ele nunca ladrilha, por
    /// construção), portanto guardá-los é guardar autoria sem significado — o que fez a grelha do
    /// painel oferecer três posições inertes por canto até 2026-08-22. ⚠️ O `Blank` **fica**:
    /// essa é a segunda posição real.
    #[test]
    fn a_corner_keeps_only_the_two_states_it_actually_has() {
        let mut dirty = SliceNine::INERT;
        for r in SliceRegion::ALL {
            dirty.tile_modes[r as usize] = TileRegionMode::Mirror;
        }
        let c = dirty.sanitized();
        for r in SliceRegion::ALL {
            let (col, row) = r.cell();
            let is_corner = col != 1 && row != 1;
            if is_corner {
                assert_eq!(
                    c.region_mode(r),
                    TileRegionMode::Stretch,
                    "{r:?} e' um canto e ficou com um modo que nao faz nada"
                );
            } else {
                assert_eq!(
                    c.region_mode(r),
                    TileRegionMode::Mirror,
                    "{r:?} e' uma borda — o saneamento comeu autoria com significado"
                );
            }
        }
        // Apagar um canto continua a ser possivel: e' a sua segunda posicao.
        let mut blanked = SliceNine::INERT;
        blanked.tile_modes[SliceRegion::TopLeft as usize] = TileRegionMode::Blank;
        assert_eq!(
            blanked.sanitized().region_mode(SliceRegion::TopLeft),
            TileRegionMode::Blank
        );
    }

    /// `size = 0` significa «usa o do sprite», por eixo independentemente.
    #[test]
    fn a_zero_size_axis_defers_to_the_sprite() {
        let s = SliceNine {
            size: [0.0, 3.0],
            ..SliceNine::INERT
        };
        assert_eq!(s.effective_size([2.0, 9.0]), [2.0, 3.0]);
    }
}
