use crate::orga;
use std::borrow::Borrow;

use ibc::applications::transfer::MODULE_ID_STR;
use ibc::core::ics24_host::identifier::PortId;
use ibc::core::router::{Module, ModuleId, Router};

use super::transfer::Transfer;

#[orga]
pub struct IbcRouter {
    pub transfer: Transfer,
}

impl Router for IbcRouter {
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module> {
        (Borrow::<str>::borrow(module_id) == MODULE_ID_STR).then_some(&self.transfer as _)
    }

    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
        (Borrow::<str>::borrow(module_id) == MODULE_ID_STR).then_some(&mut self.transfer as _)
    }

    fn lookup_module(&self, port_id: &PortId) -> Option<ModuleId> {
        let transfer_port = PortId::transfer();
        let transfer_module_id: ModuleId = ModuleId::new(MODULE_ID_STR.to_string());

        if port_id == &transfer_port {
            Some(transfer_module_id)
        } else {
            None
        }
    }
}
