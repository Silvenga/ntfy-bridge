use crate::ntfy::NtfyClientShared;

#[derive(Clone)]
pub struct AppState {
    pub ntfy_client: NtfyClientShared,
}
