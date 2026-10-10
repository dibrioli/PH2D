//! **O QUADRO AO VIVO** (MiroClone W11, `docs/MiroClone/08_ao_vivo.md`): o painel «Live», os avatares,
//! seguir alguém, e os avisos curtos que a família do quadro (`ph2d-app-board::live`) mostra.

/// A tradução de uma chave do quadro ao vivo, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ── o botão e o painel ──
        "board.live.button" => "Live",
        "board.live.tip" => "Live — draw on this board together with people on the same Wi-Fi",
        "board.live.close" => "Close",
        "board.live.surface" => "Live session",
        "board.live.open" => "Open to the network",
        "board.live.open_tip" => "Let people on the same Wi-Fi join this board with a code",
        "board.live.join_title" => "Join a board",
        "board.live.none_found" => "No open boards found on this network yet",
        "board.live.found_row" => "{board} — {host}",
        "board.live.found_tip" => "Pick this board (then type its code)",
        "board.live.code" => "Code",
        "board.live.address" => "Address (optional)",
        "board.live.join" => "Join",
        "board.live.join_tip" => "Join the board with this code",
        "board.live.code_short" => "The code has 6 letters and numbers",
        "board.live.connecting" => "Connecting…",
        "board.live.cancel" => "Cancel",
        "board.live.share" => "People on the same Wi-Fi: open Live and type this code",
        "board.live.people" => "People",
        "board.live.you" => "{name} (you)",
        "board.live.summon" => "Bring everyone here",
        "board.live.summon_tip" => "Moves everyone's view to what you see now",
        "board.live.end" => "End session",
        "board.live.leave" => "Leave",
        "board.live.in_board" => "In {host}'s board",
        "board.live.follow_tip" => "Follow {name} (your view goes where theirs goes)",
        "board.live.unfollow_tip" => "Stop following",
        "board.live.following" => "Following {name}",
        "board.live.more" => "+{n}",
        // ── os avisos curtos (a família mostra-os) ──
        "board.live.joined" => "{name} joined",
        "board.live.left" => "{name} left",
        "board.live.wrong_code" => "Wrong code — check it with the host",
        "board.live.unreachable" => "Couldn't reach that board",
        "board.live.host_left" => "The host ended the live session — this copy stays here",
        "board.live.summoned" => "{name} brought everyone here",
        "board.live.refused" => "That board is open in another version of PH2D",
        "board.live.opened" => "Live: share the code {code}",
        "board.live.firewall" => {
            "Others can't reach this computer — the firewall may be blocking port {port}"
        }
        // ── a firewall (o botão pede a senha do sistema) ──
        "board.live.fw_blocked" => "The firewall stops other computers from reaching your boards",
        "board.live.fw_allow" => "Allow on home network",
        "board.live.fw_allow_tip" => {
            "Let computers on this network reach your live boards (asks for the system password)"
        }
        "board.live.fw_remove" => "Remove the permission",
        "board.live.fw_remove_tip" => {
            "Undo what PH2D allowed in the firewall (asks for the system password)"
        }
        "board.live.fw_busy" => "Waiting for the system password…",
        "board.live.fw_hint" => {
            "Other computers can't reach this one yet — press «Allow on home network» in Live"
        }
        "board.live.fw_allowed" => "Computers on this network can now reach your live boards",
        "board.live.fw_removed" => "Permission removed — the firewall blocks other computers again",
        "board.live.fw_cancelled" => "Nothing changed — the password wasn't given",
        "board.live.fw_failed" => "Couldn't change the firewall: {error}",
        "board.live.fw_unsaved" => {
            "PH2D couldn't note what it changed ({error}) — the Remove button won't come back"
        }
        "board.live.reconnecting" => "Connection lost — reconnecting…",
        "board.live.reconnected" => "Back in the live session",
        "board.live.locked" => "Too many wrong codes — try again in {s} s",
        _ => return None,
    })
}
