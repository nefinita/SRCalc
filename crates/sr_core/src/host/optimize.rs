//! optimize — 配装优化契约方法

use sr_api::{OptimizeRequest, OptimizeResult};

use crate::engine::optimize;

pub fn run_optimize(req: OptimizeRequest) -> Result<OptimizeResult, String> {
    optimize::run(&req)
}
