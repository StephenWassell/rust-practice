use std::collections::HashSet;

fn find_rec(head: &mut Vec<u8>, tail: &Vec<u8>, words: &HashSet<String>, depth: usize) {

    for i in 0..head.len() {
        let orig = head[i];
        for j in (('a' as u8)..('z' as u8)).filter(|&c| c != orig) {
            head[i] = j;
            let head_str = std::str::from_utf8(&head).unwrap();
            if words.contains(head_str) {
                println!("{} {}", i, head_str);
            }
        }
        head[i] = orig;
    }
}

pub fn find(head: &str, tail: &str, words: &HashSet<String>, depth: usize) {
    let mut head_bytes: Vec<u8> = head.as_bytes().iter().map(|c| *c).collect();
    let tail_bytes: Vec<u8> = tail.as_bytes().iter().map(|c| *c).collect();

    find_rec(&mut head_bytes, &tail_bytes, words, depth);
}
