//! rotation — 排轴模拟契约方法

use sr_api::{RotationRequest, RotationResult};

use crate::engine::rotation;

pub fn calculate_rotation(req: RotationRequest) -> Result<RotationResult, String> {
    rotation::simulate(&req)
}
