use std::io;

#[derive(Copy, Clone)]
struct Square {
    solution: i8,
    possible: [bool; 9],
}

#[derive(Copy, Clone)]
struct State {
    unsolved_squares: i8,
    board: [[Square; 9]; 9],
}

fn main() {
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
    search_for_solution(state, 0, 0);
}

/*
Solver
*/

// Returns true if a solution was found, returns false if the provided state is a dead-end.
// `starting_i` and `starting_j` are used to skip already solved rows and cols.
fn search_for_solution(mut state: State, starting_i: usize, starting_j: usize) -> bool {
    while find_and_propagate_singletons(&mut state) {
        // Repeat `find_and_propagate_singletons` while it makes forward progress.
    }
    if state.unsolved_squares > 0 {
        // If we still haven't solved the board, recursively guess (DFS).
        for i in starting_i..9 {
            for j in starting_j..9 {
                if state.board[i][j].solution > 0 {
                    // Nothing to do for solved cells.
                    continue;
                }
                for possible_idx in 0..9 {
                    if !state.board[i][j].possible[possible_idx] {
                        // Skip invalid possibilities.
                        continue;
                    }
                    // Copy state and try the current solution.
                    let mut state_copy = state.clone();
                    propagate_solution(&mut state_copy, i, j, (possible_idx + 1) as i8);
                    if search_for_solution(state_copy, i, j) {
                        // If we found a solution, then we're done!
                        return true;
                    }
                }
                if state.board[i][j].solution == 0 {
                    // If we found no solution for this square, then we're at a dead-end.
                    return false;
                }
            }
        }
    } else {
        // If we have a solution, then we're done!
        // Print the solution.
        print_board(&state);
        return true;
    }
    // We exhaustively searched this state's possibilities and found no solution.
    return false;
}

// Finds squares with only one possible solution, and sets that as the square's solution.
// Returns true if any solution as found during this iteration, otherwise false.
fn find_and_propagate_singletons(state: &mut State) -> bool {
    // If there are no unsolved squares, then there's nothing to do.
    if state.unsolved_squares == 0 {
        return false;
    }
    let mut any_squares_solved = false;
    for i in 0..9 {
        for j in 0..9 {
            let mut num_possible = 0;
            let mut possible_idx = 0;
            if state.board[i][j].solution == 0 {
                for sln_idx in 0..9 {
                    if state.board[i][j].possible[sln_idx] {
                        num_possible += 1;
                        possible_idx = sln_idx;
                    };
                }
                if num_possible == 1 {
                    any_squares_solved = true;
                    propagate_solution(state, i, j, (possible_idx + 1) as i8)
                }
            }
        }
    }
    return any_squares_solved;
}

// Applies solution to the square at offset row, col.
// Removes solution as a possibility from the square's peers.
fn propagate_solution(state: &mut State, target_row: usize, target_col: usize, solution: i8) {
    assert!(target_row < 9);
    assert!(target_col < 9);
    assert!(solution >= 1);
    assert!(solution <= 9);
    assert!(state.unsolved_squares > 0);
    let board = &mut state.board;
    let possibility_idx = (solution - 1) as usize;
    // Set the solution.
    state.unsolved_squares -= 1;
    board[target_row][target_col].solution = solution;
    // Clear option from the row.
    for j in 0..9 {
        board[target_row][j].possible[possibility_idx] = false;
    }
    // Clear option from the col.
    for i in 0..9 {
        board[i][target_col].possible[possibility_idx] = false;
    }
    // Clear option from the sub-board.
    let sub_board_row = sub_board_offset(target_row);
    let sub_board_col = sub_board_offset(target_col);
    for i in 0..3 {
        for j in 0..3 {
            let row = sub_board_row * 3 + i;
            let col = sub_board_col * 3 + j;
            board[row][col].possible[possibility_idx] = false;
        }
    }
}

/*
Utilities
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
                propagate_solution(state, i, j, cur_byte as i8 - '0' as i8);
            }
        }
    }
}

fn sub_board_offset(index: usize) -> usize {
    // use truncating integer division to get the sub-board.
    return index / 3;
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
