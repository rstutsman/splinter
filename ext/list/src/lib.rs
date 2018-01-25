#![create_type = "dylib"]
#![feature(no_unsafe)]

extern crate db;

#[no_mangle]
pub fn init(ctx: &db::dbcontext, tableId: u64, key: Vec<u8>) -> Result<Vec<u8>> {
  let tbl = ctx.get(tableId);
  let alpha = tbl.get(key);

  match alpha {
    Some(node) => {
      let alpha_node = ListNode::deserialize(node);
      Result(find_last_elem(alpha_node))  
    },
    None => {
      Result(Vec::new())
    }
  }
}

fn find_last_elem(first_node: ListNode, tbl: &db::Table) -> Vec<u8> {
  let mut curr = first_node;
  while let Some(next) = curr.next {
    let val = tbl.get(next);
    match val {
      Some(node) => {
        curr = ListNode::deserialize(node);
      },
      None => {
        // this is an exception:
        //    there was node whose next key was a key that didn't exist in the
        //    table. This is probably a result of a broken list, as the end of
        //    the list should be a key of zero.
        assert!(false);
      }
    }
  }
  curr.data
}

struct ListNode {
  next: Option<Vec<u8>>,
  data: Vec<u8>
}

/* List node structure in bytes:
 * [has_next, n_bytes, key..?, data..]
 */
impl ListNode {
  pub fn deserialize(bytes: Vec<u8>) -> ListNode {
    assert!(bytes.len() > 2); 
    // first byte indicates whether or not there is a next
    let has_next = bytes[0];
    let (next, data) = if (has_next > 0) {
      assert!(bytes.len() > 3);
      // if there exists a next element in the list, the second byte determins the length of the key
      // TODO: figure out a way to allow for larger keys than a max of 255 bytes.
      let n_bytes = bytes[1];
      assert!(bytes.len() > 0);
      
      // the next n_bytes are the key
      let key = bytes[2..n_bytes].to_vec();

      // the remaing bytes are the data
      let data = bytes[n_bytes..].to_vec();

      (Some(key), data)
    } else {
      (None, bytes[2..].to_vec())
    }

    ListNode { next, data }
  }
 
  pub fn serialize(&self) -> Vec<u8> {
    let mut serial = Vec::new();
    match self.next {
      Some(next) => {
        serial.push(1);
        serial.push(next.len());
        serial.append(self.next);
      },
      None => {
        serial.push(0); // does not have key
        serial.push(0); // key size is zero
      }
    }
    serial.append(self.data);
    serial
  }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
