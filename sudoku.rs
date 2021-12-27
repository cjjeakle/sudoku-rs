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

fn sub_board_offset(index: usize) -> usize {
    // use truncating integer division to get the sub-board.
    return index / 3;
}

fn propagate_solution(state: &mut State, sln_row: usize, sln_col: usize, solution: i8) {
    assert!(solution >= 1);
    assert!(solution <= 9);
    let board = &mut state.board;
    let sln_val_index = (solution - 1) as usize;
    // Clear from the row.
    for j in 0..9 {
        board[sln_row][j].possible[sln_val_index] = false;
    }
    // Clear from the col.
    for i in 0..9 {
        board[i][sln_col].possible[sln_val_index] = false;
    }
    // Clear from the sub-board.
    let sub_board_row = sub_board_offset(sln_row);
    let sub_board_col = sub_board_offset(sln_col);
    for i in 0..3 {
        for j in 0..3 {
            board[sub_board_row + i][sub_board_col + j].possible[sln_val_index] = false;
        }
    }
    // Set the solution.
    assert!(state.unsolved_squares > 0);
    state.unsolved_squares -= 1;
    board[sln_row][sln_col].solution = solution;
}

// Finds squares with only one possible solution, and sets that as the square's solution.
// Returns true if any solution as found during this iteration
fn find_and_propagate_singletons(state: &mut State) -> bool {
    let mut any_change: bool = false;
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
                    any_change = true;
                    propagate_solution(state, i, j, (possible_idx + 1) as i8);
                }
            }
        }
    }
    return any_change;
}

fn init(state: &mut State) {
    // Read input.
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(ok) => {}
        Err(error) => println!("error: {}", error),
    }
    let input_bytes: Vec<u8> = input.as_bytes().to_vec();
    assert_eq!(input_bytes.len(), 82); // 81 squares, plus the null byte
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

fn print_board(state: State) {
    println!("unsolved_squares: {}", state.unsolved_squares);
    let mut row_idx = 0;
    state.board.iter().for_each(|row| {
        let mut col_idx = 0;
        row.iter().for_each(|col| {
            if col_idx == 3 || col_idx == 6 {
                print!(" |  ")
            }
            print!("{}", col.solution);
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
    init(&mut state);
    // Run the solver loop until unable to progress.
    while state.unsolved_squares > 0 && find_and_propagate_singletons(&mut state) {}

    print_board(state);
}
