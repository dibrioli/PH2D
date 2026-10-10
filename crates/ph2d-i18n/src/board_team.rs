//! **O QUADRO — a equipa e a saída** (MiroClone W8): comentários, versões, apresentação, exportar e o
//! `.excalidraw` (`docs/MiroClone/06_equipa_e_saida.md`). As palavras do Miro onde ele as tem («Show
//! resolved», «Restore», «Hide frame»).

/// A tradução de uma chave da equipa do quadro, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.tool.comment" => "Comment (C) — click the board or an object to start a thread",
        "board.nav.present" => "Present — the frames as slides",
        "board.nav.comments" => "Comments",
        "board.menu.present" => "Present",
        "board.menu.versions" => "Versions…",
        "board.menu.export" => "Export…",
        "board.menu.import" => "Open an Excalidraw file…",
        "board.comment.resolve" => "Resolve (hide it from the board) or reopen",
        "board.comment.close" => "Close",
        "board.comment.show_resolved" => "Show resolved",
        "board.comment.author_tip" => "Your name on the comments you write (kept on this computer)",
        "board.comment.open" => "Open the thread",
        "board.comment.resolved" => "Resolved",
        "board.comment.thread" => "Comments",
        "board.comment.new" => "New comment",
        "board.comment.edit" => "Edit",
        "board.comment.delete" => "Delete (the first one deletes the thread)",
        "board.comment.edited" => "(edited)",
        "board.comment.reply" => "Reply…",
        "board.comment.write" => "Write a comment and press Enter",
        "board.comment.me" => "Me",
        "board.comment.title" => "Comments",
        "board.comment.name" => "Name: {name}",
        "board.comment.none" => "No comments yet",
        "board.team.close" => "Close",
        "board.team.surface" => "Comments, versions and export",
        "board.version.save" => "Save a named version of this board",
        "board.version.title" => "Versions",
        "board.version.default" => "Version {n}",
        "board.version.save_short" => "Save",
        "board.version.rename" => "Rename",
        "board.version.hint" => "Restore opens the version as a new board",
        "board.version.none" => "No versions yet",
        "board.version.rename_tip" => "Double-click to rename",
        "board.version.restore" => "Restore",
        "board.version.restore_tip" => "Open this version as a new board (this one stays as it is)",
        "board.version.delete" => "Delete this version",
        "board.version.saved" => "Version saved",
        "board.version.restored" => "Version restored as a new board",
        "board.version.unreadable" => "This version can't be read by this build",
        "board.export.title" => "Export",
        "board.export.png" => "PNG image",
        "board.export.svg" => "SVG image (vector)",
        "board.export.pdf" => "PDF (one page per frame)",
        "board.export.excalidraw" => "Excalidraw file",
        "board.export.excalidraw_short" => "Excalidraw (.excalidraw)",
        "board.export.scope_board" => "Board",
        "board.export.scope_selection" => "Selection",
        "board.export.scope_frames" => "Frames",
        "board.export.small" => "Small",
        "board.export.medium" => "Medium",
        "board.export.large" => "Large",
        "board.export.px" => "{w} × {h} px",
        "board.export.transparent" => "Transparent background",
        "board.export.go" => "Export…",
        "board.present.prev" => "Previous frame",
        "board.present.next" => "Next frame",
        "board.present.exit" => "Stop presenting (Esc)",
        "board.present.no_frames" => "Add a frame to present",
        "board.present.hide_frame" => "Hide from presentation",
        "board.present.show_frame" => "Show in presentation",
        "board.ago.now" => "now",
        "board.ago.min" => "{n} min ago",
        "board.ago.hour" => "{n} h ago",
        "board.ago.day" => "{n} d ago",
        "board.io.exported" => "Exported to {file}",
        "board.io.export_failed" => "Could not export: {error}",
        "board.io.nothing" => "Nothing to export",
        "board.io.imported" => "Opened {name}: {n} elements",
        "board.io.skipped" => "Skipped (boards don't have these yet): {list}",
        "board.io.import_failed" => "Could not open: {error}",
        "board.io.filter_png" => "PNG image",
        "board.io.filter_svg" => "SVG image",
        "board.io.filter_pdf" => "PDF document",
        "board.io.filter_excalidraw" => "Excalidraw drawing",
        "board.smoke.team.title" => "Team review: comments, versions, presenting and export",
        "board.smoke.team.problem" => "Problem",
        "board.smoke.team.ideas" => "Ideas",
        "board.smoke.team.plan" => "Plan",
        "board.smoke.team.n_builds" => "Builds take 20 minutes",
        "board.smoke.team.n_reviews" => "Reviews wait two days",
        "board.smoke.team.n_cache" => "Cache the build",
        "board.smoke.team.n_pair" => "Pair on reviews",
        "board.smoke.team.n_small" => "Smaller pull requests",
        "board.smoke.team.measure" => "Measure",
        "board.smoke.team.cache" => "Cache",
        "board.smoke.team.ship" => "Ship",
        "board.smoke.team.version" => "Kick-off",
        "board.smoke.team.c_rui" => "Is this the CI or local builds?",
        "board.smoke.team.c_ana" => "Approved for the next sprint",
        "board.smoke.team.l1" => {
            "1. Comment: press C, click a note, write, Enter. Click Rui's blue bubble to read and reply."
        }
        "board.smoke.team.l2" => {
            "2. Versions: right-click this tab, Versions…, Save. Restore opens one as a new board."
        }
        "board.smoke.team.l3" => {
            "3. Present: the Present button at the bottom right. Arrows move, Esc stops."
        }
        "board.smoke.team.l4" => {
            "4. Export: right-click the tab, Export…, pick PNG, SVG or PDF, then Export…"
        }
        "board.smoke.team.l5" => {
            "5. Excalidraw: right-click the tab, Open an Excalidraw file…, pick example.excalidraw"
        }
        _ => return None,
    })
}
