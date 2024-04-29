pub mod query {
    use crate::msg::ValuerResp;

    pub fn value() -> ValuerResp {
        ValuerResp { value: 0 }
    }
}
