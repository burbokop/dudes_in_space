use crate::bl::Role;
use crate::bl::utils::math::Vector;

pub(crate) trait CaptainControlPanel {
    fn give_command(&self, role: Role) {}
}

pub(crate) trait NavigatorControlPanel {
    fn scan(&self) {}

    fn plan_route(&self) {}
}

pub(crate) trait GunnerControlPanel {
    fn scan(&self) -> Vector<u32> {
        todo!()
    }

    fn fire_at(&self, vessel_id: u32) {
        todo!()
    }
}

pub(crate) trait WorkerControlPanel {
    fn available_recipes(&self) -> Vector<u32> {
        todo!()
    }

    fn start_production(&self, ingredients: Vec<u32>, result: u32) {
        todo!()
    }
}
