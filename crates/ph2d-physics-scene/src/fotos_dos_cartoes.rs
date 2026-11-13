//! **As fotos dos cartões de física** (`PH2D_GAME_SMOKE=4`, `PH2D_GAME_SMOKE_FOTO=fisica_*`, ficha f1): um objecto por
//! face do cartão, sobre o nível do jogo. Folha: `ph2d-app-game` (pela catraca da crate) só as pede.

use ph2d_core::Vec2;
use ph2d_ecs::{
    ChildOf, Entity, Name, SimWorld, Transform, assign_missing_root_order,
    assign_missing_stable_ids,
};
use ph2d_physics_ecs::{BodyKind, Collider, ColliderShape, LayerNames, RigidBody};
use ph2d_render::{Sprite, WHITE_TILE_KEY};

fn por_nome(sim: &mut SimWorld, nome: &str) -> Option<Entity> {
    let w = sim.world_mut();
    let mut q = w.query::<(Entity, &Name)>();
    q.iter(w).find(|(_, n)| n.0 == nome).map(|(e, _)| e)
}

/// Que face do cartão a foto mostra.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CasoFisica {
    Dynamic,
    Kinematic,
    /// A plataforma do nível, feita pelo «Static Body» do menu Add.
    Static,
    /// A plataforma baixa antes e depois de Width 4: o Half Size do cartão tem de acompanhar.
    Baixa,
    Larga,
    Sensor,
    Bola,
    /// A bola com Scale 2 × 1: o aviso e as duas saídas.
    Esticada,
    /// O Player do nível: cápsula, com o cartão do jogador ao lado.
    Player,
    /// O Player com os cartões de cima dobrados, para Body e Shape caberem na foto.
    PlayerCartoes,
    /// O mesmo com a curva do pulo no canvas (ficha GEF 04): «Show Jump Arc» marcado e um pulo do ar.
    PlayerPulo,
    /// O mesmo sobre a «Platform Low» (12/11): o Full desce ao chão de baixo, o Standing pousa na
    /// «Platform High» e o Short pára na parede dela.
    PlayerPuloTerreno,
    /// Uma forma sem corpo acima: a «Collision Shape» acabada de nascer do menu Add.
    Solta,
    /// Uma forma dentro de um corpo.
    Peca,
    /// Uma imagem sem física: a face que oferece o corpo.
    Vazia,
    /// A caixa com camadas com nome, a matriz a separar duas e a lista aberta (onda 2).
    Camadas,
    /// O mesmo, com a camada sem par: «Hits: nothing».
    CamadaSozinha,
    /// A camada Player toca Ground e Enemy: «Hits» com os nomes e o Rename aberto.
    CamadaComPares,
    /// O mesmo, com o painel Physics à vista: a matriz e os campos dos nomes.
    CamadasNoPainel,
    /// Onda 3, as alças da forma: a caixa em edição; o canto a meio do arrasto; bola; a cápsula do Player.
    FormaCaixa,
    FormaArrasto,
    FormaBola,
    FormaPlayer,
    /// Scale 2 × 1, rodada 30°; e o mesmo com espelho.
    FormaRodado,
    FormaEspelho,
    /// O mesmo com o Pivot da imagem no canto, pelo gesto do Inspector.
    FormaPivo,
    /// O canto puxado e Ctrl+Z.
    FormaDesfaz,
    /// O canto puxado com Auto Key.
    FormaChave,
    /// As mesmas com Ctrl+Z, com Esc, com Alt, com Shift e com o segundo clique no botão.
    FormaChaveDesfaz,
    FormaEsc,
    FormaEscChave,
    FormaAlt,
    FormaShift,
    FormaDesliga,
    /// Uma peça dentro da plataforma (pai fora da origem), um Sensor e uma forma solta.
    FormaPeca,
    FormaSensor,
    FormaSolta,
    /// O clique parado no miolo: a forma por cima da Platform High, que fica selecionada (17b A1).
    FormaClique,
    /// Só na cena 120: Width 800 e o clique parado na ponta da forma, sobre a zona.
    FormaPonta,
    /// Só na cena 120: o Player, Fit to picture e Ctrl+Z; e sem o Ctrl+Z, para o cartão na foto.
    FormaFit,
    FormaFitFica,
}

