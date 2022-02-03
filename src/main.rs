use testprj::*;
use csv;
use std::env;


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("need a filename as paramater");
    }

    let mut bank: Bank = Bank::new();
    let mut csv_reader = csv::Reader::from_path(args[1].as_str()).unwrap();
    for result in csv_reader.deserialize() {
        let record: Operation = result.unwrap();
        bank.process_operation (&record);
    }

    bank.print_accounts();
}
