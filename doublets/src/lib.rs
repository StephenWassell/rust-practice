use fnv::FnvHashSet;

struct State {
    // Initially the head word, updated during the iteration until it's equal to the tail word.
    word: Vec<char>,
    // The word we're trying to reach.
    tail: Vec<char>,
    // The progress so far to the solution, starting with the head word.
    body: Vec<Vec<char>>,
    // The length of the words in chars.
    len: usize,
    // How many solutions were found.
    count: usize,
    // Remaining recursion depth, stop when it reaches 0.
    depth: usize,
}

fn to_string(v: &Vec<char>) -> String {
    v.iter().collect()
}

fn print_solution(s: &State) {
    // The first word in the body vector is the original head.
    // Print the head and tail in uppercase, the rest in lowercase.
    print!("{} ", to_string(&s.body[0]).to_uppercase());

    for word in s.body.iter().skip(1) {
        print!("{} ", to_string(&word));
    }

    println!(
        "{} ({} steps)",
        to_string(&s.tail).to_uppercase(),
        s.body.len() - 1
    );
}

// Recursively search for a solution until we reach the specified depth.
// Use the usual depth first backtracking method.
fn find_rec(s: &mut State, previous_i: usize, dict: &FnvHashSet<String>) {
    if s.depth == 0 {
        return;
    }

    s.depth -= 1;

    // Add this word to the progress so far (on the first call it's the head word).
    s.body.push(s.word.clone());

    // Iterate through each letter in the word, except the one that was changed at the previous level.
    // Without the filter it runs about 10x slower.
    for i in (0..s.len).filter(|&i| i != previous_i) {
        let orig_char = s.word[i];

        // Try substituting each of the rest of the alphabet.
        for new_char in ('a'..'z').filter(|&c| c != orig_char) {
            s.word[i] = new_char;

            // Check if this is a solution before the dictionary check, in case the tail is not a real word.
            if s.word == s.tail {
                print_solution(s);
                s.count += 1;
            // Recurse if the word is not already used, and is in the dictionary.
            } else if !s.body.contains(&s.word) && dict.contains(&to_string(&s.word)) {
                find_rec(s, i, dict);
            }
        }

        s.word[i] = orig_char;
    }

    s.body.pop();
    s.depth += 1;
}

pub fn find(head: &str, tail: &str, dict: FnvHashSet<String>, steps: usize) {
    let mut state = State {
        word: head.to_lowercase().chars().collect(),
        tail: tail.to_lowercase().chars().collect(),
        body: Vec::new(),
        len: head.len(),
        count: 0,
        // Work out the maximum recursion depth, plus one to include the head word.
        depth: 1 + if steps == 0 { head.len() } else { steps },
    };

    state.body.reserve(state.depth);

    // Start the recursive search. On the first call, previous_i must be outside the range of the string.
    find_rec(&mut state, head.len(), &dict);

    println!("Found {} solutions up to {} steps.", state.count, steps);
}
