use crate::blk::error::ParseError;

// Yields length in buffer and value
// ULEB variable length integer format: `https://en.wikipedia.org/wiki/LEB128`
#[inline]
pub fn uleb128(bytes: &[u8]) -> Result<(usize, usize), ParseError> {
    let mut result = 0_usize;
    const MASK: u8 = 1 << 7;

    // Each bytes leading bit indicates continuation, where the trailing 7 bits for the integer part of the number
    // This loop might always yield before reaching its last iteration, unless the buffer was cut too early
    for (i, current) in bytes.iter().enumerate() {
        // The bits holding the integer value, with the leading bit being unset
        let bits = (current & (MASK - 1)) as usize;

        // Shifting the bit into alignment and storing them in the intermediate variable
        // For example: 3 bytes of ULEB yield 3 * 7 = 21 bits, which would have 1-bit spacing between them if not for this alignment
        result |= bits << (7 * i);

        // The leading bit of the current byte is set, therefore the integer is complete and yields
        if MASK & current == 0 {
            return Ok((i + 1, result));
        };
    }

    // After the loop has finished, without yielding to the caller, it means something broke
    // In most cases this is due to the caller passing an invalid buffer that either ended too early, or was simply empty
    if bytes.len() == 0 {
        Err(ParseError::ZeroSizedUleb)
    } else {
        Err(ParseError::UnexpectedEndOfBufferUleb)
    }
}
