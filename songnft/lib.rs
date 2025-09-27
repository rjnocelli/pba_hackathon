#![cfg_attr(not(feature = "std"), no_std, no_main)]

use ink::{
    prelude::vec::Vec,
    primitives::AccountId,
};

#[cfg_attr(test, allow(dead_code))]

const _ON_ERC_1155_BATCH_RECEIVED_SELECTOR: [u8; 4] = [0xBC, 0x19, 0x7C, 0x81];
pub type TokenId = u128;
type Balance = <ink::env::DefaultEnvironment as ink::env::Environment>::Balance;

// The ERC-1155 error types.
#[derive(Debug, PartialEq, Eq)]
#[ink::scale_derive(Encode, Decode, TypeInfo)]
pub enum Error {
    /// This token ID has not yet been created by the contract.
    UnexistentToken,
    /// The caller tried to sending tokens to the zero-address (`0x00`).
    ZeroAddressTransfer,
    /// The caller is not approved to transfer tokens on behalf of the account.
    NotApproved,
    /// The account does not have enough funds to complete the transfer.
    InsufficientBalance,
    /// An account does not need to approve themselves to transfer tokens.
    SelfApproval,
    /// The number of tokens being transferred does not match the specified number of
    /// transfers.
    BatchTransferMismatch,
    /// The token ID already exists.
    TokenAlreadyExists,
    /// The token ID does not exist or the caller is not the owner of the token.
    UnexistentTokenOrCallerNotOwner,
}

// The ERC-1155 result types.
pub type Result<T> = core::result::Result<T, Error>;

/// Evaluate `$x:expr` and if not true return `Err($y:expr)`.
///
/// Used as `ensure!(expression_to_ensure, expression_to_return_on_false)`.
macro_rules! ensure {
    ( $condition:expr, $error:expr $(,)? ) => {{
        if !$condition {
            return ::core::result::Result::Err(::core::convert::Into::into($error))
        }
    }};
}
#[ink::trait_definition]
pub trait Songnft {
    #[ink(message)]
    fn balance_of(&self, owner: AccountId, token_id: TokenId) -> Balance;
    
    #[ink(message)]
    fn balance_of_batch(
        &self,
        owners: Vec<AccountId>,
        token_ids: Vec<TokenId>,
    ) -> Vec<Balance>;
}
#[ink::trait_definition]
pub trait SongnftTokenReceiver {
    /// Handle the receipt of a single ERC-1155 token.
    ///
    /// This should be called by a compliant ERC-1155 contract if the intended recipient
    /// is a smart contract.
    ///
    /// If the smart contract implementing this interface accepts token transfers then it
    /// must return `ON_ERC_1155_RECEIVED_SELECTOR` from this function. To reject a
    /// transfer it must revert.
    ///
    /// Any callers must revert if they receive anything other than
    /// `ON_ERC_1155_RECEIVED_SELECTOR` as a return value.
    #[ink(message, selector = 0xF23A6E61)]
    fn on_received(
        &mut self,
        operator: AccountId,
        from: AccountId,
        token_id: TokenId,
        value: Balance,
        data: Vec<u8>,
    ) -> Vec<u8>;

    /// Handle the receipt of multiple ERC-1155 tokens.
    ///
    /// This should be called by a compliant ERC-1155 contract if the intended recipient
    /// is a smart contract.
    ///
    /// If the smart contract implementing this interface accepts token transfers then it
    /// must return `BATCH_ON_ERC_1155_RECEIVED_SELECTOR` from this function. To
    /// reject a transfer it must revert.
    ///
    /// Any callers must revert if they receive anything other than
    /// `BATCH_ON_ERC_1155_RECEIVED_SELECTOR` as a return value.
    #[ink(message, selector = 0xBC197C81)]
    fn on_batch_received(
        &mut self,
        operator: AccountId,
        from: AccountId,
        token_ids: Vec<TokenId>,
        values: Vec<Balance>,
        data: Vec<u8>,
    ) -> Vec<u8>;
}

#[ink::contract]
mod songnft {
    use super::*;

    use ink::storage::Mapping;

    type Owner = AccountId;
    type Operator = AccountId;

    /// Indicate that a token transfer has occured.
    ///
    /// This must be emitted even if a zero value transfer occurs.
    #[ink(event)]
    pub struct TransferSingle {
        #[ink(topic)]
        operator: Option<AccountId>,
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        token_id: TokenId,
        value: Balance,
    }

    /// Indicate that an approval event has happened.
    #[ink(event)]
    pub struct ApprovalForAll {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        operator: AccountId,
        approved: bool,
    }

    /// Indicate that a token's URI has been updated.
    #[ink(event)]
    pub struct Uri {
        value: ink::prelude::string::String,
        #[ink(topic)]
        token_id: TokenId,
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct Contract {
        balances: Mapping<(AccountId, TokenId), Balance>,
        approvals: Mapping<(Owner, Operator), ()>,
    }

