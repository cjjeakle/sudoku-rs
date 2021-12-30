use std::cmp;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::thread;

fn main() {
    // Get command line args.
    let num_threads_arg = std::env::args()
        .nth(1)
        .expect("Please specify a number of threads via command line arg, e.g. `./sudoku 2`");
    let num_threads = num_threads_arg.parse::<isize>().unwrap();
    // We need at least one thread to do the work.
    assert!(num_threads > 0, "{}", num_threads);
    // Provision state.
    let mut state = State {
        unsolved_squares: 81,
        board: [[Square {
            solution: 0,
            num_possible: 9,
            possible: [true; 9],
        }; 9]; 9],
    };
    // Populate givens.
    populate_board_using_input(&mut state);
    // Search for a solution.
    parallel_solve(state, 0, 0, num_threads);
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
    num_possible: i8,
    possible: [bool; 9],
}

impl State {
    // Applies solution to the square at offset row, col.
    // Removes solution as a possibility from the square's peers.
    // Returns true if the board remains valid after the solution was applied, false otherwise.
    fn propagate_solution(&mut self, target_row: usize, target_col: usize, solution: i8) -> bool {
        assert!(target_row < 9);
        assert!(target_col < 9);
        assert!(solution >= 1);
        assert!(solution <= 9);
        assert!(self.unsolved_squares > 0);
        // Set the solution.
        self.unsolved_squares -= 1;
        self.board[target_row][target_col].solution = solution;
        // Clear all possibilities for the target square.
        self.board[target_row][target_col].num_possible = 0;
        for i in 0..9 {
            self.board[target_row][target_col].possible[i] = false;
        }
        // Clear option across the row.
        for j in 0..9 {
            if !self.remove_possibility(target_row, j, solution) {
                return false;
            }
        }
        // Clear option up and down the col.
        for i in 0..9 {
            if !self.remove_possibility(i, target_col, solution) {
                return false;
            }
        }
        // Clear option throughout the sub-board.
        let sub_board_row = State::sub_board_offset(target_row);
        let sub_board_col = State::sub_board_offset(target_col);
        for i in 0..3 {
            for j in 0..3 {
                let row = sub_board_row * 3 + i;
                let col = sub_board_col * 3 + j;
                if !self.remove_possibility(row, col, solution) {
                    return false;
                }
            }
        }
        return true;
    }

    // Removes the possibility from the specified square.
    // Returns whether the square remains valid/viable afterward.
    fn remove_possibility(&mut self, row: usize, col: usize, solution: i8) -> bool {
        assert!(row < 9);
        assert!(col < 9);
        assert!(solution > 0);
        assert!(solution <= 9);
        let peer_cell = &mut self.board[row][col];
        let possibility_idx = (solution - 1) as usize;
        if peer_cell.possible[possibility_idx] {
            peer_cell.num_possible -= 1;
            peer_cell.possible[possibility_idx] = false;
        }
        return peer_cell.is_valid();
    }

    fn sub_board_offset(index: usize) -> usize {
        // use truncating integer division to get the sub-board.
        return index / 3;
    }
}

impl Square {
    fn is_valid(&self) -> bool {
        // To be valid, squares need a solution or candidate solutions.
        return self.solution > 0 || self.num_possible > 0;
    }
}

/*
Solver
*/

// Returns true if a solution was found, returns false if the provided state is a dead-end.
// Skips past squares before `row`, `col` (in {row,col} order).
fn parallel_solve(state: State, mut row: usize, mut col: usize, max_threads: isize) -> bool {
    assert!(max_threads > 0);
    static EXECUTION_CANCELLED: AtomicBool = AtomicBool::new(false);
    // Cancellations are best effort, so use `Ordering::Relaxed`.
    if EXECUTION_CANCELLED.load(Ordering::Relaxed) {
        return false;
    }
    if state.unsolved_squares > 0 {
        while row < 9 {
            while col < 9 {
                if state.board[row][col].solution > 0 {
                    // Nothing to do for solved cells.
                    col += 1;
                    continue;
                }
                if !state.board[row][col].is_valid() {
                    // If any square is invalid, then this branch is a dead-end.
                    return false;
                }
                return parallel_solve_impl(state, row, col, max_threads);
            }
            row += 1;
            col = 0;
        }
    } else {
        // Print the solution and cancel other threads.
        print_board(&state);
        // Cancellations are best effort, so use `Ordering::Relaxed`.
        EXECUTION_CANCELLED.store(true, Ordering::Relaxed);
        return true;
    }
    // We should not get to this point.
    assert!(false);
    return false;
}

// Tests solutions for a given square, and recursively searches onward from each candidate solution.
// Can kick off up to `max_threads` (including the main thread) across all ongoing searches.
fn parallel_solve_impl(state: State, row: usize, col: usize, max_threads: isize) -> bool {
    // Initialize to 1, to account for the main thread.
    static THREAD_QUOTA_IN_USE: AtomicIsize = AtomicIsize::new(1);
    // Request 1 thread per possibility, don't forget to account for the current thread.
    let extra_threads_to_request: isize = (state.board[row][col].num_possible - 1).into();
    // Use `Ordering::SeqCst` to ensure `RUNNING_SOLVER_THREADS` is accurate.
    let thread_quota_available = cmp::max(
        max_threads - THREAD_QUOTA_IN_USE.fetch_add(extra_threads_to_request, Ordering::SeqCst),
        0,
    );
    let mut extra_threads_available = cmp::min(thread_quota_available, extra_threads_to_request);
    let overflow = cmp::max(extra_threads_to_request - extra_threads_available, 0);
    if overflow > 0 {
        THREAD_QUOTA_IN_USE.fetch_add(-overflow, Ordering::SeqCst);
    }
    let mut child_threads = vec![];
    for sln_idx in 0..9 {
        if !state.board[row][col].possible[sln_idx] {
            // Skip invalid possibilities.
            continue;
        }
        // Copy state and try a candidate solution for this square.
        let mut state_copy = state.clone();
        state_copy.propagate_solution(row, col, (sln_idx + 1) as i8);
        // If there are multiple possibilities to explore, and there are threads available,
        // spawn a solver in another thread.
        if extra_threads_available > 0 {
            extra_threads_available -= 1;
            child_threads.push(thread::spawn(move || -> bool {
                let solution_found = parallel_solve(state_copy, row, col + 1, max_threads);
                THREAD_QUOTA_IN_USE.fetch_add(-1, Ordering::SeqCst);
                return solution_found;
            }));
        } else {
            // Decrement if we don't end up kicking off a thread.
            if parallel_solve(state_copy, row, col + 1, max_threads) {
                // If we found a solution, then we're done!
                return true;
            }
        }
    }
    // Wait for all child threads to finish.
    for thread_handle in child_threads {
        // If any child thread found a solution, then we're done!
        if thread_handle.join().unwrap() { return true; }
    }
    // If we found no solution for this square, then the branch we're on is a dead-end.
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
                assert!(state.propagate_solution(i, j, cur_byte as i8 - '0' as i8));
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
