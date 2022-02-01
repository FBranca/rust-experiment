use testprj::*;
use std::collections::HashMap;

fn test_from_input_file (bank: &mut Bank, file: &str) {
    let mut csv_reader = csv::Reader::from_path(file).unwrap();
    for result in csv_reader.deserialize() {
        let record: Operation = result.unwrap();
        bank.process_operation (&record);
    }
    
}

#[test]
fn test_basic_input() {
    let mut bank: Bank = Bank{accounts: HashMap::new()};
    test_from_input_file (&mut bank, "tests/0001_basic_input.csv");

    assert_eq!(bank.accounts.len(), 2);

    let acc1 = bank.accounts.get(&1).unwrap();
    assert_eq!(*acc1, Account{total: 15000, frozen: 0, locked: false});

    let acc2 = bank.accounts.get(&2).unwrap();
    assert_eq!(*acc2, Account{total: 20000, frozen: 0, locked: false});
}