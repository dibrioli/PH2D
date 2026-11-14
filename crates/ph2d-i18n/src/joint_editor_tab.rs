//! A ABA Joint Editor (`docs/JointEditor/01_plano.md` §W3, versão B): montar máquinas por gabarito.

pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "joint_editor.title" => "Joint Editor",
        "joint_editor.cmd.open" => "Joint Editor\u{2026}",
        "joint_editor.assemblies" => "Assemblies",
        "joint_editor.new_assembly" => "New assembly",
        "joint_editor.new" => "+ New",
        "joint_editor.remove" => "Remove",
        "joint_editor.joint" => "Joint",
        "joint_editor.joint_tip" => "The kind of joint. Each kind has its own templates.",
        "joint_editor.template" => "Template",
        "joint_editor.template_tip" => {
            "A ready-made machine for this joint. Drop your objects into it."
        }
        "joint_editor.templates_n" => "{n} templates",
        "joint_editor.scope.both_wheels" => "Both wheels",
        "joint_editor.scope.all" => "All {n}",
        "joint_editor.scene" => "The scene",
        "joint_editor.use_scene" => "Pin to the scene",
        "joint_editor.hint" => {
            "Drag objects from the Hierarchy into the dashed shapes. Each one becomes that part."
        }
        "joint_editor.hint_assembled" => {
            "Drag the round handles to change travel, limits and wheelbase. Click a part to select it. Drop more objects into empty shapes."
        }
        "joint_editor.drop.over_a_shape" => "Let go over one of the dashed shapes.",
        "joint_editor.drop.new_assembly" => "a new {template}",
        "joint_editor.drop.one" => {
            "Let go: {name} becomes the {role} of {assembly}. Its joint is made when the part it holds on to is in too."
        }
        "joint_editor.drop.many" => {
            "Let go: these {n} objects become the {role} parts of {assembly}, in this order."
        }
        "joint_editor.drop.no_body" => "{name} has no physics body. Give it a Physics Body first.",
        "joint_editor.drop.already" => "{name} is already the {role} here.",
        "joint_editor.drop.taken" => {
            "The {role} is already {name}. Remove the assembly to start again."
        }
        "joint_editor.placed" => {
            "{role} of {assembly}: {n} {n?part|parts} in, {joints} {joints?joint|joints} made. The objects stay where they are."
        }
        "joint_editor.shape_a11y" => "{role}: {name}",
        "joint_editor.handle.min" => "Drag to set the lowest point",
        "joint_editor.handle.max" => "Drag to set the highest point",
        "joint_editor.handle.wheelbase" => "Drag to move the front wheel",
        "joint_editor.wheelbase" => "Wheelbase",
        "joint_editor.wheelbase_m" => "Wheelbase {m} m",
        "joint_editor.with_unit" => "{label} ({unit})",
        "joint_editor.on" => "On",
        "joint_editor.group.suspension" => "Suspension",
        "joint_editor.group.spring" => "Spring",
        "joint_editor.group.length" => "Length",
        "joint_editor.group.breaks" => "Breaks",
        "joint_editor.group.assembly" => "Assembly",
        "joint_editor.group.applies_to" => "Applies to",
        "joint_editor.glyph.stiffness" => "S",
        "joint_editor.glyph.damping" => "D",
        "joint_editor.glyph.speed" => "V",
        "joint_editor.glyph.force" => "F",
        "joint_editor.glyph.torque" => "T",
        "joint_editor.glyph.wheelbase" => "W",
        "joint_editor.glyph.length" => "L",
        "joint_editor.unit.m" => "m",
        "joint_editor.unit.deg" => "\u{b0}",
        "joint_editor.unit.m_per_s" => "m/s",
        "joint_editor.unit.deg_per_s" => "\u{b0}/s",
        "joint_editor.unit.n" => "N",
        "joint_editor.unit.nm" => "N\u{b7}m",
        "joint_editor.drive.all" => "Both",
        "joint_editor.drive.none" => "None",
        "joint_editor.no_motor" => "This wheel has no motor. Pick it in Drive to make it turn.",
        "joint_editor.draft_title" => "New {template}",
        "joint_editor.empty_panel" => {
            "Drop a part into the template to make its joint. The settings show up here."
        }
        "joint_editor.parts_of" => "{name} \u{b7} {n} of {of} parts \u{b7} {joints} joints",
        "joint_editor.open_inspector" => "Open in Inspector",
        "joint_editor.no_joint_yet" => "No joint yet. Drop the part it holds on to as well.",
        "joint_editor.open_short" => "Inspector",
        "joint_editor.resize_panel" => "Drag to make the panel wider or narrower",
        "joint_editor.group.wheels" => "Wheels",
        "joint_editor.group.limits" => "Limits",
        "joint_editor.group.motor" => "Motor",
        "joint_editor.label.stiffness" => "Stiffness",
        "joint_editor.label.damping" => "Damping",
        "joint_editor.label.min_travel" => "Min travel",
        "joint_editor.label.max_travel" => "Max travel",
        "joint_editor.label.min_angle" => "Min angle",
        "joint_editor.label.max_angle" => "Max angle",
        "joint_editor.label.min_distance" => "Min distance",
        "joint_editor.label.max_distance" => "Max distance",
        "joint_editor.label.speed" => "Speed",
        "joint_editor.label.force" => "Force",
        "joint_editor.label.rest_length" => "Rest length",
        "joint_editor.label.max_length" => "Max length",
        "joint_editor.label.break_force" => "Break force",
        "joint_editor.label.break_torque" => "Break torque",
        "joint_editor.label.drive" => "Drive",
        "panel.inspector.joint.open_in_joint_editor" => "Open in Joint Editor",
        _ => return None,
    })
}
