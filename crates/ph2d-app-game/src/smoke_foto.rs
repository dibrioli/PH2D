//! **As fotos do jogo** (`PH2D_GAME_SMOKE_FOTO`, planos GEF 12–15): cada uma é um instante pedido pelos MESMOS
//! gestos da mão, resolvidos no mundo para o shell os executar. Irmão de [`super`] pelo tecto de LOC.

use super::*;

/// O instante que a foto quer (`PH2D_GAME_SMOKE_FOTO`). A mão do dono faz o mesmo pelo Inspector e pelos Assets;
/// a tela virtual não recebe cliques. Plano 12: o seletor aberto sobre os montes, ou as duas imagens trocadas.
/// Plano 13: o bloco Platform High (sem quadros), com a bandeira (o aviso), já animada, e a «Platform» largada
/// no canvas (sobre o céu). Plano 14: a Platform Low em Tiled, e com Width 4. Plano 15 (o seletor «Use»): o
/// Player animado, a bandeira num bloco passada a parada, e o «heroi_azul» num bloco passado a animado (a grelha
/// proposta pela transparência).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Foto {
    Seletor,
    Trocado,
    Bloco,
    Bandeira,
    Animada,
    Largada,
    Baixa,
    Larga,
    UsoAnimada,
    UsoParada,
    PlayerParado,
    RepetidaDaParada,
    Voltas,
    HeroiAnimado,
    PlayerRepetido,
    Memoria,
    MemoriaRepetida,
}

#[must_use]
pub fn foto() -> Option<Foto> {
    match std::env::var("PH2D_GAME_SMOKE_FOTO").ok()?.trim() {
        "seletor" => Some(Foto::Seletor),
        "trocado" => Some(Foto::Trocado),
        "bloco" => Some(Foto::Bloco),
        "bandeira" => Some(Foto::Bandeira),
        "animada" => Some(Foto::Animada),
        "largada" => Some(Foto::Largada),
        "baixa" => Some(Foto::Baixa),
        "larga" => Some(Foto::Larga),
        "uso_animada" => Some(Foto::UsoAnimada),
        "uso_parada" => Some(Foto::UsoParada),
        "player_parado" => Some(Foto::PlayerParado),
        "repetida_da_parada" => Some(Foto::RepetidaDaParada),
        "voltas" => Some(Foto::Voltas),
        "heroi_animado" => Some(Foto::HeroiAnimado),
        "player_repetido" => Some(Foto::PlayerRepetido),
        "memoria" => Some(Foto::Memoria),
        "memoria_repetida" => Some(Foto::MemoriaRepetida),
        _ => None,
    }
}

/// Onde a foto «largada» põe a «Platform»: entre as duas plataformas do nível 2, à altura da alta.
pub const ONDE_LARGA: [f32; 2] = [6.0, 3.0];

/// O centro da vista nas fotos do plano 13: a plataforma alta e o sítio da queda (m).
pub const CENTRO_DAS_FOTOS_13: [f32; 2] = [7.0, 2.0];

impl Foto {
    /// Uma foto dos planos 13/14 (o bloco à vista e a secção Texture & Atlas aberta).
    #[must_use]
    pub const fn do_plano_13(self) -> bool {
        !matches!(self, Self::Seletor | Self::Trocado)
    }

    /// Uma foto do plano 15: o seletor «Use» à vista, as secções avançadas fechadas como o artista as encontra.
    #[must_use]
    pub const fn do_plano_15(self) -> bool {
        matches!(
            self,
            Self::UsoAnimada
                | Self::UsoParada
                | Self::PlayerParado
                | Self::RepetidaDaParada
                | Self::Voltas
                | Self::HeroiAnimado
                | Self::PlayerRepetido
                | Self::Memoria
                | Self::MemoriaRepetida
        )
    }
}

/// O que a foto pede, já resolvido no mundo — os gestos que o shell executa pelas MESMAS portas da mão.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanoDaFoto {
    pub foto: Foto,
    /// `(sprite, imagem)` a escolher (a linha Image).
    pub escolhas: Vec<(u64, [u8; 32])>,
    /// As edições do uso da imagem (Make it Animated, Tiled, Width/Height), pela ordem.
    pub edicoes: Vec<(u64, ph2d_editor_core::ImageUseEdit)>,
    /// A imagem a largar no canvas, onde, e o objecto por baixo da mão (a queda dos Assets, pela lei da queda).
    pub larga: Option<([u8; 32], [f32; 2], Option<u64>)>,
    /// Quem fica seleccionado (ou com o seletor aberto).
    pub alvo: u64,
}

