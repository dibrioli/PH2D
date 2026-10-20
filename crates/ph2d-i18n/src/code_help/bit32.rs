//! O texto da página `bit32` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/bit32.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.bit32" => {
            "Work with the single bits of whole numbers: flags, masks and packed colours."
        }
        "code.help.bit32.about" => {
            "Every function here treats numbers as 32-bit whole numbers without a sign, from 0 to 4294967295. Fractions are dropped and negative numbers wrap around to the top of that range, so `bit32.band(-1)` is 4294967295. Results are never negative.\n\nBits are counted from 0 (the lowest, worth 1) up to 31. A common use is keeping many on/off flags in one number, one bit each, or a colour's red, green and blue in 8 bits each. Writing numbers in hex such as `0xFF8800` makes the bits easier to see."
        }
        "code.help.bit32.arshift" => {
            "Moves the bits right, filling the top with copies of the highest bit.\n\nFor numbers below 2147483648 it is the same as [bit32.rshift]. For a negative number it keeps it \"negative\" in 32 bits, but the result is still shown as a big positive number. A negative `disp` shifts left instead."
        }
        "code.help.bit32.arshift.x" => "The number to shift.",
        "code.help.bit32.arshift.disp" => "How many places to move.",
        "code.help.bit32.arshift.returns" => "The shifted number.",
        "code.help.bit32.band" => {
            "Keeps only the bits that are on in every number given.\n\nUse it to check a flag (the result is 0 when the flag is off) or to cut out part of a number with a mask. Called with nothing, it gives 4294967295."
        }
        "code.help.bit32.band.vararg" => "Any number of numbers to combine.",
        "code.help.bit32.band.returns" => "The bits on in all of them.",
        "code.help.bit32.bnot" => {
            "Flips every bit: on becomes off and off becomes on.\n\nTogether with [bit32.band] it switches a flag off: `bit32.band(state, bit32.bnot(FLAG))`."
        }
        "code.help.bit32.bnot.x" => "The number to flip.",
        "code.help.bit32.bnot.returns" => "The flipped number, from 0 to 4294967295.",
        "code.help.bit32.bor" => {
            "Turns on every bit that is on in any of the numbers given.\n\nUse it to switch flags on or to join separate parts into one number. Called with nothing, it gives 0."
        }
        "code.help.bit32.bor.vararg" => "Any number of numbers to combine.",
        "code.help.bit32.bor.returns" => "The bits on in at least one of them.",
        "code.help.bit32.btest" => {
            "Tells whether the numbers given share at least one bit that is on.\n\nA short way of writing `bit32.band(...) ~= 0`, handy to ask \"is this flag set?\"."
        }
        "code.help.bit32.btest.x" => "The first number, often the set of flags.",
        "code.help.bit32.btest.vararg" => "Further numbers, often the flag to look for.",
        "code.help.bit32.btest.returns" => "`true` if a common bit is on, `false` otherwise.",
        "code.help.bit32.bxor" => {
            "Turns on the bits that are on in an odd number of the inputs.\n\nWith two numbers, it flips the bits of the first wherever the second has them on. Doing it twice gives back the start, which makes it a toggle. Called with nothing, it gives 0."
        }
        "code.help.bit32.bxor.vararg" => "Any number of numbers to combine.",
        "code.help.bit32.bxor.returns" => "The combined number.",
        "code.help.bit32.byteswap" => {
            "Reverses the order of the four bytes in a number.\n\nOnly needed when reading data made by programs that store bytes in the opposite order."
        }
        "code.help.bit32.byteswap.n" => "The number whose bytes to reverse.",
        "code.help.bit32.byteswap.returns" => "The number with its bytes in reverse order.",
        "code.help.bit32.countlz" => {
            "Counts the off bits at the top of the number, before the first on bit.\n\nGives 32 for 0. `31 - bit32.countlz(n)` is the position of the highest bit that is on."
        }
        "code.help.bit32.countlz.n" => "The number to look at.",
        "code.help.bit32.countlz.returns" => "A count from 0 to 32.",
        "code.help.bit32.countrz" => {
            "Counts the off bits at the bottom of the number, before the first on bit.\n\nGives 32 for 0. For a single flag, it is the flag's bit position."
        }
        "code.help.bit32.countrz.n" => "The number to look at.",
        "code.help.bit32.countrz.returns" => "A count from 0 to 32.",
        "code.help.bit32.extract" => {
            "Reads a group of bits out of a number, as a number of its own.\n\nThe group starts at bit `field` and is `width` bits wide (1 when left out). Use it to pull one channel out of a packed colour. The group must end at bit 31 or below, or the script stops with an error."
        }
        "code.help.bit32.extract.n" => "The number to read from.",
        "code.help.bit32.extract.field" => "The lowest bit of the group, from 0 to 31.",
        "code.help.bit32.extract.width" => "Optional. How many bits to read; 1 when left out.",
        "code.help.bit32.extract.returns" => "The bits of the group, moved down to start at bit 0.",
        "code.help.bit32.lrotate" => {
            "Moves the bits left in a circle: bits pushed out at the top come back in at the bottom.\n\nUnlike [bit32.lshift], no bits are lost. A negative `disp` rotates right."
        }
        "code.help.bit32.lrotate.x" => "The number to rotate.",
        "code.help.bit32.lrotate.disp" => "How many places to move.",
        "code.help.bit32.lrotate.returns" => "The rotated number.",
        "code.help.bit32.lshift" => {
            "Moves the bits left, filling the bottom with zeros.\n\nEach place doubles the number, but bits pushed past bit 31 are lost, so shifting by 32 or more gives 0. A negative `disp` shifts right instead."
        }
        "code.help.bit32.lshift.x" => "The number to shift.",
        "code.help.bit32.lshift.disp" => "How many places to move.",
        "code.help.bit32.lshift.returns" => "The shifted number.",
        "code.help.bit32.replace" => {
            "Puts a value into a group of bits of a number, leaving the other bits alone.\n\nThe group starts at bit `field` and is `width` bits wide (1 when left out); only the lowest `width` bits of `v` are used. It is the writing partner of [bit32.extract]."
        }
        "code.help.bit32.replace.n" => "The number to change.",
        "code.help.bit32.replace.v" => "The value to put into the group.",
        "code.help.bit32.replace.field" => "The lowest bit of the group, from 0 to 31.",
        "code.help.bit32.replace.width" => {
            "Optional. How many bits the group has; 1 when left out."
        }
        "code.help.bit32.replace.returns" => "A new number with the group replaced.",
        "code.help.bit32.rrotate" => {
            "Moves the bits right in a circle: bits pushed out at the bottom come back in at the top.\n\nUnlike [bit32.rshift], no bits are lost. A negative `disp` rotates left."
        }
        "code.help.bit32.rrotate.x" => "The number to rotate.",
        "code.help.bit32.rrotate.disp" => "How many places to move.",
        "code.help.bit32.rrotate.returns" => "The rotated number.",
        "code.help.bit32.rshift" => {
            "Moves the bits right, filling the top with zeros.\n\nEach place halves the number and drops the remainder. Use it with [bit32.band] to read a packed part, such as the red of a colour: `bit32.rshift(colour, 16)`. A negative `disp` shifts left instead."
        }
        "code.help.bit32.rshift.x" => "The number to shift.",
        "code.help.bit32.rshift.disp" => "How many places to move.",
        "code.help.bit32.rshift.returns" => "The shifted number.",
        _ => return None,
    })
}
