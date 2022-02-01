use std::env;
use std::collections::HashMap;
use csv;
use serde;
use serde::de;
use serde::de::Error;
#[macro_use]
extern crate serde_derive;

type ClientId = u16;
type AccountsMap = HashMap<ClientId, Account>;


#[derive(Debug)]
enum OpType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}

#[derive(Debug, Deserialize)]
struct Operation {
    #[serde(deserialize_with = "deserialize_op_type")]
    r#type: OpType, // Operation Type
    client: ClientId,    // Client id
    tx: u32,        // Transaction Id
    #[serde(deserialize_with = "deserialize_amount")]
    amount: u32     // Amount, fixed point integer representation (value 1.52 is represented by 15200)
}

/* Convert a string to the corresponding operation type */
fn deserialize_op_type<'de, D>(deserializer: D) -> Result<OpType, D::Error>
where
    D: de::Deserializer<'de>,
{
    let op_str: &str = de::Deserialize::deserialize(deserializer)?;
    match op_str {
        "deposit"    => Ok(OpType::Deposit),
        "withdrawal" => Ok(OpType::Withdrawal),
        "dispute"    => Ok(OpType::Dispute),
        "resolve"    => Ok(OpType::Resolve),
        "chargeback" => Ok(OpType::Chargeback),
        &_ => Err(D::Error::custom(format!("{} is not a valid operation", op_str)))
    }
}


/* Convert an amount from f32 (as read by serde crate)
   to a fixed point integer representation */
fn deserialize_amount<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: de::Deserializer<'de>,
{
    let amount: f32 = de::Deserialize::deserialize(deserializer)?;
    Ok((amount * 10000.0) as u32)
}

struct Account {
    total: u32,
    frozen: u32,
    locked: bool
}

impl Account {
    fn new () -> Account {
        Account {total: 0, frozen: 0, locked: false}
    }

    fn deposit (&mut self, amount: u32) {
        self.total += amount;
    }

    fn withdrawal (&mut self, amount: u32) {
        let available = self.total - self.frozen;
        if amount <= available {
            self.total -= amount;
        }
    }

    fn print_csv_header() {
        println!("client,available,held,total,locked");
    }

    fn print_csv_line(&self, client_id: &ClientId) {
        let available = self.total - self.frozen;
        println!("{},{},{},{},{}",
            client_id,
            (available as f32) / 10000.0,
            (self.frozen as f32) / 10000.0,
            (self.total as f32) / 10000.0,
            self.locked
        );
    }
}

struct Bank {
    accounts : AccountsMap // map of accounts indexed by client id
}

impl Bank {
    fn process_operation (&mut self, operation: &Operation) {
        println!("{:?}", operation);

        let account: &mut Account = self.get_account(&operation.client);
    
        match operation.r#type {
            OpType::Deposit => account.deposit(operation.amount),
            OpType::Withdrawal => account.withdrawal(operation.amount),
            OpType::Dispute => panic!("FIXME"),
            OpType::Resolve => panic!("FIXME"),
            OpType::Chargeback => panic!("FIXME"),
        }
    }

    fn get_account (&mut self, client_id: &ClientId) -> &mut Account {
        if ! self.accounts.contains_key(client_id) {
            self.accounts.insert(
                *client_id,
                Account::new()
            );
        }
        let account = self.accounts.get_mut(&client_id);
        account.unwrap()
    }

    fn print_accounts(&self) {
        Account::print_csv_header();
        for (client_id, account) in self.accounts.iter() {
            account.print_csv_line(client_id);
        }
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("need a filename as paramater");
    }

   let mut bank: Bank = Bank{accounts: HashMap::new()};
    let mut csv_reader = csv::Reader::from_path(args[1].as_str()).unwrap();
    for result in csv_reader.deserialize() {
        let record: Operation = result.unwrap();
        bank.process_operation (&record);
    }

    bank.print_accounts();
}
