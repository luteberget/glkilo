
#[derive(Debug)]
pub struct Fenwick(Vec<usize>);
impl Fenwick {
    pub fn new() -> Self {
        Fenwick(Vec::new())
    }

    #[inline]
    fn next_down(i: usize) -> usize {
        (i & i.wrapping_add(1)).wrapping_sub(1)
    }

    #[inline]
    fn next_up(i: usize) -> usize {
        i | i.wrapping_add(1)
    }

    pub fn sub(&mut self, mut idx: usize, value :usize) {
        while idx >= (self.0).len() {
            (self.0).push(0);
        }
        while idx != !0 {
            (self.0)[idx] -= value;
            idx = Self::next_down(idx);
        }
    }

    pub fn add(&mut self, mut idx :usize, value :usize) {
        while idx >= (self.0).len() {
            (self.0).push(0);
        }
        while idx != !0 {
            (self.0)[idx] += value;
            idx = Self::next_down(idx);
        }
    }

    pub fn suffix_sum(&self, mut idx :usize) -> usize {
        //println!("suffix_sum {:?}@{}", self.0, idx);
        let mut sum = 0;
        while idx < (self.0).len() {
            //println!("  {}", idx);
            sum += (self.0)[idx];
            idx = Self::next_up(idx);
        }
        sum
    }

    pub fn prefix_sum(&self, idx :usize) -> usize {
        self.suffix_sum(0) - self.suffix_sum(idx+1)
    }

    pub fn find_prefix_left(&self, sum :usize) -> usize {
        match self.find_prefix(sum) {
            Ok(x) => x+1,
            Err(x) => x,
        }
    }

    pub fn find_prefix(&self, sum :usize) -> Result<usize,usize> {
        // binary search from rust vec

        let mut size = (self.0).len();
        if size == 0 {
            return Err(0);
        }

        let mut base = 0usize;
        while size > 1 {
            let half = size/2;
            let mid = base+half;

            let value = self.prefix_sum(mid);
            base = if value > sum {
                base
            } else {
                mid
            };
            size -= half;
        }

        let value = self.prefix_sum(base);
        if value == sum {
            Ok(base)
        } else {
            Err(base + (value < sum) as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::{thread_rng, Rng, distributions::{Distribution, Range}};

    #[test]
    fn test_prefix() {
        let mut f = super::Fenwick(Vec::new());
        assert_eq!(f.find_prefix(0), Err(0));

        f.add(0,1);
        f.add(1,2);
        assert_eq!(f.find_prefix(2), Err(1));
    }

    #[test]
    fn find() {
        let mut f = super::Fenwick(Vec::new());

        assert_eq!(f.find_prefix(0),    Err(0));
        assert_eq!(f.find_prefix(1000), Err(0));

        f.add(0, 3);
        f.add(1, 3);
        f.add(2, 3);

        assert_eq!(f.prefix_sum(0), 3);
        assert_eq!(f.prefix_sum(1), 6);
        assert_eq!(f.prefix_sum(2), 9);
        assert_eq!(f.prefix_sum(3), 9);

        assert_eq!(f.find_prefix(0),  Err(0));
        assert_eq!(f.find_prefix(1),  Err(0));
        assert_eq!(f.find_prefix(2),  Err(0));
        assert_eq!(f.find_prefix(3),  Ok(0));
        assert_eq!(f.find_prefix(4),  Err(1));
        assert_eq!(f.find_prefix(5),  Err(1));
        assert_eq!(f.find_prefix(6),  Ok(1));
        assert_eq!(f.find_prefix(7),  Err(2));
        assert_eq!(f.find_prefix(8),  Err(2));
        assert_eq!(f.find_prefix(9),  Ok(2));
        assert_eq!(f.find_prefix(10), Err(3));
        assert_eq!(f.find_prefix(11), Err(3));

        assert_eq!(f.find_prefix_left(0), 0); 
        assert_eq!(f.find_prefix_left(1), 0); 
        assert_eq!(f.find_prefix_left(2), 0); 
        assert_eq!(f.find_prefix_left(3), 1); 
        assert_eq!(f.find_prefix_left(4), 1); 
        assert_eq!(f.find_prefix_left(5), 1); 
        assert_eq!(f.find_prefix_left(6), 2); 
        assert_eq!(f.find_prefix_left(7), 2); 
        assert_eq!(f.find_prefix_left(8), 2); 
        assert_eq!(f.find_prefix_left(9), 3); 
        assert_eq!(f.find_prefix_left(10),3); 
        assert_eq!(f.find_prefix_left(11),3); 
    }

    #[test]
    fn randoms() {
        let mut rng = thread_rng();
        for len in 0..130usize {
            random_one(&mut rng, len);
        }
    }

    fn random_one<TRng: Rng>(rng: &mut TRng, len: usize) {
        let mut data = vec![0; len];
        let range = Range::new_inclusive(0, 500);
        for x in data.iter_mut() {
            *x = range.sample(rng);
        }

        let mut prefix = 0;
        let mut psum = Vec::new();
        for i in 0..data.len() {
            prefix += data[i];
            psum.push(prefix);
        }

        let mut fenwick = super::Fenwick(Vec::new());
        for (i,x) in data.iter().enumerate() {
            fenwick.add(i,*x);
        }
        //println!("test {:?}\n     {:?}", data, psum);
        for (i,s) in psum.iter().enumerate() {
            assert_eq!(fenwick.prefix_sum(i), *s);
        }
    }
}


