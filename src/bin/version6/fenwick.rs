
struct Fenwick(Vec<usize>);
impl Fenwick {
    #[inline]
    fn next_down(i: usize) -> usize {
        (i & i.wrapping_add(1)).wrapping_sub(1)
    }

    #[inline]
    fn next_up(i: usize) -> usize {
        i | i.wrapping_add(1)
    }

    pub fn add(&mut self, mut idx :usize, value :usize) {
        while idx > (self.0).len() - 1 {
            (self.0).push(0);
        }
        while idx > 0 {
            (self.0)[idx] += value;
            idx = Self::next_down(idx);
        }
    }

    pub fn suffix_sum(&self, mut idx :usize) -> usize {
        let mut sum = 0;
        while idx < (self.0).len() {
            sum += (self.0)[idx];
            idx = Self::next_up(idx);
        }
        sum
    }

    pub fn prefix_sum(&self, idx :usize) -> usize {
        self.suffix_sum(0) - self.suffix_sum(idx+1)
    }

    pub fn find_prefix(&self, sum :usize) -> usize {
        let mut left = 0;
        let mut right = (self.0).len()-1;
        while left <= right {
            let mid = (left+right)/2;
            let mid_sum = self.prefix_sum(mid);
            if mid_sum < sum {
                left = mid+1;
            } else if mid_sum > sum {
                right = mid-1;
            } else {
                return mid;
            }
        }
        left
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng, distributions::{Distribution, Range}};

    #[test]
    fn randoms() {
        let mut rng = thread_rng();
        for len in 0..130usize {
            random_one(&mut rng, len);
        }
    }

    fn random_one<TRng: Rng>(rng: &mut TRng, len: usize) {
        let mut data = vec![0i32; len];
        let range = Range::new_inclusive(-50, 50);
        for x in data.iter_mut() {
            *x = range.sample(rng);
        }
    }
}


