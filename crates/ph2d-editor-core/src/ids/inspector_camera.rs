//! **Os ids da secção CAMERA** (TOP-20 #7, W3).
//!
//! ⚠️ **Irmão de [`super::inspector`] por CAP de LOC** — mesmo padrão do
//! [`super::inspector_audio`] e do [`super::inspector_timer`].
//!
//! # ⚠️ Uma secção, TRÊS corpos — e não três secções
//!
//! `GameCamera`, `CameraFollow` e `CameraLimits` são três componentes registados, e a separação
//! deles é deliberada (ver o catálogo). Mas para o artista é **uma** pergunta — *«o que este objecto
//! tem a ver com o enquadramento?»* —, e o ADR-0166 diz que o Inspector mostra **o que o objecto
//! tem**. Três secções fariam uma câmera fixa, que é o caso comum, pagar dois cabeçalhos vazios.
//!
//! # ⚠️ Não há lista, e por isso não há linha aberta
//!
//! Um objecto tem **uma** câmera, **um** seguidor e **uma** cerca. ⇒ esta secção não precisa de
//! estado de painel nenhum: ela lê o snapshot e pinta os campos, como a do áudio.

use super::*;

/// A secção CAMERA — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`] com a pega de arrasto.
pub const INSP_LIVE_CAMERA_SECTION: NodeId = hash_node_id("insp_live_camera_section");
/// CAMERA — a pega de arrasto do cabeçalho.
pub const INSP_LIVE_CAMERA_GRIP: NodeId = hash_node_id("insp_live_camera_grip");

