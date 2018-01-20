use std::collections::HashMap;

pub struct dbcontext {
  catalog: HashMap<u64, Table>, 
}

impl dbcontext {
  pub fn new() -> dbcontext {
    let map = HashMap::new();
    dbcontext { catalog: map }
  }

  pub fn get(&self, id: u64) -> Option<&Table> {
    self.catalog.get(id)
  }
  
  pub fn get_mut(&self, id: u64) -> Option<&mut Table> {
    self.catalog.get_mut(id);
  }

  pub fn put_table(&self, tbl: Table) {
    self.catalog.insert(tbl.id, tbl);
  }
}

