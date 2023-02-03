use std::{env, fs, process, sync::{Mutex, Arc}};
use maze::maze;

const USAGE: &str = "maze <file_path>\n\nInputs:\n\tfile_path: Path to a file that contains the maze.";
const MAZE_X: u8 = 9;
const MAZE_Y: u8 = 6;

fn read_fields(content: &String) -> (Vec<maze::Field>, Vec<maze::Field>) {
    let mut fields: Vec<maze::Field> = Vec::new();
    let mut ends: Vec<maze::Field> = Vec::new();
    let (mut x, mut y, mut key, mut end) = (0, 0, false, false);
    content.chars().enumerate().for_each(|(i, c)| {
        // println!("{}:{}", i, c);
        if i == 0 { // TODO remove?
            return;
        }
        if i % 15 == 14 {
            let f = maze::Field::new(Mutex::new(maze::SimpleField::new(x, y, key, end)));
            // println!("{}, {}: {}, {}", i, f, f.has_key(), f.is_end());
            fields.push(Arc::clone(&f));
            if end {
                ends.push(Arc::clone(&f));
            }
            if x == MAZE_X - 1 {
                x = 0;
                y = y + 1;
            } else {
                x = x + 1;
            }
        } else if i % 15 == 10 {
            key = c == '1';
        } else if i % 15 == 11 {
            key = key && c == '1';
        } else if i % 15 == 12 {
            end = c == '1';
        } else if i % 15 == 13 {
            end = end && c == '1';
        }
    });
    (fields, ends)
}

fn get_index(x: u8, y: u8) -> usize {
    return (y * MAZE_X + x).into();
}

fn get_move(x: u8, y: u8, direction: maze::Direction) -> Option<usize> {
    match direction {
        maze::Direction::WEST => {
            if x == 0 {
                None
            } else {
                Some(get_index(x - 1, y))
            }
        },
        maze::Direction::EAST => {
            if x == MAZE_X - 1{
                None
            } else {
                Some(get_index(x + 1, y))
            }
        },
        maze::Direction::NORTH => {
            if y == 0 {
                None
            } else {
                Some(get_index(x, y - 1))
            }
        },
        maze::Direction::SOUTH => {
            if y == MAZE_Y - 1 {
                None
            } else {
                Some(get_index(x, y + 1))
            }
        },
    }
}

fn tie_fields(content: &String, fields: &mut Vec<maze::Field>) {
    let (mut x, mut y, mut w, mut e, mut n, mut s, mut wd, mut ed, mut nd, mut sd) = (0, 0, false, false, false, false, false, false, false, false);
    content.chars().enumerate().for_each(|(i, c)| {
        // println!("{}:{}", i, c);
        if i % 15 == 14 {
            // println!("({}, {}): {}, {}, {}, {}", x, y, w, e, n, s);
            let f1 = get_index(x, y);
            let rf1 = fields.get(f1).unwrap();
            if w {
                let f2 = get_move(x, y, maze::Direction::WEST);
                if f2.is_some() {
                    let rf2 = fields.get(f2.unwrap());
                    if rf2.is_some() {
                        println!("Tying WEST: {} -> {} {}", rf1.lock().unwrap(), rf2.unwrap().lock().unwrap(), wd);
                        maze::Transition::new(wd, &maze::Direction::WEST, Arc::clone(rf1), Arc::clone(rf2.unwrap()));
                    }
                }
            }
            if e {
                let f2 = get_move(x, y, maze::Direction::EAST);
                if f2.is_some() {
                    let rf2 = fields.get(f2.unwrap());
                    if rf2.is_some() {
                        println!("Tying EAST: {} -> {} {}", rf1.lock().unwrap(), rf2.unwrap().lock().unwrap(), ed);
                        maze::Transition::new(ed, &maze::Direction::EAST, Arc::clone(rf1), Arc::clone(rf2.unwrap()));
                    }
                }
            }
            if n {
                let f2 = get_move(x, y, maze::Direction::NORTH);
                if f2.is_some() {
                    let rf2 = fields.get(f2.unwrap());
                    if rf2.is_some() {
                        println!("Tying NORTH: {} -> {} {}", rf1.lock().unwrap(), rf2.unwrap().lock().unwrap(), nd);
                        maze::Transition::new(nd, &maze::Direction::NORTH, Arc::clone(rf1), Arc::clone(rf2.unwrap()));
                    }
                }
            }
            if s {
                let f2 = get_move(x, y, maze::Direction::SOUTH);
                if f2.is_some() {
                    let rf2 = fields.get(f2.unwrap());
                    if rf2.is_some() {
                        println!("Tying SOUTH: {} -> {} {}", rf1.lock().unwrap(), rf2.unwrap().lock().unwrap(), sd);
                        maze::Transition::new(sd, &maze::Direction::SOUTH, Arc::clone(rf1), Arc::clone(rf2.unwrap()));
                    }
                }
            }

            if x == MAZE_X - 1 {
                x = 0;
                y = y + 1;
            } else {
                x = x + 1;
            }
        } else if i % 15 == 0 {
            w = c == '1';
        } else if i % 15 == 1 {
            e = c == '1';
        } else if i % 15 == 2 {
            n = c == '1';
        } else if i % 15 == 3 {
            s = c == '1';
        } else if i % 15 == 5 {
            wd = c == '1';
        } else if i % 15 == 6 {
            ed = c == '1';
        } else if i % 15 == 7 {
            nd = c == '1';
        } else if i % 15 == 8 {
            sd = c == '1';
        }
    });
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!("{}", USAGE);
        return 
    }
    let file_path = &args[1];

    let content: String;
    match fs::read_to_string(file_path) {
        Ok(s) => content = s,
        Err(_) => {
            println!("Unable to read file: {}", file_path);
            process::exit(1)
        },
    }

    let (mut fields, ends) = read_fields(&content);
    tie_fields(&content, &mut fields);
    // let p = maze::has_path(Arc::clone(&fields[0]), Arc::clone(&fields[47]));
    let p = maze::min_path(Arc::clone(&fields[0]), ends);
    if p.is_some() {
        p.unwrap().print_path();
    } else {
        println!("Path not found.")
    }
}
