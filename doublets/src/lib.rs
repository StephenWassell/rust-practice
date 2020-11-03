use crossbeam::crossbeam_channel;
use fnv::FnvHashSet;
use std::sync::Arc;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering;

// Things that don't change during the search and can be immutable.
struct Fixed {
    dict: FnvHashSet<String>,
    // The word we're trying to reach.
    tail: Vec<char>,
    // Maximum recursion depth = max body.len()
    depth: usize,
}

// The state of the search on this thread. It can be cloned to continue the search on another thread.
struct State {
    // Initially the head word, updated during the iteration until it's equal to the tail word.
    word: Vec<char>,
    // The progress so far to the solution, starting with the head word.
    body: Vec<Vec<char>>,
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
fn find_rec(
    f: &Fixed,
    s: &mut State,
    running_jobs: &AtomicIsize,
    state_sender: &crossbeam::Sender<State>,
    solution_sender: &crossbeam::Sender<Vec<Vec<char>>>,
) {
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
                // It's a solution, add to the list to return.
                solution_sender.send(s.body.clone()).unwrap();
            // Recurse if the word is not already used, and is in the dictionary.
            // Check the recursion limit first to avoid expensive lookups.
            } else if s.body.len() < f.depth
                && !s.body.contains(&s.word)
                && f.dict.contains(&to_string(&s.word))
            {
                s.previous_i = i;
                // todo: queue or recurse? running_jobs++, state_sender.send
                find_rec(f, s, running_jobs, state_sender, solution_sender);
            }
        }

        // Restore the original letter after trying all the others.
        s.word[i] = orig_char;
    }

    s.body.pop();
}

fn start_threads(fixed: &Fixed, initial_state: State) -> Vec<Vec<Vec<char>>> {
    let worker_count = 1; // todo: num of cores

    // Create a bounded channel initially containing one item, the initial state.
    let (state_sender, state_receiver) = crossbeam_channel::bounded(worker_count);
    state_sender.send(initial_state).unwrap();

    // One job in progress, the initial state. This will be incremented when a new job is queued,
    // and decremented when it's been completed by a worker thread. When it's 0 we've finished.
    let running_jobs = Arc::new(AtomicIsize::new(1));

    // Threads will return solutions through this channel.
    let (solution_sender, solution_receiver) = crossbeam_channel::unbounded();

    crossbeam::scope(|s| {
        // Create the thread pool.
        let mut workers = Vec::new();
        for _ in 0..worker_count {
            // Clone both ends of the state channel to pass to the threads.
            let state_receiver_clone = state_receiver.clone();
            let state_sender_clone = state_sender.clone();

            let running_jobs_clone = running_jobs.clone();

            let solution_sender_clone = solution_sender.clone();

            workers.push(s.spawn(move |_| {
                // todo: loop until all threads have finished working

                println!("started a thread");

                let mut state = state_receiver_clone.recv().unwrap();
                find_rec(fixed, &mut state, &running_jobs_clone, &state_sender_clone, &solution_sender_clone);

                running_jobs_clone.fetch_sub(1, Ordering::SeqCst);
                // todo: if it's now 0 stop all threads

                println!("ended a thread");
            }));
        }

        for worker in workers {
            worker.join().unwrap();
        }
    })
    .unwrap();

    println!("threads finished");

    solution_receiver.try_iter().collect()
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