    impl Contract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Default::default()
        }

        #[ink(message)]
        pub fn create(&mut self, value: Balance, token_id: u128) -> Result<TokenId> {
            let caller = self.env().caller();
            ensure!(!self.balances.contains((caller, token_id)), Error::TokenAlreadyExists);
            self.balances.insert((caller, token_id), &value);

            self.env().emit_event(TransferSingle {
                operator: Some(caller),
                from: None,
                to: if value == 0 { None } else { Some(caller) },
                token_id: token_id,
                value,
            });
            Ok(token_id)
        }

        #[ink(message)]
        pub fn mint(&mut self, token_id: TokenId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            ensure!(self.balances.contains((caller, token_id)), Error::UnexistentTokenOrCallerNotOwner);
            self.balances.insert((caller, token_id), &value);

            self.env().emit_event(TransferSingle {
                operator: Some(caller),
                from: None,
                to: Some(caller),
                token_id,
                value,
            });

            Ok(())
        }
    }

    impl super::Songnft for Contract {
        #[ink(message)]
        fn balance_of(&self, owner: AccountId, token_id: TokenId) -> Balance {
            self.balances.get((owner, token_id)).unwrap_or(0)
        }

        #[ink(message)]
        fn balance_of_batch(
            &self,
            owners: Vec<AccountId>,
            token_ids: Vec<TokenId>,
        ) -> Vec<Balance> {
            let mut output = Vec::new();
            for o in &owners {
                for t in &token_ids {
                    let amount = self.balance_of(*o, *t);
                    output.push(amount);
                }
            }
            output
        }
    }
    impl super::SongnftTokenReceiver for Contract {
        #[ink(message, selector = 0xF23A6E61)]
        fn on_received(
            &mut self,
            _operator: AccountId,
            _from: AccountId,
            _token_id: TokenId,
            _value: Balance,
            _data: Vec<u8>,
        ) -> Vec<u8> {
            unimplemented!("This smart contract does not accept token transfer.")
        }

        #[ink(message, selector = 0xBC197C81)]
        fn on_batch_received(
            &mut self,
            _operator: AccountId,
            _from: AccountId,
            _token_ids: Vec<TokenId>,
            _values: Vec<Balance>,
            _data: Vec<u8>,
        ) -> Vec<u8> {
            unimplemented!("This smart contract does not accept batch token transfers.")
        }
    }

    fn zero_address() -> AccountId {
        [0u8; 32].into()
    }

    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use crate::Songnft;

        fn set_sender(sender: AccountId) {
            ink::env::test::set_caller::<Environment>(sender);
        }

        fn default_accounts() -> ink::env::test::DefaultAccounts<Environment> {
            ink::env::test::default_accounts::<Environment>()
        }

        fn alice() -> AccountId {
            default_accounts().alice
        }

        fn bob() -> AccountId {
            default_accounts().bob
        }

        fn charlie() -> AccountId {
            default_accounts().charlie
        }

        fn init_contract() -> Contract {
            let mut erc = Contract::new();
            erc.balances.insert((alice(), 1), &10);
            erc.balances.insert((alice(), 2), &20);
            erc.balances.insert((bob(), 1), &10);

            erc
        }

        #[ink::test]
        fn can_get_correct_balance_of() {
            let erc = init_contract();

            assert_eq!(erc.balance_of(alice(), 1), 10);
            assert_eq!(erc.balance_of(alice(), 2), 20);
            assert_eq!(erc.balance_of(alice(), 3), 0);
            assert_eq!(erc.balance_of(bob(), 2), 0);
        }

        #[ink::test]
        fn can_get_correct_batch_balance_of() {
            let erc = init_contract();

            assert_eq!(
                erc.balance_of_batch(vec![alice()], vec![1, 2, 3]),
                vec![10, 20, 0]
            );
            assert_eq!(
                erc.balance_of_batch(vec![alice(), bob()], vec![1]),
                vec![10, 10]
            );

            assert_eq!(
                erc.balance_of_batch(vec![alice(), bob(), charlie()], vec![1, 2]),
                vec![10, 20, 10, 0, 0, 0]
            );
        }

        #[ink::test]
        fn minting_tokens_works() {
            let mut erc = Contract::new();

            set_sender(alice());
            assert_eq!(erc.create(0, 1u128).unwrap(), 1u128);
            assert_eq!(erc.balance_of(alice(), 1u128), 0);

            assert!(erc.mint(1u128, 123).is_ok());
            assert_eq!(erc.balance_of(alice(), 1u128), 123);
        }

        #[ink::test]
        fn minting_not_allowed_for_nonexistent_tokens() {
            let mut erc = Contract::new();

            let res = erc.mint(1, 123);
            assert_eq!(res.unwrap_err(), Error::UnexistentTokenOrCallerNotOwner);
        }
    }
}
