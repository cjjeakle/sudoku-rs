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
    search_for_solution(state);
}

/*
Solver
*/

// Returns true if a solution was found, returns false if the provided state is a dead-end.
fn search_for_solution(state: State) -> bool {
    if state.unsolved_squares > 0 {
        for i in 0..9 {
            for j in 0..9 {
                if state.board[i][j].solution > 0 {
                    // Nothing to do for solved cells.
                    continue;
                }
                for sln_idx in 0..9 {
                    if !state.board[i][j].possible[sln_idx] {
                        // Skip invalid possibilities.
                        continue;
                    }
                    // Copy state and try the current solution.
                    let mut state_copy = state.clone();
                    propagate_solution(&mut state_copy, i, j, (sln_idx + 1) as i8);
                    if search_for_solution(state_copy) {
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

// Applies solution to the square at offset row, col.
// Removes solution as a possibility from the square's peers.
fn propagate_solution(state: &mut State, target_row: usize, target_col: usize, solution: i8) {
    assert!(target_row < 9);
    assert!(target_col < 9);
    assert!(solution >= 1);
    assert!(solution <= 9);
    assert!(state.unsolved_squares > 0);
    let board = &mut state.board;
    // Set the solution.
    state.unsolved_squares -= 1;
    board[target_row][target_col].solution = solution;
    // Clear all possibilities for the target square.
    for i in 0..9 {
        board[target_row][target_col].possible[i] = false;
    }
    let sln_idx = (solution - 1) as usize;
    // Clear option from the row.
    for j in 0..9 {
        board[target_row][j].possible[sln_idx] = false;
    }
    // Clear option from the col.
    for i in 0..9 {
        board[i][target_col].possible[sln_idx] = false;
    }
    // Clear option from the sub-board.
    let sub_board_row = sub_board_offset(target_row);
    let sub_board_col = sub_board_offset(target_col);
    for i in 0..3 {
        for j in 0..3 {
            let row = sub_board_row * 3 + i;
            let col = sub_board_col * 3 + j;
            board[row][col].possible[sln_idx] = false;
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
