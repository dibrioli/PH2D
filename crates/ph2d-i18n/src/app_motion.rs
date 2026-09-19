//! **O QUE A FAMÍLIA app.motion DIZ** — os avisos (toasts), as recusas e os rótulos que a crate
//! `ph2d-app-motion` mostra (do MOTION (as recusas de ligar, os conselhos do grafo, os rótulos do cartão)), na forma `app.motion.<ficheiro>.<frase>`.
//!
//! ⚠️ **Frases com peças do código usam [`crate::tr_with`] com marcadores NOMEADOS** — uma língua
//! pode reordenar os marcadores; não pode mudar o que eles valem.
//!
//! ⛔ **O que NÃO está aqui, de propósito** (cada um com excepção NOMEADA no gate da crate): as
//! CENAS (de smoke e de demonstração), o diagnóstico de consola, os formatos de ficheiro, os nomes
//! de COLUNA (que um nó a jusante lê pelo nome) e os nomes por omissão de objecto — identidade
//! durável, não vocabulário.
//!
//! ⚠️ **Os braços entre os marcadores são do script** — uma chave nova escreve-se à mão FORA deles.

/// A tradução de uma chave `app.motion.*`, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ph2d-migrar-texto:begin
        "app.motion.motion_bridge_adapt.inserted_an_adapter_to_convert_the_types" => {
            "Inserted an adapter to convert the types"
        }
        "app.motion.motion_bridge_connect.can_t_connect_unknown_node" => {
            "Can't connect: unknown node"
        }
        "app.motion.motion_bridge_connect.can_t_connect_input_already_wired" => {
            "Can't connect: input already wired"
        }
        "app.motion.motion_bridge_connect.can_t_connect_would_create_a_cycle" => {
            "Can't connect: would create a cycle"
        }
        "app.motion.motion_bridge_connect.connected_as_1_tick_feedback_pre" => {
            "Connected as 1-tick feedback (pre)"
        }
        "app.motion.motion_bridge_connect.can_t_connect_time_remap_can_t_rewrite_time_for" => {
            "Can't connect: Time Remap can't rewrite time for a spring or integrate upstream"
        }
        "app.motion.motion_bridge_connect.can_t_connect_incompatible_ports" => {
            "Can't connect: incompatible ports"
        }
        "app.motion.motion_bridge_edit.state_wiring_is_automatic_disconnect_the_chain_f" => {
            "State wiring is automatic - disconnect the chain from the forces port instead"
        }
        "app.motion.motion_bridge_edit.that_wire_cannot_land_there_the_node_was_added_u" => {
            "That wire cannot land there - the node was added unconnected"
        }
        "app.motion.motion_bridge_fold.nodes" => "{inside} nodes",
        "app.motion.motion_bridge_heal.this_node_produces_data_nothing_downstream_consu" => {
            "This node produces data nothing downstream consumes, so it does nothing"
        }
        "app.motion.motion_bridge_heal.input_is_empty_but_a_later_one_is_wired_that_bra" => {
            "Input '{port}' is empty but a later one is wired — that branch reads 0"
        }
        "app.motion.motion_bridge_heal.another_already_drives_this_screen_pass_only_the" => {
            "Another '{ty}' already drives this screen pass — only the first one in the graph has any effect"
        }
        "app.motion.motion_bridge_heal.this_node_has_no_path_chosen_yet_so_it_passes_th" => {
            "This node has no path chosen yet, so it passes the layout straight through — pick one with 'Use Selected Path' or the Shape row"
        }
        "app.motion.motion_bridge_heal.this_node_needs_a_stream_wired_into_its_input" => {
            "This node needs a stream wired into its '{port}' input"
        }
        "app.motion.motion_bridge_heal.this_node_has_nothing_wired_into_it_so_it_has_no" => {
            "This node has nothing wired into it, so it has no data to work on"
        }
        "app.motion.motion_bridge_heal.this_node_only_gives_positions_add_a_duplicator" => {
            "This node only gives positions — on screen they are just dots. Add a Duplicator after it and wire a shape into the Duplicator to see objects"
        }
        "app.motion.motion_bridge_heal.this_node_has_no_points_to_work_on_wire_a_source" => {
            "This node has no points to work on — wire a source (Grid / Emitter) into it"
        }
        "app.motion.motion_bridge_heal.this_field_shapes_a_falloff_that_no_force_or_def" => {
            "This field shapes a falloff that no force or deformer downstream reads — add one after it"
        }
        "app.motion.motion_bridge_heal.this_constraint_needs_a_solver_integrate_sim_ste" => {
            "This constraint needs a solver (Integrate / Sim Step / Spring / Collide) downstream to have any effect"
        }
        "app.motion.motion_bridge_heal.wire_this_force_upstream_of_the_integrator_so_it" => {
            "Wire this force upstream of the integrator so it drives the motion"
        }
        "app.motion.motion_bridge_heal.wired_the_force_through_integrate_so_it_moves_th" => {
            "Wired the force through Integrate so it moves the points (undo to revert)"
        }
        "app.motion.motion_bridge_heal.nothing_upstream_carries_a_column_called_so_this" => {
            "Nothing upstream carries a column called '{column}', so this node reads zeros"
        }
        "app.motion.motion_bridge_heal.wired_the_forces_through_integrate_so_they_move" => {
            "Wired the forces through Integrate so they move the points (undo to revert)"
        }
        "app.motion.motion_bridge_intents.this_drawing_has_fewer_than_two_points_there_is" => {
            "This drawing has fewer than two points — there is no curve to follow"
        }
        "app.motion.motion_bridge_intents.give_this_drawing_a_name_in_the_hierarchy_first" => {
            "Give this drawing a name in the Hierarchy first"
        }
        "app.motion.motion_bridge_intents.the_selected_object_is_not_a_drawing_pick_a_path" => {
            "The selected object is not a drawing — pick a path"
        }
        "app.motion.motion_bridge_intents.select_a_drawing_first_on_the_canvas_or_in_the_h" => {
            "Select a drawing first — on the canvas or in the Hierarchy"
        }
        "app.motion.motion_bridge_intents.path_set_to" => "Path set to '{nome}'",
        "app.motion.motion_bridge_params_edit.unlinked_the_preset_needs_its_own" => {
            "Unlinked {join} - the preset needs its own"
        }
        "app.motion.motion_bridge_params_edit.unlinked_this_shape_has_no_such_controls" => {
            "Unlinked {join} - this shape has no such controls"
        }
        "app.motion.motion_bridge_params_edit.unlinked_this_shape_has_no_such_control" => {
            "Unlinked {labels} - this shape has no such control"
        }
        "app.motion.motion_bridge_params_file.table" => "Table",
        "app.motion.motion_bridge_params_file.audio" => "Audio",
        "app.motion.motion_bridge_params_text_rows.rule_problem" => {
            "\u{ab}{rule}\u{bb}: {say} (+{queixas} more)"
        }
        "app.motion.motion_bridge_readout.inst" => "{n} inst",
        "app.motion.motion_bridge_remove.that_node_lives_outside_this_group_leave_the_gro" => {
            "That node lives outside this group - leave the group to delete it"
        }
        "app.motion.motion_bridge_remove.state_wiring_is_automatic_disconnect_the_chain_f" => {
            "State wiring is automatic - disconnect the chain from the forces port instead"
        }
        "app.motion.motion_bridge_rewire.can_t_move_the_wire_there_the_original_stays" => {
            "Can't move the wire there - the original stays"
        }
        "app.motion.motion_bridge_rewire.state_wiring_is_automatic_disconnect_the_chain_f" => {
            "State wiring is automatic - disconnect the chain from the forces port instead"
        }
        "app.motion.motion_bridge_rewire.can_t_insert_this_node_into_this_wire" => {
            "Can't insert this node into this wire"
        }
        "app.motion.motion_bridge_rewire.can_t_reroute_this_wire" => "Can't reroute this wire",
        "app.motion.motion_bridge_rewire.no_reroute_exists_for_this_kind_of_wire" => {
            "No reroute exists for this kind of wire"
        }
        "app.motion.motion_bridge_subgraph.name" => "Name",
        "app.motion.motion_bridge_subgraph.group_nodes" => "Group ({inside} nodes)",
        "app.motion.motion_bridge_subgraph.can_t_drive_that_would_make_a_loop" => {
            "Can't drive: that would make a loop"
        }
        "app.motion.motion_bridge_subgraph.can_t_drive_that_output_is_a_per_element_stream" => {
            "Can't drive: that output is a per-element stream, not a value{cura}"
        }
        "app.motion.motion_bridge_subgraph.insert_a_to_read_one_number_from_it" => {
            " — insert a `{ty}` to read one number from it"
        }
        // ph2d-migrar-texto:end
        _ => return None,
    })
}
