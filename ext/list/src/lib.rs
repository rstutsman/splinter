#![create_type = "dylib"]
#![feature(no_unsafe)]

extern crate db;
extern crate sandstorm;


struct ListNode {
  next_node_key: Option<u64>,
  data_len: u16,
  data: u8[65536]
}

struct ListExtension {

}

impl Sandstorm::Extension for ListExtension {
  #[no_mangle]
  pub init() {
    // Here is a great place to set up initial state for the extension,
    // but for now, this extension needs no initialization logic.
  }

  #[no_mangle]
  pub destroy() {
    // Here is a great place to tear down the extension's state,
    // but for now this extension needs no de-initialization logic.
  }
  
  #[no_mangle]
  pub call(
    db: &sandstorm::DBInterface,
    tbl_id: u64,
    keys: Vec<Vec<u8>>,
    arg: Vec<u8>) -> Sandstorm::ExtensionResult {

    if (keys.len() == 0) {
      let err = "Got an empty list of keys. This extension requires exactly 1 key.";
      Sandstorm::ExtensionResult::ERROR(String::from(err));
    } else {
      let result = db.get_key<u64, ListNode>(keys.get(0));

      match result {
        Sandstorm::CoreResult::SUCCESS(alpha_node) => {
          Sandstorm::ExtensionResult::SUCESSS(find_last_elem(alpha_node))
        },
        Sandstorm::CoreResult::TABLE_DOES_NOT_EXIST => {
          let err = format!("Table with id {} was not found.", tbl_id);
          Sandstorm::ExtensionResult::ERROR(String::from(err));
        },
        Sandstorm::CoreResult::KEY_DOES_NOT_EXIST => {
          let err = format!("Key {} was not found", keys.get(0));
          Sandstorm::ExtensionResult::ERROR(String::from(err));
        }
      }
    }
  }

}

fn find_last_elem(
  first_node: ListNode,
  db: &sandstorm::DBInterface) -> Sandstorm::CoreResult<u8[65536]> {
  
  let mut curr = first_node;
  while let Some(next) = curr.next_node_key {
    let result = db.get_key<u64, ListNode>(next);
    match result {
      Sandstorm::CoreResult::SUCCESS(next) => {
        curr = next;
      },
      _ => {

        // There was an error, we just hot potato the error up to callee
        return result
      }
    }
  }
  Sandstorm::CoreResult::SUCCESS(curr.data)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