/// O gesto de cada foto das alças.
#[must_use]
pub fn plano_da_forma(caso: CasoFisica) -> Option<ph2d_shape_handles::Plano> {
    use ph2d_shape_handles::{Alca, Plano, Teclas};
    let canto = Plano {
        alca: Some(Alca::Lado(1, 1)),
        delta: [0.8, 0.9],
        ..Plano::default()
    };
    let lado = Plano {
        alca: Some(Alca::Lado(1, 0)),
        delta: [0.7, 0.0],
        ..Plano::default()
    };
    Some(match caso {
        CasoFisica::FormaCaixa => Plano::default(),
        CasoFisica::FormaArrasto => Plano {
            delta: [1.0, 0.5],
            segura: true,
            ..canto
        },
        CasoFisica::FormaBola => Plano {
            delta: [0.5, 0.0],
            segura: true,
            ..lado
        },
        CasoFisica::FormaPlayer => Plano {
            alca: Some(Alca::Lado(0, 1)),
            delta: [0.0, 0.4],
            segura: true,
            ..lado
        },
        CasoFisica::FormaRodado
        | CasoFisica::FormaEspelho
        | CasoFisica::FormaPivo
        | CasoFisica::FormaChave => canto,
        CasoFisica::FormaDesfaz | CasoFisica::FormaChaveDesfaz => Plano {
            desfaz: true,
            ..canto
        },
        CasoFisica::FormaEsc | CasoFisica::FormaEscChave => Plano { esc: true, ..canto },
        CasoFisica::FormaDesliga => Plano {
            desliga: true,
            ..canto
        },
        CasoFisica::FormaAlt => Plano {
            teclas: Teclas {
                alt: true,
                shift: false,
            },
            ..canto
        },
        CasoFisica::FormaShift => Plano {
            teclas: Teclas {
                alt: false,
                shift: true,
            },
            ..canto
        },
        CasoFisica::FormaPeca | CasoFisica::FormaSensor | CasoFisica::FormaSolta => lado,
        CasoFisica::FormaClique => Plano {
            alca: Some(ph2d_shape_handles::Alca::Miolo),
            ..Plano::default()
        },
        CasoFisica::FormaPonta => Plano {
            largura: Some("800"),
            na_ponta: true,
            ..lado
        },
        CasoFisica::FormaFit | CasoFisica::FormaFitFica => Plano {
            botao: ph2d_shape_handles::Botao::FitToPicture,
            sem_arrasto: true,
            desfaz: caso == CasoFisica::FormaFit,
            ..lado
        },
        _ => return None,
    })
}

/// Os nomes que [`caso`] aceita, separados por espaço; o arranque do smoke recusa o nome que nenhum leitor aceita
/// (`ph2d_game_smoke_fotos::nomes`), e o censo de lá confere esta lista contra os braços de [`caso`].
pub const NOMES: &str = "fisica_dynamic fisica_kinematic fisica_static fisica_baixa fisica_larga fisica_sensor fisica_bola \
    fisica_esticada fisica_player fisica_player_cartoes fisica_player_pulo fisica_player_pulo_terreno \
    fisica_solta fisica_peca fisica_vazia fisica_camadas fisica_camada_sozinha fisica_camada_com_pares \
    fisica_camadas_no_painel fisica_forma_caixa fisica_forma_arrasto fisica_forma_bola \
    fisica_forma_player fisica_forma_rodado fisica_forma_espelho fisica_forma_pivo fisica_forma_desfaz \
    fisica_forma_chave fisica_forma_chave_desfaz fisica_forma_esc fisica_forma_esc_chave \
    fisica_forma_alt fisica_forma_shift fisica_forma_desliga fisica_forma_peca fisica_forma_sensor \
    fisica_forma_solta fisica_forma_clique fisica_forma_ponta fisica_forma_fit fisica_forma_fit_fica";

