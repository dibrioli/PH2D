//! **O QUADRO — facilitar** (MiroClone W7): o cronómetro, a votação, os carimbos e agrupar notas
//! (`docs/MiroClone/05_facilitacao.md`). As palavras do Miro onde ele as tem («I'm done», «End for all»,
//! «no tag», «#uncategorized»).

/// A tradução de uma chave da facilitação do quadro, ou `None` se ela não é daqui.
pub(crate) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "board.tool.timer" => "Timer",
        "board.tool.vote" => "Voting",
        "board.fac.stamp" => "Stamp — add a reaction to the selected notes",
        "board.fac.cluster" => "Cluster — group the selected notes into frames",
        "board.fac.cluster.color" => "By color",
        "board.fac.cluster.tag" => "By tag",
        "board.fac.cluster.keyword" => "By keyword",
        "board.fac.cluster.no_tag" => "no tag",
        "board.fac.cluster.uncategorized" => "#uncategorized",
        "board.stamp.thumbs_up" => {
            "Thumbs up — click the stamp under a note to add one, Shift+click to take one back"
        }
        "board.stamp.thumbs_down" => "Thumbs down",
        "board.stamp.heart" => "Heart",
        "board.stamp.star" => "Star",
        "board.stamp.flame" => "Fire",
        "board.stamp.lightbulb" => "Idea",
        "board.stamp.question" => "Question",
        "board.stamp.party" => "Celebrate",
        "board.timer.minus" => "Less time",
        "board.timer.plus" => "More time",
        "board.timer.start" => "Start",
        "board.timer.pause" => "Pause",
        "board.timer.stop" => "Stop and reset",
        "board.timer.add1" => "+1m",
        "board.timer.add5" => "+5m",
        "board.timer.add1_tip" => "Add 1 minute",
        "board.timer.add5_tip" => "Add 5 minutes",
        "board.timer.sound" => "Chime at the end — on or off",
        "board.timer.minimize" => "Minimize",
        "board.timer.expand" => "Show the timer",
        "board.timer.close" => "Close the timer",
        "board.timer.stop_confirm" => "Stop timer",
        "board.timer.cancel" => "Cancel",
        "board.vote.title" => "Voting",
        "board.vote.results" => "Results",
        "board.vote.votes" => "Votes per person",
        "board.vote.one_per" => "One vote per object",
        "board.vote.minutes" => "Minutes",
        "board.vote.no_clock" => "Off",
        "board.vote.start" => "Start voting",
        "board.vote.left" => "{n} votes left",
        "board.vote.voted" => "{n} voted",
        "board.vote.done" => "I'm done",
        "board.vote.done_tip" => "I'm done — hand the screen to the next person",
        "board.vote.end" => "End for all",
        "board.vote.add1" => "+1 min",
        "board.vote.one_vote" => "1 vote",
        "board.vote.n_votes" => "{n} votes",
        "board.vote.untitled" => "(no text)",
        "board.vote.see_all" => "See all results",
        "board.vote.top3" => "Top 3",
        "board.vote.clear" => "Clear results",
        "board.vote.close" => "Close",
        "board.notice.no_votes_left" => "No votes left — take one back with − or press I'm done",
        "board.notice.one_vote_per_object" => "One vote per object — this one already has yours",
        "board.notice.nothing_to_vote" => "Nothing to vote on — votes go on notes and shapes",
        "board.notice.nothing_to_cluster" => "Select two or more notes to cluster them",
        "board.smoke.fac.frame" => "Facilitate a brainstorm",
        "board.smoke.fac.ideas_frame" => "Ideas",
        "board.smoke.fac.tag_speed" => "Speed",
        "board.smoke.fac.tag_team" => "Team",
        "board.smoke.fac.i1" => "Faster builds",
        "board.smoke.fac.i2" => "Builds break at night",
        "board.smoke.fac.i3" => "Slow builds block the team",
        "board.smoke.fac.i4" => "More pair programming",
        "board.smoke.fac.i5" => "Pairing on hard bugs",
        "board.smoke.fac.i6" => "Team lunch on Fridays",
        "board.smoke.fac.i7" => "Coffee machine upstairs",
        "board.smoke.fac.l1" => {
            "1. Timer — the card at the top right. Set the minutes with − and +, then Play. +1m and +5m add time; at zero it chimes."
        }
        "board.smoke.fac.l2" => {
            "2. Vote — press Voting in the left bar, choose the votes per person, then Start voting. Click the grey + at a note's corner; − takes one back."
        }
        "board.smoke.fac.l3" => {
            "3. Turns — I'm done hands the screen to the next person (their votes start at zero). End for all puts the totals on the notes."
        }
        "board.smoke.fac.l4" => {
            "4. Stamps — select a note and press Stamp in its bar. Click a stamp under a note to add one; Shift+click takes one back."
        }
        "board.smoke.fac.l5" => {
            "5. Cluster — click the Ideas frame, then Cluster in its bar: By color, By tag or By keyword. Each group becomes a frame."
        }
        _ => return None,
    })
}
