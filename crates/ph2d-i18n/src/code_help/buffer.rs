//! O texto da página `buffer` do editor de scripts (as assinaturas e os exemplos em
//! `ph2d-code-help/src/luau/buffer.rs`).

pub(super) fn tr(key: &str) -> Option<&'static str> {
    Some(match key {
        "code.help.buffer" => {
            "Fixed-size blocks of raw bytes, for packing numbers and text tightly."
        }
        "code.help.buffer.about" => {
            "A buffer is a row of bytes whose size is set when you make it, with [buffer.create] or [buffer.fromstring]; a new buffer is all zeros. You read and write numbers at a position called the offset, counted in bytes.\n\nNote: offsets start at 0, not at 1 like tables and strings. The first byte is at 0 and the last at `buffer.len(b) - 1`. Reading or writing past the end stops the script with `buffer access out of bounds`.\n\nThe name of each function tells the size: `u8` is one byte from 0 to 255, `i8` one byte from -128 to 127, `u16`/`i16` two bytes, `u32`/`i32` four bytes, and `f32`/`f64` decimal numbers in four and eight bytes. Numbers that do not fit wrap around instead of failing, and fractions are dropped for the whole-number kinds."
        }
        "code.help.buffer.copy" => {
            "Copies bytes from one buffer into another (or within the same one).\n\nWithout `sourceOffset` and `count` the whole source is copied, so the target must have room for all of it from `targetOffset` on, or the script stops with an error. Overlapping copies inside one buffer work correctly."
        }
        "code.help.buffer.copy.target" => "The buffer that receives the bytes.",
        "code.help.buffer.copy.targetOffset" => {
            "Where the bytes land in the target (0 is the first byte)."
        }
        "code.help.buffer.copy.source" => "The buffer the bytes come from.",
        "code.help.buffer.copy.sourceOffset" => {
            "Optional. Where to start reading in the source; 0 when left out."
        }
        "code.help.buffer.copy.count" => {
            "Optional. How many bytes to copy; the rest of the source when left out."
        }
        "code.help.buffer.create" => {
            "Makes a new buffer of a given number of bytes, all set to 0.\n\nThe size cannot change later, so pick enough room for everything you will write."
        }
        "code.help.buffer.create.size" => "How many bytes the buffer holds.",
        "code.help.buffer.create.returns" => "The new buffer.",
        "code.help.buffer.fill" => {
            "Sets a run of bytes to the same value.\n\nOnly the lowest byte of `value` is used, so keep it from 0 to 255. Without `count` it fills to the end of the buffer."
        }
        "code.help.buffer.fill.b" => "The buffer to change.",
        "code.help.buffer.fill.offset" => "The first byte to set (0 is the first byte).",
        "code.help.buffer.fill.value" => "The byte value to write, from 0 to 255.",
        "code.help.buffer.fill.count" => {
            "Optional. How many bytes to set; up to the end when left out."
        }
        "code.help.buffer.fromstring" => {
            "Makes a new buffer holding the bytes of a string.\n\nThe buffer is exactly as long as the string. Turn it back with [buffer.tostring]."
        }
        "code.help.buffer.fromstring.str" => "The text to copy in.",
        "code.help.buffer.fromstring.returns" => "A new buffer with the string's bytes.",
        "code.help.buffer.len" => {
            "How many bytes the buffer holds.\n\nNote: since offsets start at 0, the last valid offset is one less than this."
        }
        "code.help.buffer.len.b" => "The buffer to measure.",
        "code.help.buffer.len.returns" => "The size in bytes.",
        "code.help.buffer.readbits" => {
            "Reads a whole number made of single bits, at any bit position.\n\nBits are counted from 0, starting at the lowest bit of the first byte. Up to 32 bits can be read at once. Use it with [buffer.writebits] to pack on/off flags or small values tightly."
        }
        "code.help.buffer.readbits.b" => "The buffer to read.",
        "code.help.buffer.readbits.bitOffset" => "The first bit to read (0 is the first bit).",
        "code.help.buffer.readbits.bitCount" => "How many bits to read, from 0 to 32.",
        "code.help.buffer.readbits.returns" => "The bits as a whole number, never negative.",
        "code.help.buffer.readf32" => {
            "Reads a decimal number stored in 4 bytes.\n\nNote: 4-byte numbers are less precise than normal ones, so a value like `0.1` comes back slightly off. Use [buffer.readf64] when exact values matter."
        }
        "code.help.buffer.readf32.b" => "The buffer to read.",
        "code.help.buffer.readf32.offset" => "Where the 4 bytes start (0 is the first byte).",
        "code.help.buffer.readf32.returns" => "The number that was stored.",
        "code.help.buffer.readf64" => {
            "Reads a decimal number stored in 8 bytes.\n\nThis keeps a Luau number exactly as it was written."
        }
        "code.help.buffer.readf64.b" => "The buffer to read.",
        "code.help.buffer.readf64.offset" => "Where the 8 bytes start (0 is the first byte).",
        "code.help.buffer.readf64.returns" => "The number that was stored.",
        "code.help.buffer.readi16" => {
            "Reads a whole number from 2 bytes, as a value from -32768 to 32767."
        }
        "code.help.buffer.readi16.b" => "The buffer to read.",
        "code.help.buffer.readi16.offset" => "Where the 2 bytes start (0 is the first byte).",
        "code.help.buffer.readi16.returns" => "A whole number from -32768 to 32767.",
        "code.help.buffer.readi32" => {
            "Reads a whole number from 4 bytes, as a value that can be negative."
        }
        "code.help.buffer.readi32.b" => "The buffer to read.",
        "code.help.buffer.readi32.offset" => "Where the 4 bytes start (0 is the first byte).",
        "code.help.buffer.readi32.returns" => "A whole number from -2147483648 to 2147483647.",
        "code.help.buffer.readi8" => {
            "Reads one byte as a value from -128 to 127.\n\nThe same byte read with [buffer.readu8] gives 0 to 255 instead: 255 there is -1 here."
        }
        "code.help.buffer.readi8.b" => "The buffer to read.",
        "code.help.buffer.readi8.offset" => "Which byte to read (0 is the first byte).",
        "code.help.buffer.readi8.returns" => "A whole number from -128 to 127.",
        "code.help.buffer.readstring" => {
            "Reads a number of bytes as a string.\n\nAsking for more bytes than remain stops the script with an error."
        }
        "code.help.buffer.readstring.b" => "The buffer to read.",
        "code.help.buffer.readstring.offset" => "Where the text starts (0 is the first byte).",
        "code.help.buffer.readstring.count" => "How many bytes to read.",
        "code.help.buffer.readstring.returns" => "A new string with those bytes.",
        "code.help.buffer.readu16" => {
            "Reads a whole number from 2 bytes, as a value from 0 to 65535.\n\nThe lowest byte comes first in the buffer."
        }
        "code.help.buffer.readu16.b" => "The buffer to read.",
        "code.help.buffer.readu16.offset" => "Where the 2 bytes start (0 is the first byte).",
        "code.help.buffer.readu16.returns" => "A whole number from 0 to 65535.",
        "code.help.buffer.readu32" => {
            "Reads a whole number from 4 bytes, as a value from 0 to 4294967295."
        }
        "code.help.buffer.readu32.b" => "The buffer to read.",
        "code.help.buffer.readu32.offset" => "Where the 4 bytes start (0 is the first byte).",
        "code.help.buffer.readu32.returns" => "A whole number from 0 to 4294967295.",
        "code.help.buffer.readu8" => {
            "Reads one byte as a value from 0 to 255.\n\nNote: the first byte is at offset 0. Reading at `buffer.len(b)` is already past the end and stops the script."
        }
        "code.help.buffer.readu8.b" => "The buffer to read.",
        "code.help.buffer.readu8.offset" => "Which byte to read (0 is the first byte).",
        "code.help.buffer.readu8.returns" => "A whole number from 0 to 255.",
        "code.help.buffer.tostring" => {
            "Turns the whole buffer into a string, byte for byte.\n\nUse it to save or print what a buffer holds; [buffer.fromstring] goes the other way."
        }
        "code.help.buffer.tostring.b" => "The buffer to read.",
        "code.help.buffer.tostring.returns" => "A new string as long as the buffer.",
        "code.help.buffer.writebits" => {
            "Writes a whole number into single bits, at any bit position.\n\nBits are counted from 0, starting at the lowest bit of the first byte, and up to 32 can be written at once. Only the lowest `bitCount` bits of `value` are kept. The bits around them are left as they were."
        }
        "code.help.buffer.writebits.b" => "The buffer to change.",
        "code.help.buffer.writebits.bitOffset" => "The first bit to write (0 is the first bit).",
        "code.help.buffer.writebits.bitCount" => "How many bits to write, from 0 to 32.",
        "code.help.buffer.writebits.value" => "The number to store in those bits.",
        "code.help.buffer.writef32" => {
            "Writes a decimal number into 4 bytes.\n\nNote: this loses some precision; whole numbers above 16777216 and values like `0.1` are not kept exactly. Use [buffer.writef64] when that matters."
        }
        "code.help.buffer.writef32.b" => "The buffer to change.",
        "code.help.buffer.writef32.offset" => "Where the 4 bytes start (0 is the first byte).",
        "code.help.buffer.writef32.value" => "The number to store.",
        "code.help.buffer.writef64" => "Writes a decimal number into 8 bytes, keeping it exactly.",
        "code.help.buffer.writef64.b" => "The buffer to change.",
        "code.help.buffer.writef64.offset" => "Where the 8 bytes start (0 is the first byte).",
        "code.help.buffer.writef64.value" => "The number to store.",
        "code.help.buffer.writei16" => {
            "Writes a whole number into 2 bytes, for values from -32768 to 32767.\n\nNumbers outside that range wrap around, and fractions are dropped."
        }
        "code.help.buffer.writei16.b" => "The buffer to change.",
        "code.help.buffer.writei16.offset" => "Where the 2 bytes start (0 is the first byte).",
        "code.help.buffer.writei16.value" => "The number to store.",
        "code.help.buffer.writei32" => {
            "Writes a whole number into 4 bytes, for values that can be negative.\n\nNumbers outside -2147483648 to 2147483647 wrap around, and fractions are dropped."
        }
        "code.help.buffer.writei32.b" => "The buffer to change.",
        "code.help.buffer.writei32.offset" => "Where the 4 bytes start (0 is the first byte).",
        "code.help.buffer.writei32.value" => "The number to store.",
        "code.help.buffer.writei8" => {
            "Writes a whole number into one byte, for values from -128 to 127.\n\nNumbers outside that range wrap around: 200 reads back as -56."
        }
        "code.help.buffer.writei8.b" => "The buffer to change.",
        "code.help.buffer.writei8.offset" => "Which byte to write (0 is the first byte).",
        "code.help.buffer.writei8.value" => "The number to store.",
        "code.help.buffer.writestring" => {
            "Writes the bytes of a string into the buffer.\n\nOnly those bytes change; the rest of the buffer stays as it was. The text must fit before the end, or the script stops with an error. Give `count` to write only the start of the string."
        }
        "code.help.buffer.writestring.b" => "The buffer to change.",
        "code.help.buffer.writestring.offset" => "Where the text starts (0 is the first byte).",
        "code.help.buffer.writestring.value" => "The text to write.",
        "code.help.buffer.writestring.count" => {
            "Optional. How many bytes of the text to write; all of it when left out."
        }
        "code.help.buffer.writeu16" => {
            "Writes a whole number into 2 bytes, for values from 0 to 65535.\n\nThe lowest byte goes first. Numbers outside the range wrap around: 70000 reads back as 4464."
        }
        "code.help.buffer.writeu16.b" => "The buffer to change.",
        "code.help.buffer.writeu16.offset" => "Where the 2 bytes start (0 is the first byte).",
        "code.help.buffer.writeu16.value" => "The number to store.",
        "code.help.buffer.writeu32" => {
            "Writes a whole number into 4 bytes, for values from 0 to 4294967295.\n\nThe lowest byte goes first, so writing 1 puts it in the byte at `offset`."
        }
        "code.help.buffer.writeu32.b" => "The buffer to change.",
        "code.help.buffer.writeu32.offset" => "Where the 4 bytes start (0 is the first byte).",
        "code.help.buffer.writeu32.value" => "The number to store.",
        "code.help.buffer.writeu8" => {
            "Writes a whole number into one byte, for values from 0 to 255.\n\nNumbers outside that range wrap around (300 becomes 44), and fractions are dropped (2.9 becomes 2)."
        }
        "code.help.buffer.writeu8.b" => "The buffer to change.",
        "code.help.buffer.writeu8.offset" => "Which byte to write (0 is the first byte).",
        "code.help.buffer.writeu8.value" => "The number to store.",
        _ => return None,
    })
}
