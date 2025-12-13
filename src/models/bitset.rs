

use crate::enums::order_book_errors::OrderBookError;

pub struct Bitset<const N: usize>
where
    [(); (N + 63) / 64]: 
{
    pub bits: [u64; (N + 63) / 64]
}

impl<const N: usize> Bitset<N>
where
    [(); (N + 63) / 64]: 
{
    pub const fn new() -> Self {
        Self { 
            bits: [0; (N + 63) / 64] 
        }
    }

    #[inline]
    pub fn set(&mut self, idx: usize) -> Result<(), OrderBookError> {
        if idx > N {
            return Err(OrderBookError::BitsetIndexOutOfRange(N));
        }
        let block = ((N + 63) / 64) - (idx >> 6) - 1;
        let bit = idx & 63;
        self.bits[block] |= 1 << bit;

        Ok(())
    }

    #[inline]
    pub fn clear(&mut self, idx: usize) -> Result<(), OrderBookError> {
        if idx > N {
            return Err(OrderBookError::BitsetIndexOutOfRange(N));
        }
        let block = ((N + 63) / 64) - (idx >> 6) - 1;
        let bit = idx & 63;
        self.bits[block] &= !(1 << bit);

        Ok(())
    }

    #[inline]
    pub fn find_first_set(&self) -> Option<usize> {
        for (block_index, block) in self.bits.iter().enumerate() {
            if *block != 0 {
                let bit = block.trailing_zeros() as usize;
                return Some((((N + 63) / 64) - 1 - block_index) * 64 + bit);
            }
        }
        None
    }

    #[inline]
    pub fn find_last_set(&self) -> Option<usize> {
        for (block_index, block) in self.bits.iter().enumerate() {
            if *block != 0 {
                let bit = 63usize - block.leading_zeros() as usize;
                return Some((((N + 63) / 64) - 1 - block_index) * 64 + bit);
            }
        }
        None
    }

    #[inline]
    pub fn is_set(&self, idx: usize) -> bool {
        if idx > N {
            return false;
        }
        let block = ((N + 63) / 64) - (idx >> 6) - 1;
        let bit = idx & 63;
        (self.bits[block] & (1 << bit)) != 0
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_set_correctly_sets_specified_bit() {
        let mut bitset: Bitset<256> = Bitset::new();

        for block in bitset.bits {
            assert_eq!(block, 0);
        }

        let set_result = bitset.set(4);

        assert!(set_result.is_ok());
        assert_eq!(bitset.bits[3], 16);

        let set_result = bitset.set(196);
        
        assert!(set_result.is_ok());
        assert_eq!(bitset.bits[0], 16);
    }

    #[test]
    fn test_set_errors_bitset_index_out_of_range() {
        let mut bitset: Bitset<256> = Bitset::new();

        for block in bitset.bits {
            assert_eq!(block, 0);
        }

        let set_result = bitset.set(257);

        assert!(set_result.is_err());
        assert_eq!(set_result.err().unwrap(), OrderBookError::BitsetIndexOutOfRange(256));
        assert_eq!(bitset.bits[3], 0);
    }

    #[test]
    fn test_clear_correctly_clears_specified_bit() {
        let mut bitset: Bitset<256> = Bitset::new();

        for block in bitset.bits {
            assert_eq!(block, 0);
        }

        let set_result = bitset.set(4);

        assert!(set_result.is_ok());
        assert_eq!(bitset.bits[3], 16);

        let clear_result = bitset.clear(4);

        assert!(clear_result.is_ok());
        assert_eq!(bitset.bits[3], 0);
    }

    #[test]
    fn test_clear_errors_bitset_index_out_of_range() {
        let mut bitset: Bitset<256> = Bitset::new();

        for block in bitset.bits {
            assert_eq!(block, 0);
        }

        let set_result = bitset.set(4);

        assert!(set_result.is_ok());
        assert_eq!(bitset.bits[3], 16);

        let clear_result = bitset.clear(257);

        assert!(clear_result.is_err());
        assert_eq!(clear_result.err().unwrap(), OrderBookError::BitsetIndexOutOfRange(256));
        assert_eq!(bitset.bits[3], 16);
    }

    #[test]
    fn test_find_first_set_correctly_finds_first_bit_set() {
        let mut bitset: Bitset<256> = Bitset::new();

        for block in bitset.bits {
            assert_eq!(block, 0);
        }

        let set_result = bitset.set(4);

        assert!(set_result.is_ok());
        assert_eq!(bitset.bits[3], 16);

        let set_result = bitset.set(0);
        
        assert!(set_result.is_ok());
        assert_eq!(bitset.bits[3], 17);

        let find_first_set_result = bitset.find_first_set();

        assert!(find_first_set_result.is_some());
        assert_eq!(find_first_set_result.unwrap(), 0);
    }

    #[test]
    fn test_find_first_set_correctly_returns_none_with_0_bits_set() {
        let bitset: Bitset<256> = Bitset::new();

        for block in bitset.bits {
            assert_eq!(block, 0);
        }

        let find_first_set_result = bitset.find_first_set();

        assert!(find_first_set_result.is_none());
    }

    #[test]
    fn test_find_last_set_correctly_finds_last_bit_set() {
        let mut bitset: Bitset<256> = Bitset::new();

        for block in bitset.bits {
            assert_eq!(block, 0);
        }

        let set_result = bitset.set(4);

        assert!(set_result.is_ok());
        assert_eq!(bitset.bits[3], 16);

        let set_result = bitset.set(0);
        
        assert!(set_result.is_ok());
        assert_eq!(bitset.bits[3], 17);

        let find_last_set_result = bitset.find_last_set();

        assert!(find_last_set_result.is_some());
        assert_eq!(find_last_set_result.unwrap(), 4);
    }
    
    #[test]
    fn test_find_last_set_correctly_returns_none_with_0_bits_set() {
        let bitset: Bitset<256> = Bitset::new();

        for block in bitset.bits {
            assert_eq!(block, 0);
        }

        let find_last_set_result = bitset.find_last_set();

        assert!(find_last_set_result.is_none());
    }
}