enum ExtensionResult {
  SUCCESS(Vec<u8>),
  /// TODO: might be worth getting an enum of error types, like if the error
  ///   is an exception in the sandstorm code or if the exception was raised in
  ///   the extension code itself.
  ERROR(String),
}

pub trait Extension {

  /// This method will be called once the extension has been loaded. Allows
  /// the extension to wire up an initial state needed for the lifetime of
  /// the extension.
  fn init();

  /// This method will be called once the extension has been scheduled to be
  /// killed. Allows the extension to clean up any state needed before the
  /// extension is unmounted from the system.
  fn destroy();

  /// This method will invoke the procedure associated with this extension.
  ///
  ///     - `db`: service interface for interacting with the sandstorm db
  ///     - `tbl_id`: id of the table for this invocation of the procedure
  ///     - `keys`: the input vector of keys for this invocation of the 
  ///         procedure
  ///     - `args`: a vector of bytes for additional arguments to this 
  ///         invocation of the procedure.
  fn call(db: DBInterface, tbl_id: u64, keys: Vec<Vec<u8>>, args: Vec<u8>) -> ExtensionResult;
}
