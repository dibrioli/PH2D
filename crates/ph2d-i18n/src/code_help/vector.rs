//! O texto da página `vector` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/vector.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.vector" => {
            "Three-number vectors (x, y, z) and the maths to move, measure and blend them."
        }
        "code.help.vector.about" => {
            "A vector holds three numbers, `x`, `y` and `z`. Make one with [vector.create]; leave out `z` and it is 0, so 2D work can simply ignore it. Read the parts with `v.x`, `v.y` and `v.z`. A vector cannot be changed after it is made: `v.x = 5` stops the script, so build a new one instead. `print` writes a vector as its three numbers: `1, 2, 0`.\n\nVectors do arithmetic directly: `a + b`, `a - b`, `a * 2`, `a / 2` and `-a`, and `a * b` multiplies part by part. Each part is kept as a 32-bit number, so `0.1` comes back as `0.10000000149011612`.\n\nNote: PH2D's own positions and sizes are [ph2d.vec2] values, which are a different thing from these vectors. To use one here, copy its parts: `vector.create(p.x, p.y)`."
        }
        "code.help.vector.abs" => {
            "Makes every part of the vector positive.\n\nUseful for a size or distance per axis, when the direction does not matter."
        }
        "code.help.vector.abs.vec" => "The vector to read.",
        "code.help.vector.abs.returns" => "A new vector with no negative parts.",
        "code.help.vector.angle" => {
            "The angle between two directions, in radians.\n\nWithout `axis` the answer is always positive, from 0 to `math.pi`, so it cannot tell left from right. Give `axis` (for 2D, `vector.create(0, 0, 1)`) to get a signed angle, whose sign tells which way you turn from the first to the second."
        }
        "code.help.vector.angle.vec1" => "The first direction.",
        "code.help.vector.angle.vec2" => "The second direction.",
        "code.help.vector.angle.axis" => {
            "Optional. The axis to turn around; when given, the angle gets a sign."
        }
        "code.help.vector.angle.returns" => "The angle in radians.",
        "code.help.vector.ceil" => {
            "Rounds every part up to the next whole number.\n\nNote: \"up\" means towards positive, so `-1.7` becomes `-1`."
        }
        "code.help.vector.ceil.vec" => "The vector to round.",
        "code.help.vector.ceil.returns" => "A new vector with whole-number parts.",
        "code.help.vector.clamp" => {
            "Keeps every part between the matching parts of `min` and `max`.\n\nUse it to keep a position inside a box, such as the edges of the screen. Every part of `min` must be less than or equal to the same part of `max`, or the script stops with an error."
        }
        "code.help.vector.clamp.vec" => "The vector to limit.",
        "code.help.vector.clamp.min" => "The lowest allowed value for each part.",
        "code.help.vector.clamp.max" => "The highest allowed value for each part.",
        "code.help.vector.clamp.returns" => "A new vector inside the box.",
        "code.help.vector.create" => {
            "Makes a new vector from its parts.\n\nLeave out `z` for 2D work: it becomes 0."
        }
        "code.help.vector.create.x" => "The horizontal part.",
        "code.help.vector.create.y" => "The vertical part.",
        "code.help.vector.create.z" => "Optional. The depth part; 0 when left out.",
        "code.help.vector.create.returns" => "The new vector.",
        "code.help.vector.cross" => {
            "A vector at right angles to both of the given ones.\n\nMostly for 3D. With two flat (2D) vectors, only `z` of the answer is non-zero, and its sign tells which way one turns to reach the other."
        }
        "code.help.vector.cross.vec1" => "The first vector.",
        "code.help.vector.cross.vec2" => "The second vector.",
        "code.help.vector.cross.returns" => "A new vector at right angles to both.",
        "code.help.vector.dot" => {
            "Multiplies the matching parts and adds them up.\n\nA quick way to ask \"is this in front of me?\": with a facing direction, the answer is positive in front, 0 exactly to the side, and negative behind."
        }
        "code.help.vector.dot.vec1" => "The first vector.",
        "code.help.vector.dot.vec2" => "The second vector.",
        "code.help.vector.dot.returns" => "A single number.",
        "code.help.vector.floor" => {
            "Rounds every part down to the whole number below.\n\nNote: \"down\" means towards negative, so `-1.2` becomes `-2`. Handy for snapping a position to a grid cell."
        }
        "code.help.vector.floor.vec" => "The vector to round.",
        "code.help.vector.floor.returns" => "A new vector with whole-number parts.",
        "code.help.vector.lerp" => {
            "Blends between two vectors: `t = 0` gives the first, `t = 1` the second.\n\nUse it to slide something from one place to another over time. Values of `t` outside 0 to 1 are not limited: they go past either end."
        }
        "code.help.vector.lerp.vec1" => "Where the blend starts (`t = 0`).",
        "code.help.vector.lerp.vec2" => "Where the blend ends (`t = 1`).",
        "code.help.vector.lerp.t" => "How far along, usually from 0 to 1.",
        "code.help.vector.lerp.returns" => "The blended vector.",
        "code.help.vector.magnitude" => {
            "The length of the vector.\n\nTo get the distance between two points, take the length of their difference: `vector.magnitude(b - a)`."
        }
        "code.help.vector.magnitude.vec" => "The vector to measure.",
        "code.help.vector.magnitude.returns" => "The length, never negative.",
        "code.help.vector.max" => {
            "Takes the largest value of each part across all the vectors given.\n\nEach part is chosen on its own, so the answer can mix parts from different vectors."
        }
        "code.help.vector.max.vec" => "The first vector.",
        "code.help.vector.max.vararg" => "Any number of further vectors.",
        "code.help.vector.max.returns" => "A new vector of the largest parts.",
        "code.help.vector.min" => {
            "Takes the smallest value of each part across all the vectors given.\n\nEach part is chosen on its own, so the answer can mix parts from different vectors."
        }
        "code.help.vector.min.vec" => "The first vector.",
        "code.help.vector.min.vararg" => "Any number of further vectors.",
        "code.help.vector.min.returns" => "A new vector of the smallest parts.",
        "code.help.vector.normalize" => {
            "The same direction with a length of 1.\n\nUse it to turn \"from here to there\" into a pure direction, then multiply by a speed. Note: the zero vector has no direction, and gives `nan` in every part."
        }
        "code.help.vector.normalize.vec" => "The vector to shorten or stretch.",
        "code.help.vector.normalize.returns" => "A vector of length 1 pointing the same way.",
        "code.help.vector.one" => {
            "The vector with every part 1: `1, 1, 1`.\n\nMultiply it to get a vector with the same number in every part, such as a uniform scale."
        }
        "code.help.vector.sign" => {
            "Replaces every part with -1, 0 or 1, by whether it is negative, zero or positive.\n\nUseful to know which way something is moving on each axis."
        }
        "code.help.vector.sign.vec" => "The vector to read.",
        "code.help.vector.sign.returns" => "A new vector of -1, 0 and 1.",
        "code.help.vector.zero" => {
            "The vector with every part 0: `0, 0, 0`.\n\nA good starting value for a speed or an offset."
        }
        _ => return None,
    })
}
