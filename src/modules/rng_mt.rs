use crate::types::seed::*;

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908b0df;
const UPPER_MASK: u32 = 0x80000000;
const LOWER_MASK: u32 = 0x7fffffff;

pub struct RngMT {
    table: [u32; 624],
    index: usize,
}

impl RngMT {
    pub fn new(initial_seed: InitialSeed) -> Self {
        let mut table = [0u32; N];
        table[0] = initial_seed;

        for i in 1..N {
            let prev = table[i - 1];
            table[i] = ((prev ^ (prev >> 30)).wrapping_mul(0x6c078965)).wrapping_add(i as u32);
        }

        let mut rng = Self { table, index: N };
        rng.twist();
        return rng;
    }

    pub fn twist(&mut self) {
        for i in 0..N {
            let a = self.table[i] & UPPER_MASK;
            let b = self.table[(i + 1) % N] & LOWER_MASK;
            let k0 = a | b;
            let mut k1 = k0 >> 1;

            let k2_index = if i < (N - M) { i + M } else { i + M - N };
            let k2 = self.table[k2_index];
            k1 ^= k2;

            if k0 & 1 != 0 {
                k1 ^= MATRIX_A;
            }

            self.table[i] = k1;
        }

        self.index = 0;
    }

    pub fn next(&mut self) -> Seed {
        if N <= self.index {
            self.twist();
        }

        let next_seed = self.table[self.index];
        self.index += 1;
        return next_seed;
    }

    pub fn get_pid(&mut self, seed: Seed) -> PID {
        let k0 = (seed / 0x800) ^ seed;
        let k1 = ((k0.wrapping_mul(0x80)) & 0x9d2c5680) ^ k0;
        let k2 = ((k1.wrapping_mul(0x8000)) & 0xefc60000) ^ k1;
        let pid = (k2 / 0x40000) ^ k2;

        return pid;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_pid_test() {
        let initial_seed = 0x78000489;
        let mut mt = RngMT::new(initial_seed);
        let mut arr: [u8; 20] = [0u8; 20];
        let answer: [u8; 20] = [1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1];

        for i in 0..20 {
            let next_seed = mt.next();
            let pid = mt.get_pid(next_seed);
            arr[i] = (pid % 2 == 1) as u8;
        }

        assert!(arr == answer);
    }
}
