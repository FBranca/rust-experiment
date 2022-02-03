use testprj::*;


fn test_from_input_file (bank: &mut Bank, file: &str) {
    let mut csv_reader = csv::Reader::from_path(file).unwrap();
    for result in csv_reader.deserialize() {
        let record: Operation = result.unwrap();
        bank.process_operation (&record);
    }
    
}

// Basic input, only deposits and withdrawals with no error
#[test]
fn test_basic_input() {
    let mut bank: Bank = Bank::new();
    test_from_input_file (&mut bank, "tests/0001_basic_input.csv");

    assert_eq!(bank.accounts.len(), 2);

    let acc1 = bank.accounts.get(&1).unwrap();
    assert_eq!(*acc1, Account{total: 15000, held: 0, locked: false});

    let acc2 = bank.accounts.get(&2).unwrap();
    assert_eq!(*acc2, Account{total: 20000, held: 0, locked: false});
}

// test withdrawal without enough credit 
#[test]
fn test_withdrawal_not_enough_credit () {
    let mut bank: Bank = Bank::new();
    bank.process_operation(& Operation::new(OpType::Deposit,    1, 1, Some(30000)));
    bank.process_operation(& Operation::new(OpType::Withdrawal, 1, 2, Some(31000)));
    bank.process_operation(& Operation::new(OpType::Withdrawal, 1, 3, Some(11000)));

    assert_eq!(bank.accounts.len(), 1);

    let acc1 = bank.accounts.get(&1).unwrap();
    assert_eq!(*acc1, Account{total: 19000, held: 0, locked: false});
}

// simple dispute scenario 
#[test]
fn test_dispute () {
    let mut bank: Bank = Bank::new();
    test_from_input_file (&mut bank, "tests/dispute_01.csv");

    assert_eq!(bank.accounts.len(), 2);

    let acc1 = bank.accounts.get(&1).unwrap();
    assert_eq!(*acc1, Account{total: 40000, held: 30000, locked: false});

    let acc2 = bank.accounts.get(&2).unwrap();
    assert_eq!(*acc2, Account{total: 120000, held: 0, locked: false});
}

// dispute / dispute scenario
// Check that a dispute cannot be applied if one is already in progress
#[test]
fn test_dispute_dispute () {
    let mut bank: Bank = Bank::new();
    test_from_input_file (&mut bank, "tests/dispute_dispute.csv");

    assert_eq!(bank.accounts.len(), 1);

    let acc1 = bank.accounts.get(&1).unwrap();
    assert_eq!(*acc1, Account{total: 120000, held: 20000, locked: false});
}

// dispute / resolve / dispute scenario
// Check that a new dispute can be done after a resolve
#[test]
fn test_dispute_resolve_dispute () {
    let mut bank: Bank = Bank::new();
    test_from_input_file (&mut bank, "tests/dispute_resolve_dispute.csv");

    assert_eq!(bank.accounts.len(), 1);

    let acc1 = bank.accounts.get(&1).unwrap();
    assert_eq!(*acc1, Account{total: 160000, held: 20000, locked: false});
}

// resolve without dispute scenario
#[test]
fn test_resolve_without_dispute () {
    let mut bank: Bank = Bank::new();
    test_from_input_file (&mut bank, "tests/resolve_without_dispute.csv");

    assert_eq!(bank.accounts.len(), 1);

    let acc1 = bank.accounts.get(&1).unwrap();
    assert_eq!(*acc1, Account{total: 120000, held: 0, locked: false});
}

// dispute / chargeback of a deposit / deposit scenario
// Check that a chargeback is done and account locked (no operation allowed)
#[test]
fn test_dispute_chargeback_deposit () {
    let mut bank: Bank = Bank::new();
    test_from_input_file (&mut bank, "tests/dispute_chargeback_deposit.csv");

    assert_eq!(bank.accounts.len(), 1);

    let acc1 = bank.accounts.get(&1).unwrap();
    assert_eq!(*acc1, Account{total: 100000, held: 0, locked: true});
}

// dispute / chargeback of a withdrawal
// Check that a chargeback is done and account locked (no operation allowed)
#[test]
fn test_dispute_chargeback_withdrawl () {
    let mut bank: Bank = Bank::new();
    test_from_input_file (&mut bank, "tests/dispute_chargeback_withdrawal.csv");

    assert_eq!(bank.accounts.len(), 1);

    let acc1 = bank.accounts.get(&1).unwrap();
    assert_eq!(*acc1, Account{total: 100000, held: 0, locked: true});
}

// dispute / chargeback with a negative total in the end
// Occurs when a dispute is done with insufficient available funds
#[test]
fn test_dispute_negative_chargeback () {
    let mut bank: Bank = Bank::new();
    test_from_input_file (&mut bank, "tests/dispute_negative_chargeback.csv");

    assert_eq!(bank.accounts.len(), 1);

    let acc1 = bank.accounts.get(&1).unwrap();
    assert_eq!(*acc1, Account{total: -10000, held: 0, locked: true});
}
