use crossbeam::scope;
use crossbeam::crossbeam_channel::bounded;
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
    // Solutions found on this thread: Vec of body Vecs.
    solutions: Vec<Vec<Vec<char>>>,
    // The index of the last letter changed, so we don't change it again immediately.
    previous_i: usize,
}

fn to_string(v: &[char]) -> String {
    v.iter().collect()
}

fn print_solutions(fixed: &Fixed, solutions: &Vec<Vec<Vec<char>>>) {
    for body in solutions {
        // The first word in the body vector is the original head.
        // Print the head and tail in uppercase, the rest in lowercase.
        print!("{} ", to_string(&body[0]).to_uppercase());

        for word in body.iter().skip(1) {
            print!("{} ", to_string(&word));
        }

        println!(
            "{} ({} steps)",
            to_string(&fixed.tail).to_uppercase(),
            body.len() - 1
        );
    }
}

// Recursively search for a solution until we reach the specified depth.
// Use the usual depth first backtracking method.
fn find_rec(f: &Fixed, s: &mut State) {
    // Add this word to the progress so far (on the first call it's the head word).
    s.body.push(s.word.clone());

    let previous_i = s.previous_i;

    // Iterate through each letter in the word, except the one that was changed at the previous level.
    // Without the filter it runs about 10x slower.
    for i in (0..f.tail.len()).filter(|&i| i != previous_i) {
        let orig_char = s.word[i];

        // Try substituting each of the rest of the alphabet.
        for new_char in ('a'..'z').filter(|&c| c != orig_char) {
            s.word[i] = new_char;

            // Check if this is a solution before the dictionary check, in case the tail is not a real word.
            if s.word == f.tail {
                // print_solution(f, s);
                // let solution = s.body.clone();
                s.solutions.push(s.body.clone());
            // Recurse if the word is not already used, and is in the dictionary.
            // Check the recursion limit first to avoid expensive lookups.
            } else if s.body.len() < f.depth
                && !s.body.contains(&s.word)
                && f.dict.contains(&to_string(&s.word))
            {
                s.previous_i = i;
                // todo: queue or recurse?
                find_rec(f, s);
            }
        }

        // Restore the original letter after trying all the others.
        s.word[i] = orig_char;
    }

    s.body.pop();
}

fn start_threads(fixed: &Fixed, initial_state: State) -> Vec<Vec<Vec<char>>> {
    let mut all_solutions = Vec::new();
    let worker_count = 1; // todo: num of cores
    
    scope(|s| {
        let (sender, receiver) = bounded(worker_count);
        sender.send(initial_state).unwrap();
        
        let mut workers = Vec::new();
        for _ in 0..worker_count {
            let receiver_clone = receiver.clone();

            workers.push(s.spawn(move |_| {
                let mut solutions = Vec::new();

                let mut state = receiver_clone.recv().unwrap();
                find_rec(fixed, &mut state);
                solutions.append(&mut state.solutions);

                solutions
            }))
        }

        for worker in workers {
            let mut solutions = worker.join().unwrap();
            all_solutions.append(&mut solutions);
        }
    }).unwrap();

    all_solutions
}

pub fn find(head: &str, tail: &str, dict: FnvHashSet<String>, steps: usize) {
    // Things that don't change during the search.
    let fixed = Fixed {
        tail: tail.to_lowercase().chars().collect(),
        dict,
        // Work out the maximum recursion depth, plus one to include the head word.
        depth: 1 + if steps == 0 { head.len() } else { steps },
    };

    // The changing part of the shared state. It's updated during the recursion
    // and only cloned to put in the queue for other threads.
    let initial_state = State {
        word: head.to_lowercase().chars().collect(),
        body: Vec::with_capacity(fixed.depth),
        solutions: Vec::new(),
        // On the first call, previous_i must be outside the range of the string.
        previous_i: head.len(),
    };

    // Start the recursive search.
    let solutions = start_threads(&fixed, initial_state);

    print_solutions(&fixed, &solutions);

    println!(
        "Found {} solutions up to {} steps.",
        solutions.len(),
        fixed.depth - 1
    );
}