/// A secção FACTORY — o cabeçalho colapsável (TOP-20 #11, W3).
///
/// ⚠️ **Aqui e não na crate do painel, como as irmãs `INSP_LIVE_*`**: o cabeçalho de secção é lido
/// pelo `LIVE_SECTIONS` da fundação (a ordem das secções vivas), e não só por quem pinta.
pub const INSP_LIVE_FACTORY_SECTION: NodeId = hash_node_id("insp_live_factory_section");
/// O cabeçalho dobrável da secção TOP-DOWN PLAYER (TOP-20 #13).
pub const INSP_LIVE_TOPDOWN_SECTION: NodeId = hash_node_id("insp_live_topdown_section");
/// O cabeçalho dobrável da secção PROJECTILE MOTION (TOP-20 #14).
pub const INSP_LIVE_PROJECTILE_SECTION: NodeId = hash_node_id("insp_live_projectile_section");
/// O cabeçalho dobrável da secção STATE MACHINE (TOP-20 #15).
pub const INSP_LIVE_SM_SECTION: NodeId = hash_node_id("insp_live_sm_section");
/// O cabeçalho dobrável da secção SCRIPT (TOP-20 #16).
pub const INSP_LIVE_SCRIPT_SECTION: NodeId = hash_node_id("insp_live_script_section");
/// O cabeçalho dobrável da secção PARTICLES (TOP-20 #18).
pub const INSP_LIVE_PARTICLES_SECTION: NodeId = hash_node_id("insp_live_particles_section");
/// O cabeçalho dobrável da secção HUD (TOP-20 #20).
pub const INSP_LIVE_HUD_SECTION: NodeId = hash_node_id("insp_live_hud_section");
/// O cabeçalho dobrável da secção SEQUENCE (TOP-20 #19).
pub const INSP_LIVE_SEQ_SECTION: NodeId = hash_node_id("insp_live_seq_section");
/// O cabeçalho dobrável da secção COUNTER WATCH — a vigia do contador.
pub const INSP_LIVE_WATCH_SECTION: NodeId = hash_node_id("insp_live_watch_section");
/// O cabeçalho dobrável da secção GATILHO — a mão de quem joga (suplente #24).
pub const INSP_LIVE_TRIGGER_SECTION: NodeId = hash_node_id("insp_live_trigger_section");
/// O cabeçalho dobrável da secção RAY SENSOR — o objecto que OLHA (suplente #21).
pub const INSP_LIVE_RAY_SECTION: NodeId = hash_node_id("insp_live_ray_section");
/// O cabeçalho dobrável da secção PARALLAX — *quanto do movimento do mundo esta camada guarda*
/// (plano 24, W7). ⚠️ Ela mora na família da CÂMERA e nunca vive numa: o número dela é sobre o
/// movimento DELA, e é ali que o artista o procura.
pub const INSP_LIVE_PARALLAX_SECTION: NodeId = hash_node_id("insp_live_parallax_section");
/// O cabeçalho dobrável da secção TWEEN — «esta propriedade vai de A a B» (suplente #22).
pub const INSP_LIVE_TWEEN_SECTION: NodeId = hash_node_id("insp_live_tween_section");
/// O cabeçalho dobrável da secção PATH FOLLOW — «anda sobre a curva desenhada» (suplente #23).
pub const INSP_LIVE_PATHFOLLOW_SECTION: NodeId = hash_node_id("insp_live_pathfollow_section");
/// O cabeçalho dobrável da secção CAMERA SHAKE — *como* esta câmera treme (suplente #25).
pub const INSP_LIVE_SHAKE_SECTION: NodeId = hash_node_id("insp_live_shake_section");
/// O cabeçalho dobrável da secção SHAKE EMITTER — *ao ouvir o quê* este objecto abana a vista.
pub const INSP_LIVE_EMITTER_SECTION: NodeId = hash_node_id("insp_live_emitter_section");
/// O cabeçalho dobrável da secção WEAPON — o RITMO, o PENTE e a recarga da arma do jogador.
pub const INSP_LIVE_WEAPON_SECTION: NodeId = hash_node_id("insp_live_weapon_section");
/// O cabeçalho dobrável da secção LIVE MESH — o CATAVENTO (`docs/3D/02.2`, rota B): a pose 3D da
/// malha que este sprite mantém viva, e as voltas por segundo dela.
pub const INSP_LIVE_MESH3D_SECTION: NodeId = hash_node_id("insp_live_mesh3d_section");
/// A secção LIFECYCLE — o cabeçalho colapsável (TOP-20 #12, W3).
pub const INSP_LIVE_LIFECYCLE_SECTION: NodeId = hash_node_id("insp_live_lifecycle_section");
/// ⛔⛔ **Os PONTOS DE COR das três secções novas, e eles nasceram de um DEFEITO MEDIDO.**
///
/// As secções FACTORY e LIFECYCLE shiparam em 2026-09-14 **fora** da [`super::LIVE_SECTIONS`], e o
/// doc daquela tabela já dizia o preço por escrito: quem falta ali **não é
/// `mark_collapsible_section`ado** e **não é `is_section_header_id`** — *o cabeçalho pinta o chevron
/// e a dobra não pode acontecer*. Foi a wave seguinte (TOP-20 #13), ao ir escrever a mesma linha,
/// que o viu.
///
/// ⚠️ *É exactamente a recaída que aquela tabela existe para impedir, e ela aconteceu à mesma* —
/// porque entrar na tabela continua a ser um passo que se pode **esquecer**. A cura de fundo seria
/// um censo que exigisse que todo `INSP_LIVE_*_SECTION` estivesse na tabela; ele está escrito no
/// gate irmão desta wave.
pub const INSP_LIVE_FACTORY_GRIP: NodeId = hash_node_id("insp_live_factory_grip");
/// Ver [`INSP_LIVE_FACTORY_GRIP`].
pub const INSP_LIVE_LIFECYCLE_GRIP: NodeId = hash_node_id("insp_live_lifecycle_grip");
/// Ver [`INSP_LIVE_FACTORY_GRIP`].
pub const INSP_LIVE_TOPDOWN_GRIP: NodeId = hash_node_id("insp_live_topdown_grip");
/// Ver [`INSP_LIVE_FACTORY_GRIP`].
pub const INSP_LIVE_PROJECTILE_GRIP: NodeId = hash_node_id("insp_live_projectile_grip");
/// Ver [`INSP_LIVE_FACTORY_GRIP`].
pub const INSP_LIVE_SM_GRIP: NodeId = hash_node_id("insp_live_sm_grip");
/// Ver [`INSP_LIVE_FACTORY_GRIP`].
pub const INSP_LIVE_SCRIPT_GRIP: NodeId = hash_node_id("insp_live_script_grip");
/// Ver [`INSP_LIVE_FACTORY_GRIP`].
pub const INSP_LIVE_PARTICLES_GRIP: NodeId = hash_node_id("insp_live_particles_grip");
/// A pega de arrasto da secção HUD (TOP-20 #20).
pub const INSP_LIVE_HUD_GRIP: NodeId = hash_node_id("insp_live_hud_grip");
/// A pega de arrasto da secção SEQUENCE (TOP-20 #19).
pub const INSP_LIVE_SEQ_GRIP: NodeId = hash_node_id("insp_live_seq_grip");
/// A pega de arrasto da secção COUNTER WATCH.
pub const INSP_LIVE_WATCH_GRIP: NodeId = hash_node_id("insp_live_watch_grip");
/// A pega de arrasto da secção GATILHO.
pub const INSP_LIVE_TRIGGER_GRIP: NodeId = hash_node_id("insp_live_trigger_grip");
/// A pega de arrasto da secção TWEEN (suplente #22).
pub const INSP_LIVE_TWEEN_GRIP: NodeId = hash_node_id("insp_live_tween_grip");
/// A pega de arrasto da secção PATH FOLLOW (suplente #23).
pub const INSP_LIVE_PATHFOLLOW_GRIP: NodeId = hash_node_id("insp_live_pathfollow_grip");
/// A pega de arrasto da secção CAMERA SHAKE (suplente #25).
pub const INSP_LIVE_SHAKE_GRIP: NodeId = hash_node_id("insp_live_shake_grip");
/// A pega de arrasto da secção SHAKE EMITTER (suplente #25).
pub const INSP_LIVE_EMITTER_GRIP: NodeId = hash_node_id("insp_live_emitter_grip");
/// A pega de arrasto da secção RAY SENSOR.
pub const INSP_LIVE_RAY_GRIP: NodeId = hash_node_id("insp_live_ray_grip");
/// A pega de arrasto da secção PARALLAX — ver [`INSP_LIVE_PARALLAX_SECTION`].
pub const INSP_LIVE_PARALLAX_GRIP: NodeId = hash_node_id("insp_live_parallax_grip");
/// A pega de arrasto da secção WEAPON.
pub const INSP_LIVE_WEAPON_GRIP: NodeId = hash_node_id("insp_live_weapon_grip");
/// O cabeçalho dobrável da secção HEALTH — quem LEVA (plano 28, W3).
pub const INSP_LIVE_HEALTH_SECTION: NodeId = hash_node_id("insp_live_health_section");
/// A pega de arrasto da secção HEALTH.
pub const INSP_LIVE_HEALTH_GRIP: NodeId = hash_node_id("insp_live_health_grip");
/// O cabeçalho dobrável da secção DAMAGE — quem BATE (plano 28, W3).
pub const INSP_LIVE_DAMAGE_SECTION: NodeId = hash_node_id("insp_live_damage_section");
/// A pega de arrasto da secção DAMAGE.
pub const INSP_LIVE_DAMAGE_GRIP: NodeId = hash_node_id("insp_live_damage_grip");
/// O cabeçalho dobrável da secção HEALTH BAR — quem MOSTRA a vida (plano 28, W4).
pub const INSP_LIVE_HEALTH_BAR_SECTION: NodeId = hash_node_id("insp_live_health_bar_section");
/// A pega de arrasto da secção HEALTH BAR.
pub const INSP_LIVE_HEALTH_BAR_GRIP: NodeId = hash_node_id("insp_live_health_bar_grip");
/// A pega de arrasto da secção LIVE MESH (o catavento).
pub const INSP_LIVE_MESH3D_GRIP: NodeId = hash_node_id("insp_live_mesh3d_grip");
/// Quantas opções o segmentado do ONDE tem — a porta que o painel lê para repartir a largura.
pub const INSP_FACTORY_WHERE_LEN: usize = 3;
