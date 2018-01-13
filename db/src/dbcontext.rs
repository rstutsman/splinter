use std::collections::HashMap;

pub struct dbcontext {
  catalog: HashMap<u64, Table>, 
}

impl dbcontext {
  pub fn new() -> dbcontext {
    let tbl1 = Table::new(1);
    let tbl2 = Table::new(2);
    let tbl3 = Table::new(3);
    let map = HashMap::new();

    map.insert(1, tbl1);
    map.insert(2, tbl2);
    map.insert(3, tbl3);

    dbcontext { catalog: map }
  }

  pub fn get(&self, id: u64) -> Option<&Table> {
    self.catalog.get(id)
  }
  
  pub fn get_mut(&self, id: u64) -> Option<&mut Table> {
    self.catalog.get_mut(id);
  }
}

