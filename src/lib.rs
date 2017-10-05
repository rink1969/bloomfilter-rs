#![cfg_attr(test, feature(test))]
extern crate bit_vec;

use bit_vec::BitVec;
use std::hash::{Hash, Hasher};

extern "C" {
    pub fn spooky_hash128(message: *const u8,  // message to hash
                       length: usize,          // length of message in bytes
                       hash1: *mut u64,        // in/out: in seed 1, out hash value 1
                       hash2: *mut u64);       // in/out: in seed 2, out hash value 2    
}


#[derive(Debug)]
struct Spooky {
    hash1: u64,
    hash2: u64,
    count: u64,
}

impl Spooky {
    pub fn new() -> Self {
        Spooky {
            hash1: 0,
            hash2: 0,
            count: 0,
        }
    }
}

impl Hasher for Spooky {
    fn write(&mut self, bytes: &[u8]) {
        if self.count == 0 {
            let length = bytes.len();
            unsafe {
                spooky_hash128(bytes.as_ptr(), length, &mut self.hash1, &mut self.hash2);
            }
        }
        self.count = self.count + 1;
    }

    fn finish(&self) -> u64 {
        match self.count {
            0 => { self.hash1 }
            1 => { self.hash1.wrapping_add(self.hash2) }
            _ => {
                let p1 = self.hash2.wrapping_mul(self.count);
                self.hash1.wrapping_add(p1)
            }
        }
    }
}


// bits bitmap
// m bits length
// n item count
// k hash number
#[derive(Debug)]
pub struct Bloom {
    bits: BitVec,
    m: usize,
    n: usize,
    k: u32,
}

impl Bloom {
    /// Create a new bloom filter structure.
    pub fn new(m: usize, n: usize) -> Bloom {
        assert!(m > 0 && n > 0);
        let k = Bloom::optimal_k(m, n);
        let bits = BitVec::from_elem(m, false);
        Bloom {
            bits: bits,
            m: m,
            n: n,
            k: k,
        }
    }

    pub fn optimal_k(m: usize, n: usize) -> u32 {
        let m = m as f64;
        let n = n as f64;
        let k = (m / n * std::f64::consts::LN_2).ceil() as u32;
        std::cmp::max(k, 1)
    }

    pub fn compute_m(n: usize, fp_p: f64) -> usize {
        let ln22 = std::f64::consts::LN_2 * std::f64::consts::LN_2;
        (n as f64 * ((1.0/fp_p).ln() / ln22)).round() as usize
    }

    pub fn new_for_fp_rate(n: usize, fp_p: f64) -> Bloom {
        let m = Bloom::compute_m(n, fp_p);
        Bloom::new(m, n)
    }

    pub fn set<T>(&mut self, item: T)
    where
        T: Hash,
    {
        let mut s = Spooky::new();
        for _ in 0..self.k {
            item.hash(&mut s);
            let hash = s.finish();
            let index = hash as usize % self.m;
            self.bits.set(index, true);
        }
    }

    pub fn check<T>(&self, item: T) -> bool
    where
        T: Hash,
    {
        let mut s = Spooky::new();
        for _ in 0..self.k {
            item.hash(&mut s);
            let hash = s.finish();
            let index = hash as usize % self.m;
            if !self.bits.get(index).unwrap() {
                return false;
            }
        }
        true
    }

}


#[cfg(test)]
mod bench {
    extern crate test;
    extern crate rand;
    use self::test::Bencher;
    use self::rand::Rng;
    use super::*;

    #[bench]
    fn insert_benchmark(b: &mut Bencher) {
        let cnt = 500000;
        let rate = 0.01 as f64;

        let mut bf: Bloom = Bloom::new_for_fp_rate(cnt as usize, rate);
        let mut rng = rand::thread_rng();

        b.iter(|| {
            let v = rng.next_u32();
            bf.set(&v);
        })
    }

    #[bench]
    fn contains_benchmark(b: &mut Bencher) {
        let cnt = 500000;
        let rate = 0.01 as f64;

        let mut bf: Bloom = Bloom::new_for_fp_rate(cnt as usize, rate);
        let mut rng = rand::thread_rng();

        let mut i = 0;
        while i < cnt {
            let v = rng.next_u32();
            bf.set(&v);
            i+=1;
        }

        b.iter(|| {
            let v = rng.next_u32();
            bf.check(&v);
        })
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    use self::rand::Rng;
    use std::collections::HashSet;
    use super::*;

    #[test]
    fn spooky_hash128_test() {
        let buf = [0];
        let length = 0;
        let mut hash1 = 0;
        let mut hash2 = 0;
        unsafe {
            spooky_hash128((&buf).as_ptr(), length, &mut hash1, &mut hash2);
        }
        assert_eq!(hash1 & 0x00000000FFFFFFFF, 0x6bf50919);
    }

    #[test]
    fn simple() {
        let mut b: Bloom = Bloom::new_for_fp_rate(100 as usize, 0.01);
        b.set(&1);
        assert!(b.check(&1));
        assert!(!b.check(&2));
    }

    #[test]
    fn fpr_test() {
        let cnt = 500000;
        let rate = 0.01 as f64;

        let bits = Bloom::compute_m(cnt as usize, rate);
        assert_eq!(bits, 4792529);
        let hashes = Bloom::optimal_k(bits, cnt);
        assert_eq!(hashes, 7);

        let mut b: Bloom = Bloom::new_for_fp_rate(cnt as usize, rate);
        let mut set:HashSet<i32> = HashSet::new();
        let mut rng = rand::thread_rng();

        let mut i = 0;

        while i < cnt {
            let v = rng.gen::<i32>();
            set.insert(v);
            b.set(&v);
            i+=1;
        }

        i = 0;
        let mut false_positives = 0;
        while i < cnt {
            let v = rng.gen::<i32>();
            match (b.check(&v),set.contains(&v)) {
                (true, false) => { false_positives += 1; }
                (false, true) => { assert!(false); } // should never happen
                _ => {}
            }
            i+=1;
        }

        // make sure we're not too far off
        let actual_rate = false_positives as f64 / cnt as f64;
        assert!(actual_rate > (rate-0.001));
        assert!(actual_rate < (rate+0.001));
    }
}