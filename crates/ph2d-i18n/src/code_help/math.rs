//! O texto da página `math` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/math.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.math" => "Numbers: rounding, limits, angles, curves and randomness.",
        "code.help.math.about" => {
            "Every angle here is in radians, not degrees: a full turn is [math.tau] (about 6.28) and half a turn is [math.pi]. Convert with [math.rad] (degrees to radians) and [math.deg] (radians to degrees).\n\nSome results are not ordinary numbers: `inf` (infinity, for example `1 / 0`) and `nan` (\"not a number\", for example `0 / 0` or the square root of a negative). Check for them with [math.isinf], [math.isnan] and [math.isfinite]."
        }

        "code.help.math.abs" => {
            "Removes the minus sign: the distance of a number from zero.\n\nUse it for sizes and distances where the direction does not matter, like `math.abs(targetX - x)`."
        }
        "code.help.math.abs.n" => "Any number.",
        "code.help.math.abs.returns" => "The same number without its sign (never negative).",

        "code.help.math.acos" => {
            "The angle, in radians, whose cosine is the given number.\n\nNote: only numbers from -1 to 1 have an answer; anything outside gives `nan`."
        }
        "code.help.math.acos.n" => "A number from -1 to 1.",
        "code.help.math.acos.returns" => "An angle from 0 to [math.pi] radians.",

        "code.help.math.asin" => {
            "The angle, in radians, whose sine is the given number.\n\nNote: only numbers from -1 to 1 have an answer; anything outside gives `nan`."
        }
        "code.help.math.asin.n" => "A number from -1 to 1.",
        "code.help.math.asin.returns" => "An angle from -[math.pi]/2 to [math.pi]/2 radians.",

        "code.help.math.atan" => {
            "The angle, in radians, of a slope (how much it climbs for each step across).\n\nIt cannot tell left from right. To aim at a point, use [math.atan2], which can."
        }
        "code.help.math.atan.n" => "The slope: rise divided by run.",
        "code.help.math.atan.returns" => "An angle from -[math.pi]/2 to [math.pi]/2 radians.",

        "code.help.math.atan2" => {
            "The angle, in radians, of the direction from the origin to the point (`x`, `y`).\n\nThis is the one to aim with: pass the difference between the target and the object, `math.atan2(ty - y, tx - x)`, and you get the angle to rotate towards. Note the order: `y` comes first."
        }
        "code.help.math.atan2.y" => "How far the point is vertically.",
        "code.help.math.atan2.x" => "How far the point is horizontally.",
        "code.help.math.atan2.returns" => {
            "An angle from -[math.pi] to [math.pi] radians. 0 points right (positive `x`)."
        }

        "code.help.math.ceil" => {
            "Rounds up to the next whole number.\n\nNote: \"up\" means towards bigger numbers, so `-2.5` becomes `-2`. Use it to count how many boxes, pages or tiles you need to cover something."
        }
        "code.help.math.ceil.n" => "Any number.",
        "code.help.math.ceil.returns" => "The smallest whole number that is not below `n`.",

        "code.help.math.clamp" => {
            "Keeps a number inside a range: below `min` it gives `min`, above `max` it gives `max`.\n\nUse it for health, volume, positions inside the screen: anything that must not go past its limits. Note: if `min` is bigger than `max`, the script stops with an error."
        }
        "code.help.math.clamp.n" => "The number to keep in range.",
        "code.help.math.clamp.min" => "The lowest value allowed.",
        "code.help.math.clamp.max" => {
            "The highest value allowed. Must be greater than or equal to `min`."
        }
        "code.help.math.clamp.returns" => {
            "`n` if it was already inside, otherwise the limit it crossed."
        }

        "code.help.math.cos" => {
            "The cosine of an angle in radians: a smooth wave between -1 and 1 that starts at 1.\n\nTogether with [math.sin] it places points on a circle: `x = cx + math.cos(a) * radius`, `y = cy + math.sin(a) * radius`."
        }
        "code.help.math.cos.n" => "The angle, in radians.",
        "code.help.math.cos.returns" => "A number from -1 to 1.",

        "code.help.math.cosh" => {
            "The hyperbolic cosine: a U-shaped curve that is 1 at zero and grows fast on both sides.\n\nRarely needed in animation; the shape of a hanging rope or chain follows it."
        }
        "code.help.math.cosh.n" => "Any number.",
        "code.help.math.cosh.returns" => "A number that is 1 or more.",

        "code.help.math.deg" => {
            "Converts an angle from radians to degrees.\n\nUse it to show an angle to people, or when the rest of your scene works in degrees. The opposite is [math.rad]."
        }
        "code.help.math.deg.n" => "The angle, in radians.",
        "code.help.math.deg.returns" => "The same angle, in degrees.",

        "code.help.math.e" => {
            "Euler's number, about 2.718: the base of natural growth and of [math.exp] and [math.log]."
        }

        "code.help.math.exp" => {
            "Raises [math.e] to a power: a curve that grows faster and faster.\n\nUse `math.exp(-k * time)` for things that fade out smoothly, like a bounce that loses energy. The opposite is [math.log]."
        }
        "code.help.math.exp.n" => "The power.",
        "code.help.math.exp.returns" => "[math.e] to the power `n` (always above 0).",

        "code.help.math.floor" => {
            "Rounds down to the whole number below.\n\nNote: \"down\" means towards smaller numbers, so `-2.5` becomes `-3`. Use it for grid cells (`math.floor(x / tileSize)`) and for whole minutes in a timer."
        }
        "code.help.math.floor.n" => "Any number.",
        "code.help.math.floor.returns" => "The biggest whole number that is not above `n`.",

        "code.help.math.fmod" => {
            "The remainder of dividing `x` by `y`, keeping the sign of `x`.\n\nNote: with negative numbers this differs from the `%` operator. `math.fmod(-7, 3)` is `-1`, while `-7 % 3` is `2`. For wrapping around (looping frames, angles) `%` is usually what you want. Dividing by 0 gives `nan`, not an error."
        }
        "code.help.math.fmod.x" => "The number to divide.",
        "code.help.math.fmod.y" => "The number to divide by.",
        "code.help.math.fmod.returns" => "What is left over, with the sign of `x`.",

        "code.help.math.frexp" => {
            "Splits a number into a fraction and a power of two, so that `n = fraction * 2 ^ power`.\n\nA low-level tool; the opposite is [math.ldexp]. You will rarely need it in a scene."
        }
        "code.help.math.frexp.n" => "Any number.",
        "code.help.math.frexp.returns" => {
            "Two values: the fraction (from 0.5 up to, not including, 1, or 0 for zero) and the whole power of two."
        }

        "code.help.math.huge" => {
            "Infinity: a value bigger than any other number. `-math.huge` is smaller than any other.\n\nUse it as the starting point when looking for the smallest of something, like the nearest enemy."
        }

        "code.help.math.isfinite" => {
            "Tells whether a number is an ordinary number: not infinity and not `nan`.\n\nUse it to guard against a bad division before moving an object with the result."
        }
        "code.help.math.isfinite.x" => "The number to check.",
        "code.help.math.isfinite.returns" => {
            "`true` for an ordinary number, `false` for `inf`, `-inf` or `nan`."
        }

        "code.help.math.isinf" => {
            "Tells whether a number is infinity, positive or negative.\n\nA division by zero gives infinity instead of an error, so this is how you notice it."
        }
        "code.help.math.isinf.x" => "The number to check.",
        "code.help.math.isinf.returns" => "`true` for `inf` or `-inf`, otherwise `false`.",

        "code.help.math.isnan" => {
            "Tells whether a value is `nan` (\"not a number\").\n\n`nan` comes from sums with no answer, like `0 / 0`. Note: `nan` is never equal to anything, not even itself, so `x == math.nan` is always `false`; use this function instead."
        }
        "code.help.math.isnan.x" => "The number to check.",
        "code.help.math.isnan.returns" => "`true` if it is `nan`, otherwise `false`.",

        "code.help.math.ldexp" => {
            "Multiplies a number by two raised to a whole power: `s * 2 ^ e`.\n\nThe opposite of [math.frexp]; a low-level tool."
        }
        "code.help.math.ldexp.s" => "The number to scale.",
        "code.help.math.ldexp.e" => "The power of two (a whole number).",
        "code.help.math.ldexp.returns" => "`s` times 2 to the power `e`.",

        "code.help.math.lerp" => {
            "Finds the value part of the way from `a` to `b`: `t = 0` gives `a`, `t = 1` gives `b`, `t = 0.5` the middle.\n\nThe basic tool for in-betweens: move, fade or grow something by changing `t` from 0 to 1 over time. For easing, bend `t` first (for example `t * t` starts slowly).\n\nNote: `t` is not kept between 0 and 1. A `t` of 1.5 goes half again past `b`; wrap it in [math.clamp] if that must not happen."
        }
        "code.help.math.lerp.a" => "The value at the start (`t = 0`).",
        "code.help.math.lerp.b" => "The value at the end (`t = 1`).",
        "code.help.math.lerp.t" => "How far along, usually from 0 to 1.",
        "code.help.math.lerp.returns" => "The in-between value. With `t = 1` it is exactly `b`.",

        "code.help.math.log" => {
            "The logarithm: the power you must raise `base` to in order to get `n`.\n\nWithout `base` it uses [math.e]. Handy for things that feel even to the eye or ear, like zoom levels and volume. Note: `0` gives `-inf`, and negative numbers give `nan`."
        }
        "code.help.math.log.n" => "A number above 0.",
        "code.help.math.log.base" => "Optional. The base; leave it out for [math.e].",
        "code.help.math.log.returns" => "The power: `math.log(8, 2)` is 3 because 2 × 2 × 2 = 8.",

        "code.help.math.log10" => {
            "The logarithm in base 10: roughly, how many zeros the number has.\n\nAdd 1 to the [math.floor] of it to count the digits of a whole number, for example to pad a score."
        }
        "code.help.math.log10.n" => "A number above 0.",
        "code.help.math.log10.returns" => "The power of 10 that gives `n`: 2 for 100, 3 for 1000.",

        "code.help.math.map" => {
            "Converts a number from one range to another, keeping its place: halfway in stays halfway out.\n\nUse it to turn a slider from 0 to 1 into a colour from 0 to 255, or a mouse position into an angle. The output range may run backwards (`100` to `0`) to flip it.\n\nNote: the result is not kept inside the output range: 15 on a 0–10 range becomes 150 on 0–100. Use [math.clamp] on the result if needed. If `inmin` and `inmax` are equal, the answer is infinity or `nan`."
        }
        "code.help.math.map.x" => "The number to convert.",
        "code.help.math.map.inmin" => "The start of the range `x` comes from.",
        "code.help.math.map.inmax" => "The end of the range `x` comes from.",
        "code.help.math.map.outmin" => "What `inmin` turns into.",
        "code.help.math.map.outmax" => "What `inmax` turns into.",
        "code.help.math.map.returns" => "`x` moved into the new range (it can fall outside it).",

        "code.help.math.max" => {
            "The biggest of the numbers you give.\n\n`math.max(value, 0)` is a quick way to stop something going below zero. Needs at least one number; with none, the script stops with an error."
        }
        "code.help.math.max.x" => "The first number.",
        "code.help.math.max.vararg" => "Any number of further numbers to compare.",
        "code.help.math.max.returns" => "The largest of them.",

        "code.help.math.min" => {
            "The smallest of the numbers you give.\n\n`math.min(value, limit)` is a quick way to stop something going over a limit. Needs at least one number; with none, the script stops with an error."
        }
        "code.help.math.min.x" => "The first number.",
        "code.help.math.min.vararg" => "Any number of further numbers to compare.",
        "code.help.math.min.returns" => "The smallest of them.",

        "code.help.math.modf" => {
            "Splits a number into its whole part and the part after the point.\n\nBoth parts keep the sign: `-2.5` gives `-2` and `-0.5`. Note: the fraction can carry a tiny rounding error, like `0.7000000000000002` for `3.7`."
        }
        "code.help.math.modf.n" => "Any number.",
        "code.help.math.modf.returns" => {
            "Two values: the whole part (cut towards zero) and the fraction."
        }

        "code.help.math.nan" => {
            "The \"not a number\" value, the result of sums with no answer such as `0 / 0`.\n\nNote: it is never equal to anything, itself included. To test for it use [math.isnan]."
        }

        "code.help.math.noise" => {
            "Smooth random-looking value (Perlin noise): nearby inputs give nearby outputs, so it wobbles instead of jumping.\n\nFeed it a slowly growing time to shake a camera, flicker a flame or sway grass: `x = baseX + math.noise(time * 2) * 20`. The same input always gives the same output, so give each object its own offset (`math.noise(time, id)`) to make them differ.\n\nNote: it is always 0 when every input is a whole number, so step through it with fractions. The output stays roughly between -1 and 1; with only `x` it stays between -0.5 and 0.5."
        }
        "code.help.math.noise.x" => {
            "The position along the noise, usually time multiplied by a speed."
        }
        "code.help.math.noise.y" => {
            "Optional. A second coordinate, for 2D noise or to pick a different wobble per object."
        }
        "code.help.math.noise.z" => "Optional. A third coordinate, for 3D noise.",
        "code.help.math.noise.returns" => {
            "A number roughly from -1 to 1 (from -0.5 to 0.5 with only `x`)."
        }

        "code.help.math.phi" => {
            "The golden ratio, about 1.618.\n\nUseful for pleasing proportions in layouts and for spreading objects evenly around a circle."
        }

        "code.help.math.pi" => {
            "Pi, about 3.14159: half a turn in radians (180 degrees).\n\nA full turn is `2 * math.pi`, also available as [math.tau]."
        }

        "code.help.math.pow" => {
            "Raises `x` to the power `y`: `x` multiplied by itself `y` times.\n\nThe same as writing `x ^ y`. A power of 0.5 is the square root."
        }
        "code.help.math.pow.x" => "The base number.",
        "code.help.math.pow.y" => "The power; can be a fraction or negative.",
        "code.help.math.pow.returns" => "`x` to the power `y`.",

        "code.help.math.rad" => {
            "Converts an angle from degrees to radians.\n\nEvery angle function on this page ([math.sin], [math.cos]…) expects radians, so wrap degrees in this first. The opposite is [math.deg]."
        }
        "code.help.math.rad.n" => "The angle, in degrees.",
        "code.help.math.rad.returns" => "The same angle, in radians.",

        "code.help.math.random" => {
            "Gives a random number.\n\nWith no arguments: a fraction from 0 up to, but not including, 1. With one number `m`: a whole number from 1 to `m`, like a dice. With two, `m` and `n`: a whole number from `m` to `n`, both included.\n\nNote: fractional limits are cut down to whole numbers, and if the range is empty (like `math.random(5, 1)` or `math.random(0)`) the script stops with an error. Limits must stay within about ±2 billion. To get the same sequence each time, see [math.randomseed]."
        }
        "code.help.math.random.m" => {
            "Optional. Alone, the top of the range (from 1). With `n`, the bottom of the range."
        }
        "code.help.math.random.n" => "Optional. The top of the range.",
        "code.help.math.random.returns" => {
            "A fraction from 0 to under 1, or a whole number in the range asked for."
        }

        "code.help.math.randomseed" => {
            "Restarts [math.random] from a fixed starting point, so it gives the same sequence again.\n\nUse it when you want a \"random\" level, scatter or effect to come out identical every time you play."
        }
        "code.help.math.randomseed.seed" => {
            "Any number; the same seed always gives the same sequence."
        }

        "code.help.math.round" => {
            "Rounds to the nearest whole number.\n\nNote: halves go away from zero, so `2.5` becomes `3` and `-2.5` becomes `-3`. To keep two decimals, use `math.round(x * 100) / 100`."
        }
        "code.help.math.round.n" => "Any number.",
        "code.help.math.round.returns" => "The nearest whole number.",

        "code.help.math.sign" => {
            "Tells which side of zero a number is on: -1, 0 or 1.\n\nUse it to know which way something is moving, or to flip a sprite to face its direction."
        }
        "code.help.math.sign.n" => "Any number.",
        "code.help.math.sign.returns" => {
            "-1 for negative numbers, 1 for positive ones, 0 for zero."
        }

        "code.help.math.sin" => {
            "The sine of an angle in radians: a smooth wave between -1 and 1.\n\nUse it to make something bob up and down or pulse: `y = baseY + math.sin(time * speed) * height`. Swap in [math.noise] when the wave should look less regular."
        }
        "code.help.math.sin.n" => "The angle, in radians.",
        "code.help.math.sin.returns" => "A number from -1 to 1.",

        "code.help.math.sinh" => {
            "The hyperbolic sine: an S-shaped curve through zero that grows fast on both sides.\n\nRarely needed in animation."
        }
        "code.help.math.sinh.n" => "Any number.",
        "code.help.math.sinh.returns" => "A number with the same sign as `n`.",

        "code.help.math.sqrt" => {
            "The square root: the number that, multiplied by itself, gives `n`.\n\nWith Pythagoras it gives the distance between two points: `math.sqrt(dx * dx + dy * dy)`. Note: a negative number gives `nan`."
        }
        "code.help.math.sqrt.n" => "A number that is 0 or more.",
        "code.help.math.sqrt.returns" => "The square root of `n`.",

        "code.help.math.sqrt2" => {
            "The square root of 2, about 1.414: the length of the diagonal of a square with sides of 1.\n\nHandy to keep diagonal movement as fast as straight movement."
        }

        "code.help.math.tan" => {
            "The tangent of an angle in radians: how steep a slope at that angle is.\n\nNote: it shoots off towards infinity near a quarter turn ([math.pi]/2). The opposite is [math.atan]."
        }
        "code.help.math.tan.n" => "The angle, in radians.",
        "code.help.math.tan.returns" => "The slope: rise divided by run.",

        "code.help.math.tanh" => {
            "The hyperbolic tangent: an S-shaped curve that eases from -1 to 1.\n\nUse it to squash a value that can grow without end into a soft limit, like a smooth speed cap."
        }
        "code.help.math.tanh.n" => "Any number.",
        "code.help.math.tanh.returns" => "A number between -1 and 1.",

        "code.help.math.tau" => {
            "Tau, about 6.283: a full turn in radians (360 degrees), the same as `2 * math.pi`.\n\nUse it to spin something once per second: `angle = time * math.tau`."
        }
        _ => return None,
    })
}
