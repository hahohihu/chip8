pub type U4 = u8;
pub type U12 = u16;

pub fn n_set_bits(num_bits: u8) -> u16 {
    assert!(num_bits < 16);
    (1 << num_bits) - 1
}

/// Gets num_nibbles nibbles (4-bit sequences) starting from index
///
/// # Arguments
///
/// * `instruction` - The 2 bytes we want to get the nibbles from
/// * `index` - The index we want - for u16, one of: 0 1 2 3
/// * `num_nibbles` - The number of nibbles we want.
///
pub fn get_nibbles(instruction: u16, index: u8, num_nibbles: u8) -> u16 {
    assert!(num_nibbles >= 1); // Otherwise pointless
    assert!(num_nibbles <= 4); // Only 4 nibbles in u16
    /*
        Say we run get_nibbles(i, 1, 2) - then we want 0 [1 2] 3
        2 would be our rightmost index, 
        and we want to right-shift so we can position the relevant bits.
    */
    let right_index = index + num_nibbles - 1; // say we run get_nibbles(i, 1, 2)
    assert!(right_index <= 3); // indexes from 0-3
    // right-shift bit so the least significant bit is what we want
    // then zero out the unwanted upper bits
    (instruction >> ((3 - right_index) * 4)) & n_set_bits(num_nibbles * 4)
}

pub fn get_nibble(instruction: u16, index: u8) -> u8 {
    assert!(index <= 3);
    get_nibbles(instruction, index, 1) as u8
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    proptest! {
        #[test]
        fn never_fails(
            instruction in 0..std::u16::MAX, 
            index in 0..4, 
            num_nibbles in 1..4) 
        {
            use crate::bits::get_nibbles;
            if index + num_nibbles <= 4 {
                get_nibbles(instruction as u16, index as u8, num_nibbles as u8);
            }
        }
    }
}