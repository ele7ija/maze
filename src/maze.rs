use core::{fmt, time};
use std::{option::Option, thread::{self, sleep, JoinHandle}, sync::{Arc, Mutex}, fmt::Debug, cmp::Ordering, collections::BinaryHeap};

#[derive(Debug, PartialEq, Clone, Copy, Eq)]
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

    pub fn add_transition(&mut self, direction: &Direction, transition: Arc<Mutex<Transition>>) {
        let f = Some(transition);
        match direction {
            Direction::WEST => self.w = f,
            Direction::EAST => self.e = f,
            Direction::NORTH => self.n = f,
            Direction::SOUTH => self.s = f,
        }
    }

    pub fn get_transition(&self, direction: Direction) -> Option<Arc<Mutex<Transition>>> {
        let t: &Option<Arc<Mutex<Transition>>>;
        match direction {
            Direction::WEST => t = &self.w,
            Direction::EAST => t = &self.e,
            Direction::NORTH => t = &self.n,
            Direction::SOUTH => t = &self.s,
        }
        if let Some(real_t) = t {
            return Some(Arc::clone(real_t));
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

pub type Field = Arc<Mutex<SimpleField>>;

pub struct Transition {
    doors: bool,
    field1: Field,
    field2: Field,
}

impl Transition {
    pub fn new(doors: bool, direction: &Direction, field1: Field, field2: Field) -> Arc<Mutex<Self>> {
        let t = Transition {
            doors: doors,
            field1: Arc::clone(&field1),
            field2: Arc::clone(&field2),
        };
        let rt = Arc::new(Mutex::new(t));

        let mut f1 = field1.lock();
        // let mut f2 = field2.borrow_mut();
        f1.unwrap().add_transition(direction, Arc::clone(&rt));
        // f2.add_transition(&direction.get_opposite(), Arc::clone(&rt));
        rt
    }

    pub fn has_doors(&self) -> bool {
        self.doors
    }

    pub fn get_field1(&self) -> Field {
        Arc::clone(&self.field1)
    }

    pub fn get_field2(&self) -> Field {
        Arc::clone(&self.field2)
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut t = "-";
        if self.doors {
            t = "|";
        }
        write!(f, "{} -{}-> {}", self.get_field1().lock().unwrap(), t, self.get_field2().lock().unwrap())
    }
}

impl fmt::Debug for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut t = "-";
        if self.doors {
            t = "|";
        }
        write!(f, "{} -{}-> {}", self.get_field1().lock().unwrap(), t, self.get_field2().lock().unwrap())
    }
}

impl PartialEq for Transition {
    fn eq(&self, other: &Self) -> bool {
        return self.get_field1().lock().unwrap().eq(&other.get_field1().lock().unwrap()) && self.get_field2().lock().unwrap().eq(&other.get_field2().lock().unwrap())
            // || (self.get_field2() == other.get_field1() && self.get_field1() == other.get_field2())
    }
}

type OptionalTransition = Option<Arc<Mutex<Transition>>>;

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

#[derive(PartialEq)]
pub enum Mode {
    PARALLEL,
    SERIAL
}

