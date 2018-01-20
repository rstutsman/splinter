#![create_type = "dylib"]
#![feature(no_unsafe)]

extern crate db;

#[no_mangle]
pub fn init(ctx: &db::dbcontext, key: common::BS) -> Result<BS> {
  let tbl = ctx.get(1); // TODO: pass table id as arg or something
  let alpha = tbl.get(key);
  

  // iterate through list
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
