use crate::stage::logic::state::StageDataState;
use crate::settings::logic::state::ScannerConfig;
use super::scanner;

pub fn restart_scan(state: &mut StageDataState, config: ScannerConfig) {
    state.registry.clear_cache();
    state.scan_receiver = Some(scanner::start_scan(&config));
}

pub fn update_data(state: &mut StageDataState) {
    let Some(rx) = &state.scan_receiver else { return };

    if let Ok(new_registry) = rx.try_recv() {
        state.registry = new_registry;
        state.scan_receiver = None;
    }
}