#![crate_type = "dylib"]
// #![feature(no_unsafe)] //not allowed on stable release version??

extern crate bytes; // add bytes = "0.4" under dependencies in cargo.toml
extern crate sandstorm;
use bytes::Bytes;

type Id = u64;
type ObjectType = u16;
type AssociationType = u16;
//type Time = u32;


pub struct Tao<'a> {
    client: &'a sandstorm::DB,
    object_table_id: u64, //What's this for?
    association_table_id: u64,
    next_id: Id
}

impl<'a> Tao<'a>  {
    /// Returns a TAO instance connecting to the given client
    ///
    /// # Arguments
    ///
    /// * `client` - Access to a sandstorm::DB in which to add DB info to.
    ///
    pub fn new(client: &sandstorm::DB) -> Tao {
        let object_table_id: u64 = 0;
        let association_table_id: u64 = 1;
        client.create_table(object_table_id); // why is table_id param instead of ret val?
        client.create_table(association_table_id);

        Tao {
            client,
            // object_table_id: client.create_table()
            object_table_id,
            association_table_id,
            next_id: 0
        }
    }

    // object api

    /// Returns the Id of the newly created object.
    ///
    /// # Arguments
    ///
    /// * `object_type` -
    ///
    /// * 'data' - kvpairs which make up the object.
    ///
    pub fn object_add(&self, object_type: ObjectType, data: Bytes) -> Id {
        let object_id: Id = self.allocate_unique_id();

        let err: SandstormErr = self.client.put_key<K, V>(table_id: u64, key: &K, value: &V);

        object_id
        0
    }
    pub fn object_update(&self, id: Id, data: Bytes){

    }
    pub fn object_delete(&self, id: Id){

    }
    pub fn object_get(&self, id: Id, data: &mut Bytes) -> ObjectType {
        0
    }

    //assocation api

    pub fn assocation_add(&self, id1: Id, association_type: AssociationType, id2: Id){

    }
    pub fn association_delete(){

    }
    pub fn assocation_get(){

    }


    //helpers

    // Maybe this shouldn't be a simple unique integer, maybe some time of hash would solve our issue?
    // The paper suggests this should contain the "shard_id".
    fn allocate_unique_id() -> Id {
        0
    }

    fn reset(){

    }
}


struct Association {
    id: Id,
    // time: Time
}

struct AssociationList {
    buffer: Bytes
}

impl AssociationList {
    fn new(buffer :Bytes) -> AssociationList {
        AssociationList {
            buffer
        }
    }

    fn size(&self) -> u64 {
        // buffer.size()/std::mem::size_of::<Association>()
        // self.buffer.bytes()
        0
    }

    fn assocication_at() -> Association {
        Association {
            id: 1
        }
    }

    fn remove(id_2: Id) { // must be linear time
        // for pos in (0..self.size()) {
        //     // Find the assocation.
        //     if assocication_at(pos).id == id_2 { // must be constant time.
        //         // Shift everything down by one.
        //         for pos in (pos..size()-1) {
        //             buffer[pos] = buffer[pos+1];
        //         }
        //         // Remove extra space at end.
        //         buffer.release(pos, pos + std::mem::size_of::<Association>()) // release (start, end)
        //     }
        // }
    }

    fn add() {

    }

    fn dump(){}// ??
    fn filter(){}// ??
}






















// Multiple instances of a sinle SO *do* share state.
// Q: Does it interfere with other SOs? And/or with the hosting process?
// No: each SO is in it's own namespace.
static mut N: u32 = 0;

#[no_mangle]
pub fn init(db: &sandstorm::DB) {
  //let m;
  //unsafe {
  //  m = N;
  //  N +=1;
  //}
  //db.debug_log(&format!("TAO Initialized! {}", m));
  db.debug_log("TAO Initialized");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
