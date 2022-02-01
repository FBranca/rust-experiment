use std::collections::HashMap;
use serde;
use serde::de;
use serde::de::Error;
#[macro_use]
extern crate serde_derive;

type ClientId = u16;
type AccountsMap = HashMap<ClientId, Account>;


#[derive(Debug, Clone)]
pub enum OpType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback
}

#[derive(Debug, Clone, Deserialize)]
pub struct Operation {
    #[serde(deserialize_with = "deserialize_op_type")]
    r#type: OpType,     // Operation Type
    client: ClientId,   // Client id
    tx: u32,            // Transaction Id
    #[serde(deserialize_with = "deserialize_amount")]
    amount: Option<u32>, // Amount, fixed point integer representation (value 1.52 is represented by 15200)
    #[serde(skip_deserializing)]
    under_dispute: bool,
    #[serde(skip_deserializing)]
    charged_back: bool
}

impl Operation {
    pub fn new (r#type: OpType, client: ClientId, tx: u32, amount: Option<u32>) -> Operation {
        Operation {r#type: r#type, client: client, tx: tx, amount: amount, under_dispute: false, charged_back: false}
    }
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
fn deserialize_amount<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: de::Deserializer<'de>,
{
    let amount: Option<f32> = de::Deserialize::deserialize(deserializer)?;
    match amount {
        Some(val) => Ok(Some((val * 10000.0) as u32)),
        None => Ok(None)
    }
}

// Transaction log trait
// Recording transaction history and searching in all previous transactions 
// Implementation of this trait could be :
// most simple
//  ^
//  |  in memory history of transactions
//  |  a line number in the input CSV file (only works with a single CSV file as input)
//  |  a set of files 
//  |  connexion to a database
//  v
// most scalable
trait TransactionLog {
    fn tx_add (&mut self, operation: &Operation);
    fn tx_search (&mut self, id: &u32) -> Option<&mut Operation>;
}

// Simple in-memory tansaction log
// Would not scale to a huge amount of transation, but should be enough for smal test cases.
struct TransactionLogInMemory {
    history: HashMap<u32, Operation>
}

impl TransactionLog for TransactionLogInMemory {
    fn tx_add (&mut self, operation: &Operation) {
        self.history.insert (operation.tx, operation.clone());
    }

    fn tx_search (&mut self, id: &u32) -> Option<&mut Operation> {
        let operation = self.history.get_mut(id);
        match operation {
            Some(_) => Some(operation.unwrap()),
            None => None
        }
    }
}

impl TransactionLogInMemory {
    pub fn new () -> TransactionLogInMemory {
        TransactionLogInMemory {history: HashMap::new()}
    }
}

#[derive(Debug, PartialEq)]
pub struct Account {
    pub total: u32,
    pub held: u32,
    pub locked: bool
}

impl Account {
    fn new () -> Account {
        Account {total: 0, held: 0, locked: false}
    }

    fn print_csv_header() {
        println!("client,available,held,total,locked");
    }

    fn print_csv_line(&self, client_id: &ClientId) {
        let available = self.total - self.held;
        println!("{},{},{},{},{}",
            client_id,
            (available as f32) / 10000.0,
            (self.held as f32) / 10000.0,
            (self.total as f32) / 10000.0,
            self.locked
        );
    }
}

pub struct Bank {
    pub accounts : AccountsMap, // map of accounts indexed by client id
    transaction_log : TransactionLogInMemory
}

impl Bank {
    pub fn new () -> Bank {
        Bank {accounts: HashMap::new(), transaction_log: TransactionLogInMemory::new()}
    }

    pub fn process_operation (&mut self, operation: &Operation) {
        let mut res;

        match operation.r#type {
            OpType::Deposit => res = self.deposit(operation),
            OpType::Withdrawal => res = self.withdrawal(operation),
            OpType::Dispute => res = self.dispute(operation),
            OpType::Resolve => res = self.resolve(operation),
            OpType::Chargeback => res = self.chargeback(operation),
        }

        if ! res.is_ok() {
            eprintln!("transaction failed {}", res.err().unwrap());
        }
    }

    fn deposit (&mut self, operation: &Operation) -> Result<(), &str> {
        let account: &mut Account = Bank::get_account(&mut self.accounts, &operation.client);
        if account.locked {
            return Err ("deposit: account is locked");
        }

        account.total += operation.amount.unwrap();
        self.transaction_log.tx_add (operation);
        Ok(())
    }

    fn withdrawal (&mut self, operation: &Operation) -> Result<(), &str> {
        let account: &mut Account = Bank::get_account(&mut self.accounts, &operation.client);
        if account.locked {
            return Err ("withdrawal: account is locked");
        }

        let available = account.total - account.held;
        let amount = operation.amount.unwrap();
        if amount > available {
            return Err ("withdrawal: not enough available credit");
        }

        account.total -= amount;
        self.transaction_log.tx_add (operation);
        Ok(())
    }

    fn dispute (&mut self, operation: &Operation) -> Result<(), &str> {
        let ref_tx = self.transaction_log.tx_search(&operation.tx);
        if ref_tx.is_none() {
            return Err ("dispute: referenced transaction doesn't exist");
        }

        let mut ref_tx = ref_tx.unwrap();
        if ref_tx.under_dispute {
            return Err ("dispute: transaction allready under dispute");
        }

        if ref_tx.charged_back {
            // No sense to dispute a transaction that have been charged back
            return Err ("dispute: transaction was charged back");
        }

        if ref_tx.client != operation.client {
            // Assume the client issuing the dispute must be the same of the original transaction 
            return Err ("dispute: client id inconsistent with referenced transaction");
        }

        let amount = ref_tx.amount.unwrap();
        let account: &mut Account = Bank::get_account(&mut self.accounts, &operation.client);
        if account.locked {
            return Err ("dispute: account is locked");
        }

        account.held += amount;
        println!("dispute {} {}", account.held, amount);
        ref_tx.under_dispute = true;
        Ok(())
    }

    fn resolve (&mut self, operation: &Operation) -> Result<(), &str> {
        let ref_tx = self.transaction_log.tx_search(&operation.tx);
        if ref_tx.is_none() {
            return Err("resolve: referenced transaction doesn't exist");
        }

        let mut ref_tx = ref_tx.unwrap();
        if ! ref_tx.under_dispute {
            return Err("resolve: transaction is not under dispute");
        }

        if ref_tx.client != operation.client {
            return Err("resolve: client id inconsistent with referenced transaction");
        }

        let amount = ref_tx.amount.unwrap();
        let account: &mut Account = Bank::get_account(&mut self.accounts, &operation.client);
        if account.locked {
            return Err ("resolve: account is locked");
        }
        account.held -= amount;
        println!("resolve {} {}", account.held, amount);
        ref_tx.under_dispute = false;
        Ok(())
    }

    fn chargeback (&mut self, operation: &Operation) -> Result<(), &str> {
        let ref_tx = self.transaction_log.tx_search(&operation.tx);
        if ref_tx.is_none() {
            return Err("chargeback: referenced transaction doesn't exist");
        }

        let mut ref_tx = ref_tx.unwrap();
        if ! ref_tx.under_dispute {
            return Err("chargeback: transaction is not under dispute");
        }

        if ref_tx.client != operation.client {
            return Err("chargeback: client id inconsistent with referenced transaction");
        }

        let amount = ref_tx.amount.unwrap();
        let account: &mut Account = Bank::get_account(&mut self.accounts, &operation.client);
        if account.locked {
            return Err ("chargeback: account is locked");
        }

        account.held -= amount;
        account.total -= amount;
        ref_tx.under_dispute = false;
        ref_tx.charged_back = true;
        Ok(())
    }

    fn get_account<'a> (accounts: &'a mut AccountsMap, client_id: &ClientId) -> &'a mut Account {
        if ! accounts.contains_key(client_id) {
            accounts.insert(
                *client_id,
                Account::new()
            );
        }
        let account = accounts.get_mut(&client_id);
        account.unwrap()
    }

    pub fn print_accounts(&self) {
        Account::print_csv_header();
        for (client_id, account) in self.accounts.iter() {
            account.print_csv_line(client_id);
        }
    }
}

