== Completeness ==

All cases should be adressed by this implementation.

== Safety & Robustness ==

Current implementation doesn't adress the following points :
* Doesn't check for syntax error in the input file
* Doesn't check the amount of transaction : no limit is assumed, and there is no check that the amount is a positive value

The code can panic is some situations (syntax error in the CSV for example).

== Scalibility == 

There is no scalibity issue for accounts. As the client id is u16, there can only be 65536 client accounts.

This is more complicated for transactions : we need to keep track of previous transaction in order to handle disputes and there is no limit specified, so we can reach 2^32 transactions.
The code defines a trait for the transaction log, so different implementation can be used without changing the business logic. The current implementation is a simple in memory transaction log and won't scale to an important number of transactions. It should be replaced by other solution to scale up without using too much memory : files or database.

== Maintenability ==

Current code is simple and should not be too difficult to maintain.
It may have been better to create one file per concept (one file for accounts, one for transactions, one for transaction log), especially if these concepts are meant to grow of evolve (but it's not the case here).
Maintenability is also easier because of the presence of integration tests that helps to prevent regressions.

== Correctness ==

Some integration tests helps to check the correctness of the code.

I spent some time on the disputes on a withdrawal operation because it sounded weird to me.
For disputes is asked to hold the amount of the refered transaction without any precision. However, the case of dispute on a deposit looks very different of the one on a withdrawal.
Helding the funds of a deposit in the case of a dispute sounds natural (the funds may be charged back in the future), but this makes less sense on a withdrawal as the funds have already been removed from the account (so there should be no need to hold them).

I decided to apply what was asked : the amount is held whatever the kind of the refered transaction. Hope this is what is expected.

I also decided to allow a dispute and a chargeback if there is not enough available funds on the account.
For example :
* tx 1: Deposit 2
* tx 2: Withdrawal 1
* dispute tx 1 => there is not enough available funds, but this is allowed and gives a account status = Total:1, held:2, available:-1
* chargeback tx 1 => is allowed and will lead to a account total of -1
