pub struct Buffer {
    text :Vec<char>,
}

pub struct Document {
    original :Buffer,
    append :Buffer,

    pieces :Vec<Ref>,
    length_sum: Vec<usize>,

}

pub enum Ref {
    Original(usize, usize),
    Append(usize, usize),
}

impl Document {
    pub fn new(text :String) -> Document {
        let text :Vec<char> = text.chars().collect();
        let length = text.len();
        Document {
            original: Buffer { text: text },
            append:   Buffer { text: Vec::new() },
            pieces:    vec![ Ref::Original(0, length) ],
            length_sum:vec![length],
        }
    }

    pub fn get_ref(&self, r :&Ref) -> &[char] {
        match r {
            Ref::Original(idx,len) => &self.original.text[*idx..(idx+len)],
            Ref::Append(idx,len) => &self.append.text[*idx..(idx+len)],
        }
    }

    pub fn find_piece(&self, idx :usize) -> (usize, usize) {
        unimplemented!()
    }

    pub fn insert(&mut self) {
        unimplemented!()
    }

}



// // https://stackoverflow.com/questions/16793550/binary-indexed-tree-how-to-find-index-with-given-cumulative-frequency
// fn fw_find(sums :&[usize], mut freq :usize) -> Option<usize> {
//     let mut idx = 0;
//     let mut bitmask = (sums.len() as f64).log(2.0) as usize;
// 
//     while bitmask != 0 {
//         let t_idx = idx + bitmask; // midpoint
//         bitmask >>= 1; // halve interval
// 
//         if t_idx > sums.len() {
//             continue;
//         }
// 
//         if freq == sums[t_idx] {
//             return Some(t_idx);
//         } else if freq < sums[t_idx] {
//             idx = t_idx;
//             freq -= sums[t_idx];
//         }
//     }
//     Some(idx)
// }

// 
// pub struct Treap(Option<Node>);
// impl Treap {
//     pub fn new() -> Self { Treap(None) }
//     pub fn add(&mut self, doc: &Document, pos :usize, value :Ref) {
//         let (first,last) = split(root, pos, 0);
//         let new_node = Node::new(doc, value);
//         self.0 = first.merge(new_node).merge(last)
//     }
// 
//     pub fn remove(&mut self, doc :&Document, pos :usize) -> Option<()> {
//         let (first,second) = (self.0).take()?.split(pos, 0);
//         let (_, last )     = second.split(1, 0);
//         self.0 = first.merge(last);
//         Some(())
//     }
// }
// 
// pub struct Node {
//     priority :usize,
//     value :Ref,
//     length_sum :usize,
//     line_sum :usize,
//     left  :Option<Box<Node>>,
//     right :Option<Box<Node>>,
// }
// 
// fn rand() -> usize {
//     unimplemented!()
// }
// 
// impl Node {
//     pub fn new(doc :&Document, value :Ref) -> Self {
//         let length = doc.get_ref(&value).len();
//         let lines  = doc.get_ref(&value).chars().filter(|c| *c == '\n').count();
// 
//         Node {
//             priority: rand(),
//             value: value,
//             length_sum: length,
//             line_sum: lines,
//             left: None,
//             right: None,
//         }
//     }
// 
//     pub fn merge(self, other: Node) -> Node {
//         if self.priority > other.priority {
//             self.right = self.right.map(|n| Box::new(n.merge(other)));
//             self.update();
//             self
//         } else {
//             other.left = other.left.map(|n| Box::new(self.merge(*n)));
//             other.update();
//             other
//         }
//     }
// 
//     pub fn update(&mut self) {}
// }
