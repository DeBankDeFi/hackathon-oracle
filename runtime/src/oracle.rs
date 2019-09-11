use support::{decl_event, decl_module, decl_storage, dispatch::Result, StorageValue};
use system::ensure_signed;

#[cfg(test)]
mod oracle_test;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as OracleStorage {
        Something get(something): Option<u32>;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        pub fn do_something(origin, something: u32) -> Result {
            let who = ensure_signed(origin)?;

            Something::put(something);

            Self::deposit_event(RawEvent::SomethingStored(something, who));
            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        SomethingStored(u32, AccountId),
    }
);