/// `ids` são as imagens da cena pela ordem de [`IMAGENS_4`]; `fabrica` dá a imagem de fábrica de uma chave.
#[must_use]
pub fn plano_da_foto(
    sim: &mut SimWorld,
    ids: &[[u8; 32]],
    fabrica: &dyn Fn(u32) -> Option<[u8; 32]>,
) -> Option<PlanoDaFoto> {
    let foto = foto()?;
    let [heroi, montes] = ids else {
        return None;
    };
    let hills = por_nome(sim, CAMADAS_2[1].0)?.to_bits();
    let player = por_nome(sim, "Player")?.to_bits();
    let alta = por_nome(sim, "Platform High")?.to_bits();
    let baixa = por_nome(sim, "Platform Low")?.to_bits();
    let ceu = por_nome(sim, "Sky").map(Entity::to_bits);
    let bandeira = fabrica(ph2d_render::FLAG_SHEET_KEY)?;
    let mut plano = PlanoDaFoto {
        foto,
        escolhas: Vec::new(),
        edicoes: Vec::new(),
        larga: None,
        alvo: alta,
    };
    match foto {
        Foto::Seletor => plano.alvo = hills,
        Foto::Trocado => {
            plano.escolhas = vec![(hills, *montes), (player, *heroi)];
            plano.alvo = hills;
        }
        Foto::Bloco => {}
        Foto::Bandeira => plano.escolhas = vec![(alta, bandeira)],
        Foto::Animada => {
            plano.escolhas = vec![(alta, bandeira)];
            plano.edicoes = vec![(alta, uso(ph2d_editor_core::InspectorImageUse::Animated))];
        }
        Foto::Largada => {
            plano.larga = Some((fabrica(ph2d_render::PLATFORM_TILE_KEY)?, ONDE_LARGA, ceu));
        }
        Foto::Baixa => plano.alvo = baixa,
        Foto::Larga => {
            plano.edicoes = vec![(baixa, ph2d_editor_core::ImageUseEdit::Width(4.0))];
            plano.alvo = baixa;
        }
        Foto::UsoAnimada => plano.alvo = player,
        Foto::PlayerParado => {
            plano.edicoes = vec![(player, uso(ph2d_editor_core::InspectorImageUse::Still))];
            plano.alvo = player;
        }
        Foto::UsoParada => {
            plano.escolhas = vec![(alta, bandeira)];
            plano.edicoes = vec![(alta, uso(ph2d_editor_core::InspectorImageUse::Still))];
        }
        Foto::RepetidaDaParada => {
            plano.escolhas = vec![(alta, bandeira)];
            plano.edicoes = vec![
                (alta, uso(ph2d_editor_core::InspectorImageUse::Still)),
                (alta, uso(ph2d_editor_core::InspectorImageUse::Repeated)),
            ];
        }
        Foto::Voltas => {
            plano.escolhas = vec![(alta, bandeira)];
            plano.edicoes = [
                ph2d_editor_core::InspectorImageUse::Still,
                ph2d_editor_core::InspectorImageUse::Repeated,
                ph2d_editor_core::InspectorImageUse::Animated,
            ]
            .into_iter()
            .map(|u| (player, uso(u)))
            .chain(
                [
                    ph2d_editor_core::InspectorImageUse::Still,
                    ph2d_editor_core::InspectorImageUse::Repeated,
                    ph2d_editor_core::InspectorImageUse::Animated,
                    ph2d_editor_core::InspectorImageUse::Still,
                ]
                .into_iter()
                .map(|u| (alta, uso(u))),
            )
            .collect();
            plano.alvo = player;
        }
        Foto::HeroiAnimado => {
            plano.escolhas = vec![(alta, *heroi)];
            plano.edicoes = vec![(alta, uso(ph2d_editor_core::InspectorImageUse::Animated))];
        }
        // Cada uso guarda a sua configuração (dono 27/10): a repetida nasce com UM ladrilho; a largura que o
        // artista lhe dá fica com ela; voltar a animada devolve as animações.
        Foto::PlayerRepetido | Foto::Memoria | Foto::MemoriaRepetida => {
            use ph2d_editor_core::InspectorImageUse::{Animated, Repeated};
            plano.edicoes = vec![(player, uso(Repeated))];
            if foto != Foto::PlayerRepetido {
                plano.edicoes.extend([
                    (player, ph2d_editor_core::ImageUseEdit::Width(3.0)),
                    (player, uso(Animated)),
                ]);
            }
            if foto == Foto::MemoriaRepetida {
                plano.edicoes.push((player, uso(Repeated)));
            }
            plano.alvo = player;
        }
    }
    Some(plano)
}

fn uso(u: ph2d_editor_core::InspectorImageUse) -> ph2d_editor_core::ImageUseEdit {
    ph2d_editor_core::ImageUseEdit::Use(u)
}
