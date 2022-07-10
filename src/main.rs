use rand::{seq::SliceRandom, thread_rng};

#[derive(Clone, Copy, PartialEq)]
enum FieldInfo {
    Empty,
    X,
    O
}

const NUM_FIELDS: usize = 3;
type Board = [[FieldInfo; NUM_FIELDS]; NUM_FIELDS];

fn print_board(board: Board) {
    println!("-------");
    for row in 0..NUM_FIELDS {
        print!("|");
        for col in 0..NUM_FIELDS {
            let sign: char = match board[row][col] {
                FieldInfo::Empty => ' ',
                FieldInfo::X => 'X',
                FieldInfo::O => 'O',
            };
            print!("{}|", sign);
        }
        print!("\n");
        println!("-------");
    }
}

fn make_ai_move(board: &mut Board) {
    let mut row_range = (0..NUM_FIELDS).collect::<Vec<usize>>();
    row_range.shuffle(&mut thread_rng());
    for row in row_range {
        let mut col_range = (0..NUM_FIELDS).collect::<Vec<usize>>();
        col_range.shuffle(&mut thread_rng());
        for col in col_range {
            if board[row][col] == FieldInfo::Empty {
                board[row][col] = FieldInfo::O;
                return;
            }
        }
    }

    println!("Couldn't make move - probably game has ended");
}

fn get_input() -> Option<usize> {
    let mut user_input: String = String::new();
    let stdin: std::io::Stdin = std::io::stdin();
    let stdin_res: Result<usize, std::io::Error> = stdin.read_line(&mut user_input);
    match stdin_res {
        Ok(_) => (),
        Err(error) => { 
            println!("Error while handling user input: {:?}", error);
            return None;
        },
    };

    let val: usize = match user_input.trim().parse() {
        Ok(value) => value,
        Err(error) => {
            println!("Incorrect input: {:?}", error);
            return None;
        }
    };

    let val: usize = match val {
        n if (1..4).contains(&n) => n,
        def => {
            println!("Input out of range [1,3]: {}", def);
            return None;
        }
    };

    return Some(val);
}

fn main() {
    println!("Welcome to the Tic-Tac-Toe game!");
    let mut board: Board = [[FieldInfo::Empty; NUM_FIELDS]; NUM_FIELDS];

    loop {
        print_board(board);

        println!("Specify row:");
        let row: Option<usize> = get_input();
        if row.is_none() {
            continue;
        }
        let row: usize = row.unwrap() - 1;

        println!("Specify column:");
        let col: Option<usize> = get_input();
        if col.is_none() {
            continue;
        }
        let col: usize = col.unwrap() - 1;

        if board[row][col] != FieldInfo::Empty {
            println!("Field is not empty!");
            continue;
        }

        board[row][col] = FieldInfo::X;
        make_ai_move(&mut board);
    }
}
