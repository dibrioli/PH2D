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

/// A secção CAMERA — o cabeçalho colapsável. Entra em [`super::LIVE_SECTIONS`] com o ponto de cor.
pub const INSP_LIVE_CAMERA_SECTION: NodeId = hash_node_id("insp_live_camera_section");
/// CAMERA — ponto de cor do cabeçalho.
pub const INSP_LIVE_CAMERA_COLOR: NodeId = hash_node_id("insp_live_camera_color");

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
/// O cabeçalho dobrável da secção TWEEN — «esta propriedade vai de A a B» (suplente #22).
pub const INSP_LIVE_TWEEN_SECTION: NodeId = hash_node_id("insp_live_tween_section");
/// O cabeçalho dobrável da secção PATH FOLLOW — «anda sobre a curva desenhada» (suplente #23).
pub const INSP_LIVE_PATHFOLLOW_SECTION: NodeId = hash_node_id("insp_live_pathfollow_section");
/// O cabeçalho dobrável da secção CAMERA SHAKE — *como* esta câmera treme (suplente #25).
pub const INSP_LIVE_SHAKE_SECTION: NodeId = hash_node_id("insp_live_shake_section");
/// O cabeçalho dobrável da secção SHAKE EMITTER — *ao ouvir o quê* este objecto abana a vista.
pub const INSP_LIVE_EMITTER_SECTION: NodeId = hash_node_id("insp_live_emitter_section");
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
pub const INSP_LIVE_FACTORY_COLOR: NodeId = hash_node_id("insp_live_factory_color");
/// Ver [`INSP_LIVE_FACTORY_COLOR`].
pub const INSP_LIVE_LIFECYCLE_COLOR: NodeId = hash_node_id("insp_live_lifecycle_color");
/// Ver [`INSP_LIVE_FACTORY_COLOR`].
pub const INSP_LIVE_TOPDOWN_COLOR: NodeId = hash_node_id("insp_live_topdown_color");
/// Ver [`INSP_LIVE_FACTORY_COLOR`].
pub const INSP_LIVE_PROJECTILE_COLOR: NodeId = hash_node_id("insp_live_projectile_color");
/// Ver [`INSP_LIVE_FACTORY_COLOR`].
pub const INSP_LIVE_SM_COLOR: NodeId = hash_node_id("insp_live_sm_color");
/// Ver [`INSP_LIVE_FACTORY_COLOR`].
pub const INSP_LIVE_SCRIPT_COLOR: NodeId = hash_node_id("insp_live_script_color");
/// Ver [`INSP_LIVE_FACTORY_COLOR`].
pub const INSP_LIVE_PARTICLES_COLOR: NodeId = hash_node_id("insp_live_particles_color");
/// O ponto de cor da secção HUD (TOP-20 #20).
pub const INSP_LIVE_HUD_COLOR: NodeId = hash_node_id("insp_live_hud_color");
/// O ponto de cor da secção SEQUENCE (TOP-20 #19).
pub const INSP_LIVE_SEQ_COLOR: NodeId = hash_node_id("insp_live_seq_color");
/// O ponto de cor da secção COUNTER WATCH.
pub const INSP_LIVE_WATCH_COLOR: NodeId = hash_node_id("insp_live_watch_color");
/// O ponto de cor da secção GATILHO.
pub const INSP_LIVE_TRIGGER_COLOR: NodeId = hash_node_id("insp_live_trigger_color");
/// O ponto de cor da secção TWEEN (suplente #22).
pub const INSP_LIVE_TWEEN_COLOR: NodeId = hash_node_id("insp_live_tween_color");
/// O ponto de cor da secção PATH FOLLOW (suplente #23).
pub const INSP_LIVE_PATHFOLLOW_COLOR: NodeId = hash_node_id("insp_live_pathfollow_color");
/// O ponto de cor da secção CAMERA SHAKE (suplente #25).
pub const INSP_LIVE_SHAKE_COLOR: NodeId = hash_node_id("insp_live_shake_color");
/// O ponto de cor da secção SHAKE EMITTER (suplente #25).
pub const INSP_LIVE_EMITTER_COLOR: NodeId = hash_node_id("insp_live_emitter_color");
/// O ponto de cor da secção RAY SENSOR.
pub const INSP_LIVE_RAY_COLOR: NodeId = hash_node_id("insp_live_ray_color");
/// Quantas opções o segmentado do ONDE tem — a porta que o painel lê para repartir a largura.
pub const INSP_FACTORY_WHERE_LEN: usize = 3;
