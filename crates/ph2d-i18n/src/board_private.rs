//! **O QUADRO — o que é de uma pessoa** (MiroClone W12, ADR-0182): o modo privado e a votação entre
//! pessoas numa sessão ao vivo — o painel, o selo das notas escondidas e os avisos curtos.

/// A tradução de uma chave do modo privado e da votação ao vivo, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        // ── os avisos curtos (a família mostra-os) ──
        "board.private.only_starter" => {
            "Only the person who turned on private mode (or the host) can do that"
        }
        "board.private.reveal_mine_off" => "Revealing your own notes isn't turned on",
        "board.livevote.running" => "A vote is already running",
        "board.livevote.only_starter" => {
            "Only the person who started the vote (or the host) can end it"
        }
        "board.private.on" => "{name} turned on private mode — new notes are hidden from others",
        "board.private.revealed" => "Private mode ended — every note is revealed",
        "board.private.someone" => "Someone",
        "board.livevote.started" => "{name} started a vote — {n} votes each",
        // ── a barra e o painel do modo privado ──
        "board.fac.private" => "Private mode — write notes nobody else reads until they're revealed",
        "board.private.title" => "Private mode",
        "board.private.needs_live" => "Private mode works in a live session.",
        "board.private.needs_live_how" => "Open Live (bottom right), then come back here.",
        "board.private.explain" => "Text on sticky notes is hidden from others",
        "board.private.explain_2" => "until private mode is turned off.",
        "board.private.anonymous" => "Make names anonymous",
        "board.private.anonymous_tip" => {
            "Hide everyone's name and colour on cursors while private mode is on"
        }
        "board.private.anyone_end" => "Allow anyone to turn off",
        "board.private.anyone_end_tip" => "Let anyone in the session reveal the notes",
        "board.private.turn_on" => "Turn on private mode for all",
        "board.private.started_by" => "{name} turned on private mode",
        "board.private.started_by_you" => "You turned on private mode",
        "board.private.hidden_count" => "Hidden notes: {n}",
        "board.private.anyone_reveal" => "Anyone can reveal their stickies",
        "board.private.anyone_reveal_tip" => {
            "Let each person reveal their own notes before the end"
        }
        "board.private.reveal_mine" => "Reveal all my sticky notes",
        "board.private.reveal_mine_tip" => "Show your own hidden notes to everyone now",
        "board.private.turn_off" => "Turn off private mode for all",
        "board.private.turn_off_tip" => "Reveal every hidden note after a 5-second countdown",
        "board.private.revealing" => "Revealing in {s} s…",
        // ── a cena 13 (PH2D_BOARD_SMOKE=13) ──
        "board.smoke.faclive.title" => {
            "Private mode and voting: each person keeps their notes and votes until they're revealed"
        }
        "board.smoke.faclive.board" => "Private retro",
        "board.smoke.faclive.ideas" => "Ideas",
        "board.smoke.faclive.private" => "Write privately",
        "board.smoke.faclive.n1" => "Shorter meetings",
        "board.smoke.faclive.n2" => "Demo every Friday",
        "board.smoke.faclive.n3" => "Pair on reviews",
        "board.smoke.faclive.n4" => "Fewer tools",
        "board.smoke.faclive.l1" => {
            "1. In a 2nd PH2D window: press Live, click Private retro in the list, type the code, Join."
        }
        "board.smoke.faclive.l2" => {
            "2. Private mode panel (top right): Turn on private mode for all. New notes hide their text."
        }
        "board.smoke.faclive.l3" => {
            "3. Each window writes a note in Write privately: the other sees squiggles and a closed eye."
        }
        "board.smoke.faclive.l4" => {
            "4. Turn off private mode for all: after a 5-second countdown every note shows its text."
        }
        "board.smoke.faclive.l5" => {
            "5. Select the Ideas frame, press Voting (left bar), Start voting. Vote with + in each window."
        }
        "board.smoke.faclive.l6" => {
            "6. Nobody sees the others' votes. I'm done in both, then End for all: the totals appear."
        }
        // ── a votação ao vivo (o painel da votação) ──
        "board.livevote.done_count" => "{done} of {people} done",
        "board.livevote.done_tip" => "I'm done — let the others know (you can still change votes)",
        "board.livevote.waiting" => "Done — waiting for the vote to end",
        "board.livevote.person" => "{name}: {n} votes",
        "board.livevote.person_done" => "{name}: {n} votes · done",
        _ => return None,
    })
}
