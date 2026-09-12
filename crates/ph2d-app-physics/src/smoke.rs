//! ⭐⭐ **O ROTEADOR DAS CENAS DE FÍSICA** — `PH2D_PHYSICS_SMOKE=<n>`, lido AQUI.
//!
//! Ele viveu na `shells/desktop` até 2026-09-11 porque era um `impl App`: cada braço
//! chamava um método, e um método precisa da `App`. A Fase B converteu as 118 cenas em
//! funções livres sobre um [`crate::SceneCtx`], e o que sobrou do `match` é o que ele
//! sempre foi — uma tabela de `n → cena`, que não precisa de shell nenhuma.
//!
//! ⚠️ **O que NÃO veio, e de propósito:** o prólogo (rebobinar, armar o toggle de física,
//! abrir a timeline, decidir play/pause) continua na shell. Ele mexe no `Playhead`, nas
//! `flags` da timeline e no `HeroScreen` — três coisas que são da composição, não da
//! família. *O que sai são os CORPOS; o que decide a ordem do quadro fica.*

/// **As cenas que abrem PARADAS.** Uma cena que espera um gesto do artista
/// (adicionar um corpo, assar, arrastar um rig) não pode ter meio caído antes de
/// ele chegar ao mouse.
///
/// Uma TABELA e não uma cadeia de `|`: com vinte e poucas entradas o `matches!`
/// gastava uma linha por cena e comia o teto de LOC deste arquivo — e a lista é
/// exatamente o tipo de coisa que só cresce.
/// ⚠️ **Uma cena que pede um GESTO DE ALÇA tem de estar aqui.** As alças de ponto
/// (âncora de joint, centro/aro de roldana) são publicadas **rest-only** — durante
/// o play o overlay desenha a geometria do SOLVER, e elas autoram a AUTORADA —,
/// então uma cena que nasce tocando simplesmente **não tem alça nenhuma**, e o
/// artista relata a feature como quebrada (foi o que aconteceu com a 63).
///
/// ⚠️ **E isto é uma ENUMERAÇÃO escrita à mão**, ou seja exatamente a forma que a
/// próxima cena nasce fora. O gate `handle_scenes_start_paused` varre as mensagens
/// das cenas e exige que quem manda arrastar uma alça esteja nesta lista.
pub const PAUSED_SCENES: &[&str] = &[
    "3", "7", "14", "15", "16", "17", "21", "22", "23", "24", "37", "38", "39", "40", "41", "43",
    "44", "45", "46", "47", "51", "54", "58", "63", "64", "65", "66", "67", "68", "69", "70", "71",
    "72", "73", "74", "75",
    // ⚠️ A cena 82 (W5) nasce PAUSADA pela razão da 3: ela espera o artista
    // fazer alguma coisa, e um corpo que já caiu meio metro é um corpo cujo
    // gesto de autoria começa no lugar errado.
    "82",
    // ⚠️ A cena 96 (W17) nasce PAUSADA pela razão exata da 95: ela pede uma
    // corrida JOGADA, e com o relógio já a andar o começo da fita descreveria
    // segundos em que ninguém tinha o teclado.
    "96",
    // ⚠️ A cena 95 (W16) nasce PAUSADA pela razão da 7, e mais uma: ela pede ao
    // artista que JOGUE uma corrida, e é essa corrida que vai ser assada. Com o
    // relógio já a andar, o começo da fita descreve segundos em que ninguém
    // tinha o teclado — o bake gravaria um personagem parado antes de gravar o
    // que o artista fez.
    "95",
];

/// **O maior nível a que o roteador de facto responde** (`PH2D_PHYSICS_SMOKE=1..CENAS`).
///
/// ⚠️⚠️ **CONTADO no `match` abaixo, nunca escrito de memória** (CLAUDE.md §5.0: *«o número
/// da próxima cena de smoke CONTA-SE lendo o roteador»*). O gate
/// `o_roteador_responde_por_todo_nivel_que_promete` mede-o pelas DUAS pontas: a cena
/// `CENAS` tem de ser dela própria, e a `CENAS + 1` tem de cair no `_`.
///
/// ⚠️ Esta nota do `CLAUDE.md` já esteve parada em `97` com cenas até `102` noutro módulo —
/// é exactamente por isso que o número vive ao lado do `match` e tem gate.
pub const CENAS: u32 = 119;

