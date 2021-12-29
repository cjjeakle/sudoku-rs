use std::io;
use std::sync::atomic::{AtomicBool, AtomicI8, Ordering};
use std::thread;

fn main() {
    // Get command line args.
    let num_threads_arg = std::env::args()
        .nth(1)
        .expect("Please specify a number of threads via command line arg, e.g. `./sudoku 2`");
    let num_threads = num_threads_arg.parse::<i8>().unwrap();
    // We need at least one thread to do the work.
    assert!(num_threads > 0, "{}", num_threads);
    // Provision state.
    let mut state = State {
        unsolved_squares: 81,
        board: [[Square {
            solution: 0,
            possible: [true; 9],
        }; 9]; 9],
    };
    // Populate givens.
    populate_board_using_input(&mut state);
    // Search for a solution.
    parallel_solve(state, num_threads);
}

/*
State
*/

#[derive(Copy, Clone)]
struct State {
    unsolved_squares: i8,
    board: [[Square; 9]; 9],
}

#[derive(Copy, Clone)]
struct Square {
    solution: i8,
    possible: [bool; 9],
}

impl State {
    // Applies solution to the square at offset row, col.
    // Removes solution as a possibility from the square's peers.
    fn propagate_solution(&mut self, target_row: usize, target_col: usize, solution: i8) {
        assert!(target_row < 9);
        assert!(target_col < 9);
        assert!(solution >= 1);
        assert!(solution <= 9);
        assert!(self.unsolved_squares > 0);
        // Set the solution.
        self.unsolved_squares -= 1;
        self.board[target_row][target_col].solution = solution;
        // Clear all possibilities for the target square.
        for i in 0..9 {
            self.board[target_row][target_col].possible[i] = false;
        }
        let sln_idx = (solution - 1) as usize;
        // Clear option from the row.
        for j in 0..9 {
            self.board[target_row][j].possible[sln_idx] = false;
        }
        // Clear option from the col.
        for i in 0..9 {
            self.board[i][target_col].possible[sln_idx] = false;
        }
        // Clear option from the sub-board.
        let sub_board_row = State::sub_board_offset(target_row);
        let sub_board_col = State::sub_board_offset(target_col);
        for i in 0..3 {
            for j in 0..3 {
                let row = sub_board_row * 3 + i;
                let col = sub_board_col * 3 + j;
                self.board[row][col].possible[sln_idx] = false;
            }
        }
    }

    fn sub_board_offset(index: usize) -> usize {
        // use truncating integer division to get the sub-board.
        return index / 3;
    }
}

/*
Solver
*/

// Returns true if a solution was found, returns false if the provided state is a dead-end.
fn parallel_solve(state: State, max_threads: i8) -> bool {
    // Initialize to 1, to account for the main thread.
    static RUNNING_SOLVER_THREADS: AtomicI8 = AtomicI8::new(1);
    static EXECUTION_CANCELLED: AtomicBool = AtomicBool::new(false);
    // Cancellations are best effort, so use relaxed ordering.
    if EXECUTION_CANCELLED.load(Ordering::Relaxed) {
        return false;
    }
    if state.unsolved_squares > 0 {
        for i in 0..9 {
            for j in 0..9 {
                if state.board[i][j].solution > 0 {
                    // Nothing to do for solved cells.
                    continue;
                }
                let mut child_threads = vec![];
                for sln_idx in 0..9 {
                    if !state.board[i][j].possible[sln_idx] {
                        // Skip invalid possibilities.
                        continue;
                    }
                    // Copy state and try the current solution.
                    let mut state_copy = state.clone();
                    state_copy.propagate_solution(i, j, (sln_idx + 1) as i8);
                    // Use sequentially consistent operations, so we don't spawn too many threads.
                    let threads_running = RUNNING_SOLVER_THREADS.fetch_add(1, Ordering::SeqCst);
                    if threads_running < max_threads {
                        // Fork a solver.
                        child_threads.push(thread::spawn(move || -> bool {
                            let solution_found = parallel_solve(state_copy, max_threads);
                            RUNNING_SOLVER_THREADS.fetch_add(-1, Ordering::SeqCst);
                            return solution_found;
                        }));
                    } else {
                        // Decrement if we don't end up kicking off a thread.
                        RUNNING_SOLVER_THREADS.fetch_add(-1, Ordering::SeqCst);
                        if parallel_solve(state_copy, max_threads) {
                            // If we found a solution, then we're done!
                            return true;
                        }
                    }
                }
                // Wait for all child threads to finish.
                let mut any_solution_found = false;
                for thread_handle in child_threads {
                    any_solution_found |= thread_handle.join().unwrap();
                }
                // If we found no solution for this square, then the branch we're on is a dead-end.
                return any_solution_found;
            }
        }
    } else {
        // Print the solution and cancel other threads.
        print_board(&state);
        // Cancellations are best effort, so use relaxed ordering.
        EXECUTION_CANCELLED.store(true, Ordering::Relaxed);
        return true;
    }
    // We exhaustively searched this state's possibilities and found no solution.
    return false;
}

/*
I/O
*/

fn populate_board_using_input(state: &mut State) {
    // Read input.
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {}
        Err(error) => println!("error: {}", error),
    }
    let input_bytes: Vec<u8> = input.as_bytes().to_vec();
    // 81 squares, plus the null byte.
    assert_eq!(input_bytes.len(), 82);
    // Parse input, propagate givens.
    for i in 0..9 {
        for j in 0..9 {
            let cur_byte = input_bytes[i * 9 + j] as u8;
            if cur_byte >= '1' as u8 && cur_byte <= '9' as u8 {
                state.propagate_solution(i, j, cur_byte as i8 - '0' as i8);
            }
        }
    }
}

fn print_board(state: &State) {
    println!("unsolved_squares: {}", state.unsolved_squares);
    let mut row_idx = 0;
    state.board.iter().for_each(|row| {
        let mut col_idx = 0;
        row.iter().for_each(|col| {
            if col_idx == 3 || col_idx == 6 {
                print!(" |  ")
            }
            if col.solution > 0 {
                print!("{}", col.solution);
            } else {
                print!("_");
            }
            if col_idx < 9 {
                print!(" ");
            }
            col_idx += 1;
        });
        println!("");
        if row_idx == 2 || row_idx == 5 {
            println!("-------------------------");
        } else {
            println!("                         ");
        }
        row_idx += 1;
    });
}
