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
        "board.live.link" => "Link",
        // W23 — os cartões do painel, por caminho de ligação.
        "board.live.sec_network" => "Local network",
        "board.live.sec_internet" => "Internet",
        "board.live.join" => "Join",
        "board.live.join_tip" => "Join the board with this code",
        // ADR-0187 — pela internet.
        "board.live.invite_net" => "Invite over the internet",
        "board.live.invite_net_tip" => {
            "Make a link that people in other houses can use to join. It is copied for you to send"
        }
        "board.live.invite_net_busy" => "Preparing the internet link…",
        "board.live.invite_net_ready" => "Link copied. Send it to whoever will join",
        "board.live.invite_net_again" => "Copy the link again",
        "board.live.invite_net_copied" => "Internet link copied",
        "board.live.invite_net_failed" => "Couldn't open to the internet: {error}",
        "board.live.link_hint" => "Got a link? Paste it here",
        "board.live.join_link" => "Join by link",
        "board.live.join_link_tip" => "Join with the link you pasted in the Link field",
        "board.live.link_bad" => "That isn't a PH2D board link",
        "board.live.link_unreachable" => {
            "Couldn't find the board of that link. Ask the host to keep the board open in Live, \
             then click Join by link again"
        }
        "board.live.link_unreachable_strict" => {
            "Couldn't find the board of that link. Your internet connection may be blocking it: \
             try another one, like your phone's hotspot, then click Join by link again"
        }
        "board.live.link_closed" => {
            "The board of that link isn't open anymore. Ask the host to invite you again and \
             paste the new link"
        }
        "board.live.link_old" => {
            "That link is from an older invitation. Ask the host for the new link"
        }
        "board.live.link_offline" => {
            "This computer isn't connected to the internet. Check the connection, then click \
             Join by link again"
        }
        "board.live.link_searching" => "Looking for the board on the internet…",
        "board.live.link_searching_hint" => {
            "This can take up to {s} seconds while the two computers find each other"
        }
        "board.live.nat_strict_host" => {
            "This internet connection may keep people in other houses out. If they can't join, \
             host from another connection, like a phone's hotspot"
        }
        "board.live.nat_strict_guest" => {
            "Your internet connection may keep you out. If it fails, try another connection, \
             like your phone's hotspot"
        }
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
        "board.live.everyone_tip" => "Everyone on this board",
        "board.live.more_people_tip" => "Show the others",
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
        "board.live.fw_blocked" => {
            "The firewall is on: other computers may not find or reach boards here"
        }
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
            "The firewall may block other computers — press «Allow on home network» in Live"
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
        // ── a cena 12 (PH2D_BOARD_SMOKE=12) ──
        "board.smoke.live.title" => {
            "Live: draw on this board together with another window or computer"
        }
        "board.smoke.live.board" => "Retro",
        "board.smoke.live.frame" => "Retro",
        "board.smoke.live.n_well" => "Went well: the demo",
        "board.smoke.live.n_better" => "Better: shorter meetings",
        "board.smoke.live.n_idea" => "Idea: a shared board",
        "board.smoke.live.plan" => "Plan",
        "board.smoke.live.draw" => "Draw together",
        "board.smoke.live.share" => "Share",
        "board.smoke.live.l1" => {
            "1. This board is already open: the Live panel (button Live, bottom right) shows its code."
        }
        "board.smoke.live.l2" => {
            "2. If the panel says the firewall blocks: Allow on home network, then your system password."
        }
        "board.smoke.live.l3" => {
            "3. In a 2nd PH2D window: press Live, click Retro in the list, type the code, then Join."
        }
        "board.smoke.live.l4" => {
            "4. Draw or move a note in one window: it moves in the other. You see their arrow and name."
        }
        "board.smoke.live.l5" => {
            "5. Click a person's circle next to Live to follow their view. The × on Following stops."
        }
        "board.smoke.live.l6" => {
            "6. Bring everyone here: all views jump to yours. End session: the others keep a copy."
        }
        // ── a cena 22 (PH2D_BOARD_SMOKE=22, ADR-0187): pela internet ──
        "board.smoke.net.title" => {
            "Live over the internet: invite someone in another house with a link"
        }
        "board.smoke.net.board" => "Retro online",
        "board.smoke.net.l1" => {
            "1. This board is open and the Live panel already made the internet link: it is copied."
        }
        "board.smoke.net.l2" => {
            "2. Send that link (chat or e-mail). It is the key of this room: whoever has it can join."
        }
        "board.smoke.net.l3" => {
            "3. In the other PH2D: press Live, paste the link in the Link field, click Join by link."
        }
        "board.smoke.net.l4" => {
            "4. It can take up to 30 seconds: the two houses find each other, then the board arrives."
        }
        "board.smoke.net.l5" => {
            "5. Draw together just like at home: notes, shapes, arrows, following, bringing everyone."
        }
        "board.smoke.net.l6" => {
            "6. End session closes it for all. The next invitation makes a new link; this one stops."
        }
        // ── a cena 24 (PH2D_BOARD_SMOKE=24, W23 bloco 3): o quadro ──
        "board.smoke.w23.board" => "Board tweaks",
        "board.smoke.w23.l1" => {
            "1. Hover a node (nothing selected): + adds a child, − on its edge folds the branch."
        }
        "board.smoke.w23.l2" => {
            "2. Select the ROOT: the letter sizes in its bar grow or shrink the whole map."
        }
        "board.smoke.w23.l3" => {
            "3. This board is in Draft: the branches are hand drawn, like the nodes."
        }
        "board.smoke.w23.l4" => {
            "4. The arrow between the two boxes below is selected: in its bar, the … next to \
             each tip opens all 15 tips (UML, ER)."
        }
        "board.smoke.w23.from" => "From",
        // ── a cena 25 (PH2D_BOARD_SMOKE=25, W23 B5(1)): o segmento do meio do cotovelo ──
        "board.smoke.elbow.board" => "Elbow arrow",
        "board.smoke.elbow.start" => "Start",
        "board.smoke.elbow.finish" => "Finish",
        "board.smoke.elbow.l1" => {
            "1. The arrow is selected. Drag the small bar in the middle of its upright line \
             sideways: that line moves, the ends stay put."
        }
        "board.smoke.elbow.l2" => {
            "2. Drag the Finish box further right: the middle line keeps its place between the \
             two boxes and moves with them."
        }
        "board.smoke.elbow.l3" => {
            "3. Double-click the small bar: the line goes back to the middle."
        }
        "board.smoke.elbow.l4" => "4. Ctrl+Z undoes each drag in one step.",
        "board.smoke.w23.to" => "To",
        "board.smoke.w23.l5" => {
            "5. Right-click a shape: Copy, Paste, Duplicate, Front/Back, Lock, Delete."
        }
        "board.smoke.w23.l6" => {
            "6. Trackpad: two fingers move the board, pinch zooms. In the row at the very top \
             (Scene, Board tweaks, +), press + until ‹ › appear: they step between the boards."
        }
        // ── a cena 23 (PH2D_BOARD_SMOKE=23, W23): as duas maneiras de entrar ──
        "board.smoke.join.title" => {
            "Live: two ways in — the same Wi-Fi, or a link over the internet"
        }
        "board.smoke.join.board" => "Join a board",
        "board.smoke.join.l1" => {
            "1. The Live panel has one card per way of connecting: Local network and Internet."
        }
        "board.smoke.join.l2" => {
            "2. Same Wi-Fi: click the board in the Local network list, type its code, then Join."
        }
        "board.smoke.join.l3" => {
            "3. Got a link from another house: paste it in Link (Internet card), then Join by link."
        }
        "board.smoke.join.l4" => {
            "4. To invite: Open to the network (same Wi-Fi) or Invite over the internet (link copied)."
        }
        "board.smoke.join.l5" => {
            "5. Inside a session the card says the way you came in, then the People card."
        }
        "board.smoke.join.l6" => {
            "6. Bring everyone here and Leave or End session sit at the bottom, outside the cards."
        }
        _ => return None,
    })
}
