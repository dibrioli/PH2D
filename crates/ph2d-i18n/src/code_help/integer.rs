//! O texto da página `integer` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/integer.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.integer" => {
            "Exact 64-bit whole numbers, kept as a type of their own beside ordinary numbers."
        }
        "code.help.integer.about" => {
            "You rarely need this page. An ordinary number already holds every whole number exactly up to 9007199254740992 (2^53), which covers scores, frame counts, timers and positions. Reach for `integer` when a value must stay exact beyond that: very big counters, long IDs that arrive as text, or packing several small values (flags, colour channels) into the bits of one value.\n\nAn integer is not a number: `+`, `-`, `*`, `<`, `..` and the `math` functions all refuse it, and `integer.create(5) == 5` is `false`. Do the sums with [integer.add], [integer.mul] and friends, compare with [integer.lt] and its sisters, and only `==` works directly between two integers. Make one with [integer.create] or [integer.fromstring]; turn it back with [integer.tonumber] or `tostring`.\n\nNote: the sums never fail on overflow — they wrap around, so one past [integer.maxsigned] is [integer.minsigned]. And `print` rounds an integer bigger than 2^53 (`print(integer.maxsigned)` shows `9223372036854776000`), so print `tostring(x)` or `string.format(\"%d\", x)` to see every digit."
        }

        "code.help.integer.add" => {
            "Adds two integers.\n\nIf the result goes past [integer.maxsigned] it wraps round to the negative end instead of failing."
        }
        "code.help.integer.add.x" => "The first integer.",
        "code.help.integer.add.y" => "The integer to add to it.",
        "code.help.integer.add.returns" => "The sum, wrapped round to 64 bits.",

        "code.help.integer.arshift" => {
            "Moves the bits to the right, keeping the sign: a negative value stays negative.\n\nEach step halves the value, rounding down: `-16` shifted by 2 gives `-4`. For a shift that fills with zeros instead, use [integer.rshift]."
        }
        "code.help.integer.arshift.x" => "The integer whose bits move.",
        "code.help.integer.arshift.numBitPositions" => {
            "How many places to move. 64 or more leaves only the sign: `0` or `-1`."
        }
        "code.help.integer.arshift.returns" => "The shifted integer.",

        "code.help.integer.band" => {
            "Keeps only the bits that are on in every value given.\n\nUse it with a mask to read one flag or a group of bits. With no values at all it gives `-1` (every bit on)."
        }
        "code.help.integer.band.vararg" => "The integers to combine; any number of them.",
        "code.help.integer.band.returns" => "An integer with the bits shared by all of them.",

        "code.help.integer.bnot" => {
            "Flips every bit: on becomes off and off becomes on.\n\nFor a signed value this is the same as `-x - 1`, so `0` becomes `-1`."
        }
        "code.help.integer.bnot.x" => "The integer to flip.",
        "code.help.integer.bnot.returns" => "The flipped integer.",

        "code.help.integer.bor" => {
            "Turns on every bit that is on in any of the values given.\n\nUse it to switch flags on, or to merge packed pieces. With no values it gives `0`."
        }
        "code.help.integer.bor.vararg" => "The integers to combine; any number of them.",
        "code.help.integer.bor.returns" => "An integer with all their bits together.",

        "code.help.integer.bswap" => {
            "Reverses the order of the eight bytes of the integer.\n\nOnly useful when reading or writing data made by another program that stores bytes the other way round. Swapping twice gives the original back."
        }
        "code.help.integer.bswap.x" => "The integer to reverse.",
        "code.help.integer.bswap.returns" => "The integer with its bytes in the opposite order.",

        "code.help.integer.btest" => {
            "Checks whether the values share at least one bit that is on.\n\nThe quick way to ask \"is this flag set?\". It gives `true` when called with no values."
        }
        "code.help.integer.btest.vararg" => "The integers to test; usually a value and a mask.",
        "code.help.integer.btest.returns" => {
            "`true` if some bit is on in all of them, otherwise `false`."
        }

        "code.help.integer.bxor" => {
            "Turns on the bits that are on in an odd number of the values.\n\nWith a mask it toggles flags: applying the same mask twice brings the value back. With no values it gives `0`."
        }
        "code.help.integer.bxor.vararg" => "The integers to combine; any number of them.",
        "code.help.integer.bxor.returns" => "The combined integer.",

        "code.help.integer.clamp" => {
            "Keeps an integer inside a range.\n\nBelow `min` gives `min`, above `max` gives `max`. It stops with an error if `min` is bigger than `max`."
        }
        "code.help.integer.clamp.value" => "The integer to limit.",
        "code.help.integer.clamp.min" => "The smallest value allowed.",
        "code.help.integer.clamp.max" => "The largest value allowed.",
        "code.help.integer.clamp.returns" => {
            "The integer, moved inside the range if it was outside."
        }

        "code.help.integer.countlz" => {
            "Counts the zero bits at the top of the integer, before the first bit that is on.\n\nA negative value gives `0`; zero gives `64`."
        }
        "code.help.integer.countlz.x" => "The integer to look at.",
        "code.help.integer.countlz.returns" => "A count from 0 to 64.",

        "code.help.integer.countrz" => {
            "Counts the zero bits at the bottom of the integer, before the first bit that is on.\n\nIt tells you how many times a value divides evenly by 2. Zero gives `64`."
        }
        "code.help.integer.countrz.x" => "The integer to look at.",
        "code.help.integer.countrz.returns" => "A count from 0 to 64.",

        "code.help.integer.create" => {
            "Turns an ordinary whole number into an integer.\n\nIt gives `nil` if the number has a fractional part, is not a number (NaN) or is too big for 64 bits. A text made of digits, like `\"5\"`, is accepted too.\n\nNote: numbers above 2^53 are already rounded before they reach this function. For an exact big value, start from text with [integer.fromstring]."
        }
        "code.help.integer.create.x" => {
            "A whole number, from -2^63 up to (but not including) 2^63."
        }
        "code.help.integer.create.returns" => {
            "The integer, or `nil` if the number cannot be one exactly."
        }

        "code.help.integer.div" => {
            "Divides two integers and drops the fraction, rounding towards zero.\n\nSo `-7` divided by `2` gives `-3`. Dividing by zero is an error, and so is [integer.minsigned] divided by `-1`. To round down instead, use [integer.idiv]."
        }
        "code.help.integer.div.x" => "The integer to divide.",
        "code.help.integer.div.y" => "What to divide by; must not be zero.",
        "code.help.integer.div.returns" => "The whole part of the quotient.",

        "code.help.integer.extract" => {
            "Reads a group of bits out of an integer, counting from bit 0 at the right.\n\nUse it to unpack values stored side by side, like the red, green and blue of a colour. Asking for bits outside 0–63 is an error."
        }
        "code.help.integer.extract.value" => "The integer to read from.",
        "code.help.integer.extract.bitPosition" => "The first (lowest) bit to read, from 0 to 63.",
        "code.help.integer.extract.numBits" => {
            "How many bits to read. Leave it out to read just one bit."
        }
        "code.help.integer.extract.returns" => "The bits read, as a small positive integer.",

        "code.help.integer.fromstring" => {
            "Reads an integer from text, exactly, with all its digits.\n\nIt gives `nil` if the text is not a whole number (`\"12.5\"`, `\"abc\"`, `\"\"`). Spaces around the digits and a `0x` prefix for hexadecimal are accepted.\n\nNote: a decimal number too big for 64 bits does not fail; it sticks at [integer.maxsigned] or [integer.minsigned]."
        }
        "code.help.integer.fromstring.str" => {
            "The text with the digits, optionally starting with `-`."
        }
        "code.help.integer.fromstring.base" => {
            "The base of the digits, from 2 to 36; `16` reads hexadecimal like `\"ff\"`. Leave it out for ordinary decimal."
        }
        "code.help.integer.fromstring.returns" => {
            "The integer, or `nil` if the text is not a whole number."
        }

        "code.help.integer.ge" => {
            "Checks whether the first integer is bigger than or equal to the second.\n\nUse it in place of `>=`, which does not work on integers."
        }
        "code.help.integer.ge.x" => "The first integer.",
        "code.help.integer.ge.y" => "The integer to compare with.",
        "code.help.integer.ge.returns" => "`true` or `false`.",

        "code.help.integer.gt" => {
            "Checks whether the first integer is bigger than the second.\n\nUse it in place of `>`, which does not work on integers."
        }
        "code.help.integer.gt.x" => "The first integer.",
        "code.help.integer.gt.y" => "The integer to compare with.",
        "code.help.integer.gt.returns" => "`true` or `false`.",

        "code.help.integer.idiv" => {
            "Divides two integers and rounds the result down.\n\nSo `-7` divided by `2` gives `-4`, the same as `//` does for ordinary numbers. Dividing by zero is an error, and so is [integer.minsigned] divided by `-1`. Its partner for the leftover is [integer.mod]."
        }
        "code.help.integer.idiv.x" => "The integer to divide.",
        "code.help.integer.idiv.y" => "What to divide by; must not be zero.",
        "code.help.integer.idiv.returns" => "The quotient, rounded down.",

        "code.help.integer.le" => {
            "Checks whether the first integer is smaller than or equal to the second.\n\nUse it in place of `<=`, which does not work on integers."
        }
        "code.help.integer.le.x" => "The first integer.",
        "code.help.integer.le.y" => "The integer to compare with.",
        "code.help.integer.le.returns" => "`true` or `false`.",

        "code.help.integer.lrotate" => {
            "Moves the bits to the left, bringing the ones that fall off the top back in at the bottom.\n\nNothing is lost. The count goes round every 64 places, and a negative count turns the other way."
        }
        "code.help.integer.lrotate.x" => "The integer whose bits turn.",
        "code.help.integer.lrotate.numBitPositions" => "How many places to turn.",
        "code.help.integer.lrotate.returns" => "The rotated integer.",

        "code.help.integer.lshift" => {
            "Moves the bits to the left, filling with zeros; each step doubles the value.\n\nBits pushed past the top are lost, and a shift of 64 or more gives `0`. A negative count shifts to the right instead."
        }
        "code.help.integer.lshift.x" => "The integer whose bits move.",
        "code.help.integer.lshift.numBitPositions" => "How many places to move.",
        "code.help.integer.lshift.returns" => "The shifted integer.",

        "code.help.integer.lt" => {
            "Checks whether the first integer is smaller than the second.\n\nUse it in place of `<`, which does not work on integers. Both values must be integers; an ordinary number is an error."
        }
        "code.help.integer.lt.x" => "The first integer.",
        "code.help.integer.lt.y" => "The integer to compare with.",
        "code.help.integer.lt.returns" => "`true` or `false`.",

        "code.help.integer.max" => {
            "Gives the biggest of the integers.\n\nIt needs at least one value; with none it stops with an error."
        }
        "code.help.integer.max.first" => "The first integer.",
        "code.help.integer.max.vararg" => "Any number of other integers.",
        "code.help.integer.max.returns" => "The biggest one.",

        "code.help.integer.maxsigned" => {
            "The biggest integer there is: 9223372036854775807 (2^63 - 1).\n\nAdding one to it wraps round to [integer.minsigned]."
        }

        "code.help.integer.min" => {
            "Gives the smallest of the integers.\n\nIt needs at least one value; with none it stops with an error."
        }
        "code.help.integer.min.first" => "The first integer.",
        "code.help.integer.min.vararg" => "Any number of other integers.",
        "code.help.integer.min.returns" => "The smallest one.",

        "code.help.integer.minsigned" => {
            "The smallest integer there is: -9223372036854775808 (-2^63).\n\nIt has no positive twin: [integer.neg] of it gives it back unchanged."
        }

        "code.help.integer.mod" => {
            "The leftover of a division that rounds down; the result takes the sign of the divisor.\n\nWith a positive divisor it never goes negative, so `-1` mod `8` is `7`: handy for looping round a cycle of frames. Dividing by zero is an error. To keep the sign of the first value instead, use [integer.rem]."
        }
        "code.help.integer.mod.x" => "The integer to divide.",
        "code.help.integer.mod.y" => "What to divide by; must not be zero.",
        "code.help.integer.mod.returns" => "The leftover, with the same sign as `y`.",

        "code.help.integer.mul" => {
            "Multiplies two integers.\n\nIf the result is too big it wraps round instead of failing, so check your sizes when the values can grow large."
        }
        "code.help.integer.mul.x" => "The first integer.",
        "code.help.integer.mul.y" => "The integer to multiply by.",
        "code.help.integer.mul.returns" => "The product, wrapped round to 64 bits.",

        "code.help.integer.neg" => {
            "Flips the sign of an integer: `5` becomes `-5`.\n\nUse it in place of the minus sign, which does not work on integers. [integer.minsigned] has no positive twin and comes back unchanged."
        }
        "code.help.integer.neg.value" => "The integer to flip.",
        "code.help.integer.neg.returns" => "The integer with the opposite sign.",

        "code.help.integer.rem" => {
            "The leftover of a division that rounds towards zero; the result takes the sign of the first value.\n\nSo `-7` rem `2` is `-1`, where [integer.mod] gives `1`. It pairs with [integer.div]. Dividing by zero is an error."
        }
        "code.help.integer.rem.x" => "The integer to divide.",
        "code.help.integer.rem.y" => "What to divide by; must not be zero.",
        "code.help.integer.rem.returns" => "The leftover, with the same sign as `x`.",

        "code.help.integer.replace" => {
            "Writes a group of bits into an integer, leaving all the other bits as they were.\n\nThe partner of [integer.extract]: use it to pack small values side by side. Only the lowest `numBits` of the replacement are used. Positions outside 0–63 are an error."
        }
        "code.help.integer.replace.value" => "The integer to write into.",
        "code.help.integer.replace.replacement" => "The value to put in those bits.",
        "code.help.integer.replace.bitPosition" => "The first (lowest) bit to write, from 0 to 63.",
        "code.help.integer.replace.numBits" => {
            "How many bits to write. Leave it out to write just one bit."
        }
        "code.help.integer.replace.returns" => {
            "A new integer with those bits replaced; the original is not changed."
        }

        "code.help.integer.rrotate" => {
            "Moves the bits to the right, bringing the ones that fall off the bottom back in at the top.\n\nNothing is lost. The count goes round every 64 places, and a negative count turns the other way."
        }
        "code.help.integer.rrotate.x" => "The integer whose bits turn.",
        "code.help.integer.rrotate.numBitPositions" => "How many places to turn.",
        "code.help.integer.rrotate.returns" => "The rotated integer.",

        "code.help.integer.rshift" => {
            "Moves the bits to the right, filling with zeros from the top.\n\nThe sign is not kept: `-1` shifted by 60 gives `15`. A shift of 64 or more gives `0`, and a negative count shifts to the left. To keep the sign, use [integer.arshift]."
        }
        "code.help.integer.rshift.x" => "The integer whose bits move.",
        "code.help.integer.rshift.numBitPositions" => "How many places to move.",
        "code.help.integer.rshift.returns" => "The shifted integer.",

        "code.help.integer.sub" => {
            "Subtracts the second integer from the first.\n\nLike [integer.add], it wraps round instead of failing when the result goes past the limits."
        }
        "code.help.integer.sub.x" => "The integer to subtract from.",
        "code.help.integer.sub.y" => "The integer to take away.",
        "code.help.integer.sub.returns" => "The difference, wrapped round to 64 bits.",

        "code.help.integer.tonumber" => {
            "Turns an integer back into an ordinary number, so you can use it with `+`, `math` and the rest.\n\nNote: above 2^53 the number is rounded to the nearest one it can hold. The global `tonumber` does not do this job: given an integer it gives `nil`."
        }
        "code.help.integer.tonumber.x" => "The integer to convert.",
        "code.help.integer.tonumber.returns" => "An ordinary number, possibly rounded.",

        "code.help.integer.udiv" => {
            "Divides two integers as if neither had a sign, dropping the fraction.\n\nA negative value counts as a huge positive one here, so `-2` divided by `2` gives [integer.maxsigned]. Dividing by zero is an error."
        }
        "code.help.integer.udiv.x" => "The integer to divide, read without a sign.",
        "code.help.integer.udiv.y" => "What to divide by, read without a sign; must not be zero.",
        "code.help.integer.udiv.returns" => "The whole part of the quotient.",

        "code.help.integer.uge" => {
            "Checks bigger-than-or-equal, treating both integers as if they had no sign.\n\nA negative value counts as huge here, so `-1` is the biggest of all. For the usual comparison, use [integer.ge]."
        }
        "code.help.integer.uge.x" => "The first integer.",
        "code.help.integer.uge.y" => "The integer to compare with.",
        "code.help.integer.uge.returns" => "`true` or `false`.",

        "code.help.integer.ugt" => {
            "Checks bigger-than, treating both integers as if they had no sign.\n\nA negative value counts as huge here. For the usual comparison, use [integer.gt]."
        }
        "code.help.integer.ugt.x" => "The first integer.",
        "code.help.integer.ugt.y" => "The integer to compare with.",
        "code.help.integer.ugt.returns" => "`true` or `false`.",

        "code.help.integer.ule" => {
            "Checks smaller-than-or-equal, treating both integers as if they had no sign.\n\nA negative value counts as huge here. For the usual comparison, use [integer.le]."
        }
        "code.help.integer.ule.x" => "The first integer.",
        "code.help.integer.ule.y" => "The integer to compare with.",
        "code.help.integer.ule.returns" => "`true` or `false`.",

        "code.help.integer.ult" => {
            "Checks smaller-than, treating both integers as if they had no sign.\n\nA negative value counts as huge here, so `-1` is not smaller than `1`. For the usual comparison, use [integer.lt]."
        }
        "code.help.integer.ult.x" => "The first integer.",
        "code.help.integer.ult.y" => "The integer to compare with.",
        "code.help.integer.ult.returns" => "`true` or `false`.",

        "code.help.integer.urem" => {
            "The leftover of a division where both integers are treated as if they had no sign.\n\nIt pairs with [integer.udiv]: `-1` read without a sign is 18446744073709551615, so its leftover after dividing by `10` is `5`. Dividing by zero is an error."
        }
        "code.help.integer.urem.x" => "The integer to divide, read without a sign.",
        "code.help.integer.urem.y" => "What to divide by, read without a sign; must not be zero.",
        "code.help.integer.urem.returns" => "The leftover.",
        _ => return None,
    })
}
