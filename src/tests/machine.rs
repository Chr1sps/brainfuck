use crate::BrainfuckMachine;
#[test]
fn test_index_change_base() {
    let mut machine = BrainfuckMachine::new(10);
    assert_eq!(machine.index, 0);
    machine.move_right(5);
    assert_eq!(machine.index, 5);
    machine.move_left(3);
    assert_eq!(machine.index, 2);
}

#[test]
#[should_panic = "Index out of bounds."]
fn test_index_change_left_panic() {
    let mut machine = BrainfuckMachine::new(10);
    machine.move_left(3);
}
#[test]
#[should_panic = "Index out of bounds."]
fn test_index_change_right_panic() {
    let mut machine = BrainfuckMachine::new(10);
    machine.move_right(11);
}

#[test]
fn test_value_change_base() {
    let mut machine = BrainfuckMachine::new(10);
    assert_eq!(
        machine.tape[machine.index], 0,
        "Current cell value: {}.",
        machine.tape[machine.index]
    );
    machine.add(69);
    assert_eq!(machine.tape[machine.index], 69);
    machine.substract(42);
    assert_eq!(machine.tape[machine.index], 27);
}
#[test]
fn test_value_change_wrap() {
    let mut machine = BrainfuckMachine::new(10);
    assert_eq!(
        machine.tape[machine.index], 0,
        "Current cell value: {}.",
        machine.tape[machine.index]
    );
    machine.substract(11);
    assert_eq!(
        machine.tape[machine.index], 245,
        "Current cell value: {}.",
        machine.tape[machine.index]
    );
    machine.add(23);
    assert_eq!(
        machine.tape[machine.index], 12,
        "Current cell value: {}.",
        machine.tape[machine.index]
    );
}

#[test]
fn test_put_char() {
    let mut machine = BrainfuckMachine::new(10);
    machine.add(65);
    let result: char = machine.put_char();
    assert_eq!(
        result, 'A',
        "Different char read. Char read: {}.",
        result as u8
    );
}
#[test]
fn test_read_char() {
    let mut machine = BrainfuckMachine::new(10);
    machine.read_char('A');
    let result = machine.tape[machine.index];
    assert_eq!(result, 65, "Different char read. Char read: {}.", result);
}
#[test]
fn test_check_loop() {
    let mut machine = BrainfuckMachine::new(10);
    assert!(!machine.check_loop());
    machine.add(4);
    assert!(machine.check_loop());
    machine.substract(4);
    assert!(!machine.check_loop());
}
