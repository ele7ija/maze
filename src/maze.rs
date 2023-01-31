use core::fmt;
use std::{option::Option, rc::Rc, cell::{RefCell}, thread::sleep, time};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Direction {
    SOUTH,
    EAST,
    NORTH,
    WEST,
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

pub struct SimpleField {
    x: u8,
    y: u8,
    w: OptionalTransition,
    e: OptionalTransition,
    n: OptionalTransition,
    s: OptionalTransition,
    key: bool,
    end: bool,
}

impl SimpleField {
    pub fn new(x: u8, y: u8, key: bool, end: bool) -> Self {
        SimpleField {
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

impl fmt::Display for SimpleField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl fmt::Debug for SimpleField {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", self.x, self.y)
    }
}

impl PartialEq for SimpleField {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

pub type Field = Rc<RefCell<SimpleField>>;

pub struct Transition {
    doors: bool,
    field1: Field,
    field2: Field,
}

impl Transition {
    pub fn new(doors: bool, direction: &Direction, field1: Field, field2: Field) -> Rc<RefCell<Self>> {
        let t = Transition {
            doors: doors,
            field1: Rc::clone(&field1),
            field2: Rc::clone(&field2),
        };
        let rt = Rc::new(RefCell::new(t));

        let mut f1 = field1.borrow_mut();
        // let mut f2 = field2.borrow_mut();
        f1.add_transition(direction, Rc::clone(&rt));
        // f2.add_transition(&direction.get_opposite(), Rc::clone(&rt));
        rt
    }

    pub fn has_doors(&self) -> bool {
        self.doors
    }

    pub fn get_field1(&self) -> Field {
        Rc::clone(&self.field1)
    }

    pub fn get_field2(&self) -> Field {
        Rc::clone(&self.field2)
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut t = "-";
        if self.doors {
            t = "|";
        }
        write!(f, "{} -{}-> {}", self.get_field1().borrow(), t, self.get_field2().borrow())
    }
}

impl fmt::Debug for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut t = "-";
        if self.doors {
            t = "|";
        }
        write!(f, "{} -{}-> {}", self.get_field1().borrow(), t, self.get_field2().borrow())
    }
}

impl PartialEq for Transition {
    fn eq(&self, other: &Self) -> bool {
        return self.get_field1() == other.get_field1() && self.get_field2() == other.get_field2()
            // || (self.get_field2() == other.get_field1() && self.get_field1() == other.get_field2())
    }
}

type OptionalTransition = Option<Rc<RefCell<Transition>>>;

pub struct Path {
    steps: Vec<Direction>,
}

impl Path {
    pub fn cost(&self) -> usize {
        return self.steps.len();
    }
    pub fn print_path(&self) {
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

pub fn has_path(f1: Field, f2: Field) -> Option<Path> {
    let mut k = Keys::new();
    if let Some((path, _)) = has_path_keys(f1, f2, &mut k, &mut Vec::new(), None) {
        return Some(path)
    }
    None
}

pub struct Keys {
    fields: Vec<Field>,
    total: u16,
}

impl Keys {
    pub fn new() -> Self {
        Keys {
            fields: Vec::new(),
            total: 0,
        }
    }

    pub fn add(&mut self, f: Field) -> bool {
        if self.fields.contains(&f) {
            return false;
        }
        self.fields.push(f);
        self.total += 1;
        true
    }

    pub fn add_use(&mut self) {
        self.total += 1;
    }

    pub fn remove_use(&mut self) -> bool {
        if self.total == 0 {
            return false;
        }
        self.total -= 1;
        true
    }

    pub fn remove(&mut self) {
        self.fields.pop();
        self.total -= 1;
    }
}

fn has_path_keys(f1: Field, f2: Field, keys: &mut Keys, transitions: &mut Vec<Rc<RefCell<Transition>>>, mut min_transitions: Option<usize>) -> Option<(Path, usize)> {
    // sleep(time::Duration::from_secs(2));
    println!("Comparing: {:} and {:}", f1.borrow(), f2.borrow());
    if f1 == f2 {
        return Some((Path { steps: Vec::new() }, transitions.len()));
    }
    if let Some(curr_min_transition) = min_transitions {
        if transitions.len() == curr_min_transition {
            println!("Better path was already found.");
            return None;
        }
    }
    let bf1 = f1.borrow();
    let mut used_key = false;
    if bf1.has_key() {
        if keys.add(Rc::clone(&f1)) {
            used_key = true;
            // println!("keys: {} (+1)", keys.total);
        } else {
            println!("key on field: {} already used.", bf1);
        }
    }
    println!("keys: {}", keys.total);
    let directions = [Direction::SOUTH, Direction::EAST, Direction::WEST, Direction::NORTH];
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
            if doors {
                if keys.remove_use() {
                    println!("keys: {} (-1)", keys.total);
                } else {
                    println!("no keys left!");
                    continue;
                }
            }
            transitions.push(Rc::clone(&t_ptr));
            println!("transitions expanded to: {:?}", transitions);
            if let Some((mut steps, new_min_transitions)) = has_path_keys(f, Rc::clone(&f2), keys, transitions, min_transitions) {
                if let Some(curr_path) = &path {
                    if steps.cost() + 1 < curr_path.cost() {
                        steps.add_step(d);
                        path = Some(steps);
                        min_transitions = Some(new_min_transitions);
                    }
                } else {
                    steps.add_step(d);
                    path = Some(steps);
                    min_transitions = Some(new_min_transitions);
                }
            }
            let rt = transitions.pop();
            println!("Removed: {}", rt.unwrap().borrow());
            if doors {
                keys.add_use();
            }
        } else {
            // println!("Not adding transition: {:?} because it's been already explored.", t_ptr.borrow());
        }
    }
    if used_key {
        keys.remove();
        println!("keys: {} (-1)", keys.total);
    }
    return path.and_then(|p| Some((p, min_transitions.unwrap())));
}

#[cfg(test)]
mod test {
    use std::{rc::Rc, cell::RefCell};

    use crate::maze::has_path;

    use super::{SimpleField, Direction, Transition, Field};

    fn tie_graph(a: &[Field]) {
        Transition::new(true, &Direction::EAST, Rc::clone(&a[0]), Rc::clone(&a[1]));
        Transition::new(false, &Direction::SOUTH, Rc::clone(&a[1]), Rc::clone(&a[3]));
        Transition::new(true, &Direction::SOUTH, Rc::clone(&a[0]), Rc::clone(&a[2]));
    }

    #[test]
    fn basics() {
        let f1 = SimpleField::new(0, 0, true, false);
        assert_eq!(f1.has_key(), true);
        assert_eq!(f1.is_end(), false);
        let rf1 = Rc::new(RefCell::new(f1));

        let f2 = SimpleField::new(1, 0, false, false);
        assert_eq!(f2.has_key(), false);
        assert_eq!(f2.is_end(), false);
        let rf2 = Rc::new(RefCell::new(f2));

        let f3 = SimpleField::new(0, 1, false, false);
        assert_eq!(f3.has_key(), false);
        assert_eq!(f3.is_end(), false);
        let rf3 = Rc::new(RefCell::new(f3));

        let f4 = SimpleField::new(1, 1, false, true);
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

    #[test]
    fn not_closest() {
        let f1 = SimpleField::new(0, 0, false, false);
        let rf1 = Rc::new(RefCell::new(f1));

        let f2 = SimpleField::new(0, 1, false, false);
        let rf2 = Rc::new(RefCell::new(f2));

        let f3 = SimpleField::new(1, 1, true, false);
        let rf3 = Rc::new(RefCell::new(f3));

        let f4 = SimpleField::new(0, 2, false, true);
        let rf4 = Rc::new(RefCell::new(f4));

        Transition::new(false, &Direction::SOUTH, Rc::clone(&rf1), Rc::clone(&rf2));
        Transition::new(false, &Direction::NORTH, Rc::clone(&rf2), Rc::clone(&rf1));
        Transition::new(true, &Direction::SOUTH, Rc::clone(&rf2), Rc::clone(&rf4));
        Transition::new(false, &Direction::NORTH, Rc::clone(&rf4), Rc::clone(&rf2));
        Transition::new(false, &Direction::EAST, Rc::clone(&rf2), Rc::clone(&rf3));
        Transition::new(false, &Direction::WEST, Rc::clone(&rf3), Rc::clone(&rf2));

        let p = has_path(Rc::clone(&rf1), Rc::clone(&rf4));
        assert_eq!(p.is_some(), true);
        if let Some(pp) = &p {
            pp.print_path();
        }
    }
}