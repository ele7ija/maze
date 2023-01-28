use core::fmt;
use std::{option::Option, rc::Rc, cell::{RefCell}};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    WEST,
    EAST,
    NORTH,
    SOUTH
}

impl Direction {
    fn get_opposite(&self) -> Direction {
        match self {
            Direction::WEST => Direction::EAST,
            Direction::EAST => Direction::WEST,
            Direction::NORTH => Direction::SOUTH,
            Direction::SOUTH => Direction::NORTH,
        }
    }
}

pub struct Field {
    x: u8,
    y: u8,
    w: OptionalTransition,
    e: OptionalTransition,
    n: OptionalTransition,
    s: OptionalTransition,
    key: bool,
    end: bool,
}

impl Field {
    pub fn new(x: u8, y: u8, key: bool, end: bool) -> Self {
        Field {
            x: x,
            y: y,
            w: None,
            e: None,
            n: None,
            s: None,
            key: key,
            end: end,
        }
    }

    pub fn add_transition(&mut self, direction: &Direction, transition: Rc<RefCell<Transition>>) {
        let f = Some(transition);
        match direction {
            Direction::WEST => self.w = f,
            Direction::EAST => self.e = f,
            Direction::NORTH => self.n = f,
            Direction::SOUTH => self.s = f,
        }
    }

    pub fn get_transition(&self, direction: Direction) -> Option<Rc<RefCell<Transition>>> {
        let t: &Option<Rc<RefCell<Transition>>>;
        match direction {
            Direction::WEST => t = &self.w,
            Direction::EAST => t = &self.e,
            Direction::NORTH => t = &self.n,
            Direction::SOUTH => t = &self.s,
        }
        if let Some(real_t) = t {
            return Some(Rc::clone(real_t));
        }
        None
    }

    pub fn has_key(&self) -> bool {
        self.key
    }

    pub fn is_end(&self) -> bool {
        self.end
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl PartialEq for Field {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

pub struct Transition {
    doors: bool,
    field1: Rc<RefCell<Field>>,
    field2: Rc<RefCell<Field>>,
}

impl Transition {
    fn new(doors: bool, direction: &Direction, field1: Rc<RefCell<Field>>, field2: Rc<RefCell<Field>>) -> Rc<RefCell<Self>> {
        let t = Transition {
            doors: doors,
            field1: Rc::clone(&field1),
            field2: Rc::clone(&field2),
        };
        let rt = Rc::new(RefCell::new(t));

        let mut f1 = field1.borrow_mut();
        let mut f2 = field2.borrow_mut();
        f1.add_transition(direction, Rc::clone(&rt));
        f2.add_transition(&direction.get_opposite(), Rc::clone(&rt));
        rt
    }

    pub fn has_doors(&self) -> bool {
        self.doors
    }

    pub fn get_field1(&self) -> Rc<RefCell<Field>> {
        Rc::clone(&self.field1)
    }

    pub fn get_field2(&self) -> Rc<RefCell<Field>> {
        Rc::clone(&self.field2)
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut t = "";
        if self.doors {
            t = "|";
        }
        write!(f, "{} <-{}-> {}", self.get_field1().borrow(), t, self.get_field2().borrow())
    }
}

impl fmt::Debug for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut t = "";
        if self.doors {
            t = "|";
        }
        write!(f, "{} <-{}-> {}", self.get_field1().borrow(), t, self.get_field2().borrow())
    }
}

impl PartialEq for Transition {
    fn eq(&self, other: &Self) -> bool {
        return (self.get_field1() == other.get_field1() && self.get_field2() == other.get_field2()) ||
            (self.get_field2() == other.get_field1() && self.get_field1() == other.get_field2())
    }
}

type OptionalTransition = Option<Rc<RefCell<Transition>>>;

pub struct Path {
    steps: Vec<Direction>,
}

impl Path {
    fn cost(&self) -> usize {
        return self.steps.len();
    }
    fn print_path(&self) {
        let n = self.steps.len();
        self.steps.iter().rev().enumerate().for_each(|(i, x)| {
            print!("{:?}", x);
            if i != n - 1 {
                print!(" -> ");
            } else {
                println!();
            }
        })
    }
    fn add_step(&mut self, step: Direction) {
        self.steps.push(step);
    }
}

fn has_path(f1: Rc<RefCell<Field>>, f2: Rc<RefCell<Field>>) -> Option<Path> {
    return has_path_keys(f1, f2, 0, &mut Vec::new());
}

fn has_path_keys(f1: Rc<RefCell<Field>>, f2: Rc<RefCell<Field>>, mut keys: i8, transitions: &mut Vec<Rc<RefCell<Transition>>>) -> Option<Path> {
    println!("Comparing: {:} and {:}", f1.borrow(), f2.borrow());
    if f1 == f2 {
        return Some(Path { steps: Vec::new() });
    }
    let bf1 = f1.borrow();
    if bf1.has_key() {
        keys += 1;
        println!("keys: {} (+1)", keys);
    }
    let directions = [Direction::WEST, Direction::EAST, Direction::NORTH, Direction::SOUTH];
    let mut path: Option<Path> = None;
    for d in directions {
        let t_pos = bf1.get_transition(d);
        if t_pos.is_none() {
            continue;
        }
        println!("going {:?}", d);
        let t_ptr = t_pos.unwrap();
        if !transitions.contains(&Rc::clone(&t_ptr)) {
            let t = t_ptr.borrow();
            let (doors, f) = (t.doors, Rc::clone(&t.get_field2()));
                let mut keys_left = keys;
                if doors {
                    keys_left -= 1;
                    println!("keys: {} (-1)", keys_left);
                }
                if keys_left == -1 {
                    println!("no keys left!");
                } else {
                    transitions.push(Rc::clone(&t_ptr));
                    println!("transitions expanded to: {:?}", transitions);
                    if let Some(mut steps) = has_path_keys(f, Rc::clone(&f2), keys_left, transitions) {
                        if let Some(curr_path) = &path {
                            if steps.cost() + 1 < curr_path.cost() {
                                steps.add_step(d);
                                path = Some(steps);
                            }
                        } else {
                            steps.add_step(d);
                            path = Some(steps);
                        }
                    }
                    transitions.pop();
                    println!("Removing from transitions: {:?}", transitions);
                }
        } else {
            println!("Not adding transition: {:?} because it's been already explored.", t_ptr.borrow());
        }
    }
    return path;
}

#[cfg(test)]
mod test {
    use std::{rc::Rc, cell::RefCell};

