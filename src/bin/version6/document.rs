use fenwick::Fenwick;

pub struct Buffer {
    text :Vec<char>,
}

pub struct Document {
    original :Buffer,
    append :Buffer,

    pieces :Vec<Ref>,
    length_sum: Fenwick,
}

#[derive(Copy, Clone,Debug)]
pub enum Ref {
    Original(usize, usize),
    Append(usize, usize),
}

impl Ref {
    pub fn len(&self) -> usize { 
        match self {
            Ref::Original(_,l) => *l,
            Ref::Append(_,l) => *l,
        }
    }
}

impl Document {
    pub fn new(text :String) -> Document {
        let text :Vec<char> = text.chars().collect();
        let length     = text.len();
        let mut length_sum = Fenwick::new();
        length_sum.add(0,length);
        Document {
            original: Buffer { text: text },
            append:   Buffer { text: Vec::new() },
            pieces:    vec![ Ref::Original(0, length) ],
            length_sum:length_sum,
        }
    }

    fn get_ref(&self, r :&Ref) -> &[char] {
        match r {
            Ref::Original(idx,len) => &self.original.text[*idx..(idx+len)],
            Ref::Append(idx,len) => &self.append.text[*idx..(idx+len)],
        }
    }

    pub fn insert(&mut self, idx :usize, c :char) {
        println!("OLD piece table:");
        for x in &self.pieces {
            println!("  - {:?}", x);
        }
        println!("  orig:{}\n  apnd:{}", self.original.text.iter().collect::<String>(), self.append.text.iter().collect::<String>());
        println!("  FIND {}@ {:?}", idx, self.length_sum.find_prefix(idx));
        match self.length_sum.find_prefix(idx) {
            Ok(piece_idx) => { // Add to/after end of piece
                match self.pieces[piece_idx] {
                    Ref::Original(x, l) => {
                        // Insert new piece
                        println!("NEW PIECE");
                        let idx = self.append.text.len();
                        self.append.text.push(c);
                        self.pieces.insert(piece_idx+1, Ref::Append(idx,1));
                        
                        // update length_sum
                        for i in (piece_idx+1)..(self.pieces.len()-1) {
                            println!("SHIFT length_sum @{}", i); 
                            // Shift pieces ahead
                            self.length_sum.sub(i,   self.pieces[i].len());
                            self.length_sum.add(i+1, self.pieces[i].len());
                        }

                        self.length_sum.add(piece_idx+1, 1);
                    },
                    Ref::Append(x, l) => {
                        println!("ADD@PIECE");
                        self.append.text.push(c);
                        self.pieces[piece_idx] = Ref::Append(x,l+1);
                        self.length_sum.add(piece_idx, 1);
                    }
                }
            },
            Err(left_idx) => {
            },
        }
        println!("Updated piece table:");
        for x in &self.pieces {
            println!("  - {:?}", x);
        }
        println!("  orig:{}\n  apnd:{}", self.original.text.iter().collect::<String>(), self.append.text.iter().collect::<String>());
    }

    pub fn remove(&mut self, idx: usize, len :usize) {
    }

    pub fn get(&mut self, idx :usize) -> char {
        let left = self.length_sum.find_prefix_left(idx);
        let prefix = if left > 0 { self.length_sum.prefix_sum(left - 1) }  else { 0 };
        //println!("Getting at {}, left {} prefix {}", idx, left, prefix);
        let offset = idx - prefix;
        self.get_ref(&self.pieces[left])[offset]
    }

    fn foo(n: u32) -> impl Iterator<Item = char> {
        (0..n).map(|x| 'c')
    }
}

#[cfg(test)]
mod tests {
    use super::Document;

    #[test]
    fn test_doc_immutable() {
        let mut doc = Document::new("hallo".to_string());
        assert_eq!(doc.get(0), 'h');
        assert_eq!(doc.get(1), 'a');
        assert_eq!(doc.get(2), 'l');
        assert_eq!(doc.get(3), 'l');
        assert_eq!(doc.get(4), 'o');
    }

    #[test]
    fn test_doc_insert() {
        let mut doc = Document::new("hallo".to_string());
        doc.insert(5, 'x');
        doc.insert(6, 'y');

        assert_eq!(doc.get(0), 'h');
        assert_eq!(doc.get(1), 'a');
        assert_eq!(doc.get(2), 'l');
        assert_eq!(doc.get(3), 'l');
        assert_eq!(doc.get(4), 'o');
        assert_eq!(doc.get(5), 'x');
        assert_eq!(doc.get(6), 'y');
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
