use std::ops::{Deref, DerefMut, Index, IndexMut};

/// Associative array of u32 -> u32 with the interface and implementation optimized
/// for use as a counter for small numbers with a dense distribution.
#[derive(Default, Debug, Clone)]
pub struct FrequencyCounter {
    data: Vec<u32>,
}

impl FrequencyCounter {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}

impl Index<u32> for FrequencyCounter {
    type Output = u32;

    /// Infallible. Returns 0 if index is out of range.
    fn index(&self, index: u32) -> &Self::Output {
        self.data.get(index as usize).unwrap_or(&0)
    }
}

impl IndexMut<u32> for FrequencyCounter {
    /// Infallible. Resizes container if index is out of range.
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        let index = index as usize;
        if index >= self.data.len() {
            self.data.resize(index + 1, 0);
        }
        // Safety: The line above guarantees that `index` is in-bounds.
        debug_assert!(self.data.len() > index);
        unsafe { self.data.get_unchecked_mut(index) }
    }
}

impl Deref for FrequencyCounter {
    type Target = Vec<u32>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for FrequencyCounter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