    use crate::maze::has_path;

    use super::{Field, Direction, Transition};

    fn tie_graph(a: &[Rc<RefCell<Field>>]) {
        Rc::new(RefCell::new(Transition::new(true, &Direction::EAST, Rc::clone(&a[0]), Rc::clone(&a[1]))));
        Rc::new(RefCell::new(Transition::new(false, &Direction::SOUTH, Rc::clone(&a[1]), Rc::clone(&a[3]))));
        Rc::new(RefCell::new(Transition::new(true, &Direction::SOUTH, Rc::clone(&a[0]), Rc::clone(&a[2]))));
    }

    #[test]
    fn basics() {
        let f1 = Field::new(0, 0, true, false);
        assert_eq!(f1.has_key(), true);
        assert_eq!(f1.is_end(), false);
        let rf1 = Rc::new(RefCell::new(f1));

        let f2 = Field::new(1, 0, false, false);
        assert_eq!(f2.has_key(), false);
        assert_eq!(f2.is_end(), false);
        let rf2 = Rc::new(RefCell::new(f2));

        let f3 = Field::new(0, 1, false, false);
        assert_eq!(f3.has_key(), false);
        assert_eq!(f3.is_end(), false);
        let rf3 = Rc::new(RefCell::new(f3));

        let f4 = Field::new(1, 1, false, true);
        assert_eq!(f4.has_key(), false);
        assert_eq!(f4.is_end(), true);
        let rf4 = Rc::new(RefCell::new(f4));

        tie_graph(&[Rc::clone(&rf1), Rc::clone(&rf2), Rc::clone(&rf3), Rc::clone(&rf4)]);

        let p = has_path(Rc::clone(&rf1), Rc::clone(&rf4));
        if let Some(pp) = &p {
            pp.print_path();
        }
        assert_eq!(p.is_some(), true);
        assert_eq!(p.unwrap().cost(), 2);
        println!();

        let p = has_path(Rc::clone(&rf2), Rc::clone(&rf4));
        if let Some(pp) = &p {
            pp.print_path();
        }
        assert_eq!(p.is_some(), true);
        assert_eq!(p.unwrap().cost(), 1);
        println!();

        let p = has_path(Rc::clone(&rf2), Rc::clone(&rf3));
        assert_eq!(p.is_some(), false);
    }
}