/// O caso do nome pedido em `PH2D_GAME_SMOKE_FOTO`.
#[must_use]
pub fn caso(nome: &str) -> Option<CasoFisica> {
    Some(match nome.strip_prefix("fisica_")? {
        "dynamic" => CasoFisica::Dynamic,
        "kinematic" => CasoFisica::Kinematic,
        "static" => CasoFisica::Static,
        "baixa" => CasoFisica::Baixa,
        "larga" => CasoFisica::Larga,
        "sensor" => CasoFisica::Sensor,
        "bola" => CasoFisica::Bola,
        "esticada" => CasoFisica::Esticada,
        "player" => CasoFisica::Player,
        "player_cartoes" => CasoFisica::PlayerCartoes,
        "player_pulo" => CasoFisica::PlayerPulo,
        "player_pulo_terreno" => CasoFisica::PlayerPuloTerreno,
        "solta" => CasoFisica::Solta,
        "peca" => CasoFisica::Peca,
        "vazia" => CasoFisica::Vazia,
        "camadas" => CasoFisica::Camadas,
        "camada_sozinha" => CasoFisica::CamadaSozinha,
        "camada_com_pares" => CasoFisica::CamadaComPares,
        "camadas_no_painel" => CasoFisica::CamadasNoPainel,
        "forma_caixa" => CasoFisica::FormaCaixa,
        "forma_arrasto" => CasoFisica::FormaArrasto,
        "forma_bola" => CasoFisica::FormaBola,
        "forma_player" => CasoFisica::FormaPlayer,
        "forma_rodado" => CasoFisica::FormaRodado,
        "forma_espelho" => CasoFisica::FormaEspelho,
        "forma_pivo" => CasoFisica::FormaPivo,
        "forma_desfaz" => CasoFisica::FormaDesfaz,
        "forma_chave" => CasoFisica::FormaChave,
        "forma_chave_desfaz" => CasoFisica::FormaChaveDesfaz,
        "forma_esc" => CasoFisica::FormaEsc,
        "forma_esc_chave" => CasoFisica::FormaEscChave,
        "forma_alt" => CasoFisica::FormaAlt,
        "forma_shift" => CasoFisica::FormaShift,
        "forma_desliga" => CasoFisica::FormaDesliga,
        "forma_peca" => CasoFisica::FormaPeca,
        "forma_sensor" => CasoFisica::FormaSensor,
        "forma_solta" => CasoFisica::FormaSolta,
        "forma_clique" => CasoFisica::FormaClique,
        "forma_ponta" => CasoFisica::FormaPonta,
        "forma_fit" => CasoFisica::FormaFit,
        "forma_fit_fica" => CasoFisica::FormaFitFica,
        _ => return None,
    })
}

/// Os nomes da foto das camadas; as outras ficam com o número.
const NOMES_DA_FOTO: [&str; 3] = ["Ground", "Player", "Enemy"];

const CAIXA: ColliderShape = ColliderShape::Cuboid {
    half_x: 0.5,
    half_y: 0.5,
};

/// A forma de `corpo` passa à filha «Collision Shape» (opção B, ADR-0199); devolve a filha, que é quem
/// leva o Shape, as Collision Layers e as alças.
fn na_filha(w: &mut ph2d_ecs::World, corpo: Entity) -> Entity {
    ph2d_physics_ecs::move_forma_para_filha(w, corpo).expect("um corpo com forma")
}

/// A imagem de `alvo` (bits): o corpo da filha «Collision Shape», ou o próprio `alvo`.
#[must_use]
pub fn imagem_de(sim: &SimWorld, alvo: u64) -> u64 {
    let e = Entity::from_bits(alvo);
    sim.world()
        .get::<ChildOf>(e)
        .map_or(alvo, |c| c.parent().to_bits())
}

/// Do pé do player à borda direita da «Platform Low» (m). Sonda de 12/11 com os padrões: a 2,1 m o Standing
/// pousa na alta (x 9,46), o Full desce ao chão (11,78) e o Short bate na parede da alta (6,8); a 0,5 m o
/// Standing já passa por cima dela.
pub const RECUO_DA_BORDA: f32 = 2.1;

/// A borda direita e o topo da «Platform Low», pelo desenho do bloco (o azulejo tem o tamanho dele).
fn borda_da_baixa(sim: &mut SimWorld) -> Option<[f32; 2]> {
    let e = por_nome(sim, "Platform Low")?;
    let w = sim.world();
    let (t, s) = (w.get::<Transform>(e)?, w.get::<Sprite>(e)?);
    let meia = [0.5 * s.size[0] * t.scale.x, 0.5 * s.size[1] * t.scale.y];
    Some([t.translation.x + meia[0], t.translation.y + meia[1]])
}

