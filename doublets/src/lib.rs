use crossbeam::crossbeam_channel;
use fnv::FnvHashSet;
use std::sync::atomic::AtomicIsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Instant; // End of crossbeam::scope joins spawned threads.

// Things that don't change during the search and can be immutable.
struct Fixed {
    dict: FnvHashSet<String>,
    // The tail is the word we're trying to reach.
    tail: Vec<char>,
    // Maximum recursion depth = max body.len()
    depth: usize,
    // Concurrency parameters.
    worker_count: usize,
    channel_size: usize,
    start_time: Instant,
}

// The state of the search on this thread. It can be cloned to continue the search on another thread.
#[derive(Clone)]
struct State {
    // Initially the head word, updated during the iteration until it's equal to the tail word.
    word: Vec<char>,
    // The progress so far to the solution, starting with the head word.
    body: Vec<Vec<char>>,
    // The index of the last letter changed, so we don't change it again immediately.
    previous_i: usize,
}

// The queue can contain jobs (with a copy of the search state) or a signal to stop.
enum QueueEntry {
    Job(State),
    Finished,
}

fn to_string(v: &[char]) -> String {
    v.iter().collect()
}

fn print_solutions(fixed: &Fixed, solutions: &[Vec<Vec<char>>]) {
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

// The dictionary lookup is the slowest part. Would another data structure be faster?
fn dict_contains(dict: &FnvHashSet<String>, word: &[char]) -> bool {
    // println!("lookup");
    dict.contains(&to_string(word))
}

// Recursively search for a solution until we reach the specified depth.
// Use the usual depth first backtracking method, unless there's space
// in the job queue - then push a copy of the state and continue with
// the next attempt.
fn find_recursive(
    f: &Fixed,
    s: &mut State,
    running_jobs: &AtomicIsize,
    state_sender: &crossbeam::Sender<QueueEntry>,
    solution_sender: &crossbeam::Sender<Vec<Vec<char>>>,
) {
    // Add this word to the progress so far (on the first call it's the head word).
    s.body.push(s.word.clone());

    // The index of the letter in the word which was changed at the previous recursion level.
    let previous_i = s.previous_i;

    // Iterate through each letter in the word, except the one that was changed previously.
    // Without the previous_i filter it runs about 10x slower.
    for i in (0..f.tail.len()).filter(|&i| i != previous_i) {
        let orig_char = s.word[i];

        // Try substituting each of the rest of the alphabet for the letter at index i.
        for new_char in ('a'..'z').filter(|&c| c != orig_char) {
            s.word[i] = new_char;

            // Check if this is a solution before the dictionary check, in case the tail is not a real word.
            if s.word == f.tail {
                // We've reached the tail word, so add the sequence to the list of solutions.
                solution_sender.send(s.body.clone()).unwrap();

            // Recurse or push to queue if the word is not already used, and is in the dictionary.
            // Check the recursion limit first to avoid expensive hash lookups.
            } else if s.body.len() < f.depth
                && !s.body.contains(&s.word)
                && dict_contains(&f.dict, &s.word)
            {
                s.previous_i = i;

                // Add to the job queue if there's space, else recurse on this thread.
                // The push happens after the size check so it's possible for too many
                // jobs to be pushed, but that's harmless as long as the channel is
                // unbounded.
                // if running_jobs.load(Ordering::SeqCst) > 4 {
                if state_sender.len() >= f.channel_size {
                    find_recursive(f, s, running_jobs, state_sender, solution_sender);
                } else {
                    // Atomically increment the count of running jobs so we know when to finish.
                    // It's decremented once the thread that takes it off the queue has returned
                    // from its call to find_recursive.
                    running_jobs.fetch_add(1, Ordering::SeqCst);

                    // println!(
                    //     "{} {:?} send (",
                    //     f.start_time.elapsed().as_nanos(),
                    //     thread::current().id()
                    // );

                    // Push a copy of the search state onto the job queue.
                    state_sender.send(QueueEntry::Job(s.clone())).unwrap();

                    // println!(
                    //     "{} {:?} sent  )",
                    //     f.start_time.elapsed().as_nanos(),
                    //     thread::current().id()
                    // );
                }
            }
        }

        // Restore the original letter after trying all the others.
        s.word[i] = orig_char;
    }

    s.body.pop();
}

fn start_threads(fixed: &Fixed, initial_state: State) -> Vec<Vec<Vec<char>>> {
    // Create a channel initially containing one item, the initial state.
    // It's unbounded so there's no chance of blocking when pushing new jobs.
    let (state_sender, state_receiver) = crossbeam_channel::unbounded();
    state_sender.send(QueueEntry::Job(initial_state)).unwrap();

    // One job in progress, the initial state. This will be incremented when a new job is queued,
    // and decremented when it's been completed by a worker thread. When it's 0 we've finished.
    let running_jobs = Arc::new(AtomicIsize::new(1));

    // Threads will return solutions through this channel.
    let (solution_sender, solution_receiver) = crossbeam_channel::unbounded();

    // This allows use of variables from this function's scope in the worker threads.
    crossbeam::scope(|scope| {
        // Create the thread pool.
        for _ in 0..fixed.worker_count {
            // Clone objects to pass to the threads.
            // Do it here because they're used in the following loop.
            let state_receiver_clone = state_receiver.clone();
            let state_sender_clone = state_sender.clone();
            let solution_sender_clone = solution_sender.clone();
            let running_jobs_clone = running_jobs.clone();

            scope.spawn(move |_| {
                println!(
                    "{} {:?} create",
                    fixed.start_time.elapsed().as_nanos(),
                    thread::current().id()
                );

                // Loop until all threads have finished working.
                loop {
                    match state_receiver_clone.recv().unwrap() {
                        QueueEntry::Finished => break,
                        QueueEntry::Job(mut state) => {
                            // println!(
                            //     "{} {:?} starting",
                            //     fixed.start_time.elapsed().as_nanos(),
                            //     thread::current().id()
                            // );

                            find_recursive(
                                fixed,
                                &mut state,
                                &running_jobs_clone,
                                &state_sender_clone,
                                &solution_sender_clone,
                            );

                            // println!(
                            //     "{} {:?} waiting",
                            //     fixed.start_time.elapsed().as_nanos(),
                            //     thread::current().id()
                            // );

                            // Atomically decrement the count of running jobs.
                            let prev_running_jobs =
                                running_jobs_clone.fetch_sub(1, Ordering::SeqCst);

                            // If the old count was 1 the new count must be 0:
                            // there's no work left to do, so time to stop the threads.
                            if prev_running_jobs == 1 {
                                for _ in 0..fixed.worker_count {
                                    state_sender_clone.send(QueueEntry::Finished).unwrap();
                                }
                            }
                        }
                    }
                }

                println!(
                    "{} {:?} ended",
                    fixed.start_time.elapsed().as_nanos(),
                    thread::current().id()
                );
            });
        }

        // End of crossbeam::scope joins spawned threads.
    })
    .unwrap();

    println!("threads finished");

    solution_receiver.try_iter().collect()
}

pub fn find(
    head: &str,
    tail: &str,
    dict: FnvHashSet<String>,
    steps: usize,
    worker_count: usize,
    channel_size: usize,
) {
    // Things that don't change during the search.
    let fixed = Fixed {
        tail: tail.to_lowercase().chars().collect(),
        dict,
        // Work out the maximum recursion depth, plus one to include the head word.
        depth: 1 + if steps == 0 { head.len() } else { steps },
        worker_count,
        channel_size,
        start_time: Instant::now(),
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
