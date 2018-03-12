// #![feature(type_ascription)]

/// This type indicates to a procedure whether a call to the database
/// succeeded or failed.
pub enum SandstormErr {
    /// This value indicates that the call succeeded.
    Success,

    /// This value indicates that the call failed because the table
    /// does not exist.
    TableDoesNotExist,
}

pub trait DB {
    fn debug_log(&self, &str);

    // ADDED by SARA for build purposes. TODO: Remove later!
    /// This method should create a table at the database.
    ///
    /// - `table_id`: An identifier for the table to be created.
    ///
    /// - `return`:   True if the table was successfully created. False
    ///               otherwise.
    fn create_table(&self, table_id: u64) -> bool;

    /// This method should allow a procedure to lookup a key of type 'K' from
    /// the database. It should return a reference to a value of type 'V'.
    ///
    /// - `table_id`: An identifier for the table the key-value pair belongs to.
    /// - `key`:      A reference to the key being looked up by the procedure.
    ///
    /// - `return`:   The value corresponding to the key if one exists. A
    ///               reference to this value is handed to the procedure wrapped
    ///               inside a Result<>. The Result<> is required because the
    ///               table might not contain this key.
    fn get_key<K, V>(&self, table_id: u64, key: &K) -> Result<&V, SandstormErr>;


    /// This method should allow a procedure to lookup a key of type 'K' from
    /// the database. It should return a reference to a value of type 'V'.
    ///
    /// - `table_id`: An identifier for the table the key-value pair belongs to.
    /// - `key`:      A reference to the key being looked up by the procedure.
    ///
    /// - `return`:   The value corresponding to the key if one exists. A
    ///               reference to this value is handed to the procedure wrapped
    ///               inside a Result<>. The Result<> is required because the
    ///               table might not contain this key.
    fn delete_key<K>(&self, table_id: u64, key: &K);


        /// This method should re-interpret a slice of objects from one type (T)
        /// into another type (U).
        ///
        /// - `arg`:    The slice of objects to be re-interpretted, each of type T.
        ///
        /// - `return`: The passed in slice re-interpretted as a slice of objects,
        ///             each of type U.
        // fn _as<T, U>(&self, arg: [T]) -> [U];


    /// This method should write a key-value pair to a table. The key should
    /// be of type 'K', and the value should be of type 'V'.
    ///
    /// - `table_id`: An identifier for the table the key-value pair belongs to.
    /// - `key`:      A reference to the key being written by the procedure.
    /// - `value`:    A reference to the value being written by the procedure.
    ///
    /// - `return`:   An error code of type 'SandstormErr' indicating whether
    ///               the operation succeeded or failed.
    fn put_key<K, V>(&self, table_id: u64, key: &K, value: &V) -> SandstormErr;

    fn alloc<K>(&self, table: u64, key: &K, val_len: usize) -> Option<Vec<u8>>;
}

use std::cell::RefCell;

pub struct MockDB {
    messages: RefCell<Vec<String>>,
}

impl MockDB {
    pub fn new() -> MockDB {
        MockDB{messages: RefCell::new(Vec::new())}
    }

    pub fn assert_messages<S>(&self, messages: &[S])
        where S: std::fmt::Debug + PartialEq<String>
    {
        let found = self.messages.borrow();
        assert_eq!(messages, found.as_slice());
    }

    pub fn clear_messages(&self) {
        let mut messages = self.messages.borrow_mut();
        messages.clear();
    }
}

impl DB for MockDB {

    fn debug_log(&self, message: &str) {
        let mut messages = self.messages.borrow_mut();
        messages.push(String::from(message));
    }
    // ADDED by SARA for build purposes. TODO: Remove later!
    fn create_table(&self, _id: u64) -> bool{
        return true
    }

    // fn _as<T, U>(&self, arg: [T]) -> [U] {
    //
    // }

    fn put_key<K, V>(&self, _table_id: u64, _key: &K, _value: &V) -> SandstormErr {
        SandstormErr::Success
    }

    fn get_key<K, V>(&self, _table_id: u64, _key: &K) -> Result<&V, SandstormErr> {
        Err(SandstormErr::TableDoesNotExist)
    }

    fn delete_key<K>(&self, _table_id: u64, _key: &K){
    }
    fn alloc<K>(&self, table: u64, key: &K, val_len: usize) -> Option<Vec<u8>>{
        None
    }

}

pub struct NullDB {}

// impl NullDB {
//     pub fn new() -> NullDB {
//         NullDB{}
//     }
//
//     pub fn assert_messages<S>(&self, messages: &[S])
//         where S: std::fmt::Debug + PartialEq<String>
//     {}
//
//     pub fn clear_messages(&self) {}
// }
//
// impl DB for NullDB {
//     fn debug_log(&self, message: &str) {}
//
//     // ADDED by SARA for build purposes. TODO: Remove later!
//     fn create_table(&self, id: u64) {
//
//     }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
