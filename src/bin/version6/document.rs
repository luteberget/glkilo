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

#[derive(Copy, Clone,Debug, PartialEq, Eq)]
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

    pub fn split(&self, len :usize) -> (Ref,Ref) {
        match self {
            Ref::Original(x,l) => (Ref::Original(*x,len), Ref::Original(x+len,l-len)),
            Ref::Append(x,l) => (Ref::Append(*x,len), Ref::Append(x+len,l-len)),
        }
    }

    pub fn skip(self, len :usize) -> Ref {
        match self {
            Ref::Original(x,l) => Ref::Original(x+len,l-len),
            Ref::Append(x,l) => Ref::Append(x+len,l-len),
        }
    }

    pub fn pop(self, len:usize) -> Ref {
        match self {
            Ref::Original(x,l) => Ref::Original(x,l-len),
            Ref::Append(x,l) => Ref::Append(x,l-len),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DocOp {
    Remove(usize, Ref),
    Insert(usize, Ref),
    Set(usize, Ref, Ref),
}

impl DocOp {
    pub fn inverse(self) -> Self {
        use self::DocOp::*;
        match self {
            Remove(idx,x) => Insert(idx,x),
            Insert(idx,x) => Remove(idx,x),
            Set(idx, a, b) => Set(idx, b, a),
        }
    }
}

impl Document {

    pub fn to_string(&self) -> String {
        let mut x = String::new();
        for piece in &self.pieces {
            for c in self.get_ref(piece) {
                x.push(*c);
            }
        }
        x
    }

    pub fn prev_linebreak(&self, mut i :usize) -> Option<usize> {
        // TODO this does too much work 
        let s = self.to_string().chars().collect::<Vec<_>>();
        i = usize::min(i,s.len()-1);
        while i > 0 {
            if s[i] == '\n' {
                return Some(i);
            }
            i -= 1;
        }
        None
    }

    pub fn next_linebreak(&self, mut i :usize) -> Option<usize> {
        // TODO this does too much work 
        let s = self.to_string().chars().collect::<Vec<_>>();
        while i < s.len() {
            if s[i] == '\n' {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    pub fn empty() -> Document {
        Document {
            original: Buffer { text: Vec::new() },
            append:   Buffer { text: Vec::new() },
            pieces: Vec::new(),
            length_sum:Fenwick::new(),
        }
    }

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

    pub fn len(&self) -> usize {
        self.length_sum.suffix_sum(0)
    }

    pub fn run(&mut self, ops :&[DocOp]) {
        for op in ops {
            match op {
                DocOp::Insert(idx, x) => {
                    for i in *idx .. self.pieces.len() {
                        self.length_sum.sub(i,   self.pieces[i].len());
                        self.length_sum.add(i+1, self.pieces[i].len());
                    }
                    self.pieces.insert(*idx,*x);
                    self.length_sum.add(*idx,x.len());
                },
                DocOp::Remove(idx, x) => {
                    for i in idx+1 .. self.pieces.len() {
                        self.length_sum.sub(i,   self.pieces[i].len());
                        self.length_sum.add(i-1, self.pieces[i].len());
                    }
                    let old = self.pieces.remove(*idx);
                    assert_eq!(*x, old);
                    self.length_sum.sub(*idx, x.len());
                },
                DocOp::Set(idx, old, new) => {
                    self.length_sum.sub(*idx, old.len());
                    self.length_sum.add(*idx, new.len());
                    self.pieces[*idx] = *new;
                },
            }
            println!("DOC OP {:?}", op);
            self.print_prefixes();
            println!("DOC OP {:?}", op);
        }
    }

    pub fn remove_actions(&mut self, idx: usize) -> Vec<DocOp> {
        if self.pieces.len() == 0 { return vec![] };
        if idx == 0 {
            if self.pieces[0].len() == 1{
                vec![DocOp::Remove(0, self.pieces[0])]
            } else {
                vec![DocOp::Set(0, self.pieces[0], self.pieces[0].skip(1))]
            }
        } else {
            match self.length_sum.find_prefix(idx) {
                Ok(piece_idx) => { // remove from start of piece_idx+1
                    println!("remove: remove from start");
                    let start_idx = piece_idx+1;
                    if start_idx < self.pieces.len()  {
                        if self.pieces[start_idx].len() > 1 {
                            println!("remove: -set");
                            vec![DocOp::Set(start_idx, self.pieces[start_idx], self.pieces[start_idx].skip(1))]
                        } else {
                            println!("remove: -remove piece");
                            vec![DocOp::Remove(start_idx, self.pieces[start_idx])]
                        }
                    } else {
                        panic!("Remove from after end of buffer");
                    }
                },
                Err(piece_idx) => { // split piece
                    println!("remove: split piece");
                    if piece_idx > (self.pieces.len() -1) { panic!("Delete action after end of buffer"); }
                    let length_before_piece = if piece_idx == 0 { 0 } else { self.length_sum.prefix_sum(piece_idx -1) };
                    let (before,after) = self.pieces[piece_idx].split(idx-length_before_piece);

                    if after.len() > 1 {
                        println!("remove: -split");
                        vec![DocOp::Remove(piece_idx, self.pieces[piece_idx]),
                             DocOp::Insert(piece_idx, before),
                             DocOp::Insert(piece_idx+1, after.skip(1))]
                    } else {
                        println!("remove: -set");
                        vec![DocOp::Set(piece_idx, self.pieces[piece_idx], before)]
                    }
                }
            }
        }
    }

    pub fn insert_actions(&mut self, idx :usize, c:char) -> Vec<DocOp> {
        let append_idx = self.append.text.len();
        self.append.text.push(c);

        if idx == 0 { // TODO: get rid of this case somehow?
            vec![DocOp::Insert(0, Ref::Append(append_idx, 1))]
        } else {
            match self.length_sum.find_prefix(idx) {
                Ok(piece_idx) => { // Add to/after end of piece
                    match self.pieces[piece_idx] {
                        Ref::Original(_,_) => {
                            println!("insert: orignal -> new append");
                            vec![DocOp::Insert(piece_idx+1, Ref::Append(append_idx,1))]
                        },
                        Ref::Append(x,l) => {
                            if x+l == append_idx {
                                println!("insert: replace append");
                                vec![DocOp::Set(piece_idx, self.pieces[piece_idx], Ref::Append(x,l+1))]
                            } else {
                                println!("insert: append and new append");
                                vec![DocOp::Insert(piece_idx+1, Ref::Append(append_idx,1))]
                            }
                        }
                    }
                },
                Err(piece_idx) => {
                    if piece_idx > (self.pieces.len() -1) { panic!("Insert after end of buffer"); }
                    let length_before_piece = if piece_idx == 0 { 0 }  else { self.length_sum.prefix_sum(piece_idx-1 ) };
                    let (before,after) = self.pieces[piece_idx].split(idx-length_before_piece);

                    println!("insert: split");
                    vec![DocOp::Remove(piece_idx, self.pieces[piece_idx]),
                         DocOp::Insert(piece_idx, before),
                         DocOp::Insert(piece_idx+1, Ref::Append(append_idx,1)),
                         DocOp::Insert(piece_idx+2, after)]
                }
            }
        }
    }

    pub fn insert(&mut self, idx :usize, c:char) {
          println!("OLD piece table:");
          for x in &self.pieces {
              println!("  - {:?}", x);
          }
          println!("  orig:{}\n  apnd:{}", self.original.text.iter().collect::<String>(), self.append.text.iter().collect::<String>());
          println!("  FIND {}@ {:?}", idx, self.length_sum.find_prefix(idx));
      
        let actions = self.insert_actions(idx, c);
        println!("ACTIONS: {:?}", actions);
        self.run(&actions);
          println!("Updated piece table:");
          for x in &self.pieces {
              println!("  - {:?}", x);
          }
          println!("  orig:{}\n  apnd:{}", self.original.text.iter().collect::<String>(), self.append.text.iter().collect::<String>());
          println!(" FENWICK {:?} len{}", self.length_sum, self.len());
          self.print_prefixes();
    }

    //pub fn  print_prefixes(&self) {
    //    println!("  PREFIXES : {:?}", self.pieces.iter().enumerate().map(|(i,_)| self.length_sum.prefix_sum(i)));
    //}

    //pub fn insert(&mut self, idx :usize, c :char) {
    //    println!("OLD piece table:");
    //    for x in &self.pieces {
    //        println!("  - {:?}", x);
    //    }
    //    println!("  orig:{}\n  apnd:{}", self.original.text.iter().collect::<String>(), self.append.text.iter().collect::<String>());
    //    println!("  FIND {}@ {:?}", idx, self.length_sum.find_prefix(idx));

    //    if idx == 0 { // TODO how to get rid of this special case?
    //        for i in 0..(self.pieces.len()) {
    //            self.length_sum.sub(i,   self.pieces[i].len());
    //            self.length_sum.add(i+1, self.pieces[i].len());
    //        }

    //        let append_idx = self.append.text.len();
    //        self.append.text.push(c);
    //        self.pieces.insert(0, Ref::Append(append_idx,1));
    //        self.length_sum.add(0, 1);
    //    } else {
    //        match self.length_sum.find_prefix(idx) {
    //            Ok(piece_idx) => { // Add to/after end of piece
    //                match self.pieces[piece_idx] {
    //                    Ref::Original(x, l) => {
    //                        // Insert new piece
    //                        println!("NEW PIECE");
    //                        // update length_sum
    //                        for i in (piece_idx+1)..(self.pieces.len()) {
    //                            println!("SHIFT length_sum @{}", i); 
    //                            // Shift pieces ahead
    //                            self.length_sum.sub(i,   self.pieces[i].len());
    //                            self.length_sum.add(i+1, self.pieces[i].len());
    //                        }

    //                        let append_idx = self.append.text.len();
    //                        self.append.text.push(c);
    //                        self.pieces.insert(piece_idx+1, Ref::Append(append_idx,1));
    //                        
    //                        self.length_sum.add(piece_idx+1, 1);
    //                    },
    //                    Ref::Append(x, l) => {
    //                        println!("ADD@PIECE");
    //                        self.append.text.push(c);
    //                        self.pieces[piece_idx] = Ref::Append(x,l+1);
    //                        self.length_sum.add(piece_idx, 1);
    //                    }
    //                }
    //            },
    //            Err(left_idx) => { // need to split piece
    //                if left_idx > (self.pieces.len() -1) { panic!("Insert after end of buffer"); }
    //                let length_before_piece = if left_idx == 0 { 0 }  else { self.length_sum.prefix_sum(left_idx-1 ) };
    //                println!("SPLIT AT {} @{}-{}",left_idx, idx,length_before_piece);
    //                let (before,after) = self.pieces[left_idx].split(idx-length_before_piece);
    //                self.length_sum.sub(left_idx, after.len());

    //                // update length sum after
    //                for i in (left_idx+1)..(self.pieces.len()) {
    //                    println!("SHIFT length_sum @{}",i);
    //                    self.length_sum.sub(i,   self.pieces[i].len());
    //                    self.length_sum.add(i+2, self.pieces[i].len());
    //                }

    //                // left_idx+1 and left_idx+2 is now empty
    //                let append_idx = self.append.text.len();
    //                self.append.text.push(c);
    //                self.pieces[left_idx] = before;
    //                self.pieces.insert(left_idx+1, after);
    //                self.pieces.insert(left_idx+1, Ref::Append(append_idx, 1));
    //                self.length_sum.add(left_idx+1, 1);
    //                self.length_sum.add(left_idx+2, after.len());
    //            },
    //        }
    //    }
    //    println!("Updated piece table:");
    //    for x in &self.pieces {
    //        println!("  - {:?}", x);
    //    }
    //    println!("  orig:{}\n  apnd:{}", self.original.text.iter().collect::<String>(), self.append.text.iter().collect::<String>());
    //}

    pub fn remove(&mut self, idx: usize) {
          println!("OLD piece table:");
          for x in &self.pieces {
              println!("  - {:?}", x);
          }
          println!("  orig:{}\n  apnd:{}", self.original.text.iter().collect::<String>(), self.append.text.iter().collect::<String>());
          println!("  FIND {}@ {:?}", idx, self.length_sum.find_prefix(idx));
      
        let actions = self.remove_actions(idx);
        println!("ACTIONS: {:?}", actions);
        self.run(&actions);
          println!("Updated piece table:");
          for x in &self.pieces {
              println!("  - {:?}", x);
          }
          println!("  orig:{}\n  apnd:{}", self.original.text.iter().collect::<String>(), self.append.text.iter().collect::<String>());
          println!(" FENWICK {:?} len{}", self.length_sum, self.len());
          self.print_prefixes();
    }

    pub fn get(&mut self, idx :usize) -> char {
        println!("Document::get(&mut self, idx :usize) -> char");
          println!(" FENWICK {:?} len{}", self.length_sum, self.len());
          self.print_prefixes();
        let left = self.length_sum.find_prefix_left(idx);
        let prefix = if left > 0 { self.length_sum.prefix_sum(left - 1) }  else { 0 };
        println!("Getting at {}, left {} prefix {}", idx, left, prefix);
        let offset = idx - prefix;
        println!("  (offset:{})",offset);
        self.get_ref(&self.pieces[left])[offset]
    }

    fn foo(n: u32) -> impl Iterator<Item = char> {
        (0..n).map(|x| 'c')
    }

    fn print_prefixes(&self) {
        for i in 0..(self.pieces.len()) {
            println!("piece i len{} cum{}", self.pieces[i].len(), self.length_sum.prefix_sum(i));
        }
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
    fn test_split() {
        let mut doc = Document::new("hallo".to_string());
        doc.insert(2,'x');
        assert_eq!(doc.len(), 6);
        assert_eq!(doc.get(0), 'h');
        assert_eq!(doc.get(1), 'a');
        assert_eq!(doc.get(2), 'x');
        assert_eq!(doc.get(3), 'l');
        assert_eq!(doc.get(4), 'l');
        assert_eq!(doc.get(5), 'o');
    }

    #[test]
    fn test_remove2() {
        let mut doc = Document::empty();
        doc.insert(0,'a');
        doc.insert(1,'b');
        assert_eq!(doc.to_string(),"ab");
        doc.remove(1);
        assert_eq!(doc.to_string(),"a");
    }

    #[test]
    fn test_remove() {
        let mut doc = Document::new("Hallo".to_string());
        doc.remove(0);
        assert_eq!(doc.len(), 4);
        assert_eq!(doc.to_string(), "allo"); 

        doc.insert(0,'H');
        doc.remove(4);
        assert_eq!(doc.len(), 4);
        assert_eq!(doc.to_string(), "Hall"); 
        doc.remove(2);
        assert_eq!(doc.len(), 3);
        assert_eq!(doc.to_string(), "Hal"); 
    }

    #[test]
    fn test_doc_insert() {
        let mut doc = Document::new("hallo".to_string());
        assert_eq!(doc.len(), 5);
        doc.insert(5, 'x');
        assert_eq!(doc.len(), 6);
        doc.insert(6, 'z');
        assert_eq!(doc.len(), 7);
        doc.insert(6, 'y');
        assert_eq!(doc.len(), 8);

        assert_eq!(doc.get(1), 'a');
        doc.insert(1, 'a');
        assert_eq!(doc.get(1), 'a');

        assert_eq!(doc.len(), 9);

        assert_eq!(doc.get(0), 'h');
        assert_eq!(doc.get(1), 'a');
        assert_eq!(doc.get(2), 'a');
        assert_eq!(doc.get(3), 'l');
        assert_eq!(doc.get(4), 'l');
        assert_eq!(doc.get(5), 'o');
        assert_eq!(doc.get(6), 'x');
        assert_eq!(doc.get(7), 'y');
        assert_eq!(doc.get(8), 'z');

        println!("\n\n");

        doc.insert(0,'ö');
        println!("doc:{}",doc.to_string());
        assert_eq!(doc.len(), 10);
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
