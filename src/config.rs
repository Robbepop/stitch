// These limits mirror the WebAssembly JS-API implementation-defined limits
// (https://webassembly.github.io/spec/js-api/#limits).
pub(crate) const MAX_TYPE_COUNT: usize = 1_000_000;
pub(crate) const MAX_IMPORT_COUNT: usize = 100_000;
pub(crate) const MAX_FUNC_COUNT: usize = 1_000_000;
pub(crate) const MAX_FUNC_PARAM_COUNT: usize = 1_000;
pub(crate) const MAX_FUNC_RESULT_COUNT: usize = 1_000;
pub(crate) const MAX_TABLE_COUNT: usize = 100;
pub(crate) const MAX_MEMORY_COUNT: usize = 1;
pub(crate) const MAX_GLOBAL_COUNT: usize = 1_000_000;
pub(crate) const MAX_EXPORT_COUNT: usize = 100_000;
pub(crate) const MAX_ELEM_COUNT: usize = 100_000;
pub(crate) const MAX_ELEM_SIZE: usize = 1_000_000;
pub(crate) const MAX_FUNC_LOCAL_COUNT: usize = 50_000;
// The JS-API function-body size limit is a fixed spec constant (~7.3 MiB), not a round number.
pub(crate) const MAX_FUNC_BODY_SIZE: usize = 7_654_321;
pub(crate) const MAX_DATA_COUNT: usize = 100_000;
pub(crate) const MAX_DATA_SIZE: usize = 1_000_000;
