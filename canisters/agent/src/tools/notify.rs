use crate::tools::result::ToolResult;
use ic_cdk::export::Principal;

const COST: u64 = 10_000_000;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Notification {
    pub message: String,
    pub recipient: Principal,
    pub timestamp: u64,
}

thread_local! {
    static OWNER_PRINCIPAL: std::cell::RefCell<Option<Principal>> =
        std::cell::RefCell::new(None);
}

pub fn set_owner_principal(principal: Principal) {
    OWNER_PRINCIPAL.with(|p| {
        *p.borrow_mut() = Some(principal);
    });
}

pub fn get_owner_principal() -> Option<Principal> {
    OWNER_PRINCIPAL.with(|p| p.borrow().clone())
}

pub fn send_notification(message: &str) -> ToolResult {
    let owner = get_owner_principal();

    let notification = Notification {
        message: message.to_string(),
        recipient: owner.unwrap_or_else(ic_cdk::api::caller),
        timestamp: ic_cdk::api::time(),
    };

    let _ = ic_cdk::notify(
        notification.recipient,
        "handle_notification",
        &(serde_json::to_vec(&notification).unwrap_or_default()),
    );

    ToolResult::ok("send_notification", "Notification sent", COST)
}