/// Monta o caso no mundo e devolve quem fica selecionado: o corpo nas fotos do cartão Body, a filha
/// «Collision Shape» nas da forma (o Pivot da [`CasoFisica::FormaPivo`] é da imagem: o pai dela).
pub fn prepara(sim: &mut SimWorld, caso: CasoFisica, onde: [f32; 2]) -> Option<u64> {
    let onde = Transform::from_translation(Vec2::new(onde[0], onde[1]));
    let imagem = Sprite::atlas(WHITE_TILE_KEY, [1.0, 1.0], [1.0; 4]);
    let corpo = |kind| RigidBody { kind };
    let pai = por_nome(sim, "Platform High")?;
    match caso {
        CasoFisica::Static => return Some(pai.to_bits()),
        CasoFisica::FormaPonta | CasoFisica::FormaFit | CasoFisica::FormaFitFica => return None,
        CasoFisica::Player | CasoFisica::PlayerCartoes | CasoFisica::FormaPlayer => {
            return por_nome(sim, "Player").map(|e| e.to_bits());
        }
        CasoFisica::PlayerPulo | CasoFisica::PlayerPuloTerreno => {
            let e = por_nome(sim, "Player")?;
            let borda = match caso {
                CasoFisica::PlayerPuloTerreno => Some(borda_da_baixa(sim)?),
                _ => None,
            };
            let w = sim.world_mut();
            let mut float = 0.0;
            if let Some(mut p) = w.get_mut::<ph2d_physics_ecs::PlatformPlayer>(e) {
                p.show_jump_arc = true;
                p.air_jumps = 1;
                float = p.float_height;
            }
            if let Some([direita, topo]) = borda
                && let Some(mut t) = w.get_mut::<Transform>(e)
            {
                t.translation = Vec2::new(direita - RECUO_DA_BORDA, topo + float);
            }
            return Some(e.to_bits());
        }
        CasoFisica::Baixa | CasoFisica::Larga => {
            return por_nome(sim, "Platform Low").map(|e| e.to_bits());
        }
        _ => {}
    }
    let w = sim.world_mut();
    let alvo = match caso {
        CasoFisica::Static
        | CasoFisica::FormaPonta
        | CasoFisica::FormaFit
        | CasoFisica::FormaFitFica
        | CasoFisica::Player
        | CasoFisica::PlayerCartoes
        | CasoFisica::PlayerPulo
        | CasoFisica::PlayerPuloTerreno
        | CasoFisica::FormaPlayer
        | CasoFisica::Baixa
        | CasoFisica::Larga => {
            unreachable!("voltam acima")
        }
        CasoFisica::Camadas
        | CasoFisica::CamadaSozinha
        | CasoFisica::CamadaComPares
        | CasoFisica::CamadasNoPainel => {
            for (i, n) in NOMES_DA_FOTO.iter().enumerate() {
                LayerNames::rename(w, i as u8, n);
            }
            let col = Collider {
                shape: CAIXA,
                layer: 1,
                ..Collider::default()
            };
            let e = w.spawn((
                onde,
                Name::new("Crate"),
                imagem,
                corpo(BodyKind::Dynamic),
                col,
            ));
            let e = e.id();
            na_filha(w, e)
        }
        CasoFisica::FormaCaixa
        | CasoFisica::FormaArrasto
        | CasoFisica::FormaBola
        | CasoFisica::FormaRodado
        | CasoFisica::FormaEspelho
        | CasoFisica::FormaPivo
        | CasoFisica::FormaDesfaz
        | CasoFisica::FormaChaveDesfaz
        | CasoFisica::FormaEsc
        | CasoFisica::FormaEscChave
        | CasoFisica::FormaAlt
        | CasoFisica::FormaShift
        | CasoFisica::FormaDesliga
        | CasoFisica::FormaChave => {
            let shape = if caso == CasoFisica::FormaBola {
                ColliderShape::Ball { radius: 0.5 }
            } else {
                CAIXA
            };
            let mut pose = onde;
            if matches!(
                caso,
                CasoFisica::FormaRodado
                    | CasoFisica::FormaPivo
                    | CasoFisica::FormaDesfaz
                    | CasoFisica::FormaChaveDesfaz
                    | CasoFisica::FormaEsc
                    | CasoFisica::FormaEscChave
                    | CasoFisica::FormaAlt
                    | CasoFisica::FormaShift
                    | CasoFisica::FormaDesliga
                    | CasoFisica::FormaChave
            ) {
                pose.scale = Vec2::new(2.0, 1.0);
                pose.rotation = 30f32.to_radians();
            }
            if caso == CasoFisica::FormaEspelho {
                pose.scale = Vec2::new(-2.0, 1.0);
                pose.rotation = 30f32.to_radians();
            }
            let col = Collider {
                shape,
                offset: [0.2, 0.1],
                ..Collider::default()
            };
            let e = w.spawn((
                pose,
                Name::new("Crate"),
                imagem,
                corpo(BodyKind::Static),
                col,
            ));
            let e = e.id();
            na_filha(w, e)
        }
        CasoFisica::FormaSensor => {
            let col = Collider {
                shape: CAIXA,
                is_sensor: true,
                ..Collider::default()
            };
            let nome = Name::new("Zone");
            let e = w.spawn((onde, nome, corpo(BodyKind::Static), col)).id();
            na_filha(w, e)
        }
        CasoFisica::FormaSolta => {
            let col = Collider {
                shape: CAIXA,
                ..Collider::default()
            };
            w.spawn((onde, Name::new("Collision Shape"), col)).id()
        }
        CasoFisica::FormaPeca => {
            let col = Collider {
                shape: CAIXA,
                offset: [0.1, 0.0],
                ..Collider::default()
            };
            let junto = Transform::from_translation(Vec2::new(0.0, 1.0));
            let nome = Name::new("Collision Shape");
            w.spawn((junto, nome, col, ChildOf(pai))).id()
        }
        CasoFisica::FormaClique => {
            let sobre = w.get::<Transform>(pai).copied()?;
            let nome = Name::new("Zone");
            // O miolo fora do anel do próprio objecto, sobre a plataforma: o clique no anel fica com ele.
            let col = Collider {
                shape: CAIXA,
                offset: [0.5, 0.0],
                is_sensor: true,
                ..Collider::default()
            };
            let e = w.spawn((sobre, nome, corpo(BodyKind::Static), col)).id();
            na_filha(w, e)
        }
        CasoFisica::Dynamic | CasoFisica::Kinematic | CasoFisica::Bola | CasoFisica::Esticada => {
            let kind = if caso == CasoFisica::Kinematic {
                BodyKind::Kinematic
            } else {
                BodyKind::Dynamic
            };
            let shape = if matches!(caso, CasoFisica::Bola | CasoFisica::Esticada) {
                ColliderShape::Ball { radius: 0.5 }
            } else {
                CAIXA
            };
            let col = Collider {
                shape,
                ..Collider::default()
            };
            let nome = Name::new("Crate");
            let mut pose = onde;
            if caso == CasoFisica::Esticada {
                pose.scale = Vec2::new(1.0, 3.0);
            }
            let e = w.spawn((pose, nome, imagem, corpo(kind), col)).id();
            let filha = na_filha(w, e);
            // A cadeia (dono 10/11): a forma (2, 1) sob o corpo (1, 3) — um clique repõe as duas.
            if caso == CasoFisica::Esticada
                && let Some(mut t) = w.get_mut::<ph2d_ecs::Transform>(filha)
            {
                t.scale = Vec2::new(2.0, 1.0);
            }
            // Dynamic e Kinematic mostram o cartão Body; Bola e Esticada, o Shape.
            if matches!(caso, CasoFisica::Bola | CasoFisica::Esticada) {
                filha
            } else {
                e
            }
        }
        CasoFisica::Sensor => {
            let col = Collider {
                shape: CAIXA,
                is_sensor: true,
                ..Collider::default()
            };
            let nome = Name::new("Zone");
            let e = w.spawn((onde, nome, corpo(BodyKind::Static), col)).id();
            na_filha(w, e)
        }
        CasoFisica::Solta => {
            let col = Collider {
                shape: CAIXA,
                ..Collider::default()
            };
            w.spawn((onde, Name::new("Collision Shape"), col)).id()
        }
        CasoFisica::Peca => {
            let col = Collider {
                shape: CAIXA,
                ..Collider::default()
            };
            let junto = Transform::from_translation(Vec2::new(0.0, 1.0));
            let nome = Name::new("Collision Shape");
            w.spawn((junto, nome, col, ChildOf(pai))).id()
        }
        CasoFisica::Vazia => w.spawn((onde, Name::new("Sprite"), imagem)).id(),
    };
    assign_missing_stable_ids(w);
    assign_missing_root_order(w);
    Some(alvo.to_bits())
}

/// O que o Player tem agora, numa linha (o tamanho, o ladrilho, o corte e a memória por uso).
#[must_use]
pub fn estado_do_player(sim: &mut SimWorld) -> String {
    let Some(e) = por_nome(sim, "Player") else {
        return "sem Player".into();
    };
    let w = sim.world();
    format!(
        "size={:?} tiled={:?} grid={:?} area={:?} anims={} memoria={:?}",
        w.get::<ph2d_render::Sprite>(e).map(|s| s.size),
        w.get::<ph2d_ecs::TiledDraw>(e).map(|t| t.tile),
        w.get::<ph2d_ecs::SpriteGrid>(e)
            .map(|g| (g.hframes, g.vframes)),
        w.get::<ph2d_ecs::SpriteRegion>(e).map(|r| r.rect),
        w.get::<ph2d_ecs::SpriteAnimations>(e).is_some(),
        w.get::<ph2d_ecs::ImageUseMemory>(e)
            .map(|m| [&m.still, &m.repeated, &m.animated].map(|s| s
                .as_ref()
                .and_then(|s| s.look.map(|l| (l.size, l.tiled.map(|t| t.tile)))))),
    )
}

#[cfg(test)]
#[path = "fotos_dos_cartoes_tests.rs"]
mod tests;
