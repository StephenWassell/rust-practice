use fnv::FnvHashSet;

// Things that don't change during the search and can be immutable.
struct Fixed {
    dict: FnvHashSet<String>,
    // The word we're trying to reach.
    tail: Vec<char>,
    // Maximum recursion depth = max body.len()
    depth: usize,
}

struct State {
    // Initially the head word, updated during the iteration until it's equal to the tail word.
    word: Vec<char>,
    // The progress so far to the solution, starting with the head word.
    body: Vec<Vec<char>>,
    // How many solutions were found.
    count: usize,
}

fn to_string(v: &[char]) -> String {
    v.iter().collect()
}

fn print_solution(f: &Fixed, s: &State) {
    // The first word in the body vector is the original head.
    // Print the head and tail in uppercase, the rest in lowercase.
    print!("{} ", to_string(&s.body[0]).to_uppercase());

    for word in s.body.iter().skip(1) {
        print!("{} ", to_string(&word));
    }

    println!(
        "{} ({} steps)",
        to_string(&f.tail).to_uppercase(),
        s.body.len() - 1
    );
}

// Recursively search for a solution until we reach the specified depth.
// Use the usual depth first backtracking method.
fn find_rec(f: &Fixed, s: &mut State, previous_i: usize) {
    // Add this word to the progress so far (on the first call it's the head word).
    s.body.push(s.word.clone());

    // Iterate through each letter in the word, except the one that was changed at the previous level.
    // Without the filter it runs about 10x slower.
    for i in (0..f.tail.len()).filter(|&i| i != previous_i) {
        let orig_char = s.word[i];

        // Try substituting each of the rest of the alphabet.
        for new_char in ('a'..'z').filter(|&c| c != orig_char) {
            s.word[i] = new_char;

            // Check if this is a solution before the dictionary check, in case the tail is not a real word.
            if s.word == f.tail {
                print_solution(f, s);
                s.count += 1;
            // Recurse if the word is not already used, and is in the dictionary.
            } else if s.body.len() < f.depth
                && !s.body.contains(&s.word)
                && f.dict.contains(&to_string(&s.word))
            {
                find_rec(f, s, i);
            }
        }

        s.word[i] = orig_char;
    }

    s.body.pop();
}

pub fn find(head: &str, tail: &str, dict: FnvHashSet<String>, steps: usize) {
    let fixed = Fixed {
        tail: tail.to_lowercase().chars().collect(),
        dict,
        // Work out the maximum recursion depth, plus one to include the head word.
        depth: 1 + if steps == 0 { head.len() } else { steps },
    };

    let mut state = State {
        word: head.to_lowercase().chars().collect(),
        body: Vec::new(),
        count: 0,
    };

    state.body.reserve(fixed.depth);

    // Start the recursive search. On the first call, previous_i must be outside the range of the string.
    find_rec(&fixed, &mut state, head.len());

    println!(
        "Found {} solutions up to {} steps.",
        state.count,
        fixed.depth - 1
    );
}