pub fn min_path(f1: Field, ends: Vec<Field>, mode: Mode) -> Option<Path> {
    let mut handles: Vec<thread::JoinHandle<Option<Path>>> = Vec::new();
    for end in ends {
        let brf1 = Arc::clone(&f1);
        let brend = Arc::clone(&end);
        let handle: JoinHandle<_>;
        if mode == Mode::PARALLEL {
            handle = thread::spawn(move || {
                has_path(brf1, brend)
            });
        } else {
            let result = has_path(brf1, brend);
            handle = thread::spawn(|| {
                result
            });
        }
        handles.push(handle);
    }
    let mut min: Option<Path> = None;
    for handle in handles {
        let got_min = handle.join().unwrap();
        if got_min.is_none() {
            continue;
        }
        if min.is_none() {
            min = got_min;
            continue;
        }
        let new_min = got_min.unwrap();
        // new_min.print_path();
        let curr_min = min.unwrap();
        if new_min.cost() < curr_min.cost() {
            min = Some(new_min);
        } else {
            min = Some(curr_min)
        }
    }
    return min
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
        if self.fields.iter().any(|e| safe_equals(Arc::clone(&e), Arc::clone(&f))) {
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

fn safe_get_transition(f1: Field, direction: Direction) -> Option<Arc<Mutex<Transition>>> {
    let bf1 = f1.lock().unwrap();
    bf1.get_transition(direction)
}

fn safe_has_key(f1: Field) -> bool {
    f1.lock().unwrap().has_key()
}

fn safe_equals(f1: Field, f2: Field) -> bool {
    let lf1 = f1.lock().unwrap();
    let ff1 = (lf1.x, lf1.y);
    drop(lf1);
    let lf2 = f2.lock().unwrap();
    let ff2 = (lf2.x, lf2.y);
    drop(lf2);  
    ff1 == ff2
}

fn safe_print(f: Field) -> String {
    format!("{}", f.lock().unwrap())
}

fn safe_equals_t(t1: Arc<Mutex<Transition>>, t2: Arc<Mutex<Transition>>) -> bool {
    // return self.get_field1().lock().unwrap().eq(&other.get_field1().lock().unwrap()) && self.get_field2().lock().unwrap().eq(&other.get_field2().lock().unwrap())
    let lt1 = t1.lock().unwrap();
    let tt1 = lt1.get_field1();
    let tt2 = lt1.get_field2();
    drop(lt1);

    let lt2 = t2.lock().unwrap();
    let tt11 = lt2.get_field1();
    let tt22 = lt2.get_field2();
    drop(lt2);

    return safe_equals(Arc::clone(&tt1), Arc::clone(&tt11)) && safe_equals(Arc::clone(&tt2), Arc::clone(&tt22))
}

fn diff(x1: u8, y1: u8, x2: u8, y2: u8) -> u8 {
    return  x2.abs_diff(x1) + y2.abs_diff(y1)
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct DirectionCost {
    cost: u8,
    direction: Direction,
}

impl Ord for DirectionCost {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for DirectionCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn directions_heuristic(f1: Field, end: Field) -> Vec<Direction> {
    let mut result: Vec<_> = Vec::new();
    let directions = [Direction::SOUTH, Direction::EAST, Direction::WEST, Direction::NORTH];
    let mut distances = BinaryHeap::new();

    let (x1, y1): (u8, u8);
    // println!("Field: {}", safe_print(Arc::clone(&f1)));
    let lf1 = end.lock().unwrap();
    x1 = lf1.x;
    y1 = lf1.y;
    drop(lf1);
    for d in directions {
        let t_pos = safe_get_transition(Arc::clone(&f1), d);
        if t_pos.is_none() {
            continue;
        }
        let t_poss = t_pos.unwrap();
        let tf2 = t_poss.lock().unwrap();
        let f2 = tf2.get_field2();
        drop(tf2);
        let lff2 = f2.lock().unwrap();
        let (x2, y2) = (lff2.x, lff2.y);
        drop(lff2);
        let cost = diff(x1, y1, x2, y2);
        // println!("\tCost to {} is {}", safe_print(Arc::clone(&f2)), cost);
        let dc = DirectionCost{cost: cost, direction: d};
        distances.push(dc);
    }
    while !distances.is_empty() {
        result.push(distances.pop().unwrap().direction);
    }
    // println!("For ({}, {}) best: {:?}", x1, y1, result);
    result
}

fn has_path_keys(f1: Field, f2: Field, keys: &mut Keys, transitions: &mut Vec<Arc<Mutex<Transition>>>, mut min_transitions: Option<usize>) -> Option<(Path, usize)> {
    // sleep(time::Duration::from_secs(2));
    // println!("Comparing: {:} and {:}", safe_print(Arc::clone(&f1)), safe_print(Arc::clone(&f2)));
    if safe_equals(Arc::clone(&f1), Arc::clone(&f2)) {
        return Some((Path { steps: Vec::new() }, transitions.len()));
    }
    if let Some(curr_min_transition) = min_transitions {
        if transitions.len() == curr_min_transition {
            // println!("Better path was already found.");
            return None;
        }
    }
    let mut used_key = false;
    if safe_has_key(Arc::clone(&f1)) {
        if keys.add(Arc::clone(&f1)) {
            used_key = true;
            // println!("keys: {} (+1)", keys.total);
        } else {
            // println!("key already used.");
        }
    }
    // println!("keys: {}", keys.total);
    let directions = directions_heuristic(Arc::clone(&f1), Arc::clone(&f2));
    let mut path: Option<Path> = None;
    for d in directions {
        let t_pos = safe_get_transition(Arc::clone(&f1), d);
        if t_pos.is_none() {
            continue;
        }
        // println!("going {:?}", d);
        let t_ptr = t_pos.unwrap();
        // v.iter().any(|e| e == "hello")
        if !transitions.iter().any(|e| safe_equals_t(Arc::clone(&e), Arc::clone(&t_ptr))) {
            let t = t_ptr.lock().unwrap();
            let (doors, f) = (t.doors, Arc::clone(&t.get_field2()));
            drop(t);
            if doors {
                if keys.remove_use() {
                    // println!("keys: {} (-1)", keys.total);
                } else {
                    // println!("no keys left!");
                    continue;
                }
            }
            transitions.push(Arc::clone(&t_ptr));
            // println!("transitions expanded to: {:?}", transitions);
            if let Some((mut steps, new_min_transitions)) = has_path_keys(f, Arc::clone(&f2), keys, transitions, min_transitions) {
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
            let rt = transitions.pop().unwrap();
            let lrt = rt.lock().unwrap();
            // println!("Removed: {}", lrt);
            drop(lrt);
            if doors {
                keys.add_use();
            }
        } else {
            // println!("Not adding transition: {:?} because it's been already explored.", t_ptr.borrow());
        }
    }
    if used_key {
        keys.remove();
        // println!("keys: {} (-1)", keys.total);
    }
    return path.and_then(|p| Some((p, min_transitions.unwrap())));
}

#[cfg(test)]
mod test {
    use std::{sync::{Arc, Mutex}};

    use crate::maze::{has_path, min_path};

    use super::{SimpleField, Direction, Transition, Field};

    fn tie_graph(a: &[Field]) {
        Transition::new(true, &Direction::EAST, Arc::clone(&a[0]), Arc::clone(&a[1]));
        Transition::new(false, &Direction::SOUTH, Arc::clone(&a[1]), Arc::clone(&a[3]));
        Transition::new(true, &Direction::SOUTH, Arc::clone(&a[0]), Arc::clone(&a[2]));
    }

    #[test]
    fn basics() {
        let f1 = SimpleField::new(0, 0, true, false);
        assert_eq!(f1.has_key(), true);
        assert_eq!(f1.is_end(), false);
        let rf1 = Arc::new(Mutex::new(f1));

        let f2 = SimpleField::new(1, 0, false, false);
        assert_eq!(f2.has_key(), false);
        assert_eq!(f2.is_end(), false);
        let rf2 = Arc::new(Mutex::new(f2));

        let f3 = SimpleField::new(0, 1, false, false);
        assert_eq!(f3.has_key(), false);
        assert_eq!(f3.is_end(), false);
        let rf3 = Arc::new(Mutex::new(f3));

        let f4 = SimpleField::new(1, 1, false, true);
        assert_eq!(f4.has_key(), false);
        assert_eq!(f4.is_end(), true);
        let rf4 = Arc::new(Mutex::new(f4));

        tie_graph(&[Arc::clone(&rf1), Arc::clone(&rf2), Arc::clone(&rf3), Arc::clone(&rf4)]);

        let p = has_path(Arc::clone(&rf1), Arc::clone(&rf4));
        if let Some(pp) = &p {
            pp.print_path();
        }
        assert_eq!(p.is_some(), true);
        assert_eq!(p.unwrap().cost(), 2);
        println!();

        let p = has_path(Arc::clone(&rf2), Arc::clone(&rf4));
        if let Some(pp) = &p {
            pp.print_path();
        }
        assert_eq!(p.is_some(), true);
        assert_eq!(p.unwrap().cost(), 1);
        println!();

        let p = has_path(Arc::clone(&rf2), Arc::clone(&rf3));
        assert_eq!(p.is_some(), false);
    }

    #[test]
    fn not_closest() {
        let f1 = SimpleField::new(0, 0, false, false);
        let rf1 = Arc::new(Mutex::new(f1));

        let f2 = SimpleField::new(0, 1, false, false);
        let rf2 = Arc::new(Mutex::new(f2));

        let f3 = SimpleField::new(1, 1, true, false);
        let rf3 = Arc::new(Mutex::new(f3));

        let f4 = SimpleField::new(0, 2, false, true);
        let rf4 = Arc::new(Mutex::new(f4));

        Transition::new(false, &Direction::SOUTH, Arc::clone(&rf1), Arc::clone(&rf2));
        Transition::new(false, &Direction::NORTH, Arc::clone(&rf2), Arc::clone(&rf1));
        Transition::new(true, &Direction::SOUTH, Arc::clone(&rf2), Arc::clone(&rf4));
        Transition::new(false, &Direction::NORTH, Arc::clone(&rf4), Arc::clone(&rf2));
        Transition::new(false, &Direction::EAST, Arc::clone(&rf2), Arc::clone(&rf3));
        Transition::new(false, &Direction::WEST, Arc::clone(&rf3), Arc::clone(&rf2));

        let p = has_path(Arc::clone(&rf1), Arc::clone(&rf4));
        assert_eq!(p.is_some(), true);
        if let Some(pp) = &p {
            pp.print_path();
        }
        println!();
        println!();
        println!();

        let mut ends: Vec<Field> = Vec::new();
        ends.push(Arc::clone(&rf2));
        ends.push(Arc::clone(&rf3));
        ends.push(Arc::clone(&rf4));
        let p = min_path(Arc::clone(&rf1), ends, crate::maze::Mode::PARALLEL);
        assert_eq!(p.is_some(), true);
        if let Some(pp) = &p {
            pp.print_path();
        }
    }
}