/// **O nível que o dono pediu, lido DENTRO da crate.**
///
/// ⭐ É esta função que torna a família alcançável pelo registo: o `crate::FAMILY` declara
/// `PH2D_PHYSICS_SMOKE`, e quem o lê é a crate que o declara. Enquanto a leitura vivia na
/// shell, o registo dizia que a família respondia por uma env que ela não via.
pub fn armed_scene() -> Option<String> {
    std::env::var("PH2D_PHYSICS_SMOKE").ok()
}

/// **A tabela `n → cena`.** Cada braço povoa o `ctx` e declara o que quer da shell.
///
/// ⚠️ Um `n` que ninguém reclama cai na cena 1 (`_`), que é a que ensina o básico — nunca
/// num ecrã vazio.
pub fn scene(which: &str, ctx: &mut crate::SceneCtx<'_>) {
    match which {
        "2" => crate::physics_smoke_base::physics_smoke_pile(ctx),
        "3" => crate::physics_smoke_base::physics_smoke_author(ctx),
        "4" => crate::physics_smoke_base::physics_smoke_world(ctx),
        "5" => crate::physics_smoke_base::physics_smoke_layers(ctx),
        "6" => crate::physics_smoke_rigs::physics_smoke_joints(ctx),
        "7" => crate::physics_smoke_rigs::physics_smoke_bake(ctx),
        "8" => crate::physics_smoke_rigs::physics_smoke_parented(ctx),
        "9" => crate::physics_smoke_collider::physics_smoke_scale(ctx),
        "10" => crate::physics_smoke_collider::physics_smoke_sensor(ctx),
        "11" => crate::physics_smoke_rigs::physics_smoke_weld(ctx),
        "12" => crate::physics_smoke_props::physics_smoke_gravity(ctx),
        "13" => crate::physics_smoke_props::physics_smoke_capsule(ctx),
        "14" => crate::physics_smoke_props::physics_smoke_launch(ctx),
        "15" => crate::physics_smoke_props::physics_smoke_ccd(ctx),
        "16" => crate::physics_smoke_props::physics_smoke_lock_rotation(ctx),
        "17" => crate::physics_smoke_props::physics_smoke_offset(ctx),
        "18" => crate::physics_smoke_props::physics_smoke_freeze_position(ctx),
        "19" => crate::physics_smoke_collision::physics_smoke_mass(ctx),
        "20" => crate::physics_smoke_collision::physics_smoke_dominance(ctx),
        "21" => crate::physics_smoke_collision::physics_smoke_material(ctx),
        "22" => crate::physics_smoke_damping::physics_smoke_damping(ctx),
        "23" => crate::physics_smoke_collision::physics_smoke_one_way(ctx),
        "24" => crate::physics_smoke_collision::physics_smoke_area(ctx),
        "25" => crate::physics_smoke_contacts::physics_smoke_contacts(ctx),
        "26" => crate::physics_smoke_contacts::physics_smoke_area_drag(ctx),
        "27" => crate::physics_smoke_contacts::physics_smoke_buoyancy(ctx),
        "28" => crate::physics_smoke_contacts::physics_smoke_form_drag(ctx),
        "29" => crate::physics_smoke_events::physics_smoke_events(ctx),
        "30" => crate::physics_smoke_events::physics_smoke_impact_demolition(ctx),
        "31" => crate::physics_smoke_events::physics_smoke_fast_impact(ctx),
        "32" => crate::physics_smoke_zones::physics_smoke_spin_zone(ctx),
        "33" => crate::physics_smoke_zones::physics_smoke_author_spin(ctx),
        "34" => crate::physics_smoke_zones::physics_smoke_force_frame(ctx),
        "35" => crate::physics_smoke_zones::physics_smoke_falloff(ctx),
        "36" => crate::physics_smoke_zones::physics_smoke_mirror(ctx),
        "37" => crate::physics_smoke_rigs::physics_smoke_bake_range(ctx),
        "38" => crate::physics_smoke_authoring::physics_smoke_joint_anchor(ctx),
        "39" => crate::physics_smoke_joint_bake::physics_smoke_bake_joint(ctx),
        "40" => crate::physics_smoke_authoring::physics_smoke_author_joint(ctx),
        "41" => crate::physics_smoke_authoring::physics_smoke_anchor_follows(ctx),
        "42" => crate::physics_smoke_authoring::physics_smoke_live_tune(ctx),
        "43" => crate::physics_smoke_joint_glyphs::physics_smoke_joint_glyphs(ctx),
        "44" => crate::physics_smoke_joint_handles::physics_smoke_joint_handles(ctx),
        "45" => crate::physics_smoke_joint_pose::physics_smoke_joint_pose(ctx),
        "46" => crate::physics_smoke_joint_draw::physics_smoke_joint_draw(ctx),
        "47" => crate::physics_smoke_joint_slider::physics_smoke_joint_slider(ctx),
        "48" => crate::physics_smoke_joint_motor::physics_smoke_joint_motor(ctx),
        "49" => crate::physics_smoke_joint_break::physics_smoke_joint_break(ctx),
        "50" => crate::physics_smoke_joint_pair::physics_smoke_joint_pair(ctx),
        "51" => crate::physics_smoke_joint_rig::physics_smoke_joint_rig(ctx),
        "52" => crate::physics_smoke_grab::physics_smoke_grab(ctx),
        "53" => crate::physics_smoke_interact::physics_smoke_interact(ctx),
        "54" => crate::physics_smoke_ik::physics_smoke_ik(ctx),
        "55" => crate::physics_smoke_fk::physics_smoke_fk(ctx),
        "56" => crate::physics_smoke_rod::physics_smoke_rod(ctx),
        "57" => crate::physics_smoke_wheel::physics_smoke_wheel(ctx),
        "58" => crate::physics_smoke_pulley::physics_smoke_pulley(ctx),
        "59" => crate::physics_smoke_pulley::physics_smoke_winch(ctx),
        "60" => crate::physics_smoke_pulley_break::physics_smoke_break(ctx),
        "61" => crate::physics_smoke_pulley_tackle::physics_smoke_tackle(ctx),
        "62" => crate::physics_smoke_pulley_diff::physics_smoke_differential(ctx),
        "63" => crate::physics_smoke_pulley_comp::physics_smoke_composition(ctx),
        "64" => crate::physics_smoke_pulley_weston::physics_smoke_weston(ctx),
        "65" => crate::physics_smoke_world_pin::physics_smoke_world_pin(ctx),
        "66" => crate::physics_smoke_joint_copy::physics_smoke_joint_copy(ctx),
        "67" => crate::physics_smoke_rig::physics_smoke_rig(ctx),
        "68" => crate::physics_smoke_soft_weld::physics_smoke_soft_weld(ctx),
        "69" => crate::physics_smoke_compound::physics_smoke_compound(ctx),
        "70" => crate::physics_smoke_part::physics_smoke_part(ctx),
        "71" => crate::physics_smoke_foot::physics_smoke_foot(ctx),
        "72" => crate::physics_smoke_raft::physics_smoke_raft(ctx),
        "73" => crate::physics_smoke_signal::physics_smoke_signal(ctx),
        "74" => crate::physics_smoke_lead::physics_smoke_lead(ctx),
        "75" => crate::physics_smoke_stop::physics_smoke_stop(ctx),
        "76" => crate::physics_smoke_signal_leave::physics_smoke_signal_leave(ctx),
        "77" => crate::physics_smoke_rail_rope::physics_smoke_rail_rope(ctx),
        "78" => crate::physics_smoke_joint_anim::physics_smoke_joint_anim(ctx),
        "79" => crate::physics_smoke_joint_custom::physics_smoke_joint_custom(ctx),
        "80" => crate::physics_smoke_player::physics_smoke_float(ctx),
        "81" => crate::physics_smoke_player::physics_smoke_walk(ctx),
        "82" => crate::physics_smoke_player::physics_smoke_author_player(ctx),
        "83" => crate::physics_smoke_player::physics_smoke_jump(ctx),
        "85" => crate::physics_smoke_player::physics_smoke_reaction(ctx),
        "86" => crate::physics_smoke_player_tape::physics_smoke_tape(ctx),
        "87" => crate::physics_smoke_player_forgive::physics_smoke_forgive(ctx),
        "88" => crate::physics_smoke_player_slope::physics_smoke_slope(ctx),
        "89" => crate::physics_smoke_player_carry::physics_smoke_chimney(ctx),
        "90" => crate::physics_smoke_player_carry::physics_smoke_wagon(ctx),
        "91" => crate::physics_smoke_player_drop::physics_smoke_pass_through(ctx),
        "92" => crate::physics_smoke_player_wall::physics_smoke_well(ctx),
        "93" => crate::physics_smoke_player_dash::physics_smoke_dash(ctx),
        "94" => crate::physics_smoke_player_crouch::physics_smoke_crouch(ctx),
        "95" => crate::physics_smoke_player_bake::physics_smoke_bake_run(ctx),
        "96" => crate::physics_smoke_player_run::physics_smoke_recorded_run(ctx),
        "97" => crate::physics_smoke_player_drop::physics_smoke_drop_edges(ctx),
        "98" => crate::physics_smoke_player_flank::physics_smoke_flank(ctx),
        "99" => crate::physics_smoke_player_grab::physics_smoke_wall_grab(ctx),
        "100" => crate::physics_smoke_water::physics_smoke_water(ctx),
        "101" => crate::physics_smoke_kinematic::physics_smoke_kinematic(ctx),
        "102" => crate::physics_smoke_kin_push::physics_smoke_kin_push(ctx),
        "103" => crate::physics_smoke_kin_pure::physics_smoke_kin_pure(ctx),
        "104" => crate::physics_smoke_kin_water::physics_smoke_kin_water(ctx),
        "105" => crate::physics_smoke_swim::physics_smoke_swim(ctx),
        "106" => crate::physics_smoke_zone_force::physics_smoke_zone_force(ctx),
        "107" => crate::physics_smoke_stone::physics_smoke_stone(ctx),
        // ⚠️ **O NUMERO E' CONTADO, nunca escolhido** — a `=105` estava
        // tomada (o mergulho), e a nota que a dava como livre tinha
        // envelhecido. Quem pega o proximo LE' este `match`, e o compilador
        // e' o gate: um segundo braco com o mesmo literal e' `unreachable`.
        "108" => crate::physics_smoke_probes::physics_smoke_probes(ctx),
        "109" => crate::physics_smoke_foot_fan::physics_smoke_foot_fan(ctx),
        "110" => crate::physics_smoke_multi_jump::physics_smoke_multi_jump(ctx),
        "111" => crate::physics_smoke_ledge::physics_smoke_ledge(ctx),
        "112" => crate::physics_smoke_glide::physics_smoke_glide(ctx),
        "113" => crate::physics_smoke_out::physics_smoke_out(ctx),
        "114" => crate::physics_smoke_brake::physics_smoke_brake(ctx),
        "115" => crate::physics_smoke_surface::physics_smoke_surface(ctx),
        // ⚠️ **O NUMERO E' CONTADO, nunca escolhido** — lido deste `match`,
        // que e' a fonte; a nota da §5 do CLAUDE.md dava a `=105` como a
        // proxima livre e tinha envelhecido em onze cenas.
        "116" => crate::physics_smoke_terminal::physics_smoke_terminal(ctx),
        "117" => crate::physics_smoke_blast::physics_smoke_blast(ctx),
        // ⚠️ **O NUMERO E' CONTADO, nunca escolhido** — lido deste `match`,
        // que e' a fonte. O compilador e' o gate: um segundo braco com o
        // mesmo literal e' `unreachable`.
        "118" => crate::physics_smoke_leave::physics_smoke_leave(ctx),
        // ⚠️ **O NUMERO E' CONTADO, nunca escolhido** — lido deste `match`.
        "119" => crate::physics_smoke_brink::physics_smoke_brink(ctx),
        _ => crate::physics_smoke_base::physics_smoke_drop(ctx),
    }
}

#[cfg(test)]
#[path = "smoke_tests.rs"]
mod smoke_tests;
